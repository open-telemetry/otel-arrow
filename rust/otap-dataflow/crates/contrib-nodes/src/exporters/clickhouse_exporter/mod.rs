// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Clickhouse Exporter for OTAP records
//!
//! This exporter sends OTAP records to clickhouse instances.
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
//!       endpoint: "https://clickhouse.example.db"
//!       database: "otap"
//!       user: "default"
//!       password: ""
//!       # ... additional config
//! ```

use async_trait::async_trait;
use linkme::distributed_slice;
use otap::OTAP_EXPORTER_FACTORIES;
use otap::metrics::ExporterPDataMetrics;
use otap::pdata::OtapPdata;
use otap_df_config::node::NodeUserConfig;
use otap_df_config::validation::validate_typed_config;
use otap_df_engine::ExporterFactory;
use otap_df_engine::config::ExporterConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::error::{Error, ExporterErrorKind, format_error_sources};
use otap_df_engine::exporter::ExporterWrapper;
use otap_df_engine::local::exporter::{EffectHandler, Exporter};
use otap_df_engine::message::{ExporterMessageChannel, Message};
use otap_df_engine::node::NodeId;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry::metrics::MetricSetHandler;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::clickhouse_exporter::config::{Config, ConfigPatch};
use crate::clickhouse_exporter::metrics::ClickhouseExporterMetrics;
use crate::clickhouse_exporter::transform::transform_batch::BatchTransformer;
use crate::clickhouse_exporter::writer::ClickHouseWriter;

mod arrays;
mod config;
mod consts;
mod error;
mod idgen;
mod metrics;
mod schema;
mod tables;
mod transform;
mod writer;

/// The URN for the Clickhouse exporter
pub const CLICKHOUSE_EXPORTER_URN: &str = "urn:otel:exporter:clickhouse";

/// The list of payloads which (possibly) correspond to a table in clickhouse.
/// Payloads that always get mapped inline (span links, events, etc) do not appear here.
const TABLE_BACKED_ARROW_PAYLOAD_TYPES: &[ArrowPayloadType] = &[
    ArrowPayloadType::ResourceAttrs,
    ArrowPayloadType::ScopeAttrs,
    ArrowPayloadType::Logs,
    ArrowPayloadType::LogAttrs,
    ArrowPayloadType::Spans,
    ArrowPayloadType::SpanAttrs,
    // TODO: [support_new_signal] add payload names here
];

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
    pdata_metrics: MetricSet<ExporterPDataMetrics>,
    ch_metrics: MetricSet<ClickhouseExporterMetrics>,
}

impl ClickhouseExporter {
    /// Create a new Clickhouse exporter from configuration
    pub fn from_config(
        pipeline_ctx: PipelineContext,
        config: &serde_json::Value,
    ) -> Result<Self, otap_df_config::error::Error> {
        let ch_metrics = pipeline_ctx.register_metrics::<ClickhouseExporterMetrics>();
        let pdata_metrics = pipeline_ctx.register_metrics::<ExporterPDataMetrics>();

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
        pdata_metrics: MetricSet<ExporterPDataMetrics>,
        ch_metrics: MetricSet<ClickhouseExporterMetrics>,
    ) -> TerminalState {
        let mut snapshots = Vec::new();

        if pdata_metrics.needs_flush() {
            snapshots.push(pdata_metrics.snapshot());
        }
        if ch_metrics.needs_flush() {
            snapshots.push(ch_metrics.snapshot());
        }

        TerminalState::new(deadline, snapshots)
    }
}

/// Register Clickhouse exporter with the OTAP exporter factory
///
/// Unsafe code is temporarily used here to allow the use of `distributed_slice` macro
/// This macro is part of the `linkme` crate which is considered safe and well maintained.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
pub static CLICKHOUSE_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: CLICKHOUSE_EXPORTER_URN,
    create: |pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             exporter_config: &ExporterConfig| {
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
        mut msg_chan: ExporterMessageChannel<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        let exporter_id = effect_handler.exporter_id();
        otap_df_telemetry::otel_info!(
            "clickhouse.exporter.start",
            message = "Clickhouse exporter starting",
            endpoint = self.config.endpoint,
            database = self.config.database,
            username = self.config.username
        );

        let mut batch_transformer = BatchTransformer::new_from_config(&self.config);
        let clickhouse_writer =
            ClickHouseWriter::new(&self.config)
                .await
                .map_err(|e| Error::ExporterError {
                    exporter: exporter_id.clone(),
                    kind: ExporterErrorKind::Connect,
                    error: format!("clickhouse writer initialization error: {e}"),
                    source_detail: format_error_sources(&e),
                })?;

        // Start periodic telemetry collection (internal metrics)
        let telemetry_cancel_handle = effect_handler
            .start_periodic_telemetry(Duration::from_secs(1))
            .await?;

        // Message loop
        loop {
            match msg_chan.recv().await? {
                Message::Control(NodeControlMsg::Shutdown { deadline, .. }) => {
                    otap_df_telemetry::otel_info!(
                        "clickhouse.exporter.shutdown",
                        message = "Clickhouse exporter shutting down",
                    );
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
                    _ = metrics_reporter.report(&mut self.pdata_metrics);
                    _ = metrics_reporter.report(&mut self.ch_metrics);
                }
                Message::PData(pdata) => {
                    let signal_type = pdata.signal_type();

                    // Mark as consumed for this signal
                    self.pdata_metrics.inc_consumed(signal_type);

                    let (_context, payload) = pdata.into_parts();

                    let arrow_records: OtapArrowRecords = match payload.try_into() {
                        Ok(arrow_records) => arrow_records,
                        Err(_) => {
                            self.pdata_metrics.inc_failed(signal_type);
                            continue;
                        }
                    };

                    let write_batches = match batch_transformer.apply_plan(arrow_records) {
                        Ok(batches) => batches,
                        Err(e) => {
                            self.pdata_metrics.inc_failed(signal_type);
                            otap_df_telemetry::otel_warn!(
                                "clickhouse.exporter.transform.error",
                                message = "Error transforming batch for export.",
                                error = e.to_string(),
                                signal_type = format!("{:?}", signal_type),
                            );
                            continue;
                        }
                    };

                    match clickhouse_writer
                        .write_batches(&write_batches, &mut self.ch_metrics)
                        .await
                    {
                        Ok(_) => {
                            self.pdata_metrics.inc_exported(signal_type);
                        }
                        Err(e) => {
                            // TODO - this is not the error handling we want long term.  eventually we
                            // should have the concept of retryable & non-retryable errors and use Nack
                            //  message + a Retry processor to handle this gracefully
                            // https://github.com/open-telemetry/otel-arrow/issues/504
                            self.pdata_metrics.inc_failed(signal_type);
                            otap_df_telemetry::otel_warn!(
                                "clickhouse.exporter.write.error",
                                message =
                                    format!("Error writing batch to clickhouse: {}", e.to_string()),
                                signal_type = format!("{:?}", signal_type),
                            );
                            continue;
                        }
                    }
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

    #[test]
    fn test_urn_constant() {
        assert_eq!(CLICKHOUSE_EXPORTER_URN, "urn:otel:exporter:clickhouse");
    }
}
