#![allow(
    // Clippy bug: https://github.com/rust-lang/rust-clippy/issues/7422
    clippy::nonstandard_macro_braces,
)]

use anyhow::anyhow;
use std::error::Error as StdError;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("outer")]
struct