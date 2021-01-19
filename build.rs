
#![allow(clippy::option_if_let_else)]

use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::str;

#[cfg(all(feature = "backtrace", not(feature = "std")))]
compile_error! {
    "`backtrace` feature without `std` feature is not supported"
}

// This code exercises the surface area that we expect of the std Backtrace
// type. If the current toolchain is able to compile it, we go ahead and use
// backtrace in anyhow.
const PROBE: &str = r#"
    #![feature(error_generic_member_access, provide_any)]

    use std::any::{Demand, Provider};
    use std::backtrace::{Backtrace, BacktraceStatus};
    use std::error::Error;
    use std::fmt::{self, Display};

    #[derive(Debug)]
    struct E {
        backtrace: Backtrace,
    }

    impl Display for E {