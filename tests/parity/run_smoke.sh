#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PY_VENV="$ROOT_DIR/.venv"

echo "Running Rust core fixture smoke..."
cargo test -p drift-rust-core --test fixture_smoke

echo "Building Python binding with uv + maturin..."
command -v uv >/dev/null 2>&1 || {
  echo "ERROR: uv is required to run Python parity smoke."
  exit 1
}
if [ ! -x "$PY_VENV/bin/python" ]; then
  uv venv "$PY_VENV"
fi
uv pip install --python "$PY_VENV/bin/python" maturin
(
  unset CONDA_PREFIX
  "$PY_VENV/bin/maturin" develop --release --manifest-path "$ROOT_DIR/bindings/python/Cargo.toml"
)

echo "Running Python binding smoke..."
"$PY_VENV/bin/python" "$ROOT_DIR/tests/parity/python_binding_smoke.py"

echo "Building Node binding..."
command -v node >/dev/null 2>&1 || {
  echo "ERROR: node is required to run Node parity smoke."
  exit 1
}
command -v npm >/dev/null 2>&1 || {
  echo "ERROR: npm is required to run Node parity smoke."
  exit 1
}
(
  cd "$ROOT_DIR/bindings/node"
  npm install
  npm run build
)

echo "Running Node binding smoke..."
node "$ROOT_DIR/tests/parity/node_binding_smoke.mjs"

echo "Done."
