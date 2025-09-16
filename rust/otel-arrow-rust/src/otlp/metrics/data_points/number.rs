// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::arrays::{
    MaybeDictArrayAccessor, NullableArrayAccessor, get_f64_array_opt, get_i64_array_opt,
    get_timestamp_nanosecond_array, get_timestamp_nanosecond_array_opt, get_u16_array,
    get_u32_array, get_u32_array_opt,
};
use crate::error::{self, Error, Result};
use crate::otlp::ProtoBuffer;
use crate::otlp::attributes::store::Attribute32Store;
use crate::otlp::attributes::{Attribute32Arrays, encode_key_value};
use crate::otlp::common::{ChildIndexIter, SortedBatchCursor};
use crate::otlp::metrics::data_points::data_point_store::NumberDataPointsStore;
use crate::otlp::metrics::exemplar::{ExemplarArrays, ExemplarsStore, proto_encode_exemplar};
use crate::proto::consts::field_num::metrics::{
    NUMBER_DP_AS_DOUBLE, NUMBER_DP_AS_INT, NUMBER_DP_ATTRIBUTES, NUMBER_DP_EXEMPLARS,
    NUMBER_DP_FLAGS, NUMBER_DP_START_TIME_UNIX_NANO, NUMBER_DP_TIME_UNIX_NANO,
};
use crate::proto::consts::wire_types;
use crate::proto::opentelemetry::metrics::v1::NumberDataPoint;
use crate::proto::opentelemetry::metrics::v1::number_data_point::Value;
use crate::proto_encode_len_delimited_unknown_size;
use crate::schema::consts;
use arrow::array::{
    Float64Array, Int64Array, RecordBatch, TimestampNanosecondArray, UInt16Array, UInt32Array,
};

pub struct NumberDpArrays<'a> {
    pub id: Option<&'a UInt32Array>,
    pub parent_id: &'a UInt16Array,
    pub start_time_unix_nano: Option<&'a TimestampNanosecondArray>,
    pub time_unix_nano: Option<&'a TimestampNanosecondArray>,
    pub int_value: Option<&'a Int64Array>,
    pub double_value: Option<&'a Float64Array>,
    pub flags: Option<&'a UInt32Array>,
}

impl<'a> TryFrom<&'a RecordBatch> for NumberDpArrays<'a> {
    type Error = Error;

    fn try_from(rb: &'a RecordBatch) -> Result<Self> {
        let id = get_u32_array_opt(rb, consts::ID)?;
        let parent_id = get_u16_array(rb, consts::PARENT_ID)?;
        let start_time_unix_nano =
            get_timestamp_nanosecond_array_opt(rb, consts::START_TIME_UNIX_NANO)?;
        let time_unix_nano = get_timestamp_nanosecond_array_opt(rb, consts::TIME_UNIX_NANO)?;
        let int_value = get_i64_array_opt(rb, consts::INT_VALUE)?;
        let double_value = get_f64_array_opt(rb, consts::DOUBLE_VALUE)?;
        let flags = get_u32_array_opt(rb, consts::FLAGS)?;

        Ok(Self {
            id,
            parent_id,
            start_time_unix_nano,
            time_unix_nano,
            int_value,
            double_value,
            flags,
        })
    }
}

impl NumberDataPointsStore {
    /// Ref: https://github.com/open-telemetry/otel-arrow/blob/985aa1500a012859cec44855e187eacf46eda7c8/pkg/otel/metrics/otlp/number_data_point.go#L110
    pub fn from_record_batch(
        rb: &RecordBatch,
        exemplar_store: &mut ExemplarsStore,
        attribute_store: &Attribute32Store,
    ) -> Result<NumberDataPointsStore> {
        let mut store = NumberDataPointsStore::default();

        let id_array = get_u32_array(rb, consts::ID)?;
        let parent_id_array = get_u16_array(rb, consts::PARENT_ID)?;
        let start_time_unix_nano_array =
            get_timestamp_nanosecond_array_opt(rb, consts::START_TIME_UNIX_NANO)?;
        let time_unix_nano_array = get_timestamp_nanosecond_array(rb, consts::TIME_UNIX_NANO)?;

        // todo(hl): The receiver code of otelarrow also handles dictionary arrays for int_value field
        // but the exporter side seems only encode to Int64Array: https://github.com/open-telemetry/otel-arrow/blob/79b50d99dde17c5bb085a0204db406d8f6ad880b/pkg/otel/metrics/arrow/number_data_point.go#L138
        let int_value = get_i64_array_opt(rb, consts::INT_VALUE)?;
        let double_value = get_f64_array_opt(rb, consts::DOUBLE_VALUE)?;
        let flags = get_u32_array_opt(rb, consts::FLAGS)?;

        let mut last_id = 0;
        let mut prev_parent_id = 0;

        for idx in 0..rb.num_rows() {
            let id = id_array.value_at(idx);
            let delta = parent_id_array.value_at(idx).unwrap_or_default();
            let parent_id = prev_parent_id + delta;
            prev_parent_id = parent_id;

            let nbdps = store.get_or_default(parent_id);
            let mut nbdp = NumberDataPoint {
                attributes: vec![],
                start_time_unix_nano: start_time_unix_nano_array.value_at(idx).unwrap_or_default()
                    as u64,
                time_unix_nano: time_unix_nano_array.value_at(idx).unwrap_or_default() as u64,
                exemplars: vec![],
                flags: flags.value_at_or_default(idx),
                value: None,
            };

            match (int_value.value_at(idx), double_value.value_at(idx)) {
                (Some(int), None) => {
                    nbdp.value = Some(Value::AsInt(int));
                }
                (None, Some(double)) => {
                    nbdp.value = Some(Value::AsDouble(double));
                }
                (Some(_), Some(_)) => {
                    panic!("unexpected")
                }
                (None, None) => {
                    nbdp.value = None;
                }
            }

            if let Some(id) = id {
                last_id += id;
                let exemplars = exemplar_store.get_or_create_exemplar_by_id(last_id);
                nbdp.exemplars.extend(std::mem::take(exemplars));

                if let Some(attr) = attribute_store.attribute_by_id(last_id) {
                    nbdp.attributes = attr.to_vec();
                }
            }
            nbdps.push(nbdp);
        }

        Ok(store)
    }
}

pub(crate) fn proto_encode_number_data_point(
    index: usize,
    number_dp_arrays: &NumberDpArrays<'_>,
    attr_arrays: Option<&Attribute32Arrays<'_>>,
    attrs_cursor: &mut SortedBatchCursor,
    exemplar_arrays: Option<&ExemplarArrays<'_>>,
    exemplar_cursor: &mut SortedBatchCursor,
    exemplar_attr_arrays: Option<&Attribute32Arrays<'_>>,
    exemplar_attrs_cursor: &mut SortedBatchCursor,
    result_buf: &mut ProtoBuffer,
) -> Result<()> {
    if let Some(attrs) = attr_arrays {
        if let Some(id) = number_dp_arrays.id.value_at(index) {
            let attrs_index_iter = ChildIndexIter::new(id, &attrs.parent_id, attrs_cursor);
            for attrs_index in attrs_index_iter {
                proto_encode_len_delimited_unknown_size!(
                    NUMBER_DP_ATTRIBUTES,
                    encode_key_value(attrs, attrs_index, result_buf)?,
                    result_buf
                );
            }
        }
    }

    if let Some(col) = number_dp_arrays.start_time_unix_nano {
        if let Some(val) = col.value_at(index) {
            result_buf.encode_field_tag(NUMBER_DP_START_TIME_UNIX_NANO, wire_types::FIXED64);
            result_buf.extend_from_slice(&val.to_le_bytes());
        }
    }

    if let Some(col) = number_dp_arrays.time_unix_nano {
        if let Some(val) = col.value_at(index) {
            result_buf.encode_field_tag(NUMBER_DP_TIME_UNIX_NANO, wire_types::FIXED64);
            result_buf.extend_from_slice(&val.to_le_bytes());
        }
    }

    let mut value_is_double = false;
    if let Some(col) = number_dp_arrays.double_value {
        if let Some(val) = col.value_at(index) {
            value_is_double = true;
            result_buf.encode_field_tag(NUMBER_DP_AS_DOUBLE, wire_types::FIXED64);
            result_buf.extend_from_slice(&val.to_le_bytes());
        }
    }

    if !value_is_double {
        if let Some(col) = number_dp_arrays.int_value {
            if let Some(val) = col.value_at(index) {
                result_buf.encode_field_tag(NUMBER_DP_AS_INT, wire_types::FIXED64);
                result_buf.extend_from_slice(&val.to_le_bytes());
            }
        }
    }

    if let Some(exemplar_arrays) = exemplar_arrays {
        if let Some(id) = number_dp_arrays.id.value_at(index) {
            let parent_ids = MaybeDictArrayAccessor::Native(exemplar_arrays.parent_id);
            let exemplar_index_iter = ChildIndexIter::new(id, &parent_ids, exemplar_cursor);
            for exemplar_index in exemplar_index_iter {
                proto_encode_len_delimited_unknown_size!(
                    NUMBER_DP_EXEMPLARS,
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

    if let Some(col) = number_dp_arrays.flags {
        if let Some(val) = col.value_at(index) {
            result_buf.encode_field_tag(NUMBER_DP_FLAGS, wire_types::VARINT);
            result_buf.encode_varint(val as u64);
        }
    }

    Ok(())
}
