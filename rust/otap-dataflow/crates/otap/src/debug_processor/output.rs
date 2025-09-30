use crate::pdata::{OtapPdata, OtlpProtoBytes};
use otap_df_config::{PortName, node::HyperEdgeConfig};
use otap_df_engine::error::Error;
use otap_df_engine::local::processor as local;
use otap_df_engine::node::NodeId;
use otel_arrow_rust::proto::opentelemetry::{
    common::v1::{AnyValue, InstrumentationScope},
    logs::v1::{LogRecord, LogsData, ResourceLogs, ScopeLogs, SeverityNumber},
    resource::v1::Resource,
};
use prost::Message as _;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use tokio::fs::File;
use tokio::io::{AsyncWrite, AsyncWriteExt};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum OutputMode {
    Console,
    Outport(OutPort),
    File(String),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OutPort {
    /// Outgoing hyper-edges, keyed by port name.
    /// Each port connects this node to one or more downstream nodes, with a specific dispatch strategy.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub out_ports: HashMap<PortName, HyperEdgeConfig>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_out_port: Option<PortName>,
}

/// struct that handles the logic for sending data to console or out_ports
pub struct DebugOutput {
    writer: Option<OutputWriter>,
    ports: Option<Vec<PortName>>,
    effect_handler: local::EffectHandler<OtapPdata>,
}

/// A wrapper around AsyncWrite that simplifies error handling for debug output
pub struct OutputWriter {
    writer: Box<dyn AsyncWrite + Unpin>,
    processor_id: NodeId,
}

impl OutputWriter {
    pub async fn new(output_file: Option<String>, processor_id: NodeId) -> Result<Self, Error> {
        let writer: Box<dyn AsyncWrite + Unpin> = match output_file {
            Some(file_name) => {
                let file = File::options()
                    .write(true)
                    .append(true)
                    .create(true)
                    .open(file_name)
                    .await
                    .expect("could not open output file");
                Box::new(file)
            }
            None => Box::new(tokio::io::stdout()),
        };
        Ok(Self {
            writer,
            processor_id,
        })
    }

    pub async fn write(&mut self, data: &str) -> Result<(), Error> {
        self.writer
            .write_all(data.as_bytes())
            .await
            .map_err(|e| Error::ProcessorError {
                processor: self.processor_id.clone(),
                error: format!("Write error: {e}"),
            })
    }
}

impl DebugOutput {
    pub async fn new_from_output_mode(
        output_mode: OutputMode,
        effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<Self, Error> {
        match output_mode {
            OutputMode::Console => {
                let output_writer = OutputWriter::new(None, effect_handler.processor_id()).await?;
                Ok(Self {
                    writer: Some(output_writer),
                    ports: None,
                    effect_handler,
                })
            }
            OutputMode::Outport(out_port) => {
                // if default is set then we use it, if not then we use the out_port field
                let ports: Vec<PortName> = if let Some(out_port) = out_port.default_out_port {
                    vec![out_port.clone()]
                } else {
                    out_port.out_ports.keys().cloned().collect()
                };
                // check that the ports are actually connected
                let connected_ports = effect_handler.connected_ports();
                let valid_ports = ports.iter().all(|port| connected_ports.contains(port));
                // raise error if ports are not connected
                if !valid_ports {
                    return Err(Error::ProcessorError {
                        processor: effect_handler.processor_id(),
                        error: "ports are not connected".to_owned(),
                    });
                }

                Ok(Self {
                    writer: None,
                    ports: Some(ports),
                    effect_handler,
                })
            }
            OutputMode::File(file_name) => {
                let output_writer =
                    OutputWriter::new(Some(file_name), effect_handler.processor_id()).await?;
                Ok(Self {
                    writer: Some(output_writer),
                    ports: None,
                    effect_handler,
                })
            }
        }
    }

    /// send a message
    pub async fn output_message(&mut self, message: &str) -> Result<(), Error> {
        if let Some(ports) = &self.ports {
            // store message as a log -> create OtapPdata msg to send
            let log_message = self.generate_log_message(message.to_string());
            let mut bytes = vec![];
            log_message
                .encode(&mut bytes)
                .expect("failed to encode log data into bytes");
            let otlp_logs_bytes =
                OtapPdata::new_todo_context(OtlpProtoBytes::ExportLogsRequest(bytes).into());
            // send msg to connected ports
            for port in ports.iter().cloned() {
                self.effect_handler
                    .send_message_to(port, otlp_logs_bytes.clone())
                    .await?;
            }
        }
        if let Some(ref mut writer) = self.writer {
            writer.write(message).await?;
        }

        Ok(())
    }

    /// create a log pdata to wrap the message in (message will be used in the body field)
    fn generate_log_message(&self, body: String) -> LogsData {
        let time_unix = 0u64;
        LogsData::new(vec![
            ResourceLogs::build(Resource::default())
                .scope_logs(vec![
                    ScopeLogs::build(InstrumentationScope::default())
                        .log_records(vec![
                            LogRecord::build(
                                time_unix,
                                SeverityNumber::Debug,
                                "debug_processor_log",
                            )
                            .observed_time_unix_nano(time_unix)
                            .body(AnyValue::new_string(body))
                            .finish(),
                        ])
                        .finish(),
                ])
                .finish(),
        ])
    }
}
