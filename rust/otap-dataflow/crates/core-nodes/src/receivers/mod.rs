// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

/// Traffic generator receiver.
#[cfg(feature = "dev-tools")]
pub mod traffic_generator;

/// Topic receiver.
pub mod topic_receiver;

/// Internal telemetry receiver.
pub mod internal_telemetry_receiver;

/// Syslog CEF receiver.
pub mod syslog_cef_receiver;

/// OTAP receiver.
pub mod otap_receiver;

/// OTLP receiver.
pub mod otlp_receiver;
