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
use otap_df_engine::control::NodeControlMsg;
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
pub const DEBUG_PROCESSOR_URN: &str = "urn:otel:debug:processor";

/// processor that outputs all data received to stdout
pub struct DebugProcessor {
    config: Config,
    output: Option<String>,
}

/// Register AttributesProcessor as an OTAP processor factory
#[allow(unsafe_code)]
#[distributed_slice(OTAP_PROCESSOR_FACTORIES)]
pub static DEBUG_PROCESSOR_FACTORY: otap_df_engine::ProcessorFactory<OtapPdata> =
    otap_df_engine::ProcessorFactory {
        name: DEBUG_PROCESSOR_URN,
        create: |node: NodeId, config: &Value, proc_cfg: &ProcessorConfig| {
            let user_config = Arc::new(NodeUserConfig::new_processor_config(DEBUG_PROCESSOR_URN));
            Ok(ProcessorWrapper::local(
                DebugProcessor::from_config(config)?,
                node,
                user_config,
                proc_cfg,
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
        let raw_writer = get_writer(&self.output).await;
        let mut writer = OutputWriter::new(raw_writer, effect_handler.processor_id());

        match msg {
            Message::Control(control) => {
                match control {
                    NodeControlMsg::TimerTick {} => {
                        writer.write("Timer tick received\n").await?;
                    }
                    NodeControlMsg::Config { .. } => {
                        writer.write("Config message received\n").await?;
                    }
                    NodeControlMsg::Shutdown { .. } => {
                        writer.write("Shutdown message received\n").await?;
                    }
                    _ => {}
                }
                Ok(())
            }
            Message::PData(pdata) => {
                // make a copy of the data and convert it to protobytes that we will later convert to the views
                let otlp_bytes: OtlpProtoBytes = pdata.clone().try_into()?;
                // forward the data to the next node
                effect_handler.send_message(pdata).await?;

                match otlp_bytes {
                    OtlpProtoBytes::ExportLogsRequest(bytes) => {
                        let req = LogsData::decode(bytes.as_slice()).map_err(|e| {
                            Error::PdataConversionError {
                                error: format!("error decoding proto bytes: {e}"),
                            }
                        })?;
                        push_log(&self.config.verbosity(), req, &*marshaler, &mut writer).await?;
                    }
                    OtlpProtoBytes::ExportMetricsRequest(bytes) => {
                        let req = MetricsData::decode(bytes.as_slice()).map_err(|e| {
                            Error::PdataConversionError {
                                error: format!("error decoding proto bytesf: {e}"),
                            }
                        })?;
                        push_metric(&self.config.verbosity(), req, &*marshaler, &mut writer)
                            .await?;
                    }
                    OtlpProtoBytes::ExportTracesRequest(bytes) => {
                        let req = TracesData::decode(bytes.as_slice()).map_err(|e| {
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
async fn get_writer(output_file: &Option<String>) -> Box<dyn AsyncWrite + Unpin> {
    match output_file {
        Some(file_name) => {
            let file = File::options()
                .write(true)
                .append(true)
                .create(true)
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

#[cfg(test)]
mod tests {

    use crate::debug_processor::config::{Config, Verbosity};
    use crate::debug_processor::{DEBUG_PROCESSOR_URN, DebugProcessor};
    use crate::pdata::{OtapPdata, OtlpProtoBytes};
    use otap_df_config::node::NodeUserConfig;
    use otap_df_engine::message::Message;
    use otap_df_engine::processor::ProcessorWrapper;
    use otap_df_engine::testing::processor::TestRuntime;
    use otap_df_engine::testing::processor::{TestContext, ValidateContext};
    use otap_df_engine::testing::test_node;
    use otel_arrow_rust::proto::opentelemetry::{
        common::v1::{AnyValue, InstrumentationScope, KeyValue},
        logs::v1::{LogRecord, LogsData, ResourceLogs, ScopeLogs, SeverityNumber},
        metrics::v1::{
            Exemplar, Gauge, Metric, MetricsData, NumberDataPoint, ResourceMetrics, ScopeMetrics,
        },
        resource::v1::Resource,
        trace::v1::{
            ResourceSpans, ScopeSpans, Span, Status, TracesData, span::Event, span::Link,
            span::SpanKind, status::StatusCode,
        },
    };
    use prost::Message as _;
    use serde_json::Value;
    use std::fs::{File, remove_file};
    use std::future::Future;
    use std::io::{BufReader, read_to_string};
    use std::pin::Pin;
    use std::sync::Arc;
    use tokio::time::Duration;

    /// Validation closure that checks the outputted data
    fn validation_procedure(
        output_file: String,
    ) -> impl FnOnce(ValidateContext) -> Pin<Box<dyn Future<Output = ()>>> {
        |_| {
            Box::pin(async move {
                let file = File::open(output_file).expect("failed to open file");
                let reader = read_to_string(BufReader::new(file)).expect("failed to get string");

                // check the the processor has received the expected number of messages
                assert!(reader.contains("Received 1 resource metrics"));
                assert!(reader.contains("Received 1 metrics"));
                assert!(reader.contains("Received 1 data points"));
                assert!(reader.contains("Received 1 resource spans"));
                assert!(reader.contains("Received 1 spans"));
                assert!(reader.contains("Received 1 events"));
                assert!(reader.contains("Received 1 links"));
                assert!(reader.contains("Received 1 resource logs"));
                assert!(reader.contains("Received 1 log records"));
                assert!(reader.contains("Received 1 events"));
                assert!(reader.contains("Timer tick received"));
                assert!(reader.contains("Config message received"));
                assert!(reader.contains("Shutdown message received"));
            })
        }
    }

    /// Test closure that simulates a typical processor scenario.
    fn scenario() -> impl FnOnce(TestContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
        move |mut ctx| {
            Box::pin(async move {
                let logs_data = LogsData::new(vec![
                    ResourceLogs::build(Resource::default())
                        .scope_logs(vec![
                            ScopeLogs::build(
                                InstrumentationScope::build("library")
                                    .version("scopev1")
                                    .finish(),
                            )
                            .log_records(vec![
                                LogRecord::build(2_000_000_000u64, SeverityNumber::Info, "event1")
                                    .observed_time_unix_nano(3_000_000_000u64)
                                    .attributes(vec![KeyValue::new(
                                        "log_attr1",
                                        AnyValue::new_string("log_val_1"),
                                    )])
                                    .body(AnyValue::new_string("log_body"))
                                    .finish(),
                            ])
                            .finish(),
                        ])
                        .finish(),
                ]);

                //convert logsdata to otappdata
                let mut bytes = vec![];
                logs_data
                    .encode(&mut bytes)
                    .expect("failed to encode log data into bytes");
                let otlp_logs_bytes: OtapPdata = OtlpProtoBytes::ExportLogsRequest(bytes).into();
                ctx.process(Message::PData(otlp_logs_bytes))
                    .await
                    .expect("failed to process");
                let msgs = ctx.drain_pdata().await;
                assert_eq!(msgs.len(), 1);

                let metrics_data = MetricsData::new(vec![
                    ResourceMetrics::build(Resource::default())
                        .scope_metrics(vec![
                            ScopeMetrics::build(
                                InstrumentationScope::build("library")
                                    .version("scopev1")
                                    .finish(),
                            )
                            .metrics(vec![
                                Metric::build_gauge(
                                    "gauge name",
                                    Gauge::new(vec![
                                        NumberDataPoint::build_double(123u64, std::f64::consts::PI)
                                            .attributes(vec![KeyValue::new(
                                                "gauge_attr1",
                                                AnyValue::new_string("gauge_val"),
                                            )])
                                            .start_time_unix_nano(456u64)
                                            .exemplars(vec![
                                                Exemplar::build_int(678u64, 234i64)
                                                    .filtered_attributes(vec![KeyValue::new(
                                                        "exemplar_attr",
                                                        AnyValue::new_string("exemplar_val"),
                                                    )])
                                                    .finish(),
                                            ])
                                            .flags(1u32)
                                            .finish(),
                                    ]),
                                )
                                .description("here's a description")
                                .unit("a unit")
                                .metadata(vec![KeyValue::new(
                                    "metric_attr",
                                    AnyValue::new_string("metric_val"),
                                )])
                                .finish(),
                            ])
                            .finish(),
                        ])
                        .finish(),
                ]);
                bytes = vec![];
                metrics_data
                    .encode(&mut bytes)
                    .expect("failed to encode log data into bytes");
                let otlp_metrics_bytes: OtapPdata =
                    OtlpProtoBytes::ExportMetricsRequest(bytes).into();
                ctx.process(Message::PData(otlp_metrics_bytes))
                    .await
                    .expect("failed to process");
                let msgs = ctx.drain_pdata().await;
                assert_eq!(msgs.len(), 1);

                let traces_data = TracesData::new(vec![
                    ResourceSpans::build(Resource::default())
                        .scope_spans(vec![
                            ScopeSpans::build(
                                InstrumentationScope::build("library")
                                    .version("scopev1")
                                    .finish(),
                            )
                            .spans(vec![
                                Span::build(
                                    Vec::from("4327e52011a22f9662eac217d77d1ec0".as_bytes()),
                                    Vec::from("7271ee06d7e5925f".as_bytes()),
                                    "span_name_1",
                                    999u64,
                                )
                                .trace_state("some_state")
                                .end_time_unix_nano(1999u64)
                                .parent_span_id(vec![0, 0, 0, 0, 1, 1, 1, 1])
                                .dropped_attributes_count(7u32)
                                .dropped_events_count(11u32)
                                .dropped_links_count(29u32)
                                .kind(SpanKind::Consumer)
                                .status(Status::new("something happened", StatusCode::Error))
                                .events(vec![
                                    Event::build("an_event", 456u64)
                                        .attributes(vec![KeyValue::new(
                                            "event_attr1",
                                            AnyValue::new_string("hi"),
                                        )])
                                        .dropped_attributes_count(12345u32)
                                        .finish(),
                                ])
                                .links(vec![
                                    Link::build(
                                        vec![0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3],
                                        vec![0, 0, 0, 0, 1, 1, 1, 1],
                                    )
                                    .trace_state("some link state")
                                    .dropped_attributes_count(567u32)
                                    .flags(7u32)
                                    .attributes(vec![KeyValue::new(
                                        "link_attr1",
                                        AnyValue::new_string("hello"),
                                    )])
                                    .finish(),
                                ])
                                .finish(),
                            ])
                            .finish(),
                        ])
                        .finish(),
                ]);

                bytes = vec![];
                traces_data
                    .encode(&mut bytes)
                    .expect("failed to encode log data into bytes");
                let otlp_traces_bytes: OtapPdata =
                    OtlpProtoBytes::ExportTracesRequest(bytes).into();
                ctx.process(Message::PData(otlp_traces_bytes))
                    .await
                    .expect("failed to process");
                let msgs = ctx.drain_pdata().await;
                assert_eq!(msgs.len(), 1);

                ctx.process(Message::timer_tick_ctrl_msg())
                    .await
                    .expect("Processor failed on TimerTick");
                assert!(ctx.drain_pdata().await.is_empty());

                // Process a Config event.
                ctx.process(Message::config_ctrl_msg(Value::Null))
                    .await
                    .expect("Processor failed on Config");
                assert!(ctx.drain_pdata().await.is_empty());

                // Process a Shutdown event.
                ctx.process(Message::shutdown_ctrl_msg(
                    Duration::from_millis(200),
                    "no reason",
                ))
                .await
                .expect("Processor failed on Shutdown");
                assert!(ctx.drain_pdata().await.is_empty());
            })
        }
    }

    #[test]
    fn test_debug_processor_normal_verbosity() {
        let test_runtime = TestRuntime::new();

        let output_file = "debug_output_normal.txt".to_string();
        let config = Config::new(Verbosity::Normal);
        let user_config = Arc::new(NodeUserConfig::new_processor_config(DEBUG_PROCESSOR_URN));
        let processor = ProcessorWrapper::local(
            DebugProcessor::new(config, Some(output_file.clone())),
            test_node(test_runtime.config().name.clone()),
            user_config,
            test_runtime.config(),
        );

        test_runtime
            .set_processor(processor)
            .run_test(scenario())
            .validate(validation_procedure(output_file.clone()));

        remove_file(output_file).expect("Failed to remove file");
    }

    #[test]
    fn test_debug_processor_basic_verbosity() {
        let test_runtime = TestRuntime::new();

        let output_file = "debug_output_basic.txt".to_string();
        let config = Config::new(Verbosity::Basic);
        let user_config = Arc::new(NodeUserConfig::new_processor_config(DEBUG_PROCESSOR_URN));
        let processor = ProcessorWrapper::local(
            DebugProcessor::new(config, Some(output_file.clone())),
            test_node(test_runtime.config().name.clone()),
            user_config,
            test_runtime.config(),
        );

        test_runtime
            .set_processor(processor)
            .run_test(scenario())
            .validate(validation_procedure(output_file.clone()));

        remove_file(output_file).expect("Failed to remove file");
    }

    #[test]
    fn test_debug_processor_detailed_verbosity() {
        let test_runtime = TestRuntime::new();

        let output_file = "debug_output_detailed.txt".to_string();
        let config = Config::new(Verbosity::Detailed);
        let user_config = Arc::new(NodeUserConfig::new_processor_config(DEBUG_PROCESSOR_URN));
        let processor = ProcessorWrapper::local(
            DebugProcessor::new(config, Some(output_file.clone())),
            test_node(test_runtime.config().name.clone()),
            user_config,
            test_runtime.config(),
        );

        test_runtime
            .set_processor(processor)
            .run_test(scenario())
            .validate(validation_procedure(output_file.clone()));

        remove_file(output_file).expect("Failed to remove file");
    }
}
