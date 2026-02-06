use std::sync::Arc;

use arrow::array::{ArrayRef, RecordBatch, StructArray};
use arrow_schema::SchemaRef;

use crate::schema::consts::ID;

use super::transport_optimize::{access_column, struct_column_name};

struct RecordBatchEditor {
    schema: SchemaRef,
    columns: Vec<ArrayRef>,
}

impl RecordBatchEditor {
    pub fn access_column(&self, column: &str) -> Option<ArrayRef> {
        access_column(column, &self.schema, &self.columns)
    }

    pub fn get_col_index(&self, column: &str) -> Option<usize> {
        self.schema.fields.find(column).map(|(idx, _)| idx)
    }

    pub fn replace_col_index(&mut self, idx: usize, new_column: ArrayRef) {
        self.columns[idx] = new_column;
    }

    pub fn replace_column(&mut self, column: &str, new_column: ArrayRef) -> Result<(), ArrayRef> {
        if let Some(struct_col_name) = struct_column_name(column) {
            let struct_col_idx = self
                .schema
                .index_of(struct_col_name)
                .ok()
                .ok_or(new_column.clone())?;
            let struct_col = self
                .columns
                .get(struct_col_idx)
                .ok_or(new_column.clone())?
                .as_any()
                .downcast_ref::<StructArray>()
                .ok_or(new_column.clone())?;
        }

        let Some(idx) = self.get_col_index(column) else {
            return Err(new_column);
        };

        self.columns[idx] = new_column;
        Ok(())
    }

    pub fn into_record_batch(self) -> arrow::error::Result<RecordBatch> {
        let rb = RecordBatch::try_new(self.schema, self.columns)?;
        Ok(rb)
    }
}

impl From<RecordBatch> for RecordBatchEditor {
    fn from(rb: RecordBatch) -> Self {
        let (schema, columns, _) = rb.into_parts();
        Self { schema, columns }
    }
}
