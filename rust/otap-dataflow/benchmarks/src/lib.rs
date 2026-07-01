// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Support library for the `otap-dataflow` benchmarks.
//!
//! The heavier benchmark logic lives here (rather than directly in the Criterion
//! `benches/` targets) so it can be unit-tested for correctness. Currently this
//! hosts the [`parquet_study`] module, which compares the cost of moving OTAP
//! logs as compressed Arrow IPC versus several flattened-Parquet encodings.

#![allow(missing_docs)]

pub mod parquet_study;
