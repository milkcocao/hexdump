
#![allow(
    clippy::bool_to_int_with_if,
    clippy::diverging_sub_expression,
    clippy::extra_unused_type_parameters,
    clippy::if_same_then_else,
    clippy::ifs_same_cond,
    clippy::items_after_statements,
    clippy::let_and_return,
    clippy::let_underscore_untyped,
    clippy::match_bool,
    clippy::never_loop,
    clippy::overly_complex_bool_expr,
    clippy::redundant_closure_call,
    clippy::redundant_pattern_matching,
    clippy::too_many_lines,
    clippy::unit_arg,
    clippy::while_immutable_condition,
    clippy::zero_ptr,
    irrefutable_let_patterns
)]

use self::Enum::Generic;
use anyhow::{anyhow, ensure, Chain, Error, Result};
use std::fmt::{self, Debug};
use std::iter;
use std::marker::{PhantomData, PhantomData as P};
use std::mem;
use std::ops::Add;
use std::ptr;

struct S;

impl<T> Add<T> for S {
    type Output = bool;
    fn add(self, rhs: T) -> Self::Output {
        let _ = rhs;
        false
    }
}

trait Trait: Sized {
    const V: usize = 0;
    fn t(self, i: i32) -> i32 {
        i
    }
}

impl<T> Trait for T {}

enum Enum<T: ?Sized> {
    #[allow(dead_code)]
    Thing(PhantomData<T>),
    Generic,
}

impl<T: ?Sized> PartialEq for Enum<T> {
    fn eq(&self, rhs: &Self) -> bool {
        mem::discriminant(self) == mem::discriminant(rhs)
    }
}

impl<T: ?Sized> Debug for Enum<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("Generic")
    }
}

#[track_caller]
fn assert_err<T: Debug>(result: impl FnOnce() -> Result<T>, expected: &'static str) {
    let actual = result().unwrap_err().to_string();

    // In general different rustc versions will format the interpolated lhs and
    // rhs $:expr fragment with insignificant differences in whitespace or
    // punctuation, so we check the message in full against nightly and do just
    // a cursory test on older toolchains.
    if rustversion::cfg!(nightly) && !cfg!(miri) {
        assert_eq!(actual, expected);
    } else {
        assert_eq!(actual.contains(" vs "), expected.contains(" vs "));
    }
}

#[test]
fn test_recursion() {
    // Must not blow the default #[recursion_limit], which is 128.
    #[rustfmt::skip]
    let test = || Ok(ensure!(