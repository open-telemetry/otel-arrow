// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! SOA (Struct of Arrays) storage for data points and a nullable column helper.
//!
//! Data points are stored in column-oriented format so that they can be
//! converted to Arrow arrays at flush time with minimal copying. The
//! [`NullableColumn`] helper wraps a `Vec<T>` and a `BooleanBufferBuilder`
//! to support random-access writes for nullable fixed-width columns.

use arrow::array::{ArrowPrimitiveType, BooleanBufferBuilder, PrimitiveArray, RecordBatch};
use arrow::buffer::{NullBuffer, ScalarBuffer};
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
    /// Create an empty column.
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            validity: BooleanBufferBuilder::new(0),
        }
    }

    /// Append a non-null value.
    pub fn push_value(&mut self, value: T::Native) {
        self.values.push(value);
        self.validity.append(true);
    }

    /// Append a null.
    pub fn push_null(&mut self) {
        self.values.push(T::Native::default());
        self.validity.append(false);
    }

    /// Overwrite at `index` with a non-null value.
    pub fn set_value(&mut self, index: usize, value: T::Native) {
        self.values[index] = value;
        self.validity.set_bit(index, true);
    }

    /// Overwrite at `index` with null.
    pub fn set_null(&mut self, index: usize) {
        self.values[index] = T::Native::default();
        self.validity.set_bit(index, false);
    }

    /// Number of rows.
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

    /// Reset the column, keeping heap allocations for reuse.
    pub fn clear(&mut self) {
        self.values.clear();
        self.validity = BooleanBufferBuilder::new(0);
    }
}

// ---------------------------------------------------------------------------
// NumberDataPointColumns
// ---------------------------------------------------------------------------

/// SOA storage for number data points (gauges and sums).
pub struct NumberDataPointColumns {
    pub id: Vec<u32>,
    pub parent_id: Vec<u16>,
    pub start_time_unix_nano: NullableColumn<arrow::datatypes::TimestampNanosecondType>,
    pub time_unix_nano: Vec<i64>,
    pub int_value: NullableColumn<arrow::datatypes::Int64Type>,
    pub double_value: NullableColumn<arrow::datatypes::Float64Type>,
    pub flags: Vec<u32>,
}

impl NumberDataPointColumns {
    /// Create empty columns.
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

    /// Append a data point from a view. Returns the row index.
    pub fn push<V: NumberDataPointView>(&mut self, id: u32, parent_id: u16, dp: &V) -> usize {
        let row = self.id.len();
        self.id.push(id);
        self.parent_id.push(parent_id);
        let start = dp.start_time_unix_nano();
        if start == 0 {
            self.start_time_unix_nano.push_null();
        } else {
            self.start_time_unix_nano.push_value(start as i64);
        }
        self.time_unix_nano.push(dp.time_unix_nano() as i64);
        match dp.value() {
            Some(Value::Integer(v)) => {
                self.int_value.push_value(v);
                self.double_value.push_null();
            }
            Some(Value::Double(v)) => {
                self.int_value.push_null();
                self.double_value.push_value(v);
            }
            None => {
                self.int_value.push_null();
                self.double_value.push_null();
            }
        }
        self.flags.push(dp.flags().into_inner());
        row
    }

    /// Overwrite the data point at `row` (id and parent_id unchanged).
    pub fn overwrite<V: NumberDataPointView>(&mut self, row: usize, dp: &V) {
        let start = dp.start_time_unix_nano();
        if start == 0 {
            self.start_time_unix_nano.set_null(row);
        } else {
            self.start_time_unix_nano.set_value(row, start as i64);
        }
        self.time_unix_nano[row] = dp.time_unix_nano() as i64;
        match dp.value() {
            Some(Value::Integer(v)) => {
                self.int_value.set_value(row, v);
                self.double_value.set_null(row);
            }
            Some(Value::Double(v)) => {
                self.int_value.set_null(row);
                self.double_value.set_value(row, v);
            }
            None => {
                self.int_value.set_null(row);
                self.double_value.set_null(row);
            }
        }
        self.flags[row] = dp.flags().into_inner();
    }

    /// Number of data points stored.
    pub fn len(&self) -> usize {
        self.id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.id.is_empty()
    }

    /// Convert to a [`RecordBatch`], consuming column data.
    pub fn finish(&mut self) -> Result<RecordBatch, arrow::error::ArrowError> {
        todo!("Stage 3b: build RecordBatch from SOA columns")
    }

    /// Reset all columns, keeping heap allocations for reuse.
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

/// SOA storage for histogram data points.
pub struct HistogramDataPointColumns {
    pub id: Vec<u32>,
    pub parent_id: Vec<u16>,
    pub start_time_unix_nano: NullableColumn<arrow::datatypes::TimestampNanosecondType>,
    pub time_unix_nano: Vec<i64>,
    pub count: Vec<u64>,
    pub sum: NullableColumn<arrow::datatypes::Float64Type>,
    pub bucket_counts: Vec<Vec<u64>>,
    pub explicit_bounds: Vec<Vec<f64>>,
    pub flags: Vec<u32>,
    pub min: NullableColumn<arrow::datatypes::Float64Type>,
    pub max: NullableColumn<arrow::datatypes::Float64Type>,
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
        let start = dp.start_time_unix_nano();
        if start == 0 {
            self.start_time_unix_nano.push_null();
        } else {
            self.start_time_unix_nano.push_value(start as i64);
        }
        self.time_unix_nano.push(dp.time_unix_nano() as i64);
        self.count.push(dp.count());
        match dp.sum() {
            Some(v) => self.sum.push_value(v),
            None => self.sum.push_null(),
        }
        self.bucket_counts.push(dp.bucket_counts().collect());
        self.explicit_bounds.push(dp.explicit_bounds().collect());
        self.flags.push(dp.flags().into_inner());
        match dp.min() {
            Some(v) => self.min.push_value(v),
            None => self.min.push_null(),
        }
        match dp.max() {
            Some(v) => self.max.push_value(v),
            None => self.max.push_null(),
        }
        row
    }

    pub fn overwrite<V: HistogramDataPointView>(&mut self, row: usize, dp: &V) {
        let start = dp.start_time_unix_nano();
        if start == 0 {
            self.start_time_unix_nano.set_null(row);
        } else {
            self.start_time_unix_nano.set_value(row, start as i64);
        }
        self.time_unix_nano[row] = dp.time_unix_nano() as i64;
        self.count[row] = dp.count();
        match dp.sum() {
            Some(v) => self.sum.set_value(row, v),
            None => self.sum.set_null(row),
        }
        self.bucket_counts[row] = dp.bucket_counts().collect();
        self.explicit_bounds[row] = dp.explicit_bounds().collect();
        self.flags[row] = dp.flags().into_inner();
        match dp.min() {
            Some(v) => self.min.set_value(row, v),
            None => self.min.set_null(row),
        }
        match dp.max() {
            Some(v) => self.max.set_value(row, v),
            None => self.max.set_null(row),
        }
    }

    pub fn len(&self) -> usize {
        self.id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.id.is_empty()
    }

    pub fn finish(&mut self) -> Result<RecordBatch, arrow::error::ArrowError> {
        todo!("Stage 3b: build RecordBatch from SOA columns")
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

/// SOA storage for exponential histogram data points.
pub struct ExpHistogramDataPointColumns {
    pub id: Vec<u32>,
    pub parent_id: Vec<u16>,
    pub start_time_unix_nano: NullableColumn<arrow::datatypes::TimestampNanosecondType>,
    pub time_unix_nano: Vec<i64>,
    pub count: Vec<u64>,
    pub sum: NullableColumn<arrow::datatypes::Float64Type>,
    pub scale: Vec<i32>,
    pub zero_count: Vec<u64>,
    pub positive_offset: Vec<i32>,
    pub positive_bucket_counts: Vec<Vec<u64>>,
    pub negative_offset: Vec<i32>,
    pub negative_bucket_counts: Vec<Vec<u64>>,
    pub flags: Vec<u32>,
    pub min: NullableColumn<arrow::datatypes::Float64Type>,
    pub max: NullableColumn<arrow::datatypes::Float64Type>,
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
        let start = dp.start_time_unix_nano();
        if start == 0 {
            self.start_time_unix_nano.push_null();
        } else {
            self.start_time_unix_nano.push_value(start as i64);
        }
        self.time_unix_nano.push(dp.time_unix_nano() as i64);
        self.count.push(dp.count());
        match dp.sum() {
            Some(v) => self.sum.push_value(v),
            None => self.sum.push_null(),
        }
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
        match dp.min() {
            Some(v) => self.min.push_value(v),
            None => self.min.push_null(),
        }
        match dp.max() {
            Some(v) => self.max.push_value(v),
            None => self.max.push_null(),
        }
        self.zero_threshold.push(dp.zero_threshold());
        row
    }

    pub fn overwrite<V: ExponentialHistogramDataPointView>(&mut self, row: usize, dp: &V) {
        let start = dp.start_time_unix_nano();
        if start == 0 {
            self.start_time_unix_nano.set_null(row);
        } else {
            self.start_time_unix_nano.set_value(row, start as i64);
        }
        self.time_unix_nano[row] = dp.time_unix_nano() as i64;
        self.count[row] = dp.count();
        match dp.sum() {
            Some(v) => self.sum.set_value(row, v),
            None => self.sum.set_null(row),
        }
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
        match dp.min() {
            Some(v) => self.min.set_value(row, v),
            None => self.min.set_null(row),
        }
        match dp.max() {
            Some(v) => self.max.set_value(row, v),
            None => self.max.set_null(row),
        }
        self.zero_threshold[row] = dp.zero_threshold();
    }

    pub fn len(&self) -> usize {
        self.id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.id.is_empty()
    }

    pub fn finish(&mut self) -> Result<RecordBatch, arrow::error::ArrowError> {
        todo!("Stage 3b: build RecordBatch from SOA columns")
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

/// SOA storage for summary data points.
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
        let start = dp.start_time_unix_nano();
        if start == 0 {
            self.start_time_unix_nano.push_null();
        } else {
            self.start_time_unix_nano.push_value(start as i64);
        }
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
        let start = dp.start_time_unix_nano();
        if start == 0 {
            self.start_time_unix_nano.set_null(row);
        } else {
            self.start_time_unix_nano.set_value(row, start as i64);
        }
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
        todo!("Stage 3b: build RecordBatch from SOA columns")
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
