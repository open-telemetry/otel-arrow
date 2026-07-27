// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the AttributesProcessor node.

use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry_macros::{metric_set, AttributeEnum};

/// Actions performed on attributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum ActionType {
    /// Attribute was renamed.
    Renamed,
    /// Attribute was deleted.
    Deleted,
    /// Attribute was inserted.
    Inserted,
    /// Attribute was upserted.
    Upserted,
    /// Attribute was updated.
    Updated,
    /// Attribute was hashed.
    Hashed,
}

/// Target payload domain where transforms were applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum TargetDomain {
    /// Transforms applied to signal-level payloads.
    Signal,
    /// Transforms applied to resource-level payloads.
    Resource,
    /// Transforms applied to scope-level payloads.
    Scope,
}

/// Metrics for the AttributesProcessor node.
#[metric_set(name = "processor.attributes")]
#[derive(Debug, Default, Clone)]
pub struct AttributesProcessorMetrics {
    /// Number of failed transform attempts.
    #[metric(unit = "{op}")]
    pub transform_failed: Counter<u64>,

    /// Total number of attribute entries modified, partitioned by action type.
    #[metric(unit = "{attr}")]
    pub modified_entries: Counter<u64, ActionType>,

    /// Number of times transforms were applied, partitioned by payload domain.
    #[metric(unit = "{apply}")]
    pub domains_applied: Counter<u64, TargetDomain>,
}
