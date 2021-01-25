
#[cfg(backtrace)]
pub(crate) use std::backtrace::{Backtrace, BacktraceStatus};

#[cfg(all(not(backtrace), feature = "backtrace"))]
pub(crate) use self::capture::{Backtrace, BacktraceStatus};

#[cfg(not(any(backtrace, feature = "backtrace")))]
pub(crate) enum Backtrace {}

#[cfg(backtrace)]
macro_rules! impl_backtrace {
    () => {
        std::backtrace::Backtrace
    };
}

#[cfg(all(not(backtrace), feature = "backtrace"))]
macro_rules! impl_backtrace {
    () => {
        impl core::fmt::Debug + core::fmt::Display
    };
}

#[cfg(any(backtrace, feature = "backtrace"))]
macro_rules! backtrace {
    () => {
        Some(crate::backtrace::Backtrace::capture())
    };
}

#[cfg(not(any(backtrace, feature = "backtrace")))]
macro_rules! backtrace {
    () => {
        None
    };
}

#[cfg(backtrace)]
macro_rules! backtrace_if_absent {
    ($err:expr) => {
        match ($err as &dyn std::error::Error).request_ref::<std::backtrace::Backtrace>() {
            Some(_) => None,
            None => backtrace!(),
        }
    };
}

#[cfg(all(feature = "std", not(backtrace), feature = "backtrace"))]
macro_rules! backtrace_if_absent {
    ($err:expr) => {
        backtrace!()
    };
}

#[cfg(all(feature = "std", not(backtrace), not(feature = "backtrace")))]
macro_rules! backtrace_if_absent {
    ($err:expr) => {
        None
    };
}

#[cfg(all(not(backtrace), feature = "backtrace"))]
mod capture {
    use backtrace::{BacktraceFmt, BytesOrWideString, Frame, PrintFmt, SymbolName};
    use core::cell::UnsafeCell;
    use core::fmt::{self, Debug, Display};
    use core::sync::atomic::{AtomicUsize, Ordering};
    use std::borrow::Cow;
    use std::env;
    use std::path::{self, Path, PathBuf};
    use std::sync::Once;

    pub(crate) struct Backtrace {
        inner: Inner,
    }

    pub(crate) enum BacktraceStatus {
        Unsupported,
        Disabled,
        Captured,
    }

    enum Inner {
        Unsupported,
        Disabled,
        Captured(LazilyResolvedCapture),
    }

    struct Capture {
        actual_start: usize,
        resolved: bool,
        frames: Vec<BacktraceFrame>,
    }

    struct BacktraceFrame {
        frame: Frame,
        symbols: Vec<BacktraceSymbol>,
    }

    struct BacktraceSymbol {
        name: Option<Vec<u8>>,
        filename: Option<BytesOrWide>,
        lineno: Option<u32>,
        colno: Option<u32>,
    }

    enum BytesOrWide {
        Bytes(Vec<u8>),
        Wide(Vec<u16>),
    }

    impl Debug for Backtrace {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            let capture = match &self.inner {
                Inner::Unsupported => return fmt.write_str("<unsupported>"),
                Inner::Disabled => return fmt.write_str("<disabled>"),
                Inner::Captured(c) => c.force(),
            };

            let frames = &capture.frames[capture.actual_start..];

            write!(fmt, "Backtrace ")?;

            let mut dbg = fmt.debug_list();

            for frame in frames {
                if frame.frame.ip().is_null() {
                    continue;
                }

                dbg.entries(&frame.symbols);
            }

            dbg.finish()
        }
    }

    impl Debug for BacktraceFrame {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            let mut dbg = fmt.debug_list();
            dbg.entries(&self.symbols);
            dbg.finish()
        }
    }

    impl Debug for BacktraceSymbol {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            write!(fmt, "{{ ")?;

            if let Some(fn_name) = self.name.as_ref().map(|b| SymbolName::new(b)) {
                write!(fmt, "fn: \"{:#}\"", fn_name)?;
            } else {
                write!(fmt, "fn: <unknown>")?;
            }

            if let Some(fname) = self.filename.as_ref() {
                write!(fmt, ", file: \"{:?}\"", fname)?;
            }

            if let Some(line) = self.lineno {
                write!(fmt, ", line: {:?}", line)?;
            }

            write!(fmt, " }}")
        }
    }

    impl Debug for BytesOrWide {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            output_filename(
                fmt,
                match self {
                    BytesOrWide::Bytes(w) => BytesOrWideString::Bytes(w),
                    BytesOrWide::Wide(w) => BytesOrWideString::Wide(w),
                },
                PrintFmt::Short,
                env::current_dir().as_ref().ok(),
            )
        }