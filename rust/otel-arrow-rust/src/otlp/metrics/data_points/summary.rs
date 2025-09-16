// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::arrays::{
    NullableArrayAccessor, get_f64_array, get_f64_array_opt, get_timestamp_nanosecond_array,
    get_timestamp_nanosecond_array_opt, get_u16_array, get_u16_array_opt, get_u32_array,
    get_u32_array_opt, get_u64_array, get_u64_array_opt,
};
use crate::error::{self, Error, Result};
use crate::otlp::ProtoBuffer;
use crate::otlp::attributes::store::Attribute32Store;
use crate::otlp::attributes::{Attribute16Arrays, Attribute32Arrays, encode_key_value};
use crate::otlp::common::{ChildIndexIter, SortedBatchCursor};
use crate::otlp::metrics::AppendAndGet;
use crate::otlp::metrics::data_points::data_point_store::SummaryDataPointsStore;
use crate::proto::consts::field_num::metrics::{
    SUMMARY_DP_ATTRIBUTES, SUMMARY_DP_COUNT, SUMMARY_DP_FLAGS, SUMMARY_DP_QUANTILE_VALUES,
    SUMMARY_DP_START_TIME_UNIX_NANO, SUMMARY_DP_SUM, SUMMARY_DP_TIME_UNIX_NANO,
    VALUE_AT_QUANTILE_QUANTILE, VALUE_AT_QUANTILE_VALUE,
};
use crate::proto::consts::wire_types;
use crate::proto::opentelemetry::metrics::v1::summary_data_point::ValueAtQuantile;
use crate::proto_encode_len_delimited_unknown_size;
use crate::schema::consts;
use arrow::array::{
    Array, ArrayRef, Float64Array, ListArray, RecordBatch, StructArray, TimestampNanosecondArray,
    UInt16Array, UInt32Array, UInt64Array,
};
use snafu::OptionExt;

pub struct SummaryDpArrays<'a> {
    pub id: Option<&'a UInt32Array>,
    pub parent_id: &'a UInt16Array,
    pub start_time_unix_nano: Option<&'a TimestampNanosecondArray>,
    pub time_unix_nano: Option<&'a TimestampNanosecondArray>,
    pub summary_count: Option<&'a UInt64Array>,
    pub summary_sum: Option<&'a Float64Array>,
    pub summary_quantile_values: Option<QuantileArrays<'a>>,
    pub flags: Option<&'a UInt32Array>,
}

impl<'a> TryFrom<&'a RecordBatch> for SummaryDpArrays<'a> {
    type Error = Error;

    fn try_from(rb: &'a RecordBatch) -> Result<Self> {
        let id = get_u32_array_opt(rb, consts::ID)?;
        let parent_id = get_u16_array(rb, consts::PARENT_ID)?;
        let start_time_unix_nano =
            get_timestamp_nanosecond_array_opt(rb, consts::START_TIME_UNIX_NANO)?;
        let time_unix_nano = get_timestamp_nanosecond_array_opt(rb, consts::TIME_UNIX_NANO)?;
        let summary_count = get_u64_array_opt(rb, consts::SUMMARY_COUNT)?;
        let summary_sum = get_f64_array_opt(rb, consts::SUMMARY_SUM)?;
        let flags = get_u32_array_opt(rb, consts::FLAGS)?;
        let summary_quantile_values = rb
            .column_by_name(consts::SUMMARY_QUANTILE_VALUES)
            .map(QuantileArrays::try_new)
            .transpose()?;

        Ok(Self {
            id,
            parent_id,
            start_time_unix_nano,
            time_unix_nano,
            summary_count,
            summary_sum,
            summary_quantile_values,
            flags,
        })
    }
}

// TODO delete this
impl SummaryDataPointsStore {
    // see https://github.com/open-telemetry/otel-arrow/blob/985aa1500a012859cec44855e187eacf46eda7c8/pkg/otel/metrics/otlp/summary.go#L117
    pub fn from_record_batch(
        rb: &RecordBatch,
        attr_store: &mut Attribute32Store,
    ) -> Result<SummaryDataPointsStore> {
        let mut store = SummaryDataPointsStore::default();
        let mut prev_parent_id = 0;

        let id_arr_opt = get_u32_array_opt(rb, consts::ID)?;
        let delta_id_arr = get_u16_array(rb, consts::PARENT_ID)?;
        let start_time_unix_nano_arr =
            get_timestamp_nanosecond_array_opt(rb, consts::START_TIME_UNIX_NANO)?;
        let time_unix_nano_arr = get_timestamp_nanosecond_array(rb, consts::TIME_UNIX_NANO)?;
        let summary_count_arr = get_u64_array(rb, consts::SUMMARY_COUNT)?;
        let sum_arr = get_f64_array(rb, consts::SUMMARY_SUM)?;
        let quantile_arr =
            QuantileArrays::try_new(rb.column_by_name(consts::SUMMARY_QUANTILE_VALUES).context(
                error::ColumnNotFoundSnafu {
                    name: consts::SUMMARY_QUANTILE_VALUES,
                },
            )?)?;
        let flag_arr = get_u32_array(rb, consts::FLAGS)?;

        for idx in 0..rb.num_rows() {
            let delta = delta_id_arr.value_at_or_default(idx);
            let parent_id = prev_parent_id + delta;
            prev_parent_id = parent_id;
            let nbdps = store.get_or_default(parent_id);

            let sdp = nbdps.append_and_get();
            sdp.start_time_unix_nano = start_time_unix_nano_arr.value_at_or_default(idx) as u64;
            sdp.time_unix_nano = time_unix_nano_arr.value_at_or_default(idx) as u64;
            sdp.count = summary_count_arr.value_at_or_default(idx);
            sdp.sum = sum_arr.value_at_or_default(idx);
            if let Some(quantile) = quantile_arr.value_at(idx) {
                sdp.quantile_values = quantile;
            }
            sdp.flags = flag_arr.value_at_or_default(idx);
            if let Some(id) = id_arr_opt.value_at(idx) {
                if let Some(attr) = attr_store.attribute_by_delta_id(id) {
                    sdp.attributes = attr.to_vec();
                }
            }
        }

        Ok(store)
    }
}

pub struct QuantileArrays<'a> {
    list_array: &'a ListArray,
    quantile_array: &'a Float64Array,
    value_array: &'a Float64Array,
}

impl<'a> QuantileArrays<'a> {
    fn try_new(array: &'a ArrayRef) -> Result<Self> {
        let list = array
            .as_any()
            .downcast_ref::<ListArray>()
            .with_context(|| error::InvalidQuantileTypeSnafu {
                message: array.data_type().to_string(),
            })?;

        let struct_array = list
            .values()
            .as_any()
            .downcast_ref::<StructArray>()
            .with_context(|| error::InvalidQuantileTypeSnafu {
                message: array.data_type().to_string(),
            })?;
        let downcast_f64 =
            |struct_array: &'a StructArray, name: &str| -> Result<&'a Float64Array> {
                let field_column = struct_array
                    .column_by_name(name)
                    .context(error::ColumnNotFoundSnafu { name })?;

                field_column
                    .as_any()
                    .downcast_ref::<Float64Array>()
                    .with_context(|| error::InvalidQuantileTypeSnafu {
                        message: field_column.data_type().to_string(),
                    })
            };

        let quantile = downcast_f64(struct_array, consts::SUMMARY_QUANTILE)?;
        let value = downcast_f64(struct_array, consts::SUMMARY_VALUE)?;
        assert_eq!(value.len(), quantile.len());
        Ok(Self {
            list_array: list,
            quantile_array: quantile,
            value_array: value,
        })
    }
}

impl QuantileArrays<'_> {
    fn value_at(&self, idx: usize) -> Option<Vec<ValueAtQuantile>> {
        if !self.list_array.is_valid(idx) {
            return None;
        }
        let start = self.list_array.value_offsets()[idx];
        let end = self.list_array.value_offsets()[idx + 1];

        let quantiles = (start..end)
            .map(|idx| {
                let idx = idx as usize;
                ValueAtQuantile {
                    quantile: self.quantile_array.value_at_or_default(idx),
                    value: self.value_array.value_at_or_default(idx),
                }
            })
            .collect::<Vec<_>>();
        Some(quantiles)
    }
}

pub(crate) fn proto_encode_summary_data_point(
    index: usize,
    summary_dp_arrays: &SummaryDpArrays<'_>,
    attr_arrays: Option<&Attribute32Arrays<'_>>,
    attrs_cursor: &mut SortedBatchCursor,
    result_buf: &mut ProtoBuffer,
) -> Result<()> {
    if let Some(attrs) = attr_arrays {
        if let Some(id) = summary_dp_arrays.id.value_at(index) {
            let attrs_index_iter = ChildIndexIter::new(id, &attrs.parent_id, attrs_cursor);
            for attrs_index in attrs_index_iter {
                proto_encode_len_delimited_unknown_size!(
                    SUMMARY_DP_ATTRIBUTES,
                    encode_key_value(attrs, attrs_index, result_buf)?,
                    result_buf
                );
            }
        }
    }

    if let Some(col) = summary_dp_arrays.start_time_unix_nano {
        if let Some(val) = col.value_at(index) {
            result_buf.encode_field_tag(SUMMARY_DP_START_TIME_UNIX_NANO, wire_types::FIXED64);
            result_buf.extend_from_slice(&val.to_le_bytes());
        }
    }

    if let Some(col) = summary_dp_arrays.time_unix_nano {
        if let Some(val) = col.value_at(index) {
            result_buf.encode_field_tag(SUMMARY_DP_TIME_UNIX_NANO, wire_types::FIXED64);
            result_buf.extend_from_slice(&val.to_le_bytes());
        }
    }

    if let Some(col) = summary_dp_arrays.summary_count {
        if let Some(val) = col.value_at(index) {
            result_buf.encode_field_tag(SUMMARY_DP_COUNT, wire_types::FIXED64);
            result_buf.extend_from_slice(&val.to_le_bytes());
        }
    }

    if let Some(col) = summary_dp_arrays.summary_sum {
        if let Some(val) = col.value_at(index) {
            result_buf.encode_field_tag(SUMMARY_DP_SUM, wire_types::FIXED64);
            result_buf.extend_from_slice(&val.to_le_bytes());
        }
    }

    if let Some(quantile_arrays) = &summary_dp_arrays.summary_quantile_values {
        if quantile_arrays.list_array.is_valid(index) {
            let value_offsets = quantile_arrays.list_array.value_offsets();
            let start = value_offsets[index];
            let end = value_offsets[index + 1];

            for i in start..end {
                proto_encode_len_delimited_unknown_size!(
                    SUMMARY_DP_QUANTILE_VALUES,
                    proto_encode_value_quantile(i as usize, &quantile_arrays, result_buf),
                    result_buf
                );
            }
        }
    }

    if let Some(col) = summary_dp_arrays.flags {
        if let Some(val) = col.value_at(index) {
            result_buf.encode_field_tag(SUMMARY_DP_FLAGS, wire_types::VARINT);
            result_buf.encode_varint(val as u64);
        }
    }

    Ok(())
}

pub(crate) fn proto_encode_value_quantile(
    index: usize,
    quantile_arrays: &QuantileArrays<'_>,
    result_buf: &mut ProtoBuffer,
) {
    if let Some(val) = quantile_arrays.quantile_array.value_at(index) {
        result_buf.encode_field_tag(VALUE_AT_QUANTILE_QUANTILE, wire_types::FIXED64);
        result_buf.extend_from_slice(&val.to_le_bytes());
    }

    if let Some(val) = quantile_arrays.value_array.value_at(index) {
        result_buf.encode_field_tag(VALUE_AT_QUANTILE_VALUE, wire_types::FIXED64);
        result_buf.extend_from_slice(&val.to_le_bytes());
    }
}
