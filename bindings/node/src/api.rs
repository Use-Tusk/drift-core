use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde_json::Value as JsonValue;

use crate::conversion::{tuple_to_normalize_and_hash_result, vec_to_buffer};
use crate::error::map_core_err;

#[napi]
pub fn normalize_json(payload_json: String) -> Result<String> {
    drift_rust_core::normalize_json(&payload_json).map_err(map_core_err)
}

#[napi]
pub fn deterministic_hash(payload_json: String) -> Result<String> {
    drift_rust_core::deterministic_hash(&payload_json).map_err(map_core_err)
}

#[napi(object)]
pub struct NormalizeAndHashResult {
    pub normalized_json: String,
    pub deterministic_hash: String,
}

#[napi(object)]
pub struct ProcessExportPayloadResult {
  pub normalized_json: String,
  pub decoded_json: String,
  pub decoded_value_hash: String,
  pub decoded_schema_json: String,
  pub decoded_schema_hash: String,
  pub protobuf_struct_bytes: Buffer,
}

#[napi(object)]
pub struct BuildSpanProtoBytesInput {
  pub trace_id: String,
  pub span_id: String,
  pub parent_span_id: String,
  pub name: String,
  pub package_name: String,
  pub instrumentation_name: String,
  pub submodule_name: String,
  pub package_type: i32,
  pub environment: Option<String>,
  pub kind: i32,
  pub input_schema_json: String,
  pub output_schema_json: String,
  pub input_schema_hash: String,
  pub output_schema_hash: String,
  pub input_value_hash: String,
  pub output_value_hash: String,
  pub status_code: i32,
  pub status_message: String,
  pub is_pre_app_start: bool,
  pub is_root_span: bool,
  pub timestamp_seconds: i64,
  pub timestamp_nanos: i32,
  pub duration_seconds: i64,
  pub duration_nanos: i32,
  pub metadata_json: Option<String>,
  pub input_value_json: Option<String>,
  pub output_value_json: Option<String>,
  pub input_value_proto_struct_bytes: Option<Buffer>,
  pub output_value_proto_struct_bytes: Option<Buffer>,
}

#[napi]
pub fn normalize_and_hash(payload_json: String) -> Result<NormalizeAndHashResult> {
    drift_rust_core::normalize_and_hash(&payload_json)
        .map(tuple_to_normalize_and_hash_result)
        .map_err(map_core_err)
}

#[napi]
pub fn object_to_protobuf_struct_bytes(payload_json: String) -> Result<Buffer> {
    drift_rust_core::object_to_protobuf_struct_bytes(&payload_json)
        .map(vec_to_buffer)
        .map_err(map_core_err)
}

#[napi]
pub fn object_to_protobuf_struct_field_count(payload_json: String) -> Result<u32> {
    drift_rust_core::object_to_protobuf_struct_field_count(&payload_json)
        .map(|v| v as u32)
        .map_err(map_core_err)
}

#[napi]
pub fn process_export_payload(
  payload_json: String,
  schema_merges_json: Option<String>,
) -> Result<ProcessExportPayloadResult> {
  drift_rust_core::process_export_payload(&payload_json, schema_merges_json.as_deref())
    .map(|result| ProcessExportPayloadResult {
      normalized_json: result.normalized_json,
      decoded_json: result.decoded_json,
      decoded_value_hash: result.decoded_value_hash,
      decoded_schema_json: result.decoded_schema_json,
      decoded_schema_hash: result.decoded_schema_hash,
      protobuf_struct_bytes: Buffer::from(result.protobuf_struct_bytes),
    })
    .map_err(map_core_err)
}

#[napi]
pub fn build_span_proto_bytes(input: BuildSpanProtoBytesInput) -> Result<Buffer> {
  let input_schema: JsonValue = serde_json::from_str(&input.input_schema_json)
    .map_err(|e| Error::from_reason(format!("invalid input_schema_json: {e}")))?;
  let output_schema: JsonValue = serde_json::from_str(&input.output_schema_json)
    .map_err(|e| Error::from_reason(format!("invalid output_schema_json: {e}")))?;

  let metadata: Option<JsonValue> = input
    .metadata_json
    .as_deref()
    .map(serde_json::from_str)
    .transpose()
    .map_err(|e| Error::from_reason(format!("invalid metadata_json: {e}")))?;
  let input_value: Option<JsonValue> = input
    .input_value_json
    .as_deref()
    .map(serde_json::from_str)
    .transpose()
    .map_err(|e| Error::from_reason(format!("invalid input_value_json: {e}")))?;
  let output_value: Option<JsonValue> = input
    .output_value_json
    .as_deref()
    .map(serde_json::from_str)
    .transpose()
    .map_err(|e| Error::from_reason(format!("invalid output_value_json: {e}")))?;

  drift_rust_core::build_span_proto_bytes(drift_rust_core::BuildSpanProtoInput {
    trace_id: &input.trace_id,
    span_id: &input.span_id,
    parent_span_id: &input.parent_span_id,
    name: &input.name,
    package_name: &input.package_name,
    instrumentation_name: &input.instrumentation_name,
    submodule_name: &input.submodule_name,
    package_type: input.package_type,
    environment: input.environment.as_deref(),
    kind: input.kind,
    input_schema: &input_schema,
    output_schema: &output_schema,
    input_schema_hash: &input.input_schema_hash,
    output_schema_hash: &input.output_schema_hash,
    input_value_hash: &input.input_value_hash,
    output_value_hash: &input.output_value_hash,
    status_code: input.status_code,
    status_message: &input.status_message,
    is_pre_app_start: input.is_pre_app_start,
    is_root_span: input.is_root_span,
    timestamp_seconds: input.timestamp_seconds,
    timestamp_nanos: input.timestamp_nanos,
    duration_seconds: input.duration_seconds,
    duration_nanos: input.duration_nanos,
    metadata: metadata.as_ref(),
    input_value: input_value.as_ref(),
    output_value: output_value.as_ref(),
    input_value_proto_struct_bytes: input.input_value_proto_struct_bytes.as_deref(),
    output_value_proto_struct_bytes: input.output_value_proto_struct_bytes.as_deref(),
  })
  .map(Buffer::from)
  .map_err(map_core_err)
}

#[napi]
pub fn build_export_spans_request_bytes(
  observable_service_id: String,
  environment: String,
  sdk_version: String,
  sdk_instance_id: String,
  spans: Vec<Buffer>,
) -> Result<Buffer> {
  let span_vecs: Vec<Vec<u8>> = spans.into_iter().map(|b| b.to_vec()).collect();
  drift_rust_core::build_export_spans_request_bytes(
    &observable_service_id,
    &environment,
    &sdk_version,
    &sdk_instance_id,
    &span_vecs,
  )
  .map(Buffer::from)
  .map_err(map_core_err)
}
