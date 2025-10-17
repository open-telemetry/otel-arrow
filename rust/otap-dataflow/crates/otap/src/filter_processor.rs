// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTAP Filter processor node
//!
//! ToDo: Handle Ack and Nack messages in the pipeline
//! ToDo: Handle configuration changes
//! ToDo: Implement proper deadline function for Shutdown ctrl msg
//! ToDo: Collect telemetry like number of filtered data is removed datapoints

use self::config::Config;
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
use otap_df_engine::error::{Error, ProcessorErrorKind, format_error_sources};
use otap_df_engine::local::processor as local;
use otap_df_engine::message::Message;
use otap_df_engine::node::NodeId;
use otap_df_engine::processor::ProcessorWrapper;
 use otap_df_config::experimental::SignalType;
 use otel_arrow_rust::otap::OtapArrowRecords;
use serde_json::Value;
use std::sync::Arc;

mod config;
/// The URN for the filter processor
pub const FILTER_PROCESSOR_URN: &str = "urn:otel:filter:processor";

/// processor that outputs all data received to stdout
pub struct FilterProcessor {
    config: Config,
}

/// Factory function to create an FilterProcessor.
///
/// See the module documentation for configuration examples
pub fn create_filter_processor(
    pipeline_ctx: PipelineContext,
    node: NodeId,
    node_config: Arc<NodeUserConfig>,
    processor_config: &ProcessorConfig,
) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
    Ok(ProcessorWrapper::local(
        FilterProcessor::from_config(pipeline_ctx, &node_config.config)?,
        node,
        node_config,
        processor_config,
    ))
}

/// Register FilterProcessor as an OTAP processor factory
#[allow(unsafe_code)]
#[distributed_slice(OTAP_PROCESSOR_FACTORIES)]
pub static FILTER_PROCESSOR_FACTORY: otap_df_engine::ProcessorFactory<OtapPdata> =
    otap_df_engine::ProcessorFactory {
        name: FILTER_PROCESSOR_URN,
        create: |pipeline_ctx: PipelineContext,
                 node: NodeId,
                 node_config: Arc<NodeUserConfig>,
                 proc_cfg: &ProcessorConfig| {
            create_filter_processor(pipeline_ctx, node, node_config, proc_cfg)
        },
    };

impl FilterProcessor {
    /// Creates a new FilterProcessor
    #[must_use]
    #[allow(dead_code)]
    pub fn new(config: Config, _pipeline_ctx: PipelineContext) -> Self {
        FilterProcessor { config }
    }

    /// Creates a new FilterProcessor from a configuration object
    pub fn from_config(pipeline_ctx: PipelineContext, config: &Value) -> Result<Self, ConfigError> {
        let config: Config =
            serde_json::from_value(config.clone()).map_err(|e| ConfigError::InvalidUserConfig {
                error: e.to_string(),
            })?;
        Ok(FilterProcessor { config })
    }
}


#[async_trait(?Send)]
impl local::Processor<OtapPdata> for FilterProcessor {
    async fn process(
        &mut self,
        msg: Message<OtapPdata>,
        effect_handler: &mut local::EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {

        match msg {
            Message::Control(control) => {
                // ToDo: add internal telemetry that will be sent out here
                Ok(())
            }
            Message::PData(pdata) => {
                let signal = pdata.signal_type();
                // convert to arrow records
                let (context, payload) = pdata.into_parts();

                let arrow_records: OtapArrowRecords = payload.try_into()?;

                let filtered_arrow_records: OtapArrowRecords = match signal {
                    SignalType::Metrics => {
                        // ToDo: Add support for metrics
                        return Ok(());
                    },
                    SignalType::Logs => {
                        self.config.log_filters().filter(arrow_records).map_err(|e| {
                        let source_detail = format_error_sources(&e);
                        Error::ProcessorError {
                            processor: effect_handler.processor_id(),
                            kind: ProcessorErrorKind::Other,
                            error: format!("Filter error: {e}"),
                            source_detail,
                        }
                    })?
                    },
                    SignalType::Traces => {
                        // ToDo: Add support for traces
                        return Ok(());
                    }
                };
                effect_handler.send_message(OtapPdata::new(context, filtered_arrow_records.into())).await?;
            Ok(())
            }
        }
    }
}


// #[cfg(test)]
// mod tests {
//     use crate::pdata::{OtapPdata, OtlpProtoBytes};
//     use otap_df_config::node::NodeUserConfig;
//     use otap_df_engine::context::ControllerContext;
//     use otap_df_engine::message::Message;
//     use otap_df_engine::processor::ProcessorWrapper;
//     use otap_df_engine::testing::processor::TestRuntime;
//     use otap_df_engine::testing::processor::{TestContext, ValidateContext};
//     use otap_df_engine::testing::test_node;
//     use otap_df_telemetry::registry::MetricsRegistryHandle;
//     use otel_arrow_rust::proto::opentelemetry::{
//         common::v1::{AnyValue, InstrumentationScope, KeyValue},
//         logs::v1::{LogRecord, LogsData, ResourceLogs, ScopeLogs, SeverityNumber},
//         metrics::v1::{
//             Exemplar, Gauge, Metric, MetricsData, NumberDataPoint, ResourceMetrics, ScopeMetrics,
//         },
//         resource::v1::Resource,
//         trace::v1::{
//             ResourceSpans, ScopeSpans, Span, Status, TracesData, span::Event, span::Link,
//             span::SpanKind, status::StatusCode,
//         },
//     };
//     use prost::Message as _;
//     use serde_json::Value;
//     use std::collections::HashSet;
//     use std::future::Future;
//     use std::pin::Pin;
//     use std::sync::Arc;
//     use tokio::time::Duration;
//     use otel_arrow_rust::filter::{LogFilter, KeyValue as KeyValueFilter, AnyValue as AnyValueFilter};
//     use otel_arrow_rust::filter::filter_logs::{LogMatchProperties, LogMatchType, LogServerityNumberMatchProperties};

//     /// Validation closure that checks the outputted data
//     fn validation_procedure(
//     ) -> impl FnOnce(ValidateContext) -> Pin<Box<dyn Future<Output = ()>>> {
//         |mut ctx| {
//             Box::pin(async move {
//                 // read in the logs and verify that we received the correct 
//                 let mut received_messages = 0;

//                 while let Ok(received_signal) = ctx.recv().await {

//                     match received_signal.signal_type() {
//                         SignalType::Metrics(metric) => {
                            
//                         }
//                         SignalType::Traces(span) => {
//                             for resource in span.resource_spans.iter() {
//                                 for scope in resource.scope_spans.iter() {
//                                     received_messages += scope.spans.len();
//                                     assert!(scope.spans.len() <= MAX_BATCH);
//                                 }
//                             }
//                         }
//                         SignalType::Logs(log) => {
                            
//                         }
//                     }
//                 }
//             })
//         }
//     }

//     /// Test closure that simulates a typical processor scenario.
//     fn scenario() -> impl FnOnce(TestContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
//         move |mut ctx| {
//             Box::pin(async move {
//                 // send log message

//                 let logs_data = LogsData::new(vec![
//                     ResourceLogs::build(Resource::build(vec![KeyValue::new(
//                             "version",
//                             AnyValue::new_string("2.0"),
//                         )]))
//                         .scope_logs(vec![
//                             ScopeLogs::build(
//                                 InstrumentationScope::build("library")
//                                     .version("scopev1")
//                                     .finish(),
//                             )
//                             .log_records(vec![
//                                 LogRecord::build(2_000_000_000u64, SeverityNumber::Info, "event1")
//                                     .attributes(vec![KeyValue::new(
//                                         "log_attr1",
//                                         AnyValue::new_string("log_val_1"),
//                                     )])
//                                     .body(AnyValue::new_string("log_body"))
//                                     .finish(),
//                             ])
//                             .finish(),
//                         ])
//                         .finish(),
//                 ]);

//                 //convert logsdata to otappdata
//                 let mut bytes = vec![];
//                 logs_data
//                     .encode(&mut bytes)
//                     .expect("failed to encode log data into bytes");
//                 let otlp_logs_bytes =
//                     OtapPdata::new_default(OtlpProtoBytes::ExportLogsRequest(bytes).into());
//                 ctx.process(Message::PData(otlp_logs_bytes))
//                     .await
//                     .expect("failed to process");
//                 let msgs = ctx.drain_pdata().await;
//                 assert_eq!(msgs.len(), 1);
//             })
//         }
//     }


//     #[test]
//     fn test_filter_processor_logs_strict() {
//         let test_runtime = TestRuntime::new();
        
//         let include_resource_attributes = vec![]
//         let include_record_attributes = vec![KeyValueFilter::new("log_attr1".to_string(), AnyValueFilter::String("log_val_1".to_string()))];
//         let include_severity_texts = vec![]:
//         let include_bodies = vec!["log_bodies"];

//         let exclude_resource_attributes = vec![];
//         let exclude_record_attributes = vec![];
//         let exclude_bodies = vec![];


//         let include = LogMatchProperties::new(LogMatchType::Strict, include_resource_attributes, include_record_attributes, );
//         let exclude = LogMatchProperties::new();
        
//         let log_filter = LogFilter::new(include, exclude, vec![]);
//         let config = Config::new(
//             log_filter,
//         );
//         let user_config = Arc::new(NodeUserConfig::new_processor_config(FILTER_PROCESSOR_URN));

//         let processor = ProcessorWrapper::local(
//             FilterProcessor::new(config),
//             test_node(test_runtime.config().name.clone()),
//             user_config,
//             test_runtime.config(),
//         );

//         test_runtime
//             .set_processor(processor)
//             .run_test(scenario())
//             .validate(validation_procedure());

//     }
// }
