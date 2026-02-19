use serde_json::Value as JsonValue;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

use crate::error::{CoreError, CoreResult};
use crate::normalize::{json_roundtrip_normalize, parse_json};

pub fn deterministic_hash(payload_json: &str) -> CoreResult<String> {
    let (_, hash) = normalize_and_hash(payload_json)?;
    Ok(hash)
}

pub fn normalize_and_hash(payload_json: &str) -> CoreResult<(String, String)> {
    let input = parse_json(payload_json)?;
    let normalized_value = json_roundtrip_normalize(&input)?;
    let normalized_json = serde_json::to_string(&normalized_value)
        .map_err(|e| CoreError::SerializationError(e.to_string()))?;
    let hash = hash_json_value_deterministic(&normalized_value)?;
    Ok((normalized_json, hash))
}

pub(crate) fn hash_json_value_deterministic(v: &JsonValue) -> CoreResult<String> {
    let sorted = sort_keys_recursively(v);
    let compact =
        serde_json::to_string(&sorted).map_err(|e| CoreError::SerializationError(e.to_string()))?;
    let mut hasher = Sha256::new();
    hasher.update(compact.as_bytes());
    Ok(format!("{:x}", hasher.finalize()))
}

fn sort_keys_recursively(v: &JsonValue) -> JsonValue {
    match v {
        JsonValue::Object(map) => {
            let mut sorted = BTreeMap::new();
            for (k, child) in map {
                sorted.insert(k.clone(), sort_keys_recursively(child));
            }
            let mut out = serde_json::Map::new();
            for (k, child) in sorted {
                out.insert(k, child);
            }
            JsonValue::Object(out)
        }
        JsonValue::Array(arr) => JsonValue::Array(arr.iter().map(sort_keys_recursively).collect()),
        _ => v.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_hash_is_stable_across_object_key_order() {
        let a = r#"{"z":1,"a":{"m":2,"b":[3,4]}}"#;
        let b = r#"{"a":{"b":[3,4],"m":2},"z":1}"#;

        let hash_a = deterministic_hash(a).expect("hash should succeed");
        let hash_b = deterministic_hash(b).expect("hash should succeed");

        assert_eq!(hash_a, hash_b);
    }

    #[test]
    fn normalize_and_hash_returns_compact_json_and_sha256_hex() {
        let input = "{ \"k\": \"v\", \"n\": 3 }";
        let (normalized, hash) = normalize_and_hash(input).expect("operation should succeed");

        assert_eq!(normalized, r#"{"k":"v","n":3}"#);
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn deterministic_hash_returns_invalid_json_error() {
        let err = deterministic_hash("nope").expect_err("invalid json should fail");
        assert!(matches!(err, CoreError::InvalidJson(_)));
    }
}
