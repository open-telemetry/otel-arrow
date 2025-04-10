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
    get_f64_array_opt, get_i32_array, get_timestamp_nanosecond_array,
    get_timestamp_nanosecond_array_opt, get_u16_array, get_u32_array_opt, get_u64_array,
    NullableArrayAccessor,
};
use crate::error;
use crate::otlp::attributes::store::Attribute32Store;
use crate::otlp::data_points::data_point_store::EHistogramDataPointsStore;
use crate::otlp::data_points::histogram::ListValueAccessor;
use crate::otlp::exemplar::ExemplarsStore;
use crate::otlp::metric::AppendAndGet;
use crate::schema::consts;
use arrow::array::{Array, Int32Array, ListArray, RecordBatch, StructArray};
use arrow::datatypes::{DataType, Field, FieldRef, Fields, UInt64Type};
use crate::proto::opentelemetry::metrics::v1::exponential_histogram_data_point::Buckets;
use snafu::OptionExt;

impl EHistogramDataPointsStore {
    pub fn from_record_batch(
        rb: &RecordBatch,
        exemplar_store: &mut ExemplarsStore,
        attr_store: &Attribute32Store,
    ) -> error::Result<Self> {
        let mut store = Self::default();

        let id_arr_opt = get_u32_array_opt(rb, consts::ID)?;
        let delta_arr = get_u16_array(rb, consts::PARENT_ID)?;
        let start_time_unix_nano =
            get_timestamp_nanosecond_array_opt(rb, consts::START_TIME_UNIX_NANO)?;
        let time_unix_nano = get_timestamp_nanosecond_array(rb, consts::TIME_UNIX_NANO)?;
        let histogram_count = get_u64_array(rb, consts::HISTOGRAM_COUNT)?;
        let sum_arr = get_f64_array_opt(rb, consts::HISTOGRAM_SUM)?;
        let scale_arr = get_i32_array(rb, consts::EXP_HISTOGRAM_SCALE)?;
        let zero_count_arr = get_u64_array(rb, consts::EXP_HISTOGRAM_ZERO_COUNT)?;
        let positive_arr =
            PositiveNegativeArrayAccess::try_new(rb, consts::EXP_HISTOGRAM_POSITIVE)?;
        let negative_arr =
            PositiveNegativeArrayAccess::try_new(rb, consts::EXP_HISTOGRAM_NEGATIVE)?;
        let flags_arr = get_u32_array_opt(rb, consts::FLAGS)?;
        let min_arr = get_f64_array_opt(rb, consts::HISTOGRAM_MIN)?;
        let max_arr = get_f64_array_opt(rb, consts::HISTOGRAM_MAX)?;

        let mut prev_parent_id = 0;
        let mut last_id = 0;

        for idx in 0..rb.num_rows() {
            let delta = delta_arr.value_at_or_default(idx);
            let parent_id = prev_parent_id + delta;
            prev_parent_id = parent_id;
            let ehdps = store.get_or_default(parent_id);
            let hdp = ehdps.append_and_get();
            hdp.start_time_unix_nano = start_time_unix_nano.value_at_or_default(idx) as u64;
            hdp.time_unix_nano = time_unix_nano.value_at_or_default(idx) as u64;
            hdp.count = histogram_count.value_at_or_default(idx);
            hdp.sum = sum_arr.value_at(idx);
            hdp.scale = scale_arr.value_at_or_default(idx);
            hdp.zero_count = zero_count_arr.value_at_or_default(idx);
            let (offset, bucket_counts) = positive_arr.value_at(idx);
            hdp.positive = Some(Buckets {
                offset,
                bucket_counts,
            });
            let (offset, bucket_counts) = negative_arr.value_at(idx);
            hdp.negative = Some(Buckets {
                offset,
                bucket_counts,
            });

            hdp.flags = flags_arr.value_at_or_default(idx);
            hdp.max = max_arr.value_at(idx);
            hdp.min = min_arr.value_at(idx);

            if let Some(id) = id_arr_opt.value_at(idx) {
                last_id += id;
                let exemplars = exemplar_store.get_or_create_exemplar_by_id(last_id);
                hdp.exemplars = std::mem::take(exemplars);
                if let Some(attrs) = attr_store.attribute_by_id(last_id) {
                    hdp.attributes = attrs.to_vec();
                }
            }
        }

        Ok(store)
    }
}

struct PositiveNegativeArrayAccess<'a> {
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

    fn try_new(rb: &'a RecordBatch, name: &'static str) -> error::Result<Self> {
        let array = rb
            .column_by_name(name)
            .context(error::ColumnNotFoundSnafu { name })?;
        let struct_array = array
            .as_any()
            .downcast_ref::<StructArray>()
            .with_context(|| error::ColumnDataTypeMismatchSnafu {
                name,
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

    fn value_at(&self, idx: usize) -> (i32, Vec<u64>) {
        let offset = self.offset_array.value_at_or_default(idx);
        let bucket_count = self.bucket_count.value_at_opt(idx).unwrap_or_default();
        (offset, bucket_count)
    }
}
