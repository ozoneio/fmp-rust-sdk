# fmp-rust-sdk

An async Rust SDK for [Financial Modeling Prep](https://site.financialmodelingprep.com/developer/docs), using its current Stable API. Provides typed market and financial data, explicit subscription configuration, endpoint access probes, contextual errors, bounded retries, and optional stock/crypto/forex WebSocket connections.

```toml
[dependencies]
fmp-rust-sdk = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Enable `features = ["websocket"]` for streaming. See the [crate documentation](https://docs.rs/fmp-rust-sdk) and runnable [examples](https://github.com/ozoneio/fmp-rust-sdk/tree/main/examples) for usage, plan-identification limits, and explicit WebSocket authentication configuration.

For local use, copy `.env.example` to the git-ignored `.env` and set your credentials. `Client::from_dotenv` reads an explicit file without changing the process environment; `Client::from_env` reads existing environment variables. Run `cargo run --example quote` to use `.env`.

Maintainers can set `CARGO_REGISTRY_TOKEN` in `.env` and run `cargo run --example publish` after the release checks pass.

Requires Rust 1.88 or later. This is an independent SDK; it is not affiliated with FMP.
