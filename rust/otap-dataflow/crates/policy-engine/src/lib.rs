// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Telemetry policy evaluation engine for OTAP Dataflow.
//!
//! This crate bridges the `policy-rs` policy definitions with the
//! `otel-arrow-dfe-query-engine` pipeline stage infrastructure. It compiles
//! telemetry policies (as defined by the Tero policy specification) into a
//! sequence of query-engine `PipelineStage`s that can be executed against
//! OTAP Arrow record batches.
//!
//! # Architecture
//!
//! Policies are independent, atomic rules that match telemetry records and
//! apply actions (drop, sample, transform). This crate's planner compiles
//! a set of policies into an ordered list of pipeline stages:
//!
//! 1. **Drop filter** -- all `keep: "none"` policies are merged into a single
//!    `FilterPipelineStage` that removes matching rows up front.
//! 2. **Transform conditionals** -- each policy with transforms becomes a
//!    `ConditionalPipelineStage` that applies its transforms (remove, redact,
//!    rename, add) only to rows matching that policy's matchers.
