// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! KQL Query Processor
//!
//! This processor uses the experimental KQL recordset engine to filter and transform
//! telemetry data using Kusto Query Language (KQL) expressions.

pub mod processor;

pub use processor::*;
