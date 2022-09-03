use anyhow::{anyhow, Chain, Error};

fn error() -> Error {
    anyhow!({ 0 }).context(