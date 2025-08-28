// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Exporter used to measure the performance of the OTAP data pipeline.
//!
//! ToDo – Future developments / improvements:
//! - Replace this exporter with a processor that could be combined with a Noop exporter to achieve
//!   the same functionality. The advantage would be to allow performance measurements anywhere in
//!   the pipeline.
//! - Measure the number of memory allocations for the current thread. This would allow measuring
//!   the memory used by the pipeline. This is possible using `mimalloc-rust-sys`.
//! - Measure per-thread CPU usage. This would allow measuring the pipeline’s CPU load. This is
//!   possible using the "libc" crate function `getrusage(RUSAGE\_THREAD)`.
//! - Measure network usage either via a cgroup or via eBPF.
//! - Measure per-thread perf counters (see crates perfcnt, perfcnt2, or direct perf\_event\_open
//!   via nix/libc). We could measure task-clock, context switches, page faults, ...
//! - Measure the latency of signals traversing the pipeline. This would require adding a timestamp
//!   in the headers of pdata messages.
//! - Support live reconfiguration via control message.

use crate::OTAP_EXPORTER_FACTORIES;
use crate::grpc::OtapArrowBytes;
use crate::metrics::ExporterPDataMetrics;
use crate::pdata::OtapPdata;
use crate::perf_exporter::config::Config;
use crate::perf_exporter::metrics::PerfExporterPdataMetrics;
use async_trait::async_trait;
use fluke_hpack::Decoder;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::config::ExporterConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::error::Error;
use otap_df_engine::exporter::ExporterWrapper;
use otap_df_engine::local::exporter as local;
use otap_df_engine::message::{Message, MessageChannel};
use otap_df_engine::node::NodeId;
use otap_df_engine::{ExporterFactory, distributed_slice};
use otap_df_telemetry::metrics::MetricSet;
use otel_arrow_rust::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use serde_json::Value;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::Duration;

/// The URN for the OTAP Perf exporter
pub const OTAP_PERF_EXPORTER_URN: &str = "urn:otel:otap:perf:exporter";

/// Perf Exporter that emits performance data
pub struct PerfExporter {
    config: Config,
    metrics: MetricSet<PerfExporterPdataMetrics>,
    pdata_metrics: MetricSet<ExporterPDataMetrics>,
}

/// Declares the OTAP Perf exporter as a local exporter factory
///
/// Unsafe code is temporarily used here to allow the use of `distributed_slice` macro
/// This macro is part of the `linkme` crate which is considered safe and well maintained.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
pub static PERF_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: OTAP_PERF_EXPORTER_URN,
    create: |pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             exporter_config: &ExporterConfig| {
        Ok(ExporterWrapper::local(
            PerfExporter::from_config(pipeline, &node_config.config)?,
            node,
            node_config,
            exporter_config,
        ))
    },
};

impl PerfExporter {
    /// creates a perf exporter with the provided config
    #[must_use]
    pub fn new(pipeline_ctx: PipelineContext, config: Config) -> Self {
        let metrics = pipeline_ctx.register_metrics::<PerfExporterPdataMetrics>();
        let pdata_metrics = pipeline_ctx.register_metrics::<ExporterPDataMetrics>();

        PerfExporter {
            config,
            metrics,
            pdata_metrics,
        }
    }

    /// Creates a new PerfExporter from a configuration object
    pub fn from_config(
        pipeline_ctx: PipelineContext,
        config: &Value,
    ) -> Result<Self, otap_df_config::error::Error> {
        Ok(PerfExporter::new(
            pipeline_ctx,
            serde_json::from_value(config.clone()).map_err(|e| {
                otap_df_config::error::Error::InvalidUserConfig {
                    error: e.to_string(),
                }
            })?,
        ))
    }
}

#[async_trait(?Send)]
impl local::Exporter<OtapPdata> for PerfExporter {
    async fn start(
        mut self: Box<Self>,
        mut msg_chan: MessageChannel<OtapPdata>,
        effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        // init variables for tracking
        let mut average_pipeline_latency: f64 = 0.0;

        effect_handler.info("Starting Perf Exporter\n").await;

        // Start telemetry collection tick as a dedicated control message.
        let _ = effect_handler
            .start_periodic_telemetry(Duration::from_millis(self.config.frequency()))
            .await?;

        // Loop until a Shutdown event is received.
        loop {
            let msg = msg_chan.recv().await?;
            match msg {
                Message::Control(NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                }) => {
                    _ = metrics_reporter.report(&mut self.metrics);
                    _ = metrics_reporter.report(&mut self.pdata_metrics);
                }
                // ToDo: Handle configuration changes
                Message::Control(NodeControlMsg::Config { .. }) => {}
                Message::Control(NodeControlMsg::Shutdown { .. }) => {
                    break;
                }
                Message::PData(pdata) => {
                    // Capture signal type before moving pdata into try_from
                    let signal_type = pdata.signal_type();

                    // Increment consumed for this signal
                    self.pdata_metrics.inc_consumed(signal_type);

                    let batch = match OtapArrowBytes::try_from(pdata) {
                        Ok(OtapArrowBytes::ArrowLogs(batch)) => batch,
                        Ok(OtapArrowBytes::ArrowMetrics(batch)) => batch,
                        Ok(OtapArrowBytes::ArrowTraces(batch)) => batch,
                        Err(_) => {
                            // if the pdata is not a valid Arrow batch, we skip it.
                            // For example, when it is a not supported signal type.
                            self.pdata_metrics.inc_failed(signal_type);
                            continue;
                        }
                    };

                    // Keep track of Arrow records received
                    self.metrics
                        .arrow_records
                        .add(batch.arrow_payloads.len() as u64);

                    // Increment counters per type of OTLP signals
                    for arrow_payload in &batch.arrow_payloads {
                        match ArrowPayloadType::try_from(arrow_payload.r#type) {
                            Ok(ArrowPayloadType::UnivariateMetrics) => {
                                // Increment metrics counter (number of metric records)
                                self.metrics.metrics.add(arrow_payload.record.len() as u64);
                                break;
                            }
                            Ok(ArrowPayloadType::Logs) => {
                                self.metrics.logs.add(arrow_payload.record.len() as u64);
                                break;
                            }
                            Ok(ArrowPayloadType::Spans) => {
                                self.metrics.spans.add(arrow_payload.record.len() as u64);
                                break;
                            }
                            _ => {}
                        }
                    }

                    // ToDo (LQ) We need to introduce pdata headers without hpack encoding for data coming from other nodes
                    // decode the headers which are hpack encoded
                    // check for timestamp
                    // get time delta between now and timestamp
                    // calculate average
                    let mut decoder = Decoder::new();
                    let header_list =
                        decoder
                            .decode(&batch.headers)
                            .map_err(|_| Error::ExporterError {
                                exporter: effect_handler.exporter_id(),
                                error: "Failed to decode batch headers".to_owned(),
                            })?;
                    // find the timestamp header and parse it
                    // timestamp will be added in the receiver to enable pipeline latency calculation
                    let timestamp_pair = header_list.iter().find(|(name, _)| name == b"timestamp");
                    if let Some((_, value)) = timestamp_pair {
                        let timestamp =
                            decode_timestamp(value).map_err(|error| Error::ExporterError {
                                exporter: effect_handler.exporter_id(),
                                error,
                            })?;
                        let current_unix_time = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .map_err(|error| Error::ExporterError {
                                exporter: effect_handler.exporter_id(),
                                error: error.to_string(),
                            })?;
                        let latency = (current_unix_time - timestamp).as_secs_f64();
                        average_pipeline_latency = update_average(
                            latency,
                            average_pipeline_latency,
                            self.config.smoothing_factor() as f64,
                        );
                    }

                    // Successful perf reporting: mark as exported for this signal
                    self.pdata_metrics.inc_exported(signal_type);

                    // ToDo Report disk, io, cpu, mem usage once gauge metrics are implemented
                }
                _ => {
                    return Err(Error::ExporterError {
                        exporter: effect_handler.exporter_id(),
                        error: "Unknown control message".to_owned(),
                    });
                }
            }
        }
        Ok(())
    }
}

/// uses the exponential moving average formula to update the average
fn update_average(new_value: f64, old_average: f64, smoothing_factor: f64) -> f64 {
    // update the average using a exponential moving average which allows new data points to have a greater impact depending on the smoothing factor
    smoothing_factor * new_value + (1.0 - smoothing_factor) * old_average
}

/// decodes the byte array from the timestamp header and gets the equivalent duration value
fn decode_timestamp(timestamp: &[u8]) -> Result<Duration, String> {
    let timestamp_string = std::str::from_utf8(timestamp).map_err(|error| error.to_string())?;
    let timestamp_parts: Vec<&str> = timestamp_string.split(":").collect();
    let secs = timestamp_parts[0]
        .parse::<u64>()
        .map_err(|error| error.to_string())?;
    let nanosecs = timestamp_parts[1]
        .parse::<u32>()
        .map_err(|error| error.to_string())?;

    Ok(Duration::new(secs, nanosecs))
}

#[cfg(test)]
mod tests {

    use crate::fixtures::{
        SimpleDataGenOptions, create_simple_logs_arrow_record_batches,
        create_simple_metrics_arrow_record_batches, create_simple_trace_arrow_record_batches,
    };
    use crate::grpc::OtapArrowBytes;
    use crate::pdata::OtapPdata;
    use crate::perf_exporter::config::Config;
    use crate::perf_exporter::exporter::{OTAP_PERF_EXPORTER_URN, PerfExporter};
    use otap_df_config::node::NodeUserConfig;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::error::Error;
    use otap_df_engine::exporter::ExporterWrapper;
    use otap_df_engine::testing::exporter::TestContext;
    use otap_df_engine::testing::exporter::TestRuntime;
    use otap_df_engine::testing::test_node;
    use otap_df_telemetry::registry::MetricsRegistryHandle;
    use std::future::Future;
    use std::sync::Arc;
    use tokio::time::{Duration, sleep};

    /// Test closure that simulates a typical test scenario by sending timer ticks, config,
    /// data message, and shutdown control messages.
    ///
    fn scenario()
    -> impl FnOnce(TestContext<OtapPdata>) -> std::pin::Pin<Box<dyn Future<Output = ()>>> {
        |ctx| {
            Box::pin(async move {
                // send some messages to the exporter to calculate pipeline statistics
                for i in 0..3 {
                    let traces_batch_data =
                        create_simple_trace_arrow_record_batches(SimpleDataGenOptions {
                            id_offset: 3 * i,
                            num_rows: 5,
                            ..Default::default()
                        });
                    let logs_batch_data =
                        create_simple_logs_arrow_record_batches(SimpleDataGenOptions {
                            id_offset: 3 * i + 1,
                            num_rows: 5,
                            ..Default::default()
                        });
                    let metrics_batch_data =
                        create_simple_metrics_arrow_record_batches(SimpleDataGenOptions {
                            id_offset: 3 * i + 2,
                            num_rows: 5,
                            ..Default::default()
                        });
                    // // Send a data message
                    ctx.send_pdata(OtapArrowBytes::ArrowTraces(traces_batch_data).into())
                        .await
                        .expect("Failed to send data message");
                    ctx.send_pdata(OtapArrowBytes::ArrowLogs(logs_batch_data).into())
                        .await
                        .expect("Failed to send data message");
                    ctx.send_pdata(OtapArrowBytes::ArrowMetrics(metrics_batch_data).into())
                        .await
                        .expect("Failed to send data message");
                }

                // TODO ADD DELAY BETWEEN HERE
                _ = sleep(Duration::from_millis(5000));

                // Send shutdown
                ctx.send_shutdown(Duration::from_millis(200), "test complete")
                    .await
                    .expect("Failed to send Shutdown");
            })
        }
    }

    /// Validation closure that checks the expected counter values
    fn validation_procedure(
        metrics_registry_handle: MetricsRegistryHandle,
    ) -> impl FnOnce(
        TestContext<OtapPdata>,
        Result<(), Error>,
    ) -> std::pin::Pin<Box<dyn Future<Output = ()>>> {
        |_, exporter_result| {
            Box::pin(async move {
                assert!(exporter_result.is_ok());

                metrics_registry_handle.visit_current_metrics(
                    |_metrics_descriptor, _attrs, _metric_values| {
                        // ToDo Check the counters, once the timer tick control message is implemented in the test infrastructure.
                    },
                );
            })
        }
    }

    #[test]
    fn test_exporter_local() {
        let test_runtime = TestRuntime::new();
        let config = Config::new(1000, 0.3, true, true, true, true, true);
        let node_config = Arc::new(NodeUserConfig::new_exporter_config(OTAP_PERF_EXPORTER_URN));
        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle.clone());
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);
        let exporter = ExporterWrapper::local(
            PerfExporter::new(pipeline_ctx, config),
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        test_runtime
            .set_exporter(exporter)
            .run_test(scenario())
            .run_validation(validation_procedure(metrics_registry_handle));
    }
}
