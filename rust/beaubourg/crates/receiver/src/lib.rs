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

//! Definition of the `Receiver` trait.

pub mod effect;
pub mod signal;

use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use context::Context;
use serde_yaml::Value;
use task::TaskManager;
use tokio::sync::mpsc::Sender;

use crate::{
    effect::EffectHandler,
    signal::{Signal, SignalReceiver},
};

/// List of errors that can occur with a `Receiver`.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// The `Receiver` was not found in the receiver factory.
    #[error("unknown receiver (receiver: {receiver}, type: {receiver_type})")]
    UnknownReceiver {
        /// The name of the receiver.
        receiver: String,
        /// The type of the receiver.
        receiver_type: String,
    },

    /// The receiver configuration is invalid.
    #[error("invalid configuration (reason: {message}, receiver: {receiver}, line: {line:?}, column: {column:?})")]
    InvalidConfig {
        /// The name of the receiver.
        receiver: String,
        /// The error message.
        message: String,
        /// The line number in the configuration file.
        line: Option<usize>,
        /// The column number in the configuration file.
        column: Option<usize>,
    },

    /// The receiver failed for some reason.
    #[error("receiver error (receiver: {receiver}, reason: {error}, context: {context:?})")]
    Receiver {
        /// The name of the receiver.
        receiver: String,
        /// The error message.
        error: String,
        /// The context of the error.
        context: HashMap<String, String>,
    },

    /// The TCP listener failed to be created.
    #[error("tcp listener error (receiver: {receiver}, reason: {error})")]
    TcpListener {
        /// The name of the receiver.
        receiver: String,
        /// The error message.
        error: String,
    },
}

/// A engine handler that can be used by a receiver to interact with the engine.
pub struct EngineHandler {
    /// The task manager.
    task_manager: TaskManager,

    /// An initialization context passed to the engine.
    context: Arc<Context>,
}

impl EngineHandler {
    /// Creates a new `EngineHandler`.
    pub fn new(task_manager: TaskManager, context: Arc<Context>) -> Self {
        Self { task_manager, context }
    }

    /// Returns the task manager.
    pub fn task_manager(&self) -> TaskManager {
        self.task_manager.clone()
    }

    /// Returns the context.
    pub fn context(&self) -> Arc<Context> {
        self.context.clone()
    }
}

/// The receiver trait.
#[async_trait]
pub trait AsyncReceiver<Msg: 'static + Clone + Send>: Send {
    // Optional methods
    /// Method called when the receiver is initialized.
    async fn init(&mut self, _engine_handler: EngineHandler) -> Result<(), Error> {
        Ok(())
    }
    /// Method called when the receiver is stopped.
    async fn stop(&mut self) -> Result<(), Error> {
        Ok(())
    }

    // Mandatory methods
    /// Method called to start the receiver.
    async fn receive(
        &mut self,
        signal_receiver: SignalReceiver,
        effect_handler: EffectHandler<Msg>,
    ) -> Result<(), Error>;
}

/// The receiver factory trait.
pub trait ReceiverFactory<Msg: 'static + Clone + Send>: Send {
    /// Creates a new receiver from a name, a type, and a configuration.
    fn create(
        &self,
        receiver_name: &str,
        receiver_type: &str,
        config: Value,
    ) -> Result<Box<dyn AsyncReceiver<Msg> + Send + Sync>, Error>;
}

/// Receivers manager.
#[derive(Default)]
pub struct ReceiversController {
    /// A map of receivers.
    receivers: HashMap<String, Vec<Sender<Signal>>>,
}

impl ReceiversController {
    /// Adds a receiver to the manager.
    pub fn add_receiver(&mut self, receiver_name: String, sender: Sender<Signal>) {
        self.receivers
            .entry(receiver_name)
            .or_insert_with(Vec::new)
            .push(sender);
    }

    /// Stop all receivers.
    pub fn stop_all(&self) {
        for (receiver_name, receivers) in self.receivers.iter() {
            for receiver in receivers {
                tracing::info!(%receiver_name, "stopping receiver");
                if let Err(error) = receiver.try_send(Signal::Stop) {
                    tracing::error!(
                        %error,
                        %receiver_name,
                        "sending stop signal to receiver failed"
                    );
                }
            }
        }
    }

    /// Stop a receiver by name.
    pub fn stop_receiver(&self, receiver_name: &str) {
        if let Some(receivers) = self.receivers.get(receiver_name) {
            for receiver in receivers {
                if let Err(error) = receiver.try_send(Signal::Stop) {
                    tracing::error!(
                        %error,
                        %receiver_name,
                        "sending stop signal to receiver failed"
                    );
                }
            }
        }
    }
}
