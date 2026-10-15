// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Focused tests for ancestor eligibility, startup barriers, and scope supervision.

use super::*;
use crate::ExtensionFactory;
use crate::capability::registry::CapabilityRegistry;
use crate::control::ExtensionControlMsg;
use crate::extension::wrapper::ExtensionVariant;
use crate::extension::{EffectHandler, ExtensionWrapper};
use crate::extension_capabilities;
use crate::local::extension as local_ext;
use crate::shared::extension as shared_ext;
use crate::terminal_state::TerminalState;
use crate::testing::capability::no_op_stateless::{
    LocalNoOpStateless, NoOpStateless, SharedNoOpStateless,
};
use async_trait::async_trait;
use otel_arrow_dfe_config::ExtensionId;
use otel_arrow_dfe_config::extension::ExtensionUserConfig;
use otel_arrow_dfe_telemetry::InternalTelemetrySystem;
use std::any::TypeId;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

const LOCAL_CAPABILITIES_URN: &str = "urn:test:extension:scope_local_capabilities";
const LOCAL_BACKGROUND_URN: &str = "urn:test:extension:scope_local_background";
const SHARED_BACKGROUND_URN: &str = "urn:test:extension:scope_shared_background";
const DUAL_URN: &str = "urn:test:extension:scope_dual";
// Paused readiness waits must not advance past the std::time-based shutdown budget.
const TEST_READINESS_TIMEOUT: Duration = Duration::from_millis(100);

#[derive(Clone, Default)]
struct SharedProvider {
    // Observe the actual host lifecycle through its test-only instance-factory clones.
    state: Arc<AtomicU64>,
}

#[async_trait]
impl SharedNoOpStateless for SharedProvider {
    fn name(&self) -> &str {
        "shared"
    }

    fn echo(&self, value: u64) -> u64 {
        value
    }

    async fn ping(&self) -> u64 {
        self.state.load(Ordering::SeqCst)
    }

    async fn echo_async(&self, value: String) -> String {
        value
    }
}

#[async_trait]
impl shared_ext::Extension for SharedProvider {
    async fn start(
        self: Box<Self>,
        mut control: shared_ext::ControlChannel,
        effects: EffectHandler,
    ) -> Result<TerminalState, Error> {
        self.state.store(1, Ordering::SeqCst);
        effects.signal_ready();
        loop {
            if let ExtensionControlMsg::Shutdown { .. } = control.recv().await? {
                self.state.store(2, Ordering::SeqCst);
                return Ok(TerminalState::default());
            }
        }
    }
}

#[derive(Clone)]
struct LocalProvider;

#[async_trait(?Send)]
impl LocalNoOpStateless for LocalProvider {
    fn name(&self) -> &str {
        "local"
    }

    fn echo(&self, value: u64) -> u64 {
        value
    }

    async fn ping(&self) -> u64 {
        0
    }

    async fn echo_async(&self, value: String) -> String {
        value
    }
}

#[async_trait(?Send)]
impl local_ext::Extension for LocalProvider {
    async fn start(
        self: Rc<Self>,
        _control: local_ext::ControlChannel,
        _effects: EffectHandler,
    ) -> Result<TerminalState, Error> {
        panic!("ancestor hosts must discard the local variant before spawning");
    }
}

static FACTORY: PipelineFactory<()> = PipelineFactory::new(
    &[],
    &[],
    &[],
    &[
        ExtensionFactory {
            name: LOCAL_CAPABILITIES_URN,
            description: "local-only capability metadata",
            documentation_url: "",
            capabilities: Some(extension_capabilities!(local: LocalProvider => [NoOpStateless])),
            create: |_, _, _, _| {
                panic!("local-only metadata must be rejected before invoking the factory")
            },
            validate_config: otel_arrow_dfe_config::validation::no_config,
        },
        ExtensionFactory {
            name: LOCAL_BACKGROUND_URN,
            description: "local-only background bundle",
            documentation_url: "",
            capabilities: None,
            create: |_, name, config, runtime| {
                Ok(ExtensionWrapper::builder(name, config, runtime)
                    .background()
                    .local(Rc::new(LocalProvider))
                    .build()
                    .expect("local background bundle builds"))
            },
            validate_config: otel_arrow_dfe_config::validation::no_config,
        },
        ExtensionFactory {
            name: SHARED_BACKGROUND_URN,
            description: "shared background bundle",
            documentation_url: "",
            capabilities: None,
            create: |_, name, config, runtime| {
                Ok(ExtensionWrapper::builder(name, config, runtime)
                    .background()
                    .with_readiness_probe()
                    .shared(SharedProvider::default())
                    .build()
                    .expect("shared background bundle builds"))
            },
            validate_config: otel_arrow_dfe_config::validation::no_config,
        },
        ExtensionFactory {
            name: DUAL_URN,
            description: "active local and shared providers",
            documentation_url: "",
            capabilities: Some(extension_capabilities!(
                (shared: SharedProvider, local: LocalProvider) => [NoOpStateless]
            )),
            create: |_, name, config, runtime| {
                Ok(ExtensionWrapper::builder(name, config, runtime)
                    .active()
                    .with_readiness_probe()
                    .shared(SharedProvider::default())
                    .local(Rc::new(LocalProvider))
                    .build()
                    .expect("dual-provider bundle builds"))
            },
            validate_config: otel_arrow_dfe_config::validation::no_config,
        },
    ],
);

fn context_for(
    controller: &ControllerContext,
    scope: &ExtensionDeclarationScope,
) -> ExtensionContext {
    match scope {
        ExtensionDeclarationScope::Engine => controller.engine_extension_context(),
        ExtensionDeclarationScope::PipelineGroup(group) => {
            controller.pipeline_group_extension_context(group.clone())
        }
        ExtensionDeclarationScope::Pipeline(_, _) => panic!("not an ancestor scope"),
    }
}

fn assert_ancestor_rejected(urn: &'static str) {
    let telemetry = InternalTelemetrySystem::default();
    let controller = ControllerContext::new(telemetry.registry());
    let mut extensions = PipelineExtensions::default();
    extensions.insert("local-only".into(), ExtensionUserConfig::with_type(urn));
    for scope in [
        ExtensionDeclarationScope::Engine,
        ExtensionDeclarationScope::PipelineGroup("group".into()),
    ] {
        let error = FACTORY
            .prepare_extension_scope(
                &scope,
                &extensions,
                context_for(&controller, &scope),
                &Policies::resolve([&Policies::default()]),
            )
            .err()
            .expect("an ancestor declaration must retain a shared variant");
        assert!(matches!(
            error,
            Error::ExtensionDeclarationRequiresSharedVariant {
                extension,
                declaration_scope,
            } if extension.as_ref() == "local-only" && declaration_scope == scope
        ));
    }
}

async fn with_runtime<F>(test: impl FnOnce(InternalTelemetrySystem) -> F)
where
    F: Future<Output = ()>,
{
    task::LocalSet::new()
        .run_until(async {
            let telemetry = InternalTelemetrySystem::default();
            let collector = task::spawn_local(telemetry.collector().run_collection_loop());
            tokio::time::timeout(Duration::from_secs(30), test(telemetry))
                .await
                .expect("scope scenario must complete without hanging");
            collector.abort();
            assert!(
                collector
                    .await
                    .expect_err("collector remains alive until explicitly stopped")
                    .is_cancelled()
            );
        })
        .await;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Command {
    Ready,
    Exit,
    Fail,
    Panic,
    Ping,
    FinishShutdown,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Event {
    Started(&'static str),
    Ready(&'static str),
    Shutdown(&'static str),
    Stopped(&'static str),
    Alive(&'static str),
}

#[derive(Clone)]
struct ControlledExtension {
    name: &'static str,
    commands: async_channel::Receiver<Command>,
    events: mpsc::Sender<Event>,
    hold_shutdown: bool,
}

#[async_trait]
impl shared_ext::Extension for ControlledExtension {
    async fn start(
        self: Box<Self>,
        mut control: shared_ext::ControlChannel,
        effects: EffectHandler,
    ) -> Result<TerminalState, Error> {
        self.events
            .send(Event::Started(self.name))
            .await
            .expect("event receiver");
        loop {
            tokio::select! {
                message = control.recv() => {
                    if let ExtensionControlMsg::Shutdown { .. } = message? {
                        self.events.send(Event::Shutdown(self.name)).await.expect("event receiver");
                        if self.hold_shutdown {
                            assert_eq!(
                                self.commands.recv().await.expect("shutdown release"),
                                Command::FinishShutdown,
                            );
                        }
                        self.events.send(Event::Stopped(self.name)).await.expect("event receiver");
                        return Ok(TerminalState::default());
                    }
                }
                command = self.commands.recv() => {
                    match command.expect("test driver retains its command sender") {
                        Command::Ready => {
                            effects.signal_ready();
                            self.events.send(Event::Ready(self.name)).await.expect("event receiver");
                        }
                        Command::Exit => return Ok(TerminalState::default()),
                        Command::Fail => return Err(Error::InternalError {
                            message: format!("{} failed", self.name),
                        }),
                        Command::Panic => panic!("controlled extension panic"),
                        Command::Ping => {
                            self.events.send(Event::Alive(self.name)).await.expect("event receiver");
                        }
                        Command::FinishShutdown => panic!("shutdown was not requested"),
                    }
                }
            }
        }
    }
}

struct Driver {
    registry: ExtensionScopeRegistry,
    commands: HashMap<&'static str, async_channel::Sender<Command>>,
    events: mpsc::Receiver<Event>,
}

impl Driver {
    fn send(&self, name: &'static str, command: Command) {
        self.commands
            .get(name)
            .expect("configured extension")
            .try_send(command)
            .expect("bounded command queue has space");
    }

    async fn expect(&mut self, event: Event) {
        assert_eq!(self.events.recv().await, Some(event));
    }

    fn take_events(&mut self) -> Vec<Event> {
        let mut events = Vec::new();
        while let Ok(event) = self.events.try_recv() {
            events.push(event);
        }
        events
    }

    fn known_extensions(&self, group: &'static str) -> HashSet<ExtensionId> {
        let inherited = self.registry.registrations_for_pipeline(
            &group.into(),
            &PipelineExtensions::default(),
            std::iter::empty(),
        );
        let mut known = HashSet::new();
        inherited.extend_known_extensions(&mut known);
        known
    }

    fn assert_unpublished(&self) {
        for group in self.commands.keys().copied().chain(["missing"]) {
            assert!(
                self.registry
                    .registrations_for_pipeline(
                        &group.into(),
                        &PipelineExtensions::default(),
                        std::iter::empty()
                    )
                    .is_empty(),
                "no partial catalog may be visible from {group}"
            );
        }
    }

    fn assert_installable(&self) {
        self.assert_unpublished();
        self.registry
            .install(ExtensionScopeCatalog::default())
            .expect("cancelled or failed startup must not install even an empty catalog");
    }
}

fn abort_scope_host(
    running: &RunningExtensionScopeSupervisor,
    scope: &ExtensionDeclarationScope,
) -> task::Id {
    let host = running
        .supervisor
        .tasks
        .iter()
        .find(|host| running.supervisor.task_ids.get(&host.id()) == Some(scope))
        .expect("registered scope host task");
    let id = host.id();
    host.abort();
    id
}

async fn stop_extension_after_host_abort(
    control: crate::control::ExtensionControlSender,
    driver: &mut Driver,
    name: &'static str,
) {
    // Aborting the host bypasses its normal drain; explicitly clean up its extension task.
    control
        .send(ExtensionControlMsg::Shutdown {
            deadline: Instant::now() + EXTENSION_SHUTDOWN_GRACE,
            reason: "test cleanup after scope host cancellation".into(),
        })
        .await
        .expect("extension control remains available for test cleanup");
    driver.expect(Event::Shutdown(name)).await;
    driver.expect(Event::Stopped(name)).await;
}

fn prepared_scopes(
    telemetry: &InternalTelemetrySystem,
    with_engine: bool,
    groups: &[(&'static str, bool)],
) -> (PreparedExtensionScopeSupervisor, Driver) {
    let registry = ExtensionScopeRegistry::default();
    let controller = ControllerContext::new(telemetry.registry());
    let (events_tx, events_rx) = mpsc::channel(32);
    let mut commands = HashMap::new();
    let mut prepared = PreparedExtensionScopeSupervisor {
        engine_scope: None,
        group_scopes: Vec::new(),
        catalog: ExtensionScopeCatalog::default(),
        registry: registry.clone(),
        metrics_reporter: telemetry.reporter(),
    };
    let mut scopes = Vec::new();
    if with_engine {
        scopes.push((ExtensionDeclarationScope::Engine, "engine", false));
    }
    scopes.extend(groups.iter().map(|(name, hold_shutdown)| {
        (
            ExtensionDeclarationScope::PipelineGroup((*name).into()),
            *name,
            *hold_shutdown,
        )
    }));
    for (scope, name, hold_shutdown) in scopes {
        let context = context_for(&controller, &scope);
        let (command_tx, command_rx) = async_channel::bounded(4);
        let _ = commands.insert(name, command_tx);
        let mut bundle = ExtensionWrapper::builder(
            name.into(),
            Arc::new(ExtensionUserConfig::with_type(SHARED_BACKGROUND_URN)),
            &ExtensionConfig::new(name),
        )
        .background()
        .with_readiness_probe_timeout_override(TEST_READINESS_TIMEOUT)
        .shared(ControlledExtension {
            name,
            commands: command_rx,
            events: events_tx.clone(),
            hold_shutdown,
        })
        .build()
        .expect("controlled shared background builds");
        let mut shared = bundle.take_shared().expect("shared wrapper");
        let entity_key = context.register_extension_entity(name.into(), shared.variant());
        shared = shared.with_entity_telemetry_guard(EntityTelemetryGuard::new(
            EntityTelemetryHandle::new(context.metrics_registry(), entity_key),
        ));
        let mut policies = Policies::resolve([&Policies::default()]);
        policies.telemetry.pipeline_metrics = false;
        policies.telemetry.runtime_metrics = otel_arrow_dfe_config::MetricLevel::None;
        let host = PreparedExtensionScopeHost {
            context,
            extensions: vec![(shared, entity_key)],
            channel_metrics: Vec::new(),
            runtime_policy: ExtensionHostRuntimePolicy::from_resolved(&policies),
        };
        let catalog = ScopeCatalog {
            known_extensions: HashSet::from([name.into()]),
            registrations: Vec::new(),
        };
        match scope {
            ExtensionDeclarationScope::Engine => {
                prepared.engine_scope = Some(host);
                prepared.catalog.engine = catalog;
            }
            ExtensionDeclarationScope::PipelineGroup(group) => {
                prepared.group_scopes.push((group.clone(), host));
                let _ = prepared.catalog.groups.insert(group, catalog);
            }
            ExtensionDeclarationScope::Pipeline(_, _) => unreachable!(),
        }
    }
    (
        prepared,
        Driver {
            registry,
            commands,
            events: events_rx,
        },
    )
}

/// Scenario: engine and group declarations advertise capabilities only on a local variant.
/// Guarantees: runtime preparation rejects the declaration with its scope before invoking its factory.
#[test]
fn ancestor_preparation_rejects_local_only_capability_metadata() {
    assert_ancestor_rejected(LOCAL_CAPABILITIES_URN);
}

/// Scenario: a capability-less background factory constructs only a local extension variant.
/// Guarantees: engine and group preparation inspect the actual bundle and reject it with the declaration identity.
#[test]
fn ancestor_preparation_rejects_local_only_background_bundles() {
    assert_ancestor_rejected(LOCAL_BACKGROUND_URN);
}

/// Scenario: engine and group scopes contain dual active providers and shared backgrounds without consumers.
/// Guarantees: shared variants remain hosted, local variants never start, and backgrounds remain known without registrations.
#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn ancestor_hosts_retain_shared_variants_without_consumers() {
    with_runtime(|telemetry| async move {
        let spec: OtelDataflowSpec = serde_yaml::from_str(&format!(
            r#"
version: otel_dataflow/v1
extensions:
  engine-active:
    type: "{DUAL_URN}"
  engine-background:
    type: "{SHARED_BACKGROUND_URN}"
groups:
  group:
    extensions:
      group-active:
        type: "{DUAL_URN}"
      group-background:
        type: "{SHARED_BACKGROUND_URN}"
"#
        ))
        .expect("scope-only configuration");
        let registry = ExtensionScopeRegistry::default();
        let prepared = FACTORY
            .prepare_extension_scope_hosts(
                &spec,
                &ControllerContext::new(telemetry.registry()),
                telemetry.reporter(),
                registry.clone(),
            )
            .expect("shared ancestor declarations prepare");
        assert_eq!(prepared.group_scopes.len(), 1);
        let mut provider_states = Vec::new();
        for host in [
            prepared.engine_scope.as_ref().expect("engine host"),
            &prepared.group_scopes[0].1,
        ] {
            assert_eq!(host.extensions.len(), 2);
            for (wrapper, _) in &host.extensions {
                assert_eq!(wrapper.variant(), ExtensionVariant::Shared);
                let provider = wrapper
                    .shared_instance_factory()
                    .expect("shared factory")
                    .produce()
                    .downcast::<SharedProvider>()
                    .expect("shared provider");
                assert_eq!(provider.state.load(Ordering::SeqCst), 0);
                provider_states.push(provider.state);
            }
        }
        assert!(
            registry
                .registrations_for_pipeline(
                    &"group".into(),
                    &PipelineExtensions::default(),
                    std::iter::empty()
                )
                .is_empty()
        );
        let running = prepared
            .start()
            .await
            .expect("only shared readiness probes are retained");
        assert_eq!(running.supervisor.tasks.len(), 2);
        assert!(
            provider_states
                .iter()
                .all(|state| state.load(Ordering::SeqCst) == 1),
            "every unconsumed active and background variant must actually start"
        );
        let inherited = registry.registrations_for_pipeline(
            &"group".into(),
            &PipelineExtensions::default(),
            std::iter::empty(),
        );
        let mut known = HashSet::new();
        inherited.extend_known_extensions(&mut known);
        assert_eq!(
            known,
            HashSet::from([
                "engine-active".into(),
                "engine-background".into(),
                "group-active".into(),
                "group-background".into(),
            ])
        );
        let mut capabilities = CapabilityRegistry::new();
        inherited
            .register_into(&mut capabilities)
            .expect("shared registration");
        for id in ["engine-active", "group-active"] {
            assert!(
                capabilities
                    .get_shared(&TypeId::of::<NoOpStateless>(), id)
                    .is_none()
            );
            assert!(
                capabilities
                    .get_local(&TypeId::of::<NoOpStateless>(), id)
                    .is_none()
            );
        }
        for id in ["engine-background", "group-background"] {
            assert!(
                capabilities
                    .get_shared(&TypeId::of::<NoOpStateless>(), id)
                    .is_none()
            );
        }
        running.shutdown().await.expect("unconsumed hosts drain");
        assert!(
            provider_states
                .iter()
                .all(|state| state.load(Ordering::SeqCst) == 2),
            "every retained shared variant must receive cooperative shutdown"
        );
    })
    .await;
}

/// Scenario: engine and group extensions explicitly delay their readiness signals.
/// Guarantees: groups start after the engine barrier, and no catalog is published until every group is ready.
#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn readiness_barriers_precede_groups_and_catalog_publication() {
    with_runtime(|telemetry| async move {
        let (prepared, mut driver) = prepared_scopes(&telemetry, true, &[("group", false)]);
        let startup = task::spawn_local(prepared.start());
        driver.expect(Event::Started("engine")).await;
        driver.assert_unpublished();
        assert!(driver.take_events().is_empty(), "group must not start yet");
        driver.send("engine", Command::Ready);
        driver.expect(Event::Ready("engine")).await;
        driver.expect(Event::Started("group")).await;
        driver.assert_unpublished();
        assert!(!startup.is_finished());
        driver.send("group", Command::Ready);
        driver.expect(Event::Ready("group")).await;
        let running = startup.await.expect("startup task").expect("ready scopes");
        assert_eq!(
            driver.known_extensions("group"),
            HashSet::from(["engine".into(), "group".into()])
        );
        assert_eq!(
            driver.known_extensions("missing"),
            HashSet::from(["engine".into()])
        );
        running.shutdown().await.expect("ordered shutdown");
        assert_eq!(
            driver.take_events(),
            [
                Event::Shutdown("group"),
                Event::Stopped("group"),
                Event::Shutdown("engine"),
                Event::Stopped("engine"),
            ]
        );
    })
    .await;
}

/// Scenario: cancellation is already available when a prepared engine/group startup is first polled.
/// Guarantees: startup returns cancellation, cleans up any started engine host, and never starts groups or installs a catalog.
#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn cancellation_before_startup_does_not_publish_or_start_groups() {
    with_runtime(|telemetry| async move {
        let (prepared, mut driver) = prepared_scopes(&telemetry, true, &[("group", false)]);
        let outcome = prepared
            .start_with_shutdown(std::future::ready(()))
            .await
            .expect("pre-cancelled startup drains");
        assert!(outcome.is_none());
        let events = driver.take_events();
        assert!(
            events.is_empty()
                || events
                    == [
                        Event::Started("engine"),
                        Event::Shutdown("engine"),
                        Event::Stopped("engine"),
                    ],
            "pre-cancellation may skip spawning, but any started host must drain: {events:?}"
        );
        driver.assert_installable();
    })
    .await;
}

/// Scenario: cancellation arrives after the engine extension starts but before it signals readiness.
/// Guarantees: the in-progress engine barrier is cancellable and its host drains without publishing or starting groups.
#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn cancellation_during_engine_barrier_drains_only_engine() {
    with_runtime(|telemetry| async move {
        let (prepared, mut driver) = prepared_scopes(&telemetry, true, &[("group", false)]);
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let startup = task::spawn_local(prepared.start_with_shutdown(async move {
            shutdown_rx.await.expect("test sends cancellation");
        }));
        driver.expect(Event::Started("engine")).await;
        driver.assert_unpublished();
        shutdown_tx.send(()).expect("startup is waiting");
        assert!(
            startup
                .await
                .expect("startup task")
                .expect("cancelled startup drains")
                .is_none()
        );
        assert_eq!(
            driver.take_events(),
            [Event::Shutdown("engine"), Event::Stopped("engine")]
        );
        driver.assert_installable();
    })
    .await;
}

/// Scenario: cancellation interrupts a group barrier with a ready peer and a group that delays draining.
/// Guarantees: both groups receive shutdown together, the engine survives until both drain, and no catalog is installed.
#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn cancellation_during_group_barrier_waits_for_peer_draining() {
    for with_engine in [false, true] {
        with_runtime(|telemetry| async move {
            let (prepared, mut driver) =
                prepared_scopes(&telemetry, with_engine, &[("group", true), ("peer", false)]);
            let (shutdown_tx, shutdown_rx) = oneshot::channel();
            let startup = task::spawn_local(prepared.start_with_shutdown(async move {
                shutdown_rx.await.expect("test sends cancellation");
            }));
            if with_engine {
                driver.expect(Event::Started("engine")).await;
                driver.send("engine", Command::Ready);
                driver.expect(Event::Ready("engine")).await;
            }
            let started = HashSet::from([
                driver.events.recv().await.expect("first group starts"),
                driver.events.recv().await.expect("second group starts"),
            ]);
            assert_eq!(
                started,
                HashSet::from([Event::Started("group"), Event::Started("peer")])
            );
            driver.send("peer", Command::Ready);
            driver.expect(Event::Ready("peer")).await;
            driver.assert_unpublished();
            shutdown_tx.send(()).expect("group barrier is waiting");
            let mut draining = HashSet::new();
            for _ in 0..3 {
                let _ = draining.insert(driver.events.recv().await.expect("group drain event"));
            }
            assert_eq!(
                draining,
                HashSet::from([
                    Event::Shutdown("group"),
                    Event::Shutdown("peer"),
                    Event::Stopped("peer"),
                ])
            );
            assert!(!startup.is_finished(), "group drain is still pending");
            assert!(
                driver.take_events().is_empty(),
                "engine must not stop early"
            );
            driver.send("group", Command::FinishShutdown);
            assert!(
                startup
                    .await
                    .expect("startup task")
                    .expect("cancelled startup drains")
                    .is_none()
            );
            let mut expected = vec![Event::Stopped("group")];
            if with_engine {
                expected.extend([Event::Shutdown("engine"), Event::Stopped("engine")]);
            }
            assert_eq!(driver.take_events(), expected);
            driver.assert_installable();
        })
        .await;
    }
}

/// Scenario: an engine or group host never signals readiness before its probe expires.
/// Guarantees: the named timeout survives cleanup, only started hosts drain in reverse scope order, and publication is skipped.
#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn readiness_failure_drains_started_scopes_without_publication() {
    for failing in ["engine", "group"] {
        with_runtime(|telemetry| async move {
            let (prepared, mut driver) = prepared_scopes(&telemetry, true, &[("group", false)]);
            let startup = task::spawn_local(prepared.start());
            driver.expect(Event::Started("engine")).await;
            if failing == "group" {
                driver.send("engine", Command::Ready);
                driver.expect(Event::Ready("engine")).await;
                driver.expect(Event::Started("group")).await;
            }
            driver.assert_unpublished();
            let error = startup
                .await
                .expect("startup task")
                .err()
                .expect("readiness timeout");
            assert!(matches!(
                error,
                Error::ExtensionReadinessTimeout { extension, variant, timeout }
                    if extension == failing
                        && variant == "shared"
                        && timeout == TEST_READINESS_TIMEOUT
            ));
            let mut expected = Vec::new();
            if failing == "group" {
                expected.extend([Event::Shutdown("group"), Event::Stopped("group")]);
            }
            expected.extend([Event::Shutdown("engine"), Event::Stopped("engine")]);
            assert_eq!(driver.take_events(), expected);
            driver.assert_installable();
        })
        .await;
    }
}

/// Scenario: all new hosts become ready but their registry already contains an immutable catalog.
/// Guarantees: installation fails without replacement, and the new group and engine hosts are cooperatively drained.
#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn catalog_installation_failure_drains_hosts_and_preserves_original() {
    with_runtime(|telemetry| async move {
        let (prepared, mut driver) = prepared_scopes(&telemetry, true, &[("group", false)]);
        driver
            .registry
            .install(ExtensionScopeCatalog {
                engine: ScopeCatalog {
                    known_extensions: HashSet::from(["original".into()]),
                    ..Default::default()
                },
                ..Default::default()
            })
            .expect("original catalog installs");
        driver.send("engine", Command::Ready);
        driver.send("group", Command::Ready);
        let error = prepared
            .start()
            .await
            .err()
            .expect("second installation fails");
        assert!(matches!(
            error,
            Error::InternalError { message }
                if message == "extension scope catalog was installed more than once"
        ));
        assert_eq!(
            driver.known_extensions("group"),
            HashSet::from(["original".into()])
        );
        assert_eq!(
            driver.take_events(),
            [
                Event::Started("engine"),
                Event::Ready("engine"),
                Event::Started("group"),
                Event::Ready("group"),
                Event::Shutdown("group"),
                Event::Stopped("group"),
                Event::Shutdown("engine"),
                Event::Stopped("engine"),
            ]
        );
    })
    .await;
}

/// Scenario: a ready group extension exits cleanly, returns an error, or panics while its engine provider is live.
/// Guarantees: each completion notifies failure without panicking the host or stopping survivors before descendant shutdown.
#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn runtime_failures_notify_before_surviving_scopes_shutdown() {
    for completion in [Command::Exit, Command::Fail, Command::Panic] {
        with_runtime(|telemetry| async move {
            let (prepared, mut driver) = prepared_scopes(&telemetry, true, &[("group", false)]);
            driver.send("engine", Command::Ready);
            driver.send("group", Command::Ready);
            let running = prepared.start().await.expect("scope startup");
            assert_eq!(
                driver.take_events(),
                [
                    Event::Started("engine"),
                    Event::Ready("engine"),
                    Event::Started("group"),
                    Event::Ready("group"),
                ]
            );
            let (shutdown_tx, shutdown_rx) = oneshot::channel();
            let (failure_tx, failure_rx) = oneshot::channel();
            let task = task::spawn_local(running.run(
                async move { shutdown_rx.await.expect("descendants drain first") },
                move |error| failure_tx.send(error).expect("failure observer"),
            ));
            driver.send("group", completion);
            let notification = failure_rx.await.expect("failure notification");
            driver.send("engine", Command::Ping);
            driver.expect(Event::Alive("engine")).await;
            assert!(
                !task.is_finished(),
                "supervisor must await descendant shutdown"
            );
            assert_eq!(
                driver.known_extensions("group"),
                HashSet::from(["engine".into(), "group".into()])
            );
            shutdown_tx.send(()).expect("descendants have drained");
            let error = task
                .await
                .expect("host supervisor must not panic")
                .expect_err("host failure");
            assert_eq!(notification, error.to_string());
            match completion {
                Command::Exit => assert!(matches!(
                    error,
                    Error::ExtensionExitedBeforeShutdown { extension } if extension == "group"
                )),
                Command::Fail => assert!(matches!(
                    error,
                    Error::InternalError { message } if message == "group failed"
                )),
                Command::Panic => assert!(matches!(
                    error,
                    Error::JoinTaskError {
                        is_panic: true,
                        is_canceled: false,
                        ..
                    }
                )),
                _ => unreachable!(),
            }
            assert_eq!(
                driver.take_events(),
                [Event::Shutdown("engine"), Event::Stopped("engine")]
            );
        })
        .await;
    }
}

/// Scenario: a ready group host's registered Tokio task is aborted while an engine host survives.
/// Guarantees: its join error retains cancellation identity, notifies the caller, and does not block later engine draining.
#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn cancelled_host_task_is_routed_without_stopping_surviving_scopes() {
    with_runtime(|telemetry| async move {
        let (prepared, mut driver) = prepared_scopes(&telemetry, true, &[("group", false)]);
        let group_control = prepared.group_scopes[0].1.extensions[0]
            .0
            .extension_control_sender()
            .expect("group extension control");
        driver.send("engine", Command::Ready);
        driver.send("group", Command::Ready);
        let running = prepared.start().await.expect("scope startup");
        assert_eq!(
            driver.take_events(),
            [
                Event::Started("engine"),
                Event::Ready("engine"),
                Event::Started("group"),
                Event::Ready("group"),
            ]
        );
        let cancelled_id = abort_scope_host(
            &running,
            &ExtensionDeclarationScope::PipelineGroup("group".into()),
        );
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let (failure_tx, failure_rx) = oneshot::channel();
        let supervisor_task = task::spawn_local(running.run(
            async move { shutdown_rx.await.expect("descendants finish draining") },
            move |error| failure_tx.send(error).expect("failure observer"),
        ));
        let notification = failure_rx.await.expect("host cancellation is reported");
        driver.send("engine", Command::Ping);
        driver.expect(Event::Alive("engine")).await;
        assert!(
            !supervisor_task.is_finished(),
            "a host join failure must not preempt descendant draining"
        );
        stop_extension_after_host_abort(group_control, &mut driver, "group").await;
        shutdown_tx.send(()).expect("descendants have drained");
        let error = supervisor_task
            .await
            .expect("supervisor handles the cancelled host")
            .expect_err("host cancellation is a runtime failure");
        assert_eq!(notification, error.to_string());
        assert!(matches!(
            error,
            Error::JoinTaskError {
                is_canceled: true,
                is_panic: false,
                error,
            } if error.contains(&cancelled_id.to_string())
        ));
        assert_eq!(
            driver.take_events(),
            [Event::Shutdown("engine"), Event::Stopped("engine")]
        );
    })
    .await;
}

/// Scenario: aborting the last ready host drops the real runtime event channel's final sender.
/// Guarantees: channel loss is reported once, survives the subsequent join error, and shutdown completes without hanging.
#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn runtime_event_channel_loss_after_startup_is_reported_and_drained() {
    with_runtime(|telemetry| async move {
        let (prepared, mut driver) = prepared_scopes(&telemetry, true, &[]);
        let engine_control = prepared
            .engine_scope
            .as_ref()
            .expect("engine host")
            .extensions[0]
            .0
            .extension_control_sender()
            .expect("engine extension control");
        driver.send("engine", Command::Ready);
        let mut running = prepared.start().await.expect("scope startup");
        assert_eq!(
            driver.take_events(),
            [Event::Started("engine"), Event::Ready("engine")]
        );
        let _ = abort_scope_host(&running, &ExtensionDeclarationScope::Engine);
        // Wait for actual sender destruction, leaving the completed host handle for run() to route.
        assert!(running.supervisor.events_rx.recv().await.is_none());
        assert_eq!(running.supervisor.tasks.len(), 1);
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let (failure_tx, failure_rx) = oneshot::channel();
        let supervisor_task = task::spawn_local(running.run(
            async move { shutdown_rx.await.expect("caller controls shutdown") },
            move |error| failure_tx.send(error).expect("failure observer"),
        ));
        let notification = failure_rx.await.expect("runtime channel loss is reported");
        assert!(notification.contains(
            "extension scope event channel closed while scope hosts were still running"
        ));
        assert!(
            !supervisor_task.is_finished(),
            "even loss of every host must wait for the caller's shutdown"
        );
        stop_extension_after_host_abort(engine_control, &mut driver, "engine").await;
        shutdown_tx.send(()).expect("caller finishes draining");
        let error = supervisor_task
            .await
            .expect("supervisor routes the last cancelled host")
            .expect_err("channel loss remains the primary error");
        assert_eq!(notification, error.to_string());
        assert!(matches!(
            error,
            Error::InternalError { message }
                if message == "extension scope event channel closed while scope hosts were still running"
        ));
        assert!(driver.take_events().is_empty());
    })
    .await;
}

/// Scenario: an already-started host sends an extra readiness event followed by a secondary failure.
/// Guarantees: late readiness is a reported protocol error, it stays the primary failure, and shutdown remains caller-controlled.
#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn late_readiness_is_reported_without_preempting_shutdown() {
    with_runtime(|telemetry| async move {
        let (mut prepared, mut driver) = prepared_scopes(&telemetry, true, &[]);
        let (events_tx, events_rx) = mpsc::unbounded_channel();
        let mut supervisor = ScopeTaskSupervisor::new(events_rx);
        supervisor.spawn(
            ExtensionDeclarationScope::Engine,
            prepared
                .engine_scope
                .take()
                .expect("engine host")
                .start(&telemetry.reporter()),
            events_tx.clone(),
        );
        driver.send("engine", Command::Ready);
        let mut pending = HashSet::from([ExtensionDeclarationScope::Engine]);
        let mut never = Box::pin(std::future::pending());
        assert!(matches!(
            supervisor
                .wait_until_ready(&mut pending, never.as_mut())
                .await
                .expect("ready host"),
            StartupWaitOutcome::Ready
        ));
        assert_eq!(
            driver.take_events(),
            [Event::Started("engine"), Event::Ready("engine")]
        );
        let running = RunningExtensionScopeSupervisor { supervisor };
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let (failure_tx, failure_rx) = oneshot::channel();
        let task = task::spawn_local(running.run(
            async move { shutdown_rx.await.expect("caller-controlled shutdown") },
            move |error| failure_tx.send(error).expect("failure observer"),
        ));
        assert!(
            events_tx
                .send(ScopeEvent::Ready(ExtensionDeclarationScope::Engine))
                .is_ok()
        );
        let notification = failure_rx.await.expect("late readiness is reported");
        assert!(
            notification.contains("engine scope reported readiness after extension scope startup")
        );
        assert!(
            events_tx
                .send(ScopeEvent::Failed {
                    scope: ExtensionDeclarationScope::Engine,
                    error: Error::InternalError {
                        message: "secondary failure".into()
                    },
                })
                .is_ok()
        );
        driver.send("engine", Command::Ping);
        driver.expect(Event::Alive("engine")).await;
        shutdown_tx.send(()).expect("caller finishes draining");
        let error = task
            .await
            .expect("supervisor task")
            .expect_err("late readiness is an error");
        assert_eq!(notification, error.to_string());
        assert_eq!(
            driver.take_events(),
            [Event::Shutdown("engine"), Event::Stopped("engine")]
        );
    })
    .await;
}

/// Scenario: a scope host returns success either before or after the supervisor requests shutdown.
/// Guarantees: early success becomes a scope-named failure, and completion removes shutdown senders without double-signalling.
#[tokio::test]
async fn clean_host_completion_requires_requested_shutdown() {
    for scope in [
        ExtensionDeclarationScope::Engine,
        ExtensionDeclarationScope::PipelineGroup("group".into()),
    ] {
        for requested in [false, true] {
            let (_events_tx, events_rx) = mpsc::unbounded_channel();
            let mut supervisor = ScopeTaskSupervisor::new(events_rx);
            let task = task::spawn(async {});
            let id = task.id();
            task.await.expect("identity task completes");
            let _ = supervisor.task_ids.insert(id, scope.clone());
            let (shutdown_tx, mut shutdown_rx) = oneshot::channel();
            let _ = supervisor
                .shutdown_senders
                .insert(scope.clone(), shutdown_tx);
            let deadline = Instant::now() + EXTENSION_SHUTDOWN_GRACE;
            if requested {
                supervisor.request_shutdown(&scope, deadline);
                supervisor.request_shutdown(&scope, deadline + Duration::from_secs(1));
                assert_eq!(
                    shutdown_rx.try_recv().expect("first shutdown signal"),
                    deadline
                );
            }
            let (completed_scope, error) =
                supervisor.route_completion(Ok((id, scope.clone(), Ok(()))));
            assert_eq!(completed_scope, Some(scope.clone()));
            assert!(supervisor.task_ids.is_empty());
            assert!(supervisor.completed.contains(&scope));
            assert!(!supervisor.shutdown_senders.contains_key(&scope));
            if requested {
                assert!(error.is_none());
            } else {
                assert!(matches!(
                    error,
                    Some(Error::InternalError { message })
                        if message == format!("{scope} exited before host shutdown")
                ));
                assert!(matches!(
                    shutdown_rx.try_recv(),
                    Err(oneshot::error::TryRecvError::Closed)
                ));
            }
        }
    }
}

/// Scenario: many group tasks drain while an engine task fails outside the requested group phase.
/// Guarantees: each task ID is removed, unrelated completions do not finish the phase early, and the first error survives.
#[tokio::test(flavor = "current_thread")]
async fn many_group_completions_are_removed_by_task_id() {
    task::LocalSet::new()
        .run_until(async {
            let (_events_tx, events_rx) = mpsc::unbounded_channel();
            let mut supervisor = ScopeTaskSupervisor::new(events_rx);
            for index in 0..1024 {
                let scope =
                    ExtensionDeclarationScope::PipelineGroup(format!("group-{index}").into());
                supervisor.group_scopes.push(scope.clone());
                let (shutdown_tx, shutdown_rx) = oneshot::channel();
                let _ = supervisor
                    .shutdown_senders
                    .insert(scope.clone(), shutdown_tx);
                let task_scope = scope.clone();
                let handle = task::spawn_local(async move {
                    let _ = shutdown_rx.await.expect("group receives shutdown");
                    (task::id(), task_scope, Ok(()))
                });
                let _ = supervisor.task_ids.insert(handle.id(), scope);
                supervisor.tasks.push(handle);
            }
            let engine = task::spawn_local(async {
                (
                    task::id(),
                    ExtensionDeclarationScope::Engine,
                    Err(Error::InternalError {
                        message: "engine failed while groups drain".into(),
                    }),
                )
            });
            let _ = supervisor
                .task_ids
                .insert(engine.id(), ExtensionDeclarationScope::Engine);
            supervisor.engine_scope = Some(ExtensionDeclarationScope::Engine);
            supervisor.tasks.push(engine);
            let mut error = None;
            tokio::time::timeout(
                Duration::from_secs(10),
                supervisor.shutdown_ordered(&mut error),
            )
            .await
            .expect("all groups drain");
            assert!(matches!(error, Some(Error::InternalError { message })
            if message == "engine failed while groups drain"));
            assert_eq!(supervisor.completed.len(), 1025);
            assert!(supervisor.task_ids.is_empty());
            assert!(supervisor.shutdown_senders.is_empty());
            assert!(supervisor.tasks.is_empty());
        })
        .await;
}

/// Scenario: startup receives readiness for an unknown scope, duplicate readiness, or loses its event channel.
/// Guarantees: protocol failures terminate the barrier instead of publishing an incomplete catalog or waiting forever.
#[tokio::test(flavor = "current_thread")]
async fn startup_barrier_rejects_invalid_readiness_and_closed_events() {
    for (events, expected) in [
        (
            vec![ScopeEvent::Ready(ExtensionDeclarationScope::PipelineGroup(
                "unknown".into(),
            ))],
            "unexpected or duplicate readiness",
        ),
        (
            vec![
                ScopeEvent::Ready(ExtensionDeclarationScope::Engine),
                ScopeEvent::Ready(ExtensionDeclarationScope::Engine),
            ],
            "unexpected or duplicate readiness",
        ),
        (vec![], "event channel closed during startup"),
    ] {
        let (events_tx, events_rx) = mpsc::unbounded_channel();
        for event in events {
            assert!(events_tx.send(event).is_ok());
        }
        drop(events_tx);
        let mut supervisor = ScopeTaskSupervisor::new(events_rx);
        let mut pending = HashSet::from([
            ExtensionDeclarationScope::Engine,
            ExtensionDeclarationScope::PipelineGroup("group".into()),
        ]);
        let mut never = Box::pin(std::future::pending());
        let error = supervisor
            .wait_until_ready(&mut pending, never.as_mut())
            .await
            .err()
            .expect("invalid barrier protocol must fail");
        assert!(matches!(error, Error::InternalError { message } if message.contains(expected)));
        assert!(pending.contains(&ExtensionDeclarationScope::PipelineGroup("group".into())));
    }
}
