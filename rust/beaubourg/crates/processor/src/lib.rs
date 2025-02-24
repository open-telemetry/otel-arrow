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

//! Definition of the `Processor` trait.

use std::{collections::HashMap, sync::Arc, time::Duration};

use async_trait::async_trait;
use context::Context;
use serde_yaml::Value;
use signal::Signal;

use crate::effect::EffectHandler;

pub mod effect;
pub mod noop;

/// All the errors that can occur with a processor.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// The processor is not defined by the processor factory.
    #[error("unknown processor (processor: {processor}, type:{r#type})")]
    UnknownProcessor {
        /// The name of the processor.
        processor: String,
        /// The type of the processor.
        r#type: String,
    },

    /// The processor has an invalid configuration.
    #[error("invalid configuration (reason: {message}, processor: {processor}, line: {line:?}, column: {column:?})")]
    InvalidConfig {
        /// The name of the processor.
        processor: String,
        /// The error message.
        message: String,
        /// The line of the error.
        line: Option<usize>,
        /// The column of the error.
        column: Option<usize>,
    },

    /// The processor failed for some reason.
    #[error("processor error (processor: {processor}, reason: {error}, context: {context:?})")]
    Processor {
        /// The name of the processor.
        processor: String,
        /// The error message.
        error: String,
        /// The context of the error.
        context: HashMap<String, String>,
    },

    /// The processor (or the processor chain) failed because it doesn't support
    /// a signal variant.
    #[error("processor not supporting a signal variant (processor: {processor}, signal-variant: {signal})")]
    UnsupportedEvent {
        /// The name of the processor.
        processor: String,
        /// The name of the signal variant.
        signal: String,
    },
}

/// The Processor trait.
#[async_trait]
pub trait AsyncProcessor<Msg: 'static + Clone + Send>: Send {
    // Optional methods
    /// Method called when the processor is initialized.
    async fn init(&mut self, _engine_handler: &mut EngineHandler) -> Result<(), Error> {
        Ok(())
    }
    /// Method called when the processor is stopped.
    async fn stop(&mut self) -> Result<(), Error> {
        Ok(())
    }

    // Mandatory methods
    /// Method called when the processor receives a signal (e.g. TimerTick,
    /// MessagesReceived).
    async fn process(&mut self, signal: Signal<Msg>, effect_handler: &mut EffectHandler<Msg>) -> Result<(), Error>;
}

/// The ProcessorFactory trait.
pub trait ProcessorFactory<Msg: 'static + Clone + Send> {
    /// Create a new processor.
    fn create(
        &self,
        processor_name: &str,
        processor_type: &str,
        config: Value,
    ) -> Result<Box<dyn AsyncProcessor<Msg> + Send + Sync>, Error>;
}

/// The EngineHandler trait.
#[derive(Default)]
pub struct EngineHandler {
    /// An optional timer to schedule signals.
    timer: Option<Duration>,

    /// An initialization context passed to the engine.
    context: Arc<Context>,
}

impl EngineHandler {
    /// Create a new engine handler.
    pub fn new(context: Arc<Context>) -> Self {
        Self { timer: None, context }
    }

    /// Sets a timer to schedule periodic TimerTick signals.
    pub fn timer(&mut self, duration: Duration) {
        self.timer = Some(duration);
    }

    /// Returns the timer configured by the 'timer' method.
    pub fn get_timer(&self) -> Option<&Duration> {
        self.timer.as_ref()
    }

    /// Returns the context.
    pub fn context(&self) -> Arc<Context> {
        self.context.clone()
    }
}
