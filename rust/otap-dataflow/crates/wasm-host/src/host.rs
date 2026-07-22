// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Host state and native kernel implementations.
//!
//! The host owns the Arrow [`RecordBatch`] behind a host-managed pdata
//! resource in wasmtime's [`ResourceTable`]. Guests receive only that resource
//! handle and orchestrate kernels that execute natively here.

use arrow::array::{RecordBatch, StringArray};
use arrow::datatypes::DataType;
use wasmtime::component::{Resource, ResourceTable};

use crate::bindings::otel::otap_dataflow_plugin::otel_kernels::{self, AttrScope};

/// Host-owned data behind a host-managed `pdata` resource handle.
///
/// This is the concrete type mapped to the WIT `pdata` resource. Guests never
/// see its contents; they only pass the handle back to host kernels.
pub struct HostPdata {
    /// The root Arrow record batch the kernels operate on.
    pub record_batch: RecordBatch,
}

/// Per-instance host state stored in the wasmtime [`wasmtime::Store`].
///
/// Holds the host-managed pdata resource table. This state is confined to a single
/// pipeline/core thread and is never shared across threads.
pub struct HostState {
    /// Resource table backing the `pdata` resource.
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

impl otel_kernels::HostPdata for HostState {
    fn drop(&mut self, data: Resource<HostPdata>) -> wasmtime::Result<()> {
        let _ = self.table.delete(data)?;
        Ok(())
    }
}

impl otel_kernels::Host for HostState {
    fn pdata_num_rows(&mut self, data: Resource<HostPdata>) -> u32 {
        self.table
            .get(&data)
            .expect("invalid wasm host pdata resource handle")
            .record_batch
            .num_rows() as u32
    }

    fn filter_by_attribute_eq(
        &mut self,
        data: Resource<HostPdata>,
        scope: AttrScope,
        key: String,
        value: String,
    ) -> Resource<HostPdata> {
        // Read the input batch, consume the input handle, and return a fresh
        // handle for the result. Invalid handles are a contract violation and
        // should trap instead of silently dropping data.
        let input = self
            .table
            .get(&data)
            .expect("invalid wasm host pdata resource handle")
            .record_batch
            .clone();
        let _ = self
            .table
            .delete(data)
            .expect("invalid wasm host pdata resource handle");

        let result = match scope {
            AttrScope::Resource | AttrScope::Scope => panic!(
                "unsupported attr scope {scope:?}: this experimental slice currently supports only record scope"
            ),
            AttrScope::Record => filter_record_batch_by_column_eq(&input, &key, &value)
                .unwrap_or_else(|error| {
                    panic!(
                        "filter-by-attribute-eq failed for key {key:?} and value {value:?}: {error}"
                    )
                }),
        };

        self.table
            .push(HostPdata {
                record_batch: result,
            })
            .expect("resource table push")
    }
}

/// Native OTel-semantic filter kernel: keep rows whose `key` column equals
/// `value` (string comparison).
///
/// Handles plain `Utf8`, `LargeUtf8`, and dictionary-encoded string columns by
/// casting to `Utf8` before comparison.
///
/// TODO: filtering the root record batch does not yet reindex child
/// attribute record batches; the reference plugin operates on batches without
/// child payloads.
fn filter_record_batch_by_column_eq(
    batch: &RecordBatch,
    key: &str,
    value: &str,
) -> Result<RecordBatch, String> {
    let Some(column) = batch.column_by_name(key) else {
        return Err(format!(
            "attribute column {key:?} not present in root record batch"
        ));
    };

    let utf8 = if column.data_type() == &DataType::Utf8 {
        column.clone()
    } else {
        match arrow_cast::cast(column, &DataType::Utf8) {
            Ok(arr) => arr,
            Err(error) => {
                return Err(format!(
                    "failed to cast attribute column {key:?} to Utf8 for comparison: {error}"
                ));
            }
        }
    };

    let scalar = StringArray::new_scalar(value);
    let mask = match arrow::compute::kernels::cmp::eq(&utf8, &scalar) {
        Ok(mask) => mask,
        Err(error) => {
            return Err(format!(
                "failed to compare attribute column {key:?} against value {value:?}: {error}"
            ));
        }
    };

    match arrow_select::filter::filter_record_batch(batch, &mask) {
        Ok(filtered) => Ok(filtered),
        Err(error) => Err(format!(
            "failed to filter record batch for key {key:?} and value {value:?}: {error}"
        )),
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

    /// Scenario: Record-scope filtering receives matching and non-matching
    /// severity values.
    /// Guarantees: Only rows matching `severity_text == "ERROR"` are retained.
    #[test]
    fn filters_matching_rows() {
        let batch = batch_with_severity(&["ERROR", "INFO", "ERROR", "WARN"]);
        let out = filter_record_batch_by_column_eq(&batch, "severity_text", "ERROR")
            .expect("filter should succeed");
        assert_eq!(out.num_rows(), 2);
        assert_eq!(severity_values(&out), vec!["ERROR", "ERROR"]);
    }

    /// Scenario: Record-scope filtering references an attribute key that does
    /// not exist in the root record batch.
    /// Guarantees: The kernel reports an explicit error instead of silently
    /// passing data through unchanged.
    #[test]
    fn missing_column_is_error() {
        let batch = batch_with_severity(&["ERROR", "INFO"]);
        let result = filter_record_batch_by_column_eq(&batch, "does_not_exist", "ERROR");
        assert!(
            result.is_err(),
            "missing attribute key should be reported explicitly"
        );
    }

    /// Scenario: Record-scope filtering is invoked on a dictionary-encoded
    /// `severity_text` column.
    /// Guarantees: The kernel can cast dictionary-encoded values and keep only
    /// matching rows.
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
        let out = filter_record_batch_by_column_eq(&batch, "severity_text", "ERROR")
            .expect("filter should succeed");
        assert_eq!(out.num_rows(), 2);
    }

    /// Scenario: Guest requests `resource` or `scope` filtering in the current
    /// experimental vertical slice.
    /// Guarantees: Unsupported scopes trap immediately instead of silently
    /// passing data through.
    #[test]
    #[should_panic(expected = "unsupported attr scope")]
    fn resource_and_scope_filter_traps() {
        let mut host = HostState::new();
        let handle = host
            .table
            .push(HostPdata {
                record_batch: batch_with_severity(&["ERROR", "INFO", "WARN"]),
            })
            .expect("push input batch");

        let _ = <HostState as otel_kernels::Host>::filter_by_attribute_eq(
            &mut host,
            handle,
            AttrScope::Resource,
            "severity_text".to_string(),
            "ERROR".to_string(),
        );
    }

    /// Scenario: Guest passes an invalid pdata resource handle to
    /// `pdata-num-rows`.
    /// Guarantees: Invalid resource handles trap instead of being interpreted
    /// as empty data.
    #[test]
    #[should_panic(expected = "invalid wasm host pdata resource handle")]
    fn invalid_handle_for_pdata_num_rows_traps() {
        let mut host = HostState::new();
        let invalid = Resource::<HostPdata>::new_own(u32::MAX);
        let _ = <HostState as otel_kernels::Host>::pdata_num_rows(&mut host, invalid);
    }

    /// Scenario: Guest passes an invalid pdata resource handle to
    /// `filter-by-attribute-eq`.
    /// Guarantees: Invalid resource handles trap instead of returning fabricated
    /// filtered output.
    #[test]
    #[should_panic(expected = "invalid wasm host pdata resource handle")]
    fn invalid_handle_for_filter_traps() {
        let mut host = HostState::new();
        let invalid = Resource::<HostPdata>::new_own(u32::MAX);
        let _ = <HostState as otel_kernels::Host>::filter_by_attribute_eq(
            &mut host,
            invalid,
            AttrScope::Record,
            "severity_text".to_string(),
            "ERROR".to_string(),
        );
    }
}
