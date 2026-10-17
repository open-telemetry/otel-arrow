// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

/// ETW (Event Tracing for Windows) receiver.
#[cfg(all(feature = "etw", target_os = "windows"))]
pub mod etw_receiver;

/// Kafka receiver.
#[cfg(feature = "kafka")]
pub mod kafka_receiver;

/// Oracle database receiver.
#[cfg(feature = "oracle")]
pub mod oracle_receiver;

/// PostgreSQL database receiver.
#[cfg(feature = "postgresql")]
pub mod postgresql_receiver;

/// SQL Server configuration and query preparation without a runtime factory.
#[cfg(feature = "sql-server")]
pub mod sql_server_receiver;

/// Linux user_events receiver.
#[cfg(all(feature = "user-events", target_os = "linux"))]
pub mod user_events_receiver;

/// Windows Event Forwarding over mutual TLS.
#[cfg(feature = "windows-event-forwarding")]
pub mod windows_event_forwarding_receiver;

/// Windows performance-counter receiver.
#[cfg(all(feature = "windows-perf-counters", target_os = "windows"))]
pub mod windows_perf_counters_receiver;
