// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Utilities for sanitizing arrow arrays.
//!
//! This procedure involves removing any unreferenced data in any arrow array, specifically:
//! - removing any dictionary values that have no keys pointing to them

use std::borrow::Cow;
use std::future::Future;
use std::pin::pin;
use std::sync::Arc;
use std::task::{Context, Poll, Waker};

use arrow::{
    array::{
        Array, Capacities, DictionaryArray, MutableArrayData, PrimitiveArray, RecordBatch,
        StructArray, make_array,
    },
    datatypes::{ArrowDictionaryKeyType, ArrowNativeType, UInt8Type, UInt16Type},
};
use arrow_schema::DataType;

/// A local work quantum shared across cooperating Arrow processing phases.
#[derive(Default)]
pub struct CooperativeBudget {
    items: usize,
    bytes: usize,
    yield_disabled: bool,
}

impl CooperativeBudget {
    fn synchronous() -> Self {
        Self {
            yield_disabled: true,
            ..Self::default()
        }
    }

    /// Yield after a bounded item or byte quantum; one indivisible item may exceed it.
    pub async fn consume(&mut self, items: usize, bytes: usize) {
        if self.yield_disabled {
            return;
        }
        self.items = self.items.saturating_add(items);
        self.bytes = self.bytes.saturating_add(bytes);
        if self.items >= 128 || self.bytes >= 256 * 1024 {
            self.items = 0;
            self.bytes = 0;
            tokio::task::yield_now().await;
        }
    }
}

/// Sanitize a batch while yielding between bounded dictionary-processing units.
pub async fn sanitize_record_batch_cooperative(
    record_batch: &RecordBatch,
    budget: &mut CooperativeBudget,
) -> Option<RecordBatch> {
    let mut columns = Cow::from(record_batch.columns());
    for (index, column) in record_batch.columns().iter().enumerate() {
        if let Some(sanitized) = sanitize_column_cooperative(column.as_ref(), budget).await {
            columns.to_mut()[index] = sanitized;
        }
    }
    match columns {
        Cow::Borrowed(_) => None,
        Cow::Owned(columns) => Some(
            RecordBatch::try_new(record_batch.schema(), columns).expect("sanitized column shape"),
        ),
    }
}

async fn sanitize_column_cooperative(
    column: &dyn Array,
    budget: &mut CooperativeBudget,
) -> Option<Arc<dyn Array>> {
    budget.consume(1, 0).await;
    if let Some(dictionary) = column.as_any().downcast_ref::<DictionaryArray<UInt8Type>>() {
        return sanitized_dict_cooperative(dictionary, budget)
            .await
            .map(|array| Arc::new(array) as Arc<dyn Array>);
    }
    if let Some(dictionary) = column
        .as_any()
        .downcast_ref::<DictionaryArray<UInt16Type>>()
    {
        return sanitized_dict_cooperative(dictionary, budget)
            .await
            .map(|array| Arc::new(array) as Arc<dyn Array>);
    }
    let structure = column.as_any().downcast_ref::<StructArray>()?;
    let mut columns: Cow<'_, [Arc<dyn Array>]> = Cow::Borrowed(structure.columns());
    for (index, field) in structure.columns().iter().enumerate() {
        if let Some(sanitized) = Box::pin(sanitize_column_cooperative(field.as_ref(), budget)).await
        {
            columns.to_mut()[index] = sanitized;
        }
    }
    match columns {
        Cow::Borrowed(_) => None,
        Cow::Owned(columns) => Some(Arc::new(StructArray::new(
            structure.fields().clone(),
            columns,
            structure.nulls().cloned(),
        ))),
    }
}

async fn sanitized_dict_cooperative<Key: ArrowDictionaryKeyType>(
    dictionary: &DictionaryArray<Key>,
    budget: &mut CooperativeBudget,
) -> Option<DictionaryArray<Key>>
where
    Key::Native: SanitizeDictHelper,
{
    let values = dictionary.values();
    let mut live = vec![false; values.len().min(usize::from(u16::MAX) + 1)];
    let mut live_count = 0;
    for key in dictionary.keys().iter() {
        budget.consume(1, 0).await;
        if let Some(key) = key {
            let index = key.as_usize();
            if !live[index] {
                live[index] = true;
                live_count += 1;
                if live_count == values.len() {
                    return None;
                }
            }
        }
    }
    let data = values.to_data();
    let capacities = if matches!(values.data_type(), DataType::Utf8 | DataType::Binary) {
        let offsets = data.buffer::<i32>(0);
        let mut bytes = 0;
        for (index, is_live) in live.iter().copied().enumerate() {
            budget.consume(1, 0).await;
            if is_live {
                bytes += (offsets[index + 1] - offsets[index]) as usize;
            }
        }
        Capacities::Binary(live_count, Some(bytes))
    } else {
        Capacities::Array(live_count)
    };
    let mut selected = MutableArrayData::with_capacities(vec![&data], false, capacities);
    let mut remapped = vec![0; live.len()];
    let mut next_key = 0;
    for (index, is_live) in live.into_iter().enumerate() {
        if is_live {
            let bytes = data
                .slice(index, 1)
                .get_slice_memory_size()
                .unwrap_or(usize::MAX);
            budget.consume(1, bytes).await;
            selected.extend(0, index, index + 1);
            remapped[index] = next_key;
            next_key += 1;
        } else {
            budget.consume(1, 0).await;
        }
    }
    let mut keys = Vec::with_capacity(dictionary.len());
    for key in dictionary.keys().iter() {
        budget.consume(1, size_of::<Key::Native>()).await;
        keys.push(key.map_or_else(Key::Native::default, |key| {
            <Key::Native as SanitizeDictHelper>::from_usize(remapped[key.as_usize()])
        }));
    }
    let keys = PrimitiveArray::<Key>::new(keys.into(), dictionary.keys().nulls().cloned());
    Some(DictionaryArray::new(keys, make_array(selected.freeze())))
}

/// Sanitizes a single array column, returning `Some` with the sanitized array if any changes
/// were made, or `None` if the array was already clean.
#[must_use]
pub fn sanitize_column(column: &dyn Array) -> Option<Arc<dyn Array>> {
    complete_synchronously(sanitize_column_cooperative(
        column,
        &mut CooperativeBudget::synchronous(),
    ))
}

/// Sanitizes every column in a record batch without requiring an async runtime.
#[must_use]
pub fn sanitize_record_batch(record_batch: &RecordBatch) -> Option<RecordBatch> {
    complete_synchronously(sanitize_record_batch_cooperative(
        record_batch,
        &mut CooperativeBudget::synchronous(),
    ))
}

fn complete_synchronously<Output>(future: impl Future<Output = Output>) -> Output {
    match pin!(future).poll(&mut Context::from_waker(Waker::noop())) {
        Poll::Ready(output) => output,
        Poll::Pending => unreachable!("non-yielding sanitization must finish in one poll"),
    }
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

    /// Scenario: A dictionary contains unused values before and between referenced values.
    /// Guarantees: The synchronous column API removes unused values and remaps every key.
    #[test]
    fn test_sanitize_dict() {
        let input = DictionaryArray::new(
            UInt16Array::from_iter_values([1, 2, 5, 5, 6]),
            Arc::new(StringArray::from_iter_values([
                "0", "1", "2", "3", "4", "5", "6", "7",
            ])),
        );

        let result = sanitize_column(&input);

        let expected = DictionaryArray::new(
            UInt16Array::from_iter_values([0, 1, 2, 2, 3]),
            Arc::new(StringArray::from_iter_values(["1", "2", "5", "6"])),
        );

        assert_eq!(result.unwrap().to_data(), expected.to_data())
    }

    /// Scenario: Every dictionary value is referenced by at least one key.
    /// Guarantees: The synchronous column API reports no change for an already clean dictionary.
    #[test]
    fn test_sanitize_dict_all_keys_active() {
        let input = DictionaryArray::new(
            UInt16Array::from_iter_values([0, 1, 0, 1, 2, 1]),
            Arc::new(StringArray::from_iter_values(["0", "1", "2"])),
        );

        assert!(sanitize_column(&input).is_none());
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

    /// Scenario: Null keys coexist with referenced and orphaned dictionary values.
    /// Guarantees: Sanitization preserves null keys while remapping only their valid neighbors.
    #[test]
    fn test_sanitize_dict_with_null_keys() {
        // Null keys should be skipped -- only non-null keys determine which values are live.
        // Values at indices 0 and 2 are orphaned (no non-null key references them).
        let input = DictionaryArray::new(
            UInt16Array::from(vec![Some(1), None, Some(3), None, Some(3)]),
            Arc::new(StringArray::from_iter_values(["a", "b", "c", "d"])),
        );

        let result = sanitize_column(&input);

        let expected = DictionaryArray::new(
            UInt16Array::from(vec![Some(0), None, Some(1), None, Some(1)]),
            Arc::new(StringArray::from_iter_values(["b", "d"])),
        );

        assert_eq!(result.unwrap().to_data(), expected.to_data());
    }

    /// Scenario: Null keys coexist with non-null keys referencing every dictionary value.
    /// Guarantees: Null keys do not cause an already clean dictionary to be rebuilt.
    #[test]
    fn test_sanitize_dict_with_null_keys_all_active() {
        // All values are referenced by non-null keys, so no sanitization is needed
        // even though some keys are null.
        let input = DictionaryArray::new(
            UInt16Array::from(vec![Some(0), None, Some(1), Some(2), None]),
            Arc::new(StringArray::from_iter_values(["a", "b", "c"])),
        );

        assert!(sanitize_column(&input).is_none());
    }

    /// Scenario: Runtime-free synchronous calls exceed both cooperative thresholds in a nested dictionary.
    /// Guarantees: Column and batch sanitization finish in one poll, preserving nulls and remapping keys.
    #[test]
    fn synchronous_sanitization_needs_no_runtime() {
        let first = "x".repeat(256 * 1024);
        let second = "y".repeat(256 * 1024);
        let input = DictionaryArray::new(
            UInt16Array::from_iter((0..512).map(|row| match row % 3 {
                0 => None,
                1 => Some(1),
                _ => Some(2),
            })),
            Arc::new(StringArray::from(vec!["unused", &first, &second])),
        );
        let expected = DictionaryArray::new(
            UInt16Array::from_iter((0..512).map(|row| match row % 3 {
                0 => None,
                1 => Some(0),
                _ => Some(1),
            })),
            Arc::new(StringArray::from(vec![first, second])),
        );
        let field = Arc::new(Field::new("value", input.data_type().clone(), true));
        let input = StructArray::from(vec![(field.clone(), Arc::new(input) as Arc<dyn Array>)]);
        let expected = StructArray::from(vec![(field, Arc::new(expected) as Arc<dyn Array>)]);
        assert_eq!(
            sanitize_column(&input).unwrap().to_data(),
            expected.to_data()
        );
        let input =
            RecordBatch::try_from_iter([("body", Arc::new(input) as Arc<dyn Array>)]).unwrap();
        let expected =
            RecordBatch::try_from_iter([("body", Arc::new(expected) as Arc<dyn Array>)]).unwrap();
        assert_eq!(sanitize_record_batch(&input), Some(expected));
    }

    /// Scenario: Nested dictionaries contain null keys and many referenced large values.
    /// Guarantees: Cooperative sanitization matches synchronous output while another local task progresses.
    #[tokio::test(flavor = "current_thread")]
    async fn cooperative_sanitization_preserves_values_and_progress() {
        use std::{cell::Cell, rc::Rc};
        let values: Vec<_> = (0..512)
            .map(|index| format!("{index}:{}", "x".repeat(4096)))
            .collect();
        let dictionary = DictionaryArray::<UInt16Type>::new(
            UInt16Array::from_iter(
                (0..4096).map(|row| (row % 5 != 0).then_some((row % 511) as u16)),
            ),
            Arc::new(StringArray::from(values)),
        );
        let structure = StructArray::from(vec![(
            Arc::new(Field::new("value", dictionary.data_type().clone(), true)),
            Arc::new(dictionary) as Arc<dyn Array>,
        )]);
        let batch =
            RecordBatch::try_from_iter([("body", Arc::new(structure) as Arc<dyn Array>)]).unwrap();
        let expected = sanitize_record_batch(&batch).unwrap();
        let done = Rc::new(Cell::new(false));
        let progress = Rc::new(Cell::new(0));
        let mut budget = CooperativeBudget::default();
        let actual = tokio::task::LocalSet::new()
            .run_until(async {
                let observer = tokio::task::spawn_local({
                    let done = Rc::clone(&done);
                    let progress = Rc::clone(&progress);
                    async move {
                        while !done.get() {
                            progress.set(progress.get() + 1);
                            tokio::task::yield_now().await;
                        }
                    }
                });
                let output = sanitize_record_batch_cooperative(&batch, &mut budget)
                    .await
                    .unwrap();
                done.set(true);
                observer.await.unwrap();
                output
            })
            .await;
        assert_eq!(actual, expected);
        assert!(
            progress.get() > 1,
            "sanitization must not monopolize the runtime"
        );
    }

    /// Scenario: Dictionary inputs include both key widths, slices, nulls and primitive or byte values.
    /// Guarantees: Both entry points preserve independently specified live values, remapped keys and nulls.
    #[tokio::test(flavor = "current_thread")]
    async fn cooperative_sanitization_matches_dictionary_edge_cases() {
        use arrow::array::{BinaryArray, BooleanArray, UInt8Array};
        use arrow::compute::filter;
        let values: Vec<Arc<dyn Array>> = vec![
            Arc::new(
                StringArray::from(vec![Some("prefix"), Some("kept"), None, Some("unused")])
                    .slice(1, 3),
            ),
            Arc::new(Int32Array::from(vec![Some(1), None, Some(3)])),
            Arc::new(BinaryArray::from(vec![
                Some(b"one".as_slice()),
                None,
                Some(b"three".as_slice()),
            ])),
        ];
        for values in values {
            for (keys, expected_keys, live_values) in [
                (vec![], vec![], [false, false, false]),
                (vec![None, None], vec![None, None], [false, false, false]),
                (
                    vec![Some(0), Some(1), Some(2)],
                    vec![Some(0), Some(1), Some(2)],
                    [true, true, true],
                ),
                (
                    vec![Some(0), None, Some(0), Some(1)],
                    vec![Some(0), None, Some(0), Some(1)],
                    [true, true, false],
                ),
                (
                    vec![Some(2), None, Some(2), Some(1)],
                    vec![Some(1), None, Some(1), Some(0)],
                    [false, true, true],
                ),
            ] {
                let expected_values =
                    filter(values.as_ref(), &BooleanArray::from(live_values.to_vec())).unwrap();
                let small: Arc<dyn Array> = Arc::new(DictionaryArray::<UInt8Type>::new(
                    UInt8Array::from(keys.clone()),
                    values.clone(),
                ));
                let large: Arc<dyn Array> = Arc::new(DictionaryArray::<UInt16Type>::new(
                    UInt16Array::from_iter(keys.into_iter().map(|key| key.map(u16::from))),
                    values.clone(),
                ));
                let expected_small: Arc<dyn Array> = Arc::new(DictionaryArray::<UInt8Type>::new(
                    UInt8Array::from(expected_keys.clone()),
                    expected_values.clone(),
                ));
                let expected_large: Arc<dyn Array> = Arc::new(DictionaryArray::<UInt16Type>::new(
                    UInt16Array::from_iter(expected_keys.into_iter().map(|key| key.map(u16::from))),
                    expected_values,
                ));
                for (column, expected_column) in [(small, expected_small), (large, expected_large)]
                {
                    let batch = RecordBatch::try_from_iter([("value", column)]).unwrap();
                    let expected =
                        RecordBatch::try_from_iter([("value", expected_column)]).unwrap();
                    let synchronous =
                        sanitize_record_batch(&batch).unwrap_or_else(|| batch.clone());
                    let actual = sanitize_record_batch_cooperative(
                        &batch,
                        &mut CooperativeBudget::default(),
                    )
                    .await
                    .unwrap_or_else(|| batch.clone());
                    assert_eq!(synchronous, expected);
                    assert_eq!(actual, expected);
                }
            }
        }
    }
}
