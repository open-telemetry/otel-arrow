// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Exporter for mapping OTLP metrics to Geneva Metrics protocol version 6 packets.

// TODO: Add `otel_component_scope!` here so exporter diagnostics use
// `otel.exporter.geneva_metrics` rather than the crate-wide target.

mod component_registration;
mod config;
pub mod encoder;
mod metric_export_loop;
pub mod otlp_to_geneva;
mod publishing;

pub use component_registration::{GENEVA_METRICS_EXPORTER, GENEVA_METRICS_EXPORTER_URN};
pub use config::{AuthConfig, Config as ExporterConfig};
pub use metric_export_loop::GenevaMetricsExporter;
pub use otlp_to_geneva::{Config, ScopeAttributes};
