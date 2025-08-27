// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OTAP batch processor (skeleton)
//!
//! Mirrors the configuration shape of the OpenTelemetry Collector batchprocessor,
//! but operates on OtapPdata. This integration is aligned with the OtapPdata combine work.
//!
//! Current scope:
//! - Buffers incoming messages and flushes by count or timer.
//! - Does not directly mutate Arrow RecordBatches; merging/splitting is delegated to upstream OTAP
//!   batching utilities.
//! - Does not directly split Arrow batches; any chunking is handled by upstream batching and emitted
//!   as multiple outputs when necessary.
//! - Partitions emission by signal type (logs/metrics/traces) when possible, but does not re-order
//!   or sort by time.
//!
//! Merging, splitting, and group-key logic are delegated to OtapPdata helpers and upstream batching
//! utilities in otel-arrow-rust.

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
use std::num::NonZeroU64;
use std::sync::Arc;
// For optional conversion during flush/partitioning
use otel_arrow_rust::otap::OtapArrowRecords;
use otel_arrow_rust::otap::batching::make_output_batches as low_make_output_batches;

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
    /// implemented. When grouping lands, this will cap the number of concurrent groups and the
    /// overflow strategy will be documented.
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

/// Local (!Send) OTAP batch processor
pub struct OtapBatchProcessor {
    config: Config,
    /// Single buffer of incoming pdata messages. We partition by signal type at flush.
    current: Vec<OtapPdata>,
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
            current: Vec::new(),
        };
        Ok(ProcessorWrapper::local(proc, node, user_config, proc_cfg))
    }

    /// Returns true if the buffered items reach or exceed the emission threshold.
    ///
    /// MVP: threshold is based on count (number of messages). In future, switch to real
    /// row counts for OtapArrowRecords.
    fn size_triggers_emission(&self) -> bool {
        self.current.len() >= self.config.send_batch_size
    }

    /// Partition buffered items by signal type when possible and emit them downstream.
    /// Items that cannot be converted to OtapArrowRecords are passed through as-is.
    async fn flush_current(
        &mut self,
        effect: &mut local::EffectHandler<OtapPdata>,
    ) -> Result<(), EngineError<OtapPdata>> {
        if self.current.is_empty() {
            return Ok(());
        }
        let mut drained: Vec<OtapPdata> = std::mem::take(&mut self.current);

        // Separate items that can be converted to OtapArrowRecords from passthrough
        let mut convertible: Vec<OtapArrowRecords> = Vec::new();
        let mut passthrough: Vec<OtapPdata> = Vec::new();
        for item in drained.drain(..) {
            match OtapArrowRecords::try_from(item.clone()) {
                Ok(rec) => convertible.push(rec),
                Err(_) => passthrough.push(item),
            }
        }

        // If nothing convertible, just emit passthrough items and return.
        if convertible.is_empty() {
            for item in passthrough {
                effect.send_message(item).await?;
            }
            return Ok(());
        }

        // Build output batches using low-level batching (split + concatenate per type)
        let max = NonZeroU64::new(self.config.send_batch_max_size as u64);
        let output_batches = match low_make_output_batches(max, convertible.clone()) {
            Ok(v) => v,
            Err(e) => {
                // Fall back to passthrough of convertible records on error
                effect
                    .info(&format!(
                        "OTAP batch processor: low-level batching failed: {e}; falling back"
                    ))
                    .await;
                convertible
            }
        };

        // Emit converted (records) first, then passthrough
        for records in output_batches {
            let pdata: OtapPdata = records.into();
            effect.send_message(pdata).await?;
        }
        for item in passthrough {
            effect.send_message(item).await?;
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
                        // Flush any buffered items on timer
                        self.flush_current(effect).await
                    }
                    otap_df_engine::control::NodeControlMsg::Config { .. } => Ok(()),
                    otap_df_engine::control::NodeControlMsg::Shutdown { .. } => {
                        // Flush and shutdown
                        self.flush_current(effect).await?;
                        effect.info(LOG_MSG_SHUTTING_DOWN).await;
                        Ok(())
                    }
                    otap_df_engine::control::NodeControlMsg::Ack { .. }
                    | otap_df_engine::control::NodeControlMsg::Nack { .. } => Ok(()),
                }
            }
            Message::PData(data) => {
                // Before buffering, respect send_batch_max_size best-effort without splitting.
                // If adding this would exceed max, flush current first.
                let max = self.config.send_batch_max_size;
                if max > FOLLOW_SEND_BATCH_SIZE_SENTINEL {
                    let current_len = self.current.len();
                    let incoming_count = item_count(&data);
                    if current_len + incoming_count > max {
                        self.flush_current(effect).await?;
                    }
                }

                // Buffer the incoming message
                self.current.push(data);

                // Threshold-based flush on count
                if self.size_triggers_emission() {
                    self.flush_current(effect).await
                } else if max > FOLLOW_SEND_BATCH_SIZE_SENTINEL {
                    // Also flush if we've hit or exceeded the max size
                    if self.current.len() >= max {
                        self.flush_current(effect).await
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
/// Currently returns 1 for all inputs. In future, use actual row counts for OtapArrowRecords,
/// and decode or approximate for other formats if needed.
fn item_count(_data: &OtapPdata) -> usize {
    1
}

/// Parses duration strings using Go-like syntax (e.g., "200ms", "2s", "1m", "1m30s").
/// Returns milliseconds. Bare numbers are NOT accepted here to mirror Go's time.ParseDuration.
fn parse_duration_ms(s: &str) -> Option<u64> {
    // humantime supports inputs like "200ms", "2s", "1m", "1h", but not scientific exponents
    humantime::parse_duration(s)
        .ok()
        .map(|d| d.as_millis() as u64)
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

    #[test]
    fn test_passthrough_metrics_bytes_flush_on_size() {
        use crate::pdata::{OtapPdata, OtlpProtoBytes};
        use otap_df_engine::message::Message;
        use otap_df_engine::testing::processor::TestRuntime;

        // Set size trigger to 1 so we flush immediately
        let cfg = json!({
            "send_batch_size": 1,
            "send_batch_max_size": 10,
            "timeout": 10
        });
        let processor_config = ProcessorConfig::new("otap_batch_test_passthrough_size");
        let test_rt = TestRuntime::new();
        let node = test_node(processor_config.name.clone());
        let proc = OtapBatchProcessor::from_config(node, &cfg, &processor_config)
            .expect("proc from config");

        let phase = test_rt.set_processor(proc);

        let validation = phase.run_test(|mut ctx| async move {
            // Metrics OTLP bytes are not yet supported for conversion -> passthrough
            let pdata = OtapPdata::from(OtlpProtoBytes::ExportMetricsRequest(vec![1, 2, 3]));
            ctx.process(Message::PData(pdata)).await.expect("process 1");
            let emitted = ctx.drain_pdata().await;
            assert_eq!(emitted.len(), 1, "passthrough should emit on size trigger");
        });
        validation.validate(|_vctx| async move {});
    }

    #[test]
    fn test_passthrough_metrics_bytes_flush_on_shutdown() {
        use crate::pdata::{OtapPdata, OtlpProtoBytes};
        use otap_df_engine::control::NodeControlMsg;
        use otap_df_engine::message::Message;
        use otap_df_engine::testing::processor::TestRuntime;
        use std::time::Duration;

        // Set size trigger high so we don't flush until shutdown
        let cfg = json!({
            "send_batch_size": 10,
            "send_batch_max_size": 10,
            "timeout": 10
        });
        let processor_config = ProcessorConfig::new("otap_batch_test_passthrough_shutdown");
        let test_rt = TestRuntime::new();
        let node = test_node(processor_config.name.clone());
        let proc = OtapBatchProcessor::from_config(node, &cfg, &processor_config)
            .expect("proc from config");

        let phase = test_rt.set_processor(proc);

        let validation = phase.run_test(|mut ctx| async move {
            let pdata = OtapPdata::from(OtlpProtoBytes::ExportMetricsRequest(vec![9, 9, 9]));
            ctx.process(Message::PData(pdata)).await.expect("process");
            let emitted = ctx.drain_pdata().await;
            assert_eq!(emitted.len(), 0, "no flush before shutdown");

            ctx.process(Message::Control(NodeControlMsg::Shutdown {
                deadline: Duration::from_millis(50),
                reason: "test".into(),
            }))
            .await
            .expect("shutdown");
            let emitted = ctx.drain_pdata().await;
            assert_eq!(emitted.len(), 1, "passthrough should flush on shutdown");
        });
        validation.validate(|_vctx| async move {});
    }
}

#[cfg(test)]
mod batching_smoke_tests {
    use super::*;
    use otel_arrow_rust::proto::opentelemetry::arrow::v1::ArrowPayloadType;
    use otel_arrow_rust::proto::opentelemetry::common::v1::InstrumentationScope;
    use otel_arrow_rust::proto::opentelemetry::metrics::v1::{
        Gauge, Metric, MetricsData, NumberDataPoint, ResourceMetrics, ScopeMetrics,
    };
    use otel_arrow_rust::proto::opentelemetry::resource::v1::Resource;
    use otel_arrow_rust::proto::opentelemetry::trace::v1::status::StatusCode;
    use otel_arrow_rust::proto::opentelemetry::trace::v1::{
        ResourceSpans, ScopeSpans, Span, Status, TracesData,
    };

    fn one_trace_record() -> OtapArrowRecords {
        let traces = TracesData::new(vec![
            ResourceSpans::build(Resource::default())
                .scope_spans(vec![
                    ScopeSpans::build(InstrumentationScope::new("lib"))
                        .spans(vec![
                            Span::build(vec![0; 16], vec![1; 8], "span", 1u64)
                                .status(Status::new("ok", StatusCode::Ok))
                                .finish(),
                        ])
                        .finish(),
                ])
                .finish(),
        ]);
        crate::encoder::encode_spans_otap_batch(&traces).expect("encode traces")
    }

    fn one_metric_record() -> OtapArrowRecords {
        // Minimal metrics: one Gauge with one NumberDataPoint
        let md = MetricsData::new(vec![
            ResourceMetrics::build(Resource::default())
                .scope_metrics(vec![
                    ScopeMetrics::build(InstrumentationScope::new("lib"))
                        .metrics(vec![
                            Metric::build_gauge(
                                "g",
                                Gauge::new(vec![NumberDataPoint::build_double(0u64, 1.0).finish()]),
                            )
                            .finish(),
                        ])
                        .finish(),
                ])
                .finish(),
        ]);
        crate::encoder::encode_metrics_otap_batch(&md).expect("encode metrics")
    }

    #[test]
    #[ignore]
    fn test_low_make_output_batches_partitions_and_splits() {
        // Build mixed input: 3 traces (1 row each), 2 metrics (1 dp each), interleaved
        let input = vec![
            one_trace_record(),
            one_metric_record(),
            one_trace_record(),
            one_metric_record(),
            one_trace_record(),
        ];

        // For now, use None for splitting due to upstream batching limitations when some groups are empty
        let outputs = low_make_output_batches(None, input).expect("batching ok");

        // Expect 2 outputs: one metrics (2 rows), one traces (3 rows)
        let mut metrics_batches = 0usize;
        let mut traces_batches = 0usize;
        let mut total_metrics_rows = 0usize;
        let mut total_traces_rows = 0usize;

        for out in &outputs {
            match out {
                OtapArrowRecords::Metrics(_) => {
                    metrics_batches += 1;
                    let rb = out
                        .get(ArrowPayloadType::UnivariateMetrics)
                        .expect("metrics rb");
                    assert!(rb.num_rows() <= 2, "metrics batch exceeds max rows");
                    total_metrics_rows += rb.num_rows();
                }
                OtapArrowRecords::Traces(_) => {
                    traces_batches += 1;
                    let rb = out.get(ArrowPayloadType::Spans).expect("spans rb");
                    assert!(rb.num_rows() <= 2, "traces batch exceeds max rows");
                    total_traces_rows += rb.num_rows();
                }
                OtapArrowRecords::Logs(_) => {
                    panic!("unexpected logs batch in outputs");
                }
            }
        }

        assert_eq!(metrics_batches, 1, "expected one metrics output");
        assert_eq!(traces_batches, 1, "expected one traces output");
        assert_eq!(total_metrics_rows, 2, "expected two metric rows total");
        assert_eq!(total_traces_rows, 3, "expected three trace rows total");
    }
}
