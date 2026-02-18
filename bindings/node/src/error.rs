use napi::bindgen_prelude::Error;

pub fn map_core_err(e: drift_rust_core::CoreError) -> Error {
    Error::from_reason(e.to_string())
}
