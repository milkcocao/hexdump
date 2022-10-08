
#![allow(
    // Clippy bug: https://github.com/rust-lang/rust-clippy/issues/7422
    clippy::nonstandard_macro_braces,
)]

mod drop;

use crate::drop::{DetectDrop, Flag};
use anyhow::{Context, Error, Result};
use std::fmt::{self, Display};
use thiserror::Error;

// https://github.com/dtolnay/anyhow/issues/18
#[test]
fn test_inference() -> Result<()> {
    let x = "1";
    let y: u32 = x.parse().context("...")?;
    assert_eq!(y, 1);
    Ok(())
}

macro_rules! context_type {
    ($name:ident) => {
        #[derive(Debug)]
        struct $name {
            message: &'static str,
            #[allow(dead_code)]
            drop: DetectDrop,
        }

        impl Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str(self.message)
            }
        }
    };
}

context_type!(HighLevel);
context_type!(MidLevel);

#[derive(Error, Debug)]
#[error("{message}")]
struct LowLevel {
    message: &'static str,
    drop: DetectDrop,
}

struct Dropped {
    low: Flag,
    mid: Flag,
    high: Flag,
}

impl Dropped {
    fn none(&self) -> bool {
        !self.low.get() && !self.mid.get() && !self.high.get()
    }

    fn all(&self) -> bool {
        self.low.get() && self.mid.get() && self.high.get()
    }
}

fn make_chain() -> (Error, Dropped) {
    let dropped = Dropped {
        low: Flag::new(),
        mid: Flag::new(),
        high: Flag::new(),
    };

    let low = LowLevel {
        message: "no such file or directory",
        drop: DetectDrop::new(&dropped.low),
    };

    // impl Context for Result<T, E>
    let mid = Err::<(), LowLevel>(low)
        .context(MidLevel {
            message: "failed to load config",
            drop: DetectDrop::new(&dropped.mid),
        })
        .unwrap_err();

    // impl Context for Result<T, Error>