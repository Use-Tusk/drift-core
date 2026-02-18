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
