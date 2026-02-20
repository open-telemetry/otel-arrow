// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! A test exporter that counts received items.
//!
//! Each exporter instance is identified by a unique ID passed in the node config.
//! Counters are registered with `register_counter(id, counter)` before pipeline
//! creation and looked up by the exporter factory using the ID from config.
//!
//! This design avoids global state issues when tests run in parallel.

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
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicU64, Ordering};

/// URN for the counting exporter (ACKs and counts items).
pub const COUNTING_EXPORTER_URN: &str = "urn:otel:counting:exporter";

/// Registry of counters keyed by unique test/pipeline ID.
static COUNTER_REGISTRY: LazyLock<Mutex<HashMap<String, Arc<AtomicU64>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Register a counter for a specific test/pipeline ID.
///
/// Call this before building the pipeline. The ID should match the `counter_id`
/// field in the exporter's node config.
pub fn register_counter(id: impl Into<String>, counter: Arc<AtomicU64>) {
    let _ = COUNTER_REGISTRY.lock().insert(id.into(), counter);
}

/// Unregister a counter after the test completes.
pub fn unregister_counter(id: &str) {
    let _ = COUNTER_REGISTRY.lock().remove(id);
}

/// Get a counter by ID.
fn get_counter(id: &str) -> Option<Arc<AtomicU64>> {
    COUNTER_REGISTRY.lock().get(id).cloned()
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
        // Look up counter by ID from node config
        let counter_id = node_config
            .config
            .get("counter_id")
            .and_then(|v| v.as_str());
        let counter = counter_id.and_then(get_counter);
        Ok(ExporterWrapper::local(
            CountingExporter { counter },
            node,
            node_config,
            exporter_config,
        ))
    },
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: |_| Ok(()),
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
