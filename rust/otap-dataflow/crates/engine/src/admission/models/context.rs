// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Per-request context handed to an admission gate.

use otel_arrow_dfe_config::SignalType;

/// Transport-neutral, borrow-only context describing one admission request.
///
/// # Why this shape
///
/// * **No transport types.** There is no `http::Request`, `tonic::Request`,
///   header map, or socket here. A gate is shared by an HTTP server, a gRPC
///   service, and a UDP datagram loop, so anything protocol-specific would
///   force every component to depend on every other component's transport
///   crate.
/// * **No allocation on the hot path.** Every field is `Copy` or a borrow, so
///   building a context per request is a few register moves. `AdmissionContext`
///   is itself `Copy`, so passing it by value costs nothing.
/// * **Room for tenant and group keying.** V1 never populates
///   [`tenant_key`](Self::tenant_key) or [`scope_key`](Self::scope_key) -- the
///   receiver-instance aggregation this release ships does not need them. They
///   exist so a future tenant- or group-aware provider can be added without
///   changing the gate signature that every participating component calls, and
///   so components can start extracting an identity ahead of the provider that
///   consumes it.
///
/// Both key fields are opaque `&str`. Providers must treat them as untrusted
/// client-supplied data: they may be used to select a bucket, but must never be
/// emitted as a metric attribute (see the fixed-cardinality rule on admission
/// telemetry).
#[derive(Debug, Clone, Copy, Default)]
pub struct AdmissionContext<'a> {
    signal: Option<SignalType>,
    tenant_key: Option<&'a str>,
    scope_key: Option<&'a str>,
}

impl<'a> AdmissionContext<'a> {
    /// An empty context: no signal, no tenant, no scope.
    ///
    /// Used by components whose admission point carries no signal
    /// discrimination, such as the Syslog receiver.
    pub const EMPTY: Self = Self {
        signal: None,
        tenant_key: None,
        scope_key: None,
    };

    /// Creates a context that identifies only the telemetry signal.
    #[must_use]
    pub const fn for_signal(signal: SignalType) -> Self {
        Self {
            signal: Some(signal),
            tenant_key: None,
            scope_key: None,
        }
    }

    /// Attaches a resolved tenant identity.
    ///
    /// Reserved for a future tenant-aware provider; no built-in provider reads
    /// it in this release.
    #[must_use]
    pub const fn with_tenant_key(mut self, tenant_key: &'a str) -> Self {
        self.tenant_key = Some(tenant_key);
        self
    }

    /// Attaches a resolved pipeline or group identity.
    ///
    /// Reserved for a future group-scoped provider; no built-in provider reads
    /// it in this release.
    #[must_use]
    pub const fn with_scope_key(mut self, scope_key: &'a str) -> Self {
        self.scope_key = Some(scope_key);
        self
    }

    /// Returns the telemetry signal, when the admission point knows one.
    #[must_use]
    pub const fn signal(&self) -> Option<SignalType> {
        self.signal
    }

    /// Returns the resolved tenant identity, when one was extracted.
    #[must_use]
    pub const fn tenant_key(&self) -> Option<&'a str> {
        self.tenant_key
    }

    /// Returns the resolved pipeline or group identity, when one was extracted.
    #[must_use]
    pub const fn scope_key(&self) -> Option<&'a str> {
        self.scope_key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: a Syslog datagram admission point has no signal or identity to report.
    /// Guarantees: the empty context exposes no signal, tenant, or scope, so a provider
    /// can distinguish "not supplied" from a defaulted value.
    #[test]
    fn empty_context_reports_no_identity() {
        let ctx = AdmissionContext::EMPTY;

        assert_eq!(ctx.signal(), None);
        assert_eq!(ctx.tenant_key(), None);
        assert_eq!(ctx.scope_key(), None);
    }

    /// Scenario: an OTLP admission point supplies the signal, and a future tenant-aware
    /// component additionally supplies tenant and scope identities.
    /// Guarantees: each field round-trips independently, so the reserved keying seam
    /// stays usable without changing the gate signature.
    #[test]
    fn context_round_trips_signal_tenant_and_scope() {
        let ctx = AdmissionContext::for_signal(SignalType::Logs)
            .with_tenant_key("tenant-a")
            .with_scope_key("group-1");

        assert_eq!(ctx.signal(), Some(SignalType::Logs));
        assert_eq!(ctx.tenant_key(), Some("tenant-a"));
        assert_eq!(ctx.scope_key(), Some("group-1"));
    }

    /// Scenario: a component builds one context per request on the hot path.
    /// Guarantees: the context is `Copy`, so passing it to a gate neither allocates
    /// nor moves ownership away from the caller.
    #[test]
    fn context_is_copy_and_allocation_free() {
        let ctx = AdmissionContext::for_signal(SignalType::Metrics);
        let copied = ctx;

        assert_eq!(ctx.signal(), copied.signal());
    }
}
