// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Contributed exporter components.

/// Azure Monitor Exporter for Azure Logs Ingestion API
#[cfg(feature = "azure-monitor-exporter")]
pub mod azure_monitor_exporter;

/// Geneva exporter for Microsoft telemetry backend
#[cfg(feature = "geneva-exporter")]
pub mod geneva_exporter;
