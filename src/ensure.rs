
use crate::Error;
use alloc::string::String;
use core::fmt::{self, Debug, Write};
use core::mem::MaybeUninit;
use core::ptr;
use core::slice;
use core::str;

#[doc(hidden)]
pub trait BothDebug {
    fn __dispatch_ensure(self, msg: &'static str) -> Error;
}

impl<A, B> BothDebug for (A, B)
where
    A: Debug,
    B: Debug,
{
    fn __dispatch_ensure(self, msg: &'static str) -> Error {
        render(msg, &self.0, &self.1)
    }
}

#[doc(hidden)]
pub trait NotBothDebug {
    fn __dispatch_ensure(self, msg: &'static str) -> Error;
}

impl<A, B> NotBothDebug for &(A, B) {
    fn __dispatch_ensure(self, msg: &'static str) -> Error {
        Error::msg(msg)
    }
}

struct Buf {
    bytes: [MaybeUninit<u8>; 40],
    written: usize,
}

impl Buf {
    fn new() -> Self {
        Buf {
            bytes: [MaybeUninit::uninit(); 40],
            written: 0,
        }
    }

    fn as_str(&self) -> &str {
        unsafe {
            str::from_utf8_unchecked(slice::from_raw_parts(
                self.bytes.as_ptr().cast::<u8>(),
                self.written,
            ))
        }
    }
}

impl Write for Buf {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if s.bytes().any(|b| b == b' ' || b == b'\n') {
            return Err(fmt::Error);
        }

        let remaining = self.bytes.len() - self.written;
        if s.len() > remaining {
            return Err(fmt::Error);
        }

        unsafe {
            ptr::copy_nonoverlapping(
                s.as_ptr(),
                self.bytes.as_mut_ptr().add(self.written).cast::<u8>(),
                s.len(),
            );
        }
        self.written += s.len();
        Ok(())
    }
}
