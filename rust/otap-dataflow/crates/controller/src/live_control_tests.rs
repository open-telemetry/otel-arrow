// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use async_trait::async_trait;
use otel_arrow_dfe_config::ContextEntryName;
use otel_arrow_dfe_config::engine::ResolvedPipelineRole;
use otel_arrow_dfe_config::extension::ExtensionUserConfig;
use otel_arrow_dfe_config::observed_state::ObservedStateSettings;
use otel_arrow_dfe_config::settings::telemetry::logs::LogLevel;
use otel_arrow_dfe_engine::capability::ExtensionCapabilities;
use otel_arrow_dfe_engine::capability::registry::Capabilities;
use otel_arrow_dfe_engine::config::{
    ExporterConfig, ExtensionConfig, ProcessorConfig, ReceiverConfig,
};
use otel_arrow_dfe_engine::context::ExtensionContext;
use otel_arrow_dfe_engine::context_declaration::{
    ConfigNodeContextDeclaration, ContextDeclaration, ContextDeclarationProvider,
    NodeContextDeclarations,
};
use otel_arrow_dfe_engine::control::{
    ExtensionControlMsg, NodeControlMsg, RuntimeControlMsg, RuntimeCtrlMsgReceiver,
    runtime_ctrl_msg_channel,
};
use otel_arrow_dfe_engine::error::Error as EngineError;
use otel_arrow_dfe_engine::exporter::ExporterWrapper;
use otel_arrow_dfe_engine::extension::wrapper::ExtensionVariant;
use otel_arrow_dfe_engine::extension::{EffectHandler, ExtensionBundle, ExtensionWrapper};
use otel_arrow_dfe_engine::listener_group::ListenerProtocol;
use otel_arrow_dfe_engine::local::{exporter, receiver};
use otel_arrow_dfe_engine::message::{ExporterInbox, Message};
use otel_arrow_dfe_engine::processor::ProcessorWrapper;
use otel_arrow_dfe_engine::receiver::ReceiverWrapper;
use otel_arrow_dfe_engine::terminal_state::TerminalState;
use otel_arrow_dfe_engine::testing::capability::no_op_stateless::{
    LocalNoOpStateless, NoOpStateless, SharedNoOpStateless,
};
use otel_arrow_dfe_engine::topology::NumaTopology;
use otel_arrow_dfe_engine::wiring_contract::WiringContract;
use otel_arrow_dfe_engine::{
    ExporterFactory, ExtensionFactory, ProcessorFactory, ReceiverFactory, extension_capabilities,
};
use otel_arrow_dfe_state::pipeline_status::PipelineStatus;
use otel_arrow_dfe_telemetry::attributes::AttributeSetHandler;
use otel_arrow_dfe_telemetry::event::EngineEvent;
use otel_arrow_dfe_telemetry::log_filter::{RuntimeLogFilter, RuntimeLogFilterHandle};
use otel_arrow_dfe_telemetry::metrics::{
    MetricExportBatch, MetricSet, MetricSetSnapshot, MetricValue,
};
use otel_arrow_dfe_telemetry::tracing_init::ProviderSetup;
use otel_arrow_dfe_telemetry::{InternalTelemetrySystem, TracingSetup};
use serde::Deserialize;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio_util::sync::CancellationToken;
use tracing::{Event, Subscriber};
use tracing_subscriber::Registry;
use tracing_subscriber::layer::{Context, Layer, SubscriberExt};

struct CountingLayer(Arc<AtomicUsize>);

impl<S: Subscriber> Layer<S> for CountingLayer {
    fn on_event(&self, _event: &Event<'_>, _context: Context<'_, S>) {
        _ = self.0.fetch_add(1, Ordering::SeqCst);
    }
}

fn available_core_ids() -> Vec<CoreId> {
    vec![
        CoreId { id: 0 },
        CoreId { id: 1 },
        CoreId { id: 2 },
        CoreId { id: 3 },
        CoreId { id: 4 },
        CoreId { id: 5 },
        CoreId { id: 6 },
        CoreId { id: 7 },
    ]
}

fn test_validate_config(
    _config: &serde_json::Value,
) -> Result<(), otel_arrow_dfe_config::error::Error> {
    Ok(())
}

const CONTEXT_BINDINGS_TEST_RECEIVER_URN: &str = "urn:test:receiver:context-bindings";
static CONTEXT_BINDINGS_TEST_LOCK: Mutex<()> = Mutex::new(());
static CONTEXT_BINDINGS_TEST_CAPTURE: Mutex<Option<std::sync::Weak<CompiledContextBindings>>> =
    Mutex::new(None);
static CONTEXT_BINDINGS_TEST_RUNTIME: Mutex<Option<std::sync::Weak<ControllerRuntime<()>>>> =
    Mutex::new(None);
static CONTEXT_BINDINGS_TEST_DECLARATION_CALLS: AtomicUsize = AtomicUsize::new(0);

fn reset_context_bindings_test_capture() {
    *CONTEXT_BINDINGS_TEST_CAPTURE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
}

fn wait_for_context_bindings_test_capture() -> std::sync::Weak<CompiledContextBindings> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(policy) = CONTEXT_BINDINGS_TEST_CAPTURE
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
        {
            return policy;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for context binding installation"
        );
        thread::sleep(Duration::from_millis(25));
    }
}

#[derive(Deserialize)]
struct ContextBindingsTestConfig {
    produces: ContextEntryName,
    #[serde(default)]
    probe_controller_lock: bool,
}

impl ConfigNodeContextDeclaration for ContextBindingsTestConfig {
    fn context_declarations(&self) -> NodeContextDeclarations {
        vec![ContextDeclaration::Produces {
            entry: self.produces.clone(),
        }]
        .into_iter()
        .collect()
    }
}

fn context_bindings_test_declarations(
    value: &serde_json::Value,
) -> Result<NodeContextDeclarations, otel_arrow_dfe_config::error::Error> {
    let config: ContextBindingsTestConfig =
        serde_json::from_value(value.clone()).map_err(|error| {
            otel_arrow_dfe_config::error::Error::InvalidUserConfig {
                error: error.to_string(),
            }
        })?;
    if config.probe_controller_lock {
        let runtime = CONTEXT_BINDINGS_TEST_RUNTIME
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
            .and_then(std::sync::Weak::upgrade);
        if let Some(runtime) = runtime {
            let _state = runtime.state.try_lock().map_err(|_| {
                otel_arrow_dfe_config::error::Error::InvalidUserConfig {
                    error: "context declarations compiled while controller state was locked"
                        .to_owned(),
                }
            })?;
            let _ = CONTEXT_BINDINGS_TEST_DECLARATION_CALLS.fetch_add(1, Ordering::Relaxed);
        }
    }
    Ok(config.context_declarations())
}

fn context_bindings_test_receiver_create(
    pipeline_ctx: PipelineContext,
    node: otel_arrow_dfe_engine::node::NodeId,
    node_config: Arc<NodeUserConfig>,
    receiver_config: &ReceiverConfig,
    _capabilities: &Capabilities,
) -> Result<ReceiverWrapper<()>, otel_arrow_dfe_config::error::Error> {
    let config: ContextBindingsTestConfig = serde_json::from_value(node_config.config.clone())
        .map_err(
            |error| otel_arrow_dfe_config::error::Error::InvalidUserConfig {
                error: error.to_string(),
            },
        )?;
    config.validate_context_declarations(&pipeline_ctx)?;
    let policy = pipeline_ctx.compiled_context_bindings();
    *CONTEXT_BINDINGS_TEST_CAPTURE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(Arc::downgrade(policy));
    Ok(ReceiverWrapper::local(
        RecoveryTestReceiver,
        node,
        node_config,
        receiver_config,
    ))
}

fn test_receiver_create(
    _pipeline_ctx: PipelineContext,
    _node: otel_arrow_dfe_engine::node::NodeId,
    _node_config: Arc<NodeUserConfig>,
    _receiver_config: &ReceiverConfig,
    _capabilities: &Capabilities,
) -> Result<ReceiverWrapper<()>, otel_arrow_dfe_config::error::Error> {
    panic!("test receiver factory should not be constructed")
}

fn test_exporter_create(
    _pipeline_ctx: PipelineContext,
    _node: otel_arrow_dfe_engine::node::NodeId,
    _node_config: Arc<NodeUserConfig>,
    _exporter_config: &ExporterConfig,
    _capabilities: &Capabilities,
) -> Result<ExporterWrapper<()>, otel_arrow_dfe_config::error::Error> {
    panic!("test exporter factory should not be constructed")
}

fn test_processor_create(
    _pipeline_ctx: PipelineContext,
    _node: otel_arrow_dfe_engine::node::NodeId,
    _node_config: Arc<NodeUserConfig>,
    _processor_config: &ProcessorConfig,
    _capabilities: &Capabilities,
) -> Result<ProcessorWrapper<()>, otel_arrow_dfe_config::error::Error> {
    panic!("test processor factory should not be constructed")
}

struct RecoveryTestReceiver;

#[async_trait(?Send)]
impl receiver::Receiver<()> for RecoveryTestReceiver {
    async fn start(
        self: Box<Self>,
        mut ctrl_chan: receiver::ControlChannel<()>,
        effect_handler: receiver::EffectHandler<()>,
    ) -> Result<TerminalState, EngineError> {
        loop {
            match ctrl_chan.recv().await? {
                NodeControlMsg::DrainIngress { deadline, .. } => {
                    effect_handler.notify_receiver_drained().await?;
                    return Ok(TerminalState::new::<[MetricSetSnapshot; 0]>(deadline, []));
                }
                NodeControlMsg::Shutdown { deadline, .. } => {
                    return Ok(TerminalState::new::<[MetricSetSnapshot; 0]>(deadline, []));
                }
                _ => {}
            }
        }
    }
}

struct RecoveryTestExporter;

#[async_trait(?Send)]
impl exporter::Exporter<()> for RecoveryTestExporter {
    async fn start(
        self: Box<Self>,
        mut inbox: ExporterInbox<()>,
        _effect_handler: exporter::EffectHandler<()>,
    ) -> Result<TerminalState, EngineError> {
        loop {
            if let Message::Control(NodeControlMsg::Shutdown { deadline, .. }) =
                inbox.recv().await?
            {
                return Ok(TerminalState::new::<[MetricSetSnapshot; 0]>(deadline, []));
            }
        }
    }
}

fn recovery_test_receiver_create(
    _pipeline_ctx: PipelineContext,
    node: otel_arrow_dfe_engine::node::NodeId,
    node_config: Arc<NodeUserConfig>,
    receiver_config: &ReceiverConfig,
    _capabilities: &Capabilities,
) -> Result<ReceiverWrapper<()>, otel_arrow_dfe_config::error::Error> {
    Ok(ReceiverWrapper::local(
        RecoveryTestReceiver,
        node,
        node_config,
        receiver_config,
    ))
}

fn recovery_test_exporter_create(
    _pipeline_ctx: PipelineContext,
    node: otel_arrow_dfe_engine::node::NodeId,
    node_config: Arc<NodeUserConfig>,
    exporter_config: &ExporterConfig,
    _capabilities: &Capabilities,
) -> Result<ExporterWrapper<()>, otel_arrow_dfe_config::error::Error> {
    Ok(ExporterWrapper::local(
        RecoveryTestExporter,
        node,
        node_config,
        exporter_config,
    ))
}

#[derive(Default)]
struct VariantReloadLifecycleCounts {
    created: AtomicUsize,
    started: AtomicUsize,
    stopped: AtomicUsize,
    dropped: AtomicUsize,
}

#[derive(Clone, Default)]
struct VariantReloadProbe {
    local: Arc<VariantReloadLifecycleCounts>,
    shared: Arc<VariantReloadLifecycleCounts>,
}

// Factory callbacks are static function pointers and pipeline generations run on
// separate OS threads. This registry only routes per-test counters by unique
// config key; pipeline data paths never access it.
static VARIANT_RELOAD_PROBES: std::sync::OnceLock<Mutex<HashMap<String, VariantReloadProbe>>> =
    std::sync::OnceLock::new();

fn variant_reload_probes() -> &'static Mutex<HashMap<String, VariantReloadProbe>> {
    VARIANT_RELOAD_PROBES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn register_variant_reload_probe(key: &str) -> VariantReloadProbe {
    let probe = VariantReloadProbe::default();
    _ = variant_reload_probes()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .insert(key.to_owned(), probe.clone());
    probe
}

fn lookup_variant_reload_probe(key: &str) -> VariantReloadProbe {
    variant_reload_probes()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .get(key)
        .cloned()
        .unwrap_or_else(|| panic!("variant reload probe `{key}` is not registered"))
}

struct VariantReloadLocalExtension {
    counts: Arc<VariantReloadLifecycleCounts>,
    counts_lifecycle_drop: bool,
}

impl Clone for VariantReloadLocalExtension {
    fn clone(&self) -> Self {
        Self {
            counts: Arc::clone(&self.counts),
            counts_lifecycle_drop: false,
        }
    }
}

impl Drop for VariantReloadLocalExtension {
    fn drop(&mut self) {
        if self.counts_lifecycle_drop {
            _ = self.counts.dropped.fetch_add(1, Ordering::SeqCst);
        }
    }
}

#[async_trait(?Send)]
impl LocalNoOpStateless for VariantReloadLocalExtension {
    fn name(&self) -> &str {
        "variant-reload-local"
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
impl otel_arrow_dfe_engine::local::extension::Extension for VariantReloadLocalExtension {
    async fn start(
        self: Rc<Self>,
        mut ctrl: otel_arrow_dfe_engine::local::extension::ControlChannel,
        _effect_handler: EffectHandler,
    ) -> Result<TerminalState, EngineError> {
        _ = self.counts.started.fetch_add(1, Ordering::SeqCst);
        loop {
            match ctrl.recv().await {
                Ok(ExtensionControlMsg::Shutdown { .. }) | Err(_) => {
                    _ = self.counts.stopped.fetch_add(1, Ordering::SeqCst);
                    return Ok(TerminalState::default());
                }
                Ok(_) => {}
            }
        }
    }
}

struct VariantReloadSharedExtension {
    counts: Arc<VariantReloadLifecycleCounts>,
    counts_lifecycle_drop: bool,
}

impl Clone for VariantReloadSharedExtension {
    fn clone(&self) -> Self {
        Self {
            counts: Arc::clone(&self.counts),
            counts_lifecycle_drop: false,
        }
    }
}

impl Drop for VariantReloadSharedExtension {
    fn drop(&mut self) {
        if self.counts_lifecycle_drop {
            _ = self.counts.dropped.fetch_add(1, Ordering::SeqCst);
        }
    }
}

#[async_trait]
impl SharedNoOpStateless for VariantReloadSharedExtension {
    fn name(&self) -> &str {
        "variant-reload-shared"
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

#[async_trait]
impl otel_arrow_dfe_engine::shared::extension::Extension for VariantReloadSharedExtension {
    async fn start(
        self: Box<Self>,
        mut ctrl: otel_arrow_dfe_engine::shared::extension::ControlChannel,
        _effect_handler: EffectHandler,
    ) -> Result<TerminalState, EngineError> {
        _ = self.counts.started.fetch_add(1, Ordering::SeqCst);
        loop {
            match ctrl.recv().await {
                Ok(ExtensionControlMsg::Shutdown { .. }) | Err(_) => {
                    _ = self.counts.stopped.fetch_add(1, Ordering::SeqCst);
                    return Ok(TerminalState::default());
                }
                Ok(_) => {}
            }
        }
    }
}

const VARIANT_RELOAD_EXTENSION_URN: &str = "urn:test:extension:variant-reload";

fn variant_reload_extension_create(
    _context: &ExtensionContext,
    name: ExtensionId,
    user_config: Arc<ExtensionUserConfig>,
    runtime_config: &ExtensionConfig,
) -> Result<ExtensionBundle, otel_arrow_dfe_config::error::Error> {
    let probe_key = user_config
        .config
        .get("probe_key")
        .and_then(|value| value.as_str())
        .expect("variant reload extension config should contain probe_key");
    let probe = lookup_variant_reload_probe(probe_key);
    _ = probe.local.created.fetch_add(1, Ordering::SeqCst);
    _ = probe.shared.created.fetch_add(1, Ordering::SeqCst);

    let bundle = ExtensionWrapper::builder(name, user_config, runtime_config)
        .active()
        .shared(VariantReloadSharedExtension {
            counts: probe.shared,
            counts_lifecycle_drop: true,
        })
        .local(Rc::new(VariantReloadLocalExtension {
            counts: probe.local,
            counts_lifecycle_drop: true,
        }))
        .build()
        .expect("variant reload extension bundle should build");
    Ok(bundle)
}

const VARIANT_RELOAD_EXTENSION_FACTORY: ExtensionFactory = ExtensionFactory {
    name: VARIANT_RELOAD_EXTENSION_URN,
    description: "dual active extension for live-reload variant lifecycle tests",
    documentation_url: "",
    capabilities: Some(extension_capabilities!(
        (
            shared: VariantReloadSharedExtension,
            local: VariantReloadLocalExtension
        ) => [NoOpStateless]
    )),
    create: variant_reload_extension_create,
    validate_config: test_validate_config,
};

const VARIANT_RELOAD_RECEIVER_URN: &str = "urn:test:receiver:variant-reload";

fn variant_reload_receiver_create(
    _pipeline_ctx: PipelineContext,
    node: otel_arrow_dfe_engine::node::NodeId,
    node_config: Arc<NodeUserConfig>,
    receiver_config: &ReceiverConfig,
    capabilities: &Capabilities,
) -> Result<ReceiverWrapper<()>, otel_arrow_dfe_config::error::Error> {
    match node_config
        .config
        .get("variant")
        .and_then(|value| value.as_str())
        .expect("variant reload receiver config should contain variant")
    {
        "local" => {
            let capability = capabilities
                .require_local::<NoOpStateless>()
                .expect("local variant binding should resolve");
            assert_eq!(capability.name(), "variant-reload-local");
        }
        "shared" => {
            let capability = capabilities
                .require_shared::<NoOpStateless>()
                .expect("shared variant binding should resolve");
            assert_eq!(capability.name(), "variant-reload-shared");
        }
        variant => panic!("unsupported variant reload receiver mode `{variant}`"),
    }

    Ok(ReceiverWrapper::local(
        RecoveryTestReceiver,
        node,
        node_config,
        receiver_config,
    ))
}

const VARIANT_RELOAD_RECEIVER_FACTORY: ReceiverFactory<()> = ReceiverFactory {
    name: VARIANT_RELOAD_RECEIVER_URN,
    create: variant_reload_receiver_create,
    context_declarations: None,
    wiring_contract: WiringContract::UNRESTRICTED,
    validate_config: test_validate_config,
};

#[otel_arrow_dfe_telemetry_macros::metric_set(name = "test.extension.outer_scope_live_reconfig")]
#[derive(Debug, Default, Clone)]
struct OuterScopeLiveReconfigMetrics {
    #[metric(name = "reported", unit = "{item}")]
    reported: otel_arrow_dfe_telemetry::instrument::Counter<u64>,
}

struct OuterScopeLiveReconfigProbe {
    created: AtomicUsize,
    started: AtomicUsize,
    stopped: AtomicUsize,
    dropped: AtomicUsize,
    bindings: AtomicUsize,
    collections: AtomicUsize,
    // Delivers deterministic collection barriers from the scope-host task to the test driver.
    collection_events: tokio::sync::mpsc::UnboundedSender<usize>,
}

// Scope hosts and pipeline generations run on different threads. This registry shares only
// per-test lifecycle counters and collection barriers, keyed by immutable test configuration.
static OUTER_SCOPE_LIVE_RECONFIG_PROBES: std::sync::OnceLock<
    Mutex<HashMap<String, Arc<OuterScopeLiveReconfigProbe>>>,
> = std::sync::OnceLock::new();

fn outer_scope_live_reconfig_probes()
-> &'static Mutex<HashMap<String, Arc<OuterScopeLiveReconfigProbe>>> {
    OUTER_SCOPE_LIVE_RECONFIG_PROBES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn register_outer_scope_live_reconfig_probe(
    key: &str,
) -> (
    Arc<OuterScopeLiveReconfigProbe>,
    tokio::sync::mpsc::UnboundedReceiver<usize>,
) {
    let (collection_events, collection_events_rx) = tokio::sync::mpsc::unbounded_channel();
    let probe = Arc::new(OuterScopeLiveReconfigProbe {
        created: AtomicUsize::new(0),
        started: AtomicUsize::new(0),
        stopped: AtomicUsize::new(0),
        dropped: AtomicUsize::new(0),
        bindings: AtomicUsize::new(0),
        collections: AtomicUsize::new(0),
        collection_events,
    });
    _ = outer_scope_live_reconfig_probes()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .insert(key.to_owned(), Arc::clone(&probe));
    (probe, collection_events_rx)
}

fn lookup_outer_scope_live_reconfig_probe(key: &str) -> Arc<OuterScopeLiveReconfigProbe> {
    outer_scope_live_reconfig_probes()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .get(key)
        .cloned()
        .unwrap_or_else(|| panic!("outer-scope live-reconfig probe `{key}` is not registered"))
}

struct OuterScopeLiveReconfigExtension {
    metrics: MetricSet<OuterScopeLiveReconfigMetrics>,
    report_value: u64,
    probe: Arc<OuterScopeLiveReconfigProbe>,
    counts_lifecycle_drop: bool,
}

impl Clone for OuterScopeLiveReconfigExtension {
    fn clone(&self) -> Self {
        Self {
            metrics: self.metrics.clone(),
            report_value: self.report_value,
            probe: Arc::clone(&self.probe),
            counts_lifecycle_drop: false,
        }
    }
}

impl Drop for OuterScopeLiveReconfigExtension {
    fn drop(&mut self) {
        if self.counts_lifecycle_drop {
            _ = self.probe.dropped.fetch_add(1, Ordering::SeqCst);
        }
    }
}

#[async_trait]
impl SharedNoOpStateless for OuterScopeLiveReconfigExtension {
    fn name(&self) -> &str {
        "outer-scope-live-reconfig"
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

#[async_trait]
impl otel_arrow_dfe_engine::shared::extension::Extension for OuterScopeLiveReconfigExtension {
    async fn start(
        mut self: Box<Self>,
        mut ctrl: otel_arrow_dfe_engine::shared::extension::ControlChannel,
        _effect_handler: EffectHandler,
    ) -> Result<TerminalState, EngineError> {
        _ = self.probe.started.fetch_add(1, Ordering::SeqCst);
        loop {
            match ctrl.recv().await {
                Ok(ExtensionControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                }) => {
                    self.metrics.reported.add(self.report_value);
                    metrics_reporter
                        .report(&mut self.metrics)
                        .map_err(|error| EngineError::InternalError {
                            message: format!(
                                "outer-scope live-reconfig metric reporting failed: {error}"
                            ),
                        })?;
                    let collection = self.probe.collections.fetch_add(1, Ordering::SeqCst) + 1;
                    _ = self.probe.collection_events.send(collection);
                }
                Ok(ExtensionControlMsg::Shutdown { .. }) | Err(_) => {
                    _ = self.probe.stopped.fetch_add(1, Ordering::SeqCst);
                    return Ok(TerminalState::default());
                }
                Ok(ExtensionControlMsg::Config { .. }) => {}
            }
        }
    }
}

const OUTER_SCOPE_LIVE_RECONFIG_EXTENSION_URN: &str =
    "urn:test:extension:outer-scope-live-reconfig";

fn outer_scope_live_reconfig_extension_create(
    context: &ExtensionContext,
    name: ExtensionId,
    user_config: Arc<ExtensionUserConfig>,
    runtime_config: &ExtensionConfig,
) -> Result<ExtensionBundle, otel_arrow_dfe_config::error::Error> {
    let probe_key = user_config
        .config
        .get("probe_key")
        .and_then(|value| value.as_str())
        .expect("outer-scope extension config should contain probe_key");
    let report_value = user_config
        .config
        .get("report_value")
        .and_then(serde_json::Value::as_u64)
        .expect("outer-scope extension config should contain report_value");
    let probe = lookup_outer_scope_live_reconfig_probe(probe_key);
    _ = probe.created.fetch_add(1, Ordering::SeqCst);

    let entity_key = context.register_extension_entity(name.clone(), ExtensionVariant::Shared);
    let metrics =
        context.register_metric_set_for_entity::<OuterScopeLiveReconfigMetrics>(entity_key);
    let bundle = ExtensionWrapper::builder(name, user_config, runtime_config)
        .active()
        .shared(OuterScopeLiveReconfigExtension {
            metrics,
            report_value,
            probe,
            counts_lifecycle_drop: true,
        })
        .build()
        .expect("outer-scope live-reconfig extension should build");
    Ok(bundle)
}

const OUTER_SCOPE_LIVE_RECONFIG_EXTENSION_FACTORY: ExtensionFactory = ExtensionFactory {
    name: OUTER_SCOPE_LIVE_RECONFIG_EXTENSION_URN,
    description: "active shared extension for outer-scope live-reconfiguration tests",
    documentation_url: "",
    capabilities: Some(extension_capabilities!(
        shared: OuterScopeLiveReconfigExtension => [NoOpStateless]
    )),
    create: outer_scope_live_reconfig_extension_create,
    validate_config: test_validate_config,
};

const OUTER_SCOPE_LIVE_RECONFIG_RECEIVER_URN: &str = "urn:test:receiver:outer-scope-live-reconfig";

fn outer_scope_live_reconfig_receiver_create(
    _pipeline_ctx: PipelineContext,
    node: otel_arrow_dfe_engine::node::NodeId,
    node_config: Arc<NodeUserConfig>,
    receiver_config: &ReceiverConfig,
    capabilities: &Capabilities,
) -> Result<ReceiverWrapper<()>, otel_arrow_dfe_config::error::Error> {
    let probe_key = node_config
        .config
        .get("probe_key")
        .and_then(|value| value.as_str())
        .expect("outer-scope receiver config should contain probe_key");
    let capability = capabilities
        .require_shared::<NoOpStateless>()
        .expect("outer-scope shared capability should resolve");
    assert_eq!(capability.name(), "outer-scope-live-reconfig");
    assert_eq!(capability.echo(37), 37);
    let probe = lookup_outer_scope_live_reconfig_probe(probe_key);
    _ = probe.bindings.fetch_add(1, Ordering::SeqCst);

    Ok(ReceiverWrapper::local(
        RecoveryTestReceiver,
        node,
        node_config,
        receiver_config,
    ))
}

const OUTER_SCOPE_LIVE_RECONFIG_RECEIVER_FACTORY: ReceiverFactory<()> = ReceiverFactory {
    name: OUTER_SCOPE_LIVE_RECONFIG_RECEIVER_URN,
    create: outer_scope_live_reconfig_receiver_create,
    context_declarations: None,
    wiring_contract: WiringContract::UNRESTRICTED,
    validate_config: test_validate_config,
};

static TEST_RECEIVER_FACTORIES: &[ReceiverFactory<()>] = &[
    ReceiverFactory {
        name: "urn:test:receiver:example",
        create: test_receiver_create,
        context_declarations: None,
        wiring_contract: WiringContract::UNRESTRICTED,
        validate_config: test_validate_config,
    },
    VARIANT_RELOAD_RECEIVER_FACTORY,
    ReceiverFactory {
        name: "urn:otel:receiver:topic",
        create: test_receiver_create,
        context_declarations: None,
        wiring_contract: WiringContract::UNRESTRICTED,
        validate_config: test_validate_config,
    },
    ReceiverFactory {
        name: "urn:otel:receiver:otlp",
        create: test_receiver_create,
        context_declarations: None,
        wiring_contract: WiringContract::UNRESTRICTED,
        validate_config: test_validate_config,
    },
    ReceiverFactory {
        name: "urn:otel:receiver:internal_telemetry",
        create: test_receiver_create,
        context_declarations: None,
        wiring_contract: WiringContract::UNRESTRICTED,
        validate_config: test_validate_config,
    },
];

static TEST_PROCESSOR_FACTORIES: &[ProcessorFactory<()>] = &[ProcessorFactory {
    name: "urn:otel:processor:type_router",
    create: test_processor_create,
    context_declarations: None,
    wiring_contract: WiringContract::UNRESTRICTED,
    validate_config: test_validate_config,
}];

static TEST_EXPORTER_FACTORIES: &[ExporterFactory<()>] = &[
    ExporterFactory {
        name: "urn:test:exporter:example",
        create: test_exporter_create,
        context_declarations: None,
        wiring_contract: WiringContract::UNRESTRICTED,
        validate_config: test_validate_config,
    },
    ExporterFactory {
        name: "urn:otel:exporter:topic",
        create: test_exporter_create,
        context_declarations: None,
        wiring_contract: WiringContract::UNRESTRICTED,
        validate_config: test_validate_config,
    },
    ExporterFactory {
        name: "urn:otel:exporter:console",
        create: test_exporter_create,
        context_declarations: None,
        wiring_contract: WiringContract::UNRESTRICTED,
        validate_config: test_validate_config,
    },
    ExporterFactory {
        name: "urn:otel:exporter:noop",
        create: test_exporter_create,
        context_declarations: None,
        wiring_contract: WiringContract::UNRESTRICTED,
        validate_config: test_validate_config,
    },
];

fn test_scope_extension_create(
    _context: &ExtensionContext,
    name: ExtensionId,
    user_config: Arc<ExtensionUserConfig>,
    runtime_config: &ExtensionConfig,
) -> Result<ExtensionBundle, otel_arrow_dfe_config::error::Error> {
    Ok(ExtensionWrapper::builder(name, user_config, runtime_config)
        .passive()
        .cloned()
        .shared(())
        .build()
        .expect("test scope-hosted extension should build"))
}

static TEST_EXTENSION_FACTORIES: &[ExtensionFactory] = &[
    ExtensionFactory {
        name: "urn:test:extension:scope-shared",
        description: "shared extension for declaration-scope live-control tests",
        documentation_url: "",
        capabilities: Some(ExtensionCapabilities {
            shared: &["bearer_token_provider"],
            local: &[],
            register_shared: |_, _, _| Ok(()),
            register_local: |_, _, _| Ok(()),
        }),
        create: test_scope_extension_create,
        validate_config: test_validate_config,
    },
    VARIANT_RELOAD_EXTENSION_FACTORY,
];

static TEST_PIPELINE_FACTORY: PipelineFactory<()> = PipelineFactory::new(
    TEST_RECEIVER_FACTORIES,
    TEST_PROCESSOR_FACTORIES,
    TEST_EXPORTER_FACTORIES,
    TEST_EXTENSION_FACTORIES,
);

static CONTEXT_BINDINGS_TEST_RECEIVER_FACTORIES: &[ReceiverFactory<()>] = &[
    ReceiverFactory {
        name: CONTEXT_BINDINGS_TEST_RECEIVER_URN,
        create: context_bindings_test_receiver_create,
        context_declarations: Some(ContextDeclarationProvider {
            declarations: context_bindings_test_declarations,
        }),
        wiring_contract: WiringContract::UNRESTRICTED,
        validate_config: otel_arrow_dfe_config::validation::validate_typed_config::<
            ContextBindingsTestConfig,
        >,
    },
    ReceiverFactory {
        name: "urn:otel:receiver:internal_telemetry",
        create: test_receiver_create,
        context_declarations: None,
        wiring_contract: WiringContract::UNRESTRICTED,
        validate_config: test_validate_config,
    },
];

static CONTEXT_BINDINGS_TEST_EXPORTER_FACTORIES: &[ExporterFactory<()>] = &[
    ExporterFactory {
        name: "urn:test:exporter:example",
        create: recovery_test_exporter_create,
        context_declarations: None,
        wiring_contract: WiringContract::UNRESTRICTED,
        validate_config: test_validate_config,
    },
    ExporterFactory {
        name: "urn:otel:exporter:console",
        create: test_exporter_create,
        context_declarations: None,
        wiring_contract: WiringContract::UNRESTRICTED,
        validate_config: test_validate_config,
    },
    ExporterFactory {
        name: "urn:otel:exporter:noop",
        create: test_exporter_create,
        context_declarations: None,
        wiring_contract: WiringContract::UNRESTRICTED,
        validate_config: test_validate_config,
    },
];

static CONTEXT_BINDINGS_TEST_PIPELINE_FACTORY: PipelineFactory<()> = PipelineFactory::new(
    CONTEXT_BINDINGS_TEST_RECEIVER_FACTORIES,
    TEST_PROCESSOR_FACTORIES,
    CONTEXT_BINDINGS_TEST_EXPORTER_FACTORIES,
    &[],
);

static RECOVERY_TEST_RECEIVER_FACTORIES: &[ReceiverFactory<()>] = &[
    ReceiverFactory {
        name: "urn:test:receiver:example",
        create: recovery_test_receiver_create,
        context_declarations: None,
        wiring_contract: WiringContract::UNRESTRICTED,
        validate_config: test_validate_config,
    },
    ReceiverFactory {
        name: "urn:otel:receiver:internal_telemetry",
        create: recovery_test_receiver_create,
        context_declarations: None,
        wiring_contract: WiringContract::UNRESTRICTED,
        validate_config: test_validate_config,
    },
];

static RECOVERY_TEST_EXPORTER_FACTORIES: &[ExporterFactory<()>] = &[
    ExporterFactory {
        name: "urn:test:exporter:example",
        create: recovery_test_exporter_create,
        context_declarations: None,
        wiring_contract: WiringContract::UNRESTRICTED,
        validate_config: test_validate_config,
    },
    ExporterFactory {
        name: "urn:otel:exporter:console",
        create: recovery_test_exporter_create,
        context_declarations: None,
        wiring_contract: WiringContract::UNRESTRICTED,
        validate_config: test_validate_config,
    },
    ExporterFactory {
        name: "urn:otel:exporter:noop",
        create: recovery_test_exporter_create,
        context_declarations: None,
        wiring_contract: WiringContract::UNRESTRICTED,
        validate_config: test_validate_config,
    },
];

static RECOVERY_TEST_PIPELINE_FACTORY: PipelineFactory<()> = PipelineFactory::new(
    RECOVERY_TEST_RECEIVER_FACTORIES,
    TEST_PROCESSOR_FACTORIES,
    RECOVERY_TEST_EXPORTER_FACTORIES,
    &[],
);

static VARIANT_RELOAD_TEST_RECEIVER_FACTORIES: &[ReceiverFactory<()>] = &[
    VARIANT_RELOAD_RECEIVER_FACTORY,
    ReceiverFactory {
        name: "urn:otel:receiver:internal_telemetry",
        create: recovery_test_receiver_create,
        context_declarations: None,
        wiring_contract: WiringContract::UNRESTRICTED,
        validate_config: test_validate_config,
    },
];

static VARIANT_RELOAD_TEST_EXTENSION_FACTORIES: &[ExtensionFactory] =
    &[VARIANT_RELOAD_EXTENSION_FACTORY];

static VARIANT_RELOAD_TEST_PIPELINE_FACTORY: PipelineFactory<()> = PipelineFactory::new(
    VARIANT_RELOAD_TEST_RECEIVER_FACTORIES,
    TEST_PROCESSOR_FACTORIES,
    RECOVERY_TEST_EXPORTER_FACTORIES,
    VARIANT_RELOAD_TEST_EXTENSION_FACTORIES,
);

static OUTER_SCOPE_LIVE_RECONFIG_TEST_RECEIVER_FACTORIES: &[ReceiverFactory<()>] = &[
    OUTER_SCOPE_LIVE_RECONFIG_RECEIVER_FACTORY,
    ReceiverFactory {
        name: "urn:otel:receiver:internal_telemetry",
        create: recovery_test_receiver_create,
        context_declarations: None,
        wiring_contract: WiringContract::UNRESTRICTED,
        validate_config: test_validate_config,
    },
];

static OUTER_SCOPE_LIVE_RECONFIG_TEST_EXTENSION_FACTORIES: &[ExtensionFactory] =
    &[OUTER_SCOPE_LIVE_RECONFIG_EXTENSION_FACTORY];

static OUTER_SCOPE_LIVE_RECONFIG_TEST_PIPELINE_FACTORY: PipelineFactory<()> = PipelineFactory::new(
    OUTER_SCOPE_LIVE_RECONFIG_TEST_RECEIVER_FACTORIES,
    TEST_PROCESSOR_FACTORIES,
    RECOVERY_TEST_EXPORTER_FACTORIES,
    OUTER_SCOPE_LIVE_RECONFIG_TEST_EXTENSION_FACTORIES,
);

fn test_runtime(config: &OtelDataflowSpec) -> Arc<ControllerRuntime<()>> {
    test_runtime_with_factory_and_topology(config, &TEST_PIPELINE_FACTORY, NumaTopology::unknown())
}

fn test_runtime_with_topology(
    config: &OtelDataflowSpec,
    topology: NumaTopology,
) -> Arc<ControllerRuntime<()>> {
    test_runtime_with_factory_and_topology(config, &TEST_PIPELINE_FACTORY, topology)
}

fn test_runtime_with_factory(
    config: &OtelDataflowSpec,
    pipeline_factory: &'static PipelineFactory<()>,
) -> Arc<ControllerRuntime<()>> {
    test_runtime_with_factory_and_topology(config, pipeline_factory, NumaTopology::unknown())
}

fn test_runtime_with_factory_and_topology(
    config: &OtelDataflowSpec,
    pipeline_factory: &'static PipelineFactory<()>,
    topology: NumaTopology,
) -> Arc<ControllerRuntime<()>> {
    test_runtime_with_log_filter_and_topology(config, pipeline_factory, topology).0
}

fn test_runtime_with_extension_scope_registry(
    config: &OtelDataflowSpec,
    extension_scope_registry: ExtensionScopeRegistry,
) -> Arc<ControllerRuntime<()>> {
    test_runtime_with_factory_and_extension_scope_registry(
        config,
        &TEST_PIPELINE_FACTORY,
        extension_scope_registry,
    )
}

fn test_runtime_with_factory_and_extension_scope_registry(
    config: &OtelDataflowSpec,
    pipeline_factory: &'static PipelineFactory<()>,
    extension_scope_registry: ExtensionScopeRegistry,
) -> Arc<ControllerRuntime<()>> {
    let (log_filter, log_filter_handle) =
        RuntimeLogFilter::new(config.engine.telemetry.logs.level.as_ref());
    test_runtime_with_supplied_log_filter_topology_and_extension_scope_registry(
        config,
        pipeline_factory,
        NumaTopology::unknown(),
        log_filter,
        log_filter_handle,
        extension_scope_registry,
    )
    .0
}

fn test_runtime_with_log_filter(
    config: &OtelDataflowSpec,
    pipeline_factory: &'static PipelineFactory<()>,
) -> (
    Arc<ControllerRuntime<()>>,
    RuntimeLogFilterHandle,
    RuntimeLogFilter,
) {
    test_runtime_with_log_filter_and_topology(config, pipeline_factory, NumaTopology::unknown())
}

fn test_runtime_with_log_filter_and_topology(
    config: &OtelDataflowSpec,
    pipeline_factory: &'static PipelineFactory<()>,
    topology: NumaTopology,
) -> (
    Arc<ControllerRuntime<()>>,
    RuntimeLogFilterHandle,
    RuntimeLogFilter,
) {
    let (log_filter, log_filter_handle) =
        RuntimeLogFilter::new(config.engine.telemetry.logs.level.as_ref());
    test_runtime_with_supplied_log_filter_and_topology(
        config,
        pipeline_factory,
        topology,
        log_filter,
        log_filter_handle,
    )
}

fn test_runtime_with_supplied_log_filter_and_topology(
    config: &OtelDataflowSpec,
    pipeline_factory: &'static PipelineFactory<()>,
    topology: NumaTopology,
    log_filter: RuntimeLogFilter,
    log_filter_handle: RuntimeLogFilterHandle,
) -> (
    Arc<ControllerRuntime<()>>,
    RuntimeLogFilterHandle,
    RuntimeLogFilter,
) {
    test_runtime_with_supplied_log_filter_topology_and_extension_scope_registry(
        config,
        pipeline_factory,
        topology,
        log_filter,
        log_filter_handle,
        ExtensionScopeRegistry::default(),
    )
}

fn test_runtime_with_supplied_log_filter_topology_and_extension_scope_registry(
    config: &OtelDataflowSpec,
    pipeline_factory: &'static PipelineFactory<()>,
    topology: NumaTopology,
    log_filter: RuntimeLogFilter,
    log_filter_handle: RuntimeLogFilterHandle,
    extension_scope_registry: ExtensionScopeRegistry,
) -> (
    Arc<ControllerRuntime<()>>,
    RuntimeLogFilterHandle,
    RuntimeLogFilter,
) {
    let registry = TelemetryRegistryHandle::new();
    let observed_state_store =
        ObservedStateStore::new(&ObservedStateSettings::default(), registry.clone());
    let observed_state_handle = observed_state_store.handle();
    let engine_event_reporter = observed_state_store.reporter(Default::default());
    let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(8);
    let declared_topics =
        Controller::<()>::declare_topics(config).expect("declared topics should be valid");
    let (memory_pressure_tx, _memory_pressure_rx) =
        tokio::sync::watch::channel(MemoryPressureChanged::initial());
    let context = pipeline_factory
        .compile_initial_context(&config.resolve())
        .expect("test context bindings should compile");

    (
        Arc::new(ControllerRuntime::new(
            pipeline_factory,
            ControllerContext::new(registry),
            observed_state_store,
            observed_state_handle,
            engine_event_reporter,
            metrics_reporter,
            extension_scope_registry,
            declared_topics,
            context.runtime_requirements,
            context.bindings,
            available_core_ids(),
            topology,
            TracingSetup::new(ProviderSetup::Noop, LogLevel::default(), engine_context)
                .with_log_filter(log_filter.clone()),
            log_filter_handle.clone(),
            Duration::from_secs(1),
            memory_pressure_tx,
            config.clone(),
        )),
        log_filter_handle,
        log_filter,
    )
}

struct ObservedStateRunner {
    cancel: CancellationToken,
    join: Option<thread::JoinHandle<()>>,
}

impl ObservedStateRunner {
    fn start(runtime: &ControllerRuntime<()>) -> Self {
        let cancel = CancellationToken::new();
        let store = runtime.observed_state_store.clone();
        let cancel_clone = cancel.clone();
        let join = thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("observed-state test runtime should build");
            runtime
                .block_on(store.run(cancel_clone))
                .expect("observed-state consumer should exit cleanly");
        });
        Self {
            cancel,
            join: Some(join),
        }
    }
}

impl Drop for ObservedStateRunner {
    fn drop(&mut self) {
        self.cancel.cancel();
        if let Some(join) = self.join.take() {
            join.join()
                .expect("observed-state consumer thread should join cleanly");
        }
    }
}

fn deployed_key(
    pipeline_group_id: &str,
    pipeline_id: &str,
    core_id: usize,
    generation: u64,
) -> DeployedPipelineKey {
    DeployedPipelineKey {
        pipeline_group_id: pipeline_group_id.to_owned().into(),
        pipeline_id: pipeline_id.to_owned().into(),
        core_id,
        deployment_generation: generation,
    }
}

fn report_ready(runtime: &ControllerRuntime<()>, key: DeployedPipelineKey) {
    runtime
        .engine_event_reporter
        .report(EngineEvent::admitted(key.clone(), None));
    runtime
        .engine_event_reporter
        .report(EngineEvent::ready(key, None));
}

fn report_stopped(runtime: &ControllerRuntime<()>, key: DeployedPipelineKey) {
    runtime
        .engine_event_reporter
        .report(EngineEvent::admitted(key.clone(), None));
    runtime
        .engine_event_reporter
        .report(EngineEvent::ready(key.clone(), None));
    runtime
        .engine_event_reporter
        .report(EngineEvent::shutdown_requested(key.clone(), None));
    runtime
        .engine_event_reporter
        .report(EngineEvent::drained(key, None));
}

fn wait_for_observed_status<F>(
    runtime: &ControllerRuntime<()>,
    pipeline_key: &PipelineKey,
    predicate: F,
) -> PipelineStatus
where
    F: Fn(&PipelineStatus) -> bool,
{
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(status) = runtime.observed_state_handle.pipeline_status(pipeline_key)
            && predicate(&status)
        {
            return status;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for observed status predicate on {}:{}",
            pipeline_key.pipeline_group_id(),
            pipeline_key.pipeline_id()
        );
        thread::sleep(Duration::from_millis(25));
    }
}

fn wait_for_atomic_count(counter: &AtomicUsize, expected: usize, label: &str) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let actual = counter.load(Ordering::SeqCst);
        if actual == expected {
            return;
        }
        assert!(
            actual < expected,
            "{label} exceeded expected count {expected}: {actual}"
        );
        assert!(
            Instant::now() < deadline,
            "timed out waiting for {label} to reach {expected}; current count: {actual}"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

fn wait_for_terminal_rollout(runtime: &ControllerRuntime<()>, rollout_id: &str) -> RolloutStatus {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let status = runtime
            .rollout_status_snapshot(rollout_id)
            .expect("rollout should remain queryable");
        if matches!(
            status.state,
            ApiPipelineRolloutState::Succeeded
                | ApiPipelineRolloutState::Failed
                | ApiPipelineRolloutState::RollbackFailed
        ) {
            return status;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for rollout {rollout_id}; current state: {:?}",
            status.state
        );
        thread::sleep(Duration::from_millis(25));
    }
}

fn launch_committed_test_pipeline(
    runtime: &Arc<ControllerRuntime<()>>,
    config: &OtelDataflowSpec,
) -> DeployedPipelineKey {
    let resolved = config
        .resolve()
        .pipelines
        .into_iter()
        .find(|pipeline| {
            pipeline.role == ResolvedPipelineRole::Regular
                && pipeline.pipeline_group_id.as_ref() == "g1"
                && pipeline.pipeline_id.as_ref() == "p1"
        })
        .expect("resolved test pipeline should exist");
    let placement = runtime
        .pipeline_placement_for_resolved(&resolved)
        .expect("test pipeline placement should resolve");
    let live_placement = runtime.live_pipeline_placement_from(&resolved, placement.clone(), 0);
    let inherited_extensions =
        runtime.inherited_extensions_for_pipeline(&resolved.pipeline_group_id, &resolved.pipeline);
    runtime.register_committed_pipeline_with_inherited(
        resolved.clone(),
        inherited_extensions.clone(),
        placement,
        0,
    );
    let deployed_key = runtime
        .launch_regular_pipeline_instance(
            &resolved,
            &inherited_extensions,
            current_test_context_bindings(runtime),
            &live_placement,
            0,
            0,
        )
        .expect("initial test pipeline should launch");
    runtime
        .wait_for_pipeline_ready(&deployed_key, Instant::now() + Duration::from_secs(5))
        .expect("initial test pipeline should become ready");
    deployed_key
}

fn variant_reload_pipeline_yaml(probe_key: &str, include_local_consumer: bool) -> String {
    let local_node = include_local_consumer.then(|| {
        format!(
            r#"
  local_receiver:
    type: "{VARIANT_RELOAD_RECEIVER_URN}"
    config:
      variant: local
    capabilities:
      no_op_stateless: variant_extension"#
        )
    });
    let local_connection = include_local_consumer.then_some(
        r#"
  - from: local_receiver
    to: exporter"#,
    );

    format!(
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 1
extensions:
  variant_extension:
    type: "{VARIANT_RELOAD_EXTENSION_URN}"
    config:
      probe_key: "{probe_key}"
nodes:
  shared_receiver:
    type: "{VARIANT_RELOAD_RECEIVER_URN}"
    config:
      variant: shared
    capabilities:
      no_op_stateless: variant_extension
{local_node}
  exporter:
    type: "urn:test:exporter:example"
connections:
  - from: shared_receiver
    to: exporter
{local_connection}
"#,
        local_node = local_node.as_deref().unwrap_or(""),
        local_connection = local_connection.unwrap_or(""),
    )
}

fn variant_reload_engine_config(probe_key: &str, include_local_consumer: bool) -> OtelDataflowSpec {
    let pipeline_yaml = variant_reload_pipeline_yaml(probe_key, include_local_consumer);
    let indented_pipeline = pipeline_yaml
        .lines()
        .map(|line| format!("        {line}"))
        .collect::<Vec<_>>()
        .join("\n");
    OtelDataflowSpec::from_yaml(&format!(
        "version: otel_dataflow/v1\ngroups:\n  g1:\n    pipelines:\n      p1:\n{indented_pipeline}\n"
    ))
    .expect("variant reload engine config should parse")
}

fn variant_reload_pipeline_config(probe_key: &str, include_local_consumer: bool) -> PipelineConfig {
    PipelineConfig::from_yaml(
        "g1".into(),
        "p1".into(),
        &variant_reload_pipeline_yaml(probe_key, include_local_consumer),
    )
    .expect("variant reload pipeline config should parse")
}

#[derive(Clone, Copy)]
enum OuterScopeLiveReconfigDeclarationScope {
    Engine,
    PipelineGroup,
}

impl OuterScopeLiveReconfigDeclarationScope {
    const fn probe_key(self) -> &'static str {
        match self {
            Self::Engine => "engine-outer-scope-live-reconfig",
            Self::PipelineGroup => "group-outer-scope-live-reconfig",
        }
    }

    const fn report_value(self) -> u64 {
        match self {
            Self::Engine => 17,
            Self::PipelineGroup => 29,
        }
    }

    const fn scope_kind(self) -> &'static str {
        match self {
            Self::Engine => "engine",
            Self::PipelineGroup => "group",
        }
    }

    const fn pipeline_group_id(self) -> Option<&'static str> {
        match self {
            Self::Engine => None,
            Self::PipelineGroup => Some("g1"),
        }
    }
}

fn outer_scope_live_reconfig_pipeline_yaml(probe_key: &str, revision: u64) -> String {
    format!(
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 1
nodes:
  receiver:
    type: "{OUTER_SCOPE_LIVE_RECONFIG_RECEIVER_URN}"
    config:
      probe_key: "{probe_key}"
      revision: {revision}
    capabilities:
      no_op_stateless: outer_scope_extension
  exporter:
    type: "urn:test:exporter:example"
connections:
  - from: receiver
    to: exporter
"#
    )
}

fn outer_scope_live_reconfig_engine_config(
    declaration_scope: OuterScopeLiveReconfigDeclarationScope,
) -> OtelDataflowSpec {
    outer_scope_live_reconfig_engine_config_with_revision(declaration_scope, 0)
}

fn outer_scope_live_reconfig_engine_config_with_revision(
    declaration_scope: OuterScopeLiveReconfigDeclarationScope,
    revision: u64,
) -> OtelDataflowSpec {
    let probe_key = declaration_scope.probe_key();
    let report_value = declaration_scope.report_value();
    let pipeline_yaml = outer_scope_live_reconfig_pipeline_yaml(probe_key, revision);
    let indented_pipeline = pipeline_yaml
        .lines()
        .map(|line| format!("        {line}"))
        .collect::<Vec<_>>()
        .join("\n");
    let yaml = match declaration_scope {
        OuterScopeLiveReconfigDeclarationScope::Engine => format!(
            r#"
version: otel_dataflow/v1
extensions:
  outer_scope_extension:
    type: "{OUTER_SCOPE_LIVE_RECONFIG_EXTENSION_URN}"
    config:
      probe_key: "{probe_key}"
      report_value: {report_value}
groups:
  g1:
    pipelines:
      p1:
{indented_pipeline}
"#
        ),
        OuterScopeLiveReconfigDeclarationScope::PipelineGroup => format!(
            r#"
version: otel_dataflow/v1
groups:
  g1:
    extensions:
      outer_scope_extension:
        type: "{OUTER_SCOPE_LIVE_RECONFIG_EXTENSION_URN}"
        config:
          probe_key: "{probe_key}"
          report_value: {report_value}
    pipelines:
      p1:
{indented_pipeline}
"#
        ),
    };
    OtelDataflowSpec::from_yaml(&yaml).expect("outer-scope live-reconfig config should parse")
}

fn outer_scope_live_reconfig_replacement(
    declaration_scope: OuterScopeLiveReconfigDeclarationScope,
) -> PipelineConfig {
    outer_scope_live_reconfig_engine_config_with_revision(declaration_scope, 1)
        .groups
        .get(&PipelineGroupId::from("g1"))
        .and_then(|group| group.pipelines.get(&PipelineId::from("p1")))
        .cloned()
        .expect("outer-scope live-reconfig replacement pipeline should exist")
}

async fn wait_for_outer_scope_collection(
    collection_events: &mut tokio::sync::mpsc::UnboundedReceiver<usize>,
    expected: usize,
) {
    for _ in 0..64 {
        match collection_events.try_recv() {
            Ok(received) => {
                assert_eq!(
                    received, expected,
                    "outer-scope telemetry collection sequence must be monotonic"
                );
                return;
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                tokio::task::yield_now().await;
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                panic!("outer-scope telemetry collection channel closed");
            }
        }
    }
    panic!("timed out waiting for outer-scope telemetry collection {expected}");
}

fn assert_outer_scope_live_reconfig_metric_batch(
    batch: MetricExportBatch,
    declaration_scope: OuterScopeLiveReconfigDeclarationScope,
) -> BTreeMap<String, String> {
    let mut matching = batch.metric_sets.into_iter().filter(|metric_set| {
        metric_set.descriptor.name == "test.extension.outer_scope_live_reconfig"
    });
    let metric_set = matching
        .next()
        .expect("outer-scope custom metric set must reach the collector");
    assert!(
        matching.next().is_none(),
        "one hosted outer-scope extension must produce one custom metric set per collection"
    );
    assert_eq!(
        metric_set.values,
        vec![MetricValue::from(declaration_scope.report_value())]
    );

    let attributes = metric_set
        .attributes
        .iter_attributes()
        .map(|(key, value)| (key.to_owned(), value.to_string_value()))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(
        attributes.get("extension.id").map(String::as_str),
        Some("outer_scope_extension")
    );
    assert_eq!(
        attributes.get("extension.variant").map(String::as_str),
        Some("shared")
    );
    assert_eq!(
        attributes.get("scope.kind").map(String::as_str),
        Some(declaration_scope.scope_kind())
    );
    match declaration_scope.pipeline_group_id() {
        Some(pipeline_group_id) => assert_eq!(
            attributes.get("pipeline.group.id").map(String::as_str),
            Some(pipeline_group_id)
        ),
        None => assert!(
            attributes
                .get("pipeline.group.id")
                .map(String::as_str)
                .unwrap_or_default()
                .is_empty(),
            "engine-scoped metrics must not carry a pipeline group identity"
        ),
    }
    attributes
}

async fn exercise_outer_scope_extension_live_reconfig(
    declaration_scope: OuterScopeLiveReconfigDeclarationScope,
) {
    let (probe, mut collection_events) =
        register_outer_scope_live_reconfig_probe(declaration_scope.probe_key());
    let config = outer_scope_live_reconfig_engine_config(declaration_scope);
    let telemetry_system = InternalTelemetrySystem::default();
    let collector = telemetry_system.collector();
    let extension_scope_registry = ExtensionScopeRegistry::default();
    let controller_context = ControllerContext::new(telemetry_system.registry());
    let prepared = OUTER_SCOPE_LIVE_RECONFIG_TEST_PIPELINE_FACTORY
        .prepare_extension_scope_hosts(
            &config,
            &controller_context,
            telemetry_system.reporter(),
            extension_scope_registry.clone(),
        )
        .expect("outer-scope extension hosts should prepare");
    let running = prepared
        .start()
        .await
        .expect("outer-scope extension hosts should start");
    tokio::task::yield_now().await;
    assert_eq!(probe.created.load(Ordering::SeqCst), 1);
    assert_eq!(probe.started.load(Ordering::SeqCst), 1);

    let runtime = test_runtime_with_factory_and_extension_scope_registry(
        &config,
        &OUTER_SCOPE_LIVE_RECONFIG_TEST_PIPELINE_FACTORY,
        extension_scope_registry,
    );
    let _observed_state_runner = ObservedStateRunner::start(&runtime);
    let _initial_key = launch_committed_test_pipeline(&runtime, &config);
    wait_for_atomic_count(&probe.bindings, 1, "initial outer-scope capability binding");

    let plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: outer_scope_live_reconfig_replacement(declaration_scope),
                step_timeout_secs: 5,
                drain_timeout_secs: 5,
            },
        )
        .expect("outer-scope pipeline replacement should plan");
    assert_eq!(plan.action, RolloutAction::Replace);
    let accepted = runtime
        .spawn_rollout(plan)
        .expect("outer-scope pipeline replacement should start");
    let status = wait_for_terminal_rollout(&runtime, &accepted.rollout_id);
    assert_eq!(
        status.state,
        ApiPipelineRolloutState::Succeeded,
        "outer-scope pipeline replacement failed: {:?}",
        status.failure_reason
    );
    wait_for_atomic_count(
        &probe.bindings,
        2,
        "replacement outer-scope capability binding",
    );
    assert_eq!(probe.created.load(Ordering::SeqCst), 1);
    assert_eq!(probe.started.load(Ordering::SeqCst), 1);
    assert_eq!(probe.stopped.load(Ordering::SeqCst), 0);
    assert_eq!(probe.dropped.load(Ordering::SeqCst), 0);

    tokio::time::advance(Duration::from_secs(1)).await;
    wait_for_outer_scope_collection(&mut collection_events, 1).await;
    collector.collect_pending();
    _ = assert_outer_scope_live_reconfig_metric_batch(
        telemetry_system.registry().drain_metric_export_batch(),
        declaration_scope,
    );
    assert_eq!(probe.collections.load(Ordering::SeqCst), 1);

    let target_key = deployed_key("g1", "p1", 0, status.target_generation);
    shutdown_test_pipeline(&runtime, &target_key);
    assert_eq!(probe.stopped.load(Ordering::SeqCst), 0);
    assert_eq!(probe.dropped.load(Ordering::SeqCst), 0);

    tokio::time::resume();
    running
        .shutdown()
        .await
        .expect("outer-scope extension hosts should stop cleanly");
    assert_eq!(probe.stopped.load(Ordering::SeqCst), 1);
    assert_eq!(probe.dropped.load(Ordering::SeqCst), 1);
    collector.collect_pending();
}

fn shutdown_test_pipeline(
    runtime: &Arc<ControllerRuntime<()>>,
    deployed_key: &DeployedPipelineKey,
) {
    runtime
        .request_instance_shutdown(deployed_key, 5, "variant reload test cleanup")
        .expect("test pipeline should accept shutdown");
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let active_instances = runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .active_instances;
        if active_instances == 0 {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for test pipeline shutdown; active instances: {active_instances}"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

fn wait_for_recovery_candidate_generation(
    runtime: &ControllerRuntime<()>,
    pipeline_key: &PipelineKey,
    core_id: usize,
) -> u64 {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let candidate_generation = runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .runtime_recoveries
            .get(&(pipeline_key.clone(), core_id))
            .filter(|recovery| recovery.worker_id.is_some())
            .and_then(|recovery| recovery.candidate_generation);
        if let Some(candidate_generation) = candidate_generation {
            return candidate_generation;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for recovery candidate on {}:{} core {}",
            pipeline_key.pipeline_group_id(),
            pipeline_key.pipeline_id(),
            core_id
        );
        thread::sleep(Duration::from_millis(10));
    }
}

fn engine_config_with_pipeline(pipeline_yaml: &str) -> OtelDataflowSpec {
    OtelDataflowSpec::from_yaml(&format!(
        r#"
version: otel_dataflow/v1
groups:
  g1:
    pipelines:
      p1:
{pipeline_yaml}
"#
    ))
    .expect("engine config should parse")
}

fn empty_engine_config() -> OtelDataflowSpec {
    OtelDataflowSpec::from_yaml("version: otel_dataflow/v1\n")
        .expect("empty engine config should parse")
}

fn reconcile_request(
    config: OtelDataflowSpec,
    delete_missing: bool,
) -> EngineConfigReconcileRequest {
    EngineConfigReconcileRequest {
        config,
        step_timeout_secs: 5,
        drain_timeout_secs: 5,
        delete_timeout_secs: 5,
        delete_missing,
    }
}

fn simple_pipeline_yaml() -> &'static str {
    r#"
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#
}

fn register_existing_pipeline(runtime: &ControllerRuntime<()>, config: &OtelDataflowSpec) {
    register_pipeline(runtime, config, "g1", "p1");
}

fn current_test_context_bindings(runtime: &ControllerRuntime<()>) -> Arc<CompiledContextBindings> {
    let state = runtime
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    Arc::clone(&state.latest_context_bindings)
}

fn register_pipeline(
    runtime: &ControllerRuntime<()>,
    config: &OtelDataflowSpec,
    group_id: &str,
    pipeline_id: &str,
) {
    let resolved = config
        .resolve()
        .pipelines
        .into_iter()
        .find(|pipeline| {
            pipeline.role == ResolvedPipelineRole::Regular
                && pipeline.pipeline_group_id.as_ref() == group_id
                && pipeline.pipeline_id.as_ref() == pipeline_id
        })
        .expect("resolved pipeline should exist");
    let placement = runtime
        .pipeline_placement_for_resolved(&resolved)
        .expect("resolved pipeline placement should exist");
    runtime.register_committed_pipeline(resolved, placement, 0);
}

fn register_runtime_instance(
    runtime: &ControllerRuntime<()>,
    pipeline_group_id: &str,
    pipeline_id: &str,
    core_id: usize,
    generation: u64,
    lifecycle: RuntimeInstanceLifecycle,
) -> RuntimeCtrlMsgReceiver<()> {
    let (tx, rx) = runtime_ctrl_msg_channel::<()>(4);
    let control_sender: Arc<dyn PipelineAdminSender> = Arc::new(tx.clone());
    let is_active = matches!(&lifecycle, RuntimeInstanceLifecycle::Active);
    let mut state = runtime
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let context_bindings = Arc::clone(&state.latest_context_bindings);
    _ = state.runtime_instances.insert(
        DeployedPipelineKey {
            pipeline_group_id: pipeline_group_id.to_owned().into(),
            pipeline_id: pipeline_id.to_owned().into(),
            core_id,
            deployment_generation: generation,
        },
        RuntimeInstanceRecord {
            control_sender: Some(control_sender),
            context_bindings,
            lifecycle,
        },
    );
    if is_active {
        state.active_instances += 1;
    }
    rx
}

fn register_runtime_instance_with_sender(
    runtime: &ControllerRuntime<()>,
    pipeline_key: DeployedPipelineKey,
    control_sender: Arc<dyn PipelineAdminSender>,
    lifecycle: RuntimeInstanceLifecycle,
) {
    let is_active = matches!(&lifecycle, RuntimeInstanceLifecycle::Active);
    let mut state = runtime
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let context_bindings = Arc::clone(&state.latest_context_bindings);
    _ = state.runtime_instances.insert(
        pipeline_key,
        RuntimeInstanceRecord {
            control_sender: Some(control_sender),
            context_bindings,
            lifecycle,
        },
    );
    if is_active {
        state.active_instances += 1;
    }
}

struct RecordingPipelineAdminSender {
    calls: Arc<Mutex<Vec<String>>>,
    failure: Option<String>,
}

impl PipelineAdminSender for RecordingPipelineAdminSender {
    fn try_send_shutdown(&self, _deadline: Instant, reason: String) -> Result<(), EngineError> {
        self.calls
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(reason);
        if let Some(failure) = &self.failure {
            Err(EngineError::RuntimeMsgError {
                error: failure.clone(),
            })
        } else {
            Ok(())
        }
    }
}

fn recording_admin_sender(
    failure: Option<&str>,
) -> (Arc<dyn PipelineAdminSender>, Arc<Mutex<Vec<String>>>) {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let sender = Arc::new(RecordingPipelineAdminSender {
        calls: Arc::clone(&calls),
        failure: failure.map(ToOwned::to_owned),
    });
    (sender, calls)
}

struct NotifyingPipelineAdminSender {
    notification: std::sync::mpsc::Sender<String>,
}

impl PipelineAdminSender for NotifyingPipelineAdminSender {
    fn try_send_shutdown(&self, _deadline: Instant, reason: String) -> Result<(), EngineError> {
        self.notification
            .send(reason)
            .map_err(|error| EngineError::RuntimeMsgError {
                error: error.to_string(),
            })
    }
}

fn notifying_admin_sender() -> (
    Arc<dyn PipelineAdminSender>,
    std::sync::mpsc::Receiver<String>,
) {
    let (notification, receiver) = std::sync::mpsc::channel();
    (
        Arc::new(NotifyingPipelineAdminSender { notification }),
        receiver,
    )
}

struct DeadlineNotifyingPipelineAdminSender {
    notification: std::sync::mpsc::Sender<(String, Instant)>,
}

impl PipelineAdminSender for DeadlineNotifyingPipelineAdminSender {
    fn try_send_shutdown(&self, deadline: Instant, reason: String) -> Result<(), EngineError> {
        self.notification
            .send((reason, deadline))
            .map_err(|error| EngineError::RuntimeMsgError {
                error: error.to_string(),
            })
    }
}

fn deadline_notifying_admin_sender() -> (
    Arc<dyn PipelineAdminSender>,
    std::sync::mpsc::Receiver<(String, Instant)>,
) {
    let (notification, receiver) = std::sync::mpsc::channel();
    (
        Arc::new(DeadlineNotifyingPipelineAdminSender { notification }),
        receiver,
    )
}

struct ExitThenFailPipelineAdminSender {
    runtime: std::sync::Weak<ControllerRuntime<()>>,
    deployed_key: DeployedPipelineKey,
}

impl PipelineAdminSender for ExitThenFailPipelineAdminSender {
    fn try_send_shutdown(&self, _deadline: Instant, _reason: String) -> Result<(), EngineError> {
        if let Some(runtime) = self.runtime.upgrade() {
            runtime.note_instance_exit(self.deployed_key.clone(), RuntimeInstanceExit::Success);
        }
        Err(EngineError::RuntimeMsgError {
            error: "sender closed after clean exit".to_owned(),
        })
    }
}

fn launched_runtime_instance(
    runtime: &ControllerRuntime<()>,
    pipeline_group_id: &str,
    pipeline_id: &str,
    core_id: usize,
    generation: u64,
) -> LaunchedPipelineThread<()> {
    let (tx, _rx) = runtime_ctrl_msg_channel::<()>(4);
    let control_sender: Arc<dyn PipelineAdminSender> = Arc::new(tx);
    let context_bindings = {
        let state = runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        Arc::clone(&state.latest_context_bindings)
    };
    LaunchedPipelineThread {
        pipeline_key: DeployedPipelineKey {
            pipeline_group_id: pipeline_group_id.to_owned().into(),
            pipeline_id: pipeline_id.to_owned().into(),
            core_id,
            deployment_generation: generation,
        },
        control_sender,
        context_bindings,
        _marker: std::marker::PhantomData,
    }
}

fn wait_for_shutdown_state(
    runtime: &ControllerRuntime<()>,
    shutdown_id: &str,
    expected_state: &str,
) -> ShutdownStatus {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let status = runtime
            .shutdown_status_snapshot(shutdown_id)
            .expect("shutdown should exist");
        if status.state == expected_state {
            return status;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for shutdown {shutdown_id} to reach state {expected_state}, current state: {}",
            status.state
        );
        thread::sleep(Duration::from_millis(25));
    }
}

fn wait_for_shutdown_message(receiver: &mut RuntimeCtrlMsgReceiver<()>) -> RuntimeControlMsg<()> {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if let Ok(message) = receiver.try_recv() {
            return message;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for shutdown control message"
        );
        thread::sleep(Duration::from_millis(25));
    }
}

fn complete_instance_exit_on_shutdown(
    runtime: Arc<ControllerRuntime<()>>,
    mut receiver: RuntimeCtrlMsgReceiver<()>,
    deployed_key: DeployedPipelineKey,
    expected_reason: &'static str,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        assert!(matches!(
            wait_for_shutdown_message(&mut receiver),
            RuntimeControlMsg::Shutdown { reason, .. } if reason == expected_reason
        ));
        runtime.note_instance_exit(deployed_key, RuntimeInstanceExit::Success);
    })
}

fn terminal_rollout_record(
    pipeline_group_id: &str,
    pipeline_id: &str,
    rollout_id: &str,
) -> RolloutRecord {
    let mut rollout = RolloutRecord::new(
        rollout_id.to_owned(),
        pipeline_group_id.to_owned().into(),
        pipeline_id.to_owned().into(),
        RolloutAction::Replace,
        1,
        Some(0),
        60,
        CoreAllocationStrategy::CoreCount,
        PipelinePlacement {
            pipeline_group_id: pipeline_group_id.to_owned().into(),
            pipeline_id: pipeline_id.to_owned().into(),
            cores: Vec::new(),
        },
        Vec::new(),
    );
    rollout.state = RolloutLifecycleState::Succeeded;
    rollout
}

fn terminal_shutdown_record(
    pipeline_group_id: &str,
    pipeline_id: &str,
    shutdown_id: &str,
) -> ShutdownRecord {
    let mut shutdown = ShutdownRecord::new(
        shutdown_id.to_owned(),
        pipeline_group_id.to_owned().into(),
        pipeline_id.to_owned().into(),
        None,
        Vec::new(),
    );
    shutdown.state = ShutdownLifecycleState::Succeeded;
    shutdown
}

/// Scenario: a reconfigure request changes only the effective core
/// allocation from one assigned core to two.
/// Guarantees: rollout planning classifies the change as a resize, starts only
/// the added core, keeps the active generation unchanged, and assigns a fresh
/// placement generation.
#[test]
fn prepare_rollout_plan_accepts_core_allocation_scale_up() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);
    let _receiver =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);

    let replacement = PipelineConfig::from_yaml(
        "g1".into(),
        "p1".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 2
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("replacement should parse");

    let plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: replacement,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("core allocation changes should be planned");

    assert_eq!(plan.action, RolloutAction::Resize);
    assert_eq!(plan.current_assigned_cores, vec![0]);
    assert_eq!(plan.target_assigned_cores, vec![0, 1]);
    assert_eq!(plan.common_assigned_cores, vec![0]);
    assert_eq!(plan.added_assigned_cores, vec![1]);
    assert!(plan.removed_assigned_cores.is_empty());
    assert_eq!(plan.resize_start_cores, vec![1]);
    assert!(plan.resize_stop_cores.is_empty());
    assert_eq!(plan.target_generation, 0);
    assert_eq!(plan.target_placement.listener_group_snapshot.generation, 1);
    assert_eq!(
        plan.rollout
            .cores
            .iter()
            .map(|core| core.core_id)
            .collect::<Vec<_>>(),
        vec![1]
    );
}

#[test]
fn prepare_rollout_plan_reserves_other_committed_pipeline_cores() {
    let config = OtelDataflowSpec::from_yaml(
        r#"
version: otel_dataflow/v1
groups:
  g1:
    pipelines:
      p1:
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 4
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
      p2:
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 4
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    )
    .expect("engine config should parse");
    let runtime = test_runtime(&config);
    let mut resolved = config.resolve().pipelines;
    resolved.sort_by(|left, right| left.pipeline_id.as_ref().cmp(right.pipeline_id.as_ref()));
    let placement_snapshot = Controller::<()>::preflight_pipeline_placement(
        &resolved,
        &available_core_ids(),
        &NumaTopology::unknown(),
    )
    .expect("startup placement should resolve");

    for (pipeline, placement) in resolved.iter().cloned().zip(placement_snapshot.pipelines) {
        runtime.register_committed_pipeline(pipeline, placement, 0);
    }
    for core_id in 4..=7 {
        let _receiver = register_runtime_instance(
            &runtime,
            "g1",
            "p2",
            core_id,
            0,
            RuntimeInstanceLifecycle::Active,
        );
    }

    let replacement = PipelineConfig::from_yaml(
        "g1".into(),
        "p2".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 2
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("replacement should parse");

    let plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p2",
            &ReconfigureRequest {
                pipeline: replacement,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("core allocation changes should be planned");

    assert_eq!(plan.current_assigned_cores, vec![4, 5, 6, 7]);
    assert_eq!(plan.target_assigned_cores, vec![4, 5]);
    assert_eq!(plan.resize_stop_cores, vec![6, 7]);
    assert!(plan.resize_start_cores.is_empty());
}

#[test]
fn prepare_rollout_plan_does_not_reserve_committed_all_cores_pipeline() {
    let config = OtelDataflowSpec::from_yaml(
        r#"
version: otel_dataflow/v1
groups:
  g1:
    pipelines:
      p1:
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
      p2:
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 2
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    )
    .expect("engine config should parse");
    let runtime = test_runtime(&config);
    let mut resolved = config.resolve().pipelines;
    resolved.sort_by(|left, right| left.pipeline_id.as_ref().cmp(right.pipeline_id.as_ref()));
    let placement_snapshot = Controller::<()>::preflight_pipeline_placement(
        &resolved,
        &available_core_ids(),
        &NumaTopology::unknown(),
    )
    .expect("startup placement should resolve");

    for (pipeline, placement) in resolved.iter().cloned().zip(placement_snapshot.pipelines) {
        runtime.register_committed_pipeline(pipeline, placement, 0);
    }
    for core_id in 0..=1 {
        let _receiver = register_runtime_instance(
            &runtime,
            "g1",
            "p2",
            core_id,
            0,
            RuntimeInstanceLifecycle::Active,
        );
    }

    let replacement = PipelineConfig::from_yaml(
        "g1".into(),
        "p2".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 3
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("replacement should parse");

    let plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p2",
            &ReconfigureRequest {
                pipeline: replacement,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("all_cores committed pipeline should not reserve every core");

    assert_eq!(plan.target_assigned_cores, vec![0, 1, 2]);
}

#[test]
fn prepare_rollout_plan_rejects_core_count_all_when_no_unreserved_cores_remain() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_set
              set:
                - start: 0
                  end: 7
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);

    let p2_create = PipelineConfig::from_yaml(
        "g1".into(),
        "p2".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 0
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("p2 create should parse");

    let err = runtime
        .prepare_rollout_plan(
            "g1",
            "p2",
            &ReconfigureRequest {
                pipeline: p2_create,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect_err("live planning should reject empty effective core_count placement");

    assert!(matches!(
        err,
        ControlPlaneError::InvalidRequest { message }
            if message.contains("no unreserved cores are available")
    ));
}

#[test]
fn prepare_rollout_plan_reserves_active_rollout_target_cores() {
    let config = OtelDataflowSpec::from_yaml(
        r#"
version: otel_dataflow/v1
groups:
  g1:
    pipelines:
      p1:
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 2
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
      p2:
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 2
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    )
    .expect("engine config should parse");
    let runtime = test_runtime(&config);
    let mut resolved = config.resolve().pipelines;
    resolved.sort_by(|left, right| left.pipeline_id.as_ref().cmp(right.pipeline_id.as_ref()));
    let placement_snapshot = Controller::<()>::preflight_pipeline_placement(
        &resolved,
        &available_core_ids(),
        &NumaTopology::unknown(),
    )
    .expect("startup placement should resolve");

    for (pipeline, placement) in resolved.iter().cloned().zip(placement_snapshot.pipelines) {
        runtime.register_committed_pipeline(pipeline, placement, 0);
    }
    for core_id in 2..=3 {
        let _receiver = register_runtime_instance(
            &runtime,
            "g1",
            "p2",
            core_id,
            0,
            RuntimeInstanceLifecycle::Active,
        );
    }

    let p2_resize = PipelineConfig::from_yaml(
        "g1".into(),
        "p2".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 4
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("p2 resize should parse");
    let p2_plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p2",
            &ReconfigureRequest {
                pipeline: p2_resize,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("p2 resize should be planned");

    assert_eq!(p2_plan.target_assigned_cores, vec![2, 3, 4, 5]);
    runtime
        .insert_rollout(&p2_plan.pipeline_key, p2_plan.rollout.clone())
        .expect("p2 rollout should insert");

    let p3_create = PipelineConfig::from_yaml(
        "g1".into(),
        "p3".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 2
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("p3 create should parse");
    let p3_plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p3",
            &ReconfigureRequest {
                pipeline: p3_create,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("p3 create should be planned");

    assert_eq!(p3_plan.target_assigned_cores, vec![6, 7]);
}

#[test]
fn insert_rollout_rejects_stale_plan_conflicting_with_active_target_cores() {
    let config = OtelDataflowSpec::from_yaml(
        r#"
version: otel_dataflow/v1
groups:
  g1:
    pipelines:
      p1:
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 2
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
      p2:
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 2
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    )
    .expect("engine config should parse");
    let runtime = test_runtime(&config);
    let mut resolved = config.resolve().pipelines;
    resolved.sort_by(|left, right| left.pipeline_id.as_ref().cmp(right.pipeline_id.as_ref()));
    let placement_snapshot = Controller::<()>::preflight_pipeline_placement(
        &resolved,
        &available_core_ids(),
        &NumaTopology::unknown(),
    )
    .expect("startup placement should resolve");

    for (pipeline, placement) in resolved.iter().cloned().zip(placement_snapshot.pipelines) {
        runtime.register_committed_pipeline(pipeline, placement, 0);
    }

    let p2_resize = PipelineConfig::from_yaml(
        "g1".into(),
        "p2".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 4
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("p2 resize should parse");
    let p2_plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p2",
            &ReconfigureRequest {
                pipeline: p2_resize,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("p2 resize should be planned");

    let p3_create = PipelineConfig::from_yaml(
        "g1".into(),
        "p3".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 2
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("p3 create should parse");
    let p3_plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p3",
            &ReconfigureRequest {
                pipeline: p3_create,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("p3 create should be planned before p2 inserts");

    assert_eq!(p2_plan.target_assigned_cores, vec![2, 3, 4, 5]);
    assert_eq!(p3_plan.target_assigned_cores, vec![4, 5]);

    runtime
        .insert_rollout(&p2_plan.pipeline_key, p2_plan.rollout.clone())
        .expect("p2 rollout should insert");
    assert!(matches!(
        runtime.insert_rollout(&p3_plan.pipeline_key, p3_plan.rollout.clone()),
        Err(ControlPlaneError::RolloutConflict)
    ));
}

#[test]
fn insert_rollout_rejects_core_set_overlapping_active_core_count_target() {
    let config = OtelDataflowSpec::from_yaml(
        r#"
version: otel_dataflow/v1
groups:
  g1:
    pipelines:
      p1:
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 2
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
      p2:
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 2
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    )
    .expect("engine config should parse");
    let runtime = test_runtime(&config);
    let mut resolved = config.resolve().pipelines;
    resolved.sort_by(|left, right| left.pipeline_id.as_ref().cmp(right.pipeline_id.as_ref()));
    let placement_snapshot = Controller::<()>::preflight_pipeline_placement(
        &resolved,
        &available_core_ids(),
        &NumaTopology::unknown(),
    )
    .expect("startup placement should resolve");

    for (pipeline, placement) in resolved.iter().cloned().zip(placement_snapshot.pipelines) {
        runtime.register_committed_pipeline(pipeline, placement, 0);
    }

    let p2_resize = PipelineConfig::from_yaml(
        "g1".into(),
        "p2".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 4
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("p2 resize should parse");
    let p2_plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p2",
            &ReconfigureRequest {
                pipeline: p2_resize,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("p2 resize should be planned");

    runtime
        .insert_rollout(&p2_plan.pipeline_key, p2_plan.rollout.clone())
        .expect("p2 rollout should insert");

    let p3_create = PipelineConfig::from_yaml(
        "g1".into(),
        "p3".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_set
      set:
        - start: 4
          end: 5
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("p3 create should parse");
    let p3_plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p3",
            &ReconfigureRequest {
                pipeline: p3_create,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("p3 create should be planned");

    assert_eq!(p2_plan.target_assigned_cores, vec![2, 3, 4, 5]);
    assert_eq!(p3_plan.target_assigned_cores, vec![4, 5]);
    assert!(matches!(
        runtime.insert_rollout(&p3_plan.pipeline_key, p3_plan.rollout.clone()),
        Err(ControlPlaneError::RolloutConflict)
    ));
}

#[test]
fn insert_rollout_rejects_stale_plan_conflicting_with_new_committed_pipeline() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 2
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);

    let p2_create = PipelineConfig::from_yaml(
        "g1".into(),
        "p2".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 2
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("p2 create should parse");
    let p2_plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p2",
            &ReconfigureRequest {
                pipeline: p2_create,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("p2 create should be planned");
    assert_eq!(p2_plan.target_assigned_cores, vec![2, 3]);

    let mut config_with_p3 = config.clone();
    let group_id: PipelineGroupId = "g1".to_owned().into();
    let pipeline_id: PipelineId = "p3".to_owned().into();
    _ = config_with_p3
        .groups
        .get_mut(&group_id)
        .expect("g1 should exist")
        .pipelines
        .insert(
            pipeline_id,
            PipelineConfig::from_yaml(
                "g1".into(),
                "p3".into(),
                r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 2
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
            )
            .expect("p3 config should parse"),
        );
    let p3_resolved = config_with_p3
        .resolve()
        .pipelines
        .into_iter()
        .find(|pipeline| {
            pipeline.role == ResolvedPipelineRole::Regular
                && pipeline.pipeline_group_id.as_ref() == "g1"
                && pipeline.pipeline_id.as_ref() == "p3"
        })
        .expect("p3 should resolve");
    runtime.register_committed_pipeline(
        p3_resolved,
        PipelinePlacement {
            pipeline_group_id: "g1".to_owned().into(),
            pipeline_id: "p3".to_owned().into(),
            cores: [2, 3]
                .into_iter()
                .map(|core_id| {
                    CorePlacement::from_core_id(CoreId { id: core_id }, &NumaTopology::unknown())
                })
                .collect(),
        },
        0,
    );

    assert!(matches!(
        runtime.insert_rollout(&p2_plan.pipeline_key, p2_plan.rollout.clone()),
        Err(ControlPlaneError::RolloutConflict)
    ));
}

#[test]
fn insert_rollout_rejects_plan_after_committed_config_revision_changes() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 2
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);

    let resize_to_three = PipelineConfig::from_yaml(
        "g1".into(),
        "p1".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 3
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("resize to three should parse");
    let stale_plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: resize_to_three,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("resize to three should be planned");

    let resize_to_four = PipelineConfig::from_yaml(
        "g1".into(),
        "p1".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 4
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("resize to four should parse");
    let committed_plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: resize_to_four,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("resize to four should be planned");
    runtime.commit_pipeline_record(&committed_plan, committed_plan.target_generation);

    assert!(matches!(
        runtime.insert_rollout_plan(&stale_plan),
        Err(ControlPlaneError::RolloutConflict)
    ));
}

#[test]
fn insert_rollout_rejects_core_set_overlapping_committed_core_count_pipeline() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 2
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);

    let p2_create = PipelineConfig::from_yaml(
        "g1".into(),
        "p2".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_set
      set:
        - start: 1
          end: 2
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("p2 create should parse");
    let p2_plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p2",
            &ReconfigureRequest {
                pipeline: p2_create,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("p2 create should be planned");

    assert_eq!(p2_plan.target_assigned_cores, vec![1, 2]);
    assert!(matches!(
        runtime.insert_rollout(&p2_plan.pipeline_key, p2_plan.rollout.clone()),
        Err(ControlPlaneError::RolloutConflict)
    ));
}

#[test]
fn insert_rollout_allows_core_set_overlapping_committed_core_set_pipeline() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_set
              set:
                - start: 1
                  end: 2
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);

    let p2_create = PipelineConfig::from_yaml(
        "g1".into(),
        "p2".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_set
      set:
        - start: 2
          end: 3
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("p2 create should parse");
    let p2_plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p2",
            &ReconfigureRequest {
                pipeline: p2_create,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("p2 create should be planned");

    assert_eq!(p2_plan.target_assigned_cores, vec![2, 3]);
    runtime
        .insert_rollout(&p2_plan.pipeline_key, p2_plan.rollout.clone())
        .expect("explicit core_set overlap should be allowed");
}

/// Scenario: live-control accepts a new pipeline with listener bind config on
/// cores mapped to a known NUMA node.
/// Guarantees: placement and listener metadata are resolved during planning,
/// before any runtime worker is launched.
#[test]
fn prepare_rollout_plan_resolves_live_numa_and_listener_metadata() {
    let config = engine_config_with_pipeline(simple_pipeline_yaml());
    let topology = NumaTopology::from_node_cpulists(&[(0, "0-3".into()), (1, "4-7".into())]);
    let runtime = test_runtime_with_topology(&config, topology);

    let replacement = PipelineConfig::from_yaml(
        "g1".into(),
        "p1".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_set
      set:
        - start: 4
          end: 5
nodes:
  receiver:
    type: "urn:otel:receiver:otlp"
    config:
      protocols:
        grpc:
          listening_addr: "127.0.0.1:4317"
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("replacement should parse");

    let plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: replacement,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("listener pipeline should be planned");

    assert_eq!(plan.target_assigned_cores, vec![4, 5]);
    assert_eq!(plan.target_placement.placement.core_count(), 2);
    assert_eq!(
        plan.target_placement
            .placement
            .cores
            .iter()
            .map(|core| (core.core_id.id, core.known_numa_node_id))
            .collect::<Vec<_>>(),
        vec![(4, Some(1)), (5, Some(1))]
    );

    let snapshot = &plan.target_placement.listener_group_snapshot;
    assert_eq!(snapshot.generation, 1);
    assert_eq!(snapshot.plans.len(), 1);
    let listener_plan = snapshot
        .plan_for(
            "receiver",
            "127.0.0.1:4317".parse().unwrap(),
            ListenerProtocol::Tcp,
        )
        .expect("grpc listener group should be planned");
    assert_eq!(
        listener_plan
            .expected_members
            .iter()
            .map(|member| (member.core_id, member.numa_node_id))
            .collect::<Vec<_>>(),
        vec![(4, Some(1)), (5, Some(1))]
    );
}

/// Scenario: a socket receiver grows its `core_count` while retaining workers
/// whose immutable listener snapshots contain the old membership.
/// Guarantees: planning uses replacement so every worker receives the same new
/// listener membership and placement generation.
#[test]
fn prepare_rollout_plan_replaces_listener_pipeline_on_scale_up() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 2
        nodes:
          receiver:
            type: "urn:otel:receiver:otlp"
            config:
              protocols:
                grpc:
                  listening_addr: "127.0.0.1:4317"
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);

    let resize = PipelineConfig::from_yaml(
        "g1".into(),
        "p1".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 3
nodes:
  receiver:
    type: "urn:otel:receiver:otlp"
    config:
      protocols:
        grpc:
          listening_addr: "127.0.0.1:4317"
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("resize should parse");

    let plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: resize,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("resize should be planned");

    assert_eq!(plan.action, RolloutAction::Replace);
    assert_eq!(plan.target_generation, 1);
    assert!(plan.resize_start_cores.is_empty());
    assert!(plan.resize_stop_cores.is_empty());
    assert_eq!(plan.target_placement.listener_group_snapshot.generation, 1);
    let listener_plan = plan.target_placement.listener_group_snapshot.plans[0].clone();
    assert_eq!(
        listener_plan
            .expected_members
            .iter()
            .map(|member| member.core_id)
            .collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
}

/// Scenario: a socket receiver shrinks its `core_count` while retaining workers
/// whose immutable listener snapshots still include the removed core.
/// Guarantees: planning uses replacement so retained workers cannot keep stale
/// listener membership after the removed worker exits.
#[test]
fn prepare_rollout_plan_replaces_listener_pipeline_on_scale_down() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 3
        nodes:
          receiver:
            type: "urn:otel:receiver:otlp"
            config:
              protocols:
                grpc:
                  listening_addr: "127.0.0.1:4317"
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);

    let replacement = PipelineConfig::from_yaml(
        "g1".into(),
        "p1".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 2
nodes:
  receiver:
    type: "urn:otel:receiver:otlp"
    config:
      protocols:
        grpc:
          listening_addr: "127.0.0.1:4317"
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("replacement should parse");

    let plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: replacement,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("listener scale-down should be planned");

    assert_eq!(plan.action, RolloutAction::Replace);
    assert_eq!(plan.target_generation, 1);
    assert_eq!(plan.current_assigned_cores, vec![0, 1, 2]);
    assert_eq!(plan.target_assigned_cores, vec![0, 1]);
    assert_eq!(plan.removed_assigned_cores, vec![2]);
    assert!(plan.resize_start_cores.is_empty());
    assert!(plan.resize_stop_cores.is_empty());
    assert_eq!(plan.target_placement.listener_group_snapshot.generation, 1);
    assert_eq!(
        plan.target_placement.listener_group_snapshot.plans[0]
            .expected_members
            .iter()
            .map(|member| member.core_id)
            .collect::<Vec<_>>(),
        vec![0, 1]
    );
}

/// Scenario: a reconfigure request changes only the effective core
/// allocation from two assigned cores to one.
/// Guarantees: rollout planning classifies the change as a resize, stops only
/// the removed core, keeps the active generation unchanged, and assigns a fresh
/// placement generation.
#[test]
fn prepare_rollout_plan_accepts_core_allocation_scale_down() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 2
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);
    let _receiver0 =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);
    let _receiver1 =
        register_runtime_instance(&runtime, "g1", "p1", 1, 0, RuntimeInstanceLifecycle::Active);

    let replacement = PipelineConfig::from_yaml(
        "g1".into(),
        "p1".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 1
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("replacement should parse");

    let plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: replacement,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("core allocation changes should be planned");

    assert_eq!(plan.action, RolloutAction::Resize);
    assert_eq!(plan.current_assigned_cores, vec![0, 1]);
    assert_eq!(plan.target_assigned_cores, vec![0]);
    assert_eq!(plan.common_assigned_cores, vec![0]);
    assert!(plan.added_assigned_cores.is_empty());
    assert_eq!(plan.removed_assigned_cores, vec![1]);
    assert!(plan.resize_start_cores.is_empty());
    assert_eq!(plan.resize_stop_cores, vec![1]);
    assert_eq!(plan.target_generation, 0);
    assert_eq!(plan.target_placement.listener_group_snapshot.generation, 1);
    assert_eq!(
        plan.rollout
            .cores
            .iter()
            .map(|core| core.core_id)
            .collect::<Vec<_>>(),
        vec![1]
    );
}

/// Scenario: the submitted pipeline config is effectively identical to the
/// committed active pipeline and serving footprint.
/// Guarantees: rollout planning short-circuits to `NoOp` rather than
/// scheduling a replace or resize operation.
#[test]
fn prepare_rollout_plan_returns_noop_for_identical_active_pipeline() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);
    let _receiver =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);

    let replacement = PipelineConfig::from_yaml(
        "g1".into(),
        "p1".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 1
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("replacement should parse");

    let plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: replacement,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("identical updates should be planned");

    assert_eq!(plan.action, RolloutAction::NoOp);
    assert_eq!(plan.target_generation, 0);
    assert_eq!(plan.target_placement.listener_group_snapshot.generation, 0);
    assert!(plan.rollout.cores.is_empty());
    assert!(plan.resize_start_cores.is_empty());
    assert!(plan.resize_stop_cores.is_empty());
}

/// Scenario: a stable core count resolves to a different non-monotonic NUMA placement.
/// Guarantees: placement divergence triggers replacement on the newly selected cores.
#[test]
fn prepare_rollout_plan_replaces_divergent_non_monotonic_numa_placement() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 6
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let topology = NumaTopology::from_node_cpulists(&[(0, "4-7".into()), (1, "0-3".into())]);
    let runtime = test_runtime_with_topology(&config, topology);
    register_existing_pipeline(&runtime, &config);
    for core_id in [4, 5, 6, 7, 0, 1] {
        let _rx = register_runtime_instance(
            &runtime,
            "g1",
            "p1",
            core_id,
            0,
            RuntimeInstanceLifecycle::Active,
        );
    }

    let replacement = PipelineConfig::from_yaml(
        "g1".into(),
        "p1".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 6
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("replacement should parse");

    let plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: replacement,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("divergent non-monotonic NUMA update should be planned");

    assert_eq!(plan.target_assigned_cores, vec![0, 1, 2, 3, 4, 5]);
    assert_eq!(plan.action, RolloutAction::Replace);
}

/// Scenario: an unchanged effective limiter map moves from engine scope to the
/// pipeline being reconfigured.
/// Guarantees: declaration scope alone is a V1 planning no-op, so the controller
/// does not replace the pipeline and reset otherwise identical receiver buckets.
#[test]
fn prepare_rollout_plan_returns_noop_for_rate_limiter_scope_only_change() {
    let config = OtelDataflowSpec::from_yaml(
        r#"
version: otel_dataflow/v1
policies:
  resources:
    memory_limiter:
      mode: enforce
      source: auto
    rate_limiters:
      ingress:
        enforcement: enforce
        aggregation: receiver_instance
        unit: request_bytes
        pressure: soft
        token_bucket: { allow: 1024, interval: 1s, burst: 1024 }
groups:
  g1:
    pipelines:
      p1:
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    )
    .expect("engine-scoped limiter config should parse");
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);
    let _receiver =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);

    let replacement = PipelineConfig::from_yaml(
        "g1".into(),
        "p1".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 1
    rate_limiters:
      ingress:
        enforcement: enforce
        aggregation: receiver_instance
        unit: request_bytes
        pressure: soft
        token_bucket: { allow: 1024, interval: 1s, burst: 1024 }
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("pipeline-scoped limiter config should parse");

    let plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: replacement,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("scope-only limiter change should be planned");

    assert_eq!(plan.action, RolloutAction::NoOp);
    assert_eq!(plan.target_generation, 0);
    assert!(plan.rollout.cores.is_empty());
}

/// Scenario: the controller executes a rollout plan that has already been
/// classified as `NoOp`.
/// Guarantees: the controller returns an immediate successful rollout
/// snapshot, preserves the committed generation, and leaves no in-flight
/// rollout summary behind.
#[test]
fn spawn_rollout_returns_immediate_success_for_noop() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);
    let _receiver =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);

    let replacement = PipelineConfig::from_yaml(
        "g1".into(),
        "p1".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 1
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("replacement should parse");

    let plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: replacement,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("identical updates should be planned");

    let status = runtime
        .spawn_rollout(plan)
        .expect("noop rollout should succeed");

    assert_eq!(status.action, "noop");
    assert_eq!(status.state, ApiPipelineRolloutState::Succeeded);
    assert_eq!(status.target_generation, 0);
    assert!(status.cores.is_empty());

    let pipeline_key = PipelineKey::new("g1".into(), "p1".into());
    let details = runtime
        .pipeline_details_snapshot(&pipeline_key)
        .expect("group should exist")
        .expect("pipeline should exist");
    assert_eq!(details.active_generation, Some(0));
    assert!(details.rollout.is_none());

    let rollout = runtime
        .rollout_status_snapshot(&status.rollout_id)
        .expect("completed rollout should remain queryable");
    assert_eq!(rollout.state, ApiPipelineRolloutState::Succeeded);
}

/// Scenario: a reconfigure request changes the runtime graph shape while
/// also changing the resource footprint.
/// Guarantees: planning keeps the safer replace path instead of collapsing
/// the update into a resource-only resize.
#[test]
fn prepare_rollout_plan_keeps_replace_when_runtime_shape_changes() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);
    let _receiver =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);

    let replacement = PipelineConfig::from_yaml(
        "g1".into(),
        "p1".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 2
nodes:
  input:
    type: "urn:test:receiver:example"
    config: null
  output:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: input
    to: output
"#,
    )
    .expect("replacement should parse");

    let plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: replacement,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("runtime shape changes should still be planned");

    assert_eq!(plan.action, RolloutAction::Replace);
    assert_eq!(plan.target_generation, 1);
    assert_eq!(plan.common_assigned_cores, vec![0]);
    assert_eq!(plan.added_assigned_cores, vec![1]);
    assert!(plan.resize_start_cores.is_empty());
    assert!(plan.resize_stop_cores.is_empty());
    assert_eq!(
        plan.rollout
            .cores
            .iter()
            .map(|core| core.core_id)
            .collect::<Vec<_>>(),
        vec![0, 1]
    );
}

/// Scenario: a reconfigure request would require a runtime topic-broker
/// mutation for an existing logical pipeline.
/// Guarantees: planning rejects the request before rollout starts and
/// surfaces an invalid-request error to the caller.
#[test]
fn prepare_rollout_plan_rejects_topic_runtime_mutation() {
    let config = OtelDataflowSpec::from_yaml(
        r#"
version: otel_dataflow/v1
topics:
  shared: {}
groups:
  g1:
    pipelines:
      p1:
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          to_topic:
            type: "urn:otel:exporter:topic"
            config:
              topic: shared
        connections:
          - from: receiver
            to: to_topic
"#,
    )
    .expect("config should parse");
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);

    let replacement = PipelineConfig::from_yaml(
        "g1".into(),
        "p1".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 1
nodes:
  from_topic:
    type: "urn:otel:receiver:topic"
    config:
      topic: shared
      subscription:
        mode: balanced
        group: workers
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: from_topic
    to: exporter
"#,
    )
    .expect("replacement should parse");

    let err = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: replacement,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect_err("topic runtime changes should be rejected");

    match err {
        ControlPlaneError::InvalidRequest { message } => {
            assert!(message.contains("topic broker mutation"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

/// Scenario: a second rollout is requested for a logical pipeline that
/// already has an active rollout record.
/// Guarantees: planning rejects the new request with a rollout conflict
/// instead of interleaving two rollout state machines.
#[test]
fn prepare_rollout_plan_rejects_concurrent_rollout_for_same_pipeline() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);

    let replacement = PipelineConfig::from_yaml(
        "g1".into(),
        "p1".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 1
nodes:
  input:
    type: "urn:test:receiver:example"
    config: null
  output:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: input
    to: output
"#,
    )
    .expect("replacement should parse");
    let plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: replacement.clone(),
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("first rollout plan should be accepted");
    runtime
        .insert_rollout(&plan.pipeline_key, plan.rollout.clone())
        .expect("rollout should register");

    let err = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: replacement,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect_err("second rollout should conflict");

    assert_eq!(err, ControlPlaneError::RolloutConflict);
}

/// Scenario: a new rollout has been registered for a logical pipeline but
/// has not yet committed its candidate config.
/// Guarantees: pipeline details still return the committed config while
/// exposing the pending rollout summary separately.
#[test]
fn pipeline_details_returns_committed_config_while_rollout_is_pending() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);

    let replacement = PipelineConfig::from_yaml(
        "g1".into(),
        "p1".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 1
nodes:
  input:
    type: "urn:test:receiver:example"
    config: null
  output:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: input
    to: output
"#,
    )
    .expect("replacement should parse");
    let plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: replacement.clone(),
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("rollout plan should be accepted");
    runtime
        .insert_rollout(&plan.pipeline_key, plan.rollout.clone())
        .expect("rollout should register");

    let details = runtime
        .pipeline_details_snapshot(&PipelineKey::new("g1".into(), "p1".into()))
        .expect("group should exist")
        .expect("pipeline details should exist");

    let mut committed_nodes = details
        .pipeline
        .node_iter()
        .map(|(node_id, _)| node_id.as_ref().to_owned())
        .collect::<Vec<_>>();
    committed_nodes.sort();
    assert_eq!(
        committed_nodes,
        vec!["exporter".to_owned(), "receiver".to_owned()]
    );
    assert_eq!(details.active_generation, Some(0));
    assert_eq!(
        details
            .rollout
            .expect("pending rollout summary should be present")
            .target_generation,
        1
    );
}

/// Scenario: panic diagnostics are captured for a worker panic with explicit
/// thread metadata.
/// Guarantees: the short summary stays operator-friendly while the detailed
/// form includes thread context and a captured backtrace.
#[test]
fn panic_report_formats_summary_and_detail() {
    let report = PanicReport::capture(
        "rollout worker",
        Box::new("boom"),
        Some("rollout-g1-p1".to_owned()),
        Some(17),
        Some(3),
    );

    assert_eq!(report.summary_message(), "rollout worker panicked: boom");
    let detail = report.detail_message();
    assert!(detail.contains("rollout worker panicked: boom"));
    assert!(detail.contains("thread_name=rollout-g1-p1"));
    assert!(detail.contains("thread_id=17"));
    assert!(detail.contains("core_id=3"));
    assert!(detail.contains("backtrace:"));
}

/// Scenario: a panic is raised with a non-string payload.
/// Guarantees: the captured panic summary stays readable and avoids the older
/// generic placeholder text.
#[test]
fn panic_report_non_string_payload_has_useful_fallback() {
    let report = PanicReport::capture("shutdown worker", Box::new(7usize), None, None, None);

    assert_eq!(
        report.summary_message(),
        "shutdown worker panicked: non-string panic payload"
    );
    assert!(
        !report
            .summary_message()
            .contains("panic payload was not a string")
    );
}

/// Scenario: a detached rollout worker panics before it reaches the normal
/// terminal-state bookkeeping path.
/// Guarantees: the rollout is forced into a failed terminal state and the
/// logical pipeline no longer stays blocked by a stale active-rollout entry.
#[test]
fn rollout_worker_panic_marks_failed_and_clears_conflict() {
    let config = engine_config_with_pipeline(simple_pipeline_yaml());
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);

    let replacement = PipelineConfig::from_yaml(
        "g1".into(),
        "p1".into(),
        r#"
nodes:
  input:
    type: "urn:test:receiver:example"
    config: null
  output:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: input
    to: output
"#,
    )
    .expect("replacement should parse");
    let plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: replacement.clone(),
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("rollout plan should be accepted");
    runtime
        .insert_rollout(&plan.pipeline_key, plan.rollout.clone())
        .expect("rollout should register");

    runtime.handle_rollout_worker_panic(
        &plan.pipeline_key,
        &plan.rollout.rollout_id,
        "rollout-g1-p1".to_owned(),
        Box::new("boom"),
    );

    let status = runtime
        .rollout_status_snapshot(&plan.rollout.rollout_id)
        .expect("rollout should remain queryable");
    assert_eq!(status.state, ApiPipelineRolloutState::Failed);
    assert!(
        status
            .failure_reason
            .as_deref()
            .is_some_and(|message| message.contains("rollout worker panicked: boom"))
    );
    assert!(
        status
            .failure_reason
            .as_deref()
            .is_some_and(|message| !message.contains("backtrace:"))
    );

    let state = runtime
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    assert!(!state.active_rollouts.contains_key(&plan.pipeline_key));
    drop(state);

    let _next_plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: replacement,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("rollout conflict should be cleared after panic cleanup");
}

/// Scenario: a rollout worker panics after launching an uncommitted candidate
/// generation.
/// Guarantees: panic cleanup requests shutdown for the candidate generation
/// before clearing the active rollout, avoiding active orphan instances.
#[test]
fn rollout_worker_panic_requests_shutdown_for_uncommitted_candidate_generation() {
    let config = engine_config_with_pipeline(simple_pipeline_yaml());
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);

    let replacement = PipelineConfig::from_yaml(
        "g1".into(),
        "p1".into(),
        r#"
nodes:
  input:
    type: "urn:test:receiver:example"
    config: null
  output:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: input
    to: output
"#,
    )
    .expect("replacement should parse");
    let plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: replacement,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("rollout plan should be accepted");
    runtime
        .insert_rollout(&plan.pipeline_key, plan.rollout.clone())
        .expect("rollout should register");

    let candidate_key = deployed_key("g1", "p1", 0, plan.target_generation);
    let mut candidate_rx = register_runtime_instance(
        &runtime,
        "g1",
        "p1",
        0,
        plan.target_generation,
        RuntimeInstanceLifecycle::Active,
    );

    runtime.handle_rollout_worker_panic(
        &plan.pipeline_key,
        &plan.rollout.rollout_id,
        "rollout-g1-p1".to_owned(),
        Box::new("boom"),
    );

    assert!(matches!(
        wait_for_shutdown_message(&mut candidate_rx),
        RuntimeControlMsg::Shutdown { reason, .. } if reason == "rollout panic cleanup"
    ));

    let state = runtime
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    assert!(
        state
            .runtime_instances
            .get(&candidate_key)
            .expect("candidate instance should still be tracked until exit")
            .control_sender
            .is_none(),
        "panic cleanup should release the retained sender after shutdown dispatch"
    );
    assert!(!state.active_rollouts.contains_key(&plan.pipeline_key));
}

/// Scenario: a rollout worker panics after the target generation was already
/// committed as serving.
/// Guarantees: panic cleanup does not shut down the committed generation,
/// which would turn a late bookkeeping panic into runtime outage.
#[test]
fn rollout_worker_panic_does_not_shutdown_committed_target_generation() {
    let config = engine_config_with_pipeline(simple_pipeline_yaml());
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);

    let replacement = PipelineConfig::from_yaml(
        "g1".into(),
        "p1".into(),
        r#"
nodes:
  input:
    type: "urn:test:receiver:example"
    config: null
  output:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: input
    to: output
"#,
    )
    .expect("replacement should parse");
    let plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: replacement,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("rollout plan should be accepted");
    runtime
        .insert_rollout(&plan.pipeline_key, plan.rollout.clone())
        .expect("rollout should register");
    runtime.commit_pipeline_record(&plan, plan.target_generation);

    let mut candidate_rx = register_runtime_instance(
        &runtime,
        "g1",
        "p1",
        0,
        plan.target_generation,
        RuntimeInstanceLifecycle::Active,
    );

    runtime.handle_rollout_worker_panic(
        &plan.pipeline_key,
        &plan.rollout.rollout_id,
        "rollout-g1-p1".to_owned(),
        Box::new("boom"),
    );

    assert!(
        candidate_rx.try_recv().is_err(),
        "committed target generation must not receive panic-cleanup shutdown"
    );
}

/// Scenario: a resize rollback must clean up cores that were already started
/// before a later step fails.
/// Guarantees: rollback sends shutdown to those started cores instead of
/// leaving them running after the rollout fails.
#[test]
fn rollback_resize_rollout_cleans_up_started_cores() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);
    let _existing =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);

    let replacement = PipelineConfig::from_yaml(
        "g1".into(),
        "p1".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 2
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("replacement should parse");
    let plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: replacement,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("resize rollout plan should be accepted");
    runtime
        .insert_rollout(&plan.pipeline_key, plan.rollout.clone())
        .expect("rollout should register");

    let started_key = deployed_key("g1", "p1", 1, plan.target_generation);
    let started_rx = register_runtime_instance(
        &runtime,
        "g1",
        "p1",
        1,
        plan.target_generation,
        RuntimeInstanceLifecycle::Active,
    );
    let exit_thread = complete_instance_exit_on_shutdown(
        Arc::clone(&runtime),
        started_rx,
        started_key.clone(),
        "rollback cleanup",
    );

    let result = runtime.rollback_resize_rollout(&plan, &[1], &[], "boom".to_owned());

    assert!(matches!(
        result,
        Err(RolloutExecutionError::Failed(reason)) if reason == "boom"
    ));
    exit_thread
        .join()
        .expect("resize rollback shutdown helper should join cleanly");
    assert!(matches!(
        runtime.instance_exit(&started_key),
        Some(RuntimeInstanceExit::Success)
    ));
}

/// Scenario: a replace rollback must clean up added candidate cores that were
/// already serving the target generation before a later step fails.
/// Guarantees: rollback sends shutdown to those activated added cores instead
/// of leaving the candidate generation running.
#[test]
fn rollback_replace_rollout_cleans_up_activated_added_cores() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);
    let _existing =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);

    let replacement = PipelineConfig::from_yaml(
        "g1".into(),
        "p1".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 2
nodes:
  input:
    type: "urn:test:receiver:example"
    config: null
  output:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: input
    to: output
"#,
    )
    .expect("replacement should parse");
    let plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: replacement,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("replace rollout plan should be accepted");
    runtime
        .insert_rollout(&plan.pipeline_key, plan.rollout.clone())
        .expect("rollout should register");

    let added_key = deployed_key("g1", "p1", 1, plan.target_generation);
    let added_rx = register_runtime_instance(
        &runtime,
        "g1",
        "p1",
        1,
        plan.target_generation,
        RuntimeInstanceLifecycle::Active,
    );
    let exit_thread = complete_instance_exit_on_shutdown(
        Arc::clone(&runtime),
        added_rx,
        added_key.clone(),
        "rollback cleanup",
    );

    let result = runtime.rollback_replace_rollout(&plan, &[], &[1], &[], "boom".to_owned());

    assert!(matches!(
        result,
        Err(RolloutExecutionError::Failed(reason)) if reason == "boom"
    ));
    exit_thread
        .join()
        .expect("replace rollback shutdown helper should join cleanly");
    assert!(matches!(
        runtime.instance_exit(&added_key),
        Some(RuntimeInstanceExit::Success)
    ));
}

/// Scenario: a replace rollout starts with one recovered core on generation 1
/// and one sibling on committed generation 0, then fails before switching cores.
/// Guarantees: rollback retains the recovered core's serving override so status
/// and compaction continue to select both pre-rollout runtime instances.
#[test]
fn rollback_replace_rollout_restores_recovered_serving_generation() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 2
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    let _runner = ObservedStateRunner::start(&runtime);
    register_existing_pipeline(&runtime, &config);
    let _recovered =
        register_runtime_instance(&runtime, "g1", "p1", 0, 1, RuntimeInstanceLifecycle::Active);
    let _sibling =
        register_runtime_instance(&runtime, "g1", "p1", 1, 0, RuntimeInstanceLifecycle::Active);
    report_ready(&runtime, deployed_key("g1", "p1", 0, 1));
    report_ready(&runtime, deployed_key("g1", "p1", 1, 0));

    let pipeline_key = PipelineKey::new("g1".into(), "p1".into());
    {
        let mut state = runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let _ = state.generation_counters.insert(pipeline_key.clone(), 2);
        let context_bindings = Arc::clone(&state.latest_context_bindings);
        let _ = state.runtime_recoveries.insert(
            (pipeline_key.clone(), 0),
            RuntimeRecoveryState {
                serving_generation: 1,
                context_bindings,
                restart_count: 1,
                ready_since: Some(Instant::now()),
                worker_id: None,
                candidate_generation: None,
                cancel_requested: false,
            },
        );
    }
    runtime
        .observed_state_store
        .set_pipeline_serving_generation(pipeline_key.clone(), 0, 1);
    let _ = wait_for_observed_status(&runtime, &pipeline_key, |status| {
        status.total_cores() == 2 && status.running_cores() == 2
    });

    let replacement = PipelineConfig::from_yaml(
        "g1".into(),
        "p1".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 2
nodes:
  input:
    type: "urn:test:receiver:example"
    config: null
  output:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: input
    to: output
"#,
    )
    .expect("replacement should parse");
    let plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: replacement,
                step_timeout_secs: 2,
                drain_timeout_secs: 2,
            },
        )
        .expect("mixed-generation replace rollout should be planned");
    assert_eq!(plan.current_serving_generations.get(&0), Some(&1));
    assert_eq!(plan.current_serving_generations.get(&1), Some(&0));

    let result = runtime.rollback_replace_rollout(&plan, &[], &[], &[], "boom".to_owned());

    assert!(matches!(
        result,
        Err(RolloutExecutionError::Failed(reason)) if reason == "boom"
    ));
    let status = runtime
        .observed_state_handle
        .pipeline_status(&pipeline_key)
        .expect("pipeline status should remain available");
    assert_eq!(status.serving_generations().get(&0), Some(&1));
    assert_eq!(status.total_cores(), 2);
    assert_eq!(status.running_cores(), 2);
    assert!(status.readiness());

    runtime
        .observed_state_store
        .compact_pipeline_instances(&pipeline_key);
    let compacted = runtime
        .observed_state_handle
        .pipeline_status(&pipeline_key)
        .expect("compacted pipeline status should remain available");
    assert!(compacted.instance_status(0, 1).is_some());
    assert!(compacted.instance_status(1, 0).is_some());
}

/// Scenario: a shutdown request targets a group id that does not exist in
/// the controller's committed config.
/// Guarantees: per-pipeline shutdown fails fast with `GroupNotFound`
/// instead of creating a shutdown record.
#[test]
fn request_shutdown_pipeline_rejects_missing_group() {
    let config = engine_config_with_pipeline(
        r#"
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);

    let err = runtime
        .request_shutdown_pipeline("missing", "p1", 5)
        .expect_err("missing group should be rejected");

    assert_eq!(err, ControlPlaneError::GroupNotFound);
}

/// Scenario: a shutdown request targets a pipeline id that is not present
/// in an existing group.
/// Guarantees: per-pipeline shutdown rejects the request with
/// `PipelineNotFound` before any runtime instances are touched.
#[test]
fn request_shutdown_pipeline_rejects_missing_pipeline() {
    let config = engine_config_with_pipeline(
        r#"
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);

    let err = runtime
        .request_shutdown_pipeline("g1", "missing", 5)
        .expect_err("missing pipeline should be rejected");

    assert_eq!(err, ControlPlaneError::PipelineNotFound);
}

/// Scenario: a control-plane caller creates a new empty pipeline group after
/// the controller has started with no groups.
/// Guarantees: the group is added to committed live config and can be read
/// back without registering any logical pipelines.
#[test]
fn create_group_adds_empty_group_to_live_config() {
    let config = empty_engine_config();
    let runtime = test_runtime(&config);
    let group = PipelineGroupConfig::new();

    let created = runtime
        .create_group("g1", group.clone())
        .expect("empty group should be created");

    let group_id: PipelineGroupId = "g1".to_string().into();
    assert_eq!(created, group);
    assert_eq!(runtime.group_details_snapshot(&group_id), Some(group));
    let snapshot = runtime.engine_config_snapshot();
    assert!(snapshot.groups.contains_key(&group_id));
    assert!(snapshot.groups[&group_id].pipelines.is_empty());
}

/// Scenario: a control-plane caller attempts to create a group that is already
/// present in committed live config.
/// Guarantees: the runtime rejects the request with a conflict-level error and
/// leaves the existing group unchanged.
#[test]
fn create_group_rejects_duplicate_group() {
    let config = engine_config_with_pipeline(simple_pipeline_yaml());
    let runtime = test_runtime(&config);

    let err = runtime
        .create_group("g1", PipelineGroupConfig::new())
        .expect_err("duplicate group should be rejected");

    assert_eq!(err, ControlPlaneError::GroupAlreadyExists);
    let group_id: PipelineGroupId = "g1".to_string().into();
    assert!(runtime.group_details_snapshot(&group_id).is_some());
}

/// Scenario: a control-plane caller submits a group-create payload that already
/// contains pipelines.
/// Guarantees: PR1's endpoint boundary stays scoped to empty group creation,
/// leaving pipeline creation to the existing pipeline reconfigure endpoint.
#[test]
fn create_group_rejects_payload_with_pipelines() {
    let config = empty_engine_config();
    let runtime = test_runtime(&config);
    let pipeline = PipelineConfig::from_yaml("g1".into(), "p1".into(), simple_pipeline_yaml())
        .expect("pipeline should parse");
    let mut group = PipelineGroupConfig::new();
    _ = group.pipelines.insert("p1".to_string().into(), pipeline);

    let err = runtime
        .create_group("g1", group)
        .expect_err("non-empty group should be rejected");

    assert!(matches!(
        err,
        ControlPlaneError::InvalidRequest { ref message }
            if message == "pipeline group creation only supports empty groups"
    ));
    let group_id: PipelineGroupId = "g1".to_string().into();
    assert!(runtime.group_details_snapshot(&group_id).is_none());
}

/// Scenario: generation 0 prunes an unconsumed local extension variant, then
/// live reload adds a node that consumes the local capability.
/// Guarantees: the replacement generation constructs and starts a fresh local
/// variant while the disposed generation-0 local variant remains stopped.
#[test]
fn live_reload_recreates_previously_pruned_local_extension_variant() {
    let initial_probe = register_variant_reload_probe("variant-reload-unused-to-needed-initial");
    let target_probe = register_variant_reload_probe("variant-reload-unused-to-needed-target");
    let config = variant_reload_engine_config("variant-reload-unused-to-needed-initial", false);
    let runtime = test_runtime_with_factory(&config, &VARIANT_RELOAD_TEST_PIPELINE_FACTORY);
    let _observed_state_runner = ObservedStateRunner::start(&runtime);
    let _initial_key = launch_committed_test_pipeline(&runtime, &config);

    wait_for_atomic_count(&initial_probe.local.created, 1, "initial local creation");
    wait_for_atomic_count(&initial_probe.shared.created, 1, "initial shared creation");
    wait_for_atomic_count(&initial_probe.local.dropped, 1, "initial local drop");
    wait_for_atomic_count(&initial_probe.shared.started, 1, "initial shared start");
    assert_eq!(initial_probe.local.started.load(Ordering::SeqCst), 0);
    assert_eq!(initial_probe.local.stopped.load(Ordering::SeqCst), 0);
    assert_eq!(initial_probe.shared.stopped.load(Ordering::SeqCst), 0);
    assert_eq!(initial_probe.shared.dropped.load(Ordering::SeqCst), 0);

    let plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: variant_reload_pipeline_config(
                    "variant-reload-unused-to-needed-target",
                    true,
                ),
                step_timeout_secs: 5,
                drain_timeout_secs: 5,
            },
        )
        .expect("variant-enabling replacement should plan");
    assert_eq!(plan.action, RolloutAction::Replace);
    let accepted = runtime
        .spawn_rollout(plan)
        .expect("variant-enabling replacement should start");
    let status = wait_for_terminal_rollout(&runtime, &accepted.rollout_id);
    assert_eq!(
        status.state,
        ApiPipelineRolloutState::Succeeded,
        "variant-enabling rollout failed: {:?}",
        status.failure_reason
    );

    wait_for_atomic_count(&target_probe.local.created, 1, "target local creation");
    wait_for_atomic_count(&target_probe.shared.created, 1, "target shared creation");
    wait_for_atomic_count(&target_probe.local.started, 1, "target local start");
    wait_for_atomic_count(&target_probe.shared.started, 1, "target shared start");
    wait_for_atomic_count(&initial_probe.shared.stopped, 1, "initial shared stop");
    wait_for_atomic_count(&initial_probe.shared.dropped, 1, "initial shared drop");
    assert_eq!(initial_probe.local.started.load(Ordering::SeqCst), 0);
    assert_eq!(initial_probe.local.stopped.load(Ordering::SeqCst), 0);
    assert_eq!(target_probe.local.stopped.load(Ordering::SeqCst), 0);
    assert_eq!(target_probe.local.dropped.load(Ordering::SeqCst), 0);
    assert_eq!(target_probe.shared.stopped.load(Ordering::SeqCst), 0);
    assert_eq!(target_probe.shared.dropped.load(Ordering::SeqCst), 0);

    let target_key = deployed_key("g1", "p1", 0, status.target_generation);
    shutdown_test_pipeline(&runtime, &target_key);
    wait_for_atomic_count(&target_probe.local.stopped, 1, "target local stop");
    wait_for_atomic_count(&target_probe.local.dropped, 1, "target local drop");
    wait_for_atomic_count(&target_probe.shared.stopped, 1, "target shared stop");
    wait_for_atomic_count(&target_probe.shared.dropped, 1, "target shared drop");
}

/// Scenario: generation 0 uses both extension variants, then live reload
/// removes the only node that consumes the local capability.
/// Guarantees: the replacement generation drops its newly constructed local
/// variant before startup while retaining and starting the shared variant.
#[test]
fn live_reload_prunes_local_extension_variant_after_last_consumer_is_removed() {
    let initial_probe = register_variant_reload_probe("variant-reload-needed-to-unused-initial");
    let target_probe = register_variant_reload_probe("variant-reload-needed-to-unused-target");
    let config = variant_reload_engine_config("variant-reload-needed-to-unused-initial", true);
    let runtime = test_runtime_with_factory(&config, &VARIANT_RELOAD_TEST_PIPELINE_FACTORY);
    let _observed_state_runner = ObservedStateRunner::start(&runtime);
    let _initial_key = launch_committed_test_pipeline(&runtime, &config);

    wait_for_atomic_count(&initial_probe.local.created, 1, "initial local creation");
    wait_for_atomic_count(&initial_probe.shared.created, 1, "initial shared creation");
    wait_for_atomic_count(&initial_probe.local.started, 1, "initial local start");
    wait_for_atomic_count(&initial_probe.shared.started, 1, "initial shared start");
    assert_eq!(initial_probe.local.stopped.load(Ordering::SeqCst), 0);
    assert_eq!(initial_probe.local.dropped.load(Ordering::SeqCst), 0);
    assert_eq!(initial_probe.shared.stopped.load(Ordering::SeqCst), 0);
    assert_eq!(initial_probe.shared.dropped.load(Ordering::SeqCst), 0);

    let plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: variant_reload_pipeline_config(
                    "variant-reload-needed-to-unused-target",
                    false,
                ),
                step_timeout_secs: 5,
                drain_timeout_secs: 5,
            },
        )
        .expect("variant-pruning replacement should plan");
    assert_eq!(plan.action, RolloutAction::Replace);
    let accepted = runtime
        .spawn_rollout(plan)
        .expect("variant-pruning replacement should start");
    let status = wait_for_terminal_rollout(&runtime, &accepted.rollout_id);
    assert_eq!(
        status.state,
        ApiPipelineRolloutState::Succeeded,
        "variant-pruning rollout failed: {:?}",
        status.failure_reason
    );

    wait_for_atomic_count(&target_probe.local.created, 1, "target local creation");
    wait_for_atomic_count(&target_probe.shared.created, 1, "target shared creation");
    wait_for_atomic_count(&target_probe.local.dropped, 1, "target local drop");
    wait_for_atomic_count(&target_probe.shared.started, 1, "target shared start");
    wait_for_atomic_count(&initial_probe.local.stopped, 1, "initial local stop");
    wait_for_atomic_count(&initial_probe.local.dropped, 1, "initial local drop");
    wait_for_atomic_count(&initial_probe.shared.stopped, 1, "initial shared stop");
    wait_for_atomic_count(&initial_probe.shared.dropped, 1, "initial shared drop");
    assert_eq!(target_probe.local.started.load(Ordering::SeqCst), 0);
    assert_eq!(target_probe.local.stopped.load(Ordering::SeqCst), 0);
    assert_eq!(target_probe.shared.stopped.load(Ordering::SeqCst), 0);
    assert_eq!(target_probe.shared.dropped.load(Ordering::SeqCst), 0);

    let target_key = deployed_key("g1", "p1", 0, status.target_generation);
    shutdown_test_pipeline(&runtime, &target_key);
    assert_eq!(target_probe.local.started.load(Ordering::SeqCst), 0);
    assert_eq!(target_probe.local.stopped.load(Ordering::SeqCst), 0);
    wait_for_atomic_count(&target_probe.shared.stopped, 1, "target shared stop");
    wait_for_atomic_count(&target_probe.shared.dropped, 1, "target shared drop");
}

/// Scenario: A pipeline using an engine-scoped extension is replaced while its provider host runs.
/// Guarantees: Both generations bind one retained host, which reports scoped metrics after cutover.
#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn live_reload_retains_engine_extension_host_and_metric_collection() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(exercise_outer_scope_extension_live_reconfig(
            OuterScopeLiveReconfigDeclarationScope::Engine,
        ))
        .await;
}

/// Scenario: A pipeline using a group-scoped extension is replaced inside that pipeline group.
/// Guarantees: Both generations bind one retained group host, which reports metrics after cutover.
#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn live_reload_retains_pipeline_group_extension_host_and_metric_collection() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(exercise_outer_scope_extension_live_reconfig(
            OuterScopeLiveReconfigDeclarationScope::PipelineGroup,
        ))
        .await;
}

/// Scenario: a committed pipeline inherits an engine provider, then a replacement
/// shadows the same extension ID with a pipeline-local declaration.
/// Guarantees: rollout planning and commit retain exact, generation-specific
/// inherited snapshots instead of consulting mutable scope-registry state.
#[test]
fn rollout_generation_records_exact_inherited_extension_snapshot() {
    let config = OtelDataflowSpec::from_yaml(
        r#"
version: otel_dataflow/v1
extensions:
  root_auth:
    type: urn:test:extension:scope-shared
groups:
  g1:
    pipelines:
      p1:
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          receiver:
            type: urn:test:receiver:example
          exporter:
            type: urn:test:exporter:example
        connections:
          - from: receiver
            to: exporter
"#,
    )
    .expect("config should parse");

    let async_runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("test runtime should build");
    let local = tokio::task::LocalSet::new();
    async_runtime.block_on(local.run_until(async {
        let extension_scope_registry = ExtensionScopeRegistry::default();
        let controller_context = ControllerContext::new(TelemetryRegistryHandle::new());
        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(8);
        let prepared = TEST_PIPELINE_FACTORY
            .prepare_extension_scope_hosts(
                &config,
                &controller_context,
                metrics_reporter,
                extension_scope_registry.clone(),
            )
            .expect("extension scope hosts should prepare");
        let running = prepared
            .start()
            .await
            .expect("extension scope hosts should start");

        let runtime = test_runtime_with_extension_scope_registry(&config, extension_scope_registry);
        register_existing_pipeline(&runtime, &config);
        let _control =
            register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);
        let replacement = PipelineConfig::from_yaml(
            "g1".into(),
            "p1".into(),
            r#"
extensions:
  root_auth:
    type: urn:test:extension:scope-shared
policies:
  resources:
    core_allocation:
      type: core_count
      count: 1
nodes:
  receiver:
    type: urn:test:receiver:example
  exporter:
    type: urn:test:exporter:example
connections:
  - from: receiver
    to: exporter
"#,
        )
        .expect("replacement should parse");

        let plan = runtime
            .prepare_rollout_plan(
                "g1",
                "p1",
                &ReconfigureRequest {
                    pipeline: replacement,
                    step_timeout_secs: 5,
                    drain_timeout_secs: 5,
                },
            )
            .expect("shadowing replacement should plan");

        assert_eq!(plan.action, RolloutAction::Replace);
        assert!(
            !plan
                .current_record
                .as_ref()
                .expect("replacement should retain its previous record")
                .inherited_extensions
                .is_empty()
        );
        assert!(
            plan.target_inherited_extensions.is_empty(),
            "pipeline-local shadowing must be frozen into the target generation"
        );

        runtime.commit_pipeline_record(&plan, plan.target_generation);
        assert!(
            runtime
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .logical_pipelines
                .get(&plan.pipeline_key)
                .expect("target generation should be committed")
                .inherited_extensions
                .is_empty()
        );

        runtime.note_instance_exit(deployed_key("g1", "p1", 0, 0), RuntimeInstanceExit::Success);
        running
            .shutdown()
            .await
            .expect("extension scope hosts should stop cleanly");
    }));
}

/// Scenario: a control-plane caller deletes a stopped logical pipeline.
/// Guarantees: the pipeline is removed from committed live config and the
/// containing group remains available as an empty group.
#[test]
fn delete_pipeline_removes_stopped_pipeline_from_live_config() {
    let config = engine_config_with_pipeline(simple_pipeline_yaml());
    let runtime = test_runtime(&config);

    let status = runtime
        .request_delete_pipeline("g1", "p1", 5)
        .expect("stopped pipeline should be deleted");

    assert_eq!(status.state, "succeeded");
    assert!(status.shutdown.is_none());
    let snapshot = runtime.engine_config_snapshot();
    let group_id: PipelineGroupId = "g1".into();
    let pipeline_id: PipelineId = "p1".into();
    assert!(snapshot.groups.contains_key(&group_id));
    assert!(
        !snapshot.groups[&group_id]
            .pipelines
            .contains_key(&pipeline_id)
    );
}

/// Scenario: a pipeline with context declarations is launched and deleted.
/// Guarantees: the runtime receives its policy. Deletion updates the policy and releases the old one.
#[test]
fn delete_pipeline_recompiles_context_bindings_without_removed_declarations() {
    let _capture_guard = CONTEXT_BINDINGS_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    reset_context_bindings_test_capture();
    let config = engine_config_with_pipeline(
        r#"
        nodes:
          receiver:
            type: "urn:test:receiver:context-bindings"
            config:
              produces: X-Tenant
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
        "#,
    );
    let runtime = test_runtime_with_factory(&config, &CONTEXT_BINDINGS_TEST_PIPELINE_FACTORY);
    let _runner = ObservedStateRunner::start(&runtime);
    let initial_bindings = {
        let state = runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        Arc::clone(&state.latest_context_bindings)
    };
    let resolved = config
        .resolve()
        .pipelines
        .into_iter()
        .find(|pipeline| {
            pipeline.role == ResolvedPipelineRole::Regular
                && pipeline.pipeline_group_id.as_ref() == "g1"
                && pipeline.pipeline_id.as_ref() == "p1"
        })
        .expect("resolved pipeline should exist");
    let placement = runtime
        .pipeline_placement_for_resolved(&resolved)
        .expect("resolved pipeline placement should exist");
    let live_placement = runtime.live_pipeline_placement_from(&resolved, placement.clone(), 0);
    runtime.register_committed_pipeline(resolved.clone(), placement, 0);
    let core_id = live_placement
        .placement
        .cores
        .first()
        .expect("pipeline should have a core")
        .core_id
        .id;
    let _deployed_key = runtime
        .launch_regular_pipeline_instance(
            &resolved,
            &runtime
                .inherited_extensions_for_pipeline(&resolved.pipeline_group_id, &resolved.pipeline),
            Arc::clone(&initial_bindings),
            &live_placement,
            core_id,
            0,
        )
        .expect("pipeline should launch");
    let installed_bindings = wait_for_context_bindings_test_capture();
    assert!(Arc::ptr_eq(
        &initial_bindings,
        &installed_bindings
            .upgrade()
            .expect("installed policy should be live")
    ));
    drop(initial_bindings);

    let status = runtime
        .request_delete_pipeline("g1", "p1", 5)
        .expect("stopped pipeline should be deleted");
    assert_eq!(status.state, "succeeded");

    let (committed_config, committed_policy) = {
        let state = runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        (
            state.live_config.clone(),
            Arc::clone(&state.latest_context_bindings),
        )
    };
    let expected_bindings = CONTEXT_BINDINGS_TEST_PIPELINE_FACTORY
        .compile_initial_context(&committed_config.resolve())
        .expect("post-delete policy should compile")
        .bindings;

    assert!(committed_policy.eq(&expected_bindings));
    assert!(
        installed_bindings.upgrade().is_none(),
        "deleted pipeline policy should be released"
    );
}

/// Scenario: deleting a pipeline recompiles declarations for a remaining pipeline.
/// Guarantees: declaration callbacks run without holding the controller state lock.
#[test]
fn delete_pipeline_compiles_context_bindings_outside_controller_lock() {
    let _capture_guard = CONTEXT_BINDINGS_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let config = OtelDataflowSpec::from_yaml(
        r#"
version: otel_dataflow/v1
groups:
  g1:
    pipelines:
      delete:
        nodes:
          receiver:
            type: "urn:test:receiver:context-bindings"
            config:
              produces: delete-marker
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
      remain:
        nodes:
          receiver:
            type: "urn:test:receiver:context-bindings"
            config:
              produces: remain-marker
              probe_controller_lock: true
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    )
    .expect("engine config should parse");
    let runtime = test_runtime_with_factory(&config, &CONTEXT_BINDINGS_TEST_PIPELINE_FACTORY);
    CONTEXT_BINDINGS_TEST_DECLARATION_CALLS.store(0, Ordering::Relaxed);
    *CONTEXT_BINDINGS_TEST_RUNTIME
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(Arc::downgrade(&runtime));

    let status = runtime
        .request_delete_pipeline("g1", "delete", 5)
        .expect("deletion should compile declarations without the state lock");

    *CONTEXT_BINDINGS_TEST_RUNTIME
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
    assert_eq!(status.state, "succeeded");
    assert_eq!(
        CONTEXT_BINDINGS_TEST_DECLARATION_CALLS.load(Ordering::Relaxed),
        1
    );
}

/// Scenario: reconfiguration changes a deployed pipeline's context declaration.
/// Guarantees: the update is rejected because old and new generations may overlap.
#[test]
fn reconfigure_rejects_context_bindings_changes_to_target_pipeline() {
    let config = engine_config_with_pipeline(
        r#"
        nodes:
          receiver:
            type: "urn:test:receiver:context-bindings"
            config:
              produces: X-Tenant
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
        "#,
    );
    let runtime = test_runtime_with_factory(&config, &CONTEXT_BINDINGS_TEST_PIPELINE_FACTORY);
    register_existing_pipeline(&runtime, &config);

    let replacement = PipelineConfig::from_yaml(
        "g1".into(),
        "p1".into(),
        r#"
nodes:
  receiver:
    type: "urn:test:receiver:context-bindings"
    config:
      produces: X-Account
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("replacement should parse");
    let error = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: replacement,
                step_timeout_secs: 5,
                drain_timeout_secs: 5,
            },
        )
        .expect_err("target pipeline context binding changes should be rejected");

    match error {
        ControlPlaneError::InvalidRequest { message } => {
            assert!(message.contains("g1:p1"), "{message}");
            assert!(message.contains("restart the engine"), "{message}");
        }
        other => panic!("expected invalid request, got {other:?}"),
    }
}

/// Scenario: one pipeline starts requiring original names that the engine currently discards.
/// Guarantees: the live update is rejected because the engine-wide representation is immutable.
#[test]
fn reconfigure_rejects_context_bindings_changes_to_other_pipelines() {
    let config = OtelDataflowSpec::from_yaml(
        r#"
version: otel_dataflow/v1
groups:
  g1:
    pipelines:
      capture:
        nodes:
          receiver:
            type: "urn:test:receiver:context-bindings"
            header_capture:
              headers:
                - match_names: ["x-tenant-id"]
                  store_as: tenant
            config:
              produces: capture-marker
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
      propagate:
        nodes:
          receiver:
            type: "urn:test:receiver:context-bindings"
            config:
              produces: source-marker
          exporter:
            type: "urn:test:exporter:example"
            header_propagation:
              default:
                selector:
                  type: all_captured
                name: stored_name
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    )
    .expect("engine config should parse");
    let runtime = test_runtime_with_factory(&config, &CONTEXT_BINDINGS_TEST_PIPELINE_FACTORY);
    register_pipeline(&runtime, &config, "g1", "capture");
    register_pipeline(&runtime, &config, "g1", "propagate");

    let replacement = PipelineConfig::from_yaml(
        "g1".into(),
        "propagate".into(),
        r#"
nodes:
  receiver:
    type: "urn:test:receiver:context-bindings"
    config:
      produces: source-marker
  exporter:
    type: "urn:test:exporter:example"
    header_propagation:
      default:
        selector:
          type: all_captured
        name: preserve
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("replacement should parse");

    let error = runtime
        .prepare_rollout_plan(
            "g1",
            "propagate",
            &ReconfigureRequest {
                pipeline: replacement,
                step_timeout_secs: 5,
                drain_timeout_secs: 5,
            },
        )
        .expect_err("cross-pipeline context binding changes should be rejected");

    match error {
        ControlPlaneError::InvalidRequest { message } => {
            assert!(
                message.contains("original transport-header names"),
                "{message}"
            );
            assert!(message.contains("restart the engine"), "{message}");
        }
        other => panic!("expected invalid request, got {other:?}"),
    }
}

/// Scenario: deleting the last original-name consumer leaves a capture pipeline deployed.
/// Guarantees: deletion succeeds and the remaining generation keeps preserving original names.
#[test]
fn delete_preserves_installed_context_runtime_requirements() {
    let config = OtelDataflowSpec::from_yaml(
        r#"
version: otel_dataflow/v1
groups:
  g1:
    pipelines:
      capture:
        nodes:
          receiver:
            type: "urn:test:receiver:context-bindings"
            header_capture:
              headers:
                - match_names: ["x-tenant-id"]
                  store_as: tenant
            config:
              produces: capture-marker
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
      propagate:
        nodes:
          receiver:
            type: "urn:test:receiver:context-bindings"
            config:
              produces: source-marker
          exporter:
            type: "urn:test:exporter:example"
            header_propagation:
              default:
                selector:
                  type: all_captured
                name: preserve
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    )
    .expect("engine config should parse");
    let runtime = test_runtime_with_factory(&config, &CONTEXT_BINDINGS_TEST_PIPELINE_FACTORY);
    register_pipeline(&runtime, &config, "g1", "capture");
    register_pipeline(&runtime, &config, "g1", "propagate");
    let initial_bindings = {
        let state = runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        Arc::clone(&state.latest_context_bindings)
    };

    let status = runtime
        .request_delete_pipeline("g1", "propagate", 5)
        .expect("removing an original-name consumer should remain compatible");

    let state = runtime
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    assert_eq!(status.state, "succeeded");
    assert!(
        !state.live_config.groups[&PipelineGroupId::from("g1")]
            .pipelines
            .contains_key(&PipelineId::from("propagate"))
    );
    assert!(initial_bindings.pipeline_bindings_match(
        &state.latest_context_bindings,
        &PipelineKey::new("g1".into(), "capture".into())
    ));
    let unpinned_bindings = CONTEXT_BINDINGS_TEST_PIPELINE_FACTORY
        .compile_initial_context(&state.live_config.resolve())
        .expect("remaining config should compile without installed requirements")
        .bindings;
    assert!(!state.latest_context_bindings.pipeline_bindings_match(
        &unpinned_bindings,
        &PipelineKey::new("g1".into(), "capture".into())
    ));
}

/// Scenario: a new pipeline requires an original name already preserved by the engine.
/// Guarantees: the compatible pipeline is accepted without changing deployed bindings.
#[test]
fn reconfigure_accepts_supported_context_runtime_requirements() {
    let config = engine_config_with_pipeline(
        r#"
        nodes:
          receiver:
            type: "urn:test:receiver:context-bindings"
            config:
              produces: source-marker
          exporter:
            type: "urn:test:exporter:example"
            header_propagation:
              default:
                selector:
                  type: named
                  named: [x-tenant]
                name: preserve
            config: null
        connections:
          - from: receiver
            to: exporter
        "#,
    );
    let runtime = test_runtime_with_factory(&config, &CONTEXT_BINDINGS_TEST_PIPELINE_FACTORY);
    register_existing_pipeline(&runtime, &config);
    let pipeline =
        config.groups[&PipelineGroupId::from("g1")].pipelines[&PipelineId::from("p1")].clone();

    let plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p2",
            &ReconfigureRequest {
                pipeline,
                step_timeout_secs: 5,
                drain_timeout_secs: 5,
            },
        )
        .expect("installed original-name requirements should support the new pipeline");

    assert_eq!(plan.action, RolloutAction::Create);
}

/// Scenario: recovery restarts a failed pipeline generation.
/// Guarantees: recovery reuses that generation's binding snapshot.
#[test]
fn runtime_recovery_reuses_context_bindings_snapshot() {
    assert_runtime_recovery_reuses_context_bindings_snapshot(false);
}

/// Scenario: a reserved pipeline fails before activation after the controller's latest bindings change.
/// Guarantees: recovery retains the failed launch's compiled context snapshot rather than the latest one.
#[test]
fn runtime_recovery_before_activation_reuses_context_bindings_snapshot() {
    assert_runtime_recovery_reuses_context_bindings_snapshot(true);
}

fn assert_runtime_recovery_reuses_context_bindings_snapshot(exit_before_activation: bool) {
    let _capture_guard = CONTEXT_BINDINGS_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    reset_context_bindings_test_capture();
    let config = engine_config_with_pipeline(
        r#"
        policies:
          runtime_recovery:
            max_restarts: 1
            initial_backoff: 1ms
            max_backoff: 1ms
            startup_timeout: 2s
            reset_after: 1m
        nodes:
          receiver:
            type: "urn:test:receiver:context-bindings"
            config:
              produces: X-Tenant
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
        "#,
    );
    let runtime = test_runtime_with_factory(&config, &CONTEXT_BINDINGS_TEST_PIPELINE_FACTORY);
    let _runner = ObservedStateRunner::start(&runtime);
    register_existing_pipeline(&runtime, &config);
    let expected_bindings = {
        let state = runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        Arc::clone(&state.latest_context_bindings)
    };
    let _runtime_control = if exit_before_activation {
        runtime
            .reserve_instance_launch(
                &deployed_key("g1", "p1", 0, 0),
                Arc::clone(&expected_bindings),
            )
            .expect("launch should reserve its context snapshot");
        None
    } else {
        Some(register_runtime_instance(
            &runtime,
            "g1",
            "p1",
            0,
            0,
            RuntimeInstanceLifecycle::Active,
        ))
    };
    report_ready(&runtime, deployed_key("g1", "p1", 0, 0));

    let latest_bindings = runtime
        .pipeline_factory
        .compile_initial_context(&empty_engine_config().resolve())
        .expect("new context snapshot should compile")
        .bindings;
    assert!(!Arc::ptr_eq(&expected_bindings, &latest_bindings));
    runtime
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .latest_context_bindings = latest_bindings;

    runtime.note_instance_exit(
        deployed_key("g1", "p1", 0, 0),
        RuntimeInstanceExit::Error(RuntimeInstanceError::runtime("boom".to_owned())),
    );

    let recovered_bindings = wait_for_context_bindings_test_capture()
        .upgrade()
        .expect("recovered policy should remain installed");
    assert!(Arc::ptr_eq(&expected_bindings, &recovered_bindings));

    let pipeline_key = PipelineKey::new("g1".into(), "p1".into());
    let _ = wait_for_observed_status(&runtime, &pipeline_key, |status| {
        status
            .instance_status(0, 1)
            .is_some_and(|instance| matches!(instance.phase(), PipelinePhase::Running))
    });
    runtime
        .request_instance_shutdown(
            &deployed_key("g1", "p1", 0, 1),
            2,
            "context binding recovery test cleanup",
        )
        .expect("recovered runtime should accept shutdown");
}

/// Scenario: a control-plane caller deletes a pipeline that cannot be found.
/// Guarantees: missing groups and missing pipelines map to distinct typed
/// errors before any live state is changed.
#[test]
fn delete_pipeline_rejects_missing_targets() {
    let config = engine_config_with_pipeline(simple_pipeline_yaml());
    let runtime = test_runtime(&config);

    let missing_group = runtime
        .request_delete_pipeline("missing", "p1", 5)
        .expect_err("missing group should be rejected");
    assert_eq!(missing_group, ControlPlaneError::GroupNotFound);

    let missing_pipeline = runtime
        .request_delete_pipeline("g1", "missing", 5)
        .expect_err("missing pipeline should be rejected");
    assert_eq!(missing_pipeline, ControlPlaneError::PipelineNotFound);

    let snapshot = runtime.engine_config_snapshot();
    assert!(
        snapshot.groups[&PipelineGroupId::from("g1")]
            .pipelines
            .contains_key(&PipelineId::from("p1"))
    );
}

/// Scenario: a control-plane caller deletes a pipeline while a shutdown is
/// already active for the same logical pipeline.
/// Guarantees: delete rejects with the same conflict boundary as rollout and
/// shutdown planning.
#[test]
fn delete_pipeline_rejects_active_operation_conflict() {
    let config = engine_config_with_pipeline(simple_pipeline_yaml());
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);
    let _rx =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);
    let plan = runtime
        .prepare_shutdown_plan("g1", "p1", 5)
        .expect("shutdown plan should be accepted");
    runtime
        .insert_shutdown(&plan.pipeline_key, plan.shutdown)
        .expect("shutdown should register");

    let err = runtime
        .request_delete_pipeline("g1", "p1", 5)
        .expect_err("active shutdown should block delete");

    assert_eq!(err, ControlPlaneError::RolloutConflict);
}

/// Scenario: deletion drains an active pipeline before removing it.
/// Guarantees: the nested engine-owned shutdown status has no external initiator.
#[test]
fn delete_pipeline_shutdown_has_no_external_initiator() {
    let config = engine_config_with_pipeline(simple_pipeline_yaml());
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);
    let mut notifications =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);
    let delete_runtime = Arc::clone(&runtime);
    let delete = thread::spawn(move || delete_runtime.request_delete_pipeline("g1", "p1", 5));

    assert!(matches!(
        wait_for_shutdown_message(&mut notifications),
        RuntimeControlMsg::Shutdown { .. }
    ));
    runtime.note_instance_exit(
        DeployedPipelineKey {
            pipeline_group_id: "g1".into(),
            pipeline_id: "p1".into(),
            core_id: 0,
            deployment_generation: 0,
        },
        RuntimeInstanceExit::Success,
    );

    let status = delete
        .join()
        .expect("delete worker should not panic")
        .expect("pipeline should be deleted");
    assert_eq!(status.state, "succeeded");
    assert_eq!(
        status
            .shutdown
            .expect("active pipeline delete should include shutdown status")
            .initiator,
        None
    );
}

/// Scenario: an engine-scoped lifecycle operation is already active.
/// Guarantees: public config mutation entry points reject instead of
/// interleaving with the active full-engine operation.
#[test]
fn engine_scoped_operation_rejects_public_lifecycle_mutations() {
    let config = engine_config_with_pipeline(simple_pipeline_yaml());
    let runtime = test_runtime(&config);
    {
        let mut state = runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.active_engine_operation = Some("reconcile-42".to_owned());
    }

    let create_err = runtime
        .create_group("g2", PipelineGroupConfig::new())
        .expect_err("active engine operation should block group creation");
    assert_eq!(create_err, ControlPlaneError::RolloutConflict);

    let delete_err = runtime
        .request_delete_pipeline("g1", "p1", 5)
        .expect_err("active engine operation should block pipeline deletion");
    assert_eq!(delete_err, ControlPlaneError::RolloutConflict);

    let rollout_err = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: config.groups[&PipelineGroupId::from("g1")].pipelines
                    [&PipelineId::from("p1")]
                    .clone(),
                step_timeout_secs: 5,
                drain_timeout_secs: 5,
            },
        )
        .expect_err("active engine operation should block rollout planning");
    assert_eq!(rollout_err, ControlPlaneError::RolloutConflict);
}

/// Scenario: a control-plane caller deletes an empty group.
/// Guarantees: the group is removed from committed live config without
/// requiring pipeline shutdown work.
#[test]
fn delete_group_removes_empty_group_from_live_config() {
    let config = empty_engine_config();
    let runtime = test_runtime(&config);
    let _created = runtime
        .create_group("g1", PipelineGroupConfig::new())
        .expect("group should be created");

    let status = runtime
        .request_delete_group("g1", 5)
        .expect("empty group should be deleted");

    assert_eq!(status.state, "succeeded");
    assert!(status.pipelines.is_empty());
    assert!(
        !runtime
            .engine_config_snapshot()
            .groups
            .contains_key(&PipelineGroupId::from("g1"))
    );
}

/// Scenario: a control-plane caller deletes a group containing stopped
/// pipelines.
/// Guarantees: the runtime deletes each pipeline in deterministic order and
/// removes the empty group from committed live config.
#[test]
fn delete_group_removes_stopped_pipelines_and_group() {
    let config = OtelDataflowSpec::from_yaml(&format!(
        r#"
version: otel_dataflow/v1
groups:
  g1:
    pipelines:
      p1:
{p1}
      p2:
{p2}
"#,
        p1 = simple_pipeline_yaml(),
        p2 = simple_pipeline_yaml()
    ))
    .expect("config should parse");
    let runtime = test_runtime(&config);

    let status = runtime
        .request_delete_group("g1", 5)
        .expect("group should be deleted");

    assert_eq!(status.state, "succeeded");
    assert_eq!(
        status
            .pipelines
            .iter()
            .map(|pipeline| pipeline.pipeline_id.as_ref().to_owned())
            .collect::<Vec<_>>(),
        vec!["p1".to_owned(), "p2".to_owned()]
    );
    assert!(
        !runtime
            .engine_config_snapshot()
            .groups
            .contains_key(&PipelineGroupId::from("g1"))
    );
}

/// Scenario: a full-config reconciliation request matches the current live
/// pipeline configuration and runtime assignment.
/// Guarantees: reconciliation records a no-op change and leaves committed
/// config intact.
#[test]
fn reconcile_engine_config_reports_noop_for_matching_live_config() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);
    let _rx =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);

    let status = runtime
        .reconcile_engine_config(reconcile_request(config.clone(), true))
        .expect("matching config should reconcile");

    assert_eq!(status.state, EngineConfigReconcileState::Succeeded);
    assert_eq!(status.changes.len(), 1);
    assert_eq!(status.changes[0].action, ConfigChangeAction::Noop);
    assert_eq!(status.changes[0].state, "succeeded");
    assert!(
        runtime
            .engine_config_snapshot()
            .groups
            .contains_key(&PipelineGroupId::from("g1"))
    );
}

/// Scenario: successful full-config reconciliation changes the configured log level.
/// Guarantees: the shared runtime filter follows warn -> info -> warn without restart.
#[test]
fn reconcile_engine_config_applies_runtime_log_level() {
    let mut config = empty_engine_config();
    config.engine.telemetry.logs.level =
        Some(serde_json::from_value(serde_json::json!("warn")).expect("warn level should parse"));
    let (runtime, log_filter_handle, log_filter) =
        test_runtime_with_log_filter(&config, &TEST_PIPELINE_FACTORY);
    let event_count = Arc::new(AtomicUsize::new(0));
    let dispatch = tracing::Dispatch::new(
        Registry::default()
            .with(log_filter.layer())
            .with(CountingLayer(Arc::clone(&event_count))),
    );
    let emit_info = || otel_info!("test.controller.runtime_filter");

    tracing::dispatcher::with_default(&dispatch, emit_info);
    assert_eq!(event_count.swap(0, Ordering::SeqCst), 0);

    let mut desired = config.clone();
    desired.engine.telemetry.logs.level =
        Some(serde_json::from_value(serde_json::json!("info")).expect("info level should parse"));
    let status = runtime
        .reconcile_engine_config(reconcile_request(desired, true))
        .expect("info level should reconcile");

    assert_eq!(status.state, EngineConfigReconcileState::Succeeded);
    assert_eq!(log_filter_handle.effective_level().as_str(), "info");
    tracing::dispatcher::with_default(&dispatch, emit_info);
    assert_eq!(event_count.swap(0, Ordering::SeqCst), 1);

    let status = runtime
        .reconcile_engine_config(reconcile_request(config, true))
        .expect("warn level should reconcile");

    assert_eq!(status.state, EngineConfigReconcileState::Succeeded);
    assert_eq!(log_filter_handle.effective_level().as_str(), "warn");
    tracing::dispatcher::with_default(&dispatch, emit_info);
    assert_eq!(event_count.load(Ordering::SeqCst), 0);
}

/// Scenario: an existing shared filter is stricter than an explicit initial engine log level.
/// Guarantees: activation applies the explicit level before pipelines run, and an unchanged
/// reconciliation preserves that effective level.
#[test]
fn initial_config_activation_applies_log_level_before_noop_reconciliation() {
    let mut config = empty_engine_config();
    config.engine.telemetry.logs.level =
        Some(serde_json::from_value(serde_json::json!("info")).expect("info level should parse"));
    let bootstrap_level =
        serde_json::from_value(serde_json::json!("error")).expect("error level should parse");
    let (log_filter, log_filter_handle) = RuntimeLogFilter::new_configured(&bootstrap_level);
    let event_count = Arc::new(AtomicUsize::new(0));
    let dispatch = tracing::Dispatch::new(
        Registry::default()
            .with(log_filter.layer())
            .with(CountingLayer(Arc::clone(&event_count))),
    );
    tracing::dispatcher::with_default(&dispatch, || {
        otel_info!("test.controller.bootstrap_runtime_filter");
    });
    assert_eq!(event_count.swap(0, Ordering::SeqCst), 0);

    let (runtime, log_filter_handle, _log_filter) =
        test_runtime_with_supplied_log_filter_and_topology(
            &config,
            &TEST_PIPELINE_FACTORY,
            NumaTopology::unknown(),
            log_filter,
            log_filter_handle,
        );

    assert_eq!(log_filter_handle.effective_level().as_str(), "info");
    tracing::dispatcher::with_default(&dispatch, || {
        otel_info!("test.controller.activated_runtime_filter");
    });
    assert_eq!(event_count.swap(0, Ordering::SeqCst), 1);

    let status = runtime
        .reconcile_engine_config(reconcile_request(config, true))
        .expect("unchanged config should reconcile");

    assert_eq!(status.state, EngineConfigReconcileState::Succeeded);
    assert_eq!(log_filter_handle.effective_level().as_str(), "info");
    tracing::dispatcher::with_default(&dispatch, || {
        otel_info!("test.controller.reconciled_runtime_filter");
    });
    assert_eq!(event_count.load(Ordering::SeqCst), 1);
}

/// Scenario: full-config reconciliation changes a running engine-scoped
/// extension declaration.
/// Guarantees: the request is rejected before committed configuration changes.
#[test]
fn reconcile_rejects_engine_extension_declaration_mutation() {
    let config = OtelDataflowSpec::from_yaml(
        r#"
version: otel_dataflow/v1
extensions:
  root_auth:
    type: urn:test:extension:scope-shared
    config:
      audience: current
"#,
    )
    .expect("config should parse");
    let desired = OtelDataflowSpec::from_yaml(
        r#"
version: otel_dataflow/v1
extensions:
  root_auth:
    type: urn:test:extension:scope-shared
    config:
      audience: desired
"#,
    )
    .expect("desired config should parse");
    let runtime = test_runtime(&config);

    let err = runtime
        .reconcile_engine_config(reconcile_request(desired, true))
        .expect_err("engine extension mutation should require restart");

    assert!(matches!(
        err,
        ControlPlaneError::InvalidRequest { ref message }
            if message.contains("runtime engine extension mutation")
    ));
    assert_eq!(runtime.engine_config_snapshot(), config);
}

/// Scenario: full-config reconciliation changes a running group-scoped
/// extension declaration.
/// Guarantees: the request is rejected before committed configuration changes.
#[test]
fn reconcile_rejects_pipeline_group_extension_declaration_mutation() {
    let config = OtelDataflowSpec::from_yaml(
        r#"
version: otel_dataflow/v1
groups:
  g1:
    extensions:
      group_auth:
        type: urn:test:extension:scope-shared
        config:
          audience: current
"#,
    )
    .expect("config should parse");
    let desired = OtelDataflowSpec::from_yaml(
        r#"
version: otel_dataflow/v1
groups:
  g1:
    extensions:
      group_auth:
        type: urn:test:extension:scope-shared
        config:
          audience: desired
"#,
    )
    .expect("desired config should parse");
    let runtime = test_runtime(&config);

    let err = runtime
        .reconcile_engine_config(reconcile_request(desired, true))
        .expect_err("group extension mutation should require restart");

    assert!(matches!(
        err,
        ControlPlaneError::InvalidRequest { ref message }
            if message.contains("pipeline group `g1`")
    ));
    assert_eq!(runtime.engine_config_snapshot(), config);
}

/// Scenario: partial reconciliation omits immutable engine and group extension
/// declarations while retaining their declaration scopes.
/// Guarantees: `delete_missing: false` preserves engine and pipeline-group
/// extension declarations.
#[test]
fn reconcile_preserves_omitted_extension_scope_declarations() {
    let config = OtelDataflowSpec::from_yaml(
        r#"
version: otel_dataflow/v1
extensions:
  root_auth:
    type: urn:test:extension:scope-shared
  root_store:
    type: urn:test:extension:scope-shared
groups:
  g1:
    extensions:
      group_auth:
        type: urn:test:extension:scope-shared
      group_store:
        type: urn:test:extension:scope-shared
"#,
    )
    .expect("config should parse");
    let desired = OtelDataflowSpec::from_yaml(
        r#"
version: otel_dataflow/v1
extensions:
  root_auth:
    type: urn:test:extension:scope-shared
groups:
  g1:
    extensions:
      group_auth:
        type: urn:test:extension:scope-shared
"#,
    )
    .expect("desired config should parse");
    let runtime = test_runtime(&config);

    let status = runtime
        .reconcile_engine_config(reconcile_request(desired, false))
        .expect("omitted immutable declarations should be retained");

    assert_eq!(status.state, EngineConfigReconcileState::Succeeded);
    let snapshot = runtime.engine_config_snapshot();
    assert_eq!(snapshot.extensions, config.extensions);
    assert_eq!(
        snapshot.groups[&PipelineGroupId::from("g1")].extensions,
        config.groups[&PipelineGroupId::from("g1")].extensions
    );
}

/// Scenario: full-config reconciliation changes channel capacity consumed by
/// a running engine-scoped extension host.
/// Guarantees: policies that would require rebuilding the host are rejected.
#[test]
fn reconcile_rejects_extension_host_policy_mutation() {
    let config = OtelDataflowSpec::from_yaml(
        r#"
version: otel_dataflow/v1
extensions:
  root_auth:
    type: urn:test:extension:scope-shared
"#,
    )
    .expect("config should parse");
    let desired = OtelDataflowSpec::from_yaml(
        r#"
version: otel_dataflow/v1
policies:
  channel_capacity:
    control:
      node: 200
      pipeline: 201
      completion: 203
    pdata: 202
extensions:
  root_auth:
    type: urn:test:extension:scope-shared
"#,
    )
    .expect("desired config should parse");
    let runtime = test_runtime(&config);

    let err = runtime
        .reconcile_engine_config(reconcile_request(desired, true))
        .expect_err("host runtime policy mutation should require restart");

    assert!(matches!(
        err,
        ControlPlaneError::InvalidRequest { ref message }
            if message.contains("policies used by hosted engine extensions")
    ));
    assert_eq!(runtime.engine_config_snapshot(), config);
}

/// Scenario: full-config reconciliation changes telemetry settings that a
/// running engine extension host does not consume.
/// Guarantees: unrelated live telemetry behavior remains reloadable.
#[test]
fn reconcile_accepts_irrelevant_extension_host_policy_changes() {
    let config = OtelDataflowSpec::from_yaml(
        r#"
version: otel_dataflow/v1
policies:
  telemetry:
    tokio_metrics: true
    runtime_metrics: basic
extensions:
  root_auth:
    type: urn:test:extension:scope-shared
"#,
    )
    .expect("config should parse");
    let desired = OtelDataflowSpec::from_yaml(
        r#"
version: otel_dataflow/v1
policies:
  telemetry:
    tokio_metrics: false
    runtime_metrics: detailed
extensions:
  root_auth:
    type: urn:test:extension:scope-shared
"#,
    )
    .expect("desired config should parse");
    let runtime = test_runtime(&config);

    let status = runtime
        .reconcile_engine_config(reconcile_request(desired.clone(), true))
        .expect("unused hosted-extension telemetry changes should reconcile");

    assert_eq!(status.state, EngineConfigReconcileState::Succeeded);
    assert_eq!(runtime.engine_config_snapshot(), desired);
}

/// Scenario: a full-config reconciliation request omits live stopped
/// resources with `delete_missing` enabled.
/// Guarantees: reconciliation deletes the omitted pipeline and then the
/// now-empty group from committed live config.
#[test]
fn reconcile_engine_config_deletes_missing_resources_by_default() {
    let config = engine_config_with_pipeline(simple_pipeline_yaml());
    let runtime = test_runtime(&config);

    let status = runtime
        .reconcile_engine_config(reconcile_request(empty_engine_config(), true))
        .expect("missing resources should be deleted");

    assert_eq!(status.state, EngineConfigReconcileState::Succeeded);
    assert_eq!(
        status
            .changes
            .iter()
            .map(|change| (
                change.pipeline_group_id.as_ref().map(|id| id.as_ref()),
                change.pipeline_id.as_ref().map(|id| id.as_ref()),
                change.action,
                change.state.as_str(),
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                Some("g1"),
                Some("p1"),
                ConfigChangeAction::Delete,
                "succeeded"
            ),
            (Some("g1"), None, ConfigChangeAction::Delete, "succeeded"),
        ]
    );
    assert!(
        !runtime
            .engine_config_snapshot()
            .groups
            .contains_key(&PipelineGroupId::from("g1"))
    );
}

/// Scenario: reconciliation retains one pipeline and omits another pipeline
/// with context declarations while `delete_missing` is enabled.
/// Guarantees: the retained pipeline is a no-op and the omitted pipeline is deleted.
#[test]
fn reconcile_engine_config_deletes_missing_context_pipeline() {
    let config = OtelDataflowSpec::from_yaml(
        r#"
version: otel_dataflow/v1
groups:
  g1:
    policies:
      resources:
        core_allocation:
          type: core_set
          set:
            - start: 0
              end: 0
    pipelines:
      p1:
        nodes:
          receiver:
            type: "urn:test:receiver:context-bindings"
            config:
              produces: retained-marker
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
      p2:
        nodes:
          receiver:
            type: "urn:test:receiver:context-bindings"
            config:
              produces: X-Tenant
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    )
    .expect("engine config should parse");
    let mut desired = config.clone();
    _ = desired
        .groups
        .get_mut(&PipelineGroupId::from("g1"))
        .expect("test group should exist")
        .pipelines
        .remove(&PipelineId::from("p2"));
    let runtime = test_runtime_with_factory(&config, &CONTEXT_BINDINGS_TEST_PIPELINE_FACTORY);
    register_pipeline(&runtime, &config, "g1", "p1");
    register_pipeline(&runtime, &config, "g1", "p2");
    let _p1_runtime =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);

    let status = runtime
        .reconcile_engine_config(reconcile_request(desired, true))
        .expect("missing context pipeline should be deleted");

    assert_eq!(status.state, EngineConfigReconcileState::Succeeded);
    assert_eq!(
        status
            .changes
            .iter()
            .map(|change| (
                change.pipeline_id.as_ref().map(|id| id.as_ref()),
                change.action,
            ))
            .collect::<Vec<_>>(),
        vec![
            (Some("p1"), ConfigChangeAction::Noop),
            (Some("p2"), ConfigChangeAction::Delete),
        ]
    );
}

/// Scenario: a partial reconciliation requests unchanged pipeline p1 while
/// omitting deployed pipeline p2 with context declarations.
/// Guarantees: `delete_missing=false` plans p1 as a no-op and retains p2.
#[test]
fn reconcile_engine_config_preserves_missing_resources_when_requested() {
    let desired_config = engine_config_with_pipeline(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_set
              set:
                - start: 0
                  end: 0
        nodes:
          receiver:
            type: "urn:test:receiver:context-bindings"
            config:
              produces: X-Tenant
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let mut config = desired_config.clone();
    let retained_pipeline =
        config.groups[&PipelineGroupId::from("g1")].pipelines[&PipelineId::from("p1")].clone();
    _ = config
        .groups
        .get_mut(&PipelineGroupId::from("g1"))
        .expect("test group should exist")
        .pipelines
        .insert("p2".into(), retained_pipeline);
    let runtime = test_runtime_with_factory(&config, &CONTEXT_BINDINGS_TEST_PIPELINE_FACTORY);
    register_existing_pipeline(&runtime, &config);
    register_pipeline(&runtime, &config, "g1", "p2");
    let _p1_runtime =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);
    let _p2_runtime =
        register_runtime_instance(&runtime, "g1", "p2", 0, 0, RuntimeInstanceLifecycle::Active);

    let status = runtime
        .reconcile_engine_config(reconcile_request(desired_config, false))
        .expect("missing resources should be preserved");

    assert_eq!(status.state, EngineConfigReconcileState::Succeeded);
    assert_eq!(status.changes.len(), 1);
    assert_eq!(status.changes[0].action, ConfigChangeAction::Noop);
    let snapshot = runtime.engine_config_snapshot();
    for pipeline_id in ["p1", "p2"] {
        assert!(
            snapshot.groups[&PipelineGroupId::from("g1")]
                .pipelines
                .contains_key(&PipelineId::from(pipeline_id))
        );
    }
}

/// Scenario: full-config reconciliation is rejected after validation because a
/// target pipeline already has an active rollout.
/// Guarantees: desired engine-level fields and the active log filter remain
/// unchanged when the request fails before applying all requested changes.
#[test]
fn reconcile_engine_config_does_not_publish_scaffold_on_conflict() {
    let mut config = engine_config_with_pipeline(simple_pipeline_yaml());
    config.engine.telemetry.logs.level =
        Some(serde_json::from_value(serde_json::json!("warn")).expect("warn level should parse"));
    let (runtime, log_filter_handle, _log_filter) =
        test_runtime_with_log_filter(&config, &TEST_PIPELINE_FACTORY);
    let pipeline_key = PipelineKey::new("g1".into(), "p1".into());
    {
        let mut state = runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        _ = state
            .active_rollouts
            .insert(pipeline_key, "rollout-42".to_owned());
    }

    let mut desired = config.clone();
    desired.engine.telemetry.logs.level =
        Some(serde_json::from_value(serde_json::json!("info")).expect("info level should parse"));
    _ = desired
        .engine
        .custom
        .insert("desired".to_owned(), serde_json::json!({"enabled": true}));

    let err = runtime
        .reconcile_engine_config(reconcile_request(desired, true))
        .expect_err("active rollout should reject full-config reconciliation");

    assert_eq!(err, ControlPlaneError::RolloutConflict);
    assert!(runtime.engine_config_snapshot().engine.custom.is_empty());
    assert_eq!(log_filter_handle.effective_level().as_str(), "warn");
}

/// Scenario: full-config reconciliation would change an existing topic
/// runtime profile.
/// Guarantees: reconciliation rejects the request before starting rollout or
/// mutating committed live config.
#[test]
fn reconcile_engine_config_rejects_runtime_topic_mutation() {
    let config = OtelDataflowSpec::from_yaml(
        r#"
version: otel_dataflow/v1
topics:
  shared: {}
groups:
  g1:
    pipelines:
      p1:
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          to_topic:
            type: "urn:otel:exporter:topic"
            config:
              topic: shared
        connections:
          - from: receiver
            to: to_topic
"#,
    )
    .expect("config should parse");
    let desired = OtelDataflowSpec::from_yaml(
        r#"
version: otel_dataflow/v1
topics:
  shared: {}
groups:
  g1:
    pipelines:
      p1:
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          from_topic:
            type: "urn:otel:receiver:topic"
            config:
              topic: shared
              subscription:
                mode: balanced
                group: workers
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: from_topic
            to: exporter
"#,
    )
    .expect("desired config should parse");
    let runtime = test_runtime(&config);

    let err = runtime
        .reconcile_engine_config(reconcile_request(desired, true))
        .expect_err("topic runtime changes should be rejected");

    match err {
        ControlPlaneError::InvalidRequest { message } => {
            assert!(message.contains("runtime topic broker mutation"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
    assert_eq!(runtime.engine_config_snapshot(), config);
}

#[test]
fn reconcile_engine_config_rejects_core_relocation_before_mutating() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 2
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);

    let desired = OtelDataflowSpec::from_yaml(
        r#"
version: otel_dataflow/v1
groups:
  g1:
    pipelines:
      p1:
        policies:
          resources:
            core_allocation:
              type: core_set
              set:
                - start: 2
                  end: 3
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
      p2:
        policies:
          resources:
            core_allocation:
              type: core_set
              set:
                - start: 0
                  end: 1
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    )
    .expect("desired config should parse");

    let err = runtime
        .reconcile_engine_config(reconcile_request(desired, true))
        .expect_err("reconcile should reject vacate-before-claim placement");

    match err {
        ControlPlaneError::InvalidRequest { message } => {
            assert!(message.contains("conflicts with committed or in-flight"));
            assert!(message.contains("stage the conflicting delete or resize"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
    assert_eq!(runtime.engine_config_snapshot(), config);
}

#[test]
fn reconcile_engine_config_phases_inherited_core_allocation() {
    let config = OtelDataflowSpec::from_yaml(
        r#"
version: otel_dataflow/v1
groups:
  a_count:
    policies:
      resources:
        core_allocation:
          type: core_count
          count: 4
    pipelines:
      p1:
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
  b_set:
    policies:
      resources:
        core_allocation:
          type: core_set
          set:
            - start: 0
              end: 3
    pipelines:
      p1:
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    )
    .expect("config should parse");
    let runtime = test_runtime(&config);

    for resolved in config
        .resolve()
        .pipelines
        .into_iter()
        .filter(|pipeline| pipeline.role == ResolvedPipelineRole::Regular)
    {
        let assigned_cores = match resolved.pipeline_group_id.as_ref() {
            "a_count" => vec![4, 5, 6, 7],
            "b_set" => vec![0, 1, 2, 3],
            other => panic!("unexpected group: {other}"),
        };
        let placement = PipelinePlacement {
            pipeline_group_id: resolved.pipeline_group_id.clone(),
            pipeline_id: resolved.pipeline_id.clone(),
            cores: assigned_cores
                .iter()
                .copied()
                .map(|id| CorePlacement::from_core_id(CoreId { id }, &NumaTopology::unknown()))
                .collect(),
        };
        let group_id = resolved.pipeline_group_id.as_ref().to_owned();
        let pipeline_id = resolved.pipeline_id.as_ref().to_owned();
        runtime.register_committed_pipeline(resolved, placement, 0);
        for core_id in assigned_cores {
            let _rx = register_runtime_instance(
                &runtime,
                &group_id,
                &pipeline_id,
                core_id,
                0,
                RuntimeInstanceLifecycle::Active,
            );
        }
    }

    let status = runtime
        .reconcile_engine_config(reconcile_request(config, true))
        .expect("matching inherited placement config should reconcile");

    assert_eq!(status.state, EngineConfigReconcileState::Succeeded);
    assert_eq!(
        status
            .changes
            .iter()
            .map(|change| (
                change.pipeline_group_id.as_ref().map(|id| id.as_ref()),
                change.action,
                change.state.as_str(),
            ))
            .collect::<Vec<_>>(),
        vec![
            (Some("b_set"), ConfigChangeAction::Noop, "succeeded"),
            (Some("a_count"), ConfigChangeAction::Noop, "succeeded"),
        ]
    );
}

/// Scenario: a detached shutdown worker panics before it reaches the normal
/// terminal-state bookkeeping path.
/// Guarantees: the shutdown is forced into a failed terminal state and the
/// logical pipeline no longer stays blocked by a stale active-shutdown entry.
/// Scenario: full-config reconciliation changes the process-wide memory limiter policy.
/// Guarantees: live reconciliation rejects startup-owned sampler changes before mutating committed config.
#[test]
fn reconcile_engine_config_rejects_runtime_memory_limiter_mutation() {
    let config = OtelDataflowSpec::from_yaml(
        r#"
version: otel_dataflow/v1
policies:
  resources:
    memory_limiter:
      mode: enforce
      source: rss
      check_interval: 1s
      soft_limit: "64 MiB"
      hard_limit: "96 MiB"
groups:
  g1:
    pipelines:
      p1:
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    )
    .expect("config should parse");
    let desired = OtelDataflowSpec::from_yaml(
        r#"
version: otel_dataflow/v1
policies:
  resources:
    memory_limiter:
      mode: enforce
      source: rss
      check_interval: 1s
      soft_limit: "128 MiB"
      hard_limit: "192 MiB"
groups:
  g1:
    pipelines:
      p1:
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    )
    .expect("desired config should parse");
    let runtime = test_runtime(&config);

    let err = runtime
        .reconcile_engine_config(reconcile_request(desired, true))
        .expect_err("memory limiter runtime changes should be rejected");

    match err {
        ControlPlaneError::InvalidRequest { message } => {
            assert!(message.contains("runtime memory_limiter mutation"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
    assert_eq!(runtime.engine_config_snapshot(), config);
}

/// Scenario: a detached shutdown worker panics before it reaches the normal
/// terminal-state bookkeeping path.
/// Guarantees: the shutdown is forced into a failed terminal state and the
/// logical pipeline no longer stays blocked by a stale active-shutdown entry.
#[test]
fn shutdown_worker_panic_marks_failed_and_clears_conflict() {
    let config = engine_config_with_pipeline(simple_pipeline_yaml());
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);
    let _rx =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);

    let plan = runtime
        .prepare_shutdown_plan("g1", "p1", 5)
        .expect("shutdown plan should be accepted");
    runtime
        .insert_shutdown(&plan.pipeline_key, plan.shutdown.clone())
        .expect("shutdown should register");

    runtime.handle_shutdown_worker_panic(
        &plan.pipeline_key,
        &plan.shutdown.shutdown_id,
        "shutdown-g1-p1".to_owned(),
        Box::new("boom"),
    );

    let status = runtime
        .shutdown_status_snapshot(&plan.shutdown.shutdown_id)
        .expect("shutdown should remain queryable");
    assert_eq!(status.state, "failed");
    assert!(
        status
            .failure_reason
            .as_deref()
            .is_some_and(|message| message.contains("shutdown worker panicked: boom"))
    );
    assert!(
        status
            .failure_reason
            .as_deref()
            .is_some_and(|message| !message.contains("backtrace:"))
    );

    let state = runtime
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    assert!(!state.active_shutdowns.contains_key(&plan.pipeline_key));
    drop(state);

    let _next_plan = runtime
        .prepare_shutdown_plan("g1", "p1", 5)
        .expect("shutdown conflict should be cleared after panic cleanup");
}

/// Scenario: a shutdown request arrives while the same logical pipeline is
/// already under rollout.
/// Guarantees: shutdown is rejected with a rollout conflict so the rollout
/// controller remains the single owner of that pipeline's lifecycle.
#[test]
fn request_shutdown_pipeline_rejects_active_rollout() {
    let config = engine_config_with_pipeline(
        r#"
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);

    let pipeline_key = PipelineKey::new("g1".into(), "p1".into());
    let mut state = runtime
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    _ = state
        .active_rollouts
        .insert(pipeline_key, "rollout-42".to_owned());
    drop(state);

    let err = runtime
        .request_shutdown_pipeline("g1", "p1", 5)
        .expect_err("active rollout should conflict");

    assert_eq!(err, ControlPlaneError::RolloutConflict);
}

/// Scenario: a second shutdown request targets a logical pipeline that
/// already has an active shutdown operation.
/// Guarantees: the controller rejects the duplicate request instead of
/// creating competing shutdown records.
#[test]
fn request_shutdown_pipeline_rejects_active_shutdown() {
    let config = engine_config_with_pipeline(
        r#"
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);
    let pipeline_key = PipelineKey::new("g1".into(), "p1".into());
    let shutdown = ShutdownRecord::new(
        "shutdown-0".to_owned(),
        "g1".into(),
        "p1".into(),
        None,
        vec![ShutdownCoreProgress {
            core_id: 0,
            deployment_generation: 0,
            state: "pending".to_owned(),
            updated_at: timestamp_now(),
            detail: None,
        }],
    );
    runtime
        .insert_shutdown(&pipeline_key, shutdown)
        .expect("shutdown should register");

    let err = runtime
        .request_shutdown_pipeline("g1", "p1", 5)
        .expect_err("active shutdown should conflict");

    assert_eq!(err, ControlPlaneError::RolloutConflict);
}

/// Scenario: a shutdown request targets a committed pipeline that currently
/// has no active runtime instances.
/// Guarantees: the controller rejects the request as an invalid already
/// stopped pipeline instead of synthesizing a no-op shutdown operation.
#[test]
fn request_shutdown_pipeline_rejects_already_stopped_pipeline() {
    let config = engine_config_with_pipeline(
        r#"
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);

    let err = runtime
        .request_shutdown_pipeline("g1", "p1", 5)
        .expect_err("already stopped pipeline should be rejected");

    match err {
        ControlPlaneError::InvalidRequest { message } => {
            assert!(message.contains("already stopped"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

/// Scenario: dfctl explicitly requests shutdown of an active logical pipeline.
/// Guarantees: immediate, polled, and terminal shutdown status retain the dfctl initiator.
#[test]
fn explicit_shutdown_retains_initiator_in_status() {
    let config = engine_config_with_pipeline(
        r#"
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);
    let mut notifications =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);
    let control_plane = runtime.control_plane();

    let initial = control_plane
        .shutdown_pipeline("g1", "p1", 5, PipelineShutdownInitiator::Dfctl)
        .expect("shutdown request should be accepted");

    assert_eq!(initial.initiator, Some(PipelineShutdownInitiator::Dfctl));
    assert_eq!(
        control_plane
            .shutdown_status("g1", "p1", &initial.shutdown_id)
            .expect("shutdown status lookup should succeed")
            .expect("shutdown status should be retained")
            .initiator,
        Some(PipelineShutdownInitiator::Dfctl)
    );
    assert!(matches!(
        wait_for_shutdown_message(&mut notifications),
        RuntimeControlMsg::Shutdown { .. }
    ));

    runtime.note_instance_exit(
        DeployedPipelineKey {
            pipeline_group_id: "g1".into(),
            pipeline_id: "p1".into(),
            core_id: 0,
            deployment_generation: 0,
        },
        RuntimeInstanceExit::Success,
    );
    assert_eq!(
        wait_for_shutdown_state(&runtime, &initial.shutdown_id, "succeeded").initiator,
        Some(PipelineShutdownInitiator::Dfctl)
    );
}

/// Scenario: an instance can only finish after the engine reaches its graceful
/// drain deadline and force-stops unresolved node work.
/// Guarantees: the controller allows bounded post-deadline completion time and
/// does not falsely report the forced runtime exit as a drain timeout.
#[test]
fn shutdown_instance_waits_for_exit_after_graceful_drain_deadline() {
    let config = engine_config_with_pipeline(simple_pipeline_yaml());
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);
    let deployed_key = deployed_key("g1", "p1", 0, 0);
    let mut notifications =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);
    {
        // A rollout retains the terminal runtime record long enough for its
        // worker to observe the exit rather than compacting it immediately.
        let mut state = runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        _ = state.active_rollouts.insert(
            PipelineKey::new("g1".into(), "p1".into()),
            "deadline-completion-test".to_owned(),
        );
    }

    let exit_runtime = Arc::clone(&runtime);
    let exit_key = deployed_key.clone();
    let exit_thread = thread::spawn(move || {
        let RuntimeControlMsg::Shutdown { deadline, .. } =
            wait_for_shutdown_message(&mut notifications)
        else {
            panic!("instance should receive shutdown");
        };
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                break;
            }
            thread::sleep(remaining.min(Duration::from_millis(10)));
        }
        exit_runtime.note_instance_exit(exit_key, RuntimeInstanceExit::Success);
    });

    runtime
        .shutdown_instance(&deployed_key, 1, "deadline completion test")
        .expect("forced shutdown should complete during the controller grace period");
    exit_thread.join().expect("exit reporter should not panic");
}

/// Scenario: a shutdown request targets one logical pipeline while other
/// pipelines and exited instances still exist in the runtime registry.
/// Guarantees: only active instances for the requested logical pipeline
/// receive shutdown control messages and relinquish their control senders.
#[test]
fn request_shutdown_pipeline_targets_only_active_instances_for_pipeline() {
    let config = OtelDataflowSpec::from_yaml(
        r#"
version: otel_dataflow/v1
groups:
  g1:
    pipelines:
      p1:
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 2
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
      p2:
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    )
    .expect("config should parse");
    let runtime = test_runtime(&config);
    register_pipeline(&runtime, &config, "g1", "p1");
    register_pipeline(&runtime, &config, "g1", "p2");

    let mut p1_core0 =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);
    let mut p1_core1 =
        register_runtime_instance(&runtime, "g1", "p1", 1, 0, RuntimeInstanceLifecycle::Active);
    let mut p1_exited = register_runtime_instance(
        &runtime,
        "g1",
        "p1",
        2,
        0,
        RuntimeInstanceLifecycle::Exited(RuntimeInstanceExit::Success),
    );
    let mut p2_core0 =
        register_runtime_instance(&runtime, "g1", "p2", 3, 0, RuntimeInstanceLifecycle::Active);

    let _shutdown = runtime
        .request_shutdown_pipeline("g1", "p1", 5)
        .expect("shutdown request should be accepted");

    assert!(matches!(
        wait_for_shutdown_message(&mut p1_core0),
        RuntimeControlMsg::Shutdown { reason, .. } if reason == "pipeline shutdown"
    ));
    assert!(matches!(
        wait_for_shutdown_message(&mut p1_core1),
        RuntimeControlMsg::Shutdown { reason, .. } if reason == "pipeline shutdown"
    ));
    assert!(
        p1_exited.try_recv().is_err(),
        "exited runtime should not receive shutdown"
    );
    assert!(
        p2_core0.try_recv().is_err(),
        "other pipelines must not receive shutdown"
    );
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let state = runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let p1_core0_released = state
            .runtime_instances
            .get(&DeployedPipelineKey {
                pipeline_group_id: "g1".into(),
                pipeline_id: "p1".into(),
                core_id: 0,
                deployment_generation: 0,
            })
            .and_then(|instance| instance.control_sender.as_ref())
            .is_none();
        let p1_core1_released = state
            .runtime_instances
            .get(&DeployedPipelineKey {
                pipeline_group_id: "g1".into(),
                pipeline_id: "p1".into(),
                core_id: 1,
                deployment_generation: 0,
            })
            .and_then(|instance| instance.control_sender.as_ref())
            .is_none();
        let p2_core0_retained = state
            .runtime_instances
            .get(&DeployedPipelineKey {
                pipeline_group_id: "g1".into(),
                pipeline_id: "p2".into(),
                core_id: 3,
                deployment_generation: 0,
            })
            .and_then(|instance| instance.control_sender.as_ref())
            .is_some();
        drop(state);

        if p1_core0_released && p1_core1_released && p2_core0_retained {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for targeted control senders to be released"
        );
        thread::sleep(Duration::from_millis(25));
    }
}

/// Scenario: global shutdown dispatch encounters a send failure for one
/// active runtime instance while other active instances still need the signal.
/// Guarantees: shutdown dispatch is best effort across the whole snapshot:
/// every active sender is attempted, successful sends relinquish their retained
/// control sender, repeated calls do not re-signal instances that already
/// accepted shutdown, and failures are reported only after the full pass.
#[test]
fn request_shutdown_all_attempts_all_active_instances_before_returning_error() {
    let runtime = test_runtime(&engine_config_with_pipeline(simple_pipeline_yaml()));
    let key0 = deployed_key("g1", "p1", 0, 0);
    let key1 = deployed_key("g1", "p1", 1, 0);
    let key2 = deployed_key("g1", "p1", 2, 0);
    let (sender0, calls0) = recording_admin_sender(None);
    let (sender1, calls1) = recording_admin_sender(Some("simulated send failure"));
    let (sender2, calls2) = recording_admin_sender(None);

    register_runtime_instance_with_sender(
        &runtime,
        key0.clone(),
        sender0,
        RuntimeInstanceLifecycle::Active,
    );
    register_runtime_instance_with_sender(
        &runtime,
        key1.clone(),
        sender1,
        RuntimeInstanceLifecycle::Active,
    );
    register_runtime_instance_with_sender(
        &runtime,
        key2.clone(),
        sender2,
        RuntimeInstanceLifecycle::Active,
    );

    let err = runtime
        .request_shutdown_all(5)
        .expect_err("shutdown-all should report the failed sender after dispatching all sends");

    assert_eq!(
        *calls0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()),
        vec!["global shutdown".to_owned()]
    );
    assert_eq!(
        *calls1
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()),
        vec!["global shutdown".to_owned()]
    );
    assert_eq!(
        *calls2
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()),
        vec!["global shutdown".to_owned()]
    );

    let ControlPlaneError::Internal { message } = err else {
        panic!("unexpected shutdown-all error: {err:?}");
    };
    assert!(message.contains("g1:p1 core=1 generation=0"));
    assert!(message.contains("simulated send failure"));

    let state = runtime
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    assert!(
        state
            .runtime_instances
            .get(&key0)
            .and_then(|instance| instance.control_sender.as_ref())
            .is_none(),
        "successful shutdown send should release key0 control sender"
    );
    assert!(
        state
            .runtime_instances
            .get(&key1)
            .and_then(|instance| instance.control_sender.as_ref())
            .is_some(),
        "failed shutdown send should retain key1 control sender"
    );
    assert!(
        state
            .runtime_instances
            .get(&key2)
            .and_then(|instance| instance.control_sender.as_ref())
            .is_none(),
        "successful shutdown send should release key2 control sender"
    );
    drop(state);

    // The first pass released the control sender for successful instances, so
    // a retry should only reattempt the instance whose shutdown send failed.
    let err = runtime
        .request_shutdown_all(5)
        .expect_err("shutdown-all retry should still report the failed sender");

    let ControlPlaneError::Internal { message } = err else {
        panic!("unexpected shutdown-all retry error: {err:?}");
    };
    assert!(message.contains("g1:p1 core=1 generation=0"));
    assert!(message.contains("simulated send failure"));
    assert_eq!(
        *calls0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()),
        vec!["global shutdown".to_owned()]
    );
    assert_eq!(
        *calls1
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()),
        vec!["global shutdown".to_owned(), "global shutdown".to_owned()]
    );
    assert_eq!(
        *calls2
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()),
        vec!["global shutdown".to_owned()]
    );
}

/// Scenario: an active pipeline exits cleanly after global shutdown snapshots
/// its sender but before that sender reports a closed-channel failure.
/// Guarantees: the retained terminal result wins the race, so clean completion
/// is not misreported as a fatal shutdown-send failure.
#[test]
fn request_shutdown_all_recognizes_exit_racing_with_send_failure() {
    let runtime = test_runtime(&empty_engine_config());
    let deployed_key = deployed_key("g1", "p1", 0, 0);
    let sender: Arc<dyn PipelineAdminSender> = Arc::new(ExitThenFailPipelineAdminSender {
        runtime: Arc::downgrade(&runtime),
        deployed_key: deployed_key.clone(),
    });
    register_runtime_instance_with_sender(
        &runtime,
        deployed_key.clone(),
        sender,
        RuntimeInstanceLifecycle::Active,
    );

    runtime
        .request_shutdown_all(1)
        .expect("a clean racing exit should satisfy global shutdown");
    assert!(runtime.wait_for_global_shutdown_completion_for(Duration::from_secs(1)));
    assert!(
        matches!(
            runtime.instance_exit(&deployed_key),
            Some(RuntimeInstanceExit::Success)
        ),
        "the clean terminal result should remain available during global shutdown"
    );
    assert!(!runtime.has_fatal_runtime_error());
}

/// Scenario: callers repeat global shutdown with different timeout requests
/// while teardown is already active.
/// Guarantees: producer instances retain the first accepted absolute deadline;
/// later calls cannot reset, shorten, or extend that phase's budget.
#[test]
fn request_shutdown_all_preserves_first_absolute_deadline() {
    let runtime = test_runtime(&empty_engine_config());
    runtime
        .request_shutdown_all(30)
        .expect("initial global shutdown should succeed");
    let first_deadline = runtime
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .global_shutdown_deadline
        .expect("initial shutdown should establish a deadline");

    runtime
        .request_shutdown_all(1)
        .expect("shorter repeated global shutdown should succeed");
    let retained_deadline = runtime
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .global_shutdown_deadline
        .expect("repeated shutdown should retain a deadline");
    assert_eq!(retained_deadline, first_deadline);

    runtime
        .request_shutdown_all(60)
        .expect("longer repeated global shutdown should succeed");
    assert_eq!(
        runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .global_shutdown_deadline,
        Some(first_deadline)
    );
}

/// Scenario: global shutdown begins while a runtime recovery worker still owns
/// its fencing token and may take a long time to unwind.
/// Guarantees: launch admission closes and recovery cancellation is requested
/// without blocking shutdown dispatch on the worker's release.
#[test]
fn request_shutdown_all_does_not_wait_for_recovery_worker_release() {
    let runtime = test_runtime(&empty_engine_config());
    let pipeline_key = PipelineKey::new("g1".into(), "p1".into());
    {
        let mut state = runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let context_bindings = Arc::clone(&state.latest_context_bindings);
        _ = state.runtime_recoveries.insert(
            (pipeline_key, 0),
            RuntimeRecoveryState {
                serving_generation: 0,
                context_bindings,
                restart_count: 0,
                ready_since: None,
                worker_id: Some(42),
                candidate_generation: None,
                cancel_requested: false,
            },
        );
    }

    let started = Instant::now();
    runtime
        .request_shutdown_all(1)
        .expect("shutdown dispatch should not wait for recovery release");
    assert!(
        started.elapsed() < Duration::from_millis(250),
        "global shutdown must not block on recovery cleanup"
    );

    let state = runtime
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let recovery = state
        .runtime_recoveries
        .values()
        .next()
        .expect("synthetic recovery should remain present");
    assert!(recovery.cancel_requested);
    assert!(state.launches_closed);
    drop(state);
    assert!(!runtime.wait_for_runtime_recoveries_for(Duration::from_millis(20)));
}

/// Scenario: global shutdown includes regular producer instances and the
/// engine's system observability instance.
/// Guarantees: all regular instances receive shutdown first, and the system
/// observability sender is not called until both regular instances and the
/// extension scope hosts have stopped, with a fresh, fixed observability deadline.
#[test]
fn request_shutdown_all_defers_observability_until_extension_scope_hosts_stop() {
    let runtime = test_runtime(&engine_config_with_pipeline(simple_pipeline_yaml()));
    let regular_key0 = deployed_key("g1", "p1", 0, 0);
    let regular_key1 = deployed_key("g1", "p1", 1, 0);
    let observability_key = deployed_key(
        SYSTEM_PIPELINE_GROUP_ID,
        SYSTEM_OBSERVABILITY_PIPELINE_ID,
        2,
        0,
    );
    let (regular_sender0, regular_notifications0) = notifying_admin_sender();
    let (regular_sender1, regular_notifications1) = notifying_admin_sender();
    let (observability_sender, observability_notifications) = deadline_notifying_admin_sender();
    register_runtime_instance_with_sender(
        &runtime,
        regular_key0.clone(),
        regular_sender0,
        RuntimeInstanceLifecycle::Active,
    );
    register_runtime_instance_with_sender(
        &runtime,
        regular_key1.clone(),
        regular_sender1,
        RuntimeInstanceLifecycle::Active,
    );
    register_runtime_instance_with_sender(
        &runtime,
        observability_key.clone(),
        observability_sender,
        RuntimeInstanceLifecycle::Active,
    );

    let shutdown_runtime = Arc::clone(&runtime);
    let (shutdown_result_tx, shutdown_result_rx) = std::sync::mpsc::channel();
    let shutdown_thread = thread::spawn(move || {
        shutdown_result_tx
            .send(shutdown_runtime.request_shutdown_all(5))
            .expect("shutdown result receiver should remain open");
    });

    assert_eq!(
        regular_notifications0
            .recv_timeout(Duration::from_secs(1))
            .expect("first regular instance should receive shutdown"),
        "global shutdown"
    );
    assert_eq!(
        regular_notifications1
            .recv_timeout(Duration::from_secs(1))
            .expect("second regular instance should receive shutdown"),
        "global shutdown"
    );
    shutdown_result_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("shutdown dispatch should return before regular instances exit")
        .expect("initial shutdown dispatch should succeed");
    shutdown_thread
        .join()
        .expect("global shutdown dispatch thread should join");
    let original_deadline = runtime
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .global_shutdown_deadline
        .expect("global shutdown should establish one deadline");
    runtime
        .request_shutdown_all(30)
        .expect("a longer repeated request should remain idempotent");
    assert_eq!(
        runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .global_shutdown_deadline,
        Some(original_deadline),
        "a repeated request must not extend the original deadline"
    );
    assert!(
        observability_notifications.try_recv().is_err(),
        "observability must remain active while regular instances drain"
    );

    runtime.note_instance_exit(regular_key0, RuntimeInstanceExit::Success);
    assert!(
        observability_notifications.try_recv().is_err(),
        "one remaining regular instance must keep observability active"
    );
    runtime.note_instance_exit(regular_key1, RuntimeInstanceExit::Success);
    assert!(runtime.wait_for_global_shutdown_completion());
    assert!(
        observability_notifications.try_recv().is_err(),
        "extension scope hosts must stop before observability is stopped"
    );

    let observability_started = Instant::now();
    runtime.mark_extension_scope_hosts_stopped();
    runtime
        .request_shutdown_all(5)
        .expect("observability shutdown dispatch should succeed");

    let (reason, deadline) = observability_notifications
        .recv_timeout(Duration::from_secs(1))
        .expect("observability should receive shutdown after producers exit");
    assert_eq!(reason, "global shutdown");
    assert!(
        deadline >= observability_started + ControllerRuntime::<()>::OBSERVABILITY_SHUTDOWN_TIMEOUT
    );
    assert!(deadline > original_deadline);
    runtime
        .request_shutdown_all(60)
        .expect("a repeated request must retain the active observability deadline");
    assert_eq!(
        runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .observability_shutdown_deadline,
        Some(deadline)
    );
    runtime.note_instance_exit(observability_key, RuntimeInstanceExit::Success);
    assert!(
        runtime.wait_for_global_shutdown_completion(),
        "the phased shutdown coordinator should complete after observability exits"
    );
    assert!(runtime.all_instances_exited());

    runtime
        .request_shutdown_all(5)
        .expect("repeated global shutdown should remain idempotent");
    assert!(regular_notifications0.try_recv().is_err());
    assert!(regular_notifications1.try_recv().is_err());
    assert!(observability_notifications.try_recv().is_err());
}

/// Scenario: a producer misses its shutdown deadline while observability is still running.
/// Guarantees: observability remains available for bounded terminal reporting before it stops.
#[test]
fn request_shutdown_all_keeps_observability_active_when_producer_times_out() {
    let runtime = test_runtime(&engine_config_with_pipeline(simple_pipeline_yaml()));
    let regular_key = deployed_key("g1", "p1", 0, 0);
    let observability_key = deployed_key(
        SYSTEM_PIPELINE_GROUP_ID,
        SYSTEM_OBSERVABILITY_PIPELINE_ID,
        1,
        0,
    );
    let (regular_sender, regular_notifications) = notifying_admin_sender();
    let (observability_sender, observability_notifications) = notifying_admin_sender();
    register_runtime_instance_with_sender(
        &runtime,
        regular_key.clone(),
        regular_sender,
        RuntimeInstanceLifecycle::Active,
    );
    register_runtime_instance_with_sender(
        &runtime,
        observability_key.clone(),
        observability_sender,
        RuntimeInstanceLifecycle::Active,
    );

    runtime
        .request_shutdown_all(1)
        .expect("initial shutdown dispatch should succeed");
    let _ = regular_notifications
        .recv_timeout(Duration::from_secs(1))
        .expect("regular producer should receive shutdown");
    assert!(
        observability_notifications
            .recv_timeout(Duration::from_millis(1_200))
            .is_err(),
        "observability must remain active when a producer misses its deadline"
    );
    assert!(runtime.wait_for_global_shutdown_completion());
    assert!(
        runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .instance_wait_released,
        "a timed-out global shutdown must release the main lifecycle wait"
    );

    runtime.note_instance_exit(regular_key, RuntimeInstanceExit::Success);
    runtime.mark_extension_scope_hosts_stopped();
    runtime
        .request_shutdown_all(1)
        .expect("a later request should retry the restored observability sender");
    let _ = observability_notifications
        .recv_timeout(Duration::from_secs(1))
        .expect("observability should stop once the producer has exited");
    runtime.note_instance_exit(observability_key, RuntimeInstanceExit::Success);
    assert!(runtime.wait_for_global_shutdown_completion());
    assert!(runtime.all_instances_exited());
}

/// Scenario: observability finishes launching before or after scope hosts stop, after producer grace expires.
/// Guarantees: late activation waits for providers and uses the fixed, fresh observability deadline.
#[test]
fn global_shutdown_late_observability_launch_gets_its_own_deadline() {
    for activate_before_hosts_stop in [true, false] {
        let runtime = test_runtime(&empty_engine_config());
        let key = deployed_key(
            SYSTEM_PIPELINE_GROUP_ID,
            SYSTEM_OBSERVABILITY_PIPELINE_ID,
            0,
            0,
        );
        runtime
            .reserve_instance_launch(&key, current_test_context_bindings(&runtime))
            .expect("observability launch should reserve");
        let producer_deadline = Instant::now() - Duration::from_secs(1);
        runtime
            .request_shutdown_all_until(producer_deadline)
            .expect("producer shutdown should start");
        let (sender, notifications) = deadline_notifying_admin_sender();
        let phase_started = Instant::now();
        if activate_before_hosts_stop {
            runtime.complete_instance_launch(key.clone(), sender);
            assert!(notifications.try_recv().is_err());
            assert!(
                runtime
                    .state
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .observability_shutdown_deadline
                    .is_none(),
                "launching observability must not start its drain clock before providers stop"
            );
            runtime.mark_extension_scope_hosts_stopped();
        } else {
            runtime.mark_extension_scope_hosts_stopped();
            runtime.complete_instance_launch(key.clone(), sender);
        }
        runtime
            .request_shutdown_all(60)
            .expect("observability shutdown should dispatch");
        let (_, deadline) = notifications
            .recv_timeout(Duration::from_secs(1))
            .expect("late observability instance should receive shutdown");
        assert!(
            deadline >= phase_started + ControllerRuntime::<()>::OBSERVABILITY_SHUTDOWN_TIMEOUT
        );
        assert_eq!(
            runtime.observability_shutdown_deadline_or_insert(),
            deadline,
            "repeated shutdown must not replenish observability grace"
        );
        assert_eq!(
            runtime.global_shutdown_deadline_or_insert(Duration::from_secs(60)),
            producer_deadline
        );
        runtime.note_instance_exit(key, RuntimeInstanceExit::Success);
        assert!(runtime.wait_for_global_shutdown_completion_for(Duration::from_secs(1)));
    }
}

/// Scenario: an observability launch completes after providers stop but a reserved producer is still live.
/// Guarantees: activation does not shut down observability until that final producer actually exits.
#[test]
fn global_shutdown_late_observability_launch_waits_for_remaining_producer() {
    let runtime = test_runtime(&empty_engine_config());
    let producer_key = deployed_key("g1", "p1", 0, 0);
    let observability_key = deployed_key(
        SYSTEM_PIPELINE_GROUP_ID,
        SYSTEM_OBSERVABILITY_PIPELINE_ID,
        1,
        0,
    );
    runtime
        .reserve_instance_launch(&producer_key, current_test_context_bindings(&runtime))
        .expect("producer launch should reserve");
    runtime
        .reserve_instance_launch(&observability_key, current_test_context_bindings(&runtime))
        .expect("observability launch should reserve");
    runtime
        .request_shutdown_all(5)
        .expect("producer shutdown should start");
    runtime.mark_extension_scope_hosts_stopped();
    let (sender, notifications) = deadline_notifying_admin_sender();
    runtime.complete_instance_launch(observability_key.clone(), sender);
    assert!(
        notifications.try_recv().is_err(),
        "a late observability launch must still wait for live producers"
    );
    runtime.note_instance_exit(producer_key, RuntimeInstanceExit::Success);
    assert!(runtime.wait_for_global_shutdown_completion_for(Duration::from_secs(1)));
    runtime
        .request_shutdown_all(5)
        .expect("observability shutdown should dispatch after producer exit");
    _ = notifications
        .recv_timeout(Duration::from_secs(1))
        .expect("observability should receive its final shutdown");
    runtime.note_instance_exit(observability_key, RuntimeInstanceExit::Success);
    assert!(runtime.wait_for_global_shutdown_completion_for(Duration::from_secs(1)));
}

/// Scenario: all targeted runtime instances exit cleanly after a pipeline
/// shutdown request is accepted.
/// Guarantees: the shutdown record reaches `succeeded`, tracks per-core
/// completion, and removes the active shutdown lock for that pipeline.
#[test]
fn request_shutdown_pipeline_tracks_completion() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 2
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);

    let mut core0 =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);
    let mut core1 =
        register_runtime_instance(&runtime, "g1", "p1", 1, 0, RuntimeInstanceLifecycle::Active);

    let shutdown = runtime
        .request_shutdown_pipeline("g1", "p1", 5)
        .expect("shutdown request should be accepted");
    assert_eq!(shutdown.state, "pending");

    assert!(matches!(
        wait_for_shutdown_message(&mut core0),
        RuntimeControlMsg::Shutdown { reason, .. } if reason == "pipeline shutdown"
    ));
    assert!(matches!(
        wait_for_shutdown_message(&mut core1),
        RuntimeControlMsg::Shutdown { reason, .. } if reason == "pipeline shutdown"
    ));

    runtime.note_instance_exit(
        DeployedPipelineKey {
            pipeline_group_id: "g1".into(),
            pipeline_id: "p1".into(),
            core_id: 0,
            deployment_generation: 0,
        },
        RuntimeInstanceExit::Success,
    );
    {
        let state = runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        assert!(
            state.runtime_instances.contains_key(&DeployedPipelineKey {
                pipeline_group_id: "g1".into(),
                pipeline_id: "p1".into(),
                core_id: 0,
                deployment_generation: 0,
            }),
            "active shutdown should retain exited instances until completion"
        );
    }
    runtime.note_instance_exit(
        DeployedPipelineKey {
            pipeline_group_id: "g1".into(),
            pipeline_id: "p1".into(),
            core_id: 1,
            deployment_generation: 0,
        },
        RuntimeInstanceExit::Success,
    );

    let status = wait_for_shutdown_state(&runtime, &shutdown.shutdown_id, "succeeded");
    assert_eq!(status.cores.len(), 2);
    assert!(status.cores.iter().all(|core| core.state == "exited"));

    let state = runtime
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    assert!(
        !state
            .active_shutdowns
            .contains_key(&PipelineKey::new("g1".into(), "p1".into()))
    );
    assert!(!state.runtime_instances.contains_key(&DeployedPipelineKey {
        pipeline_group_id: "g1".into(),
        pipeline_id: "p1".into(),
        core_id: 0,
        deployment_generation: 0,
    }));
    assert!(!state.runtime_instances.contains_key(&DeployedPipelineKey {
        pipeline_group_id: "g1".into(),
        pipeline_id: "p1".into(),
        core_id: 1,
        deployment_generation: 0,
    }));
}

/// Scenario: a pipeline shutdown request is accepted but the targeted
/// runtime instance never exits before the shutdown deadline.
/// Guarantees: the shutdown record transitions to `failed`, preserves the
/// timeout reason, and records the failed per-core state for callers.
#[test]
fn request_shutdown_pipeline_tracks_timeout_failure() {
    let config = engine_config_with_pipeline(
        r#"
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);

    let mut core0 =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);

    let shutdown = runtime
        .request_shutdown_pipeline("g1", "p1", 1)
        .expect("shutdown request should be accepted");
    assert!(matches!(
        wait_for_shutdown_message(&mut core0),
        RuntimeControlMsg::Shutdown { reason, .. } if reason == "pipeline shutdown"
    ));

    let status = wait_for_shutdown_state(&runtime, &shutdown.shutdown_id, "failed");
    assert!(
        status
            .failure_reason
            .as_deref()
            .is_some_and(|reason| reason.contains("timed out waiting"))
    );
    assert_eq!(status.cores.len(), 1);
    assert_eq!(status.cores[0].state, "failed");
}

/// Scenario: terminal rollout history grows beyond the retention cap for one
/// logical pipeline while another pipeline also retains rollout history.
/// Guarantees: eviction is oldest-first and scoped per logical pipeline rather
/// than dropping unrelated rollout history.
#[test]
fn terminal_rollout_history_is_bounded_per_pipeline() {
    let runtime = test_runtime(&engine_config_with_pipeline(simple_pipeline_yaml()));
    let pipeline_key = PipelineKey::new("g1".into(), "p1".into());
    let other_pipeline_key = PipelineKey::new("g1".into(), "p2".into());

    let mut state = runtime
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    for index in 0..=TERMINAL_ROLLOUT_RETENTION_LIMIT {
        let rollout_id = format!("rollout-{index}");
        _ = state.rollouts.insert(
            rollout_id.clone(),
            terminal_rollout_record("g1", "p1", &rollout_id),
        );
        ControllerRuntime::<()>::record_terminal_rollout_locked(
            &mut state,
            &pipeline_key,
            &rollout_id,
            Instant::now(),
        );
    }

    let other_rollout_id = "rollout-other".to_owned();
    _ = state.rollouts.insert(
        other_rollout_id.clone(),
        terminal_rollout_record("g1", "p2", &other_rollout_id),
    );
    ControllerRuntime::<()>::record_terminal_rollout_locked(
        &mut state,
        &other_pipeline_key,
        &other_rollout_id,
        Instant::now(),
    );

    assert!(!state.rollouts.contains_key("rollout-0"));
    assert!(state.rollouts.contains_key("rollout-1"));
    assert!(state.rollouts.contains_key(&other_rollout_id));
    assert_eq!(
        state
            .terminal_rollouts
            .get(&pipeline_key)
            .map(|queue| queue.len()),
        Some(TERMINAL_ROLLOUT_RETENTION_LIMIT)
    );
    assert_eq!(
        state
            .terminal_rollouts
            .get(&other_pipeline_key)
            .map(|queue| queue.len()),
        Some(1)
    );
}

/// Scenario: terminal shutdown history grows beyond the retention cap for one
/// logical pipeline while another pipeline also retains shutdown history.
/// Guarantees: shutdown eviction is oldest-first and scoped per logical
/// pipeline rather than trimming unrelated shutdown history.
#[test]
fn terminal_shutdown_history_is_bounded_per_pipeline() {
    let runtime = test_runtime(&engine_config_with_pipeline(simple_pipeline_yaml()));
    let pipeline_key = PipelineKey::new("g1".into(), "p1".into());
    let other_pipeline_key = PipelineKey::new("g1".into(), "p2".into());

    let mut state = runtime
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    for index in 0..=TERMINAL_SHUTDOWN_RETENTION_LIMIT {
        let shutdown_id = format!("shutdown-{index}");
        _ = state.shutdowns.insert(
            shutdown_id.clone(),
            terminal_shutdown_record("g1", "p1", &shutdown_id),
        );
        ControllerRuntime::<()>::record_terminal_shutdown_locked(
            &mut state,
            &pipeline_key,
            &shutdown_id,
            Instant::now(),
        );
    }

    let other_shutdown_id = "shutdown-other".to_owned();
    _ = state.shutdowns.insert(
        other_shutdown_id.clone(),
        terminal_shutdown_record("g1", "p2", &other_shutdown_id),
    );
    ControllerRuntime::<()>::record_terminal_shutdown_locked(
        &mut state,
        &other_pipeline_key,
        &other_shutdown_id,
        Instant::now(),
    );

    assert!(!state.shutdowns.contains_key("shutdown-0"));
    assert!(state.shutdowns.contains_key("shutdown-1"));
    assert!(state.shutdowns.contains_key(&other_shutdown_id));
    assert_eq!(
        state
            .terminal_shutdowns
            .get(&pipeline_key)
            .map(|queue| queue.len()),
        Some(TERMINAL_SHUTDOWN_RETENTION_LIMIT)
    );
    assert_eq!(
        state
            .terminal_shutdowns
            .get(&other_pipeline_key)
            .map(|queue| queue.len()),
        Some(1)
    );
}

/// Scenario: terminal rollout and shutdown ids outlive their retention TTL in
/// the controller's in-memory history.
/// Guarantees: history pruning expires those terminal records and subsequent
/// by-id lookups return not found instead of growing unboundedly.
#[test]
fn terminal_operation_history_expires_after_ttl() {
    let runtime = test_runtime(&engine_config_with_pipeline(simple_pipeline_yaml()));
    let pipeline_key = PipelineKey::new("g1".into(), "p1".into());
    let rollout_id = "rollout-old".to_owned();
    let shutdown_id = "shutdown-old".to_owned();
    let prune_now = Instant::now()
        .checked_add(TERMINAL_OPERATION_RETENTION_TTL + Duration::from_secs(2))
        .expect("synthetic prune deadline should be representable");
    let expired_at = prune_now
        .checked_sub(TERMINAL_OPERATION_RETENTION_TTL + Duration::from_secs(1))
        .expect("synthetic completed_at should be representable");

    {
        let mut state = runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        let mut rollout = terminal_rollout_record("g1", "p1", &rollout_id);
        rollout.completed_at = Some(expired_at);
        _ = state.rollouts.insert(rollout_id.clone(), rollout);
        state
            .terminal_rollouts
            .entry(pipeline_key.clone())
            .or_default()
            .push_back(rollout_id.clone());

        let mut shutdown = terminal_shutdown_record("g1", "p1", &shutdown_id);
        shutdown.completed_at = Some(expired_at);
        _ = state.shutdowns.insert(shutdown_id.clone(), shutdown);
        state
            .terminal_shutdowns
            .entry(pipeline_key.clone())
            .or_default()
            .push_back(shutdown_id.clone());

        // Use a synthetic future `now` here instead of relying on
        // `Instant::now() - ttl`, which can underflow on Windows near the
        // monotonic clock origin.
        ControllerRuntime::<()>::prune_terminal_operation_history_locked(&mut state, prune_now);
    }

    assert!(runtime.rollout_status_snapshot(&rollout_id).is_none());
    assert!(runtime.shutdown_status_snapshot(&shutdown_id).is_none());

    let state = runtime
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    assert!(!state.rollouts.contains_key(&rollout_id));
    assert!(!state.shutdowns.contains_key(&shutdown_id));
    assert!(!state.terminal_rollouts.contains_key(&pipeline_key));
    assert!(!state.terminal_shutdowns.contains_key(&pipeline_key));
}

/// Scenario: an instance exits when there is no active rollout or shutdown for
/// its logical pipeline.
/// Guarantees: the controller does not retain that exited runtime instance as
/// history once no active control-plane operation depends on it.
#[test]
fn exited_runtime_instances_without_active_operation_are_pruned_immediately() {
    let config = engine_config_with_pipeline(simple_pipeline_yaml());
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);
    let _rx =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);

    let deployed_key = DeployedPipelineKey {
        pipeline_group_id: "g1".into(),
        pipeline_id: "p1".into(),
        core_id: 0,
        deployment_generation: 0,
    };
    runtime.note_instance_exit(deployed_key.clone(), RuntimeInstanceExit::Success);

    let state = runtime
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    assert!(!state.runtime_instances.contains_key(&deployed_key));
}

/// Scenario: a pipeline launch is reserved immediately before its OS thread is
/// created and has not yet published a control sender.
/// Guarantees: liveness includes the launching thread and spawn failure can
/// roll the reservation back exactly once.
#[test]
fn launch_reservation_counts_liveness_before_activation() {
    let runtime = test_runtime(&empty_engine_config());
    let deployed_key = deployed_key("g1", "p1", 0, 7);

    runtime
        .reserve_instance_launch(&deployed_key, current_test_context_bindings(&runtime))
        .expect("launch reservation should succeed");
    {
        let state = runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        assert_eq!(state.active_instances, 1);
        assert!(state.launching_instances.contains_key(&deployed_key));
        assert!(!state.runtime_instances.contains_key(&deployed_key));
    }

    runtime.abort_instance_launch(&deployed_key);
    runtime.abort_instance_launch(&deployed_key);
    let state = runtime
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    assert_eq!(state.active_instances, 0);
    assert!(!state.launching_instances.contains_key(&deployed_key));
}

/// Scenario: the controller's latest context snapshot changes between launch reservation and activation.
/// Guarantees: the activated runtime retains the reserved generation's snapshot rather than the latest one.
#[test]
fn launch_activation_retains_reserved_context_bindings() {
    let config = empty_engine_config();
    let runtime = test_runtime(&config);
    let key = deployed_key("g1", "p1", 0, 7);
    let reserved_bindings = current_test_context_bindings(&runtime);
    runtime
        .reserve_instance_launch(&key, Arc::clone(&reserved_bindings))
        .expect("launch reservation should succeed");
    let latest_bindings = runtime
        .pipeline_factory
        .compile_initial_context(&config.resolve())
        .expect("new context snapshot should compile")
        .bindings;
    assert!(!Arc::ptr_eq(&reserved_bindings, &latest_bindings));
    runtime
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .latest_context_bindings = latest_bindings;

    let (sender, _calls) = recording_admin_sender(None);
    runtime.complete_instance_launch(key.clone(), sender);
    {
        let state = runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let instance = state
            .runtime_instances
            .get(&key)
            .expect("activated instance");
        assert!(Arc::ptr_eq(&reserved_bindings, &instance.context_bindings));
        assert!(!state.launching_instances.contains_key(&key));
    }
    runtime.note_instance_exit(key, RuntimeInstanceExit::Success);
    assert!(runtime.all_instances_exited());
}

/// Scenario: shutdown-when-done observes a producer whose launch is reserved
/// but whose runtime record is not active yet.
/// Guarantees: producer drain waits for the launching thread instead of
/// advancing to observability shutdown early.
#[test]
fn producer_wait_includes_launch_reservations() {
    let runtime = test_runtime(&empty_engine_config());
    let deployed_key = deployed_key("g1", "p1", 0, 7);
    runtime
        .reserve_instance_launch(&deployed_key, current_test_context_bindings(&runtime))
        .expect("launch reservation should succeed");

    let waiter = {
        let runtime = Arc::clone(&runtime);
        thread::spawn(move || runtime.wait_until_all_producer_instances_exit())
    };
    thread::sleep(Duration::from_millis(100));
    assert!(
        !waiter.is_finished(),
        "producer wait must include in-flight launches"
    );

    runtime.note_instance_exit(deployed_key, RuntimeInstanceExit::Success);
    waiter.join().expect("producer wait should finish");
}

/// Scenario: a fatal extension scope failure releases shutdown-when-done while a
/// producer launch remains reserved.
/// Guarantees: the producer wait reaches bounded extension scope teardown instead of
/// blocking forever on the stalled launch.
#[test]
fn producer_wait_honors_fatal_release_latch() {
    let runtime = test_runtime(&empty_engine_config());
    let deployed_key = deployed_key("g1", "p1", 0, 7);
    runtime
        .reserve_instance_launch(&deployed_key, current_test_context_bindings(&runtime))
        .expect("launch reservation should succeed");

    let waiter = {
        let runtime = Arc::clone(&runtime);
        thread::spawn(move || runtime.wait_until_all_producer_instances_exit())
    };
    thread::sleep(Duration::from_millis(100));
    assert!(!waiter.is_finished());

    runtime.release_instance_wait();
    waiter
        .join()
        .expect("fatal release should unblock producer wait");
    runtime.abort_instance_launch(&deployed_key);
}

/// Scenario: an extension scope failure closes launch admission during bootstrap.
/// Guarantees: the rejected launch reports the recorded extension scope failure
/// rather than replacing it with only a generic admission error.
#[test]
fn closed_launch_admission_preserves_fatal_runtime_error() {
    let runtime = test_runtime(&empty_engine_config());
    runtime.record_fatal_runtime_error("engine extension `auth` failed".to_owned());
    runtime.close_pipeline_launches();

    let error = runtime
        .reserve_instance_launch(
            &deployed_key("g1", "p1", 0, 8),
            current_test_context_bindings(&runtime),
        )
        .expect_err("launch admission should be closed");
    assert!(error.to_string().contains("engine extension `auth` failed"));
}

/// Scenario: a pipeline thread exits after launch reservation but before the
/// spawning thread can activate its runtime record.
/// Guarantees: the exit consumes the reservation, decrements liveness once,
/// and a late activation cannot resurrect the exited instance.
#[test]
fn exit_before_activation_consumes_launch_reservation_once() {
    let runtime = test_runtime(&empty_engine_config());
    let deployed_key = deployed_key("g1", "p1", 0, 8);
    let (sender, _calls) = recording_admin_sender(None);

    runtime
        .reserve_instance_launch(&deployed_key, current_test_context_bindings(&runtime))
        .expect("launch reservation should succeed");
    runtime.note_instance_exit(deployed_key.clone(), RuntimeInstanceExit::Success);
    assert!(
        runtime
            .activate_instance_launch(deployed_key.clone(), sender)
            .is_none()
    );

    let state = runtime
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    assert_eq!(state.active_instances, 0);
    assert!(!state.launching_instances.contains_key(&deployed_key));
    assert!(!state.runtime_instances.contains_key(&deployed_key));
}

/// Scenario: global shutdown begins after a launch reservation but before its
/// pipeline thread publishes a control sender.
/// Guarantees: launch admission closes atomically, the in-flight activation
/// inherits shutdown, and later launches are rejected.
#[test]
fn global_shutdown_catches_inflight_launch_activation() {
    let runtime = test_runtime(&empty_engine_config());
    let in_flight_key = deployed_key("g1", "p1", 0, 9);
    let observability_key = deployed_key(
        SYSTEM_PIPELINE_GROUP_ID,
        SYSTEM_OBSERVABILITY_PIPELINE_ID,
        0,
        0,
    );
    let (observability_sender, observability_shutdown) = deadline_notifying_admin_sender();
    register_runtime_instance_with_sender(
        &runtime,
        observability_key.clone(),
        observability_sender,
        RuntimeInstanceLifecycle::Active,
    );
    runtime
        .reserve_instance_launch(&in_flight_key, current_test_context_bindings(&runtime))
        .expect("in-flight launch should reserve");

    runtime
        .request_shutdown_all(2)
        .expect("shutdown dispatch should succeed");
    assert!(
        observability_shutdown
            .recv_timeout(Duration::from_millis(100))
            .is_err(),
        "observability must wait for the in-flight producer"
    );

    let (sender, calls) = recording_admin_sender(None);
    runtime.complete_instance_launch(in_flight_key.clone(), sender);
    assert_eq!(
        calls
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_slice(),
        ["global shutdown"]
    );
    assert!(
        runtime
            .reserve_instance_launch(
                &deployed_key("g1", "p1", 1, 9),
                current_test_context_bindings(&runtime),
            )
            .is_err()
    );

    runtime.note_instance_exit(in_flight_key, RuntimeInstanceExit::Success);
    assert!(runtime.wait_for_global_shutdown_completion());
    runtime.mark_extension_scope_hosts_stopped();
    runtime
        .request_shutdown_all(2)
        .expect("observability shutdown dispatch should succeed");
    let (reason, _) = observability_shutdown
        .recv_timeout(Duration::from_secs(1))
        .expect("observability should stop after the producer exits");
    assert_eq!(reason, "global shutdown");
    runtime.note_instance_exit(observability_key, RuntimeInstanceExit::Success);
    assert!(runtime.wait_for_global_shutdown_completion());
    assert!(runtime.all_instances_exited());
}

/// Scenario: a shutdown-racing launch exits cleanly while its immediate
/// shutdown send observes the already-closed control channel.
/// Guarantees: activation uses the retained terminal result and does not turn
/// the clean exit into a fatal extension-scope shutdown error.
#[test]
fn global_shutdown_activation_recognizes_clean_exit_before_send_failure() {
    let runtime = test_runtime(&empty_engine_config());
    let in_flight_key = deployed_key("g1", "p1", 0, 10);
    runtime
        .reserve_instance_launch(&in_flight_key, current_test_context_bindings(&runtime))
        .expect("in-flight launch should reserve");
    runtime
        .request_shutdown_all(1)
        .expect("shutdown dispatch should succeed");

    let sender: Arc<dyn PipelineAdminSender> = Arc::new(ExitThenFailPipelineAdminSender {
        runtime: Arc::downgrade(&runtime),
        deployed_key: in_flight_key.clone(),
    });
    runtime.complete_instance_launch(in_flight_key.clone(), sender);

    assert!(runtime.wait_for_global_shutdown_completion_for(Duration::from_secs(1)));
    assert!(
        matches!(
            runtime.instance_exit(&in_flight_key),
            Some(RuntimeInstanceExit::Success)
        ),
        "the launch-racing clean exit should remain available"
    );
    assert!(!runtime.has_fatal_runtime_error());
}

/// Scenario: scope-host shutdown is followed by observability draining and runtime completion.
/// Guarantees: the default supervisor join includes completion grace and coordination slack.
#[test]
fn extension_scope_supervisor_guard_budget_includes_observability_completion_grace() {
    let runtime = test_runtime(&empty_engine_config());
    let handle = spawn_thread_local_task(
        "test-scope-observability-completion-budget",
        TracingSetup::new(ProviderSetup::Noop, LogLevel::default(), engine_context),
        move |cancellation_token| async move {
            cancellation_token.cancelled().await;
            Ok::<(), Error>(())
        },
    )
    .expect("test extension scope supervisor should spawn");
    let guard = ExtensionScopeSupervisorGuard::new(handle, runtime);
    let scope_shutdown_start = Instant::now();
    let observability_drain_deadline = scope_shutdown_start
        + RunningExtensionScopeSupervisor::SHUTDOWN_TIMEOUT
        + ControllerRuntime::<()>::OBSERVABILITY_SHUTDOWN_TIMEOUT;
    let observability_completion_deadline =
        pipeline_shutdown_completion_deadline(observability_drain_deadline);

    assert_eq!(
        guard.supervisor_shutdown_timeout,
        observability_completion_deadline.duration_since(scope_shutdown_start)
            + Duration::from_secs(1)
    );
}

/// Scenario: an early controller error drops the extension scope supervisor
/// guard while a reserved pipeline thread is still alive.
/// Guarantees: the guard closes launch admission and waits for the descendant
/// exit before cancelling the extension scope supervisor task.
#[test]
fn extension_scope_supervisor_guard_keeps_hosts_alive_until_reserved_pipeline_exits() {
    let runtime = test_runtime(&empty_engine_config());
    let in_flight_key = deployed_key("g1", "p1", 0, 10);
    runtime
        .reserve_instance_launch(&in_flight_key, current_test_context_bindings(&runtime))
        .expect("launch reservation should succeed");

    let (started_tx, started_rx) = std::sync::mpsc::channel();
    let (stopped_tx, stopped_rx) = std::sync::mpsc::channel();
    let handle = spawn_thread_local_task(
        "test-extension-scope-guard",
        TracingSetup::new(ProviderSetup::Noop, LogLevel::default(), engine_context),
        move |cancellation_token| async move {
            _ = started_tx.send(());
            cancellation_token.cancelled().await;
            _ = stopped_tx.send(());
            Ok::<(), Error>(())
        },
    )
    .expect("test extension scope supervisor should spawn");
    started_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("test extension scope supervisor should start");

    let guard = ExtensionScopeSupervisorGuard::new(handle, Arc::clone(&runtime));
    let drop_thread = thread::spawn(move || drop(guard));
    thread::sleep(Duration::from_millis(100));
    assert!(
        stopped_rx.try_recv().is_err(),
        "extension scope supervisor must remain alive while a descendant is reserved"
    );

    runtime.note_instance_exit(in_flight_key, RuntimeInstanceExit::Success);
    drop_thread
        .join()
        .expect("extension scope supervisor guard cleanup should finish");
    stopped_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("extension scope supervisor should stop after descendants exit");
}

/// Scenario: teardown begins while a recovery worker owns its fencing token
/// but no producer pipeline remains active.
/// Guarantees: the extension scope supervisor guard initiates global shutdown,
/// requests recovery cancellation, and waits for the worker before stopping providers.
#[test]
fn extension_scope_supervisor_guard_cancels_recovery_without_live_producer() {
    let runtime = test_runtime(&empty_engine_config());
    let pipeline_key = PipelineKey::new("g1".into(), "p1".into());
    {
        let mut state = runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let context_bindings = Arc::clone(&state.latest_context_bindings);
        _ = state.runtime_recoveries.insert(
            (pipeline_key.clone(), 0),
            RuntimeRecoveryState {
                serving_generation: 0,
                context_bindings,
                restart_count: 0,
                ready_since: None,
                worker_id: Some(43),
                candidate_generation: None,
                cancel_requested: false,
            },
        );
    }

    let recovery_runtime = Arc::clone(&runtime);
    let recovery_thread = thread::spawn(move || {
        for _ in 0..100 {
            let mut state = recovery_runtime
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let recovery = state
                .runtime_recoveries
                .get_mut(&(pipeline_key.clone(), 0))
                .expect("synthetic recovery should remain present");
            if recovery.cancel_requested {
                recovery.worker_id = None;
                recovery_runtime.state_changed.notify_all();
                return;
            }
            drop(state);
            thread::sleep(Duration::from_millis(5));
        }
        panic!("extension scope supervisor guard did not cancel the synthetic recovery worker");
    });

    let (stopped_tx, stopped_rx) = std::sync::mpsc::channel();
    let handle = spawn_thread_local_task(
        "test-extension-scope-recovery-drain",
        TracingSetup::new(ProviderSetup::Noop, LogLevel::default(), engine_context),
        move |cancellation_token| async move {
            cancellation_token.cancelled().await;
            _ = stopped_tx.send(());
            Ok::<(), Error>(())
        },
    )
    .expect("test extension scope supervisor should spawn");
    let guard = ExtensionScopeSupervisorGuard::new_with_timeouts(
        handle,
        Arc::clone(&runtime),
        Duration::from_secs(1),
        Duration::from_secs(1),
    );

    drop(guard);
    recovery_thread
        .join()
        .expect("synthetic recovery worker should stop");
    stopped_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("extension scope supervisor should stop after recovery worker release");
    assert!(runtime.wait_for_runtime_recoveries_for(Duration::ZERO));
}

/// Scenario: a descendant pipeline remains live beyond the extension scope
/// supervisor guard's finite cleanup budget.
/// Guarantees: scope-host teardown returns a timeout error and cancels the
/// supervisor instead of waiting forever.
#[test]
fn extension_scope_supervisor_guard_returns_after_descendant_timeout() {
    let runtime = test_runtime(&empty_engine_config());
    let stalled_key = deployed_key("g1", "p1", 0, 11);
    runtime
        .reserve_instance_launch(&stalled_key, current_test_context_bindings(&runtime))
        .expect("launch reservation should succeed");

    let (stopped_tx, stopped_rx) = std::sync::mpsc::channel();
    let handle = spawn_thread_local_task(
        "test-extension-scope-timeout",
        TracingSetup::new(ProviderSetup::Noop, LogLevel::default(), engine_context),
        move |cancellation_token| async move {
            cancellation_token.cancelled().await;
            _ = stopped_tx.send(());
            Ok::<(), Error>(())
        },
    )
    .expect("test extension scope supervisor should spawn");
    let mut guard = ExtensionScopeSupervisorGuard::new_with_timeouts(
        handle,
        Arc::clone(&runtime),
        Duration::from_millis(150),
        Duration::from_millis(150),
    );

    let started = Instant::now();
    let error = guard
        .shutdown_after_descendants()
        .expect("stalled descendant should produce a timeout");
    assert!(
        error
            .to_string()
            .contains("global shutdown deadline elapsed")
    );
    assert!(
        started.elapsed() < Duration::from_secs(1),
        "descendant and supervisor cleanup must each remain bounded"
    );
    stopped_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("extension scope supervisor should be cancelled after timeout");

    runtime.abort_instance_launch(&stalled_key);
    assert!(runtime.wait_for_global_shutdown_completion_for(Duration::from_secs(2)));
}

/// Scenario: producer draining has finished but its deadline is already past when scope shutdown begins.
/// Guarantees: the supervisor gets its own finite window without changing the producer deadline.
#[test]
fn extension_scope_supervisor_guard_starts_new_budget_after_producer_deadline() {
    let runtime = test_runtime(&empty_engine_config());
    let producer_deadline = Instant::now() - Duration::from_secs(1);
    runtime
        .request_shutdown_all_until(producer_deadline)
        .expect("completed producer phase should establish its deadline");
    let handle = spawn_thread_local_task(
        "test-scope-independent-shutdown-budget",
        TracingSetup::new(ProviderSetup::Noop, LogLevel::default(), engine_context),
        move |cancellation_token| async move {
            cancellation_token.cancelled().await;
            tokio::time::sleep(Duration::from_millis(100)).await;
            Ok::<(), Error>(())
        },
    )
    .expect("scope supervisor should start");
    let mut guard = ExtensionScopeSupervisorGuard::new(handle, Arc::clone(&runtime));
    let error = guard.shutdown_after_descendants();
    assert!(
        error.is_none(),
        "expired producer grace must not truncate the next phase: {error:?}"
    );
    assert_eq!(
        runtime.global_shutdown_deadline_or_insert(Duration::from_secs(60)),
        producer_deadline
    );
}

/// Scenario: a scope thread outlives its join budget and reports metrics after controller teardown returns.
/// Guarantees: timeout detaches without stopping collection; observability gets fresh grace and owns final draining.
#[tokio::test(flavor = "current_thread")]
async fn extension_scope_supervisor_guard_bounds_scope_host_thread_join() {
    let runtime = test_runtime(&empty_engine_config());
    let observability_key = deployed_key(
        SYSTEM_PIPELINE_GROUP_ID,
        SYSTEM_OBSERVABILITY_PIPELINE_ID,
        0,
        0,
    );
    let (observability_sender, observability_shutdown) = deadline_notifying_admin_sender();
    register_runtime_instance_with_sender(
        &runtime,
        observability_key.clone(),
        observability_sender,
        RuntimeInstanceLifecycle::Active,
    );

    struct ExitObservabilityOnDrop(Arc<ControllerRuntime<()>>, DeployedPipelineKey);
    impl Drop for ExitObservabilityOnDrop {
        fn drop(&mut self) {
            self.0
                .note_instance_exit(self.1.clone(), RuntimeInstanceExit::Success);
        }
    }
    let observability_exit = ExitObservabilityOnDrop(Arc::clone(&runtime), observability_key);

    let telemetry = InternalTelemetrySystem::default();
    let collector = telemetry.collector();
    let (collector_ready_tx, collector_ready_rx) = std_mpsc::channel();
    let (collector_stopped_tx, collector_stopped_rx) = std_mpsc::channel();
    // As in controller teardown, only lifetime ownership crosses these threads.
    let collector_handle = Arc::new(
        spawn_thread_local_task(
            "test-deferred-scope-collector",
            TracingSetup::new(ProviderSetup::Noop, LogLevel::default(), engine_context),
            move |cancellation_token| {
                let task = collector.run(cancellation_token);
                _ = collector_ready_tx.send(());
                async move {
                    let result = task.await;
                    _ = collector_stopped_tx.send(());
                    result
                }
            },
        )
        .expect("collector should start"),
    );
    collector_ready_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("collector should be running");
    let collector_lease = Arc::clone(&collector_handle);
    let context = ControllerContext::new(telemetry.registry());
    let mut metrics = telemetry
        .registry()
        .register_metric_set_for_entity::<OuterScopeLiveReconfigMetrics>(
            context.register_engine_entity(),
        );
    let mut reporter = telemetry.reporter();
    let (release_tx, release_rx) = tokio::sync::oneshot::channel();
    let (stopped_tx, stopped_rx) = std::sync::mpsc::channel();
    let (cleanup_tx, cleanup_rx) = std_mpsc::channel();
    let completion_runtime = Arc::clone(&runtime);
    let handle = spawn_thread_local_task_with_cleanup(
        "test-slow-extension-scope-shutdown",
        TracingSetup::new(ProviderSetup::Noop, LogLevel::default(), engine_context),
        move |cancellation_token| async move {
            cancellation_token.cancelled().await;
            release_rx
                .await
                .map_err(|error| Error::PipelineRuntimeError {
                    source: Box::new(error),
                })?;
            metrics.reported.add(37);
            reporter.report(&mut metrics)?;
            reporter.flush().await?;
            _ = stopped_tx.send(Instant::now());
            Ok::<(), Error>(())
        },
        move |cancellation_token| {
            completion_runtime.finish_shutdown_after_extension_scopes(&cancellation_token);
            drop(collector_lease);
            _ = cleanup_tx.send(());
        },
    )
    .expect("test extension scope supervisor should spawn");
    let mut guard = ExtensionScopeSupervisorGuard::new_with_timeouts(
        handle,
        Arc::clone(&runtime),
        Duration::from_millis(100),
        Duration::from_millis(100),
    );

    let started = Instant::now();
    let error = guard
        .shutdown_after_descendants()
        .expect("slow extension scope shutdown should time out");
    assert!(matches!(error, Error::ThreadJoinTimeout { .. }));
    assert!(
        started.elapsed() < Duration::from_millis(300),
        "extension scope supervisor thread join must honor its own deadline"
    );
    assert!(
        observability_shutdown.try_recv().is_err(),
        "join timeout must not open observability shutdown"
    );
    let telemetry_error =
        shutdown_telemetry_task("test-deferred-scope-collector", collector_handle)
            .expect_err("controller must defer collector teardown while the scope thread is alive");
    assert!(telemetry_error.to_string().contains("shutdown deferred"));
    telemetry
        .reporter()
        .flush()
        .await
        .expect("collector must remain responsive after the join timeout");
    runtime.release_instance_wait();
    release_tx
        .send(())
        .expect("detached scope thread should still accept its completion barrier");
    let scope_finished = stopped_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("detached scope thread should report its final metrics");
    let (reason, deadline) = observability_shutdown
        .recv_timeout(Duration::from_secs(1))
        .expect("observability should stop after the extension scope hosts");
    assert_eq!(reason, "global shutdown");
    assert!(deadline >= scope_finished + ControllerRuntime::<()>::OBSERVABILITY_SHUTDOWN_TIMEOUT);
    let batch = telemetry.registry().drain_metric_export_batch();
    let values = batch
        .metric_sets
        .into_iter()
        .filter(|set| set.descriptor.name == "test.extension.outer_scope_live_reconfig")
        .map(|set| set.values)
        .collect::<Vec<_>>();
    assert_eq!(values, vec![vec![MetricValue::from(37_u64)]]);
    assert!(
        collector_stopped_rx.try_recv().is_err(),
        "fatal wait release must not stop collection while observability is still alive"
    );
    drop(observability_exit);
    collector_stopped_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("collector should stop after observability actually exits");
    cleanup_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("detached supervisor should release all deferred resources");
    assert!(runtime.wait_for_global_shutdown_completion_for(Duration::from_secs(1)));
}

/// Scenario: global shutdown begins while system observability and the
/// extension scope supervisor are both active.
/// Guarantees: extension scope hosts stop before observability receives its
/// shutdown signal, preserving terminal scope-host telemetry.
#[test]
fn extension_scope_supervisor_guard_stops_observability_after_scope_hosts() {
    let runtime = test_runtime(&empty_engine_config());
    let observability_key = deployed_key(
        SYSTEM_PIPELINE_GROUP_ID,
        SYSTEM_OBSERVABILITY_PIPELINE_ID,
        0,
        0,
    );
    let events = Arc::new(Mutex::new(Vec::new()));
    let observability_sender: Arc<dyn PipelineAdminSender> =
        Arc::new(RecordingPipelineAdminSender {
            calls: Arc::clone(&events),
            failure: None,
        });
    register_runtime_instance_with_sender(
        &runtime,
        observability_key.clone(),
        observability_sender,
        RuntimeInstanceLifecycle::Active,
    );
    runtime
        .request_shutdown_all(1)
        .expect("initial producer shutdown phase should succeed");
    assert!(
        events
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .is_empty(),
        "observability must remain active before extension scope shutdown"
    );

    let extension_scope_events = Arc::clone(&events);
    let handle = spawn_thread_local_task(
        "test-extension-scope-observability-order",
        TracingSetup::new(ProviderSetup::Noop, LogLevel::default(), engine_context),
        move |cancellation_token| async move {
            cancellation_token.cancelled().await;
            extension_scope_events
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .push("extension scope hosts stopped".to_owned());
            Ok::<(), Error>(())
        },
    )
    .expect("test extension scope supervisor should spawn");
    let guard = ExtensionScopeSupervisorGuard::new_with_timeouts(
        handle,
        Arc::clone(&runtime),
        Duration::from_secs(2),
        Duration::from_secs(2),
    );
    let drop_thread = thread::spawn(move || drop(guard));

    let deadline = Instant::now() + Duration::from_secs(1);
    loop {
        let observed = events
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        if observed.len() >= 2 {
            assert_eq!(
                observed,
                ["extension scope hosts stopped", "global shutdown"]
            );
            break;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for ordered extension scope and observability shutdown"
        );
        thread::sleep(Duration::from_millis(10));
    }

    runtime.note_instance_exit(observability_key, RuntimeInstanceExit::Success);
    drop_thread
        .join()
        .expect("extension scope supervisor guard cleanup should finish");
}

/// Scenario: a runtime thread reports exit before the controller finishes
/// registering the launched instance as active.
/// Guarantees: early exit bookkeeping is reconciled during registration, so
/// active-instance tracking does not leak and the pending-exit entry is cleared.
#[test]
fn register_launched_instance_reconciles_early_exit_without_leaking_active_count() {
    let config = engine_config_with_pipeline(simple_pipeline_yaml());
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);

    let deployed_key = deployed_key("g1", "p1", 0, 0);
    runtime.note_instance_exit(deployed_key.clone(), RuntimeInstanceExit::Success);

    runtime.register_launched_instance(launched_runtime_instance(&runtime, "g1", "p1", 0, 0));

    let state = runtime
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    assert_eq!(state.active_instances, 0);
    assert!(!state.pending_instance_exits.contains_key(&deployed_key));
    assert!(!state.runtime_instances.contains_key(&deployed_key));
}

/// Scenario: a controller extension fails at runtime while a pipeline instance
/// is still active and never drains (e.g. the graceful shutdown request stalls).
/// Guarantees: `release_instance_wait` unblocks `wait_until_all_instances_exit`
/// unconditionally, so the main controller thread proceeds to teardown instead
/// of hanging. Regression test for the removed `thread::park`/`unpark` escape
/// hatch -- the condvar wait is now the only wake path and must honor the latch.
#[test]
fn release_instance_wait_unblocks_wait_with_active_instances() {
    let runtime = test_runtime(&empty_engine_config());

    // Simulate a launched, still-active pipeline instance that never exits.
    runtime.register_launched_instance(launched_runtime_instance(&runtime, "g1", "p1", 0, 0));
    assert_eq!(
        runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .active_instances,
        1
    );

    let waiter = {
        let runtime = Arc::clone(&runtime);
        thread::spawn(move || runtime.wait_until_all_instances_exit())
    };

    // The waiter must stay blocked while the instance is active and unreleased.
    thread::sleep(Duration::from_millis(100));
    assert!(
        !waiter.is_finished(),
        "waiter should block while an instance is active"
    );

    // Fatal-shutdown escape hatch: release the wait without the instance draining.
    runtime.release_instance_wait();

    let deadline = Instant::now() + Duration::from_secs(5);
    while !waiter.is_finished() {
        assert!(
            Instant::now() < deadline,
            "release_instance_wait did not unblock wait_until_all_instances_exit"
        );
        thread::sleep(Duration::from_millis(25));
    }
    waiter.join().expect("waiter thread should not panic");

    let state = runtime
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    assert_eq!(
        state.active_instances, 1,
        "release must not fabricate an instance exit"
    );
    assert!(state.instance_wait_released);
}

/// Scenario: standard engine mode has no active pipeline instances and has not received shutdown.
/// Guarantees: the lifecycle wait remains blocked until an explicit global shutdown is requested.
#[test]
fn global_shutdown_wait_keeps_an_empty_engine_alive() {
    let runtime = test_runtime(&empty_engine_config());

    let waiter = {
        let runtime = Arc::clone(&runtime);
        thread::spawn(move || runtime.wait_until_global_shutdown_drains_or_released())
    };

    thread::sleep(Duration::from_millis(100));
    assert!(
        !waiter.is_finished(),
        "an empty engine should remain alive before global shutdown"
    );

    runtime
        .request_shutdown_all(1)
        .expect("empty engine should accept global shutdown");
    waiter.join().expect("lifecycle waiter should not panic");
}

/// Scenario: a completed rollout has advanced the committed active generation,
/// but observed state still contains the older generation for the same core.
/// Guarantees: controller cleanup compacts observed state to the selected
/// active generation so retained instance memory no longer grows with rollout
/// count after completion.
#[test]
fn prune_pipeline_runtime_and_history_compacts_observed_state_to_active_generation() {
    let config = engine_config_with_pipeline(simple_pipeline_yaml());
    let runtime = test_runtime(&config);
    let _runner = ObservedStateRunner::start(&runtime);
    register_existing_pipeline(&runtime, &config);

    let pipeline_key = PipelineKey::new("g1".into(), "p1".into());
    report_ready(&runtime, deployed_key("g1", "p1", 0, 0));
    report_ready(&runtime, deployed_key("g1", "p1", 0, 1));
    let status = wait_for_observed_status(&runtime, &pipeline_key, |status| {
        status.per_instance().len() == 2
    });
    assert!(status.instance_status(0, 0).is_some());
    assert!(status.instance_status(0, 1).is_some());

    runtime
        .observed_state_store
        .set_pipeline_active_generation(pipeline_key.clone(), 1);
    runtime.prune_pipeline_runtime_and_history(&pipeline_key);

    let status = wait_for_observed_status(&runtime, &pipeline_key, |status| {
        status.per_instance().len() == 1
    });
    assert!(status.instance_status(0, 1).is_some());
    assert!(status.instance_status(0, 0).is_none());
}

/// Scenario: a logical pipeline has fully shut down and observed state still
/// contains an older generation alongside the final stopped generation.
/// Guarantees: controller cleanup keeps the last stopped generation per core so
/// `/status` remains useful after shutdown while superseded generations are
/// released.
#[test]
fn prune_pipeline_runtime_and_history_keeps_last_stopped_generation_view() {
    let config = engine_config_with_pipeline(simple_pipeline_yaml());
    let runtime = test_runtime(&config);
    let _runner = ObservedStateRunner::start(&runtime);
    register_existing_pipeline(&runtime, &config);

    let pipeline_key = PipelineKey::new("g1".into(), "p1".into());
    report_stopped(&runtime, deployed_key("g1", "p1", 0, 0));
    report_stopped(&runtime, deployed_key("g1", "p1", 0, 1));
    let status = wait_for_observed_status(&runtime, &pipeline_key, |status| {
        status.per_instance().len() == 2
    });
    assert!(status.instance_status(0, 0).is_some());
    assert!(status.instance_status(0, 1).is_some());

    runtime
        .observed_state_store
        .set_pipeline_active_generation(pipeline_key.clone(), 1);
    runtime.prune_pipeline_runtime_and_history(&pipeline_key);

    let status = wait_for_observed_status(&runtime, &pipeline_key, |status| {
        status.per_instance().len() == 1
    });
    assert_eq!(status.total_cores(), 1);
    assert_eq!(status.running_cores(), 0);
    assert!(matches!(
        status
            .instance_status(0, 1)
            .expect("latest stopped generation should remain")
            .phase(),
        PipelinePhase::Stopped
    ));
    assert!(status.instance_status(0, 0).is_none());
}

/// Scenario: a pure resize-down retires one core without changing the active
/// generation, and observed state still retains both core instances on that
/// same generation.
/// Guarantees: controller cleanup compacts observed state to the committed
/// active core footprint so `/status` stops counting the drained core as
/// serving after the resize completes.
#[test]
fn prune_pipeline_runtime_and_history_compacts_resize_down_same_generation() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 2
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    let _runner = ObservedStateRunner::start(&runtime);
    register_existing_pipeline(&runtime, &config);

    let pipeline_key = PipelineKey::new("g1".into(), "p1".into());
    report_ready(&runtime, deployed_key("g1", "p1", 0, 0));
    report_stopped(&runtime, deployed_key("g1", "p1", 1, 0));
    let status = wait_for_observed_status(&runtime, &pipeline_key, |status| {
        status.per_instance().len() == 2
    });
    assert_eq!(status.total_cores(), 2);
    assert_eq!(status.running_cores(), 1);
    assert!(status.instance_status(0, 0).is_some());
    assert!(status.instance_status(1, 0).is_some());

    runtime
        .observed_state_store
        .set_pipeline_active_cores(pipeline_key.clone(), [0]);
    runtime.prune_pipeline_runtime_and_history(&pipeline_key);

    let status = wait_for_observed_status(&runtime, &pipeline_key, |status| {
        status.per_instance().len() == 1
    });
    assert_eq!(status.total_cores(), 1);
    assert_eq!(status.running_cores(), 1);
    assert!(status.instance_status(0, 0).is_some());
    assert!(status.instance_status(1, 0).is_none());
}

/// Scenario: a runtime instance exits while a shutdown operation for the same
/// logical pipeline is still active and observed state contains overlapping
/// generations.
/// Guarantees: observed state is not compacted early, so controller wait paths
/// can continue reading generation-specific status until the shutdown finishes.
#[test]
fn note_instance_exit_does_not_compact_observed_state_while_shutdown_is_active() {
    let config = engine_config_with_pipeline(simple_pipeline_yaml());
    let runtime = test_runtime(&config);
    let _runner = ObservedStateRunner::start(&runtime);
    register_existing_pipeline(&runtime, &config);
    let _rx =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);

    let pipeline_key = PipelineKey::new("g1".into(), "p1".into());
    report_ready(&runtime, deployed_key("g1", "p1", 0, 0));
    report_ready(&runtime, deployed_key("g1", "p1", 0, 1));
    let status = wait_for_observed_status(&runtime, &pipeline_key, |status| {
        status.per_instance().len() == 2
    });
    assert!(status.instance_status(0, 0).is_some());
    assert!(status.instance_status(0, 1).is_some());

    {
        let mut state = runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let _ = state
            .active_shutdowns
            .insert(pipeline_key.clone(), "shutdown-0".to_owned());
    }

    runtime.note_instance_exit(deployed_key("g1", "p1", 0, 0), RuntimeInstanceExit::Success);

    let status = wait_for_observed_status(&runtime, &pipeline_key, |status| {
        status.per_instance().len() == 2
    });
    assert!(status.instance_status(0, 0).is_some());
    assert!(status.instance_status(0, 1).is_some());
}

/// Scenario: a watched runtime thread panics during an explicit operation after
/// the runtime instance has already been admitted and marked ready.
/// Guarantees: the public runtime error message stays short while the recent
/// event stores richer panic diagnostics in `ErrorSummary::source`.
#[test]
fn runtime_thread_panic_populates_error_source_in_observed_status() {
    let config = engine_config_with_pipeline(simple_pipeline_yaml());
    let runtime = test_runtime(&config);
    let _runner = ObservedStateRunner::start(&runtime);
    register_existing_pipeline(&runtime, &config);

    let deployed_key = deployed_key("g1", "p1", 0, 0);
    let _rx =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);
    report_ready(&runtime, deployed_key.clone());

    let pipeline_key = PipelineKey::new("g1".into(), "p1".into());
    let _ = wait_for_observed_status(&runtime, &pipeline_key, |status| {
        matches!(
            status
                .instance_status(0, 0)
                .map(|instance| instance.phase()),
            Some(PipelinePhase::Running)
        )
    });
    {
        let mut state = runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        _ = state
            .active_rollouts
            .insert(pipeline_key.clone(), "diagnostic-test".to_owned());
    }

    runtime.note_instance_exit(
        deployed_key,
        RuntimeInstanceExit::Error(RuntimeInstanceError::from_panic(PanicReport::capture(
            "runtime thread",
            Box::new("boom"),
            Some("pipeline-g1-p1-core-0".to_owned()),
            Some(11),
            Some(0),
        ))),
    );

    let status = wait_for_observed_status(&runtime, &pipeline_key, |status| {
        matches!(
            status
                .instance_status(0, 0)
                .map(|instance| instance.phase()),
            Some(PipelinePhase::Failed(_))
        )
    });
    let json = serde_json::to_value(&status).expect("status should serialize");
    let recent_event = &json["instances"][0]["status"]["recentEvents"][0]["Engine"];
    let error = &recent_event["type"]["Error"]["RuntimeError"]["Pipeline"];
    assert_eq!(
        recent_event["message"],
        "Pipeline encountered a runtime error."
    );
    assert_eq!(error["error_kind"], "panic");
    assert_eq!(error["message"], "runtime thread panicked: boom");
    let source = error["source"]
        .as_str()
        .expect("runtime panic source should be serialized");
    assert!(source.contains("thread_name=pipeline-g1-p1-core-0"));
    assert!(source.contains("thread_id=11"));
    assert!(source.contains("core_id=0"));
    assert!(source.contains("backtrace:"));
}

/// Scenario: one core in a two-core regular pipeline exits with a runtime error
/// while its sibling remains healthy, then a later rollout is planned.
/// Guarantees: the controller promotes a ready replacement generation only for
/// the failed core, preserves both cores in the serving readiness view, and
/// records each core's actual serving generation for the later rollout.
#[test]
fn runtime_error_recovers_failed_core_on_new_generation() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          runtime_recovery:
            max_restarts: 5
            initial_backoff: 5ms
            max_backoff: 20ms
            startup_timeout: 2s
            reset_after: 50ms
          resources:
            core_allocation:
              type: core_count
              count: 2
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime_with_factory(&config, &RECOVERY_TEST_PIPELINE_FACTORY);
    let _runner = ObservedStateRunner::start(&runtime);
    register_existing_pipeline(&runtime, &config);
    let _core0_rx =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);
    let _core1_rx =
        register_runtime_instance(&runtime, "g1", "p1", 1, 0, RuntimeInstanceLifecycle::Active);
    report_ready(&runtime, deployed_key("g1", "p1", 0, 0));
    report_ready(&runtime, deployed_key("g1", "p1", 1, 0));

    runtime.note_instance_exit(
        deployed_key("g1", "p1", 0, 0),
        RuntimeInstanceExit::Error(RuntimeInstanceError::runtime("boom".to_owned())),
    );

    let pipeline_key = PipelineKey::new("g1".into(), "p1".into());
    let status = wait_for_observed_status(&runtime, &pipeline_key, |status| {
        status
            .instance_status(0, 1)
            .is_some_and(|instance| matches!(instance.phase(), PipelinePhase::Running))
            && status.total_cores() == 2
            && status.running_cores() == 2
    });
    assert!(status.instance_status(1, 0).is_some());
    assert!(status.readiness());

    {
        let state = runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let recovery = state
            .runtime_recoveries
            .get(&(pipeline_key.clone(), 0))
            .expect("recovery state should remain available");
        assert_eq!(recovery.serving_generation, 1);
        assert_eq!(recovery.restart_count, 1);
        assert!(recovery.ready_since.is_some());
        assert!(recovery.worker_id.is_none());
        assert_eq!(state.active_instances, 2);
        assert!(state.first_error.is_none());
    }

    let replacement = config
        .groups
        .get(&PipelineGroupId::from("g1"))
        .and_then(|group| group.pipelines.get(&PipelineId::from("p1")))
        .cloned()
        .expect("replacement pipeline should exist");
    let plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: replacement,
                step_timeout_secs: 2,
                drain_timeout_secs: 2,
            },
        )
        .expect("a later rollout should accept mixed serving generations");
    assert_eq!(plan.action, RolloutAction::Replace);
    assert_eq!(plan.current_serving_generations.get(&0), Some(&1));
    assert_eq!(plan.current_serving_generations.get(&1), Some(&0));
    assert!(
        plan.rollout
            .cores
            .iter()
            .any(|core| core.core_id == 0 && core.previous_generation == Some(1))
    );
    assert!(
        plan.rollout
            .cores
            .iter()
            .any(|core| core.core_id == 1 && core.previous_generation == Some(0))
    );

    runtime
        .request_instance_shutdown(
            &deployed_key("g1", "p1", 0, 1),
            2,
            "runtime recovery test cleanup",
        )
        .expect("recovered runtime should accept shutdown");
    runtime.note_instance_exit(deployed_key("g1", "p1", 1, 0), RuntimeInstanceExit::Success);
}

/// Scenario: one serving core fails during rollout reservation and another
/// fails after active rollout insertion, with backoff shorter than both gaps.
/// Guarantees: both recoveries remain deferred through explicit ownership and
/// start only after the rollout releases the pipeline.
#[test]
fn runtime_error_during_rollout_handoff_is_deferred_until_finish() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          runtime_recovery:
            max_restarts: 5
            initial_backoff: 1ms
            max_backoff: 2ms
            startup_timeout: 2s
            reset_after: 1m
          resources:
            core_allocation:
              type: core_count
              count: 2
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime_with_factory(&config, &RECOVERY_TEST_PIPELINE_FACTORY);
    let _runner = ObservedStateRunner::start(&runtime);
    register_existing_pipeline(&runtime, &config);
    let _core0_rx =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);
    let _core1_rx =
        register_runtime_instance(&runtime, "g1", "p1", 1, 0, RuntimeInstanceLifecycle::Active);
    report_ready(&runtime, deployed_key("g1", "p1", 0, 0));
    report_ready(&runtime, deployed_key("g1", "p1", 1, 0));

    let pipeline_key = PipelineKey::new("g1".into(), "p1".into());
    let reservation = runtime
        .begin_pipeline_operation_reservation(pipeline_key.clone(), PipelineOperationKind::Rollout)
        .expect("rollout ownership should be reserved");
    runtime.note_instance_exit(
        deployed_key("g1", "p1", 0, 0),
        RuntimeInstanceExit::Error(RuntimeInstanceError::runtime("boom".to_owned())),
    );
    thread::sleep(Duration::from_millis(25));

    {
        let mut state = runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        assert!(
            state
                .deferred_runtime_recoveries
                .contains_key(&deployed_key("g1", "p1", 0, 0))
        );
        assert!(
            !state
                .runtime_recoveries
                .contains_key(&(pipeline_key.clone(), 0))
        );
        assert_eq!(state.generation_counters.get(&pipeline_key), Some(&1));
        let _ = state
            .active_rollouts
            .insert(pipeline_key.clone(), "rollout-handoff".to_owned());
    }
    drop(reservation);
    runtime.note_instance_exit(
        deployed_key("g1", "p1", 1, 0),
        RuntimeInstanceExit::Error(RuntimeInstanceError::runtime("boom sibling".to_owned())),
    );
    thread::sleep(Duration::from_millis(25));

    {
        let state = runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        assert!(
            state
                .deferred_runtime_recoveries
                .contains_key(&deployed_key("g1", "p1", 0, 0))
        );
        assert!(
            state
                .deferred_runtime_recoveries
                .contains_key(&deployed_key("g1", "p1", 1, 0))
        );
        assert!(
            !state
                .runtime_recoveries
                .contains_key(&(pipeline_key.clone(), 0))
        );
        assert!(
            !state
                .runtime_recoveries
                .contains_key(&(pipeline_key.clone(), 1))
        );
        assert_eq!(state.generation_counters.get(&pipeline_key), Some(&1));
    }

    runtime.finish_rollout(&pipeline_key, "rollout-handoff");
    let status = wait_for_observed_status(&runtime, &pipeline_key, |status| {
        status.total_cores() == 2 && status.running_cores() == 2
    });
    assert_eq!(status.total_cores(), 2);
    assert_eq!(status.running_cores(), 2);
    assert!(status.readiness());
    let (core0_generation, core1_generation) = {
        let state = runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        assert!(state.deferred_runtime_recoveries.is_empty());
        let core0_generation = state
            .runtime_recoveries
            .get(&(pipeline_key.clone(), 0))
            .expect("core 0 recovery should remain tracked")
            .serving_generation;
        let core1_generation = state
            .runtime_recoveries
            .get(&(pipeline_key.clone(), 1))
            .expect("core 1 recovery should remain tracked")
            .serving_generation;
        (core0_generation, core1_generation)
    };
    let mut generations = [core0_generation, core1_generation];
    generations.sort_unstable();
    assert_eq!(generations, [1, 2]);

    runtime
        .request_instance_shutdown(
            &deployed_key("g1", "p1", 0, core0_generation),
            2,
            "runtime recovery handoff test cleanup",
        )
        .expect("recovered runtime should accept shutdown");
    runtime
        .request_instance_shutdown(
            &deployed_key("g1", "p1", 1, core1_generation),
            2,
            "runtime recovery handoff test cleanup",
        )
        .expect("recovered sibling runtime should accept shutdown");
}

/// Scenario: a failed core already has a recovery worker in long backoff when
/// rollout planning reserves the pipeline and cancels that worker.
/// Guarantees: cancellation preserves the failed serving generation and starts
/// a fresh recovery worker if planning releases ownership without a rollout.
#[test]
fn rollout_reservation_preserves_cancelled_recovery_work() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          runtime_recovery:
            max_restarts: 5
            initial_backoff: 5s
            max_backoff: 5s
            startup_timeout: 1s
            reset_after: 1m
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime_with_factory(&config, &RECOVERY_TEST_PIPELINE_FACTORY);
    register_existing_pipeline(&runtime, &config);
    let _rx =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);
    let pipeline_key = PipelineKey::new("g1".into(), "p1".into());
    runtime.note_instance_exit(
        deployed_key("g1", "p1", 0, 0),
        RuntimeInstanceExit::Error(RuntimeInstanceError::runtime("boom".to_owned())),
    );

    let reservation = runtime
        .begin_pipeline_operation_reservation(pipeline_key.clone(), PipelineOperationKind::Rollout)
        .expect("rollout ownership should be reserved");
    runtime.cancel_runtime_recoveries_for_pipeline(&pipeline_key);
    {
        let state = runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        assert!(
            state
                .deferred_runtime_recoveries
                .contains_key(&deployed_key("g1", "p1", 0, 0))
        );
        assert!(
            state
                .runtime_recoveries
                .get(&(pipeline_key.clone(), 0))
                .is_some_and(|recovery| recovery.worker_id.is_none())
        );
    }

    drop(reservation);
    {
        let state = runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        assert!(state.deferred_runtime_recoveries.is_empty());
        assert!(
            state
                .runtime_recoveries
                .get(&(pipeline_key.clone(), 0))
                .is_some_and(|recovery| recovery.worker_id.is_some())
        );
    }
    runtime
        .request_shutdown_all(1)
        .expect("global shutdown should cancel the replacement worker");
}

/// Scenario: two rollout reservations cancel the same failed core while each
/// recovery worker is waiting in backoff with a two-launch restart budget.
/// Guarantees: cancelled waits consume no restart budget, and the first actual
/// replacement launch is still attempted and charged exactly once.
#[test]
fn cancelled_recovery_backoffs_do_not_consume_restart_budget() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          runtime_recovery:
            max_restarts: 2
            initial_backoff: 250ms
            max_backoff: 250ms
            startup_timeout: 2s
            reset_after: 1m
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime_with_factory(&config, &RECOVERY_TEST_PIPELINE_FACTORY);
    let _runner = ObservedStateRunner::start(&runtime);
    register_existing_pipeline(&runtime, &config);
    let _rx =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);
    report_ready(&runtime, deployed_key("g1", "p1", 0, 0));

    let pipeline_key = PipelineKey::new("g1".into(), "p1".into());
    runtime.note_instance_exit(
        deployed_key("g1", "p1", 0, 0),
        RuntimeInstanceExit::Error(RuntimeInstanceError::runtime("boom".to_owned())),
    );

    let mut cancelled_generations = Vec::new();
    for cancellation in 1..=2 {
        cancelled_generations.push(wait_for_recovery_candidate_generation(
            &runtime,
            &pipeline_key,
            0,
        ));
        let reservation = runtime
            .begin_pipeline_operation_reservation(
                pipeline_key.clone(),
                PipelineOperationKind::Rollout,
            )
            .expect("rollout ownership should be reserved");
        runtime.cancel_runtime_recoveries_for_pipeline(&pipeline_key);
        {
            let state = runtime
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let recovery = state
                .runtime_recoveries
                .get(&(pipeline_key.clone(), 0))
                .expect("cancelled recovery should remain tracked");
            assert_eq!(
                recovery.restart_count, 0,
                "cancellation {cancellation} consumed restart budget"
            );
            assert!(recovery.worker_id.is_none());
            assert!(recovery.candidate_generation.is_none());
            assert!(
                state
                    .deferred_runtime_recoveries
                    .contains_key(&deployed_key("g1", "p1", 0, 0))
            );
        }
        drop(reservation);
    }

    let last_cancelled_generation = *cancelled_generations
        .last()
        .expect("two recovery candidates should have been cancelled");
    let status = wait_for_observed_status(&runtime, &pipeline_key, |status| {
        status.running_cores() == 1
            && status
                .serving_generations()
                .get(&0)
                .is_some_and(|generation| *generation > last_cancelled_generation)
    });
    assert!(status.readiness());
    let serving_generation = {
        let state = runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let recovery = state
            .runtime_recoveries
            .get(&(pipeline_key.clone(), 0))
            .expect("successful recovery should remain tracked");
        assert_eq!(recovery.restart_count, 1);
        recovery.serving_generation
    };

    runtime
        .request_instance_shutdown(
            &deployed_key("g1", "p1", 0, serving_generation),
            2,
            "restart budget test cleanup",
        )
        .expect("recovered runtime should accept shutdown");
}

/// Scenario: a serving core fails while an engine-wide operation owns the
/// controller but does not replace or delete that pipeline.
/// Guarantees: the engine operation defers the failure and dropping its guard
/// resumes recovery for the still-serving generation.
#[test]
fn runtime_error_during_engine_operation_recovers_after_guard_release() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          runtime_recovery:
            max_restarts: 5
            initial_backoff: 1ms
            max_backoff: 2ms
            startup_timeout: 2s
            reset_after: 1m
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime_with_factory(&config, &RECOVERY_TEST_PIPELINE_FACTORY);
    let _runner = ObservedStateRunner::start(&runtime);
    register_existing_pipeline(&runtime, &config);
    let _rx =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);
    report_ready(&runtime, deployed_key("g1", "p1", 0, 0));

    let guard = runtime
        .begin_named_engine_operation("test-engine-operation".to_owned())
        .expect("engine operation should acquire ownership");
    runtime.note_instance_exit(
        deployed_key("g1", "p1", 0, 0),
        RuntimeInstanceExit::Error(RuntimeInstanceError::runtime("boom".to_owned())),
    );
    {
        let state = runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        assert!(
            state
                .deferred_runtime_recoveries
                .contains_key(&deployed_key("g1", "p1", 0, 0))
        );
        assert!(
            !state
                .runtime_recoveries
                .contains_key(&(PipelineKey::new("g1".into(), "p1".into()), 0))
        );
    }

    drop(guard);
    let pipeline_key = PipelineKey::new("g1".into(), "p1".into());
    let status = wait_for_observed_status(&runtime, &pipeline_key, |status| {
        status
            .instance_status(0, 1)
            .is_some_and(|instance| matches!(instance.phase(), PipelinePhase::Running))
            && status.total_cores() == 1
            && status.running_cores() == 1
    });
    assert!(status.readiness());
    assert!(
        runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .deferred_runtime_recoveries
            .is_empty()
    );

    runtime
        .request_instance_shutdown(
            &deployed_key("g1", "p1", 0, 1),
            2,
            "engine operation recovery test cleanup",
        )
        .expect("recovered runtime should accept shutdown");
}

/// Scenario: every replacement runtime fails before it becomes ready and the
/// configured restart budget is two attempts.
/// Guarantees: early exit races consume exactly two generations before the
/// controller records a fatal error and requests coordinated engine shutdown.
#[test]
fn runtime_recovery_exhaustion_fails_process_after_bounded_attempts() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          runtime_recovery:
            max_restarts: 2
            initial_backoff: 1ms
            max_backoff: 2ms
            startup_timeout: 200ms
            reset_after: 1s
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    let _runner = ObservedStateRunner::start(&runtime);
    register_existing_pipeline(&runtime, &config);
    let _rx =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);
    report_ready(&runtime, deployed_key("g1", "p1", 0, 0));

    runtime.note_instance_exit(
        deployed_key("g1", "p1", 0, 0),
        RuntimeInstanceExit::Error(RuntimeInstanceError::runtime(
            "initial runtime failure".to_owned(),
        )),
    );

    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let failed = {
            let state = runtime
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state.global_shutdown_requested && state.first_error.is_some()
        };
        if failed {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "runtime recovery did not exhaust within the test deadline"
        );
        thread::sleep(Duration::from_millis(10));
    }

    let state = runtime
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let recovery = state
        .runtime_recoveries
        .get(&(PipelineKey::new("g1".into(), "p1".into()), 0))
        .expect("recovery streak should remain available");
    assert_eq!(recovery.restart_count, 2);
    assert!(recovery.worker_id.is_none());
    assert_eq!(
        state
            .generation_counters
            .get(&PipelineKey::new("g1".into(), "p1".into())),
        Some(&3)
    );
    assert!(
        state
            .first_error
            .as_deref()
            .is_some_and(|error| error.contains("after 2 restart attempt(s)"))
    );
}

/// Scenario: a regular pipeline disables in-process runtime recovery and its
/// serving core exits unexpectedly while another active instance never drains.
/// Guarantees: no replacement generation is allocated, fatal coordinated
/// shutdown is requested, and the global lifecycle wait is released without
/// fabricating an exit for the non-draining instance.
#[test]
fn disabled_runtime_recovery_fails_without_launching_replacement() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          runtime_recovery:
            enabled: false
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);
    let _rx =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);
    let _non_draining =
        register_runtime_instance(&runtime, "g2", "p2", 1, 0, RuntimeInstanceLifecycle::Active);

    let waiter = {
        let runtime = Arc::clone(&runtime);
        thread::spawn(move || runtime.wait_until_global_shutdown_drains_or_released())
    };

    thread::sleep(Duration::from_millis(100));
    assert!(
        !waiter.is_finished(),
        "global lifecycle wait should block before fatal recovery"
    );

    runtime.note_instance_exit(
        deployed_key("g1", "p1", 0, 0),
        RuntimeInstanceExit::Error(RuntimeInstanceError::runtime("boom".to_owned())),
    );

    let deadline = Instant::now() + Duration::from_secs(5);
    while !waiter.is_finished() {
        assert!(
            Instant::now() < deadline,
            "fatal runtime recovery did not release the global lifecycle wait"
        );
        thread::sleep(Duration::from_millis(25));
    }
    waiter.join().expect("lifecycle waiter should not panic");

    let state = runtime
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    assert_eq!(
        state.active_instances, 1,
        "fatal recovery must not fabricate an instance exit"
    );
    assert!(state.instance_wait_released);
    assert!(state.global_shutdown_requested);
    assert_eq!(
        state
            .generation_counters
            .get(&PipelineKey::new("g1".into(), "p1".into())),
        Some(&1)
    );
    assert!(
        state
            .first_error
            .as_deref()
            .is_some_and(|error| error.contains("runtime recovery is disabled"))
    );
}

/// Scenario: an explicit rollout is planned while a failed core is waiting in
/// a long recovery backoff.
/// Guarantees: rollout planning cancels and joins the recovery worker before
/// snapshotting runtime state, preventing a late replacement launch.
#[test]
fn rollout_planning_cancels_scheduled_runtime_recovery() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          runtime_recovery:
            max_restarts: 5
            initial_backoff: 5s
            max_backoff: 5s
            startup_timeout: 1s
            reset_after: 1m
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime_with_factory(&config, &RECOVERY_TEST_PIPELINE_FACTORY);
    register_existing_pipeline(&runtime, &config);
    let _rx =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);
    runtime.note_instance_exit(
        deployed_key("g1", "p1", 0, 0),
        RuntimeInstanceExit::Error(RuntimeInstanceError::runtime("boom".to_owned())),
    );

    let replacement = config
        .groups
        .get(&PipelineGroupId::from("g1"))
        .and_then(|group| group.pipelines.get(&PipelineId::from("p1")))
        .cloned()
        .expect("replacement pipeline should exist");
    let plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: replacement,
                step_timeout_secs: 2,
                drain_timeout_secs: 2,
            },
        )
        .expect("rollout planning should cancel recovery");

    assert_eq!(plan.action, RolloutAction::Replace);
    let state = runtime
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    assert!(
        state
            .runtime_recoveries
            .get(&(PipelineKey::new("g1".into(), "p1".into()), 0))
            .is_some_and(|recovery| recovery.worker_id.is_none())
    );
    assert_eq!(state.active_instances, 0);
    assert!(state.first_error.is_none());
}
