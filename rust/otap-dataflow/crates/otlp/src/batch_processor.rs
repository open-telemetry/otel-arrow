// SPDX-License-Identifier: Apache-2.0

//! BatchProcessor: Buffers messages and emits them in batches.
use async_trait::async_trait;
use otap_df_engine::error::Error;
use otap_df_engine::message::{ControlMsg, Message};
use otap_df_engine::processor::{EffectHandler, Processor};
use otap_df_channel::mpsc::{Channel, Receiver};

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
impl<PData: Clone + Send + 'static> Processor for BatchProcessor<PData> {
    type PData = PData;

    async fn process(
        &mut self,
        msg: Message<Self::PData>,
        effect_handler: &mut EffectHandler<Self::PData>,
    ) -> Result<(), Error<Self::PData>> {
        match msg {
            Message::PData(data) => {
                self.batch.push(data);
                if self.batch.len() >= self.batch_size {
                    let out = std::mem::take(&mut self.batch);
                    // Send the batch to the next node
                    for item in out {
                        effect_handler.send_message(item).await?;
                    }
                }
                Ok(())
            }
            Message::Control(ctrl_msg) => {
                println!("[BatchProcessor] Received ControlMsg: {:?}", ctrl_msg);
                match ctrl_msg {
                    ControlMsg::TimerTick { .. } => {
                        if !self.batch.is_empty() {
                            let out = std::mem::take(&mut self.batch);
                            for item in out {
                                effect_handler.send_message(item).await?;
                            }
                        }
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
    use otap_df_channel::mpsc::Channel;
    use serde_json;

    #[derive(Debug, Clone, PartialEq)]
    struct TestData(pub u32);

    fn make_effect_handler() -> (EffectHandler<TestData>, Receiver<TestData>) {
        let (tx, rx) = Channel::new(10);
        (EffectHandler::new("test", tx), rx)
    }

    #[tokio::test]
    async fn test_batch_emission_on_size() {
        let mut processor = BatchProcessor::<TestData>::new(3);
        let (mut effect_handler, mut rx) = make_effect_handler();
        let _ = processor
            .process(Message::PData(TestData(1)), &mut effect_handler)
            .await
            .unwrap();
        let _ = processor
            .process(Message::PData(TestData(2)), &mut effect_handler)
            .await
            .unwrap();
        let _ = processor
            .process(Message::PData(TestData(3)), &mut effect_handler)
            .await
            .unwrap();

        // Verify we received all 3 messages
        let mut received = Vec::new();
        for _ in 0..3 {
            if let Ok(msg) = rx.recv().await {
                received.push(msg);
            }
        }
        assert_eq!(received, vec![TestData(1), TestData(2), TestData(3)]);
    }

    #[tokio::test]
    async fn test_batch_emission_on_timer_tick() {
        let mut processor = BatchProcessor::<TestData>::new(10);
        let (mut effect_handler, mut rx) = make_effect_handler();
        let _ = processor
            .process(Message::PData(TestData(1)), &mut effect_handler)
            .await
            .unwrap();
        let _ = processor
            .process(Message::PData(TestData(2)), &mut effect_handler)
            .await
            .unwrap();
        let _ = processor
            .process(
                Message::Control(ControlMsg::TimerTick {}),
                &mut effect_handler,
            )
            .await
            .unwrap();

        // Verify we received both messages
        let mut received = Vec::new();
        for _ in 0..2 {
            if let Ok(msg) = rx.recv().await {
                received.push(msg);
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
        let _ = processor
            .process(
                Message::Control(ControlMsg::Shutdown {
                    reason: "bye".to_string(),
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
        let _ = processor
            .process(
                Message::Control(ControlMsg::TimerTick {}),
                &mut effect_handler,
            )
            .await
            .unwrap();
        let _ = processor
            .process(
                Message::Control(ControlMsg::Shutdown {
                    reason: "bye".to_string(),
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
        let _ = processor
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
        let _ = processor
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
        let _ = processor
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
