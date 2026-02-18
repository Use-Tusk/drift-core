# Tusk Drift Core

Shared Rust core and native language bindings for the Tusk Drift SDK suite.

This repository centralizes performance-sensitive logic used across Drift SDKs
(Python and Node), then exposes that logic through language-specific bindings.

## What this repo contains

- `crates/drift-rust-core`:
  shared Rust implementation of normalization, hashing, and protobuf export helpers
- `bindings/python`:
  Python extension module package published as `drift-core-python`
- `bindings/node`:
  Node native addon package published as `drift-core-node`
- `tests/`:
  cross-language parity fixtures and smoke tests

## Why this repo exists

The goal is to avoid re-implementing the same CPU-heavy logic in each SDK and
to reduce runtime overhead in hot export paths.

Key capabilities currently implemented in the Rust core include:

- deterministic JSON normalization and hashing
- object-to-protobuf `Struct` conversion helpers
- coalesced export payload processing helpers
- span protobuf byte construction
- export request protobuf byte construction

## Schema dependency

[`tusk-drift-schemas`](https://github.com/Use-Tusk/tusk-drift-schemas) is the protobuf source of truth.  [`drift-rust-core`](crates/drift-rust-core) consumes generated Rust types from the published [`tusk-drift-schemas`](https://crates.io/crates/tusk-drift-schemas) crate.

## Runtime expectations for SDK users

End users of Python/Node SDKs should not need a Rust toolchain during normal
installation.

- Python path: distribute platform wheels for `drift-core-python`
- Node path: distribute platform prebuilt native artifacts for `drift-core-node`
- Source builds remain fallback for unsupported targets
