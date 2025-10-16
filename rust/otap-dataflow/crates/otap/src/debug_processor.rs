// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTLP Debug processor node
//!
//! ToDo: Handle Ack and Nack messages in the pipeline
//! ToDo: Handle configuration changes, configuration change should update the sampler
//! ToDo: Implement proper deadline function for Shutdown ctrl msg
//! ToDo: Use OTLP Views instead of the OTLP Request structs

use self::config::{Config, DisplayMode, SignalActive, Verbosity};
use self::metrics::DebugPdataMetrics;
use self::output::{DebugOutput, DebugOutputPorts, DebugOutputWriter, OutputMode};
use self::sampling::Sampler;
use crate::{
    OTAP_PROCESSOR_FACTORIES,
    pdata::{OtapPdata, OtlpProtoBytes},
};
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::PortName;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::config::ProcessorConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{CallData, NodeControlMsg};
use otap_df_engine::error::Error;
use otap_df_engine::local::processor as local;
use otap_df_engine::message::Message;
use otap_df_engine::node::NodeId;
use otap_df_engine::processor::ProcessorWrapper;
use otap_df_engine::{Interests, ProducerEffectHandlerExtension};
use otap_df_telemetry::metrics::MetricSet;
use otel_arrow_rust::proto::opentelemetry::{
    logs::v1::LogsData,
    metrics::v1::{MetricsData, metric::Data},
    trace::v1::TracesData,
};
use prost::Message as _;
use serde_json::Value;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

mod config;
mod detailed_marshaler;
mod filter;
mod marshaler;
mod metrics;
mod normal_marshaler;
mod output;
mod predicate;
mod sampling;

/// The URN for the debug processor
pub const DEBUG_PROCESSOR_URN: &str = "urn:otel:debug:processor";

/// processor that outputs all data received to stdout
pub struct DebugProcessor {
    config: Config,
    metrics: MetricSet<DebugPdataMetrics>,
    sampler: Sampler,
}

/// Factory function to create an DebugProcessor.
///
/// See the module documentation for configuration examples
pub fn create_debug_processor(
    pipeline_ctx: PipelineContext,
    node: NodeId,
    node_config: Arc<NodeUserConfig>,
    processor_config: &ProcessorConfig,
) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
    Ok(ProcessorWrapper::local(
        DebugProcessor::from_config(pipeline_ctx, &node_config.config)?,
        node,
        node_config,
        processor_config,
    ))
}

/// Register AttributesProcessor as an OTAP processor factory
#[allow(unsafe_code)]
#[distributed_slice(OTAP_PROCESSOR_FACTORIES)]
pub static DEBUG_PROCESSOR_FACTORY: otap_df_engine::ProcessorFactory<OtapPdata> =
    otap_df_engine::ProcessorFactory {
        name: DEBUG_PROCESSOR_URN,
        create: |pipeline_ctx: PipelineContext,
                 node: NodeId,
                 node_config: Arc<NodeUserConfig>,
                 proc_cfg: &ProcessorConfig| {
            create_debug_processor(pipeline_ctx, node, node_config, proc_cfg)
        },
    };

impl DebugProcessor {
    /// Creates a new Debug processor
    #[must_use]
    #[allow(dead_code)]
    pub fn new(config: Config, pipeline_ctx: PipelineContext) -> Self {
        let metrics = pipeline_ctx.register_metrics::<DebugPdataMetrics>();
        let sampler = Sampler::new(config.sampling());
        DebugProcessor {
            config,
            metrics,
            sampler,
        }
    }

    /// Creates a new DebugProcessor from a configuration object
    pub fn from_config(pipeline_ctx: PipelineContext, config: &Value) -> Result<Self, ConfigError> {
        let metrics = pipeline_ctx.register_metrics::<DebugPdataMetrics>();
        let config: Config =
            serde_json::from_value(config.clone()).map_err(|e| ConfigError::InvalidUserConfig {
                error: e.to_string(),
            })?;
        let sampler = Sampler::new(config.sampling());
        Ok(DebugProcessor {
            config,
            metrics,
            sampler,
        })
    }
}

#[derive(Debug)]
struct DebugCallData {
    micros: u128,
}

impl Default for DebugCallData {
    fn default() -> Self {
        Self {
            micros: Self::current_micros(),
        }
    }
}

impl DebugCallData {
    fn elapsed(&self) -> Duration {
        let elap = Self::current_micros() - self.micros;
        Duration::from_micros(elap as u64)
    }

    fn current_micros() -> u128 {
        let now = SystemTime::now();
        match now.duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_micros(),
            Err(_) => unreachable!(),
        }
    }
}

impl From<DebugCallData> for CallData {
    fn from(value: DebugCallData) -> Self {
        let msb = (value.micros >> 64) as u64;
        let lsb = value.micros as u64;
        smallvec::smallvec![msb.into(), lsb.into()]
    }
}

impl TryFrom<CallData> for DebugCallData {
    type Error = Error;

    fn try_from(value: CallData) -> Result<Self, Self::Error> {
        if value.len() != 2 {
            return Err(Self::Error::InternalError {
                message: "invalid calldata".into(),
            });
        }
        let msb: u64 = value[0].into();
        let lsb: u64 = value[1].into();
        let m128 = (msb as u128) << 64;
        let l128 = lsb as u128;
        let micros = m128 | l128;
        Ok(Self { micros })
    }
}

#[async_trait(?Send)]
impl local::Processor<OtapPdata> for DebugProcessor {
    async fn process(
        &mut self,
        msg: Message<OtapPdata>,
        effect_handler: &mut local::EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        // create a marshaler to take the otlp objects and extract various data to report
        let active_signals = self.config.signals();
        let output_mode = self.config.output();

        // if the outputmode is via outports then we can have multiple outports configured
        // so there is no clear default we need to determine which portnames are for the main port
        let main_ports: Option<Vec<PortName>> = if let OutputMode::Outports(ref ports) = output_mode
        {
            let connected_ports = effect_handler.connected_ports();
            Some(
                connected_ports
                    .iter()
                    .filter(|port| !ports.contains(port))
                    .cloned()
                    .collect(),
            )
        } else {
            None
        };

        // determine which output method to use to use
        let mut debug_output: Box<dyn DebugOutput> = match output_mode {
            OutputMode::Console => Box::new(
                DebugOutputWriter::new(
                    None,
                    effect_handler.processor_id(),
                    self.config.verbosity(),
                    self.config.mode(),
                )
                .await?,
            ),
            OutputMode::File(file_name) => Box::new(
                DebugOutputWriter::new(
                    Some(file_name),
                    effect_handler.processor_id(),
                    self.config.verbosity(),
                    self.config.mode(),
                )
                .await?,
            ),
            OutputMode::Outports(ports) => Box::new(DebugOutputPorts::new(
                ports.clone(),
                self.config.mode(),
                effect_handler.clone(),
            )?),
        };

        match msg {
            Message::Control(control) => {
                match control {
                    NodeControlMsg::TimerTick {} => {
                        debug_output.output_message("Timer tick received\n").await?;
                    }
                    NodeControlMsg::Config { .. } => {
                        debug_output
                            .output_message("Config message received\n")
                            .await?;
                    }
                    NodeControlMsg::Shutdown { .. } => {
                        debug_output
                            .output_message("Shutdown message received\n")
                            .await?;
                    }
                    NodeControlMsg::Ack(ackmsg) => {
                        let dd: DebugCallData = ackmsg.calldata.try_into()?;
                        debug_output
                            .output_message(&format!("ACK received after {:?}\n", dd.elapsed()))
                            .await?;
                    }
                    NodeControlMsg::Nack(nackmsg) => {
                        let dd: DebugCallData = nackmsg.calldata.try_into()?;
                        debug_output
                            .output_message(&format!("NACK received after {:?}\n", dd.elapsed()))
                            .await?;
                    }
                    NodeControlMsg::CollectTelemetry {
                        mut metrics_reporter,
                    } => {
                        _ = metrics_reporter.report(&mut self.metrics);
                    }
                    _ => {}
                }
                Ok(())
            }
            Message::PData(mut pdata) => {
                if self.config.verbosity() == Verbosity::Detailed {
                    // Print ACK/NACK detail only in Detailed mode.
                    effect_handler.subscribe_to(
                        Interests::ACKS | Interests::NACKS,
                        DebugCallData::default().into(),
                        &mut pdata,
                    );
                }
                // ToDo: handle multiple out_ports differently here?
                if let Some(ports) = main_ports {
                    for port in ports {
                        // Note each clone has its own clone of the context.
                        effect_handler.send_message_to(port, pdata.clone()).await?;
                    }
                } else {
                    effect_handler.send_message(pdata.clone()).await?;
                }

                let (_context, payload) = pdata.into_parts();
                let otlp_bytes: OtlpProtoBytes = payload.try_into()?;
                match otlp_bytes {
                    OtlpProtoBytes::ExportLogsRequest(bytes) => {
                        if active_signals.contains(&SignalActive::Logs) {
                            let req = LogsData::decode(bytes.as_slice()).map_err(|e| {
                                Error::PdataConversionError {
                                    error: format!("error decoding proto bytes: {e}"),
                                }
                            })?;
                            self.process_log(req, debug_output.as_mut()).await?;
                        }
                        self.metrics.logs_consumed.add(1);
                    }
                    OtlpProtoBytes::ExportMetricsRequest(bytes) => {
                        if active_signals.contains(&SignalActive::Metrics) {
                            let req = MetricsData::decode(bytes.as_slice()).map_err(|e| {
                                Error::PdataConversionError {
                                    error: format!("error decoding proto bytes: {e}"),
                                }
                            })?;
                            self.process_metric(req, debug_output.as_mut()).await?;
                        }
                        self.metrics.metrics_consumed.add(1);
                    }
                    OtlpProtoBytes::ExportTracesRequest(bytes) => {
                        if active_signals.contains(&SignalActive::Spans) {
                            let req = TracesData::decode(bytes.as_slice()).map_err(|e| {
                                Error::PdataConversionError {
                                    error: format!("error decoding proto bytes: {e}"),
                                }
                            })?;
                            self.process_trace(req, debug_output.as_mut()).await?;
                        }
                        self.metrics.traces_consumed.add(1);
                    }
                }
                Ok(())
            }
        }
    }
}

impl DebugProcessor {
    /// Function to collect and report the data contained in a Metrics object received by the Debug processor
    async fn process_metric(
        &mut self,
        mut metric_request: MetricsData,
        debug_output: &mut dyn DebugOutput,
    ) -> Result<(), Error> {
        // collect number of resource metrics
        // collect number of metrics
        // collect number of datapoints
        let filters = self.config.filters();
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

        self.metrics.metric_signals_consumed.add(metrics as u64);
        self.metrics
            .metric_datapoints_consumed
            .add(data_points as u64);

        let report_basic = format!(
            "Received {resource_metrics} resource metrics\nReceived {metrics} metrics\nReceived {data_points} data points\n"
        );

        debug_output.output_message(report_basic.as_str()).await?;

        // return early if don't need to output anymore information
        if debug_output.is_basic() {
            return Ok(());
        }

        // if there are filters to apply then apply them
        if !filters.is_empty() {
            for filter in filters {
                filter.filter_metrics(&mut metric_request)
            }
        }

        debug_output
            .output_metrics(metric_request, &mut self.sampler)
            .await?;

        Ok(())
    }

    async fn process_trace(
        &mut self,
        mut trace_request: TracesData,
        debug_output: &mut dyn DebugOutput,
    ) -> Result<(), Error> {
        // collect number of resource spans
        // collect number of spans
        let filters = self.config.filters();
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
        self.metrics.span_signals_consumed.add(spans as u64);
        self.metrics.span_events_consumed.add(events as u64);
        self.metrics.span_links_consumed.add(links as u64);

        let report_basic = format!(
            "Received {resource_spans} resource spans\nReceived {spans} spans\nReceived {events} events\nReceived {links} links\n"
        );

        debug_output.output_message(report_basic.as_str()).await?;
        // return early if don't need to output anymore information
        if debug_output.is_basic() {
            return Ok(());
        }

        // if there are filters to apply then apply them
        if !filters.is_empty() {
            for filter in filters {
                filter.filter_traces(&mut trace_request)
            }
        }

        debug_output
            .output_traces(trace_request, &mut self.sampler)
            .await?;
        Ok(())
    }

    async fn process_log(
        &mut self,
        mut log_request: LogsData,
        debug_output: &mut dyn DebugOutput,
    ) -> Result<(), Error> {
        let filters = self.config.filters();
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
        self.metrics.log_signals_consumed.add(log_records as u64);
        self.metrics.events_consumed.add(events);

        let report_basic = format!(
            "Received {resource_logs} resource logs\nReceived {log_records} log records\nReceived {events} events\n"
        );

        debug_output.output_message(report_basic.as_str()).await?;

        // return early if don't need to output anymore information
        if debug_output.is_basic() {
            return Ok(());
        }

        if !filters.is_empty() {
            for filter in filters {
                filter.filter_logs(&mut log_request)
            }
        }
        debug_output
            .output_logs(log_request, &mut self.sampler)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::debug_processor::config::{Config, DisplayMode, SignalActive, Verbosity};
    use crate::debug_processor::filter::{FilterMode, FilterRules};
    use crate::debug_processor::output::OutputMode;
    use crate::debug_processor::predicate::{
        KeyValue as PredicateKeyValue, MatchValue, Predicate, SignalField,
    };
    use crate::debug_processor::sampling::SamplingConfig;
    use crate::debug_processor::{DEBUG_PROCESSOR_URN, DebugProcessor};
    use crate::pdata::{OtapPdata, OtlpProtoBytes};
    use otap_df_config::node::NodeUserConfig;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::message::Message;
    use otap_df_engine::processor::ProcessorWrapper;
    use otap_df_engine::testing::processor::TestRuntime;
    use otap_df_engine::testing::processor::{TestContext, ValidateContext};
    use otap_df_engine::testing::test_node;
    use otap_df_telemetry::registry::MetricsRegistryHandle;
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
    use std::collections::HashSet;
    use std::fs::{File, remove_file};
    use std::future::Future;
    use std::io::{BufReader, read_to_string};
    use std::ops::Add;
    use std::pin::Pin;
    use std::sync::Arc;
    use std::time::Instant;
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
                    Instant::now().add(Duration::from_millis(200)),
                    "no reason",
                ))
                .await
                .expect("Processor failed on Shutdown");
                assert!(ctx.drain_pdata().await.is_empty());

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
                let otlp_logs_bytes =
                    OtapPdata::new_default(OtlpProtoBytes::ExportLogsRequest(bytes).into());
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
                let otlp_metrics_bytes =
                    OtapPdata::new_default(OtlpProtoBytes::ExportMetricsRequest(bytes).into());
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
                let otlp_traces_bytes =
                    OtapPdata::new_default(OtlpProtoBytes::ExportTracesRequest(bytes).into());
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
                    Instant::now().add(Duration::from_millis(200)),
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
        let signals = HashSet::from([
            SignalActive::Metrics,
            SignalActive::Logs,
            SignalActive::Spans,
        ]);
        let output_file = "debug_output_normal.txt".to_string();
        let sampling = SamplingConfig::NoSampling;
        let config = Config::new(
            Verbosity::Normal,
            DisplayMode::Batch,
            signals,
            OutputMode::File(output_file.clone()),
            Vec::new(),
            sampling,
        );
        let user_config = Arc::new(NodeUserConfig::new_processor_config(DEBUG_PROCESSOR_URN));

        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

        let processor = ProcessorWrapper::local(
            DebugProcessor::new(config, pipeline_ctx),
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
        let signals = HashSet::from([
            SignalActive::Metrics,
            SignalActive::Logs,
            SignalActive::Spans,
        ]);
        let output_file = "debug_output_basic.txt".to_string();
        let sampling = SamplingConfig::NoSampling;
        let config = Config::new(
            Verbosity::Basic,
            DisplayMode::Batch,
            signals,
            OutputMode::File(output_file.clone()),
            Vec::new(),
            sampling,
        );
        let user_config = Arc::new(NodeUserConfig::new_processor_config(DEBUG_PROCESSOR_URN));
        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);
        let processor = ProcessorWrapper::local(
            DebugProcessor::new(config, pipeline_ctx),
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
        let signals = HashSet::from([
            SignalActive::Metrics,
            SignalActive::Logs,
            SignalActive::Spans,
        ]);
        let output_file = "debug_output_detailed.txt".to_string();
        let sampling = SamplingConfig::NoSampling;
        let config = Config::new(
            Verbosity::Detailed,
            DisplayMode::Batch,
            signals,
            OutputMode::File(output_file.clone()),
            Vec::new(),
            sampling,
        );
        let user_config = Arc::new(NodeUserConfig::new_processor_config(DEBUG_PROCESSOR_URN));
        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);
        let processor = ProcessorWrapper::local(
            DebugProcessor::new(config, pipeline_ctx),
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

    fn validation_procedure_logs_only(
        output_file: String,
    ) -> impl FnOnce(ValidateContext) -> Pin<Box<dyn Future<Output = ()>>> {
        |_| {
            Box::pin(async move {
                let file = File::open(output_file).expect("failed to open file");
                let reader = read_to_string(BufReader::new(file)).expect("failed to get string");

                // check the the processor has received the expected number of messages
                assert!(!reader.contains("Received 1 resource metrics"));
                assert!(!reader.contains("Received 1 metrics"));
                assert!(!reader.contains("Received 1 data points"));
                assert!(!reader.contains("Received 1 resource spans"));
                assert!(!reader.contains("Received 1 spans"));
                assert!(reader.contains("Received 1 resource logs"));
                assert!(reader.contains("Received 1 log records"));
                assert!(reader.contains("Received 1 events"));
                assert!(reader.contains("Timer tick received"));
                assert!(reader.contains("Config message received"));
                assert!(reader.contains("Shutdown message received"));
            })
        }
    }

    #[test]
    fn test_debug_processor_only_logs() {
        let test_runtime = TestRuntime::new();

        let output_file = "debug_logs.txt".to_string();
        let signals = HashSet::from([SignalActive::Logs]);
        let sampling = SamplingConfig::NoSampling;
        let config = Config::new(
            Verbosity::Detailed,
            DisplayMode::Batch,
            signals,
            OutputMode::File(output_file.clone()),
            Vec::new(),
            sampling,
        );
        let user_config = Arc::new(NodeUserConfig::new_processor_config(DEBUG_PROCESSOR_URN));
        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);
        let processor = ProcessorWrapper::local(
            DebugProcessor::new(config, pipeline_ctx),
            test_node(test_runtime.config().name.clone()),
            user_config,
            test_runtime.config(),
        );

        test_runtime
            .set_processor(processor)
            .run_test(scenario())
            .validate(validation_procedure_logs_only(output_file.clone()));

        remove_file(output_file).expect("Failed to remove file");
    }

    fn validation_procedure_metrics_only(
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
                assert!(!reader.contains("Received 1 resource spans"));
                assert!(!reader.contains("Received 1 spans"));
                assert!(!reader.contains("Received 1 resource logs"));
                assert!(!reader.contains("Received 1 log records"));
                assert!(!reader.contains("Received 1 events"));
                assert!(reader.contains("Timer tick received"));
                assert!(reader.contains("Config message received"));
                assert!(reader.contains("Shutdown message received"));
            })
        }
    }

    #[test]
    fn test_debug_processor_only_metrics() {
        let test_runtime = TestRuntime::new();

        let output_file = "debug_metrics.txt".to_string();
        let signals = HashSet::from([SignalActive::Metrics]);
        let sampling = SamplingConfig::NoSampling;
        let config = Config::new(
            Verbosity::Detailed,
            DisplayMode::Batch,
            signals,
            OutputMode::File(output_file.clone()),
            Vec::new(),
            sampling,
        );
        let user_config = Arc::new(NodeUserConfig::new_processor_config(DEBUG_PROCESSOR_URN));
        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);
        let processor = ProcessorWrapper::local(
            DebugProcessor::new(config, pipeline_ctx),
            test_node(test_runtime.config().name.clone()),
            user_config,
            test_runtime.config(),
        );

        test_runtime
            .set_processor(processor)
            .run_test(scenario())
            .validate(validation_procedure_metrics_only(output_file.clone()));

        remove_file(output_file).expect("Failed to remove file");
    }

    fn validation_procedure_spans_only(
        output_file: String,
    ) -> impl FnOnce(ValidateContext) -> Pin<Box<dyn Future<Output = ()>>> {
        |_| {
            Box::pin(async move {
                let file = File::open(output_file).expect("failed to open file");
                let reader = read_to_string(BufReader::new(file)).expect("failed to get string");

                // check the the processor has received the expected number of messages
                assert!(!reader.contains("Received 1 resource metrics"));
                assert!(!reader.contains("Received 1 metrics"));
                assert!(!reader.contains("Received 1 data points"));
                assert!(reader.contains("Received 1 resource spans"));
                assert!(reader.contains("Received 1 spans"));
                assert!(reader.contains("Received 1 events"));
                assert!(reader.contains("Received 1 links"));
                assert!(!reader.contains("Received 1 resource logs"));
                assert!(!reader.contains("Received 1 log records"));
                assert!(reader.contains("Timer tick received"));
                assert!(reader.contains("Config message received"));
                assert!(reader.contains("Shutdown message received"));
            })
        }
    }

    fn validation_procedure_exclude_attribute(
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

                // signal with attribute should not be present
                assert!(!reader.contains("Attributes: log_attr1=log_val_1"));
            })
        }
    }

    fn validation_procedure_include_attribute(
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

                // signal with attribute should be present
                assert!(reader.contains("Attributes: log_attr1=log_val_1"));
            })
        }
    }
    #[test]
    fn test_debug_processor_only_spans() {
        let test_runtime = TestRuntime::new();

        let output_file = "debug_spans.txt".to_string();
        let signals = HashSet::from([SignalActive::Spans]);
        let sampling = SamplingConfig::NoSampling;

        let config = Config::new(
            Verbosity::Detailed,
            DisplayMode::Batch,
            signals,
            OutputMode::File(output_file.clone()),
            Vec::new(),
            sampling,
        );
        let user_config = Arc::new(NodeUserConfig::new_processor_config(DEBUG_PROCESSOR_URN));
        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);
        let processor = ProcessorWrapper::local(
            DebugProcessor::new(config, pipeline_ctx),
            test_node(test_runtime.config().name.clone()),
            user_config,
            test_runtime.config(),
        );

        test_runtime
            .set_processor(processor)
            .run_test(scenario())
            .validate(validation_procedure_spans_only(output_file.clone()));

        remove_file(output_file).expect("Failed to remove file");
    }

    #[test]
    fn test_debug_processor_signal_mode() {
        let test_runtime = TestRuntime::new();
        let signals = HashSet::from([
            SignalActive::Metrics,
            SignalActive::Logs,
            SignalActive::Spans,
        ]);
        let output_file = "debug_signal_mode.txt".to_string();
        let sampling = SamplingConfig::NoSampling;
        let config = Config::new(
            Verbosity::Normal,
            DisplayMode::Signal,
            signals,
            OutputMode::File(output_file.clone()),
            Vec::new(),
            sampling,
        );
        let user_config = Arc::new(NodeUserConfig::new_processor_config(DEBUG_PROCESSOR_URN));
        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);
        let processor = ProcessorWrapper::local(
            DebugProcessor::new(config, pipeline_ctx),
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
    fn test_debug_processor_filter_include() {
        let test_runtime = TestRuntime::new();
        let signals = HashSet::from([
            SignalActive::Metrics,
            SignalActive::Logs,
            SignalActive::Spans,
        ]);
        let output_file = "debug_output_filter_include.txt".to_string();
        let sampling = SamplingConfig::NoSampling;
        let filterrule = vec![FilterRules::new(
            Predicate::new(
                SignalField::Attribute,
                MatchValue::KeyValue(vec![PredicateKeyValue::new(
                    "log_attr1".to_string(),
                    MatchValue::String("log_val_1".to_string()),
                )]),
            ),
            FilterMode::Include,
        )];
        let config = Config::new(
            Verbosity::Normal,
            DisplayMode::Batch,
            signals,
            OutputMode::File(output_file.clone()),
            filterrule,
            sampling,
        );
        let user_config = Arc::new(NodeUserConfig::new_processor_config(DEBUG_PROCESSOR_URN));
        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);
        let processor = ProcessorWrapper::local(
            DebugProcessor::new(config, pipeline_ctx),
            test_node(test_runtime.config().name.clone()),
            user_config,
            test_runtime.config(),
        );

        test_runtime
            .set_processor(processor)
            .run_test(scenario())
            .validate(validation_procedure_include_attribute(output_file.clone()));

        remove_file(output_file).expect("Failed to remove file");
    }

    #[test]
    fn test_debug_processor_filter_exclude() {
        let test_runtime = TestRuntime::new();
        let signals = HashSet::from([
            SignalActive::Metrics,
            SignalActive::Logs,
            SignalActive::Spans,
        ]);
        let output_file = "debug_output_filter_exclude.txt".to_string();
        let sampling = SamplingConfig::NoSampling;
        let filterrule = vec![FilterRules::new(
            Predicate::new(
                SignalField::Attribute,
                MatchValue::KeyValue(vec![PredicateKeyValue::new(
                    "log_attr1".to_string(),
                    MatchValue::String("log_val_1".to_string()),
                )]),
            ),
            FilterMode::Exclude,
        )];
        let config = Config::new(
            Verbosity::Normal,
            DisplayMode::Batch,
            signals,
            OutputMode::File(output_file.clone()),
            filterrule,
            sampling,
        );
        let user_config = Arc::new(NodeUserConfig::new_processor_config(DEBUG_PROCESSOR_URN));
        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);
        let processor = ProcessorWrapper::local(
            DebugProcessor::new(config, pipeline_ctx),
            test_node(test_runtime.config().name.clone()),
            user_config,
            test_runtime.config(),
        );

        test_runtime
            .set_processor(processor)
            .run_test(scenario())
            .validate(validation_procedure_exclude_attribute(output_file.clone()));

        remove_file(output_file).expect("Failed to remove file");
    }

    fn validation_procedure_zap_sampling(
        output_file: String,
    ) -> impl FnOnce(ValidateContext) -> Pin<Box<dyn Future<Output = ()>>> {
        |_| {
            Box::pin(async move {
                let file = File::open(output_file).expect("failed to open file");
                let reader = read_to_string(BufReader::new(file)).expect("failed to get string");

                // initial message of 1 should be outputted
                assert!(reader.contains("ResourceLog"));
                // we don't log the second message
                assert!(!reader.contains("ResourceMetric"));
                // third message will get outputted (sampling_thereafter = 2)
                assert!(reader.contains("ResourceSpan"));
            })
        }
    }
    #[test]
    fn test_debug_processor_zap_sampling() {
        let test_runtime = TestRuntime::new();
        let signals = HashSet::from([
            SignalActive::Metrics,
            SignalActive::Logs,
            SignalActive::Spans,
        ]);
        let output_file = "debug_output_samping.txt".to_string();
        let sampling = SamplingConfig::ZapSampling {
            sampling_initial: 1,
            sampling_thereafter: 2,
            sampling_interval: 1,
        };
        let config = Config::new(
            Verbosity::Normal,
            DisplayMode::Batch,
            signals,
            OutputMode::File(output_file.clone()),
            Vec::new(),
            sampling,
        );
        let user_config = Arc::new(NodeUserConfig::new_processor_config(DEBUG_PROCESSOR_URN));

        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

        let processor = ProcessorWrapper::local(
            DebugProcessor::new(config, pipeline_ctx),
            test_node(test_runtime.config().name.clone()),
            user_config,
            test_runtime.config(),
        );

        test_runtime
            .set_processor(processor)
            .run_test(scenario())
            .validate(validation_procedure_zap_sampling(output_file.clone()));

        remove_file(output_file).expect("Failed to remove file");
    }
}
