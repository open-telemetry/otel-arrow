// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::{
    ops::{Range, Sub},
    sync::Arc,
};

use arrow::{
    array::{
        Array, ArrayRef, ArrowNativeTypeOp, ArrowPrimitiveType, BinaryArray, BooleanArray,
        DictionaryArray, Float64Array, Int64Array, NullBufferBuilder, PrimitiveArray, RecordBatch,
        StringArray, UInt8Array, UInt16Array, UInt32Array,
    },
    buffer::{BooleanBuffer, NullBuffer, ScalarBuffer},
    compute::{SortOptions, cast, concat, not, partition, rank, take},
    datatypes::{DataType, Schema, ToByteSlice, UInt8Type, UInt16Type},
    util::{bit_iterator::BitIndexIterator, bit_util},
};

use crate::{
    arrays::{get_required_array, get_u8_array},
    error::{Error, Result},
    otap::transform::create_next_element_equality_array,
    otlp::attributes::AttributeValueType,
    schema::{FieldExt, consts},
};

pub(crate) fn transport_optimize_encode_attrs<T: ArrowPrimitiveType>(
    record_batch: &RecordBatch,
) -> Result<RecordBatch>
where
    <T as ArrowPrimitiveType>::Native: Ord + Sub<Output = <T as ArrowPrimitiveType>::Native>,
{
    // builders for each attribute value column, indexed by AttributeValueType.
    //
    // the final sorted layout for each column will be:
    // - rows grouped by attribute type (matching the type column)
    // - within each type segment, rows sorted by: key, then value
    // - rows outside a column's type segment are set to null
    //
    // example for ATTRIBUTE_STR (type=1):
    //   - Rows where type=1: non-null, sorted by (key, value)
    //   - Rows where type≠1: null
    let mut sorted_val_columns: [Option<SortedValuesColumnBuilder>; 8] = [
        None, // empty
        record_batch
            .column_by_name(consts::ATTRIBUTE_STR)
            .map(|col| SortedValuesColumnBuilder::try_new(col))
            .transpose()?,
        record_batch
            .column_by_name(consts::ATTRIBUTE_INT)
            .map(|col| SortedValuesColumnBuilder::try_new(col))
            .transpose()?,
        record_batch
            .column_by_name(consts::ATTRIBUTE_DOUBLE)
            .map(|col| SortedValuesColumnBuilder::try_new(col))
            .transpose()?,
        record_batch
            .column_by_name(consts::ATTRIBUTE_BOOL)
            .map(|col| SortedValuesColumnBuilder::try_new(col))
            .transpose()?,
        None, // map
        None, // slice
        record_batch
            .column_by_name(consts::ATTRIBUTE_BYTES)
            .map(|col| SortedValuesColumnBuilder::try_new(col))
            .transpose()?,
    ];

    // ser column is special case because more than one attribute type is stored in this
    // column (both AttributeValueType::Map and AttributeValueType::Slice)
    let mut sorted_ser_column = record_batch
        .column_by_name(consts::ATTRIBUTE_SER)
        .map(|col| SortedValuesColumnBuilder::try_new(col))
        .transpose()?;

    let parent_id_col = get_required_array(record_batch, consts::PARENT_ID)?;

    // if the parent ID column is a dictionary, we cast it to a primitive array so we can work
    // directly with the ScalarBuffer containing the values. We then cast it back to dictionary
    // array when reconstructing the final dataset.
    //
    // TODO investigate if there's a more performant alternative to doing this cast
    let mut parent_dict_key_type = None;
    let parent_id_column_vals = match parent_id_col.data_type() {
        DataType::Dictionary(k, v) => {
            let as_prim = cast(parent_id_col, &T::DATA_TYPE).map_err(|_| {
                // cast would only fail here if the dictionary values were not something that can
                // be cast to <T as ArrowPrimitiveType>::DATA_TYPE (e.g. if the parent_id column
                // was completely the wong type). This would be very unusual and would suggest a
                // malformed record batch
                Error::ColumnDataTypeMismatch {
                    name: consts::PARENT_ID.into(),
                    actual: *v.clone(),
                    expect: T::DATA_TYPE.clone(),
                }
            })?;
            parent_dict_key_type = Some(*k.clone());
            as_prim
                .as_any()
                .downcast_ref::<PrimitiveArray<T>>()
                // safety: as_prim is created casting to the type we're expecting here
                .expect("casted to PrimitiveArray<T>")
                .values()
                .clone()
        }
        other_dt => parent_id_col
            .as_any()
            .downcast_ref::<PrimitiveArray<T>>()
            .ok_or_else(|| Error::ColumnDataTypeMismatch {
                name: consts::PARENT_ID.into(),
                actual: other_dt.clone(),
                expect: T::DATA_TYPE.clone(),
            })?
            .values()
            .clone(), // internally just an Arc<Bytes>
    };

    // this is the buffer into which we'll be appending sorted + quasi-delta encoded parent_ids
    let mut encoded_parent_id_column = Vec::with_capacity(record_batch.num_rows());

    // first sort by type/key to indices. This will allow us to take new columns for type/key
    // and we'll use these indices later on to take values columns/parent_id column for each
    // partition of rows with equivalent type/value
    let type_col = get_u8_array(record_batch, consts::ATTRIBUTE_TYPE)?;
    let key_col = get_required_array(record_batch, consts::ATTRIBUTE_KEY)?;
    let type_and_key_indices = sort_attrs_type_and_keys_to_indices(type_col, key_col.clone())?;

    // safety: we can call "expect" here because the indices we're taking are computed from the
    // indices of the array we're taking by the sort_attrs_type_and_keys_to_indices function
    let type_col_sorted =
        take(type_col, &type_and_key_indices, None).expect("can take sorted indices in bounds");
    let key_col_sorted =
        take(key_col, &type_and_key_indices, None).expect("can take sorted indices in bounds");

    // safety: `type_col_sorted` was computed just above by "taking" the type column, which we have
    // already checked is a UInt8Array by the call to `get_u8_array`
    let type_prim_arr = type_col_sorted
        .as_any()
        .downcast_ref::<UInt8Array>()
        .expect("type is UInt8Array");
    let type_col_sorted_bytes = type_prim_arr.values().inner().as_slice();

    // safety: partition can only be expected to fail if multiple arrays are passed that have
    // different lengths, or if an array type is passed for which the values cannot be compared.
    // neither of these failing criteria are met for a sinlge uint8 column
    let type_partitions =
        partition(&[type_col_sorted.clone()]).expect("can partition single UInt8 column");

    // iterates of ranges of of value types in ascending order. For example, we iterate over type
    // ranges Empty, String, Int. and so on..
    for type_range in type_partitions.ranges() {
        // the byte
        let type_range_attr_type = type_col_sorted_bytes[type_range.start];

        let is_ser_col_type = type_range_attr_type == AttributeValueType::Map as u8
            || type_range_attr_type == AttributeValueType::Slice as u8;

        let sorted_val_col_builder = if is_ser_col_type {
            sorted_ser_column.as_mut()
        } else {
            sorted_val_columns[type_range_attr_type as usize].as_mut()
        };

        // this contains indices of rows from the original record batch that are of the type for
        // the range we're currently handling sorted by key. For example, if the current range is
        // for string type, this will contain all indices from the original record batch that are a
        // string type attribute, sorted attribute key.
        let type_range_indices_key_sorted =
            type_and_key_indices.slice(type_range.start, type_range.len());

        // TODO:
        // - we might actually be able to defer materializing this
        // - reuse the vec for each type
        // - pre-allocate this vec
        let parent_id_type_range_by_key = type_range_indices_key_sorted
            .values()
            .iter()
            .map(|idx| parent_id_column_vals[*idx as usize])
            .collect::<Vec<_>>();

        // sort the values columns for values of this type
        if let Some(sorted_val_col) = sorted_val_col_builder {
            let values_type_range_by_key =
                sorted_val_col.take_source(&type_range_indices_key_sorted)?; // Arc t.2 (tmp)

            // set hint for the number of nulls
            let null_count = values_type_range_by_key.null_count();
            sorted_val_col.set_null_count_hint(null_count);

            let mut values_sorter = AttrValuesSorter::new(&values_type_range_by_key);
            let keys_range_sorted = key_col_sorted.slice(type_range.start, type_range.len());

            // partition key ranges (using SIMD)
            let next_eq_arr_key = create_next_element_equality_array(&keys_range_sorted)?; // Arc t.5 (tmp)
            let next_eq_inverted = not(&next_eq_arr_key).unwrap(); // Arc t.6 (tmp)
            let key_ranges = ranges(next_eq_inverted.values()); // Arc t.7 (tmp)

            let mut values_key_range_sorted_indices =
                Vec::with_capacity(key_ranges.iter().map(Range::len).max().unwrap_or_default());
            let mut values_ranges = Vec::new();

            for key_range in key_ranges {
                values_key_range_sorted_indices.clear();
                values_sorter
                    .sort_range_to_indices(&key_range, &mut values_key_range_sorted_indices); // Arc k.1 (in here for non dicts)

                // Map the sorted indices back to original source indices
                let original_indices: Vec<u32> = values_key_range_sorted_indices
                    .iter()
                    .map(|&i| {
                        type_range_indices_key_sorted
                            .value(key_range.start + i as usize)
                    })
                    .collect();

                sorted_val_col.append_indices(&original_indices)?;

                // TODO this creates a bunch of Arcs internally - we could make it faster by
                // optimizing the case where the array isn't dictionary encoded as well
                values_ranges.clear();
                values_sorter.take_and_partition_range(
                    &key_range,
                    &values_key_range_sorted_indices,
                    &mut values_ranges,
                )?;

                // TODO - reuse a vec (heal alloc) here
                let mut parent_id_key_range_sorted = values_key_range_sorted_indices
                    .iter()
                    .map(|idx| parent_id_type_range_by_key[key_range.start + *idx as usize])
                    .collect::<Vec<_>>();
                for values_range in &values_ranges {
                    let parent_ids_range = &mut parent_id_key_range_sorted
                        [values_range.range.start..values_range.range.end];

                    // nulls never count as equal for the purposes of delta encoding.
                    // Map & Slice type are never considered "equal" for the purposes of delta
                    // encoding so we skip adding quasi-delta encoding for this range, which means
                    // the parent_id segment also does not need to be sorted
                    if !values_range.is_null
                        && type_range_attr_type != AttributeValueType::Map as u8
                        && type_range_attr_type != AttributeValueType::Slice as u8
                    {
                        parent_ids_range.sort_unstable();
                        delta_encode_parent_id_slice(parent_ids_range);
                    }
                    encoded_parent_id_column.extend_from_slice(parent_ids_range);
                }
            }
        } else {
            // The values column is missing - which either means that the column was all null, or the
            // attribute type is "empty". Either way, we interpret this as "null' attribute, which
            // and nulls are not equal for the purposes of whether we should delta encode the column
            // so we can just append the unsorted parent IDs for the type range
            encoded_parent_id_column.extend_from_slice(&parent_id_type_range_by_key);
        }

        // push the unsorted values for columns not of this type to fill in gaps
        for attr_type in [
            AttributeValueType::Str as u8,
            AttributeValueType::Int as u8,
            AttributeValueType::Double as u8,
            AttributeValueType::Bool as u8,
            AttributeValueType::Bytes as u8,
        ] {
            if attr_type == type_range_attr_type {
                continue; // skip cause we already pushed the sorted section for this type
            }

            if let Some(sorted_val_col) = sorted_val_columns[attr_type as usize].as_mut() {
                sorted_val_col.append_nulls(type_range.len());
            }
        }

        // push unsorted values from slice/map type if not already pushed. This is handled as special
        // case from the loop above b/c both slice and map attrs types use the same column
        if type_range_attr_type != AttributeValueType::Map as u8
            && type_range_attr_type != AttributeValueType::Slice as u8
        {
            if let Some(sorted_val_col) = sorted_ser_column.as_mut() {
                sorted_val_col.append_nulls(type_range.len());
            }
        }
    }

    let mut parent_id_col = Arc::new(PrimitiveArray::<T>::new(
        ScalarBuffer::from(encoded_parent_id_column),
        None,
    )) as ArrayRef;

    // TODO comment about why this casting stuff is goofy
    if let Some(dict_key_type) = parent_dict_key_type {
        parent_id_col = cast(
            &parent_id_col,
            &DataType::Dictionary(Box::new(dict_key_type), Box::new(T::DATA_TYPE)),
        )
        .unwrap();
    }

    let mut fields = vec![];
    let mut columns = vec![];
    for field in record_batch.schema().fields() {
        let field_name = field.name();

        if field.name() == consts::PARENT_ID {
            fields.push(
                field
                    .as_ref()
                    .clone()
                    .with_encoding(consts::metadata::encodings::QUASI_DELTA),
            );
        } else {
            fields.push(field.as_ref().clone());
        }

        if field_name == consts::ATTRIBUTE_TYPE {
            columns.push(type_col_sorted.clone());
            continue;
        }

        if field_name == consts::ATTRIBUTE_KEY {
            columns.push(key_col_sorted.clone());
            // let sorted_keys_refs = sorted_keys.iter().map(|k| k.as_ref()).collect::<Vec<_>>();
            // columns.push(concat(&sorted_keys_refs).unwrap());
            continue;
        }

        if field_name == consts::ATTRIBUTE_STR {
            let sorted_col = sorted_val_columns[AttributeValueType::Str as usize]
                .as_ref()
                .unwrap();
            columns.push(sorted_col.finish()?);
            continue;
        }

        if field_name == consts::ATTRIBUTE_INT {
            let sorted_col = sorted_val_columns[AttributeValueType::Int as usize]
                .as_ref()
                .unwrap();
            columns.push(sorted_col.finish()?);
            continue;
        }

        if field_name == consts::ATTRIBUTE_BOOL {
            let sorted_col = sorted_val_columns[AttributeValueType::Bool as usize]
                .as_ref()
                .unwrap();
            columns.push(sorted_col.finish()?);
            continue;
        }

        if field_name == consts::ATTRIBUTE_DOUBLE {
            let sorted_col = sorted_val_columns[AttributeValueType::Double as usize]
                .as_ref()
                .unwrap();
            columns.push(sorted_col.finish()?);
            continue;
        }

        if field_name == consts::ATTRIBUTE_BYTES {
            let sorted_col = sorted_val_columns[AttributeValueType::Bytes as usize]
                .as_ref()
                .unwrap();
            columns.push(sorted_col.finish()?);
            continue;
        }

        if field_name == consts::ATTRIBUTE_SER {
            let sorted_col = sorted_ser_column.as_ref().unwrap();
            columns.push(sorted_col.finish()?);
            continue;
        }

        if field_name == consts::PARENT_ID {
            columns.push(parent_id_col.clone());
            continue;
        }

        todo!("handle bad col name {field_name}")
    }

    let schema = Schema::new(fields);
    let batch = RecordBatch::try_new(Arc::new(schema), columns).unwrap();

    Ok(batch)
}

// TODO better method naming & maybe this should take the Vec?
fn ranges(boundaries: &BooleanBuffer) -> Vec<Range<usize>> {
    let mut out = vec![];
    let mut current = 0;
    for idx in boundaries.set_indices() {
        let t = current;
        current = idx + 1;
        out.push(t..current)
    }
    let last = boundaries.len() + 1;
    if current != last {
        out.push(current..last)
    }
    out
}

fn delta_encode_parent_id_slice<T: Copy + Sub<Output = T>>(vals: &mut [T]) {
    let mut prev = vals[0];
    for i in 1..vals.len() {
        let curr = vals[i];
        // TODO should be wrapping_sub?
        vals[i] = curr - prev;
        prev = curr;
    }
}

fn sort_attrs_type_and_keys_to_indices(
    type_col: &UInt8Array,
    key_col: ArrayRef,
) -> Result<UInt32Array> {
    let len = type_col.len();

    let type_bytes = type_col.values().inner().as_slice();

    match key_col.data_type() {
        DataType::Dictionary(key, val) => match (*key.clone(), *val.clone()) {
            (DataType::UInt8, DataType::Utf8) => {
                // safety: we've just checked that the datatype is what we're casting to
                let dict_arr = key_col
                    .as_any()
                    .downcast_ref::<DictionaryArray<UInt8Type>>()
                    .expect("can downcast to dict<u8>");
                let keys = dict_arr.keys();
                let keys_values = keys.values().inner().as_slice();

                // safety: we've checked the datatype for this values array is utf8 and rank will
                // will only return error for types that don't support rank, which utf8 does
                let val_ranks = rank(dict_arr.values(), None).expect("can rank string array");

                // concat the type and key rank into a a u16 vec where the higher end byte of each
                // element is the type, and the lower end is the key
                let mut to_sort = vec![0u16; len];
                for i in 0..len {
                    to_sort[i] += (type_bytes[i] as u16) << 8u16;
                }
                for i in 0..len {
                    let rank = val_ranks[keys_values[i] as usize];
                    to_sort[i] += rank as u16;
                }

                let mut with_indices = to_sort.into_iter().enumerate().collect::<Vec<_>>();
                with_indices.sort_unstable_by(|a, b| a.1.cmp(&b.1));

                Ok(PrimitiveArray::from_iter_values(
                    with_indices.into_iter().map(|(rank, _)| rank as u32),
                ))
            }
            (DataType::UInt16, DataType::Utf8) => {
                // safety: we've just checked that the datatype is what we're casting to
                let dict_arr = key_col
                    .as_any()
                    .downcast_ref::<DictionaryArray<UInt16Type>>()
                    .expect("can downcast to dict<u16>");
                let keys = dict_arr.keys();
                let keys_values = keys.values();

                // safety: we've checked the datatype for this values array is utf8 and rank will
                // will only return error for types that don't support rank, which utf8 does
                let val_ranks = rank(dict_arr.values(), None).expect("can rank string array");

                // concat the type and key rank into a u32 vec where the higher end byte of each
                // element is the type, and the lower bytes are the key rank
                let mut to_sort = vec![0u32; len];
                for i in 0..len {
                    to_sort[i] += (type_bytes[i] as u32) << 16u32;
                }
                for i in 0..len {
                    let rank = val_ranks[keys_values[i] as usize];
                    to_sort[i] += rank;
                }

                let mut with_indices = to_sort.into_iter().enumerate().collect::<Vec<_>>();
                with_indices.sort_unstable_by(|a, b| a.1.cmp(&b.1));

                Ok(PrimitiveArray::from_iter_values(
                    with_indices.into_iter().map(|(rank, _)| rank as u32),
                ))
            }
            (other_key, DataType::Utf8) => {
                return Err(Error::UnsupportedDictionaryKeyType {
                    expect_oneof: vec![DataType::UInt8, DataType::UInt16],
                    actual: other_key,
                });
            }
            (_, other_value) => {
                return Err(Error::UnsupportedDictionaryKeyType {
                    expect_oneof: vec![DataType::Utf8],
                    actual: other_value,
                });
            }
        },
        DataType::Utf8 => {
            // sort by native keys. It would be unusual if we ended up in this branch, which is
            // why it is probably not as well optimized as the rest of the attribute encoding
            // implementation. Normally, we'd only end up with non-dict encoded keys if a batch
            // were received that had more than u16::MAX_VALUE unique attribute keys.

            // safety: we've already checked the datatype is Utf8 just above
            let keys_as_str = key_col
                .as_any()
                .downcast_ref::<StringArray>()
                .expect("can downcast to StringArray");

            // safety: we've checked the datatype for this values array is utf8 and rank will
            // will only return error for types that don't support rank, which utf8 doe
            let keys_rank = rank(keys_as_str, None).expect("can rank string array");

            let mut to_sort = vec![0u64; len];
            for i in 0..len {
                to_sort[i] += (type_bytes[i] as u64) << 32;
            }
            for i in 0..len {
                to_sort[i] += keys_rank[i] as u64;
            }

            let mut with_indices = to_sort.into_iter().enumerate().collect::<Vec<_>>();
            with_indices.sort_unstable_by(|a, b| a.1.cmp(&b.1));

            Ok(PrimitiveArray::from_iter_values(
                with_indices.into_iter().map(|(rank, _)| rank as u32),
            ))
        }
        _ => {
            todo!("bad data type")
        }
    }
}

#[derive(Debug)]
enum SortedValuesColumnSegment {
    Nulls(usize),
    NonNull(Vec<u32>), // indices into the original source array
}

struct SortedValuesColumnBuilder {
    // TODO - we might consider refactoring this into something not Arc to reduce heap allocations?
    sorted_segments: Vec<SortedValuesColumnSegment>,

    null_count_hint: Option<usize>,

    // TODO - we could get into the internals of `take` and figure out if it's faster not to take from
    // an Arc dyn array, then we'd avoid a heap allocation here as well
    source: Arc<dyn Array>,
}

impl SortedValuesColumnBuilder {
    fn try_new(source: &Arc<dyn Array>) -> Result<Self> {
        Ok(Self {
            sorted_segments: Vec::new(),
            source: Arc::clone(source),
            null_count_hint: None,
        })
    }

    fn take_source(&self, indices: &UInt32Array) -> Result<Arc<dyn Array>> {
        Ok(take(self.source.as_ref(), indices, None).unwrap())
    }

    fn append_indices(&mut self, indices: &[u32]) -> Result<()> {
        self.sorted_segments
            .push(SortedValuesColumnSegment::NonNull(indices.to_vec()));
        Ok(())
    }

    fn set_null_count_hint(&mut self, null_count: usize) {
        self.null_count_hint = Some(null_count);
    }

    fn gen_source_nulls(&self, count: usize) -> Arc<dyn Array> {
        match self.source.data_type() {
            DataType::Dictionary(k, _) => {
                match **k {
                    DataType::UInt8 => {
                        let dict_arr = self
                            .source
                            .as_any()
                            .downcast_ref::<DictionaryArray<UInt8Type>>()
                            .unwrap();
                        // TODO - either use new_unchecked here, or add a method to arrow-rs to speed this up if it's all null
                        // TOOD - safety comments
                        // TODO - feature unsafe gate?
                        #[allow(unsafe_code)]
                        let new_dict = unsafe {
                            DictionaryArray::new_unchecked(
                                UInt8Array::new_null(count),
                                dict_arr.values().clone(),
                            )
                        };
                        Arc::new(new_dict)
                    }
                    DataType::UInt16 => {
                        let dict_arr = self
                            .source
                            .as_any()
                            .downcast_ref::<DictionaryArray<UInt16Type>>()
                            .unwrap();
                        #[allow(unsafe_code)]
                        let new_dict = unsafe {
                            DictionaryArray::new_unchecked(
                                UInt16Array::new_null(count),
                                dict_arr.values().clone(),
                            )
                        };
                        Arc::new(new_dict)
                    }
                    _ => {
                        todo!("invalid dict")
                    }
                }
            }
            DataType::Binary => Arc::new(BinaryArray::new_null(count)),
            DataType::Boolean => Arc::new(BooleanArray::new_null(count)),
            DataType::Int64 => Arc::new(Int64Array::new_null(count)),
            DataType::Float64 => Arc::new(Float64Array::new_null(count)),
            DataType::Utf8 => Arc::new(StringArray::new_null(count)),
            _ => {
                todo!("invalid attrs array")
            }
        }
    }

    fn append_nulls(&mut self, count: usize) {
        self.sorted_segments
            .push(SortedValuesColumnSegment::Nulls(count))
    }

    fn finish(&self) -> Result<Arc<dyn Array>> {
        if self.sorted_segments.is_empty() {
            return Ok(self.gen_source_nulls(0));
        }

        // Calculate total length
        let total_len: usize = self
            .sorted_segments
            .iter()
            .map(|seg| match seg {
                SortedValuesColumnSegment::Nulls(count) => *count,
                SortedValuesColumnSegment::NonNull(indices) => indices.len(),
            })
            .sum();

        // Fast path: if only one segment, handle directly
        if self.sorted_segments.len() == 1 {
            match &self.sorted_segments[0] {
                SortedValuesColumnSegment::Nulls(count) => {
                    return Ok(self.gen_source_nulls(*count));
                }
                SortedValuesColumnSegment::NonNull(indices) => {
                    let indices_array = UInt32Array::from(indices.clone());
                    return Ok(take(self.source.as_ref(), &indices_array, None).unwrap());
                }
            }
        }

        // Check if we have any nulls
        let has_nulls = self
            .sorted_segments
            .iter()
            .any(|seg| matches!(seg, SortedValuesColumnSegment::Nulls(_)));

        // For dictionary arrays, build directly to avoid concat overhead
        if let DataType::Dictionary(key_type, _) = self.source.data_type() {
            return self.finish_dictionary_direct(total_len, &**key_type);
        }

        // For non-dictionary types with no nulls, fast path
        if !has_nulls {
            let mut all_indices = Vec::with_capacity(total_len);
            for segment in &self.sorted_segments {
                if let SortedValuesColumnSegment::NonNull(indices) = segment {
                    all_indices.extend_from_slice(indices);
                }
            }
            let indices_array = UInt32Array::from(all_indices);
            return Ok(take(self.source.as_ref(), &indices_array, None).unwrap());
        }

        // Fall back to concat approach for non-dictionary types with nulls
        // First, coalesce consecutive segments to reduce the number of arrays we create

        // Calculate total non-null count for preallocation
        let total_non_null_count: usize = self
            .sorted_segments
            .iter()
            .filter_map(|seg| match seg {
                SortedValuesColumnSegment::NonNull(indices) => Some(indices.len()),
                _ => None,
            })
            .sum();

        let mut coalesced: Vec<SortedValuesColumnSegment> = Vec::new();
        let mut current_nulls = 0;
        let mut current_indices = Vec::with_capacity(total_non_null_count);

        for segment in &self.sorted_segments {
            match segment {
                SortedValuesColumnSegment::Nulls(count) => {
                    // Flush accumulated indices if any
                    if !current_indices.is_empty() {
                        coalesced.push(SortedValuesColumnSegment::NonNull(std::mem::take(
                            &mut current_indices,
                        )));
                    }
                    current_nulls += count;
                }
                SortedValuesColumnSegment::NonNull(indices) => {
                    // Flush accumulated nulls if any
                    if current_nulls > 0 {
                        coalesced.push(SortedValuesColumnSegment::Nulls(current_nulls));
                        current_nulls = 0;
                    }
                    current_indices.extend_from_slice(indices);
                }
            }
        }
        // Flush remaining
        if current_nulls > 0 {
            coalesced.push(SortedValuesColumnSegment::Nulls(current_nulls));
        }
        if !current_indices.is_empty() {
            coalesced.push(SortedValuesColumnSegment::NonNull(current_indices));
        }

        // Now build arrays from coalesced segments (will be much fewer)
        let mut arrays = Vec::with_capacity(coalesced.len());
        for segment in &coalesced {
            match segment {
                SortedValuesColumnSegment::Nulls(count) => {
                    arrays.push(self.gen_source_nulls(*count));
                }
                SortedValuesColumnSegment::NonNull(indices) => {
                    let segment_indices = UInt32Array::from(indices.clone());
                    arrays.push(take(self.source.as_ref(), &segment_indices, None).unwrap());
                }
            }
        }

        let sorted_keys_refs = arrays.iter().map(|k| k.as_ref()).collect::<Vec<_>>();
        Ok(concat(sorted_keys_refs.as_ref()).unwrap())
    }

    fn finish_dictionary_direct(
        &self,
        total_len: usize,
        key_type: &DataType,
    ) -> Result<Arc<dyn Array>> {
        match *key_type {
            DataType::UInt8 => {
                let source_dict = self
                    .source
                    .as_any()
                    .downcast_ref::<DictionaryArray<UInt8Type>>()
                    .unwrap();
                let source_keys = source_dict.keys().values().iter().as_slice();

                let mut keys_builder = Vec::with_capacity(total_len);
                let mut null_buffer_builder = NullBufferBuilder::new(total_len);

                for segment in &self.sorted_segments {
                    match segment {
                        SortedValuesColumnSegment::Nulls(count) => {
                            // Append null keys
                            keys_builder.resize(keys_builder.len() + count, 0u8);
                            null_buffer_builder.append_n_nulls(*count);
                        }
                        SortedValuesColumnSegment::NonNull(indices) => {
                            // Take keys from source dictionary using indices
                            keys_builder
                                .extend(indices.iter().map(|idx| source_keys[*idx as usize]));

                            // Check if any of the source positions were null
                            if let Some(source_nulls) = source_dict.nulls() {
                                // Batch consecutive valid/null runs to reduce append overhead
                                let mut i = 0;
                                while i < indices.len() {
                                    let idx = indices[i] as usize;
                                    let is_valid = source_nulls.is_valid(idx);

                                    // Count consecutive runs of same validity
                                    let mut run_len = 1;
                                    while i + run_len < indices.len() {
                                        let next_idx = indices[i + run_len] as usize;
                                        if source_nulls.is_valid(next_idx) == is_valid {
                                            run_len += 1;
                                        } else {
                                            break;
                                        }
                                    }

                                    // Append the run
                                    if is_valid {
                                        null_buffer_builder.append_n_non_nulls(run_len);
                                    } else {
                                        null_buffer_builder.append_n_nulls(run_len);
                                    }

                                    i += run_len;
                                }
                            } else {
                                // Fast path: no nulls in source
                                for _ in 0..indices.len() {
                                    null_buffer_builder.append_non_null();
                                }
                            }
                        }
                    }
                }

                let null_buffer = null_buffer_builder.finish();
                let keys_array = UInt8Array::new(ScalarBuffer::from(keys_builder), null_buffer);
                #[allow(unsafe_code)]
                let result = unsafe {
                    DictionaryArray::new_unchecked(keys_array, source_dict.values().clone())
                };
                Ok(Arc::new(result))
            }
            DataType::UInt16 => {
                let source_dict = self
                    .source
                    .as_any()
                    .downcast_ref::<DictionaryArray<UInt16Type>>()
                    .unwrap();
                let source_keys = source_dict.keys().values().iter().as_slice();

                let mut keys_builder = Vec::with_capacity(total_len);

                // TODO need to make this same optimization for u8
                let mut null_buffer_builder = NullBufferBuilder::new(total_len);
                // let mut null_buffer_builder = self.null_count_hint
                //     .and_then(|null_count| (null_count > 0).then_some());

                for segment in &self.sorted_segments {
                    match segment {
                        SortedValuesColumnSegment::Nulls(count) => {
                            keys_builder.resize(keys_builder.len() + count, 0u16);
                            null_buffer_builder.append_n_nulls(*count);
                        }
                        SortedValuesColumnSegment::NonNull(indices) => {
                            // Take keys from source dictionary using indices
                            keys_builder
                                .extend(indices.iter().map(|idx| source_keys[*idx as usize]));

                            // Check if any of the source positions were null
                            let null_hint_zero = self.null_count_hint == Some(0);
                            if let Some(source_nulls) = source_dict.nulls()
                                && !null_hint_zero
                            {
                                // Batch consecutive valid/null runs to reduce append overhead
                                let mut i = 0;
                                while i < indices.len() {
                                    let idx = indices[i] as usize;
                                    let is_valid = source_nulls.is_valid(idx);

                                    // Count consecutive runs of same validity
                                    let mut run_len = 1;
                                    while i + run_len < indices.len() {
                                        let next_idx = indices[i + run_len] as usize;
                                        if source_nulls.is_valid(next_idx) == is_valid {
                                            run_len += 1;
                                        } else {
                                            break;
                                        }
                                    }

                                    // Append the run
                                    if is_valid {
                                        null_buffer_builder.append_n_non_nulls(run_len);
                                    } else {
                                        null_buffer_builder.append_n_nulls(run_len);
                                    }

                                    i += run_len;
                                }
                            } else {
                                // // TODO not sure this path is needed
                                // // Fast path: no nulls in source
                                // for _ in 0..indices.len() {
                                //     null_buffer_builder.append_non_null();
                                // }
                                null_buffer_builder.append_n_non_nulls(indices.len());
                            }
                        }
                    }
                }

                let null_buffer = null_buffer_builder.finish();
                let keys_array = UInt16Array::new(ScalarBuffer::from(keys_builder), null_buffer);
                #[allow(unsafe_code)]
                let result = unsafe {
                    DictionaryArray::new_unchecked(keys_array, source_dict.values().clone())
                };
                Ok(Arc::new(result))
            }
            _ => {
                // Fallback for unsupported key types
                Err(Error::Format {
                    error: "Unsupported dictionary key type".to_string(),
                })
            }
        }
    }
}

struct AttrValueDictKeysAndRanks {
    keys: Vec<u16>,
    ranks: Vec<u16>,
    nulls: Option<NullBuffer>,
}

enum AttrsValueSorterInner {
    KeysAndRanks(AttrValueDictKeysAndRanks),
    Array(ArrayRef),
}

struct AttrValuesSorter {
    inner: AttrsValueSorterInner,
    rank_sort_scratch: Vec<(usize, u16)>,
    key_partition_scratch: Vec<u16>,
    partition_buffer: Vec<u8>,
    partition_buffer_bitlen: usize,
}

struct NullableRange {
    range: Range<usize>,
    is_null: bool,
}

impl AttrValuesSorter {
    fn new(values_arr: &ArrayRef) -> Self {
        let inner = match values_arr.data_type() {
            DataType::Dictionary(k, v) => match **k {
                // TODO - not sure this branch is needed? I don't think we have u8 dict vals
                DataType::UInt8 => {
                    todo!("same as below")
                }
                DataType::UInt16 => {
                    let dict_arr = values_arr
                        .as_any()
                        .downcast_ref::<DictionaryArray<UInt16Type>>()
                        .unwrap();
                    let value_ranks = rank(
                        dict_arr.values(),
                        Some(SortOptions {
                            nulls_first: false,
                            ..Default::default()
                        }),
                    )
                    .unwrap();
                    // TODO - just copy the null buffer b/c doing it this way is slow ...
                    let key_ranks = dict_arr
                        .keys()
                        .values()
                        .iter()
                        .map(|k| value_ranks[*k as usize] as u16)
                        .collect::<Vec<_>>();

                    let rank_nulls = if dict_arr.keys().null_count() > 0 {
                        dict_arr.keys().nulls().cloned()
                    } else {
                        None
                    };

                    AttrsValueSorterInner::KeysAndRanks(AttrValueDictKeysAndRanks {
                        ranks: key_ranks,
                        nulls: rank_nulls,
                        keys: dict_arr.keys().values().to_vec(),
                    })
                }
                _ => {
                    todo!("bad dict key")
                }
            },
            _ => AttrsValueSorterInner::Array(values_arr.clone()),
        };

        Self {
            inner,
            rank_sort_scratch: Vec::new(),
            key_partition_scratch: Vec::new(),
            partition_buffer: Vec::new(),
            partition_buffer_bitlen: 0,
        }
    }

    fn sort_range_to_indices(&mut self, range: &Range<usize>, result: &mut Vec<u32>) {
        match &self.inner {
            AttrsValueSorterInner::Array(arr) => {
                // TODO Arcs created and dropped here
                let slice = arr.slice(range.start, range.len());
                let sorted_indices = arrow::compute::sort_to_indices(
                    &slice,
                    Some(SortOptions {
                        nulls_first: false,
                        ..Default::default()
                    }),
                    None,
                )
                .unwrap();
                result.extend(sorted_indices.values());
            }
            AttrsValueSorterInner::KeysAndRanks(ranks) => {
                self.rank_sort_scratch.clear();
                self.rank_sort_scratch.reserve(range.len());
                self.rank_sort_scratch.extend(
                    ranks
                        .ranks
                        .iter()
                        .skip(range.start)
                        .take(range.len())
                        .copied()
                        .enumerate(),
                );
                if let Some(nulls) = &ranks.nulls {
                    // slower path for nulls
                    self.rank_sort_scratch.sort_by(|a, b| {
                        match (nulls.is_valid(a.0), nulls.is_valid(b.0)) {
                            (true, true) => std::cmp::Ordering::Equal,
                            (true, false) => std::cmp::Ordering::Less,
                            (false, true) => std::cmp::Ordering::Greater,
                            (false, false) => a.1.cmp(&b.1),
                        }
                    });
                } else {
                    self.rank_sort_scratch.sort_by(|a, b| a.1.cmp(&b.1));
                }
                result.extend(self.rank_sort_scratch.iter().map(|(idx, _)| *idx as u32));
            }
        }
    }

    fn take_and_partition_range(
        &mut self,
        range: &Range<usize>,
        indices: &Vec<u32>,
        result: &mut Vec<NullableRange>,
    ) -> Result<()> {
        match &self.inner {
            AttrsValueSorterInner::Array(arr) => {
                let indices = UInt32Array::from_iter_values(indices.iter().copied());
                let values_range = arr.slice(range.start, range.len());
                let values_range_sorted = take(&values_range, &indices, None).unwrap();

                let next_eq_arr_key = create_next_element_equality_array(&values_range_sorted)?; // Arc k.6
                let next_eq_inverted = not(&next_eq_arr_key).unwrap(); // Arc k.7
                let values_ranges = ranges(next_eq_inverted.values());
                result.extend(values_ranges.into_iter().map(|range| NullableRange {
                    range,
                    is_null: false,
                }));

                if arr.null_count() > 0 {
                    for nullable_range in result.iter_mut() {
                        if values_range_sorted.is_null(nullable_range.range.start) {
                            nullable_range.is_null = true
                        }
                    }
                }
            }
            AttrsValueSorterInner::KeysAndRanks(keys) => {
                let keys_range = &keys.keys[range.start..range.end];
                self.key_partition_scratch.clear();
                self.key_partition_scratch.reserve(keys_range.len());
                self.key_partition_scratch
                    .extend(indices.iter().map(|i| keys_range[*i as usize]));
                self.fill_keys_partition_from_scratch_buffer();

                {
                    let boundaries_len = self.partition_buffer_bitlen;
                    let set_indices =
                        BitIndexIterator::new(&self.partition_buffer, 0, boundaries_len);
                    let mut current = 0;
                    for idx in set_indices {
                        let t = current;
                        current = idx + 1;
                        result.push(NullableRange {
                            range: t..current,
                            is_null: false,
                        })
                    }
                    let last = boundaries_len + 1;
                    if current != last {
                        result.push(NullableRange {
                            range: current..last,
                            is_null: false,
                        })
                    }
                }
            }
        }

        // fill in the null values
        if let AttrsValueSorterInner::KeysAndRanks(keys) = &self.inner {
            if let Some(nulls) = &keys.nulls {
                for nullable_range in result {
                    nullable_range.is_null =
                        nulls.is_null(indices[nullable_range.range.start] as usize)
                }
            }
        }

        Ok(())
    }

    fn fill_keys_partition_from_scratch_buffer(&mut self) {
        let len = self.key_partition_scratch.len() - 1;
        self.partition_buffer.clear();
        self.partition_buffer.reserve(bit_util::ceil(len, 64) * 8);

        let left = &self.key_partition_scratch[0..len];
        let right = &self.key_partition_scratch[1..len + 1];

        let f = |i: usize| -> bool {
            #[allow(unsafe_code)]
            let a = unsafe { *left.get_unchecked(i) };
            #[allow(unsafe_code)]
            let b = unsafe { *right.get_unchecked(i) };
            a.is_eq(b)
        };

        let chunks = len / 64;
        let remainder = len % 64;
        for chunk in 0..chunks {
            let mut packed = 0;
            for bit_idx in 0..64 {
                let i = bit_idx + chunk * 64;
                packed |= (f(i) as u64) << bit_idx;
            }

            // // SAFETY: Already allocated sufficient capacity
            // unsafe { buffer.push_unchecked(packed) }
            self.partition_buffer
                .extend_from_slice((!packed).to_byte_slice());
        }

        if remainder != 0 {
            let mut packed = 0;
            for bit_idx in 0..remainder {
                let i = bit_idx + chunks * 64;
                packed |= (f(i) as u64) << bit_idx;
            }

            // // SAFETY: Already allocated sufficient capacity
            // unsafe { buffer.push_unchecked(packed) }
            self.partition_buffer
                .extend_from_slice((!packed).to_byte_slice());
        }

        self.partition_buffer.truncate(bit_util::ceil(len, 8));
        self.partition_buffer_bitlen = len;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use arrow::datatypes::{Field, Schema, UInt32Type};

    #[test]
    fn test_transport_optimize_encode_attrs() {
        let input = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(
                    consts::ATTRIBUTE_KEY,
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                    false,
                ),
                Field::new(
                    consts::ATTRIBUTE_STR,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                    true,
                ),
                Field::new(
                    consts::ATTRIBUTE_INT,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Int64)),
                    true,
                ),
                Field::new(consts::ATTRIBUTE_DOUBLE, DataType::Float64, true),
                Field::new(consts::ATTRIBUTE_BOOL, DataType::Boolean, true),
                Field::new(
                    consts::ATTRIBUTE_BYTES,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Binary)),
                    true,
                ),
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values([
                    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11,
                ])),
                Arc::new(UInt8Array::from_iter_values([
                    AttributeValueType::Int as u8,
                    AttributeValueType::Int as u8,
                    AttributeValueType::Bool as u8,
                    AttributeValueType::Bool as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Bytes as u8,
                    AttributeValueType::Double as u8,
                ])),
                Arc::new(DictionaryArray::new(
                    UInt8Array::from_iter_values([1, 0, 0, 0, 0, 1, 0, 1, 1, 1, 0, 0]),
                    Arc::new(StringArray::from_iter_values(["ka", "kb"])),
                )),
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter([
                        None,
                        None,
                        None,
                        None,
                        Some(0),
                        Some(1),
                        Some(0),
                        None,
                        None,
                        Some(1),
                        None,
                        None,
                    ]),
                    Arc::new(StringArray::from_iter_values(["va", "vb"])),
                )),
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter([
                        Some(1),
                        Some(0),
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                    ]),
                    Arc::new(Int64Array::from_iter_values([0i64, 1i64])),
                )),
                Arc::new(Float64Array::from_iter([
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(2.0),
                    Some(1.0),
                    None,
                    None,
                    Some(1.0),
                ])),
                Arc::new(BooleanArray::from_iter([
                    None,
                    None,
                    Some(true),
                    Some(false),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                ])),
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter([
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        Some(0),
                        None,
                    ]),
                    Arc::new(BinaryArray::from_iter_values([b"a"])),
                )),
            ],
        )
        .unwrap();

        let expected = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false)
                    .with_encoding(consts::metadata::encodings::QUASI_DELTA),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(
                    consts::ATTRIBUTE_KEY,
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                    false,
                ),
                Field::new(
                    consts::ATTRIBUTE_STR,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                    true,
                ),
                Field::new(
                    consts::ATTRIBUTE_INT,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Int64)),
                    true,
                ),
                Field::new(consts::ATTRIBUTE_DOUBLE, DataType::Float64, true),
                Field::new(consts::ATTRIBUTE_BOOL, DataType::Boolean, true),
                Field::new(
                    consts::ATTRIBUTE_BYTES,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Binary)),
                    true,
                ),
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values([
                    4, 2, 5, 4, 1, 0, 11, 8, 7, 3, 2, 10,
                ])),
                Arc::new(UInt8Array::from_iter_values([
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Int as u8,
                    AttributeValueType::Int as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Bool as u8,
                    AttributeValueType::Bool as u8,
                    AttributeValueType::Bytes as u8,
                ])),
                Arc::new(DictionaryArray::new(
                    UInt8Array::from_iter_values([0, 0, 1, 1, 0, 1, 0, 1, 1, 0, 0, 0]),
                    Arc::new(StringArray::from_iter_values(["ka", "kb"])),
                )),
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter([
                        Some(0),
                        Some(0),
                        Some(1),
                        Some(1),
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                    ]),
                    Arc::new(StringArray::from_iter_values(["va", "vb"])),
                )),
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter([
                        None,
                        None,
                        None,
                        None,
                        Some(0),
                        Some(1),
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                    ]),
                    Arc::new(Int64Array::from_iter_values([0i64, 1i64])),
                )),
                Arc::new(Float64Array::from_iter([
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(1.0),
                    Some(1.0),
                    Some(2.0),
                    None,
                    None,
                    None,
                ])),
                Arc::new(BooleanArray::from_iter([
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(false),
                    Some(true),
                    None,
                ])),
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter([
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        Some(0),
                    ]),
                    Arc::new(BinaryArray::from_iter_values([b"a"])),
                )),
            ],
        )
        .unwrap();

        let result = transport_optimize_encode_attrs::<UInt16Type>(&input).unwrap();
        pretty_assertions::assert_eq!(result, expected);
    }

    #[test]
    fn test_transport_optimize_encode_attrs_sorts_by_parent_id_u16() {
        let input = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(
                    consts::ATTRIBUTE_KEY,
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                    false,
                ),
                Field::new(
                    consts::ATTRIBUTE_STR,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                    false,
                ),
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values([5, 4, 0, 3, 2, 1])),
                Arc::new(UInt8Array::from_iter_values([
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                ])),
                Arc::new(DictionaryArray::new(
                    UInt8Array::from_iter_values([1, 1, 1, 0, 0, 0]),
                    Arc::new(StringArray::from_iter_values(["a", "b"])),
                )),
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter_values([0, 1, 0, 1, 0, 1]),
                    Arc::new(StringArray::from_iter_values(["1", "2"])),
                )),
            ],
        )
        .unwrap();

        let expected = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false)
                    .with_encoding(consts::metadata::encodings::QUASI_DELTA),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(
                    consts::ATTRIBUTE_KEY,
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                    false,
                ),
                Field::new(
                    consts::ATTRIBUTE_STR,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                    false,
                ),
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values([2, 1, 2, 0, 5, 4])),
                Arc::new(UInt8Array::from_iter_values([
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                ])),
                Arc::new(DictionaryArray::new(
                    UInt8Array::from_iter_values([0, 0, 0, 1, 1, 1]),
                    Arc::new(StringArray::from_iter_values(["a", "b"])),
                )),
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter_values([0, 1, 1, 0, 0, 1]),
                    Arc::new(StringArray::from_iter_values(["1", "2"])),
                )),
            ],
        )
        .unwrap();

        let result = transport_optimize_encode_attrs::<UInt16Type>(&input).unwrap();
        pretty_assertions::assert_eq!(result, expected);
    }

    #[test]
    fn test_transport_optimize_encode_attrs_sorts_u32_parent_id() {
        let test_cases: &[(DataType, ArrayRef, ArrayRef)] = &[
            (
                DataType::UInt32,
                Arc::new(UInt32Array::from_iter_values([5, 4, 0, 3, 2, 1])) as ArrayRef,
                Arc::new(UInt32Array::from_iter_values([2, 1, 2, 0, 5, 4])) as ArrayRef,
            ),
            (
                DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::UInt32)),
                Arc::new(DictionaryArray::new(
                    UInt8Array::from_iter_values([5, 4, 0, 3, 2, 1]),
                    Arc::new(UInt32Array::from_iter_values([0, 1, 2, 3, 4, 5])),
                )) as ArrayRef,
                Arc::new(DictionaryArray::new(
                    UInt8Array::from_iter_values([2, 1, 2, 0, 5, 4]),
                    Arc::new(UInt32Array::from_iter_values([0, 1, 2, 3, 4, 5])),
                )) as ArrayRef,
            ),
            // TODO -- add a u32 variant of this ...
        ];

        for (data_type, input_parent_ids, expected_parent_ids) in test_cases {
            let input = RecordBatch::try_new(
                Arc::new(Schema::new(vec![
                    Field::new(consts::PARENT_ID, data_type.clone(), false),
                    Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                    Field::new(
                        consts::ATTRIBUTE_KEY,
                        DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                        false,
                    ),
                    Field::new(
                        consts::ATTRIBUTE_STR,
                        DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                        false,
                    ),
                ])),
                vec![
                    input_parent_ids.clone(),
                    Arc::new(UInt8Array::from_iter_values([
                        AttributeValueType::Str as u8,
                        AttributeValueType::Str as u8,
                        AttributeValueType::Str as u8,
                        AttributeValueType::Str as u8,
                        AttributeValueType::Str as u8,
                        AttributeValueType::Str as u8,
                    ])),
                    Arc::new(DictionaryArray::new(
                        UInt8Array::from_iter_values([1, 1, 1, 0, 0, 0]),
                        Arc::new(StringArray::from_iter_values(["a", "b"])),
                    )),
                    Arc::new(DictionaryArray::new(
                        UInt16Array::from_iter_values([0, 1, 0, 1, 0, 1]),
                        Arc::new(StringArray::from_iter_values(["1", "2"])),
                    )),
                ],
            )
            .unwrap();

            let expected = RecordBatch::try_new(
                Arc::new(Schema::new(vec![
                    Field::new(consts::PARENT_ID, data_type.clone(), false)
                        .with_encoding(consts::metadata::encodings::QUASI_DELTA),
                    Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                    Field::new(
                        consts::ATTRIBUTE_KEY,
                        DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                        false,
                    ),
                    Field::new(
                        consts::ATTRIBUTE_STR,
                        DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                        false,
                    ),
                ])),
                vec![
                    expected_parent_ids.clone(),
                    Arc::new(UInt8Array::from_iter_values([
                        AttributeValueType::Str as u8,
                        AttributeValueType::Str as u8,
                        AttributeValueType::Str as u8,
                        AttributeValueType::Str as u8,
                        AttributeValueType::Str as u8,
                        AttributeValueType::Str as u8,
                    ])),
                    Arc::new(DictionaryArray::new(
                        UInt8Array::from_iter_values([0, 0, 0, 1, 1, 1]),
                        Arc::new(StringArray::from_iter_values(["a", "b"])),
                    )),
                    Arc::new(DictionaryArray::new(
                        UInt16Array::from_iter_values([0, 1, 1, 0, 0, 1]),
                        Arc::new(StringArray::from_iter_values(["1", "2"])),
                    )),
                ],
            )
            .unwrap();

            let result = transport_optimize_encode_attrs::<UInt32Type>(&input).unwrap();
            pretty_assertions::assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_transport_optimize_encode_attrs_with_complex_value_types() {
        let schema = Arc::new(Schema::new(vec![
            Field::new(consts::PARENT_ID, DataType::UInt16, false),
            Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
            Field::new(
                consts::ATTRIBUTE_KEY,
                DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                true,
            ),
            Field::new(
                consts::ATTRIBUTE_STR,
                DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                true,
            ),
            Field::new(
                consts::ATTRIBUTE_SER,
                DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Binary)),
                true,
            ),
        ]));

        let input = RecordBatch::try_new(
            schema.clone(),
            vec![
                // 7 and 6 are in descending here to ensure we don't bother sorting
                // segments that have the same key/value if the type is a complex type
                // b/c we don't need to delta encode it
                Arc::new(UInt16Array::from_iter_values([0, 1, 2, 3, 4, 5, 7, 6, 8])),
                Arc::new(UInt8Array::from_iter_values([
                    AttributeValueType::Str as u8,
                    AttributeValueType::Slice as u8,
                    AttributeValueType::Slice as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Map as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Map as u8,
                    AttributeValueType::Map as u8,
                    AttributeValueType::Str as u8,
                ])),
                Arc::new(DictionaryArray::new(
                    UInt8Array::from_iter_values([0, 0, 0, 0, 0, 0, 0, 0, 0]),
                    Arc::new(StringArray::from_iter_values(["ka"])),
                )),
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter([
                        Some(0),
                        None,
                        None,
                        Some(1),
                        None,
                        Some(1),
                        None,
                        None,
                        Some(0),
                    ]),
                    Arc::new(StringArray::from_iter_values(["b", "a"])),
                )),
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter([
                        None,
                        Some(1),
                        Some(0),
                        None,
                        Some(2),
                        None,
                        Some(3),
                        Some(3),
                        None,
                    ]),
                    // Note - these aren't valid CBOR serialized attributes, but we're just testing
                    // that they sort in increasing byte order, so it's OK
                    Arc::new(BinaryArray::from_iter_values([b"a", b"b", b"c", b"d"])),
                )),
            ],
        )
        .unwrap();

        let expected = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false)
                    .with_encoding(consts::metadata::encodings::QUASI_DELTA),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(
                    consts::ATTRIBUTE_KEY,
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                    true,
                ),
                Field::new(
                    consts::ATTRIBUTE_STR,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                    true,
                ),
                Field::new(
                    consts::ATTRIBUTE_SER,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Binary)),
                    true,
                ),
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values([3, 2, 0, 8, 4, 7, 6, 2, 1])),
                Arc::new(UInt8Array::from_iter_values([
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Map as u8,
                    AttributeValueType::Map as u8,
                    AttributeValueType::Map as u8,
                    AttributeValueType::Slice as u8,
                    AttributeValueType::Slice as u8,
                ])),
                Arc::new(DictionaryArray::new(
                    UInt8Array::from_iter_values([0, 0, 0, 0, 0, 0, 0, 0, 0]),
                    Arc::new(StringArray::from_iter_values(["ka"])),
                )),
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter([
                        Some(1),
                        Some(1),
                        Some(0),
                        Some(0),
                        None,
                        None,
                        None,
                        None,
                        None,
                    ]),
                    Arc::new(StringArray::from_iter_values(["b", "a"])),
                )),
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter([
                        None,
                        None,
                        None,
                        None,
                        Some(2),
                        Some(3),
                        Some(3),
                        Some(0),
                        Some(1),
                    ]),
                    Arc::new(BinaryArray::from_iter_values([b"a", b"b", b"c", b"d"])),
                )),
            ],
        )
        .unwrap();

        let result = transport_optimize_encode_attrs::<UInt16Type>(&input).unwrap();
        pretty_assertions::assert_eq!(result, expected);
    }

    #[test]
    fn test_sort_and_apply_transport_delta_encoding_for_attr_empty_attrs() {
        let input = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(
                    consts::ATTRIBUTE_KEY,
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                    true,
                ),
                Field::new(
                    consts::ATTRIBUTE_STR,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                    true,
                ),
                Field::new(consts::ATTRIBUTE_DOUBLE, DataType::Float64, true),
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values([0, 1, 2, 3, 4, 5, 6, 7])),
                Arc::new(UInt8Array::from_iter_values([
                    AttributeValueType::Str as u8,
                    AttributeValueType::Empty as u8,
                    AttributeValueType::Empty as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Empty as u8,
                ])),
                Arc::new(DictionaryArray::new(
                    UInt8Array::from_iter_values([0, 1, 0, 0, 0, 0, 0, 0]),
                    Arc::new(StringArray::from_iter_values(["ka", "kb"])),
                )),
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter([
                        Some(0),
                        None,
                        None,
                        Some(1),
                        None,
                        None,
                        Some(0),
                        None,
                    ]),
                    Arc::new(StringArray::from_iter_values(["a", "b"])),
                )),
                Arc::new(Float64Array::from_iter([
                    None,
                    None,
                    None,
                    None,
                    Some(2.0),
                    Some(1.9999),
                    None,
                    None,
                ])),
            ],
        )
        .unwrap();

        let expected = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false)
                    .with_encoding(consts::metadata::encodings::QUASI_DELTA),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(
                    consts::ATTRIBUTE_KEY,
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                    true,
                ),
                Field::new(
                    consts::ATTRIBUTE_STR,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                    true,
                ),
                Field::new(consts::ATTRIBUTE_DOUBLE, DataType::Float64, true),
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values([2, 7, 1, 0, 6, 3, 5, 4])),
                Arc::new(UInt8Array::from_iter_values([
                    AttributeValueType::Empty as u8,
                    AttributeValueType::Empty as u8,
                    AttributeValueType::Empty as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Double as u8,
                ])),
                Arc::new(DictionaryArray::new(
                    UInt8Array::from_iter_values([0, 0, 1, 0, 0, 0, 0, 0]),
                    Arc::new(StringArray::from_iter_values(["ka", "kb"])),
                )),
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter([
                        None,
                        None,
                        None,
                        Some(0),
                        Some(0),
                        Some(1),
                        None,
                        None,
                    ]),
                    Arc::new(StringArray::from_iter_values(["a", "b"])),
                )),
                Arc::new(Float64Array::from_iter([
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(1.9999),
                    Some(2.0),
                ])),
            ],
        )
        .unwrap();

        let result = transport_optimize_encode_attrs::<UInt16Type>(&input).unwrap();
        pretty_assertions::assert_eq!(result, expected);
    }

    // TODO need a test for fully empty (null) column...

    #[test]
    fn test_sort_and_apply_transport_delta_encoding_for_sorts_by_keys_all_representations() {
        // test all combinations of how dictionary keys can be sorted
        let test_cases = &[
            (
                DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                Arc::new(DictionaryArray::new(
                    UInt8Array::from_iter_values([0, 2, 3, 1]),
                    Arc::new(StringArray::from_iter_values(["a", "b", "c", "d"])),
                )) as ArrayRef,
                Arc::new(DictionaryArray::new(
                    UInt8Array::from_iter_values([0, 1, 2, 3]),
                    Arc::new(StringArray::from_iter_values(["a", "b", "c", "d"])),
                )) as ArrayRef,
            ),
            (
                DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter_values([0, 2, 3, 1]),
                    Arc::new(StringArray::from_iter_values(["a", "b", "c", "d"])),
                )) as ArrayRef,
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter_values([0, 1, 2, 3]),
                    Arc::new(StringArray::from_iter_values(["a", "b", "c", "d"])),
                )) as ArrayRef,
            ),
            (
                DataType::Utf8,
                Arc::new(StringArray::from_iter_values(["a", "c", "d", "b"])),
                Arc::new(StringArray::from_iter_values(["a", "b", "c", "d"])),
            ),
        ];

        for (key_data_type, input_key_col, expected_key_col) in test_cases {
            let input = RecordBatch::try_new(
                Arc::new(Schema::new(vec![
                    Field::new(consts::PARENT_ID, DataType::UInt16, false),
                    Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                    Field::new(consts::ATTRIBUTE_KEY, key_data_type.clone(), false),
                    Field::new(consts::ATTRIBUTE_BOOL, DataType::Boolean, true),
                ])),
                vec![
                    Arc::new(UInt16Array::from_iter_values([0, 2, 3, 1])),
                    Arc::new(UInt8Array::from_iter_values([
                        AttributeValueType::Bool as u8,
                        AttributeValueType::Bool as u8,
                        AttributeValueType::Bool as u8,
                        AttributeValueType::Bool as u8,
                    ])),
                    input_key_col.clone(),
                    Arc::new(BooleanArray::from(vec![true, true, true, true])),
                ],
            )
            .unwrap();

            let expected = RecordBatch::try_new(
                Arc::new(Schema::new(vec![
                    Field::new(consts::PARENT_ID, DataType::UInt16, false)
                        .with_encoding(consts::metadata::encodings::QUASI_DELTA),
                    Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                    Field::new(consts::ATTRIBUTE_KEY, key_data_type.clone(), false),
                    Field::new(consts::ATTRIBUTE_BOOL, DataType::Boolean, true),
                ])),
                vec![
                    Arc::new(UInt16Array::from_iter_values([0, 1, 2, 3])),
                    Arc::new(UInt8Array::from_iter_values([
                        AttributeValueType::Bool as u8,
                        AttributeValueType::Bool as u8,
                        AttributeValueType::Bool as u8,
                        AttributeValueType::Bool as u8,
                    ])),
                    expected_key_col.clone(),
                    Arc::new(BooleanArray::from(vec![true, true, true, true])),
                ],
            )
            .unwrap();

            let result = transport_optimize_encode_attrs::<UInt16Type>(&input).unwrap();
            arrow::util::pretty::print_batches(&[input.clone()]);
            arrow::util::pretty::print_batches(&[expected.clone()]);
            arrow::util::pretty::print_batches(&[result.clone()]);

            pretty_assertions::assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_sort_and_apply_transport_delta_encoding_for_attr_null_attrs() {
        // create a record batch with some null attrs, both dict encoded and non-dict encoded
        // just to ensure that both are handled correctly.
        // - nulls are sorted last
        // - IDs for null values are not delta-encoded
        let input = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(
                    consts::ATTRIBUTE_KEY,
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                    true,
                ),
                Field::new(
                    consts::ATTRIBUTE_STR,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                    true,
                ),
                Field::new(consts::ATTRIBUTE_DOUBLE, DataType::Float64, true),
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values([0, 1, 2, 3, 4, 5, 6, 4, 7])),
                Arc::new(UInt8Array::from_iter_values([
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Double as u8,
                ])),
                Arc::new(DictionaryArray::new(
                    UInt8Array::from_iter_values([0, 0, 0, 0, 0, 0, 0, 0, 0]),
                    Arc::new(StringArray::from_iter_values(["ka"])),
                )),
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter([
                        Some(0),
                        Some(1),
                        None,
                        None,
                        None, // null str attr (dict encoded)
                        None,
                        None, // null str attr (dict encoded)
                        None, // null str attr (dict encoded)
                        None,
                    ]),
                    Arc::new(StringArray::from_iter_values(["a", "b"])),
                )),
                Arc::new(Float64Array::from_iter([
                    None,
                    None,
                    Some(2.0),
                    None, // null float attr (not dict encoded)
                    None,
                    Some(1.5),
                    None,
                    None,
                    None, // null float attr (not dict encoded)
                ])),
            ],
        )
        .unwrap();

        let expected = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false)
                    .with_encoding(consts::metadata::encodings::QUASI_DELTA),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(
                    consts::ATTRIBUTE_KEY,
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                    true,
                ),
                Field::new(
                    consts::ATTRIBUTE_STR,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                    true,
                ),
                Field::new(consts::ATTRIBUTE_DOUBLE, DataType::Float64, true),
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values([0, 1, 4, 6, 4, 5, 2, 3, 7])),
                Arc::new(UInt8Array::from_iter_values([
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Double as u8,
                ])),
                Arc::new(DictionaryArray::new(
                    UInt8Array::from_iter_values([0, 0, 0, 0, 0, 0, 0, 0, 0]),
                    Arc::new(StringArray::from_iter_values(["ka"])),
                )),
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter([
                        Some(0),
                        Some(1),
                        None, // null str attr (dict encoded)
                        None, // null str attr (dict encoded)
                        None, // null str attr (dict encoded)
                        None,
                        None,
                        None,
                        None,
                    ]),
                    Arc::new(StringArray::from_iter_values(["a", "b"])),
                )),
                Arc::new(Float64Array::from_iter([
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(1.5),
                    Some(2.0),
                    None, // null float attr (not dict encoded)
                    None, // null float attr (not dict encoded)
                ])),
            ],
        )
        .unwrap();

        let result = transport_optimize_encode_attrs::<UInt16Type>(&input).unwrap();
        pretty_assertions::assert_eq!(result, expected);
    }
}
