use anyhow::{anyhow, Chain, Error};

fn error() -> Error {
    anyhow!({ 0 }).context(1).context(2).context(3)
}

#[test]
fn test_iter() {
    let e = error();
    let mut chain = e.chain();
    assert_eq!("3", chain.next().unwrap().to_string());
    assert_eq!("2", chain.next().unwrap().to_st