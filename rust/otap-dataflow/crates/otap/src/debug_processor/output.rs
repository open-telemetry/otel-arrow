use otap_df_config::{PortName, node::HyperEdgeConfig},


pub enum OutputMode {
    Console,
    Outport(OutPort),
    File(String)
}

pub struct OutPort {
    /// Outgoing hyper-edges, keyed by port name.
    /// Each port connects this node to one or more downstream nodes, with a specific dispatch strategy.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub out_ports: HashMap<PortName, HyperEdgeConfig>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_out_port: Option<PortName>,

}

/// struct that handles the logic for sending data to console or out_ports
pub struct DebugOutput{ 
    writer: Option<OutputWriter>,
    ports: Vec<PortName>
}

/// A wrapper around AsyncWrite that simplifies error handling for debug output
struct OutputWriter {
    writer: Box<dyn AsyncWrite + Unpin>,
    processor_id: NodeId,
}

impl OutputWriter {
    async fn new(output_file: Option<String> processor_id: NodeId) -> Result<Self, Error>{
        let writer = match output_file {
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
            None => Box::new(tokio::io::stdout())
        }
        Self {
            writer,
            processor_id,
        }
    }

    async fn write(&mut self, data: &str) -> Result<(), Error> {
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
    pub fn new(writer: Option<OutputWriter>, ports: Vec<PortName>) -> Self {
        Self {
            writer, ports
        }
    }

    async pub fn new_from_output_mode(output_mode: OutputMode ) -> ResultSelf{
        match output_mode {
            OutputMode::Console => {
                let output_writer = OutputWriter::new(None).await?; 
                Self {
                    writer: Some(output_writer).
                    ports: vec![]
                }
            }
            OutputMode::Outport(out_ports) => {
                let ports = 
                Self {
                    writer: None,
                    ports
                }
            }
            OutputMode::File(file_name) => {
                let output_writer = OutputWriter::new(file_name).await?; 
                Self {
                    writer: Some(output_writer),
                    ports: vec![]
                }
            }
        } 
    }

    /// send a message
    pub async fn output_message(message: String, local_effect_handler: Option<EffectHandler) -> Result<> {
        if let Some(effect_handler) = local_effect_handler {
            for port in self.ports {
                let log_message = self.generate_log_message();
                effect_handler.send_message_to(port, log_message).await?;
            }
        }
        if let Some(writer) = self.writer {
            writer.write(message).await?;
        }
    }


    /// create a log pdata to wrap the message in (message will be used in the body field)
    fn generate_log_message(body: String) -> LogsData {
        let time_unix = 0u64;
        LogsData::new(vec![
                    ResourceLogs::build(Resource::default())
                        .scope_logs(vec![
                            ScopeLogs::build(
                                InstrumentationScope::default()
                            )
                            .log_records(vec![
                                LogRecord::build(time_unix, SeverityNumber::Debug, "debug_processor_log")
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