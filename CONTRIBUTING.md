# Contributing

This document covers local development, testing, CI expectations, versioning,
and release workflow for `drift-core`.

## Prerequisites

- Rust toolchain from `rust-toolchain.toml` (`1.93.1`)
- Python 3.9+ (for Python binding workflows)
- Node 20+ and npm (for Node binding workflows)
- `uv` (used by parity smoke script for Python env setup)

## Repository structure

- `crates/drift-rust-core`: shared Rust core logic
- `bindings/python`: `pyo3` Python extension package (`drift-core-python`)
- `bindings/node`: `napi-rs` Node addon package (`drift-core-node`)
- `tests/fixtures`: shared parity fixtures
- `tests/parity`: cross-language smoke checks

## Local development

### Rust workspace checks

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo check --workspace
```

### Python binding local build

```bash
cd bindings/python
python -m pip install maturin
maturin develop --release
```

Python bindings are built with `pyo3` `abi3` (`abi3-py39`) so one wheel can be
reused across CPython 3.9+ on the same OS/arch.

If you hit future binding/runtime issues that seem Python-version-specific:

- first suspect whether the implementation started relying on APIs outside the
  stable `abi3` surface
- reproduce with a source build on the affected Python version
- if needed, temporarily disable `abi3` and publish per-version wheels while
  investigating

### Node binding local build

```bash
cd bindings/node
npm install
npm run build
```

## Parity verification

Use the smoke runner to validate Rust core behavior against Python/Node bindings:

```bash
bash tests/parity/run_smoke.sh
```

Current parity checks cover:

- `normalize_json`
- `deterministic_hash`
- `object_to_protobuf_struct_bytes`
- `object_to_protobuf_struct_field_count`

Implementation notes:

- Prefer semantic comparisons over raw encoded bytes where wire output can vary.
- For protobuf outputs, decode and compare structure whenever practical.

### [Future] property-based fuzz parity

Add a fuzz lane that generates nested/random JSON and verifies:

- Rust direct == Python binding == Node binding
- deterministic hash invariants hold for equivalent payloads

Recommended setup:

- small required seed set on normal CI
- larger nightly fuzz run for deeper coverage

## CI workflows

### `ci.yml`

Runs on PRs and pushes to `main`:

- Rust quality gates (`fmt`, `clippy`, `test`, `check`)
- parity smoke suite (`tests/parity/run_smoke.sh`)

### `publish-packages.yml`

Runs on GitHub Release publish (and manual dispatch):

- builds Python wheels (Linux/macOS/Windows) + sdist
- publishes Python package to PyPI (`drift-core-python`)
- builds Node native addons for release matrix
- publishes Node package to npm (`drift-core-node`)

Required secrets:

- `NPM_TOKEN`

PyPI publishing uses GitHub trusted publishing (OIDC), so no PyPI API token is
required. Ensure the PyPI project `drift-core-python` has a trusted publisher
configured for this repository/workflow.

## Versioning model

This repo currently uses lockstep versioning across:

- `Cargo.toml` (`[workspace.package].version`)
- `bindings/python/pyproject.toml` (`drift-core-python`)
- `bindings/node/package.json` (`drift-core-node`)

The Rust binding crate manifests use `version.workspace = true` and `publish = false`.

### Compatibility expectations

- breaking interface/protocol changes: major bump
- backward-compatible behavior/perf updates: minor/patch bump

## Release process

Use the release helper script:

```bash
./scripts/release.sh [patch|minor]
```

What it does:

1. Runs preflight checks (branch, clean tree, sync with remote, workspace checks)
2. Verifies lockstep version consistency
3. Bumps lockstep versions in:
   - `Cargo.toml`
   - `bindings/python/pyproject.toml`
   - `bindings/node/package.json` + lockfile
4. Commits version bump and creates/pushes tag
5. Optionally creates GitHub Release via `gh`

Publishing is then handled by GitHub Actions release workflows.

## Notes on binding strategy (UniFFI)

UniFFI remains a valid future option if we want an IDL-driven generated binding
layer. For now, we intentionally keep explicit `pyo3` + `napi-rs` wrappers for:

- tighter control over APIs and error mapping
- simpler debugging while interfaces are still evolving
