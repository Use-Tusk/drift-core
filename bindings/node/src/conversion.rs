use napi::bindgen_prelude::Buffer;

use crate::api::NormalizeAndHashResult;

pub fn tuple_to_normalize_and_hash_result(tuple: (String, String)) -> NormalizeAndHashResult {
    NormalizeAndHashResult {
        normalized_json: tuple.0,
        deterministic_hash: tuple.1,
    }
}

pub fn vec_to_buffer(bytes: Vec<u8>) -> Buffer {
    Buffer::from(bytes)
}
