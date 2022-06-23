
/// Return early with an error.
///
/// This macro is equivalent to `return Err(`[`anyhow!($args...)`][anyhow!]`)`.
///
/// The surrounding function's or closure's return value is required to be
/// `Result<_,`[`anyhow::Error`][crate::Error]`>`.
///
/// [anyhow!]: crate::anyhow
///
/// # Example
///
/// ```
/// # use anyhow::{bail, Result};
/// #
/// # fn has_permission(user: usize, resource: usize) -> bool {
/// #     true
/// # }
/// #
/// # fn main() -> Result<()> {
/// #     let user = 0;
/// #     let resource = 0;
/// #
/// if !has_permission(user, resource) {
///     bail!("permission denied for accessing {}", resource);
/// }
/// #     Ok(())
/// # }
/// ```
///
/// ```
/// # use anyhow::{bail, Result};
/// # use thiserror::Error;
/// #
/// # const MAX_DEPTH: usize = 1;
/// #
/// #[derive(Error, Debug)]
/// enum ScienceError {
///     #[error("recursion limit exceeded")]
///     RecursionLimitExceeded,
///     # #[error("...")]
///     # More = (stringify! {
///     ...
///     # }, 1).1,
/// }
///
/// # fn main() -> Result<()> {
/// #     let depth = 0;
/// #
/// if depth > MAX_DEPTH {
///     bail!(ScienceError::RecursionLimitExceeded);
/// }
/// #     Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! bail {
    ($msg:literal $(,)?) => {
        return $crate::__private::Err($crate::__anyhow!($msg))
    };
    ($err:expr $(,)?) => {
        return $crate::__private::Err($crate::__anyhow!($err))
    };
    ($fmt:expr, $($arg:tt)*) => {
        return $crate::__private::Err($crate::__anyhow!($fmt, $($arg)*))
    };
}

/// Return early with an error if a condition is not satisfied.
///
/// This macro is equivalent to `if !$cond { return
/// Err(`[`anyhow!($args...)`][anyhow!]`); }`.
///
/// The surrounding function's or closure's return value is required to be
/// `Result<_,`[`anyhow::Error`][crate::Error]`>`.
///
/// Analogously to `assert!`, `ensure!` takes a condition and exits the function
/// if the condition fails. Unlike `assert!`, `ensure!` returns an `Error`
/// rather than panicking.
///
/// [anyhow!]: crate::anyhow
///
/// # Example
///
/// ```
/// # use anyhow::{ensure, Result};
/// #
/// # fn main() -> Result<()> {
/// #     let user = 0;
/// #
/// ensure!(user == 0, "only user 0 is allowed");
/// #     Ok(())
/// # }
/// ```
///
/// ```
/// # use anyhow::{ensure, Result};
/// # use thiserror::Error;
/// #
/// # const MAX_DEPTH: usize = 1;
/// #
/// #[derive(Error, Debug)]
/// enum ScienceError {
///     #[error("recursion limit exceeded")]
///     RecursionLimitExceeded,
///     # #[error("...")]
///     # More = (stringify! {
///     ...
///     # }, 1).1,
/// }
///
/// # fn main() -> Result<()> {
/// #     let depth = 0;
/// #
/// ensure!(depth <= MAX_DEPTH, ScienceError::RecursionLimitExceeded);
/// #     Ok(())
/// # }
/// ```
#[cfg(doc)]
#[macro_export]
macro_rules! ensure {
    ($cond:expr $(,)?) => {
        if !$cond {
            return $crate::__private::Err($crate::Error::msg(
                $crate::__private::concat!("Condition failed: `", $crate::__private::stringify!($cond), "`")
            ));
        }
    };
    ($cond:expr, $msg:literal $(,)?) => {
        if !$cond {
            return $crate::__private::Err($crate::__anyhow!($msg));
        }
    };
    ($cond:expr, $err:expr $(,)?) => {
        if !$cond {
            return $crate::__private::Err($crate::__anyhow!($err));
        }
    };
    ($cond:expr, $fmt:expr, $($arg:tt)*) => {
        if !$cond {
            return $crate::__private::Err($crate::__anyhow!($fmt, $($arg)*));
        }
    };
}

#[cfg(not(doc))]
#[macro_export]
macro_rules! ensure {
    ($($tt:tt)*) => {
        $crate::__parse_ensure!(
            /* state */ 0
            /* stack */ ()
            /* bail */ ($($tt)*)
            /* fuel */ (~~~~~~~~~~ ~~~~~~~~~~ ~~~~~~~~~~ ~~~~~~~~~~ ~~~~~~~~~~ ~~~~~~~~~~ ~~~~~~~~~~ ~~~~~~~~~~ ~~~~~~~~~~ ~~~~~~~~~~ ~~~~~~~~~~ ~~~~~~~~~~)
            /* parse */ {()}
            /* dup */ ($($tt)*)
            /* rest */ $($tt)*
        )
    };
}

/// Construct an ad-hoc error from a string or existing non-`anyhow` error
/// value.
///
/// This evaluates to an [`Error`][crate::Error]. It can take either just a
/// string, or a format string with arguments. It also can take any custom type
/// which implements `Debug` and `Display`.
///
/// If called with a single argument whose type implements `std::error::Error`
/// (in addition to `Debug` and `Display`, which are always required), then that
/// Error impl's `source` is preserved as the `source` of the resulting
/// `anyhow::Error`.
///
/// # Example
///
/// ```
/// # type V = ();
/// #