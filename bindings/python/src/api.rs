use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyList};

use crate::conversion::{
    json_value_to_py, py_any_to_optional_bytes, py_any_to_optional_json, py_to_json_value,
};
use crate::error::map_core_err;

#[pyfunction]
pub fn normalize_json(payload_json: &str) -> PyResult<String> {
    drift_rust_core::normalize_json(payload_json).map_err(map_core_err)
}

#[pyfunction]
pub fn deterministic_hash(payload_json: &str) -> PyResult<String> {
    drift_rust_core::deterministic_hash(payload_json).map_err(map_core_err)
}

#[pyfunction]
pub fn normalize_and_hash(payload_json: &str) -> PyResult<(String, String)> {
    drift_rust_core::normalize_and_hash(payload_json).map_err(map_core_err)
}

#[pyfunction]
pub fn object_to_protobuf_struct_bytes(payload_json: &str) -> PyResult<Vec<u8>> {
    drift_rust_core::object_to_protobuf_struct_bytes(payload_json).map_err(map_core_err)
}

#[pyfunction]
pub fn object_to_protobuf_struct_field_count(payload_json: &str) -> PyResult<u32> {
    drift_rust_core::object_to_protobuf_struct_field_count(payload_json)
        .map(|v| v as u32)
        .map_err(map_core_err)
}

#[pyfunction]
pub fn process_export_payload(
    payload_json: &str,
    schema_merges_json: Option<&str>,
) -> PyResult<(String, String, String, String, String, Vec<u8>)> {
    drift_rust_core::process_export_payload(payload_json, schema_merges_json)
        .map(|result| {
            (
                result.normalized_json,
                result.decoded_json,
                result.decoded_value_hash,
                result.decoded_schema_json,
                result.decoded_schema_hash,
                result.protobuf_struct_bytes,
            )
        })
        .map_err(map_core_err)
}

#[pyfunction]
pub fn process_export_payload_pyobject(
    py: Python<'_>,
    payload: &Bound<'_, PyAny>,
    schema_merges: Option<&Bound<'_, PyAny>>,
) -> PyResult<(Py<PyAny>, String, Py<PyAny>, String, Vec<u8>)> {
    let payload_value = py_to_json_value(payload)?;
    let schema_merges_json = if let Some(merges) = schema_merges {
        let merges_value = py_to_json_value(merges)?;
        Some(serde_json::to_string(&merges_value).map_err(|e| {
            pyo3::exceptions::PyValueError::new_err(format!("invalid schema merges: {e}"))
        })?)
    } else {
        None
    };

    drift_rust_core::process_export_payload_value(&payload_value, schema_merges_json.as_deref())
        .and_then(|result| {
            let normalized_py = json_value_to_py(py, &result.normalized_value)
                .map_err(|e| drift_rust_core::CoreError::SerializationError(e.to_string()))?;
            let schema_py = json_value_to_py(py, &result.decoded_schema_value)
                .map_err(|e| drift_rust_core::CoreError::SerializationError(e.to_string()))?;
            Ok((
                normalized_py,
                result.decoded_value_hash,
                schema_py,
                result.decoded_schema_hash,
                result.protobuf_struct_bytes,
            ))
        })
        .map_err(map_core_err)
}

#[pyfunction]
#[allow(clippy::too_many_arguments)]
pub fn build_span_proto_bytes_pyobject(
    trace_id: &str,
    span_id: &str,
    parent_span_id: &str,
    name: &str,
    package_name: &str,
    instrumentation_name: &str,
    submodule_name: &str,
    package_type: i32,
    environment: Option<&str>,
    kind: i32,
    input_schema: &Bound<'_, PyAny>,
    output_schema: &Bound<'_, PyAny>,
    input_schema_hash: &str,
    output_schema_hash: &str,
    input_value_hash: &str,
    output_value_hash: &str,
    status_code: i32,
    status_message: &str,
    is_pre_app_start: bool,
    is_root_span: bool,
    timestamp_seconds: i64,
    timestamp_nanos: i32,
    duration_seconds: i64,
    duration_nanos: i32,
    metadata: Option<&Bound<'_, PyAny>>,
    input_value: Option<&Bound<'_, PyAny>>,
    output_value: Option<&Bound<'_, PyAny>>,
    input_value_proto_struct_bytes: Option<&Bound<'_, PyAny>>,
    output_value_proto_struct_bytes: Option<&Bound<'_, PyAny>>,
) -> PyResult<Vec<u8>> {
    let input_schema_value = py_to_json_value(input_schema)?;
    let output_schema_value = py_to_json_value(output_schema)?;
    let metadata_value = py_any_to_optional_json(metadata)?;
    let input_value_json = py_any_to_optional_json(input_value)?;
    let output_value_json = py_any_to_optional_json(output_value)?;
    let input_struct_bytes = py_any_to_optional_bytes(input_value_proto_struct_bytes)?;
    let output_struct_bytes = py_any_to_optional_bytes(output_value_proto_struct_bytes)?;

    drift_rust_core::build_span_proto_bytes(drift_rust_core::BuildSpanProtoInput {
        trace_id,
        span_id,
        parent_span_id,
        name,
        package_name,
        instrumentation_name,
        submodule_name,
        package_type,
        environment,
        kind,
        input_schema: &input_schema_value,
        output_schema: &output_schema_value,
        input_schema_hash,
        output_schema_hash,
        input_value_hash,
        output_value_hash,
        status_code,
        status_message,
        is_pre_app_start,
        is_root_span,
        timestamp_seconds,
        timestamp_nanos,
        duration_seconds,
        duration_nanos,
        metadata: metadata_value.as_ref(),
        input_value: input_value_json.as_ref(),
        output_value: output_value_json.as_ref(),
        input_value_proto_struct_bytes: input_struct_bytes.as_deref(),
        output_value_proto_struct_bytes: output_struct_bytes.as_deref(),
    })
    .map_err(map_core_err)
}

#[pyfunction]
pub fn build_export_spans_request_bytes_pyobject(
    observable_service_id: &str,
    environment: &str,
    sdk_version: &str,
    sdk_instance_id: &str,
    spans: &Bound<'_, PyAny>,
) -> PyResult<Vec<u8>> {
    let span_list = spans
        .cast::<PyList>()
        .map_err(|_| pyo3::exceptions::PyTypeError::new_err("spans must be a list of bytes"))?;
    let mut span_bytes = Vec::with_capacity(span_list.len());
    for item in span_list.iter() {
        let py_bytes = item
            .cast::<PyBytes>()
            .map_err(|_| pyo3::exceptions::PyTypeError::new_err("each span must be bytes"))?;
        span_bytes.push(py_bytes.as_bytes().to_vec());
    }

    drift_rust_core::build_export_spans_request_bytes(
        observable_service_id,
        environment,
        sdk_version,
        sdk_instance_id,
        &span_bytes,
    )
    .map_err(map_core_err)
}
