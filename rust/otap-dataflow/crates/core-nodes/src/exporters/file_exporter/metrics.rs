// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bounded telemetry instruments for file exporter outcomes and filesystem operations.
//!
//! Metrics use closed signal and operation attributes and deliberately omit destination paths so
//! component observability cannot introduce unbounded cardinality or expose sensitive locations.

use otap_df_config::SignalType;
use otap_df_telemetry::common_attributes::SignalAttributes;
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry_macros::{AttributeEnum, attribute_set, metric_set};

/// Bounded file operation associated with an I/O failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum FileOperation {
    /// File acquisition, directory creation, open, probe, or tail validation.
    Open,
    /// Frame write or flush.
    Write,
    /// Data synchronization.
    Sync,
    /// Failed-write rollback.
    Rollback,
}

/// Signal and operation dimensions for file failures.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct SignalOperationAttributes {
    /// Signal associated with the file writer.
    pub signal: SignalType,
    /// Failed file operation.
    pub operation: FileOperation,
}

/// Successful writes and append-tail recovery, partitioned by signal.
#[metric_set(name = "exporter.file", measurement_attributes = SignalAttributes)]
#[derive(Debug, Default, Clone)]
pub struct FileSignalMetrics {
    /// Signal items in frames successfully written before ACK routing.
    #[metric(unit = "{item}")]
    pub items: Counter<u64>,
    /// Successfully written frame bytes, including newline delimiters.
    #[metric(unit = "By")]
    pub bytes: Counter<u64>,
    /// Append-mode partial tails successfully repaired.
    #[metric(unit = "{recovery}")]
    pub tail_recoveries: Counter<u64>,
    /// Bytes removed by successful append-tail recovery.
    #[metric(unit = "By")]
    pub tail_recovered_bytes: Counter<u64>,
}

/// File I/O failures, partitioned by signal and bounded operation.
#[metric_set(
    name = "exporter.file",
    measurement_attributes = SignalOperationAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct FileFailureMetrics {
    /// Open, write, sync, or rollback failures.
    #[metric(unit = "{failure}")]
    pub write_failures: Counter<u64>,
}
