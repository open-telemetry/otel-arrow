// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTLP Debug processor node
//!
//! ToDo: Handle Ack and Nack messages in the pipeline
//! ToDo: Handle configuration changes
//! ToDo: Implement proper deadline function for Shutdown ctrl msg


use crate::{
    OTAP_PROCESSOR_FACTORIES,
    pdata::OtapPdata
};
use async_trait::async_trait;
use linkme::distributed_slice;

use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::config::ProcessorConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::error::Error;
use otap_df_engine::local::processor as local;
use otap_df_engine::message::Message;
use otap_df_engine::node::NodeId;
use otap_df_engine::processor::ProcessorWrapper;
use serde_json::Value;
use std::sync::Arc;

/// The URN for the filter processor
pub const FILTER_PROCESSOR_URN: &str = "urn:otel:filter:processor";

/// processor that outputs all data received to stdout
pub struct FilterProcessor {
    config: Config,
}

// ToDo: some telemetry to collect -> number of filtered data is removed datapoints

#[async_trait(?Send)]
impl local::Processor<OtapPdata> for FilterProcessor {
    async fn process(
        &mut self,
        msg: Message<OtapPdata>,
        effect_handler: &mut local::EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {

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
                    _ => {}
                }
                Ok(())
            }
            Message::PData(pdata) => {

                // convert to arrow records
                let (context, payload) = pdata.into_parts();

                let arrow_records: OtapArrowRecords = payload.try_into()?;

                let filtered_arrow_records = match pdata.signal_type() {
                    SignalType::Metrics => {
                        // ToDo: Add support for traces
                    },
                    SignalType::Logs => {
                        // ToDo: Add support for logs
                        self.config.logs.filter(arrow_records)?
                    },
                    SignalType::Traces => {
                        // ToDo: Add support for traces
                    }
                };
                effect_handler.send_message(OtapPdata::new(context, arrow_records.into())).await()?;
            Ok(())
            }
        }
    }
}


#[cfg(test)]
mod tests {
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
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::Arc;
    use tokio::time::Duration;

    /// Validation closure that checks the outputted data
    fn validation_procedure(
    ) -> impl FnOnce(ValidateContext) -> Pin<Box<dyn Future<Output = ()>>> {
        |_| {
            Box::pin(async move {
                // read in the logs and verify that we received the correct 
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
                    Duration::from_millis(200),
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
    fn test_filter_processor_logs() {
        let test_runtime = TestRuntime::new();
        let log_filter = LogFilter::new();
        let config = Config::new(
            log_filter,
        );
        let user_config = Arc::new(NodeUserConfig::new_processor_config(FILTER_PROCESSOR_URN));
        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);
        let processor = ProcessorWrapper::local(
            FilterProcessor::new(config, pipeline_ctx),
            test_node(test_runtime.config().name.clone()),
            user_config,
            test_runtime.config(),
        );

        test_runtime
            .set_processor(processor)
            .run_test(scenario())
            .validate(validation_procedure());

    }
}
