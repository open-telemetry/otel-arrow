// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Handles the output flow of the debug processor

use crate::pdata::{OtapPdata, OtlpProtoBytes};
use async_trait::async_trait;
use otap_df_config::PortName;
use otap_df_engine::error::Error;
use otap_df_engine::local::processor as local;
use otap_df_engine::node::NodeId;
use otel_arrow_rust::proto::opentelemetry::{
    common::v1::{AnyValue, InstrumentationScope},
    logs::v1::{LogRecord, LogsData, ResourceLogs, ScopeLogs, SeverityNumber},
    resource::v1::Resource,
};
use prost::Message as _;
use serde::Deserialize;
use tokio::fs::File;
use tokio::io::{AsyncWrite, AsyncWriteExt};

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum OutputMode {
    Console,
    Outports(Vec<PortName>),
    File(String),
}

#[async_trait(?Send)]
pub trait DebugOutput {
    async fn output_metrics(&mut self, metric_request: MetricsData) -> Result<(), Error>;
    async fn output_traces(&mut self, trace_request: TracesData) -> Result<(), Error>;
    async fn output_logs(&mut self, log_request: LogsData) -> Result<(), Error>;
}

/// A wrapper around AsyncWrite that simplifies error handling for debug output
pub struct DebugOutputWriter {
    writer: Box<dyn AsyncWrite + Unpin>,
    processor_id: NodeId,
    marshaler: Option<Box<dyn ViewMarshaler>>,
    display_mode: DisplayMode,
}

impl DebugOutputWriter {
    pub async fn new(
        output_file: Option<String>,
        processor_id: NodeId,
        verbosity: Verbosity,
        display_mode: DisplayMode,
    ) -> Result<Self, Error> {
        let writer: Box<dyn AsyncWrite + Unpin> = match output_file {
            Some(file_name) => {
                let file = File::options()
                    .write(true)
                    .append(true)
                    .create(true)
                    .open(file_name)
                    .await
                    .map_err(|e| Error::ProcessorError {
                        processor: processor_id.clone(),
                        error: format!("File error: {e}"),
                    })?;
                Box::new(file)
            }
            None => Box::new(tokio::io::stdout()),
        };
        // determine which marshler to use
        let marshaler: Box<dyn ViewMarshaler> = match verbosity {
            Verbosity::Detailed => Some(Box::new(DetailedViewMarshaler)),
            Verbosity::Normal => Some(Box::new(NormalViewMarshaler)),
            Verbosity::Basic => None,
        };

        Ok(Self {
            writer,
            processor_id,
            marshaler,
            display_mode,
        })
    }

    async fn output_message(&mut self, data: &str) -> Result<(), Error> {
        self.writer
            .write_all(data.as_bytes())
            .await
            .map_err(|e| Error::ProcessorError {
                processor: self.processor_id.clone(),
                error: format!("Write error: {e}"),
            })
    }
}

#[async_trait(?Send)]
impl DebugOutput for DebugOutputWriter {
    async fn output_metrics(&mut self, metric_request: MetricsData) -> Result<(), Error> {
        match self.display_mode {
            DisplayMode::Batch => {
                let report = self.marshaler.marshal_metrics(metric_request);
                self.output_message(format!("{report}\n").as_str()).await?;
            }
            DisplayMode::Signal => {
                let metric_signals = metric_request
                    .resource_metrics
                    .into_iter()
                    .flat_map(|resource| resource.scope_metrics)
                    .flat_map(|scope| scope.metrics);
                for (index, metric) in metric_signals.enumerate() {
                    let report = self.marshaler.marshal_metric_signal(&metric, index);
                    self.output_message(format!("{report}\n").as_str()).await?;
                }
            }
        }
        Ok(())
    }
    async fn output_traces(&mut self, trace_request: TracesData) -> Result<(), Error> {
        match self.display_mode {
            DisplayMode::Batch => {
                let report = self.marshaler.marshal_traces(trace_request);
                self.output_message(format!("{report}\n").as_str()).await?;
            }
            DisplayMode::Signal => {
                let span_signals = trace_request
                    .resource_spans
                    .into_iter()
                    .flat_map(|resource| resource.scope_spans)
                    .flat_map(|scope| scope.spans);
                for (index, span) in span_signals.enumerate() {
                    let report = self.marshaler.marshal_span_signal(&span, index);
                    self.output_message(format!("{report}\n").as_str()).await?;
                }
            }
        }
        Ok(())
    }
    async fn output_logs(&mut self, log_request: LogsData) -> Result<(), Error> {
        match self.display_mode {
            DisplayMode::Batch => {
                let report = self.marshaler.marshal_logs(log_request);
                self.output_message(format!("{report}\n").as_str()).await?;
            }
            DisplayMode::Signal => {
                let log_signals = log_request
                    .resource_logs
                    .into_iter()
                    .flat_map(|resource| resource.scope_logs)
                    .flat_map(|scope| scope.log_records);
                for (index, log_record) in log_signals.enumerate() {
                    let report = self.marshaler.marshal_log_signal(&log_record, index);
                    self.output_message(format!("{report}\n").as_str()).await?;
                }
            }
        }
        Ok(())
    }
}

pub struct DebugOutputPorts {
    ports: Vec<PortName>,
    display_mode: DisplayMode,
    effect_handler: local::EffectHandler<OtapPdata>,
}

impl DebugOutputPorts {
    pub fn new(
        ports: Vec<PortName>,
        display_mode: DisplayMode,
        effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<Self, Error> {
        // check that the ports are actually connected
        let connected_ports = effect_handler.connected_ports();
        // collect ports that are not connected
        let mut invalid_ports = ports
            .iter()
            .filter(|port| !connected_ports.contains(port))
            .peekable();
        // raise error if ports are not connected
        if invalid_ports.peek().is_some() {
            let invalid_ports_list = invalid_ports
                .map(|port_name| port_name.as_ref())
                .collect::<Vec<&str>>()
                .join(", ");
            let connected_ports_list = connected_ports
                .iter()
                .map(|port_name| port_name.as_ref())
                .collect::<Vec<&str>>()
                .join(", ");
            return Err(Error::ProcessorError {
                processor: effect_handler.processor_id(),
                error: format!(
                    "The following ports are not connected [{invalid_ports_list}], these are connected ports to this node [{connected_ports_list}]"
                ),
            });
        }

        Ok(Self {
            ports,
            display_mode,
            effect_handler,
        })
    }

    fn split_metrics(&self, metric_request: MetricsData) -> Vec<MetricsData> {
        metric_request
            .resource_metrics
            .into_iter()
            .flat_map(|resource_metric| {
                // Clone metadata that must be replicated for each metric
                let resource = resource_metric.resource.clone();
                let resource_schema_url = resource_metric.schema_url.clone();
                resource_metric
                    .scope_metrics
                    .into_iter()
                    .flat_map(move |scope_metrics| {
                        let scope = scope_metrics.scope.clone();
                        let scope_schema_url = scope_metric.schema_url.clone();
                        scope_metric
                            .metrics
                            .into_iter()
                            .map(move |metric| MetricsData {
                                resource_metrics: vec![ResourceMetrics {
                                    resource: resource.clone(),
                                    scope_metrics: vec![ScopeMetrics {
                                        scope: scope.clone(),
                                        metrics: vec![metric],
                                        schema_url: scope_schema_url.clone(),
                                    }],
                                    schema_url: resource_schema_url.clone(),
                                }],
                            })
                    })
            })
            .collect()
    }

    fn split_traces(&self, trace_request: TracesData) -> Vec<TracesData> {
        trace_request
            .resource_spans
            .into_iter()
            .flat_map(|resource_span| {
                // Clone metadata that must be replicated for each span
                let resource = resource_span.resource.clone();
                let resource_schema_url = resource_span.schema_url.clone();
                resource_span
                    .scope_spans
                    .into_iter()
                    .flat_map(move |scope_spans| {
                        let scope = scope_spans.scope.clone();
                        let scope_schema_url = scope_span.schema_url.clone();
                        scope_span.spans.into_iter().map(move |span| TracesData {
                            resource_traces: vec![ResourceSpans {
                                resource: resource.clone(),
                                scope_traces: vec![ScopeSpans {
                                    scope: scope.clone(),
                                    spans: vec![span],
                                    schema_url: scope_schema_url.clone(),
                                }],
                                schema_url: resource_schema_url.clone(),
                            }],
                        })
                    })
            })
            .collect()
    }

    fn split_logs(&self, log_request: LogsData) -> Vec<LogsData> {
        log_request
            .resource_logs
            .into_iter()
            .flat_map(|resource_log| {
                // Clone metadata that must be replicated for each log
                let resource = resource_log.resource.clone();
                let resource_schema_url = resource_log.schema_url.clone();
                resource_log
                    .scope_logs
                    .into_iter()
                    .flat_map(move |scope_logs| {
                        let scope = scope_logs.scope.clone();
                        let scope_schema_url = scope_log.schema_url.clone();
                        scope_log
                            .log_records
                            .into_iter()
                            .map(move |log_record| LogsData {
                                resource_logs: vec![ResourceLogs {
                                    resource: resource.clone(),
                                    scope_logs: vec![ScopeLogs {
                                        scope: scope.clone(),
                                        log_records: vec![log_record],
                                        schema_url: scope_schema_url.clone(),
                                    }],
                                    schema_url: resource_schema_url.clone(),
                                }],
                            })
                    })
            })
            .collect()
    }
}


#[async_trait(?Send)]
impl DebugOutput for DebugOutputPorts {
    async fn output_message(&mut self, message: OtapPdata) -> Result<(), Error> {
        for port in self.ports.iter().cloned() {
            self.effect_handler
                .send_message_to(port, otlp_logs_bytes.clone())
                .await?;
        }

        Ok(())
    }

    async fn output_metrics(&mut self, metric_request: MetricsData) -> Result<(), Error> {
        match self.display_mode {
            DisplayMode::Batch => {
                self.output_message(OTLPSignal::Metrics(metric_request).try_into()?)
                    .await?;
            }
            DisplayMode::Signal => {
                let metric_signals = self.split_metrics(metric_request);
                for metric in metric_signals.into_iter() {
                    self.output_message(OTLPSignal::Metrics(metric).try_into()?).await?;
                }
            }
        }
    }
    async fn output_traces(&mut self, trace_request: TracesData) -> Result<(), Error> {
        match self.display_mode {
            DisplayMode::Batch => {
                self.output_message(OTLPSignal::Traces(trace_request).try_into()?)
                    .await?;
            }
            DisplayMode::Signal => {
                let trace_signals = self.split_traces(trace_request);
                for trace in trace_signals.into_iter() {
                    self.output_message(OTLPSignal::Traces(trace_request).try_into()?)
                        .await?;
                }
            }
        }
    }
    async fn output_logs(&mut self, log_request: LogsData) -> Result<(), Error> {
        match self.display_mode {
            DisplayMode::Batch => {
                self.output_message(OTLPSignal::Logs(log_request).try_into()?).await?;
            }
            DisplayMode::Signal => {
                let log_signals = self.split_logs(log_request);
                for log in log_signals.into_iter() {
                    self.output_message(OTLPSignal::Logs(log_request).try_into()?).await?;
                }
            }
        }
    }
}
