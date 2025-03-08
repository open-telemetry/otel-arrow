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

//! Definition of the exporter trait.

use std::{collections::HashMap, sync::Arc, time::Duration};

use async_trait::async_trait;
use context::Context;
use effect::EffectHandler;
use serde_yaml::Value;
use signal::SignalReceiver;
use task::labels::{ProcessLabels, TaskLabels};

pub mod effect;

/// All the errors that can occur with an exporter.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// The exporter is not found in the exporter factory.
    #[error("unknown exporter (name: {name}, type: {exporter_type})")]
    UnknownExporter {
        /// The name of the exporter.
        name: String,
        /// The type of the exporter.
        exporter_type: String,
    },

    /// The exporter has an invalid configuration.
    #[error("invalid configuration (reason: {message}, exporter: {exporter}, line: {line:?}, column: {column:?})")]
    InvalidConfig {
        /// The name of the exporter.
        exporter: String,
        /// The error message.
        message: String,
        /// The line of the error.
        line: Option<usize>,
        /// The column of the error.
        column: Option<usize>,
    },

    /// The exporter failed for some reason.
    #[error("exporter error (exporter: {exporter}, reason: {error}, context: {context:?})")]
    Exporter {
        /// The name of the exporter.
        exporter: String,
        /// The error message.
        error: String,
        /// The context of the error.
        context: HashMap<String, String>,
    },

    /// The exporter failed because it doesn't support a signal variant.
    #[error("exporter not supporting a signal variant (exporter: {exporter}, signal-variant: {signal})")]
    UnsupportedEvent {
        /// The name of the exporter.
        exporter: String,
        /// The name of the signal variant.
        signal: String,
    },
}

/// A structure containing a set of debug information for an exporter.
#[derive(Clone, Debug)]
pub struct DebugInfo {
    /// The name of the pipeline where the exporter is used.
    pub pipeline_name: String,
    /// The name of the exporter.
    pub exporter_name: String,
    /// The id of the process where the exporter is used.
    pub process_id: String,
    /// The category of the task where the exporter is used.
    pub task_cat: String,
    /// The id of the task where the exporter is used.
    pub task_id: String,
    /// The source of the task where the exporter is used.
    pub task_source: String,
}

impl DebugInfo {
    /// Creates a new `DebugInfo` structure.
    pub fn new(
        pipeline_name: String,
        exporter_name: String,
        process_labels: ProcessLabels,
        task_labels: TaskLabels,
    ) -> Self {
        Self {
            pipeline_name,
            exporter_name,
            process_id: process_labels.process_id,
            task_cat: task_labels.task_cat.clone(),
            task_id: task_labels.task_id.clone(),
            task_source: task_labels.task_source,
        }
    }
}

/// The exporter trait.
#[async_trait]
pub trait AsyncExporter<Msg: 'static + Clone + Send>: Send {
    // Optional methods
    /// Method called when the exporter is initialized.
    async fn init(&mut self, _engine_handler: &mut EngineHandler) -> Result<(), Error> {
        Ok(())
    }
    /// Method called when the exporter is stopped.
    async fn stop(&mut self) -> Result<(), Error> {
        Ok(())
    }

    // Mandatory methods
    /// Method called when the exporter is started.
    async fn export(
        &mut self,
        signal_receiver: SignalReceiver<Msg>,
        effect_handler: EffectHandler<Msg>,
    ) -> Result<(), Error>;
}

/// Concurrency model for the exporter.
#[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ConcurrencyModel {
    /// The exporter is not concurrent.
    Singleton,
    /// Number of tasks per CPU core.
    TaskPerCore(u16),
}

/// A trait to build an exporter for a given name and type.
pub trait ExporterBuilder<Msg>
where
    Msg: 'static + Clone + Send,
{
    /// Returns the name of the exporter.
    fn name(&self) -> String;
    /// Returns the type of the exporter.
    fn r#type(&self) -> String;
    /// Returns the concurrency model of the exporter.
    fn concurrency_model(&self) -> ConcurrencyModel;
    /// Builds and returns a new instance of this exporter.
    fn build(&self) -> Result<Box<dyn AsyncExporter<Msg> + Send + Sync>, Error>;
}

/// The exporter factory.
pub trait ExporterFactory<Msg: 'static + Clone + Send> {
    /// Returns an exporter builder or None if the exporter is not found.
    fn builder(
        &self,
        exporter_name: &str,
        exporter_type: &str,
        config: Value,
    ) -> Option<Box<dyn ExporterBuilder<Msg> + Send + Sync>>;
}

/// The engine handler.
#[derive(Default)]
pub struct EngineHandler {
    /// A timer to be used by the exporters.
    timer: Option<Duration>,

    /// An initialization context passed to the engine.
    context: Arc<Context>,
}

impl EngineHandler {
    /// Create a new engine handler.
    pub fn new(context: Arc<Context>) -> Self {
        Self { timer: None, context }
    }

    /// Set a timer to be used by the exporters.
    pub fn timer(&mut self, duration: Duration) {
        self.timer = Some(duration);
    }

    /// Get the timer configured with the timer method.
    pub fn get_timer(&self) -> Option<&Duration> {
        self.timer.as_ref()
    }

    /// Returns the context.
    pub fn context(&self) -> Arc<Context> {
        self.context.clone()
    }
}

#[cfg(test)]
mod test {
    use crate::ConcurrencyModel;

    #[derive(serde::Serialize, serde::Deserialize)]
    struct ExporterConfig {
        concurreny_model: ConcurrencyModel,
    }

    #[test]
    fn serialize_singleton() {
        let config = ExporterConfig {
            concurreny_model: ConcurrencyModel::Singleton,
        };

        let yaml = serde_yaml::to_string(&config).unwrap();
        assert_eq!(
            yaml,
            "---
concurreny_model: singleton
"
        );
    }

    #[test]
    fn deserialize_singleton() {
        let yaml = "---
concurreny_model: singleton
";

        let config: ExporterConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.concurreny_model, ConcurrencyModel::Singleton);
    }

    #[test]
    fn deserialize_thread_per_core() {
        let yaml = "---
concurreny_model:
  task_per_core: 2
";

        let config: ExporterConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.concurreny_model, ConcurrencyModel::TaskPerCore(2));
    }
}
