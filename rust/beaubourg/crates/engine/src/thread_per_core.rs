//! A single-thread-per-core engine.
//!
//! A native OS thread is created per core [to do and pinned to the
//! corresponding core]. A single threaded Tokio runtime is created per native
//! OS thread created.

use std::{marker::PhantomData, path::Path, sync::Arc};

use config::Config;
use context::Context;
use exporter::ExporterFactory;
use processor::ProcessorFactory;
use receiver::ReceiverFactory;
use task::labels::ProcessLabels;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::{
    controllers::EngineController, create_pipelines, singleton::SingletonManager, Command, CommandHandler, Error,
    Observer, PipelineContext,
};

/// A single-thread-per-core scheduler.
pub struct Engine<Msg, R, P, E> {
    receiver_factory: Arc<R>,
    processor_factory: Arc<P>,
    exporter_factory: Arc<E>,
    context: Arc<Context>,
    observer: Option<Box<dyn Observer + Send + Sync>>,
    command_sender: Sender<Command>,
    command_receiver: Receiver<Command>,
    controller: EngineController,
    phantom_msg: PhantomData<Msg>,
}

impl<Msg, Rec, Proc, Exp> Engine<Msg, Rec, Proc, Exp>
where
    Rec: ReceiverFactory<Msg> + 'static + Send + Sync,
    Proc: ProcessorFactory<Msg> + 'static + Send + Sync,
    Exp: ExporterFactory<Msg> + 'static + Send + Sync,
    Msg: 'static + Clone + Send,
{
    /// Creates a new engine.
    pub fn new(receiver_factory: Rec, processor_factory: Proc, exporter_factory: Exp) -> Self {
        let (command_sender, command_receiver) = tokio::sync::mpsc::channel(10);

        Self {
            receiver_factory: Arc::new(receiver_factory),
            processor_factory: Arc::new(processor_factory),
            exporter_factory: Arc::new(exporter_factory),
            context: Arc::new(Context::new()),
            observer: None,
            command_sender,
            command_receiver,
            controller: EngineController::default(),
            phantom_msg: Default::default(),
        }
    }

    /// Creates a new engine with a context.
    pub fn with_context(
        receiver_factory: Rec,
        processor_factory: Proc,
        exporter_factory: Exp,
        context: Context,
    ) -> Self {
        let (command_sender, command_receiver) = tokio::sync::mpsc::channel(10);

        Self {
            receiver_factory: Arc::new(receiver_factory),
            processor_factory: Arc::new(processor_factory),
            exporter_factory: Arc::new(exporter_factory),
            context: Arc::new(context),
            observer: None,
            command_sender,
            command_receiver,
            controller: EngineController::default(),
            phantom_msg: Default::default(),
        }
    }

    /// Sets an observer for this engine.
    pub fn observer(&mut self, observer: impl Observer + Send + Sync + 'static) {
        self.observer = Some(Box::new(observer));
    }

    /// Returns a command handler to interact with the engine from a separate
    /// thread once it is started.
    pub fn command_handler(&self) -> CommandHandler {
        CommandHandler::new(self.command_sender.clone())
    }

    fn process_command(&mut self) {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async move {
                tracing::info!("Ready to process commands");
                while let Some(command) = self.command_receiver.recv().await {
                    match command {
                        Command::StopAll => {
                            tracing::info!("Received 'StopAll' command");
                            if let Err(err) = self.controller.stop_all() {
                                tracing::error!("Failed to stop all pipelines: {:?}", err);
                            } else {
                                tracing::info!("'StopAll' command sent to the engine controller");
                                break;
                            }
                        }
                    }
                }
                tracing::info!("Stopped processing the commands sent to the engine");
            });
    }
}

impl<Msg, Rec, Proc, Exp> crate::Engine<Msg> for Engine<Msg, Rec, Proc, Exp>
where
    Rec: ReceiverFactory<Msg> + 'static + Send + Sync,
    Proc: ProcessorFactory<Msg> + 'static + Send + Sync,
    Exp: ExporterFactory<Msg> + 'static + Send + Sync,
    Msg: 'static + Clone + Send,
{
    /// Starts the engine.
    fn run<P>(&mut self, process_labels: ProcessLabels, config_file_path: P) -> Result<(), Error>
    where
        P: AsRef<Path> + Clone + Send + 'static,
    {
        let mut handlers = Vec::new();

        // Create a Tokio runtime for each core.
        tracing::info!(
            "Creating {} single-threaded Tokio runtime(s) (one per core)",
            num_cpus::get()
        );
        let singleton_manager = SingletonManager::default();

        for _ in 0..num_cpus::get() {
            let process_labels = process_labels.clone();
            let config_file_path = config_file_path.clone();
            let receiver_factory = self.receiver_factory.clone();
            let processor_factory = self.processor_factory.clone();
            let exporter_factory = self.exporter_factory.clone();
            let context = self.context.clone();
            let engine_controller = self.controller.clone();
            let singleton_manager = singleton_manager.clone();

            // Create a new native OS thread per Tokio runtime.
            let h = std::thread::spawn(move || {
                let config = Config::load_with_factories(
                    config_file_path,
                    receiver_factory,
                    processor_factory,
                    exporter_factory,
                );

                match config {
                    Ok(config) => {
                        // Create a single-threaded Tokio runtime for each core.
                        // And start a new pipeline instance for each core.
                        tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                            .unwrap()
                            .block_on(create_pipelines(
                                PipelineContext {
                                    context,
                                    process_labels,
                                    reusable_tcp_listener: true,
                                    cpu_multiplier: 1,
                                },
                                None,
                                config,
                                engine_controller,
                                singleton_manager,
                            ));
                    }
                    Err(e) => panic!("{}", e.to_string()),
                }
            });
            handlers.push(h);
        }

        // Drop to release all the senders registered in the singleton manager.
        drop(singleton_manager);

        let observer = self.observer.take();
        // Call the on_started callback.
        if let Some(observer) = &observer {
            observer.on_started();
        }

        // Wait for commands from the command handler.
        self.process_command();

        // Wait for all threads to finish.
        tracing::info!(num_thread=%handlers.len(), "Waiting for all threads to finish");
        for h in handlers {
            h.join().unwrap();
        }
        tracing::info!("All threads finished");

        // Call the on_stopped callback.
        if let Some(observer) = &observer {
            observer.on_stopped();
        }

        tracing::info!("Engine stopped");
        Ok(())
    }
}
