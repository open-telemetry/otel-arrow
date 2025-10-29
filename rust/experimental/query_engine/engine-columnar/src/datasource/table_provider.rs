// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use async_trait::async_trait;
use datafusion::arrow::datatypes::SchemaRef;
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::catalog::Session;
use datafusion::catalog::memory::MemorySourceConfig;
use datafusion::common::Result;
use datafusion::datasource::{TableProvider, TableType};
use datafusion::logical_expr::Expr;
use datafusion::physical_plan::ExecutionPlan;
use otel_arrow_rust::proto::opentelemetry::arrow::v1::ArrowPayloadType;

use crate::datasource::exec::OtapDataSourceExec;

#[derive(Debug)]
pub struct OtapBatchTable {
    payload_type: ArrowPayloadType,
    current_batch: RecordBatch,
}

impl OtapBatchTable {
    pub fn new(payload_type: ArrowPayloadType, record_batch: RecordBatch) -> Self {
        Self {
            payload_type,
            current_batch: record_batch,
        }
    }
}

#[async_trait]
impl TableProvider for OtapBatchTable {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn schema(&self) -> SchemaRef {
        self.current_batch.schema()
    }

    fn table_type(&self) -> TableType {
        TableType::Base
    }

    async fn scan(
        &self,
        _state: &dyn Session,
        projection: Option<&Vec<usize>>,
        _filters: &[Expr],
        _limit: Option<usize>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        // println!("projection (table) = {:?}", projection);

        let schema = self.current_batch.schema();
        let data_source = MemorySourceConfig::try_new(
            &[vec![
                // TODO -- validate if it's somehow possible to avoid the clone here
                self.current_batch.clone(),
            ]],
            schema,
            projection.cloned(),
        )?;

        Ok(Arc::new(OtapDataSourceExec::new(
            self.payload_type,
            data_source,
        )))
    }
}
