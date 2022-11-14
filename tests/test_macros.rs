#![allow(
    clippy::assertions_on_result_states,
    clippy::eq_op,
    clippy::items_after_statements,
    clippy::needless_pass_by_value,
    clippy::shadow_unrelated,
    clippy::wildcard_imports
)]

mod common;

use self::common::*;
use anyhow::{anyhow, ensure};
use std::cell::Cell;
use std::future;

#[test]
fn test_messages() {
    assert_eq!("oh no!", bail_literal().unwrap_err().to_string());
    assert_eq!("oh no!", bail_fmt().u