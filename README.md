# aevo-sdk-rust

Official Rust SDK for [Aevo](https://aevo.xyz).

> **Status: pre-release, unpublished.** The crate name is `aevo-sdk`, version `0.1.0`. Not yet published to crates.io; see [RELEASING.md](RELEASING.md).

## Install

This repository is not published yet. Use it from a local checkout:

```toml
[dependencies]
aevo-sdk = { path = "../aevo-sdk-rust" }
```

## Quick Start

```rust
use aevo_sdk::{AevoClient, AuthMode, Env};
use aevo_sdk::models::MarketsRequest;

#[tokio::main]
async fn main() -> aevo_sdk::Result<()> {
    let client = AevoClient::builder()
        .env(Env::Testnet)
        .api_key(std::env::var("AEVO_API_KEY").unwrap_or_default())
        .api_secret(std::env::var("AEVO_API_SECRET").unwrap_or_default())
        .auth_mode(AuthMode::Hmac)
        .build()?;

    let markets = client
        .markets(MarketsRequest {
            asset: Some("ETH".into()),
            instrument_type: Some("PERPETUAL".into()),
        })
        .await?;

    println!("{markets:#}");
    Ok(())
}
```

## Signing

Signing is pure and public under `aevo_sdk::signing`. The test suite copies and verifies `tests/vectors/vectors.json`, generated from the exchange backend, and reproduces every hash and signature byte-for-byte.

```rust
use aevo_sdk::config::Env;
use aevo_sdk::signing::{self, OrderToSign};

let order = OrderToSign::builder()
    .maker("0x7D19833b5aF3b4e4D75DBA556ded46930469FA27")
    .is_buy(true)
    .limit_price("2700")?
    .amount("1")?
    .instrument("1")
    .builder_fields("builder_0123456789abcdef", "0.0003")
    .build()?;

let signed = signing::sign_order(Env::Testnet, "<signing private key>", order)?;
println!("{}", signed.hash);
println!("{}", serde_json::to_string_pretty(&signed.payload)?);
# Ok::<(), aevo_sdk::AevoError>(())
```

Helpers:

- `to_raw6("1.234567") -> "1234567"`
- `bps_to_rate(3) -> "0.0003"`
- `rate_to_raw("0.0003") -> "300"`

## REST

`AevoClient` maps directly to Aevo API endpoints. Public methods include `markets`, `instrument`, `orderbook`, `index`, `time`, `builder_config`, and `builder_profile`.

Authenticated methods include `account`, `portfolio`, `positions`, `orders`, `create_order`, `edit_order`, `cancel_order`, `cancel_all_orders`, `batch_create_orders`, `batch_cancel_orders`, `trade_history`, `register`, `withdraw`, and `transfer`.

Builder methods include `register_builder`, `update_builder_profile`, `approve_builder`, `revoke_builder`, `builder_approvals`, `builder_approval`, `builder_stats`, `builder_markets`, `builder_users`, `builder_fills`, and `download_builder_fills_csv`.

Authentication supports both backend modes:

- `AuthMode::Hmac`: `AEVO-KEY`, `AEVO-TIMESTAMP`, `AEVO-SIGNATURE`.
- `AuthMode::SecretHeader`: `AEVO-KEY`, `AEVO-SECRET`.

## WebSocket

The `ws` module provides frame builders plus `AevoWebSocket` for connecting, authenticating, subscribing, publishing order actions, pinging, streaming typed messages, and closing gracefully.

```rust
use aevo_sdk::ws::{AevoWebSocket, WsChannel};

let mut ws = AevoWebSocket::connect(&client).await?;
ws.auth("<api key>", "<api secret>").await?;
ws.subscribe(&[WsChannel::Ticker("ETH-PERP".into())]).await?;
# Ok::<(), aevo_sdk::AevoError>(())
```

Reconnect is intentionally left to callers for now; keep the subscription list and create a new `AevoWebSocket` when the stream closes.

## Builder Example

`examples/builder_e2e.rs` mirrors the builder flow in dry-run mode by default:

```sh
cargo run --example builder_e2e
SEND=1 cargo run --example builder_e2e
```

Set `AEVO_API_KEY`, `AEVO_API_SECRET`, `AEVO_WALLET_KEY`, `AEVO_SIGNING_KEY`, `AEVO_ACCOUNT`, `AEVO_BUILDER_ID`, and `AEVO_INSTRUMENT` as needed.

## Development

CI runs formatting, clippy, tests, examples, crate packaging, documentation with warnings denied, the MSRV check, and the signing vector guard. See [CONTRIBUTING.md](CONTRIBUTING.md) for the required SDK standard and [RELEASING.md](RELEASING.md) for the tag-and-approval release process.

## Versioning

This crate follows semver. While the crate is `0.x`, minor versions may include breaking changes and patch versions should remain backward-compatible.

## Sibling SDKs

| Language | Repository |
| --- | --- |
| Python | [aevoxyz/aevo-sdk](https://github.com/aevoxyz/aevo-sdk) |
| TypeScript | [ribbon-finance/aevo-sdk-ts](https://github.com/ribbon-finance/aevo-sdk-ts) |
| Rust | [ribbon-finance/aevo-sdk-rust](https://github.com/ribbon-finance/aevo-sdk-rust) (this repo) |

## Test

```sh
cargo fmt --check
cargo +1.88.0 check --locked --all-targets
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets
cargo build --locked --examples
cargo package --locked
RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps
```

## License

MIT. Copyright (c) 2026 Aevo. See [LICENSE](LICENSE).
