// SPDX-License-Identifier: Apache-2.0

//! Type-safe API and NUMA-aware telemetry data structures and utilities.
//!
//! Note: The NUMA-aware design is not fully implemented yet.
//! ToDo: First aggregation pass per NUMA node, then global aggregation.

pub mod registry;
pub mod perf_exporter_metrics;
mod registry2;
pub mod metrics;
pub mod aggregator;
pub mod counter;
mod descriptor;
mod attributes;
