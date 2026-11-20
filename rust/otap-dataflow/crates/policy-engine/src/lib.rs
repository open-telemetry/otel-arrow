// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Pure-Rust columnar telemetry policy evaluation engine prototype for OTAP / Arrow Dataflow (`open-telemetry/otel-arrow#4726`).

#![forbid(unsafe_code)]

mod engine;
pub mod model;
pub(crate) mod otap_ops;

#[cfg(test)]
mod tests;

use otel_arrow_dfe_pdata::otap::OtapArrowRecords;

pub use crate::engine::{CompileOutput, EvaluationScratch, PolicyEngine, RowSelection};
pub use crate::model::{
    Action, AttributePath, LogAdd, LogAttributeSelector, LogFieldSelector, LogFilterPolicy,
    LogMatcher, LogRecordField, LogRedact, LogRemove, LogRename, LogTargetSelector,
    LogTransformPolicy, MetricFilterPolicy, NumericValue, Policy, PolicyCompileError,
    PolicyDefinition, PolicyError, Predicate, ScalarValue, ScopeField, TraceSamplingPolicy,
};

/// Result of evaluating a policy set against an `OtapArrowRecords` batch.
#[derive(Debug)]
pub struct EvaluationOutput {
    /// Resulting batch (`None` if all rows were dropped).
    pub records: Option<OtapArrowRecords>,
    /// Number of rows retained.
    pub kept_rows: u64,
    /// Number of rows dropped.
    pub dropped_rows: u64,
    /// Number of transform operations applied.
    pub transform_hits: u64,
}
