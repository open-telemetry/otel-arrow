// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

/// Geneva Exporter for Microsoft telemetry backend
#[cfg(feature = "geneva")]
pub mod geneva_exporter;

/// Azure Monitor Exporter for Azure Logs Ingestion API
#[cfg(feature = "azure-monitor")]
pub mod azure_monitor_exporter;

/// ClickHouse Exporter for columnar telemetry storage
#[cfg(feature = "clickhouse")]
pub mod clickhouse_exporter;
/// Kafka Exporter for Apache Kafka
#[cfg(feature = "kafka")]
pub mod kafka_exporter;
