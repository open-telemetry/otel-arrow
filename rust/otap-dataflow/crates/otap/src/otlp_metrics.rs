// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared OTLP receiver metric definitions.

use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry_macros::metric_set;

/// OTLP receiver metrics.
#[metric_set(name = "otlp.receiver.metrics")]
#[derive(Debug, Default, Clone)]
pub struct OtlpReceiverMetrics {
    /// Number of acks received from downstream (routed back to the caller).
    #[metric(unit = "{acks}")]
    pub acks_received: Counter<u64>,

    /// Number of nacks received from downstream (routed back to the caller).
    #[metric(unit = "{nacks}")]
    pub nacks_received: Counter<u64>,

    /// Number of invalid/expired acks/nacks.
    #[metric(unit = "{ack_or_nack}")]
    pub acks_nacks_invalid_or_expired: Counter<u64>,

    /// Number of OTLP RPCs started.
    #[metric(unit = "{requests}")]
    pub requests_started: Counter<u64>,

    /// Number of OTLP RPCs completed (success + nack).
    #[metric(unit = "{requests}")]
    pub requests_completed: Counter<u64>,

    /// Number of OTLP RPCs rejected before entering the pipeline (e.g. slot exhaustion).
    #[metric(unit = "{requests}")]
    pub rejected_requests: Counter<u64>,

    /// Number of OTLP RPCs rejected specifically because process-wide memory pressure was active.
    #[metric(unit = "{requests}")]
    pub refused_memory_pressure: Counter<u64>,

    /// Number of transport-level errors surfaced by tonic/server.
    #[metric(unit = "{errors}")]
    pub transport_errors: Counter<u64>,

    /// Total decompressed payload bytes for successfully received OTLP requests.
    #[metric(unit = "By")]
    pub request_bytes: Counter<u64>,
}
