
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
        false | false | false | false | false | false | false | false | false |
        false | false | false | false | false | false | false | false | false |
        false | false | false | false | false | false | false | false | false |
        false | false | false | false | false | false | false | false | false |
        false | false | false | false | false | false | false | false | false |
        false | false | false | false | false | false | false | false | false |
        false | false | false | false | false | false | false | false | false
    ));

    test().unwrap_err();
}

#[test]
fn test_low_precedence_control_flow() {
    #[allow(unreachable_code)]
    let test = || {
        let val = loop {
            // Break has lower precedence than the comparison operators so the
            // expression here is `S + (break (1 == 1))`. It would be bad if the
            // ensure macro partitioned this input into `(S + break 1) == (1)`
            // because that means a different thing than what was written.
            ensure!(S + break 1 == 1);
        };
        Ok(val)
    };

    assert!(test().unwrap());
}

#[test]
fn test_low_precedence_binary_operator() {
    // Must not partition as `false == (true && false)`.
    let test = || Ok(ensure!(false == true && false));
    assert_err(test, "Condition failed: `false == true && false`");

    // But outside the root level, it is fine.
    let test = || Ok(ensure!(while false == true && false {} < ()));
    assert_err(
        test,
        "Condition failed: `while false == true && false {} < ()` (() vs ())",
    );
}

#[test]
fn test_closure() {
    // Must not partition as `(S + move) || (1 == 1)` by treating move as an
    // identifier, nor as `(S + move || 1) == (1)` by misinterpreting the
    // closure precedence.
    let test = || Ok(ensure!(S + move || 1 == 1));
    assert_err(test, "Condition failed: `S + (move || 1 == 1)`");

    let test = || Ok(ensure!(S + || 1 == 1));
    assert_err(test, "Condition failed: `S + (|| 1 == 1)`");

    // Must not partition as `S + ((move | ()) | 1) == 1` by treating those
    // pipes as bitwise-or.
    let test = || Ok(ensure!(S + move |()| 1 == 1));
    assert_err(test, "Condition failed: `S + (move |()| 1 == 1)`");

    let test = || Ok(ensure!(S + |()| 1 == 1));
    assert_err(test, "Condition failed: `S + (|()| 1 == 1)`");
}

#[test]
fn test_unary() {
    let mut x = &1;
    let test = || Ok(ensure!(*x == 2));
    assert_err(test, "Condition failed: `*x == 2` (1 vs 2)");

    let test = || Ok(ensure!(!x == 1));
    assert_err(test, "Condition failed: `!x == 1` (-2 vs 1)");

    let test = || Ok(ensure!(-x == 1));
    assert_err(test, "Condition failed: `-x == 1` (-1 vs 1)");

    let test = || Ok(ensure!(&x == &&2));
    assert_err(test, "Condition failed: `&x == &&2` (1 vs 2)");

    let test = || Ok(ensure!(&mut x == *&&mut &2));
    assert_err(test, "Condition failed: `&mut x == *&&mut &2` (1 vs 2)");
}

#[test]
fn test_if() {
    #[rustfmt::skip]
    let test = || Ok(ensure!(if false {}.t(1) == 2));
    assert_err(test, "Condition failed: `if false {}.t(1) == 2` (1 vs 2)");

    #[rustfmt::skip]
    let test = || Ok(ensure!(if false {} else {}.t(1) == 2));
    assert_err(
        test,
        "Condition failed: `if false {} else {}.t(1) == 2` (1 vs 2)",
    );

    #[rustfmt::skip]
    let test = || Ok(ensure!(if false {} else if false {}.t(1) == 2));
    assert_err(
        test,
        "Condition failed: `if false {} else if false {}.t(1) == 2` (1 vs 2)",
    );

    #[rustfmt::skip]
    let test = || Ok(ensure!(if let 1 = 2 {}.t(1) == 2));
    assert_err(
        test,
        "Condition failed: `if let 1 = 2 {}.t(1) == 2` (1 vs 2)",
    );

    #[rustfmt::skip]
    let test = || Ok(ensure!(if let 1 | 2 = 2 {}.t(1) == 2));
    assert_err(
        test,
        "Condition failed: `if let 1 | 2 = 2 {}.t(1) == 2` (1 vs 2)",
    );

    #[rustfmt::skip]
    let test = || Ok(ensure!(if let | 1 | 2 = 2 {}.t(1) == 2));
    assert_err(
        test,
        "Condition failed: `if let 1 | 2 = 2 {}.t(1) == 2` (1 vs 2)",
    );
}

#[test]
fn test_loop() {
    #[rustfmt::skip]
    let test = || Ok(ensure!(1 + loop { break 1 } == 1));
    assert_err(
        test,
        "Condition failed: `1 + loop { break 1 } == 1` (2 vs 1)",
    );

    #[rustfmt::skip]
    let test = || Ok(ensure!(1 + 'a: loop { break 'a 1 } == 1));
    assert_err(
        test,
        "Condition failed: `1 + 'a: loop { break 'a 1 } == 1` (2 vs 1)",
    );

    #[rustfmt::skip]
    let test = || Ok(ensure!(while false {}.t(1) == 2));
    assert_err(
        test,
        "Condition failed: `while false {}.t(1) == 2` (1 vs 2)",
    );

    #[rustfmt::skip]
    let test = || Ok(ensure!(while let None = Some(1) {}.t(1) == 2));
    assert_err(
        test,
        "Condition failed: `while let None = Some(1) {}.t(1) == 2` (1 vs 2)",
    );

    #[rustfmt::skip]
    let test = || Ok(ensure!(for _x in iter::once(0) {}.t(1) == 2));
    assert_err(
        test,
        "Condition failed: `for _x in iter::once(0) {}.t(1) == 2` (1 vs 2)",
    );

    #[rustfmt::skip]
    let test = || Ok(ensure!(for | _x in iter::once(0) {}.t(1) == 2));
    assert_err(
        test,
        "Condition failed: `for _x in iter::once(0) {}.t(1) == 2` (1 vs 2)",
    );

    #[rustfmt::skip]
    let test = || Ok(ensure!(for true | false in iter::empty() {}.t(1) == 2));
    assert_err(
        test,
        "Condition failed: `for true | false in iter::empty() {}.t(1) == 2` (1 vs 2)",
    );
}

#[test]
fn test_match() {
    #[rustfmt::skip]
    let test = || Ok(ensure!(match 1 == 1 { true => 1, false => 0 } == 2));
    assert_err(
        test,
        "Condition failed: `match 1 == 1 { true => 1, false => 0, } == 2` (1 vs 2)",
    );
}

#[test]
fn test_atom() {
    let test = || Ok(ensure!([false, false].len() > 3));
    assert_err(
        test,
        "Condition failed: `[false, false].len() > 3` (2 vs 3)",
    );

    #[rustfmt::skip]
    let test = || Ok(ensure!({ let x = 1; x } >= 3));
    assert_err(test, "Condition failed: `{ let x = 1; x } >= 3` (1 vs 3)");

    let test = || Ok(ensure!(S + async { 1 } == true));
    assert_err(
        test,
        "Condition failed: `S + async { 1 } == true` (false vs true)",
    );

    let test = || Ok(ensure!(S + async move { 1 } == true));
    assert_err(
        test,
        "Condition failed: `S + async move { 1 } == true` (false vs true)",
    );

    let x = &1;
    let test = || Ok(ensure!(S + unsafe { ptr::read(x) } == true));
    assert_err(
        test,
        "Condition failed: `S + unsafe { ptr::read(x) } == true` (false vs true)",
    );
}

#[test]
fn test_path() {
    let test = || Ok(ensure!(crate::S.t(1) == 2));
    assert_err(test, "Condition failed: `crate::S.t(1) == 2` (1 vs 2)");

    let test = || Ok(ensure!(::anyhow::Error::root_cause.t(1) == 2));
    assert_err(
        test,
        "Condition failed: `::anyhow::Error::root_cause.t(1) == 2` (1 vs 2)",
    );

    let test = || Ok(ensure!(Error::msg::<&str>.t(1) == 2));
    assert_err(
        test,
        "Condition failed: `Error::msg::<&str>.t(1) == 2` (1 vs 2)",
    );