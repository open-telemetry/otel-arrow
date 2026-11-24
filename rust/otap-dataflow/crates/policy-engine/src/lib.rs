// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Pure-Rust columnar telemetry policy evaluation engine for OTAP / Arrow Dataflow (`open-telemetry/otel-arrow#4726`).

#![forbid(unsafe_code)]

pub mod model;

#[cfg(test)]
mod tests;

pub use crate::model::{
    Action, AttributePath, CustomPolicy, LogAdd, LogAttributeSelector, LogFieldSelector,
    LogFilterPolicy, LogMatcher, LogRecordField, LogRedact, LogRemove, LogRename,
    LogTargetSelector, LogTransformPolicy, MetricFilterPolicy, NumericValue, Policy,
    PolicyCompileError, PolicyDefinition, PolicyError, Predicate, SamplingConfig, SamplingMode,
    ScalarValue, ScopeField, SpanSamplingPolicy,
};
