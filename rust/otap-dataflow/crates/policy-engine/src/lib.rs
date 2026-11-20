// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Pure-Rust columnar telemetry policy evaluation engine prototype for OTAP / Arrow Dataflow (`open-telemetry/otel-arrow#4726`).

#![forbid(unsafe_code)]
#![allow(
    unused_qualifications,
    unused_results,
    variant_size_differences,
    missing_docs
)]

pub mod engine;
pub mod model;
pub mod otap_ops;

#[cfg(test)]
mod tests;

use otel_arrow_dfe_pdata::otap::OtapArrowRecords;
use roaring::RoaringBitmap;

pub use crate::engine::{PolicyEngine, extract_required_ngrams};
pub use crate::model::{
    Action, AttributePath, LogAdd, LogAttributeSelector, LogFieldSelector, LogFilterPolicy,
    LogMatcher, LogRecordField, LogRedact, LogRemove, LogRename, LogTargetSelector,
    LogTransformPolicy, MetricFilterPolicy, NumericValue, Policy, PolicyDefinition, PolicyError,
    Predicate, ScalarValue, ScopeField, TraceSamplingPolicy,
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

/// Trait implemented by OTAP columnar policy evaluation engines.
pub trait PolicyEvaluationEngine {
    /// Human-readable engine identifier.
    fn name(&self) -> &'static str;

    /// Evaluates filter policies against `batch` and returns the bitmap of kept row indices (`0..num_rows`).
    fn evaluate_filter(&self, batch: &OtapArrowRecords) -> Result<RoaringBitmap, PolicyError>;

    /// Evaluates filter and transform policies against `batch` and returns the filtered/transformed `EvaluationOutput`.
    fn evaluate(&self, batch: OtapArrowRecords) -> Result<EvaluationOutput, PolicyError>;

    /// Estimated compiled heap memory footprint in bytes.
    fn estimated_heap_bytes(&self) -> usize;
}
