use serde_json::Value as JsonValue;
use tusk_drift_schemas::tusk::drift::core::v1::{PackageType, SpanKind, StatusCode};

#[derive(Debug, Clone)]
pub struct ExportPayloadResult {
    pub normalized_json: String,
    pub decoded_json: String,
    pub decoded_value_hash: String,
    pub decoded_schema_json: String,
    pub decoded_schema_hash: String,
    pub protobuf_struct_bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct ExportPayloadValueResult {
    pub normalized_value: JsonValue,
    pub decoded_value: JsonValue,
    pub decoded_value_hash: String,
    pub decoded_schema_value: JsonValue,
    pub decoded_schema_hash: String,
    pub protobuf_struct_bytes: Vec<u8>,
}

pub struct BuildSpanProtoInput<'a> {
    pub trace_id: &'a str,
    pub span_id: &'a str,
    pub parent_span_id: &'a str,
    pub name: &'a str,
    pub package_name: &'a str,
    pub instrumentation_name: &'a str,
    pub submodule_name: &'a str,
    pub package_type: PackageType,
    pub environment: Option<&'a str>,
    pub kind: SpanKind,
    pub input_schema: &'a JsonValue,
    pub output_schema: &'a JsonValue,
    pub input_schema_hash: &'a str,
    pub output_schema_hash: &'a str,
    pub input_value_hash: &'a str,
    pub output_value_hash: &'a str,
    pub status_code: StatusCode,
    pub status_message: &'a str,
    pub is_pre_app_start: bool,
    pub is_root_span: bool,
    pub timestamp_seconds: i64,
    pub timestamp_nanos: i32,
    pub duration_seconds: i64,
    pub duration_nanos: i32,
    pub metadata: Option<&'a JsonValue>,
    pub input_value: Option<&'a JsonValue>,
    pub output_value: Option<&'a JsonValue>,
    pub input_value_proto_struct_bytes: Option<&'a [u8]>,
    pub output_value_proto_struct_bytes: Option<&'a [u8]>,
}
