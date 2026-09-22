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

    /// Returns the inherited registrations requested by a pipeline's node bindings.
    ///
    /// Pipeline declarations shadow pipeline-group and engine declarations.
    /// Pipeline-group declarations shadow engine declarations with the same
    /// extension ID. Unused providers remain hosted but their factories are not
    /// copied into the pipeline. All visible declaration IDs remain known for
    /// binding diagnostics, including capability-less declarations.
    #[must_use]
    pub fn registrations_for_pipeline<'a>(
        &self,
        pipeline_group_id: &PipelineGroupId,
        pipeline_extensions: &PipelineExtensions,
        requested_extensions: impl IntoIterator<Item = &'a ExtensionId>,
    ) -> InheritedExtensionRegistrations {
        let requested: HashSet<_> = requested_extensions.into_iter().collect();
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
                        requested.contains(&registration.extension_id)
                            && !pipeline_extensions.contains_key(&registration.extension_id)
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
                    requested.contains(&registration.extension_id)
                        && !pipeline_extensions.contains_key(&registration.extension_id)
                        && !group.is_some_and(|group| {
                            group.known_extensions.contains(&registration.extension_id)
                        })
                })
                .cloned(),
        );

        inherited
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::registry::{Capabilities, ConsumedTracker, resolve_bindings};
    use crate::extension_capabilities;
    use crate::testing::capability::no_op_stateless::{NoOpStateless, SharedNoOpStateless};
    use async_trait::async_trait;
    use otel_arrow_dfe_config::extension::ExtensionUserConfig;
    use std::any::TypeId;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct Provider {
        name: &'static str,
        // Shared state lets independent snapshot consumers observe the same provider.
        token: Arc<AtomicU64>,
        clones: Arc<AtomicU64>,
    }

    impl Clone for Provider {
        fn clone(&self) -> Self {
            let _ = self.clones.fetch_add(1, Ordering::SeqCst);
            Self {
                name: self.name,
                token: self.token.clone(),
                clones: self.clones.clone(),
            }
        }
    }

    impl Provider {
        fn new(name: &'static str) -> Self {
            Self {
                name,
                token: Arc::new(AtomicU64::new(0)),
                clones: Arc::new(AtomicU64::new(0)),
            }
        }
    }

    #[async_trait]
    impl SharedNoOpStateless for Provider {
        fn name(&self) -> &str {
            self.name
        }

        fn echo(&self, value: u64) -> u64 {
            value
        }

        async fn ping(&self) -> u64 {
            self.token.load(Ordering::SeqCst)
        }

        async fn echo_async(&self, value: String) -> String {
            value
        }
    }

    fn registration(extension_id: &'static str, provider: Provider) -> SharedExtensionRegistration {
        SharedExtensionRegistration {
            extension_id: extension_id.into(),
            capabilities: extension_capabilities!(shared: Provider => [NoOpStateless]),
            instance_factory: SharedInstanceFactory::new(move || Box::new(provider.clone())),
        }
    }

    fn catalog(
        registrations: Vec<SharedExtensionRegistration>,
        background: &[&'static str],
    ) -> ScopeCatalog {
        ScopeCatalog {
            known_extensions: registrations
                .iter()
                .map(|registration| registration.extension_id.clone())
                .chain(background.iter().map(|id| (*id).into()))
                .collect(),
            registrations,
        }
    }

    fn resolve(
        inherited: &InheritedExtensionRegistrations,
        extension_id: &'static str,
    ) -> Result<Capabilities, Error> {
        let mut registry = CapabilityRegistry::new();
        inherited.register_into(&mut registry)?;
        let mut known_extensions = HashSet::new();
        inherited.extend_known_extensions(&mut known_extensions);
        resolve_bindings(
            &HashMap::from([("no_op_stateless".into(), extension_id.into())]),
            &registry,
            &known_extensions,
            &mut ConsumedTracker::new(),
        )
    }

    fn provider(
        inherited: &InheritedExtensionRegistrations,
        extension_id: &'static str,
    ) -> Box<dyn SharedNoOpStateless> {
        resolve(inherited, extension_id)
            .expect("visible provider resolves")
            .require_shared::<NoOpStateless>()
            .expect("shared provider is consumable")
    }

    /// Scenario: cloned registry handles install twice and consumers retain snapshots independently.
    /// Guarantees: installation is single-shot, the original catalog survives, and snapshots share provider state.
    #[tokio::test]
    async fn installation_is_single_shot_and_snapshots_preserve_provider_state() {
        let registry = ExtensionScopeRegistry::default();
        let cloned_registry = registry.clone();
        let group = PipelineGroupId::from("group");
        let before_install = cloned_registry.registrations_for_pipeline(
            &group,
            &PipelineExtensions::default(),
            &["provider".into()],
        );
        assert!(before_install.is_empty());

        let original = Provider::new("original");
        let token = original.token.clone();
        registry
            .install(ExtensionScopeCatalog {
                engine: catalog(vec![registration("provider", original)], &[]),
                ..Default::default()
            })
            .expect("first catalog installs");
        let snapshot = cloned_registry.registrations_for_pipeline(
            &group,
            &PipelineExtensions::default(),
            &["provider".into()],
        );
        let cloned_snapshot = snapshot.clone();
        let first_consumer = provider(&snapshot, "provider");

        let error = cloned_registry
            .install(ExtensionScopeCatalog {
                engine: catalog(
                    vec![registration("replacement", Provider::new("replacement"))],
                    &[],
                ),
                ..Default::default()
            })
            .expect_err("a second installation must fail");
        assert!(matches!(
            error,
            Error::InternalError { message }
                if message == "extension scope catalog was installed more than once"
        ));

        let after_rejection = registry.registrations_for_pipeline(
            &group,
            &PipelineExtensions::default(),
            &["provider".into()],
        );
        assert_eq!(
            after_rejection.known_extensions,
            HashSet::from(["provider".into()])
        );
        assert!(before_install.is_empty(), "snapshots are not live views");
        drop(registry);
        drop(cloned_registry);
        token.store(42, Ordering::SeqCst);
        for inherited in [&snapshot, &cloned_snapshot, &after_rejection] {
            let consumer = provider(inherited, "provider");
            assert_eq!(consumer.name(), "original");
            assert_eq!(consumer.ping().await, 42);
        }
        assert_eq!(first_consumer.ping().await, 42);
    }

    /// Scenario: one group overrides a provider and masks another with a capability-less declaration.
    /// Guarantees: shadowing covers the whole ID, siblings stay isolated, and absent groups inherit only engine entries.
    #[test]
    fn group_shadowing_masks_whole_ids_and_isolates_siblings() {
        let registry = ExtensionScopeRegistry::default();
        registry
            .install(ExtensionScopeCatalog {
                engine: catalog(
                    vec![
                        registration("shared", Provider::new("engine")),
                        registration("masked", Provider::new("engine-masked")),
                    ],
                    &[],
                ),
                groups: HashMap::from([
                    (
                        "left".into(),
                        catalog(
                            vec![
                                registration("shared", Provider::new("left")),
                                registration("left-only", Provider::new("left-only")),
                            ],
                            &["masked"],
                        ),
                    ),
                    (
                        "right".into(),
                        catalog(
                            vec![registration("right-only", Provider::new("right-only"))],
                            &[],
                        ),
                    ),
                ]),
            })
            .expect("catalog installs");

        let requested = [
            "shared".into(),
            "masked".into(),
            "left-only".into(),
            "right-only".into(),
        ];
        let left = registry.registrations_for_pipeline(
            &"left".into(),
            &PipelineExtensions::default(),
            &requested,
        );
        assert_eq!(provider(&left, "shared").name(), "left");
        assert_eq!(
            left.known_extensions,
            HashSet::from(["shared".into(), "masked".into(), "left-only".into()])
        );
        let error = resolve(&left, "masked")
            .expect_err("a background declaration must not expose the ancestor's capability");
        assert!(
            error
                .to_string()
                .contains("extension 'masked' does not provide it")
        );
        assert!(resolve(&left, "right-only").is_err());

        let right = registry.registrations_for_pipeline(
            &"right".into(),
            &PipelineExtensions::default(),
            &requested,
        );
        assert_eq!(provider(&right, "shared").name(), "engine");
        assert_eq!(provider(&right, "masked").name(), "engine-masked");
        assert_eq!(provider(&right, "right-only").name(), "right-only");
        assert!(resolve(&right, "left-only").is_err());

        let missing = registry.registrations_for_pipeline(
            &"missing".into(),
            &PipelineExtensions::default(),
            &requested,
        );
        assert_eq!(
            missing.known_extensions,
            HashSet::from(["shared".into(), "masked".into()])
        );
        assert_eq!(provider(&missing, "shared").name(), "engine");
        assert_eq!(provider(&missing, "masked").name(), "engine-masked");
    }

    /// Scenario: a pipeline shadows inherited declarations after cloning a group snapshot.
    /// Guarantees: both filtering entry points remove the whole ID without mutating sibling snapshots or catalogs.
    #[test]
    fn pipeline_shadowing_only_filters_its_own_snapshot() {
        let registry = ExtensionScopeRegistry::default();
        let group = PipelineGroupId::from("group");
        registry
            .install(ExtensionScopeCatalog {
                engine: catalog(
                    vec![registration("shared", Provider::new("engine"))],
                    &["background"],
                ),
                groups: HashMap::from([(
                    group.clone(),
                    catalog(vec![registration("shared", Provider::new("group"))], &[]),
                )]),
            })
            .expect("catalog installs");
        let requested = ["shared".into(), "background".into()];
        let original =
            registry.registrations_for_pipeline(&group, &PipelineExtensions::default(), &requested);
        let mut filtered = original.clone();
        let config = ExtensionUserConfig::with_type("urn:test:extension:background");
        let mut pipeline_extensions = PipelineExtensions::default();
        pipeline_extensions.insert("shared".into(), config.clone());
        filtered.remove_shadowed_by(&pipeline_extensions);
        let direct = registry.registrations_for_pipeline(&group, &pipeline_extensions, &requested);
        for snapshot in [&filtered, &direct] {
            assert_eq!(
                snapshot.known_extensions,
                HashSet::from(["background".into()])
            );
            assert!(snapshot.registrations.is_empty());
            assert!(!snapshot.is_empty(), "background declarations remain known");
            assert!(resolve(snapshot, "shared").is_err());
        }

        pipeline_extensions.insert("background".into(), config);
        filtered.remove_shadowed_by(&pipeline_extensions);
        assert!(filtered.is_empty());
        assert!(
            registry
                .registrations_for_pipeline(&group, &pipeline_extensions, &requested)
                .is_empty()
        );
        assert_eq!(provider(&original, "shared").name(), "group");
        let fresh =
            registry.registrations_for_pipeline(&group, &PipelineExtensions::default(), &requested);
        assert_eq!(fresh.known_extensions, original.known_extensions);
        assert_eq!(provider(&fresh, "shared").name(), "group");
    }

    /// Scenario: many ancestor providers are visible but pipeline bindings use zero, one, or repeated IDs.
    /// Guarantees: unused factories are never cloned or registered, and repeated bindings copy one registration.
    #[test]
    fn sparse_bindings_do_not_clone_unused_provider_factories() {
        let registry = ExtensionScopeRegistry::default();
        let used = Provider::new("used");
        let used_clones = used.clones.clone();
        let unused = Provider::new("unused");
        let unused_clones = unused.clones.clone();
        registry
            .install(ExtensionScopeCatalog {
                engine: catalog(
                    vec![registration("used", used), registration("unused", unused)],
                    &["background"],
                ),
                ..Default::default()
            })
            .expect("catalog installs");
        for requested in [
            vec![],
            vec!["used".into()],
            vec!["used".into(), "used".into()],
        ] {
            used_clones.store(0, Ordering::SeqCst);
            let snapshot = registry.registrations_for_pipeline(
                &"group".into(),
                &PipelineExtensions::default(),
                &requested,
            );
            assert_eq!(snapshot.known_extensions.len(), 3);
            assert_eq!(
                snapshot.registrations.len(),
                usize::from(!requested.is_empty())
            );
            assert_eq!(
                used_clones.load(Ordering::SeqCst),
                u64::from(!requested.is_empty())
            );
            for _ in 0..16 {
                let core_snapshot = snapshot.clone();
                let mut capabilities = CapabilityRegistry::new();
                core_snapshot
                    .register_into(&mut capabilities)
                    .expect("core registration");
                assert!(
                    capabilities
                        .get_shared(&TypeId::of::<NoOpStateless>(), "unused")
                        .is_none()
                );
                assert_eq!(
                    capabilities
                        .get_shared(&TypeId::of::<NoOpStateless>(), "used")
                        .is_some(),
                    !requested.is_empty()
                );
            }
            assert_eq!(unused_clones.load(Ordering::SeqCst), 0);
        }
    }

    /// Scenario: inherited providers are registered into an already populated capability registry.
    /// Guarantees: duplicate registration is surfaced with its extension ID and leaves the existing entry intact.
    #[test]
    fn inherited_registration_failure_names_the_extension() {
        let inherited = InheritedExtensionRegistrations {
            known_extensions: HashSet::from(["provider".into()]),
            registrations: vec![registration("provider", Provider::new("original"))],
        };
        let mut registry = CapabilityRegistry::new();
        inherited
            .register_into(&mut registry)
            .expect("initial registration succeeds");
        let error = inherited
            .register_into(&mut registry)
            .expect_err("duplicate registration fails");
        assert!(matches!(
            error,
            Error::CapabilityRegistrationFailed { extension, message }
                if extension.as_ref() == "provider"
                    && message.contains("duplicate shared capability registration")
        ));
        assert!(
            registry
                .get_shared(&TypeId::of::<NoOpStateless>(), "provider")
                .is_some()
        );
    }
}
