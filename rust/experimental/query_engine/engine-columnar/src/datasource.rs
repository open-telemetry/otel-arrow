// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::any::Any;
use std::fmt::{self, Formatter};
use std::sync::Arc;

use arrow::array::RecordBatch;
use datafusion::catalog::memory::{DataSourceExec, MemorySourceConfig};
use datafusion::datasource::sink::DataSinkExec;
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
    pub fn try_with_next_batch(&self, rb: RecordBatch) -> Result<Self> {
        let data_source = self.source_plan.data_source();
        if let Some(mem_ds) = data_source.as_any().downcast_ref::<MemorySourceConfig>() {
            let schema = rb.schema();
            let next_data_source =
                MemorySourceConfig::try_new(&[vec![rb]], schema, mem_ds.projection().clone())?;
            Ok(Self {
                payload_type: self.payload_type,
                source_plan: DataSourceExec::new(Arc::new(next_data_source)),
            })
        } else {
            todo!("throw")
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
