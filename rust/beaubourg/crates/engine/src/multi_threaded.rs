//! An engine based on a multithreading tokio runtime.

use std::{marker::PhantomData, path::Path, sync::Arc};

use config::Config;
use context::Context;
use exporter::ExporterFactory;
use processor::ProcessorFactory;
use receiver::ReceiverFactory;
use task::labels::ProcessLabels;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::{
    create_pipelines, AsyncObserver, Command, CommandHandler, EngineController, Error, PipelineContext,
    SingletonManager,
};

/// A multithreading engine.
pub struct Engine<Msg, R, P, E> {
    receiver_factory: Arc<R>,
    processor_factory: Arc<P>,
    exporter_factory: Arc<E>,
    context: Arc<Context>,
    observer: Option<Box<dyn AsyncObserver + Send + Sync>>,
    command_sender: Sender<Command>,
    command_receiver: Option<Receiver<Command>>,
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
            command_receiver: Some(command_receiver),
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
            command_receiver: Some(command_receiver),
            controller: EngineController::default(),
            phantom_msg: Default::default(),
        }
    }

    /// Sets an observer for this engine.
    pub fn observer(&mut self, observer: impl AsyncObserver + Send + Sync + 'static) {
        self.observer = Some(Box::new(observer));
    }

    /// Returns a command handler to interact with the engine from a separate
    /// thread once it is started.
    pub fn command_handler(&self) -> CommandHandler {
        CommandHandler::new(self.command_sender.clone())
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
        let config = Config::load_with_factories(
            config_file_path,
            self.receiver_factory.clone(),
            self.processor_factory.clone(),
            self.exporter_factory.clone(),
        )?;

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(|e| Error::Runtime { error: e.to_string() })?;

        // Runs in parallel the command loop.
        let command_receiver = self.command_receiver.take();
        let controller = self.controller.clone();
        let join_handle = rt.spawn(process_command(command_receiver, controller));

        // Creates and runs the pipelines.
        rt.block_on(create_pipelines(
            PipelineContext {
                context: self.context.clone(),
                process_labels,
                reusable_tcp_listener: false,
                cpu_multiplier: num_cpus::get(),
            },
            self.observer.take(),
            config,
            self.controller.clone(),
            SingletonManager::default(),
        ));
        tracing::info!("pipelines stopped");

        // Waits for the command loop to finish.
        let _ = rt.block_on(join_handle);
        tracing::info!("command loop stopped");

        // Call the on_stopped callback.
        if let Some(observer) = self.observer.take() {
            rt.block_on(observer.on_stopped());
            tracing::info!("on_stopper observer stopped");
        }
        tracing::info!("engine run stopped");

        Ok(())
    }
}

async fn process_command(command_receiver: Option<Receiver<Command>>, controller: EngineController) {
    if let Some(mut command_receiver) = command_receiver {
        tracing::info!("Ready to process the commands sent to the engine ");
        while let Some(command) = command_receiver.recv().await {
            match command {
                Command::StopAll => {
                    tracing::info!("Received StopAll command");
                    if let Err(err) = controller.stop_all() {
                        tracing::error!("Failed to stop all pipelines: {:?}", err);
                    } else {
                        tracing::info!("StopAll command sent to the engine controller");
                        break;
                    }
                }
            }
        }
        tracing::info!("Stopped processing the commands sent to the engine");
    }
}
