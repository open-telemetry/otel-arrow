use async_trait::async_trait;
use datafusion::arrow::array::{Int32Array, StringArray};
use datafusion::arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::catalog::{MemTable, Session};
use datafusion::common::Result;
use datafusion::datasource::{TableProvider, TableType};
use datafusion::execution::context::{SessionContext, SessionState};
use datafusion::logical_expr::Expr;
use datafusion::physical_plan::ExecutionPlan;
use datafusion::prelude::*;
use std::sync::{Arc, RwLock};

// ---------------------
// MutableMemTable
// ---------------------
#[derive(Debug)]
pub struct MutableMemTable {
    schema: SchemaRef,
    data: Arc<RwLock<Vec<RecordBatch>>>,
}

impl MutableMemTable {
    pub fn new(schema: SchemaRef) -> Self {
        Self {
            schema,
            data: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn replace_batches(&self, new_batches: Vec<RecordBatch>) {
        let mut guard = self.data.write().unwrap();
        *guard = new_batches;
    }
}

#[async_trait]
impl TableProvider for MutableMemTable {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn schema(&self) -> SchemaRef {
        self.schema.clone()
    }

    fn table_type(&self) -> TableType {
        TableType::Base
    }

    async fn scan(
        &self,
        _state: &dyn Session,
        _projection: Option<&Vec<usize>>,
        _filters: &[Expr],
        _limit: Option<usize>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        let batches = self.data.read().unwrap().clone();
        // Ok(Arc::new(MemoryE::try_new(&[batches], self.schema.clone(), None)?))
        let memtable = MemTable::try_new(self.schema.clone(), vec![batches])?;
        // MemTable implements TableProvider, so we can just return it as an ExecutionPlan
        println!("here");
        memtable.scan(_state, _projection, _filters, _limit).await
    }
}
