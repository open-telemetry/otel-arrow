// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use datafusion::catalog::{MemTable, TableProvider};
use datafusion::config::ConfigOptions;
use datafusion::error::{DataFusionError, Result};
use datafusion::physical_optimizer::optimizer::PhysicalOptimizerRule;
use datafusion::physical_plan::joins::HashJoinExec;
use datafusion::physical_plan::{ExecutionPlan, with_new_children_if_necessary};
use otel_arrow_rust::otap::OtapArrowRecords;

use crate::datasource::OtapDataSourceExec;

// TODO:
// - document what this is doing
// - check the debug implementation if it spews out a bunch of data
#[derive(Debug)]
pub struct UpdateDataSourceOptimizer {
    otap_batch: OtapArrowRecords,
}

impl UpdateDataSourceOptimizer {
    pub fn new(otap_batch: OtapArrowRecords) -> Self {
        Self { otap_batch }
    }
}

impl PhysicalOptimizerRule for UpdateDataSourceOptimizer {
    fn name(&self) -> &'static str {
        "UpdateDataSourceOptimizer"
    }

    fn schema_check(&self) -> bool {
        // TODO double check that this is what we want.
        false
    }

    // TODO add a note about the implementation here about why it works like it does and how
    // we need it to call with_new_children recursively so it resets the state of all the
    // stateful execution plan steps parents above the datasource (e.g. repartitioning and stuff)
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
        } else if let Some(curr_hash_join) = plan.as_any().downcast_ref::<HashJoinExec>() {
            // TODO comment on why we do this
            let left = self.optimize(curr_hash_join.left.clone(), config)?;
            let right = self.optimize(curr_hash_join.right.clone(), config)?;
            println!("projection = {:?}", curr_hash_join.projection);
            let new_hash_join = HashJoinExec::try_new(
                left,
                right, 
                curr_hash_join.on.clone(), 
                curr_hash_join.filter.clone(),
                curr_hash_join.join_type(),
                curr_hash_join.projection.clone(),
                curr_hash_join.partition_mode().clone(), 
                curr_hash_join.null_equality.clone()
            )?;
            Ok(Arc::new(new_hash_join))
        
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
