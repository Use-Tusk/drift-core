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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_json_compacts_valid_json() {
        let input = "{\n  \"b\": 2,\n  \"a\": { \"x\": true }\n}";
        let normalized = normalize_json(input).expect("normalization should succeed");

        assert_eq!(normalized, r#"{"a":{"x":true},"b":2}"#);
    }

    #[test]
    fn normalize_json_returns_invalid_json_error() {
        let err = normalize_json("{not-valid-json").expect_err("invalid json should fail");
        assert!(matches!(err, CoreError::InvalidJson(_)));
    }
}
