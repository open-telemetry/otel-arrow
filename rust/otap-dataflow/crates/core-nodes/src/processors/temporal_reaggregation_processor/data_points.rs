// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! SOA (Struct of Arrays) storage for data points and a nullable column helper.
//!
//! Data points are stored in column-oriented format so that they can be
//! converted to Arrow arrays at flush time with minimal copying. The
//! [`NullableColumn`] helper wraps a `Vec<T>` and a `BooleanBufferBuilder`
//! to support random-access writes for nullable fixed-width columns.

use std::sync::Arc;

use arrow::array::{
    Array, ArrowPrimitiveType, BooleanBufferBuilder, Float64Array, Int32Array, ListArray,
    PrimitiveArray, RecordBatch, StructArray, TimestampNanosecondArray, UInt16Array, UInt32Array,
    UInt64Array,
};
use arrow::buffer::{NullBuffer, OffsetBuffer, ScalarBuffer};
use arrow::datatypes::{DataType, Field, Float64Type, Int64Type, Schema, TimeUnit};
use otap_df_pdata::schema::consts;
use otap_df_pdata_views::views::metrics::{
    BucketsView, ExponentialHistogramDataPointView, HistogramDataPointView, NumberDataPointView,
    SummaryDataPointView, Value, ValueAtQuantileView,
};

// ---------------------------------------------------------------------------
// NullableColumn<T>
// ---------------------------------------------------------------------------

/// A column of nullable primitive values that supports random-access writes.
///
/// At flush time, converts to an Arrow [`PrimitiveArray`] via zero-copy for
/// the values buffer and a constructed null bitmap.
pub struct NullableColumn<T: ArrowPrimitiveType> {
    values: Vec<T::Native>,
    validity: BooleanBufferBuilder,
}

impl<T: ArrowPrimitiveType> NullableColumn<T> {
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            validity: BooleanBufferBuilder::new(0),
        }
    }

    pub fn push_value(&mut self, value: T::Native) {
        self.values.push(value);
        self.validity.append(true);
    }

    pub fn push_null(&mut self) {
        self.values.push(T::Native::default());
        self.validity.append(false);
    }

    pub fn set_value(&mut self, index: usize, value: T::Native) {
        self.values[index] = value;
        self.validity.set_bit(index, true);
    }

    pub fn set_null(&mut self, index: usize) {
        self.values[index] = T::Native::default();
        self.validity.set_bit(index, false);
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Convert to an Arrow [`PrimitiveArray`]. The values buffer is zero-copy.
    pub fn finish(&mut self) -> PrimitiveArray<T> {
        let values = ScalarBuffer::from(std::mem::take(&mut self.values));
        let nulls = NullBuffer::new(self.validity.finish());
        self.validity = BooleanBufferBuilder::new(0);
        PrimitiveArray::<T>::new(values, Some(nulls))
    }

    pub fn clear(&mut self) {
        self.values.clear();
        self.validity = BooleanBufferBuilder::new(0);
    }
}

// ---------------------------------------------------------------------------
// Helper: build ListArray from Vec<Vec<T>>
// ---------------------------------------------------------------------------

fn build_list_u64(data: &[Vec<u64>], field_name: &str) -> ListArray {
    let mut offsets = Vec::with_capacity(data.len() + 1);
    let mut values = Vec::new();
    offsets.push(0i32);
    for row in data {
        values.extend_from_slice(row);
        offsets.push(values.len() as i32);
    }
    let values_array = UInt64Array::from(values);
    let offsets_buffer = OffsetBuffer::new(ScalarBuffer::from(offsets));
    ListArray::new(
        Arc::new(Field::new(field_name, DataType::UInt64, false)),
        offsets_buffer,
        Arc::new(values_array),
        None,
    )
}

fn build_list_f64(data: &[Vec<f64>], field_name: &str) -> ListArray {
    let mut offsets = Vec::with_capacity(data.len() + 1);
    let mut values = Vec::new();
    offsets.push(0i32);
    for row in data {
        values.extend_from_slice(row);
        offsets.push(values.len() as i32);
    }
    let values_array = Float64Array::from(values);
    let offsets_buffer = OffsetBuffer::new(ScalarBuffer::from(offsets));
    ListArray::new(
        Arc::new(Field::new(field_name, DataType::Float64, false)),
        offsets_buffer,
        Arc::new(values_array),
        None,
    )
}

// ---------------------------------------------------------------------------
// NumberDataPointColumns
// ---------------------------------------------------------------------------

pub struct NumberDataPointColumns {
    pub id: Vec<u32>,
    pub parent_id: Vec<u16>,
    pub start_time_unix_nano: NullableColumn<arrow::datatypes::TimestampNanosecondType>,
    pub time_unix_nano: Vec<i64>,
    pub int_value: NullableColumn<Int64Type>,
    pub double_value: NullableColumn<Float64Type>,
    pub flags: Vec<u32>,
}

impl NumberDataPointColumns {
    pub fn new() -> Self {
        Self {
            id: Vec::new(),
            parent_id: Vec::new(),
            start_time_unix_nano: NullableColumn::new(),
            time_unix_nano: Vec::new(),
            int_value: NullableColumn::new(),
            double_value: NullableColumn::new(),
            flags: Vec::new(),
        }
    }

    pub fn push<V: NumberDataPointView>(&mut self, id: u32, parent_id: u16, dp: &V) -> usize {
        let row = self.id.len();
        self.id.push(id);
        self.parent_id.push(parent_id);
        push_start_time(&mut self.start_time_unix_nano, dp.start_time_unix_nano());
        self.time_unix_nano.push(dp.time_unix_nano() as i64);
        push_number_value(&mut self.int_value, &mut self.double_value, dp.value());
        self.flags.push(dp.flags().into_inner());
        row
    }

    pub fn overwrite<V: NumberDataPointView>(&mut self, row: usize, dp: &V) {
        set_start_time(
            &mut self.start_time_unix_nano,
            row,
            dp.start_time_unix_nano(),
        );
        self.time_unix_nano[row] = dp.time_unix_nano() as i64;
        set_number_value(&mut self.int_value, &mut self.double_value, row, dp.value());
        self.flags[row] = dp.flags().into_inner();
    }

    pub fn len(&self) -> usize {
        self.id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.id.is_empty()
    }

    pub fn finish(&mut self) -> Result<RecordBatch, arrow::error::ArrowError> {
        if self.id.is_empty() {
            return RecordBatch::try_new_with_options(
                Arc::new(Schema::empty()),
                vec![],
                &arrow::array::RecordBatchOptions::new().with_row_count(Some(0)),
            );
        }

        let schema = Arc::new(Schema::new(vec![
            Field::new(consts::ID, DataType::UInt32, false),
            Field::new(consts::PARENT_ID, DataType::UInt16, false),
            Field::new(
                consts::START_TIME_UNIX_NANO,
                DataType::Timestamp(TimeUnit::Nanosecond, None),
                true,
            ),
            Field::new(
                consts::TIME_UNIX_NANO,
                DataType::Timestamp(TimeUnit::Nanosecond, None),
                false,
            ),
            Field::new(consts::INT_VALUE, DataType::Int64, true),
            Field::new(consts::DOUBLE_VALUE, DataType::Float64, true),
            Field::new(consts::FLAGS, DataType::UInt32, false),
        ]));

        RecordBatch::try_new(
            schema,
            vec![
                Arc::new(UInt32Array::from(std::mem::take(&mut self.id))),
                Arc::new(UInt16Array::from(std::mem::take(&mut self.parent_id))),
                Arc::new(self.start_time_unix_nano.finish()),
                Arc::new(TimestampNanosecondArray::from(std::mem::take(
                    &mut self.time_unix_nano,
                ))),
                Arc::new(self.int_value.finish()),
                Arc::new(self.double_value.finish()),
                Arc::new(UInt32Array::from(std::mem::take(&mut self.flags))),
            ],
        )
    }

    pub fn clear(&mut self) {
        self.id.clear();
        self.parent_id.clear();
        self.start_time_unix_nano.clear();
        self.time_unix_nano.clear();
        self.int_value.clear();
        self.double_value.clear();
        self.flags.clear();
    }
}

// ---------------------------------------------------------------------------
// HistogramDataPointColumns
// ---------------------------------------------------------------------------

pub struct HistogramDataPointColumns {
    pub id: Vec<u32>,
    pub parent_id: Vec<u16>,
    pub start_time_unix_nano: NullableColumn<arrow::datatypes::TimestampNanosecondType>,
    pub time_unix_nano: Vec<i64>,
    pub count: Vec<u64>,
    pub sum: NullableColumn<Float64Type>,
    pub bucket_counts: Vec<Vec<u64>>,
    pub explicit_bounds: Vec<Vec<f64>>,
    pub flags: Vec<u32>,
    pub min: NullableColumn<Float64Type>,
    pub max: NullableColumn<Float64Type>,
}

impl HistogramDataPointColumns {
    pub fn new() -> Self {
        Self {
            id: Vec::new(),
            parent_id: Vec::new(),
            start_time_unix_nano: NullableColumn::new(),
            time_unix_nano: Vec::new(),
            count: Vec::new(),
            sum: NullableColumn::new(),
            bucket_counts: Vec::new(),
            explicit_bounds: Vec::new(),
            flags: Vec::new(),
            min: NullableColumn::new(),
            max: NullableColumn::new(),
        }
    }

    pub fn push<V: HistogramDataPointView>(&mut self, id: u32, parent_id: u16, dp: &V) -> usize {
        let row = self.id.len();
        self.id.push(id);
        self.parent_id.push(parent_id);
        push_start_time(&mut self.start_time_unix_nano, dp.start_time_unix_nano());
        self.time_unix_nano.push(dp.time_unix_nano() as i64);
        self.count.push(dp.count());
        push_optional_f64(&mut self.sum, dp.sum());
        self.bucket_counts.push(dp.bucket_counts().collect());
        self.explicit_bounds.push(dp.explicit_bounds().collect());
        self.flags.push(dp.flags().into_inner());
        push_optional_f64(&mut self.min, dp.min());
        push_optional_f64(&mut self.max, dp.max());
        row
    }

    pub fn overwrite<V: HistogramDataPointView>(&mut self, row: usize, dp: &V) {
        set_start_time(
            &mut self.start_time_unix_nano,
            row,
            dp.start_time_unix_nano(),
        );
        self.time_unix_nano[row] = dp.time_unix_nano() as i64;
        self.count[row] = dp.count();
        set_optional_f64(&mut self.sum, row, dp.sum());
        self.bucket_counts[row] = dp.bucket_counts().collect();
        self.explicit_bounds[row] = dp.explicit_bounds().collect();
        self.flags[row] = dp.flags().into_inner();
        set_optional_f64(&mut self.min, row, dp.min());
        set_optional_f64(&mut self.max, row, dp.max());
    }

    pub fn len(&self) -> usize {
        self.id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.id.is_empty()
    }

    pub fn finish(&mut self) -> Result<RecordBatch, arrow::error::ArrowError> {
        if self.id.is_empty() {
            return RecordBatch::try_new_with_options(
                Arc::new(Schema::empty()),
                vec![],
                &arrow::array::RecordBatchOptions::new().with_row_count(Some(0)),
            );
        }

        let schema = Arc::new(Schema::new(vec![
            Field::new(consts::ID, DataType::UInt32, false),
            Field::new(consts::PARENT_ID, DataType::UInt16, false),
            Field::new(
                consts::START_TIME_UNIX_NANO,
                DataType::Timestamp(TimeUnit::Nanosecond, None),
                false,
            ),
            Field::new(
                consts::TIME_UNIX_NANO,
                DataType::Timestamp(TimeUnit::Nanosecond, None),
                false,
            ),
            Field::new(consts::HISTOGRAM_COUNT, DataType::UInt64, false),
            Field::new(
                consts::HISTOGRAM_BUCKET_COUNTS,
                DataType::List(Arc::new(Field::new("item", DataType::UInt64, false))),
                false,
            ),
            Field::new(
                consts::HISTOGRAM_EXPLICIT_BOUNDS,
                DataType::List(Arc::new(Field::new("item", DataType::Float64, false))),
                false,
            ),
            Field::new(consts::HISTOGRAM_SUM, DataType::Float64, true),
            Field::new(consts::FLAGS, DataType::UInt32, false),
            Field::new(consts::HISTOGRAM_MIN, DataType::Float64, true),
            Field::new(consts::HISTOGRAM_MAX, DataType::Float64, true),
        ]));

        RecordBatch::try_new(
            schema,
            vec![
                Arc::new(UInt32Array::from(std::mem::take(&mut self.id))),
                Arc::new(UInt16Array::from(std::mem::take(&mut self.parent_id))),
                Arc::new(self.start_time_unix_nano.finish()),
                Arc::new(TimestampNanosecondArray::from(std::mem::take(
                    &mut self.time_unix_nano,
                ))),
                Arc::new(UInt64Array::from(std::mem::take(&mut self.count))),
                Arc::new(build_list_u64(&self.bucket_counts, "item")),
                Arc::new(build_list_f64(&self.explicit_bounds, "item")),
                Arc::new(self.sum.finish()),
                Arc::new(UInt32Array::from(std::mem::take(&mut self.flags))),
                Arc::new(self.min.finish()),
                Arc::new(self.max.finish()),
            ],
        )
    }

    pub fn clear(&mut self) {
        self.id.clear();
        self.parent_id.clear();
        self.start_time_unix_nano.clear();
        self.time_unix_nano.clear();
        self.count.clear();
        self.sum.clear();
        self.bucket_counts.clear();
        self.explicit_bounds.clear();
        self.flags.clear();
        self.min.clear();
        self.max.clear();
    }
}

// ---------------------------------------------------------------------------
// ExpHistogramDataPointColumns
// ---------------------------------------------------------------------------

pub struct ExpHistogramDataPointColumns {
    pub id: Vec<u32>,
    pub parent_id: Vec<u16>,
    pub start_time_unix_nano: NullableColumn<arrow::datatypes::TimestampNanosecondType>,
    pub time_unix_nano: Vec<i64>,
    pub count: Vec<u64>,
    pub sum: NullableColumn<Float64Type>,
    pub scale: Vec<i32>,
    pub zero_count: Vec<u64>,
    pub positive_offset: Vec<i32>,
    pub positive_bucket_counts: Vec<Vec<u64>>,
    pub negative_offset: Vec<i32>,
    pub negative_bucket_counts: Vec<Vec<u64>>,
    pub flags: Vec<u32>,
    pub min: NullableColumn<Float64Type>,
    pub max: NullableColumn<Float64Type>,
    pub zero_threshold: Vec<f64>,
}

impl ExpHistogramDataPointColumns {
    pub fn new() -> Self {
        Self {
            id: Vec::new(),
            parent_id: Vec::new(),
            start_time_unix_nano: NullableColumn::new(),
            time_unix_nano: Vec::new(),
            count: Vec::new(),
            sum: NullableColumn::new(),
            scale: Vec::new(),
            zero_count: Vec::new(),
            positive_offset: Vec::new(),
            positive_bucket_counts: Vec::new(),
            negative_offset: Vec::new(),
            negative_bucket_counts: Vec::new(),
            flags: Vec::new(),
            min: NullableColumn::new(),
            max: NullableColumn::new(),
            zero_threshold: Vec::new(),
        }
    }

    pub fn push<V: ExponentialHistogramDataPointView>(
        &mut self,
        id: u32,
        parent_id: u16,
        dp: &V,
    ) -> usize {
        let row = self.id.len();
        self.id.push(id);
        self.parent_id.push(parent_id);
        push_start_time(&mut self.start_time_unix_nano, dp.start_time_unix_nano());
        self.time_unix_nano.push(dp.time_unix_nano() as i64);
        self.count.push(dp.count());
        push_optional_f64(&mut self.sum, dp.sum());
        self.scale.push(dp.scale());
        self.zero_count.push(dp.zero_count());
        match dp.positive() {
            Some(b) => {
                self.positive_offset.push(b.offset());
                self.positive_bucket_counts
                    .push(b.bucket_counts().collect());
            }
            None => {
                self.positive_offset.push(0);
                self.positive_bucket_counts.push(Vec::new());
            }
        }
        match dp.negative() {
            Some(b) => {
                self.negative_offset.push(b.offset());
                self.negative_bucket_counts
                    .push(b.bucket_counts().collect());
            }
            None => {
                self.negative_offset.push(0);
                self.negative_bucket_counts.push(Vec::new());
            }
        }
        self.flags.push(dp.flags().into_inner());
        push_optional_f64(&mut self.min, dp.min());
        push_optional_f64(&mut self.max, dp.max());
        self.zero_threshold.push(dp.zero_threshold());
        row
    }

    pub fn overwrite<V: ExponentialHistogramDataPointView>(&mut self, row: usize, dp: &V) {
        set_start_time(
            &mut self.start_time_unix_nano,
            row,
            dp.start_time_unix_nano(),
        );
        self.time_unix_nano[row] = dp.time_unix_nano() as i64;
        self.count[row] = dp.count();
        set_optional_f64(&mut self.sum, row, dp.sum());
        self.scale[row] = dp.scale();
        self.zero_count[row] = dp.zero_count();
        match dp.positive() {
            Some(b) => {
                self.positive_offset[row] = b.offset();
                self.positive_bucket_counts[row] = b.bucket_counts().collect();
            }
            None => {
                self.positive_offset[row] = 0;
                self.positive_bucket_counts[row] = Vec::new();
            }
        }
        match dp.negative() {
            Some(b) => {
                self.negative_offset[row] = b.offset();
                self.negative_bucket_counts[row] = b.bucket_counts().collect();
            }
            None => {
                self.negative_offset[row] = 0;
                self.negative_bucket_counts[row] = Vec::new();
            }
        }
        self.flags[row] = dp.flags().into_inner();
        set_optional_f64(&mut self.min, row, dp.min());
        set_optional_f64(&mut self.max, row, dp.max());
        self.zero_threshold[row] = dp.zero_threshold();
    }

    pub fn len(&self) -> usize {
        self.id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.id.is_empty()
    }

    pub fn finish(&mut self) -> Result<RecordBatch, arrow::error::ArrowError> {
        if self.id.is_empty() {
            return RecordBatch::try_new_with_options(
                Arc::new(Schema::empty()),
                vec![],
                &arrow::array::RecordBatchOptions::new().with_row_count(Some(0)),
            );
        }

        let buckets_fields = vec![
            Field::new(consts::EXP_HISTOGRAM_OFFSET, DataType::Int32, false),
            Field::new(
                consts::EXP_HISTOGRAM_BUCKET_COUNTS,
                DataType::List(Arc::new(Field::new("item", DataType::UInt64, false))),
                false,
            ),
        ];

        let schema = Arc::new(Schema::new(vec![
            Field::new(consts::ID, DataType::UInt32, false),
            Field::new(consts::PARENT_ID, DataType::UInt16, false),
            Field::new(
                consts::START_TIME_UNIX_NANO,
                DataType::Timestamp(TimeUnit::Nanosecond, None),
                false,
            ),
            Field::new(
                consts::TIME_UNIX_NANO,
                DataType::Timestamp(TimeUnit::Nanosecond, None),
                false,
            ),
            Field::new(consts::HISTOGRAM_COUNT, DataType::UInt64, false),
            Field::new(consts::HISTOGRAM_SUM, DataType::Float64, true),
            Field::new(consts::EXP_HISTOGRAM_SCALE, DataType::Int32, false),
            Field::new(consts::EXP_HISTOGRAM_ZERO_COUNT, DataType::UInt64, false),
            Field::new(
                consts::EXP_HISTOGRAM_POSITIVE,
                DataType::Struct(buckets_fields.clone().into()),
                false,
            ),
            Field::new(
                consts::EXP_HISTOGRAM_NEGATIVE,
                DataType::Struct(buckets_fields.into()),
                false,
            ),
            Field::new(consts::FLAGS, DataType::UInt32, false),
            Field::new(consts::HISTOGRAM_MIN, DataType::Float64, true),
            Field::new(consts::HISTOGRAM_MAX, DataType::Float64, true),
            Field::new(
                consts::EXP_HISTOGRAM_ZERO_THRESHOLD,
                DataType::Float64,
                true,
            ),
        ]));

        let positive = StructArray::from(vec![
            (
                Arc::new(Field::new(
                    consts::EXP_HISTOGRAM_OFFSET,
                    DataType::Int32,
                    false,
                )),
                Arc::new(Int32Array::from(std::mem::take(&mut self.positive_offset)))
                    as Arc<dyn Array>,
            ),
            (
                Arc::new(Field::new(
                    consts::EXP_HISTOGRAM_BUCKET_COUNTS,
                    DataType::List(Arc::new(Field::new("item", DataType::UInt64, false))),
                    false,
                )),
                Arc::new(build_list_u64(&self.positive_bucket_counts, "item")) as Arc<dyn Array>,
            ),
        ]);
        self.positive_bucket_counts.clear();

        let negative = StructArray::from(vec![
            (
                Arc::new(Field::new(
                    consts::EXP_HISTOGRAM_OFFSET,
                    DataType::Int32,
                    false,
                )),
                Arc::new(Int32Array::from(std::mem::take(&mut self.negative_offset)))
                    as Arc<dyn Array>,
            ),
            (
                Arc::new(Field::new(
                    consts::EXP_HISTOGRAM_BUCKET_COUNTS,
                    DataType::List(Arc::new(Field::new("item", DataType::UInt64, false))),
                    false,
                )),
                Arc::new(build_list_u64(&self.negative_bucket_counts, "item")) as Arc<dyn Array>,
            ),
        ]);
        self.negative_bucket_counts.clear();

        RecordBatch::try_new(
            schema,
            vec![
                Arc::new(UInt32Array::from(std::mem::take(&mut self.id))),
                Arc::new(UInt16Array::from(std::mem::take(&mut self.parent_id))),
                Arc::new(self.start_time_unix_nano.finish()),
                Arc::new(TimestampNanosecondArray::from(std::mem::take(
                    &mut self.time_unix_nano,
                ))),
                Arc::new(UInt64Array::from(std::mem::take(&mut self.count))),
                Arc::new(self.sum.finish()),
                Arc::new(Int32Array::from(std::mem::take(&mut self.scale))),
                Arc::new(UInt64Array::from(std::mem::take(&mut self.zero_count))),
                Arc::new(positive),
                Arc::new(negative),
                Arc::new(UInt32Array::from(std::mem::take(&mut self.flags))),
                Arc::new(self.min.finish()),
                Arc::new(self.max.finish()),
                Arc::new(Float64Array::from(std::mem::take(&mut self.zero_threshold))),
            ],
        )
    }

    pub fn clear(&mut self) {
        self.id.clear();
        self.parent_id.clear();
        self.start_time_unix_nano.clear();
        self.time_unix_nano.clear();
        self.count.clear();
        self.sum.clear();
        self.scale.clear();
        self.zero_count.clear();
        self.positive_offset.clear();
        self.positive_bucket_counts.clear();
        self.negative_offset.clear();
        self.negative_bucket_counts.clear();
        self.flags.clear();
        self.min.clear();
        self.max.clear();
        self.zero_threshold.clear();
    }
}

// ---------------------------------------------------------------------------
// SummaryDataPointColumns
// ---------------------------------------------------------------------------

pub struct SummaryDataPointColumns {
    pub id: Vec<u32>,
    pub parent_id: Vec<u16>,
    pub start_time_unix_nano: NullableColumn<arrow::datatypes::TimestampNanosecondType>,
    pub time_unix_nano: Vec<i64>,
    pub count: Vec<u64>,
    pub sum: Vec<f64>,
    pub quantiles: Vec<Vec<(f64, f64)>>,
    pub flags: Vec<u32>,
}

impl SummaryDataPointColumns {
    pub fn new() -> Self {
        Self {
            id: Vec::new(),
            parent_id: Vec::new(),
            start_time_unix_nano: NullableColumn::new(),
            time_unix_nano: Vec::new(),
            count: Vec::new(),
            sum: Vec::new(),
            quantiles: Vec::new(),
            flags: Vec::new(),
        }
    }

    pub fn push<V: SummaryDataPointView>(&mut self, id: u32, parent_id: u16, dp: &V) -> usize {
        let row = self.id.len();
        self.id.push(id);
        self.parent_id.push(parent_id);
        push_start_time(&mut self.start_time_unix_nano, dp.start_time_unix_nano());
        self.time_unix_nano.push(dp.time_unix_nano() as i64);
        self.count.push(dp.count());
        self.sum.push(dp.sum());
        self.quantiles.push(
            dp.quantile_values()
                .map(|q| (q.quantile(), q.value()))
                .collect(),
        );
        self.flags.push(dp.flags().into_inner());
        row
    }

    pub fn overwrite<V: SummaryDataPointView>(&mut self, row: usize, dp: &V) {
        set_start_time(
            &mut self.start_time_unix_nano,
            row,
            dp.start_time_unix_nano(),
        );
        self.time_unix_nano[row] = dp.time_unix_nano() as i64;
        self.count[row] = dp.count();
        self.sum[row] = dp.sum();
        self.quantiles[row] = dp
            .quantile_values()
            .map(|q| (q.quantile(), q.value()))
            .collect();
        self.flags[row] = dp.flags().into_inner();
    }

    pub fn len(&self) -> usize {
        self.id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.id.is_empty()
    }

    pub fn finish(&mut self) -> Result<RecordBatch, arrow::error::ArrowError> {
        if self.id.is_empty() {
            return RecordBatch::try_new_with_options(
                Arc::new(Schema::empty()),
                vec![],
                &arrow::array::RecordBatchOptions::new().with_row_count(Some(0)),
            );
        }

        let quantile_fields = vec![
            Field::new(consts::SUMMARY_QUANTILE, DataType::Float64, false),
            Field::new(consts::SUMMARY_VALUE, DataType::Float64, false),
        ];

        // Build the quantiles list of structs
        let mut offsets = Vec::with_capacity(self.quantiles.len() + 1);
        let mut q_vals = Vec::new();
        let mut v_vals = Vec::new();
        offsets.push(0i32);
        for row in &self.quantiles {
            for &(q, v) in row {
                q_vals.push(q);
                v_vals.push(v);
            }
            offsets.push(q_vals.len() as i32);
        }
        let quantile_struct = StructArray::from(vec![
            (
                Arc::new(Field::new(
                    consts::SUMMARY_QUANTILE,
                    DataType::Float64,
                    false,
                )),
                Arc::new(Float64Array::from(q_vals)) as Arc<dyn Array>,
            ),
            (
                Arc::new(Field::new(consts::SUMMARY_VALUE, DataType::Float64, false)),
                Arc::new(Float64Array::from(v_vals)) as Arc<dyn Array>,
            ),
        ]);
        let quantile_list = ListArray::new(
            Arc::new(Field::new(
                "item",
                DataType::Struct(quantile_fields.into()),
                false,
            )),
            OffsetBuffer::new(ScalarBuffer::from(offsets)),
            Arc::new(quantile_struct),
            None,
        );
        self.quantiles.clear();

        let schema = Arc::new(Schema::new(vec![
            Field::new(consts::ID, DataType::UInt32, false),
            Field::new(consts::PARENT_ID, DataType::UInt16, false),
            Field::new(
                consts::START_TIME_UNIX_NANO,
                DataType::Timestamp(TimeUnit::Nanosecond, None),
                false,
            ),
            Field::new(
                consts::TIME_UNIX_NANO,
                DataType::Timestamp(TimeUnit::Nanosecond, None),
                false,
            ),
            Field::new(consts::SUMMARY_COUNT, DataType::UInt64, false),
            Field::new(consts::SUMMARY_SUM, DataType::Float64, false),
            Field::new(
                consts::SUMMARY_QUANTILE_VALUES,
                quantile_list.data_type().clone(),
                false,
            ),
            Field::new(consts::FLAGS, DataType::UInt32, false),
        ]));

        RecordBatch::try_new(
            schema,
            vec![
                Arc::new(UInt32Array::from(std::mem::take(&mut self.id))),
                Arc::new(UInt16Array::from(std::mem::take(&mut self.parent_id))),
                Arc::new(self.start_time_unix_nano.finish()),
                Arc::new(TimestampNanosecondArray::from(std::mem::take(
                    &mut self.time_unix_nano,
                ))),
                Arc::new(UInt64Array::from(std::mem::take(&mut self.count))),
                Arc::new(Float64Array::from(std::mem::take(&mut self.sum))),
                Arc::new(quantile_list),
                Arc::new(UInt32Array::from(std::mem::take(&mut self.flags))),
            ],
        )
    }

    pub fn clear(&mut self) {
        self.id.clear();
        self.parent_id.clear();
        self.start_time_unix_nano.clear();
        self.time_unix_nano.clear();
        self.count.clear();
        self.sum.clear();
        self.quantiles.clear();
        self.flags.clear();
    }
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn push_start_time(
    col: &mut NullableColumn<arrow::datatypes::TimestampNanosecondType>,
    value: u64,
) {
    if value == 0 {
        col.push_null();
    } else {
        col.push_value(value as i64);
    }
}

fn set_start_time(
    col: &mut NullableColumn<arrow::datatypes::TimestampNanosecondType>,
    row: usize,
    value: u64,
) {
    if value == 0 {
        col.set_null(row);
    } else {
        col.set_value(row, value as i64);
    }
}

fn push_number_value(
    int_col: &mut NullableColumn<Int64Type>,
    double_col: &mut NullableColumn<Float64Type>,
    value: Option<Value>,
) {
    match value {
        Some(Value::Integer(v)) => {
            int_col.push_value(v);
            double_col.push_null();
        }
        Some(Value::Double(v)) => {
            int_col.push_null();
            double_col.push_value(v);
        }
        None => {
            int_col.push_null();
            double_col.push_null();
        }
    }
}

fn set_number_value(
    int_col: &mut NullableColumn<Int64Type>,
    double_col: &mut NullableColumn<Float64Type>,
    row: usize,
    value: Option<Value>,
) {
    match value {
        Some(Value::Integer(v)) => {
            int_col.set_value(row, v);
            double_col.set_null(row);
        }
        Some(Value::Double(v)) => {
            int_col.set_null(row);
            double_col.set_value(row, v);
        }
        None => {
            int_col.set_null(row);
            double_col.set_null(row);
        }
    }
}

fn push_optional_f64(col: &mut NullableColumn<Float64Type>, value: Option<f64>) {
    match value {
        Some(v) => col.push_value(v),
        None => col.push_null(),
    }
}

fn set_optional_f64(col: &mut NullableColumn<Float64Type>, row: usize, value: Option<f64>) {
    match value {
        Some(v) => col.set_value(row, v),
        None => col.set_null(row),
    }
}
