// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

/// Topic receiver.
#[cfg(feature = "topic-receiver")]
pub mod topic_receiver;

/// Internal telemetry receiver.
#[cfg(feature = "internal-telemetry-receiver")]
pub mod internal_telemetry_receiver;

/// Syslog CEF receiver.
#[cfg(feature = "syslog-cef-receiver")]
pub mod syslog_cef_receiver;

/// OTAP receiver.
#[cfg(feature = "otap-receiver")]
pub mod otap_receiver;

/// OTLP receiver.
#[cfg(feature = "otlp-receiver")]
pub mod otlp_receiver;

/// Host metrics receiver.
#[cfg(feature = "host-metrics-receiver")]
pub mod host_metrics_receiver;

/// Journald receiver.
#[cfg(feature = "journald-receiver")]
pub mod journald_receiver;
