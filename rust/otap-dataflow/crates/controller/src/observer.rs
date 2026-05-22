// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Observer context for embedders using the controller as a library.

use otap_df_state::store::ObservedStateHandle;
use otap_df_telemetry::registry::TelemetryRegistryHandle;

/// Context provided to observer callbacks during controller startup.
///
/// Gives embedders zero-overhead, in-process access to pipeline state
/// and internal metrics without requiring the HTTP admin server.
///
/// Both handles are cheaply cloneable (`Arc`-based) and safe to move
/// into background threads.
#[derive(Debug, Clone)]
pub struct EngineObserverContext {
    state: ObservedStateHandle,
    telemetry: TelemetryRegistryHandle,
}

impl EngineObserverContext {
    /// Creates a new observer context from the given handles.
    pub(crate) fn new(state: ObservedStateHandle, telemetry: TelemetryRegistryHandle) -> Self {
        Self { state, telemetry }
    }

    /// Returns a reference to the observed pipeline state handle.
    ///
    /// Use this to query pipeline liveness, readiness, and health status
    /// without going through the admin HTTP server.
    #[must_use]
    pub fn state_handle(&self) -> &ObservedStateHandle {
        &self.state
    }

    /// Returns a reference to the telemetry registry handle.
    ///
    /// Use this to read internal metrics snapshots (e.g. per-node throughput,
    /// queue depths, processing latencies) without enabling the admin HTTP
    /// server.
    #[must_use]
    pub fn telemetry_handle(&self) -> &TelemetryRegistryHandle {
        &self.telemetry
    }

    /// Consumes the context and returns both handles.
    #[must_use]
    pub fn into_parts(self) -> (ObservedStateHandle, TelemetryRegistryHandle) {
        (self.state, self.telemetry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_config::observed_state::ObservedStateSettings;
    use otap_df_state::store::ObservedStateStore;

    fn make_context() -> EngineObserverContext {
        let registry = TelemetryRegistryHandle::new();
        let store = ObservedStateStore::new(
            &ObservedStateSettings::default(),
            registry.clone(),
        );
        EngineObserverContext::new(store.handle(), registry)
    }

    #[test]
    fn accessors_return_valid_handles() {
        let ctx = make_context();
        // state_handle should return an empty snapshot (no pipelines registered).
        assert!(ctx.state_handle().snapshot().is_empty());
        // telemetry_handle is accessible.
        let _telemetry = ctx.telemetry_handle();
    }

    #[test]
    fn clone_shares_underlying_state() {
        let ctx = make_context();
        let cloned = ctx.clone();
        // Both clones should see the same (empty) snapshot.
        assert_eq!(
            ctx.state_handle().snapshot().len(),
            cloned.state_handle().snapshot().len()
        );
    }

    #[test]
    fn into_parts_yields_both_handles() {
        let ctx = make_context();
        let (state, telemetry) = ctx.into_parts();
        assert!(state.snapshot().is_empty());
        let _telemetry = telemetry;
    }
}
