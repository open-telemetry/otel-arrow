// SPDX-License-Identifier: Apache-2.0

//! BatchProcessor: Buffers messages and emits them in batches.
use async_trait::async_trait;
use otap_df_engine::error::Error;
use otap_df_engine::message::{ControlMsg, Message};
use log;
use otap_df_engine::processor::{SendEffectHandler, Processor, EffectHandlerTrait};
use std::time::Duration;

/// A processor that buffers messages and emits them in batches.
pub struct BatchProcessor<PData> {
    batch: Vec<PData>,
    batch_size: usize,
}

#[allow(dead_code)]
impl<PData> BatchProcessor<PData> {
    pub fn new(batch_size: usize) -> Self {
        Self {
            batch: Vec::with_capacity(batch_size),
            batch_size,
        }
    }
}

#[async_trait(?Send)]
impl<PData: Clone + Send + 'static> Processor<PData, SendEffectHandler<PData>> for BatchProcessor<PData> {

    async fn process(
        &mut self,
        msg: Message<PData>,
        effect_handler: &mut SendEffectHandler<PData>,
    ) -> Result<(), Error<PData>> {
        match msg {
            Message::PData(data) => {
                self.batch.push(data);
                if self.batch.len() >= self.batch_size {
                    let batch = std::mem::take(&mut self.batch);
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
                            let batch = std::mem::take(&mut self.batch);
                            for item in batch {
                                effect_handler.send_message(item).await?;
                            }
                        }
                        Ok(())
                    }
                    ControlMsg::Shutdown { deadline, reason } => {
                        // Flush any remaining batch on shutdown
                        if !self.batch.is_empty() {
                            let batch = std::mem::take(&mut self.batch);
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

    #[derive(Debug, Clone, PartialEq)]
    struct TestData(pub u32);

    fn make_effect_handler() -> (SendEffectHandler<TestData>, mpsc::Receiver<TestData>) {
        let (tx, rx) = mpsc::channel(10);
        (SendEffectHandler::new("test", tx), rx)
    }

    #[tokio::test]
    async fn test_batch_emission_on_size() {
        let mut processor = BatchProcessor::<TestData>::new(3);
        let (mut effect_handler, mut rx) = make_effect_handler();
        
        // Process first two messages - no batch yet
        processor
            .process(Message::PData(TestData(1)), &mut effect_handler)
            .await
            .unwrap();
        processor
            .process(Message::PData(TestData(2)), &mut effect_handler)
            .await
            .unwrap();
        
        // Process third message - should trigger batch
        processor
            .process(Message::PData(TestData(3)), &mut effect_handler)
            .await
            .unwrap();
        
        // Verify we received all messages in order
        let mut received = Vec::new();
        for _ in 0..3 {
            if let Some(item) = rx.recv().await {
                received.push(item);
            }
        }
        assert_eq!(received, vec![TestData(1), TestData(2), TestData(3)]);
    }

    #[tokio::test]
    async fn test_batch_emission_on_timer_tick() {
        let mut processor = BatchProcessor::<TestData>::new(10);
        let (mut effect_handler, mut rx) = make_effect_handler();
        
        // Process two messages
        processor
            .process(Message::PData(TestData(1)), &mut effect_handler)
            .await
            .unwrap();
        processor
            .process(Message::PData(TestData(2)), &mut effect_handler)
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
        assert_eq!(received, vec![TestData(1), TestData(2)]);
    }

    #[tokio::test]
    async fn test_batch_emission_on_shutdown() {
        let mut processor = BatchProcessor::<TestData>::new(10);
        let (mut effect_handler, mut rx) = make_effect_handler();
        let _ = processor
            .process(Message::PData(TestData(1)), &mut effect_handler)
            .await
            .unwrap();
        let batch = processor
            .process(
                Message::Control(ControlMsg::Shutdown {
                    deadline: Duration::from_secs(5),
                    reason: "bye".to_string()
                }),
                &mut effect_handler,
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_no_batch_on_empty_timer_or_shutdown() {
        let mut processor = BatchProcessor::<TestData>::new(3);
        let (mut effect_handler, mut rx) = make_effect_handler();
        processor
            .process(
                Message::Control(ControlMsg::TimerTick {}),
                &mut effect_handler,
            )
            .await
            .unwrap();
        let batch = processor
            .process(
                Message::Control(ControlMsg::Shutdown {
                    deadline: Duration::from_secs(5),
                    reason: "bye".to_string()
                }),
                &mut effect_handler,
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_ack_control_msg() {
        let mut processor = BatchProcessor::<TestData>::new(3);
        let (mut effect_handler, mut rx) = make_effect_handler();
        processor
            .process(
                Message::Control(ControlMsg::Ack { id: 42 }),
                &mut effect_handler,
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_nack_control_msg() {
        let mut processor = BatchProcessor::<TestData>::new(3);
        let (mut effect_handler, mut rx) = make_effect_handler();
        processor
            .process(
                Message::Control(ControlMsg::Nack {
                    id: 99,
                    reason: "fail".to_string(),
                }),
                &mut effect_handler,
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_config_control_msg() {
        let mut processor = BatchProcessor::<TestData>::new(3);
        let (mut effect_handler, mut rx) = make_effect_handler();
        processor
            .process(
                Message::Control(ControlMsg::Config {
                    config: serde_json::json!({"batch_size": 5}),
                }),
                &mut effect_handler,
            )
            .await
            .unwrap();
    }
}
