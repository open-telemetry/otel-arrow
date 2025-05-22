use arrow::array::RecordBatch;
use std::sync::Arc;

/// schema constants
pub mod consts {
    /// keys for arrow schema/field metadata
    pub mod metadata {
        /// schema metadata for which columns the record batch is sorted by
        pub const SORT_COLUMNS: &str = "sort_columns";
    }
}

/// Insert the new key/value pair into the record batch
pub fn insert_schema_metadata(
    record_batch: RecordBatch,
    key: String,
    value: String,
) -> RecordBatch {
    let schema = record_batch.schema_ref();
    let mut schema_metadata = schema.metadata.clone();
    let _ = schema_metadata.insert(key, value);

    let new_schema = schema.as_ref().clone().with_metadata(schema_metadata);

    // TODO expect, safety, etc
    record_batch.with_schema(Arc::new(new_schema)).unwrap()
}
