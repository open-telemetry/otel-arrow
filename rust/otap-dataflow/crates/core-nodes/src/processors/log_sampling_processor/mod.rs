// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Log sampling processor.
//!
//! Reduces log volume by discarding a portion of incoming log records
//! according to a configurable sampling strategy. Non-log signals
//! (metrics and traces) pass through unchanged.

mod config;
mod metrics;
mod samplers;

use self::config::Config;
use self::metrics::LogSamplingMetrics;
use self::samplers::{Sampler, sampler_from_config};

use arrow::array::BooleanBufferBuilder;
use arrow::buffer::MutableBuffer;
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::SignalType;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::ConsumerEffectHandlerExtension;
use otap_df_engine::MessageSourceLocalEffectHandlerExtension;
use otap_df_engine::config::ProcessorConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{AckMsg, NackMsg, NodeControlMsg};
use otap_df_engine::error::Error as EngineError;
use otap_df_engine::local::processor as local;
use otap_df_engine::message::Message;
use otap_df_engine::node::NodeId;
use otap_df_engine::processor::ProcessorWrapper;
use otap_df_otap::OTAP_PROCESSOR_FACTORIES;
use otap_df_otap::pdata::OtapPdata;
use otap_df_pdata::OtapPayload;
use otap_df_pdata::otap::OtapArrowRecords;
use otap_df_pdata::otap::filter::{IdBitmapPool, filter_otap_batch};
use otap_df_telemetry::metrics::MetricSet;
use serde_json::Value;
use std::sync::Arc;

const LOG_SAMPLING_PROCESSOR_URN: &str = "urn:otel:processor:log_sampling";

#[allow(unsafe_code)]
#[distributed_slice(OTAP_PROCESSOR_FACTORIES)]
static LOG_SAMPLING_PROCESSOR_FACTORY: otap_df_engine::ProcessorFactory<OtapPdata> =
    otap_df_engine::ProcessorFactory {
        name: LOG_SAMPLING_PROCESSOR_URN,
        create: |pipeline_ctx: PipelineContext,
                 node: NodeId,
                 node_config: Arc<NodeUserConfig>,
                 proc_cfg: &ProcessorConfig| {
            create_log_sampling_processor(pipeline_ctx, node, node_config, proc_cfg)
        },
        validate_config: otap_df_config::validation::validate_typed_config::<Config>,
        wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    };

/// Log sampling processor.
struct LogSamplingProcessor {
    /// The chosen sampler
    sampler: Box<dyn Sampler>,
    /// Telemetry metrics.
    metrics: MetricSet<LogSamplingMetrics>,
    /// Reusable bitmap pool for child batch filtering.
    id_bitmap_pool: IdBitmapPool,
    /// Reusable buffer for filtering
    filter_buffer: MutableBuffer,
}

impl LogSamplingProcessor {
    fn from_config(pipeline_ctx: PipelineContext, config: &Value) -> Result<Self, ConfigError> {
        let config: Config =
            serde_json::from_value(config.clone()).map_err(|e| ConfigError::InvalidUserConfig {
                error: e.to_string(),
            })?;
        config.validate()?;

        let sampler = sampler_from_config(&config.policy);
        let metrics = pipeline_ctx.register_metrics::<LogSamplingMetrics>();

        Ok(Self {
            sampler,
            metrics,
            id_bitmap_pool: IdBitmapPool::new(),
            filter_buffer: MutableBuffer::with_capacity(0),
        })
    }

    /// Processes a log payload: sample, filter, and forward or ack.
    async fn process_logs(
        &mut self,
        pdata: OtapPdata,
        effect_handler: &mut local::EffectHandler<OtapPdata>,
    ) -> Result<(), EngineError> {
        let total = pdata.num_items();
        self.metrics.log_signals_consumed.add(total as u64);

        // Convert to Arrow records (no-op if already Arrow)
        let (context, payload) = pdata.into_parts();
        let mut records: OtapArrowRecords = payload.try_into()?;
        records.decode_transport_optimized_ids()?;

        // Prepare the filter buffer.
        //
        // We make sure we have capacity for the whole result here to avoid
        // multiple resizes through samplers calling things like `append` or
        // `append_n`
        let buf_len = total.div_ceil(8);
        let mut buf = std::mem::take(&mut self.filter_buffer);
        buf.resize(buf_len, 0);

        // Filter
        let mut builder = BooleanBufferBuilder::new_from_buffer(buf, 0);
        let selection = self.sampler.sample_arrow_records(&records, &mut builder);
        let filter_result = filter_otap_batch(&selection, &records, &mut self.id_bitmap_pool);

        // Attempt to reclaim the buffer before checking the result. If this
        // fails then we have a 0 size mutable buffer and it will simply get
        // resized appropriately the next time around.
        //
        // This _should_ succeed if the sampler called finish as the builder
        // will drop its reference.
        let (buf, _) = selection.into_parts();
        match buf.into_inner().into_mutable() {
            Ok(mutable) => self.filter_buffer = mutable,
            Err(_) => {
                self.metrics.filter_buffer_reclamation_failures.inc();
            }
        }

        let filtered = match filter_result {
            Ok(filtered) => filtered,
            Err(e) => {
                self.metrics.filtering_errors.inc();
                let pdata = OtapPdata::new(context, OtapPayload::OtapArrowRecords(records));
                effect_handler
                    .notify_nack(NackMsg::new(
                        format!("failed to filter otap batch: {e}"),
                        pdata,
                    ))
                    .await?;
                return Ok(());
            }
        };

        // Compute dropped count from the difference in item counts.
        let kept = filtered.num_items();
        let dropped = total - kept;
        self.metrics.log_signals_dropped.add(dropped as u64);

        let pdata = OtapPdata::new(context, OtapPayload::OtapArrowRecords(filtered));
        if kept == 0 {
            effect_handler.notify_ack(AckMsg::new(pdata)).await?;
        } else {
            effect_handler.send_message_with_source_node(pdata).await?;
        }

        Ok(())
    }
}

#[async_trait(?Send)]
impl local::Processor<OtapPdata> for LogSamplingProcessor {
    async fn process(
        &mut self,
        msg: Message<OtapPdata>,
        effect_handler: &mut local::EffectHandler<OtapPdata>,
    ) -> Result<(), EngineError> {
        // Let the sampler perform any one-time initialization (e.g. start timer).
        self.sampler.ensure_init(effect_handler).await?;

        match msg {
            Message::PData(pdata) => {
                match pdata.signal_type() {
                    SignalType::Logs => self.process_logs(pdata, effect_handler).await,
                    // Metrics and traces pass through unchanged
                    SignalType::Metrics | SignalType::Traces => {
                        effect_handler.send_message_with_source_node(pdata).await?;
                        Ok(())
                    }
                }
            }
            Message::Control(ctrl) => match ctrl {
                NodeControlMsg::TimerTick {} => {
                    self.sampler.notify_timer_tick();
                    Ok(())
                }
                NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                } => {
                    let _ = metrics_reporter.report(&mut self.metrics);
                    Ok(())
                }
                NodeControlMsg::Shutdown { .. }
                | NodeControlMsg::Config { .. }
                | NodeControlMsg::Ack(_)
                | NodeControlMsg::Nack(_)
                | NodeControlMsg::MemoryPressureChanged { .. }
                | NodeControlMsg::DrainIngress { .. }
                | NodeControlMsg::Wakeup { .. }
                | NodeControlMsg::DelayedData { .. } => Ok(()),
            },
        }
    }
}

/// Creates a new [`LogSamplingProcessor`] from pipeline configuration.
fn create_log_sampling_processor(
    pipeline_ctx: PipelineContext,
    node: NodeId,
    node_config: Arc<NodeUserConfig>,
    processor_config: &ProcessorConfig,
) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
    Ok(ProcessorWrapper::local(
        LogSamplingProcessor::from_config(pipeline_ctx, &node_config.config)?,
        node,
        node_config,
        processor_config,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::AsArray;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::message::Message;
    use otap_df_engine::processor::ProcessorWrapper;
    use otap_df_engine::testing::processor::{TestContext, TestRuntime};
    use otap_df_engine::testing::test_node;
    use otap_df_otap::pdata::Context;
    use otap_df_pdata::encode::{encode_logs_otap_batch, encode_spans_otap_batch};
    use otap_df_pdata::otap::OtapBatchStore;
    use otap_df_pdata::proto::OtlpProtoMessage;
    use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
    use otap_df_pdata::testing::fixtures::{
        logs_with_varying_attributes_and_properties, metrics_sum_with_full_resource_and_scope,
        traces_with_full_resource_and_scope,
    };
    use otap_df_pdata::testing::round_trip::otlp_message_to_bytes;
    use otap_df_pdata::{logs, record_batch};
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
    use std::future::Future;

    // ==================== Integration Tests ====================

    #[test]
    fn test_transport_optimized_encoding_removed() {
        let config = serde_json::json!({
            "policy": {
                "ratio": {
                    "emit": 2,
                    "out_of": 3
                }
            }
        });

        run_processor_test(config, |mut ctx: TestContext<OtapPdata>| {
            Box::pin(async move {
                // Three log records each identified by a letter and with a matching
                // "name" attribute.
                //
                // The ids will get delta encoded to [0, 1, 1]. If the delta encoding
                // is not removed then we'll keep attribute `name: "c"` in the final
                // output
                #[rustfmt::skip]
                let input_logs = logs!(
                    (Logs,
                        ("id", UInt16, vec![1, 2, 3]),
                        ("body.type", UInt8, vec![2, 2, 2]),
                        ("body.str", Utf8, vec!["a", "b", "c"])),
                    (LogAttrs,
                        ("parent_id", UInt16, vec![1, 2, 3]),
                        ("key", Utf8, vec!["name", "name", "name"]),
                        ("type", UInt8, vec![2, 2, 2]),
                        ("str", Utf8, vec!["a", "b", "c"])),
                );

                let mut records = OtapArrowRecords::Logs(input_logs);
                records.encode_transport_optimized().unwrap();
                let pdata =
                    OtapPdata::new(Context::default(), OtapPayload::OtapArrowRecords(records));

                ctx.process(Message::PData(pdata)).await.expect("process");
                let msgs = ctx.drain_pdata().await;
                assert_eq!(msgs.len(), 1);
                assert_eq!(msgs[0].num_items(), 2);

                let output_payload = msgs[0].clone().into_parts().1.take_payload();
                let output_otap = match output_payload {
                    OtapPayload::OtlpBytes(_) => panic!("Unexpected otlp bytes"),
                    OtapPayload::OtapArrowRecords(otap_arrow_records) => otap_arrow_records,
                };

                let output_attrs = output_otap.get(ArrowPayloadType::LogAttrs).unwrap();
                // Make sure we only have two attributes in the output
                assert_eq!(2, output_attrs.num_rows());
                // Make sure none of them are "c"
                assert!(
                    !output_attrs
                        .column_by_name("str")
                        .unwrap()
                        .as_string::<i32>()
                        .iter()
                        .any(|s| s == Some("c"))
                )
            })
        });
    }

    #[test]
    fn test_zip_basic_flow() {
        let config = serde_json::json!({
            "policy": {
                "zip": {
                    "interval": "60s",
                    "max_items": 20
                }
            }
        });

        run_processor_test(config, |mut ctx: TestContext<OtapPdata>| {
            Box::pin(async move {
                // Send 10 logs (within budget of 20)
                let pdata = make_log_pdata_arrow(10);
                ctx.process(Message::PData(pdata)).await.expect("process");
                let msgs = ctx.drain_pdata().await;
                assert_eq!(msgs.len(), 1, "all 10 should be forwarded");
                assert_eq!(msgs[0].num_items(), 10);

                // Send 15 more logs (exceeds remaining budget of 10)
                let pdata = make_log_pdata_arrow(15);
                ctx.process(Message::PData(pdata)).await.expect("process");
                let msgs = ctx.drain_pdata().await;
                assert_eq!(msgs.len(), 1, "partial batch should be forwarded");
                assert_eq!(msgs[0].num_items(), 10, "only 10 remaining budget");

                // Send 5 more (budget exhausted, should be acked/dropped)
                let pdata = make_log_pdata_arrow(5);
                ctx.process(Message::PData(pdata)).await.expect("process");
                let msgs = ctx.drain_pdata().await;
                assert_eq!(msgs.len(), 0, "budget exhausted, nothing forwarded");
            })
        });
    }

    #[test]
    fn test_ratio_basic_flow() {
        let config = serde_json::json!({
            "policy": {
                "ratio": {
                    "emit": 1,
                    "out_of": 10
                }
            }
        });

        run_processor_test(config, |mut ctx: TestContext<OtapPdata>| {
            Box::pin(async move {
                // Send 100 logs, expect 10 (1:10 ratio)
                let pdata = make_log_pdata_arrow(100);
                ctx.process(Message::PData(pdata)).await.expect("process");
                let msgs = ctx.drain_pdata().await;
                assert_eq!(msgs.len(), 1);
                assert_eq!(msgs[0].num_items(), 10);
            })
        });
    }

    #[test]
    fn test_pass_through_traces() {
        let config = serde_json::json!({
            "policy": {
                "ratio": {
                    "emit": 1,
                    "out_of": 10
                }
            }
        });

        run_processor_test(config, |mut ctx: TestContext<OtapPdata>| {
            Box::pin(async move {
                let pdata = make_trace_pdata_arrow();
                let original_items = pdata.num_items();
                ctx.process(Message::PData(pdata)).await.expect("process");
                let msgs = ctx.drain_pdata().await;
                assert_eq!(msgs.len(), 1, "traces should pass through");
                assert_eq!(msgs[0].num_items(), original_items);
            })
        });
    }

    #[test]
    fn test_pass_through_traces_otlp() {
        let config = serde_json::json!({
            "policy": {
                "ratio": {
                    "emit": 1,
                    "out_of": 10
                }
            }
        });

        run_processor_test(config, |mut ctx: TestContext<OtapPdata>| {
            Box::pin(async move {
                let pdata = make_trace_pdata_otlp();
                let original_items = pdata.num_items();
                ctx.process(Message::PData(pdata)).await.expect("process");
                let msgs = ctx.drain_pdata().await;
                assert_eq!(msgs.len(), 1, "traces (OTLP bytes) should pass through");
                assert_eq!(msgs[0].num_items(), original_items);
            })
        });
    }

    #[test]
    fn test_pass_through_metrics() {
        let config = serde_json::json!({
            "policy": {
                "ratio": {
                    "emit": 1,
                    "out_of": 10
                }
            }
        });

        run_processor_test(config, |mut ctx: TestContext<OtapPdata>| {
            Box::pin(async move {
                let pdata = make_metrics_pdata_otlp();
                let original_items = pdata.num_items();
                ctx.process(Message::PData(pdata)).await.expect("process");
                let msgs = ctx.drain_pdata().await;
                assert_eq!(msgs.len(), 1, "metrics should pass through");
                assert_eq!(msgs[0].num_items(), original_items);
            })
        });
    }

    #[test]
    fn test_timer_tick_resets_zip_counter() {
        let config = serde_json::json!({
            "policy": {
                "zip": {
                    "interval": "60s",
                    "max_items": 10
                }
            }
        });

        run_processor_test(config, |mut ctx: TestContext<OtapPdata>| {
            Box::pin(async move {
                // Fill the budget
                let pdata = make_log_pdata_arrow(10);
                ctx.process(Message::PData(pdata)).await.expect("process");
                let msgs = ctx.drain_pdata().await;
                assert_eq!(msgs.len(), 1);
                assert_eq!(msgs[0].num_items(), 10);

                // Budget exhausted
                let pdata = make_log_pdata_arrow(5);
                ctx.process(Message::PData(pdata)).await.expect("process");
                let msgs = ctx.drain_pdata().await;
                assert_eq!(msgs.len(), 0, "budget exhausted");

                // Timer tick resets the counter
                ctx.process(Message::timer_tick_ctrl_msg())
                    .await
                    .expect("timer tick");

                // Now we have budget again
                let pdata = make_log_pdata_arrow(5);
                ctx.process(Message::PData(pdata)).await.expect("process");
                let msgs = ctx.drain_pdata().await;
                assert_eq!(msgs.len(), 1, "budget restored after timer tick");
                assert_eq!(msgs[0].num_items(), 5);
            })
        });
    }

    // ==================== Helpers ====================

    /// Create a processor wrapped in TestRuntime, run a scenario, validate.
    fn run_processor_test<F, Fut>(config_json: Value, scenario: F)
    where
        F: FnOnce(TestContext<OtapPdata>) -> Fut + 'static,
        Fut: Future<Output = ()> + 'static,
    {
        let test_runtime = TestRuntime::new();
        let user_config = Arc::new(NodeUserConfig::new_processor_config(
            LOG_SAMPLING_PROCESSOR_URN,
        ));

        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let processor = ProcessorWrapper::local(
            LogSamplingProcessor::from_config(pipeline_ctx, &config_json).expect("valid config"),
            test_node(test_runtime.config().name.clone()),
            user_config,
            test_runtime.config(),
        );

        test_runtime
            .set_processor(processor)
            .run_test(scenario)
            .validate(|_ctx| async {});
    }

    fn make_log_pdata_arrow(n: usize) -> OtapPdata {
        let logs_data = logs_with_varying_attributes_and_properties(n);
        let records = encode_logs_otap_batch(&logs_data).expect("encode");
        OtapPdata::new(Context::default(), OtapPayload::OtapArrowRecords(records))
    }

    fn make_trace_pdata_arrow() -> OtapPdata {
        let traces_data = traces_with_full_resource_and_scope();
        let records = encode_spans_otap_batch(&traces_data).expect("encode");
        OtapPdata::new(Context::default(), OtapPayload::OtapArrowRecords(records))
    }

    fn make_trace_pdata_otlp() -> OtapPdata {
        let traces_data = traces_with_full_resource_and_scope();
        let otlp_bytes = otlp_message_to_bytes(&OtlpProtoMessage::Traces(traces_data));
        OtapPdata::new(Context::default(), otlp_bytes.into())
    }

    fn make_metrics_pdata_otlp() -> OtapPdata {
        let metrics_data = metrics_sum_with_full_resource_and_scope();
        let otlp_bytes = otlp_message_to_bytes(&OtlpProtoMessage::Metrics(metrics_data));
        OtapPdata::new(Context::default(), otlp_bytes.into())
    }
}
