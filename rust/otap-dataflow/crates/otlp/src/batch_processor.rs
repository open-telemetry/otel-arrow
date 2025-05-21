// SPDX-License-Identifier: Apache-2.0
//! BatchProcessor: Buffers messages and emits them in batches.
use super::types::OTLPRequest;
use crate::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
use crate::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
use crate::proto::opentelemetry::collector::trace::v1::ExportTraceServiceRequest;
use crate::proto::opentelemetry::common::v1::AnyValue;
use crate::proto::opentelemetry::common::v1::InstrumentationScope;
use crate::proto::opentelemetry::common::v1::KeyValue;
use crate::proto::opentelemetry::common::v1::any_value::Value;
use crate::proto::opentelemetry::resource::v1::Resource;
use crate::proto::opentelemetry::trace::v1::ResourceSpans;
use crate::proto::opentelemetry::trace::v1::ScopeSpans;
use crate::proto::opentelemetry::trace::v1::Span;
use async_trait::async_trait;
use otap_df_engine::error::Error;
use otap_df_engine::local::processor::{EffectHandler, Processor};
use otap_df_engine::message::{ControlMsg, Message};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Configuration for the BatchProcessor.
#[derive(Debug, Clone)]
pub struct Config {
    /// The number of messages to buffer before sending a batch.
    pub send_batch_size: u32,
    /// The timeout after which a batch will be sent even if it's not full.
    pub timeout: Duration,
    /// List of metadata keys to use for batching
    pub metadata_keys: Vec<String>,
    /// Maximum number of distinct metadata combinations to track
    pub metadata_cardinality_limit: u32,
}

impl Config {
    /// The maximum allowed value for send_batch_size.
    const MAX_BATCH_SIZE: u32 = 1000;
    fn validate(&self) -> Result<(), Error<()>> {
        if self.send_batch_size > Self::MAX_BATCH_SIZE {
            return Err(Error::IoError {
                node: "BatchProcessor".into(),
                error: std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!(
                        "send_batch_size must be less than or equal to {}",
                        Self::MAX_BATCH_SIZE
                    ),
                ),
            });
        }
        if self.metadata_keys.is_empty() {
            return Err(Error::IoError {
                node: "BatchProcessor".into(),
                error: std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "metadata_keys must not be empty".to_string(),
                ),
            });
        }
        if self.metadata_cardinality_limit == 0 {
            return Err(Error::IoError {
                node: "BatchProcessor".into(),
                error: std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "metadata_cardinality_limit must be greater than 0".to_string(),
                ),
            });
        }
        Ok(())
    }
}

/// Internal representation for a batch, with time tracking for timeout.
struct BatchEntry {
    otlp_requests: Vec<OTLPRequest>,
    last_update: Instant,
}

/// A processor that buffers messages and emits them in batches.
pub struct BatchProcessor {
    /// Map from metadata key combination to batches, each with last update time.
    batches: HashMap<Vec<String>, BatchEntry>,
    config: Config,
}

impl BatchProcessor {
    pub fn new(config: Config) -> Result<Self, Error<()>> {
        config.validate()?;
        Ok(Self {
            batches: HashMap::new(),
            config,
        })
    }

    /// Get or create the batch for a given metadata combination.
    /// Enforce cardinality:
    ///  - If a new batch would exceed the cardinality, emit and clear all batches, then start fresh.
    fn get_or_create_batch<'a>(
        &'a mut self,
        metadata: Vec<String>,
        effect_handler: &mut EffectHandler<OTLPRequest>,
    ) -> Result<&'a mut BatchEntry, Error<OTLPRequest>> {
        // If the combination is not seen before and limit is reached, emit all and clear.
        let config = &self.config;
        let need_new = !self.batches.contains_key(&metadata);
        if need_new && (self.batches.len() as u32) >= config.metadata_cardinality_limit {
            // Emit and clear all batches.
            let mut batches_to_send = Vec::new();
            for (_, batch_entry) in self.batches.drain() {
                if !batch_entry.otlp_requests.is_empty() {
                    batches_to_send.push(batch_entry.otlp_requests);
                }
            }
            for batch in batches_to_send {
                if let Some(combined) = Self::combine_batches(&batch) {
                    // Ignore errors on emit during this forced flush.
                    let _ = futures::executor::block_on(effect_handler.send_message(combined));
                }
            }
        }
        // Now safe to insert or fetch.
        Ok(self.batches.entry(metadata).or_insert_with(|| BatchEntry {
            otlp_requests: Vec::new(),
            last_update: Instant::now(),
        }))
    }

    /// Extracts a vector of batch metadata values from an OTLPRequest.
    /// Here we only examine the **first** resource in the **first** resource_spans for traces.
    /// If finer-grained or more comprehensive splitting is needed, refactor accordingly.
    fn extract_metadata(&self, data: &OTLPRequest) -> Vec<String> {
        let mut metadata = Vec::new();

        // Helper closure to extract metadata from a resource
        let extract_from_resource = |resource: &Resource, key: &str| -> Option<String> {
            resource
                .attributes
                .iter()
                .find(|attr| attr.key == key)
                .and_then(|attr| attr.value.as_ref())
                .and_then(|value| match &value.value {
                    Some(Value::StringValue(s)) => Some(s.clone()),
                    Some(Value::IntValue(i)) => Some(i.to_string()),
                    Some(Value::BoolValue(b)) => Some(b.to_string()),
                    _ => None,
                })
        };

        match data {
            OTLPRequest::Traces(request) => {
                if let Some(resource_span) = request.resource_spans.first() {
                    if let Some(resource) = resource_span.resource.as_ref() {
                        for key in &self.config.metadata_keys {
                            if let Some(value) = extract_from_resource(resource, key) {
                                metadata.push(value);
                            }
                        }
                    }
                }
            }
            OTLPRequest::Metrics(request) => {
                if let Some(resource_metrics) = request.resource_metrics.first() {
                    if let Some(resource) = resource_metrics.resource.as_ref() {
                        for key in &self.config.metadata_keys {
                            if let Some(value) = extract_from_resource(resource, key) {
                                metadata.push(value);
                            }
                        }
                    }
                }
            }
            OTLPRequest::Logs(request) => {
                if let Some(resource_logs) = request.resource_logs.first() {
                    if let Some(resource) = resource_logs.resource.as_ref() {
                        for key in &self.config.metadata_keys {
                            if let Some(value) = extract_from_resource(resource, key) {
                                metadata.push(value);
                            }
                        }
                    }
                }
            }
        }

        metadata
    }

    /// Combine a batch into a single OTLPRequest. Returns None if batch is empty.
    fn combine_batches(batch: &Vec<OTLPRequest>) -> Option<OTLPRequest> {
        if batch.is_empty() {
            return None;
        }
        match &batch[0] {
            OTLPRequest::Traces(_) => {
                let mut combined = ExportTraceServiceRequest::default();
                for item in batch {
                    if let OTLPRequest::Traces(req) = item {
                        combined.resource_spans.extend(req.resource_spans.clone());
                    }
                }
                Some(OTLPRequest::Traces(combined))
            }
            OTLPRequest::Metrics(_) => {
                let mut combined = ExportMetricsServiceRequest::default();
                for item in batch {
                    if let OTLPRequest::Metrics(req) = item {
                        combined
                            .resource_metrics
                            .extend(req.resource_metrics.clone());
                    }
                }
                Some(OTLPRequest::Metrics(combined))
            }
            OTLPRequest::Logs(_) => {
                let mut combined = ExportLogsServiceRequest::default();
                for item in batch {
                    if let OTLPRequest::Logs(req) = item {
                        combined.resource_logs.extend(req.resource_logs.clone());
                    }
                }
                Some(OTLPRequest::Logs(combined))
            }
        }
    }

    /// Checks all batches for timeout and emits-and-deletes any that have timed out.
    async fn flush_timed_out_batches(&mut self, effect_handler: &mut EffectHandler<OTLPRequest>) {
        let timeout = self.config.timeout;
        if timeout == Duration::from_secs(0) {
            return;
        }
        // Collect keys first to avoid borrow issues
        let expired_keys: Vec<_> = self
            .batches
            .iter()
            .filter(|(_, entry)| {
                entry.last_update.elapsed() >= timeout && !entry.otlp_requests.is_empty()
            })
            .map(|(k, _)| k.clone())
            .collect();
        for key in expired_keys {
            if let Some(entry) = self.batches.remove(&key) {
                if let Some(combined) = Self::combine_batches(&entry.otlp_requests) {
                    let _ = effect_handler.send_message(combined).await;
                }
            }
        }
    }

    /// Emits all non-empty batches and clears the map.
    async fn flush_all(&mut self, effect_handler: &mut EffectHandler<OTLPRequest>) {
        let mut to_send = Vec::new();
        for (_, entry) in self.batches.drain() {
            if let Some(combined) = Self::combine_batches(&entry.otlp_requests) {
                to_send.push(combined);
            }
        }
        for combined in to_send {
            let _ = effect_handler.send_message(combined).await;
        }
    }
}

#[async_trait(?Send)]
impl Processor<OTLPRequest> for BatchProcessor {
    async fn process(
        &mut self,
        msg: Message<OTLPRequest>,
        effect_handler: &mut EffectHandler<OTLPRequest>,
    ) -> Result<(), Error<OTLPRequest>> {
        match msg {
            Message::PData(data) => {
                // Get metadata and config fields before borrowing self mutably
                let metadata = self.extract_metadata(&data);
                let batch_size_threshold = self.config.send_batch_size as usize;

                // Now it's safe to mutably borrow self
                let batch_entry = self.get_or_create_batch(metadata, effect_handler)?;
                batch_entry.otlp_requests.push(data);
                batch_entry.last_update = Instant::now();
                let batch_len = batch_entry.otlp_requests.len();

                if batch_len >= batch_size_threshold {
                    if let Some(combined) = Self::combine_batches(&batch_entry.otlp_requests) {
                        effect_handler.send_message(combined).await?;
                        batch_entry.otlp_requests.clear();
                        batch_entry.last_update = Instant::now(); // (optional) update time
                    }
                }
                // Even for batch size < threshold, we check for timeout **periodically** via ControlMsg/TimerTick.
                Ok(())
            }
            Message::Control(ctrl_msg) => {
                log::info!("[BatchProcessor] Received ControlMsg: {:?}", ctrl_msg);
                match ctrl_msg {
                    ControlMsg::TimerTick { .. } => {
                        self.flush_timed_out_batches(effect_handler).await;
                        Ok(())
                    }
                    ControlMsg::Shutdown { .. } => {
                        self.flush_all(effect_handler).await;
                        Ok(())
                    }
                    _ => Ok(()),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_engine::config::ProcessorConfig;
    use otap_df_engine::processor::ProcessorWrapper;
    use otap_df_engine::testing::processor::{TestContext, TestRuntime, ValidateContext};
    use std::time::Duration;

    fn create_test_trace_request(trace_id: u8) -> OTLPRequest {
        OTLPRequest::Traces(ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "service.name".to_string(),
                        value: Some(AnyValue {
                            value: Some(Value::StringValue("service1".to_string())),
                        }),
                    }],
                    entity_refs: vec![],
                    ..Default::default()
                }),
                scope_spans: vec![ScopeSpans {
                    scope: Some(InstrumentationScope {
                        name: "scope1".to_string(),
                        version: "1.0.0".to_string(),
                        ..Default::default()
                    }),
                    spans: vec![Span {
                        trace_id: vec![trace_id; 16],
                        span_id: vec![1, 2, 3, 4, 5, 6, 7, 8],
                        name: "span1".to_string(),
                        ..Default::default()
                    }],
                    schema_url: "".to_string(),
                }],
                schema_url: "".to_string(),
            }],
        })
    }

    fn create_test_trace_request_with_trace_id(trace_id: &str) -> OTLPRequest {
        OTLPRequest::Traces(ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: Some(Resource {
                    attributes: vec![
                        KeyValue {
                            key: "service.name".to_string(),
                            value: Some(AnyValue {
                                value: Some(Value::StringValue("service1".to_string())),
                            }),
                        },
                        KeyValue {
                            key: "traceId".to_string(),
                            value: Some(AnyValue {
                                value: Some(Value::StringValue(trace_id.to_string())),
                            }),
                        },
                    ],
                    dropped_attributes_count: 0,
                    entity_refs: vec![],  // Add this line
                    ..Default::default()  // This will handle any other required fields
                }),
                scope_spans: vec![ScopeSpans {
                    scope: Some(InstrumentationScope {
                        name: "span".to_string(),
                        ..Default::default()
                    }),
                    spans: vec![Span {
                        trace_id: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                        span_id: vec![1, 2, 3, 4, 5, 6, 7, 8],
                        ..Default::default()
                    }],
                    schema_url: "".to_string(),
                }],
                schema_url: "".to_string(),
            }],
        })
    }

    #[test]
    fn test_batch_processor() {
        let test_runtime = TestRuntime::new();
        let processor = BatchProcessor::new(Config {
            send_batch_size: 2,
            timeout: Duration::from_secs(0),
            metadata_keys: vec!["traceId".to_string()],
            metadata_cardinality_limit: 10,
        })
        .unwrap();

        let config = ProcessorConfig::new("batch_processor_test");
        let wrapper = ProcessorWrapper::local(processor, &config);

        test_runtime
            .set_processor(wrapper)
            .run_test(|mut ctx| async move {
                // Create two requests with traceId "1" (should batch)
                let req1 = create_test_trace_request_with_trace_id("1");
                let req1_clone = req1.clone();
                let req2 = create_test_trace_request_with_trace_id("1");
                ctx.process(Message::PData(req1)).await.unwrap();
                ctx.process(Message::PData(req2)).await.unwrap();

                // Create one request with traceId "2" (separate batch)
                let req3 = create_test_trace_request_with_trace_id("2");
                let req3_clone = req3.clone();
                ctx.process(Message::PData(req3)).await.unwrap();

                // Verify results
                if let OTLPRequest::Traces(req) = &req1_clone {
                    let span_count = req.resource_spans
                        .iter()
                        .flat_map(|rs| rs.scope_spans.iter())
                        .flat_map(|ss| ss.spans.iter())
                        .count();
                    assert_eq!(span_count, 1);
                }

                ctx.process(Message::Control(ControlMsg::TimerTick {})).await.unwrap(); // <--- flush emitted
                // Verify first batch has 2 spans (traceId "1")
                if let OTLPRequest::Traces(req) = &req1_clone {
                    println!("req1_clone: {:#?}", req);
                    let span_count = req
                        .resource_spans
                        .iter()
                        .flat_map(|rs| rs.scope_spans.iter())
                        .flat_map(|ss| ss.spans.iter())
                        .count();
                    assert_eq!(span_count, 2); // NOTE: THIS SHOULD BE 2?
                }

                // Verify second batch has 1 span (traceId "2")
                if let OTLPRequest::Traces(req) = &req3_clone {
                    let span_count = req
                        .resource_spans
                        .iter()
                        .flat_map(|rs| rs.scope_spans.iter())
                        .flat_map(|ss| ss.spans.iter())
                        .count();
                    assert_eq!(span_count, 1);
                }
            })
            .validate(|_ctx| async move {
                // Validation is handled by the framework
            });
    }

    #[test]
    fn test_batch_timeout() {
        let test_runtime = TestRuntime::new();
        let processor = BatchProcessor::new(Config {
            send_batch_size: 10,
            timeout: Duration::from_secs(1),
            metadata_keys: vec!["traceId".to_string()],
            metadata_cardinality_limit: 10,
        })
        .unwrap();

        let config = ProcessorConfig::new("batch_processor_test");
        let wrapper = ProcessorWrapper::local(processor, &config);

        test_runtime
            .set_processor(wrapper)
            .run_test(|mut ctx| async move {
                // Create test request
                let req = create_test_trace_request(1);

                // Process one message
                ctx.process(Message::PData(req)).await.unwrap();

                // Wait for timeout to trigger batch
                sleep(Duration::from_secs(1)).await;
            })
            .validate(|_ctx| async move {
                // Validation is handled by the framework
            });
    }

    #[test]
    fn test_cardinality_limit() {
        let test_runtime = TestRuntime::new();
        let processor = BatchProcessor::new(Config {
            send_batch_size: 1,
            timeout: Duration::from_secs(0),
            metadata_keys: vec!["traceId".to_string()],
            metadata_cardinality_limit: 2,
        })
        .unwrap();

        let config = ProcessorConfig::new("batch_processor_test");
        let wrapper = ProcessorWrapper::local(processor, &config);

        test_runtime
            .set_processor(wrapper)
            .run_test(|mut ctx| async move {
                // Create three different trace IDs
                let req1 = create_test_trace_request(1);
                let req2 = create_test_trace_request(2);
                let req3 = create_test_trace_request(3);

                ctx.process(Message::PData(req1)).await.unwrap();
                ctx.process(Message::PData(req2)).await.unwrap();
                ctx.process(Message::PData(req3)).await.unwrap();
            })
            .validate(|_ctx| async move {
                // Validation is handled by the framework
            });
    }

    #[test]
    fn test_combine_batches() {
        // Create three trace requests with the same trace ID
        let trace_id = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let span_id_1 = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let span_id_2 = vec![9, 10, 11, 12, 13, 14, 15, 16];
        let span_id_3 = vec![17, 18, 19, 20, 21, 22, 23, 24];
        let test_runtime = TestRuntime::new();
        let processor = BatchProcessor::new(Config {
            send_batch_size: 10,
            timeout: Duration::from_secs(0),
            metadata_keys: vec!["traceId".to_string()],
            metadata_cardinality_limit: 10,
        })
        .unwrap();

        let config = ProcessorConfig::new("batch_processor_test");
        let wrapper = ProcessorWrapper::local(processor, &config);

        test_runtime
            .set_processor(wrapper)
            .run_test(|mut ctx| async move {
                // Create three different spans in separate requests
                let req1 = OTLPRequest::Traces(ExportTraceServiceRequest {
                    resource_spans: vec![ResourceSpans {
                        resource: Some(Resource {
                            attributes: vec![KeyValue {
                                key: "service.name".to_string(),
                                value: Some(AnyValue {
                                    value: Some(Value::StringValue("service1".to_string())),
                                }),
                            }],
                            dropped_attributes_count: 0,
                            entity_refs: vec![],
                        }),
                        scope_spans: vec![ScopeSpans {
                            scope: Some(InstrumentationScope {
                                name: "span1".to_string(),
                                ..Default::default()
                            }),
                            spans: vec![Span {
                                trace_id: trace_id.clone(),
                                span_id: span_id_1,
                                ..Default::default()
                            }],
                            schema_url: "".to_string(),
                        }],
                        schema_url: "".to_string(),
                    }],
                });

                ctx.process(Message::PData(req1)).await.unwrap();

                let req2 = OTLPRequest::Traces(ExportTraceServiceRequest {
                    resource_spans: vec![ResourceSpans {
                        resource: Some(Resource {
                            attributes: vec![KeyValue {
                                key: "service.name".to_string(),
                                value: Some(AnyValue {
                                    value: Some(Value::StringValue("service1".to_string())),
                                }),
                            }],
                            dropped_attributes_count: 0,
                            entity_refs: vec![],
                        }),
                        scope_spans: vec![ScopeSpans {
                            scope: Some(InstrumentationScope {
                                name: "span2".to_string(),
                                ..Default::default()
                            }),
                            spans: vec![Span {
                                trace_id: trace_id.clone(),
                                span_id: span_id_2,
                                ..Default::default()
                            }],
                            schema_url: "".to_string(),
                        }],
                        schema_url: "".to_string(),
                    }],
                });

                ctx.process(Message::PData(req2)).await.unwrap();

                let req3 = OTLPRequest::Traces(ExportTraceServiceRequest {
                    resource_spans: vec![ResourceSpans {
                        resource: Some(Resource {
                            attributes: vec![KeyValue {
                                key: "service.name".to_string(),
                                value: Some(AnyValue {
                                    value: Some(Value::StringValue("service1".to_string())),
                                }),
                            }], 
                            dropped_attributes_count: 0,
                            entity_refs: vec![],
                        }),
                        scope_spans: vec![ScopeSpans {
                            scope: Some(InstrumentationScope {
                                name: "span3".to_string(),
                                ..Default::default()
                            }),
                            spans: vec![Span {
                                trace_id: trace_id.clone(),
                                span_id: span_id_3,
                                ..Default::default()
                            }],
                            schema_url: "".to_string(),
                        }],
                        schema_url: "".to_string(),
                    }],
                });

                ctx.process(Message::PData(req3)).await.unwrap();
            })
            .validate(|_ctx| async move {
                // Validation is handled by the framework
            });
    }
}
