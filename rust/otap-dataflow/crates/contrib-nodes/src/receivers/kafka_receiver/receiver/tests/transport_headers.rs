// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Transport-header and header-extraction integration tests.

use super::*;

/// Scenario (routing and payload correctness): an OTLP-proto trace record carries a Kafka header `x-tenant-id`
/// while the receiver is configured to map that header to a resource
/// attribute `tenant.id`.
/// Guarantees: every resource gains a `tenant.id` string attribute equal to
/// the header value, and no span-level `tenant.id` attribute is added.
#[tokio::test]
async fn test_kafka_receiver_traces_header_extraction() {
    const TOPIC: &str = "test-traces-headers";
    with_cluster(
        KafkaTestCluster::builder().topic(TOPIC),
        |cluster| async move {
            let producer = cluster.producer().build();

            // Build a trace request with real spans.
            let req = create_traces_with_spans();
            let mut payload_bytes = vec![];
            req.encode(&mut payload_bytes).expect("encode");

            // Configure extraction: map Kafka header "x-tenant-id" to a resource
            // attribute "tenant.id".
            let mut resource_attrs_from_headers = HashMap::new();
            let _ = resource_attrs_from_headers.insert(
                "x-tenant-id".to_string(),
                HeaderExtraction {
                    key: "tenant.id".to_string(),
                    value_type: AttributeValueType::String,
                },
            );

            let tenant_value = "acme-corp";

            // Send 3 messages, each with the same headers.
            for i in 0..3 {
                let key = format!("test-key-{i}");
                producer
                    .send_full(
                        SendRecord::new(TOPIC, &payload_bytes)
                            .key(key.as_bytes())
                            .header("x-tenant-id", tenant_value.as_bytes()),
                    )
                    .await
                    .expect("Failed to send message");
            }

            let cfg = auto_config(
                cluster.bootstrap_servers(),
                &[TOPIC],
                &[],
                &[],
                MessageFormat::OtlpProto,
                resource_attrs_from_headers,
            );
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            for i in 0..3 {
                let mut pdata = receiver.recv_pdata().await;
                let proto: OtlpProtoBytes = pdata
                    .take_payload()
                    .try_into_with_default()
                    .expect("to OtlpProtoBytes");
                let result =
                    ExportTraceServiceRequest::decode(proto.as_bytes()).expect("decode result");

                // Every resource should have the injected tenant.id attribute.
                for rs in &result.resource_spans {
                    let resource = rs.resource.as_ref().expect("should have resource");
                    let tenant_attr = resource
                        .attributes
                        .iter()
                        .find(|kv| kv.key == "tenant.id")
                        .unwrap_or_else(|| {
                            panic!("message {i}: resource missing tenant.id attribute")
                        });
                    let value = tenant_attr
                        .value
                        .as_ref()
                        .expect("should have value")
                        .value
                        .as_ref()
                        .expect("should have inner value");
                    assert!(
                        matches!(
                            value,
                            any_value::Value::StringValue(s) if s == tenant_value
                        ),
                        "message {i}: resource tenant.id should be '{tenant_value}'",
                    );

                    // Span attributes should NOT have tenant.id
                    for ss in &rs.scope_spans {
                        for span in &ss.spans {
                            assert!(
                                !span.attributes.iter().any(|kv| kv.key == "tenant.id"),
                                "message {i}: span '{}' should NOT have tenant.id attribute",
                                span.name,
                            );
                        }
                    }
                }
            }

            receiver.shutdown(Duration::from_secs(5));
            receiver.await_stopped().await;
        },
    )
    .await;
}

/// Scenario (routing and payload correctness): an OTAP-Arrow trace record carries a Kafka header `x-tenant-id`
/// plus the `MessageFormat` OTAP marker while the receiver maps that header
/// to a resource attribute `tenant.id`.
/// Guarantees: after decoding the OTAP payload back to OTLP, every resource
/// gains a `tenant.id` string attribute equal to the header value, and no
/// span-level `tenant.id` attribute is added.
#[tokio::test]
async fn test_kafka_receiver_traces_header_extraction_otap() {
    const TOPIC: &str = "test-traces-headers-otap";
    with_cluster(
        KafkaTestCluster::builder().topic(TOPIC),
        |cluster| async move {
            let producer = cluster.producer().build();

            // Build OTAP Arrow bytes from a real trace request with spans.
            let otap_bytes = create_traces_with_spans_otap_bytes();

            // Configure extraction: map Kafka header "x-tenant-id" to a resource
            // attribute "tenant.id".
            let mut resource_attrs_from_headers = HashMap::new();
            let _ = resource_attrs_from_headers.insert(
                "x-tenant-id".to_string(),
                HeaderExtraction {
                    key: "tenant.id".to_string(),
                    value_type: AttributeValueType::String,
                },
            );

            let tenant_value = "acme-corp";

            // Send 3 messages, each with the same headers and the OTAP
            // MessageFormat header so the receiver uses the OTAP path.
            for i in 0..3 {
                let key = format!("test-key-{i}");
                producer
                    .send_full(
                        SendRecord::new(TOPIC, &otap_bytes)
                            .key(key.as_bytes())
                            .header("x-tenant-id", tenant_value.as_bytes())
                            .header("MessageFormat", MSG_FORMAT_OTAP),
                    )
                    .await
                    .expect("Failed to send message");
            }

            let cfg = auto_config(
                cluster.bootstrap_servers(),
                &[TOPIC],
                &[],
                &[],
                MessageFormat::OtapProto,
                resource_attrs_from_headers,
            );
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            for i in 0..3 {
                let mut pdata = receiver.recv_pdata().await;

                // Convert OTAP result back to OTLP protobuf for assertions
                let result = otap_pdata_to_traces(&mut pdata);

                // Every resource should have the injected tenant.id attribute.
                for rs in &result.resource_spans {
                    let resource = rs.resource.as_ref().expect("should have resource");
                    let tenant_attr = resource
                        .attributes
                        .iter()
                        .find(|kv| kv.key == "tenant.id")
                        .unwrap_or_else(|| {
                            panic!("message {i}: resource missing tenant.id attribute")
                        });
                    let value = tenant_attr
                        .value
                        .as_ref()
                        .expect("should have value")
                        .value
                        .as_ref()
                        .expect("should have inner value");
                    assert!(
                        matches!(
                            value,
                            any_value::Value::StringValue(s) if s == tenant_value
                        ),
                        "message {i}: resource tenant.id should be '{tenant_value}'",
                    );

                    // Span attributes should NOT have tenant.id
                    for ss in &rs.scope_spans {
                        for span in &ss.spans {
                            assert!(
                                !span.attributes.iter().any(|kv| kv.key == "tenant.id"),
                                "message {i}: span '{}' should NOT have tenant.id attribute",
                                span.name,
                            );
                        }
                    }
                }
            }

            receiver.shutdown(Duration::from_secs(5));
            receiver.await_stopped().await;
        },
    )
    .await;
}

/// Scenario (routing and payload correctness): a raw Syslog Kafka record carries an
/// `x-tenant-id` header configured for resource-attribute extraction.
/// Guarantees: Syslog uses the shared extraction orchestration and emits the header
/// value as the `tenant.id` resource attribute on the decoded log.
#[tokio::test]
async fn test_kafka_receiver_logs_header_extraction_syslog() {
    const TOPIC: &str = "test-logs-headers-syslog";
    with_cluster(
        KafkaTestCluster::builder().topic(TOPIC),
        |cluster| async move {
            let producer = cluster.producer().build();
            let tenant_value = "acme-corp";
            let syslog = b"<34>1 2003-10-11T22:14:15.003Z host app - ID47 - Test message";

            producer
                .send_full(
                    SendRecord::new(TOPIC, syslog)
                        .header("x-tenant-id", tenant_value.as_bytes())
                        .header("MessageFormat", MSG_FORMAT_SYSLOG),
                )
                .await
                .expect("send Syslog message");

            let mut resource_attrs_from_headers = HashMap::new();
            let _ = resource_attrs_from_headers.insert(
                "x-tenant-id".to_string(),
                HeaderExtraction {
                    key: "tenant.id".to_string(),
                    value_type: AttributeValueType::String,
                },
            );

            let cfg = KafkaReceiverConfig::try_from(
                KafkaReceiverConfigBuilder::new(
                    cluster.bootstrap_servers(),
                    "test-group",
                    "test-client",
                )
                .with_logs(
                    SignalConfig::new(vec![TOPIC.to_string()]).with_encoding(MessageFormat::Syslog),
                )
                .with_commit(CommitConfig {
                    mode: ConfigCommitMode::Auto,
                    interval_ms: Some(1000),
                })
                .with_auto_offset_reset(AutoOffsetReset::Earliest)
                .with_isolation_level(IsolationLevel::ReadUncommitted)
                .with_resource_attrs_from_headers(resource_attrs_from_headers),
            )
            .expect("test config valid");
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            let mut pdata = receiver.recv_pdata().await;
            let otlp: OtlpProtoBytes = pdata
                .take_payload()
                .try_into_with_default()
                .expect("convert Syslog Arrow logs to OTLP");
            let result =
                ExportLogsServiceRequest::decode(otlp.as_bytes()).expect("decode OTLP logs");
            let resource = result.resource_logs[0]
                .resource
                .as_ref()
                .expect("log should have a resource");
            let tenant_attr = resource
                .attributes
                .iter()
                .find(|attribute| attribute.key == "tenant.id")
                .expect("resource should contain tenant.id");

            assert!(matches!(
                tenant_attr
                    .value
                    .as_ref()
                    .and_then(|value| value.value.as_ref()),
                Some(any_value::Value::StringValue(value)) if value == tenant_value
            ));

            receiver.shutdown(Duration::from_secs(5));
            receiver.await_stopped().await;
        },
    )
    .await;
}

/// Scenario (routing and payload correctness): a capture policy captures `X-Tenant-Id` (stored as `tenant_id`)
/// and `X-Request-Id` (default lowercased name) but not `X-Unrelated`.
/// Guarantees: exactly the two matching Kafka headers are captured into the
/// OtapPdata transport headers with their configured store-names and
/// preserved wire names, and the unmatched header is dropped.
#[tokio::test]
async fn test_kafka_receiver_capture_policy_captures_headers() {
    const TOPIC: &str = "test-capture-policy";
    with_cluster(
        KafkaTestCluster::builder().topic(TOPIC),
        |cluster| async move {
            let producer = cluster.producer().build();

            let req = create_traces_with_spans();
            let mut payload_bytes = vec![];
            req.encode(&mut payload_bytes).expect("encode");

            // Send a message with Kafka headers.
            producer
                .send_full(
                    SendRecord::new(TOPIC, &payload_bytes)
                        .key(b"key-1")
                        .header("X-Tenant-Id", b"acme-corp")
                        .header("X-Request-Id", b"req-12345")
                        .header("X-Unrelated", b"ignored"),
                )
                .await
                .expect("Failed to send message");

            // Set up a capture policy that captures X-Tenant-Id and X-Request-Id
            // but not X-Unrelated.
            let capture_policy = HeaderCapturePolicy::new(
                CaptureDefaults::default(),
                vec![
                    CaptureRule {
                        match_names: vec!["X-Tenant-Id".to_string()],
                        store_as: Some("tenant_id".to_string()),
                        sensitive: false,
                        value_kind: None,
                    },
                    CaptureRule {
                        match_names: vec!["X-Request-Id".to_string()],
                        store_as: None, // defaults to lowercased wire name
                        sensitive: false,
                        value_kind: None,
                    },
                ],
            );

            let cfg = auto_config(
                cluster.bootstrap_servers(),
                &[TOPIC],
                &[],
                &[],
                MessageFormat::OtlpProto,
                HashMap::new(),
            );
            let mut receiver =
                KafkaReceiverHarness::start_with_capture(&cluster, cfg, Some(capture_policy));

            let pdata = receiver.recv_pdata().await;

            // Verify transport headers were captured.
            let transport_headers = pdata
                .transport_headers()
                .expect("transport_headers should be set");

            // Two headers should be captured (X-Tenant-Id and X-Request-Id).
            assert_eq!(
                transport_headers.len(),
                2,
                "expected 2 captured headers, got {}",
                transport_headers.len()
            );

            // Check X-Tenant-Id was stored as "tenant_id".
            let tenant_headers: Vec<_> = transport_headers.find_by_name("tenant_id").collect();
            assert_eq!(tenant_headers.len(), 1, "expected one tenant_id header");
            assert_eq!(
                tenant_headers[0].value_as_str(),
                Some("acme-corp"),
                "tenant_id value mismatch"
            );
            assert_eq!(
                tenant_headers[0].wire_name, "X-Tenant-Id",
                "wire_name should be preserved"
            );

            // Check X-Request-Id was stored as "x-request-id" (lowercased).
            let request_headers: Vec<_> = transport_headers.find_by_name("x-request-id").collect();
            assert_eq!(request_headers.len(), 1, "expected one x-request-id header");
            assert_eq!(
                request_headers[0].value_as_str(),
                Some("req-12345"),
                "x-request-id value mismatch"
            );

            // X-Unrelated should NOT be captured (not in the policy).
            let unrelated: Vec<_> = transport_headers.find_by_name("x-unrelated").collect();
            assert!(unrelated.is_empty(), "X-Unrelated should not be captured");

            receiver.shutdown(Duration::from_secs(5));
            receiver.await_stopped().await;
        },
    )
    .await;
}

/// Scenario (routing and payload correctness): a record carries a Kafka header but the receiver is started
/// without any capture policy.
/// Guarantees: transport headers are left unset on the OtapPdata context
/// (existing behavior is preserved when capture is not configured).
#[tokio::test]
async fn test_kafka_receiver_no_capture_policy_no_transport_headers() {
    const TOPIC: &str = "test-no-capture-policy";
    with_cluster(
        KafkaTestCluster::builder().topic(TOPIC),
        |cluster| async move {
            let producer = cluster.producer().build();

            let req = create_traces_with_spans();
            let mut payload_bytes = vec![];
            req.encode(&mut payload_bytes).expect("encode");

            // Send a message with headers, but without a capture policy.
            producer
                .send_full(
                    SendRecord::new(TOPIC, &payload_bytes)
                        .key(b"key-1")
                        .header("X-Tenant-Id", b"acme-corp"),
                )
                .await
                .expect("Failed to send message");

            // No capture policy set on the receiver.
            let cfg = auto_config(
                cluster.bootstrap_servers(),
                &[TOPIC],
                &[],
                &[],
                MessageFormat::OtlpProto,
                HashMap::new(),
            );
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            let pdata = receiver.recv_pdata().await;

            // Transport headers should NOT be set when no capture policy is configured.
            assert!(
                pdata.transport_headers().is_none(),
                "transport_headers should be None when no capture policy is configured"
            );

            receiver.shutdown(Duration::from_secs(5));
            receiver.await_stopped().await;
        },
    )
    .await;
}

/// Scenario (routing and payload correctness): a record carries `X-Tenant-Id` (captured to a transport header)
/// and `x-env` (mapped to a resource attribute) while both the capture policy
/// and resource-attribute-from-header extraction are configured.
/// Guarantees: the transport header and the injected resource attribute are
/// produced independently and simultaneously from the same record.
#[tokio::test]
async fn test_kafka_receiver_capture_policy_coexists_with_resource_attrs_from_headers() {
    const TOPIC: &str = "test-capture-and-extract";
    with_cluster(
        KafkaTestCluster::builder().topic(TOPIC),
        |cluster| async move {
            let producer = cluster.producer().build();

            let req = create_traces_with_spans();
            let mut payload_bytes = vec![];
            req.encode(&mut payload_bytes).expect("encode");

            // Send a message with headers for both mechanisms.
            producer
                .send_full(
                    SendRecord::new(TOPIC, &payload_bytes)
                        .key(b"key-1")
                        .header("X-Tenant-Id", b"acme-corp")
                        .header("x-env", b"production"),
                )
                .await
                .expect("Failed to send message");

            // Configure resource_attrs_from_headers: x-env -> deployment.environment resource attribute
            let mut resource_attrs_from_headers = HashMap::new();
            let _ = resource_attrs_from_headers.insert(
                "x-env".to_string(),
                HeaderExtraction {
                    key: "deployment.environment".to_string(),
                    value_type: AttributeValueType::String,
                },
            );

            // Configure capture policy: X-Tenant-Id -> transport header "tenant_id"
            let capture_policy = HeaderCapturePolicy::new(
                CaptureDefaults::default(),
                vec![CaptureRule {
                    match_names: vec!["X-Tenant-Id".to_string()],
                    store_as: Some("tenant_id".to_string()),
                    sensitive: false,
                    value_kind: None,
                }],
            );

            let cfg = auto_config(
                cluster.bootstrap_servers(),
                &[TOPIC],
                &[],
                &[],
                MessageFormat::OtlpProto,
                resource_attrs_from_headers,
            );
            let mut receiver =
                KafkaReceiverHarness::start_with_capture(&cluster, cfg, Some(capture_policy));

            let mut pdata = receiver.recv_pdata().await;

            // 1. Verify transport headers were captured (capture policy).
            let transport_headers = pdata
                .transport_headers()
                .expect("transport_headers should be set");
            let tenant_headers: Vec<_> = transport_headers.find_by_name("tenant_id").collect();
            assert_eq!(tenant_headers.len(), 1);
            assert_eq!(tenant_headers[0].value_as_str(), Some("acme-corp"));

            // 2. Verify resource attributes were injected (resource_attrs_from_headers).
            let proto: OtlpProtoBytes = pdata
                .take_payload()
                .try_into_with_default()
                .expect("to OtlpProtoBytes");
            let result =
                ExportTraceServiceRequest::decode(proto.as_bytes()).expect("decode result");
            for rs in &result.resource_spans {
                let resource = rs.resource.as_ref().expect("should have resource");
                let env_attr = resource
                    .attributes
                    .iter()
                    .find(|kv| kv.key == "deployment.environment")
                    .expect("resource should have deployment.environment attribute");
                let value = env_attr
                    .value
                    .as_ref()
                    .expect("should have value")
                    .value
                    .as_ref()
                    .expect("should have inner value");
                assert!(
                    matches!(
                        value,
                        any_value::Value::StringValue(s) if s == "production"
                    ),
                    "deployment.environment should be 'production'"
                );
            }

            receiver.shutdown(Duration::from_secs(5));
            receiver.await_stopped().await;
        },
    )
    .await;
}

/// Scenario (routing and payload correctness): a capture policy is applied to an OTAP-Arrow record that also
/// carries the `MessageFormat` OTAP marker header.
/// Guarantees: the matching `X-Tenant-Id` header is captured as a transport
/// header even for OTAP payloads, while the `MessageFormat` control header is
/// not captured.
#[tokio::test]
async fn test_kafka_receiver_capture_policy_otap_format() {
    const TOPIC: &str = "test-capture-policy-otap";
    with_cluster(
        KafkaTestCluster::builder().topic(TOPIC),
        |cluster| async move {
            let producer = cluster.producer().build();

            let otap_bytes = create_traces_with_spans_otap_bytes();

            producer
                .send_full(
                    SendRecord::new(TOPIC, &otap_bytes)
                        .key(b"key-1")
                        .header("X-Tenant-Id", b"acme-corp")
                        .header("MessageFormat", MSG_FORMAT_OTAP),
                )
                .await
                .expect("Failed to send message");

            let capture_policy = HeaderCapturePolicy::new(
                CaptureDefaults::default(),
                vec![CaptureRule {
                    match_names: vec!["X-Tenant-Id".to_string()],
                    store_as: Some("tenant_id".to_string()),
                    sensitive: false,
                    value_kind: None,
                }],
            );

            let cfg = auto_config(
                cluster.bootstrap_servers(),
                &[TOPIC],
                &[],
                &[],
                MessageFormat::OtapProto,
                HashMap::new(),
            );
            let mut receiver =
                KafkaReceiverHarness::start_with_capture(&cluster, cfg, Some(capture_policy));

            let pdata = receiver.recv_pdata().await;

            // Verify transport headers were captured for OTAP format.
            let transport_headers = pdata
                .transport_headers()
                .expect("transport_headers should be set for OTAP messages");
            let tenant_headers: Vec<_> = transport_headers.find_by_name("tenant_id").collect();
            assert_eq!(tenant_headers.len(), 1);
            assert_eq!(tenant_headers[0].value_as_str(), Some("acme-corp"));

            // The MessageFormat header should NOT be captured (not in policy).
            let format_headers: Vec<_> = transport_headers.find_by_name("messageformat").collect();
            assert!(
                format_headers.is_empty(),
                "MessageFormat header should not be captured"
            );

            receiver.shutdown(Duration::from_secs(5));
            receiver.await_stopped().await;
        },
    )
    .await;
}
