#![deny(
    trivial_numeric_casts,
    missing_docs,
    unsafe_code,
    unstable_features,
    unstable_features,
    unused_import_braces,
    unused_qualifications,
    unused_extern_crates,
    unused_results
)]
#![warn(rust_2021_compatibility, unreachable_pub)]

//! Configuration objects for the Beaubourg pipeline.

use std::{collections::HashMap, fs::File, io::BufReader, path::Path, sync::Arc};

use exporter::{ExporterBuilder, ExporterFactory};
use processor::{AsyncProcessor, ProcessorFactory};
use receiver::{AsyncReceiver, ReceiverFactory};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use thiserror::Error;
use tracing::debug;
use validator::Validate;

/// Errors for the config module.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// The config file could not be read.
    #[error("invalid configuration file `{file}` - {message}")]
    InvalidConfig {
        /// The path to the config file.
        file: String,
        /// The error message.
        message: String,
    },

    /// At least 2 pipelines with the same name have been found
    #[error("duplicate pipeline '{pipeline}' found' (config file '{file}')")]
    DuplicatePipeline {
        /// The path to the config file.
        file: String,
        /// The pipeline name.
        pipeline: String,
    },

    /// An extension was not found.
    #[error("extension '{extension}' not found (config file '{file}')")]
    ExtensionNotFound {
        /// The path to the config file.
        file: String,
        /// The extension name.
        extension: String,
    },

    /// At least 2 receivers with the same name have been found
    #[error("duplicate receiver '{receiver}' found in pipeline '{pipeline}' (config file '{file}')")]
    DuplicateReceiver {
        /// The path to the config file.
        file: String,
        /// The pipeline name.
        pipeline: String,
        /// The receiver name.
        receiver: String,
    },

    /// A receiver was not found.
    #[error("receiver '{receiver}' not found in pipeline '{pipeline}' (config file '{file}')")]
    ReceiverNotFound {
        /// The path to the config file.
        file: String,
        /// The pipeline name.
        pipeline: String,
        /// The receiver name.
        receiver: String,
    },

    /// The creation of a receiver failed.
    #[error("receiver '{receiver}' not created in pipeline '{pipeline}' (config file '{file}', reason: {reason})")]
    ReceiverNotCreated {
        /// The path to the config file.
        file: String,
        /// The pipeline name.
        pipeline: String,
        /// The receiver name.
        receiver: String,
        /// The error message.
        reason: String,
    },

    /// A processor was not found.
    #[error("processor '{processor}' not found in pipeline '{pipeline}' (config file '{file}')")]
    ProcessorNotFound {
        /// The path to the config file.
        file: String,
        /// The pipeline name.
        pipeline: String,
        /// The processor name.
        processor: String,
    },

    /// The creation of a processor failed.
    #[error("processor '{processor}' not created in pipeline '{pipeline}' (config file '{file}', reason: {reason})")]
    ProcessorNotCreated {
        /// The path to the config file.
        file: String,
        /// The pipeline name.
        pipeline: String,
        /// The processor name.
        processor: String,
        /// The error message.
        reason: String,
    },

    /// At least 2 exporters with the same name have been found
    #[error("duplicate exporter '{exporter}' found in pipeline '{pipeline}' (config file '{file}')")]
    DuplicateExporter {
        /// The path to the config file.
        file: String,
        /// The pipeline name.
        pipeline: String,
        /// The exporter name.
        exporter: String,
    },

    /// An exporter was not found.
    #[error("exporter '{exporter}' not found in pipeline '{pipeline}' (config file '{file}')")]
    ExporterNotFound {
        /// The path to the config file.
        file: String,
        /// The pipeline name.
        pipeline: String,
        /// The exporter name.
        exporter: String,
    },

    /// The creation of an exporter failed.
    #[error("exporter '{exporter}' not created in pipeline '{pipeline}' (config file '{file}', reason: {reason})")]
    ExporterNotCreated {
        /// The path to the config file.
        file: String,
        /// The pipeline name.
        pipeline: String,
        /// The exporter name.
        exporter: String,
        /// The error message.
        reason: String,
    },
}

/// Collector configuration.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct DynConfig {
    receivers: HashMap<String, Value>,
    processors: HashMap<String, Value>,
    exporters: HashMap<String, Value>,
    extensions: Option<HashMap<String, Value>>,
    service: UnresolvedService,
}

/// Service configuration (with unresolved references).
#[derive(Serialize, Deserialize, Debug, Clone)]
struct UnresolvedService {
    /// List of the extensions configured for this service
    extensions: Option<Vec<String>>,

    /// Set of pipelines.
    pipelines: HashMap<String, PipelineConfig>,
}

/// Service configuration (with resolved references).
pub struct ResolvedService<Msg: 'static + Clone + Send> {
    /// List of the extensions configured for this service
    pub extensions: Option<Vec<String>>,

    /// Set of pipelines.
    pub pipelines: HashMap<String, Pipeline<Msg>>,
}

/// Pipeline configuration.
#[derive(Serialize, Deserialize, Debug, Clone, Validate)]
pub struct PipelineConfig {
    /// List of receivers configured for this pipeline.
    receivers: Vec<String>,

    /// List of processors configured for this pipeline.
    processors: Vec<String>,

    /// List of exporters configured for this pipeline.
    exporters: Vec<String>,

    // === Extended configuration (all optional) ===
    /// Receivers are connected to processors via a MPSC channel.
    /// This parameter configure the size of this channel.
    #[serde(default = "default_receiver_channel_size")]
    #[validate(range(min = 1, max = 1000))]
    receiver_channel_size: usize,

    /// Depending on the pipeline engine model, processors are inter-connected
    /// via MPSC channels. This parameter configure the size of these
    /// channels.
    #[serde(default = "default_processor_channel_size")]
    #[validate(range(min = 1, max = 1000))]
    processor_channel_size: usize,

    /// Processors are connected to exporters via MPSC channels.
    /// This parameter configure the size of these channels.
    #[serde(default = "default_exporter_channel_size")]
    #[validate(range(min = 1, max = 1000))]
    exporter_channel_size: usize,
}

fn default_receiver_channel_size() -> usize {
    100
}

fn default_processor_channel_size() -> usize {
    100
}

fn default_exporter_channel_size() -> usize {
    100
}

/// Pipeline configuration (with resolved references).
pub struct Config<Msg>
where
    Msg: 'static + Clone + Send,
{
    /// List of extensions configured for this pipeline.
    pub extensions: Option<HashMap<String, Value>>,
    /// The service configuration.
    pub service: ResolvedService<Msg>,
}

impl<Msg> Config<Msg>
where
    Msg: 'static + Clone + Send,
{
    /// Loads and checks the configuration of the OpenTelemetry collector.
    ///
    /// # Argument
    /// * `config_file_path` - Path to the configuration
    pub fn load_with_factories<P: AsRef<Path> + Clone + Send + 'static>(
        config_file_path: P,
        receiver_factory: Arc<impl ReceiverFactory<Msg>>,
        processor_factory: Arc<impl ProcessorFactory<Msg>>,
        exporter_factory: Arc<impl ExporterFactory<Msg>>,
    ) -> Result<Self, Error> {
        let config_file_name = config_file_path.as_ref().display().to_string();

        debug!("loading {}", config_file_name);

        let config_file = File::open(config_file_path).map_err(|err| Error::InvalidConfig {
            file: config_file_name.clone(),
            message: err.to_string(),
        })?;
        let reader = BufReader::new(config_file);
        let dyn_config: DynConfig = serde_yaml::from_reader(reader).map_err(|err| Error::InvalidConfig {
            file: config_file_name.clone(),
            message: err.to_string(),
        })?;
        let extensions = dyn_config.extensions.clone();
        let service_extensions = dyn_config.service.extensions.clone();

        // builds pipelines with their corresponding configuration
        let service = ResolvedService {
            extensions: service_extensions,
            pipelines: resolve_pipelines(
                &config_file_name,
                dyn_config,
                receiver_factory,
                processor_factory,
                exporter_factory,
            )?,
        };

        debug!("{} loaded", config_file_name);

        Ok(Self { extensions, service })
    }
}

/// The configuration of a pipeline.
pub struct Pipeline<Msg: 'static + Clone + Send> {
    /// A set of receivers.
    pub receivers: ReceiverSet<Msg>,
    /// A set of processors.
    pub processors: ProcessorSet<Msg>,
    /// A set of exporters.
    pub exporters: ExporterSet<Msg>,
}

/// A set of receivers.
pub struct ReceiverSet<Msg: 'static + Clone + Send> {
    /// A map of receivers.
    pub receivers: HashMap<String, Box<dyn AsyncReceiver<Msg> + Send + Sync>>,

    /// Configuration parameters
    pub channel_size: usize,
}

/// A named processor.
pub struct NamedProcessor<Msg: 'static + Clone + Send> {
    /// The name of the processor.
    pub name: String,
    /// The processor.
    pub processor: Box<dyn AsyncProcessor<Msg> + Send + Sync>,
}

/// A set of processors.
pub struct ProcessorSet<Msg: 'static + Clone + Send> {
    /// A map of processors.
    pub processors: Vec<NamedProcessor<Msg>>,

    /// configuration parameters
    pub channel_size: usize,
}

/// A set of exporters.
pub struct ExporterSet<Msg: 'static + Clone + Send> {
    /// A map of exporters.
    pub exporters: HashMap<String, Box<dyn ExporterBuilder<Msg> + Send + Sync>>,

    /// configuration parameters
    pub channel_size: usize,
}

/// An engine extension with its configuration.
pub enum EngineExtension {
    /// An engine implementing a work stealing scheduler (e.g. tokio multithread
    /// runtime).
    WorkStealing {
        /// The configuration of the engine.
        config: Value,
    },
    /// An engine implementing a single-thread-per-core scheduler (e.g. tokio
    /// current-thread runtime).
    ThreadPerCore {
        /// The configuration of the engine.
        config: Value,
    },
}

/// By default the ThreadPerCore engine is used.
impl Default for EngineExtension {
    fn default() -> Self {
        EngineExtension::ThreadPerCore { config: Value::Null }
    }
}

/// A list of all declared extensions
#[derive(Default)]
pub struct Extensions {
    /// The list of engine extensions stored in a map by name.
    pub engines: HashMap<String, EngineExtension>,
}

/// Creates the 'extensions' section of service configuration.
// fn create_dyn_extensions(
//     config_file_name: &str,
//     dyn_config: DynConfig,
// ) -> Result<Extensions, Error> {
//     let mut extensions = Extensions::default();
//
//     if let Some(extension_map) = dyn_config.extensions {
//         for (extension_name, extension_config) in extension_map {
//             let extension_type = extract_type(&extension_name);
//             match extension_type {
//                 "work-stealing-engine" => {
// check that the extension is not already defined
//                     if extensions.engines.insert(
//                         extension_name.clone(),
//                         EngineExtension::WorkStealing {
//                             config: extension_config,
//                         },
//                     ).is_some() {
//                         return Err(Error::InvalidConfig {
//                             file: config_file_name.to_string(),
//                             message: format!("extension '{}' already defined", extension_name),
//                         });
//                     }
//                 },
//                 "thread-per-core-engine" => {
//                     if extensions.engines.insert(
//                         extension_name.clone(),
//                         EngineExtension::ThreadPerCore {
//                             config: extension_config,
//                         },
//                     ).is_some() {
//                         return Err(Error::InvalidConfig {
//                             file: config_file_name.to_string(),
//                             message: format!("extension '{}' already defined", extension_name),
//                         });
//                     }
//                 },
//                 extension => {
//                     return Err(Error::ExtensionNotFound {
//                         file: config_file_name.to_string(),
//                         extension: extension.to_string()
//                     });
//                 }
//             }
//         }
//     }
//
//     Ok(extensions)
// }

/// Resolves the pipeline configuration and builds the pipelines.
fn resolve_pipelines<Msg: 'static + Clone + Send>(
    config_file_name: &str,
    dyn_config: DynConfig,
    receiver_factory: Arc<impl ReceiverFactory<Msg>>,
    processor_factory: Arc<impl ProcessorFactory<Msg>>,
    exporter_factory: Arc<impl ExporterFactory<Msg>>,
) -> Result<HashMap<String, Pipeline<Msg>>, Error> {
    let mut pipelines = HashMap::default();

    for (pipeline_name, pipeline_config) in dyn_config.service.pipelines {
        let mut receivers = HashMap::default();
        let mut processors = vec![];
        let mut exporters = HashMap::default();

        for receiver_name in pipeline_config.receivers {
            if let Some(value) = dyn_config.receivers.get(&receiver_name) {
                if receivers
                    .insert(
                        receiver_name.clone(),
                        receiver_factory
                            .create(&receiver_name, extract_type(&receiver_name), value.clone())
                            .map_err(|err| Error::ReceiverNotCreated {
                                file: config_file_name.into(),
                                pipeline: pipeline_name.clone(),
                                receiver: receiver_name.clone(),
                                reason: err.to_string(),
                            })?,
                    )
                    .is_some()
                {
                    return Err(Error::DuplicateReceiver {
                        file: config_file_name.into(),
                        pipeline: pipeline_name.clone(),
                        receiver: receiver_name.clone(),
                    });
                }
            } else {
                return Err(Error::ReceiverNotFound {
                    file: config_file_name.into(),
                    pipeline: pipeline_name,
                    receiver: receiver_name.clone(),
                });
            }
        }

        for processor_name in pipeline_config.processors {
            if let Some(value) = dyn_config.processors.get(&processor_name) {
                processors.push(NamedProcessor {
                    name: processor_name.clone(),
                    processor: processor_factory
                        .create(&processor_name, extract_type(&processor_name), value.clone())
                        .map_err(|err| Error::ProcessorNotCreated {
                            file: config_file_name.into(),
                            pipeline: pipeline_name.clone(),
                            processor: processor_name.clone(),
                            reason: err.to_string(),
                        })?,
                });
            } else {
                return Err(Error::ProcessorNotFound {
                    file: config_file_name.into(),
                    pipeline: pipeline_name,
                    processor: processor_name.clone(),
                });
            }
        }

        for exporter_name in pipeline_config.exporters {
            if let Some(value) = dyn_config.exporters.get(&exporter_name) {
                let exporter_builder = exporter_factory
                    .builder(&exporter_name, extract_type(&exporter_name), value.clone())
                    .ok_or_else(|| Error::ExporterNotCreated {
                        file: config_file_name.into(),
                        pipeline: pipeline_name.clone(),
                        exporter: exporter_name.clone(),
                        reason: "exporter not found".to_string(),
                    })?;
                if exporters.insert(exporter_name.clone(), exporter_builder).is_some() {
                    return Err(Error::DuplicateExporter {
                        file: config_file_name.into(),
                        pipeline: pipeline_name.clone(),
                        exporter: exporter_name.clone(),
                    });
                }
            } else {
                return Err(Error::ExporterNotFound {
                    file: config_file_name.into(),
                    pipeline: pipeline_name,
                    exporter: exporter_name.clone(),
                });
            }
        }

        if pipelines
            .insert(
                pipeline_name.clone(),
                Pipeline {
                    receivers: ReceiverSet {
                        receivers,
                        channel_size: pipeline_config.receiver_channel_size,
                    },
                    processors: ProcessorSet {
                        processors,
                        channel_size: pipeline_config.processor_channel_size,
                    },
                    exporters: ExporterSet {
                        exporters,
                        channel_size: pipeline_config.exporter_channel_size,
                    },
                },
            )
            .is_some()
        {
            return Err(Error::DuplicatePipeline {
                file: config_file_name.into(),
                pipeline: pipeline_name.clone(),
            });
        }
    }

    Ok(pipelines)
}

/// Extract the type from the name of a component.
fn extract_type(name: &str) -> &str {
    name.split('/').next().unwrap()
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use async_trait::async_trait;
    use exporter::{AsyncExporter, ConcurrencyModel, Error, ExporterBuilder, ExporterFactory};
    use processor::{noop::NoOp, AsyncProcessor, ProcessorFactory};
    use receiver::{effect::EffectHandler, AsyncReceiver, ReceiverFactory};
    use serde_yaml::Value;
    use signal::Signal;

    use crate::{extract_type, Config};

    #[derive(Default)]
    pub(crate) struct TestReceiverFactory {}

    impl ReceiverFactory<String> for TestReceiverFactory {
        fn create(
            &self,
            receiver_name: &str,
            receiver_type: &str,
            _config: Value,
        ) -> Result<Box<dyn AsyncReceiver<String> + Send + Sync>, receiver::Error> {
            match receiver_type {
                "test" => {
                    let receiver = Box::new(TestReceiver {
                        name: receiver_name.to_string(),
                    });
                    Ok(receiver as Box<dyn AsyncReceiver<String> + Send + Sync>)
                }
                _ => Err(receiver::Error::UnknownReceiver {
                    receiver: receiver_name.into(),
                    receiver_type: receiver_type.into(),
                }),
            }
        }
    }

    pub(crate) struct TestReceiver {
        name: String,
    }

    #[async_trait]
    impl AsyncReceiver<String> for TestReceiver {
        async fn receive(
            &mut self,
            _signal_receiver: receiver::signal::SignalReceiver,
            effects_handler: EffectHandler<String>,
        ) -> Result<(), receiver::Error> {
            for i in 0..10 {
                effects_handler
                    .send_messages(vec![format!("msg_{}", i)])
                    .await
                    .map_err(|e| receiver::Error::Receiver {
                        receiver: self.name.clone(),
                        error: e.to_string(),
                        context: Default::default(),
                    })?;
            }

            Ok(())
        }
    }

    #[derive(Default)]
    pub(crate) struct TestProcessorFactory {}

    impl ProcessorFactory<String> for TestProcessorFactory {
        fn create(
            &self,
            processor_name: &str,
            processor_type: &str,
            _config: Value,
        ) -> Result<Box<dyn AsyncProcessor<String> + Send + Sync>, processor::Error> {
            //let plugin: Result<Box<dyn Processor<String> + Send + Sync>,
            // processor::Error> = noop::Plugin {}.create(processor_name, config);
            match processor_type {
                "noop" => {
                    let processor = Box::new(NoOp::new(processor_name.to_string()));
                    Ok(processor as Box<dyn AsyncProcessor<String> + Send + Sync>)
                }
                _ => Err(processor::Error::UnknownProcessor {
                    processor: processor_name.into(),
                    processor_type: processor_type.into(),
                }),
            }
        }
    }

    #[derive(Default)]
    pub(crate) struct TestExporterFactory {}

    struct TestExporterBuilder {
        name: String,
    }

    impl ExporterBuilder<String> for TestExporterBuilder {
        fn name(&self) -> String {
            self.name.clone()
        }

        fn r#type(&self) -> String {
            "test".into()
        }

        fn concurrency_model(&self) -> ConcurrencyModel {
            ConcurrencyModel::TaskPerCore(1)
        }

        fn build(&self) -> Result<Box<dyn AsyncExporter<String> + Send + Sync>, Error> {
            Ok(Box::new(TestExporter {
                name: self.name.clone(),
            }) as Box<dyn AsyncExporter<String> + Send + Sync>)
        }
    }

    impl ExporterFactory<String> for TestExporterFactory {
        fn builder(
            &self,
            exporter_name: &str,
            exporter_type: &str,
            _config: Value,
        ) -> Option<Box<dyn ExporterBuilder<String> + Send + Sync>> {
            match exporter_type {
                "test" => Some(Box::new(TestExporterBuilder {
                    name: exporter_name.to_string(),
                })),
                _ => None,
            }
        }
    }

    pub(crate) struct TestExporter {
        name: String,
    }

    #[async_trait]
    impl AsyncExporter<String> for TestExporter {
        async fn export(
            &mut self,
            mut signal_receiver: signal::SignalReceiver<String>,
            _effects_handler: exporter::effect::EffectHandler<String>,
        ) -> Result<(), Error> {
            loop {
                match signal_receiver.recv().await {
                    Signal::TimerTick { .. } => println!("Exporter timer tick received"),
                    Signal::Messages { messages } => {
                        println!("Exporter '{}' messages received: {messages:?}", self.name)
                    }
                    Signal::Stop => break,
                    _ => break,
                }
            }
            Ok(())
        }
    }

    #[test]
    fn test_extract_type() {
        assert_eq!("test", extract_type("test"));
        assert_eq!("test", extract_type("test/1"));
        assert_eq!("test", extract_type("test/2"));
        assert_eq!("", extract_type(""));
    }

    #[test]
    fn load_config() -> Result<(), Box<dyn std::error::Error>> {
        let receiver_factory = Arc::new(TestReceiverFactory::default());
        let processor_factory = Arc::new(TestProcessorFactory::default());
        let exporter_factory = Arc::new(TestExporterFactory::default());

        let config = Config::load_with_factories(
            "data/config.yaml",
            receiver_factory,
            processor_factory,
            exporter_factory,
        )?;

        assert_eq!(1, config.service.pipelines.len(), "1 pipeline should be observed");

        let pipeline = config.service.pipelines.get("test");

        assert!(pipeline.is_some());

        let pipeline = pipeline.unwrap();
        assert_eq!(pipeline.receivers.receivers.len(), 3, "3 receivers should be observed");
        assert!(pipeline.receivers.receivers.get("test").is_some());
        assert!(pipeline.receivers.receivers.get("test/1").is_some());
        assert!(pipeline.receivers.receivers.get("test/2").is_some());

        assert_eq!(
            pipeline.processors.processors.len(),
            1,
            "1 processor should be observed"
        );
        assert_eq!(pipeline.processors.processors[0].name, "noop".to_string());

        assert_eq!(pipeline.exporters.exporters.len(), 3, "3 exporters should be observed");
        assert!(pipeline.exporters.exporters.get("test").is_some());
        assert!(pipeline.exporters.exporters.get("test/1").is_some());
        assert!(pipeline.exporters.exporters.get("test/2").is_some());

        Ok(())
    }
}
