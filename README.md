# aevo-sdk-rust

Official Rust SDK for [Aevo](https://aevo.xyz).

> **Status: planning.** Nothing is published yet. This repository will hold the Rust SDK; the API below is the intended shape, not a released crate.

## Scope

- **REST client** for the public and authenticated Aevo API, with request/response types generated from the Aevo API specification.
- **Signing** (EIP-712): orders (including builder fields), builder approvals, registration, withdrawals and transfers.
- **Websocket client** for market data and private channels, built for low-latency trading.
- **Builder Codes**: register a builder, approve/revoke builders, attach a builder fee to orders, and pull builder reports (stats, markets, users, fills, CSV).
- **Typed errors**: API error codes (for example `BUILDER_FEE_EXCEEDS_USER_LIMIT`) as an error enum.

## Design

- **Thin, 1:1 with the API.** SDK methods map directly onto API endpoints; no SDK-only behaviour.
- **Explicit units.** Fee rates are decimal fractions (`0.0003` = 3 bps) with bps helpers; amounts are decimal strings, never floats; timestamps are nanoseconds where the API uses nanoseconds.
- **Shared test vectors.** Signing is verified in CI against a shared set of vectors (inputs and expected hashes and signatures) that the Python and TypeScript SDKs also reproduce, so all three sign identically.
- **Testnet first.** Every example runs against testnet by default.

## Sibling SDKs

| Language | Repository |
| --- | --- |
| Python | [aevoxyz/aevo-sdk](https://github.com/aevoxyz/aevo-sdk) |
| TypeScript | [ribbon-finance/aevo-sdk-ts](https://github.com/ribbon-finance/aevo-sdk-ts) |
| Rust | [ribbon-finance/aevo-sdk-rust](https://github.com/ribbon-finance/aevo-sdk-rust) (this repo) |

## Roadmap

1. REST types and client from the API specification
2. EIP-712 signing and shared test vectors
3. Orders, builder approvals and builder reporting
4. Websocket client
5. First crates.io release

## License

To be decided before the first release.
