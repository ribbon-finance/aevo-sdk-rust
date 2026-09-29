# Contributing

Thanks for helping keep the Aevo Rust SDK boring in the best way: predictable, tested, and byte-for-byte compatible with the backend.

## Required Standard

Every change must follow this standard:

- Signing must reproduce the shared vectors in `tests/vectors/vectors.json` byte-for-byte.
- Add unit tests for every public method. Mock HTTP tests must cover method, path, query, body, auth headers, and error mapping.
- Money-unit helpers must stay 100% covered, including bad inputs, 6-decimal boundaries, huge values, and overflow returning an error instead of panicking.
- Do not use floats for money. Use strings, integers, or fixed-point helpers.
- Typed errors must carry the API `code` when the API returns one.
- Examples must dry-run by default.
- `SEND=1` examples must refuse the public test-vector keys.
- Do not commit secrets. The only test keys allowed in the repo are derived from public seeds documented in the vectors file.
- Keep `README.md` and `CHANGELOG.md` updated for user-visible behavior, release, or compatibility changes.

## Local Checks

Run the same commands as CI before opening a PR:

```sh
cargo fmt --check
cargo +1.88.0 check --locked --all-targets
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets
cargo build --locked --examples
cargo package --locked
RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps
```

Use Rust `1.88.0` for the MSRV check. Stable Rust is used for formatting, clippy, tests, examples, packaging, and docs.
