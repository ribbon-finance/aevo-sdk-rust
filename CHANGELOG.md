# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project follows semver. While the crate is `0.x`, minor versions may include breaking changes.

## [Unreleased]

### Added

- Added external signer support for builder approvals, including EIP-712 typed-data/digest helpers, `DigestSigner`, `LocalKeySigner`, optional-auth submission, and wallet-only `/builder/approve` requests with `account`.

## [0.1.0] - unreleased

### Added

- Initial Rust SDK for Aevo REST APIs, including public, authenticated account/order, transfer/withdraw, and builder endpoints.
- Deterministic signing helpers for orders, registration, sign-key, builder approval, withdraw, transfer, REST HMAC, and websocket HMAC flows.
- WebSocket frame helpers and typed message parsing for auth, subscribe/unsubscribe, and order actions.
- Builder-code support for attributed orders and builder approval flows.
- Shared signing vector coverage generated from `ribbon-finance/exchange-backend` `scripts/sdk-vectors`.
