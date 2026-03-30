// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Utilities for sanitizing arrow arrays.
//!
//! This procedure involves removing any unreferenced data in any arrow array, specifically:
//! - removing any dictionary values that have no keys pointing to them

use std::borrow::Cow;
use std::sync::Arc;

use arrow::{
    array::{Array, BooleanArray, DictionaryArray, PrimitiveArray, RecordBatch, StructArray},
    buffer::ScalarBuffer,
    compute::kernels::filter::filter,
    datatypes::{ArrowDictionaryKeyType, ArrowNativeType, UInt8Type, UInt16Type},
    util::{bit_iterator::BitSliceIterator, bit_util},
};
use arrow_schema::DataType;

/// Sanitizes a single array column, returning `Some` with the sanitized array if any changes
/// were made, or `None` if the array was already clean.
fn sanitize_column(column: &dyn Array) -> Option<Arc<dyn Array>> {
    match column.data_type() {
        DataType::Dictionary(k, _) => match k.as_ref() {
            DataType::UInt8 => {
                let dict_arr = column
                    .as_any()
                    .downcast_ref::<DictionaryArray<UInt8Type>>()
                    // safety: we've checked the type
                    .expect("can downcast to dict");
                sanitized_dict(dict_arr).map(|s| Arc::new(s) as Arc<dyn Array>)
            }
            DataType::UInt16 => {
                let dict_arr = column
                    .as_any()
                    .downcast_ref::<DictionaryArray<UInt16Type>>()
                    // safety: we've checked the type
                    .expect("can downcast to dict");
                sanitized_dict(dict_arr).map(|s| Arc::new(s) as Arc<dyn Array>)
            }
            _ => None,
        },
        DataType::Struct(_) => {
            let struct_arr = column
                .as_any()
                .downcast_ref::<StructArray>()
                .expect("can downcast to struct");
            sanitize_struct(struct_arr).map(|s| Arc::new(s) as Arc<dyn Array>)
        }
        _ => None,
    }
}

/// sanitize all columns in the
#[must_use]
pub fn sanitize_record_batch(record_batch: &RecordBatch) -> Option<RecordBatch> {
    let mut columns = Cow::from(record_batch.columns());

    for i in 0..record_batch.num_columns() {
        if let Some(sanitized) = sanitize_column(record_batch.column(i)) {
            columns.to_mut()[i] = sanitized;
        }
    }

    match columns {
        // no modifications were made:
        Cow::Borrowed(_) => None,

        // create new batch with sanitized columns
        Cow::Owned(new_columns) => {
            // safety: we haven't changed the length/type of any column, nor the column order
            // so it is safe to expect that try_new will not error here
            let sanitized_batch = RecordBatch::try_new(record_batch.schema(), new_columns)
                .expect("can create sanitized batch");
            Some(sanitized_batch)
        }
    }
}

/// Sanitizes a struct array by removing orphaned dictionary values from any dictionary fields.
fn sanitize_struct(struct_arr: &StructArray) -> Option<StructArray> {
    let mut columns: Cow<'_, [Arc<dyn Array>]> = Cow::Borrowed(struct_arr.columns());

    for i in 0..struct_arr.num_columns() {
        if let Some(sanitized) = sanitize_column(struct_arr.column(i)) {
            columns.to_mut()[i] = sanitized;
        }
    }

    match columns {
        Cow::Borrowed(_) => None,
        Cow::Owned(new_columns) => {
            let fields = struct_arr.fields().clone();
            let nulls = struct_arr.nulls().cloned();
            Some(StructArray::new(fields, new_columns, nulls))
        }
    }
}

/// Ensures that the values inside a dictionary with no keys pointing to them are removed.
fn sanitized_dict<K: ArrowDictionaryKeyType>(
    dict_arr: &DictionaryArray<K>,
) -> Option<DictionaryArray<K>>
where
    K::Native: SanitizeDictHelper,
{
    let dict_values = dict_arr.values();
    let dict_values_len = dict_values.len();
    let dict_keys = dict_arr.keys();

    // first determine which dictionary values are live (e.g. those with keys that reference them)
    let mut live_values_set = vec![0u8; bit_util::ceil(dict_values_len, 8)];
    let mut live_values_count = 0;

    // helper closure to set a value in the live values set. It also returns `true` if this
    // function can return early because it has determined that all dict values are live
    let mut set_value_live = |i: usize| {
        if !bit_util::get_bit(&live_values_set, i) {
            bit_util::set_bit(&mut live_values_set, i);
            live_values_count += 1;
            if live_values_count >= dict_values_len {
                return true;
            }
        }
        false
    };

    // go through the valid (non-null) ranges from the dictionary keys, marking which values are
    // live while trying to return early if all are live ...
    let dict_key_values_buffer = dict_keys.values();
    if let Some(nulls) = dict_keys.nulls() {
        for (start, end) in nulls.valid_slices() {
            for i in start..end {
                let all_values_live = set_value_live(dict_key_values_buffer[i].as_usize());
                if all_values_live {
                    return None;
                }
            }
        }
    } else {
        for i in 0..dict_keys.len() {
            let all_values_live = set_value_live(dict_key_values_buffer[i].as_usize());
            if all_values_live {
                return None;
            }
        }
    }

    // build dictionary key remapping
    let mut remapped_keys = vec![0; dict_values.len()];
    let mut rows_removed = 0;
    let mut last_valid_range_end = 0;
    for (start, end) in BitSliceIterator::new(&live_values_set, 0, dict_values_len) {
        if last_valid_range_end < start {
            rows_removed += start - last_valid_range_end;
        }

        for (i, remapped_key) in remapped_keys.iter_mut().enumerate().take(end).skip(start) {
            *remapped_key = i - rows_removed
        }
        last_valid_range_end = end;
    }

    // build remapped dictionary keys
    let mut new_keys: Vec<K::Native> = vec![K::Native::default(); dict_keys.len()];
    for (i, dict_key) in dict_keys.iter().enumerate() {
        if let Some(key) = dict_key {
            let new_key = remapped_keys[key.as_usize()];
            new_keys[i] = <K::Native as SanitizeDictHelper>::from_usize(new_key);
        }
    }
    let new_keys_values = ScalarBuffer::from(new_keys);
    let new_keys = PrimitiveArray::<K>::new(new_keys_values, dict_keys.nulls().cloned());

    // take only the live values
    let new_values = filter(
        dict_values,
        &BooleanArray::new_from_packed(live_values_set, 0, dict_values_len),
    )
    // safety: this will only return error here if our selection vec is the wrong length, but since
    // we're explicitly passing a boolean array with the same length as the array being filtered,
    // this can be considered safe
    .expect("can filter");

    Some(DictionaryArray::new(new_keys, new_values))
}

// helper trait for making sanitize_dict generic over supported key types
trait SanitizeDictHelper {
    fn from_usize(val: usize) -> Self;
}

impl SanitizeDictHelper for u8 {
    fn from_usize(val: usize) -> Self {
        val as Self
    }
}

impl SanitizeDictHelper for u16 {
    fn from_usize(val: usize) -> Self {
        val as Self
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use arrow::array::{DictionaryArray, Int32Array, StringArray, UInt16Array};
    use arrow_schema::Field;

    #[test]
    fn test_sanitize_dict() {
        let input = DictionaryArray::new(
            UInt16Array::from_iter_values([1, 2, 5, 5, 6]),
            Arc::new(StringArray::from_iter_values([
                "0", "1", "2", "3", "4", "5", "6", "7",
            ])),
        );

        let result = sanitized_dict(&input);

        let expected = DictionaryArray::new(
            UInt16Array::from_iter_values([0, 1, 2, 2, 3]),
            Arc::new(StringArray::from_iter_values(["1", "2", "5", "6"])),
        );

        assert_eq!(result.unwrap(), expected)
    }

    #[test]
    fn test_sanitize_dict_all_keys_active() {
        let input = DictionaryArray::new(
            UInt16Array::from_iter_values([0, 1, 0, 1, 2, 1]),
            Arc::new(StringArray::from_iter_values(["0", "1", "2"])),
        );

        assert!(sanitized_dict(&input).is_none());
    }

    #[test]
    fn test_sanitize_struct_with_dict_field() {
        let dict_field = DictionaryArray::new(
            UInt16Array::from_iter_values([1, 3, 3]),
            Arc::new(StringArray::from_iter_values(["a", "b", "c", "d"])),
        );
        let int_field = Int32Array::from(vec![10, 20, 30]);

        let fields = vec![
            Arc::new(Field::new(
                "dict",
                DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                false,
            )),
            Arc::new(Field::new("ints", DataType::Int32, false)),
        ];

        let struct_arr = StructArray::new(
            fields.into(),
            vec![Arc::new(dict_field) as Arc<dyn Array>, Arc::new(int_field)],
            None,
        );

        let schema = Arc::new(arrow_schema::Schema::new(vec![Field::new(
            "s",
            DataType::Struct(struct_arr.fields().clone()),
            false,
        )]));
        let batch = RecordBatch::try_new(schema, vec![Arc::new(struct_arr)]).unwrap();

        let result = sanitize_record_batch(&batch).expect("should sanitize");

        let result_struct = result
            .column(0)
            .as_any()
            .downcast_ref::<StructArray>()
            .unwrap();
        let result_dict = result_struct
            .column(0)
            .as_any()
            .downcast_ref::<DictionaryArray<UInt16Type>>()
            .unwrap();

        let expected_dict = DictionaryArray::new(
            UInt16Array::from_iter_values([0, 1, 1]),
            Arc::new(StringArray::from_iter_values(["b", "d"])),
        );
        assert_eq!(result_dict, &expected_dict);

        // int field should be unchanged
        let result_ints = result_struct
            .column(1)
            .as_any()
            .downcast_ref::<Int32Array>()
            .unwrap();
        assert_eq!(result_ints, &Int32Array::from(vec![10, 20, 30]));
    }

    #[test]
    fn test_sanitize_struct_no_changes() {
        let dict_field = DictionaryArray::new(
            UInt16Array::from_iter_values([0, 1, 0]),
            Arc::new(StringArray::from_iter_values(["a", "b"])),
        );
        let int_field = Int32Array::from(vec![10, 20, 30]);

        let fields = vec![
            Arc::new(Field::new(
                "dict",
                DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                false,
            )),
            Arc::new(Field::new("ints", DataType::Int32, false)),
        ];

        let struct_arr = StructArray::new(
            fields.into(),
            vec![Arc::new(dict_field) as Arc<dyn Array>, Arc::new(int_field)],
            None,
        );

        let schema = Arc::new(arrow_schema::Schema::new(vec![Field::new(
            "s",
            DataType::Struct(struct_arr.fields().clone()),
            false,
        )]));
        let batch = RecordBatch::try_new(schema, vec![Arc::new(struct_arr)]).unwrap();

        assert!(
            sanitize_record_batch(&batch).is_none(),
            "should return None when all dict values are live"
        );
    }

    #[test]
    fn test_sanitize_dict_with_null_keys() {
        // Null keys should be skipped — only non-null keys determine which values are live.
        // Values at indices 0 and 2 are orphaned (no non-null key references them).
        let input = DictionaryArray::new(
            UInt16Array::from(vec![Some(1), None, Some(3), None, Some(3)]),
            Arc::new(StringArray::from_iter_values(["a", "b", "c", "d"])),
        );

        let result = sanitized_dict(&input);

        let expected = DictionaryArray::new(
            UInt16Array::from(vec![Some(0), None, Some(1), None, Some(1)]),
            Arc::new(StringArray::from_iter_values(["b", "d"])),
        );

        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_sanitize_dict_with_null_keys_all_active() {
        // All values are referenced by non-null keys, so no sanitization is needed
        // even though some keys are null.
        let input = DictionaryArray::new(
            UInt16Array::from(vec![Some(0), None, Some(1), Some(2), None]),
            Arc::new(StringArray::from_iter_values(["a", "b", "c"])),
        );

        assert!(sanitized_dict(&input).is_none());
    }
}
