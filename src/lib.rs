#![no_std]

extern crate libc;
#[cfg(test)]
extern crate std;

use core::ops::Deref;
use core::slice;
use core::str::{Utf8Error, self};
use libc::{c_char, c_void};

const DUMMY_PTR: *mut c_void = 0x1 as *mut c_void;

/// A type that represents a `malloc`'d chunk of memory.
pub struct Malloc<T: ?Sized> {
    ptr: *mut T,
}

impl<T: Copy> Malloc<[T]> {
    /// Constructs a new `Malloc` for a `malloc`'d buffer
    /// with the given length at the given pointer.
    /// Returns `None` if the given pointer is null and the length is not 0.
    ///
    /// When this `Malloc` drops, the buffer will be `free`'d.
    ///
    /// Unsafe because there must be `len` contiguous, valid instances of `T`
    /// at `ptr`.
    pub unsafe fn from_array(ptr: *mut T, len: usize) -> Option<Malloc<[T]>> {
        let ptr = match (ptr.is_null(), len) {
            // Even a 0-size slice cannot be null, so just use another pointer
            (true, 0) => DUMMY_PTR as *mut T,
            (true, _) => return None,
            (false, _) => ptr,
        };
        let slice = slice::from_raw_parts(ptr, len);
        Some(Malloc { ptr: slice as *const [T] as *mut [T] })
    }
}

impl Malloc<str> {
    pub unsafe fn from_c_str(ptr: *mut c_char)
            -> Result<Malloc<str>, Utf8Error> {
        let len = libc::strlen(ptr);
        let slice = slice::from_raw_parts(ptr as *mut u8, len);
        str::from_utf8(slice).map(|s| {
            Malloc { ptr: s as *const str as *mut str }
        })
    }
}

impl<T: ?Sized> Deref for Malloc<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.ptr }
    }
}

impl<T: ?Sized> Drop for Malloc<T> {
    fn drop(&mut self) {
        let ptr = self.ptr as *mut c_void;
        if ptr != DUMMY_PTR {
            unsafe {
                libc::free(ptr);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ptr;
    use libc::{c_char, self};

    use super::Malloc;

    #[test]
    fn test_null_buf() {
        let buf = unsafe {
            Malloc::<[u32]>::from_array(ptr::null_mut(), 0).unwrap()
        };
        assert!(&*buf == []);
        assert!(Some(&*buf) == Some(&[]));

        let buf = unsafe {
            Malloc::<[u32]>::from_array(ptr::null_mut(), 7)
        };
        assert!(buf.is_none());
    }

    #[test]
    fn test_buf() {
        let buf = unsafe {
            let ptr = libc::malloc(12) as *mut u32;
            *ptr = 1;
            *ptr.offset(1) = 2;
            *ptr.offset(2) = 3;
            Malloc::from_array(ptr, 3).unwrap()
        };
        assert!(&*buf == [1, 2, 3]);
    }

    #[test]
    fn test_string() {
        let s = unsafe {
            let ptr = libc::malloc(4) as *mut c_char;
            *ptr = 'h' as c_char;
            *ptr.offset(1) = 'e' as c_char;
            *ptr.offset(2) = 'y' as c_char;
            *ptr.offset(3) = '\0' as c_char;
            Malloc::from_c_str(ptr).unwrap()
        };
        assert!(&*s == "hey");
    }
}
