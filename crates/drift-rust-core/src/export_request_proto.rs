use prost::Message;
use tusk_drift_schemas::tusk::drift::backend::v1::ExportSpansRequest;
use tusk_drift_schemas::tusk::drift::core::v1::Span;

use crate::error::{CoreError, CoreResult};

pub fn build_export_spans_request_bytes(
    observable_service_id: &str,
    environment: &str,
    sdk_version: &str,
    sdk_instance_id: &str,
    span_proto_bytes_list: &[Vec<u8>],
) -> CoreResult<Vec<u8>> {
    let spans = span_proto_bytes_list
        .iter()
        .map(|span_bytes| {
            Span::decode(span_bytes.as_slice()).map_err(|e| {
                CoreError::SerializationError(format!("failed to decode span proto bytes: {e}"))
            })
        })
        .collect::<CoreResult<Vec<_>>>()?;

    let req = ExportSpansRequest {
        observable_service_id: observable_service_id.to_string(),
        environment: environment.to_string(),
        sdk_version: sdk_version.to_string(),
        sdk_instance_id: sdk_instance_id.to_string(),
        spans,
    };

    Ok(req.encode_to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_generated_export_spans_request_bytes() {
        let span = Span {
            trace_id: "trace-1".to_string(),
            span_id: "span-1".to_string(),
            ..Default::default()
        };
        let span_bytes = span.encode_to_vec();

        let request_bytes = build_export_spans_request_bytes(
            "svc-1",
            "test",
            "0.1.0",
            "sdk-instance-1",
            &[span_bytes],
        )
        .expect("request bytes should build");

        let decoded = ExportSpansRequest::decode(request_bytes.as_slice())
            .expect("request bytes should decode");
        assert_eq!(decoded.observable_service_id, "svc-1");
        assert_eq!(decoded.environment, "test");
        assert_eq!(decoded.spans.len(), 1);
        assert_eq!(decoded.spans[0].trace_id, "trace-1");
    }

    #[test]
    fn returns_error_when_span_bytes_are_invalid() {
        let err = build_export_spans_request_bytes(
            "svc-1",
            "test",
            "0.1.0",
            "sdk-instance-1",
            &[vec![0xff, 0x00, 0xab]],
        )
        .expect_err("invalid span bytes should fail");

        assert!(matches!(err, CoreError::SerializationError(_)));
    }
}
