// SPDX-License-Identifier: Apache-2.0
// Copyright The OpenTelemetry Authors

#![warn(missing_docs)]

//! Syslog CEF receiver implementation

/// Syslog CEF receiver module
pub mod syslog_cef_receiver;

/// Parser module for syslog message parsing
mod parser;

/// Arrow records encoder for syslog messages
mod arrow_records_encoder;
