use async_trait::async_trait;
use exporter::{AsyncExporter, ConcurrencyModel, Error, ExporterBuilder, ExporterFactory};
use serde_yaml::Value;
use signal::{Signal, SignalReceiver};

use crate::common::Message;

pub struct TestExporter {
    name: String,
    message_received: usize,
}

#[async_trait]
impl AsyncExporter<Message> for TestExporter {
    async fn stop(&mut self) -> Result<(), beaubourg::exporter::Error> {
        println!(
            "Exporter '{}' received {} messages (expected 10 messages)",
            self.name, self.message_received
        );
        Ok(())
    }

    async fn export(
        &mut self,
        mut signal_receiver: SignalReceiver<Message>,
        _effects_handler: beaubourg::exporter::effect::EffectHandler<Message>,
    ) -> Result<(), beaubourg::exporter::Error> {
        tracing::trace!("Exporter '{}' started", self.name);
        loop {
            let signal = signal_receiver.recv().await;
            tracing::trace!("Exporter '{}' received signal: {:?}", self.name, signal);
            match signal {
                Signal::TimerTick { .. } => { /* do nothing */ }
                Signal::Messages { messages } => {
                    self.message_received += messages.len();
                }
                Signal::Stop => break,
                signal => {
                    return Err(beaubourg::exporter::Error::UnsupportedEvent {
                        exporter: self.name.clone(),
                        signal: signal.to_string(),
                    })
                }
            }
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct TestExporterFactory {}

struct TestExporterBuilder {
    name: String,
}

impl ExporterBuilder<Message> for TestExporterBuilder {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn r#type(&self) -> String {
        "test".into()
    }

    fn concurrency_model(&self) -> ConcurrencyModel {
        ConcurrencyModel::TaskPerCore(1)
    }

    fn build(&self) -> Result<Box<dyn AsyncExporter<Message> + Send + Sync>, Error> {
        Ok(Box::new(TestExporter {
            name: self.name.clone(),
            message_received: 0,
        }))
    }
}

impl ExporterFactory<Message> for TestExporterFactory {
    fn builder(
        &self,
        exporter_name: &str,
        exporter_type: &str,
        _config: Value,
    ) -> Option<Box<dyn ExporterBuilder<Message> + Send + Sync>> {
        match exporter_type {
            "test" => Some(Box::new(TestExporterBuilder {
                name: exporter_name.to_string(),
            })),
            _ => None,
        }
    }
}
