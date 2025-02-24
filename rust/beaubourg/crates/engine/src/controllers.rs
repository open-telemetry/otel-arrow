//! Controllers expose an interface (set of commands) for interacting with the
//! engine, pipelines, and other components from the exterior.
//! The engine controller is composed of a set of pipeline controllers.
//! The pipeline controllers are composed of a set of receiver controllers.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use receiver::ReceiversController;

use crate::Error;

/// An engine controller composed of a set of pipeline controllers.
/// PipelineControllers are stored in a HashMap by their names
#[derive(Default, Clone)]
pub(crate) struct EngineController {
    /// Multiple pipeline controller instances can be stored in a HashMap under
    /// the same name. This can occur when multiple instances of the same
    /// pipeline are running concurrently.
    pipelines: Arc<Mutex<HashMap<String, Vec<PipelineController>>>>,
}

impl EngineController {
    /// Adds a new pipeline controller.
    pub(crate) fn add_pipeline(
        &self,
        pipeline_name: String,
        pipeline_controller: PipelineController,
    ) -> Result<(), Error> {
        self.pipelines
            .lock()
            .map_err(|e| Error::Runtime {
                error: format!("failed to lock pipelines hashmap (error: {:?}", e),
            })?
            .entry(pipeline_name)
            .or_insert_with(Vec::new)
            .push(pipeline_controller);
        Ok(())
    }

    /// Stops all the pipelines.
    pub(crate) fn stop_all(&self) -> Result<(), Error> {
        let pipelines = self
            .pipelines
            .lock()
            .map_err(|e| Error::Runtime { error: e.to_string() })?;

        for (pipeline_name, pipelines) in pipelines.iter() {
            tracing::info!(%pipeline_name, num_instances=%pipelines.len(), "Stopping the engine and all its pipeline instances");
            for pipeline in pipelines {
                pipeline.receivers.stop_all();
            }
        }
        Ok(())
    }
}

/// A pipeline controller composed of a collection of receiver controllers.
pub(crate) struct PipelineController {
    receivers: ReceiversController,
}

impl PipelineController {
    /// Creates a new pipeline controller.
    pub(crate) fn new(receivers_controller: ReceiversController) -> Self {
        Self {
            receivers: receivers_controller,
        }
    }
}
