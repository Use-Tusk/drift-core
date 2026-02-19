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

#[cfg(test)]
mod tests {
    use super::*;
    use prost_types::value::Kind;

    #[test]
    fn object_to_protobuf_struct_preserves_top_level_fields() {
        let payload = r#"{"n":1,"s":"x","b":true,"arr":[1,2],"obj":{"k":"v"},"nullv":null}"#;
        let s = object_to_protobuf_struct(payload).expect("conversion should succeed");

        assert_eq!(s.fields.len(), 6);
        assert!(matches!(
            s.fields.get("nullv").and_then(|v| v.kind.as_ref()),
            Some(Kind::NullValue(_))
        ));
    }

    #[test]
    fn non_object_json_yields_empty_struct() {
        let s = object_to_protobuf_struct("[1,2,3]").expect("conversion should succeed");
        assert!(s.fields.is_empty());
    }

    #[test]
    fn object_to_protobuf_struct_field_count_matches_field_total() {
        let payload = r#"{"a":1,"b":2,"c":{"nested":3}}"#;
        let count =
            object_to_protobuf_struct_field_count(payload).expect("field count should succeed");
        assert_eq!(count, 3);
    }
}
