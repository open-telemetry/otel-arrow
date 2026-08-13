// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Reusable machinery for `BearerTokenProvider` extensions.
//!
//! Every bearer-token extension in this crate shares the same shape: an
//! `Arc<Inner>` holding a token cache, a coalescing slow path, a negative
//! cache, and an active background refresh loop with jittered scheduling and
//! exponential backoff. The only thing that differs between them is *how a
//! token is acquired* and *which metric set is recorded*.
//!
//! This module owns everything that is common. An extension supplies a
//! [`TokenSource`] plus a [`TokenProviderMetrics`] metric set and gets the
//! capability implementation, the refresh loop, and the lifecycle wiring for
//! free by aliasing [`TokenProviderExtension`]:
//!
//! ```rust,ignore
//! pub type MyAuthExtension = TokenProviderExtension<MyAuth, MyAuthMetrics>;
//! ```

mod metrics;
mod provider;

#[cfg(test)]
mod tests;

pub use metrics::{TokenProviderMetrics, TokenProviderMetricsTracker};
pub use provider::{TokenProviderExtension, TokenSource};
