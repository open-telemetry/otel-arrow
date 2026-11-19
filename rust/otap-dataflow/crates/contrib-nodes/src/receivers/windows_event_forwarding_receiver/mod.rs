// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Windows Event Forwarding receiver over mutual TLS.
//!
//! This receiver accepts rendered events from Windows Event Collector sources
//! and converts them into OTAP log batches using the shared
//! [`crate::common::windows_event`] layer. Modules are introduced
//! incrementally; configuration, stable subscription identifiers, and source
//! identity authorization with bounded mutual TLS handshakes are available,
//! along with WS-Man routing, request validation, and bounded basic responses.
//! No receiver is registered yet, and no subscriptions are advertised.

/// Configuration and stable subscription identifiers.
pub mod config;

/// Identity and authorization of mutually authenticated sources.
pub mod identity;

/// Mandatory mutual TLS and source authorization for accepted connections.
pub mod tls;

/// Bounded WS-Man text decoding, endpoint routing, and SOAP envelope parsing.
pub mod wsman;
