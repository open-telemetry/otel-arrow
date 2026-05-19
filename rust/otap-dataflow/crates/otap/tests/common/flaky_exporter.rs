// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! A test exporter that NACKs until switched to ACK mode.
//!
//! Each exporter instance is identified by a unique ID passed in the node config.
//! State is registered with `register_state(id, ...)` before pipeline creation
//! and looked up by the exporter factory using the ID from config.
//!
//! Two coordination styles are supported:
//!
//! 1. **External flip (manual)** — register with [`register_state`] (or
//!    [`register_state_with_auto_flip`] passing `0`) and later call
//!    [`set_should_ack_by_id`] from the test thread. Required when the test
//!    needs to interleave permanent/transient NACK phases.
//! 2. **Auto-flip (deterministic)** — register with
//!    [`register_state_with_auto_flip(id, counter, nack_first_n)`] where
//!    `nack_first_n > 0`. The exporter sends exactly `nack_first_n` NACKs and
//!    then atomically flips itself to ACK mode in the same task that handles
//!    the inbound PData. This removes any cross-thread timing race between the
//!    test thread polling for a NACK and the pipeline actually producing one.
//!
//! This design avoids global state issues when tests run in parallel.

use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::config::ExporterConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{AckMsg, NackMsg, NodeControlMsg};
use otap_df_engine::error::Error;
use otap_df_engine::exporter::ExporterWrapper;
use otap_df_engine::local::exporter::{EffectHandler, Exporter};
use otap_df_engine::message::{ExporterInbox, Message};
use otap_df_engine::node::NodeId;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_engine::{ConsumerEffectHandlerExtension, ExporterFactory};
use otap_df_otap::OTAP_EXPORTER_FACTORIES;
use otap_df_otap::pdata::OtapPdata;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// URN for the flaky exporter (NACKs until switched to ACK mode).
pub const FLAKY_EXPORTER_URN: &str = "urn:otel:exporter:flaky";

/// State for a single flaky exporter instance.
struct FlakyState {
    should_ack: Arc<AtomicBool>,
    counter: Arc<AtomicU64>,
    nack_count: Arc<AtomicU64>,
    /// When true, NACKs are sent as permanent (non-retryable).
    permanent_nack: Arc<AtomicBool>,
    /// Count of permanent NACKs sent.
    permanent_nack_count: Arc<AtomicU64>,
    /// If non-zero, the exporter atomically flips `should_ack` to true once it
    /// has sent at least this many transient NACKs, making the flip
    /// deterministic instead of requiring an external coordinating thread.
    /// A value of `0` disables auto-flip (callers must call
    /// [`set_should_ack_by_id`] explicitly).
    auto_ack_after_nacks: Arc<AtomicU64>,
}

/// Registry of flaky exporter states keyed by unique test/pipeline ID.
static STATE_REGISTRY: LazyLock<Mutex<HashMap<String, FlakyState>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Register state for a specific test/pipeline ID.
///
/// Call this before building the pipeline. The ID should match the `flaky_id`
/// field in the exporter's node config.
pub fn register_state(id: impl Into<String>, counter: Arc<AtomicU64>, should_ack: bool) {
    register_state_with_auto_flip(id, counter, should_ack, 0);
}

/// Register state with a deterministic auto-flip threshold.
///
/// If `nack_first_n > 0`, the exporter sends NACKs until the combined count of
/// transient and permanent NACKs reaches `nack_first_n`, at which point it
/// atomically flips itself to ACK mode in the same task that processes
/// incoming PData. This eliminates the need for a separate flip thread polling
/// [`nack_count_by_id`]/[`permanent_nack_count_by_id`] and avoids any
/// cross-thread timing race between detection of the first NACK and the retry
/// processor's elapsed-time budget. The check fires after either NACK branch,
/// so tests that run the exporter in permanent-only mode also benefit.
///
/// `nack_first_n == 0` is equivalent to [`register_state`] (manual flip mode).
pub fn register_state_with_auto_flip(
    id: impl Into<String>,
    counter: Arc<AtomicU64>,
    should_ack: bool,
    nack_first_n: u64,
) {
    let state = FlakyState {
        should_ack: Arc::new(AtomicBool::new(should_ack)),
        counter,
        nack_count: Arc::new(AtomicU64::new(0)),
        permanent_nack: Arc::new(AtomicBool::new(false)),
        permanent_nack_count: Arc::new(AtomicU64::new(0)),
        auto_ack_after_nacks: Arc::new(AtomicU64::new(nack_first_n)),
    };
    let _ = STATE_REGISTRY.lock().insert(id.into(), state);
}

/// Unregister state after the test completes.
pub fn unregister_state(id: &str) {
    let _ = STATE_REGISTRY.lock().remove(id);
}

/// Switch the exporter between ACK and NACK mode for a specific ID.
pub fn set_should_ack_by_id(id: &str, ack: bool) {
    if let Some(state) = STATE_REGISTRY.lock().get(id) {
        state.should_ack.store(ack, Ordering::SeqCst);
    }
}

/// Get the number of NACKs for a specific ID.
#[must_use]
pub fn nack_count_by_id(id: &str) -> u64 {
    STATE_REGISTRY
        .lock()
        .get(id)
        .map(|s| s.nack_count.load(Ordering::Relaxed))
        .unwrap_or(0)
}

/// Switch the exporter to send permanent (non-retryable) NACKs.
pub fn set_permanent_nack_by_id(id: &str, permanent: bool) {
    if let Some(state) = STATE_REGISTRY.lock().get(id) {
        state.permanent_nack.store(permanent, Ordering::SeqCst);
    }
}

/// Get the number of permanent NACKs for a specific ID.
#[must_use]
pub fn permanent_nack_count_by_id(id: &str) -> u64 {
    STATE_REGISTRY
        .lock()
        .get(id)
        .map(|s| s.permanent_nack_count.load(Ordering::Relaxed))
        .unwrap_or(0)
}

/// Get state by ID - returns cloned Arcs for the exporter.
fn get_state(
    id: &str,
) -> Option<(
    Arc<AtomicU64>,
    Arc<AtomicBool>,
    Arc<AtomicU64>,
    Arc<AtomicBool>,
    Arc<AtomicU64>,
    Arc<AtomicU64>,
)> {
    STATE_REGISTRY.lock().get(id).map(|s| {
        (
            s.counter.clone(),
            s.should_ack.clone(),
            s.nack_count.clone(),
            s.permanent_nack.clone(),
            s.permanent_nack_count.clone(),
            s.auto_ack_after_nacks.clone(),
        )
    })
}

struct FlakyExporter {
    counter: Option<Arc<AtomicU64>>,
    should_ack: Option<Arc<AtomicBool>>,
    nack_count: Option<Arc<AtomicU64>>,
    permanent_nack: Option<Arc<AtomicBool>>,
    permanent_nack_count: Option<Arc<AtomicU64>>,
    auto_ack_after_nacks: Option<Arc<AtomicU64>>,
}

#[allow(unsafe_code)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
static FLAKY_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: FLAKY_EXPORTER_URN,
    create: |_pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             exporter_config: &ExporterConfig| {
        // Look up state by ID from node config
        let flaky_id = node_config.config.get("flaky_id").and_then(|v| v.as_str());
        let (counter, should_ack, nack_count, permanent_nack, permanent_nack_count, auto_ack) =
            flaky_id
                .and_then(get_state)
                .map(|(c, a, n, p, pc, aa)| {
                    (Some(c), Some(a), Some(n), Some(p), Some(pc), Some(aa))
                })
                .unwrap_or((None, None, None, None, None, None));
        Ok(ExporterWrapper::local(
            FlakyExporter {
                counter,
                should_ack,
                nack_count,
                permanent_nack,
                permanent_nack_count,
                auto_ack_after_nacks: auto_ack,
            },
            node,
            node_config,
            exporter_config,
        ))
    },
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: |_| Ok(()),
};

#[async_trait(?Send)]
impl Exporter<OtapPdata> for FlakyExporter {
    async fn start(
        self: Box<Self>,
        mut msg_chan: ExporterInbox<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        loop {
            match msg_chan.recv().await? {
                Message::Control(NodeControlMsg::Shutdown { .. }) => {
                    break;
                }
                Message::PData(data) => {
                    let should_ack = self
                        .should_ack
                        .as_ref()
                        .map(|f| f.load(Ordering::SeqCst))
                        .unwrap_or(false);

                    // The counters are used by tests for shutdown triggers,
                    // so we need to ensure the ACK/NACK is queued before observers
                    // see the counters change.
                    if should_ack {
                        // ACK mode: acknowledge first, then count.
                        let items = data.num_items() as u64;
                        effect_handler.notify_ack(AckMsg::new(data)).await?;
                        if let Some(ref counter) = self.counter {
                            let _ = counter.fetch_add(items, Ordering::Relaxed);
                        }
                    } else {
                        // NACK mode: check if permanent or transient
                        let is_permanent = self
                            .permanent_nack
                            .as_ref()
                            .map(|f| f.load(Ordering::SeqCst))
                            .unwrap_or(false);

                        if is_permanent {
                            effect_handler
                                .notify_nack(NackMsg::new_permanent(
                                    "simulated permanent failure",
                                    data,
                                ))
                                .await?;
                            if let Some(ref pc) = self.permanent_nack_count {
                                let _ = pc.fetch_add(1, Ordering::Release);
                            }
                        } else {
                            effect_handler
                                .notify_nack(NackMsg::new("simulated transient failure", data))
                                .await?;
                            if let Some(ref nack_count) = self.nack_count {
                                let _ = nack_count.fetch_add(1, Ordering::Release);
                            }
                        }

                        // Deterministic self-flip: once we've sent the
                        // configured number of NACKs (transient + permanent
                        // combined), atomically switch to ACK mode in the same
                        // task that handles inbound PData. This makes recovery
                        // data-driven instead of relying on an external thread
                        // polling and racing with the retry processor's
                        // elapsed-time budget. The check covers both NACK
                        // branches so tests configured for permanent-only NACK
                        // mode also benefit.
                        if let Some(ref auto) = self.auto_ack_after_nacks {
                            let threshold = auto.load(Ordering::Relaxed);
                            if threshold > 0 {
                                let transient = self
                                    .nack_count
                                    .as_ref()
                                    .map(|n| n.load(Ordering::Acquire))
                                    .unwrap_or(0);
                                let permanent = self
                                    .permanent_nack_count
                                    .as_ref()
                                    .map(|n| n.load(Ordering::Acquire))
                                    .unwrap_or(0);
                                if transient.saturating_add(permanent) >= threshold {
                                    if let Some(ref should_ack) = self.should_ack {
                                        should_ack.store(true, Ordering::SeqCst);
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(TerminalState::default())
    }
}
