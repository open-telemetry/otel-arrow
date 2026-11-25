// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Windows Event Forwarding receiver over mutual TLS.
//!
//! This receiver accepts rendered events from Windows Event Collector sources
//! and converts them into OTAP log batches using the shared
//! [`crate::common::windows_event`] layer. Modules are introduced
//! incrementally; configuration, stable subscription identifiers, and source
//! identity authorization with bounded mutual TLS handshakes are available,
//! along with WS-Man routing, request validation, and bounded subscription
//! advertisement encoding, bounded event extraction, and atomic shared event
//! mapping. Bookmark candidates and bounded memory-only progress are available;
//! downstream feedback and resume advertisement integration remain deferred.
//! No receiver is registered or serving HTTP yet.

/// Bounded memory-only bookmark progress and admission state.
pub mod bookmark;

/// Configuration and stable subscription identifiers.
pub mod config;

/// Atomic rendered event conversion and synthetic marker filtering.
pub mod event;

/// Identity and authorization of mutually authenticated sources.
pub mod identity;

/// Mandatory mutual TLS and source authorization for accepted connections.
pub mod tls;

/// Bounded WS-Man text decoding, endpoint routing, and SOAP envelope parsing.
pub mod wsman;
