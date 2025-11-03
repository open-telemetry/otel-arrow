// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Handles the output flow of the debug processor

use crate::debug_processor::DisplayMode;
use crate::debug_processor::Verbosity;
use crate::debug_processor::detailed_marshaler::DetailedViewMarshaler;
use crate::debug_processor::marshaler::ViewMarshaler;
use crate::debug_processor::normal_marshaler::NormalViewMarshaler;
use crate::debug_processor::sampling::Sampler;
use crate::fake_data_generator::config::OTLPSignal;
use crate::pdata::OtapPdata;
use async_trait::async_trait;
use otap_df_config::PortName;
use otap_df_engine::error::{Error, ProcessorErrorKind, format_error_sources};
use otap_df_engine::local::processor as local;
use otap_df_engine::node::NodeId;
use otel_arrow_rust::proto::opentelemetry::{
    logs::v1::{LogsData, ResourceLogs, ScopeLogs},
    metrics::v1::{MetricsData, ResourceMetrics, ScopeMetrics},
    trace::v1::{ResourceSpans, ScopeSpans, TracesData},
};
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
    async fn output_message(&mut self, message: &str) -> Result<(), Error>;
    async fn output_metrics(
        &mut self,
        metric_request: MetricsData,
        sampler: &mut Sampler,
    ) -> Result<(), Error>;
    async fn output_traces(
        &mut self,
        trace_request: TracesData,
        sampler: &mut Sampler,
    ) -> Result<(), Error>;
    async fn output_logs(
        &mut self,
        log_request: LogsData,
        sampler: &mut Sampler,
    ) -> Result<(), Error>;
    fn is_basic(&self) -> bool;
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
                    .map_err(|e| {
                        let source_detail = format_error_sources(&e);
                        Error::ProcessorError {
                            processor: processor_id.clone(),
                            kind: ProcessorErrorKind::Configuration,
                            error: format!("File error: {e}"),
                            source_detail,
                        }
                    })?;
                Box::new(file)
            }
            None => Box::new(tokio::io::stdout()),
        };
        // determine which marshler to use
        let marshaler: Option<Box<dyn ViewMarshaler>> = match verbosity {
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
}

#[async_trait(?Send)]
impl DebugOutput for DebugOutputWriter {
    async fn output_message(&mut self, message: &str) -> Result<(), Error> {
        self.writer
            .write_all(message.as_bytes())
            .await
            .map_err(|e| {
                let source_detail = format_error_sources(&e);
                Error::ProcessorError {
                    processor: self.processor_id.clone(),
                    kind: ProcessorErrorKind::Transport,
                    error: format!("Write error: {e}"),
                    source_detail,
                }
            })
    }
    async fn output_metrics(
        &mut self,
        metric_request: MetricsData,
        sampler: &mut Sampler,
    ) -> Result<(), Error> {
        if let Some(marshaler) = self.marshaler.take() {
            match self.display_mode {
                DisplayMode::Batch => {
                    let send_message = async || -> Result<(), Error> {
                        let report = marshaler.marshal_metrics(metric_request);
                        self.output_message(format!("{report}\n").as_str()).await
                    };
                    sampler.sample(send_message).await?;
                }
                DisplayMode::Signal => {
                    let metric_signals = metric_request
                        .resource_metrics
                        .into_iter()
                        .flat_map(|resource| resource.scope_metrics)
                        .flat_map(|scope| scope.metrics);
                    for (index, metric) in metric_signals.enumerate() {
                        let send_message = async || -> Result<(), Error> {
                            let report = marshaler.marshal_metric_signal(&metric, index);
                            self.output_message(format!("{report}\n").as_str()).await
                        };
                        sampler.sample(send_message).await?;
                    }
                }
            }
            self.marshaler = Some(marshaler);
        }
        Ok(())
    }
    async fn output_traces(
        &mut self,
        trace_request: TracesData,
        sampler: &mut Sampler,
    ) -> Result<(), Error> {
        if let Some(marshaler) = self.marshaler.take() {
            match self.display_mode {
                DisplayMode::Batch => {
                    let send_message = async || -> Result<(), Error> {
                        let report = marshaler.marshal_traces(trace_request);
                        self.output_message(format!("{report}\n").as_str()).await
                    };
                    sampler.sample(send_message).await?;
                }
                DisplayMode::Signal => {
                    let span_signals = trace_request
                        .resource_spans
                        .into_iter()
                        .flat_map(|resource| resource.scope_spans)
                        .flat_map(|scope| scope.spans);
                    for (index, span) in span_signals.enumerate() {
                        let send_message = async || -> Result<(), Error> {
                            let report = marshaler.marshal_span_signal(&span, index);
                            self.output_message(format!("{report}\n").as_str()).await
                        };
                        sampler.sample(send_message).await?;
                    }
                }
            }
            self.marshaler = Some(marshaler);
        }
        Ok(())
    }
    async fn output_logs(
        &mut self,
        log_request: LogsData,
        sampler: &mut Sampler,
    ) -> Result<(), Error> {
        if let Some(marshaler) = self.marshaler.take() {
            match self.display_mode {
                DisplayMode::Batch => {
                    let send_message = async || -> Result<(), Error> {
                        let report = marshaler.marshal_logs(log_request);
                        self.output_message(format!("{report}\n").as_str()).await
                    };
                    sampler.sample(send_message).await?;
                }
                DisplayMode::Signal => {
                    let log_signals = log_request
                        .resource_logs
                        .into_iter()
                        .flat_map(|resource| resource.scope_logs)
                        .flat_map(|scope| scope.log_records);
                    for (index, log_record) in log_signals.enumerate() {
                        let send_message = async || -> Result<(), Error> {
                            let report = marshaler.marshal_log_signal(&log_record, index);
                            self.output_message(format!("{report}\n").as_str()).await
                        };
                        sampler.sample(send_message).await?;
                    }
                }
            }
            self.marshaler = Some(marshaler);
        }
        Ok(())
    }

    fn is_basic(&self) -> bool {
        // if no marshaler is set then the verbosity set is basic
        self.marshaler.is_none()
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
                kind: ProcessorErrorKind::Configuration,
                error: format!(
                    "The following ports are not connected [{invalid_ports_list}], these are connected ports to this node [{connected_ports_list}]"
                ),
                source_detail: "".to_owned(),
            });
        }

        Ok(Self {
            ports,
            display_mode,
            effect_handler,
        })
    }

    // helper functions to split the view objects into individual signals
    fn split_metrics(&self, metric_request: MetricsData) -> Vec<MetricsData> {
        metric_request
            .resource_metrics
            .into_iter()
            .flat_map(|resource_metric| {
                // Clone metadata that must be replicated for each metric
                let resource = resource_metric.resource;
                let resource_schema_url = resource_metric.schema_url;
                resource_metric
                    .scope_metrics
                    .into_iter()
                    .flat_map(move |scope_metric| {
                        let scope = scope_metric.scope.clone();
                        let scope_schema_url = scope_metric.schema_url.clone();
                        let mapped_resource = resource.clone();
                        let mapped_resource_schema_url = resource_schema_url.clone();
                        scope_metric
                            .metrics
                            .into_iter()
                            .map(move |metric| MetricsData {
                                resource_metrics: vec![ResourceMetrics {
                                    resource: mapped_resource.clone(),
                                    scope_metrics: vec![ScopeMetrics {
                                        scope: scope.clone(),
                                        metrics: vec![metric],
                                        schema_url: scope_schema_url.clone(),
                                    }],
                                    schema_url: mapped_resource_schema_url.clone(),
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
                let resource = resource_span.resource;
                let resource_schema_url = resource_span.schema_url;
                resource_span
                    .scope_spans
                    .into_iter()
                    .flat_map(move |scope_span| {
                        let scope = scope_span.scope.clone();
                        let scope_schema_url = scope_span.schema_url.clone();
                        let mapped_resource = resource.clone();
                        let mapped_resource_schema_url = resource_schema_url.clone();
                        scope_span.spans.into_iter().map(move |span| TracesData {
                            resource_spans: vec![ResourceSpans {
                                resource: mapped_resource.clone(),
                                scope_spans: vec![ScopeSpans {
                                    scope: scope.clone(),
                                    spans: vec![span],
                                    schema_url: scope_schema_url.clone(),
                                }],
                                schema_url: mapped_resource_schema_url.clone(),
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
                let resource = resource_log.resource;
                let resource_schema_url = resource_log.schema_url;
                resource_log
                    .scope_logs
                    .into_iter()
                    .flat_map(move |scope_log| {
                        let scope = scope_log.scope.clone();
                        let scope_schema_url = scope_log.schema_url.clone();
                        let mapped_resource = resource.clone();
                        let mapped_resource_schema_url = resource_schema_url.clone();
                        scope_log
                            .log_records
                            .into_iter()
                            .map(move |log_record| LogsData {
                                resource_logs: vec![ResourceLogs {
                                    resource: mapped_resource.clone(),
                                    scope_logs: vec![ScopeLogs {
                                        scope: scope.clone(),
                                        log_records: vec![log_record],
                                        schema_url: scope_schema_url.clone(),
                                    }],
                                    schema_url: mapped_resource_schema_url.clone(),
                                }],
                            })
                    })
            })
            .collect()
    }

    async fn send_outports(&mut self, message: OtapPdata) -> Result<(), Error> {
        for port in self.ports.iter().cloned() {
            self.effect_handler
                .send_message_to(port, message.clone())
                .await?;
        }

        Ok(())
    }
}

#[async_trait(?Send)]
impl DebugOutput for DebugOutputPorts {
    async fn output_message(&mut self, _message: &str) -> Result<(), Error> {
        // since we are sending to ports we don't do anything with the &str data
        Ok(())
    }
    async fn output_metrics(
        &mut self,
        metric_request: MetricsData,
        sampler: &mut Sampler,
    ) -> Result<(), Error> {
        match self.display_mode {
            DisplayMode::Batch => {
                let send_message = async || -> Result<(), Error> {
                    self.send_outports(OTLPSignal::Metrics(metric_request).try_into()?)
                        .await
                };
                sampler.sample(send_message).await?;
            }
            DisplayMode::Signal => {
                let metric_signals = self.split_metrics(metric_request);
                for metric in metric_signals {
                    let send_message = async || -> Result<(), Error> {
                        self.send_outports(OTLPSignal::Metrics(metric).try_into()?)
                            .await
                    };
                    sampler.sample(send_message).await?;
                }
            }
        }
        Ok(())
    }
    async fn output_traces(
        &mut self,
        trace_request: TracesData,
        sampler: &mut Sampler,
    ) -> Result<(), Error> {
        match self.display_mode {
            DisplayMode::Batch => {
                let send_message = async || -> Result<(), Error> {
                    self.send_outports(OTLPSignal::Traces(trace_request).try_into()?)
                        .await
                };
                sampler.sample(send_message).await?;
            }
            DisplayMode::Signal => {
                let trace_signals = self.split_traces(trace_request);
                for trace in trace_signals {
                    let send_message = async || -> Result<(), Error> {
                        self.send_outports(OTLPSignal::Traces(trace).try_into()?)
                            .await
                    };
                    sampler.sample(send_message).await?;
                }
            }
        }
        Ok(())
    }
    async fn output_logs(
        &mut self,
        log_request: LogsData,
        sampler: &mut Sampler,
    ) -> Result<(), Error> {
        match self.display_mode {
            DisplayMode::Batch => {
                let send_message = async || -> Result<(), Error> {
                    self.send_outports(OTLPSignal::Logs(log_request).try_into()?)
                        .await
                };
                sampler.sample(send_message).await?;
            }
            DisplayMode::Signal => {
                let log_signals = self.split_logs(log_request);
                for log in log_signals {
                    let send_message = async || -> Result<(), Error> {
                        self.send_outports(OTLPSignal::Logs(log).try_into()?).await
                    };
                    sampler.sample(send_message).await?;
                }
            }
        }
        Ok(())
    }

    fn is_basic(&self) -> bool {
        // if we choose to output to outports then we don't care about verbosity level
        false
    }
}
