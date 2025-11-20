// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains code for planning pipeline execution

use data_engine_expressions::PipelineExpression;
use datafusion::prelude::SessionContext;
use otap_df_pdata::OtapArrowRecords;

use crate::error::Result;
use crate::pipeline::PipelineStage;

/// Converts an pipeline expression (AST) into a series of executable pipeline stages.
///
/// The planner analyzes the pipeline definition and decides:
/// - Which operations can be handled by DataFusion stages
/// - Which operations need custom stages (e.g., cross-table filters)
/// - Optimizing by group operations into efficient stages
pub struct PipelinePlanner {}

impl PipelinePlanner {
    /// creates a new instance of `PipelinePlanner`
    pub fn new() -> Self {
        Self {}
    }

    /// Create pipeline stages from the pipeline definition.
    ///
    /// # Parameters
    /// - `session_context`: For creating DataFusion logical/physical plans
    /// - `pipeline_def`: The OPL expression tree to compile
    /// - `otap_batch`: The first batch, used to extract schemas for planning
    ///
    /// # Returns
    /// A vector of compiled stages ready for execution
    pub fn plan_stages(
        &mut self,
        _session_context: &SessionContext,
        _pipeline: &PipelineExpression,
        _otap_batch: &OtapArrowRecords,
    ) -> Result<Vec<Box<dyn PipelineStage>>> {
        todo!()
    }
}
