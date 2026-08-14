// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Clickhouse Exporter for OTAP records
//!
//! This exporter sends OTAP records to ClickHouse instances using the official
//! ClickHouse Rust client.
//! It implements the `Exporter<OtapPdata>` trait
//! for integration with the OTAP dataflow engine.
//!
//! ## Usage
//!
//! This exporter is automatically discovered by the `data-plane` binary via `linkme`.
//! Users configure it in YAML:
//!
//! ```yaml
//! nodes:
//!   - id: clickhouse-exporter
//!     urn: "urn:otel:exporter:clickhouse"
//!     config:
//!       # `endpoint` is an HTTP(S) URL pointing at the ClickHouse HTTP interface (default port 8123).
//!       endpoint: "http://clickhouse.example.db:8123"
//!       database: "otap"
//!       username: "default"
//!       # Secrets can be sourced from the environment, e.g. "${env:CLICKHOUSE_PASSWORD}".
//!       password: ""
//!       # ... additional config
//! ```

use async_trait::async_trait;
use futures::future::LocalBoxFuture;
use linkme::distributed_slice;
use otap_df_config::node::NodeUserConfig;
use otap_df_config::validation::validate_typed_config;
use otap_df_config::{SignalFormat, SignalType};
use otap_df_engine::ExporterFactory;
use otap_df_engine::config::ExporterConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::error::{Error, ExporterErrorKind, format_error_sources};
use otap_df_engine::exporter::ExporterWrapper;
use otap_df_engine::local::exporter::{EffectHandler, Exporter};
use otap_df_engine::message::{ExporterInbox, Message};
use otap_df_engine::node::NodeId;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_otap::OTAP_EXPORTER_FACTORIES;
use otap_df_otap::metrics::ExporterExportMetrics;
use otap_df_otap::pdata::OtapPdata;
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::TryIntoWithOptions;
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otap_df_telemetry::common_attributes::{Outcome, SignalOutcomeAttributes};
use otap_df_telemetry::metrics::MetricSetHandler;
use otap_df_telemetry::metrics::{MeasurementMetricSet, MetricSet};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::exporters::clickhouse_exporter::config::{Config, ConfigPatch};
use crate::exporters::clickhouse_exporter::in_flight::{CompletedWrite, InFlightWrites};
use crate::exporters::clickhouse_exporter::metrics::ClickhouseExporterMetrics;
use crate::exporters::clickhouse_exporter::transform::logs_fast::{
    LogsFastTransform, LogsFastTransformer,
};
use crate::exporters::clickhouse_exporter::transform::transform_batch::BatchTransformer;
use crate::exporters::clickhouse_exporter::writer::ClickHouseWriter;

mod arrays;
#[cfg(feature = "clickhouse-exporter-bench")]
#[doc(hidden)]
pub mod bench_support;
mod config;
mod consts;
mod error;
mod in_flight;
mod metrics;
mod schema;
mod tables;
mod transform;
mod writer;

/// The URN for the Clickhouse exporter
pub const CLICKHOUSE_EXPORTER_URN: &str = "urn:otel:exporter:clickhouse";

/// The list of all payloads that we intend to handle. There seems to be no way to iterate the enum
/// during module setup (we need to generate a static transform plan for all payloads bassed on configs)
const SUPPORTED_ARROW_PAYLOAD_TYPES: &[ArrowPayloadType] = &[
    ArrowPayloadType::ResourceAttrs,
    ArrowPayloadType::ScopeAttrs,
    ArrowPayloadType::Logs,
    ArrowPayloadType::LogAttrs,
    ArrowPayloadType::Spans,
    ArrowPayloadType::SpanAttrs,
    ArrowPayloadType::SpanEventAttrs,
    ArrowPayloadType::SpanEvents,
    ArrowPayloadType::SpanLinkAttrs,
    ArrowPayloadType::SpanLinks,
    // TODO: [support_new_signal] add payload names here
];

/// Clickhouse exporter that sends OTAP data to Clickhouse backend
pub struct ClickhouseExporter {
    config: Config,
    pdata_metrics: MeasurementMetricSet<ExporterExportMetrics>,
    ch_metrics: MetricSet<ClickhouseExporterMetrics>,
}

impl ClickhouseExporter {
    /// Create a new Clickhouse exporter from configuration
    pub fn from_config(
        pipeline_ctx: PipelineContext,
        config: &serde_json::Value,
    ) -> Result<Self, otap_df_config::error::Error> {
        let ch_metrics = pipeline_ctx.register_metrics::<ClickhouseExporterMetrics>();
        let pdata_metrics = ExporterExportMetrics::register(&pipeline_ctx);

        let patch: ConfigPatch = serde_json::from_value(config.clone()).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;
        let config: Config = Config::from_patch(patch);

        Ok(Self {
            config,
            pdata_metrics,
            ch_metrics,
        })
    }

    /// Get exporter configuration
    #[must_use]
    pub fn config(&self) -> &Config {
        &self.config
    }

    fn terminal_state(
        deadline: Instant,
        mut pdata_metrics: MeasurementMetricSet<ExporterExportMetrics>,
        ch_metrics: MetricSet<ClickhouseExporterMetrics>,
    ) -> TerminalState {
        let mut snapshots = Vec::new();

        snapshots.extend(pdata_metrics.terminal_snapshots());
        if ch_metrics.needs_flush() {
            snapshots.push(ch_metrics.snapshot());
        }

        TerminalState::new(deadline, snapshots)
    }

    fn finalize_write(&mut self, completed: CompletedWrite) {
        let CompletedWrite {
            signal_type,
            export_started_at,
            result,
        } = completed;

        match result {
            Ok(written_rows) => {
                for (payload_type, rows) in written_rows {
                    self.ch_metrics.add(rows, payload_type);
                }
                self.pdata_metrics
                    .with(SignalOutcomeAttributes {
                        signal: signal_type,
                        outcome: Outcome::Success,
                    })
                    .record(export_started_at.elapsed());
            }
            Err(error) => {
                self.pdata_metrics
                    .with(SignalOutcomeAttributes {
                        signal: signal_type,
                        outcome: Outcome::Failure,
                    })
                    .record(export_started_at.elapsed());
                otap_df_telemetry::otel_warn!(
                    "clickhouse.exporter.write.error",
                    message = format!("Error writing batch to clickhouse: {error}"),
                    signal_type = format!("{signal_type:?}"),
                );
            }
        }
    }
}

/// Register Clickhouse exporter with the OTAP exporter factory
///
/// Unsafe code is temporarily used here to allow the use of `distributed_slice` macro
/// This macro is part of the `linkme` crate which is considered safe and well maintained.
#[allow(unsafe_code)]
#[otap_df_engine::component_inventory(category = Exporter)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
pub static CLICKHOUSE_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: CLICKHOUSE_EXPORTER_URN,
    create: |pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             exporter_config: &ExporterConfig,
             _capabilities: &otap_df_engine::capability::registry::Capabilities| {
        Ok(ExporterWrapper::local(
            ClickhouseExporter::from_config(pipeline, &node_config.config)?,
            node,
            node_config,
            exporter_config,
        ))
    },
    validate_config: validate_typed_config::<ConfigPatch>,
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
};

#[async_trait(?Send)]
impl Exporter<OtapPdata> for ClickhouseExporter {
    async fn start(
        mut self: Box<Self>,
        mut inbox: ExporterInbox<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        let exporter_id = effect_handler.exporter_id();
        otap_df_telemetry::otel_info!(
            "clickhouse.exporter.start",
            message = "Clickhouse exporter starting",
            endpoint = self.config.endpoint,
            database = self.config.database,
            username = self.config.username,
            max_in_flight = self.config.max_in_flight.get()
        );

        let mut batch_transformer = BatchTransformer::new();
        let mut logs_fast_transformer = LogsFastTransformer::default();
        let clickhouse_writer =
            Rc::new(ClickHouseWriter::new(&self.config).await.map_err(|e| {
                Error::ExporterError {
                    exporter: exporter_id.clone(),
                    kind: ExporterErrorKind::Connect,
                    error: format!("clickhouse writer initialization error: {e}"),
                    source_detail: format_error_sources(&e),
                }
            })?);
        let mut in_flight_writes = InFlightWrites::new(self.config.max_in_flight);

        // Start periodic telemetry collection (internal metrics)
        let telemetry_cancel_handle = effect_handler
            .start_periodic_telemetry(Duration::from_secs(1))
            .await?;

        // Message loop
        loop {
            let accepting_pdata = !in_flight_writes.is_at_capacity();
            let message = tokio::select! {
                biased;

                completed = in_flight_writes.next_completion(), if !in_flight_writes.is_empty() => {
                    if let Some(completed) = completed {
                        self.finalize_write(completed);
                    }
                    continue;
                }
                message = inbox.recv_when(accepting_pdata) => message?,
            };

            match message {
                Message::Control(NodeControlMsg::Shutdown { deadline, .. }) => {
                    otap_df_telemetry::otel_info!(
                        "clickhouse.exporter.shutdown",
                        message = "Clickhouse exporter shutting down",
                    );
                    let abandoned = in_flight_writes
                        .drain_until(tokio::time::Instant::from_std(deadline), |completed| {
                            self.finalize_write(completed);
                        })
                        .await;
                    if abandoned > 0 {
                        otap_df_telemetry::otel_warn!(
                            "clickhouse.exporter.shutdown.deadline_exceeded",
                            message = "ClickHouse writes abandoned at the shutdown deadline",
                            abandoned_writes = abandoned,
                        );
                    }
                    let _ = telemetry_cancel_handle.cancel().await;
                    return Ok(Self::terminal_state(
                        deadline,
                        self.pdata_metrics,
                        self.ch_metrics,
                    ));
                }
                Message::Control(NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                }) => {
                    _ = metrics_reporter.report_measurement(&mut self.pdata_metrics);
                    _ = metrics_reporter.report(&mut self.ch_metrics);
                }
                Message::PData(pdata) => {
                    let export_started_at = Instant::now();
                    let signal_type = pdata.signal_type();
                    let signal_format = pdata.signal_format();

                    let (_context, payload) = pdata.into_parts();

                    let mut arrow_records: OtapArrowRecords = match payload.try_into_with_default()
                    {
                        Ok(arrow_records) => arrow_records,
                        Err(e) => {
                            self.pdata_metrics
                                .with(SignalOutcomeAttributes {
                                    signal: signal_type,
                                    outcome: Outcome::Failure,
                                })
                                .record(export_started_at.elapsed());
                            otap_df_telemetry::otel_warn!(
                                "clickhouse.exporter.convert.error",
                                message =
                                    format!("Failed to convert payload to OtapArrowRecords: {e:?}"),
                                signal_type = format!("{:?}", signal_type),
                            );
                            continue;
                        }
                    };

                    // decode the transport optimized IDs before joining payloads against them.
                    arrow_records
                        .decode_transport_optimized_ids()
                        .map_err(|e| {
                            self.pdata_metrics
                                .with(SignalOutcomeAttributes {
                                    signal: signal_type,
                                    outcome: Outcome::Failure,
                                })
                                .record(export_started_at.elapsed());
                            let source_detail = format_error_sources(&e);
                            Error::ExporterError {
                                exporter: exporter_id.clone(),
                                kind: ExporterErrorKind::Other,
                                error: format!("Failed to decode transport optimized IDs: {e}"),
                                source_detail,
                            }
                        })?;

                    let transform_result = if signal_type == SignalType::Logs
                        && signal_format == SignalFormat::OtapRecords
                    {
                        match logs_fast_transformer.try_apply(&arrow_records) {
                            Ok(LogsFastTransform::Applied(batch)) => {
                                self.ch_metrics.record_log_fast_path();
                                Ok(HashMap::from([(ArrowPayloadType::Logs, batch)]))
                            }
                            Ok(LogsFastTransform::NotApplicable(_)) => {
                                self.ch_metrics.record_log_transform_fallback();
                                batch_transformer.apply_plan(arrow_records)
                            }
                            Err(error) => Err(error),
                        }
                    } else {
                        if signal_type == SignalType::Logs {
                            self.ch_metrics.record_log_transform_fallback();
                        }
                        batch_transformer.apply_plan(arrow_records)
                    };

                    let write_batches = match transform_result {
                        Ok(batches) => batches,
                        Err(e) => {
                            self.pdata_metrics
                                .with(SignalOutcomeAttributes {
                                    signal: signal_type,
                                    outcome: Outcome::Failure,
                                })
                                .record(export_started_at.elapsed());
                            otap_df_telemetry::otel_warn!(
                                "clickhouse.exporter.transform.error",
                                message = "Error transforming batch for export.",
                                error = e.to_string(),
                                signal_type = format!("{:?}", signal_type),
                            );
                            continue;
                        }
                    };
                    let writer = Rc::clone(&clickhouse_writer);
                    let write_future: LocalBoxFuture<'static, CompletedWrite> =
                        Box::pin(async move {
                            CompletedWrite {
                                signal_type,
                                export_started_at,
                                result: writer.write_batches(&write_batches).await,
                            }
                        });
                    in_flight_writes.push(write_future);
                }
                _ => {
                    // Ignore other messages
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: the ClickHouse exporter is registered with its public component URN.
    /// Guarantees: configuration references continue to resolve to the exporter factory.
    #[test]
    fn test_urn_constant() {
        assert_eq!(CLICKHOUSE_EXPORTER_URN, "urn:otel:exporter:clickhouse");
    }
}
