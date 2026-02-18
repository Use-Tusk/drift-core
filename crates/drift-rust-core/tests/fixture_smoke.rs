use std::fs;
use std::path::PathBuf;

use drift_rust_core::{
    deterministic_hash, normalize_and_hash, normalize_json, object_to_protobuf_struct_bytes,
    object_to_protobuf_struct_field_count,
};

fn fixture_path(name: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop(); // crates/
    p.pop(); // repo root
    p.push("tests");
    p.push("fixtures");
    p.push(name);
    p
}

#[test]
fn basic_fixture_smoke() {
    let raw = fs::read_to_string(fixture_path("basic.json")).expect("fixture should be readable");
    let fixture: serde_json::Value = serde_json::from_str(&raw).expect("fixture should be valid json");
    let input = fixture
        .get("input")
        .expect("fixture must include input")
        .to_string();

    let normalized = normalize_json(&input).expect("normalize_json should succeed");
    assert!(!normalized.is_empty(), "normalized output should not be empty");

    let hash = deterministic_hash(&input).expect("deterministic_hash should succeed");
    assert_eq!(hash.len(), 64, "sha256 hex digest must be 64 chars");
    let (normalized_via_combo, hash_via_combo) =
        normalize_and_hash(&input).expect("normalize_and_hash should succeed");
    assert_eq!(normalized_via_combo, normalized, "combined API should match normalize_json");
    assert_eq!(hash_via_combo, hash, "combined API should match deterministic_hash");

    let bytes = object_to_protobuf_struct_bytes(&input).expect("protobuf bytes should be produced");
    assert!(!bytes.is_empty(), "protobuf bytes should not be empty");

    let fields = object_to_protobuf_struct_field_count(&input).expect("field count should be produced");
    assert!(fields > 0, "top-level protobuf struct should contain fields");
}
