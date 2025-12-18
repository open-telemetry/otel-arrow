// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! End-to-end test of span start/end encoding to OTLP format.
//!
//! This test verifies that spans are correctly encoded as pairs of log records
//! (span.start and span.end) with proper attributes and duration tracking.

#[cfg(test)]
mod tests {
    use otap_df_pdata::otlp::stateful_encoder::StatefulOtlpEncoder;
    use otap_df_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
    use otap_df_pdata::proto::opentelemetry::logs::v1::LogsData;
    use otap_df_telemetry::tracing_integration::OtlpTracingLayer;
    use prost::Message as ProstMessage;
    use std::sync::{Arc, Mutex};
    use tracing_subscriber::prelude::*;

    #[test]
    fn test_span_start_and_end_encoded() {
        let encoder = Arc::new(Mutex::new(StatefulOtlpEncoder::new(64 * 1024)));
        let encoder_clone = encoder.clone();
        let resource_bytes = vec![];

        let layer = OtlpTracingLayer::new(move |log_record| {
            if let Ok(mut enc) = encoder_clone.lock() {
                let _ = enc.encode_log_record(&log_record, &resource_bytes, log_record.target());
            }
        });

        let subscriber = tracing_subscriber::registry().with(layer);

        // Generate span with attributes
        tracing::subscriber::with_default(subscriber, || {
            let span = tracing::info_span!("database_query", query_id = 42, db = "users");
            let _guard = span.enter();
            // Span closes when _guard is dropped
        });

        // Decode the encoded bytes
        let encoded = encoder.lock().unwrap().flush();
        let decoded: ExportLogsServiceRequest = ProstMessage::decode(&encoded[..])
            .expect("Should decode as valid OTLP");

        let logs_data = LogsData {
            resource_logs: decoded.resource_logs,
        };

        // Should have 2 scopes: one for span.start (test module), one for span.end (tracing::span)
        let resource_logs = &logs_data.resource_logs[0];
        assert_eq!(
            resource_logs.scope_logs.len(),
            2,
            "Should have 2 scopes: span.start (test target) and span.end (tracing::span)"
        );

        // First scope has span.start
        let span_start_scope = &resource_logs.scope_logs[0];
        assert_eq!(span_start_scope.log_records.len(), 1, "Should have span.start");

        let span_start = &span_start_scope.log_records[0];
        
        // Verify span.start has the span attributes
        let has_query_id = span_start.attributes.iter()
            .any(|kv| kv.key == "query_id");
        let has_db = span_start.attributes.iter()
            .any(|kv| kv.key == "db");
        let has_span_id = span_start.attributes.iter()
            .any(|kv| kv.key == "span.id");
        
        assert!(has_query_id, "span.start should have query_id");
        assert!(has_db, "span.start should have db");
        assert!(has_span_id, "span.start should have span.id");

        // Second scope has span.end
        let span_end_scope = &resource_logs.scope_logs[1];
        assert_eq!(span_end_scope.scope.as_ref().unwrap().name, "tracing::span");
        assert_eq!(span_end_scope.log_records.len(), 1, "Should have span.end");

        let span_end = &span_end_scope.log_records[0];
        
        // Verify span.end has duration
        let has_duration = span_end.attributes.iter()
            .any(|kv| kv.key == "span.duration_nanos");
        
        assert!(has_duration, "span.end should have span.duration_nanos");
    }

    #[test]
    fn test_span_interleaved_with_events() {
        let encoder = Arc::new(Mutex::new(StatefulOtlpEncoder::new(64 * 1024)));
        let encoder_clone = encoder.clone();
        let resource_bytes = vec![];

        let layer = OtlpTracingLayer::new(move |log_record| {
            if let Ok(mut enc) = encoder_clone.lock() {
                let _ = enc.encode_log_record(&log_record, &resource_bytes, log_record.target());
            }
        });

        let subscriber = tracing_subscriber::registry().with(layer);

        tracing::subscriber::with_default(subscriber, || {
            tracing::info!("before span");

            let span = tracing::info_span!("my_operation");
            let _guard = span.enter();

            tracing::info!("inside span");

            drop(_guard);

            tracing::info!("after span");
        });

        let encoded = encoder.lock().unwrap().flush();
        let decoded: ExportLogsServiceRequest = ProstMessage::decode(&encoded[..])
            .expect("Should decode as valid OTLP");

        let logs_data = LogsData {
            resource_logs: decoded.resource_logs,
        };

        // Should have 2 scopes
        let resource_logs = &logs_data.resource_logs[0];
        assert_eq!(resource_logs.scope_logs.len(), 2);

        // First scope: all events + span.start (same test module target)
        let first_scope = &resource_logs.scope_logs[0];
        assert_eq!(
            first_scope.log_records.len(),
            4,
            "Should have: before, span.start, inside, after"
        );

        // Second scope: span.end only
        let second_scope = &resource_logs.scope_logs[1];
        assert_eq!(second_scope.scope.as_ref().unwrap().name, "tracing::span");
        assert_eq!(second_scope.log_records.len(), 1, "Should have span.end");
    }

    #[test]
    fn test_nested_spans() {
        let encoder = Arc::new(Mutex::new(StatefulOtlpEncoder::new(64 * 1024)));
        let encoder_clone = encoder.clone();
        let resource_bytes = vec![];

        let layer = OtlpTracingLayer::new(move |log_record| {
            if let Ok(mut enc) = encoder_clone.lock() {
                let _ = enc.encode_log_record(&log_record, &resource_bytes, log_record.target());
            }
        });

        let subscriber = tracing_subscriber::registry().with(layer);

        tracing::subscriber::with_default(subscriber, || {
            let outer = tracing::info_span!("outer_span");
            let _outer_guard = outer.enter();

            tracing::info!("in outer");

            let inner = tracing::info_span!("inner_span");
            let _inner_guard = inner.enter();

            tracing::info!("in inner");

            drop(_inner_guard);
            drop(_outer_guard);
        });

        let encoded = encoder.lock().unwrap().flush();
        let decoded: ExportLogsServiceRequest = ProstMessage::decode(&encoded[..])
            .expect("Should decode as valid OTLP");

        let logs_data = LogsData {
            resource_logs: decoded.resource_logs,
        };

        // Should have 2 scopes
        let resource_logs = &logs_data.resource_logs[0];
        assert_eq!(resource_logs.scope_logs.len(), 2);

        // First scope: events + both span.start records
        let first_scope = &resource_logs.scope_logs[0];
        assert_eq!(
            first_scope.log_records.len(),
            4,
            "Should have: outer.start, event, inner.start, event"
        );

        // Second scope: both span.end records
        let second_scope = &resource_logs.scope_logs[1];
        assert_eq!(second_scope.scope.as_ref().unwrap().name, "tracing::span");
        assert_eq!(
            second_scope.log_records.len(),
            2,
            "Should have: inner.end, outer.end"
        );
    }

    #[test]
    fn test_span_duration_is_positive() {
        let encoder = Arc::new(Mutex::new(StatefulOtlpEncoder::new(64 * 1024)));
        let encoder_clone = encoder.clone();
        let resource_bytes = vec![];

        let layer = OtlpTracingLayer::new(move |log_record| {
            if let Ok(mut enc) = encoder_clone.lock() {
                let _ = enc.encode_log_record(&log_record, &resource_bytes, log_record.target());
            }
        });

        let subscriber = tracing_subscriber::registry().with(layer);

        tracing::subscriber::with_default(subscriber, || {
            let span = tracing::info_span!("timed_operation");
            let _guard = span.enter();
            
            // Small sleep to ensure measurable duration
            std::thread::sleep(std::time::Duration::from_millis(10));
        });

        let encoded = encoder.lock().unwrap().flush();
        let decoded: ExportLogsServiceRequest = ProstMessage::decode(&encoded[..])
            .expect("Should decode as valid OTLP");

        let logs_data = LogsData {
            resource_logs: decoded.resource_logs,
        };

        // Get span.end from second scope
        let span_end_scope = &logs_data.resource_logs[0].scope_logs[1];
        let span_end = &span_end_scope.log_records[0];

        // Find duration attribute
        use otap_df_pdata::proto::opentelemetry::common::v1::any_value::Value;
        let duration = span_end.attributes.iter()
            .find(|kv| kv.key == "span.duration_nanos")
            .and_then(|kv| kv.value.as_ref())
            .and_then(|v| match &v.value {
                Some(Value::IntValue(i)) => Some(*i),
                _ => None,
            });

        assert!(duration.is_some(), "Should have duration");
        assert!(duration.unwrap() > 0, "Duration should be positive");
        // Should be at least 10ms (10_000_000 nanos)
        assert!(
            duration.unwrap() >= 10_000_000,
            "Duration should be at least 10ms, got {}ns",
            duration.unwrap()
        );
    }
}
