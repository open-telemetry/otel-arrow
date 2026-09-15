// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared telemetry for database receivers.
//!
//! Measurement attributes are intentionally omitted. The two candidate
//! dimensions, `db.system.name` and the configured source identity, are either
//! constant for a registered receiver instance or operator-authored free text
//! whose cardinality cannot be bounded by a closed attribute enum. Both are
//! instead attached to the structured `otel_*` events emitted alongside these
//! counters, keeping the metric set bounded.

use otel_arrow_dfe_telemetry::instrument::Counter;
use otel_arrow_dfe_telemetry_macros::metric_set;

/// Lifecycle, delivery, and checkpoint metrics shared by database receivers.
#[metric_set(name = "receiver.database")]
#[derive(Clone, Debug, Default)]
pub struct DatabaseReceiverMetrics {
    /// Receiver starts.
    #[metric(unit = "{start}")]
    pub starts: Counter<u64>,
    /// Bounded page polls attempted against the database.
    #[metric(unit = "{poll}")]
    pub polls: Counter<u64>,
    /// Query executions that failed.
    #[metric(unit = "{failure}")]
    pub query_failures: Counter<u64>,
    /// Pages sent downstream.
    #[metric(unit = "{batch}")]
    pub batches_sent: Counter<u64>,
    /// Database rows sent downstream.
    #[metric(unit = "{row}")]
    pub rows_sent: Counter<u64>,
    /// Encoded OTLP bytes sent downstream.
    #[metric(unit = "By")]
    pub encoded_bytes_sent: Counter<u64>,
    /// Records whose source event time cannot fit OTLP's timestamp range.
    #[metric(unit = "{record}")]
    pub event_time_fallbacks: Counter<u64>,
    /// Downstream acknowledgements matching an in-flight page.
    #[metric(unit = "{ack}")]
    pub acks: Counter<u64>,
    /// Downstream negative acknowledgements matching an in-flight page.
    #[metric(unit = "{nack}")]
    pub nacks: Counter<u64>,
    /// Pages re-queried after a negative acknowledgement.
    #[metric(unit = "{replay}")]
    pub replays: Counter<u64>,
    /// ACK/NACK feedback discarded because it did not match the in-flight page.
    #[metric(unit = "{feedback}")]
    pub stale_feedback: Counter<u64>,
    /// Durable checkpoint commits.
    #[metric(unit = "{commit}")]
    pub checkpoint_commits: Counter<u64>,
    /// Durable checkpoint write failures.
    #[metric(unit = "{failure}")]
    pub checkpoint_failures: Counter<u64>,
    /// Stale checkpoint revisions that could not be removed.
    #[metric(unit = "{failure}")]
    pub checkpoint_cleanup_failures: Counter<u64>,
    /// Active database operations cancelled by a control message.
    #[metric(unit = "{cancellation}")]
    pub cancellations: Counter<u64>,
    /// Clean ingress drains.
    #[metric(unit = "{drain}")]
    pub drains: Counter<u64>,
    /// Immediate shutdowns.
    #[metric(unit = "{shutdown}")]
    pub shutdowns: Counter<u64>,
}
