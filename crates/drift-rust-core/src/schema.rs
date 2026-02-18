use base64::Engine as _;
use prost::Message;
use serde::Deserialize;
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;

use crate::error::{CoreError, CoreResult};
use crate::hash::hash_json_value_deterministic;
use crate::normalize::json_roundtrip_normalize;
use crate::protobuf_struct::json_object_to_struct;
use crate::types::ExportPayloadValueResult;

#[derive(Debug, Deserialize)]
struct MergeRule {
    #[serde(default)]
    encoding: Option<i32>,
    #[serde(default)]
    decoded_type: Option<i32>,
    #[serde(default)]
    match_importance: Option<f64>,
}

pub(crate) fn process_export_payload_value(
    payload_value: &JsonValue,
    schema_merges_json: Option<&str>,
) -> CoreResult<ExportPayloadValueResult> {
    let normalized = json_roundtrip_normalize(payload_value)?;
    let decoded = if let Some(merges_json) = schema_merges_json {
        apply_schema_merges_top_level(&normalized, merges_json)?
    } else {
        normalized.clone()
    };
    let merge_map = if let Some(merges_json) = schema_merges_json {
        serde_json::from_str::<BTreeMap<String, MergeRule>>(merges_json)
            .map_err(|e| CoreError::InvalidJson(e.to_string()))?
    } else {
        BTreeMap::new()
    };
    let decoded_schema_value = generate_schema_json_value(&decoded, Some(&merge_map), true);
    let decoded_value_hash = hash_json_value_deterministic(&decoded)?;
    let decoded_schema_hash = hash_json_value_deterministic(&decoded_schema_value)?;
    let protobuf_struct_bytes = json_object_to_struct(&normalized).encode_to_vec();

    Ok(ExportPayloadValueResult {
        normalized_value: normalized,
        decoded_value: decoded,
        decoded_value_hash,
        decoded_schema_value,
        decoded_schema_hash,
        protobuf_struct_bytes,
    })
}

fn apply_schema_merges_top_level(
    normalized: &JsonValue,
    schema_merges_json: &str,
) -> CoreResult<JsonValue> {
    let merge_map: BTreeMap<String, MergeRule> = serde_json::from_str(schema_merges_json)
        .map_err(|e| CoreError::InvalidJson(e.to_string()))?;

    if merge_map.is_empty() {
        return Ok(normalized.clone());
    }

    let mut decoded = match normalized {
        JsonValue::Object(map) => map.clone(),
        _ => return Ok(normalized.clone()),
    };

    for (key, merge) in &merge_map {
        let Some(value) = decoded.get(key) else {
            continue;
        };
        let mut working_value = value.clone();

        if merge.encoding == Some(1)
            && let JsonValue::String(s) = &working_value
            && let Ok(bytes) =
                base64::engine::general_purpose::STANDARD.decode(s.as_bytes())
        {
            working_value = JsonValue::String(String::from_utf8_lossy(&bytes).to_string());
        }

        if merge.decoded_type == Some(1)
            && let JsonValue::String(s) = &working_value
            && let Ok(parsed) = serde_json::from_str::<JsonValue>(s)
        {
            working_value = parsed;
        }

        decoded.insert(key.clone(), working_value);
    }

    Ok(JsonValue::Object(decoded))
}

fn json_type_code(value: &JsonValue) -> i64 {
    match value {
        JsonValue::Null => 4,      // NULL
        JsonValue::Bool(_) => 3,   // BOOLEAN
        JsonValue::Number(_) => 1, // NUMBER
        JsonValue::String(_) => 2, // STRING
        JsonValue::Array(_) => 7,  // ORDERED_LIST
        JsonValue::Object(_) => 6, // OBJECT
    }
}

fn generate_schema_json_value(
    value: &JsonValue,
    schema_merges: Option<&BTreeMap<String, MergeRule>>,
    at_object_root: bool,
) -> JsonValue {
    let mut schema_obj = serde_json::Map::new();
    schema_obj.insert(
        "type".to_string(),
        JsonValue::Number(json_type_code(value).into()),
    );
    schema_obj.insert(
        "properties".to_string(),
        JsonValue::Object(serde_json::Map::new()),
    );

    match value {
        JsonValue::Array(arr) => {
            if let Some(first) = arr.first() {
                schema_obj.insert(
                    "items".to_string(),
                    generate_schema_json_value(first, None, false),
                );
            }
        }
        JsonValue::Object(map) => {
            let mut props = serde_json::Map::new();
            for (k, child) in map {
                let mut child_schema = generate_schema_json_value(child, None, false);
                if at_object_root
                    && let Some(merges) = schema_merges
                    && let Some(merge) = merges.get(k)
                    && let JsonValue::Object(child_obj) = &mut child_schema
                {
                    if let Some(enc) = merge.encoding {
                        child_obj.insert(
                            "encoding".to_string(),
                            JsonValue::Number(enc.into()),
                        );
                    }
                    if let Some(decoded_type) = merge.decoded_type {
                        child_obj.insert(
                            "decoded_type".to_string(),
                            JsonValue::Number(decoded_type.into()),
                        );
                    }
                    if let Some(match_importance) = merge.match_importance
                        && let Some(n) = serde_json::Number::from_f64(match_importance)
                    {
                        child_obj.insert(
                            "match_importance".to_string(),
                            JsonValue::Number(n),
                        );
                    }
                }
                props.insert(k.clone(), child_schema);
            }
            schema_obj.insert("properties".to_string(), JsonValue::Object(props));
        }
        _ => {}
    }

    JsonValue::Object(schema_obj)
}
