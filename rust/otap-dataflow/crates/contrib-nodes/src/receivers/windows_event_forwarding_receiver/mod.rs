// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

/// Bounded in-memory delivery progress for a single subscription.
pub mod bookmark;

/// Configuration and stable subscription identifiers.
pub mod config;

/// Windows Event XML conversion to Arrow log batches.
pub mod event;

/// Identity and authorization of mutually authenticated sources.
pub mod identity;

/// Authenticated subscription management and rendered-event delivery endpoints.
pub mod http;

/// Dataflow receiver registration and lifecycle.
pub mod receiver;

mod metrics;
mod runtime;
mod xml;

/// Mandatory mutual TLS transport setup.
pub mod tls;

/// Bounded SOAP decoding and subscription request routing.
pub mod wsman;
