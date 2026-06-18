// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the Kafka Receiver node.

use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry_macros::metric_set;

/// Metrics for the Kafka Receiver.
#[metric_set(name = "receiver.kafka")]
#[derive(Debug, Default, Clone)]
pub struct KafkaReceiverMetrics {
    // ── Message Counters ────────────────────────────────────
    /// Total messages received from Kafka across all signal types
    #[metric(unit = "{msg}")]
    pub messages_received: Counter<u64>,
    /// Total payload bytes consumed from Kafka
    #[metric(unit = "{byte}")]
    pub bytes_received: Counter<u64>,
    /// Number of log messages received from the kafka broker
    #[metric(unit = "{msg}")]
    pub log_msgs_received: Counter<u64>,
    /// Number of metric messages received from the kafka broker
    #[metric(unit = "{msg}")]
    pub metric_msgs_received: Counter<u64>,
    /// Number of trace messages received from the kafka broker
    #[metric(unit = "{msg}")]
    pub trace_msgs_received: Counter<u64>,

    // ── Pipeline Feedback ───────────────────────────────────
    /// Number of acks received from downstream
    #[metric(unit = "{ack}")]
    pub acks_received: Counter<u64>,
    /// Number of nacks received from downstream
    #[metric(unit = "{nack}")]
    pub nacks_received: Counter<u64>,

    // ── Error Tracking ──────────────────────────────────────
    /// Number of messages that failed processing and were skipped
    #[metric(unit = "{msg}")]
    pub processing_errors: Counter<u64>,
    /// Trace messages that failed to unmarshal
    #[metric(unit = "{msg}")]
    pub unmarshal_failed_traces: Counter<u64>,
    /// Metric messages that failed to unmarshal
    #[metric(unit = "{msg}")]
    pub unmarshal_failed_metrics: Counter<u64>,
    /// Log messages that failed to unmarshal
    #[metric(unit = "{msg}")]
    pub unmarshal_failed_logs: Counter<u64>,
    /// Messages with empty payload
    #[metric(unit = "{msg}")]
    pub empty_payloads: Counter<u64>,
    /// Messages from topics that don't match any configured signal
    #[metric(unit = "{error}")]
    pub unknown_topic_errors: Counter<u64>,
    /// Number of Kafka transport errors encountered (non-fatal)
    #[metric(unit = "{error}")]
    pub transport_errors: Counter<u64>,

    // ── Consumer Health ─────────────────────────────────────
    /// Number of offset commits performed
    #[metric(unit = "{commit}")]
    pub offset_commits: Counter<u64>,
    /// Number of offset commit failures
    #[metric(unit = "{error}")]
    pub offset_commit_errors: Counter<u64>,
    /// Messages skipped due to idempotency check (duplicate detection)
    #[metric(unit = "{msg}")]
    pub idempotent_skips: Counter<u64>,
    /// Messages dropped because the topic ID space was exhausted (overflow guard)
    #[metric(unit = "{msg}")]
    pub topic_id_exhausted: Counter<u64>,
}
