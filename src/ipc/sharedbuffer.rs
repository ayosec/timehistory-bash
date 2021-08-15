//! IPC between bash and children.

use std::cell::UnsafeCell;
use std::io;
use std::mem::{self, MaybeUninit};
use std::ptr;
use std::slice;
use std::time::Duration;

/// Minimum size for the shared buffer.
const MIN_BUFFER_SIZE: usize = 4 * 1024;

/// Buffer that can be shared between multiple processes.
pub struct SharedBuffer {
    buf: *mut libc::c_void,
    len: usize,
}

/// Data at the beginning of the buffer.
///
/// `repr(C)` is added to ensure that the layout is not affected
/// by the value of `N`.
#[repr(C)]
struct SharedBufferHeader<const N: usize> {
    mutex: UnsafeCell<libc::pthread_mutex_t>,
    cursor: usize,
    data: [u8; N],
}

// It is safe to implement both `Send` and `Sync` because `SharedBuffer`
// is equivalent to `Mutex<[u8; N]>`.

unsafe impl Send for SharedBuffer {}
unsafe impl Sync for SharedBuffer {}

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

        // Initialize a process-shared mutex.

        unsafe {
            macro_rules! check {
                ($e:expr) => {
                    match $e {
                        0 => (),

                        e => {
                            libc::munmap(buf, len);
                            return Err(io::Error::from_raw_os_error(e));
                        }
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

            // Data for the underlying buffer.
            header.cursor = 0;
        }

        Ok(SharedBuffer { buf, len })
    }

    /// Returns a raw pointer to the mutex in the shared buffer.
    unsafe fn mutex(&self) -> *mut libc::pthread_mutex_t {
        let header: &SharedBufferHeader<0> = &*self.buf.cast();
        header.mutex.get()
    }

    /// Acquires a lock to the data in the shared buffer.
    pub fn lock(&self, timeout: Duration) -> io::Result<SharedBufferGuard> {
        let abstime = compute_abstime(timeout);
        let res = unsafe { libc::pthread_mutex_timedlock(self.mutex(), &abstime) };

        if res != 0 {
            return Err(io::Error::from_raw_os_error(res));
        }

        Ok(SharedBufferGuard(self))
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

/// Compute a timeout based on the *realtime* clock.
fn compute_abstime(timeout: Duration) -> libc::timespec {
    const NS_PER_SEC: libc::c_long = 1_000_000_000;

    let mut ts = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };

    let r = unsafe { libc::clock_gettime(libc::CLOCK_REALTIME, &mut ts) };

    if r == 0 {
        ts.tv_sec += timeout.as_secs() as libc::time_t;
        ts.tv_nsec += timeout.subsec_nanos() as libc::c_long;

        if ts.tv_nsec > NS_PER_SEC {
            ts.tv_sec += 1;
            ts.tv_nsec -= NS_PER_SEC;
        }
    }

    ts
}

/// Access to the data in the shared buffer.
pub struct SharedBufferGuard<'a>(&'a SharedBuffer);

impl SharedBufferGuard<'_> {
    fn header(&self) -> &SharedBufferHeader<1> {
        unsafe { &*self.0.buf.cast() }
    }

    fn header_mut(&mut self) -> &mut SharedBufferHeader<1> {
        unsafe { &mut *self.0.buf.cast() }
    }

    fn data(&self) -> *const u8 {
        self.header().data.as_ptr()
    }

    fn data_mut(&mut self) -> *mut u8 {
        self.header_mut().data.as_mut_ptr()
    }

    fn capacity(&self) -> usize {
        self.0.len - mem::size_of::<SharedBufferHeader<0>>()
    }

    /// Discard data in the shared buffer, and reset the write cursor to `0`.
    pub fn clear(&mut self) {
        self.header_mut().cursor = 0;
    }

    /// Move the write cursor `n` bytes, usually called after updating the
    /// shared buffer with the slice from [`output`].
    ///
    /// If the new position exceeds the capacity, it returns `false`.
    pub fn advance(&mut self, n: usize) -> bool {
        let capacity = self.capacity();
        let header = self.header_mut();

        let cursor = header.cursor + n;
        if cursor > capacity {
            return false;
        }

        header.cursor = cursor;
        true
    }

    /// Returns a slice of the data written in the shared buffer.
    pub fn input(&self) -> &[u8] {
        let cursor = self.header().cursor;
        unsafe { slice::from_raw_parts(self.data(), cursor) }
    }

    /// Returns a mutable slice to write data in the shared buffer. [`advance`]
    /// is required to update the write cursor.
    pub fn output(&mut self) -> &mut [u8] {
        let header = self.header();
        let cursor = header.cursor;
        let len = self.capacity() - cursor;

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
    use std::sync::{Arc, Barrier};

    const EXPECTED_HEADER_SIZE: usize =
        mem::size_of::<libc::pthread_mutex_t>() + mem::size_of::<usize>();

    #[test]
    fn send_data() {
        let lock_timeout = Duration::from_secs(1);
        let buffer = SharedBuffer::new(MIN_BUFFER_SIZE).unwrap();

        let mut pids = [0; 4];
        for (idx, pid) in pids.iter_mut().enumerate() {
            *pid = unsafe { libc::fork() };
            assert!(*pid >= 0);

            if *pid == 0 {
                // Inside the forked process, loop until the buffer is filled.
                loop {
                    let mut lock = buffer.lock(lock_timeout).unwrap();
                    let data = lock.output();

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
        let mut lock = buffer.lock(lock_timeout).unwrap();
        assert_eq!(lock.output().len(), 0);

        let data = lock.input();
        assert_eq!(data.len(), MIN_BUFFER_SIZE - EXPECTED_HEADER_SIZE);

        for (a, b) in data.iter().zip("ABCD".chars().cycle()) {
            assert_eq!(*a as char, b);
        }

        lock.clear();
        assert_eq!(lock.input().len(), 0);
    }

    #[test]
    fn lock_timeouts() {
        let barrier = Arc::new(Barrier::new(2));
        let barrier2 = barrier.clone();

        let buffer = Arc::new(SharedBuffer::new(MIN_BUFFER_SIZE).unwrap());
        let buffer2 = buffer.clone();

        std::thread::spawn(move || {
            let lock: SharedBufferGuard = buffer2.lock(Duration::from_secs(1)).unwrap();
            barrier2.wait();
            std::mem::forget(lock);
        });

        barrier.wait();

        let start = std::time::Instant::now();
        let lock_res = buffer.lock(Duration::from_millis(20));
        assert!((20..120).contains(&start.elapsed().as_millis()));
        assert_eq!(lock_res.err().unwrap().kind(), std::io::ErrorKind::TimedOut);
    }
}
