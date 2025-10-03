// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Handles the output flow of the debug processor

use crate::pdata::{OtapPdata, OtlpProtoBytes};
use async_trait::async_trait;
use otap_df_config::PortName;
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
use tokio::fs::File;
use tokio::io::{AsyncWrite, AsyncWriteExt};

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum OutputMode {
    Console,
    Outports(Vec<PortName>),
    File(String),
}

#[async_trait(?Send)]
pub trait DebugOutput {
    async fn output_message(&mut self, message: &str) -> Result<(), Error>;
}

/// A wrapper around AsyncWrite that simplifies error handling for debug output
pub struct DebugOutputWriter {
    writer: Box<dyn AsyncWrite + Unpin>,
    processor_id: NodeId,
}

impl DebugOutputWriter {
    pub async fn new(output_file: Option<String>, processor_id: NodeId) -> Result<Self, Error> {
        let writer: Box<dyn AsyncWrite + Unpin> = match output_file {
            Some(file_name) => {
                let file = File::options()
                    .write(true)
                    .append(true)
                    .create(true)
                    .open(file_name)
                    .await
                    .map_err(|e| Error::ProcessorError {
                        processor: processor_id.clone(),
                        error: format!("File error: {e}"),
                    })?;
                Box::new(file)
            }
            None => Box::new(tokio::io::stdout()),
        };
        Ok(Self {
            writer,
            processor_id,
        })
    }
}

#[async_trait(?Send)]
impl DebugOutput for DebugOutputWriter {
    async fn output_message(&mut self, data: &str) -> Result<(), Error> {
        self.writer
            .write_all(data.as_bytes())
            .await
            .map_err(|e| Error::ProcessorError {
                processor: self.processor_id.clone(),
                error: format!("Write error: {e}"),
            })
    }
}

pub struct DebugOutputPorts {
    ports: Vec<PortName>,
    effect_handler: local::EffectHandler<OtapPdata>,
}

impl DebugOutputPorts {
    pub fn new(
        ports: Vec<PortName>,
        effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<Self, Error> {
        // check that the ports are actually connected
        let connected_ports = effect_handler.connected_ports();
        // collect ports that are not connected
        let mut invalid_ports = ports
            .iter()
            .filter(|port| !connected_ports.contains(port))
            .peekable();
        // raise error if ports are not connected
        if invalid_ports.peek().is_some() {
            let invalid_ports_list = invalid_ports
                .map(|port_name| port_name.as_ref())
                .collect::<Vec<&str>>()
                .join(", ");
            let connected_ports_list = connected_ports
                .iter()
                .map(|port_name| port_name.as_ref())
                .collect::<Vec<&str>>()
                .join(", ");
            return Err(Error::ProcessorError {
                processor: effect_handler.processor_id(),
                error: format!(
                    "The following ports are not connected [{invalid_ports_list}], these are connected ports to this node [{connected_ports_list}]"
                ),
            });
        }

        Ok(Self {
            ports,
            effect_handler,
        })
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

#[async_trait(?Send)]
impl DebugOutput for DebugOutputPorts {
    async fn output_message(&mut self, message: &str) -> Result<(), Error> {
        // store message as a log -> create OtapPdata msg to send
        let log_message = self.generate_log_message(message.to_string());
        let mut bytes = vec![];
        log_message
            .encode(&mut bytes)
            .expect("failed to encode log data into bytes");
        let otlp_logs_bytes =
            OtapPdata::new_todo_context(OtlpProtoBytes::ExportLogsRequest(bytes).into());
        // send msg to connected ports
        for port in self.ports.iter().cloned() {
            self.effect_handler
                .send_message_to(port, otlp_logs_bytes.clone())
                .await?;
        }

        Ok(())
    }
}
