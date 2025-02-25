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
#![warn(unreachable_pub)]

//! Definition of the `Engine` trait and the different implementations.

use std::{collections::HashMap, path::Path, sync::Arc};

use config::{Config, ExporterSet, ProcessorSet, ReceiverSet};
use context::Context;
use exporter::{ConcurrencyModel, DebugInfo, ExporterBuilder};
use receiver::ReceiversController;
use task::{
    labels::{ProcessLabels, TaskLabels},
    TaskManager,
};
use tokio::sync::mpsc::Sender;
use tracing::{debug, error};

use crate::{
    controllers::{EngineController, PipelineController},
    processor_chain::SeqProcessorChain,
    singleton::SingletonManager,
};

mod controllers;
pub mod multi_threaded;
mod processor_chain;
mod singleton;
pub mod thread_per_core;

/// All the errors that can occur when using the `Engine`.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// A runtime error occurred.
    #[error("engine runtime error (reason: {error})")]
    Runtime {
        /// The error that occurred.
        error: String,
    },

    /// A receiver error occurred.
    #[error("receiver error (reason: {error}, receiver: {receiver})")]
    Receiver {
        /// The receiver that failed.
        receiver: String,
        /// The error that occurred.
        error: String,
    },

    /// A processor error occurred.
    #[error("processor chain error (reason: {error})")]
    ProcessorChain {
        /// The error that occurred.
        error: String,
    },

    /// An exporter error occurred.
    #[error("exporter error (reason: {error}, exporter: {exporter})")]
    Exporter {
        /// The exporter that failed.
        exporter: String,

        /// The error that occurred.
        error: String,
    },

    /// A configuration error occurred.
    #[error("configuration error (reason: {0})")]
    Config(#[from] config::Error),

    /// A command error occurred.
    #[error("command error (reason: {error}, command: {command})")]
    Command {
        /// The command that failed.
        command: String,
        /// The error that occurred.
        error: String,
    },
}

/// The trait that all engines must implement.
pub trait Engine<Msg>
where
    Msg: 'static + Clone + Send,
{
    /// Starts the engine.
    fn run<P>(&mut self, process_labels: ProcessLabels, config_file_path: P) -> Result<(), Error>
    where
        P: AsRef<Path> + Clone + Send + 'static;
}

/// A set of methods that will be called by the engine when it is started and
/// stopped.
#[async_trait::async_trait]
pub trait AsyncObserver: Send {
    /// Called when the engine is started.
    async fn on_started(&self);
    /// Called when the engine is stopped.
    async fn on_stopped(&self);
}

/// A set of methods that will be called by the engine when it is started and
/// stopped.
#[async_trait::async_trait]
pub trait Observer: Send {
    /// Called when the engine is started.
    fn on_started(&self) {}
    /// Called when the engine is stopped.
    fn on_stopped(&self) {}
}

/// All the commands supported by the engine.
#[derive(Debug)]
#[non_exhaustive]
pub enum Command {
    /// Stops all the pipelines and all the Beaubourg components (i.e.
    /// receivers, processors, exporters).
    StopAll,
}

impl Command {
    /// Returns the name of the command.
    pub fn name(&self) -> &'static str {
        match self {
            Command::StopAll => "stop_all",
        }
    }
}

/// Command handler to interact with the engine.
pub struct CommandHandler {
    sender: Sender<Command>,
}

impl CommandHandler {
    /// Creates a new engine handler.
    pub fn new(sender: Sender<Command>) -> Self {
        Self { sender }
    }

    /// Sends a command to the engine.
    pub fn send(&self, command: Command) -> Result<(), Error> {
        let command_name = command.name().to_string();
        self.sender.try_send(command).map_err(|e| Error::Command {
            command: command_name,
            error: e.to_string(),
        })
    }
}

#[derive(Clone)]
pub(crate) struct PipelineContext {
    pub(crate) context: Arc<Context>,
    pub(crate) process_labels: ProcessLabels,
    pub(crate) reusable_tcp_listener: bool,
    pub(crate) cpu_multiplier: usize,
}

/// Creates all the pipelines declared in the configuration file.
pub(crate) async fn create_pipelines<Msg>(
    pipeline_context: PipelineContext,
    observer: Option<Box<(dyn AsyncObserver + Send + Sync)>>,
    config: Config<Msg>,
    engine_controller: EngineController,
    singleton_manager: SingletonManager<Msg>,
) where
    Msg: 'static + Clone + Send,
{
    debug!("starting pipeline one a single-thread-per-core engine");

    let task_manager = TaskManager::with_process_labels(pipeline_context.process_labels.clone());

    // For each pipeline declared in the configuration
    for (pipeline_name, pipeline) in config.service.pipelines {
        debug!(%pipeline_name, "starting pipeline '{}'", pipeline_name);

        // Creates the exporters and returns the senders to send messages to the
        // exporters.
        let exporters_channel = match start_exporters(
            pipeline_context.clone(),
            task_manager.clone(),
            &pipeline_name,
            pipeline.exporters,
            singleton_manager.clone(),
        )
        .await
        {
            Ok(channel) => channel,
            Err(err) => {
                error!(
                    process_id= %task_manager.process_labels().process_id,
                    %pipeline_name,
                    error= %err,
                    "exporters not started"
                );
                return;
            }
        };

        // Creates the processors, connects them with the exporters' senders and
        // returns the senders to send messages to the processors.
        let processors_channel = start_processors(
            pipeline_context.context.clone(),
            task_manager.clone(),
            &pipeline_name,
            pipeline.processors,
            exporters_channel,
            pipeline.receivers.channel_size,
        )
        .await;

        // Creates the receivers, connects them with the processors' senders.
        let receivers_controller = match processors_channel {
            Ok(processors_channel) => {
                match start_receivers(
                    pipeline_context.context.clone(),
                    task_manager.clone(),
                    &pipeline_name,
                    pipeline.receivers,
                    processors_channel,
                    pipeline_context.reusable_tcp_listener,
                )
                .await
                {
                    Ok(receiver_channels) => {
                        debug!(%pipeline_name, "pipeline '{}' started", pipeline_name);
                        receiver_channels
                    }
                    Err(err) => {
                        error!(
                            process_id= %task_manager.process_labels().process_id,
                            %pipeline_name,
                            error= %err,
                            "receivers not started"
                        );
                        return;
                    }
                }
            }
            Err(err) => {
                error!(
                    process_id= %task_manager.process_labels().process_id,
                    %pipeline_name,
                    error= %err,
                    "processors not started"
                );
                return;
            }
        };

        if let Err(err) =
            engine_controller.add_pipeline(pipeline_name.clone(), PipelineController::new(receivers_controller))
        {
            tracing::error!(%pipeline_name, error= ?err, "Pipeline not added");
        }
    }

    debug!("pipeline engine started");
    if let Some(observer) = observer {
        observer.on_started().await;
    }

    // Drop
    drop(singleton_manager);

    task_manager.join().await;
}

pub(crate) async fn start_receivers<Msg>(
    context: Arc<Context>,
    mut task_manager: TaskManager,
    pipeline_name: &str,
    receivers: ReceiverSet<Msg>,
    processors_channel: flume::Sender<Vec<Msg>>,
    reusable_tcp_listener: bool,
) -> Result<ReceiversController, Error>
where
    Msg: 'static + Clone + Send,
{
    let mut receivers_mgr = ReceiversController::default();

    for (receiver_name, mut receiver) in receivers.receivers {
        let sender = processors_channel.clone();
        let process_labels = task_manager.process_labels();
        let task_labels = TaskLabels {
            task_cat: "receiver".into(),
            task_id: format!("receiver/{}/{}", pipeline_name, receiver_name),
            task_source: "NA".into(),
        };
        let task_labels_clone = task_labels.clone();
        let pipeline_name = pipeline_name.to_string();
        let task_manager_clone = task_manager.clone();
        let context_clone = context.clone();

        // Create a mpsc channel to send signals to the receiver.
        let (signal_sender, signal_receiver) = tokio::sync::mpsc::channel(1);
        receivers_mgr.add_receiver(receiver_name.clone(), signal_sender);

        let join_handler = tokio::task::spawn(async move {
            let engine_handler = receiver::EngineHandler::new(task_manager_clone.clone(), context_clone);

            if let Err(error) = receiver.init(engine_handler).await {
                error!(
                    %error,
                    process_id = %process_labels.process_id,
                    task_cat= %task_labels_clone.task_cat,
                    task_id= %task_labels_clone.task_id,
                    task_source= %task_labels_clone.task_source,
                    %pipeline_name,
                    %receiver_name,
                    "receiver initialization failed"
                );
                return TaskManager::no_task_cleaner(process_labels, task_labels_clone);
            }

            let run_result = {
                let signal_receiver = receiver::signal::SignalReceiver::with_receiver(signal_receiver);
                let effect_handler =
                    receiver::effect::EffectHandler::with_channel(receiver_name.clone(), sender, reusable_tcp_listener);
                receiver.receive(signal_receiver, effect_handler).await
            };

            if let Err(err) = run_result {
                error!(
                    task_id= %task_labels_clone.task_id,
                    task_cat= %task_labels_clone.task_cat,
                    task_source= %task_labels_clone.task_source,
                    process_id = %process_labels.process_id,
                    error= %err,
                    "start receiver failed"
                );
            } else if let Err(err) = receiver.stop().await {
                error!(
                    task_id= %task_labels_clone.task_id,
                    task_cat= %task_labels_clone.task_cat,
                    task_source= %task_labels_clone.task_source,
                    process_id = %process_labels.process_id,
                    error= %err,
                    "stop receiver failed"
                );
            }

            tracing::info!(
                task_id= %task_labels_clone.task_id,
                task_cat= %task_labels_clone.task_cat,
                task_source= %task_labels_clone.task_source,
                process_id = %process_labels.process_id,
                "Receiver has stopped normally"
            );
            TaskManager::no_task_cleaner(process_labels, task_labels_clone)
        });

        task_manager.register(join_handler, &task_labels);
    }
    Ok(receivers_mgr)
}

pub(crate) async fn start_processors<Msg>(
    context: Arc<Context>,
    task_manager: TaskManager,
    pipeline_name: &str,
    processors: ProcessorSet<Msg>,
    exporters_channel: HashMap<String, flume::Sender<Vec<Msg>>>,
    receivers_channel_size: usize,
) -> Result<flume::Sender<Vec<Msg>>, Error>
where
    Msg: 'static + Clone + Send,
{
    let (sender, receiver) = flume::bounded(receivers_channel_size);

    SeqProcessorChain::start::<Msg>(
        context,
        task_manager,
        pipeline_name,
        processors,
        receiver,
        exporters_channel,
    )
    .await
    .map_err(|err| Error::ProcessorChain { error: err.to_string() })?;

    Ok(sender)
}

/// Creates all the exporters defined in the ExporterSet.
/// In the current implementation a mpmc channel is created between the
/// processor layer and the exporters. Each exporter runs in an independent
/// tokio task and processes messages coming from the mpmc channel.
///
/// A message issued by a chain of processors will be sent to all the exporter
/// types by default, unless a processor specifies a specific exporter name.
///
/// Returns a HashMap of exporter names to their respective flume channels.
pub(crate) async fn start_exporters<Msg>(
    pipeline_context: PipelineContext,
    task_manager: TaskManager,
    pipeline_name: &str,
    exporters: ExporterSet<Msg>,
    singleton_manager: SingletonManager<Msg>,
) -> Result<HashMap<String, flume::Sender<Vec<Msg>>>, Error>
where
    Msg: 'static + Clone + Send,
{
    let mut senders = HashMap::new();
    let process_labels = task_manager.process_labels();

    // Creates all the exporters defined in the ExporterSet.
    for (exporter_idx, (exporter_name, exporter_builder)) in exporters.exporters.into_iter().enumerate() {
        // For a specific exporter name:
        // - creates a MPMC channel between the processor layer and the exporter
        // - creates N tokio task that runs the exporter (N is based on the concurrency
        //   model exposed by the exporter)
        // - registers the exporter tasks in the task manager
        // - registers the exporter channel in the senders map
        let process_labels = process_labels.clone();
        let task_labels = TaskLabels {
            task_cat: "exporter".into(),
            task_id: format!("exporter/{}/{}", pipeline_name, exporter_name),
            task_source: "processors".into(),
        };

        // Creates N tokio tasks that runs the exporter
        let sender = match exporter_builder.concurrency_model() {
            ConcurrencyModel::TaskPerCore(count) => {
                let (sender, receiver) = flume::bounded(exporters.channel_size);
                let concurrent_tasks = count * pipeline_context.cpu_multiplier as u16;
                create_exporters(
                    ExporterContext {
                        concurrent_tasks,
                        pipeline_name: pipeline_name.to_string(),
                        exporter_name: exporter_name.clone(),
                        exporter_idx,
                        process_labels,
                        task_labels: task_labels.clone(),
                        context: pipeline_context.context.clone(),
                    },
                    exporter_builder,
                    receiver,
                    task_manager.clone(),
                )
                .await?;
                sender
            }
            ConcurrencyModel::Singleton => {
                let (sender, receiver) =
                    singleton_manager.get_or_create_exporter_channel(&exporter_name, exporters.channel_size);
                match receiver {
                    None => sender,
                    Some(receiver) => {
                        create_exporters(
                            ExporterContext {
                                concurrent_tasks: 1,
                                pipeline_name: pipeline_name.to_string(),
                                exporter_name: exporter_name.clone(),
                                exporter_idx,
                                process_labels,
                                task_labels: task_labels.clone(),
                                context: pipeline_context.context.clone(),
                            },
                            exporter_builder,
                            receiver,
                            task_manager.clone(),
                        )
                        .await?;
                        sender
                    }
                }
            }
        };

        if senders.insert(exporter_name.clone(), sender).is_some() {
            tracing::warn!(
                process_id = %task_manager.process_labels().process_id,
                task_cat= %task_labels.task_cat,
                task_id= %task_labels.task_id,
                task_source= %task_labels.task_source,
                %pipeline_name,
                exporter_name=%exporter_name,
                "registration of a duplicated exporter"
            );
        }
    }

    Ok(senders)
}

pub(crate) struct ExporterContext {
    pub(crate) concurrent_tasks: u16,
    pub(crate) pipeline_name: String,
    pub(crate) exporter_name: String,
    pub(crate) exporter_idx: usize,
    pub(crate) process_labels: ProcessLabels,
    pub(crate) task_labels: TaskLabels,
    pub(crate) context: Arc<Context>,
}

pub(crate) async fn create_exporters<Msg: 'static + Send + Clone>(
    exporter_context: ExporterContext,
    exporter_builder: Box<dyn ExporterBuilder<Msg> + Send + Sync>,
    receiver: flume::Receiver<Vec<Msg>>,
    mut task_manager: TaskManager,
) -> Result<(), Error> {
    for _ in 0..exporter_context.concurrent_tasks {
        let context = exporter_context.context.clone();
        let receiver = receiver.clone();
        let debug_info = DebugInfo::new(
            exporter_context.pipeline_name.to_string(),
            exporter_context.exporter_name.to_string(),
            exporter_context.process_labels.clone(),
            exporter_context.task_labels.clone(),
        );
        let process_labels_clone = exporter_context.process_labels.clone();
        let task_labels_clone = exporter_context.task_labels.clone();

        let mut exporter = exporter_builder.build().map_err(|err| Error::Exporter {
            exporter: exporter_context.exporter_name.clone(),
            error: err.to_string(),
        })?;

        let join_handle = tokio::task::spawn(async move {
            let mut engine_handler = exporter::EngineHandler::new(context);

            if let Err(error) = exporter.init(&mut engine_handler).await {
                error!(%error,?debug_info,"exporter initialization failed");
            } else {
                let signal_receiver = if let Some(timer) = engine_handler.get_timer() {
                    signal::SignalReceiver::with_receiver_timer(receiver, *timer, exporter_context.exporter_idx)
                } else {
                    signal::SignalReceiver::with_receiver(receiver)
                };
                let effect_handler = exporter::effect::EffectHandler::new(debug_info.clone());
                if let Err(error) = exporter.export(signal_receiver, effect_handler).await {
                    tracing::error!(%error,?debug_info,"Run exporter failed");
                } else if let Err(error) = exporter.stop().await {
                    tracing::error!(%error,debug_info=?debug_info,"Exporter didn't stop properly");
                }
            }

            tracing::info!(?debug_info, "Exporter has stopped normally");
            TaskManager::no_task_cleaner(process_labels_clone, task_labels_clone)
        });

        // Registers the exporter task in the task manager
        task_manager.register(join_handle, &exporter_context.task_labels);
    }
    Ok(())
}
