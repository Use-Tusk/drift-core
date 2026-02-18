use prost::Message;
use prost_types::{Duration, Struct, Timestamp};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use tusk_drift_schemas::tusk::drift::core::v1::{JsonSchema, Span, SpanStatus};

use crate::error::{CoreError, CoreResult};
use crate::protobuf_struct::json_object_to_struct;
use crate::types::BuildSpanProtoInput;

pub fn build_span_proto_bytes(input: BuildSpanProtoInput<'_>) -> CoreResult<Vec<u8>> {
    let input_struct = if let Some(bytes) = input.input_value_proto_struct_bytes {
        Struct::decode(bytes).map_err(|e| CoreError::SerializationError(e.to_string()))?
    } else {
        json_object_to_struct(input.input_value.unwrap_or(&JsonValue::Object(serde_json::Map::new())))
    };

    let output_struct = if let Some(bytes) = input.output_value_proto_struct_bytes {
        Struct::decode(bytes).map_err(|e| CoreError::SerializationError(e.to_string()))?
    } else {
        json_object_to_struct(input.output_value.unwrap_or(&JsonValue::Object(serde_json::Map::new())))
    };

    let metadata_struct = json_object_to_struct(input.metadata.unwrap_or(&JsonValue::Object(serde_json::Map::new())));

    let span = Span {
        trace_id: input.trace_id.to_string(),
        span_id: input.span_id.to_string(),
        parent_span_id: input.parent_span_id.to_string(),
        name: input.name.to_string(),
        package_name: input.package_name.to_string(),
        instrumentation_name: input.instrumentation_name.to_string(),
        submodule_name: input.submodule_name.to_string(),
        package_type: input.package_type,
        input_value: Some(input_struct),
        output_value: Some(output_struct),
        input_schema: Some(json_schema_from_value(input.input_schema)),
        output_schema: Some(json_schema_from_value(input.output_schema)),
        input_schema_hash: input.input_schema_hash.to_string(),
        output_schema_hash: input.output_schema_hash.to_string(),
        input_value_hash: input.input_value_hash.to_string(),
        output_value_hash: input.output_value_hash.to_string(),
        kind: input.kind,
        status: Some(SpanStatus {
            code: input.status_code,
            message: input.status_message.to_string(),
        }),
        is_pre_app_start: input.is_pre_app_start,
        timestamp: Some(Timestamp {
            seconds: input.timestamp_seconds,
            nanos: input.timestamp_nanos,
        }),
        duration: Some(Duration {
            seconds: input.duration_seconds,
            nanos: input.duration_nanos,
        }),
        is_root_span: input.is_root_span,
        metadata: Some(metadata_struct),
        environment: input.environment.map(|v| v.to_string()),
        id: None,
    };

    Ok(span.encode_to_vec())
}

fn json_schema_from_value(value: &JsonValue) -> JsonSchema {
    let obj = value.as_object();
    let schema_type = obj
        .and_then(|o| o.get("type"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i32;

    let properties = obj
        .and_then(|o| o.get("properties"))
        .and_then(|v| v.as_object())
        .map(|props| {
            props
                .iter()
                .map(|(k, v)| (k.clone(), json_schema_from_value(v)))
                .collect::<HashMap<String, JsonSchema>>()
        })
        .unwrap_or_default();

    let items = obj
        .and_then(|o| o.get("items"))
        .map(|v| Box::new(json_schema_from_value(v)));

    let encoding = obj
        .and_then(|o| o.get("encoding"))
        .and_then(|v| v.as_i64())
        .map(|v| v as i32);
    let decoded_type = obj
        .and_then(|o| o.get("decoded_type"))
        .and_then(|v| v.as_i64())
        .map(|v| v as i32);
    let match_importance = obj
        .and_then(|o| o.get("match_importance"))
        .and_then(|v| v.as_f64());

    JsonSchema {
        r#type: schema_type,
        properties,
        items,
        encoding,
        decoded_type,
        match_importance,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BuildSpanProtoInput;

    #[test]
    fn build_span_proto_bytes_decodes_as_generated_span() {
        let input_schema = serde_json::json!({"type": 6, "properties": {}});
        let output_schema = serde_json::json!({"type": 6, "properties": {}});
        let input_value = serde_json::json!({"hello": "world"});
        let output_value = serde_json::json!({"ok": true});
        let metadata = serde_json::json!({"source": "test"});

        let bytes = build_span_proto_bytes(BuildSpanProtoInput {
            trace_id: "trace-1",
            span_id: "span-1",
            parent_span_id: "",
            name: "test-span",
            package_name: "http",
            instrumentation_name: "instr",
            submodule_name: "GET",
            package_type: 1,
            environment: Some("test"),
            kind: 2,
            input_schema: &input_schema,
            output_schema: &output_schema,
            input_schema_hash: "ih",
            output_schema_hash: "oh",
            input_value_hash: "ivh",
            output_value_hash: "ovh",
            status_code: 1,
            status_message: "ok",
            is_pre_app_start: false,
            is_root_span: true,
            timestamp_seconds: 1,
            timestamp_nanos: 2,
            duration_seconds: 3,
            duration_nanos: 4,
            metadata: Some(&metadata),
            input_value: Some(&input_value),
            output_value: Some(&output_value),
            input_value_proto_struct_bytes: None,
            output_value_proto_struct_bytes: None,
        })
        .expect("span bytes should build");

        let decoded = Span::decode(bytes.as_slice()).expect("span bytes should decode");
        assert_eq!(decoded.trace_id, "trace-1");
        assert_eq!(decoded.package_type, 1);
        assert_eq!(decoded.kind, 2);
        assert_eq!(decoded.environment.as_deref(), Some("test"));
    }
}
