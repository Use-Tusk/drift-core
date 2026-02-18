use serde_json::Value as JsonValue;

use crate::error::{CoreError, CoreResult};

pub fn normalize_json(payload_json: &str) -> CoreResult<String> {
    let input = parse_json(payload_json)?;
    let normalized = json_roundtrip_normalize(&input)?;
    serde_json::to_string(&normalized).map_err(|e| CoreError::SerializationError(e.to_string()))
}

pub(crate) fn parse_json(payload_json: &str) -> CoreResult<JsonValue> {
    serde_json::from_str(payload_json).map_err(|e| CoreError::InvalidJson(e.to_string()))
}

pub(crate) fn json_roundtrip_normalize(v: &JsonValue) -> CoreResult<JsonValue> {
    let s = serde_json::to_string(v).map_err(|e| CoreError::SerializationError(e.to_string()))?;
    serde_json::from_str(&s).map_err(|e| CoreError::SerializationError(e.to_string()))
}
