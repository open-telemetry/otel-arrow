// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Core OTAP receiver components.

/// Fake data generator for testing
pub mod fake_data_generator;

/// Implementation of OTAP Receiver that implements the receiver trait
pub mod otap_receiver;

/// This receiver receives OTLP bytes from the grpc service request and
/// produce for the pipeline OTAP PData
pub mod otlp_receiver;

/// Receiver that reads in syslog data
pub mod syslog_cef_receiver;
