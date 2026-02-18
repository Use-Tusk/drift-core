use prost::Message;
use prost_types::{Struct, Value, value::Kind};
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;

use crate::error::CoreResult;
use crate::normalize::{json_roundtrip_normalize, parse_json};

pub fn object_to_protobuf_struct(payload_json: &str) -> CoreResult<Struct> {
    let input = parse_json(payload_json)?;
    let normalized = json_roundtrip_normalize(&input)?;
    Ok(json_object_to_struct(&normalized))
}

pub fn object_to_protobuf_struct_bytes(payload_json: &str) -> CoreResult<Vec<u8>> {
    let s = object_to_protobuf_struct(payload_json)?;
    Ok(s.encode_to_vec())
}

pub fn object_to_protobuf_struct_field_count(payload_json: &str) -> CoreResult<usize> {
    let s = object_to_protobuf_struct(payload_json)?;
    Ok(s.fields.len())
}

pub(crate) fn json_object_to_struct(v: &JsonValue) -> Struct {
    match v {
        JsonValue::Object(map) => {
            let mut fields = BTreeMap::new();
            for (k, child) in map {
                fields.insert(k.clone(), json_to_protobuf_value(child));
            }
            Struct { fields }
        }
        _ => Struct {
            fields: BTreeMap::new(),
        },
    }
}

fn json_to_protobuf_value(v: &JsonValue) -> Value {
    match v {
        JsonValue::Null => Value {
            kind: Some(Kind::NullValue(0)),
        },
        JsonValue::Bool(b) => Value {
            kind: Some(Kind::BoolValue(*b)),
        },
        JsonValue::Number(n) => Value {
            kind: Some(Kind::NumberValue(n.as_f64().unwrap_or(0.0))),
        },
        JsonValue::String(s) => Value {
            kind: Some(Kind::StringValue(s.clone())),
        },
        JsonValue::Array(arr) => Value {
            kind: Some(Kind::ListValue(prost_types::ListValue {
                values: arr.iter().map(json_to_protobuf_value).collect(),
            })),
        },
        JsonValue::Object(map) => {
            let mut fields = BTreeMap::new();
            for (k, child) in map {
                fields.insert(k.clone(), json_to_protobuf_value(child));
            }
            Value {
                kind: Some(Kind::StructValue(Struct { fields })),
            }
        }
    }
}
