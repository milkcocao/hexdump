#![allow(clippy::module_name_repetitions)]

use std::error::Error as StdError;
use std::fmt::{self, Display};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Debug)]
pub struct Flag {
    atomic: Arc<AtomicBool>,
}

impl Flag {
    pub fn new() -> Self {
        Flag {
  