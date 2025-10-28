// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::any::Any;
use std::fmt::{self, Formatter};
use std::sync::Arc;

use arrow::array::RecordBatch;
use arrow::datatypes::{Field, SchemaRef};
use datafusion::catalog::memory::{DataSourceExec, MemorySourceConfig};
use datafusion::error::Result;
use datafusion::execution::{SendableRecordBatchStream, TaskContext};
use datafusion::physical_plan::display::DisplayFormatType;
use datafusion::physical_plan::{DisplayAs, ExecutionPlan, PlanProperties};
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

    pub fn try_with_next_batch(&self, next_batch: RecordBatch) -> Result<Self> {
        let data_source = self.source_plan.data_source();
        if let Some(curr_data_source) = data_source.as_any().downcast_ref::<MemorySourceConfig>() {
            let curr_batch_schema = curr_data_source.original_schema();
            let curr_batch_projection = curr_data_source.projection();
            let next_batch_schema = next_batch.schema();
            let (next_batch_projection, placeholders) = self.next_project(
                curr_batch_projection,
                curr_batch_schema,
                next_batch_schema.clone(),
            );

            println!("next projection = {:?}", next_batch_projection);

            if let Some(placeholders) = placeholders {
                todo!("add column placeholders")
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
