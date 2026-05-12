// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! End-to-end integration tests for the extension/capability wiring in
//! [`PipelineFactory::build`] and [`RuntimePipeline::run_forever`].
//!
//! Each test instantiates a self-contained `PipelineFactory<()>` with
//! one synthetic receiver factory, one noop exporter factory, and one
//! or more synthetic extension factories, then exercises the build
//! and/or run paths to verify the engine's contract:
//!
//! 1. **Passive flow** — a passive extension's capability is resolvable
//!    via `Capabilities::require_local::<C>()` from within a receiver's
//!    `create()` body.
//! 2. **Per-variant pruning** — a dual-registration bundle whose
//!    consumers only call one of `require_local` / `require_shared`
//!    drops the unused variant before the runtime pipeline is handed
//!    off to `run_forever`.
//! 3. **Active spawn ordering** — an active extension's `start()` runs
//!    BEFORE data-path nodes, but capability *construction* (the
//!    receiver factory's `create()` body) runs *before* `start()` is
//!    invoked at all (because `create()` is build-time).
//! 4. **Fail-fast on extension error** — an active extension whose
//!    `start()` returns immediately aborts the pipeline; data-path
//!    drain is not awaited.
//! 5. **Shutdown ordering** — extensions receive
//!    `ExtensionControlMsg::Shutdown` only after data-path nodes have
//!    drained.

use async_trait::async_trait;
use otap_df_config::observed_state::{ObservedStateSettings, SendPolicy};
use otap_df_config::pipeline::PipelineConfig;
use otap_df_config::policy::{ChannelCapacityPolicy, TelemetryPolicy};
use otap_df_config::{DeployedPipelineKey, PipelineGroupId, PipelineId};
use otap_df_engine::ExporterFactory;
use otap_df_engine::ExtensionFactory;
use otap_df_engine::ReceiverFactory;
use otap_df_engine::capability::registry::Capabilities;
use otap_df_engine::config::{ExporterConfig, ExtensionConfig, ReceiverConfig};
use otap_df_engine::context::{ControllerContext, PipelineContext};
use otap_df_engine::control::{
    ExtensionControlMsg, RuntimeControlMsg, pipeline_completion_msg_channel,
    runtime_ctrl_msg_channel,
};
use otap_df_engine::error::Error as EngineError;
use otap_df_engine::exporter::ExporterWrapper;
use otap_df_engine::extension::{EffectHandler, ExtensionBundle, ExtensionWrapper};
use otap_df_engine::local::exporter as local_exp;
use otap_df_engine::local::processor as local_proc;
use otap_df_engine::local::receiver as local_recv;
use otap_df_engine::message::{ExporterInbox, Message};
use otap_df_engine::processor::ProcessorWrapper;
use otap_df_engine::receiver::ReceiverWrapper;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_engine::testing::capability::no_op_stateful::NoOpStateful;
use otap_df_engine::testing::capability::no_op_stateful::local::NoOpStateful as LocalNoOpStateful;
use otap_df_engine::testing::capability::no_op_stateful::shared::NoOpStateful as SharedNoOpStateful;
use otap_df_engine::testing::capability::no_op_stateless::NoOpStateless;
use otap_df_engine::testing::capability::no_op_stateless::local::NoOpStateless as LocalNoOpStateless;
use otap_df_engine::testing::capability::no_op_stateless::shared::NoOpStateless as SharedNoOpStateless;
use otap_df_engine::{PipelineFactory, extension_capabilities};
use otap_df_state::store::ObservedStateStore;
use otap_df_telemetry::InternalTelemetrySystem;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::time::{Duration, Instant};

// ─────────────────────────────────────────────────────────────────────
// Shared helpers — global probe registries thread per-test state into
// `static fn` factory bodies without unsafe code.
// ─────────────────────────────────────────────────────────────────────
//
// Receiver/extension factories are `static fn` pointers and cannot
// capture closures over per-test state. To pass `Arc<AtomicUsize>` /
// `Arc<Mutex<…>>` handles into a factory body, each test:
//
//   1. Inserts its handles into a global `RECEIVER_PROBES` /
//      `ACTIVE_EXT_PROBES` / `SHUTDOWN_RECORDING_PROBES` map keyed by a
//      string token (the extension id is reused as the key).
//   2. Embeds the same key in the node/extension `config` JSON as
//      `{"probe_key": "<token>"}`.
//   3. Inside the factory body, looks up the key in the global
//      registry and clones out the handles it needs.
//
// The registry is `Mutex<HashMap<…>>` (cheap; tests are not perf-
// critical), which avoids any `unsafe` and works fine across the
// rt-multi-thread bits we use in `run_pipeline_with_shutdown_after`.

/// Per-test state for the [`ProbeReceiver`] factory.
///
/// `sequence` selects which `Capabilities` accessors the factory body
/// invokes and in what order, so a single receiver factory can serve
/// every test variant (one-call, double-call one-shot enforcement,
/// stateful increment, etc.). Outcomes are captured in the various
/// counter / mutex fields.
#[derive(Clone)]
struct ReceiverProbe {
    /// Bumped by 1 on each `create()` call.
    create_calls: Arc<AtomicUsize>,
    /// Bumped by 1 if the FIRST claim on the binding succeeded.
    first_call_succeeded: Arc<AtomicUsize>,
    /// Bumped by 1 if the SECOND claim returned
    /// `Err(CapabilityAlreadyConsumed)` (one-shot enforcement
    /// tests). For sequences with no second call, stays at 0.
    second_call_already_consumed: Arc<AtomicUsize>,
    /// Captured `Display` of the second call's error (if any), so the
    /// one-shot tests can do a substring assert on the variant name.
    second_call_error_message: Arc<parking_lot::Mutex<Option<String>>>,
    /// Optional name returned by `NoOpStateless::name()` after a successful
    /// `require_local::<NoOpStateless>` call.
    captured_name: Arc<parking_lot::Mutex<Option<String>>>,
    /// Return value from `NoOpStateful::increment()` if the sequence
    /// invokes that path. Tests for shared state across nodes assert
    /// on this to verify each node observed a distinct counter value
    /// reflecting the shared `Arc<AtomicU64>`.
    stateful_increment_return: Arc<parking_lot::Mutex<Option<u64>>>,
    /// Selects which call sequence the factory body executes.
    sequence: CallSequence,
}

/// Enumerates the call sequences the [`ProbeReceiver`] factory can
/// execute against its `&Capabilities` input. Adding new variants is
/// strictly additive — existing tests keep their behavior.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)] // Some variants are reserved for future tests.
enum CallSequence {
    /// `require_local::<NoOpStateless>()` once.
    Local,
    /// `require_shared::<NoOpStateless>()` once.
    Shared,
    /// `require_local::<NoOpStateless>()`, then `require_local::<NoOpStateless>()`.
    /// Second call must error with `CapabilityAlreadyConsumed`.
    RequireLocalTwice,
    /// `require_local::<NoOpStateless>()`, then `require_shared::<NoOpStateless>()`.
    /// Second call must error with `CapabilityAlreadyConsumed`.
    RequireLocalThenRequireShared,
    /// `require_local::<NoOpStateless>()`, then `optional_local::<NoOpStateless>()`.
    /// Second call must error with `CapabilityAlreadyConsumed`.
    RequireLocalThenOptionalLocal,
    /// `require_local::<NoOpStateless>()`, then `optional_shared::<NoOpStateless>()`.
    /// Second call must error with `CapabilityAlreadyConsumed`.
    RequireLocalThenOptionalShared,
    /// `optional_local::<NoOpStateless>()`, then `require_local::<NoOpStateless>()`.
    /// Second call must error with `CapabilityAlreadyConsumed`.
    OptionalLocalThenRequireLocal,
    /// `optional_local::<NoOpStateless>()`, then `optional_local::<NoOpStateless>()`.
    /// Second call must error with `CapabilityAlreadyConsumed`.
    OptionalLocalThenOptionalLocal,
    /// `optional_shared::<NoOpStateless>()`, then `require_shared::<NoOpStateless>()`.
    /// Second call must error with `CapabilityAlreadyConsumed`.
    OptionalSharedThenRequireShared,
    /// `optional_shared::<NoOpStateless>()`, then `optional_shared::<NoOpStateless>()`.
    /// Second call must error with `CapabilityAlreadyConsumed`.
    OptionalSharedThenOptionalShared,
    /// `require_local::<NoOpStateful>()`, then call `.increment()` on the
    /// boxed handle. Captures the return value for shared-state tests.
    StatefulIncrement,
    /// `require_shared::<NoOpStateful>()`, then call `.increment()` on the
    /// boxed shared handle (sync `&mut self` through the `Send` trait
    /// variant). Captures the return value.
    SharedStatefulIncrement,
    /// `require_local::<NoOpStateful>()`; the create() body keeps the
    /// boxed handle alive by stashing it on the receiver, which then
    /// invokes `.record(7).await` from inside its `start()` body —
    /// exercising the async `&mut self` path on the local trait variant.
    LocalStatefulRecordAsync,
    /// `require_shared::<NoOpStateful>()`; analogous to
    /// [`Self::LocalStatefulRecordAsync`] but for the shared trait
    /// variant (async `&mut self` with `Send` future).
    SharedStatefulRecordAsync,
    /// `require_shared::<NoOpStateful>()` and call `count()` once at
    /// build time, capturing the value into the probe's
    /// `stateful_increment_return` slot. Used to assert that, before
    /// the active extension's `start()` task runs, capability
    /// consumers observe pre-mutation state.
    SharedStatefulReadCount,
    /// Don't claim anything. Used by tests where the receiver is just
    /// a structural placeholder.
    None,
}

static RECEIVER_PROBES: std::sync::OnceLock<
    std::sync::Mutex<std::collections::HashMap<String, ReceiverProbe>>,
> = std::sync::OnceLock::new();

fn receiver_probes() -> &'static std::sync::Mutex<std::collections::HashMap<String, ReceiverProbe>>
{
    RECEIVER_PROBES.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()))
}

fn register_receiver_probe(key: &str, probe: ReceiverProbe) {
    let _ = receiver_probes()
        .lock()
        .expect("receiver probes mutex poisoned")
        .insert(key.to_owned(), probe);
}

fn lookup_receiver_probe(key: &str) -> ReceiverProbe {
    receiver_probes()
        .lock()
        .expect("receiver probes mutex poisoned")
        .get(key)
        .cloned()
        .unwrap_or_else(|| panic!("no ReceiverProbe registered for key '{key}'"))
}

const PROBE_RECEIVER_URN: &str = "urn:test:receiver:probe";
const PROBE_PROCESSOR_URN: &str = "urn:test:processor:probe";
const PROBE_EXPORTER_URN: &str = "urn:test:exporter:probe";
const NOOP_EXPORTER_URN: &str = "urn:test:exporter:noop";
const PASSIVE_EXTENSION_URN: &str = "urn:test:extension:passive_extension";
const DUAL_EXTENSION_URN: &str = "urn:test:extension:dual_extension";
const ACTIVE_EXTENSION_URN: &str = "urn:test:extension:active_extension";
const FAILING_EXTENSION_URN: &str = "urn:test:extension:failing_extension";
const SHUTDOWN_RECORDING_EXTENSION_URN: &str = "urn:test:extension:shutdown_recording_extension";

// ─────────────────────────────────────────────────────────────────────
// Probe receiver — exercises Capabilities API in create()
// ─────────────────────────────────────────────────────────────────────

struct ProbeReceiver {
    lifecycle: Option<NodeLifecycleProbe>,
    /// If set, `start()` calls `handle.record(7).await` on the local
    /// stateful handle and stores the return value into the probe's
    /// `stateful_increment_return` slot. Exercises async `&mut self`
    /// on the local trait variant.
    async_local_stateful: Option<Box<dyn LocalNoOpStateful>>,
    /// As above but for the shared trait variant — exercises async
    /// `&mut self` on the `Send` shared handle.
    async_shared_stateful: Option<Box<dyn SharedNoOpStateful>>,
    /// Probe key used to publish async record return values back to
    /// the test (only consulted when one of the async fields is Some).
    probe_key: Option<String>,
}

#[async_trait(?Send)]
impl local_recv::Receiver<()> for ProbeReceiver {
    async fn start(
        self: Box<Self>,
        mut ctrl: local_recv::ControlChannel<()>,
        _eh: local_recv::EffectHandler<()>,
    ) -> Result<TerminalState, EngineError> {
        if let Some(lc) = &self.lifecycle {
            *lc.receiver_start_at.lock() = Some(Instant::now());
        }

        // Take the boxed async handles out of `self` and exercise them
        // before entering the control loop. This proves the async
        // `&mut self` codegen path is end-to-end invokable through a
        // `Box<dyn ...>` returned from the capability registry.
        let probe_key = self.probe_key.clone();
        let mut this = self;
        if let Some(mut handle) = this.async_local_stateful.take() {
            let value = handle.record(7).await;
            if let Some(key) = probe_key.as_deref() {
                let probe = lookup_receiver_probe(key);
                *probe.stateful_increment_return.lock() = Some(value);
            }
        }
        if let Some(mut handle) = this.async_shared_stateful.take() {
            let value = handle.record(11).await;
            if let Some(key) = probe_key.as_deref() {
                let probe = lookup_receiver_probe(key);
                *probe.stateful_increment_return.lock() = Some(value);
            }
        }

        loop {
            match ctrl.recv().await {
                Ok(otap_df_engine::control::NodeControlMsg::Shutdown { .. }) | Err(_) => break,
                Ok(_) => {}
            }
        }
        if let Some(lc) = &this.lifecycle {
            *lc.receiver_end_at.lock() = Some(Instant::now());
        }
        Ok(TerminalState::default())
    }
}

fn probe_receiver_create(
    _pipeline_ctx: PipelineContext,
    node: otap_df_engine::node::NodeId,
    node_config: Arc<otap_df_config::node::NodeUserConfig>,
    receiver_config: &ReceiverConfig,
    capabilities: &Capabilities,
) -> Result<ReceiverWrapper<()>, otap_df_config::error::Error> {
    let key = node_config
        .config
        .get("probe_key")
        .and_then(|v| v.as_str())
        .expect("probe_key present in receiver node config");
    let probe = lookup_receiver_probe(key);

    let _ = probe.create_calls.fetch_add(1, Ordering::SeqCst);

    // Optional async stateful handles, populated by the call sequences
    // that intentionally defer invocation to receiver `start()`.
    let mut async_local_stateful: Option<Box<dyn LocalNoOpStateful>> = None;
    let mut async_shared_stateful: Option<Box<dyn SharedNoOpStateful>> = None;

    fn record_already_consumed<E: std::fmt::Display>(probe: &ReceiverProbe, e: E) {
        let _ = probe
            .second_call_already_consumed
            .fetch_add(1, Ordering::SeqCst);
        *probe.second_call_error_message.lock() = Some(format!("{e}"));
    }

    match probe.sequence {
        CallSequence::None => {}
        CallSequence::Local => {
            if let Ok(handle) = capabilities.require_local::<NoOpStateless>() {
                let _ = probe.first_call_succeeded.fetch_add(1, Ordering::SeqCst);
                *probe.captured_name.lock() = Some(handle.name().to_owned());
            }
        }
        CallSequence::Shared => {
            if capabilities.require_shared::<NoOpStateless>().is_ok() {
                let _ = probe.first_call_succeeded.fetch_add(1, Ordering::SeqCst);
            }
        }
        CallSequence::RequireLocalTwice => {
            if capabilities.require_local::<NoOpStateless>().is_ok() {
                let _ = probe.first_call_succeeded.fetch_add(1, Ordering::SeqCst);
            }
            // Second call on the same binding must error.
            if let Err(e) = capabilities.require_local::<NoOpStateless>() {
                record_already_consumed(&probe, e);
            }
        }
        CallSequence::RequireLocalThenRequireShared => {
            if capabilities.require_local::<NoOpStateless>().is_ok() {
                let _ = probe.first_call_succeeded.fetch_add(1, Ordering::SeqCst);
            }
            if let Err(e) = capabilities.require_shared::<NoOpStateless>() {
                record_already_consumed(&probe, e);
            }
        }
        CallSequence::RequireLocalThenOptionalLocal => {
            if capabilities.require_local::<NoOpStateless>().is_ok() {
                let _ = probe.first_call_succeeded.fetch_add(1, Ordering::SeqCst);
            }
            if let Err(e) = capabilities.optional_local::<NoOpStateless>() {
                record_already_consumed(&probe, e);
            }
        }
        CallSequence::RequireLocalThenOptionalShared => {
            if capabilities.require_local::<NoOpStateless>().is_ok() {
                let _ = probe.first_call_succeeded.fetch_add(1, Ordering::SeqCst);
            }
            if let Err(e) = capabilities.optional_shared::<NoOpStateless>() {
                record_already_consumed(&probe, e);
            }
        }
        CallSequence::OptionalLocalThenRequireLocal => {
            if matches!(capabilities.optional_local::<NoOpStateless>(), Ok(Some(_))) {
                let _ = probe.first_call_succeeded.fetch_add(1, Ordering::SeqCst);
            }
            if let Err(e) = capabilities.require_local::<NoOpStateless>() {
                record_already_consumed(&probe, e);
            }
        }
        CallSequence::OptionalLocalThenOptionalLocal => {
            if matches!(capabilities.optional_local::<NoOpStateless>(), Ok(Some(_))) {
                let _ = probe.first_call_succeeded.fetch_add(1, Ordering::SeqCst);
            }
            if let Err(e) = capabilities.optional_local::<NoOpStateless>() {
                record_already_consumed(&probe, e);
            }
        }
        CallSequence::OptionalSharedThenRequireShared => {
            if matches!(capabilities.optional_shared::<NoOpStateless>(), Ok(Some(_))) {
                let _ = probe.first_call_succeeded.fetch_add(1, Ordering::SeqCst);
            }
            if let Err(e) = capabilities.require_shared::<NoOpStateless>() {
                record_already_consumed(&probe, e);
            }
        }
        CallSequence::OptionalSharedThenOptionalShared => {
            if matches!(capabilities.optional_shared::<NoOpStateless>(), Ok(Some(_))) {
                let _ = probe.first_call_succeeded.fetch_add(1, Ordering::SeqCst);
            }
            if let Err(e) = capabilities.optional_shared::<NoOpStateless>() {
                record_already_consumed(&probe, e);
            }
        }
        CallSequence::StatefulIncrement => {
            if let Ok(mut handle) = capabilities.require_local::<NoOpStateful>() {
                let _ = probe.first_call_succeeded.fetch_add(1, Ordering::SeqCst);
                let value = handle.increment();
                *probe.stateful_increment_return.lock() = Some(value);
            }
        }
        CallSequence::SharedStatefulIncrement => {
            if let Ok(mut handle) = capabilities.require_shared::<NoOpStateful>() {
                let _ = probe.first_call_succeeded.fetch_add(1, Ordering::SeqCst);
                let value = handle.increment();
                *probe.stateful_increment_return.lock() = Some(value);
            }
        }
        CallSequence::LocalStatefulRecordAsync => {
            if let Ok(handle) = capabilities.require_local::<NoOpStateful>() {
                let _ = probe.first_call_succeeded.fetch_add(1, Ordering::SeqCst);
                async_local_stateful = Some(handle);
            }
        }
        CallSequence::SharedStatefulRecordAsync => {
            if let Ok(handle) = capabilities.require_shared::<NoOpStateful>() {
                let _ = probe.first_call_succeeded.fetch_add(1, Ordering::SeqCst);
                async_shared_stateful = Some(handle);
            }
        }
        CallSequence::SharedStatefulReadCount => {
            if let Ok(handle) = capabilities.require_shared::<NoOpStateful>() {
                let _ = probe.first_call_succeeded.fetch_add(1, Ordering::SeqCst);
                let value = handle.count();
                *probe.stateful_increment_return.lock() = Some(value);
            }
        }
    }

    Ok(ReceiverWrapper::local(
        ProbeReceiver {
            lifecycle: node_config
                .config
                .get("lifecycle_key")
                .and_then(|v| v.as_str())
                .map(lookup_node_lifecycle_probe),
            async_local_stateful,
            async_shared_stateful,
            probe_key: Some(key.to_owned()),
        },
        node,
        node_config,
        receiver_config,
    ))
}

const PROBE_RECEIVER_FACTORY: ReceiverFactory<()> = ReceiverFactory {
    name: PROBE_RECEIVER_URN,
    create: probe_receiver_create,
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: otap_df_config::validation::no_config,
};

// ─────────────────────────────────────────────────────────────────────
// Noop exporter
// ─────────────────────────────────────────────────────────────────────

struct NoopExporter;

#[async_trait(?Send)]
impl local_exp::Exporter<()> for NoopExporter {
    async fn start(
        self: Box<Self>,
        mut inbox: ExporterInbox<()>,
        _eh: local_exp::EffectHandler<()>,
    ) -> Result<TerminalState, EngineError> {
        loop {
            if let Message::Control(otap_df_engine::control::NodeControlMsg::Shutdown { .. }) =
                inbox.recv().await?
            {
                break;
            }
        }
        Ok(TerminalState::default())
    }
}

fn noop_exporter_create(
    _pipeline_ctx: PipelineContext,
    node: otap_df_engine::node::NodeId,
    node_config: Arc<otap_df_config::node::NodeUserConfig>,
    exporter_config: &ExporterConfig,
    _capabilities: &Capabilities,
) -> Result<ExporterWrapper<()>, otap_df_config::error::Error> {
    Ok(ExporterWrapper::local(
        NoopExporter,
        node,
        node_config,
        exporter_config,
    ))
}

const NOOP_EXPORTER_FACTORY: ExporterFactory<()> = ExporterFactory {
    name: NOOP_EXPORTER_URN,
    create: noop_exporter_create,
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: otap_df_config::validation::no_config,
};

// ─────────────────────────────────────────────────────────────────────
// Stateless no-op extension impl shared across local + shared variants.
// ─────────────────────────────────────────────────────────────────────

#[derive(Clone)]
struct NoOpStatelessImpl {
    name: &'static str,
}

#[async_trait]
impl SharedNoOpStateless for NoOpStatelessImpl {
    fn name(&self) -> &str {
        self.name
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
impl LocalNoOpStateless for NoOpStatelessImpl {
    fn name(&self) -> &str {
        self.name
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

/// Distinct second concrete type so dual-registration tests can satisfy
/// the builder's "local and shared variants must use different concrete
/// types" rule. Functionally identical to [`NoOpStatelessImpl`].
#[derive(Clone)]
struct NoOpStatelessImplLocal {
    name: &'static str,
}

#[async_trait(?Send)]
impl LocalNoOpStateless for NoOpStatelessImplLocal {
    fn name(&self) -> &str {
        self.name
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

// ─────────────────────────────────────────────────────────────────────
// Passive (no-lifecycle) extension factory — provides NoOpStateless
// ─────────────────────────────────────────────────────────────────────

fn passive_extension_create(
    _pipeline_ctx: PipelineContext,
    name: otap_df_config::ExtensionId,
    user_config: Arc<otap_df_config::extension::ExtensionUserConfig>,
    extension_config: &ExtensionConfig,
) -> Result<ExtensionBundle, otap_df_config::error::Error> {
    let bundle = ExtensionWrapper::builder(name, user_config, extension_config)
        .passive()
        .cloned()
        .shared::<NoOpStatelessImpl>(NoOpStatelessImpl {
            name: "passive-noop",
        })
        .build()
        .expect("passive extension bundle builds");
    Ok(bundle)
}

const PASSIVE_EXTENSION_FACTORY: ExtensionFactory = ExtensionFactory {
    name: PASSIVE_EXTENSION_URN,
    description: "passive no-op extension providing NoOpStateless",
    documentation_url: "",
    capabilities: Some(extension_capabilities!(
        shared: NoOpStatelessImpl => [NoOpStateless]
    )),
    create: passive_extension_create,
    validate_config: otap_df_config::validation::no_config,
};

// ─────────────────────────────────────────────────────────────────────
// Dual extension factory — registers BOTH local and shared variants so
// the per-variant pruning test has something to drop.
// ─────────────────────────────────────────────────────────────────────

fn dual_extension_create(
    _pipeline_ctx: PipelineContext,
    name: otap_df_config::ExtensionId,
    user_config: Arc<otap_df_config::extension::ExtensionUserConfig>,
    extension_config: &ExtensionConfig,
) -> Result<ExtensionBundle, otap_df_config::error::Error> {
    let bundle = ExtensionWrapper::builder(name, user_config, extension_config)
        .passive()
        .cloned()
        .shared::<NoOpStatelessImpl>(NoOpStatelessImpl { name: "dual-noop" })
        .local::<NoOpStatelessImplLocal>(NoOpStatelessImplLocal { name: "dual-noop" })
        .build()
        .expect("dual extension bundle builds");
    Ok(bundle)
}

const DUAL_EXTENSION_FACTORY: ExtensionFactory = ExtensionFactory {
    name: DUAL_EXTENSION_URN,
    description: "passive extension with both local and shared NoOpStateless",
    documentation_url: "",
    capabilities: Some(extension_capabilities!(
        (shared: NoOpStatelessImpl, local: NoOpStatelessImplLocal) => [NoOpStateless]
    )),
    create: dual_extension_create,
    validate_config: otap_df_config::validation::no_config,
};

// ─────────────────────────────────────────────────────────────────────
// Active extension — observable start() / shutdown() side effects
// ─────────────────────────────────────────────────────────────────────

#[derive(Clone)]
struct ActiveExtImpl {
    started: Arc<AtomicBool>,
    start_at: Arc<parking_lot::Mutex<Option<Instant>>>,
    shutdown_at: Arc<parking_lot::Mutex<Option<Instant>>>,
}

#[async_trait]
impl SharedNoOpStateless for ActiveExtImpl {
    fn name(&self) -> &str {
        "active-noop"
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
impl otap_df_engine::shared::extension::Extension for ActiveExtImpl {
    async fn start(
        self: Box<Self>,
        mut ctrl: otap_df_engine::shared::extension::ControlChannel,
        _eh: EffectHandler,
    ) -> Result<TerminalState, EngineError> {
        self.started.store(true, Ordering::SeqCst);
        *self.start_at.lock() = Some(Instant::now());
        loop {
            match ctrl.recv().await {
                Ok(ExtensionControlMsg::Shutdown { .. }) | Err(_) => {
                    *self.shutdown_at.lock() = Some(Instant::now());
                    break;
                }
                Ok(_) => {}
            }
        }
        Ok(TerminalState::default())
    }
}

#[derive(Clone)]
struct ActiveExtProbe {
    started: Arc<AtomicBool>,
    start_at: Arc<parking_lot::Mutex<Option<Instant>>>,
    shutdown_at: Arc<parking_lot::Mutex<Option<Instant>>>,
}

static ACTIVE_EXT_PROBES: std::sync::OnceLock<
    std::sync::Mutex<std::collections::HashMap<String, ActiveExtProbe>>,
> = std::sync::OnceLock::new();

fn active_ext_probes()
-> &'static std::sync::Mutex<std::collections::HashMap<String, ActiveExtProbe>> {
    ACTIVE_EXT_PROBES.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()))
}

fn register_active_ext_probe(key: &str, probe: ActiveExtProbe) {
    let _ = active_ext_probes()
        .lock()
        .expect("active ext probes mutex poisoned")
        .insert(key.to_owned(), probe);
}

fn lookup_active_ext_probe(key: &str) -> ActiveExtProbe {
    active_ext_probes()
        .lock()
        .expect("active ext probes mutex poisoned")
        .get(key)
        .cloned()
        .unwrap_or_else(|| panic!("no ActiveExtProbe registered for key '{key}'"))
}

fn active_extension_create(
    _pipeline_ctx: PipelineContext,
    name: otap_df_config::ExtensionId,
    user_config: Arc<otap_df_config::extension::ExtensionUserConfig>,
    extension_config: &ExtensionConfig,
) -> Result<ExtensionBundle, otap_df_config::error::Error> {
    let key = user_config
        .config
        .get("probe_key")
        .and_then(|v| v.as_str())
        .expect("probe_key present in active extension config");
    let probe = lookup_active_ext_probe(key);
    let impl_ = ActiveExtImpl {
        started: Arc::clone(&probe.started),
        start_at: Arc::clone(&probe.start_at),
        shutdown_at: Arc::clone(&probe.shutdown_at),
    };
    let bundle = ExtensionWrapper::builder(name, user_config, extension_config)
        .active()
        .shared::<ActiveExtImpl>(impl_)
        .build()
        .expect("active extension bundle builds");
    Ok(bundle)
}

const ACTIVE_EXTENSION_FACTORY: ExtensionFactory = ExtensionFactory {
    name: ACTIVE_EXTENSION_URN,
    description: "active extension exposing NoOpStateless with observable start/shutdown",
    documentation_url: "",
    capabilities: Some(extension_capabilities!(
        shared: ActiveExtImpl => [NoOpStateless]
    )),
    create: active_extension_create,
    validate_config: otap_df_config::validation::no_config,
};

// ─────────────────────────────────────────────────────────────────────
// Active SHARED-COUNTER extension — same struct provides BOTH the
// extension lifecycle AND a `NoOpStateful` capability backed by an
// `Arc<AtomicU64>`. The `start()` task bumps the counter a fixed
// number of times before entering its event loop, so capability
// consumers can observe the active extension mutating shared state
// during its run.
// ─────────────────────────────────────────────────────────────────────

#[derive(Clone)]
struct ActiveSharedCounterImpl {
    counter: Arc<AtomicU64>,
    bumps: u64,
}

#[async_trait]
impl SharedNoOpStateful for ActiveSharedCounterImpl {
    fn count(&self) -> u64 {
        self.counter.load(Ordering::SeqCst)
    }
    fn increment(&mut self) -> u64 {
        self.counter.fetch_add(1, Ordering::SeqCst) + 1
    }
    fn reset(&mut self) {
        self.counter.store(0, Ordering::SeqCst);
    }
    async fn record(&mut self, value: u64) -> u64 {
        self.counter.fetch_add(value, Ordering::SeqCst) + value
    }
    fn last_recorded(&self) -> Option<u64> {
        Some(self.counter.load(Ordering::SeqCst))
    }
}

#[async_trait]
impl otap_df_engine::shared::extension::Extension for ActiveSharedCounterImpl {
    async fn start(
        self: Box<Self>,
        mut ctrl: otap_df_engine::shared::extension::ControlChannel,
        _eh: EffectHandler,
    ) -> Result<TerminalState, EngineError> {
        // Mutate shared state from inside the active task — the canonical
        // pattern for an active extension publishing data to capability
        // consumers (e.g., a token-refresh loop or a state-warmup task).
        for _ in 0..self.bumps {
            let _ = self.counter.fetch_add(1, Ordering::SeqCst);
        }
        loop {
            match ctrl.recv().await {
                Ok(ExtensionControlMsg::Shutdown { .. }) | Err(_) => break,
                Ok(_) => {}
            }
        }
        Ok(TerminalState::default())
    }
}

const ACTIVE_SHARED_COUNTER_EXTENSION_URN: &str =
    "urn:test:extension:active_shared_counter_extension";

fn active_shared_counter_extension_create(
    _pipeline_ctx: PipelineContext,
    name: otap_df_config::ExtensionId,
    user_config: Arc<otap_df_config::extension::ExtensionUserConfig>,
    extension_config: &ExtensionConfig,
) -> Result<ExtensionBundle, otap_df_config::error::Error> {
    let key = user_config
        .config
        .get("probe_key")
        .and_then(|v| v.as_str())
        .expect("probe_key present in active_shared_counter extension config");
    let bumps = user_config
        .config
        .get("bumps")
        .and_then(|v| v.as_u64())
        .unwrap_or(3);
    let probe = lookup_shared_counter_probe(key);
    let impl_ = ActiveSharedCounterImpl {
        counter: Arc::clone(&probe.counter),
        bumps,
    };
    let bundle = ExtensionWrapper::builder(name, user_config, extension_config)
        .active()
        .shared::<ActiveSharedCounterImpl>(impl_)
        .build()
        .expect("active shared counter extension bundle builds");
    Ok(bundle)
}

const ACTIVE_SHARED_COUNTER_EXTENSION_FACTORY: ExtensionFactory = ExtensionFactory {
    name: ACTIVE_SHARED_COUNTER_EXTENSION_URN,
    description: "active extension whose start() task mutates an Arc<AtomicU64> exposed via NoOpStateful",
    documentation_url: "",
    capabilities: Some(extension_capabilities!(
        shared: ActiveSharedCounterImpl => [NoOpStateful]
    )),
    create: active_shared_counter_extension_create,
    validate_config: otap_df_config::validation::no_config,
};

// ─────────────────────────────────────────────────────────────────────
// Failing extension — start() returns an error immediately
// ─────────────────────────────────────────────────────────────────────

#[derive(Clone)]
struct FailingExtImpl;

#[async_trait]
impl SharedNoOpStateless for FailingExtImpl {
    fn name(&self) -> &str {
        "failing"
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
impl otap_df_engine::shared::extension::Extension for FailingExtImpl {
    async fn start(
        self: Box<Self>,
        _ctrl: otap_df_engine::shared::extension::ControlChannel,
        _eh: EffectHandler,
    ) -> Result<TerminalState, EngineError> {
        Err(EngineError::InternalError {
            message: "synthetic extension start failure".into(),
        })
    }
}

fn failing_extension_create(
    _pipeline_ctx: PipelineContext,
    name: otap_df_config::ExtensionId,
    user_config: Arc<otap_df_config::extension::ExtensionUserConfig>,
    extension_config: &ExtensionConfig,
) -> Result<ExtensionBundle, otap_df_config::error::Error> {
    let bundle = ExtensionWrapper::builder(name, user_config, extension_config)
        .active()
        .shared::<FailingExtImpl>(FailingExtImpl)
        .build()
        .expect("failing extension bundle builds");
    Ok(bundle)
}

const FAILING_EXTENSION_FACTORY: ExtensionFactory = ExtensionFactory {
    name: FAILING_EXTENSION_URN,
    description: "active extension whose start() returns an error",
    documentation_url: "",
    capabilities: Some(extension_capabilities!(
        shared: FailingExtImpl => [NoOpStateless]
    )),
    create: failing_extension_create,
    validate_config: otap_df_config::validation::no_config,
};

// ─────────────────────────────────────────────────────────────────────
// Shutdown-recording extension — captures the wall-clock instant at
// which Shutdown was received.
// ─────────────────────────────────────────────────────────────────────

#[derive(Clone)]
struct ShutdownRecordingExtImpl {
    shutdown_at: Arc<parking_lot::Mutex<Option<Instant>>>,
}

#[async_trait]
impl SharedNoOpStateless for ShutdownRecordingExtImpl {
    fn name(&self) -> &str {
        "shutdown-recorder"
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
impl otap_df_engine::shared::extension::Extension for ShutdownRecordingExtImpl {
    async fn start(
        self: Box<Self>,
        mut ctrl: otap_df_engine::shared::extension::ControlChannel,
        _eh: EffectHandler,
    ) -> Result<TerminalState, EngineError> {
        loop {
            match ctrl.recv().await {
                Ok(ExtensionControlMsg::Shutdown { .. }) | Err(_) => {
                    *self.shutdown_at.lock() = Some(Instant::now());
                    break;
                }
                Ok(_) => {}
            }
        }
        Ok(TerminalState::default())
    }
}

#[derive(Clone)]
struct ShutdownRecordingProbe {
    shutdown_at: Arc<parking_lot::Mutex<Option<Instant>>>,
}

static SHUTDOWN_RECORDING_PROBES: std::sync::OnceLock<
    std::sync::Mutex<std::collections::HashMap<String, ShutdownRecordingProbe>>,
> = std::sync::OnceLock::new();

fn shutdown_recording_probes()
-> &'static std::sync::Mutex<std::collections::HashMap<String, ShutdownRecordingProbe>> {
    SHUTDOWN_RECORDING_PROBES
        .get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()))
}

fn register_shutdown_recording_probe(key: &str, probe: ShutdownRecordingProbe) {
    let _ = shutdown_recording_probes()
        .lock()
        .expect("shutdown recording probes mutex poisoned")
        .insert(key.to_owned(), probe);
}

fn lookup_shutdown_recording_probe(key: &str) -> ShutdownRecordingProbe {
    shutdown_recording_probes()
        .lock()
        .expect("shutdown recording probes mutex poisoned")
        .get(key)
        .cloned()
        .unwrap_or_else(|| panic!("no ShutdownRecordingProbe registered for key '{key}'"))
}

fn shutdown_recording_extension_create(
    _pipeline_ctx: PipelineContext,
    name: otap_df_config::ExtensionId,
    user_config: Arc<otap_df_config::extension::ExtensionUserConfig>,
    extension_config: &ExtensionConfig,
) -> Result<ExtensionBundle, otap_df_config::error::Error> {
    let key = user_config
        .config
        .get("probe_key")
        .and_then(|v| v.as_str())
        .expect("probe_key present in shutdown_recording extension config");
    let probe = lookup_shutdown_recording_probe(key);
    let impl_ = ShutdownRecordingExtImpl {
        shutdown_at: Arc::clone(&probe.shutdown_at),
    };
    let bundle = ExtensionWrapper::builder(name, user_config, extension_config)
        .active()
        .shared::<ShutdownRecordingExtImpl>(impl_)
        .build()
        .expect("shutdown_recording extension bundle builds");
    Ok(bundle)
}

const SHUTDOWN_RECORDING_EXTENSION_FACTORY: ExtensionFactory = ExtensionFactory {
    name: SHUTDOWN_RECORDING_EXTENSION_URN,
    description: "active extension that records the moment Shutdown is received",
    documentation_url: "",
    capabilities: Some(extension_capabilities!(
        shared: ShutdownRecordingExtImpl => [NoOpStateless]
    )),
    create: shutdown_recording_extension_create,
    validate_config: otap_df_config::validation::no_config,
};

// ─────────────────────────────────────────────────────────────────────
// Dual-active extension — registers BOTH `.active().shared(...)` and
// `.active().local(Rc::new(...))`. Each side has its own observable
// `start()` so a test that consumes only one variant can assert the
// OTHER variant's wrapper was pruned (its `start()` never ran).
// ─────────────────────────────────────────────────────────────────────

/// `!Send` Active extension impl. Functionally similar to
/// [`ActiveExtImpl`] but a separate concrete type so the builder's
/// "local and shared variants must use different concrete types"
/// invariant is satisfied.
#[derive(Clone)]
struct ActiveLocalExtImpl {
    started: Arc<AtomicBool>,
    start_at: Arc<parking_lot::Mutex<Option<Instant>>>,
    shutdown_at: Arc<parking_lot::Mutex<Option<Instant>>>,
}

#[async_trait(?Send)]
impl LocalNoOpStateless for ActiveLocalExtImpl {
    fn name(&self) -> &str {
        "active-local-noop"
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
impl otap_df_engine::local::extension::Extension for ActiveLocalExtImpl {
    async fn start(
        self: Rc<Self>,
        mut ctrl: otap_df_engine::local::extension::ControlChannel,
        _eh: EffectHandler,
    ) -> Result<TerminalState, EngineError> {
        self.started.store(true, Ordering::SeqCst);
        *self.start_at.lock() = Some(Instant::now());
        loop {
            match ctrl.recv().await {
                Ok(ExtensionControlMsg::Shutdown { .. }) | Err(_) => {
                    *self.shutdown_at.lock() = Some(Instant::now());
                    break;
                }
                Ok(_) => {}
            }
        }
        Ok(TerminalState::default())
    }
}

const DUAL_ACTIVE_EXTENSION_URN: &str = "urn:test:extension:dual_active_extension";

fn dual_active_extension_create(
    _pipeline_ctx: PipelineContext,
    name: otap_df_config::ExtensionId,
    user_config: Arc<otap_df_config::extension::ExtensionUserConfig>,
    extension_config: &ExtensionConfig,
) -> Result<ExtensionBundle, otap_df_config::error::Error> {
    let local_key = user_config
        .config
        .get("local_probe_key")
        .and_then(|v| v.as_str())
        .expect("local_probe_key present in dual_active extension config");
    let shared_key = user_config
        .config
        .get("shared_probe_key")
        .and_then(|v| v.as_str())
        .expect("shared_probe_key present in dual_active extension config");
    let local_probe = lookup_active_ext_probe(local_key);
    let shared_probe = lookup_active_ext_probe(shared_key);

    let local_impl = ActiveLocalExtImpl {
        started: Arc::clone(&local_probe.started),
        start_at: Arc::clone(&local_probe.start_at),
        shutdown_at: Arc::clone(&local_probe.shutdown_at),
    };
    let shared_impl = ActiveExtImpl {
        started: Arc::clone(&shared_probe.started),
        start_at: Arc::clone(&shared_probe.start_at),
        shutdown_at: Arc::clone(&shared_probe.shutdown_at),
    };

    let bundle = ExtensionWrapper::builder(name, user_config, extension_config)
        .active()
        .shared::<ActiveExtImpl>(shared_impl)
        .local::<ActiveLocalExtImpl>(Rc::new(local_impl))
        .build()
        .expect("dual-active extension bundle builds");
    Ok(bundle)
}

const DUAL_ACTIVE_EXTENSION_FACTORY: ExtensionFactory = ExtensionFactory {
    name: DUAL_ACTIVE_EXTENSION_URN,
    description: "active extension with separate active local + active shared variants",
    documentation_url: "",
    capabilities: Some(extension_capabilities!(
        (shared: ActiveExtImpl, local: ActiveLocalExtImpl) => [NoOpStateless]
    )),
    create: dual_active_extension_create,
    validate_config: otap_df_config::validation::no_config,
};

// ─────────────────────────────────────────────────────────────────────
// Background extension — no capabilities, engine-driven event loop
// ─────────────────────────────────────────────────────────────────────

#[derive(Clone)]
struct BackgroundExtImpl {
    started: Arc<AtomicBool>,
    shutdown_at: Arc<parking_lot::Mutex<Option<Instant>>>,
}

#[async_trait]
impl otap_df_engine::shared::extension::Extension for BackgroundExtImpl {
    async fn start(
        self: Box<Self>,
        mut ctrl: otap_df_engine::shared::extension::ControlChannel,
        _eh: EffectHandler,
    ) -> Result<TerminalState, EngineError> {
        self.started.store(true, Ordering::SeqCst);
        loop {
            match ctrl.recv().await {
                Ok(ExtensionControlMsg::Shutdown { .. }) | Err(_) => {
                    *self.shutdown_at.lock() = Some(Instant::now());
                    break;
                }
                Ok(_) => {}
            }
        }
        Ok(TerminalState::default())
    }
}

const BACKGROUND_EXTENSION_URN: &str = "urn:test:extension:background_extension";

#[derive(Clone)]
struct BackgroundProbe {
    started: Arc<AtomicBool>,
    shutdown_at: Arc<parking_lot::Mutex<Option<Instant>>>,
}

static BACKGROUND_PROBES: std::sync::OnceLock<
    std::sync::Mutex<std::collections::HashMap<String, BackgroundProbe>>,
> = std::sync::OnceLock::new();

fn background_probes()
-> &'static std::sync::Mutex<std::collections::HashMap<String, BackgroundProbe>> {
    BACKGROUND_PROBES.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()))
}

fn register_background_probe(key: &str, probe: BackgroundProbe) {
    let _ = background_probes()
        .lock()
        .expect("background probes mutex poisoned")
        .insert(key.to_owned(), probe);
}

fn lookup_background_probe(key: &str) -> BackgroundProbe {
    background_probes()
        .lock()
        .expect("background probes mutex poisoned")
        .get(key)
        .cloned()
        .unwrap_or_else(|| panic!("no BackgroundProbe registered for key '{key}'"))
}

fn background_extension_create(
    _pipeline_ctx: PipelineContext,
    name: otap_df_config::ExtensionId,
    user_config: Arc<otap_df_config::extension::ExtensionUserConfig>,
    extension_config: &ExtensionConfig,
) -> Result<ExtensionBundle, otap_df_config::error::Error> {
    let key = user_config
        .config
        .get("probe_key")
        .and_then(|v| v.as_str())
        .expect("probe_key present in background extension config");
    let probe = lookup_background_probe(key);
    let impl_ = BackgroundExtImpl {
        started: Arc::clone(&probe.started),
        shutdown_at: Arc::clone(&probe.shutdown_at),
    };
    let bundle = ExtensionWrapper::builder(name, user_config, extension_config)
        .background()
        .shared::<BackgroundExtImpl>(impl_)
        .build()
        .expect("background extension bundle builds");
    Ok(bundle)
}

const BACKGROUND_EXTENSION_FACTORY: ExtensionFactory = ExtensionFactory {
    name: BACKGROUND_EXTENSION_URN,
    description: "background extension — engine-driven event loop, no capabilities",
    documentation_url: "",
    // `None` is the engine's runtime signal that this is a Background
    // extension: `register_into` skips capability registration, and
    // pruning treats this bundle as "always kept".
    capabilities: None,
    create: background_extension_create,
    validate_config: otap_df_config::validation::no_config,
};

// ─────────────────────────────────────────────────────────────────────
// Shared-counter extension — provides NoOpStateful via passive Cloned;
// holds an `Arc<AtomicU64>` counter so cloned consumers share state.
// ─────────────────────────────────────────────────────────────────────

#[derive(Clone)]
struct SharedCounterImpl {
    counter: Arc<AtomicU64>,
}

#[async_trait]
impl SharedNoOpStateful for SharedCounterImpl {
    fn count(&self) -> u64 {
        self.counter.load(Ordering::SeqCst)
    }
    fn increment(&mut self) -> u64 {
        // `&mut self` is fine — interior mutability via the `Arc<AtomicU64>`
        // means clones of `SharedCounterImpl` all observe the same
        // underlying value, demonstrating cross-consumer state sharing
        // when the impl explicitly opts in.
        self.counter.fetch_add(1, Ordering::SeqCst) + 1
    }
    fn reset(&mut self) {
        self.counter.store(0, Ordering::SeqCst);
    }
    async fn record(&mut self, value: u64) -> u64 {
        self.counter.fetch_add(value, Ordering::SeqCst) + value
    }
    fn last_recorded(&self) -> Option<u64> {
        Some(self.counter.load(Ordering::SeqCst))
    }
}

#[async_trait(?Send)]
impl LocalNoOpStateful for SharedCounterImpl {
    fn count(&self) -> u64 {
        self.counter.load(Ordering::SeqCst)
    }
    fn increment(&mut self) -> u64 {
        self.counter.fetch_add(1, Ordering::SeqCst) + 1
    }
    fn reset(&mut self) {
        self.counter.store(0, Ordering::SeqCst);
    }
    async fn record(&mut self, value: u64) -> u64 {
        self.counter.fetch_add(value, Ordering::SeqCst) + value
    }
    fn last_recorded(&self) -> Option<u64> {
        Some(self.counter.load(Ordering::SeqCst))
    }
}

const SHARED_COUNTER_EXTENSION_URN: &str = "urn:test:extension:shared_counter_extension";

#[derive(Clone)]
struct SharedCounterProbe {
    counter: Arc<AtomicU64>,
}

static SHARED_COUNTER_PROBES: std::sync::OnceLock<
    std::sync::Mutex<std::collections::HashMap<String, SharedCounterProbe>>,
> = std::sync::OnceLock::new();

fn shared_counter_probes()
-> &'static std::sync::Mutex<std::collections::HashMap<String, SharedCounterProbe>> {
    SHARED_COUNTER_PROBES.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()))
}

fn register_shared_counter_probe(key: &str, probe: SharedCounterProbe) {
    let _ = shared_counter_probes()
        .lock()
        .expect("shared counter probes mutex poisoned")
        .insert(key.to_owned(), probe);
}

fn lookup_shared_counter_probe(key: &str) -> SharedCounterProbe {
    shared_counter_probes()
        .lock()
        .expect("shared counter probes mutex poisoned")
        .get(key)
        .cloned()
        .unwrap_or_else(|| panic!("no SharedCounterProbe registered for key '{key}'"))
}

fn shared_counter_extension_create(
    _pipeline_ctx: PipelineContext,
    name: otap_df_config::ExtensionId,
    user_config: Arc<otap_df_config::extension::ExtensionUserConfig>,
    extension_config: &ExtensionConfig,
) -> Result<ExtensionBundle, otap_df_config::error::Error> {
    let key = user_config
        .config
        .get("probe_key")
        .and_then(|v| v.as_str())
        .expect("probe_key present in shared_counter extension config");
    let probe = lookup_shared_counter_probe(key);
    let impl_ = SharedCounterImpl {
        counter: Arc::clone(&probe.counter),
    };
    // `.passive().cloned()` — each consumer gets its own `Clone` of the
    // prototype. The `Arc<AtomicU64>` field means clones all point at
    // the same underlying counter, which is the explicit
    // share-state-via-`Arc` pattern documented in the architecture.
    let bundle = ExtensionWrapper::builder(name, user_config, extension_config)
        .passive()
        .cloned()
        .local::<SharedCounterImpl>(impl_)
        .build()
        .expect("shared counter extension bundle builds");
    Ok(bundle)
}

const SHARED_COUNTER_EXTENSION_FACTORY: ExtensionFactory = ExtensionFactory {
    name: SHARED_COUNTER_EXTENSION_URN,
    description: "passive cloned extension that shares an Arc<AtomicU64> counter across consumers",
    documentation_url: "",
    capabilities: Some(extension_capabilities!(
        local: SharedCounterImpl => [NoOpStateful]
    )),
    create: shared_counter_extension_create,
    validate_config: otap_df_config::validation::no_config,
};

// ─────────────────────────────────────────────────────────────────────
// Shared-counter extension (shared variant) — same `SharedCounterImpl`
// registered under `.passive().cloned().shared(...)` so tests can
// exercise the `require_shared::<NoOpStateful>()` path (sync + async
// `&mut self` on the `Send` shared trait variant).
// ─────────────────────────────────────────────────────────────────────

const SHARED_COUNTER_SHARED_EXTENSION_URN: &str =
    "urn:test:extension:shared_counter_shared_extension";

fn shared_counter_shared_extension_create(
    _pipeline_ctx: PipelineContext,
    name: otap_df_config::ExtensionId,
    user_config: Arc<otap_df_config::extension::ExtensionUserConfig>,
    extension_config: &ExtensionConfig,
) -> Result<ExtensionBundle, otap_df_config::error::Error> {
    let key = user_config
        .config
        .get("probe_key")
        .and_then(|v| v.as_str())
        .expect("probe_key present in shared_counter_shared extension config");
    let probe = lookup_shared_counter_probe(key);
    let impl_ = SharedCounterImpl {
        counter: Arc::clone(&probe.counter),
    };
    let bundle = ExtensionWrapper::builder(name, user_config, extension_config)
        .passive()
        .cloned()
        .shared::<SharedCounterImpl>(impl_)
        .build()
        .expect("shared counter (shared variant) extension bundle builds");
    Ok(bundle)
}

const SHARED_COUNTER_SHARED_EXTENSION_FACTORY: ExtensionFactory = ExtensionFactory {
    name: SHARED_COUNTER_SHARED_EXTENSION_URN,
    description: "passive cloned extension exposing SharedCounterImpl via the shared trait variant",
    documentation_url: "",
    capabilities: Some(extension_capabilities!(
        shared: SharedCounterImpl => [NoOpStateful]
    )),
    create: shared_counter_shared_extension_create,
    validate_config: otap_df_config::validation::no_config,
};

// ─────────────────────────────────────────────────────────────────────
// Constructed extension — `.passive().constructed(closure)` so each
// consumer triggers a fresh instance from the user-supplied closure.
// The closure increments a counter so the test can verify it ran once
// per consumer.
// ─────────────────────────────────────────────────────────────────────

#[derive(Clone)]
struct ConstructedNoOpImpl;

#[async_trait(?Send)]
impl LocalNoOpStateless for ConstructedNoOpImpl {
    fn name(&self) -> &str {
        "constructed-noop"
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

const CONSTRUCTED_EXTENSION_URN: &str = "urn:test:extension:constructed_extension";

#[derive(Clone)]
struct ConstructedProbe {
    /// Bumped by 1 every time the user-supplied factory closure runs.
    closure_invocations: Arc<AtomicUsize>,
}

static CONSTRUCTED_PROBES: std::sync::OnceLock<
    std::sync::Mutex<std::collections::HashMap<String, ConstructedProbe>>,
> = std::sync::OnceLock::new();

fn constructed_probes()
-> &'static std::sync::Mutex<std::collections::HashMap<String, ConstructedProbe>> {
    CONSTRUCTED_PROBES.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()))
}

fn register_constructed_probe(key: &str, probe: ConstructedProbe) {
    let _ = constructed_probes()
        .lock()
        .expect("constructed probes mutex poisoned")
        .insert(key.to_owned(), probe);
}

fn lookup_constructed_probe(key: &str) -> ConstructedProbe {
    constructed_probes()
        .lock()
        .expect("constructed probes mutex poisoned")
        .get(key)
        .cloned()
        .unwrap_or_else(|| panic!("no ConstructedProbe registered for key '{key}'"))
}

fn constructed_extension_create(
    _pipeline_ctx: PipelineContext,
    name: otap_df_config::ExtensionId,
    user_config: Arc<otap_df_config::extension::ExtensionUserConfig>,
    extension_config: &ExtensionConfig,
) -> Result<ExtensionBundle, otap_df_config::error::Error> {
    let key = user_config
        .config
        .get("probe_key")
        .and_then(|v| v.as_str())
        .expect("probe_key present in constructed extension config");
    let probe = lookup_constructed_probe(key);
    // `.passive().constructed(closure)` — the closure is invoked once
    // per consumer at `Capabilities::require_local` time. Each consumer
    // gets a fresh `ConstructedNoOpImpl` value, demonstrating
    // per-consumer instantiation.
    let counter = Arc::clone(&probe.closure_invocations);
    let bundle = ExtensionWrapper::builder(name, user_config, extension_config)
        .passive()
        .constructed()
        .local::<ConstructedNoOpImpl, _>(move || {
            let _ = counter.fetch_add(1, Ordering::SeqCst);
            ConstructedNoOpImpl
        })
        .build()
        .expect("constructed extension bundle builds");
    Ok(bundle)
}

const CONSTRUCTED_EXTENSION_FACTORY: ExtensionFactory = ExtensionFactory {
    name: CONSTRUCTED_EXTENSION_URN,
    description: "passive constructed extension; closure runs once per consumer",
    documentation_url: "",
    capabilities: Some(extension_capabilities!(
        local: ConstructedNoOpImpl => [NoOpStateless]
    )),
    create: constructed_extension_create,
    validate_config: otap_df_config::validation::no_config,
};

// ─────────────────────────────────────────────────────────────────────
// Rc-counter extension (LOCAL ONLY) — proves shared mutable state via
// `Rc<RefCell<...>>` is observable across multiple consumers when the
// impl is registered with `.passive().cloned()`. `Rc` is `!Send` so
// this impl can only be wired through the local trait variant.
// ─────────────────────────────────────────────────────────────────────

#[derive(Clone)]
struct RcCounterImpl {
    counter: Rc<std::cell::RefCell<u64>>,
}

#[async_trait(?Send)]
impl LocalNoOpStateful for RcCounterImpl {
    fn count(&self) -> u64 {
        *self.counter.borrow()
    }
    fn increment(&mut self) -> u64 {
        let mut c = self.counter.borrow_mut();
        *c += 1;
        *c
    }
    fn reset(&mut self) {
        *self.counter.borrow_mut() = 0;
    }
    async fn record(&mut self, value: u64) -> u64 {
        let mut c = self.counter.borrow_mut();
        *c += value;
        *c
    }
    fn last_recorded(&self) -> Option<u64> {
        Some(*self.counter.borrow())
    }
}

const RC_COUNTER_EXTENSION_URN: &str = "urn:test:extension:rc_counter_extension";

fn rc_counter_extension_create(
    _pipeline_ctx: PipelineContext,
    name: otap_df_config::ExtensionId,
    user_config: Arc<otap_df_config::extension::ExtensionUserConfig>,
    extension_config: &ExtensionConfig,
) -> Result<ExtensionBundle, otap_df_config::error::Error> {
    // The prototype owns one `Rc<RefCell<u64>>`; `.passive().cloned()`
    // hands each consumer a shallow `Clone` of the prototype, and
    // `Rc::clone` keeps the inner `RefCell` shared across consumers.
    let impl_ = RcCounterImpl {
        counter: Rc::new(std::cell::RefCell::new(0)),
    };
    let bundle = ExtensionWrapper::builder(name, user_config, extension_config)
        .passive()
        .cloned()
        .local::<RcCounterImpl>(impl_)
        .build()
        .expect("rc counter extension bundle builds");
    Ok(bundle)
}

const RC_COUNTER_EXTENSION_FACTORY: ExtensionFactory = ExtensionFactory {
    name: RC_COUNTER_EXTENSION_URN,
    description: "passive cloned extension whose impl shares an Rc<RefCell<u64>> across consumers",
    documentation_url: "",
    capabilities: Some(extension_capabilities!(
        local: RcCounterImpl => [NoOpStateful]
    )),
    create: rc_counter_extension_create,
    validate_config: otap_df_config::validation::no_config,
};

// ─────────────────────────────────────────────────────────────────────
// Node lifecycle probe — captures `start()` entry / exit timestamps for
// the probe receiver, processor, and exporter so lifecycle ordering
// tests can compare against extension start/shutdown timestamps.
// Separate from `ReceiverProbe` to keep the existing capability-call
// probe focused on its job.
// ─────────────────────────────────────────────────────────────────────

#[derive(Clone, Default)]
struct NodeLifecycleProbe {
    receiver_start_at: Arc<parking_lot::Mutex<Option<Instant>>>,
    receiver_end_at: Arc<parking_lot::Mutex<Option<Instant>>>,
    processor_first_call_at: Arc<parking_lot::Mutex<Option<Instant>>>,
    processor_end_at: Arc<parking_lot::Mutex<Option<Instant>>>,
    exporter_start_at: Arc<parking_lot::Mutex<Option<Instant>>>,
    exporter_end_at: Arc<parking_lot::Mutex<Option<Instant>>>,
}

static NODE_LIFECYCLE_PROBES: std::sync::OnceLock<
    std::sync::Mutex<std::collections::HashMap<String, NodeLifecycleProbe>>,
> = std::sync::OnceLock::new();

fn node_lifecycle_probes()
-> &'static std::sync::Mutex<std::collections::HashMap<String, NodeLifecycleProbe>> {
    NODE_LIFECYCLE_PROBES.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()))
}

fn register_node_lifecycle_probe(key: &str, probe: NodeLifecycleProbe) {
    let _ = node_lifecycle_probes()
        .lock()
        .expect("node lifecycle probes mutex poisoned")
        .insert(key.to_owned(), probe);
}

fn lookup_node_lifecycle_probe(key: &str) -> NodeLifecycleProbe {
    node_lifecycle_probes()
        .lock()
        .expect("node lifecycle probes mutex poisoned")
        .get(key)
        .cloned()
        .unwrap_or_else(|| panic!("no NodeLifecycleProbe registered for key '{key}'"))
}

// ─────────────────────────────────────────────────────────────────────
// Probe processor — exercises Capabilities API in create() and records
// lifecycle timestamps. Pass-through for pdata; loops on Shutdown.
// ─────────────────────────────────────────────────────────────────────

struct ProbeProcessor {
    lifecycle: Option<NodeLifecycleProbe>,
}

#[async_trait(?Send)]
impl local_proc::Processor<()> for ProbeProcessor {
    async fn process(
        &mut self,
        msg: Message<()>,
        _eh: &mut local_proc::EffectHandler<()>,
    ) -> Result<(), EngineError> {
        if let Some(lc) = &self.lifecycle {
            let mut slot = lc.processor_first_call_at.lock();
            if slot.is_none() {
                *slot = Some(Instant::now());
            }
        }
        if let Message::Control(otap_df_engine::control::NodeControlMsg::Shutdown { .. }) = msg {
            if let Some(lc) = &self.lifecycle {
                *lc.processor_end_at.lock() = Some(Instant::now());
            }
        }
        Ok(())
    }
}

fn probe_processor_create(
    _pipeline_ctx: PipelineContext,
    node: otap_df_engine::node::NodeId,
    node_config: Arc<otap_df_config::node::NodeUserConfig>,
    processor_config: &otap_df_engine::config::ProcessorConfig,
    capabilities: &Capabilities,
) -> Result<ProcessorWrapper<()>, otap_df_config::error::Error> {
    let probe_key = node_config.config.get("probe_key").and_then(|v| v.as_str());
    let lifecycle_key = node_config
        .config
        .get("lifecycle_key")
        .and_then(|v| v.as_str());
    let optional_only = node_config
        .config
        .get("optional_only")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if let Some(key) = probe_key {
        let probe = lookup_receiver_probe(key);
        let _ = probe.create_calls.fetch_add(1, Ordering::SeqCst);
        if optional_only {
            // Validate the "optional capability absent" path: with no
            // extension declared, optional_local must return Ok(None).
            let result = capabilities.optional_local::<NoOpStateless>();
            if matches!(result, Ok(None)) {
                let _ = probe.first_call_succeeded.fetch_add(1, Ordering::SeqCst);
            }
        } else if let Ok(handle) = capabilities.require_local::<NoOpStateless>() {
            let _ = probe.first_call_succeeded.fetch_add(1, Ordering::SeqCst);
            *probe.captured_name.lock() = Some(handle.name().to_owned());
        }
    }

    let lifecycle = lifecycle_key.map(lookup_node_lifecycle_probe);
    Ok(ProcessorWrapper::local(
        ProbeProcessor { lifecycle },
        node,
        node_config,
        processor_config,
    ))
}

const PROBE_PROCESSOR_FACTORY: otap_df_engine::ProcessorFactory<()> =
    otap_df_engine::ProcessorFactory {
        name: PROBE_PROCESSOR_URN,
        create: probe_processor_create,
        wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
        validate_config: otap_df_config::validation::no_config,
    };

// ─────────────────────────────────────────────────────────────────────
// Probe exporter — exercises Capabilities API in create() and records
// lifecycle timestamps. Identical lifecycle to NoopExporter otherwise.
// ─────────────────────────────────────────────────────────────────────

struct ProbeExporter {
    lifecycle: Option<NodeLifecycleProbe>,
}

#[async_trait(?Send)]
impl local_exp::Exporter<()> for ProbeExporter {
    async fn start(
        self: Box<Self>,
        mut inbox: ExporterInbox<()>,
        _eh: local_exp::EffectHandler<()>,
    ) -> Result<TerminalState, EngineError> {
        if let Some(lc) = &self.lifecycle {
            *lc.exporter_start_at.lock() = Some(Instant::now());
        }
        loop {
            if let Message::Control(otap_df_engine::control::NodeControlMsg::Shutdown { .. }) =
                inbox.recv().await?
            {
                break;
            }
        }
        if let Some(lc) = &self.lifecycle {
            *lc.exporter_end_at.lock() = Some(Instant::now());
        }
        Ok(TerminalState::default())
    }
}

fn probe_exporter_create(
    _pipeline_ctx: PipelineContext,
    node: otap_df_engine::node::NodeId,
    node_config: Arc<otap_df_config::node::NodeUserConfig>,
    exporter_config: &ExporterConfig,
    capabilities: &Capabilities,
) -> Result<ExporterWrapper<()>, otap_df_config::error::Error> {
    let probe_key = node_config.config.get("probe_key").and_then(|v| v.as_str());
    let lifecycle_key = node_config
        .config
        .get("lifecycle_key")
        .and_then(|v| v.as_str());
    let optional_only = node_config
        .config
        .get("optional_only")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if let Some(key) = probe_key {
        let probe = lookup_receiver_probe(key);
        let _ = probe.create_calls.fetch_add(1, Ordering::SeqCst);
        if optional_only {
            let result = capabilities.optional_local::<NoOpStateless>();
            if matches!(result, Ok(None)) {
                let _ = probe.first_call_succeeded.fetch_add(1, Ordering::SeqCst);
            }
        } else if let Ok(handle) = capabilities.require_local::<NoOpStateless>() {
            let _ = probe.first_call_succeeded.fetch_add(1, Ordering::SeqCst);
            *probe.captured_name.lock() = Some(handle.name().to_owned());
        }
    }

    let lifecycle = lifecycle_key.map(lookup_node_lifecycle_probe);
    Ok(ExporterWrapper::local(
        ProbeExporter { lifecycle },
        node,
        node_config,
        exporter_config,
    ))
}

const PROBE_EXPORTER_FACTORY: ExporterFactory<()> = ExporterFactory {
    name: PROBE_EXPORTER_URN,
    create: probe_exporter_create,
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: otap_df_config::validation::no_config,
};

// ─────────────────────────────────────────────────────────────────────
// PipelineFactory<()> wiring all the test factories
// ─────────────────────────────────────────────────────────────────────

const RECEIVER_FACTORIES: &[ReceiverFactory<()>] = &[PROBE_RECEIVER_FACTORY];
const PROCESSOR_FACTORIES: &[otap_df_engine::ProcessorFactory<()>] = &[PROBE_PROCESSOR_FACTORY];
const EXPORTER_FACTORIES: &[ExporterFactory<()>] = &[NOOP_EXPORTER_FACTORY, PROBE_EXPORTER_FACTORY];
const EXTENSION_FACTORIES: &[ExtensionFactory] = &[
    PASSIVE_EXTENSION_FACTORY,
    DUAL_EXTENSION_FACTORY,
    ACTIVE_EXTENSION_FACTORY,
    ACTIVE_SHARED_COUNTER_EXTENSION_FACTORY,
    FAILING_EXTENSION_FACTORY,
    SHUTDOWN_RECORDING_EXTENSION_FACTORY,
    DUAL_ACTIVE_EXTENSION_FACTORY,
    BACKGROUND_EXTENSION_FACTORY,
    SHARED_COUNTER_EXTENSION_FACTORY,
    SHARED_COUNTER_SHARED_EXTENSION_FACTORY,
    CONSTRUCTED_EXTENSION_FACTORY,
    RC_COUNTER_EXTENSION_FACTORY,
];

static TEST_PIPELINE_FACTORY: PipelineFactory<()> = PipelineFactory::new(
    RECEIVER_FACTORIES,
    PROCESSOR_FACTORIES,
    EXPORTER_FACTORIES,
    EXTENSION_FACTORIES,
);

// ─────────────────────────────────────────────────────────────────────
// Pipeline-build / run helpers shared across tests
// ─────────────────────────────────────────────────────────────────────

fn fresh_pipeline_context() -> (
    InternalTelemetrySystem,
    PipelineContext,
    otap_df_telemetry::registry::EntityKey,
) {
    let telemetry_system = InternalTelemetrySystem::default();
    let registry = telemetry_system.registry();
    let controller_ctx = ControllerContext::new(registry);
    let ctx = controller_ctx.pipeline_context_with(
        PipelineGroupId::from("test-group"),
        PipelineId::from("test-pipeline"),
        0,
        1,
        0,
    );
    let entity_key = ctx.register_pipeline_entity();
    (telemetry_system, ctx, entity_key)
}

fn run_pipeline_with_shutdown_after(
    runtime_pipeline: otap_df_engine::runtime_pipeline::RuntimePipeline<()>,
    pipeline_ctx: PipelineContext,
    pipeline_entity_key: otap_df_telemetry::registry::EntityKey,
    telemetry_system: InternalTelemetrySystem,
    shutdown_after: Duration,
) -> Result<Vec<()>, EngineError> {
    let channel_capacity_policy = ChannelCapacityPolicy::default();
    let (runtime_ctrl_tx, runtime_ctrl_rx) =
        runtime_ctrl_msg_channel(channel_capacity_policy.control.pipeline);
    let (pipeline_completion_tx, pipeline_completion_rx) =
        pipeline_completion_msg_channel(channel_capacity_policy.control.completion);

    let registry = telemetry_system.registry();
    let observed_state_store = ObservedStateStore::new(&ObservedStateSettings::default(), registry);
    let pipeline_key = DeployedPipelineKey {
        pipeline_group_id: pipeline_ctx.pipeline_group_id(),
        pipeline_id: pipeline_ctx.pipeline_id(),
        core_id: 0,
        deployment_generation: 0,
    };
    let metrics_reporter = telemetry_system.reporter();
    let event_reporter = observed_state_store.reporter(SendPolicy::default());

    let runtime_ctrl_tx_for_shutdown = runtime_ctrl_tx.clone();
    let _shutdown_handle = std::thread::spawn(move || {
        std::thread::sleep(shutdown_after);
        let deadline = Instant::now() + Duration::from_millis(200);
        let _ = runtime_ctrl_tx_for_shutdown.try_send(RuntimeControlMsg::Shutdown {
            deadline,
            reason: "test-driven shutdown".to_owned(),
        });
    });

    let _pipeline_entity_guard = otap_df_engine::entity_context::set_pipeline_entity_key(
        pipeline_ctx.metrics_registry(),
        pipeline_entity_key,
    );
    let (_memory_pressure_tx, memory_pressure_rx) = tokio::sync::watch::channel(
        otap_df_engine::memory_limiter::MemoryPressureChanged::initial(),
    );
    runtime_pipeline.run_forever(
        pipeline_key,
        pipeline_ctx,
        event_reporter,
        metrics_reporter,
        Duration::from_secs(1),
        memory_pressure_rx,
        runtime_ctrl_tx,
        runtime_ctrl_rx,
        pipeline_completion_tx,
        pipeline_completion_rx,
    )
}

fn build_test_runtime_pipeline(
    yaml: &str,
) -> (
    otap_df_engine::runtime_pipeline::RuntimePipeline<()>,
    PipelineContext,
    otap_df_telemetry::registry::EntityKey,
    InternalTelemetrySystem,
) {
    let config = PipelineConfig::from_yaml("test-group".into(), "test-pipeline".into(), yaml)
        .expect("yaml config parses + validates");
    let (telemetry_system, pipeline_ctx, entity_key) = fresh_pipeline_context();
    let runtime_pipeline = TEST_PIPELINE_FACTORY
        .build(
            pipeline_ctx.clone(),
            config,
            ChannelCapacityPolicy::default(),
            TelemetryPolicy::default(),
            None,
            None,
        )
        .expect("pipeline builds");
    (runtime_pipeline, pipeline_ctx, entity_key, telemetry_system)
}

struct ReceiverProbeHandles {
    create_calls: Arc<AtomicUsize>,
    first_call_succeeded: Arc<AtomicUsize>,
    second_call_already_consumed: Arc<AtomicUsize>,
    second_call_error_message: Arc<parking_lot::Mutex<Option<String>>>,
    captured_name: Arc<parking_lot::Mutex<Option<String>>>,
    stateful_increment_return: Arc<parking_lot::Mutex<Option<u64>>>,
}

fn make_probe(key: &str, sequence: CallSequence) -> ReceiverProbeHandles {
    let create_calls = Arc::new(AtomicUsize::new(0));
    let first_call_succeeded = Arc::new(AtomicUsize::new(0));
    let second_call_already_consumed = Arc::new(AtomicUsize::new(0));
    let second_call_error_message: Arc<parking_lot::Mutex<Option<String>>> =
        Arc::new(parking_lot::Mutex::new(None));
    let captured_name: Arc<parking_lot::Mutex<Option<String>>> =
        Arc::new(parking_lot::Mutex::new(None));
    let stateful_increment_return: Arc<parking_lot::Mutex<Option<u64>>> =
        Arc::new(parking_lot::Mutex::new(None));
    register_receiver_probe(
        key,
        ReceiverProbe {
            create_calls: Arc::clone(&create_calls),
            first_call_succeeded: Arc::clone(&first_call_succeeded),
            second_call_already_consumed: Arc::clone(&second_call_already_consumed),
            second_call_error_message: Arc::clone(&second_call_error_message),
            captured_name: Arc::clone(&captured_name),
            stateful_increment_return: Arc::clone(&stateful_increment_return),
            sequence,
        },
    );
    ReceiverProbeHandles {
        create_calls,
        first_call_succeeded,
        second_call_already_consumed,
        second_call_error_message,
        captured_name,
        stateful_increment_return,
    }
}

// ─────────────────────────────────────────────────────────────────────
// Test 1 — passive extension provides NoOpStateless to a node consumer
// ─────────────────────────────────────────────────────────────────────

#[test]
fn test_passive_extension_provides_capability_to_node() {
    let key = "passive-test";
    let probe = make_probe(key, CallSequence::Local);

    let yaml = format!(
        r#"
nodes:
  receiver:
    type: "{PROBE_RECEIVER_URN}"
    config:
      probe_key: "{key}"
    capabilities:
      no_op_stateless: "noop-ext"
  exporter:
    type: "{NOOP_EXPORTER_URN}"

extensions:
  noop-ext:
    type: "{PASSIVE_EXTENSION_URN}"

connections:
  - from: receiver
    to: exporter
"#
    );
    let (_runtime_pipeline, _ctx, _key, _ts) = build_test_runtime_pipeline(&yaml);

    assert_eq!(
        probe.create_calls.load(Ordering::SeqCst),
        1,
        "create() called once"
    );
    assert_eq!(
        probe.first_call_succeeded.load(Ordering::SeqCst),
        1,
        "require_local::<NoOpStateless>() succeeded"
    );
    assert_eq!(
        probe.captured_name.lock().as_deref(),
        Some("passive-noop"),
        "name from passive-cloned extension is observable to the consumer"
    );
}

// ─────────────────────────────────────────────────────────────────────
// Test 2 — dual ACTIVE local+shared; receiver consumes only local;
//          assert the shared variant's wrapper was pruned (start()
//          never ran). This is Category 3a in lib.rs — the unused
//          variant is silently dropped while the consumed variant
//          is kept and spawned.
// ─────────────────────────────────────────────────────────────────────

#[test]
fn test_dual_active_unused_variant_is_pruned_and_never_starts() {
    let receiver_key = "dual-active-recv";
    let probe = make_probe(receiver_key, CallSequence::Local);

    // Two distinct probes — one for the local Active variant, one for
    // the shared Active variant. After the run we'll assert local
    // started and shared did NOT (because pruning dropped its wrapper
    // before `run_forever` could spawn it).
    let local_probe_key = "dual-active-local-probe";
    let local_started = Arc::new(AtomicBool::new(false));
    let local_start_at: Arc<parking_lot::Mutex<Option<Instant>>> =
        Arc::new(parking_lot::Mutex::new(None));
    let local_shutdown_at: Arc<parking_lot::Mutex<Option<Instant>>> =
        Arc::new(parking_lot::Mutex::new(None));
    register_active_ext_probe(
        local_probe_key,
        ActiveExtProbe {
            started: Arc::clone(&local_started),
            start_at: Arc::clone(&local_start_at),
            shutdown_at: Arc::clone(&local_shutdown_at),
        },
    );

    let shared_probe_key = "dual-active-shared-probe";
    let shared_started = Arc::new(AtomicBool::new(false));
    let shared_start_at: Arc<parking_lot::Mutex<Option<Instant>>> =
        Arc::new(parking_lot::Mutex::new(None));
    let shared_shutdown_at: Arc<parking_lot::Mutex<Option<Instant>>> =
        Arc::new(parking_lot::Mutex::new(None));
    register_active_ext_probe(
        shared_probe_key,
        ActiveExtProbe {
            started: Arc::clone(&shared_started),
            start_at: Arc::clone(&shared_start_at),
            shutdown_at: Arc::clone(&shared_shutdown_at),
        },
    );

    let yaml = format!(
        r#"
nodes:
  receiver:
    type: "{PROBE_RECEIVER_URN}"
    config:
      probe_key: "{receiver_key}"
    capabilities:
      no_op_stateless: "dual-active-ext"
  exporter:
    type: "{NOOP_EXPORTER_URN}"

extensions:
  dual-active-ext:
    type: "{DUAL_ACTIVE_EXTENSION_URN}"
    config:
      local_probe_key: "{local_probe_key}"
      shared_probe_key: "{shared_probe_key}"

connections:
  - from: receiver
    to: exporter
"#
    );
    let (runtime_pipeline, ctx, entity_key, ts) = build_test_runtime_pipeline(&yaml);

    // Receiver consumed `require_local` at build time, before any
    // spawn. Neither variant has started yet.
    assert_eq!(probe.first_call_succeeded.load(Ordering::SeqCst), 1);
    assert!(!local_started.load(Ordering::SeqCst));
    assert!(!shared_started.load(Ordering::SeqCst));

    let result = run_pipeline_with_shutdown_after(
        runtime_pipeline,
        ctx,
        entity_key,
        ts,
        Duration::from_millis(150),
    );
    assert!(
        result.is_ok(),
        "pipeline must shut down cleanly: {result:?}"
    );

    // Consumed local variant: kept and spawned.
    assert!(
        local_started.load(Ordering::SeqCst),
        "local variant was consumed by the receiver, so its wrapper must be \
         kept and `start()` must run"
    );
    assert!(
        local_shutdown_at.lock().is_some(),
        "local variant must observe Shutdown"
    );

    // Unused shared variant: pruned and NEVER spawned.
    assert!(
        !shared_started.load(Ordering::SeqCst),
        "shared variant of the dual-active extension was NOT consumed, so \
         its wrapper must be pruned and `start()` must never run"
    );
    assert!(
        shared_start_at.lock().is_none(),
        "shared variant start_at must remain None"
    );
    assert!(
        shared_shutdown_at.lock().is_none(),
        "shared variant shutdown_at must remain None"
    );
}

// ─────────────────────────────────────────────────────────────────────
// Test 3 — active extension start() runs after build (capability
//          construction in receiver's create() observes start()=false)
// ─────────────────────────────────────────────────────────────────────

#[test]
fn test_active_extension_start_runs_after_create() {
    let receiver_key = "active-recv";
    let ext_key = "active-ext-probe";
    let probe = make_probe(receiver_key, CallSequence::Local);

    let started = Arc::new(AtomicBool::new(false));
    let start_at: Arc<parking_lot::Mutex<Option<Instant>>> =
        Arc::new(parking_lot::Mutex::new(None));
    let shutdown_at: Arc<parking_lot::Mutex<Option<Instant>>> =
        Arc::new(parking_lot::Mutex::new(None));
    register_active_ext_probe(
        ext_key,
        ActiveExtProbe {
            started: Arc::clone(&started),
            start_at: Arc::clone(&start_at),
            shutdown_at: Arc::clone(&shutdown_at),
        },
    );

    let yaml = format!(
        r#"
nodes:
  receiver:
    type: "{PROBE_RECEIVER_URN}"
    config:
      probe_key: "{receiver_key}"
    capabilities:
      no_op_stateless: "active-ext"
  exporter:
    type: "{NOOP_EXPORTER_URN}"

extensions:
  active-ext:
    type: "{ACTIVE_EXTENSION_URN}"
    config:
      probe_key: "{ext_key}"

connections:
  - from: receiver
    to: exporter
"#
    );
    let (runtime_pipeline, ctx, entity_key, ts) = build_test_runtime_pipeline(&yaml);

    // Capability construction happened during build, before any task
    // was spawned — assert start() is still false at this point.
    assert_eq!(probe.create_calls.load(Ordering::SeqCst), 1);
    assert_eq!(probe.first_call_succeeded.load(Ordering::SeqCst), 1);
    assert!(
        !started.load(Ordering::SeqCst),
        "start() must not have been called before run_forever spawns it"
    );

    let result = run_pipeline_with_shutdown_after(
        runtime_pipeline,
        ctx,
        entity_key,
        ts,
        Duration::from_millis(150),
    );
    assert!(
        result.is_ok(),
        "pipeline should shut down cleanly: {result:?}"
    );
    assert!(
        started.load(Ordering::SeqCst),
        "start() must run before pipeline returns"
    );
    assert!(
        shutdown_at.lock().is_some(),
        "extension must observe Shutdown"
    );
    assert!(
        start_at.lock().is_some(),
        "extension must record start timestamp"
    );
}

// ─────────────────────────────────────────────────────────────────────
// Test 4 — fail-fast on extension error
// ─────────────────────────────────────────────────────────────────────

#[test]
fn test_active_extension_failure_aborts_pipeline_fast() {
    let receiver_key = "failing-recv";
    let _probe = make_probe(receiver_key, CallSequence::Local);

    let yaml = format!(
        r#"
nodes:
  receiver:
    type: "{PROBE_RECEIVER_URN}"
    config:
      probe_key: "{receiver_key}"
    capabilities:
      no_op_stateless: "failing-ext"
  exporter:
    type: "{NOOP_EXPORTER_URN}"

extensions:
  failing-ext:
    type: "{FAILING_EXTENSION_URN}"

connections:
  - from: receiver
    to: exporter
"#
    );
    let (runtime_pipeline, ctx, entity_key, ts) = build_test_runtime_pipeline(&yaml);

    // 60s shutdown grace; if fail-fast works the test returns far sooner.
    let started_at = Instant::now();
    let result = run_pipeline_with_shutdown_after(
        runtime_pipeline,
        ctx,
        entity_key,
        ts,
        Duration::from_secs(60),
    );
    let elapsed = started_at.elapsed();
    assert!(
        result.is_err(),
        "pipeline should abort fast when an active extension errors at start: {result:?}"
    );
    assert!(
        elapsed < Duration::from_secs(5),
        "pipeline should have failed fast (well under 5s), but took {elapsed:?}"
    );
    let err = result.err().unwrap();
    let msg = format!("{err}");
    assert!(
        msg.contains("synthetic extension start failure"),
        "error should propagate the extension's failure reason, got: {msg}"
    );
}

// ─────────────────────────────────────────────────────────────────────
// Test 5 — shutdown ordering: extension records Shutdown timestamp
//          inside the pipeline run window
// ─────────────────────────────────────────────────────────────────────

#[test]
fn test_extension_receives_shutdown_within_run_window() {
    let receiver_key = "shutdown-recv";
    let ext_key = "shutdown-ext-probe";
    let _probe = make_probe(receiver_key, CallSequence::Local);

    let shutdown_at: Arc<parking_lot::Mutex<Option<Instant>>> =
        Arc::new(parking_lot::Mutex::new(None));
    register_shutdown_recording_probe(
        ext_key,
        ShutdownRecordingProbe {
            shutdown_at: Arc::clone(&shutdown_at),
        },
    );

    let yaml = format!(
        r#"
nodes:
  receiver:
    type: "{PROBE_RECEIVER_URN}"
    config:
      probe_key: "{receiver_key}"
    capabilities:
      no_op_stateless: "rec-ext"
  exporter:
    type: "{NOOP_EXPORTER_URN}"

extensions:
  rec-ext:
    type: "{SHUTDOWN_RECORDING_EXTENSION_URN}"
    config:
      probe_key: "{ext_key}"

connections:
  - from: receiver
    to: exporter
"#
    );
    let (runtime_pipeline, ctx, entity_key, ts) = build_test_runtime_pipeline(&yaml);

    let pipeline_start = Instant::now();
    let result = run_pipeline_with_shutdown_after(
        runtime_pipeline,
        ctx,
        entity_key,
        ts,
        Duration::from_millis(150),
    );
    let pipeline_end = Instant::now();
    assert!(
        result.is_ok(),
        "pipeline should shut down cleanly: {result:?}"
    );

    let observed_shutdown = shutdown_at
        .lock()
        .expect("extension must record Shutdown timestamp");
    // The extension receives Shutdown after the data-path has drained,
    // but the precise interleaving is implementation-defined and the
    // timestamps may be very close. The contract we assert here is
    // that the extension *did* observe Shutdown and did so within the
    // pipeline run window.
    assert!(
        observed_shutdown >= pipeline_start && observed_shutdown <= pipeline_end,
        "extension shutdown timestamp {observed_shutdown:?} must be within \
         pipeline run window [{pipeline_start:?}, {pipeline_end:?}]"
    );
}

// ─────────────────────────────────────────────────────────────────────
// Test 6 — one-shot enforcement: require_local twice
// ─────────────────────────────────────────────────────────────────────

#[test]
fn test_one_shot_require_local_twice_errors() {
    let key = "one-shot-rl-rl";
    let probe = make_probe(key, CallSequence::RequireLocalTwice);

    let yaml = format!(
        r#"
nodes:
  receiver:
    type: "{PROBE_RECEIVER_URN}"
    config:
      probe_key: "{key}"
    capabilities:
      no_op_stateless: "ext"
  exporter:
    type: "{NOOP_EXPORTER_URN}"

extensions:
  ext:
    type: "{PASSIVE_EXTENSION_URN}"

connections:
  - from: receiver
    to: exporter
"#
    );
    let (_runtime_pipeline, _ctx, _key, _ts) = build_test_runtime_pipeline(&yaml);

    assert_eq!(
        probe.first_call_succeeded.load(Ordering::SeqCst),
        1,
        "first require_local must succeed"
    );
    assert_eq!(
        probe.second_call_already_consumed.load(Ordering::SeqCst),
        1,
        "second require_local must error with CapabilityAlreadyConsumed"
    );
    let msg = probe
        .second_call_error_message
        .lock()
        .clone()
        .expect("second call must record an error");
    assert!(
        msg.to_ascii_lowercase().contains("already") || msg.contains("AlreadyConsumed"),
        "expected an 'already consumed' error message, got: {msg}"
    );
}

// ─────────────────────────────────────────────────────────────────────
// Test 7 — one-shot enforcement: require_local then require_shared
// ─────────────────────────────────────────────────────────────────────

#[test]
fn test_one_shot_require_local_then_require_shared_errors() {
    let key = "one-shot-rl-rs";
    let probe = make_probe(key, CallSequence::RequireLocalThenRequireShared);

    let yaml = format!(
        r#"
nodes:
  receiver:
    type: "{PROBE_RECEIVER_URN}"
    config:
      probe_key: "{key}"
    capabilities:
      no_op_stateless: "ext"
  exporter:
    type: "{NOOP_EXPORTER_URN}"

extensions:
  ext:
    type: "{PASSIVE_EXTENSION_URN}"

connections:
  - from: receiver
    to: exporter
"#
    );
    let (_runtime_pipeline, _ctx, _key, _ts) = build_test_runtime_pipeline(&yaml);

    assert_eq!(probe.first_call_succeeded.load(Ordering::SeqCst), 1);
    assert_eq!(
        probe.second_call_already_consumed.load(Ordering::SeqCst),
        1,
        "claiming local then shared on the same binding must error on the second call"
    );
}

// ─────────────────────────────────────────────────────────────────────
// Test 8 — one-shot enforcement: require_local then optional_local
// ─────────────────────────────────────────────────────────────────────

#[test]
fn test_one_shot_require_local_then_optional_local_errors() {
    let key = "one-shot-rl-ol";
    let probe = make_probe(key, CallSequence::RequireLocalThenOptionalLocal);

    let yaml = format!(
        r#"
nodes:
  receiver:
    type: "{PROBE_RECEIVER_URN}"
    config:
      probe_key: "{key}"
    capabilities:
      no_op_stateless: "ext"
  exporter:
    type: "{NOOP_EXPORTER_URN}"

extensions:
  ext:
    type: "{PASSIVE_EXTENSION_URN}"

connections:
  - from: receiver
    to: exporter
"#
    );
    let (_runtime_pipeline, _ctx, _key, _ts) = build_test_runtime_pipeline(&yaml);

    assert_eq!(probe.first_call_succeeded.load(Ordering::SeqCst), 1);
    assert_eq!(
        probe.second_call_already_consumed.load(Ordering::SeqCst),
        1,
        "optional_local after require_local on same binding must error"
    );
}

// ─────────────────────────────────────────────────────────────────────
// Test 9 — one-shot enforcement: require_local then optional_shared
// ─────────────────────────────────────────────────────────────────────

#[test]
fn test_one_shot_require_local_then_optional_shared_errors() {
    let key = "one-shot-rl-os";
    let probe = make_probe(key, CallSequence::RequireLocalThenOptionalShared);

    let yaml = format!(
        r#"
nodes:
  receiver:
    type: "{PROBE_RECEIVER_URN}"
    config:
      probe_key: "{key}"
    capabilities:
      no_op_stateless: "ext"
  exporter:
    type: "{NOOP_EXPORTER_URN}"

extensions:
  ext:
    type: "{PASSIVE_EXTENSION_URN}"

connections:
  - from: receiver
    to: exporter
"#
    );
    let (_runtime_pipeline, _ctx, _key, _ts) = build_test_runtime_pipeline(&yaml);

    assert_eq!(probe.first_call_succeeded.load(Ordering::SeqCst), 1);
    assert_eq!(
        probe.second_call_already_consumed.load(Ordering::SeqCst),
        1,
        "optional_shared after require_local on same binding must error"
    );
}

// ─────────────────────────────────────────────────────────────────────
// Test 10 — Background extension is kept (no warning) and runs even
//           though no node binds any of its capabilities (it has none).
// ─────────────────────────────────────────────────────────────────────

#[test]
fn test_background_extension_runs_without_node_bindings() {
    // The receiver claims nothing — pipeline has no capability bindings
    // anywhere. We're testing the Background lifecycle in isolation.
    let receiver_key = "bg-receiver";
    let _probe = make_probe(receiver_key, CallSequence::None);

    let bg_key = "bg-probe";
    let started = Arc::new(AtomicBool::new(false));
    let shutdown_at: Arc<parking_lot::Mutex<Option<Instant>>> =
        Arc::new(parking_lot::Mutex::new(None));
    register_background_probe(
        bg_key,
        BackgroundProbe {
            started: Arc::clone(&started),
            shutdown_at: Arc::clone(&shutdown_at),
        },
    );

    let yaml = format!(
        r#"
nodes:
  receiver:
    type: "{PROBE_RECEIVER_URN}"
    config:
      probe_key: "{receiver_key}"
  exporter:
    type: "{NOOP_EXPORTER_URN}"

extensions:
  bg:
    type: "{BACKGROUND_EXTENSION_URN}"
    config:
      probe_key: "{bg_key}"

connections:
  - from: receiver
    to: exporter
"#
    );
    let (runtime_pipeline, ctx, entity_key, ts) = build_test_runtime_pipeline(&yaml);

    let result = run_pipeline_with_shutdown_after(
        runtime_pipeline,
        ctx,
        entity_key,
        ts,
        Duration::from_millis(150),
    );
    assert!(
        result.is_ok(),
        "pipeline with a Background extension should shut down cleanly: {result:?}"
    );
    assert!(
        started.load(Ordering::SeqCst),
        "Background extension's start() must run even though no node binds it"
    );
    assert!(
        shutdown_at.lock().is_some(),
        "Background extension must observe Shutdown"
    );
}

// ─────────────────────────────────────────────────────────────────────
// Test 11 — Background extension survives pruning even when an
//           unbound *non-background* extension is dropped.
//           Two extensions in the same pipeline; receiver binds neither.
//           Asserts contrasting outcomes prove the BG special-case.
// ─────────────────────────────────────────────────────────────────────

#[test]
fn test_background_kept_while_unbound_active_dropped() {
    let receiver_key = "contrast-receiver";
    let _probe = make_probe(receiver_key, CallSequence::None);

    let bg_key = "contrast-bg";
    let bg_started = Arc::new(AtomicBool::new(false));
    let bg_shutdown_at: Arc<parking_lot::Mutex<Option<Instant>>> =
        Arc::new(parking_lot::Mutex::new(None));
    register_background_probe(
        bg_key,
        BackgroundProbe {
            started: Arc::clone(&bg_started),
            shutdown_at: Arc::clone(&bg_shutdown_at),
        },
    );

    let active_key = "contrast-active";
    let active_started = Arc::new(AtomicBool::new(false));
    let active_start_at: Arc<parking_lot::Mutex<Option<Instant>>> =
        Arc::new(parking_lot::Mutex::new(None));
    let active_shutdown_at: Arc<parking_lot::Mutex<Option<Instant>>> =
        Arc::new(parking_lot::Mutex::new(None));
    register_active_ext_probe(
        active_key,
        ActiveExtProbe {
            started: Arc::clone(&active_started),
            start_at: Arc::clone(&active_start_at),
            shutdown_at: Arc::clone(&active_shutdown_at),
        },
    );

    // Both a Background and an Active extension are declared. The
    // receiver binds NEITHER (its `capabilities:` block is omitted).
    // Expected outcomes:
    //   - Background bundle: kept (Category 1) → start() runs.
    //   - Active bundle: dropped (Category 2: defined-but-unbound) →
    //     start() never runs.
    let yaml = format!(
        r#"
nodes:
  receiver:
    type: "{PROBE_RECEIVER_URN}"
    config:
      probe_key: "{receiver_key}"
  exporter:
    type: "{NOOP_EXPORTER_URN}"

extensions:
  bg:
    type: "{BACKGROUND_EXTENSION_URN}"
    config:
      probe_key: "{bg_key}"
  active:
    type: "{ACTIVE_EXTENSION_URN}"
    config:
      probe_key: "{active_key}"

connections:
  - from: receiver
    to: exporter
"#
    );
    let (runtime_pipeline, ctx, entity_key, ts) = build_test_runtime_pipeline(&yaml);

    let result = run_pipeline_with_shutdown_after(
        runtime_pipeline,
        ctx,
        entity_key,
        ts,
        Duration::from_millis(150),
    );
    assert!(
        result.is_ok(),
        "pipeline must shut down cleanly: {result:?}"
    );

    assert!(
        bg_started.load(Ordering::SeqCst),
        "Background extension survived pruning and ran its start()"
    );
    assert!(
        bg_shutdown_at.lock().is_some(),
        "Background extension observed Shutdown"
    );

    assert!(
        !active_started.load(Ordering::SeqCst),
        "unbound Active extension must be dropped before runtime spawn — start() must not run"
    );
    assert!(
        active_start_at.lock().is_none(),
        "unbound Active extension start_at must remain None"
    );
}

// ─────────────────────────────────────────────────────────────────────
// Test 12 — Shared state: two receiver nodes binding the same
//           passive-cloned extension share an Arc<AtomicU64> counter.
//           Each consumer's `increment()` is observable to the other.
// ─────────────────────────────────────────────────────────────────────

#[test]
fn test_shared_state_across_two_nodes_via_arc() {
    let receiver_a = "shared-a";
    let receiver_b = "shared-b";
    let probe_a = make_probe(receiver_a, CallSequence::StatefulIncrement);
    let probe_b = make_probe(receiver_b, CallSequence::StatefulIncrement);

    let ext_key = "shared-counter";
    let shared_counter: Arc<AtomicU64> = Arc::new(AtomicU64::new(0));
    register_shared_counter_probe(
        ext_key,
        SharedCounterProbe {
            counter: Arc::clone(&shared_counter),
        },
    );

    let yaml = format!(
        r#"
nodes:
  receiver_a:
    type: "{PROBE_RECEIVER_URN}"
    config:
      probe_key: "{receiver_a}"
    capabilities:
      no_op_stateful: "shared-counter-ext"
  receiver_b:
    type: "{PROBE_RECEIVER_URN}"
    config:
      probe_key: "{receiver_b}"
    capabilities:
      no_op_stateful: "shared-counter-ext"
  exporter:
    type: "{NOOP_EXPORTER_URN}"

extensions:
  shared-counter-ext:
    type: "{SHARED_COUNTER_EXTENSION_URN}"
    config:
      probe_key: "{ext_key}"

connections:
  - from: receiver_a
    to: exporter
  - from: receiver_b
    to: exporter
"#
    );
    let (_runtime_pipeline, _ctx, _key, _ts) = build_test_runtime_pipeline(&yaml);

    // Both nodes claimed the capability and called `increment()`.
    assert_eq!(probe_a.first_call_succeeded.load(Ordering::SeqCst), 1);
    assert_eq!(probe_b.first_call_succeeded.load(Ordering::SeqCst), 1);
    let val_a = probe_a
        .stateful_increment_return
        .lock()
        .expect("receiver_a observed an increment return value");
    let val_b = probe_b
        .stateful_increment_return
        .lock()
        .expect("receiver_b observed an increment return value");

    // The two consumers were each handed a Box<dyn Local> via .cloned()
    // (clones of the prototype). The prototype's `Arc<AtomicU64>` is
    // shared across clones, so the two `increment()` returns must be
    // distinct values 1 and 2 (in build-order — receiver_a first,
    // receiver_b second). Plus the underlying counter is now 2.
    let mut returns = [val_a, val_b];
    returns.sort_unstable();
    assert_eq!(
        returns,
        [1, 2],
        "the two nodes must observe distinct increments reflecting shared state"
    );
    assert_eq!(
        shared_counter.load(Ordering::SeqCst),
        2,
        "underlying Arc<AtomicU64> must reflect both increments"
    );
}

// ─────────────────────────────────────────────────────────────────────
// Test 13 — `.passive().constructed()` policy: the user closure runs
//           ONCE PER CONSUMER, so two nodes binding to the same
//           constructed extension yield two closure invocations and
//           two fresh instances.
// ─────────────────────────────────────────────────────────────────────

#[test]
fn test_constructed_policy_yields_fresh_instance_per_consumer() {
    let receiver_a = "constructed-a";
    let receiver_b = "constructed-b";
    let probe_a = make_probe(receiver_a, CallSequence::Local);
    let probe_b = make_probe(receiver_b, CallSequence::Local);

    let ext_key = "constructed-probe";
    let closure_invocations: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    register_constructed_probe(
        ext_key,
        ConstructedProbe {
            closure_invocations: Arc::clone(&closure_invocations),
        },
    );

    let yaml = format!(
        r#"
nodes:
  receiver_a:
    type: "{PROBE_RECEIVER_URN}"
    config:
      probe_key: "{receiver_a}"
    capabilities:
      no_op_stateless: "constructed-ext"
  receiver_b:
    type: "{PROBE_RECEIVER_URN}"
    config:
      probe_key: "{receiver_b}"
    capabilities:
      no_op_stateless: "constructed-ext"
  exporter:
    type: "{NOOP_EXPORTER_URN}"

extensions:
  constructed-ext:
    type: "{CONSTRUCTED_EXTENSION_URN}"
    config:
      probe_key: "{ext_key}"

connections:
  - from: receiver_a
    to: exporter
  - from: receiver_b
    to: exporter
"#
    );
    let (_runtime_pipeline, _ctx, _key, _ts) = build_test_runtime_pipeline(&yaml);

    assert_eq!(probe_a.first_call_succeeded.load(Ordering::SeqCst), 1);
    assert_eq!(probe_b.first_call_succeeded.load(Ordering::SeqCst), 1);
    assert_eq!(
        closure_invocations.load(Ordering::SeqCst),
        2,
        "closure must run once per consumer (2 nodes => 2 invocations)"
    );
    assert_eq!(
        probe_a.captured_name.lock().as_deref(),
        Some("constructed-noop"),
        "receiver_a sees the constructed instance"
    );
    assert_eq!(
        probe_b.captured_name.lock().as_deref(),
        Some("constructed-noop"),
        "receiver_b sees its own constructed instance"
    );
}

// ─────────────────────────────────────────────────────────────────────
// Test 14 — one-shot enforcement: optional_local then require_local
//           Proves that a successful `optional_local` claim consumes
//           the binding's one-shot, blocking subsequent `require_*`.
// ─────────────────────────────────────────────────────────────────────

#[test]
fn test_one_shot_optional_local_then_require_local_errors() {
    let key = "one-shot-ol-rl";
    let probe = make_probe(key, CallSequence::OptionalLocalThenRequireLocal);

    let yaml = format!(
        r#"
nodes:
  receiver:
    type: "{PROBE_RECEIVER_URN}"
    config:
      probe_key: "{key}"
    capabilities:
      no_op_stateless: "ext"
  exporter:
    type: "{NOOP_EXPORTER_URN}"

extensions:
  ext:
    type: "{PASSIVE_EXTENSION_URN}"

connections:
  - from: receiver
    to: exporter
"#
    );
    let (_runtime_pipeline, _ctx, _key, _ts) = build_test_runtime_pipeline(&yaml);

    assert_eq!(probe.first_call_succeeded.load(Ordering::SeqCst), 1);
    assert_eq!(
        probe.second_call_already_consumed.load(Ordering::SeqCst),
        1,
        "require_local after optional_local on same binding must error"
    );
}

// ─────────────────────────────────────────────────────────────────────
// Test 15 — one-shot enforcement: optional_local then optional_local
//           Proves that two optional_* calls on the same binding
//           still trip the one-shot guard.
// ─────────────────────────────────────────────────────────────────────

#[test]
fn test_one_shot_optional_local_then_optional_local_errors() {
    let key = "one-shot-ol-ol";
    let probe = make_probe(key, CallSequence::OptionalLocalThenOptionalLocal);

    let yaml = format!(
        r#"
nodes:
  receiver:
    type: "{PROBE_RECEIVER_URN}"
    config:
      probe_key: "{key}"
    capabilities:
      no_op_stateless: "ext"
  exporter:
    type: "{NOOP_EXPORTER_URN}"

extensions:
  ext:
    type: "{PASSIVE_EXTENSION_URN}"

connections:
  - from: receiver
    to: exporter
"#
    );
    let (_runtime_pipeline, _ctx, _key, _ts) = build_test_runtime_pipeline(&yaml);

    assert_eq!(probe.first_call_succeeded.load(Ordering::SeqCst), 1);
    assert_eq!(
        probe.second_call_already_consumed.load(Ordering::SeqCst),
        1,
        "second optional_local on same binding must error \
         (an optional accessor that finds the binding present is \
         exactly as 'consuming' as require_*)"
    );
}

// ─────────────────────────────────────────────────────────────────────
// Test 16 — one-shot enforcement: optional_shared then require_shared
//           Mirror of test 14, exercises the shared-side accessors.
// ─────────────────────────────────────────────────────────────────────

#[test]
fn test_one_shot_optional_shared_then_require_shared_errors() {
    let key = "one-shot-os-rs";
    let probe = make_probe(key, CallSequence::OptionalSharedThenRequireShared);

    let yaml = format!(
        r#"
nodes:
  receiver:
    type: "{PROBE_RECEIVER_URN}"
    config:
      probe_key: "{key}"
    capabilities:
      no_op_stateless: "ext"
  exporter:
    type: "{NOOP_EXPORTER_URN}"

extensions:
  ext:
    type: "{PASSIVE_EXTENSION_URN}"

connections:
  - from: receiver
    to: exporter
"#
    );
    let (_runtime_pipeline, _ctx, _key, _ts) = build_test_runtime_pipeline(&yaml);

    assert_eq!(probe.first_call_succeeded.load(Ordering::SeqCst), 1);
    assert_eq!(
        probe.second_call_already_consumed.load(Ordering::SeqCst),
        1,
        "require_shared after optional_shared on same binding must error"
    );
}

// ─────────────────────────────────────────────────────────────────────
// Test 17 — one-shot enforcement: optional_shared then optional_shared
//           Mirror of test 15, exercises the shared-side accessors.
// ─────────────────────────────────────────────────────────────────────

#[test]
fn test_one_shot_optional_shared_then_optional_shared_errors() {
    let key = "one-shot-os-os";
    let probe = make_probe(key, CallSequence::OptionalSharedThenOptionalShared);

    let yaml = format!(
        r#"
nodes:
  receiver:
    type: "{PROBE_RECEIVER_URN}"
    config:
      probe_key: "{key}"
    capabilities:
      no_op_stateless: "ext"
  exporter:
    type: "{NOOP_EXPORTER_URN}"

extensions:
  ext:
    type: "{PASSIVE_EXTENSION_URN}"

connections:
  - from: receiver
    to: exporter
"#
    );
    let (_runtime_pipeline, _ctx, _key, _ts) = build_test_runtime_pipeline(&yaml);

    assert_eq!(probe.first_call_succeeded.load(Ordering::SeqCst), 1);
    assert_eq!(
        probe.second_call_already_consumed.load(Ordering::SeqCst),
        1,
        "second optional_shared on same binding must error"
    );
}

// ─────────────────────────────────────────────────────────────────────
// Lifecycle ordering — extension `start()` invoked before any node task
// ─────────────────────────────────────────────────────────────────────
//
// Asserts the framework's "extensions start first" lifecycle invariant:
// the active extension's `start()` records its entry timestamp before
// the receiver, processor, and exporter `start()` bodies record theirs.
// All three node types also consume `require_local::<NoOpStateless>()`
// at build time, validating capability resolution works for every
// node type (not just receivers).

#[test]
fn test_extension_started_before_node_tasks() {
    let recv_probe_key = "lifecycle-start-recv";
    let proc_probe_key = "lifecycle-start-proc";
    let exp_probe_key = "lifecycle-start-exp";
    let lifecycle_key = "lifecycle-start";
    let ext_probe_key = "lifecycle-start-ext";

    let recv_probe = make_probe(recv_probe_key, CallSequence::Local);
    let proc_probe = make_probe(proc_probe_key, CallSequence::None);
    let exp_probe = make_probe(exp_probe_key, CallSequence::None);

    register_node_lifecycle_probe(lifecycle_key, NodeLifecycleProbe::default());
    let lifecycle = lookup_node_lifecycle_probe(lifecycle_key);

    let ext_started = Arc::new(AtomicBool::new(false));
    let ext_start_at: Arc<parking_lot::Mutex<Option<Instant>>> =
        Arc::new(parking_lot::Mutex::new(None));
    let ext_shutdown_at: Arc<parking_lot::Mutex<Option<Instant>>> =
        Arc::new(parking_lot::Mutex::new(None));
    register_active_ext_probe(
        ext_probe_key,
        ActiveExtProbe {
            started: Arc::clone(&ext_started),
            start_at: Arc::clone(&ext_start_at),
            shutdown_at: Arc::clone(&ext_shutdown_at),
        },
    );

    let yaml = format!(
        r#"
nodes:
  receiver:
    type: "{PROBE_RECEIVER_URN}"
    config:
      probe_key: "{recv_probe_key}"
      lifecycle_key: "{lifecycle_key}"
    capabilities:
      no_op_stateless: "active-ext"
  processor:
    type: "{PROBE_PROCESSOR_URN}"
    config:
      probe_key: "{proc_probe_key}"
      lifecycle_key: "{lifecycle_key}"
    capabilities:
      no_op_stateless: "active-ext"
  exporter:
    type: "{PROBE_EXPORTER_URN}"
    config:
      probe_key: "{exp_probe_key}"
      lifecycle_key: "{lifecycle_key}"
    capabilities:
      no_op_stateless: "active-ext"

extensions:
  active-ext:
    type: "{ACTIVE_EXTENSION_URN}"
    config:
      probe_key: "{ext_probe_key}"

connections:
  - from: receiver
    to: processor
  - from: processor
    to: exporter
"#
    );

    let (runtime_pipeline, ctx, entity_key, ts) = build_test_runtime_pipeline(&yaml);

    // Capabilities resolved at build time for all three node types.
    assert_eq!(
        recv_probe.first_call_succeeded.load(Ordering::SeqCst),
        1,
        "receiver consumed require_local::<NoOpStateless>()"
    );
    assert_eq!(
        proc_probe.create_calls.load(Ordering::SeqCst),
        1,
        "processor create() ran"
    );
    assert_eq!(
        exp_probe.create_calls.load(Ordering::SeqCst),
        1,
        "exporter create() ran"
    );

    let result = run_pipeline_with_shutdown_after(
        runtime_pipeline,
        ctx,
        entity_key,
        ts,
        Duration::from_millis(150),
    );
    assert!(
        result.is_ok(),
        "pipeline should shut down cleanly: {result:?}"
    );

    let ext_start = ext_start_at.lock().expect("extension must record start_at");
    let receiver_start = lifecycle
        .receiver_start_at
        .lock()
        .expect("receiver must record start_at");
    let exporter_start = lifecycle
        .exporter_start_at
        .lock()
        .expect("exporter must record start_at");

    assert!(
        ext_start <= receiver_start,
        "extension start ({ext_start:?}) must precede receiver start ({receiver_start:?})"
    );
    assert!(
        ext_start <= exporter_start,
        "extension start ({ext_start:?}) must precede exporter start ({exporter_start:?})"
    );
    // The processor's `start()` is engine-internal and not exposed to
    // the trait impl; we observe it via its first `process()` call,
    // which can only happen after spawn. If the receiver never sends
    // pdata, this slot stays None — that's fine and we skip the assert.
    if let Some(processor_first) = *lifecycle.processor_first_call_at.lock() {
        assert!(
            ext_start <= processor_first,
            "extension start ({ext_start:?}) must precede processor first \
             process() call ({processor_first:?})"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────
// Lifecycle ordering — extension receives Shutdown after nodes drain
// ─────────────────────────────────────────────────────────────────────
//
// Asserts the framework's "extensions stop last" lifecycle invariant:
// the receiver and exporter record their `start()`-body exit
// timestamps, and the active extension records when it observed the
// `Shutdown` control message. The extension's shutdown timestamp must
// not precede either node's exit timestamp.

#[test]
fn test_extension_shutdown_received_after_nodes_exit() {
    let recv_probe_key = "lifecycle-stop-recv";
    let proc_probe_key = "lifecycle-stop-proc";
    let exp_probe_key = "lifecycle-stop-exp";
    let lifecycle_key = "lifecycle-stop";
    let ext_probe_key = "lifecycle-stop-ext";

    let _recv_probe = make_probe(recv_probe_key, CallSequence::Local);
    let _proc_probe = make_probe(proc_probe_key, CallSequence::None);
    let _exp_probe = make_probe(exp_probe_key, CallSequence::None);

    register_node_lifecycle_probe(lifecycle_key, NodeLifecycleProbe::default());
    let lifecycle = lookup_node_lifecycle_probe(lifecycle_key);

    let ext_shutdown_at: Arc<parking_lot::Mutex<Option<Instant>>> =
        Arc::new(parking_lot::Mutex::new(None));
    register_shutdown_recording_probe(
        ext_probe_key,
        ShutdownRecordingProbe {
            shutdown_at: Arc::clone(&ext_shutdown_at),
        },
    );

    let yaml = format!(
        r#"
nodes:
  receiver:
    type: "{PROBE_RECEIVER_URN}"
    config:
      probe_key: "{recv_probe_key}"
      lifecycle_key: "{lifecycle_key}"
    capabilities:
      no_op_stateless: "rec-ext"
  processor:
    type: "{PROBE_PROCESSOR_URN}"
    config:
      probe_key: "{proc_probe_key}"
      lifecycle_key: "{lifecycle_key}"
    capabilities:
      no_op_stateless: "rec-ext"
  exporter:
    type: "{PROBE_EXPORTER_URN}"
    config:
      probe_key: "{exp_probe_key}"
      lifecycle_key: "{lifecycle_key}"
    capabilities:
      no_op_stateless: "rec-ext"

extensions:
  rec-ext:
    type: "{SHUTDOWN_RECORDING_EXTENSION_URN}"
    config:
      probe_key: "{ext_probe_key}"

connections:
  - from: receiver
    to: processor
  - from: processor
    to: exporter
"#
    );

    let (runtime_pipeline, ctx, entity_key, ts) = build_test_runtime_pipeline(&yaml);
    let result = run_pipeline_with_shutdown_after(
        runtime_pipeline,
        ctx,
        entity_key,
        ts,
        Duration::from_millis(150),
    );
    assert!(
        result.is_ok(),
        "pipeline should shut down cleanly: {result:?}"
    );

    let ext_shutdown = ext_shutdown_at
        .lock()
        .expect("extension must record shutdown_at");
    let receiver_end = lifecycle
        .receiver_end_at
        .lock()
        .expect("receiver must record end_at");
    let exporter_end = lifecycle
        .exporter_end_at
        .lock()
        .expect("exporter must record end_at");

    assert!(
        ext_shutdown >= receiver_end,
        "extension shutdown ({ext_shutdown:?}) must occur at or after \
         receiver exit ({receiver_end:?})"
    );
    assert!(
        ext_shutdown >= exporter_end,
        "extension shutdown ({ext_shutdown:?}) must occur at or after \
         exporter exit ({exporter_end:?})"
    );
}

// ─────────────────────────────────────────────────────────────────────
// Optional capability — `optional_local` returns Ok(None) when no
// extension provides the capability (no provider declared in YAML).
// Validates the optional path for processor and exporter consumers.
// ─────────────────────────────────────────────────────────────────────

#[test]
fn test_optional_capability_returns_none_when_no_provider() {
    let recv_probe_key = "optional-absent-recv";
    let proc_probe_key = "optional-absent-proc";
    let exp_probe_key = "optional-absent-exp";

    // Receiver does not consume any capability (CallSequence::None)
    // because we want a pipeline with NO extension wiring at all.
    let _recv_probe = make_probe(recv_probe_key, CallSequence::None);
    let proc_probe = make_probe(proc_probe_key, CallSequence::None);
    let exp_probe = make_probe(exp_probe_key, CallSequence::None);

    let yaml = format!(
        r#"
nodes:
  receiver:
    type: "{PROBE_RECEIVER_URN}"
    config:
      probe_key: "{recv_probe_key}"
  processor:
    type: "{PROBE_PROCESSOR_URN}"
    config:
      probe_key: "{proc_probe_key}"
      optional_only: true
  exporter:
    type: "{PROBE_EXPORTER_URN}"
    config:
      probe_key: "{exp_probe_key}"
      optional_only: true

connections:
  - from: receiver
    to: processor
  - from: processor
    to: exporter
"#
    );

    let (runtime_pipeline, ctx, entity_key, ts) = build_test_runtime_pipeline(&yaml);

    // Both processor and exporter create() called optional_local with
    // no provider; both must have observed Ok(None).
    assert_eq!(
        proc_probe.first_call_succeeded.load(Ordering::SeqCst),
        1,
        "processor optional_local::<NoOpStateless>() must return Ok(None) \
         when no extension provides the capability"
    );
    assert_eq!(
        exp_probe.first_call_succeeded.load(Ordering::SeqCst),
        1,
        "exporter optional_local::<NoOpStateless>() must return Ok(None) \
         when no extension provides the capability"
    );

    let result = run_pipeline_with_shutdown_after(
        runtime_pipeline,
        ctx,
        entity_key,
        ts,
        Duration::from_millis(150),
    );
    assert!(
        result.is_ok(),
        "pipeline with no extensions should still build and run: {result:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────
// `&mut self` (sync) on the SHARED trait variant — proves the shared
// trait's `Send` bound doesn't block sync `&mut self` invocation
// through a `Box<dyn …Shared>` returned by `require_shared`.
// ─────────────────────────────────────────────────────────────────────

#[test]
fn test_shared_handle_sync_mut_self_increment() {
    let key = "shared-mut-sync";
    let probe = make_probe(key, CallSequence::SharedStatefulIncrement);

    let counter = Arc::new(AtomicU64::new(0));
    register_shared_counter_probe(
        key,
        SharedCounterProbe {
            counter: Arc::clone(&counter),
        },
    );

    let yaml = format!(
        r#"
nodes:
  receiver:
    type: "{PROBE_RECEIVER_URN}"
    config:
      probe_key: "{key}"
    capabilities:
      no_op_stateful: "shared-counter-shared"
  exporter:
    type: "{NOOP_EXPORTER_URN}"

extensions:
  shared-counter-shared:
    type: "{SHARED_COUNTER_SHARED_EXTENSION_URN}"
    config:
      probe_key: "{key}"

connections:
  - from: receiver
    to: exporter
"#
    );
    let (_runtime_pipeline, _ctx, _key, _ts) = build_test_runtime_pipeline(&yaml);

    assert_eq!(
        probe.first_call_succeeded.load(Ordering::SeqCst),
        1,
        "require_shared::<NoOpStateful>() must succeed"
    );
    assert_eq!(
        probe.stateful_increment_return.lock().as_ref().copied(),
        Some(1),
        "shared handle's `&mut self` increment() must return 1 on first call"
    );
    assert_eq!(
        counter.load(Ordering::SeqCst),
        1,
        "underlying Arc<AtomicU64> must reflect the shared-handle mutation"
    );
}

// ─────────────────────────────────────────────────────────────────────
// async `&mut self` on the LOCAL trait variant — proves the async
// codegen path is invokable through a `Box<dyn …Local>` retrieved at
// build time and awaited inside a node task.
// ─────────────────────────────────────────────────────────────────────

#[test]
fn test_local_handle_async_mut_self_record() {
    let key = "local-mut-async";
    let probe = make_probe(key, CallSequence::LocalStatefulRecordAsync);

    let counter = Arc::new(AtomicU64::new(0));
    register_shared_counter_probe(
        key,
        SharedCounterProbe {
            counter: Arc::clone(&counter),
        },
    );

    let yaml = format!(
        r#"
nodes:
  receiver:
    type: "{PROBE_RECEIVER_URN}"
    config:
      probe_key: "{key}"
    capabilities:
      no_op_stateful: "shared-counter"
  exporter:
    type: "{NOOP_EXPORTER_URN}"

extensions:
  shared-counter:
    type: "{SHARED_COUNTER_EXTENSION_URN}"
    config:
      probe_key: "{key}"

connections:
  - from: receiver
    to: exporter
"#
    );
    let (runtime_pipeline, ctx, entity_key, ts) = build_test_runtime_pipeline(&yaml);

    assert_eq!(
        probe.first_call_succeeded.load(Ordering::SeqCst),
        1,
        "require_local::<NoOpStateful>() must succeed at build time"
    );
    // `record(7).await` runs after spawn, so the return slot is empty
    // until the pipeline runs.
    assert!(probe.stateful_increment_return.lock().is_none());

    let result = run_pipeline_with_shutdown_after(
        runtime_pipeline,
        ctx,
        entity_key,
        ts,
        Duration::from_millis(150),
    );
    assert!(
        result.is_ok(),
        "pipeline should shut down cleanly: {result:?}"
    );

    assert_eq!(
        probe.stateful_increment_return.lock().as_ref().copied(),
        Some(7),
        "local handle's async `&mut self` record(7) must return 7 (counter += 7)"
    );
    assert_eq!(
        counter.load(Ordering::SeqCst),
        7,
        "underlying Arc<AtomicU64> must reflect the async record(7) mutation"
    );
}

// ─────────────────────────────────────────────────────────────────────
// async `&mut self` on the SHARED trait variant — proves the async +
// `Send` codegen path is invokable through a `Box<dyn …Shared>` and
// awaited from inside a node task.
// ─────────────────────────────────────────────────────────────────────

#[test]
fn test_shared_handle_async_mut_self_record() {
    let key = "shared-mut-async";
    let probe = make_probe(key, CallSequence::SharedStatefulRecordAsync);

    let counter = Arc::new(AtomicU64::new(0));
    register_shared_counter_probe(
        key,
        SharedCounterProbe {
            counter: Arc::clone(&counter),
        },
    );

    let yaml = format!(
        r#"
nodes:
  receiver:
    type: "{PROBE_RECEIVER_URN}"
    config:
      probe_key: "{key}"
    capabilities:
      no_op_stateful: "shared-counter-shared"
  exporter:
    type: "{NOOP_EXPORTER_URN}"

extensions:
  shared-counter-shared:
    type: "{SHARED_COUNTER_SHARED_EXTENSION_URN}"
    config:
      probe_key: "{key}"

connections:
  - from: receiver
    to: exporter
"#
    );
    let (runtime_pipeline, ctx, entity_key, ts) = build_test_runtime_pipeline(&yaml);

    assert_eq!(
        probe.first_call_succeeded.load(Ordering::SeqCst),
        1,
        "require_shared::<NoOpStateful>() must succeed at build time"
    );
    assert!(probe.stateful_increment_return.lock().is_none());

    let result = run_pipeline_with_shutdown_after(
        runtime_pipeline,
        ctx,
        entity_key,
        ts,
        Duration::from_millis(150),
    );
    assert!(
        result.is_ok(),
        "pipeline should shut down cleanly: {result:?}"
    );

    assert_eq!(
        probe.stateful_increment_return.lock().as_ref().copied(),
        Some(11),
        "shared handle's async `&mut self` record(11) must return 11 (counter += 11)"
    );
    assert_eq!(
        counter.load(Ordering::SeqCst),
        11,
        "underlying Arc<AtomicU64> must reflect the async record(11) mutation"
    );
}

// ─────────────────────────────────────────────────────────────────────
// Cross-node shared state via `Rc<RefCell<...>>` on the LOCAL trait
// variant — proves that an `Rc`-wrapped field on a `.passive().cloned()`
// impl is observably shared across multiple consumers (clones bump
// the Rc refcount and point at the same RefCell).
// ─────────────────────────────────────────────────────────────────────

#[test]
fn test_local_shared_state_across_two_nodes_via_rc() {
    let receiver_a = "rc-shared-a";
    let receiver_b = "rc-shared-b";
    let probe_a = make_probe(receiver_a, CallSequence::StatefulIncrement);
    let probe_b = make_probe(receiver_b, CallSequence::StatefulIncrement);

    let yaml = format!(
        r#"
nodes:
  receiver_a:
    type: "{PROBE_RECEIVER_URN}"
    config:
      probe_key: "{receiver_a}"
    capabilities:
      no_op_stateful: "rc-counter-ext"
  receiver_b:
    type: "{PROBE_RECEIVER_URN}"
    config:
      probe_key: "{receiver_b}"
    capabilities:
      no_op_stateful: "rc-counter-ext"
  exporter:
    type: "{NOOP_EXPORTER_URN}"

extensions:
  rc-counter-ext:
    type: "{RC_COUNTER_EXTENSION_URN}"

connections:
  - from: receiver_a
    to: exporter
  - from: receiver_b
    to: exporter
"#
    );
    let (_runtime_pipeline, _ctx, _key, _ts) = build_test_runtime_pipeline(&yaml);

    assert_eq!(probe_a.first_call_succeeded.load(Ordering::SeqCst), 1);
    assert_eq!(probe_b.first_call_succeeded.load(Ordering::SeqCst), 1);
    let val_a = probe_a
        .stateful_increment_return
        .lock()
        .expect("receiver_a observed an increment return value");
    let val_b = probe_b
        .stateful_increment_return
        .lock()
        .expect("receiver_b observed an increment return value");

    // The two consumers each got a `Clone` of the prototype. The
    // prototype's `Rc<RefCell<u64>>` is shared via Rc::clone, so the
    // two `increment()` calls observe distinct values 1 and 2.
    let mut returns = [val_a, val_b];
    returns.sort_unstable();
    assert_eq!(
        returns,
        [1, 2],
        "Rc<RefCell<u64>>-backed local impl must yield distinct increments \
         reflecting shared state across the two consumers"
    );
}

// ─────────────────────────────────────────────────────────────────────
// Cross-node shared state via `Arc<AtomicU64>` on the SHARED trait
// variant — same idea as the existing local-Arc multi-node test, but
// goes through the `require_shared` path with the `Send` trait
// variant.
// ─────────────────────────────────────────────────────────────────────

#[test]
fn test_shared_state_across_two_nodes_via_shared_arc() {
    let receiver_a = "shared-arc-a";
    let receiver_b = "shared-arc-b";
    let probe_a = make_probe(receiver_a, CallSequence::SharedStatefulIncrement);
    let probe_b = make_probe(receiver_b, CallSequence::SharedStatefulIncrement);

    let ext_key = "shared-arc-ext";
    let shared_counter: Arc<AtomicU64> = Arc::new(AtomicU64::new(0));
    register_shared_counter_probe(
        ext_key,
        SharedCounterProbe {
            counter: Arc::clone(&shared_counter),
        },
    );

    let yaml = format!(
        r#"
nodes:
  receiver_a:
    type: "{PROBE_RECEIVER_URN}"
    config:
      probe_key: "{receiver_a}"
    capabilities:
      no_op_stateful: "shared-arc-ext"
  receiver_b:
    type: "{PROBE_RECEIVER_URN}"
    config:
      probe_key: "{receiver_b}"
    capabilities:
      no_op_stateful: "shared-arc-ext"
  exporter:
    type: "{NOOP_EXPORTER_URN}"

extensions:
  shared-arc-ext:
    type: "{SHARED_COUNTER_SHARED_EXTENSION_URN}"
    config:
      probe_key: "{ext_key}"

connections:
  - from: receiver_a
    to: exporter
  - from: receiver_b
    to: exporter
"#
    );
    let (_runtime_pipeline, _ctx, _key, _ts) = build_test_runtime_pipeline(&yaml);

    assert_eq!(probe_a.first_call_succeeded.load(Ordering::SeqCst), 1);
    assert_eq!(probe_b.first_call_succeeded.load(Ordering::SeqCst), 1);
    let val_a = probe_a
        .stateful_increment_return
        .lock()
        .expect("receiver_a observed an increment return value");
    let val_b = probe_b
        .stateful_increment_return
        .lock()
        .expect("receiver_b observed an increment return value");

    let mut returns = [val_a, val_b];
    returns.sort_unstable();
    assert_eq!(
        returns,
        [1, 2],
        "Arc<AtomicU64>-backed shared impl must yield distinct increments \
         reflecting shared state across the two consumers via require_shared"
    );
    assert_eq!(
        shared_counter.load(Ordering::SeqCst),
        2,
        "underlying Arc<AtomicU64> must reflect both shared-handle increments"
    );
}

// ─────────────────────────────────────────────────────────────────────
// Active extension mutates shared `Arc<AtomicU64>` state during its
// `start()` task — proves capability consumers observe pre-mutation
// state at build time and can read the post-mutation state through
// the same `Arc` after the run. The extension's impl provides BOTH
// the lifecycle and the `NoOpStateful` capability, so writes from
// `start()` and reads via the capability go through one Arc.
// ─────────────────────────────────────────────────────────────────────

#[test]
fn test_active_extension_mutates_shared_arc_observed_by_consumers() {
    let receiver_key = "active-arc-recv";
    let probe = make_probe(receiver_key, CallSequence::SharedStatefulReadCount);

    let ext_key = "active-shared-counter";
    let counter: Arc<AtomicU64> = Arc::new(AtomicU64::new(0));
    register_shared_counter_probe(
        ext_key,
        SharedCounterProbe {
            counter: Arc::clone(&counter),
        },
    );

    let bumps: u64 = 3;
    let yaml = format!(
        r#"
nodes:
  receiver:
    type: "{PROBE_RECEIVER_URN}"
    config:
      probe_key: "{receiver_key}"
    capabilities:
      no_op_stateful: "active-counter-ext"
  exporter:
    type: "{NOOP_EXPORTER_URN}"

extensions:
  active-counter-ext:
    type: "{ACTIVE_SHARED_COUNTER_EXTENSION_URN}"
    config:
      probe_key: "{ext_key}"
      bumps: {bumps}

connections:
  - from: receiver
    to: exporter
"#
    );
    let (runtime_pipeline, ctx, entity_key, ts) = build_test_runtime_pipeline(&yaml);

    // At build time (before run_forever spawns anything), the receiver
    // acquired the capability and read `count()`. The active extension
    // hasn't run yet, so the value must be 0.
    assert_eq!(
        probe.first_call_succeeded.load(Ordering::SeqCst),
        1,
        "require_shared::<NoOpStateful>() must succeed at build time"
    );
    assert_eq!(
        probe.stateful_increment_return.lock().as_ref().copied(),
        Some(0),
        "build-time count() must observe pre-mutation state"
    );
    assert_eq!(
        counter.load(Ordering::SeqCst),
        0,
        "underlying Arc<AtomicU64> must be 0 before pipeline runs"
    );

    let result = run_pipeline_with_shutdown_after(
        runtime_pipeline,
        ctx,
        entity_key,
        ts,
        Duration::from_millis(150),
    );
    assert!(
        result.is_ok(),
        "pipeline should shut down cleanly: {result:?}"
    );

    // After shutdown, the active extension's start() task has bumped
    // the Arc-shared counter exactly `bumps` times. Capability consumers
    // share that same Arc (via the registry-handed-out clone of the
    // impl), so reads through the capability would observe the same
    // value — here we verify directly through the probe's Arc handle.
    assert_eq!(
        counter.load(Ordering::SeqCst),
        bumps,
        "active extension's start() must have mutated the shared Arc<AtomicU64> \
         exactly `bumps` times, observable to capability consumers"
    );
}
