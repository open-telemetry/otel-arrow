// SPDX-License-Identifier: Apache-2.0

//! BatchProcessor: Buffers messages and emits them in batches.
use async_trait::async_trait;
use otap_df_engine::error::Error;
use otap_df_engine::message::{ControlMsg, Message};
use otap_df_engine::processor::{SendEffectHandler, Processor, EffectHandlerTrait};
#[allow(unused_imports)]
use std::time::{Duration, Instant};
use std::collections::HashMap;

/// Configuration for the BatchProcessor.
#[derive(Debug, Clone)]
pub struct Config {
    /// The maximum number of messages to buffer before sending a batch.
    pub send_batch_size: u32,
    /// The maximum allowed value for send_batch_size.
    pub send_batch_max_size: u32,
    /// The timeout after which a batch will be sent even if it's not full.
    pub timeout: Duration,
    /// List of metadata keys to use for batching
    pub metadata_keys: Vec<String>,
    /// Maximum number of distinct metadata combinations to track
    pub metadata_cardinality_limit: u32,
}

impl Config {
    fn validate(&self) -> Result<(), Error<()>> {
        if self.send_batch_size > self.send_batch_max_size && self.send_batch_max_size > 0 {
            return Err(Error::InvalidConfig("send_batch_size must be less than or equal to send_batch_max_size".to_string()));
        }
        if self.metadata_keys.is_empty() {
            return Err(Error::InvalidConfig("metadata_keys must not be empty".to_string()));
        }
        if self.metadata_cardinality_limit == 0 {
            return Err(Error::InvalidConfig("metadata_cardinality_limit must be greater than 0".to_string()));
        }
        Ok(())
    }
}

/// A processor that buffers messages and emits them in batches.
pub struct BatchProcessor<OTLPRequest> {
    /// A map from metadata key combination to batches
    batches: HashMap<Vec<String>, Vec<OTLPRequest>>,
    config: Config,
    /// Track when the last batch was sent for timeout purposes
    last_batch_time: Instant,
}

#[allow(dead_code)]
impl<OTLPRequest> BatchProcessor<OTLPRequest> {
    pub fn new(config: Config) -> Result<Self, Error<OTLPRequest>> {
        // Validate configuration
        config.validate()?;
        
        Ok(Self {
            batches: HashMap::new(),
            config,
            last_batch_time: Instant::now(),
        })
    }

    /// Get the batch vector for a given metadata combination
    fn get_batch(&mut self, metadata: &[String]) -> &mut Vec<OTLPRequest> {
        self.batches.entry(metadata.to_vec())
            .or_insert_with(|| Vec::with_capacity(self.config.send_batch_size as usize))
    }

    /// Extract metadata from the data (implementation will depend on OTLPRequest type)
    fn extract_metadata(&self, data: &OTLPRequest) -> Vec<String> {
        // This is a placeholder, you need to implement the actual metadata extraction
        vec![]
    }
}

#[async_trait(?Send)]
impl<OTLPRequest: Clone + Send + 'static> Processor<OTLPRequest, SendEffectHandler<OTLPRequest>> for BatchProcessor<OTLPRequest> {

    async fn process(
        &mut self,
        msg: Message<OTLPRequest>,
        effect_handler: &mut SendEffectHandler<OTLPRequest>,
    ) -> Result<(), Error<OTLPRequest>> {
        match msg {
            Message::PData(data) => {
                // Extract metadata from the data
                let metadata = self.extract_metadata(&data);
                
                // Get the appropriate batch for this metadata combination
                let batch = self.get_batch(&metadata);
                batch.push(data);
                
                // Check if we should emit this batch
                if batch.len() >= self.config.send_batch_size as usize {
                    let batch = std::mem::replace(batch, Vec::with_capacity(self.config.send_batch_size as usize));
                    for item in batch {
                        effect_handler.send_message(item).await?;
                    }
                    self.last_batch_time = Instant::now();
                } else if self.config.timeout.as_secs() > 0 && self.last_batch_time.elapsed() >= self.config.timeout {
                    let batch = std::mem::replace(batch, Vec::with_capacity(self.config.send_batch_size as usize));
                    for item in batch {
                        effect_handler.send_message(item).await?;
                    }
                    self.last_batch_time = Instant::now();
                }
                Ok(())
            }
            Message::Control(ctrl_msg) => {
                // Log every ControlMsg variant as it is discovered
                log::info!("[BatchProcessor] Received ControlMsg: {:?}", ctrl_msg);
                match ctrl_msg {
                    ControlMsg::TimerTick { .. } => {
                        if !self.batches.is_empty() {
                            for (_, batch) in self.batches.drain() {
                                for item in batch {
                                    effect_handler.send_message(item).await?;
                                }
                            }
                        }
                        Ok(())
                    }
                    ControlMsg::Shutdown { deadline: _, reason: _ } => {
                        // Flush any remaining batch on shutdown
                        if !self.batches.is_empty() {
                            for (_, batch) in self.batches.drain() {
                                for item in batch {
                                    effect_handler.send_message(item).await?;
                                }
                            }
                        }
                        Ok(())
                    }
                    ControlMsg::Ack { id: _ } => {
                        // Optionally handle ack logic here (e.g., mark message as acknowledged)
                        Ok(())
                    }
                    ControlMsg::Nack { id: _, reason: _ } => {
                        // Optionally handle nack logic here (e.g., log or retry)
                        Ok(())
                    }
                    ControlMsg::Config { config: _ } => {
                        // Optionally handle config update here (e.g., update batch_size)
                        Ok(())
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    #[tokio::test]
    async fn test_batch_emission_on_size_with_metadata() {
        let mut processor = BatchProcessor::<OTLPRequest>::new(Config {
            send_batch_size: 2,
            send_batch_max_size: 10,
            timeout: Duration::from_secs(0),
            metadata_keys: vec!["traceId".to_string()],
            metadata_cardinality_limit: 10,
        }).unwrap();
        let (mut effect_handler, mut rx) = make_effect_handler();
        
        // Create test requests with different metadata
        let mut req1 = create_test_trace_request(1);
        let mut req2 = create_test_trace_request(2);
        let mut req3 = create_test_trace_request(3);
        
        // Add different metadata to requests
        req1.metadata.insert("traceId".to_string(), "trace1".to_string());
        req2.metadata.insert("traceId".to_string(), "trace1".to_string());
        req3.metadata.insert("traceId".to_string(), "trace2".to_string());
        
        // Process first two requests with same metadata - should trigger batch
        processor
            .process(Message::PData(req1), &mut effect_handler)
            .await
            .unwrap();
        processor
            .process(Message::PData(req2), &mut effect_handler)
            .await
            .unwrap();
        
        // Verify we received both messages from trace1 batch
        let mut received = Vec::new();
        for _ in 0..2 {
            if let Some(item) = rx.recv().await {
                received.push(item);
            }
        }
        assert_eq!(received.len(), 2);
        
        // Process third request with different metadata
        processor
            .process(Message::PData(req3), &mut effect_handler)
            .await
            .unwrap();
        
        // Verify we received the trace2 batch
        let received = rx.recv().await;
        assert!(received.is_some());
    }

    #[tokio::test]
    async fn test_batch_timeout() {
        let timeout = Duration::from_secs(1);
        let mut processor = BatchProcessor::<OTLPRequest>::new(Config {
            send_batch_size: 10,
            send_batch_max_size: 10,
            timeout,
            metadata_keys: vec!["traceId".to_string()],
            metadata_cardinality_limit: 10,
        }).unwrap();
        let (mut effect_handler, mut rx) = make_effect_handler();
        
        // Create test request
        let req = create_test_trace_request(1);
        
        // Process one message
        processor
            .process(Message::PData(req), &mut effect_handler)
            .await
            .unwrap();
        
        // Wait for timeout to trigger batch
        tokio::time::sleep(timeout + Duration::from_millis(100)).await;
        
        // Verify message was sent
        let received = rx.recv().await;
        assert!(received.is_some());
    }

    #[tokio::test]
    async fn test_timer_tick_with_metadata() {
        let mut processor = BatchProcessor::<OTLPRequest>::new(Config {
            send_batch_size: 10,
            send_batch_max_size: 10,
            timeout: Duration::from_secs(0),
            metadata_keys: vec!["traceId".to_string()],
            metadata_cardinality_limit: 10,
        }).unwrap();
        let (mut effect_handler, mut rx) = make_effect_handler();
        
        // Create test requests with different metadata
        let mut req1 = create_test_trace_request(1);
        let mut req2 = create_test_trace_request(2);
        
        // Add different metadata
        req1.metadata.insert("traceId".to_string(), "trace1".to_string());
        req2.metadata.insert("traceId".to_string(), "trace2".to_string());
        
        // Process messages
        processor
            .process(Message::PData(req1), &mut effect_handler)
            .await
            .unwrap();
        processor
            .process(Message::PData(req2), &mut effect_handler)
            .await
            .unwrap();
        
        // Send timer tick
        processor
            .process(Message::Control(ControlMsg::TimerTick {}), &mut effect_handler)
            .await
            .unwrap();
        
        // Verify we received both messages
        let mut received = Vec::new();
        for _ in 0..2 {
            if let Some(item) = rx.recv().await {
                received.push(item);
            }
        }
        assert_eq!(received.len(), 2);
    }

    #[tokio::test]
    async fn test_shutdown_with_metadata() {
        let mut processor = BatchProcessor::<OTLPRequest>::new(Config {
            send_batch_size: 10,
            send_batch_max_size: 10,
            timeout: Duration::from_secs(0),
            metadata_keys: vec!["traceId".to_string()],
            metadata_cardinality_limit: 10,
        }).unwrap();
        let (mut effect_handler, mut rx) = make_effect_handler();
        
        // Create test requests with different metadata
        let mut req1 = create_test_trace_request(1);
        let mut req2 = create_test_trace_request(2);
        
        // Add different metadata
        req1.metadata.insert("traceId".to_string(), "trace1".to_string());
        req2.metadata.insert("traceId".to_string(), "trace2".to_string());
        
        // Process messages
        processor
            .process(Message::PData(req1), &mut effect_handler)
            .await
            .unwrap();
        processor
            .process(Message::PData(req2), &mut effect_handler)
            .await
            .unwrap();
        
        // Send shutdown
        processor
            .process(Message::Control(ControlMsg::Shutdown {
                deadline: Duration::from_secs(5),
                reason: "bye".to_string()
            }), &mut effect_handler)
            .await
            .unwrap();
        
        // Verify we received both messages
        let mut received = Vec::new();
        while let Ok(item) = rx.try_recv() {
            received.push(item);
        }
        assert_eq!(received.len(), 2);
    }

    #[tokio::test]
    async fn test_config_validation() {
        // Test invalid batch size configuration
        let invalid_config = Config {
            send_batch_size: 10,
            send_batch_max_size: 5,
            timeout: Duration::from_secs(0),
            metadata_keys: vec!["traceId".to_string()],
            metadata_cardinality_limit: 10,
        };
        assert!(BatchProcessor::<OTLPRequest>::new(invalid_config).is_err());

        // Test valid configuration
        let valid_config = Config {
            send_batch_size: 5,
            send_batch_max_size: 10,
            timeout: Duration::from_secs(0),
            metadata_keys: vec!["traceId".to_string()],
            metadata_cardinality_limit: 10,
        };
        assert!(BatchProcessor::<OTLPRequest>::new(valid_config).is_ok());
    }

    #[tokio::test]
    async fn test_empty_timer_tick() {
        let mut processor = BatchProcessor::<OTLPRequest>::new(Config {
            send_batch_size: 10,
            send_batch_max_size: 10,
            timeout: Duration::from_secs(0),
            metadata_keys: vec!["traceId".to_string()],
            metadata_cardinality_limit: 10,
        }).unwrap();
        let (mut effect_handler, mut rx) = make_effect_handler();
        
        // Send timer tick with empty batches - should not panic
        processor
            .process(Message::Control(ControlMsg::TimerTick {}), &mut effect_handler)
            .await
            .unwrap();
        
        // Verify no messages were sent
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn test_empty_shutdown() {
        let mut processor = BatchProcessor::<OTLPRequest>::new(Config {
            send_batch_size: 10,
            send_batch_max_size: 10,
            timeout: Duration::from_secs(0),
            metadata_keys: vec!["traceId".to_string()],
            metadata_cardinality_limit: 10,
        }).unwrap();
        let (mut effect_handler, mut rx) = make_effect_handler();
        
        // Send shutdown with empty batches - should not panic
        processor
            .process(Message::Control(ControlMsg::Shutdown {
                deadline: Duration::from_secs(5),
                reason: "bye".to_string()
            }), &mut effect_handler)
            .await
            .unwrap();
        
        // Verify no messages were sent
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn test_metadata_extraction_traces() {
        // Create a test trace request with multiple spans
        let mut processor = BatchProcessor::<OTLPRequest>::new(Config {
            send_batch_size: 10,
            send_batch_max_size: 10,
            timeout: Duration::from_secs(0),
            metadata_keys: vec!["traceId".to_string()],
            metadata_cardinality_limit: 10,
        }).unwrap();
        
        // Create test trace request with multiple spans
        let trace_id = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let span_id = vec![1, 2, 3, 4, 5, 6, 7, 8];
        
        let request = ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: None,
                scope_spans: vec![ScopeSpans {
                    scope: None,
                    spans: vec![
                        Span {
                            trace_id: trace_id.clone(),
                            span_id: span_id.clone(),
                            trace_state: "".to_string(),
                            parent_span_id: vec![],
                            name: "test-span-1".to_string(),
                            kind: 1,
                            start_time_unix_nano: 0,
                            end_time_unix_nano: 0,
                            ..Default::default()
                        },
                        Span {
                            trace_id: trace_id.clone(),
                            span_id: vec![9, 10, 11, 12, 13, 14, 15, 16],
                            trace_state: "".to_string(),
                            parent_span_id: vec![],
                            name: "test-span-2".to_string(),
                            kind: 1,
                            start_time_unix_nano: 0,
                            end_time_unix_nano: 0,
                            ..Default::default()
                        },
                    ],
                    schema_url: "".to_string(),
                }],
                schema_url: "".to_string(),
            }],
        };
        
        let data = OTLPRequest::Traces(request);
        let metadata = processor.extract_metadata(&data);
        
        // Verify we get metadata for both spans
        assert_eq!(metadata.len(), 4); // 2 spans * 2 metadata fields each
        assert!(metadata.contains(&format!("traceId:{}", hex::encode(&trace_id))));
        assert!(metadata.contains(&format!("spanId:{}", hex::encode(&span_id))));
        assert!(metadata.contains(&format!("spanId:{}", hex::encode(&vec![9, 10, 11, 12, 13, 14, 15, 16]))));
    }

    #[tokio::test]
    async fn test_metadata_extraction_metrics() {
        let mut processor = BatchProcessor::<OTLPRequest>::new(Config {
            send_batch_size: 10,
            send_batch_max_size: 10,
            timeout: Duration::from_secs(0),
            metadata_keys: vec!["traceId".to_string()],
            metadata_cardinality_limit: 10,
        }).unwrap();
        
        // Create test metrics request with labels
        let request = ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                resource: None,
                scope_metrics: vec![ScopeMetrics {
                    scope: None,
                    metrics: vec![Metric {
                        name: "test.metric".to_string(),
                        description: "".to_string(),
                        unit: "".to_string(),
                        labels: Some(
                            vec![
                                ("env".to_string(), "prod".to_string()),
                                ("region".to_string(), "us-east-1".to_string()),
                            ].into_iter().collect()
                        ),
                        ..Default::default()
                    }],
                    schema_url: "".to_string(),
                }],
                schema_url: "".to_string(),
            }],
        };
        
        let data = OTLPRequest::Metrics(request);
        let metadata = processor.extract_metadata(&data);
        
        // Verify we get all metadata fields
        assert_eq!(metadata.len(), 3);
        assert!(metadata.contains(&"metricName:test.metric".to_string()));
        assert!(metadata.contains(&"label:env:prod".to_string()));
        assert!(metadata.contains(&"label:region:us-east-1".to_string()));
    }

    #[tokio::test]
    async fn test_metadata_extraction_logs() {
        let mut processor = BatchProcessor::<OTLPRequest>::new(Config {
            send_batch_size: 10,
            send_batch_max_size: 10,
            timeout: Duration::from_secs(0),
            metadata_keys: vec!["traceId".to_string()],
            metadata_cardinality_limit: 10,
        }).unwrap();
        
        // Create test logs request with trace context
        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord {
                        time_unix_nano: 0,
                        trace_id: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                        span_id: vec![1, 2, 3, 4, 5, 6, 7, 8],
                        ..Default::default()
                    },
                    LogRecord {
                        time_unix_nano: 0,
                        trace_id: vec![],  // Empty trace_id
                        span_id: vec![],   // Empty span_id
                        ..Default::default()
                    }],
                    schema_url: "".to_string(),
                }],
                schema_url: "".to_string(),
            }],
        };
        
        let data = OTLPRequest::Logs(request);
        let metadata = processor.extract_metadata(&data);
        
        // Verify we get metadata for the first log record (with trace context)
        assert_eq!(metadata.len(), 2);
        assert!(metadata.contains(&format!("traceId:{}", hex::encode(&vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]))));
        assert!(metadata.contains(&format!("spanId:{}", hex::encode(&vec![1, 2, 3, 4, 5, 6, 7, 8]))));
    }

    #[tokio::test]
    async fn test_batch_emission_with_metadata() {
        let mut processor = BatchProcessor::<OTLPRequest>::new(Config {
            send_batch_size: 2,
            send_batch_max_size: 10,
            timeout: Duration::from_secs(0),
            metadata_keys: vec!["traceId".to_string()],
            metadata_cardinality_limit: 10,
        }).unwrap();
        let (mut effect_handler, mut rx) = make_effect_handler();
        
        // Create test requests with different metadata
        let trace_id_1 = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let trace_id_2 = vec![17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32];
        
        let req1 = OTLPRequest::Traces(ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: None,
                scope_spans: vec![ScopeSpans {
                    scope: None,
                    spans: vec![
                        Span {
                            trace_id: trace_id_1.clone(),
                            span_id: vec![1, 2, 3, 4, 5, 6, 7, 8],
                            ..Default::default()
                        },
                        Span {
                            trace_id: trace_id_1.clone(),
                            span_id: vec![9, 10, 11, 12, 13, 14, 15, 16],
                            ..Default::default()
                        },
                    ],
                    schema_url: "".to_string(),
                }],
                schema_url: "".to_string(),
            }],
        });
        
        let req2 = OTLPRequest::Traces(ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: None,
                scope_spans: vec![ScopeSpans {
                    scope: None,
                    spans: vec![
                        Span {
                            trace_id: trace_id_2.clone(),
                            span_id: vec![1, 2, 3, 4, 5, 6, 7, 8],
                            ..Default::default()
                        },
                        Span {
                            trace_id: trace_id_2.clone(),
                            span_id: vec![9, 10, 11, 12, 13, 14, 15, 16],
                            ..Default::default()
                        },
                    ],
                    schema_url: "".to_string(),
                }],
                schema_url: "".to_string(),
            }],
        });
        
        // Process first two requests with same metadata - should trigger batch
        processor
            .process(Message::PData(req1), &mut effect_handler)
            .await
            .unwrap();
        processor
            .process(Message::PData(req2), &mut effect_handler)
            .await
            .unwrap();
        
        // Verify we received both messages from trace1 batch
        let mut received = Vec::new();
        for _ in 0..2 {
            if let Some(item) = rx.recv().await {
                received.push(item);
            }
        }
        assert_eq!(received.len(), 2);
        
        // Process third request with different metadata
        processor
            .process(Message::PData(req2), &mut effect_handler)
            .await
            .unwrap();
        
        // Verify we received the trace2 batch
        let received = rx.recv().await;
        assert!(received.is_some());
    }

    #[tokio::test]
    async fn test_metadata_config_validation() {
        // Test empty metadata keys
        let invalid_config = Config {
            send_batch_size: 10,
            send_batch_max_size: 10,
            timeout: Duration::from_secs(0),
            metadata_keys: vec![],
            metadata_cardinality_limit: 10,
        };
        assert!(BatchProcessor::<OTLPRequest>::new(invalid_config).is_err());
        
        // Test zero cardinality limit
        let invalid_config = Config {
            send_batch_size: 10,
            send_batch_max_size: 10,
            timeout: Duration::from_secs(0),
            metadata_keys: vec!["traceId".to_string()],
            metadata_cardinality_limit: 0,
        };
        assert!(BatchProcessor::<OTLPRequest>::new(invalid_config).is_err());
        
        // Test valid configuration
        let valid_config = Config {
            send_batch_size: 10,
            send_batch_max_size: 10,
            timeout: Duration::from_secs(0),
            metadata_keys: vec!["traceId".to_string()],
            metadata_cardinality_limit: 10,
        };
        assert!(BatchProcessor::<OTLPRequest>::new(valid_config).is_ok());
    }

    #[tokio::test]
    async fn test_cardinality_limit() {
        let mut processor = BatchProcessor::<OTLPRequest>::new(Config {
            send_batch_size: 1,
            send_batch_max_size: 10,
            timeout: Duration::from_secs(0),
            metadata_keys: vec!["traceId".to_string()],
            metadata_cardinality_limit: 2,
        }).unwrap();
        let (mut effect_handler, mut rx) = make_effect_handler();
        
        // Create three different trace IDs
        let trace_id_1 = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let trace_id_2 = vec![17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32];
        let trace_id_3 = vec![33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48];
        
        // First trace ID should create a new batch
        let req1 = OTLPRequest::Traces(ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: None,
                scope_spans: vec![ScopeSpans {
                    scope: None,
                    spans: vec![Span {
                        trace_id: trace_id_1.clone(),
                        span_id: vec![1, 2, 3, 4, 5, 6, 7, 8],
                        ..Default::default()
                    }],
                    schema_url: "".to_string(),
                }],
                schema_url: "".to_string(),
            }],
        });
        processor.process(Message::PData(req1), &mut effect_handler).await.unwrap();
        assert!(rx.try_recv().is_ok()); // First batch should be sent
        
        // Second trace ID should create another batch
        let req2 = OTLPRequest::Traces(ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: None,
                scope_spans: vec![ScopeSpans {
                    scope: None,
                    spans: vec![Span {
                        trace_id: trace_id_2.clone(),
                        span_id: vec![1, 2, 3, 4, 5, 6, 7, 8],
                        ..Default::default()
                    }],
                    schema_url: "".to_string(),
                }],
                schema_url: "".to_string(),
            }],
        });
        processor.process(Message::PData(req2), &mut effect_handler).await.unwrap();
        assert!(rx.try_recv().is_ok()); // Second batch should be sent
        
        // Third trace ID should evict one of the existing batches
        let req3 = OTLPRequest::Traces(ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: None,
                scope_spans: vec![ScopeSpans {
                    scope: None,
                    spans: vec![Span {
                        trace_id: trace_id_3.clone(),
                        span_id: vec![1, 2, 3, 4, 5, 6, 7, 8],
                        ..Default::default()
                    }],
                    schema_url: "".to_string(),
                }],
                schema_url: "".to_string(),
            }],
        });
        processor.process(Message::PData(req3), &mut effect_handler).await.unwrap();
        assert!(rx.try_recv().is_ok()); // Third batch should be sent
    }

    #[tokio::test]
    async fn test_multiple_metadata_keys() {
        let mut processor = BatchProcessor::<OTLPRequest>::new(Config {
            send_batch_size: 1,
            send_batch_max_size: 10,
            timeout: Duration::from_secs(0),
            metadata_keys: vec!["traceId".to_string(), "spanId".to_string()],
            metadata_cardinality_limit: 10,
        }).unwrap();
        let (mut effect_handler, mut rx) = make_effect_handler();
        
        // Create two spans with same trace but different span IDs
        let trace_id = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let span_id_1 = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let span_id_2 = vec![9, 10, 11, 12, 13, 14, 15, 16];
        
        let req1 = OTLPRequest::Traces(ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: None,
                scope_spans: vec![ScopeSpans {
                    scope: None,
                    spans: vec![Span {
                        trace_id: trace_id.clone(),
                        span_id: span_id_1.clone(),
                        ..Default::default()
                    }],
                    schema_url: "".to_string(),
                }],
                schema_url: "".to_string(),
            }],
        });
        processor.process(Message::PData(req1), &mut effect_handler).await.unwrap();
        assert!(rx.try_recv().is_ok()); // First batch should be sent
        
        let req2 = OTLPRequest::Traces(ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: None,
                scope_spans: vec![ScopeSpans {
                    scope: None,
                    spans: vec![Span {
                        trace_id: trace_id.clone(),
                        span_id: span_id_2.clone(),
                        ..Default::default()
                    }],
                    schema_url: "".to_string(),
                }],
                schema_url: "".to_string(),
            }],
        });
        processor.process(Message::PData(req2), &mut effect_handler).await.unwrap();
        assert!(rx.try_recv().is_ok()); // Second batch should be sent
    }

    #[tokio::test]
    async fn test_full_trace_batching() {
        let mut processor = BatchProcessor::<OTLPRequest>::new(Config {
            send_batch_size: 2,
            send_batch_max_size: 10,
            timeout: Duration::from_secs(0),
            metadata_keys: vec!["traceId".to_string()],
            metadata_cardinality_limit: 10,
        }).unwrap();
        let (mut effect_handler, mut rx) = make_effect_handler();

        // Create a realistic trace with multiple spans
        let trace_id = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let span_id_parent = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let span_id_child = vec![9, 10, 11, 12, 13, 14, 15, 16];

        // Create parent span (HTTP request)
        let parent_span = Span {
            trace_id: trace_id.clone(),
            span_id: span_id_parent.clone(),
            trace_state: "".to_string(),
            parent_span_id: vec![],
            name: "HTTP GET /users".to_string(),
            kind: 1, // SPAN_KIND_CLIENT
            start_time_unix_nano: 1684273550000000000,
            end_time_unix_nano: 1684273550100000000,
            attributes: vec![
                KeyValue {
                    key: "http.method".to_string(),
                    value: Some(AnyValue {
                        value: Some(any_value::Value::StringValue("GET".to_string())),
                    }),
                },
                KeyValue {
                    key: "http.url".to_string(),
                    value: Some(AnyValue {
                        value: Some(any_value::Value::StringValue("/users".to_string())),
                    }),
                },
            ],
            status: Some(Status {
                code: 1, // STATUS_OK
                message: "".to_string(),
            }),
            ..Default::default()
        };

        // Create child span (database query)
        let child_span = Span {
            trace_id: trace_id.clone(),
            span_id: span_id_child.clone(),
            trace_state: "".to_string(),
            parent_span_id: span_id_parent.clone(),
            name: "SQL SELECT users".to_string(),
            kind: 2, // SPAN_KIND_INTERNAL
            start_time_unix_nano: 1684273550050000000,
            end_time_unix_nano: 1684273550090000000,
            attributes: vec![
                KeyValue {
                    key: "db.statement".to_string(),
                    value: Some(AnyValue {
                        value: Some(any_value::Value::StringValue("SELECT * FROM users".to_string())),
                    }),
                },
            ],
            status: Some(Status {
                code: 1, // STATUS_OK
                message: "".to_string(),
            }),
            ..Default::default()
        };

        // Create the trace request with both spans
        let request = ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: Some(Resource {
                    attributes: vec![
                        KeyValue {
                            key: "service.name".to_string(),
                            value: Some(AnyValue {
                                value: Some(any_value::Value::StringValue("user-service".to_string())),
                            }),
                        },
                    ],
                    ..Default::default()
                }),
                scope_spans: vec![ScopeSpans {
                    scope: Some(InstrumentationScope {
                        name: "user-service".to_string(),
                        version: Some("1.0.0".to_string()),
                        ..Default::default()
                    }),
                    spans: vec![parent_span, child_span],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };

        // Process the trace (should create a batch)
        let data = OTLPRequest::Traces(request);
        processor.process(Message::PData(data), &mut effect_handler).await.unwrap();

        // Verify we got a batch with both spans
        let received = rx.recv().await.unwrap();
        if let OTLPRequest::Traces(received_request) = received {
            assert_eq!(received_request.resource_spans.len(), 1);
            assert_eq!(received_request.resource_spans[0].scope_spans.len(), 1);
            assert_eq!(received_request.resource_spans[0].scope_spans[0].spans.len(), 2);
        } else {
            panic!("Unexpected message type");
        }
    }

    #[tokio::test]
    async fn test_full_metrics_batching() {
        let mut processor = BatchProcessor::<OTLPRequest>::new(Config {
            send_batch_size: 2,
            send_batch_max_size: 10,
            timeout: Duration::from_secs(0),
            metadata_keys: vec!["metricName".to_string(), "label:env".to_string()],
            metadata_cardinality_limit: 10,
        }).unwrap();
        let (mut effect_handler, mut rx) = make_effect_handler();

        // Create a realistic metrics request with multiple metrics
        let request = ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                resource: Some(Resource {
                    attributes: vec![
                        KeyValue {
                            key: "service.name".to_string(),
                            value: Some(AnyValue {
                                value: Some(any_value::Value::StringValue("metrics-service".to_string())),
                            }),
                        },
                    ],
                    ..Default::default()
                }),
                scope_metrics: vec![ScopeMetrics {
                    scope: Some(InstrumentationScope {
                        name: "metrics-service".to_string(),
                        version: Some("1.0.0".to_string()),
                        ..Default::default()
                    }),
                    metrics: vec![
                        // Counter metric
                        Metric {
                            name: "http.requests".to_string(),
                            description: "Number of HTTP requests".to_string(),
                            unit: "1".to_string(),
                            data: Some(metric::Data::Sum(Sum {
                                data_points: vec![
                                    NumberDataPoint {
                                        attributes: vec![
                                            KeyValue {
                                                key: "env".to_string(),
                                                value: Some(AnyValue {
                                                    value: Some(any_value::Value::StringValue("prod".to_string())),
                                                }),
                                            },
                                        ],
                                        start_time_unix_nano: 1684273550000000000,
                                        time_unix_nano: 1684273550100000000,
                                        value: Some(number_data_point::Value::AsDouble(100.0)),
                                        ..Default::default()
                                    },
                                ],
                                aggregation_temporality: 2, // CUMULATIVE
                                is_monotonic: true,
                            })),
                            ..Default::default()
                        },
                        // Gauge metric
                        Metric {
                            name: "memory.usage".to_string(),
                            description: "Memory usage in bytes".to_string(),
                            unit: "By".to_string(),
                            data: Some(metric::Data::Gauge(Gauge {
                                data_points: vec![
                                    NumberDataPoint {
                                        attributes: vec![
                                            KeyValue {
                                                key: "env".to_string(),
                                                value: Some(AnyValue {
                                                    value: Some(any_value::Value::StringValue("prod".to_string())),
                                                }),
                                            },
                                        ],
                                        time_unix_nano: 1684273550100000000,
                                        value: Some(number_data_point::Value::AsInt(1048576)),
                                        ..Default::default()
                                    },
                                ],
                            })),
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };

        // Process the metrics (should create a batch)
        let data = OTLPRequest::Metrics(request);
        processor.process(Message::PData(data), &mut effect_handler).await.unwrap();

        // Verify we got a batch with both metrics
        let received = rx.recv().await.unwrap();
        if let OTLPRequest::Metrics(received_request) = received {
            assert_eq!(received_request.resource_metrics.len(), 1);
            assert_eq!(received_request.resource_metrics[0].scope_metrics.len(), 1);
            assert_eq!(received_request.resource_metrics[0].scope_metrics[0].metrics.len(), 2);
        } else {
            panic!("Unexpected message type");
        }
    }

    #[tokio::test]
    async fn test_full_logs_batching() {
        let mut processor = BatchProcessor::<OTLPRequest>::new(Config {
            send_batch_size: 2,
            send_batch_max_size: 10,
            timeout: Duration::from_secs(0),
            metadata_keys: vec!["traceId".to_string(), "spanId".to_string()],
            metadata_cardinality_limit: 10,
        }).unwrap();
        let (mut effect_handler, mut rx) = make_effect_handler();

        // Create a realistic logs request with multiple log records
        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource {
                    attributes: vec![
                        KeyValue {
                            key: "service.name".to_string(),
                            value: Some(AnyValue {
                                value: Some(any_value::Value::StringValue("logs-service".to_string())),
                            }),
                        },
                    ],
                    ..Default::default()
                }),
                scope_logs: vec![ScopeLogs {
                    scope: Some(InstrumentationScope {
                        name: "logs-service".to_string(),
                        version: Some("1.0.0".to_string()),
                        ..Default::default()
                    }),
                    log_records: vec![
                        // Log with trace context
                        LogRecord {
                            time_unix_nano: 1684273550000000000,
                            trace_id: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                            span_id: vec![1, 2, 3, 4, 5, 6, 7, 8],
                            severity_text: Some("INFO".to_string()),
                            body: Some(AnyValue {
                                value: Some(any_value::Value::StringValue("User login successful".to_string())),
                            }),
                            attributes: vec![
                                KeyValue {
                                    key: "user.id".to_string(),
                                    value: Some(AnyValue {
                                        value: Some(any_value::Value::StringValue("123".to_string())),
                                    }),
                                },
                            ],
                            ..Default::default()
                        },
                        // Another log with same trace context
                        LogRecord {
                            time_unix_nano: 1684273550100000000,
                            trace_id: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                            span_id: vec![1, 2, 3, 4, 5, 6, 7, 8],
                            severity_text: Some("INFO".to_string()),
                            body: Some(AnyValue {
                                value: Some(any_value::Value::StringValue("User session started".to_string())),
                            }),
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };

        // Process the logs (should create a batch)
        let data = OTLPRequest::Logs(request);
        processor.process(Message::PData(data), &mut effect_handler).await.unwrap();

        // Verify we got a batch with both logs
        let received = rx.recv().await.unwrap();
        if let OTLPRequest::Logs(received_request) = received {
            assert_eq!(received_request.resource_logs.len(), 1);
            assert_eq!(received_request.resource_logs[0].scope_logs.len(), 1);
            assert_eq!(received_request.resource_logs[0].scope_logs[0].log_records.len(), 2);
        } else {
            panic!("Unexpected message type");
        }
    }
}
