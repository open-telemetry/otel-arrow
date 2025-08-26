// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTLP Debug processor node
//!
//! ToDo: Handle Ack and Nack messages in the pipeline
//! ToDo: Handle configuration changes
//! ToDo: Implement proper deadline function for Shutdown ctrl msg
//! ToDo: Use OTLP Views instead of the OTLP Request structs

use self::config::{Config, Verbosity};
use self::detailed_marshaler::DetailedViewMarshaler;
use self::marshaler::ViewMarshaler;
use self::normal_marshaler::NormalViewMarshaler;
use crate::{
    OTAP_PROCESSOR_FACTORIES,
    pdata::{OtapPdata, OtlpProtoBytes},
};
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::config::ProcessorConfig;
use otap_df_engine::error::Error;
use otap_df_engine::local::processor as local;
use otap_df_engine::message::Message;
use otap_df_engine::node::NodeId;
use otap_df_engine::processor::ProcessorWrapper;
use otel_arrow_rust::proto::opentelemetry::{
    logs::v1::LogsData,
    metrics::v1::{MetricsData, metric::Data},
    trace::v1::TracesData,
};
use prost::Message as _;
use serde_json::Value;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncWrite, AsyncWriteExt};

mod config;
mod detailed_marshaler;
mod marshaler;
mod normal_marshaler;

/// A wrapper around AsyncWrite that simplifies error handling for debug output
struct OutputWriter {
    writer: Box<dyn AsyncWrite + Unpin>,
    processor_id: NodeId,
}

impl OutputWriter {
    fn new(writer: Box<dyn AsyncWrite + Unpin>, processor_id: NodeId) -> Self {
        Self {
            writer,
            processor_id,
        }
    }

    async fn write(&mut self, data: &str) -> Result<(), Error<OtapPdata>> {
        self.writer
            .write_all(data.as_bytes())
            .await
            .map_err(|e| Error::ProcessorError {
                processor: self.processor_id.clone(),
                error: format!("Write error: {e}"),
            })
    }
}

/// The URN for the debug processor
pub const DEBUG_processor_URN: &str = "urn:otel:debug:processor";

/// processor that outputs all data received to stdout
pub struct DebugProcessor {
    config: Config,
    output: Option<String>,
}

// TODO: Create debug processor factory here
/// Register AttributesProcessor as an OTAP processor factory
#[allow(unsafe_code)]
#[distributed_slice(OTAP_PROCESSOR_FACTORIES)]
pub static DEBUG_PROCESSOR_FACTORY: otap_df_engine::ProcessorFactory<OtapPdata> =
    otap_df_engine::ProcessorFactory {
        name: DEBUG_PROCESSOR_URN,
        create: |node: NodeId, config: &Value, proc_cfg: &ProcessorConfig| {
            Ok(ProcessorWrapper::local(
                DebugProcessor::from_config(config)?,
                node,
                
            ))
        },
    };



    #[allow(unsafe_code)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
pub static OTAP_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: OTAP_EXPORTER_URN,
    create: |node: NodeId, node_config: Arc<NodeUserConfig>, exporter_config: &ExporterConfig| {
        Ok(ExporterWrapper::local(
            OTAPExporter::from_config(&node_config.config)?,
            node,
            node_config,
            exporter_config,
        ))
    },
};
impl DebugProcessor {
    /// Creates a new Debug processor
    #[must_use]
    #[allow(dead_code)]
    pub fn new(config: Config, output: Option<String>) -> Self {
        DebugProcessor { config, output }
    }

    /// Creates a new DebugProcessor from a configuration object
    pub fn from_config(config: &Value) -> Result<Self, otap_df_config::error::Error> {
        let config: Config = serde_json::from_value(config.clone()).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;
        Ok(DebugProcessor {
            config,
            output: None,
        })
    }
}

#[async_trait(?Send)]
impl local::Processor<OtapPdata> for DebugProcessor {
    async fn process(
        &mut self,
        msg: Message<OtapPdata>,
        effect_handler: &mut local::EffectHandler<OtapPdata>,
    ) -> Result<(), Error<OtapPdata>> {
        // create a marshaler to take the otlp objects and extract various data to report
        let marshaler: Box<dyn ViewMarshaler> = if self.config.verbosity() == Verbosity::Normal {
            Box::new(NormalViewMarshaler)
        } else {
            Box::new(DetailedViewMarshaler)
        };

        // get a writer to write to stdout or to a file
        let raw_writer = get_writer(self.output).await;
        let mut writer = OutputWriter::new(raw_writer, effect_handler.processor_id());

        match msg {
            Message::Control(_) => Ok(()),
            Message::PData(pdata) => {
                // forward the data to next node
                let otlp_bytes: OtlpProtoBytes = pdata.clone().try_into()?;
                effect_handler.send_message(pdata).await;

                match otlp_bytes {
                    OtlpProtoBytes::ExportLogsRequest(bytes) => {
                        let req =
                            LogsData::decode(bytes.as_slice()).map_err(|e| {
                                Error::PdataConversionError {
                                    error: format!("error decoding proto bytes: {e}"),
                                }
                            })?;
                        push_log(&self.config.verbosity(), req, &*marshaler, &mut writer).await?;
                    }
                    OtlpProtoBytes::ExportMetricsRequest(bytes) => {
                        let req =
                            MetricsData::decode(bytes.as_slice()).map_err(|e| {
                                Error::PdataConversionError {
                                    error: format!("error decoding proto bytesf: {e}"),
                                }
                            })?;
                        push_metric(&self.config.verbosity(), req, &*marshaler, &mut writer)
                            .await?;
                    }
                    OtlpProtoBytes::ExportTracesRequest(bytes) => {
                        let req =
                            TracesData::decode(bytes.as_slice()).map_err(|e| {
                                Error::PdataConversionError {
                                    error: format!("error decoding proto bytes: {e}"),
                                }
                            })?;
                        push_trace(&self.config.verbosity(), req, &*marshaler, &mut writer).await?;
                    }
                }
                Ok(())
            }
        }
    }
}

/// determine if output goes to console or to a file
async fn get_writer(output_file: Option<String>) -> Box<dyn AsyncWrite + Unpin> {
    match output_file {
        Some(file_name) => {
            let file = File::options()
                .write(true)
                .create(true)
                .truncate(true)
                .open(file_name)
                .await
                .expect("could not open output file");
            Box::new(file)
        }
        None => Box::new(tokio::io::stdout()),
    }
}

/// Function to collect and report the data contained in a Metrics object received by the Debug processor
async fn push_metric(
    verbosity: &Verbosity,
    metric_request: MetricsData,
    marshaler: &dyn ViewMarshaler,
    writer: &mut OutputWriter,
) -> Result<(), Error<OtapPdata>> {
    // collect number of resource metrics
    // collect number of metrics
    // collect number of datapoints
    let resource_metrics = metric_request.resource_metrics.len();
    let mut data_points = 0;
    let mut metrics = 0;
    for resource_metrics in &metric_request.resource_metrics {
        for scope_metrics in &resource_metrics.scope_metrics {
            metrics += scope_metrics.metrics.len();
            for metric in &scope_metrics.metrics {
                if let Some(data) = &metric.data {
                    match data {
                        Data::Gauge(gauge) => {
                            data_points += gauge.data_points.len();
                        }
                        Data::Sum(sum) => {
                            data_points += sum.data_points.len();
                        }
                        Data::Histogram(histogram) => {
                            data_points += histogram.data_points.len();
                        }
                        Data::ExponentialHistogram(exponential_histogram) => {
                            data_points += exponential_histogram.data_points.len();
                        }
                        Data::Summary(summary) => {
                            data_points += summary.data_points.len();
                        }
                    }
                }
            }
        }
    }

    writer
        .write(&format!("Received {resource_metrics} resource metrics\n"))
        .await?;
    writer
        .write(&format!("Received {metrics} metrics\n"))
        .await?;
    writer
        .write(&format!("Received {data_points} data points\n"))
        .await?;
    // if verbosity is basic we don't report anymore information, if a higher verbosity is specified than we call the marshaler
    if *verbosity == Verbosity::Basic {
        return Ok(());
    }

    let report = marshaler.marshal_metrics(metric_request);
    writer.write(&format!("{report}\n")).await?;
    Ok(())
}

async fn push_trace(
    verbosity: &Verbosity,
    trace_request: TracesData,
    marshaler: &dyn ViewMarshaler,
    writer: &mut OutputWriter,
) -> Result<(), Error<OtapPdata>> {
    // collect number of resource spans
    // collect number of spans
    let resource_spans = trace_request.resource_spans.len();
    let mut spans = 0;
    let mut events = 0;
    let mut links = 0;
    for resource_span in &trace_request.resource_spans {
        for scope_span in &resource_span.scope_spans {
            spans += scope_span.spans.len();
            for span in &scope_span.spans {
                events += span.events.len();
                links += span.links.len();
            }
        }
    }

    writer
        .write(&format!("Received {resource_spans} resource spans\n"))
        .await?;
    writer.write(&format!("Received {spans} spans\n")).await?;
    writer.write(&format!("Received {events} events\n")).await?;
    writer.write(&format!("Received {links} links\n")).await?;
    // if verbosity is basic we don't report anymore information, if a higher verbosity is specified than we call the marshaler
    if *verbosity == Verbosity::Basic {
        return Ok(());
    }

    let report = marshaler.marshal_traces(trace_request);
    writer.write(&format!("{report}\n")).await?;
    Ok(())
}

async fn push_log(
    verbosity: &Verbosity,
    log_request: LogsData,
    marshaler: &dyn ViewMarshaler,
    writer: &mut OutputWriter,
) -> Result<(), Error<OtapPdata>> {
    let resource_logs = log_request.resource_logs.len();
    let mut log_records = 0;
    let mut events = 0;
    for resource_log in &log_request.resource_logs {
        for scope_log in &resource_log.scope_logs {
            log_records += scope_log.log_records.len();
            for log_record in &scope_log.log_records {
                if !log_record.event_name.is_empty() {
                    events += 1;
                }
            }
        }
    }
    writer
        .write(&format!("Received {resource_logs} resource logs\n"))
        .await?;
    writer
        .write(&format!("Received {log_records} log records\n"))
        .await?;
    writer.write(&format!("Received {events} events\n")).await?;
    if *verbosity == Verbosity::Basic {
        return Ok(());
    }

    let report = marshaler.marshal_logs(log_request);
    writer.write(&format!("{report}\n")).await?;
    Ok(())
}

// #[cfg(test)]
// mod tests {

//     use crate::debug_processor::config::{Config, Verbosity};
//     use crate::debug_processor::processor::{DEBUG_processor_URN, DebugProcessor};
//     use crate::grpc::OtapPdata;
//     use crate::mock::{
//         create_otlp_log, create_otlp_metric, create_otlp_profile, create_otlp_trace,
//     };

//     use otap_df_engine::error::Error;
//     use otap_df_engine::processor::processorWrapper;
//     use otap_df_engine::testing::processor::TestContext;
//     use otap_df_engine::testing::processor::TestRuntime;
//     use otap_df_engine::testing::test_node;
//     use tokio::time::{Duration, sleep};

//     use otap_df_config::node::NodeUserConfig;
//     use std::fs::{File, remove_file};
//     use std::future::Future;
//     use std::io::{BufReader, read_to_string};
//     use std::sync::Arc;

//     /// Validation closure that checks the expected counter values
//     fn validation_procedure(
//         output_file: String,
//     ) -> impl FnOnce(
//         TestContext<OtapPdata>,
//         Result<(), Error<OtapPdata>>,
//     ) -> std::pin::Pin<Box<dyn Future<Output = ()>>> {
//         |_, processor_result| {
//             Box::pin(async move {
//                 assert!(processor_result.is_ok());

//                 // get a file to read and validate the output
//                 // open file
//                 // read the output file
//                 // assert each line accordingly
//                 let file = File::open(output_file).expect("failed to open file");
//                 let reader = read_to_string(BufReader::new(file)).expect("failed to get string");

//                 // check the the processor has received the expected number of messages
//                 assert!(reader.contains("Timer tick received"));
//                 assert!(reader.contains("OTLP Metric objects received: 0"));
//                 assert!(reader.contains("OTLP Trace objects received: 0"));
//                 assert!(reader.contains("OTLP Profile objects received: 0"));
//                 assert!(reader.contains("OTLP Log objects received: 0"));
//                 assert!(reader.contains("Received 1 resource metrics"));
//                 assert!(reader.contains("Received 5 metrics"));
//                 assert!(reader.contains("Received 5 data points"));
//                 assert!(reader.contains("Received 1 resource spans"));
//                 assert!(reader.contains("Received 1 spans"));
//                 assert!(reader.contains("Received 1 events"));
//                 assert!(reader.contains("Received 1 links"));
//                 assert!(reader.contains("Received 1 resource logs"));
//                 assert!(reader.contains("Received 1 log records"));
//                 assert!(reader.contains("Received 1 events"));
//                 assert!(reader.contains("Received 1 resource profiles"));
//                 assert!(reader.contains("Received 0 samples"));
//                 assert!(reader.contains("Shutdown message received"));
//             })
//         }
//     }

// }
