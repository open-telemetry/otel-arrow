use async_trait::async_trait;
use processor::{AsyncProcessor, ProcessorFactory};
use serde_yaml::Value;
use signal::Signal;

use crate::common::Message;

pub struct TestProcessor {
    name: String,
}

#[async_trait]
impl AsyncProcessor<Message> for TestProcessor {
    async fn process(
        &mut self,
        signal: Signal<Message>,
        effects_handler: &mut beaubourg::processor::effect::EffectHandler<Message>,
    ) -> Result<(), beaubourg::processor::Error> {
        match signal {
            Signal::TimerTick { .. } => Ok(()),
            Signal::Messages { messages } => {
                tracing::trace!(name=%self.name, message_count=%messages.len(), "'processing a new batch of messages");

                // An example where the messages are routed based on their origin.
                // If all the exporters must receive the same messages then use
                // effect_handler.emit_messages(messages)
                for msg in messages.iter() {
                    effects_handler.route_message(&[msg.origin.clone()], msg.clone());
                }
                Ok(())
            }
            Signal::Stop => Ok(()),
            signal => {
                return Err(beaubourg::processor::Error::UnsupportedEvent {
                    processor: self.name.clone(),
                    signal: signal.to_string(),
                })
            }
        }
    }
}

#[derive(Default)]
pub struct TestProcessorFactory {}

impl ProcessorFactory<Message> for TestProcessorFactory {
    fn create(
        &self,
        processor_name: &str,
        processor_type: &str,
        _config: Value,
    ) -> Result<Box<dyn AsyncProcessor<Message> + Send + Sync>, beaubourg::processor::Error> {
        match processor_type {
            "noop" => {
                let processor = Box::new(TestProcessor {
                    name: processor_name.into(),
                });

                Ok(processor as Box<dyn AsyncProcessor<Message> + Send + Sync>)
            }
            _ => Err(beaubourg::processor::Error::UnknownProcessor {
                processor: processor_name.into(),
                r#type: processor_type.into(),
            }),
        }
    }
}
