// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! A test exporter that NACKs until switched to ACK mode.
//!
//! This allows testing retry behavior within a single pipeline run:
//! 1. Start with `configure(counter, false)` - exporter NACKs everything
//! 2. Later call `set_should_ack(true)` - exporter starts ACKing
//! 3. Verify that retried data eventually gets delivered
//!
//! Also counts delivered items for validation.

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
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// URN for the flaky exporter (NACKs until switched to ACK mode).
pub const FLAKY_EXPORTER_URN: &str = "urn:otel:flaky:exporter";

/// Global state for the flaky exporter.
struct FlakyState {
    should_ack: Option<Arc<AtomicBool>>,
    counter: Option<Arc<AtomicU64>>,
    nack_count: Option<Arc<AtomicU64>>,
}

static STATE: Mutex<FlakyState> = Mutex::new(FlakyState {
    should_ack: None,
    counter: None,
    nack_count: None,
});

/// Configure the flaky exporter for a test run.
///
/// # Arguments
/// * `counter` - Atomic counter to track delivered items
/// * `should_ack` - Initial mode: `false` = NACK everything, `true` = ACK everything
pub fn configure(counter: Arc<AtomicU64>, should_ack: bool) {
    let mut state = STATE.lock();
    state.counter = Some(counter);
    state.should_ack = Some(Arc::new(AtomicBool::new(should_ack)));
    state.nack_count = Some(Arc::new(AtomicU64::new(0)));
}

/// Switch the exporter between ACK and NACK mode.
///
/// Can be called during pipeline execution to simulate downstream recovery.
pub fn set_should_ack(ack: bool) {
    if let Some(ref flag) = STATE.lock().should_ack {
        flag.store(ack, Ordering::SeqCst);
    }
}

/// Get the number of NACKs sent since last `configure()` or `clear()`.
#[must_use]
pub fn nack_count() -> u64 {
    STATE
        .lock()
        .nack_count
        .as_ref()
        .map(|c| c.load(Ordering::Relaxed))
        .unwrap_or(0)
}

/// Clear the flaky exporter state.
///
/// Should be called after each test to reset state.
pub fn clear() {
    let mut state = STATE.lock();
    state.counter = None;
    state.should_ack = None;
    state.nack_count = None;
}

fn get_state() -> (
    Option<Arc<AtomicU64>>,
    Option<Arc<AtomicBool>>,
    Option<Arc<AtomicU64>>,
) {
    let state = STATE.lock();
    (
        state.counter.clone(),
        state.should_ack.clone(),
        state.nack_count.clone(),
    )
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
        let (counter, should_ack, nack_count) = get_state();
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
