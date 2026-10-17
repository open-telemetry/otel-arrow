// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Windows Event Forwarding receiver over mutual TLS.
//!
//! This receiver accepts rendered events from Windows Event Collector sources
//! and converts them into OTAP log batches using the shared
//! [`crate::common::windows_event`] layer. Modules are introduced
//! incrementally; configuration and stable subscription identifiers land first.

/// Configuration and stable subscription identifiers.
pub mod config;
