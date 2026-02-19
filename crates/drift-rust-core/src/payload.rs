use serde_json::Value as JsonValue;

use crate::error::{CoreError, CoreResult};
use crate::normalize::parse_json;
use crate::schema;
use crate::types::{ExportPayloadResult, ExportPayloadValueResult};

pub fn process_export_payload(
    payload_json: &str,
    schema_merges_json: Option<&str>,
) -> CoreResult<ExportPayloadResult> {
    let input = parse_json(payload_json)?;
    let value_result = process_export_payload_value(&input, schema_merges_json)?;
    let normalized_json = serde_json::to_string(&value_result.normalized_value)
        .map_err(|e| CoreError::SerializationError(e.to_string()))?;
    let decoded_json = serde_json::to_string(&value_result.decoded_value)
        .map_err(|e| CoreError::SerializationError(e.to_string()))?;

    Ok(ExportPayloadResult {
        normalized_json,
        decoded_json,
        decoded_value_hash: value_result.decoded_value_hash,
        decoded_schema_json: serde_json::to_string(&value_result.decoded_schema_value)
            .map_err(|e| CoreError::SerializationError(e.to_string()))?,
        decoded_schema_hash: value_result.decoded_schema_hash,
        protobuf_struct_bytes: value_result.protobuf_struct_bytes,
    })
}

pub fn process_export_payload_value(
    payload_value: &JsonValue,
    schema_merges_json: Option<&str>,
) -> CoreResult<ExportPayloadValueResult> {
    schema::process_export_payload_value(payload_value, schema_merges_json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_export_payload_value_without_merges_keeps_decoded_equal_normalized() {
        let payload = serde_json::json!({"k":"v","n":1});
        let result = process_export_payload_value(&payload, None).expect("processing should work");

        assert_eq!(result.decoded_value, result.normalized_value);
        assert_eq!(result.decoded_schema_value["type"], serde_json::json!(6));
        assert!(!result.protobuf_struct_bytes.is_empty());
    }

    #[test]
    fn process_export_payload_value_applies_base64_and_json_decoding_merges() {
        let payload = serde_json::json!({
            "decoded_blob": "eyJrIjoidiJ9"
        });
        let merges = r#"{"decoded_blob":{"encoding":1,"decoded_type":1,"match_importance":0.75}}"#;

        let result = process_export_payload_value(&payload, Some(merges))
            .expect("processing with merges should work");

        assert_eq!(
            result.decoded_value["decoded_blob"]["k"],
            serde_json::json!("v")
        );
        assert_eq!(
            result.decoded_schema_value["properties"]["decoded_blob"]["encoding"],
            serde_json::json!(1)
        );
        assert_eq!(
            result.decoded_schema_value["properties"]["decoded_blob"]["decoded_type"],
            serde_json::json!(1)
        );
    }

    #[test]
    fn process_export_payload_returns_error_for_invalid_merges_json() {
        let payload = r#"{"k":"v"}"#;
        let err = process_export_payload(payload, Some("{not-json"))
            .expect_err("invalid merges json should fail");
        assert!(matches!(err, CoreError::InvalidJson(_)));
    }
}
