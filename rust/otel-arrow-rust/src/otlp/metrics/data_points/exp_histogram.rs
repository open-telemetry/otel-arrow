// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::arrays::{
    NullableArrayAccessor, get_f64_array_opt, get_i32_array_opt,
    get_timestamp_nanosecond_array_opt, get_u16_array, get_u32_array_opt, get_u64_array_opt,
};
use crate::error::{self, Error, Result};
use crate::otlp::ProtoBuffer;
use crate::otlp::attributes::{Attribute32Arrays, encode_key_value};
use crate::otlp::common::{ChildIndexIter, SortedBatchCursor};
use crate::otlp::metrics::data_points::histogram::ListValueAccessor;
use crate::otlp::metrics::exemplar::{ExemplarArrays, proto_encode_exemplar};
use crate::proto::consts::field_num::metrics::{
    EXP_HISTOGRAM_BUCKET_BUCKET_COUNTS, EXP_HISTOGRAM_BUCKET_OFFSET, EXP_HISTOGRAM_DP_ATTRIBUTES,
    EXP_HISTOGRAM_DP_COUNT, EXP_HISTOGRAM_DP_EXEMPLARS, EXP_HISTOGRAM_DP_FLAGS,
    EXP_HISTOGRAM_DP_MAX, EXP_HISTOGRAM_DP_MIN, EXP_HISTOGRAM_DP_NEGATIVE,
    EXP_HISTOGRAM_DP_POSITIVE, EXP_HISTOGRAM_DP_SCALE, EXP_HISTOGRAM_DP_START_TIME_UNIX_NANO,
    EXP_HISTOGRAM_DP_SUM, EXP_HISTOGRAM_DP_TIME_UNIX_NANO, EXP_HISTOGRAM_DP_ZERO_COUNT,
    EXP_HISTOGRAM_DP_ZERO_THRESHOLD,
};
use crate::proto::consts::wire_types;
use crate::proto_encode_len_delimited_unknown_size;
use crate::schema::consts;
use arrow::array::{
    Array, ArrayRef, Float64Array, Int32Array, ListArray, RecordBatch, StructArray,
    TimestampNanosecondArray, UInt16Array, UInt32Array, UInt64Array,
};
use arrow::datatypes::{DataType, Field, FieldRef, Fields, UInt64Type};
use snafu::OptionExt;

pub struct ExpHistogramDpArrays<'a> {
    pub id: Option<&'a UInt32Array>,
    pub parent_id: &'a UInt16Array,
    pub start_time_unix_nano: Option<&'a TimestampNanosecondArray>,
    pub time_unix_nano: Option<&'a TimestampNanosecondArray>,
    pub histogram_count: Option<&'a UInt64Array>,
    pub histogram_sum: Option<&'a Float64Array>,
    pub exp_histogram_scale: Option<&'a Int32Array>,
    pub exp_histogram_zero_count: Option<&'a UInt64Array>,
    pub exp_histogram_positive: Option<PositiveNegativeArrayAccess<'a>>,
    pub exp_histogram_negative: Option<PositiveNegativeArrayAccess<'a>>,
    pub flags: Option<&'a UInt32Array>,
    pub histogram_min: Option<&'a Float64Array>,
    pub histogram_max: Option<&'a Float64Array>,
    pub zero_threshold: Option<&'a Float64Array>,
}

impl<'a> TryFrom<&'a RecordBatch> for ExpHistogramDpArrays<'a> {
    type Error = Error;

    fn try_from(rb: &'a RecordBatch) -> Result<Self> {
        let id = get_u32_array_opt(rb, consts::ID)?;
        let parent_id = get_u16_array(rb, consts::PARENT_ID)?;
        let start_time_unix_nano =
            get_timestamp_nanosecond_array_opt(rb, consts::START_TIME_UNIX_NANO)?;
        let time_unix_nano = get_timestamp_nanosecond_array_opt(rb, consts::TIME_UNIX_NANO)?;
        let histogram_count = get_u64_array_opt(rb, consts::HISTOGRAM_COUNT)?;
        let histogram_sum = get_f64_array_opt(rb, consts::HISTOGRAM_SUM)?;
        let exp_histogram_scale = get_i32_array_opt(rb, consts::EXP_HISTOGRAM_SCALE)?;
        let exp_histogram_zero_count = get_u64_array_opt(rb, consts::EXP_HISTOGRAM_ZERO_COUNT)?;
        let exp_histogram_positive = rb
            .column_by_name(consts::EXP_HISTOGRAM_POSITIVE)
            .map(|arr| PositiveNegativeArrayAccess::try_new(arr, consts::EXP_HISTOGRAM_POSITIVE))
            .transpose()?;
        let exp_histogram_negative = rb
            .column_by_name(consts::EXP_HISTOGRAM_NEGATIVE)
            .map(|arr| PositiveNegativeArrayAccess::try_new(arr, consts::EXP_HISTOGRAM_NEGATIVE))
            .transpose()?;
        let flags = get_u32_array_opt(rb, consts::FLAGS)?;
        let histogram_min = get_f64_array_opt(rb, consts::HISTOGRAM_MIN)?;
        let histogram_max = get_f64_array_opt(rb, consts::HISTOGRAM_MAX)?;
        let zero_threshold = get_f64_array_opt(rb, consts::EXP_HISTOGRAM_ZERO_THRESHOLD)?;

        Ok(Self {
            id,
            parent_id,
            start_time_unix_nano,
            time_unix_nano,
            histogram_count,
            histogram_sum,
            exp_histogram_scale,
            exp_histogram_zero_count,
            exp_histogram_positive,
            exp_histogram_negative,
            flags,
            histogram_min,
            histogram_max,
            zero_threshold,
        })
    }
}

pub struct PositiveNegativeArrayAccess<'a> {
    offset_array: &'a Int32Array,
    bucket_count: ListValueAccessor<'a, UInt64Type>,
}

impl<'a> PositiveNegativeArrayAccess<'a> {
    fn bucket_counts_data_type() -> DataType {
        DataType::List(
            FieldRef::new(Field::new("", DataType::UInt64, true)), //todo: find the inner name here
        )
    }

    fn data_type() -> DataType {
        DataType::Struct(Fields::from(vec![
            Field::new(consts::EXP_HISTOGRAM_OFFSET, DataType::Int32, true),
            Field::new(
                consts::EXP_HISTOGRAM_BUCKET_COUNTS,
                Self::bucket_counts_data_type(),
                true,
            ),
        ]))
    }

    fn try_new(array: &'a ArrayRef, column_name: &str) -> Result<Self> {
        let struct_array = array
            .as_any()
            .downcast_ref::<StructArray>()
            .with_context(|| error::ColumnDataTypeMismatchSnafu {
                name: column_name,
                expect: Self::data_type(),
                actual: array.data_type().clone(),
            })?;

        let offset_array = struct_array
            .column_by_name(consts::EXP_HISTOGRAM_OFFSET)
            .context(error::ColumnNotFoundSnafu {
                name: consts::EXP_HISTOGRAM_OFFSET,
            })?;

        let offset_array = offset_array.as_any().downcast_ref::<Int32Array>().context(
            error::ColumnDataTypeMismatchSnafu {
                name: consts::EXP_HISTOGRAM_OFFSET,
                expect: DataType::Int32,
                actual: offset_array.data_type().clone(),
            },
        )?;

        let bucket_count_array = struct_array
            .column_by_name(consts::EXP_HISTOGRAM_BUCKET_COUNTS)
            .context(error::ColumnNotFoundSnafu {
                name: consts::EXP_HISTOGRAM_BUCKET_COUNTS,
            })?;

        let bucket_count_array = bucket_count_array
            .as_any()
            .downcast_ref::<ListArray>()
            .with_context(|| error::ColumnDataTypeMismatchSnafu {
                name: consts::EXP_HISTOGRAM_BUCKET_COUNTS,
                expect: Self::bucket_counts_data_type(),
                actual: bucket_count_array.data_type().clone(),
            })?;

        let bucket_count = ListValueAccessor::try_new_from_list(bucket_count_array)?;
        Ok(Self {
            offset_array,
            bucket_count,
        })
    }
}

pub(crate) fn proto_encode_exp_hist_data_point(
    index: usize,
    exp_hist_dp_arrays: &ExpHistogramDpArrays<'_>,
    attrs: Option<&Attribute32Arrays<'_>>,
    attrs_cursor: &mut SortedBatchCursor,
    exemplar_arrays: Option<&ExemplarArrays<'_>>,
    exemplar_cursor: &mut SortedBatchCursor,
    exemplar_attr_arrays: Option<&Attribute32Arrays<'_>>,
    exemplar_attrs_cursor: &mut SortedBatchCursor,
    result_buf: &mut ProtoBuffer,
) -> Result<()> {
    if let Some(attrs) = attrs {
        if let Some(id) = exp_hist_dp_arrays.id.value_at(index) {
            let attrs_index_iter = ChildIndexIter::new(id, &attrs.parent_id, attrs_cursor);
            for attrs_index in attrs_index_iter {
                proto_encode_len_delimited_unknown_size!(
                    EXP_HISTOGRAM_DP_ATTRIBUTES,
                    encode_key_value(attrs, attrs_index, result_buf)?,
                    result_buf
                );
            }
        }
    }

    if let Some(col) = exp_hist_dp_arrays.start_time_unix_nano {
        if let Some(val) = col.value_at(index) {
            result_buf.encode_field_tag(EXP_HISTOGRAM_DP_START_TIME_UNIX_NANO, wire_types::FIXED64);
            result_buf.extend_from_slice(&val.to_le_bytes());
        }
    }

    if let Some(col) = exp_hist_dp_arrays.time_unix_nano {
        if let Some(val) = col.value_at(index) {
            result_buf.encode_field_tag(EXP_HISTOGRAM_DP_TIME_UNIX_NANO, wire_types::FIXED64);
            result_buf.extend_from_slice(&val.to_le_bytes());
        }
    }

    if let Some(col) = exp_hist_dp_arrays.histogram_count {
        if let Some(val) = col.value_at(index) {
            result_buf.encode_field_tag(EXP_HISTOGRAM_DP_COUNT, wire_types::FIXED64);
            result_buf.extend_from_slice(&val.to_le_bytes());
        }
    }

    if let Some(col) = exp_hist_dp_arrays.histogram_sum {
        if let Some(val) = col.value_at(index) {
            result_buf.encode_field_tag(EXP_HISTOGRAM_DP_SUM, wire_types::FIXED64);
            result_buf.extend_from_slice(&val.to_le_bytes());
        }
    }

    if let Some(col) = exp_hist_dp_arrays.exp_histogram_scale {
        if let Some(val) = col.value_at(index) {
            result_buf.encode_field_tag(EXP_HISTOGRAM_DP_SCALE, wire_types::VARINT);
            result_buf.encode_sint32(val);
        }
    }

    if let Some(col) = exp_hist_dp_arrays.exp_histogram_zero_count {
        if let Some(val) = col.value_at(index) {
            result_buf.encode_field_tag(EXP_HISTOGRAM_DP_ZERO_COUNT, wire_types::FIXED64);
            result_buf.extend_from_slice(&val.to_le_bytes());
        }
    }

    if let Some(bucket_arrays) = exp_hist_dp_arrays.exp_histogram_positive.as_ref() {
        proto_encode_len_delimited_unknown_size!(
            EXP_HISTOGRAM_DP_POSITIVE,
            proto_encode_buckets(index, bucket_arrays, result_buf),
            result_buf
        )
    }

    if let Some(bucket_arrays) = exp_hist_dp_arrays.exp_histogram_negative.as_ref() {
        proto_encode_len_delimited_unknown_size!(
            EXP_HISTOGRAM_DP_NEGATIVE,
            proto_encode_buckets(index, bucket_arrays, result_buf),
            result_buf
        );
    }

    if let Some(exemplar_arrays) = exemplar_arrays {
        if let Some(id) = exp_hist_dp_arrays.id.value_at(index) {
            let exemplar_index_iter =
                ChildIndexIter::new(id, &exemplar_arrays.parent_id, exemplar_cursor);
            for exemplar_index in exemplar_index_iter {
                proto_encode_len_delimited_unknown_size!(
                    EXP_HISTOGRAM_DP_EXEMPLARS,
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

    if let Some(col) = exp_hist_dp_arrays.flags {
        if let Some(val) = col.value_at(index) {
            result_buf.encode_field_tag(EXP_HISTOGRAM_DP_FLAGS, wire_types::VARINT);
            result_buf.encode_varint(val as u64);
        }
    }

    if let Some(col) = exp_hist_dp_arrays.histogram_min {
        if let Some(val) = col.value_at(index) {
            result_buf.encode_field_tag(EXP_HISTOGRAM_DP_MIN, wire_types::FIXED64);
            result_buf.extend_from_slice(&val.to_le_bytes());
        }
    }

    if let Some(col) = exp_hist_dp_arrays.histogram_max {
        if let Some(val) = col.value_at(index) {
            result_buf.encode_field_tag(EXP_HISTOGRAM_DP_MAX, wire_types::FIXED64);
            result_buf.extend_from_slice(&val.to_le_bytes());
        }
    }

    if let Some(col) = exp_hist_dp_arrays.zero_threshold {
        if let Some(val) = col.value_at(index) {
            result_buf.encode_field_tag(EXP_HISTOGRAM_DP_ZERO_THRESHOLD, wire_types::FIXED64);
            result_buf.extend_from_slice(&val.to_le_bytes());
        }
    }

    Ok(())
}

fn proto_encode_buckets(
    index: usize,
    buckets_arrays: &PositiveNegativeArrayAccess<'_>,
    result_buf: &mut ProtoBuffer,
) {
    if let Some(val) = buckets_arrays.offset_array.value_at(index) {
        result_buf.encode_field_tag(EXP_HISTOGRAM_BUCKET_OFFSET, wire_types::VARINT);
        result_buf.encode_sint32(val);
    }

    if buckets_arrays.bucket_count.list.is_valid(index) {
        let value_offsets = buckets_arrays.bucket_count.list.value_offsets();
        let start = value_offsets[index] as usize;
        let end = value_offsets[index + 1] as usize;

        for i in start..end {
            if buckets_arrays.bucket_count.value.is_valid(i) {
                let val = buckets_arrays.bucket_count.value.value(i);
                result_buf.encode_field_tag(EXP_HISTOGRAM_BUCKET_BUCKET_COUNTS, wire_types::VARINT);
                result_buf.encode_varint(val);
            }
        }
    }
}
