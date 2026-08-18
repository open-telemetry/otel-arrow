// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Exporter used to measure the performance of the OTAP data pipeline.
//!
//! ToDo - Future developments / improvements:
//! - Replace this exporter with a processor that could be combined with a Noop exporter to achieve
//!   the same functionality. The advantage would be to allow performance measurements anywhere in
//!   the pipeline.
//! - Measure the number of memory allocations for the current thread. This would allow measuring
//!   the memory used by the pipeline. This is possible using `mimalloc-sys`.
//! - Measure per-thread CPU usage. This would allow measuring the pipeline's CPU load. This is
//!   possible using the "libc" crate function `getrusage(RUSAGE_THREAD)`.
//! - Measure network usage either via a cgroup or via eBPF.
//! - Measure per-thread perf counters (see crates perfcnt, perfcnt2, or direct perf_event_open
//!   via nix/libc). We could measure task-clock, context switches, page faults, ...
//! - Measure the latency of signals traversing the pipeline. This would require adding a timestamp
//!   in the headers of pdata messages.
//! - Support live reconfiguration via control message.

otap_df_telemetry::otel_component_scope!(
    urn = OTAP_PERF_EXPORTER_URN,
    target = "otel.exporter.perf",
);

pub mod config;

use crate::exporters::perf_exporter::config::Config;
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::ConsumerEffectHandlerExtension;
use otap_df_engine::ExporterFactory;
use otap_df_engine::config::ExporterConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{AckMsg, NodeControlMsg};
use otap_df_engine::error::{Error, ExporterErrorKind};
use otap_df_engine::exporter::ExporterWrapper;
use otap_df_engine::local::exporter as local;
use otap_df_engine::message::{ExporterInbox, Message};
use otap_df_engine::node::NodeId;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_otap::OTAP_EXPORTER_FACTORIES;
use otap_df_otap::metrics::ExporterExportMetrics;
use otap_df_otap::pdata::OtapPdata;
use otap_df_telemetry::common_attributes::{Outcome, SignalOutcomeAttributes};
use otap_df_telemetry::metrics::MeasurementMetricSet;
use serde_json::Value;
use std::sync::Arc;
use std::time::Instant;

/// The URN for the OTAP Perf exporter
pub const OTAP_PERF_EXPORTER_URN: &str = "urn:otel:exporter:perf";

/// Perf Exporter that emits performance data
pub struct PerfExporter {
    config: Config,
    pdata_metrics: MeasurementMetricSet<ExporterExportMetrics>,
}

/// Declares the OTAP Perf exporter as a local exporter factory
///
/// Unsafe code is temporarily used here to allow the use of `distributed_slice` macro
/// This macro is part of the `linkme` crate which is considered safe and well maintained.
#[allow(unsafe_code)]
#[otap_df_engine::component_inventory(category = Exporter)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
pub static PERF_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: OTAP_PERF_EXPORTER_URN,
    create: |pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             exporter_config: &ExporterConfig,
             _capabilities: &otap_df_engine::capability::registry::Capabilities| {
        Ok(ExporterWrapper::local(
            PerfExporter::from_config(pipeline, &node_config.config)?,
            node,
            node_config,
            exporter_config,
        ))
    },
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: otap_df_config::validation::validate_typed_config::<Config>,
};

impl PerfExporter {
    /// creates a perf exporter with the provided config
    #[must_use]
    pub fn new(pipeline_ctx: PipelineContext, config: Config) -> Self {
        let pdata_metrics = ExporterExportMetrics::register(&pipeline_ctx);

        PerfExporter {
            config,
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

    fn terminal_state(&mut self, deadline: Instant) -> TerminalState {
        let mut snapshots = Vec::new();
        snapshots.extend(self.pdata_metrics.terminal_snapshots());

        TerminalState::new(deadline, snapshots)
    }
}

#[async_trait(?Send)]
impl local::Exporter<OtapPdata> for PerfExporter {
    async fn start(
        mut self: Box<Self>,
        mut msg_chan: ExporterInbox<OtapPdata>,
        effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        // init variables for tracking
        // let mut average_pipeline_latency: f64 = 0.0;

        otel_info!(
            "perf_exporter.start",
            frequency_ms = self.config.frequency(),
            message = "Starting Perf Exporter"
        );

        // Loop until a Shutdown event is received.
        loop {
            let msg = msg_chan.recv().await?;
            match msg {
                Message::Control(NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                }) => {
                    _ = metrics_reporter.report_measurement(&mut self.pdata_metrics);
                }
                // ToDo: Handle configuration changes
                Message::Control(NodeControlMsg::Config { .. }) => {}
                Message::Control(NodeControlMsg::Shutdown { deadline, .. }) => {
                    return Ok(self.terminal_state(deadline));
                }
                Message::PData(pdata) => {
                    let export_start = Instant::now();
                    let signal_type = pdata.signal_type();
                    let export_duration = export_start.elapsed();

                    // The local no-op export is complete at dequeue. Record it
                    // independently of whether the upstream Ack can be routed.
                    self.pdata_metrics
                        .with(SignalOutcomeAttributes {
                            signal: signal_type,
                            outcome: Outcome::Success,
                        })
                        .record(export_duration);

                    let _ = effect_handler.notify_ack(AckMsg::new(pdata)).await?;

                    // ToDo (LQ) We need to introduce pdata headers without hpack encoding for data coming from other nodes
                    // decode the headers which are hpack encoded
                    // check for timestamp
                    // get time delta between now and timestamp
                    // calculate average
                    // ToDo Temporary disable latency calculation until we have a better way to add the timestamp header
                    // let mut decoder = Decoder::new();
                    // let header_list =
                    //     decoder
                    //         .decode(&batch.headers)
                    //         .map_err(|_| Error::ExporterError {
                    //             exporter: effect_handler.exporter_id(),
                    //             error: "Failed to decode batch headers".to_owned(),
                    //         })?;
                    // find the timestamp header and parse it
                    // timestamp will be added in the receiver to enable pipeline latency calculation
                    // let timestamp_pair = header_list.iter().find(|(name, _)| name == b"timestamp");
                    // if let Some((_, value)) = timestamp_pair {
                    //     let timestamp =
                    //         decode_timestamp(value).map_err(|error| Error::ExporterError {
                    //             exporter: effect_handler.exporter_id(),
                    //             error,
                    //         })?;
                    //     let current_unix_time = SystemTime::now()
                    //         .duration_since(UNIX_EPOCH)
                    //         .map_err(|error| Error::ExporterError {
                    //             exporter: effect_handler.exporter_id(),
                    //             error: error.to_string(),
                    //         })?;
                    //     let latency = (current_unix_time - timestamp).as_secs_f64();
                    //     average_pipeline_latency = update_average(
                    //         latency,
                    //         average_pipeline_latency,
                    //         self.config.smoothing_factor() as f64,
                    //     );
                    // }

                    // ToDo Report disk, io, cpu, mem usage once gauge metrics are implemented
                }
                _ => {
                    return Err(Error::ExporterError {
                        exporter: effect_handler.exporter_id(),
                        kind: ExporterErrorKind::Other,
                        error: "Unknown control message".to_owned(),
                        source_detail: String::new(),
                    });
                }
            }
        }
    }
}

// uses the exponential moving average formula to update the average
// fn update_average(new_value: f64, old_average: f64, smoothing_factor: f64) -> f64 {
//     // update the average using a exponential moving average which allows new data points to have a greater impact depending on the smoothing factor
//     smoothing_factor * new_value + (1.0 - smoothing_factor) * old_average
// }

// decodes the byte array from the timestamp header and gets the equivalent duration value
// fn decode_timestamp(timestamp: &[u8]) -> Result<Duration, String> {
//     let timestamp_string = std::str::from_utf8(timestamp).map_err(|error| error.to_string())?;
//     let timestamp_parts: Vec<&str> = timestamp_string.split(":").collect();
//     let secs = timestamp_parts[0]
//         .parse::<u64>()
//         .map_err(|error| error.to_string())?;
//     let nanosecs = timestamp_parts[1]
//         .parse::<u32>()
//         .map_err(|error| error.to_string())?;
//
//     Ok(Duration::new(secs, nanosecs))
// }

#[cfg(test)]
mod tests {
    use super::{OTAP_PERF_EXPORTER_URN, PerfExporter};
    use crate::exporters::perf_exporter::config::Config;
    use otap_df_config::node::NodeUserConfig;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::error::Error;
    use otap_df_engine::exporter::ExporterWrapper;
    use otap_df_engine::testing::exporter::TestContext;
    use otap_df_engine::testing::exporter::TestRuntime;
    use otap_df_engine::testing::test_node;
    use otap_df_otap::pdata::OtapPdata;
    use otap_df_otap::testing::create_test_pdata;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
    use std::future::Future;
    use std::ops::Add;
    use std::sync::Arc;
    use std::time::Instant;
    use tokio::time::Duration;

    /// Test closure that sends three PData messages containing one log record each, then shuts down.
    fn scenario()
    -> impl FnOnce(TestContext<OtapPdata>) -> std::pin::Pin<Box<dyn Future<Output = ()>>> {
        |ctx| {
            Box::pin(async move {
                for _ in 0..3 {
                    ctx.send_pdata(create_test_pdata())
                        .await
                        .expect("Failed to send data message");
                }

                // Send shutdown
                ctx.send_shutdown(
                    Instant::now().add(Duration::from_millis(200)),
                    "test complete",
                )
                .await
                .expect("Failed to send Shutdown");
            })
        }
    }

    /// Validation closure that checks the exporter completed successfully.
    fn validation_procedure() -> impl FnOnce(
        TestContext<OtapPdata>,
        Result<(), Error>,
    ) -> std::pin::Pin<Box<dyn Future<Output = ()>>> {
        |_, exporter_result| {
            Box::pin(async move {
                exporter_result.unwrap();
            })
        }
    }

    /// Scenario: A local performance exporter receives three PData messages and then shuts down.
    /// Guarantees: The exporter acknowledges the messages and terminates without an error.
    #[test]
    fn test_exporter_local() {
        let test_runtime = TestRuntime::new();
        let config = Config::new(1000, 0.3, true, true, true, true, true);
        let node_config = Arc::new(NodeUserConfig::new_exporter_config(OTAP_PERF_EXPORTER_URN));
        let controller_ctx = ControllerContext::new(TelemetryRegistryHandle::new());
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let exporter = ExporterWrapper::local(
            PerfExporter::new(pipeline_ctx, config),
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        test_runtime
            .set_exporter(exporter)
            .run_test(scenario())
            .run_validation(validation_procedure());
    }
}
