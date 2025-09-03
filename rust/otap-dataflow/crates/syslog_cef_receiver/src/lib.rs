// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]

//! Syslog CEF receiver implementation

/// Syslog CEF receiver module
pub mod syslog_cef_receiver;

/// Parser module for syslog message parsing
pub mod parser;

/// Arrow records encoder for syslog messages
pub mod arrow_records_encoder;
