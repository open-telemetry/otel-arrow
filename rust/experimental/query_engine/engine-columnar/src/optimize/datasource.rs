// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use datafusion::catalog::{MemTable, TableProvider};
use datafusion::config::ConfigOptions;
use datafusion::error::{DataFusionError, Result};
use datafusion::physical_optimizer::optimizer::PhysicalOptimizerRule;
use datafusion::physical_plan::{ExecutionPlan, with_new_children_if_necessary};
use otel_arrow_rust::otap::OtapArrowRecords;

use crate::datasource::OtapDataSourceExec;

// TODO:
// - document what this is doing
// - check the debug implementation if it spews out a bunch of data
#[derive(Debug)]
struct UpdateDataSourceOptimizer {
    otap_batch: OtapArrowRecords,
}

impl PhysicalOptimizerRule for UpdateDataSourceOptimizer {
    fn name(&self) -> &'static str {
        "UpdateDataSourceOptimizer"
    }

    fn schema_check(&self) -> bool {
        // TODO double check that this is what we want.
        false
    }

    fn optimize(
        &self,
        plan: Arc<dyn ExecutionPlan>,
        config: &ConfigOptions,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        if plan.as_any().is::<OtapDataSourceExec>() {
            // TODO combine this next statement with the if condition like if let Some(...
            // safety: we've just checked the type
            let curr_batch_exec = plan
                .as_any()
                .downcast_ref::<OtapDataSourceExec>()
                .expect("can downcast to type");
            if let Some(rb) = self.otap_batch.get(curr_batch_exec.payload_type) {
                let next_batch_exec = curr_batch_exec.try_with_next_batch(rb.clone())?;
                Ok(Arc::new(next_batch_exec))
            } else {
                // TODO if the plan selects a batch that doesn't contain some payload type, we should redo the planning
                Err(DataFusionError::Plan(format!(
                    "received physical query plan selecting nonexistent OTAP batch {:?}",
                    curr_batch_exec.payload_type
                )))
            }
        } else {
            let children = plan
                .children()
                .into_iter()
                .map(|child| self.optimize(child.clone(), config))
                .collect::<Result<Vec<_>>>()?;
            with_new_children_if_necessary(plan, children)
        }
    }
}
