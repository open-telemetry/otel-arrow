// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// This module contains a validation test suite for OTLP and OTAP data.
// These are integration tests that spawn the OpenTelemetry Collector as a child process.

// Allow test-friendly patterns in this test-only module
#![allow(clippy::unwrap_used)]
#![allow(clippy::print_stderr)]

mod collector;
mod error;
mod otap;
mod otlp;
mod scenarios;
mod service_type;
mod tcp_stream;

#[cfg(test)]
mod testdata;
