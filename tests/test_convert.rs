#![allow(clippy::unnecessary_wraps)]

mod drop;

use self::drop::{DetectDrop, Flag};
use anyhow::{Error, Result};
use std::error::Error as StdError;

#[test]
fn test_convert() {
    let has_dropped = Flag::new();
    let error = Error::new(DetectDrop::new(&has_dropped));
    let box_dyn 