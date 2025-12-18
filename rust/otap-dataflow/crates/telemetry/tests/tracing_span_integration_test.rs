// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Tests for span start/end handling as log records.
//!
//! This unconventional approach treats spans as pairs of log events:
//! - Span creation → "span.start" log record
//! - Span closure → "span.end" log record with duration
//!
//! This aligns with the unified dataflow architecture where everything
//! flows through the same log pipeline.

#[cfg(test)]
mod tests {
    use otap_df_pdata::views::common::{AnyValueView, AttributeView};
    use otap_df_pdata::views::logs::LogRecordView;
    use otap_df_telemetry::tracing_integration::OtlpTracingLayer;
    use std::sync::{Arc, Mutex};
    use tracing_subscriber::prelude::*;

    #[test]
    fn test_span_creates_start_and_end_events() {
        // Collect all log records (events + span start/end)
        let captured = Arc::new(Mutex::new(Vec::new()));
        let captured_clone = captured.clone();

        let layer = OtlpTracingLayer::new(move |log_record| {
            let mut records = captured_clone.lock().unwrap();
            records.push((
                log_record.event_name().map(|s| String::from_utf8_lossy(s).to_string()),
                log_record.severity_text().map(|s| String::from_utf8_lossy(s).to_string()),
            ));
        });

        let subscriber = tracing_subscriber::registry().with(layer);

        tracing::subscriber::with_default(subscriber, || {
            let span = tracing::info_span!("my_operation", user_id = 42);
            let _guard = span.enter();
            
            // Do some work
            tracing::info!("work happening");
            
            // Span closes when _guard is dropped
        });

        let records = captured.lock().unwrap();
        
        // Should have: span.start, event, span.end
        assert_eq!(records.len(), 3);
        
        // First record: span.start
        assert!(records[0].0.as_ref().unwrap().starts_with("span.start"));
        assert_eq!(records[0].1, Some("INFO".to_string()));
        
        // Second record: regular event
        assert!(records[1].0.as_ref().unwrap().starts_with("event"));
        
        // Third record: span.end
        assert!(records[2].0.as_ref().unwrap().starts_with("span.end"));
        assert_eq!(records[2].1, Some("INFO".to_string()));
    }

    #[test]
    fn test_span_includes_attributes_in_start_event() {
        let captured = Arc::new(Mutex::new(Vec::new()));
        let captured_clone = captured.clone();

        let layer = OtlpTracingLayer::new(move |log_record| {
            let mut records = captured_clone.lock().unwrap();
            
            // Collect attributes
            let attributes: Vec<(String, String)> = log_record
                .attributes()
                .map(|attr| {
                    let key = String::from_utf8_lossy(attr.key()).to_string();
                    let value = if let Some(val) = attr.value() {
                        if let Some(int_val) = val.as_int64() {
                            int_val.to_string()
                        } else {
                            "unknown".to_string()
                        }
                    } else {
                        "unknown".to_string()
                    };
                    (key, value)
                })
                .collect();
            
            records.push((
                log_record.event_name().map(|s| String::from_utf8_lossy(s).to_string()),
                attributes,
            ));
        });

        let subscriber = tracing_subscriber::registry().with(layer);

        tracing::subscriber::with_default(subscriber, || {
            let span = tracing::info_span!("api_call", user_id = 123, request_id = 456);
            let _guard = span.enter();
        });

        let records = captured.lock().unwrap();
        
        // Should have start and end
        assert_eq!(records.len(), 2);
        
        // Check span.start attributes
        let start_attrs = &records[0].1;
        
        // Should have user_id, request_id, and span.id
        let has_user_id = start_attrs.iter().any(|(k, v)| k == "user_id" && v == "123");
        let has_request_id = start_attrs.iter().any(|(k, v)| k == "request_id" && v == "456");
        let has_span_id = start_attrs.iter().any(|(k, _)| k == "span.id");
        
        assert!(has_user_id, "Should have user_id attribute");
        assert!(has_request_id, "Should have request_id attribute");
        assert!(has_span_id, "Should have span.id attribute");
    }

    #[test]
    fn test_span_end_includes_duration() {
        let captured = Arc::new(Mutex::new(Vec::new()));
        let captured_clone = captured.clone();

        let layer = OtlpTracingLayer::new(move |log_record| {
            let mut records = captured_clone.lock().unwrap();
            
            let event_name = log_record.event_name().map(|s| String::from_utf8_lossy(s).to_string());
            
            // Collect attributes for span.end
            let attributes: Vec<(String, i64)> = log_record
                .attributes()
                .filter_map(|attr| {
                    let key = String::from_utf8_lossy(attr.key()).to_string();
                    if let Some(val) = attr.value() {
                        if let Some(int_val) = val.as_int64() {
                            return Some((key, int_val));
                        }
                    }
                    None
                })
                .collect();
            
            records.push((event_name, attributes));
        });

        let subscriber = tracing_subscriber::registry().with(layer);

        tracing::subscriber::with_default(subscriber, || {
            let span = tracing::info_span!("timed_operation");
            let _guard = span.enter();
            
            // Simulate some work
            std::thread::sleep(std::time::Duration::from_millis(10));
        });

        let records = captured.lock().unwrap();
        assert_eq!(records.len(), 2);
        
        // Check span.end for duration
        let end_attrs = &records[1].1;
        let duration = end_attrs.iter()
            .find(|(k, _)| k == "span.duration_nanos")
            .map(|(_, v)| *v);
        
        assert!(duration.is_some(), "Should have duration in span.end");
        assert!(duration.unwrap() > 0, "Duration should be positive");
        // Duration should be at least 10ms (10_000_000 nanos)
        assert!(duration.unwrap() >= 10_000_000, "Duration should be at least 10ms");
    }

    #[test]
    fn test_nested_spans_create_separate_events() {
        let captured = Arc::new(Mutex::new(Vec::new()));
        let captured_clone = captured.clone();

        let layer = OtlpTracingLayer::new(move |log_record| {
            let mut records = captured_clone.lock().unwrap();
            records.push(
                log_record.event_name().map(|s| String::from_utf8_lossy(s).to_string())
            );
        });

        let subscriber = tracing_subscriber::registry().with(layer);

        tracing::subscriber::with_default(subscriber, || {
            let outer = tracing::info_span!("outer");
            let _outer_guard = outer.enter();
            
            let inner = tracing::info_span!("inner");
            let _inner_guard = inner.enter();
            
            // Inner span closes first
            drop(_inner_guard);
            
            // Outer span closes second
        });

        let records = captured.lock().unwrap();
        
        // Should have: outer.start, inner.start, inner.end, outer.end
        assert_eq!(records.len(), 4);
        
        assert!(records[0].as_ref().unwrap().contains("span.start"));
        assert!(records[0].as_ref().unwrap().contains("outer"));
        
        assert!(records[1].as_ref().unwrap().contains("span.start"));
        assert!(records[1].as_ref().unwrap().contains("inner"));
        
        assert!(records[2].as_ref().unwrap().contains("span.end"));
        
        assert!(records[3].as_ref().unwrap().contains("span.end"));
    }

    #[test]
    fn test_span_and_event_interleaving() {
        let captured = Arc::new(Mutex::new(Vec::new()));
        let captured_clone = captured.clone();

        let layer = OtlpTracingLayer::new(move |log_record| {
            let mut records = captured_clone.lock().unwrap();
            records.push(
                log_record.event_name().map(|s| String::from_utf8_lossy(s).to_string())
            );
        });

        let subscriber = tracing_subscriber::registry().with(layer);

        tracing::subscriber::with_default(subscriber, || {
            tracing::info!("before span");
            
            let span = tracing::info_span!("operation");
            let _guard = span.enter();
            
            tracing::info!("inside span");
            
            drop(_guard);
            
            tracing::info!("after span");
        });

        let records = captured.lock().unwrap();
        
        // Should have: event, span.start, event, span.end, event
        // Note: span.end happens when the span is dropped, not when guard is dropped
        assert_eq!(records.len(), 5, "Expected 5 records, got: {:?}", records);
        
        assert!(records[0].as_ref().unwrap().starts_with("event"), "Record 0: {:?}", records[0]);
        assert!(records[1].as_ref().unwrap().contains("span.start"), "Record 1: {:?}", records[1]);
        assert!(records[2].as_ref().unwrap().starts_with("event"), "Record 2: {:?}", records[2]);
        // span.end can come after the last event since span is dropped at end of scope
        if records.len() >= 4 {
            let has_span_end = records[3].as_ref().unwrap().contains("span.end") || 
                               records[4].as_ref().unwrap().contains("span.end");
            assert!(has_span_end, "Should have span.end in records 3 or 4: {:?}", records);
        }
    }

    #[test]
    fn test_span_target_for_scope_batching() {
        let captured = Arc::new(Mutex::new(Vec::new()));
        let captured_clone = captured.clone();

        let layer = OtlpTracingLayer::new(move |log_record| {
            let mut records = captured_clone.lock().unwrap();
            records.push((
                log_record.event_name().map(|s| String::from_utf8_lossy(s).to_string()),
                log_record.target().to_string(),
            ));
        });

        let subscriber = tracing_subscriber::registry().with(layer);

        tracing::subscriber::with_default(subscriber, || {
            let span = tracing::info_span!("operation");
            let _guard = span.enter();
        });

        let records = captured.lock().unwrap();
        
        // Both span.start and span.end should have target for scope batching
        assert_eq!(records.len(), 2);
        
        // span.start should have the actual target from metadata
        assert!(!records[0].1.is_empty(), "span.start should have target");
        
        // span.end uses a generic target since we don't have metadata
        assert_eq!(records[1].1, "tracing::span", "span.end should use tracing::span target");
    }
}
