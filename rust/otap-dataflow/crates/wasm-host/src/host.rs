// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Host state and native kernel implementations.
//!
//! The host owns OTAP records behind a host-managed pdata resource in
//! wasmtime's [`ResourceTable`]. Guests receive only that resource handle and
//! orchestrate kernels that execute natively here.

use arrow::array::{Array, AsArray, BooleanArray, DictionaryArray, StringArray};
use arrow::datatypes::{
    ArrowDictionaryKeyType, ArrowNativeType, DataType, Int8Type, Int16Type, Int32Type, Int64Type,
    UInt8Type, UInt16Type, UInt32Type, UInt64Type,
};
use wasmtime::component::{Resource, ResourceTable};

use crate::bindings::otel::otap_dataflow_plugin::otel_kernels::{self, AttrScope};
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::otap::filter::{IdBitmapPool, filter_otap_batch};

/// Host-owned data behind a host-managed `pdata` resource handle.
///
/// This is the concrete type mapped to the WIT `pdata` resource. Guests never
/// see its contents; they only pass the handle back to host kernels.
pub struct HostPdata {
    /// The OTAP payload that host kernels operate on.
    pub otap_batch: OtapArrowRecords,
}

/// Per-instance host state stored in the wasmtime [`wasmtime::Store`].
///
/// Holds the host-managed pdata resource table. This state is confined to a single
/// pipeline/core thread and is never shared across threads.
pub struct HostState {
    /// Resource table backing the `pdata` resource.
    pub table: ResourceTable,
    /// Reusable bitmap pool for OTAP child-batch filtering propagation.
    pub id_bitmap_pool: IdBitmapPool,
    /// Accumulator for total host kernel invocations during a single guest call.
    /// Drained and added to telemetry counters after each `run_guest` return.
    pub kernel_calls: u64,
}

impl HostState {
    /// Create empty host state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            table: ResourceTable::new(),
            id_bitmap_pool: IdBitmapPool::new(),
            kernel_calls: 0,
        }
    }

    /// Drain the per-call kernel call counter, returning the accumulated count
    /// and resetting to zero.
    pub fn drain_kernel_counters(&mut self) -> u64 {
        let calls = self.kernel_calls;
        self.kernel_calls = 0;
        calls
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
        self.kernel_calls += 1;
        self.table
            .get(&data)
            .expect("invalid wasm host pdata resource handle")
            .otap_batch
            .root_record_batch()
            .map_or(0, |batch| batch.num_rows() as u32)
    }

    fn filter_by_attribute_eq(
        &mut self,
        data: Resource<HostPdata>,
        scope: AttrScope,
        key: String,
        value: String,
    ) -> Resource<HostPdata> {
        self.kernel_calls += 1;
        // Consume the input handle and take ownership of the batch. Invalid
        // handles are a contract violation and should trap instead of silently
        // dropping data.
        let input = self
            .table
            .delete(data)
            .expect("invalid wasm host pdata resource handle")
            .otap_batch;

        let result = match scope {
            AttrScope::Resource | AttrScope::Scope => {
                panic!(
                    "unsupported attr scope {scope:?}: this experimental slice currently supports only record scope"
                )
            }
            AttrScope::Record => filter_otap_batch_by_column_eq(
                &input,
                &key,
                &value,
                &mut self.id_bitmap_pool,
            )
            .unwrap_or_else(|error| {
                panic!("filter-by-attribute-eq failed for key {key:?} and value {value:?}: {error}")
            }),
        };

        self.table
            .push(HostPdata { otap_batch: result })
            .expect("resource table push")
    }
}

/// Native OTel-semantic filter kernel: keep rows whose `key` column equals
/// `value` (string comparison).
///
/// Handles plain `Utf8`, `LargeUtf8`, and dictionary-encoded string columns by
/// using a dictionary-aware comparison fast path and falling back to `Utf8`
/// casting for other encodings.
///
fn filter_otap_batch_by_column_eq(
    otap_batch: &OtapArrowRecords,
    key: &str,
    value: &str,
    id_bitmap_pool: &mut IdBitmapPool,
) -> Result<OtapArrowRecords, String> {
    let Some(root_batch) = otap_batch.root_record_batch() else {
        return Err("root record batch not present for filtering".to_string());
    };

    let Some(column) = root_batch.column_by_name(key) else {
        return Err(format!(
            "attribute column {key:?} not present in root record batch"
        ));
    };

    let mask = if let Some(mask) = dictionary_string_eq_mask(column.as_ref(), value)? {
        mask
    } else {
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
        match arrow::compute::kernels::cmp::eq(&utf8, &scalar) {
            Ok(mask) => mask,
            Err(error) => {
                return Err(format!(
                    "failed to compare attribute column {key:?} against value {value:?}: {error}"
                ));
            }
        }
    };

    filter_otap_batch(&mask, otap_batch, id_bitmap_pool).map_err(|error| {
        format!("failed to filter OTAP payload for key {key:?} and value {value:?}: {error}")
    })
}

pub(crate) fn dictionary_string_eq_mask(
    column: &dyn Array,
    value: &str,
) -> Result<Option<BooleanArray>, String> {
    let DataType::Dictionary(key_type, value_type) = column.data_type() else {
        return Ok(None);
    };

    if !matches!(**value_type, DataType::Utf8 | DataType::LargeUtf8) {
        return Ok(None);
    }

    macro_rules! dispatch_key_type {
        ($key_ty:ty) => {{
            let dict = column
                .as_any()
                .downcast_ref::<DictionaryArray<$key_ty>>()
                .ok_or_else(|| "failed to downcast dictionary column".to_string())?;
            Ok(Some(dictionary_eq_mask_impl(dict, value)))
        }};
    }

    match key_type.as_ref() {
        DataType::Int8 => dispatch_key_type!(Int8Type),
        DataType::Int16 => dispatch_key_type!(Int16Type),
        DataType::Int32 => dispatch_key_type!(Int32Type),
        DataType::Int64 => dispatch_key_type!(Int64Type),
        DataType::UInt8 => dispatch_key_type!(UInt8Type),
        DataType::UInt16 => dispatch_key_type!(UInt16Type),
        DataType::UInt32 => dispatch_key_type!(UInt32Type),
        DataType::UInt64 => dispatch_key_type!(UInt64Type),
        _ => Ok(None),
    }
}

fn dictionary_eq_mask_impl<K: ArrowDictionaryKeyType>(
    dict: &DictionaryArray<K>,
    value: &str,
) -> BooleanArray
where
    K::Native: ArrowNativeType,
{
    let keys = dict.keys();
    let mut matches = Vec::with_capacity(dict.len());
    match dict.values().data_type() {
        DataType::Utf8 => {
            let values = dict.values().as_string::<i32>();
            for i in 0..dict.len() {
                if keys.is_null(i) {
                    matches.push(false);
                    continue;
                }
                let key_index = keys.value(i).as_usize();
                matches.push(!values.is_null(key_index) && values.value(key_index) == value);
            }
        }
        DataType::LargeUtf8 => {
            let values = dict.values().as_string::<i64>();
            for i in 0..dict.len() {
                if keys.is_null(i) {
                    matches.push(false);
                    continue;
                }
                let key_index = keys.value(i).as_usize();
                matches.push(!values.is_null(key_index) && values.value(key_index) == value);
            }
        }
        _ => {
            matches.resize(dict.len(), false);
        }
    }
    BooleanArray::from(matches)
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{Array, DictionaryArray, RecordBatch, StringArray, UInt16Array};
    use arrow::datatypes::{Field, Schema, UInt8Type};
    use otap_df_pdata::otap::Logs;
    use otap_df_pdata::proto::OtlpProtoMessage;
    use otap_df_pdata::proto::opentelemetry::common::v1::{AnyValue, KeyValue};
    use otap_df_pdata::proto::opentelemetry::logs::v1::LogRecord;
    use otap_df_pdata::testing::round_trip::{otap_to_otlp, to_otap_logs};
    use std::sync::Arc;

    fn batch_with_severity(values: &[&str]) -> OtapArrowRecords {
        let schema = Schema::new(vec![
            Field::new("id", DataType::UInt16, true),
            Field::new("severity_text", DataType::Utf8, true),
        ]);
        let record_batch = RecordBatch::try_new(
            Arc::new(schema),
            vec![
                Arc::new(UInt16Array::from(
                    (0..values.len() as u16).collect::<Vec<_>>(),
                )),
                Arc::new(StringArray::from(values.to_vec())),
            ],
        )
        .unwrap();
        let mut otap = OtapArrowRecords::Logs(Logs::default());
        otap.set(
            otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType::Logs,
            record_batch,
        )
        .expect("set logs root batch");
        otap
    }

    fn severity_values(otap_batch: &OtapArrowRecords) -> Vec<String> {
        let col = otap_batch
            .root_record_batch()
            .expect("root batch present")
            .column_by_name("severity_text")
            .unwrap();
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
        let mut pool = IdBitmapPool::new();
        let out = filter_otap_batch_by_column_eq(&batch, "severity_text", "ERROR", &mut pool)
            .expect("filter should succeed");
        assert_eq!(out.root_record_batch().expect("root batch").num_rows(), 2);
        assert_eq!(severity_values(&out), vec!["ERROR", "ERROR"]);
    }

    /// Scenario: Record-scope filtering references an attribute key that does
    /// not exist in the root record batch.
    /// Guarantees: The kernel reports an explicit error instead of silently
    /// passing data through unchanged.
    #[test]
    fn missing_column_is_error() {
        let batch = batch_with_severity(&["ERROR", "INFO"]);
        let mut pool = IdBitmapPool::new();
        let result = filter_otap_batch_by_column_eq(&batch, "does_not_exist", "ERROR", &mut pool);
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
        let mut otap = OtapArrowRecords::Logs(Logs::default());
        otap.set(
            otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType::Logs,
            batch,
        )
        .expect("set logs root batch");
        let mut pool = IdBitmapPool::new();
        let out = filter_otap_batch_by_column_eq(&otap, "severity_text", "ERROR", &mut pool)
            .expect("filter should succeed");
        assert_eq!(out.root_record_batch().expect("root batch").num_rows(), 2);
    }

    /// Scenario: Record-scope filtering receives a dictionary-encoded string
    /// column with null keys and null dictionary values.
    /// Guarantees: Null keys and null dictionary values do not match the target
    /// value and are excluded from filtered results.
    #[test]
    fn dictionary_encoded_nulls_do_not_match() {
        let dict: DictionaryArray<UInt8Type> =
            vec![Some("ERROR"), None, Some("INFO"), Some("ERROR"), None]
                .into_iter()
                .collect();
        let schema = Schema::new(vec![Field::new(
            "severity_text",
            dict.data_type().clone(),
            true,
        )]);
        let batch = RecordBatch::try_new(Arc::new(schema), vec![Arc::new(dict)]).unwrap();
        let mut otap = OtapArrowRecords::Logs(Logs::default());
        otap.set(
            otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType::Logs,
            batch,
        )
        .expect("set logs root batch");
        let mut pool = IdBitmapPool::new();
        let out = filter_otap_batch_by_column_eq(&otap, "severity_text", "ERROR", &mut pool)
            .expect("filter should succeed");
        assert_eq!(out.root_record_batch().expect("root batch").num_rows(), 2);
        assert_eq!(severity_values(&out), vec!["ERROR", "ERROR"]);
    }

    /// Scenario: Root log records are filtered from a payload that includes
    /// per-record log attributes.
    /// Guarantees: Child log attribute rows are filtered with the same parent
    /// selection, so only attributes of surviving records remain.
    #[test]
    fn filtering_preserves_log_attribute_relationships() {
        let input = to_otap_logs(vec![
            LogRecord::build()
                .severity_text("ERROR")
                .attributes(vec![KeyValue::new("k", AnyValue::new_string("e0"))])
                .finish(),
            LogRecord::build()
                .severity_text("INFO")
                .attributes(vec![KeyValue::new("k", AnyValue::new_string("i1"))])
                .finish(),
            LogRecord::build()
                .severity_text("ERROR")
                .attributes(vec![KeyValue::new("k", AnyValue::new_string("e2"))])
                .finish(),
        ]);

        let mut pool = IdBitmapPool::new();
        let output =
            filter_otap_batch_by_column_eq(&input, "severity_text", "ERROR", &mut pool).unwrap();
        let OtlpProtoMessage::Logs(logs) = otap_to_otlp(&output) else {
            panic!("expected logs payload");
        };

        let records = &logs.resource_logs[0].scope_logs[0].log_records;
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].severity_text, "ERROR");
        assert_eq!(records[1].severity_text, "ERROR");
        let attr_values: Vec<String> =
            records
                .iter()
                .map(|record| {
                    record.attributes[0]
                        .value
                        .as_ref()
                        .expect("attribute value")
                })
                .map(|value| {
                    match value.value.as_ref().expect("typed attribute value") {
                otap_df_pdata::proto::opentelemetry::common::v1::any_value::Value::StringValue(
                    s,
                ) => s.clone(),
                other => panic!("expected string attribute value, got {other:?}"),
            }
                })
                .collect();
        assert_eq!(attr_values, vec!["e0", "e2"]);
    }

    /// Scenario: Guest calls `pdata-num-rows` or `filter-by-attribute-eq`
    /// with record scope.
    /// Guarantees: Each kernel invocation increments `kernel_calls` by one.
    #[test]
    fn kernel_calls_incremented_per_invocation() {
        let mut host = HostState::new();
        let batch = batch_with_severity(&["ERROR", "INFO", "WARN"]);

        // pdata_num_rows counts as one kernel call.
        let h0 = host
            .table
            .push(HostPdata {
                otap_batch: batch.clone(),
            })
            .expect("push batch");
        let _ = <HostState as otel_kernels::Host>::pdata_num_rows(&mut host, h0);
        assert_eq!(host.kernel_calls, 1);

        // filter_by_attribute_eq (record scope) also counts as one kernel call.
        let h1 = host
            .table
            .push(HostPdata { otap_batch: batch })
            .expect("push batch");
        let out_handle = <HostState as otel_kernels::Host>::filter_by_attribute_eq(
            &mut host,
            h1,
            AttrScope::Record,
            "severity_text".to_string(),
            "ERROR".to_string(),
        );
        let _ = host.table.delete(out_handle).expect("delete output handle");
        assert_eq!(host.kernel_calls, 2);
    }

    /// Scenario: `drain_kernel_counters` is called after accumulating counts.
    /// Guarantees: The returned value equals the accumulated count and the
    /// accumulator is reset to zero for the next call.
    #[test]
    fn drain_kernel_counters_resets_to_zero() {
        let mut host = HostState::new();
        host.kernel_calls = 5;

        let calls = host.drain_kernel_counters();
        assert_eq!(calls, 5);
        assert_eq!(host.kernel_calls, 0, "kernel_calls must reset after drain");
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
                otap_batch: batch_with_severity(&["ERROR", "INFO", "WARN"]),
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

    /// Scenario: `HostState::default()` is used to construct initial state.
    /// Guarantees: The default construction produces the same empty state as
    /// `HostState::new()` -- zero kernel counters and an empty resource table.
    #[test]
    fn host_state_default_matches_new() {
        let from_default = HostState::default();
        assert_eq!(from_default.kernel_calls, 0);
    }

    /// Scenario: A dictionary-encoded column uses `LargeUtf8` values rather
    /// than the more common `Utf8`.
    /// Guarantees: The `LargeUtf8` branch in `dictionary_eq_mask_impl` is
    /// exercised and correctly identifies matching rows.
    #[test]
    fn handles_large_utf8_dictionary_values() {
        use arrow::array::{Int8Array, LargeStringArray};

        let keys = Int8Array::from(vec![0i8, 1, 0, 2]);
        let values = Arc::new(LargeStringArray::from(vec!["ERROR", "INFO", "WARN"]));
        let dict =
            DictionaryArray::try_new(keys, values as Arc<dyn Array>).expect("build LargeUtf8 dict");
        let mask = dictionary_string_eq_mask(&dict, "ERROR")
            .expect("LargeUtf8 dict mask should succeed")
            .expect("LargeUtf8 dict column should produce a mask");
        let kept: Vec<bool> = (0..mask.len()).map(|i| mask.value(i)).collect();
        assert_eq!(kept, vec![true, false, true, false]);
    }

    /// Scenario: A dictionary-encoded column uses `UInt32` integer keys.
    /// Guarantees: The `UInt32` dispatch arm in `dictionary_string_eq_mask` is
    /// exercised and matching rows are correctly identified.
    #[test]
    fn handles_uint32_key_dictionary() {
        use arrow::datatypes::UInt32Type;

        let dict: DictionaryArray<UInt32Type> =
            vec!["ERROR", "INFO", "ERROR", "WARN"].into_iter().collect();
        let mask = dictionary_string_eq_mask(&dict, "ERROR")
            .expect("UInt32-key dict mask should succeed")
            .expect("UInt32-key dict column should produce a mask");
        let kept: Vec<bool> = (0..mask.len()).map(|i| mask.value(i)).collect();
        assert_eq!(kept, vec![true, false, true, false]);
    }
}
