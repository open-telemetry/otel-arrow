// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use async_trait::async_trait;
use datafusion::config::ConfigOptions;
use datafusion::execution::TaskContext;
use datafusion::execution::context::SessionContext;
use otap_df_pdata::OtapArrowRecords;

use crate::error::{Error, Result};
use crate::pipeline::PipelineStage;
use crate::pipeline::state::ExecutionState;

/// TODO docs
#[async_trait]
pub trait Router {
    /// TODO docs
    async fn send(&self, route_name: &str, otap_batch: OtapArrowRecords) -> Result<()>;
}

/// TODO comments
pub struct RouteToPipelineStage {
    outport_name: String,
}

impl RouteToPipelineStage {
    /// TODO comments
    pub fn new(outport_name: &str) -> Self {
        Self {
            outport_name: outport_name.to_string(),
        }
    }
}

#[async_trait(?Send)]
impl PipelineStage for RouteToPipelineStage {
    async fn execute(
        &mut self,
        mut otap_batch: OtapArrowRecords,
        session_context: &SessionContext,
        _config_options: &ConfigOptions,
        task_context: Arc<TaskContext>,
        _exec_state: &mut ExecutionState,
    ) -> Result<OtapArrowRecords> {
        // route otap_batch to self.outport_name
        // return empty batch
        todo!()
    }
}
