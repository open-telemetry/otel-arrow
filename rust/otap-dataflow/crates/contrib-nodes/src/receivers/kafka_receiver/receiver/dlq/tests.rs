// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Unit tests for the broker-independent DLQ manager logic: reason/signal
//! string mapping, completion construction, capture gating, topic resolution,
//! and the in-flight admission bound.

use super::*;
use crate::receivers::kafka_receiver::config::{
    CommitConfig, CommitMode, DlqCapture, DlqConfig, DlqPerSignalTopics, KafkaReceiverConfig,
    KafkaReceiverConfigBuilder, SignalConfig,
};

/// Build a manual-commit config with a DLQ block for testing.
fn config_with_dlq(dlq: DlqConfig) -> KafkaReceiverConfig {
    KafkaReceiverConfig::try_from(
        KafkaReceiverConfigBuilder::new("b:9092", "g", "c")
            .with_traces(SignalConfig::new(vec!["otlp_spans".to_string()]))
            .with_metrics(SignalConfig::new(vec!["otlp_metrics".to_string()]))
            .with_commit(CommitConfig {
                mode: CommitMode::Manual,
                interval_ms: None,
            })
            .with_dlq(dlq),
    )
    .expect("test DLQ config valid")
}

/// Build a `DlqManager` (no broker I/O happens at construction; rdkafka clients
/// connect lazily on first use).
fn manager(dlq: DlqConfig) -> DlqManager {
    DlqManager::new(&config_with_dlq(dlq))
        .expect("manager builds")
        .expect("dlq enabled")
}

fn source() -> DlqSource {
    DlqSource {
        topic: Arc::from("otlp_spans"),
        partition: 1,
        offset: 7,
    }
}

fn meta(reason: DlqReason, signal: Option<SignalType>) -> JobMeta {
    JobMeta {
        reason,
        source: source(),
        signal,
        error: "boom".to_string(),
        topic: "otel_dlq".to_string(),
    }
}

/// Scenario: each `DlqReason` renders its wire string for the `dlq.reason` header.
/// Guarantees: the strings match the documented capture-category names.
#[test]
fn reason_as_str_maps_each_variant() {
    assert_eq!(DlqReason::Decode.as_str(), "decode");
    assert_eq!(DlqReason::UnknownTopic.as_str(), "unknown_topic");
    assert_eq!(DlqReason::PermanentNack.as_str(), "permanent_nack");
}

/// Scenario: the signal string covers every signal plus the unknown case.
/// Guarantees: `dlq.signal` is well-defined for signal-less categories.
#[test]
fn signal_str_maps_each_variant() {
    assert_eq!(signal_str(Some(SignalType::Traces)), "traces");
    assert_eq!(signal_str(Some(SignalType::Metrics)), "metrics");
    assert_eq!(signal_str(Some(SignalType::Logs)), "logs");
    assert_eq!(signal_str(None), "unknown");
}

/// Scenario: a loss completion is built from a job's metadata.
/// Guarantees: it carries the source/signal/reason and marks not-produced,
/// permanent-failure so the receiver records `dlq.loss` and advances.
#[test]
fn completion_loss_carries_meta() {
    let completion = DlqCompletion::loss(meta(DlqReason::Decode, Some(SignalType::Traces)));
    assert!(!completion.produced);
    assert!(completion.permanent_failure);
    assert_eq!(completion.reason, DlqReason::Decode);
    assert_eq!(completion.signal, Some(SignalType::Traces));
    assert_eq!(completion.source.offset, 7);
}

/// Scenario: a producer `Produced` outcome maps to a completion.
/// Guarantees: `produced` is set and no permanent-failure flag.
#[test]
fn completion_from_produced_outcome() {
    let completion =
        DlqCompletion::from_outcome(meta(DlqReason::Decode, None), DlqSendOutcome::Produced);
    assert!(completion.produced);
    assert!(!completion.permanent_failure);
}

/// Scenario: a producer `Failed` outcome maps to a completion, preserving the
/// permanent flag for logging.
/// Guarantees: not-produced, and `permanent_failure` mirrors the outcome.
#[test]
fn completion_from_failed_outcome_preserves_permanent_flag() {
    let permanent = DlqCompletion::from_outcome(
        meta(DlqReason::PermanentNack, None),
        DlqSendOutcome::Failed { permanent: true },
    );
    assert!(!permanent.produced);
    assert!(permanent.permanent_failure);

    let transient = DlqCompletion::from_outcome(
        meta(DlqReason::PermanentNack, None),
        DlqSendOutcome::Failed { permanent: false },
    );
    assert!(!transient.produced);
    assert!(!transient.permanent_failure);
}

/// Scenario: `captures` reflects the configured capture set.
/// Guarantees: only the categories in `capture` return true.
#[test]
fn captures_reflects_config() {
    let mgr = manager(DlqConfig {
        topic: Some("otel_dlq".to_string()),
        per_signal: None,
        capture: vec![DlqCapture::Decode, DlqCapture::PermanentNack],
        connection: None,
    });
    assert!(mgr.captures(DlqReason::Decode));
    assert!(!mgr.captures(DlqReason::UnknownTopic));
    assert!(mgr.captures(DlqReason::PermanentNack));
}

/// Scenario: topic resolution prefers a per-signal override, falls back to the
/// global topic, and for a signal-less category uses any configured topic.
/// Guarantees: `topic_for` resolves the documented precedence.
#[test]
fn topic_for_precedence() {
    let mgr = manager(DlqConfig {
        topic: Some("global_dlq".to_string()),
        per_signal: Some(DlqPerSignalTopics {
            traces: Some("traces_dlq".to_string()),
            metrics: None,
            logs: None,
        }),
        capture: vec![DlqCapture::Decode, DlqCapture::UnknownTopic],
        connection: None,
    });
    // Per-signal override wins.
    assert_eq!(
        mgr.topic_for(Some(SignalType::Traces)).as_deref(),
        Some("traces_dlq")
    );
    // Falls back to the global topic.
    assert_eq!(
        mgr.topic_for(Some(SignalType::Metrics)).as_deref(),
        Some("global_dlq")
    );
    // Signal-less category resolves to some configured topic.
    assert!(mgr.topic_for(None).is_some());
}

/// Scenario: the in-flight set is saturated to `DLQ_MAX_IN_FLIGHT` and a new job
/// is submitted.
/// Guarantees: `admit` returns an immediate loss completion (no queue) so the
/// caller records `dlq.loss` and advances the source offset.
#[test]
fn admit_returns_loss_when_in_flight_full() {
    let mut mgr = manager(DlqConfig {
        topic: Some("otel_dlq".to_string()),
        per_signal: None,
        capture: vec![DlqCapture::Decode],
        connection: None,
    });

    // Saturate the in-flight set with completions that never resolve during the
    // test (they are never polled), simulating max outstanding deliveries.
    for _ in 0..DLQ_MAX_IN_FLIGHT {
        mgr.in_flight
            .push(Box::pin(std::future::pending::<DlqCompletion>()));
    }
    assert_eq!(mgr.in_flight.len(), DLQ_MAX_IN_FLIGHT);

    let outcome = mgr.submit_inline(
        DlqReason::Decode,
        source(),
        Some(SignalType::Traces),
        "boom".to_string(),
        b"payload".to_vec(),
        None,
    );
    let completion = outcome.expect("in-flight full yields an immediate loss");
    assert!(!completion.produced);
    assert!(completion.permanent_failure);
    assert_eq!(completion.reason, DlqReason::Decode);
    // No slot was consumed by the rejected job.
    assert_eq!(mgr.in_flight.len(), DLQ_MAX_IN_FLIGHT);
}

/// Scenario: a job is submitted while a slot is free.
/// Guarantees: `admit` accepts it (returns `None`) and occupies one slot,
/// deferring the outcome to the completion future.
#[test]
fn admit_accepts_when_slot_free() {
    let mut mgr = manager(DlqConfig {
        topic: Some("otel_dlq".to_string()),
        per_signal: None,
        capture: vec![DlqCapture::Decode],
        connection: None,
    });
    assert_eq!(mgr.in_flight.len(), 0);
    let outcome = mgr.submit_inline(
        DlqReason::Decode,
        source(),
        Some(SignalType::Traces),
        "boom".to_string(),
        b"payload".to_vec(),
        None,
    );
    assert!(outcome.is_none(), "a free slot defers the outcome");
    assert_eq!(mgr.in_flight.len(), 1);
}
