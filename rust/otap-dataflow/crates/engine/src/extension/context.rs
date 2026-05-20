// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Standalone context for extension-hosting code.
//!
//! Extensions today are pipeline-scoped, but they are not conceptually
//! "a pipeline minus the node bits". To keep that distinction honest
//! and to leave room for an engine-scoped extension lifecycle later,
//! [`ExtensionContext`] is its own type with its own named
//! constructors — it never has to be born from a [`PipelineContext`].
//!
//! Today the only constructor is
//! [`ExtensionContext::from_pipeline`]. When engine-scoped
//! extensions land, an `ExtensionContext::from_engine` constructor
//! will be added alongside it. Consumers of this type are deliberately
//! kept blind to which scope they're running under — they get a
//! registry handle and a way to register a per-extension metrics
//! entity, and that's it. The parent attribute hierarchy is an
//! implementation detail of `ExtensionContext`.

use crate::attributes::{ExtensionAttributeSet, PipelineAttributeSet};
use crate::context::PipelineContext;
use otap_df_config::ExtensionId;
use otap_df_telemetry::registry::{EntityKey, TelemetryRegistryHandle};

/// Context handed to extension-hosting code (the extension lifecycle
/// today; potentially more in the future).
///
/// Carries only what extensions actually need — a metrics registry
/// handle and a way to register a per-extension metrics entity that
/// composes into whatever parent attribute hierarchy is appropriate
/// for the current scope. Consumers cannot and should not depend on
/// being at any particular scope (pipeline today, engine later).
#[derive(Clone, Debug)]
pub struct ExtensionContext {
    registry: TelemetryRegistryHandle,
    /// Parent attribute hierarchy the per-extension entity composes
    /// into. Kept private so callers don't get to know whether they're
    /// pipeline- or engine-scoped. Today there is exactly one variant.
    attribute_set: AttributeSet,
}

#[derive(Clone, Debug)]
enum AttributeSet {
    Pipeline(PipelineAttributeSet),
    // Future: Engine(EngineAttributeSet),
}

impl ExtensionContext {
    /// Builds a pipeline-scoped extension context from raw parts.
    /// Useful when there is no [`PipelineContext`] handy (tests,
    /// non-runtime callers that already hold the parent attribute set).
    #[must_use]
    pub fn from_pipeline_parts(
        registry: TelemetryRegistryHandle,
        pipeline_attrs: PipelineAttributeSet,
    ) -> Self {
        Self {
            registry,
            attribute_set: AttributeSet::Pipeline(pipeline_attrs),
        }
    }

    /// Builds an extension context for an extension hosted inside the
    /// given pipeline.
    #[must_use]
    pub fn from_pipeline(pipeline_context: &PipelineContext) -> Self {
        Self::from_pipeline_parts(
            pipeline_context.metrics_registry(),
            pipeline_context.pipeline_attribute_set(),
        )
    }

    /// Returns a metrics registry handle.
    #[must_use]
    pub fn metrics_registry(&self) -> TelemetryRegistryHandle {
        self.registry.clone()
    }

    /// Registers a per-extension metrics entity under whatever parent
    /// hierarchy this context represents, and returns its key.
    /// Callers should then use the returned [`EntityKey`] to register
    /// any metric sets they own for this extension.
    #[must_use]
    pub fn register_extension_entity(&self, extension_id: &ExtensionId) -> EntityKey {
        match &self.attribute_set {
            AttributeSet::Pipeline(pipeline_attrs) => {
                self.registry.register_entity(ExtensionAttributeSet {
                    extension_id: extension_id.clone(),
                    pipeline_attrs: pipeline_attrs.clone(),
                })
            }
        }
    }
}
