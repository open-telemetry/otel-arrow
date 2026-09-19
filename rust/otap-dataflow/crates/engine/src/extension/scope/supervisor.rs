// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Startup, supervision, and ordered shutdown for extension scope hosts.
//!
//! Extensions declared at engine or pipeline-group scope are instantiated once on a
//! controller-owned `LocalSet`. Only their shared variant is retained because
//! every inherited capability handle may cross pipeline-thread boundaries.

use super::host::{
    ExtensionHostRuntimePolicy, PreparedExtensionScopeHost, RunningExtensionScopeHost, ScopeEvent,
};
use super::registry::{
    ExtensionScopeCatalog, ExtensionScopeRegistry, ScopeCatalog, SharedExtensionRegistration,
};
use crate::PipelineFactory;
use crate::channel_metrics::ChannelMetricsRegistry;
use crate::config::ExtensionConfig;
use crate::context::{ControllerContext, ExtensionContext};
use crate::entity_context::{EntityTelemetryGuard, EntityTelemetryHandle};
use crate::error::Error;
use crate::extension_lifecycle::{EXTENSION_SHUTDOWN_DRAIN_SLACK, EXTENSION_SHUTDOWN_GRACE};
use futures::stream::{FuturesUnordered, StreamExt};
use otel_arrow_dfe_config::PipelineGroupId;
use otel_arrow_dfe_config::engine::OtelDataflowSpec;
use otel_arrow_dfe_config::extension::ExtensionDeclarationScope;
use otel_arrow_dfe_config::pipeline::PipelineExtensions;
use otel_arrow_dfe_config::policy::{Policies, ResolvedPolicies};
use otel_arrow_dfe_telemetry::otel_warn;
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::future::Future;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot};
use tokio::task;

/// Prepared controller-owned extension scopes.
///
/// Construction is synchronous and does not spawn tasks. Call [`start`](Self::start)
/// from inside the host thread's `LocalSet`.
pub struct PreparedExtensionScopeSupervisor {
    engine_scope: Option<PreparedExtensionScopeHost>,
    group_scopes: Vec<(PipelineGroupId, PreparedExtensionScopeHost)>,
    catalog: ExtensionScopeCatalog,
    registry: ExtensionScopeRegistry,
    metrics_reporter: MetricsReporter,
}

impl PreparedExtensionScopeSupervisor {
    /// Starts the extension scope hosts without a startup cancellation signal.
    pub async fn start(self) -> Result<RunningExtensionScopeSupervisor, Error> {
        self.start_with_shutdown(std::future::pending())
            .await?
            .ok_or_else(|| Error::InternalError {
                message: "uncancelled extension scope startup was cancelled".to_owned(),
            })
    }

    /// Starts every engine and group extension, waits for spawn and readiness
    /// barriers, and publishes the immutable capability catalog.
    pub async fn start_with_shutdown<F>(
        self,
        shutdown: F,
    ) -> Result<Option<RunningExtensionScopeSupervisor>, Error>
    where
        F: Future<Output = ()>,
    {
        let Self {
            engine_scope,
            group_scopes,
            catalog,
            registry,
            metrics_reporter,
        } = self;
        let (events_tx, events_rx) = mpsc::unbounded_channel();
        let mut supervisor = ScopeTaskSupervisor::new(events_rx);
        let mut shutdown = Box::pin(shutdown);

        if let Some(engine_scope) = engine_scope {
            let scope = ExtensionDeclarationScope::Engine;
            supervisor.spawn(
                scope.clone(),
                engine_scope.start(&metrics_reporter),
                events_tx.clone(),
            );
            let mut pending = HashSet::from([scope]);
            match supervisor
                .wait_until_ready(&mut pending, shutdown.as_mut())
                .await
            {
                Ok(StartupWaitOutcome::Ready) => {}
                Ok(StartupWaitOutcome::Cancelled) => {
                    let mut first_error = None;
                    supervisor.shutdown_ordered(&mut first_error).await;
                    return first_error.map_or(Ok(None), Err);
                }
                Err(error) => {
                    let mut first_error = Some(error);
                    supervisor.shutdown_ordered(&mut first_error).await;
                    return Err(first_error.expect("startup failure must be preserved"));
                }
            }
        }

        // Every group starts only after the engine-level barrier succeeds.
        // Groups are peers, so they start concurrently.
        let mut pending_groups = HashSet::new();
        for (pipeline_group_id, group_scope) in group_scopes {
            let scope = ExtensionDeclarationScope::PipelineGroup(pipeline_group_id);
            let _ = pending_groups.insert(scope.clone());
            supervisor.spawn(
                scope,
                group_scope.start(&metrics_reporter),
                events_tx.clone(),
            );
        }
        drop(events_tx);
        match supervisor
            .wait_until_ready(&mut pending_groups, shutdown.as_mut())
            .await
        {
            Ok(StartupWaitOutcome::Ready) => {}
            Ok(StartupWaitOutcome::Cancelled) => {
                let mut first_error = None;
                supervisor.shutdown_ordered(&mut first_error).await;
                return first_error.map_or(Ok(None), Err);
            }
            Err(error) => {
                let mut first_error = Some(error);
                supervisor.shutdown_ordered(&mut first_error).await;
                return Err(first_error.expect("startup failure must be preserved"));
            }
        }

        if let Err(error) = registry.install(catalog) {
            let mut first_error = Some(error);
            supervisor.shutdown_ordered(&mut first_error).await;
            return Err(first_error.expect("catalog installation failure must be preserved"));
        }

        Ok(Some(RunningExtensionScopeSupervisor { supervisor }))
    }
}

/// Running controller-owned engine and pipeline-group extension scopes.
pub struct RunningExtensionScopeSupervisor {
    supervisor: ScopeTaskSupervisor,
}

impl RunningExtensionScopeSupervisor {
    /// Combined grace for the parallel group phase followed by the engine phase.
    ///
    /// The controller must budget this separately from descendant pipeline
    /// draining, and allow additional time for thread coordination and observability.
    pub const SHUTDOWN_TIMEOUT: Duration = EXTENSION_SHUTDOWN_GRACE
        .saturating_add(EXTENSION_SHUTDOWN_DRAIN_SLACK)
        .saturating_mul(2);

    /// Drives all scopes until cancellation or the first extension failure.
    ///
    /// On failure, `notify_failure` runs before any surviving scope is stopped.
    /// Providers remain alive until `shutdown` completes so the controller can
    /// drain descendant pipelines first.
    pub async fn run<F, N>(mut self, shutdown: F, notify_failure: N) -> Result<(), Error>
    where
        F: Future<Output = ()> + 'static,
        N: FnOnce(String),
    {
        if self.supervisor.tasks.is_empty() {
            shutdown.await;
            return Ok(());
        }

        let mut first_error = None;
        let mut notify_failure = Some(notify_failure);
        let mut events_open = true;
        let mut shutdown = Box::pin(shutdown);
        loop {
            tokio::select! {
                biased;
                event = self.supervisor.events_rx.recv(), if events_open => {
                    match event {
                        Some(ScopeEvent::Failed { scope, error }) => {
                            Self::observe_failure(
                                scope,
                                error,
                                &mut first_error,
                                &mut notify_failure,
                            );
                        }
                        Some(ScopeEvent::Ready(scope)) => {
                            Self::observe_failure(
                                scope.clone(),
                                Error::InternalError {
                                    message: format!(
                                        "{scope} reported readiness after extension scope startup"
                                    ),
                                },
                                &mut first_error,
                                &mut notify_failure,
                            );
                        }
                        None => {
                            events_open = false;
                            if !self.supervisor.tasks.is_empty() {
                                Self::observe_failure(
                                    ExtensionDeclarationScope::Engine,
                                    Error::InternalError {
                                        message: "extension scope event channel closed while scope hosts were still running".to_owned(),
                                    },
                                    &mut first_error,
                                    &mut notify_failure,
                                );
                            }
                        }
                    }
                }
                Some(joined) = self.supervisor.tasks.next(), if !self.supervisor.tasks.is_empty() => {
                    let (scope, error) = self.supervisor.route_completion(joined);
                    if let Some(error) = error {
                        Self::observe_failure(
                            scope.unwrap_or(ExtensionDeclarationScope::Engine),
                            error,
                            &mut first_error,
                            &mut notify_failure,
                        );
                    }
                }
                _ = shutdown.as_mut() => break,
            }
        }

        self.supervisor.shutdown_ordered(&mut first_error).await;
        first_error.map_or(Ok(()), Err)
    }

    /// Stops all scope hosts in declaration order without reporting a runtime failure.
    pub async fn shutdown(mut self) -> Result<(), Error> {
        let mut first_error = None;
        self.supervisor.shutdown_ordered(&mut first_error).await;
        first_error.map_or(Ok(()), Err)
    }

    fn observe_failure<N>(
        scope: ExtensionDeclarationScope,
        error: Error,
        first_error: &mut Option<Error>,
        notify_failure: &mut Option<N>,
    ) where
        N: FnOnce(String),
    {
        if first_error.is_none() {
            if let Some(notify_failure) = notify_failure.take() {
                notify_failure(error.to_string());
            }
            *first_error = Some(error);
        } else {
            otel_warn!(
                "extension_scope.host.secondary_failure",
                scope = scope.to_string(),
                error = error.to_string()
            );
        }
    }
}

enum StartupWaitOutcome {
    Ready,
    Cancelled,
}

struct ScopeTaskSupervisor {
    events_rx: mpsc::UnboundedReceiver<ScopeEvent>,
    tasks: FuturesUnordered<task::JoinHandle<(ExtensionDeclarationScope, Result<(), Error>)>>,
    task_ids: HashMap<task::Id, ExtensionDeclarationScope>,
    shutdown_senders: HashMap<ExtensionDeclarationScope, oneshot::Sender<Instant>>,
    shutdown_requested: HashSet<ExtensionDeclarationScope>,
    completed: HashSet<ExtensionDeclarationScope>,
    engine_scope: Option<ExtensionDeclarationScope>,
    group_scopes: Vec<ExtensionDeclarationScope>,
}

impl ScopeTaskSupervisor {
    fn new(events_rx: mpsc::UnboundedReceiver<ScopeEvent>) -> Self {
        Self {
            events_rx,
            tasks: FuturesUnordered::new(),
            task_ids: HashMap::new(),
            shutdown_senders: HashMap::new(),
            shutdown_requested: HashSet::new(),
            completed: HashSet::new(),
            engine_scope: None,
            group_scopes: Vec::new(),
        }
    }

    fn spawn(
        &mut self,
        scope: ExtensionDeclarationScope,
        running_scope: RunningExtensionScopeHost,
        events: mpsc::UnboundedSender<ScopeEvent>,
    ) {
        match &scope {
            ExtensionDeclarationScope::Engine => self.engine_scope = Some(scope.clone()),
            ExtensionDeclarationScope::PipelineGroup(_) => self.group_scopes.push(scope.clone()),
            ExtensionDeclarationScope::Pipeline(_, _) => {
                unreachable!("pipeline-scoped extensions are hosted by runtime pipelines")
            }
        }
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let task_scope = scope.clone();
        let handle = task::spawn_local(async move {
            let result = running_scope
                .run(task_scope.clone(), events, shutdown_rx)
                .await;
            (task_scope, result)
        });
        let _ = self.task_ids.insert(handle.id(), scope.clone());
        self.tasks.push(handle);
        let _ = self.shutdown_senders.insert(scope, shutdown_tx);
    }

    async fn wait_until_ready<F>(
        &mut self,
        pending: &mut HashSet<ExtensionDeclarationScope>,
        mut shutdown: std::pin::Pin<&mut F>,
    ) -> Result<StartupWaitOutcome, Error>
    where
        F: Future<Output = ()>,
    {
        while !pending.is_empty() {
            tokio::select! {
                biased;
                event = self.events_rx.recv() => {
                    match event {
                        Some(ScopeEvent::Ready(scope)) => {
                            if !pending.remove(&scope) {
                                return Err(Error::InternalError {
                                    message: format!(
                                        "received unexpected or duplicate readiness from {scope}"
                                    ),
                                });
                            }
                        }
                        Some(ScopeEvent::Failed { error, .. }) => return Err(error),
                        None => {
                            return Err(Error::InternalError {
                                message: "extension scope event channel closed during startup".to_owned(),
                            });
                        }
                    }
                }
                Some(joined) = self.tasks.next(), if !self.tasks.is_empty() => {
                    let (scope, error) = self.route_completion(joined);
                    return Err(error.unwrap_or_else(|| Error::InternalError {
                        message: scope.map_or_else(
                            || "unknown extension scope host exited during startup".to_owned(),
                            |scope| format!("{scope} exited during startup"),
                        ),
                    }));
                }
                _ = shutdown.as_mut() => return Ok(StartupWaitOutcome::Cancelled),
            }
        }
        Ok(StartupWaitOutcome::Ready)
    }

    fn request_shutdown(&mut self, scope: &ExtensionDeclarationScope, deadline: Instant) {
        if self.completed.contains(scope) || !self.shutdown_requested.insert(scope.clone()) {
            return;
        }
        if let Some(shutdown_tx) = self.shutdown_senders.remove(scope) {
            let _ = shutdown_tx.send(deadline);
        }
    }

    async fn shutdown_ordered(&mut self, first_error: &mut Option<Error>) {
        let group_scopes = self.group_scopes.clone();
        let group_deadline = Instant::now() + EXTENSION_SHUTDOWN_GRACE;
        for scope in &group_scopes {
            self.request_shutdown(scope, group_deadline);
        }
        self.drain_scopes(&group_scopes, first_error).await;

        if let Some(engine_scope) = self.engine_scope.clone() {
            self.request_shutdown(&engine_scope, Instant::now() + EXTENSION_SHUTDOWN_GRACE);
            self.drain_scopes(&[engine_scope], first_error).await;
        }

        while let Ok(event) = self.events_rx.try_recv() {
            if let ScopeEvent::Failed { error, .. } = event
                && first_error.is_none()
            {
                *first_error = Some(error);
            }
        }
    }

    async fn drain_scopes(
        &mut self,
        scopes: &[ExtensionDeclarationScope],
        first_error: &mut Option<Error>,
    ) {
        let mut events_open = true;
        while scopes.iter().any(|scope| !self.completed.contains(scope)) {
            tokio::select! {
                biased;
                event = self.events_rx.recv(), if events_open => {
                    match event {
                        Some(ScopeEvent::Failed { error, .. }) if first_error.is_none() => {
                            *first_error = Some(error);
                        }
                        Some(_) => {}
                        None => events_open = false,
                    }
                }
                joined = self.tasks.next(), if !self.tasks.is_empty() => {
                    let Some(joined) = joined else {
                        break;
                    };
                    let (_, error) = self.route_completion(joined);
                    if first_error.is_none() {
                        *first_error = error;
                    }
                }
                else => break,
            }
        }
        if first_error.is_none() && scopes.iter().any(|scope| !self.completed.contains(scope)) {
            *first_error = Some(Error::InternalError {
                message: "extension scope host tasks disappeared during shutdown".to_owned(),
            });
        }
    }

    fn route_completion(
        &mut self,
        joined: Result<(ExtensionDeclarationScope, Result<(), Error>), task::JoinError>,
    ) -> (Option<ExtensionDeclarationScope>, Option<Error>) {
        match joined {
            Ok((scope, result)) => {
                self.task_ids
                    .retain(|_, mapped_scope| mapped_scope != &scope);
                let _ = self.completed.insert(scope.clone());
                let _ = self.shutdown_senders.remove(&scope);
                let shutdown_was_requested = self.shutdown_requested.contains(&scope);
                let error = match result {
                    Ok(()) if shutdown_was_requested => None,
                    Ok(()) => Some(Error::InternalError {
                        message: format!("{scope} exited before host shutdown"),
                    }),
                    Err(error) => Some(error),
                };
                (Some(scope), error)
            }
            Err(error) => {
                let scope = self.task_ids.remove(&error.id());
                if let Some(scope) = scope.as_ref() {
                    let _ = self.completed.insert(scope.clone());
                    let _ = self.shutdown_senders.remove(scope);
                }
                (
                    scope,
                    Some(Error::JoinTaskError {
                        is_canceled: error.is_cancelled(),
                        is_panic: error.is_panic(),
                        error: error.to_string(),
                    }),
                )
            }
        }
    }
}

impl<PData: 'static + Clone + Debug> PipelineFactory<PData> {
    /// Prepares all engine and group extension declarations for the dedicated
    /// extension-scope supervisor thread.
    pub fn prepare_extension_scope_hosts(
        &'static self,
        config: &OtelDataflowSpec,
        controller_context: &ControllerContext,
        metrics_reporter: MetricsReporter,
        registry: ExtensionScopeRegistry,
    ) -> Result<PreparedExtensionScopeSupervisor, Error> {
        let engine_declaration_scope = ExtensionDeclarationScope::Engine;
        let engine_policies = Policies::resolve([&config.policies]);
        let (engine_scope, engine_catalog) = self.prepare_extension_scope(
            &engine_declaration_scope,
            &config.extensions,
            controller_context.engine_extension_context(),
            &engine_policies,
        )?;

        let mut group_scopes = Vec::new();
        let mut catalog = ExtensionScopeCatalog {
            engine: engine_catalog,
            groups: HashMap::new(),
        };

        let mut groups: Vec<_> = config.groups.iter().collect();
        groups.sort_by(|(left, _), (right, _)| left.as_ref().cmp(right.as_ref()));
        for (pipeline_group_id, pipeline_group) in groups {
            let policies = pipeline_group.policies.as_ref().map_or_else(
                || Policies::resolve([&config.policies]),
                |group_policies| Policies::resolve([group_policies, &config.policies]),
            );
            let declaration_scope =
                ExtensionDeclarationScope::PipelineGroup(pipeline_group_id.clone());
            let (scope, group_catalog) = self.prepare_extension_scope(
                &declaration_scope,
                &pipeline_group.extensions,
                controller_context.pipeline_group_extension_context(pipeline_group_id.clone()),
                &policies,
            )?;
            if let Some(scope) = scope {
                group_scopes.push((pipeline_group_id.clone(), scope));
            }
            if !group_catalog.known_extensions.is_empty() {
                let _ = catalog
                    .groups
                    .insert(pipeline_group_id.clone(), group_catalog);
            }
        }

        Ok(PreparedExtensionScopeSupervisor {
            engine_scope,
            group_scopes,
            catalog,
            registry,
            metrics_reporter,
        })
    }

    fn prepare_extension_scope(
        &self,
        declaration_scope: &ExtensionDeclarationScope,
        extensions: &PipelineExtensions,
        context: ExtensionContext,
        policies: &ResolvedPolicies,
    ) -> Result<(Option<PreparedExtensionScopeHost>, ScopeCatalog), Error> {
        let runtime_policy = ExtensionHostRuntimePolicy::from_resolved(policies);
        let mut configured_extensions: Vec<_> = extensions.iter().collect();
        configured_extensions.sort_by(|(left, _), (right, _)| left.as_ref().cmp(right.as_ref()));

        let mut wrappers = Vec::with_capacity(configured_extensions.len());
        let mut channel_metrics = ChannelMetricsRegistry::default();
        let mut catalog = ScopeCatalog::default();
        for (extension_id, user_config) in configured_extensions {
            let _ = catalog.known_extensions.insert(extension_id.clone());
            let raw_urn = user_config.r#type.as_str();
            let factory = self
                .get_extension_factory_map()
                .get(raw_urn)
                .ok_or_else(|| Error::UnknownExtension {
                    plugin_urn: raw_urn.to_string(),
                })?;

            if factory
                .capabilities
                .as_ref()
                .is_some_and(|capabilities| capabilities.shared.is_empty())
            {
                return Err(Error::ExtensionDeclarationRequiresSharedVariant {
                    extension: extension_id.clone(),
                    declaration_scope: declaration_scope.clone(),
                });
            }

            let runtime_config = ExtensionConfig::with_control_channel_capacity(
                extension_id.clone(),
                runtime_policy.control_node_capacity(),
            );
            let mut bundle = (factory.create)(
                &context,
                extension_id.clone(),
                user_config.clone(),
                &runtime_config,
            )
            .map_err(|error| Error::ConfigError(Box::new(error)))?;
            let Some(mut shared) = bundle.take_shared() else {
                return Err(Error::ExtensionDeclarationRequiresSharedVariant {
                    extension: extension_id.clone(),
                    declaration_scope: declaration_scope.clone(),
                });
            };

            if let Some(capabilities) = factory.capabilities.as_ref() {
                let instance_factory = shared
                    .shared_instance_factory()
                    .expect("a shared wrapper always has a shared instance factory")
                    .clone();
                catalog.registrations.push(SharedExtensionRegistration {
                    extension_id: extension_id.clone(),
                    capabilities: capabilities.clone(),
                    instance_factory,
                });
            }

            let entity_key =
                context.register_extension_entity(extension_id.clone(), shared.variant());
            let entity_handle = EntityTelemetryHandle::new(context.metrics_registry(), entity_key);
            shared = shared.with_control_channel_metrics(
                &entity_handle,
                &context,
                &mut channel_metrics,
                runtime_policy.channel_metrics_enabled(),
            );
            shared = shared.with_entity_telemetry_guard(EntityTelemetryGuard::new(entity_handle));
            wrappers.push((shared, entity_key));
        }

        let scope = (!wrappers.is_empty()).then(|| PreparedExtensionScopeHost {
            context,
            extensions: wrappers,
            channel_metrics: channel_metrics.into_handles(),
            runtime_policy,
        });
        Ok((scope, catalog))
    }
}
