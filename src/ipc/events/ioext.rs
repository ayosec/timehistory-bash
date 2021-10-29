//! Extensions for the `Read`/`Write` traits.

use std::ffi::OsString;
use std::io::{self, Read, Write};
use std::mem::{self, MaybeUninit};
use std::os::unix::ffi::OsStringExt;
use std::{ptr, slice};

pub(super) trait ReadExt {
    /// Read any `Copy` value.
    ///
    /// This function is unsafe because the caller has to ensure that `T`
    /// does not contain any reference.
    unsafe fn read_value<T: Copy + 'static>(&mut self) -> io::Result<T>;

    /// Extract a C string, as written by `WriteExt::write_cstr`.
    fn read_cstr(&mut self) -> io::Result<OsString>;
}

pub(super) trait WriteExt {
    /// Write any `Copy` value.
    fn write_value<T: Copy + 'static>(&mut self, value: &T) -> io::Result<()>;

    /// Write a C string to `output`.
    ///
    /// The size is written as a `usize` before the string, and it is limited to
    /// `limit`.
    ///
    /// Returns how many bytes are written.
    unsafe fn write_cstr(&mut self, ptr: *const libc::c_char, limit: usize) -> io::Result<usize>;
}

impl<R: Read> ReadExt for R {
    unsafe fn read_value<T: Copy + 'static>(&mut self) -> io::Result<T> {
        let mut data = MaybeUninit::<T>::uninit();
        let buf = slice::from_raw_parts_mut(data.as_mut_ptr().cast(), mem::size_of::<T>());
        self.read_exact(buf)?;
        Ok(ptr::read(data.as_ptr()))
    }

    fn read_cstr(&mut self) -> io::Result<OsString> {
        let size = unsafe { self.read_value::<usize>()? };
        let mut bytes = vec![0; size];
        self.read_exact(&mut bytes)?;
        Ok(OsString::from_vec(bytes))
    }
}

impl<W: Write> WriteExt for W {
    fn write_value<T: Copy + 'static>(&mut self, value: &T) -> io::Result<()> {
        let slice =
            unsafe { slice::from_raw_parts(value as *const T as *const u8, mem::size_of::<T>()) };
        self.write_all(slice)
    }

    unsafe fn write_cstr(&mut self, ptr: *const libc::c_char, limit: usize) -> io::Result<usize> {
        // String size.
        let size = libc::strnlen(ptr.cast(), limit);

        self.write_value(&size)?;

        // String bytes.
        let slice = std::slice::from_raw_parts(ptr.cast(), size);
        self.write_all(slice)?;

        Ok(size)
    }
}

#[test]
fn read_and_write_primitives() {
    #[derive(Copy, Clone, Debug, PartialEq)]
    struct SomeData {
        a: u32,
        b: u32,
    }

    let mut data: Vec<u8> = vec![];

    let mut output = io::Cursor::new(&mut data);
    output.write_value(&10_u16).unwrap();
    output.write_value(&SomeData { a: 1, b: 2 }).unwrap();
    drop(output);

    let mut input = io::Cursor::new(&data);
    assert_eq!(unsafe { input.read_value::<u16>().unwrap() }, 10_u16);
    assert_eq!(
        unsafe { input.read_value::<SomeData>().unwrap() },
        SomeData { a: 1, b: 2 }
    );
}
