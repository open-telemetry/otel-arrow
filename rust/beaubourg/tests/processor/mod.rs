use std::sync::Mutex;

use async_trait::async_trait;
use once_cell::sync::Lazy;
use processor::{AsyncProcessor, EngineHandler, Error, ProcessorFactory};
use serde_yaml::Value;
use signal::Signal;

use crate::common::Message;

pub static COUNTERS: Lazy<Mutex<Counters>> = Lazy::new(|| Mutex::new(Counters::default()));

/// A set of counters for the integration tests.
#[derive(Default, Debug)]
pub struct Counters {
    pub init_count: usize,
    pub process_count: usize,
    pub stop_count: usize,
}

pub struct TestProcessor {
    name: String,
}

#[async_trait]
impl AsyncProcessor<Message> for TestProcessor {
    async fn init(&mut self, _engine_handler: &mut EngineHandler) -> Result<(), Error> {
        COUNTERS.lock().expect("lock failed").init_count += 1;
        Ok(())
    }

    async fn process(
        &mut self,
        signal: Signal<Message>,
        effects_handler: &mut beaubourg::processor::effect::EffectHandler<Message>,
    ) -> Result<(), Error> {
        COUNTERS.lock().expect("lock failed").process_count += 1;
        match signal {
            Signal::TimerTick { .. } => Ok(()),
            Signal::Messages { messages } => {
                tracing::trace!(name=%self.name, message_count=%messages.len(), "'processing a new batch of messages");

                // An example where the messages are routed based on the content of the message.
                for msg in messages.iter() {
                    if msg.payload.eq("quit") {
                        effects_handler.emit_message(msg.clone());
                    } else {
                        effects_handler.route_message(&[msg.payload.clone()], msg.clone());
                    }
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

    async fn stop(&mut self) -> Result<(), Error> {
        COUNTERS.lock().expect("lock failed").stop_count += 1;
        Ok(())
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
