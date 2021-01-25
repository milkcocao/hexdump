
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