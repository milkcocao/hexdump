use crate::StdError;
use core::fmt::{self, Debug, Display};

#[cfg(backtrace)]
use std::any::Demand;

#[repr(transparent)]
pub struct