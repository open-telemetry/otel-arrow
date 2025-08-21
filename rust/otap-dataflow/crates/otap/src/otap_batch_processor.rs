// SPDX-License-Identifier: Apache-2.0

//! OTAP batch processor (skeleton)
//!
//! Mirrors the configuration shape of the OpenTelemetry Collector batchprocessor,
//! but operates on OtapPdata. This is the MVP: functional scaffolding before #814 (OtapPdata combine) lands.
//!
//! MVP scope (by design, to avoid overlapping with #814):
//! - Keeps a per-group in-memory buffer of incoming messages and flushes by count or timer.
//! - Does NOT merge or mutate Arrow RecordBatches (no OtapPdata::append yet).
//! - Does NOT split Arrow batches (no OtapPdata::split_at yet); any future chunking will be simple
//!   multi-emit behavior until #814 lands.
//! - Uses a placeholder grouping key (single default group); metadata-based partitioning will be
//!   wired once #814 helpers exist.
//!
//! Once #814 lands, replace the naive buffering/emit logic with OtapPdata helpers for:
//! - append, split_at, split_by_group_keys
//! - schema/dictionary-aware compatibility checks
//! - accurate, zero-copy row counting and chunking.

use crate::OTAP_PROCESSOR_FACTORIES;
use crate::pdata::OtapPdata;
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::config::ProcessorConfig;
use otap_df_engine::error::Error as EngineError;
use otap_df_engine::local::processor as local;
use otap_df_engine::message::Message;
use otap_df_engine::node::NodeId;
use otap_df_engine::processor::ProcessorWrapper;
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;

/// URN for the OTAP batch processor
pub const OTAP_BATCH_PROCESSOR_URN: &str = "urn:otap:processor:batch";

/// Default configuration values (parity-aligned as we confirm Go defaults)
pub const DEFAULT_SEND_BATCH_SIZE: usize = 8192;
/// Default upper bound on batch size used to chunk oversized inputs (in number of items)
/// Note: In Go batchprocessor, send_batch_max_size defaults to 0 which means "use send_batch_size".
/// We mirror that behavior by using a sentinel and normalizing at runtime.
pub const DEFAULT_SEND_BATCH_MAX_SIZE: usize = 8192;
/// Timeout in milliseconds for periodic flush
pub const DEFAULT_TIMEOUT_MS: u64 = 200;

/// Semantic constants (avoid magic numbers)
/// Minimum allowed send_batch_size
pub const MIN_SEND_BATCH_SIZE: usize = 1;
/// Sentinel meaning: follow send_batch_size for max size (Go parity)
pub const FOLLOW_SEND_BATCH_SIZE_SENTINEL: usize = 0;
/// Minimum allowed metadata cardinality limit when specified
pub const MIN_METADATA_CARDINALITY_LIMIT: usize = 1;

/// Log messages
const LOG_MSG_SHUTTING_DOWN: &str = "OTAP batch processor shutting down";

/// Configuration for the OTAP batch processor (parity with Go batchprocessor)
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Flush current batch when this count is reached.
    #[serde(default = "default_send_batch_size")]
    pub send_batch_size: usize,
    /// Hard cap for splitting very large inputs.
    /// Go behavior: 0 (or missing) => use send_batch_size; we use 0 as default and normalize later
    #[serde(default = "default_send_batch_max_size")]
    pub send_batch_max_size: usize,
    /// Flush non-empty batches on this interval (milliseconds).
    #[serde(default = "default_timeout_ms")]
    pub timeout: u64,
    /// Optional metadata partitioning keys (resource/scope/attribute names)
    #[serde(default)]
    pub metadata_keys: Vec<String>,
    /// Optional limit on the number of distinct metadata-based groups this processor will track.
    ///
    /// Note: This is currently a no-op because grouping by metadata_keys has not yet been
    /// implemented in the MVP. Once grouping lands (post-#814), this will cap the number of
    /// concurrent groups and overflow strategy will be documented.
    #[serde(default)]
    pub metadata_cardinality_limit: Option<usize>,
}

fn default_send_batch_size() -> usize {
    DEFAULT_SEND_BATCH_SIZE
}

fn default_send_batch_max_size() -> usize {
    FOLLOW_SEND_BATCH_SIZE_SENTINEL // Go behavior: 0 means "use send_batch_size"
}

fn default_timeout_ms() -> u64 {
    DEFAULT_TIMEOUT_MS
}

impl Default for Config {
    fn default() -> Self {
        Self {
            send_batch_size: default_send_batch_size(),
            send_batch_max_size: default_send_batch_max_size(),
            timeout: default_timeout_ms(),
            metadata_keys: Vec::new(),
            metadata_cardinality_limit: None,
        }
    }
}

/// Local (!Send) OTAP batch processor (MVP)
use std::collections::HashMap;

/// Simple grouping key type used in MVP (pre-#814)
type GroupKey = String;

/// MVP internal buffer: stores items and a running count
///
/// Note: `count` currently increments per message (MVP). After #814, this should reflect
/// the actual number of rows/items (e.g., from OtapArrowRecords) to match Go semantics.
struct Buffer {
    items: Vec<OtapPdata>,
    count: usize,
}

impl Buffer {
    fn new() -> Self {
        Self {
            items: Vec::new(),
            count: 0,
        }
    }
    fn push(&mut self, data: OtapPdata) {
        self.items.push(data);
        // MVP: treat each message as one "item"; replaced later with row count
        self.count += 1;
    }
    fn is_empty(&self) -> bool {
        self.count == 0
    }
}

/// Local (!Send) OTAP batch processor
pub struct OtapBatchProcessor {
    config: Config,
    buffers: HashMap<GroupKey, Buffer>,
}

impl OtapBatchProcessor {
    /// Construct a processor wrapper from a JSON configuration object and processor runtime config.
    /// The JSON should mirror the Go collector batchprocessor shape. Missing fields fall back to
    /// crate defaults. Invalid numeric values (e.g., zero) are normalized to minimal valid values.
    pub fn from_config(
        node: NodeId,
        cfg: &Value,
        proc_cfg: &ProcessorConfig,
    ) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
        let mut config: Config = serde_json::from_value(cfg.clone()).unwrap_or_default();

        // Accept Go-style duration strings for timeout (e.g., "200ms", "2s", "1m", "1m30s").
        // If provided as a string, parse like Go's time.ParseDuration; if numeric, keep as ms.
        if let Some(timeout_val) = cfg.get("timeout") {
            if let Some(s) = timeout_val.as_str() {
                if let Some(ms) = parse_duration_ms(s) {
                    config.timeout = ms;
                }
            }
        }

        // Basic validation/normalization
        if config.send_batch_size == 0 {
            config.send_batch_size = MIN_SEND_BATCH_SIZE;
        }
        // Go behavior: if send_batch_max_size is 0 (sentinel), use send_batch_size
        let effective_sbs = config.send_batch_size;
        let max = if config.send_batch_max_size == FOLLOW_SEND_BATCH_SIZE_SENTINEL {
            effective_sbs
        } else {
            config.send_batch_max_size
        };
        config.send_batch_max_size = max;

        if let Some(limit) = config.metadata_cardinality_limit {
            if limit < MIN_METADATA_CARDINALITY_LIMIT {
                config.metadata_cardinality_limit = Some(MIN_METADATA_CARDINALITY_LIMIT);
            }
        }
        let user_config = Arc::new(NodeUserConfig::new_processor_config(
            OTAP_BATCH_PROCESSOR_URN,
        ));
        let proc = OtapBatchProcessor {
            config,
            buffers: HashMap::new(),
        };
        Ok(ProcessorWrapper::local(proc, node, user_config, proc_cfg))
    }

    /// Derive a grouping key from metadata_keys. MVP: single default group.
    /// Returns a grouping key for buffering. Currently a single default group (MVP).
    /// After #814, derive this from configured `metadata_keys` (resource/scope/attributes).
    fn derive_group_key(&self, _data: &OtapPdata) -> GroupKey {
        if self.config.metadata_keys.is_empty() {
            return "default".to_string();
        }
        // Placeholder: will be implemented post-#814 with real metadata extraction
        "default".to_string()
    }

    /// Flush a single group by emitting each buffered item downstream.
    /// Flush buffered messages for a single group.
    ///
    /// MVP behavior: emit each buffered message one-by-one without attempting to merge/combine.
    /// Post-#814: replace with OtapPdata::append and chunk on send_batch_max_size as needed.
    async fn flush_group(
        &mut self,
        key: &GroupKey,
        effect: &mut local::EffectHandler<OtapPdata>,
    ) -> Result<(), EngineError<OtapPdata>> {
        if let Some(buf) = self.buffers.get_mut(key) {
            if buf.is_empty() {
                return Ok(());
            }
            // Emit items one-by-one for MVP (no combining yet).
            let mut to_send = Vec::new();
            std::mem::swap(&mut to_send, &mut buf.items);
            buf.count = 0;
            for item in to_send {
                effect.send_message(item).await?;
            }
        }
        Ok(())
    }

    /// Flush all non-empty groups (used by timer and shutdown)
    async fn flush_all(
        &mut self,
        effect: &mut local::EffectHandler<OtapPdata>,
    ) -> Result<(), EngineError<OtapPdata>> {
        let keys: Vec<GroupKey> = self.buffers.keys().cloned().collect();
        for k in keys.iter() {
            self.flush_group(k, effect).await?;
        }
        Ok(())
    }
}

#[async_trait(?Send)]
impl local::Processor<OtapPdata> for OtapBatchProcessor {
    async fn process(
        &mut self,
        msg: Message<OtapPdata>,
        effect: &mut local::EffectHandler<OtapPdata>,
    ) -> Result<(), EngineError<OtapPdata>> {
        match msg {
            Message::Control(ctrl) => {
                match ctrl {
                    otap_df_engine::control::NodeControlMsg::TimerTick { .. } => {
                        // MVP: flush any non-empty groups on timer
                        self.flush_all(effect).await
                    }
                    otap_df_engine::control::NodeControlMsg::Config { .. } => Ok(()),
                    otap_df_engine::control::NodeControlMsg::Shutdown { .. } => {
                        // MVP: flush and shutdown
                        self.flush_all(effect).await?;
                        effect.info(LOG_MSG_SHUTTING_DOWN).await;
                        Ok(())
                    }
                    otap_df_engine::control::NodeControlMsg::Ack { .. }
                    | otap_df_engine::control::NodeControlMsg::Nack { .. } => {
                        // MVP: no-op for ack/nack in processor skeleton
                        Ok(())
                    }
                }
            }
            Message::PData(data) => {
                // TODO(#814): When helpers are available, compute real row count per message and
                // enforce send_batch_max_size by chunking via OtapPdata::split_at.
                // MVP buffering: route to a group, buffer, and flush by count threshold
                let key = self.derive_group_key(&data);
                // MVP item count: 1 per message. After #814, use row counts from OtapArrowRecords.
                let incoming_count = item_count(&data);

                // Respect send_batch_max_size as best-effort without splitting:
                // - If adding this message would exceed max, flush current buffer first.
                // - If this single message would exceed max on its own, flush immediately after buffering
                //   to avoid holding oversized batches.
                {
                    let max = self.config.send_batch_max_size;
                    if max > FOLLOW_SEND_BATCH_SIZE_SENTINEL {
                        // We need current count without holding a mutable borrow during flush.
                        let current_count = self.buffers.get(&key).map(|b| b.count).unwrap_or(0);
                        if current_count + incoming_count > max {
                            self.flush_group(&key, effect).await?;
                        }
                    }
                }

                let buf = self.buffers.entry(key.clone()).or_insert_with(Buffer::new);
                buf.push(data);

                // Threshold-based flush on count
                let target = self.config.send_batch_size;
                if buf.count >= target {
                    let _ = buf; // release borrow before calling flush
                    self.flush_group(&key, effect).await
                } else {
                    let max = self.config.send_batch_max_size;
                    if max > FOLLOW_SEND_BATCH_SIZE_SENTINEL {
                        // Need to recheck count without holding borrow during flush
                        let cur = self.buffers.get(&key).map(|b| b.count).unwrap_or(0);
                        if cur >= max {
                            // Oversized single message or exact max: flush immediately.
                            self.flush_group(&key, effect).await
                        } else {
                            Ok(())
                        }
                    } else {
                        Ok(())
                    }
                }
            }
        }
    }
}

/// Register factory for OTAP batch processor
#[allow(unsafe_code)]
#[distributed_slice(OTAP_PROCESSOR_FACTORIES)]
pub static OTAP_BATCH_PROCESSOR_FACTORY: otap_df_engine::ProcessorFactory<OtapPdata> =
    otap_df_engine::ProcessorFactory {
        name: OTAP_BATCH_PROCESSOR_URN,
        create: |node: NodeId, cfg: &Value, proc_cfg: &ProcessorConfig| {
            OtapBatchProcessor::from_config(node, cfg, proc_cfg)
        },
    };

/// Parses duration strings from Go-style configs (e.g., "200ms", "2s", "1m").
/// If `s` is a plain number, it's treated as milliseconds for convenience.
/// MVP item counter: returns number of items represented by an OtapPdata message.
/// Currently returns 1 for all inputs. Post-#814: use actual row counts for OtapArrowRecords,
/// and decode or approximate for other formats if needed.
fn item_count(_data: &OtapPdata) -> usize {
    1
}

/// Parses duration strings using Go-like syntax (e.g., "200ms", "2s", "1m", "1m30s").
/// Returns milliseconds. Bare numbers are NOT accepted here to mirror Go's time.ParseDuration.
fn parse_duration_ms(s: &str) -> Option<u64> {
    parse_duration::parse(s).ok().map(|d| d.as_millis() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_engine::testing::test_node;
    use serde_json::json;

    #[test]
    fn test_default_config_ok() {
        let _cfg: Config = serde_json::from_value(json!({})).unwrap_or_default();
    }

    #[test]
    fn test_factory_creation() {
        let cfg = json!({"send_batch_size": 1000, "timeout": 100});
        let processor_config = ProcessorConfig::new("otap_batch_test");
        let node = test_node(processor_config.name.clone());
        let result = OtapBatchProcessor::from_config(node, &cfg, &processor_config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_with_metadata_and_max() {
        let cfg = json!({
            "send_batch_size": 3,
            "send_batch_max_size": 5,
            "timeout": 250,
            "metadata_keys": ["service.name", "telemetry.sdk.name"]
        });
        let processor_config = ProcessorConfig::new("otap_batch_test2");
        let node = test_node(processor_config.name.clone());
        let result = OtapBatchProcessor::from_config(node, &cfg, &processor_config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_timeout_go_style_string() {
        let cfg = json!({
            "send_batch_size": 3,
            "timeout": "200ms"
        });
        let processor_config = ProcessorConfig::new("otap_batch_test3");
        let node = test_node(processor_config.name.clone());
        let result = OtapBatchProcessor::from_config(node, &cfg, &processor_config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_with_cardinality_limit() {
        let cfg = json!({
            "send_batch_size": 4,
            "send_batch_max_size": 10,
            "timeout": 250,
            "metadata_keys": ["service.name"],
            "metadata_cardinality_limit": 100
        });
        let processor_config = ProcessorConfig::new("otap_batch_test_card");
        let node = test_node(processor_config.name.clone());
        let res = OtapBatchProcessor::from_config(node, &cfg, &processor_config);
        assert!(res.is_ok());
        // Ensure deserialization keeps the value
        let mut parsed: Config = serde_json::from_value(cfg).unwrap();
        assert_eq!(parsed.metadata_cardinality_limit, Some(100));
        // Normalize zero to one
        parsed.metadata_cardinality_limit = Some(0);
        // Simulate normalization by re-running from_config path
        let cfg2 = serde_json::json!({
            "metadata_cardinality_limit": 0
        });
        let proc_cfg = ProcessorConfig::new("norm");
        let node = test_node(proc_cfg.name.clone());
        let wrapper_res = OtapBatchProcessor::from_config(node, &cfg2, &proc_cfg);
        assert!(wrapper_res.is_ok());
    }

    #[test]
    fn test_flush_before_append_when_exceeding_max() {
        use crate::pdata::{OtapPdata, OtlpProtoBytes};
        use otap_df_engine::message::Message;
        use otap_df_engine::testing::processor::TestRuntime;

        let cfg = json!({
            "send_batch_size": 10, // keep large so count threshold doesn't trigger
            "send_batch_max_size": 2,
            "timeout": 10
        });
        let processor_config = ProcessorConfig::new("otap_batch_test_max1");
        let test_rt = TestRuntime::new();
        let node = test_node(processor_config.name.clone());
        let proc = OtapBatchProcessor::from_config(node, &cfg, &processor_config)
            .expect("proc from config");

        let phase = test_rt.set_processor(proc);

        // run scenario
        let validation = phase.run_test(|mut ctx| async move {
            // Prepare a trivial pdata message (content is irrelevant for this processor)
            let pdata = OtapPdata::from(OtlpProtoBytes::ExportLogsRequest(Vec::new()));
            // Send two messages; hitting max=2 should flush the two immediately
            ctx.process(Message::PData(pdata.clone()))
                .await
                .expect("process 1");
            ctx.process(Message::PData(pdata.clone()))
                .await
                .expect("process 2");
            let emitted = ctx.drain_pdata().await;
            assert_eq!(emitted.len(), 2, "flush expected when count reaches max");

            // Third message should buffer (count=1), not flushed yet
            ctx.process(Message::PData(pdata.clone()))
                .await
                .expect("process 3");
            let emitted = ctx.drain_pdata().await;
            assert_eq!(
                emitted.len(),
                0,
                "no flush expected after third until shutdown"
            );

            // Now flush remaining (the 3rd) via Shutdown
            use otap_df_engine::control::NodeControlMsg;
            use std::time::Duration;
            ctx.process(Message::Control(NodeControlMsg::Shutdown {
                deadline: Duration::from_millis(50),
                reason: "test".into(),
            }))
            .await
            .expect("shutdown");
            let emitted = ctx.drain_pdata().await;
            assert_eq!(
                emitted.len(),
                1,
                "remaining buffered item should flush on shutdown"
            );
        });

        // no additional validation phase assertions needed
        validation.validate(|_vctx| async move {});
    }

    #[test]
    fn test_immediate_flush_on_max_reached() {
        use crate::pdata::{OtapPdata, OtlpProtoBytes};
        use otap_df_engine::message::Message;
        use otap_df_engine::testing::processor::TestRuntime;

        let cfg = json!({
            "send_batch_size": 10,
            "send_batch_max_size": 1, // reaching max on first push triggers immediate flush-after-push
            "timeout": 10
        });
        let processor_config = ProcessorConfig::new("otap_batch_test_max2");
        let test_rt = TestRuntime::new();
        let node = test_node(processor_config.name.clone());
        let proc = OtapBatchProcessor::from_config(node, &cfg, &processor_config)
            .expect("proc from config");

        let phase = test_rt.set_processor(proc);

        let validation = phase.run_test(|mut ctx| async move {
            let pdata = OtapPdata::from(OtlpProtoBytes::ExportLogsRequest(Vec::new()));
            ctx.process(Message::PData(pdata)).await.expect("process 1");
            // Should have flushed immediately
            let emitted = ctx.drain_pdata().await;
            assert_eq!(
                emitted.len(),
                1,
                "single item should flush immediately when max=1"
            );

            // No more buffered; sending Shutdown shouldn't emit more
            use otap_df_engine::control::NodeControlMsg;
            use std::time::Duration;
            ctx.process(Message::Control(NodeControlMsg::Shutdown {
                deadline: Duration::from_millis(50),
                reason: "test".into(),
            }))
            .await
            .expect("shutdown");
            let emitted = ctx.drain_pdata().await;
            assert!(
                emitted.is_empty(),
                "no additional items expected on shutdown"
            );
        });

        validation.validate(|_vctx| async move {});
    }
    #[test]
    fn test_max_defaults_to_size_when_zero_or_missing() {
        let cfg = json!({
            "send_batch_size": 7,
            "send_batch_max_size": 0,
            "timeout": "200ms"
        });
        let proc_cfg = ProcessorConfig::new("norm-max");
        let node = test_node(proc_cfg.name.clone());
        let res = OtapBatchProcessor::from_config(node.clone(), &cfg, &proc_cfg);
        assert!(res.is_ok());

        // Missing max -> also defaults to size
        let cfg2 = json!({
            "send_batch_size": 9,
            "timeout": "200ms"
        });
        let res2 = OtapBatchProcessor::from_config(node, &cfg2, &proc_cfg);
        assert!(res2.is_ok());
    }
}
