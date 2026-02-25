# Tusk Drift Core

[![CI](https://github.com/Use-Tusk/drift-core/actions/workflows/ci.yml/badge.svg)](https://github.com/Use-Tusk/drift-core/actions/workflows/ci.yml)
[![PyPI](https://img.shields.io/pypi/v/drift-core-python)](https://pypi.org/project/drift-core-python/)
[![npm](https://img.shields.io/npm/v/%40use-tusk%2Fdrift-core-node)](https://www.npmjs.com/package/@use-tusk/drift-core-node)

Shared Rust core and native language bindings for the Tusk Drift SDK suite.

This repository centralizes shared, performance-sensitive logic used by Drift
SDKs (Python and Node) and exposes it through native bindings.

## Repository layout

- `crates/drift-rust-core`: shared Rust implementation
- `bindings/python`: Python package (`drift-core-python`)
- `bindings/node`: Node package (`@use-tusk/drift-core-node`)
- `tests/`: cross-language parity fixtures and smoke checks

## Schema source of truth

[`tusk-drift-schemas`](https://github.com/Use-Tusk/tusk-drift-schemas) owns
the protobuf contracts. `drift-rust-core` consumes generated Rust types from
the published [`tusk-drift-schemas`](https://crates.io/crates/tusk-drift-schemas)
crate.

## Key docs

- [Architecture and design](docs/design.md)
- [Compatibility matrix](docs/compatibility-matrix.md)
- [Development and release workflow](CONTRIBUTING.md)

## Runtime expectations

SDK users should not need a Rust toolchain for normal installs:

- Python: prebuilt wheels for `drift-core-python`
- Node: prebuilt native artifacts for `@use-tusk/drift-core-node`
- Source builds are fallback for unsupported targets
