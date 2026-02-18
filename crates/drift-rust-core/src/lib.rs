mod error;
mod export_request_proto;
mod hash;
mod normalize;
mod payload;
mod protobuf_struct;
mod schema;
mod span_proto;
mod types;

pub use error::{CoreError, CoreResult};
pub use export_request_proto::build_export_spans_request_bytes;
pub use hash::{deterministic_hash, normalize_and_hash};
pub use normalize::normalize_json;
pub use payload::{process_export_payload, process_export_payload_value};
pub use protobuf_struct::{
    object_to_protobuf_struct, object_to_protobuf_struct_bytes, object_to_protobuf_struct_field_count,
};
pub use span_proto::build_span_proto_bytes;
pub use types::{BuildSpanProtoInput, ExportPayloadResult, ExportPayloadValueResult};
