// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! A test exporter that NACKs until switched to ACK mode.
//!
//! Each exporter instance is identified by a unique ID passed in the node config.
//! State is registered with `register_state(id, ...)` before pipeline creation
//! and looked up by the exporter factory using the ID from config.
//!
//! This allows testing retry behavior within a single pipeline run:
//! 1. Register state with `register_state(id, counter, false)` - exporter NACKs everything
//! 2. Later call `set_should_ack(id, true)` - exporter starts ACKing
//! 3. Verify that retried data eventually gets delivered
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
use otap_df_engine::message::{Message, MessageChannel};
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
pub const FLAKY_EXPORTER_URN: &str = "urn:otel:flaky:exporter";

/// State for a single flaky exporter instance.
struct FlakyState {
    should_ack: Arc<AtomicBool>,
    counter: Arc<AtomicU64>,
    nack_count: Arc<AtomicU64>,
}

/// Registry of flaky exporter states keyed by unique test/pipeline ID.
static STATE_REGISTRY: LazyLock<Mutex<HashMap<String, FlakyState>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Register state for a specific test/pipeline ID.
///
/// Call this before building the pipeline. The ID should match the `flaky_id`
/// field in the exporter's node config.
pub fn register_state(id: impl Into<String>, counter: Arc<AtomicU64>, should_ack: bool) {
    let state = FlakyState {
        should_ack: Arc::new(AtomicBool::new(should_ack)),
        counter,
        nack_count: Arc::new(AtomicU64::new(0)),
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

/// Get state by ID - returns cloned Arcs for the exporter.
fn get_state(id: &str) -> Option<(Arc<AtomicU64>, Arc<AtomicBool>, Arc<AtomicU64>)> {
    STATE_REGISTRY.lock().get(id).map(|s| {
        (
            s.counter.clone(),
            s.should_ack.clone(),
            s.nack_count.clone(),
        )
    })
}

struct FlakyExporter {
    counter: Option<Arc<AtomicU64>>,
    should_ack: Option<Arc<AtomicBool>>,
    nack_count: Option<Arc<AtomicU64>>,
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
        let (counter, should_ack, nack_count) = flaky_id
            .and_then(get_state)
            .map(|(c, a, n)| (Some(c), Some(a), Some(n)))
            .unwrap_or((None, None, None));
        Ok(ExporterWrapper::local(
            FlakyExporter {
                counter,
                should_ack,
                nack_count,
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
        mut msg_chan: MessageChannel<OtapPdata>,
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

                    if should_ack {
                        // ACK mode: count and acknowledge
                        let items = data.num_items() as u64;
                        if let Some(ref counter) = self.counter {
                            let _ = counter.fetch_add(items, Ordering::Relaxed);
                        }
                        effect_handler.notify_ack(AckMsg::new(data)).await?;
                    } else {
                        // NACK mode: reject
                        if let Some(ref nack_count) = self.nack_count {
                            let _ = nack_count.fetch_add(1, Ordering::Relaxed);
                        }
                        effect_handler
                            .notify_nack(NackMsg::new("simulated failure", data))
                            .await?;
                    }
                }
                _ => {}
            }
        }
        Ok(TerminalState::default())
    }
}
