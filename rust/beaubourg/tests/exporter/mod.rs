use std::sync::Mutex;

use async_trait::async_trait;
use exporter::{AsyncExporter, ConcurrencyModel, EngineHandler, Error, ExporterBuilder, ExporterFactory};
use once_cell::sync::Lazy;
use serde_yaml::Value;
use signal::{Signal, SignalReceiver};
use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::common::Message;

pub static COUNTERS: Lazy<Mutex<Counters>> = Lazy::new(|| Mutex::new(Counters::default()));

/// A set of counters for the integration tests.
#[derive(Default, Debug)]
pub struct Counters {
    pub init_count: usize,
    pub export_count: usize,
    pub stop_count: usize,
}

pub struct TestExporter {
    name: String,
    message_received: usize,
    tcp_port: u16,
}

#[async_trait]
impl AsyncExporter<Message> for TestExporter {
    async fn init(&mut self, engine_handler: &mut EngineHandler) -> Result<(), Error> {
        COUNTERS.lock().expect("lock failed").init_count += 1;
        self.tcp_port = engine_handler.context().get_value::<u16>("exporter_tcp_port", 50000);
        Ok(())
    }

    async fn export(
        &mut self,
        mut signal_receiver: SignalReceiver<Message>,
        _effects_handler: beaubourg::exporter::effect::EffectHandler<Message>,
    ) -> Result<(), beaubourg::exporter::Error> {
        COUNTERS.lock().expect("lock failed").export_count += 1;
        loop {
            let signal = signal_receiver.recv().await;
            tracing::trace!("Exporter '{}' received signal: {:?}", self.name, signal);
            match signal {
                Signal::TimerTick { .. } => { /* do nothing */ }
                Signal::Messages { messages } => {
                    self.message_received += messages.len();
                    tracing::info!(exporter_name=%self.name, messages=?messages, total_message_received=%self.message_received, "Received messages");
                    for msg in messages {
                        let stream = TcpStream::connect(format!("127.0.0.1:{}", self.tcp_port)).await;
                        match stream {
                            Ok(mut stream) => {
                                if let Err(e) = stream.write(msg.payload.as_bytes()).await {
                                    panic!(
                                        "Error while sending message to the TCP test server: {} (port: {})",
                                        e, self.tcp_port
                                    );
                                }
                            }
                            Err(e) => {
                                tracing::error!(
                                    "error while connecting to the TCP test server: {} (port: {})",
                                    e,
                                    self.tcp_port
                                );
                                return Err(Error::Exporter {
                                    exporter: self.name.clone(),
                                    error: e.to_string(),
                                    context: Default::default(),
                                });
                            }
                        }
                    }
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

    async fn stop(&mut self) -> Result<(), Error> {
        COUNTERS.lock().expect("lock failed").stop_count += 1;
        Ok(())
    }
}

#[derive(Default)]
pub struct TestExporterFactory {}

struct TestExporterBuilder {
    name: String,
    concurrency_model: ConcurrencyModel,
}

impl TestExporterBuilder {
    pub fn new(name: String, config: Value) -> Self {
        let concurrency_model = match config.get("concurrency_model") {
            None => ConcurrencyModel::TaskPerCore(1),
            Some(value) => serde_yaml::from_value(value.clone()).expect("failed to deserialize concurrency model"),
        };

        TestExporterBuilder {
            name,
            concurrency_model,
        }
    }
}

impl ExporterBuilder<Message> for TestExporterBuilder {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn r#type(&self) -> String {
        "test".into()
    }

    fn concurrency_model(&self) -> ConcurrencyModel {
        self.concurrency_model.clone()
    }

    fn build(&self) -> Result<Box<dyn AsyncExporter<Message> + Send + Sync>, exporter::Error> {
        tracing::info!(exporter_name=%self.name, concurrency_model=?self.concurrency_model, "Building test exporter");
        Ok(Box::new(TestExporter {
            name: self.name.clone(),
            message_received: 0,
            tcp_port: 0,
        }))
    }
}

impl ExporterFactory<Message> for TestExporterFactory {
    fn builder(
        &self,
        exporter_name: &str,
        exporter_type: &str,
        config: Value,
    ) -> Option<Box<dyn ExporterBuilder<Message> + Send + Sync>> {
        match exporter_type {
            "test" => Some(Box::new(TestExporterBuilder::new(exporter_name.to_string(), config))),
            _ => None,
        }
    }
}
