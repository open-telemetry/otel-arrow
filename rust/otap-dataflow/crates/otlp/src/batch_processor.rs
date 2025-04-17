// SPDX-License-Identifier: Apache-2.0

//! BatchProcessor: Buffers messages and emits them in batches.
use async_trait::async_trait;
use otap_df_engine::error::Error;
use otap_df_engine::message::{ControlMsg, Message};
use otap_df_engine::processor::{EffectHandler, Processor};

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
    type Msg = PData;

    async fn process(
        &mut self,
        msg: Message<Self::Msg>,
        _effect_handler: &mut EffectHandler<Self::Msg>,
    ) -> Result<Option<Vec<Self::Msg>>, Error<Self::Msg>> {
        match msg {
            Message::PData(data) => {
                self.batch.push(data);
                if self.batch.len() >= self.batch_size {
                    let out = std::mem::take(&mut self.batch);
                    Ok(Some(out))
                } else {
                    Ok(None)
                }
            }
            Message::Control(ctrl_msg) => {
                // Log every ControlMsg variant as it is discovered
                println!("[BatchProcessor] Received ControlMsg: {:?}", ctrl_msg);
                match ctrl_msg {
                    ControlMsg::TimerTick { .. } => {
                        if !self.batch.is_empty() {
                            let out = std::mem::take(&mut self.batch);
                            Ok(Some(out))
                        } else {
                            Ok(None)
                        }
                    }
                    ControlMsg::Shutdown { .. } => {
                        // Flush any remaining batch on shutdown
                        if !self.batch.is_empty() {
                            let out = std::mem::take(&mut self.batch);
                            Ok(Some(out))
                        } else {
                            Ok(None)
                        }
                    }
                    ControlMsg::Ack { id: _ } => {
                        // Optionally handle ack logic here (e.g., mark message as acknowledged)
                        Ok(None)
                    }
                    ControlMsg::Nack { id: _, reason: _ } => {
                        // Optionally handle nack logic here (e.g., log or retry)
                        Ok(None)
                    }
                    ControlMsg::Config { config: _ } => {
                        // Optionally handle config update here (e.g., update batch_size)
                        Ok(None)
                    }
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

    fn make_effect_handler() -> EffectHandler<TestData> {
        let (tx, _rx) = Channel::new(10);
        EffectHandler::new("test", tx)
    }

    #[tokio::test]
    async fn test_batch_emission_on_size() {
        let mut processor = BatchProcessor::<TestData>::new(3);
        let mut effect_handler = make_effect_handler();
        let _ = processor
            .process(Message::PData(TestData(1)), &mut effect_handler)
            .await
            .unwrap();
        let _ = processor
            .process(Message::PData(TestData(2)), &mut effect_handler)
            .await
            .unwrap();
        let batch = processor
            .process(Message::PData(TestData(3)), &mut effect_handler)
            .await
            .unwrap();
        assert_eq!(batch, Some(vec![TestData(1), TestData(2), TestData(3)]));
    }

    #[tokio::test]
    async fn test_batch_emission_on_timer_tick() {
        let mut processor = BatchProcessor::<TestData>::new(10);
        let mut effect_handler = make_effect_handler();
        let _ = processor
            .process(Message::PData(TestData(1)), &mut effect_handler)
            .await
            .unwrap();
        let _ = processor
            .process(Message::PData(TestData(2)), &mut effect_handler)
            .await
            .unwrap();
        let batch = processor
            .process(
                Message::Control(ControlMsg::TimerTick {}),
                &mut effect_handler,
            )
            .await
            .unwrap();
        assert_eq!(batch, Some(vec![TestData(1), TestData(2)]));
    }

    #[tokio::test]
    async fn test_batch_emission_on_shutdown() {
        let mut processor = BatchProcessor::<TestData>::new(10);
        let mut effect_handler = make_effect_handler();
        let _ = processor
            .process(Message::PData(TestData(1)), &mut effect_handler)
            .await
            .unwrap();
        let batch = processor
            .process(
                Message::Control(ControlMsg::Shutdown {
                    reason: "bye".to_string(),
                }),
                &mut effect_handler,
            )
            .await
            .unwrap();
        assert_eq!(batch, Some(vec![TestData(1)]));
    }

    #[tokio::test]
    async fn test_no_batch_on_empty_timer_or_shutdown() {
        let mut processor = BatchProcessor::<TestData>::new(3);
        let mut effect_handler = make_effect_handler();
        let batch = processor
            .process(
                Message::Control(ControlMsg::TimerTick {}),
                &mut effect_handler,
            )
            .await
            .unwrap();
        assert_eq!(batch, None);
        let batch = processor
            .process(
                Message::Control(ControlMsg::Shutdown {
                    reason: "bye".to_string(),
                }),
                &mut effect_handler,
            )
            .await
            .unwrap();
        assert_eq!(batch, None);
    }

    #[tokio::test]
    async fn test_ack_control_msg() {
        let mut processor = BatchProcessor::<TestData>::new(3);
        let mut effect_handler = make_effect_handler();
        let batch = processor
            .process(
                Message::Control(ControlMsg::Ack { id: 42 }),
                &mut effect_handler,
            )
            .await
            .unwrap();
        assert_eq!(batch, None);
    }

    #[tokio::test]
    async fn test_nack_control_msg() {
        let mut processor = BatchProcessor::<TestData>::new(3);
        let mut effect_handler = make_effect_handler();
        let batch = processor
            .process(
                Message::Control(ControlMsg::Nack {
                    id: 99,
                    reason: "fail".to_string(),
                }),
                &mut effect_handler,
            )
            .await
            .unwrap();
        assert_eq!(batch, None);
    }

    #[tokio::test]
    async fn test_config_control_msg() {
        let mut processor = BatchProcessor::<TestData>::new(3);
        let mut effect_handler = make_effect_handler();
        let batch = processor
            .process(
                Message::Control(ControlMsg::Config {
                    config: serde_json::json!({"batch_size": 5}),
                }),
                &mut effect_handler,
            )
            .await
            .unwrap();
        assert_eq!(batch, None);
    }
}
