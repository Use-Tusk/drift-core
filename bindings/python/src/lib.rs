mod api;
mod conversion;
mod error;

use pyo3::prelude::*;

#[pymodule]
fn drift_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(api::normalize_json, m)?)?;
    m.add_function(wrap_pyfunction!(api::deterministic_hash, m)?)?;
    m.add_function(wrap_pyfunction!(api::normalize_and_hash, m)?)?;
    m.add_function(wrap_pyfunction!(api::object_to_protobuf_struct_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(
        api::object_to_protobuf_struct_field_count,
        m
    )?)?;
    m.add_function(wrap_pyfunction!(api::process_export_payload, m)?)?;
    m.add_function(wrap_pyfunction!(api::process_export_payload_pyobject, m)?)?;
    m.add_function(wrap_pyfunction!(api::build_span_proto_bytes_pyobject, m)?)?;
    m.add_function(wrap_pyfunction!(
        api::build_export_spans_request_bytes_pyobject,
        m
    )?)?;
    Ok(())
}
