// SPDX-License-Identifier: Apache-2.0

//! BatchProcessor: Buffers messages and emits them in batches.
use async_trait::async_trait;
use otap_df_engine::error::Error;
use otap_df_engine::message::{ControlMsg, Message};
use otap_df_engine::processor::{SendEffectHandler, Processor, EffectHandlerTrait};
#[allow(unused_imports)]
use std::time::Duration;

/// Configuration for the BatchProcessor.
#[derive(Debug, Clone)]
pub struct Config {
    /// The maximum number of messages to buffer before sending a batch.
    pub send_batch_size: u32,
    /// The maximum allowed value for send_batch_size.
    pub send_batch_max_size: u32,
}

/// A processor that buffers messages and emits them in batches.
pub struct BatchProcessor<OTLPRequest> {
    batch: Vec<OTLPRequest>,
    config: Config,
}

#[allow(dead_code)]
impl<OTLPRequest> BatchProcessor<OTLPRequest> {
    pub fn new(config: Config) -> Self {
        // Validate configuration
        if config.send_batch_size > config.send_batch_max_size && config.send_batch_max_size > 0 {
            panic!("send_batch_size must be less than or equal to send_batch_max_size");
        }
        Self {
            batch: Vec::with_capacity(config.send_batch_size as usize),
            config,
        }
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
                self.batch.push(data);
                if self.batch.len() >= self.config.send_batch_size as usize {
                    let batch = std::mem::replace(&mut self.batch, Vec::with_capacity(self.config.send_batch_size as usize));
                    for item in batch {
                        effect_handler.send_message(item).await?;
                    }
                }
                Ok(())
            }
            Message::Control(ctrl_msg) => {
                // Log every ControlMsg variant as it is discovered
                log::info!("[BatchProcessor] Received ControlMsg: {:?}", ctrl_msg);
                match ctrl_msg {
                    ControlMsg::TimerTick { .. } => {
                        if !self.batch.is_empty() {
                            let batch = std::mem::replace(&mut self.batch, Vec::with_capacity(self.config.send_batch_size as usize));
                            for item in batch {
                                effect_handler.send_message(item).await?;
                            }
                        }
                        Ok(())
                    }
                    ControlMsg::Shutdown { deadline: _, reason: _ } => {
                        // Flush any remaining batch on shutdown
                        if !self.batch.is_empty() {
                            let batch = std::mem::replace(&mut self.batch, Vec::with_capacity(self.config.send_batch_size as usize));
                            for item in batch {
                                effect_handler.send_message(item).await?;
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
    use tokio::sync::mpsc;
    use serde_json;

    use crate::grpc::OTLPRequest;
    use crate::proto::opentelemetry::collector::trace::v1::ExportTraceServiceRequest;
    use crate::proto::opentelemetry::trace::v1::{ResourceSpans, ScopeSpans, Span};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn make_effect_handler() -> (SendEffectHandler<OTLPRequest>, mpsc::Receiver<OTLPRequest>) {
        let (tx, rx) = mpsc::channel(10);
        (SendEffectHandler::new("test", tx), rx)
    }
    
    fn create_test_trace_request(id: u32) -> OTLPRequest {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
            
        let span = Span {
            trace_id: vec![id as u8; 16],
            span_id: vec![id as u8; 8],
            parent_span_id: vec![],
            name: format!("test-span-{}", id),
            kind: 1, // Internal
            start_time_unix_nano: timestamp,
            end_time_unix_nano: timestamp + 1000,
            ..Default::default()
        };
        
        let scope_spans = ScopeSpans {
            spans: vec![span],
            ..Default::default()
        };
        
        let resource_spans = ResourceSpans {
            scope_spans: vec![scope_spans],
            ..Default::default()
        };
        
        let request = ExportTraceServiceRequest {
            resource_spans: vec![resource_spans],
        };
        
        OTLPRequest::Traces(request)
    }

    #[tokio::test]
    async fn test_batch_emission_on_size() {
        let mut processor = BatchProcessor::<OTLPRequest>::new(Config {
            send_batch_size: 3,
            send_batch_max_size: 10,
        });
        let (mut effect_handler, mut rx) = make_effect_handler();
        
        // Create test trace requests
        let req1 = create_test_trace_request(1);
        let req2 = create_test_trace_request(2);
        let req3 = create_test_trace_request(3);
        
        // Process first two messages - no batch yet
        processor
            .process(Message::PData(req1.clone()), &mut effect_handler)
            .await
            .unwrap();
        processor
            .process(Message::PData(req2.clone()), &mut effect_handler)
            .await
            .unwrap();
        
        // Process third message - should trigger batch
        processor
            .process(Message::PData(req3.clone()), &mut effect_handler)
            .await
            .unwrap();
        
        // Verify we received all messages in order
        let mut received = Vec::new();
        for _ in 0..3 {
            if let Some(item) = rx.recv().await {
                received.push(item);
            }
        }
        
        // We can't directly compare OTLPRequest with ==, so we'll just check the count
        assert_eq!(received.len(), 3);
    }

    #[tokio::test]
    async fn test_batch_emission_on_timer_tick() {
        let mut processor = BatchProcessor::<OTLPRequest>::new(Config {
            send_batch_size: 10,
            send_batch_max_size: 10,
        });
        let (mut effect_handler, mut rx) = make_effect_handler();
        
        // Create test trace requests
        let req1 = create_test_trace_request(1);
        let req2 = create_test_trace_request(2);
        
        // Process two messages
        processor
            .process(Message::PData(req1), &mut effect_handler)
            .await
            .unwrap();
        processor
            .process(Message::PData(req2), &mut effect_handler)
            .await
            .unwrap();
        
        // Send timer tick - should trigger batch
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
    async fn test_batch_emission_on_shutdown() {
        let mut processor = BatchProcessor::<OTLPRequest>::new(Config {
            send_batch_size: 10,
            send_batch_max_size: 10,
        });
        let (mut effect_handler, mut rx) = make_effect_handler();
        
        // Create test trace requests
        let req1 = create_test_trace_request(1);
        let req2 = create_test_trace_request(2);
        
        // Process some messages
        processor
            .process(Message::PData(req1), &mut effect_handler)
            .await
            .unwrap();
        processor
            .process(Message::PData(req2), &mut effect_handler)
            .await
            .unwrap();
        
        // Send shutdown - should trigger batch
        processor
            .process(Message::Control(ControlMsg::Shutdown {
                deadline: Duration::from_secs(5),
                reason: "bye".to_string()
            }), &mut effect_handler)
            .await
            .unwrap();
        
        // Verify we received all messages
        let mut received = Vec::new();
        while let Ok(item) = rx.try_recv() {
            received.push(item);
        }
        assert_eq!(received.len(), 2);
    }

    #[tokio::test]
    async fn test_ack_control_msg() {
        let mut processor = BatchProcessor::<OTLPRequest>::new(Config {
            send_batch_size: 10,
            send_batch_max_size: 10,
        });
        let (mut effect_handler, _rx) = make_effect_handler();
        
        // ACK should be handled without panicking
        processor
            .process(Message::Control(ControlMsg::Ack { id: 1 }), &mut effect_handler)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_no_batch_on_empty_timer_or_shutdown() {
        let mut processor = BatchProcessor::<OTLPRequest>::new(Config {
            send_batch_size: 10,
            send_batch_max_size: 10,
        });
        let (mut effect_handler, mut rx) = make_effect_handler();
        
        // Send timer tick with empty batch - should not panic
        processor
            .process(Message::Control(ControlMsg::TimerTick {}), &mut effect_handler)
            .await
            .unwrap();
            
        // Send shutdown with empty batch - should not panic
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
    async fn test_nack_control_msg() {
        let mut processor = BatchProcessor::<OTLPRequest>::new(Config {
            send_batch_size: 3,
            send_batch_max_size: 10,
        });
        let (mut effect_handler, _rx) = make_effect_handler();
        
        // NACK should be handled without panicking
        processor
            .process(
                Message::Control(ControlMsg::Nack {
                    id: 1,
                    reason: "test".to_string(),
                }),
                &mut effect_handler,
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_config_control_msg() {
        let mut processor = BatchProcessor::<OTLPRequest>::new(Config {
            send_batch_size: 3,
            send_batch_max_size: 10,
        });
        let (mut effect_handler, _rx) = make_effect_handler();
        
        // Config update should be handled without panicking
        processor
            .process(
                Message::Control(ControlMsg::Config {
                    config: serde_json::json!({}),
                }),
                &mut effect_handler,
            )
            .await
            .unwrap();
    }
}
