
Anyhow&ensp;¯\\\_(°ペ)\_/¯
==========================

[<img alt="github" src="https://img.shields.io/badge/github-dtolnay/anyhow-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/dtolnay/anyhow)
[<img alt="crates.io" src="https://img.shields.io/crates/v/anyhow.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/anyhow)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-anyhow-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/anyhow)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/dtolnay/anyhow/ci.yml?branch=master&style=for-the-badge" height="20">](https://github.com/dtolnay/anyhow/actions?query=branch%3Amaster)

This library provides [`anyhow::Error`][Error], a trait object based error type
for easy idiomatic error handling in Rust applications.

[Error]: https://docs.rs/anyhow/1.0/anyhow/struct.Error.html

```toml
[dependencies]
anyhow = "1.0"
```

*Compiler support: requires rustc 1.39+*

<br>

## Details

- Use `Result<T, anyhow::Error>`, or equivalently `anyhow::Result<T>`, as the
  return type of any fallible function.

  Within the function, use `?` to easily propagate any error that implements the
  `std::error::Error` trait.

  ```rust
  use anyhow::Result;

  fn get_cluster_info() -> Result<ClusterMap> {
      let config = std::fs::read_to_string("cluster.json")?;
      let map: ClusterMap = serde_json::from_str(&config)?;
      Ok(map)
  }
  ```

- Attach context to help the person troubleshooting the error understand where
  things went wrong. A low-level error like "No such file or directory" can be
  annoying to debug without more context about what higher level step the
  application was in the middle of.