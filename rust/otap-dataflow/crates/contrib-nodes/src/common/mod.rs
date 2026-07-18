// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared functions and data types for contrib node implementations.

/// Shared Kafka utilities for Kafka receiver and exporter.
#[cfg(any(feature = "kafka-receiver", feature = "kafka-exporter"))]
pub mod kafka;
