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
    NullableArrayAccessor, get_f64_array_opt, get_timestamp_nanosecond_array,
    get_timestamp_nanosecond_array_opt, get_u16_array, get_u32_array, get_u32_array_opt,
    get_u64_array,
};
use crate::error;
use crate::otlp::attributes::store::Attribute32Store;
use crate::otlp::metrics::AppendAndGet;
use crate::otlp::metrics::data_points::data_point_store::HistogramDataPointsStore;
use crate::otlp::metrics::exemplar::ExemplarsStore;
use crate::schema::consts;
use arrow::array::{Array, ArrayRef, ListArray, PrimitiveArray, RecordBatch};
use arrow::datatypes::{
    ArrowNativeType, ArrowPrimitiveType, DataType, Field, FieldRef, Float64Type, UInt64Type,
};
use snafu::OptionExt;

impl HistogramDataPointsStore {
    // See https://github.com/open-telemetry/otel-arrow/blob/985aa1500a012859cec44855e187eacf46eda7c8/pkg/otel/metrics/otlp/histogram.go#L139
    pub fn from_record_batch(
        rb: &RecordBatch,
        exemplar_store: &mut ExemplarsStore,
        attrs_store: &Attribute32Store,
    ) -> error::Result<HistogramDataPointsStore> {
        let mut store = HistogramDataPointsStore::default();

        let id_array_opt = get_u32_array_opt(rb, consts::ID)?;
        let delta_id = get_u16_array(rb, consts::PARENT_ID)?;
        let start_time_unix_nano =
            get_timestamp_nanosecond_array_opt(rb, consts::START_TIME_UNIX_NANO)?;
        let time_unix_nano = get_timestamp_nanosecond_array(rb, consts::TIME_UNIX_NANO)?;
        let histogram_count = get_u64_array(rb, consts::HISTOGRAM_COUNT)?;
        let sum = get_f64_array_opt(rb, consts::HISTOGRAM_SUM)?;
        let bucket_counts_arr: ListValueAccessor<UInt64Type> = ListValueAccessor::try_new(
            rb.column_by_name(consts::HISTOGRAM_BUCKET_COUNTS).context(
                error::ColumnNotFoundSnafu {
                    name: consts::HISTOGRAM_BUCKET_COUNTS,
                },
            )?,
        )?;
        let explicit_bounds_arr: ListValueAccessor<Float64Type> = ListValueAccessor::try_new(
            rb.column_by_name(consts::HISTOGRAM_EXPLICIT_BOUNDS)
                .context(error::ColumnNotFoundSnafu {
                    name: consts::HISTOGRAM_EXPLICIT_BOUNDS,
                })?,
        )?;
        let flags_arr = get_u32_array(rb, consts::FLAGS)?;
        let max_arr = get_f64_array_opt(rb, consts::HISTOGRAM_MAX)?;
        let min_arr = get_f64_array_opt(rb, consts::HISTOGRAM_MIN)?;

        let mut prev_parent_id = 0;
        let mut last_id = 0;

        for idx in 0..rb.num_rows() {
            let delta = delta_id.value_at_or_default(idx);
            let parent_id = prev_parent_id + delta;
            prev_parent_id = parent_id;

            // Creates a new HistogramDataPoint and append to the list.
            let hdps = store.get_or_default(parent_id).append_and_get();

            hdps.start_time_unix_nano = start_time_unix_nano.value_at_or_default(idx) as u64;
            hdps.time_unix_nano = time_unix_nano.value_at_or_default(idx) as u64;
            hdps.count = histogram_count.value_at_or_default(idx);
            hdps.sum = sum.value_at(idx);
            if let Some(bucket_counts) = bucket_counts_arr.value_at_opt(idx) {
                hdps.bucket_counts = bucket_counts;
            }
            if let Some(explicit_bounds) = explicit_bounds_arr.value_at_opt(idx) {
                hdps.explicit_bounds = explicit_bounds;
            }

            hdps.flags = flags_arr.value_at_or_default(idx);
            hdps.max = max_arr.value_at(idx);
            hdps.min = min_arr.value_at(idx);

            if let Some(id) = id_array_opt.value_at(idx) {
                last_id += id;
                let exemplars = exemplar_store.get_or_create_exemplar_by_id(last_id);
                hdps.exemplars = std::mem::take(exemplars);
                if let Some(attrs) = attrs_store.attributes_by_id(last_id) {
                    hdps.attributes = attrs.to_vec();
                }
            }
        }

        Ok(store)
    }
}

/// Helper to access the element in a list array.
pub struct ListValueAccessor<'a, T: ArrowPrimitiveType> {
    list: &'a ListArray,
    value: &'a PrimitiveArray<T>,
}

impl<'a, T> ListValueAccessor<'a, T>
where
    T: ArrowPrimitiveType,
{
    pub fn try_new(list: &'a ArrayRef) -> error::Result<Self> {
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

    pub fn try_new_from_list(list: &'a ListArray) -> error::Result<Self> {
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
