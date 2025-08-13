// SPDX-License-Identifier: Apache-2.0

//! Metric and Attribute descriptor types for metrics reflection.

/// Enumerates supported metric field kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricsKind {
    /// Monotonic counter (u64) aggregated by summation.
    Counter,
}

/// Metadata describing a single field inside a metrics struct.
#[derive(Debug, Clone, Copy)]
pub struct MetricsField {
    /// Canonical metric name (e.g., "bytes.rx").
    pub name: &'static str,
    /// Unit (e.g., "bytes", "count").
    pub unit: &'static str,
    /// Field kind (counter, etc.).
    pub kind: MetricsKind,
}

/// Descriptor for a multivariate metrics.
#[derive(Debug)]
pub struct MetricsDescriptor {
    /// Human-friendly group name.
    pub name: &'static str,
    /// Ordered field metadata.
    pub fields: &'static [MetricsField],
}

/// Immutable attributes captured at metrics creation time.
#[derive(Debug, Clone, Copy, Default)]
pub struct StaticAttrs {
    /// Pipeline identifier (numeric mapping if available, 0 means unknown).
    pub pipeline_id: u32,
    /// Core identifier (0 means unknown).
    pub core_id: u16,
    /// NUMA node identifier (0 means unknown/single node).
    pub numa_node_id: u16,
    /// Process identifier.
    pub process_id: u32,
}