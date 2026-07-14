// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Host state and native kernel implementations.
//!
//! The host owns the Arrow [`RecordBatch`] behind an opaque handle managed by
//! wasmtime's [`ResourceTable`]. Guests receive only the handle and orchestrate
//! kernels that execute natively here.

use arrow::array::{RecordBatch, StringArray};
use arrow::datatypes::DataType;
use wasmtime::component::{Resource, ResourceTable};

use crate::bindings::otel::otap_dataflow_plugin::otel_kernels::{self, AttrScope};

/// Host-owned data behind an opaque `batch` handle.
///
/// This is the concrete type mapped to the WIT `batch` resource. Guests never
/// see its contents; they only pass the handle back to host kernels.
pub struct HostBatchData {
    /// The root Arrow record batch the kernels operate on.
    pub record_batch: RecordBatch,
}

/// Per-instance host state stored in the wasmtime [`wasmtime::Store`].
///
/// Holds the opaque-handle table. This state is confined to a single
/// pipeline/core thread and is never shared across threads.
pub struct HostState {
    /// Opaque handle table backing the `batch` resource.
    pub table: ResourceTable,
}

impl HostState {
    /// Create empty host state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            table: ResourceTable::new(),
        }
    }
}

impl Default for HostState {
    fn default() -> Self {
        Self::new()
    }
}

impl otel_kernels::HostBatch for HostState {
    fn drop(&mut self, b: Resource<HostBatchData>) -> wasmtime::Result<()> {
        let _ = self.table.delete(b)?;
        Ok(())
    }
}

impl otel_kernels::Host for HostState {
    fn batch_num_rows(&mut self, b: Resource<HostBatchData>) -> u32 {
        self.table
            .get(&b)
            .map(|d| d.record_batch.num_rows())
            .unwrap_or(0) as u32
    }

    fn filter_by_attribute_eq(
        &mut self,
        b: Resource<HostBatchData>,
        scope: AttrScope,
        key: String,
        value: String,
    ) -> Resource<HostBatchData> {
        // Read the input batch, consume the input handle, and return a fresh
        // handle for the result. Invalid handles are a contract violation and
        // should trap instead of silently dropping data.
        let input = self
            .table
            .get(&b)
            .expect("invalid wasm host batch handle")
            .record_batch
            .clone();
        let _ = self.table.delete(b).expect("invalid wasm host batch handle");

        let result = match scope {
            // TODO: implement `resource`/`scope` attribute scopes and
            // dedicated attribute record batches. For now these pass through.
            AttrScope::Resource | AttrScope::Scope => input,
            AttrScope::Record => filter_record_batch_by_column_eq(&input, &key, &value),
        };

        self.table
            .push(HostBatchData {
                record_batch: result,
            })
            .expect("resource table push")
    }
}

/// Native OTel-semantic filter kernel: keep rows whose `key` column equals
/// `value` (string comparison).
///
/// Handles plain `Utf8`, `LargeUtf8`, and dictionary-encoded string columns by
/// casting to `Utf8` before comparison. If the column is missing or cannot be
/// compared, the batch is returned unchanged (a safe no-op).
///
/// TODO: filtering the root record batch does not yet reindex child
/// attribute record batches; the reference plugin operates on batches without
/// child payloads.
fn filter_record_batch_by_column_eq(batch: &RecordBatch, key: &str, value: &str) -> RecordBatch {
    let Some(column) = batch.column_by_name(key) else {
        return batch.clone();
    };

    let utf8 = if column.data_type() == &DataType::Utf8 {
        column.clone()
    } else {
        match arrow_cast::cast(column, &DataType::Utf8) {
            Ok(arr) => arr,
            Err(_) => return batch.clone(),
        }
    };

    let scalar = StringArray::new_scalar(value);
    let mask = match arrow::compute::kernels::cmp::eq(&utf8, &scalar) {
        Ok(mask) => mask,
        Err(_) => return batch.clone(),
    };

    match arrow_select::filter::filter_record_batch(batch, &mask) {
        Ok(filtered) => filtered,
        Err(_) => batch.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{Array, DictionaryArray, StringArray, UInt16Array};
    use arrow::datatypes::{Field, Schema, UInt8Type};
    use std::sync::Arc;

    fn batch_with_severity(values: &[&str]) -> RecordBatch {
        let schema = Schema::new(vec![
            Field::new("id", DataType::UInt16, true),
            Field::new("severity_text", DataType::Utf8, true),
        ]);
        RecordBatch::try_new(
            Arc::new(schema),
            vec![
                Arc::new(UInt16Array::from(
                    (0..values.len() as u16).collect::<Vec<_>>(),
                )),
                Arc::new(StringArray::from(values.to_vec())),
            ],
        )
        .unwrap()
    }

    fn severity_values(batch: &RecordBatch) -> Vec<String> {
        let col = batch.column_by_name("severity_text").unwrap();
        let arr = arrow_cast::cast(col, &DataType::Utf8).unwrap();
        let strings = arr.as_any().downcast_ref::<StringArray>().unwrap();
        (0..strings.len())
            .map(|i| strings.value(i).to_string())
            .collect()
    }

    #[test]
    fn filters_matching_rows() {
        let batch = batch_with_severity(&["ERROR", "INFO", "ERROR", "WARN"]);
        let out = filter_record_batch_by_column_eq(&batch, "severity_text", "ERROR");
        assert_eq!(out.num_rows(), 2);
        assert_eq!(severity_values(&out), vec!["ERROR", "ERROR"]);
    }

    #[test]
    fn missing_column_is_passthrough() {
        let batch = batch_with_severity(&["ERROR", "INFO"]);
        let out = filter_record_batch_by_column_eq(&batch, "does_not_exist", "ERROR");
        assert_eq!(out.num_rows(), 2);
    }

    #[test]
    fn handles_dictionary_encoded_columns() {
        // OTAP severity_text is typically dictionary-encoded; the kernel must
        // still compare correctly after casting to Utf8.
        let dict: DictionaryArray<UInt8Type> = vec!["ERROR", "INFO", "ERROR"].into_iter().collect();
        let schema = Schema::new(vec![Field::new(
            "severity_text",
            dict.data_type().clone(),
            true,
        )]);
        let batch = RecordBatch::try_new(Arc::new(schema), vec![Arc::new(dict)]).unwrap();
        let out = filter_record_batch_by_column_eq(&batch, "severity_text", "ERROR");
        assert_eq!(out.num_rows(), 2);
    }
}
