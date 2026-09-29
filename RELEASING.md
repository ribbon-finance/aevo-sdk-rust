# Releasing

The crate is MIT-licensed (Copyright (c) 2026 Aevo); the release workflow refuses to publish if the license is ever unset, `TBD` or `UNLICENSED`.

## One-Time Setup

- Create the GitHub Actions `release` environment and require reviewer approval.
- Add `CARGO_REGISTRY_TOKEN` as a repository or environment secret for crates.io publishing.
- Confirm crates.io ownership for `aevo-sdk`.
- For the TypeScript sibling repo, confirm npm org ownership for `@aevo` and add `NPM_TOKEN`.

## Release Steps

1. Bump `version` in `Cargo.toml`.
2. Move the matching `CHANGELOG.md` section from `unreleased` to the release date and keep `## [Unreleased]` at the top.
3. Confirm `license = "MIT"` is still set in `Cargo.toml`.
4. Open a PR and wait for CI to pass.
5. Merge to `main`.
6. Create and push the tag:

```sh
git tag vX.Y.Z
git push origin vX.Y.Z
```

7. Approve the `release` environment in GitHub Actions.
8. Confirm the crate appears on crates.io and the GitHub Release notes match the changelog section.

## Semver

This project follows semver. While the crate is `0.x`, minor versions may include breaking changes and patch versions should remain backward-compatible bug fixes or documentation updates.

## Release Workflow Guarantees

The tag must match `Cargo.toml` exactly, such as `v0.1.0` for version `0.1.0`. The changelog must contain a section for that version. The workflow reruns the full CI command set before publishing, then publishes only after the protected `release` environment is approved.

Publishing fails with a clear error if `Cargo.toml` has no real `license` or `license-file`, or if the license is still `TBD` or `UNLICENSED`.
