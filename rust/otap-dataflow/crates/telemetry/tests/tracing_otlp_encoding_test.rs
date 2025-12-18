// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Comprehensive test verifying that tracing events are correctly encoded to OTLP format.
//!
//! This test demonstrates the complete round-trip:
//! 1. Emit tracing events from different modules/scopes
//! 2. Capture them via OtlpTracingLayer
//! 3. Encode to OTLP bytes using StatefulOtlpEncoder
//! 4. Decode the bytes back to protobuf structures
//! 5. Verify the structure matches expected OTLP format using equivalence testing

#[cfg(test)]
mod tests {
    use otap_df_pdata::otlp::stateful_encoder::StatefulOtlpEncoder;
    use otap_df_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
    use otap_df_pdata::proto::opentelemetry::common::v1::{AnyValue, InstrumentationScope, KeyValue};
    use otap_df_pdata::proto::opentelemetry::logs::v1::{LogRecord, LogsData, ResourceLogs, ScopeLogs, SeverityNumber};
    use otap_df_pdata::proto::opentelemetry::resource::v1::Resource;
    use otap_df_pdata::testing::equiv::logs::assert_logs_equivalent;
    use otap_df_telemetry::tracing_integration::OtlpTracingLayer;
    use prost::Message as ProstMessage;
    use std::sync::{Arc, Mutex};
    use tracing_subscriber::prelude::*;

    /// Helper to encode resource as OTLP bytes (same as the example uses)
    fn encode_resource_bytes(_resource: &Resource) -> Vec<u8> {
        // For this test, we use empty resource as requested in the initial requirements
        vec![]
    }

    #[test]
    fn test_tracing_events_encode_to_valid_otlp() {
        // Create shared encoder
        let encoder = Arc::new(Mutex::new(StatefulOtlpEncoder::new(64 * 1024)));
        let encoder_clone = encoder.clone();

        // Use empty resource (as per initial requirements)
        let resource = Resource::default();
        let resource_bytes = encode_resource_bytes(&resource);

        // Create layer that encodes tracing events to OTLP
        let layer = OtlpTracingLayer::new(move |log_record| {
            // Use tracing target (module path) as scope name
            let scope_name = log_record.target();
            
            if let Ok(mut enc) = encoder_clone.lock() {
                enc.encode_log_record(&log_record, &resource_bytes, scope_name)
                    .expect("Failed to encode log record");
            }
        });

        // Install the subscriber
        let subscriber = tracing_subscriber::registry().with(layer);

        // Emit tracing events from different scopes/modules
        tracing::subscriber::with_default(subscriber, || {
            // First scope: app::server (2 events)
            tracing::info!(target: "app::server", port = 8080, "Server starting");
            tracing::info!(target: "app::server", "Server ready");
            
            // Second scope: app::database (2 events)
            tracing::warn!(target: "app::database", retry_count = 3, "Connection retry");
            tracing::error!(target: "app::database", "Connection failed");
            
            // Third scope: app::cache (1 event)
            tracing::debug!(target: "app::cache", hit_rate = 0.95, "Cache stats");
            
            // Fourth scope: third_party::lib (1 event)
            tracing::info!(target: "third_party::lib", version = "2.0", "Library initialized");
        });

        // Flush and get the encoded OTLP bytes
        let otlp_bytes = encoder.lock().unwrap().flush();
        
        // Verify we got some data
        assert!(!otlp_bytes.is_empty(), "Should have generated OTLP bytes");
        println!("Generated {} bytes of OTLP data", otlp_bytes.len());

        // Decode the bytes back to ExportLogsServiceRequest
        let decoded_request: ExportLogsServiceRequest = ProstMessage::decode(&otlp_bytes[..])
            .expect("Failed to decode OTLP bytes");

        println!("Decoded {} ResourceLogs", decoded_request.resource_logs.len());

        // Verify structure: should have 1 ResourceLogs with 4 ScopeLogs
        assert_eq!(decoded_request.resource_logs.len(), 1, "Should have 1 ResourceLogs");
        
        let resource_logs = &decoded_request.resource_logs[0];
        assert_eq!(resource_logs.scope_logs.len(), 4, "Should have 4 ScopeLogs (one per scope)");

        // Verify each scope has the correct number of log records
        let scope_names: Vec<String> = resource_logs.scope_logs
            .iter()
            .map(|sl| sl.scope.as_ref().map(|s| s.name.clone()).unwrap_or_default())
            .collect();
        
        assert_eq!(scope_names, vec![
            "app::server",
            "app::database", 
            "app::cache",
            "third_party::lib"
        ], "Scope names should match targets");

        // Verify log record counts per scope
        assert_eq!(resource_logs.scope_logs[0].log_records.len(), 2, "app::server should have 2 logs");
        assert_eq!(resource_logs.scope_logs[1].log_records.len(), 2, "app::database should have 2 logs");
        assert_eq!(resource_logs.scope_logs[2].log_records.len(), 1, "app::cache should have 1 log");
        assert_eq!(resource_logs.scope_logs[3].log_records.len(), 1, "third_party::lib should have 1 log");

        // Construct expected OTLP structure using builder pattern
        let expected = LogsData {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource::default()),
                scope_logs: vec![
                    // Scope 1: app::server
                    ScopeLogs {
                        scope: Some(InstrumentationScope::build()
                            .name("app::server")
                            .finish()),
                        log_records: vec![
                            LogRecord::build()
                                .severity_number(SeverityNumber::Info)
                                .severity_text("INFO")
                                .body(AnyValue::new_string("Server starting"))
                                .attributes(vec![KeyValue::new("port", AnyValue::new_int(8080))])
                                .finish(),
                            LogRecord::build()
                                .severity_number(SeverityNumber::Info)
                                .severity_text("INFO")
                                .body(AnyValue::new_string("Server ready"))
                                .finish(),
                        ],
                        schema_url: "".into(),
                    },
                    // Scope 2: app::database
                    ScopeLogs {
                        scope: Some(InstrumentationScope::build()
                            .name("app::database")
                            .finish()),
                        log_records: vec![
                            LogRecord::build()
                                .severity_number(SeverityNumber::Warn)
                                .severity_text("WARN")
                                .body(AnyValue::new_string("Connection retry"))
                                .attributes(vec![KeyValue::new("retry_count", AnyValue::new_int(3))])
                                .finish(),
                            LogRecord::build()
                                .severity_number(SeverityNumber::Error)
                                .severity_text("ERROR")
                                .body(AnyValue::new_string("Connection failed"))
                                .finish(),
                        ],
                        schema_url: "".into(),
                    },
                    // Scope 3: app::cache
                    ScopeLogs {
                        scope: Some(InstrumentationScope::build()
                            .name("app::cache")
                            .finish()),
                        log_records: vec![
                            LogRecord::build()
                                .severity_number(SeverityNumber::Debug)
                                .severity_text("DEBUG")
                                .body(AnyValue::new_string("Cache stats"))
                                .attributes(vec![KeyValue::new("hit_rate", AnyValue::new_double(0.95))])
                                .finish(),
                        ],
                        schema_url: "".into(),
                    },
                    // Scope 4: third_party::lib
                    ScopeLogs {
                        scope: Some(InstrumentationScope::build()
                            .name("third_party::lib")
                            .finish()),
                        log_records: vec![
                            LogRecord::build()
                                .severity_number(SeverityNumber::Info)
                                .severity_text("INFO")
                                .body(AnyValue::new_string("Library initialized"))
                                .attributes(vec![KeyValue::new("version", AnyValue::new_string("2.0"))])
                                .finish(),
                        ],
                        schema_url: "".into(),
                    },
                ],
                schema_url: "".into(),
            }],
        };

        // Convert decoded request to LogsData format for equivalence testing
        let decoded_logs_data = LogsData {
            resource_logs: decoded_request.resource_logs,
        };

        // Use equivalence testing to verify the structure matches
        // This handles:
        // - Different attribute orderings
        // - Timestamp variations (we don't check exact timestamps)
        // - Potential restructuring of the hierarchy
        // Note: Since timestamps are generated at runtime, we need to copy them from decoded to expected
        let mut expected_with_timestamps = expected.clone();
        for (expected_rl, decoded_rl) in expected_with_timestamps.resource_logs.iter_mut()
            .zip(decoded_logs_data.resource_logs.iter()) {
            for (expected_sl, decoded_sl) in expected_rl.scope_logs.iter_mut()
                .zip(decoded_rl.scope_logs.iter()) {
                for (expected_lr, decoded_lr) in expected_sl.log_records.iter_mut()
                    .zip(decoded_sl.log_records.iter()) {
                    expected_lr.time_unix_nano = decoded_lr.time_unix_nano;
                    expected_lr.observed_time_unix_nano = decoded_lr.observed_time_unix_nano;
                }
            }
        }

        // Assert equivalence
        assert_logs_equivalent(&[decoded_logs_data], &[expected_with_timestamps]);

        println!("✓ OTLP encoding verified: structure matches expected format");
        println!("✓ 6 log records encoded across 4 scopes");
        println!("✓ Scopes derived from tracing targets (module paths)");
        println!("✓ Automatic batching by scope name works correctly");
    }

    #[test]
    fn test_tracing_scope_batching() {
        // This test specifically verifies that multiple events with the same target
        // are batched into a single ScopeLogs message

        let encoder = Arc::new(Mutex::new(StatefulOtlpEncoder::new(64 * 1024)));
        let encoder_clone = encoder.clone();

        let resource_bytes = vec![];

        let layer = OtlpTracingLayer::new(move |log_record| {
            let scope_name = log_record.target();
            if let Ok(mut enc) = encoder_clone.lock() {
                enc.encode_log_record(&log_record, &resource_bytes, scope_name)
                    .expect("Failed to encode");
            }
        });

        let subscriber = tracing_subscriber::registry().with(layer);

        tracing::subscriber::with_default(subscriber, || {
            // Emit 5 events from same scope
            for i in 0..5 {
                tracing::info!(target: "same::scope", count = i, "Event");
            }
        });

        let otlp_bytes = encoder.lock().unwrap().flush();
        let decoded: ExportLogsServiceRequest = ProstMessage::decode(&otlp_bytes[..]).unwrap();

        // Verify: 1 ResourceLogs, 1 ScopeLogs, 5 LogRecords (all batched)
        assert_eq!(decoded.resource_logs.len(), 1);
        assert_eq!(decoded.resource_logs[0].scope_logs.len(), 1);
        assert_eq!(decoded.resource_logs[0].scope_logs[0].log_records.len(), 5);
        assert_eq!(
            decoded.resource_logs[0].scope_logs[0].scope.as_ref().unwrap().name,
            "same::scope"
        );

        println!("✓ Batching verified: 5 events with same target batched into 1 ScopeLogs");
    }

    #[test]
    fn test_tracing_scope_switching() {
        // This test verifies that when the target changes, the encoder
        // properly closes the previous scope and opens a new one

        let encoder = Arc::new(Mutex::new(StatefulOtlpEncoder::new(64 * 1024)));
        let encoder_clone = encoder.clone();

        let resource_bytes = vec![];

        let layer = OtlpTracingLayer::new(move |log_record| {
            let scope_name = log_record.target();
            if let Ok(mut enc) = encoder_clone.lock() {
                enc.encode_log_record(&log_record, &resource_bytes, scope_name)
                    .expect("Failed to encode");
            }
        });

        let subscriber = tracing_subscriber::registry().with(layer);

        tracing::subscriber::with_default(subscriber, || {
            tracing::info!(target: "scope1", "Message 1");
            tracing::info!(target: "scope2", "Message 2");
            tracing::info!(target: "scope1", "Message 3"); // Back to scope1
            tracing::info!(target: "scope2", "Message 4"); // Back to scope2
        });

        let otlp_bytes = encoder.lock().unwrap().flush();
        let decoded: ExportLogsServiceRequest = ProstMessage::decode(&otlp_bytes[..]).unwrap();

        // Should have 4 ScopeLogs because of the interleaving:
        // scope1 (msg1), scope2 (msg2), scope1 (msg3), scope2 (msg4)
        assert_eq!(decoded.resource_logs[0].scope_logs.len(), 4);
        
        let scope_names: Vec<String> = decoded.resource_logs[0].scope_logs
            .iter()
            .map(|sl| sl.scope.as_ref().unwrap().name.clone())
            .collect();
        
        assert_eq!(scope_names, vec!["scope1", "scope2", "scope1", "scope2"]);

        println!("✓ Scope switching verified: encoder closes and reopens scopes correctly");
    }
}
