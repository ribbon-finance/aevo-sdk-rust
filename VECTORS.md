# Signing Vectors

`tests/vectors/vectors.json` is copied from `ribbon-finance/exchange-backend`.

- Source repo: `ribbon-finance/exchange-backend`
- Source path: `scripts/sdk-vectors`
- Regeneration command in the backend repo: `go run ./scripts/sdk-vectors`
- Schema: `aevo-sdk-signing-vectors/v1`
- SHA-256: `d498458d502223fb3d299b18758992365b3791a4df9f3e16bb6a5f0e3beab6ef`

The Rust SDK treats the vectors as the shared signing contract. CI runs tests that require both `mainnet` and `testnet` domains to contain every expected EIP-712 vector kind:

- `order_plain`
- `order_builder`
- `register`
- `sign_key`
- `approve_builder`
- `withdraw`
- `transfer`

The guard also requires REST and websocket HMAC vectors. A truncated vectors file should fail before any release.
