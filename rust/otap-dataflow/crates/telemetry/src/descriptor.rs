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