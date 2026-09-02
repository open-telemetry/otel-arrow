// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Services created once for one pipeline runtime and injected into nodes.
//!
//! A dependency belongs here when its lifetime is the complete pipeline
//! runtime, multiple node families need the same instance, and central creation
//! or validation protects pipeline-local invariants. Mutable services should
//! also benefit from staying on the pipeline's runtime thread.
//!
//! Node configuration, node-local scheduling and metrics, process-wide state,
//! and data-type-specific control channels should remain with their existing
//! owners. This boundary keeps the service bundle focused and avoids turning it
//! into a container for unrelated runtime objects.

use otel_arrow_dfe_pdata_codec::{CodecService, RegistryError};

/// Runtime-owned services shared by every effect handler in one pipeline.
///
/// Adding a service here should satisfy the module-level ownership criteria and
/// preserve the single-threaded pipeline runtime's state-sharing semantics.
#[derive(Clone)]
pub struct PipelineRuntimeServices {
    codecs: CodecService,
}

impl PipelineRuntimeServices {
    /// Validates linked codec extensions and creates lazy pipeline-local state.
    pub fn new() -> Result<Self, RegistryError> {
        Ok(Self {
            codecs: CodecService::new()?,
        })
    }

    /// Pipeline-local codec access.
    #[must_use]
    pub const fn codecs(&self) -> &CodecService {
        &self.codecs
    }

    /// Returns whether two handles address the same pipeline-owned services.
    #[must_use]
    pub fn shares_state_with(&self, other: &Self) -> bool {
        self.codecs.shares_state_with(&other.codecs)
    }
}

/// Common pdata codec access exposed by receiver, processor, and exporter effects.
pub trait CodecEffectHandler {
    /// Returns the codec service injected by the pipeline runtime.
    fn codec_service(&self) -> &CodecService;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::test_node;
    use otel_arrow_dfe_telemetry::reporter::MetricsReporter;

    fn requires_codec_effect_handler<T: CodecEffectHandler>() {}

    /// Scenario: A pipeline injects its runtime services into all six effect-handler families.
    /// Guarantees: Local and shared receivers, processors, and exporters expose one common service contract.
    #[test]
    fn all_pdata_effect_handlers_use_the_shared_service_contract() {
        requires_codec_effect_handler::<crate::local::receiver::EffectHandler<()>>();
        requires_codec_effect_handler::<crate::shared::receiver::EffectHandler<()>>();
        requires_codec_effect_handler::<crate::local::processor::EffectHandler<()>>();
        requires_codec_effect_handler::<crate::shared::processor::EffectHandler<()>>();
        requires_codec_effect_handler::<crate::local::exporter::EffectHandler<()>>();
        requires_codec_effect_handler::<crate::shared::exporter::EffectHandler<()>>();

        let runtime_services = PipelineRuntimeServices::new().unwrap();
        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let local = crate::local::exporter::EffectHandler::<()>::new(
            test_node("local"),
            metrics_reporter.clone(),
            runtime_services.clone(),
        );
        let shared = crate::shared::exporter::EffectHandler::<()>::new(
            test_node("shared"),
            metrics_reporter,
            runtime_services.clone(),
        );

        assert!(
            local
                .codec_service()
                .shares_state_with(shared.codec_service())
        );
    }
}
