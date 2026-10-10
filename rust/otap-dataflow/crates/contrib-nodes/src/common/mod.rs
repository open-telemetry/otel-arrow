// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared functions and data types for contrib node implementations.

/// Bounded, read-only XML document view shared by XML-based nodes.
#[cfg(feature = "windows-event-forwarding")]
pub mod xml;

/// Shared, cross-platform Windows event representation and parsing.
#[cfg(feature = "windows-event-forwarding")]
pub mod windows_event;

/// Shared Kafka utilities for Kafka receiver and exporter.
#[cfg(feature = "kafka")]
pub mod kafka;
