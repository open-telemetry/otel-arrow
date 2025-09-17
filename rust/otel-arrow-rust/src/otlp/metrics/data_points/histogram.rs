// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::arrays::{
    NullableArrayAccessor, get_f64_array_opt, get_timestamp_nanosecond_array_opt, get_u16_array,
    get_u32_array_opt, get_u64_array_opt,
};
use crate::error::{self, Error, Result};
use crate::otlp::ProtoBuffer;
use crate::otlp::attributes::{Attribute32Arrays, encode_key_value};
use crate::otlp::common::{ChildIndexIter, SortedBatchCursor};
use crate::otlp::metrics::exemplar::{ExemplarArrays, proto_encode_exemplar};
use crate::proto::consts::field_num::metrics::{
    HISTOGRAM_DP_ATTRIBUTES, HISTOGRAM_DP_BUCKET_COUNTS, HISTOGRAM_DP_COUNT,
    HISTOGRAM_DP_EXEMPLARS, HISTOGRAM_DP_EXPLICIT_BOUNDS, HISTOGRAM_DP_FLAGS, HISTOGRAM_DP_MAX,
    HISTOGRAM_DP_MIN, HISTOGRAM_DP_START_TIME_UNIX_NANO, HISTOGRAM_DP_SUM,
    HISTOGRAM_DP_TIME_UNIX_NANO,
};
use crate::proto::consts::wire_types;
use crate::proto_encode_len_delimited_unknown_size;
use crate::schema::consts;
use arrow::array::{
    Array, ArrayRef, Float64Array, ListArray, PrimitiveArray, RecordBatch,
    TimestampNanosecondArray, UInt16Array, UInt32Array, UInt64Array,
};
use arrow::datatypes::{
    ArrowNativeType, ArrowPrimitiveType, DataType, Field, FieldRef, Float64Type, UInt64Type,
};
use snafu::OptionExt;

pub struct HistogramDpArrays<'a> {
    pub id: Option<&'a UInt32Array>,
    pub parent_id: &'a UInt16Array,
    pub start_time_unix_nano: Option<&'a TimestampNanosecondArray>,
    pub time_unix_nano: Option<&'a TimestampNanosecondArray>,
    pub histogram_count: Option<&'a UInt64Array>,
    pub histogram_sum: Option<&'a Float64Array>,
    pub histogram_bucket_counts: Option<ListValueAccessor<'a, UInt64Type>>,
    pub histogram_explicit_bounds: Option<ListValueAccessor<'a, Float64Type>>,
    pub flags: Option<&'a UInt32Array>,
    pub histogram_min: Option<&'a Float64Array>,
    pub histogram_max: Option<&'a Float64Array>,
}

impl<'a> TryFrom<&'a RecordBatch> for HistogramDpArrays<'a> {
    type Error = Error;

    fn try_from(rb: &'a RecordBatch) -> Result<Self> {
        let id = get_u32_array_opt(rb, consts::ID)?;
        let parent_id = get_u16_array(rb, consts::PARENT_ID)?;
        let start_time_unix_nano =
            get_timestamp_nanosecond_array_opt(rb, consts::START_TIME_UNIX_NANO)?;
        let time_unix_nano = get_timestamp_nanosecond_array_opt(rb, consts::TIME_UNIX_NANO)?;
        let histogram_count = get_u64_array_opt(rb, consts::HISTOGRAM_COUNT)?;
        let histogram_sum = get_f64_array_opt(rb, consts::HISTOGRAM_SUM)?;
        let histogram_bucket_counts = rb
            .column_by_name(consts::HISTOGRAM_BUCKET_COUNTS)
            .map(ListValueAccessor::try_new)
            .transpose()?;
        let histogram_explicit_bucket_counts = rb
            .column_by_name(consts::HISTOGRAM_EXPLICIT_BOUNDS)
            .map(ListValueAccessor::try_new)
            .transpose()?;
        let flags = get_u32_array_opt(rb, consts::FLAGS)?;
        let histogram_min = get_f64_array_opt(rb, consts::HISTOGRAM_MIN)?;
        let histogram_max = get_f64_array_opt(rb, consts::HISTOGRAM_MAX)?;

        Ok(Self {
            id,
            parent_id,
            start_time_unix_nano,
            time_unix_nano,
            histogram_count,
            histogram_sum,
            histogram_bucket_counts,
            histogram_explicit_bounds: histogram_explicit_bucket_counts,
            flags,
            histogram_min,
            histogram_max,
        })
    }
}

/// Helper to access the element in a list array.
pub struct ListValueAccessor<'a, T: ArrowPrimitiveType> {
    pub list: &'a ListArray,
    pub value: &'a PrimitiveArray<T>,
}

impl<'a, T> ListValueAccessor<'a, T>
where
    T: ArrowPrimitiveType,
{
    pub fn try_new(list: &'a ArrayRef) -> Result<Self> {
        let list = list.as_any().downcast_ref::<ListArray>().with_context(|| {
            error::InvalidListArraySnafu {
                //todo: maybe set the field name here.
                expect_oneof: vec![DataType::List(FieldRef::new(Field::new(
                    "",
                    T::DATA_TYPE,
                    true,
                )))],
                actual: list.data_type().clone(),
            }
        })?;
        Self::try_new_from_list(list)
    }

    pub fn try_new_from_list(list: &'a ListArray) -> Result<Self> {
        let value_array = list.values();
        let value = value_array
            .as_any()
            .downcast_ref::<PrimitiveArray<T>>()
            .with_context(|| error::InvalidListArraySnafu {
                expect_oneof: vec![T::DATA_TYPE],
                actual: value_array.data_type().clone(),
            })?;

        Ok(Self { list, value })
    }

    #[must_use]
    pub fn value_at_opt(&self, idx: usize) -> Option<Vec<T::Native>> {
        if !self.list.is_valid(idx) {
            return None;
        }
        let start = self.list.offsets()[idx].as_usize();
        let end = self.list.offsets()[idx + 1].as_usize();
        let vec = (start..end)
            .map(|idx| self.value.value_at(idx).unwrap_or_default())
            .collect();

        Some(vec)
    }
}

pub(crate) fn proto_encode_histogram_data_point(
    index: usize,
    hist_dp_arrays: &HistogramDpArrays<'_>,
    attrs: Option<&Attribute32Arrays<'_>>,
    attrs_cursor: &mut SortedBatchCursor,
    exemplar_arrays: Option<&ExemplarArrays<'_>>,
    exemplar_cursor: &mut SortedBatchCursor,
    exemplar_attr_arrays: Option<&Attribute32Arrays<'_>>,
    exemplar_attrs_cursor: &mut SortedBatchCursor,
    result_buf: &mut ProtoBuffer,
) -> Result<()> {
    if let Some(attrs) = attrs {
        if let Some(id) = hist_dp_arrays.id.value_at(index) {
            let attrs_index_iter = ChildIndexIter::new(id, &attrs.parent_id, attrs_cursor);
            for attrs_index in attrs_index_iter {
                proto_encode_len_delimited_unknown_size!(
                    HISTOGRAM_DP_ATTRIBUTES,
                    encode_key_value(attrs, attrs_index, result_buf)?,
                    result_buf
                );
            }
        }
    }

    if let Some(col) = hist_dp_arrays.start_time_unix_nano {
        if let Some(val) = col.value_at(index) {
            result_buf.encode_field_tag(HISTOGRAM_DP_START_TIME_UNIX_NANO, wire_types::FIXED64);
            result_buf.extend_from_slice(&val.to_le_bytes());
        }
    }

    if let Some(col) = hist_dp_arrays.time_unix_nano {
        if let Some(val) = col.value_at(index) {
            result_buf.encode_field_tag(HISTOGRAM_DP_TIME_UNIX_NANO, wire_types::FIXED64);
            result_buf.extend_from_slice(&val.to_le_bytes());
        }
    }

    if let Some(col) = hist_dp_arrays.histogram_count {
        if let Some(val) = col.value_at(index) {
            result_buf.encode_field_tag(HISTOGRAM_DP_COUNT, wire_types::FIXED64);
            result_buf.extend_from_slice(&val.to_le_bytes());
        }
    }

    if let Some(col) = hist_dp_arrays.histogram_sum {
        if let Some(val) = col.value_at(index) {
            result_buf.encode_field_tag(HISTOGRAM_DP_SUM, wire_types::FIXED64);
            result_buf.extend_from_slice(&val.to_le_bytes());
        }
    }

    if let Some(bucket_counts) = &hist_dp_arrays.histogram_bucket_counts {
        if bucket_counts.list.is_valid(index) {
            let value_offsets = bucket_counts.list.value_offsets();
            let start = value_offsets[index] as usize;
            let end = value_offsets[index + 1] as usize;
            for i in start..end {
                if bucket_counts.value.is_valid(i) {
                    let val = bucket_counts.value.value(i);
                    result_buf.encode_field_tag(HISTOGRAM_DP_BUCKET_COUNTS, wire_types::FIXED64);
                    result_buf.extend_from_slice(&val.to_le_bytes());
                }
            }
        }
    }

    if let Some(explicit_bounds) = &hist_dp_arrays.histogram_explicit_bounds {
        if explicit_bounds.list.is_valid(index) {
            let value_offsets = explicit_bounds.list.value_offsets();
            let start = value_offsets[index] as usize;
            let end = value_offsets[index + 1] as usize;
            for i in start..end {
                if explicit_bounds.value.is_valid(i) {
                    let val = explicit_bounds.value.value(i);
                    result_buf.encode_field_tag(HISTOGRAM_DP_EXPLICIT_BOUNDS, wire_types::FIXED64);
                    result_buf.extend_from_slice(&val.to_le_bytes());
                }
            }
        }
    }

    if let Some(exemplar_arrays) = exemplar_arrays {
        if let Some(id) = hist_dp_arrays.id.value_at(index) {
            let exemplar_index_iter =
                ChildIndexIter::new(id, &exemplar_arrays.parent_id, exemplar_cursor);
            for exemplar_index in exemplar_index_iter {
                proto_encode_len_delimited_unknown_size!(
                    HISTOGRAM_DP_EXEMPLARS,
                    proto_encode_exemplar(
                        exemplar_index,
                        exemplar_arrays,
                        exemplar_attr_arrays,
                        exemplar_attrs_cursor,
                        result_buf
                    )?,
                    result_buf
                );
            }
        }
    }

    if let Some(col) = hist_dp_arrays.flags {
        if let Some(val) = col.value_at(index) {
            result_buf.encode_field_tag(HISTOGRAM_DP_FLAGS, wire_types::VARINT);
            result_buf.encode_varint(val as u64);
        }
    }

    if let Some(col) = hist_dp_arrays.histogram_min {
        if let Some(val) = col.value_at(index) {
            result_buf.encode_field_tag(HISTOGRAM_DP_MIN, wire_types::FIXED64);
            result_buf.extend_from_slice(&val.to_le_bytes());
        }
    }

    if let Some(col) = hist_dp_arrays.histogram_max {
        if let Some(val) = col.value_at(index) {
            result_buf.encode_field_tag(HISTOGRAM_DP_MAX, wire_types::FIXED64);
            result_buf.extend_from_slice(&val.to_le_bytes());
        }
    }

    Ok(())
}
