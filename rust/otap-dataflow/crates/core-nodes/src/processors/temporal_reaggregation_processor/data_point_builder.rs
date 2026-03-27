// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Specialized builders for data point record batches. The motivation behind
//! these is having an API to overwrite data points that have already been pushed
//! because a newer data point came in. Many metric types are aggregated in this
//! way by taking the latest point.
//!
//! APIs to write at a specific Array position are typically not supported by
//! arrow-rs ArrayBuilder types and by extension not supported by the builder
//! types in [otap_df_pdata::encode::record]. These types usually only support
//! appends.
//!
//! Random write APIs _are_, however, generally supported by the arrow-rs
//! BufferBuilder types (including [BooleanBufferBuilder] for [NullBuffer]s).
//! Arrow-rs also generally supports direct conversion to Arrays from Vec<T> for
//! fixed-width types.
//!
//! So, we have these builders which use Vec<T> as much as possible along with
//! [BooleanBufferBuilder] directly for nulls.
//!
//! Note: These builders currently make no attempt to optimize for space via
//! dictionary encoding. We may want to add this in the future, however a typical
//! pipeline configuration may be to place a batch processor immeditaly following
//! this component in which case the dictionary could get re-written or removed
//! anyway. Figuring out under what circumstances it's better to use dictionaries
//! here needs investigation.

use std::sync::{Arc, LazyLock};

use arrow::array::{
    Array, ArrowPrimitiveType, BooleanBufferBuilder, Float64Array, Int32Array, ListArray,
    PrimitiveArray, RecordBatch, StructArray, TimestampNanosecondArray, UInt16Array, UInt32Array,
    UInt64Array,
};
use arrow::buffer::{NullBuffer, OffsetBuffer, ScalarBuffer};
use arrow::datatypes::{DataType, Field, Fields, Float64Type, Int64Type, Schema, TimeUnit};
use otap_df_pdata::schema::{FieldExt, consts};
use otap_df_pdata_views::views::metrics::{
    BucketsView, ExponentialHistogramDataPointView, HistogramDataPointView, NumberDataPointView,
    SummaryDataPointView, Value, ValueAtQuantileView,
};

/// A column of nullable primitive values that supports random-access writes.
///
/// At flush time, converts to an Arrow [`PrimitiveArray`] via zero-copy for
/// the values buffer and a constructed null bitmap.
pub struct NullableColumnBuilder<T: ArrowPrimitiveType> {
    values: Vec<T::Native>,
    validity: BooleanBufferBuilder,
}

impl<T: ArrowPrimitiveType> NullableColumnBuilder<T> {
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            validity: BooleanBufferBuilder::new(0),
        }
    }

    pub fn append_value(&mut self, value: T::Native) {
        self.values.push(value);
        self.validity.append(true);
    }

    pub fn append_null(&mut self) {
        self.values.push(T::Native::default());
        self.validity.append(false);
    }

    /// Write a value at `index`. If `index == len`, appends; otherwise
    /// overwrites the existing entry.
    pub fn write_value(&mut self, index: usize, value: T::Native) {
        if index == self.values.len() {
            self.append_value(value);
        } else {
            self.set_value(index, value);
        }
    }

    fn set_value(&mut self, index: usize, value: T::Native) {
        self.values[index] = value;
        self.validity.set_bit(index, true);
    }

    fn set_null(&mut self, index: usize) {
        self.validity.set_bit(index, false);
    }

    /// Write a null at `index`. If `index == len`, appends; otherwise
    /// overwrites the existing entry.
    pub fn write_null(&mut self, index: usize) {
        if index == self.values.len() {
            self.append_null();
        } else {
            self.set_null(index);
        }
    }

    /// Convert to an Arrow [`PrimitiveArray`]. The internal buffers are consumed
    /// and replaced with new 0 size buffers.
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

pub struct NumberDataPointBuilder {
    pub id: Vec<u32>,
    pub parent_id: Vec<u16>,
    pub start_time_unix_nano: Vec<i64>,
    pub time_unix_nano: Vec<i64>,
    pub int_value: NullableColumnBuilder<Int64Type>,
    pub double_value: NullableColumnBuilder<Float64Type>,
    pub flags: Vec<u32>,
}

impl NumberDataPointBuilder {
    pub fn new() -> Self {
        Self {
            id: Vec::new(),
            parent_id: Vec::new(),
            start_time_unix_nano: Vec::new(),
            time_unix_nano: Vec::new(),
            int_value: NullableColumnBuilder::new(),
            double_value: NullableColumnBuilder::new(),
            flags: Vec::new(),
        }
    }

    /// Append a new data point
    pub fn append<V: NumberDataPointView>(&mut self, id: u32, parent_id: u16, dp: &V) -> usize {
        let row = self.id.len();
        self.id.push(id);
        self.parent_id.push(parent_id);
        self.write(row, dp);
        row
    }

    /// Replace a data point's data with the given view
    pub fn replace<V: NumberDataPointView>(&mut self, row: usize, dp: &V) {
        self.write(row, dp);
    }

    /// Helper to write the data point's data which excludes id/parent_id
    fn write<V: NumberDataPointView>(&mut self, row: usize, dp: &V) {
        write_or_push(
            &mut self.start_time_unix_nano,
            row,
            dp.start_time_unix_nano() as i64,
        );
        write_or_push(&mut self.time_unix_nano, row, dp.time_unix_nano() as i64);

        match dp.value() {
            Some(Value::Integer(v)) => {
                self.int_value.write_value(row, v);
                self.double_value.write_null(row);
            }
            Some(Value::Double(v)) => {
                self.int_value.write_null(row);
                self.double_value.write_value(row, v);
            }
            None => {
                self.int_value.write_null(row);
                self.double_value.write_null(row);
            }
        }

        write_or_push(&mut self.flags, row, dp.flags().into_inner());
    }

    /// Consume all of the internal buffers to produce a record batch. If finish
    /// is successful, then this builder is in a cleared state to begin building
    /// a new record batch.
    pub fn finish(&mut self) -> Result<RecordBatch, arrow::error::ArrowError> {
        if self.id.is_empty() {
            return empty_record_batch();
        }

        RecordBatch::try_new(
            NUMBER_DP_SCHEMA.clone(),
            vec![
                Arc::new(UInt32Array::from(std::mem::take(&mut self.id))),
                Arc::new(UInt16Array::from(std::mem::take(&mut self.parent_id))),
                Arc::new(TimestampNanosecondArray::from(std::mem::take(
                    &mut self.start_time_unix_nano,
                ))),
                Arc::new(TimestampNanosecondArray::from(std::mem::take(
                    &mut self.time_unix_nano,
                ))),
                Arc::new(self.int_value.finish()),
                Arc::new(self.double_value.finish()),
                Arc::new(UInt32Array::from(std::mem::take(&mut self.flags))),
            ],
        )
    }

    /// Clear all internal buffers
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

static NUMBER_DP_SCHEMA: LazyLock<Arc<Schema>> = LazyLock::new(|| {
    Arc::new(Schema::new(vec![
        Field::new(consts::ID, DataType::UInt32, false).with_plain_encoding(),
        Field::new(consts::PARENT_ID, DataType::UInt16, false).with_plain_encoding(),
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
        Field::new(consts::INT_VALUE, DataType::Int64, true),
        Field::new(consts::DOUBLE_VALUE, DataType::Float64, true),
        Field::new(consts::FLAGS, DataType::UInt32, false),
    ]))
});

pub struct HistogramDataPointBuilder {
    pub id: Vec<u32>,
    pub parent_id: Vec<u16>,
    pub start_time_unix_nano: Vec<i64>,
    pub time_unix_nano: Vec<i64>,
    pub count: Vec<u64>,
    pub sum: NullableColumnBuilder<Float64Type>,
    pub bucket_counts: Vec<Vec<u64>>,
    pub explicit_bounds: Vec<Vec<f64>>,
    pub flags: Vec<u32>,
    pub min: NullableColumnBuilder<Float64Type>,
    pub max: NullableColumnBuilder<Float64Type>,
}

impl HistogramDataPointBuilder {
    pub fn new() -> Self {
        Self {
            id: Vec::new(),
            parent_id: Vec::new(),
            start_time_unix_nano: Vec::new(),
            time_unix_nano: Vec::new(),
            count: Vec::new(),
            sum: NullableColumnBuilder::new(),
            bucket_counts: Vec::new(),
            explicit_bounds: Vec::new(),
            flags: Vec::new(),
            min: NullableColumnBuilder::new(),
            max: NullableColumnBuilder::new(),
        }
    }

    pub fn append<V: HistogramDataPointView>(&mut self, id: u32, parent_id: u16, dp: &V) -> usize {
        let row = self.id.len();
        self.id.push(id);
        self.parent_id.push(parent_id);
        self.write(row, dp);
        row
    }

    pub fn replace<V: HistogramDataPointView>(&mut self, row: usize, dp: &V) {
        self.write(row, dp);
    }

    fn write<V: HistogramDataPointView>(&mut self, row: usize, dp: &V) {
        write_or_push(
            &mut self.start_time_unix_nano,
            row,
            dp.start_time_unix_nano() as i64,
        );
        write_or_push(&mut self.time_unix_nano, row, dp.time_unix_nano() as i64);
        write_or_push(&mut self.count, row, dp.count());
        write_optional_f64(&mut self.sum, row, dp.sum());
        write_or_push(&mut self.bucket_counts, row, dp.bucket_counts().collect());
        write_or_push(
            &mut self.explicit_bounds,
            row,
            dp.explicit_bounds().collect(),
        );
        write_or_push(&mut self.flags, row, dp.flags().into_inner());
        write_optional_f64(&mut self.min, row, dp.min());
        write_optional_f64(&mut self.max, row, dp.max());
    }

    pub fn finish(&mut self) -> Result<RecordBatch, arrow::error::ArrowError> {
        if self.id.is_empty() {
            return empty_record_batch();
        }

        RecordBatch::try_new(
            HISTOGRAM_DP_SCHEMA.clone(),
            vec![
                Arc::new(UInt32Array::from(std::mem::take(&mut self.id))),
                Arc::new(UInt16Array::from(std::mem::take(&mut self.parent_id))),
                Arc::new(TimestampNanosecondArray::from(std::mem::take(
                    &mut self.start_time_unix_nano,
                ))),
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

static HISTOGRAM_DP_SCHEMA: LazyLock<Arc<Schema>> = LazyLock::new(|| {
    Arc::new(Schema::new(vec![
        Field::new(consts::ID, DataType::UInt32, false).with_plain_encoding(),
        Field::new(consts::PARENT_ID, DataType::UInt16, false).with_plain_encoding(),
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
    ]))
});

pub struct ExpHistogramDataPointBuilder {
    pub id: Vec<u32>,
    pub parent_id: Vec<u16>,
    pub start_time_unix_nano: Vec<i64>,
    pub time_unix_nano: Vec<i64>,
    pub count: Vec<u64>,
    pub sum: NullableColumnBuilder<Float64Type>,
    pub scale: Vec<i32>,
    pub zero_count: Vec<u64>,
    pub positive_offset: Vec<i32>,
    pub positive_bucket_counts: Vec<Vec<u64>>,
    pub negative_offset: Vec<i32>,
    pub negative_bucket_counts: Vec<Vec<u64>>,
    pub flags: Vec<u32>,
    pub min: NullableColumnBuilder<Float64Type>,
    pub max: NullableColumnBuilder<Float64Type>,
    pub zero_threshold: Vec<f64>,
}

impl ExpHistogramDataPointBuilder {
    pub fn new() -> Self {
        Self {
            id: Vec::new(),
            parent_id: Vec::new(),
            start_time_unix_nano: Vec::new(),
            time_unix_nano: Vec::new(),
            count: Vec::new(),
            sum: NullableColumnBuilder::new(),
            scale: Vec::new(),
            zero_count: Vec::new(),
            positive_offset: Vec::new(),
            positive_bucket_counts: Vec::new(),
            negative_offset: Vec::new(),
            negative_bucket_counts: Vec::new(),
            flags: Vec::new(),
            min: NullableColumnBuilder::new(),
            max: NullableColumnBuilder::new(),
            zero_threshold: Vec::new(),
        }
    }

    pub fn append<V: ExponentialHistogramDataPointView>(
        &mut self,
        id: u32,
        parent_id: u16,
        dp: &V,
    ) -> usize {
        let row = self.id.len();
        self.id.push(id);
        self.parent_id.push(parent_id);
        self.write(row, dp);
        row
    }

    pub fn replace<V: ExponentialHistogramDataPointView>(&mut self, row: usize, dp: &V) {
        // We keep the same parent_id/id here and just write the rest of the fields
        self.write(row, dp);
    }

    fn write<V: ExponentialHistogramDataPointView>(&mut self, row: usize, dp: &V) {
        write_or_push(
            &mut self.start_time_unix_nano,
            row,
            dp.start_time_unix_nano() as i64,
        );
        write_or_push(&mut self.time_unix_nano, row, dp.time_unix_nano() as i64);
        write_or_push(&mut self.count, row, dp.count());
        write_optional_f64(&mut self.sum, row, dp.sum());
        write_or_push(&mut self.scale, row, dp.scale());
        write_or_push(&mut self.zero_count, row, dp.zero_count());
        let (pos_offset, pos_counts) = match dp.positive() {
            Some(b) => (b.offset(), b.bucket_counts().collect()),
            None => (0, Vec::new()),
        };
        write_or_push(&mut self.positive_offset, row, pos_offset);
        write_or_push(&mut self.positive_bucket_counts, row, pos_counts);
        let (neg_offset, neg_counts) = match dp.negative() {
            Some(b) => (b.offset(), b.bucket_counts().collect()),
            None => (0, Vec::new()),
        };
        write_or_push(&mut self.negative_offset, row, neg_offset);
        write_or_push(&mut self.negative_bucket_counts, row, neg_counts);
        write_or_push(&mut self.flags, row, dp.flags().into_inner());
        write_optional_f64(&mut self.min, row, dp.min());
        write_optional_f64(&mut self.max, row, dp.max());
        write_or_push(&mut self.zero_threshold, row, dp.zero_threshold());
    }

    pub fn finish(&mut self) -> Result<RecordBatch, arrow::error::ArrowError> {
        if self.id.is_empty() {
            return empty_record_batch();
        }

        let positive = StructArray::from(vec![
            (
                EXP_HISTOGRAM_OFFSET_FIELD.clone(),
                Arc::new(Int32Array::from(std::mem::take(&mut self.positive_offset)))
                    as Arc<dyn Array>,
            ),
            (
                EXP_HISTOGRAM_BUCKET_COUNTS_FIELD.clone(),
                Arc::new(build_list_u64(&self.positive_bucket_counts, "item")) as Arc<dyn Array>,
            ),
        ]);
        self.positive_bucket_counts.clear();

        let negative = StructArray::from(vec![
            (
                EXP_HISTOGRAM_OFFSET_FIELD.clone(),
                Arc::new(Int32Array::from(std::mem::take(&mut self.negative_offset)))
                    as Arc<dyn Array>,
            ),
            (
                EXP_HISTOGRAM_BUCKET_COUNTS_FIELD.clone(),
                Arc::new(build_list_u64(&self.negative_bucket_counts, "item")) as Arc<dyn Array>,
            ),
        ]);
        self.negative_bucket_counts.clear();

        RecordBatch::try_new(
            EXP_HISTOGRAM_DP_SCHEMA.clone(),
            vec![
                Arc::new(UInt32Array::from(std::mem::take(&mut self.id))),
                Arc::new(UInt16Array::from(std::mem::take(&mut self.parent_id))),
                Arc::new(TimestampNanosecondArray::from(std::mem::take(
                    &mut self.start_time_unix_nano,
                ))),
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

static EXP_HISTOGRAM_OFFSET_FIELD: LazyLock<Arc<Field>> = LazyLock::new(|| {
    Arc::new(Field::new(
        consts::EXP_HISTOGRAM_OFFSET,
        DataType::Int32,
        false,
    ))
});

static EXP_HISTOGRAM_BUCKET_COUNTS_FIELD: LazyLock<Arc<Field>> = LazyLock::new(|| {
    Arc::new(Field::new(
        consts::EXP_HISTOGRAM_BUCKET_COUNTS,
        DataType::List(Arc::new(Field::new("item", DataType::UInt64, false))),
        false,
    ))
});

static EXP_HISTOGRAM_BUCKETS_FIELDS: LazyLock<Fields> = LazyLock::new(|| {
    Fields::from(vec![
        Field::clone(&EXP_HISTOGRAM_OFFSET_FIELD),
        Field::clone(&EXP_HISTOGRAM_BUCKET_COUNTS_FIELD),
    ])
});

static EXP_HISTOGRAM_DP_SCHEMA: LazyLock<Arc<Schema>> = LazyLock::new(|| {
    Arc::new(Schema::new(vec![
        Field::new(consts::ID, DataType::UInt32, false).with_plain_encoding(),
        Field::new(consts::PARENT_ID, DataType::UInt16, false).with_plain_encoding(),
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
            DataType::Struct(EXP_HISTOGRAM_BUCKETS_FIELDS.clone()),
            false,
        ),
        Field::new(
            consts::EXP_HISTOGRAM_NEGATIVE,
            DataType::Struct(EXP_HISTOGRAM_BUCKETS_FIELDS.clone()),
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
    ]))
});

pub struct SummaryDataPointBuilder {
    pub id: Vec<u32>,
    pub parent_id: Vec<u16>,
    pub start_time_unix_nano: Vec<i64>,
    pub time_unix_nano: Vec<i64>,
    pub count: Vec<u64>,
    pub sum: Vec<f64>,
    pub quantiles: Vec<Vec<(f64, f64)>>,
    pub flags: Vec<u32>,
}

impl SummaryDataPointBuilder {
    pub fn new() -> Self {
        Self {
            id: Vec::new(),
            parent_id: Vec::new(),
            start_time_unix_nano: Vec::new(),
            time_unix_nano: Vec::new(),
            count: Vec::new(),
            sum: Vec::new(),
            quantiles: Vec::new(),
            flags: Vec::new(),
        }
    }

    pub fn append<V: SummaryDataPointView>(&mut self, id: u32, parent_id: u16, dp: &V) -> usize {
        let row = self.id.len();
        self.id.push(id);
        self.parent_id.push(parent_id);
        self.write(row, dp);
        row
    }

    pub fn replace<V: SummaryDataPointView>(&mut self, row: usize, dp: &V) {
        self.write(row, dp);
    }

    fn write<V: SummaryDataPointView>(&mut self, row: usize, dp: &V) {
        write_or_push(
            &mut self.start_time_unix_nano,
            row,
            dp.start_time_unix_nano() as i64,
        );
        write_or_push(&mut self.time_unix_nano, row, dp.time_unix_nano() as i64);
        write_or_push(&mut self.count, row, dp.count());
        write_or_push(&mut self.sum, row, dp.sum());
        write_or_push(
            &mut self.quantiles,
            row,
            dp.quantile_values()
                .map(|q| (q.quantile(), q.value()))
                .collect(),
        );
        write_or_push(&mut self.flags, row, dp.flags().into_inner());
    }

    pub fn finish(&mut self) -> Result<RecordBatch, arrow::error::ArrowError> {
        if self.id.is_empty() {
            return empty_record_batch();
        }

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
                SUMMARY_QUANTILE_FIELD.clone(),
                Arc::new(Float64Array::from(q_vals)) as Arc<dyn Array>,
            ),
            (
                SUMMARY_VALUE_FIELD.clone(),
                Arc::new(Float64Array::from(v_vals)) as Arc<dyn Array>,
            ),
        ]);
        let quantile_list = ListArray::new(
            SUMMARY_QUANTILE_LIST_FIELD.clone(),
            OffsetBuffer::new(ScalarBuffer::from(offsets)),
            Arc::new(quantile_struct),
            None,
        );
        self.quantiles.clear();

        RecordBatch::try_new(
            SUMMARY_DP_SCHEMA.clone(),
            vec![
                Arc::new(UInt32Array::from(std::mem::take(&mut self.id))),
                Arc::new(UInt16Array::from(std::mem::take(&mut self.parent_id))),
                Arc::new(TimestampNanosecondArray::from(std::mem::take(
                    &mut self.start_time_unix_nano,
                ))),
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

static SUMMARY_QUANTILE_FIELD: LazyLock<Arc<Field>> = LazyLock::new(|| {
    Arc::new(Field::new(
        consts::SUMMARY_QUANTILE,
        DataType::Float64,
        false,
    ))
});

static SUMMARY_VALUE_FIELD: LazyLock<Arc<Field>> =
    LazyLock::new(|| Arc::new(Field::new(consts::SUMMARY_VALUE, DataType::Float64, false)));

static SUMMARY_QUANTILE_FIELDS: LazyLock<Fields> = LazyLock::new(|| {
    Fields::from(vec![
        Field::clone(&SUMMARY_QUANTILE_FIELD),
        Field::clone(&SUMMARY_VALUE_FIELD),
    ])
});

static SUMMARY_QUANTILE_LIST_FIELD: LazyLock<Arc<Field>> = LazyLock::new(|| {
    Arc::new(Field::new(
        "item",
        DataType::Struct(SUMMARY_QUANTILE_FIELDS.clone()),
        false,
    ))
});

static SUMMARY_DP_SCHEMA: LazyLock<Arc<Schema>> = LazyLock::new(|| {
    Arc::new(Schema::new(vec![
        Field::new(consts::ID, DataType::UInt32, false).with_plain_encoding(),
        Field::new(consts::PARENT_ID, DataType::UInt16, false).with_plain_encoding(),
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
            DataType::List(SUMMARY_QUANTILE_LIST_FIELD.clone()),
            false,
        ),
        Field::new(consts::FLAGS, DataType::UInt32, false),
    ]))
});

fn empty_record_batch() -> Result<RecordBatch, arrow::error::ArrowError> {
    RecordBatch::try_new_with_options(
        Arc::new(Schema::empty()),
        vec![],
        &arrow::array::RecordBatchOptions::new().with_row_count(Some(0)),
    )
}

/// Write `value` into `vec` at `index`. Appends if `index == vec.len()`,
/// otherwise overwrites.
fn write_or_push<T>(vec: &mut Vec<T>, index: usize, value: T) {
    if index == vec.len() {
        vec.push(value);
    } else {
        vec[index] = value;
    }
}

fn write_optional_f64(
    col: &mut NullableColumnBuilder<Float64Type>,
    index: usize,
    value: Option<f64>,
) {
    match value {
        Some(v) => col.write_value(index, v),
        None => col.write_null(index),
    }
}

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
