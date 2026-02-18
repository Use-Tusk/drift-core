use serde_json::Value as JsonValue;

use crate::error::{CoreError, CoreResult};
use crate::normalize::parse_json;
use crate::schema;
use crate::types::{ExportPayloadResult, ExportPayloadValueResult};

pub fn process_export_payload(payload_json: &str, schema_merges_json: Option<&str>) -> CoreResult<ExportPayloadResult> {
    let input = parse_json(payload_json)?;
    let value_result = process_export_payload_value(&input, schema_merges_json)?;
    let normalized_json =
        serde_json::to_string(&value_result.normalized_value).map_err(|e| CoreError::SerializationError(e.to_string()))?;
    let decoded_json =
        serde_json::to_string(&value_result.decoded_value).map_err(|e| CoreError::SerializationError(e.to_string()))?;

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
