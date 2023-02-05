use anyhow::{ensure, Result};

fn main() -> Result<()> {
    // `ensure!` must not partition this in