// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the Perf Exporter node
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

pub mod config;
pub mod metrics;

use crate::exporters::perf_exporter::config::Config;
use crate::exporters::perf_exporter::metrics::PerfExporterPdataMetrics;
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::SignalType;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::ConsumerEffectHandlerExtension;
use otap_df_engine::ExporterFactory;
use otap_df_engine::config::ExporterConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{AckMsg, NodeControlMsg};
use otap_df_engine::error::{Error, ExporterErrorKind};
use otap_df_engine::exporter::ExporterWrapper;
use otap_df_engine::local::exporter as local;
use otap_df_engine::message::{Message, MessageChannel};
use otap_df_engine::node::NodeId;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_otap::OTAP_EXPORTER_FACTORIES;
use otap_df_otap::metrics::ExporterPDataMetrics;
use otap_df_otap::pdata::OtapPdata;
use otap_df_pdata::otap::OtapArrowRecords;
use otap_df_telemetry::metrics::{MetricSet, MetricSetHandler};
use otap_df_telemetry::otel_info;
use serde_json::Value;
use std::sync::Arc;
use std::time::Instant;
use tokio::time::Duration;

/// The URN for the OTAP Perf exporter
pub const OTAP_PERF_EXPORTER_URN: &str = "urn:otel:exporter:perf";

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
	wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
	validate_config: otap_df_config::validation::validate_typed_config::<Config>,
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

	fn terminal_state(&self, deadline: Instant) -> TerminalState {
		let mut snapshots = Vec::new();

		if self.metrics.needs_flush() {
			snapshots.push(self.metrics.snapshot());
		}

		if self.pdata_metrics.needs_flush() {
			snapshots.push(self.pdata_metrics.snapshot());
		}

		TerminalState::new(deadline, snapshots)
	}
}

#[async_trait(?Send)]
impl local::Exporter<OtapPdata> for PerfExporter {
	async fn start(
		mut self: Box<Self>,
		mut msg_chan: MessageChannel<OtapPdata>,
		effect_handler: local::EffectHandler<OtapPdata>,
	) -> Result<TerminalState, Error> {
		otel_info!(
			"perf_exporter.start",
			frequency_ms = self.config.frequency(),
			message = "Starting Perf Exporter"
		);

		let timer_cancel_handle = effect_handler
			.start_periodic_telemetry(Duration::from_millis(self.config.frequency()))
			.await?;

		loop {
			let msg = msg_chan.recv().await?;
			match msg {
				Message::Control(NodeControlMsg::CollectTelemetry {
					mut metrics_reporter,
				}) => {
					_ = metrics_reporter.report(&mut self.metrics);
					_ = metrics_reporter.report(&mut self.pdata_metrics);
				}
				Message::Control(NodeControlMsg::Config { .. }) => {}
				Message::Control(NodeControlMsg::Shutdown { deadline, .. }) => {
					_ = timer_cancel_handle.cancel().await;
					return Ok(self.terminal_state(deadline));
				}
				Message::PData(mut pdata) => {
					let signal_type = pdata.signal_type();
					self.pdata_metrics.inc_consumed(signal_type);

					let payload = pdata.take_payload();
					_ = effect_handler.notify_ack(AckMsg::new(pdata)).await?;

					let batch: OtapArrowRecords = match payload.try_into() {
						Ok(batch) => batch,
						Err(_) => {
							self.pdata_metrics.inc_failed(signal_type);
							continue;
						}
					};

					match signal_type {
						SignalType::Metrics => {
							self.metrics.metrics.add(batch.num_items() as u64);
						}
						SignalType::Logs => {
							self.metrics.logs.add(batch.num_items() as u64);
						}
						SignalType::Traces => {
							self.metrics.spans.add(batch.num_items() as u64);
						}
					}

					self.pdata_metrics.inc_exported(signal_type);
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
	use tokio::time::{Duration, sleep};

	fn scenario()
	-> impl FnOnce(TestContext<OtapPdata>) -> std::pin::Pin<Box<dyn Future<Output = ()>>> {
		|ctx| {
			Box::pin(async move {
				for _ in 0..3 {
					ctx.send_pdata(create_test_pdata())
						.await
						.expect("Failed to send data message");
				}

				_ = sleep(Duration::from_millis(5000));

				ctx.send_shutdown(
					Instant::now().add(Duration::from_millis(200)),
					"test complete",
				)
				.await
				.expect("Failed to send Shutdown");
			})
		}
	}

	fn validation_procedure(
		telemetry_registry_handle: TelemetryRegistryHandle,
	) -> impl FnOnce(
		TestContext<OtapPdata>,
		Result<(), Error>,
	) -> std::pin::Pin<Box<dyn Future<Output = ()>>> {
		|_, exporter_result| {
			Box::pin(async move {
				exporter_result.unwrap();

				telemetry_registry_handle.visit_current_metrics(
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
		let telemetry_registry_handle = TelemetryRegistryHandle::new();
		let controller_ctx = ControllerContext::new(telemetry_registry_handle.clone());
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
			.run_validation(validation_procedure(telemetry_registry_handle));
	}
}
