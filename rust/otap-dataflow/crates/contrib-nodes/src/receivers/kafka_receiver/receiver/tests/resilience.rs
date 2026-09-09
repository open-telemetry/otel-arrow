// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Resilience tests: the receive loop survives adversarial input, transport
//! errors, and broker outages without crashing, stalling, or losing data.

use super::*;

// ---- Failure recovery ----

/// Scenario (failure recovery): a long run of fetch errors is injected
/// before a manual-commit receiver starts, held active long enough to
/// observe that the receiver cannot make progress, and only then cleared.
/// Guarantees: while the transport fault is active the receiver encounters
/// the failure and delivers no records (the fetch path keeps erroring), yet
/// the receive loop is non-fatal -- it keeps running rather than
/// terminating -- and once the fault clears the same loop reconnects and
/// delivers every record, proving the transport-error arm's
/// encounter-then-recover contract (not merely post-clear delivery).
#[tokio::test]
async fn transport_error_is_non_fatal_and_recovers() {
    const TOPIC: &str = "failure-transport-traces";
    const RECORDS: usize = 4;
    let group = "failure-transport-group";
    with_cluster(
        KafkaTestCluster::builder().topic(TOPIC),
        |cluster| async move {
            let producer = cluster.producer().build();
            let req = create_traces_with_spans();
            let mut bytes = vec![];
            req.encode(&mut bytes).expect("encode");

            for i in 0..RECORDS {
                let key = format!("rec-{i}");
                producer
                    .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                    .await
                    .expect("send record");
            }

            // Inject a LONG run of fetch errors (consumed one-per-request in
            // order) so the fault stays active across the whole observation
            // window below -- long enough that it cannot be silently
            // exhausted before the receiver would otherwise deliver. This is
            // what forces the receiver to actually encounter the transport
            // failure rather than sailing past a couple of quickly-retried
            // errors.
            let fetch_errors = vec![RDKafkaRespErr::RD_KAFKA_RESP_ERR_REQUEST_TIMED_OUT; 512];
            cluster.faults().fail_fetch(&fetch_errors);

            let cfg = manual_traces_config(cluster.bootstrap_servers(), group, TOPIC, 500, None);
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            // While the fault is active the receiver must encounter the
            // failure and make no progress: no record is delivered within a
            // generous window. This proves a failure was hit *before* the
            // fault is cleared, not just that delivery works afterward.
            assert!(
                receiver
                    .try_recv_pdata(Duration::from_secs(3))
                    .await
                    .is_none(),
                "receiver delivered a record while the fetch fault was active; \
                     the transport failure was not actually encountered",
            );

            // Clear the fault so fetches can succeed. librdkafka retries the
            // injected fetch errors internally, so rather than assert on the
            // (best-effort, mock-timing-dependent)
            // `receiver.kafka.transport.errors` counter,
            // the observable guarantee is that the loop survived the errors
            // (it did not terminate during the window above) and resumes
            // delivery once they clear.
            cluster.faults().clear_fetch_failures();

            // The same receive loop must now deliver every record -- it was
            // not killed by the sustained transport errors.
            for _ in 0..RECORDS {
                let pdata = receiver.recv_pdata().await;
                receiver.ack(pdata);
            }

            receiver.shutdown(Duration::from_secs(5));
            receiver.await_stopped().await;
        },
    )
    .await;
}

/// Scenario (failure recovery): every broker is taken down mid-stream after a
/// manual-commit receiver has consumed a first batch, then brought back up
/// and more records are produced.
/// Guarantees: a prolonged broker outage does not kill the receiver -- no
/// records are delivered while all brokers are down, and once the brokers
/// recover the same receiver reconnects and delivers the post-outage records
/// without loss, exercising librdkafka's reconnect/backoff behavior.
#[tokio::test]
async fn broker_outage_then_recovery_resumes_without_loss() {
    const TOPIC: &str = "failure-outage-traces";
    const PRE: usize = 3;
    const POST: usize = 3;
    let group = "failure-outage-group";
    with_cluster(
        KafkaTestCluster::builder().topic(TOPIC),
        |cluster| async move {
            let producer = cluster.producer().build();
            let req = create_traces_with_spans();
            let mut bytes = vec![];
            req.encode(&mut bytes).expect("encode");

            for i in 0..PRE {
                let key = format!("pre-{i}");
                producer
                    .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                    .await
                    .expect("send pre-outage record");
            }

            let cfg = manual_traces_config(cluster.bootstrap_servers(), group, TOPIC, 500, None);
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            // Consume and ack the first batch before the outage.
            for _ in 0..PRE {
                let pdata = receiver.recv_pdata().await;
                receiver.ack(pdata);
            }

            // Prolonged outage: every broker down. No new records must be
            // delivered while the brokers are unreachable.
            cluster.faults().all_brokers_down();
            assert!(
                receiver
                    .try_recv_pdata(Duration::from_secs(2))
                    .await
                    .is_none(),
                "no records should be delivered while all brokers are down",
            );

            // Recover: bring brokers back and produce more records.
            cluster.faults().all_brokers_up();
            for i in 0..POST {
                let key = format!("post-{i}");
                producer
                    .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                    .await
                    .expect("send post-outage record");
            }

            // The same receiver must reconnect and deliver every post-outage
            // record without loss.
            for _ in 0..POST {
                let pdata = receiver.recv_pdata().await;
                receiver.ack(pdata);
            }

            receiver.shutdown(Duration::from_secs(5));
            receiver.await_stopped().await;
        },
    )
    .await;
}

/// Scenario (failure recovery): a manual-commit receiver delivers and acks a
/// first batch, then hits a transient network interruption -- simulated by a
/// burst of injected fetch errors that block fetches for a bounded window --
/// which is later cleared and more records produced.
/// Guarantees: the receiver makes no progress while the interruption is
/// active (no records delivered), yet the loop is non-fatal and, once the
/// interruption clears, the same receiver recovers and delivers the
/// post-interruption records with no loss (its committed offset reaches the
/// full produced count). Models an intermittent network hiccup distinct from
/// the sustained full-outage case; a truly asymmetric (one-way) partition is
/// not modeled by the mock.
#[tokio::test]
async fn intermittent_network_interruption_recovers_without_loss() {
    const TOPIC: &str = "failure-netblip-traces";
    const PRE: usize = 3;
    const POST: usize = 3;
    let group = "failure-netblip-group";
    with_cluster(
        KafkaTestCluster::builder().topic(TOPIC),
        |cluster| async move {
            let producer = cluster.producer().build();
            let req = create_traces_with_spans();
            let mut bytes = vec![];
            req.encode(&mut bytes).expect("encode");

            for i in 0..PRE {
                let key = format!("pre-{i}");
                producer
                    .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                    .await
                    .expect("send pre-interruption record");
            }

            let cfg = manual_traces_config(cluster.bootstrap_servers(), group, TOPIC, 500, None);
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            // Consume and ack the first batch before the interruption.
            for _ in 0..PRE {
                let pdata = receiver.recv_pdata().await;
                receiver.ack(pdata);
            }

            // Transient network interruption: a long burst of fetch errors
            // that blocks fetches while active. Consumed one-per-request in
            // order, sized to outlast the observation window below.
            let fetch_errors = vec![RDKafkaRespErr::RD_KAFKA_RESP_ERR_REQUEST_TIMED_OUT; 512];
            cluster.faults().fail_fetch(&fetch_errors);

            // Produce during the interruption; nothing must be delivered while
            // it is active.
            for i in 0..POST {
                let key = format!("post-{i}");
                producer
                    .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                    .await
                    .expect("send post-interruption record");
            }
            assert!(
                receiver
                    .try_recv_pdata(Duration::from_secs(3))
                    .await
                    .is_none(),
                "no records should be delivered during the network interruption",
            );

            // Clear the interruption so fetches can succeed again.
            cluster.faults().clear_fetch_failures();

            // The same receiver must recover and deliver every post-interruption
            // record without loss.
            for _ in 0..POST {
                let pdata = receiver.recv_pdata().await;
                receiver.ack(pdata);
            }

            // No loss: the committed offset reaches the full produced count.
            let brokers = cluster.bootstrap_servers().to_string();
            let committed = poll_until(Duration::from_secs(5), Duration::from_millis(250), || {
                committed_offset(&brokers, group, TOPIC, 0)
                    .expect("kafka-test: committed-offset probe failed")
                    .is_some_and(|o| o >= (PRE + POST) as i64)
            })
            .await;
            assert!(
                committed,
                "after recovery the committed offset should reach the full \
                     produced count {}, got {:?}",
                PRE + POST,
                committed_offset(&brokers, group, TOPIC, 0)
                    .expect("kafka-test: committed-offset probe failed"),
            );

            receiver.shutdown(Duration::from_secs(5));
            receiver.await_stopped().await;
        },
    )
    .await;
}

/// Scenario (failure recovery): a manual-commit receiver runs against a
/// cluster where every broker has an injected per-request round-trip latency
/// (a slow-but-reachable broker, not an outage), then consumes and acks
/// every produced record.
/// Guarantees: bounded broker latency slows but does not corrupt offset
/// accounting -- every record is still delivered and the committed offset
/// advances to exactly the produced count with no loss and no commit errors
/// -- so a laggy broker cannot desynchronize the receiver's offset tracking.
#[tokio::test]
async fn broker_latency_does_not_corrupt_offset_accounting() {
    const TOPIC: &str = "failure-latency-traces";
    const RECORDS: i64 = 3;
    let group = "failure-latency-group";
    with_cluster(
        KafkaTestCluster::builder().topic(TOPIC),
        |cluster| async move {
            let producer = cluster.producer().build();
            let req = create_traces_with_spans();
            let mut bytes = vec![];
            req.encode(&mut bytes).expect("encode");
            for i in 0..RECORDS {
                let key = format!("rec-{i}");
                producer
                    .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                    .await
                    .expect("send record");
            }

            // Inject a bounded per-request latency on all brokers. The broker
            // stays reachable; requests merely take longer.
            cluster
                .faults()
                .round_trip_time(-1, Duration::from_millis(50));

            let cfg = manual_traces_config_no_timer(cluster.bootstrap_servers(), group, TOPIC);
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            // A larger per-record timeout absorbs the injected latency; every
            // record must still arrive.
            for _ in 0..RECORDS {
                let pdata = receiver
                    .try_recv_pdata(Duration::from_secs(10))
                    .await
                    .expect("record delivered despite broker latency");
                receiver.ack(pdata);
            }

            let brokers = cluster.bootstrap_servers().to_string();
            let committed = poll_until(Duration::from_secs(8), Duration::from_millis(200), || {
                committed_offset(&brokers, group, TOPIC, 0)
                    .expect("kafka-test: committed-offset probe failed")
                    .is_some_and(|o| o >= RECORDS)
            })
            .await;
            assert!(
                committed,
                "under bounded broker latency the committed offset must reach \
                     the full produced count {RECORDS} with no loss, got {:?}",
                committed_offset(&brokers, group, TOPIC, 0)
                    .expect("kafka-test: committed-offset probe failed"),
            );

            receiver.shutdown(Duration::from_secs(5));
            let terminal = receiver.await_terminal_state().await;
            assert_eq!(
                measurement_counter(
                    terminal.metrics(),
                    "receiver.kafka.offset_commits",
                    &[("outcome", "failure")],
                    "commits",
                ),
                0,
                "broker latency must not induce offset commit errors",
            );
        },
    )
    .await;
}

// ---- Security ----

/// Scenario (security): a receiver subscribed via a `^`-prefixed topic regex
/// (which lets a broker-supplied topic name reach the receiver's log sites)
/// receives a well-formed OTAP record carrying an adversarial header value
/// (control characters plus a large string on a configured extraction key),
/// followed by an undecodable OTAP record on the same topic.
/// Guarantees: the adversarial header value and topic name -- both of which
/// flow into `otel_*` log fields and into a resource attribute -- do not
/// crash or stall the receive loop: the good record is delivered with the
/// header extracted verbatim onto its resource, the poison record is counted
/// as a processing error rather than aborting the loop, and the receiver
/// still shuts down cleanly. This bounds the blast radius of adversarial
/// client-controlled topic/header values reaching telemetry.
#[tokio::test]
async fn adversarial_topic_and_header_values_do_not_stall_loop() {
    const TOPIC: &str = "sec-adversarial-traces";
    let group = "sec-adversarial-group";
    with_cluster(
        KafkaTestCluster::builder().topic(TOPIC),
        |cluster| async move {
            let producer = cluster.producer().build();
            let good = create_traces_with_spans_otap_bytes();
            let poison = b"not-a-valid-otap-arrow-payload".to_vec();

            // A control-char + oversized header value on the configured
            // extraction key. It is client-controlled and reaches both the
            // resource attribute and (on any parse failure) the log line.
            let adversarial_value = format!("acme\r\n\t\x1b[31m-{}", "Z".repeat(2048));

            // Good OTAP record with the adversarial header, then a poison
            // OTAP record, both on the regex-matched topic.
            producer
                .send_full(
                    SendRecord::new(TOPIC, &good)
                        .key(b"good")
                        .header("x-tenant-id", adversarial_value.as_bytes())
                        .header("MessageFormat", MSG_FORMAT_OTAP),
                )
                .await
                .expect("send good");
            producer
                .send_full(
                    SendRecord::new(TOPIC, &poison)
                        .key(b"poison")
                        .header("MessageFormat", MSG_FORMAT_OTAP),
                )
                .await
                .expect("send poison");

            // Configure a `^`-regex subscription (so the broker topic name
            // reaches the receiver) plus a header->resource-attribute
            // extraction for the adversarial header, OTAP encoding.
            let mut extraction = HashMap::new();
            let _ = extraction.insert(
                "x-tenant-id".to_string(),
                HeaderExtraction {
                    key: "tenant.id".to_string(),
                    value_type: AttributeValueType::String,
                },
            );
            let builder =
                KafkaReceiverConfigBuilder::new(cluster.bootstrap_servers(), group, "test-client")
                    .with_traces(
                        SignalConfig::new(vec!["^sec-adversarial-.*".to_string()])
                            .with_encoding(MessageFormat::OtapProto),
                    )
                    .with_commit(CommitConfig {
                        mode: ConfigCommitMode::Manual,
                        interval_ms: None,
                    })
                    .with_auto_offset_reset(AutoOffsetReset::Earliest)
                    .with_isolation_level(IsolationLevel::ReadUncommitted)
                    .with_resource_attrs_from_headers(extraction);
            let cfg = KafkaReceiverConfig::try_from(builder).expect("test config valid");
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            // The good record must be delivered despite the adversarial
            // header value; its value is extracted verbatim onto the resource.
            let mut pdata = receiver.recv_pdata().await;
            let result = otap_pdata_to_traces(&mut pdata);
            let mut found_tenant = false;
            for rs in &result.resource_spans {
                let resource = rs.resource.as_ref().expect("resource present");
                if let Some(kv) = resource.attributes.iter().find(|kv| kv.key == "tenant.id") {
                    if let Some(any_value::Value::StringValue(s)) =
                        kv.value.as_ref().and_then(|v| v.value.as_ref())
                    {
                        assert_eq!(
                            s, &adversarial_value,
                            "adversarial header value is extracted verbatim",
                        );
                        found_tenant = true;
                    }
                }
            }
            assert!(found_tenant, "the tenant.id attribute should be extracted");
            receiver.ack(pdata);

            // The poison record must not be forwarded (it is counted as an
            // error), and the loop keeps running.
            assert!(
                receiver
                    .try_recv_pdata(Duration::from_secs(2))
                    .await
                    .is_none(),
                "poison record must not be forwarded, and the loop must not stall",
            );

            receiver.shutdown(Duration::from_secs(5));
            let terminal = receiver.await_terminal_state().await;
            let decode_rejections = measurement_counter(
                terminal.metrics(),
                "receiver.kafka.rejections",
                &[
                    ("signal", "traces"),
                    ("error.type", "invalid_request"),
                    ("reason", "decode"),
                ],
                "messages",
            );
            assert!(
                decode_rejections >= 1,
                "the poison record must be counted as a decode rejection, got {decode_rejections}",
            );
        },
    )
    .await;
}
