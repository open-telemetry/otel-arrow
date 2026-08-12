// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the Kafka Receiver node.

use otap_df_telemetry::instrument::{Counter, Gauge};
use otap_df_telemetry_macros::metric_set;

/// Metrics for the Kafka Receiver.
#[metric_set(name = "receiver.kafka")]
#[derive(Debug, Default, Clone)]
pub struct KafkaReceiverMetrics {
    // -- Message Counters ------------------------------------
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

    // -- Pipeline Feedback -----------------------------------
    /// Number of acks received from downstream
    #[metric(unit = "{ack}")]
    pub acks_received: Counter<u64>,
    /// Number of nacks received from downstream
    #[metric(unit = "{nack}")]
    pub nacks_received: Counter<u64>,

    // -- Error Tracking --------------------------------------
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

    // -- Consumer Health -------------------------------------
    /// Number of offset commits acknowledged by the broker.
    ///
    /// Populated from the consumer commit callback (not at commit-issue time),
    /// so it counts commits the broker actually accepted -- covering both the
    /// receiver's asynchronous steady-state commits and the synchronous
    /// pre-rebalance commit-before-revoke.
    #[metric(unit = "{commit}")]
    pub offset_commits: Counter<u64>,
    /// Number of offset commits the broker rejected.
    ///
    /// Populated from the consumer commit callback (see [`offset_commits`]).
    #[metric(unit = "{error}")]
    pub offset_commit_errors: Counter<u64>,
    /// Messages skipped due to idempotency check (duplicate detection)
    #[metric(unit = "{msg}")]
    pub idempotent_skips: Counter<u64>,
    /// Messages dropped because the topic ID space was exhausted (overflow guard)
    #[metric(unit = "{msg}")]
    pub topic_id_exhausted: Counter<u64>,

    // -- Consumer-group Rebalances ---------------------------
    /// Total number of consumer-group rebalance (assign) events observed by
    /// this consumer.
    ///
    /// Manual commit mode only (`commit.mode: manual`); stays `0` under
    /// auto-commit.
    #[metric(unit = "{rebalance}")]
    pub rebalances_total: Counter<u64>,
    /// Current number of partitions owned by this consumer.
    ///
    /// A point-in-time gauge reflecting the size of the current assignment,
    /// refreshed after each rebalance. Contrast with [`partition_assignments`]
    /// (cumulative acquisitions) and [`partition_revocations`] (cumulative
    /// revocations).
    ///
    /// Manual commit mode only (`commit.mode: manual`); stays `0` under
    /// auto-commit.
    #[metric(unit = "{partition}")]
    pub partitions_assigned: Gauge<u64>,
    /// Cumulative count of partitions newly acquired by this consumer across
    /// rebalances (retained partitions are not re-counted).
    ///
    /// Manual commit mode only (`commit.mode: manual`); stays `0` under
    /// auto-commit.
    #[metric(unit = "{partition}")]
    pub partition_assignments: Counter<u64>,
    /// Cumulative count of genuinely-owned partitions revoked from this consumer
    /// across rebalances (a revoke reported for a partition this consumer did
    /// not own is not counted).
    ///
    /// Manual commit mode only (`commit.mode: manual`); stays `0` under
    /// auto-commit.
    #[metric(unit = "{partition}")]
    pub partition_revocations: Counter<u64>,
    /// Mean consumer-group lag across owned partitions, in records.
    ///
    /// Per partition, `max(0, high_watermark - broker_committed_offset)`, using
    /// the offsets Kafka has acknowledged for this consumer group. It reflects
    /// how far behind the group's committed position is, not the receiver's local
    /// progress.
    ///
    /// The mean covers every owned partition: a refresh that cannot measure all
    /// of them (a failed broker read, or an owned partition with no committed
    /// offset yet) is abandoned and the previous value is retained rather than
    /// computing the mean from a subset. When ownership drops to zero the gauge
    /// is reset to `0`.
    ///
    /// Manual commit mode only and opt-in via `lag_refresh_interval_ms`;
    /// otherwise stays `0`.
    #[metric(unit = "{message}")]
    pub consumer_lag: Gauge<f64>,
    /// Offset commit failures during pre-rebalance revoke.
    ///
    /// Manual commit mode only (`commit.mode: manual`); stays `0` under
    /// auto-commit.
    #[metric(unit = "{error}")]
    pub rebalance_commit_errors: Counter<u64>,
    /// Acks/nacks skipped because the partition was no longer assigned
    #[metric(unit = "{ack}")]
    pub acks_for_revoked_partition: Counter<u64>,
}
