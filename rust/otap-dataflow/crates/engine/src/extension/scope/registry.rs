// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Extension provider lookup across declaration scopes.

use crate::capability::registry::CapabilityRegistry;
use crate::capability::{ExtensionCapabilities, SharedInstanceFactory};
use crate::error::Error;
use otel_arrow_dfe_config::pipeline::PipelineExtensions;
use otel_arrow_dfe_config::{ExtensionId, PipelineGroupId};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

/// A pipeline-owned snapshot of shared capability providers inherited from
/// engine and pipeline-group scopes.
#[derive(Clone, Default)]
pub struct InheritedExtensionRegistrations {
    known_extensions: HashSet<ExtensionId>,
    registrations: Vec<SharedExtensionRegistration>,
}

impl Debug for InheritedExtensionRegistrations {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("InheritedExtensionRegistrations")
            .field("known_extensions", &self.known_extensions)
            .field(
                "registered_extensions",
                &self
                    .registrations
                    .iter()
                    .map(|registration| &registration.extension_id)
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl InheritedExtensionRegistrations {
    /// Returns whether the snapshot contains no inherited extension entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.known_extensions.is_empty() && self.registrations.is_empty()
    }

    pub(crate) fn remove_shadowed_by(&mut self, pipeline_extensions: &PipelineExtensions) {
        self.known_extensions
            .retain(|extension_id| !pipeline_extensions.contains_key(extension_id));
        self.registrations
            .retain(|registration| !pipeline_extensions.contains_key(&registration.extension_id));
    }

    pub(crate) fn extend_known_extensions(&self, known_extensions: &mut HashSet<ExtensionId>) {
        known_extensions.extend(self.known_extensions.iter().cloned());
    }

    pub(crate) fn register_into(&self, registry: &mut CapabilityRegistry) -> Result<(), Error> {
        for registration in &self.registrations {
            (registration.capabilities.register_shared)(
                registration.extension_id.clone(),
                registration.instance_factory.clone(),
                registry,
            )
            .map_err(|error| Error::CapabilityRegistrationFailed {
                extension: registration.extension_id.clone(),
                message: error.to_string(),
            })?;
        }
        Ok(())
    }
}

#[derive(Clone)]
pub(super) struct SharedExtensionRegistration {
    pub(super) extension_id: ExtensionId,
    pub(super) capabilities: ExtensionCapabilities,
    pub(super) instance_factory: SharedInstanceFactory,
}

#[derive(Default)]
pub(super) struct ScopeCatalog {
    pub(super) known_extensions: HashSet<ExtensionId>,
    pub(super) registrations: Vec<SharedExtensionRegistration>,
}

#[derive(Default)]
pub(super) struct ExtensionScopeCatalog {
    pub(super) engine: ScopeCatalog,
    pub(super) groups: HashMap<PipelineGroupId, ScopeCatalog>,
}

#[derive(Default)]
struct ExtensionScopeRegistryState {
    installed: bool,
    catalog: ExtensionScopeCatalog,
}

/// Immutable capability catalogs for engine and pipeline-group declarations.
///
/// The mutex is used only to publish the catalog once and clone build-time
/// snapshots across controller threads. Pipeline data paths never access it.
#[derive(Clone, Default)]
pub struct ExtensionScopeRegistry {
    state: Arc<Mutex<ExtensionScopeRegistryState>>,
}

impl ExtensionScopeRegistry {
    pub(super) fn install(&self, catalog: ExtensionScopeCatalog) -> Result<(), Error> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.installed {
            return Err(Error::InternalError {
                message: "extension scope catalog was installed more than once".into(),
            });
        }
        state.catalog = catalog;
        state.installed = true;
        Ok(())
    }

    /// Returns the inherited registrations visible from a pipeline.
    ///
    /// Pipeline declarations shadow pipeline-group and engine declarations.
    /// Pipeline-group declarations shadow engine declarations with the same
    /// extension ID.
    #[must_use]
    pub fn registrations_for_pipeline(
        &self,
        pipeline_group_id: &PipelineGroupId,
        pipeline_extensions: &PipelineExtensions,
    ) -> InheritedExtensionRegistrations {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut inherited = InheritedExtensionRegistrations::default();
        let group = state.catalog.groups.get(pipeline_group_id);

        if let Some(group) = group {
            for extension_id in &group.known_extensions {
                if !pipeline_extensions.contains_key(extension_id) {
                    let _ = inherited.known_extensions.insert(extension_id.clone());
                }
            }
            inherited.registrations.extend(
                group
                    .registrations
                    .iter()
                    .filter(|registration| {
                        !pipeline_extensions.contains_key(&registration.extension_id)
                    })
                    .cloned(),
            );
        }

        for extension_id in &state.catalog.engine.known_extensions {
            let shadowed_by_group =
                group.is_some_and(|group| group.known_extensions.contains(extension_id));
            if !pipeline_extensions.contains_key(extension_id) && !shadowed_by_group {
                let _ = inherited.known_extensions.insert(extension_id.clone());
            }
        }
        inherited.registrations.extend(
            state
                .catalog
                .engine
                .registrations
                .iter()
                .filter(|registration| {
                    !pipeline_extensions.contains_key(&registration.extension_id)
                        && !group.is_some_and(|group| {
                            group.known_extensions.contains(&registration.extension_id)
                        })
                })
                .cloned(),
        );

        inherited
    }
}
