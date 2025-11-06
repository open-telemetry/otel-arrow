// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::any::Any;
use std::fmt::{self, Debug, Formatter};
use std::sync::{Arc, RwLock};

use arrow::array::{Array, Int16Array, RecordBatch, RunArray, new_null_array};
use arrow::datatypes::{Field, Schema, SchemaRef};
use datafusion::catalog::memory::{DataSourceExec, MemorySourceConfig};
use datafusion::common::Statistics;
use datafusion::config::ConfigOptions;
use datafusion::datasource::source::DataSource;
use datafusion::error::{DataFusionError, Result};
use datafusion::execution::{SendableRecordBatchStream, TaskContext};
use datafusion::physical_expr::EquivalenceProperties;
use datafusion::physical_optimizer::PhysicalOptimizerRule;
use datafusion::physical_plan::display::DisplayFormatType;
use datafusion::physical_plan::execution_plan::SchedulingType;
use datafusion::physical_plan::joins::HashJoinExec;
use datafusion::physical_plan::projection::ProjectionExpr;
use datafusion::physical_plan::{
    DisplayAs, ExecutionPlan, Partitioning, PlanProperties, with_new_children_if_necessary,
};
use otel_arrow_rust::otap::OtapArrowRecords;
use otel_arrow_rust::proto::opentelemetry::arrow::v1::ArrowPayloadType;

/// Custom implementation of [`DataSource`] that allows the in-memory batch and projections to
/// be updated at runtime. The intention is for this to be used by [`OtapBatchDataSource`] so it
/// can update the current batch the physical plan will receive as input, without having to
/// rebuild the entire plan. This is done to get optimal performance.
#[derive(Debug)]
pub struct OtapBatchDataSource {
    curr_memory_source: RwLock<MemorySourceConfig>,
}

impl OtapBatchDataSource {
    pub fn try_new(batch: RecordBatch, projections: Option<Vec<usize>>) -> Result<Self> {
        let schema = batch.schema();
        Ok(Self {
            curr_memory_source: RwLock::new(MemorySourceConfig::try_new(
                &[vec![batch]],
                schema,
                projections,
            )?),
        })
    }

    fn original_schema(&self) -> SchemaRef {
        let curr_source = self.curr_memory_source.read().unwrap();
        curr_source.original_schema()
    }

    fn projection(&self) -> Option<Vec<usize>> {
        let curr_source = self.curr_memory_source.read().unwrap();
        // TODO see if there's a way to avoid the clone here
        curr_source.projection().clone()
    }

    fn replace_batch(
        &self,
        next_batch: RecordBatch,
        projections: Option<Vec<usize>>,
    ) -> Result<()> {
        let schema = next_batch.schema();
        let next_source = MemorySourceConfig::try_new(&[vec![next_batch]], schema, projections)?;
        let mut curr_source = self.curr_memory_source.write().unwrap();
        *curr_source = next_source;
        Ok(())
    }
}

// TODO there might be certain cases where we do want to overwrite the behaviour so revisit this
// TODO the current implementation here has a bunch of unwraps on the curr lock. pls fix
impl DataSource for OtapBatchDataSource {
    fn open(
        &self,
        partition: usize,
        context: Arc<TaskContext>,
    ) -> Result<SendableRecordBatchStream> {
        let curr_source = self.curr_memory_source.read().unwrap();
        curr_source.open(partition, context)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn fmt_as(&self, t: DisplayFormatType, f: &mut Formatter) -> fmt::Result {
        let curr_source = self.curr_memory_source.read().unwrap();
        curr_source.fmt_as(t, f)
    }

    fn output_partitioning(&self) -> Partitioning {
        let curr_source = self.curr_memory_source.read().unwrap();
        curr_source.output_partitioning()
    }

    fn eq_properties(&self) -> EquivalenceProperties {
        let curr_source = self.curr_memory_source.read().unwrap();
        curr_source.eq_properties()
    }

    fn scheduling_type(&self) -> SchedulingType {
        let curr_source = self.curr_memory_source.read().unwrap();
        curr_source.scheduling_type()
    }

    fn statistics(&self) -> Result<Statistics> {
        let curr_source = self.curr_memory_source.read().unwrap();
        curr_source.statistics()
    }

    fn with_fetch(&self, _limit: Option<usize>) -> Option<Arc<dyn DataSource>> {
        todo!("implement with_fetch")
    }

    fn fetch(&self) -> Option<usize> {
        let curr_source = self.curr_memory_source.read().unwrap();
        curr_source.fetch()
    }

    fn try_swapping_with_projection(
        &self,
        _projection: &[ProjectionExpr],
    ) -> Result<Option<Arc<dyn DataSource>>> {
        todo!("implement try swapping with projections")
    }
}

/// This is a light wrapper around [`DataSourceExec`] that can update the batch that will be
/// produced when `execute` method is called.
#[derive(Clone, Debug)]
pub struct OtapDataSourceExec {
    pub payload_type: ArrowPayloadType,
    source_plan: DataSourceExec,
}

impl OtapDataSourceExec {
    pub fn new(payload_type: ArrowPayloadType, data_source: OtapBatchDataSource) -> Self {
        Self {
            payload_type,
            source_plan: DataSourceExec::new(Arc::new(data_source)),
        }
    }

    /// Update the [`DataSourceExec`] with the next batch. This will return a new instance of `self`
    /// if columns that are produced will be modified. This is a signal to the caller that the
    /// entire physical plan needs to be reconstructed due to the modified projection. If this
    /// returns `None`, it means the [`DataSource`] was modified in place and the parent plan
    /// can be reused.
    pub fn try_with_next_batch(&self, mut next_batch: RecordBatch) -> Result<Option<Self>> {
        let data_source = self.source_plan.data_source();
        if let Some(curr_data_source) = data_source.as_any().downcast_ref::<OtapBatchDataSource>() {
            let curr_batch_schema = curr_data_source.original_schema();
            let curr_batch_projection = curr_data_source.projection();
            let mut next_batch_schema = next_batch.schema();
            let next_batch_projection = self.next_project(
                curr_batch_projection.as_deref(),
                curr_batch_schema,
                next_batch_schema.clone(),
            );

            // the current batch has the same schema as the next batch, so we will just update the
            // datasource's current batch and return
            if Some(&next_batch_projection.projected_columns) == curr_batch_projection.as_ref()
                && next_batch_projection.placeholders.is_none()
                && !next_batch_projection.must_replan
            {
                curr_data_source
                    .replace_batch(next_batch, Some(next_batch_projection.projected_columns))?;
                return Ok(None);
            }

            // some columns from the previous batch were not present in this batch, so we need to
            // add some all-null placeholders
            //
            // TODO - not entirely sure this is necessary .. revisit this
            if let Some(placeholders) = next_batch_projection.placeholders {
                let mut new_columns = next_batch.columns().to_vec();
                let mut new_fields = next_batch_schema.fields.to_vec();

                // to optimize memory, the placeholders will be a RunArray with a single run of
                // nulls spanning entire length of batch
                // TODO handle if there's more than 2^16 rows in the batch
                let run_ends = Int16Array::from_iter_values([next_batch.num_rows() as i16]);

                for placeholder in placeholders {
                    // safety: none of the criteria that would make run-end panic are met here:
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

            let next_data_source = OtapBatchDataSource::try_new(
                next_batch,
                Some(next_batch_projection.projected_columns),
            )?;
            Ok(Some(Self {
                payload_type: self.payload_type,
                source_plan: DataSourceExec::new(Arc::new(next_data_source)),
            }))
        } else {
            Err(DataFusionError::Plan(format!(
                "Invalid type of DataSource found in OtapDataSourceExec. Found {:?} but expected OtapBatchDataSource",
                data_source
            )))
        }
    }

    /// Computes the projection of the given batch based on the columns from the previous batch.
    ///
    /// We try to project the same columns in the same order because doing this allows the parent
    /// physical plan to be reused. However, this is only possible the exact same set of columns is
    /// present AND all the types are the same
    ///
    /// For columns that were present in the previous batch which are not found in the current
    /// batch, placeholder definitions are returned that should be added to the next batch by the
    /// caller.
    ///
    /// If any existing columns have a different type, the `must_replan` flag will be set to true
    /// on the result.
    //
    // TODO - tests & optimization
    fn next_project(
        &self,
        curr_batch_projection: Option<&[usize]>,
        curr_batch_schema: SchemaRef,
        next_batch_schema: SchemaRef,
    ) -> NextProjection {
        let mut must_replan = false;

        let next_batch_max_field_id = next_batch_schema.fields.len();
        let mut next_batch_projection = Vec::with_capacity(next_batch_max_field_id);
        let mut placeholders = None;

        // first we want to project all columns from the next batch in the same order they were
        // projected in the previous batch
        for curr_batch_field_id in
            ProjectionIter::new(curr_batch_projection, curr_batch_schema.fields.len())
        {
            let curr_batch_field = curr_batch_schema.field(curr_batch_field_id);
            let mut found_next_batch_field_id = None;

            // check if the next batch contains the same field in the same location. This would be
            // the most common case where two subsequent batches have identical schemas
            if curr_batch_field_id < next_batch_max_field_id {
                let next_batch_field = next_batch_schema.field(curr_batch_field_id);
                if curr_batch_field.name() == next_batch_field.name() {
                    found_next_batch_field_id = Some(curr_batch_field_id);
                }
            }

            // if this field in next batch wasn't in the same location as previous batch we search
            if found_next_batch_field_id.is_none() {
                for next_batch_field_id in 0..next_batch_max_field_id {
                    let next_batch_field = next_batch_schema.field(next_batch_field_id);
                    if curr_batch_field.name() == next_batch_field.name() {
                        found_next_batch_field_id = Some(next_batch_field_id);
                        break;
                    }
                }
            }

            if let Some(next_batch_field_id) = found_next_batch_field_id {
                // we must do a light-weight replanning if the type of the column changes
                let curr_data_type = curr_batch_schema.field(curr_batch_field_id).data_type();
                let next_data_type = next_batch_schema.field(next_batch_field_id).data_type();
                let changed_data_type = curr_data_type != next_data_type;
                must_replan |= changed_data_type;

                // push the field ID into the projection
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
                let already_projected = next_batch_projection.contains(&next_batch_field_id);
                if !already_projected {
                    next_batch_projection.push(next_batch_field_id)
                }
            }
        }

        NextProjection {
            projected_columns: next_batch_projection,
            placeholders,
            must_replan,
        }
    }
}

pub struct NextProjection {
    projected_columns: Vec<usize>,
    placeholders: Option<Vec<Field>>,
    must_replan: bool,
}

/// Simple helper type for iterating over a column projection.
enum ProjectionIter<'a> {
    // iterate over some specific columns in a given order
    Slice(std::iter::Copied<std::slice::Iter<'a, usize>>),

    /// iterate over all columns in the range of field IDs
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

/// This implementation of [`PhysicalOptimizerRule`] is responsible for updating the current
/// physical plan to process the batch that it contains. In practice, it gives us a way to reuse
/// the physical plan in an inexpensive way, either by simply updating the underlying
/// [`DataSource`] (if possible), or otherwise by creating a deep clone of the plan. This is less
/// expensive than producing a new physical plan for each incoming batch.
///
// TODO: revisit the debug implementation. We don't want this thing to print all the data in the
// current batch
#[derive(Debug)]
pub struct UpdateDataSourceOptimizer {
    otap_batch: OtapArrowRecords,
}

impl UpdateDataSourceOptimizer {
    pub fn new(otap_batch: OtapArrowRecords) -> Self {
        Self { otap_batch }
    }

    pub fn take_batch(self) -> OtapArrowRecords {
        self.otap_batch
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
        _config: &ConfigOptions,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        if let Some(curr_batch_exec) = plan.as_any().downcast_ref::<OtapDataSourceExec>() {
            if let Some(rb) = self.otap_batch.get(curr_batch_exec.payload_type) {
                let next_batch_exec = curr_batch_exec.try_with_next_batch(rb.clone())?;
                Ok(match next_batch_exec {
                    Some(next_batch_exec) => Arc::new(next_batch_exec),
                    None => plan, // reuse exiting plan
                })
            } else {
                // TODO if the plan selects a batch that doesn't contain some payload type
                // we should redo the planning
                Err(DataFusionError::Plan(format!(
                    "received physical query plan selecting nonexistent OTAP batch {:?}",
                    curr_batch_exec.payload_type
                )))
            }
        } else if let Some(curr_hash_join) = plan.as_any().downcast_ref::<HashJoinExec>() {
            // [`HashJoinExec`] is a special case we need to handle. Internally, it keeps a future
            // of what the left-side of the join produces and this future is of a type that will
            // only execute once. So for this case, we need to somehow reset the plan

            // traverse down both sides of the join and update the current batch.
            let curr_left = curr_hash_join.left.clone();
            let curr_right = curr_hash_join.right.clone();
            let left = self.optimize(curr_left.clone(), _config)?;
            let right = self.optimize(curr_right.clone(), _config)?;

            // if we are able to reuse the left/right side, just call reset_state or otherwise
            // create a whole new HashJoinExec plan.
            //
            // TODO not sure if this matters a lot in terms of saving execution time, e.g. doing
            // reset_state vs try_new might not make a huge difference, need to remeasure.
            if Arc::ptr_eq(&curr_left, &left) && Arc::ptr_eq(&curr_right, &right) {
                plan.reset_state()
            } else {
                let new_hash_join = HashJoinExec::try_new(
                    left,
                    right,
                    curr_hash_join.on.clone(),
                    curr_hash_join.filter.clone(),
                    curr_hash_join.join_type(),
                    curr_hash_join.projection.clone(),
                    *curr_hash_join.partition_mode(),
                    curr_hash_join.null_equality,
                )?;
                Ok(Arc::new(new_hash_join))
            }
        } else {
            let children = plan
                .children()
                .into_iter()
                .map(|child| self.optimize(child.clone(), _config))
                .collect::<Result<Vec<_>>>()?;
            with_new_children_if_necessary(plan, children)
        }
    }
}
