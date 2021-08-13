//! IPC between bash and children.

use std::cell::UnsafeCell;
use std::io;
use std::marker::PhantomData;
use std::mem::{self, MaybeUninit};
use std::ptr;
use std::slice;

/// Minimum size for the shared buffer.
const MIN_BUFFER_SIZE: usize = 4 * 1024;

pub struct SharedBuffer {
    buf: *mut libc::c_void,
    len: usize,
}

/// Data at the beginning of the buffer
struct SharedBufferHeader<const N: usize> {
    mutex: UnsafeCell<libc::pthread_mutex_t>,
    capacity: usize,
    cursor: usize,
    data: [u8; N],
}

impl SharedBuffer {
    pub fn new(len: usize) -> io::Result<Self> {
        if len < MIN_BUFFER_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "buffer length too small",
            ));
        }

        // Allocate memory.

        let buf = unsafe {
            libc::mmap(
                ptr::null_mut(),
                len,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED | libc::MAP_ANONYMOUS,
                -1,
                0,
            )
        };

        if buf == libc::MAP_FAILED {
            return Err(io::Error::last_os_error());
        }

        // Initialize the mutex between processes.

        unsafe {
            macro_rules! check {
                ($e:expr) => {
                    if $e != 0 {
                        let err = Err(io::Error::last_os_error());
                        libc::munmap(buf, len);
                        return err;
                    }
                };
            }

            let header: &mut SharedBufferHeader<0> = &mut *buf.cast();

            let mut attr = MaybeUninit::zeroed();

            check!(libc::pthread_mutexattr_init(attr.as_mut_ptr()));

            check!(libc::pthread_mutexattr_settype(
                attr.as_mut_ptr(),
                libc::PTHREAD_MUTEX_NORMAL
            ));

            check!(libc::pthread_mutexattr_setpshared(
                attr.as_mut_ptr(),
                libc::PTHREAD_PROCESS_SHARED
            ));

            header.mutex = UnsafeCell::new(libc::PTHREAD_MUTEX_INITIALIZER);
            check!(libc::pthread_mutex_init(header.mutex.get(), attr.as_ptr()));

            // Fields to track written bytes.
            header.cursor = 0;
            header.capacity = len - mem::size_of::<SharedBufferHeader<0>>();
        }

        Ok(SharedBuffer { buf, len })
    }

    /// Returns a raw pointer to the mutex in the shared buffer.
    unsafe fn mutex(&self) -> *mut libc::pthread_mutex_t {
        let header: &SharedBufferHeader<0> = &*self.buf.cast();
        header.mutex.get()
    }

    /// Acquires a lock to data in the shared buffer.
    pub fn lock(&self) -> io::Result<SharedBufferGuard> {
        if unsafe { libc::pthread_mutex_lock(self.mutex()) } != 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(SharedBufferGuard {
            buf: self.buf,
            shared_buffer: PhantomData,
        })
    }
}

impl Drop for SharedBuffer {
    fn drop(&mut self) {
        unsafe {
            libc::pthread_mutex_destroy(self.mutex());
            libc::munmap(self.buf, self.len);
        }
    }
}

/// Guard for the shared buffer.
pub struct SharedBufferGuard<'a> {
    buf: *mut libc::c_void,
    shared_buffer: PhantomData<&'a SharedBuffer>,
}

impl SharedBufferGuard<'_> {
    fn header(&self) -> &SharedBufferHeader<1> {
        unsafe { &*self.buf.cast() }
    }

    fn header_mut(&mut self) -> &mut SharedBufferHeader<1> {
        unsafe { &mut *self.buf.cast() }
    }

    fn data(&self) -> *const u8 {
        self.header().data.as_ptr()
    }

    fn data_mut(&mut self) -> *mut u8 {
        self.header_mut().data.as_mut_ptr()
    }

    /// Discard data in the shared buffer, and reset the write cursor to `0`.
    pub fn clear(&mut self) {
        self.header_mut().cursor = 0;
    }

    /// Move the write cursor `n` bytes, usually called after updating the
    /// shared buffer with the slice from [`as_output`].
    ///
    /// If the new position exceeds the capacity, it returns `false`.
    pub fn advance(&mut self, n: usize) -> bool {
        let header = self.header_mut();
        let cursor = header.cursor + n;
        if cursor > header.capacity {
            return false;
        }

        header.cursor = cursor;
        true
    }

    /// Returns a slice of the data written in the shared buffer.
    pub fn as_input(&self) -> &[u8] {
        let cursor = self.header().cursor;
        unsafe { slice::from_raw_parts(self.data(), cursor) }
    }

    /// Returns a mutable slice to write data in the shared buffer. [`advance`]
    /// is required to update the write cursor.
    pub fn as_output(&mut self) -> &mut [u8] {
        let header = self.header();
        let cursor = header.cursor;
        let capacity = header.capacity;
        let len = capacity - cursor;

        unsafe { slice::from_raw_parts_mut(self.data_mut().add(cursor), len) }
    }
}

impl Drop for SharedBufferGuard<'_> {
    fn drop(&mut self) {
        unsafe {
            libc::pthread_mutex_unlock(self.header().mutex.get());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    #[test]
    fn send_data() {
        let buffer = SharedBuffer::new(MIN_BUFFER_SIZE).unwrap();

        let mut pids = [0; 4];
        for (idx, pid) in pids.iter_mut().enumerate() {
            *pid = unsafe { libc::fork() };
            assert!(*pid >= 0);

            if *pid == 0 {
                // Inside the forked process, loop until the buffer is filled.
                loop {
                    let mut lock = buffer.lock().unwrap();
                    let data = lock.as_output();

                    if data.is_empty() {
                        drop(lock);
                        unsafe { libc::exit(0) };
                    }

                    if (MIN_BUFFER_SIZE - data.len()) % pids.len() == idx {
                        data[0] = idx as u8 + b'A';
                        lock.advance(1);
                    }

                    drop(lock);
                    std::thread::yield_now();
                }
            }
        }

        // Wait for the children.
        for pid in pids {
            assert_eq!(unsafe { libc::waitpid(pid, ptr::null_mut(), 0) }, pid);
        }

        // Check written data.
        let mut lock = buffer.lock().unwrap();
        assert_eq!(lock.as_output().len(), 0);

        let data = lock.as_input();
        assert_eq!(
            data.len(),
            MIN_BUFFER_SIZE - mem::size_of::<SharedBufferHeader<0>>()
        );

        for (a, b) in data.iter().zip("ABCD".chars().cycle()) {
            assert_eq!(*a as char, b);
        }

        lock.clear();
        assert_eq!(lock.as_input().len(), 0);
    }
}
