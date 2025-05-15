// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::arrays::{
    NullableArrayAccessor, get_f64_array, get_timestamp_nanosecond_array,
    get_timestamp_nanosecond_array_opt, get_u16_array, get_u32_array, get_u32_array_opt,
    get_u64_array,
};
use crate::error;
use crate::otlp::attributes::store::Attribute32Store;
use crate::otlp::metrics::AppendAndGet;
use crate::otlp::metrics::data_points::data_point_store::SummaryDataPointsStore;
use crate::proto::opentelemetry::metrics::v1::summary_data_point::ValueAtQuantile;
use crate::schema::consts;
use arrow::array::{Array, ArrayRef, Float64Array, ListArray, RecordBatch, StructArray};
use snafu::OptionExt;

impl SummaryDataPointsStore {
    // see https://github.com/open-telemetry/otel-arrow/blob/985aa1500a012859cec44855e187eacf46eda7c8/pkg/otel/metrics/otlp/summary.go#L117
    pub fn from_record_batch(
        rb: &RecordBatch,
        attr_store: &mut Attribute32Store,
    ) -> error::Result<SummaryDataPointsStore> {
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

struct QuantileArrays<'a> {
    list_array: &'a ListArray,
    quantile_array: &'a Float64Array,
    value_array: &'a Float64Array,
}

impl<'a> QuantileArrays<'a> {
    fn try_new(array: &'a ArrayRef) -> error::Result<Self> {
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
            |struct_array: &'a StructArray, name: &str| -> error::Result<&'a Float64Array> {
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
