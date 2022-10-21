#![allow(clippy::unnecessary_wraps)]

mod drop;

use self::drop::{DetectDrop, Flag};
use anyhow::{Error, Result};
use std::error::Error as StdError;

#[test]
fn test_convert() {
    let has_dr