// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::any::Any;
use std::fmt::{self, Formatter};
use std::sync::Arc;

use arrow::array::{Array, Int16Array, RecordBatch, RunArray, new_null_array};
use arrow::datatypes::{Field, Schema, SchemaRef};
use datafusion::catalog::memory::{DataSourceExec, MemorySourceConfig};
use datafusion::config::ConfigOptions;
use datafusion::error::{DataFusionError, Result};
use datafusion::execution::{SendableRecordBatchStream, TaskContext};
use datafusion::physical_optimizer::PhysicalOptimizerRule;
use datafusion::physical_plan::display::DisplayFormatType;
use datafusion::physical_plan::joins::HashJoinExec;
use datafusion::physical_plan::{
    DisplayAs, ExecutionPlan, PlanProperties, with_new_children_if_necessary,
};
use otel_arrow_rust::otap::OtapArrowRecords;
use otel_arrow_rust::proto::opentelemetry::arrow::v1::ArrowPayloadType;

// TODO comment on what this is for
#[derive(Clone, Debug)]
pub struct OtapDataSourceExec {
    pub payload_type: ArrowPayloadType,
    source_plan: DataSourceExec,
}

impl OtapDataSourceExec {
    pub fn new(payload_type: ArrowPayloadType, data_source: MemorySourceConfig) -> Self {
        Self {
            payload_type,
            source_plan: DataSourceExec::new(Arc::new(data_source)),
        }
    }

    pub fn try_with_next_batch(&self, mut next_batch: RecordBatch) -> Result<Self> {
        let data_source = self.source_plan.data_source();
        if let Some(curr_data_source) = data_source.as_any().downcast_ref::<MemorySourceConfig>() {
            let curr_batch_schema = curr_data_source.original_schema();
            let curr_batch_projection = curr_data_source.projection();
            let mut next_batch_schema = next_batch.schema();
            let (next_batch_projection, placeholders) = self.next_project(
                curr_batch_projection,
                curr_batch_schema,
                next_batch_schema.clone(),
            );

            // some columns from the previous batch were not present in this batch, so we need to
            // add some all-null placeholders
            if let Some(placeholders) = placeholders {
                let mut new_columns = next_batch.columns().to_vec();
                let mut new_fields = next_batch_schema.fields.to_vec();

                // to optimize memory, the placeholders will be a RunArray with a single run of
                // nulls spanning entire length of batch
                // TODO handle if there's more than 2^16 rows in the batch
                let run_ends = Int16Array::from_iter_values([next_batch.num_rows() as i16]);

                for placeholder in placeholders {
                    // safety: non of the criteria that would make run-end panic are met here:
                    // https://github.com/apache/arrow-rs/blob/57447434d1921701456543f9dfb92741e5d86734/arrow-array/src/array/run_array.rs#L109-L115
                    let new_column = RunArray::try_new(
                        &run_ends,
                        new_null_array(placeholder.data_type(), 1).as_ref(),
                    )
                    .expect("valid run end");
                    let new_field = placeholder.with_data_type(new_column.data_type().clone());
                    new_fields.push(Arc::new(new_field));
                    new_columns.push(Arc::new(new_column));
                }

                next_batch_schema = Arc::new(Schema::new(new_fields));
                // safety: TODO explain why this is safe
                next_batch = RecordBatch::try_new(next_batch_schema.clone(), new_columns)
                    .expect("can build new record batch");
            }

            let next_data_source = MemorySourceConfig::try_new(
                &[vec![next_batch]],
                next_batch_schema,
                Some(next_batch_projection),
            )?;

            Ok(Self {
                payload_type: self.payload_type,
                source_plan: DataSourceExec::new(Arc::new(next_data_source)),
            })
        } else {
            todo!("throw")
        }
    }

    // TODO - comments
    // TODO - tests
    // TODO - optimize
    fn next_project(
        &self,
        curr_batch_projection: &Option<Vec<usize>>,
        curr_batch_schema: SchemaRef,
        next_batch_schema: SchemaRef,
    ) -> (Vec<usize>, Option<Vec<Field>>) {
        let next_batch_max_field_id = next_batch_schema.fields.len();
        let mut next_batch_projection = Vec::with_capacity(next_batch_max_field_id);
        let mut placeholders = None;

        // first we want to project all columns from the next batch in the same order they were
        // projected in the previous batch
        for curr_batch_field_id in ProjectionIter::new(
            curr_batch_projection.as_deref(),
            curr_batch_schema.fields.len(),
        ) {
            let curr_batch_field = curr_batch_schema.field(curr_batch_field_id);

            // check if the next batch contains the same field in the same location. This would be
            // the most common case where two subsequent batches have identical schemas
            if curr_batch_field_id < next_batch_max_field_id {
                let next_batch_field = next_batch_schema.field(curr_batch_field_id);
                if curr_batch_field.name() == next_batch_field.name() {
                    next_batch_projection.push(curr_batch_field_id);
                    continue;
                }
            }

            // this field in next batch wasn't in the same location as previous batch
            // so we search for it
            let mut found_next_batch_field_id = None;
            for next_batch_field_id in 0..next_batch_max_field_id {
                let next_batch_field = next_batch_schema.field(next_batch_field_id);
                if curr_batch_field.name() == next_batch_field.name() {
                    found_next_batch_field_id = Some(next_batch_field_id);
                    break;
                }
            }

            if let Some(next_batch_field_id) = found_next_batch_field_id {
                next_batch_projection.push(next_batch_field_id);
                continue;
            }

            // the current batch's field wasn't found in the next batch, so we need to
            // add a placeholder
            if placeholders.is_none() {
                placeholders = Some(Vec::new());
            }
            let placeholders = placeholders.as_mut().expect("placeholders initialized");

            // the placeholder columns will be appended to the end of the columns of the batch
            next_batch_projection.push(next_batch_max_field_id + placeholders.len());
            placeholders.push(curr_batch_field.clone());
        }

        // check if it's possible we haven't added to the projection some columns from the next batch
        if next_batch_schema.fields.len() != curr_batch_schema.fields.len()
            || placeholders.is_some()
        {
            // add the columns from the next batch that were not projected
            for next_batch_field_id in 0..next_batch_max_field_id {
                let already_projected = next_batch_projection
                    .iter()
                    .any(|projected_field_id| *projected_field_id == next_batch_field_id);
                if !already_projected {
                    next_batch_projection.push(next_batch_field_id)
                }
            }
        }

        (next_batch_projection, placeholders)
    }
}

// TODO comment on what this is for
enum ProjectionIter<'a> {
    Slice(std::iter::Copied<std::slice::Iter<'a, usize>>),
    Range(std::ops::Range<usize>),
}

impl<'a> ProjectionIter<'a> {
    fn new(source: Option<&'a [usize]>, num_fields: usize) -> Self {
        match source {
            Some(field_ids) => Self::Slice(field_ids.iter().copied()),
            None => Self::Range(0..num_fields),
        }
    }
}

impl<'a> Iterator for ProjectionIter<'a> {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ProjectionIter::Slice(it) => it.next(),
            ProjectionIter::Range(it) => it.next(),
        }
    }
}

// TODO refactor into using this
struct BatchProjection {
    columns: Vec<usize>,
    placeholders: Option<Vec<Field>>,
}

impl DisplayAs for OtapDataSourceExec {
    fn fmt_as(&self, t: DisplayFormatType, f: &mut Formatter) -> fmt::Result {
        match t {
            DisplayFormatType::Default | DisplayFormatType::Verbose => {
                write!(f, "OtapDataSourceExec: ")?;
            }
            DisplayFormatType::TreeRender => {}
        }
        self.source_plan.fmt_as(t, f)
    }
}

impl ExecutionPlan for OtapDataSourceExec {
    fn name(&self) -> &'static str {
        "OtapDataSourceExec"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn properties(&self) -> &PlanProperties {
        self.source_plan.properties()
    }

    fn children(&self) -> Vec<&Arc<dyn ExecutionPlan>> {
        Vec::new()
    }

    fn with_new_children(
        self: Arc<Self>,
        _children: Vec<Arc<dyn ExecutionPlan>>,
    ) -> datafusion::error::Result<Arc<dyn ExecutionPlan>> {
        Ok(self)
    }

    fn execute(
        &self,
        partition: usize,
        context: Arc<TaskContext>,
    ) -> Result<SendableRecordBatchStream> {
        self.source_plan.execute(partition, context)
    }
}

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
                curr_hash_join.null_equality.clone(),
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
