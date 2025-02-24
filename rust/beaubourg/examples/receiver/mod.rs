use async_trait::async_trait;
use beaubourg::receiver::{AsyncReceiver, ReceiverFactory};
use receiver::signal::SignalReceiver;
use serde_yaml::Value;

use crate::common::Message;

pub struct TestReceiver {
    name: String,
}

#[async_trait]
impl AsyncReceiver<Message> for TestReceiver {
    async fn receive(
        &mut self,
        _signal_receiver: SignalReceiver,
        effect_handler: beaubourg::receiver::effect::EffectHandler<Message>,
    ) -> Result<(), beaubourg::receiver::Error> {
        for i in 0..10 {
            effect_handler
                .send_messages(vec![Message {
                    origin: self.name.clone(),
                    payload: i,
                }])
                .await
                .map_err(|e| beaubourg::receiver::Error::Receiver {
                    receiver: self.name.clone(),
                    error: e.to_string(),
                    context: Default::default(),
                })?;
        }

        Ok(())
    }
}

#[derive(Default)]
pub struct TestReceiverFactory {}

impl ReceiverFactory<Message> for TestReceiverFactory {
    fn create(
        &self,
        receiver_name: &str,
        receiver_type: &str,
        _config: Value,
    ) -> Result<Box<dyn AsyncReceiver<Message> + Send + Sync>, beaubourg::receiver::Error> {
        match receiver_type {
            "test" => {
                let receiver = Box::new(TestReceiver {
                    name: receiver_name.into(),
                });

                Ok(receiver as Box<dyn AsyncReceiver<Message> + Send + Sync>)
            }
            _ => Err(beaubourg::receiver::Error::UnknownReceiver {
                receiver: receiver_name.into(),
                r#type: receiver_type.into(),
            }),
        }
    }
}
