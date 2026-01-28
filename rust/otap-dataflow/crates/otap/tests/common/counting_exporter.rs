// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! A test exporter that counts received items.
//!
//! Uses a global atomic counter that can be set before pipeline runs and
//! read after to verify data flow.

use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::config::ExporterConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{AckMsg, NodeControlMsg};
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
use std::sync::atomic::{AtomicU64, Ordering};

/// URN for the counting exporter (ACKs and counts items).
pub const COUNTING_EXPORTER_URN: &str = "urn:otel:counting:exporter";

/// Global counter storage. Protected by mutex to allow swapping the counter
/// between test runs.
static COUNTER: Mutex<Option<Arc<AtomicU64>>> = Mutex::new(None);

/// Set the counter for counting exporter instances.
pub fn set_counter(counter: Arc<AtomicU64>) {
    *COUNTER.lock() = Some(counter);
}

/// Clear the counter.
pub fn clear_counter() {
    *COUNTER.lock() = None;
}

/// Get the current counter (cloned Arc).
fn get_counter() -> Option<Arc<AtomicU64>> {
    COUNTER.lock().clone()
}

struct CountingExporter {
    counter: Option<Arc<AtomicU64>>,
}

#[allow(unsafe_code)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
static COUNTING_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: COUNTING_EXPORTER_URN,
    create: |_pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             exporter_config: &ExporterConfig| {
        let counter = get_counter();
        Ok(ExporterWrapper::local(
            CountingExporter { counter },
            node,
            node_config,
            exporter_config,
        ))
    },
};

#[async_trait(?Send)]
impl Exporter<OtapPdata> for CountingExporter {
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
                    let items = data.num_items() as u64;
                    // Count items before ACKing
                    if let Some(ref counter) = self.counter {
                        let _ = counter.fetch_add(items, Ordering::Relaxed);
                    }
                    effect_handler.notify_ack(AckMsg::new(data)).await?;
                }
                _ => {}
            }
        }
        Ok(TerminalState::default())
    }
}
