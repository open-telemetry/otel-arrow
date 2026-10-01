// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

/// Topic receiver.
#[cfg(feature = "topic")]
pub mod topic_receiver;

/// Internal telemetry receiver.
pub mod internal_telemetry_receiver;

/// Syslog CEF receiver.
#[cfg(feature = "syslog-cef")]
pub mod syslog_cef_receiver;

/// OTAP receiver.
#[cfg(feature = "otap")]
pub mod otap_receiver;

/// OTLP receiver.
#[cfg(feature = "otlp")]
pub mod otlp_receiver;

/// Host metrics receiver.
#[cfg(feature = "host-metrics")]
pub mod host_metrics_receiver;

/// Journald receiver.
#[cfg(feature = "journald")]
pub mod journald_receiver;

/// Filelog source primitives.
pub mod filelog_receiver;
