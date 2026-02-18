#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path


def main() -> int:
    repo_root = Path(__file__).resolve().parents[2]
    fixture_path = repo_root / "tests" / "fixtures" / "basic.json"
    fixture = json.loads(fixture_path.read_text(encoding="utf-8"))
    payload_json = json.dumps(fixture["input"], separators=(",", ":"))

    try:
        import drift_core as binding
    except Exception as exc:
        raise RuntimeError(
            "drift_core is not importable. "
            "Build/install the Python binding before running parity smoke tests."
        ) from exc

    normalized = binding.normalize_json(payload_json)
    digest = binding.deterministic_hash(payload_json)
    combined_normalized, combined_digest = binding.normalize_and_hash(payload_json)
    proto_bytes = binding.object_to_protobuf_struct_bytes(payload_json)
    field_count = binding.object_to_protobuf_struct_field_count(payload_json)

    assert isinstance(normalized, str) and len(normalized) > 0
    assert isinstance(digest, str) and len(digest) == 64
    assert combined_normalized == normalized
    assert combined_digest == digest
    assert isinstance(proto_bytes, (bytes, bytearray)) and len(proto_bytes) > 0
    assert isinstance(field_count, int) and field_count > 0

    print("OK: python binding smoke passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
