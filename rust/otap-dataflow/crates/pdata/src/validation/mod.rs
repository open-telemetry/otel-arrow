// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// This module contains a validation test suite for OTLP and OTAP data.

mod collector;
mod error;
mod otap;
mod otlp;
mod scenarios;
mod service_type;
mod tcp_stream;

#[cfg(test)]
mod testdata;
