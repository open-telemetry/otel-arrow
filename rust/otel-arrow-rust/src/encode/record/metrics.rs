// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains builders for record batches for metrics.

use std::sync::Arc;

use arrow::{
    array::{
        Array, LargeListArray, LargeListBuilder, PrimitiveBuilder, RecordBatch, StructArray,
        StructBuilder,
    },
    datatypes::{DataType, Field, Fields, Float64Type, Schema, UInt64Type},
    error::ArrowError,
};

use crate::{
    encode::record::{
        array::{
            ArrayAppend, ArrayAppendNulls, ArrayAppendStr, ArrayOptions, CheckedArrayAppendSlice,
            FixedSizeBinaryArrayBuilder, Float64ArrayBuilder, Int32ArrayBuilder, Int64ArrayBuilder,
            StringArrayBuilder, TimestampNanosecondArrayBuilder, UInt16ArrayBuilder,
            UInt32ArrayBuilder, UInt64ArrayBuilder, dictionary::DictionaryOptions,
        },
        logs::{ResourceBuilder, ScopeBuilder},
    },
    schema::{SpanId, TraceId, consts, no_nulls},
};

use super::array::{
    UInt8ArrayBuilder,
    boolean::{AdaptiveBooleanArrayBuilder, BooleanBuilderOptions},
};

/// Record batch builder for metrics
pub struct MetricsRecordBatchBuilder {
    id: UInt16ArrayBuilder,

    /// the builder for the resource struct for this metric record batch
    pub resource: ResourceBuilder,

    /// the builder for the scope struct for this metric record batch
    pub scope: ScopeBuilder,

    scope_schema_url: StringArrayBuilder,
    metric_type: UInt8ArrayBuilder,
    name: StringArrayBuilder,
    description: StringArrayBuilder,
    unit: StringArrayBuilder,
    aggregation_temporality: Int32ArrayBuilder,
    is_monotonic: AdaptiveBooleanArrayBuilder,
}

impl MetricsRecordBatchBuilder {
    /// Create a new instance of `Metrics`
    #[must_use]
    pub fn new() -> Self {
        Self {
            id: UInt16ArrayBuilder::new(ArrayOptions {
                optional: false,
                dictionary_options: None,
                default_values_optional: false,
            }),
            resource: ResourceBuilder::new(),
            scope: ScopeBuilder::new(),
            scope_schema_url: StringArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: Some(DictionaryOptions::dict8()),
                ..Default::default()
            }),
            metric_type: UInt8ArrayBuilder::new(ArrayOptions {
                optional: false,
                dictionary_options: None,
                ..Default::default()
            }),
            name: StringArrayBuilder::new(ArrayOptions {
                optional: false,
                dictionary_options: Some(DictionaryOptions::dict8()),
                ..Default::default()
            }),
            description: StringArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: Some(DictionaryOptions::dict8()),
                ..Default::default()
            }),
            unit: StringArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: Some(DictionaryOptions::dict8()),
                ..Default::default()
            }),
            aggregation_temporality: Int32ArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: Some(DictionaryOptions::dict8()),
                ..Default::default()
            }),
            is_monotonic: AdaptiveBooleanArrayBuilder::new(BooleanBuilderOptions {
                optional: true,
            }),
        }
    }

    /// Append a value to the `id` array.
    pub fn append_id(&mut self, val: u16) {
        self.id.append_value(&val);
    }

    /// Append a value to the `scope_schema_url` array.
    pub fn append_scope_schema_url(&mut self, val: &str) {
        self.scope_schema_url.append_str(val);
    }

    /// Append a value to the `scope_schema_url` array `count` times.
    pub fn append_scope_schema_url_n(&mut self, val: &str, count: usize) {
        self.scope_schema_url.append_str_n(val, count);
    }

    /// Append a value to the `metric_type` array.
    pub fn append_metric_type(&mut self, val: u8) {
        self.metric_type.append_value(&val);
    }

    /// Append a value to the `name` array.
    pub fn append_name(&mut self, val: &str) {
        self.name.append_str(val);
    }

    /// Append a value to the `description` array.
    pub fn append_description(&mut self, val: &str) {
        self.description.append_str(val);
    }

    /// Append a value to the `unit` array.
    pub fn append_unit(&mut self, val: &str) {
        self.unit.append_str(val);
    }

    /// Append a value to the `aggregation_temporality` array.
    pub fn append_aggregation_temporality(&mut self, val: Option<i32>) {
        match val {
            Some(val) => self.aggregation_temporality.append_value(&val),
            None => self.aggregation_temporality.append_null(),
        }
    }

    /// Append a value to the `is_monotonic` array.
    pub fn append_is_monotonic(&mut self, val: Option<bool>) {
        match val {
            Some(val) => self.is_monotonic.append_value(val),
            None => self.is_monotonic.append_null(),
        }
    }

    /// Construct an OTAP Metrics RecordBatch from the builders.
    pub fn finish(&mut self) -> Result<RecordBatch, ArrowError> {
        let mut fields = Vec::with_capacity(10);
        let mut columns = Vec::with_capacity(10);

        // SAFETY: `expect` is safe here because `AdaptiveArrayBuilder` guarantees that for
        // non-optional arrays, `finish()` will always return an array, even if it is empty.
        let array = self.id.finish().expect("finish returns `Some(array)`");
        fields.push(Field::new(consts::ID, array.data_type().clone(), false));
        columns.push(array);

        let resource = self.resource.finish()?;
        if !resource.is_empty() {
            fields.push(Field::new(
                consts::RESOURCE,
                resource.data_type().clone(),
                true,
            ));
            columns.push(Arc::new(resource));
        }

        let scope = self.scope.finish()?;
        if !scope.is_empty() {
            fields.push(Field::new(consts::SCOPE, scope.data_type().clone(), true));
            columns.push(Arc::new(scope));
        }

        if let Some(array) = self.scope_schema_url.finish() {
            fields.push(Field::new(
                consts::SCHEMA_URL,
                array.data_type().clone(),
                false,
            ));
            columns.push(array);
        }

        // SAFETY: `expect` is safe here because `AdaptiveArrayBuilder` guarantees that for
        // non-optional arrays, `finish()` will always return an array, even if it is empty.
        let array = self
            .metric_type
            .finish()
            .expect("finish returns `Some(array)`");
        fields.push(Field::new(
            consts::METRIC_TYPE,
            array.data_type().clone(),
            false,
        ));
        columns.push(array);

        // SAFETY: `expect` is safe here because `AdaptiveArrayBuilder` guarantees that for
        // non-optional arrays, `finish()` will always return an array, even if it is empty.
        let array = self.name.finish().expect("finish returns `Some(array)`");
        fields.push(Field::new(consts::NAME, array.data_type().clone(), false));
        columns.push(array);

        if let Some(array) = self.description.finish() {
            fields.push(Field::new(
                consts::DESCRIPTION,
                array.data_type().clone(),
                false,
            ));
            columns.push(array);
        }

        if let Some(array) = self.unit.finish() {
            fields.push(Field::new(consts::UNIT, array.data_type().clone(), false));
            columns.push(array);
        }

        if let Some(array) = self.aggregation_temporality.finish() {
            fields.push(Field::new(
                consts::AGGREGATION_TEMPORALITY,
                array.data_type().clone(),
                true,
            ));
            columns.push(array);
        }

        if let Some(array) = self.is_monotonic.finish() {
            fields.push(Field::new(
                consts::IS_MONOTONIC,
                array.data_type().clone(),
                true,
            ));
            columns.push(array);
        }

        RecordBatch::try_new(Arc::new(Schema::new(fields)), check(columns))
    }
}

fn check(v: Vec<Arc<dyn Array + 'static>>) -> Vec<Arc<dyn Array + 'static>> {
    let lens: Vec<usize> = v.iter().map(|x| x.len()).collect();
    let len = lens.first().unwrap_or(&0);
    for other in &lens[1..] {
        if len != other {
            panic!("boo {lens:?}");
        }
    }
    v
}

/// Record batch builder for NumberDataPoints
pub struct NumberDataPointsRecordBatchBuilder {
    id: UInt32ArrayBuilder,
    parent_id: UInt16ArrayBuilder,
    start_time_unix_nano: TimestampNanosecondArrayBuilder,
    time_unix_nano: TimestampNanosecondArrayBuilder,
    int_value: Int64ArrayBuilder,
    double_value: Float64ArrayBuilder,
    flags: UInt32ArrayBuilder,
}

impl NumberDataPointsRecordBatchBuilder {
    /// Create a new instance of `NumberDataPoints`
    #[must_use]
    pub fn new() -> Self {
        Self {
            id: UInt32ArrayBuilder::new(ArrayOptions {
                optional: false,
                dictionary_options: None,
                default_values_optional: false,
            }),
            parent_id: UInt16ArrayBuilder::new(ArrayOptions {
                optional: false,
                dictionary_options: None,
                default_values_optional: false,
            }),
            start_time_unix_nano: TimestampNanosecondArrayBuilder::new(ArrayOptions {
                optional: false,
                dictionary_options: None,
                ..Default::default()
            }),
            time_unix_nano: TimestampNanosecondArrayBuilder::new(ArrayOptions {
                optional: false,
                dictionary_options: None,
                ..Default::default()
            }),
            int_value: Int64ArrayBuilder::new(ArrayOptions {
                optional: false,
                dictionary_options: None,
                ..Default::default()
            }),
            double_value: Float64ArrayBuilder::new(ArrayOptions {
                optional: false,
                dictionary_options: None,
                ..Default::default()
            }),
            flags: UInt32ArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                ..Default::default()
            }),
        }
    }

    /// Append a value to the `id` array.
    pub fn append_id(&mut self, val: u32) {
        self.id.append_value(&val);
    }

    /// Append a value to the `parent_id` array.
    pub fn append_parent_id(&mut self, val: u16) {
        self.parent_id.append_value(&val);
    }

    /// Append a value to the `start_time_unix_nano` array.
    pub fn append_start_time_unix_nano(&mut self, val: Option<i64>) {
        match val {
            Some(val) => self.start_time_unix_nano.append_value(&val),
            None => self.start_time_unix_nano.append_null(),
        }
    }

    /// Append a value to the `time_unix_nano` array.
    pub fn append_time_unix_nano(&mut self, val: i64) {
        self.time_unix_nano.append_value(&val);
    }

    /// Append a value to the `int_value` array.
    pub fn append_int_value(&mut self, val: Option<i64>) {
        match val {
            Some(val) => self.int_value.append_value(&val),
            None => self.int_value.append_null(),
        }
    }

    /// Append a value to the `double_value` array.
    pub fn append_double_value(&mut self, val: Option<f64>) {
        match val {
            Some(val) => self.double_value.append_value(&val),
            None => self.double_value.append_null(),
        }
    }

    /// Append a value to the `flags` array.
    pub fn append_flags(&mut self, val: u32) {
        self.flags.append_value(&val);
    }

    /// Construct an OTAP NumberDataPoints RecordBatch from the builders.
    pub fn finish(&mut self) -> Result<RecordBatch, ArrowError> {
        let mut fields = Vec::with_capacity(7);
        let mut columns = Vec::with_capacity(7);

        // SAFETY: `expect` is safe here because `AdaptiveArrayBuilder` guarantees that for
        // non-optional arrays, `finish()` will always return an array, even if it is empty.
        let array = self.id.finish().expect("finish returns `Some(array)`");
        fields.push(Field::new(consts::ID, array.data_type().clone(), false));
        columns.push(array);

        // SAFETY: `expect` is safe here because `AdaptiveArrayBuilder` guarantees that for
        // non-optional arrays, `finish()` will always return an array, even if it is empty.
        let array = self
            .parent_id
            .finish()
            .expect("finish returns `Some(array)`");
        fields.push(Field::new(
            consts::PARENT_ID,
            array.data_type().clone(),
            false,
        ));
        columns.push(array);

        // SAFETY: `expect` is safe here because `AdaptiveArrayBuilder` guarantees that for
        // non-optional arrays, `finish()` will always return an array, even if it is empty.
        let array = self
            .start_time_unix_nano
            .finish()
            .expect("finish returns `Some(array)`");
        fields.push(Field::new(
            consts::START_TIME_UNIX_NANO,
            array.data_type().clone(),
            true,
        ));
        columns.push(array);

        // SAFETY: `expect` is safe here because `AdaptiveArrayBuilder` guarantees that for
        // non-optional arrays, `finish()` will always return an array, even if it is empty.
        let array = self
            .time_unix_nano
            .finish()
            .expect("finish returns `Some(array)`");
        fields.push(Field::new(
            consts::TIME_UNIX_NANO,
            array.data_type().clone(),
            false,
        ));
        columns.push(array);

        // SAFETY: `expect` is safe here because `AdaptiveArrayBuilder` guarantees that for
        // non-optional arrays, `finish()` will always return an array, even if it is empty.
        let array = self
            .int_value
            .finish()
            .expect("finish returns `Some(array)`");
        fields.push(Field::new(
            consts::INT_VALUE,
            array.data_type().clone(),
            true,
        ));
        columns.push(array);

        // SAFETY: `expect` is safe here because `AdaptiveArrayBuilder` guarantees that for
        // non-optional arrays, `finish()` will always return an array, even if it is empty.
        let array = self
            .double_value
            .finish()
            .expect("finish returns `Some(array)`");
        fields.push(Field::new(
            consts::DOUBLE_VALUE,
            array.data_type().clone(),
            true,
        ));
        columns.push(array);

        if let Some(array) = self.flags.finish() {
            fields.push(Field::new(consts::FLAGS, array.data_type().clone(), false));
            columns.push(array);
        }

        RecordBatch::try_new(Arc::new(Schema::new(fields)), check(columns))
    }
}

/// Record batch builder for Exemplars
pub struct ExemplarsRecordBatchBuilder {
    id: UInt32ArrayBuilder,
    parent_id: UInt32ArrayBuilder,
    time_unix_nano: TimestampNanosecondArrayBuilder,
    int_value: Int64ArrayBuilder,
    double_value: Float64ArrayBuilder,
    span_id: FixedSizeBinaryArrayBuilder,
    trace_id: FixedSizeBinaryArrayBuilder,
}

impl ExemplarsRecordBatchBuilder {
    /// Create a new instance of `Exemplars`
    #[must_use]
    pub fn new() -> Self {
        Self {
            id: UInt32ArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                default_values_optional: false,
            }),
            parent_id: UInt32ArrayBuilder::new(ArrayOptions {
                optional: false,
                dictionary_options: None,
                default_values_optional: false,
            }),
            time_unix_nano: TimestampNanosecondArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                ..Default::default()
            }),
            int_value: Int64ArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                ..Default::default()
            }),
            double_value: Float64ArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                ..Default::default()
            }),
            span_id: FixedSizeBinaryArrayBuilder::new_with_args(
                ArrayOptions {
                    optional: true,
                    dictionary_options: Some(DictionaryOptions::dict8()),
                    ..Default::default()
                },
                8,
            ),
            trace_id: FixedSizeBinaryArrayBuilder::new_with_args(
                ArrayOptions {
                    optional: true,
                    dictionary_options: Some(DictionaryOptions::dict8()),
                    ..Default::default()
                },
                16,
            ),
        }
    }

    /// Append a value to the `id` array.
    pub fn append_id(&mut self, val: u32) {
        self.id.append_value(&val);
    }

    /// Append a value to the `parent_id` array.
    pub fn append_parent_id(&mut self, val: u32) {
        self.parent_id.append_value(&val);
    }

    /// Append a value to the `time_unix_nano` array.
    pub fn append_time_unix_nano(&mut self, val: i64) {
        self.time_unix_nano.append_value(&val);
    }

    /// Append a value to the `int_value` array.
    pub fn append_int_value(&mut self, val: i64) {
        self.int_value.append_value(&val);
    }

    /// Append a value to the `double_value` array.
    pub fn append_double_value(&mut self, val: f64) {
        self.double_value.append_value(&val);
    }

    /// Append a value to the `span_id` array.
    pub fn append_span_id(&mut self, val: &SpanId) -> Result<(), ArrowError> {
        self.span_id.append_slice(val)
    }

    /// Append a value to the `trace_id` array.
    pub fn append_trace_id(&mut self, val: &TraceId) -> Result<(), ArrowError> {
        self.trace_id.append_slice(val)
    }

    /// Construct an OTAP Exemplars RecordBatch from the builders.
    pub fn finish(&mut self) -> Result<RecordBatch, ArrowError> {
        let mut fields = Vec::with_capacity(7);
        let mut columns = Vec::with_capacity(7);

        if let Some(array) = self.id.finish() {
            fields.push(Field::new(consts::ID, array.data_type().clone(), false));
            columns.push(array);
        }

        // SAFETY: `expect` is safe here because `AdaptiveArrayBuilder` guarantees that for
        // non-optional arrays, `finish()` will always return an array, even if it is empty.
        let array = self
            .parent_id
            .finish()
            .expect("finish returns `Some(array)`");
        fields.push(Field::new(
            consts::PARENT_ID,
            array.data_type().clone(),
            false,
        ));
        columns.push(array);

        if let Some(array) = self.time_unix_nano.finish() {
            fields.push(Field::new(
                consts::TIME_UNIX_NANO,
                array.data_type().clone(),
                false,
            ));
            columns.push(array);
        }

        if let Some(array) = self.int_value.finish() {
            fields.push(Field::new(
                consts::INT_VALUE,
                array.data_type().clone(),
                false,
            ));
            columns.push(array);
        }

        if let Some(array) = self.double_value.finish() {
            fields.push(Field::new(
                consts::DOUBLE_VALUE,
                array.data_type().clone(),
                false,
            ));
            columns.push(array);
        }

        if let Some(array) = self.span_id.finish() {
            fields.push(Field::new(
                consts::SPAN_ID,
                array.data_type().clone(),
                false,
            ));
            columns.push(array);
        }

        if let Some(array) = self.trace_id.finish() {
            fields.push(Field::new(
                consts::TRACE_ID,
                array.data_type().clone(),
                false,
            ));
            columns.push(array);
        }

        RecordBatch::try_new(Arc::new(Schema::new(fields)), check(columns))
    }
}

/// Record batch builder for QuantileValues
///
/// Ultimately, what we want is a ListOf(StructOf(quantile, value)).
pub struct QuantileRecordBatchBuilder {
    lists: LargeListBuilder<StructBuilder>,
}

impl QuantileRecordBatchBuilder {
    /// Create a new instance of `Quantile`
    #[must_use]
    pub fn new() -> Self {
        Self {
            lists: LargeListBuilder::new(StructBuilder::from_fields(
                vec![
                    Field::new(consts::SUMMARY_QUANTILE, DataType::Float64, false),
                    Field::new(consts::SUMMARY_VALUE, DataType::Float64, false),
                ],
                2,
            )),
        }
    }

    /// Append a (possibly empty) sequence of quantile-value pairs.
    pub fn append(&mut self, val: impl Iterator<Item = (f64, f64)>) {
        let mut val = val.peekable();
        if val.peek().is_some() {
            let builder = self.lists.values();
            let (left, right) = builder.field_builders_mut().split_at_mut(1);

            // SAFETY: These four `expect` calls should never fire since by construction, we have
            // exactly two fields in the inner struct array and each field is of type `f64`.
            let quantile_builder: &mut PrimitiveBuilder<Float64Type> = left
                .get_mut(0)
                .expect("we should have exactly two fields")
                .as_any_mut()
                .downcast_mut()
                .expect("`quantile` field builder should be f64");
            let value_builder: &mut PrimitiveBuilder<Float64Type> = right
                .get_mut(0)
                .expect("we should have exactly two fields")
                .as_any_mut()
                .downcast_mut()
                .expect("`value` field builder should be f64");

            let mut count: usize = 0;
            for (quantile, value) in val {
                quantile_builder.append_value(quantile);
                value_builder.append_value(value);
                count += 1;
            }

            // Why keep a second loop when this one statement could go in the first one? Because
            // then we'd have multiple mutable borrows to `builder`.
            for _ in 0..count {
                builder.append(true);
            }
        }

        // Conceptually, we always store some sequence of pairs; sometimes that sequence is empty,
        // but it is never null!
        self.lists.append(true);
    }

    /// Construct an OTAP Quantile `StructArray` from the builders.
    pub fn finish(&mut self) -> Option<LargeListArray> {
        let array = self.lists.finish();
        (!array.is_empty()).then_some(array)
    }
}

/// Record batch builder for SummaryDataPoints
pub struct SummaryDataPointsRecordBatchBuilder {
    id: UInt32ArrayBuilder,
    parent_id: UInt16ArrayBuilder,
    start_time_unix_nano: TimestampNanosecondArrayBuilder,
    time_unix_nano: TimestampNanosecondArrayBuilder,
    count: UInt64ArrayBuilder,
    sum: Float64ArrayBuilder,

    /// the builder for quantile-value pairs
    pub quantile_values: QuantileRecordBatchBuilder,
    flags: UInt32ArrayBuilder,
}

impl SummaryDataPointsRecordBatchBuilder {
    /// Create a new instance of `SummaryDataPoints`
    #[must_use]
    pub fn new() -> Self {
        Self {
            id: UInt32ArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                default_values_optional: false,
            }),
            parent_id: UInt16ArrayBuilder::new(ArrayOptions {
                optional: false,
                dictionary_options: None,
                default_values_optional: false,
            }),
            start_time_unix_nano: TimestampNanosecondArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                ..Default::default()
            }),
            time_unix_nano: TimestampNanosecondArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                ..Default::default()
            }),
            count: UInt64ArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                ..Default::default()
            }),
            sum: Float64ArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                ..Default::default()
            }),
            quantile_values: QuantileRecordBatchBuilder::new(),
            flags: UInt32ArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                ..Default::default()
            }),
        }
    }

    /// Append a value to the `id` array.
    pub fn append_id(&mut self, val: u32) {
        self.id.append_value(&val);
    }

    /// Append a value to the `parent_id` array.
    pub fn append_parent_id(&mut self, val: u16) {
        self.parent_id.append_value(&val);
    }

    /// Append a value to the `start_time_unix_nano` array.
    pub fn append_start_time_unix_nano(&mut self, val: i64) {
        self.start_time_unix_nano.append_value(&val);
    }

    /// Append a value to the `time_unix_nano` array.
    pub fn append_time_unix_nano(&mut self, val: i64) {
        self.time_unix_nano.append_value(&val);
    }

    /// Append a value to the `count` array.
    pub fn append_count(&mut self, val: u64) {
        self.count.append_value(&val);
    }

    /// Append a value to the `sum` array.
    pub fn append_sum(&mut self, val: f64) {
        self.sum.append_value(&val);
    }

    /// Append a value to the `flags` array.
    pub fn append_flags(&mut self, val: u32) {
        self.flags.append_value(&val);
    }

    /// Construct an OTAP SummaryDataPoints RecordBatch from the builders.
    pub fn finish(&mut self) -> Result<RecordBatch, ArrowError> {
        let mut fields = Vec::with_capacity(8);
        let mut columns = Vec::with_capacity(8);

        if let Some(array) = self.id.finish() {
            fields.push(Field::new(consts::ID, array.data_type().clone(), false));
            columns.push(array);
        }

        // SAFETY: `expect` is safe here because `AdaptiveArrayBuilder` guarantees that for
        // non-optional arrays, `finish()` will always return an array, even if it is empty.
        let array = self
            .parent_id
            .finish()
            .expect("finish returns `Some(array)`");
        fields.push(Field::new(
            consts::PARENT_ID,
            array.data_type().clone(),
            false,
        ));
        columns.push(array);

        if let Some(array) = self.start_time_unix_nano.finish() {
            fields.push(Field::new(
                consts::START_TIME_UNIX_NANO,
                array.data_type().clone(),
                false,
            ));
            columns.push(array);
        }

        if let Some(array) = self.time_unix_nano.finish() {
            fields.push(Field::new(
                consts::TIME_UNIX_NANO,
                array.data_type().clone(),
                false,
            ));
            columns.push(array);
        }

        if let Some(array) = self.count.finish() {
            fields.push(Field::new(
                consts::SUMMARY_COUNT,
                array.data_type().clone(),
                false,
            ));
            columns.push(array);
        }

        if let Some(array) = self.sum.finish() {
            fields.push(Field::new(
                consts::SUMMARY_SUM,
                array.data_type().clone(),
                false,
            ));
            columns.push(array);
        }

        if let Some(array) = self.quantile_values.finish() {
            fields.push(Field::new(
                consts::SUMMARY_QUANTILE_VALUES,
                array.data_type().clone(),
                false,
            ));
            columns.push(Arc::new(array));
        }

        if let Some(array) = self.flags.finish() {
            fields.push(Field::new(consts::FLAGS, array.data_type().clone(), false));
            columns.push(array);
        }

        RecordBatch::try_new(Arc::new(Schema::new(fields)), check(columns))
    }
}

/// Record batch builder for HistogramDataPoints
pub struct HistogramDataPointsRecordBatchBuilder {
    id: UInt32ArrayBuilder,
    parent_id: UInt16ArrayBuilder,
    start_time_unix_nano: TimestampNanosecondArrayBuilder,
    time_unix_nano: TimestampNanosecondArrayBuilder,
    count: UInt64ArrayBuilder,
    bucket_counts: LargeListBuilder<PrimitiveBuilder<UInt64Type>>,
    explicit_bounds: LargeListBuilder<PrimitiveBuilder<Float64Type>>,
    sum: Float64ArrayBuilder,
    flags: UInt32ArrayBuilder,
    min: Float64ArrayBuilder,
    max: Float64ArrayBuilder,
}

impl HistogramDataPointsRecordBatchBuilder {
    /// Create a new instance of `HistogramDataPoints`
    #[must_use]
    pub fn new() -> Self {
        Self {
            id: UInt32ArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                default_values_optional: false,
            }),
            parent_id: UInt16ArrayBuilder::new(ArrayOptions {
                optional: false,
                dictionary_options: None,
                default_values_optional: false,
            }),
            start_time_unix_nano: TimestampNanosecondArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                ..Default::default()
            }),
            time_unix_nano: TimestampNanosecondArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                ..Default::default()
            }),
            count: UInt64ArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                ..Default::default()
            }),
            bucket_counts: LargeListBuilder::new(PrimitiveBuilder::new()),
            explicit_bounds: LargeListBuilder::new(PrimitiveBuilder::new()),
            sum: Float64ArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                ..Default::default()
            }),
            flags: UInt32ArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                ..Default::default()
            }),
            min: Float64ArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                ..Default::default()
            }),
            max: Float64ArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                ..Default::default()
            }),
        }
    }

    /// Append a value to the `id` array.
    pub fn append_id(&mut self, val: u32) {
        self.id.append_value(&val);
    }

    /// Append a value to the `parent_id` array.
    pub fn append_parent_id(&mut self, val: u16) {
        self.parent_id.append_value(&val);
    }

    /// Append a value to the `start_time_unix_nano` array.
    pub fn append_start_time_unix_nano(&mut self, val: i64) {
        self.start_time_unix_nano.append_value(&val);
    }

    /// Append a value to the `time_unix_nano` array.
    pub fn append_time_unix_nano(&mut self, val: i64) {
        self.time_unix_nano.append_value(&val);
    }

    /// Append a value to the `count` array.
    pub fn append_count(&mut self, val: u64) {
        self.count.append_value(&val);
    }

    /// Append a value to the `bucket_counts` array.
    pub fn append_bucket_counts(&mut self, val: impl Iterator<Item = u64>) {
        self.bucket_counts.append_value(val.map(Some))
    }

    /// Append a value to the `explicit_bounds` array.
    pub fn append_explicit_bounds(&mut self, val: impl Iterator<Item = f64>) {
        self.explicit_bounds.append_value(val.map(Some))
    }

    /// Append a value to the `sum` array.
    pub fn append_sum(&mut self, val: Option<f64>) {
        match val {
            Some(val) => self.sum.append_value(&val),
            None => self.sum.append_null(),
        }
    }

    /// Append a value to the `flags` array.
    pub fn append_flags(&mut self, val: u32) {
        self.flags.append_value(&val);
    }

    /// Append a value to the `min` array.
    pub fn append_min(&mut self, val: Option<f64>) {
        match val {
            Some(val) => self.min.append_value(&val),
            None => self.min.append_null(),
        }
    }

    /// Append a value to the `max` array.
    pub fn append_max(&mut self, val: Option<f64>) {
        match val {
            Some(val) => self.max.append_value(&val),
            None => self.max.append_null(),
        }
    }

    /// Construct an OTAP HistogramDataPoints RecordBatch from the builders.
    pub fn finish(&mut self) -> Result<RecordBatch, ArrowError> {
        let mut fields = Vec::with_capacity(11);
        let mut columns = Vec::with_capacity(11);

        if let Some(array) = self.id.finish() {
            fields.push(Field::new(consts::ID, array.data_type().clone(), false));
            columns.push(array);
        }

        // SAFETY: `expect` is safe here because `AdaptiveArrayBuilder` guarantees that for
        // non-optional arrays, `finish()` will always return an array, even if it is empty.
        let array = self
            .parent_id
            .finish()
            .expect("finish returns `Some(array)`");
        fields.push(Field::new(
            consts::PARENT_ID,
            array.data_type().clone(),
            false,
        ));
        columns.push(array);

        if let Some(array) = self.start_time_unix_nano.finish() {
            fields.push(Field::new(
                consts::START_TIME_UNIX_NANO,
                array.data_type().clone(),
                false,
            ));
            columns.push(array);
        }

        if let Some(array) = self.time_unix_nano.finish() {
            fields.push(Field::new(
                consts::TIME_UNIX_NANO,
                array.data_type().clone(),
                false,
            ));
            columns.push(array);
        }

        if let Some(array) = self.count.finish() {
            fields.push(Field::new(
                consts::HISTOGRAM_COUNT,
                array.data_type().clone(),
                false,
            ));
            columns.push(array);
        }

        let array = no_nulls(self.bucket_counts.finish());
        if !array.is_empty() {
            fields.push(Field::new_large_list(
                consts::HISTOGRAM_BUCKET_COUNTS,
                Field::new_list_field(DataType::UInt64, false),
                false,
            ));
            columns.push(Arc::new(array));
        }

        let array = no_nulls(self.explicit_bounds.finish());
        if !array.is_empty() {
            fields.push(Field::new_large_list(
                consts::HISTOGRAM_EXPLICIT_BOUNDS,
                Field::new("item", DataType::Float64, false),
                false,
            ));
            columns.push(Arc::new(array));
        }

        if let Some(array) = self.sum.finish() {
            fields.push(Field::new(
                consts::HISTOGRAM_SUM,
                array.data_type().clone(),
                true,
            ));
            columns.push(array);
        }

        if let Some(array) = self.flags.finish() {
            fields.push(Field::new(consts::FLAGS, array.data_type().clone(), false));
            columns.push(array);
        }

        if let Some(array) = self.min.finish() {
            fields.push(Field::new(
                consts::HISTOGRAM_MIN,
                array.data_type().clone(),
                true,
            ));
            columns.push(array);
        }

        if let Some(array) = self.max.finish() {
            fields.push(Field::new(
                consts::HISTOGRAM_MAX,
                array.data_type().clone(),
                true,
            ));
            columns.push(array);
        }

        RecordBatch::try_new(Arc::new(Schema::new(fields)), check(columns))
    }
}

/// Record batch builder for ExponentialHistogramDataPoints
pub struct ExponentialHistogramDataPointsRecordBatchBuilder {
    id: UInt32ArrayBuilder,
    parent_id: UInt16ArrayBuilder,
    start_time_unix_nano: TimestampNanosecondArrayBuilder,
    time_unix_nano: TimestampNanosecondArrayBuilder,
    count: UInt64ArrayBuilder,
    sum: Float64ArrayBuilder,
    scale: Int32ArrayBuilder,
    zero_count: UInt64ArrayBuilder,
    /// The builder for positive buckets.
    pub positive: BucketsRecordBatchBuilder,
    /// The builder for negative buckets.
    pub negative: BucketsRecordBatchBuilder,
    flags: UInt32ArrayBuilder,
    min: Float64ArrayBuilder,
    max: Float64ArrayBuilder,
}

impl ExponentialHistogramDataPointsRecordBatchBuilder {
    /// Create a new instance of `ExponentialHistogramDataPoints`
    #[must_use]
    pub fn new() -> Self {
        Self {
            id: UInt32ArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                default_values_optional: false,
            }),
            parent_id: UInt16ArrayBuilder::new(ArrayOptions {
                optional: false,
                dictionary_options: None,
                default_values_optional: false,
            }),
            start_time_unix_nano: TimestampNanosecondArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                ..Default::default()
            }),
            time_unix_nano: TimestampNanosecondArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                ..Default::default()
            }),
            count: UInt64ArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                ..Default::default()
            }),
            sum: Float64ArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                ..Default::default()
            }),
            scale: Int32ArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                ..Default::default()
            }),
            zero_count: UInt64ArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                ..Default::default()
            }),
            positive: BucketsRecordBatchBuilder::new(),
            negative: BucketsRecordBatchBuilder::new(),
            flags: UInt32ArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                ..Default::default()
            }),
            min: Float64ArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                ..Default::default()
            }),
            max: Float64ArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                ..Default::default()
            }),
        }
    }

    /// Append a value to the `id` array.
    pub fn append_id(&mut self, val: u32) {
        self.id.append_value(&val);
    }

    /// Append a value to the `parent_id` array.
    pub fn append_parent_id(&mut self, val: u16) {
        self.parent_id.append_value(&val);
    }

    /// Append a value to the `start_time_unix_nano` array.
    pub fn append_start_time_unix_nano(&mut self, val: i64) {
        self.start_time_unix_nano.append_value(&val);
    }

    /// Append a value to the `time_unix_nano` array.
    pub fn append_time_unix_nano(&mut self, val: i64) {
        self.time_unix_nano.append_value(&val);
    }

    /// Append a value to the `count` array.
    pub fn append_count(&mut self, val: u64) {
        self.count.append_value(&val);
    }

    /// Append a value to the `sum` array.
    pub fn append_sum(&mut self, val: Option<f64>) {
        match val {
            Some(val) => self.sum.append_value(&val),
            None => self.sum.append_null(),
        }
    }

    /// Append a value to the `scale` array.
    pub fn append_scale(&mut self, val: i32) {
        self.scale.append_value(&val);
    }

    /// Append a value to the `zero_count` array.
    pub fn append_zero_count(&mut self, val: u64) {
        self.zero_count.append_value(&val);
    }

    /// Append a value to the `flags` array.
    pub fn append_flags(&mut self, val: u32) {
        self.flags.append_value(&val);
    }

    /// Append a value to the `min` array.
    pub fn append_min(&mut self, val: Option<f64>) {
        match val {
            Some(val) => self.min.append_value(&val),
            None => self.min.append_null(),
        }
    }

    /// Append a value to the `max` array.
    pub fn append_max(&mut self, val: Option<f64>) {
        match val {
            Some(val) => self.max.append_value(&val),
            None => self.max.append_null(),
        }
    }

    /// Construct an OTAP ExponentialHistogramDataPoints RecordBatch from the builders.
    pub fn finish(&mut self) -> Result<RecordBatch, ArrowError> {
        let mut fields = Vec::with_capacity(13);
        let mut columns = Vec::with_capacity(13);

        if let Some(array) = self.id.finish() {
            fields.push(Field::new(consts::ID, array.data_type().clone(), false));
            columns.push(array);
        }

        // SAFETY: `expect` is safe here because `AdaptiveArrayBuilder` guarantees that for
        // non-optional arrays, `finish()` will always return an array, even if it is empty.
        let array = self
            .parent_id
            .finish()
            .expect("finish returns `Some(array)`");
        fields.push(Field::new(
            consts::PARENT_ID,
            array.data_type().clone(),
            false,
        ));
        columns.push(array);

        if let Some(array) = self.start_time_unix_nano.finish() {
            fields.push(Field::new(
                consts::START_TIME_UNIX_NANO,
                array.data_type().clone(),
                false,
            ));
            columns.push(array);
        }

        if let Some(array) = self.time_unix_nano.finish() {
            fields.push(Field::new(
                consts::TIME_UNIX_NANO,
                array.data_type().clone(),
                false,
            ));
            columns.push(array);
        }

        if let Some(array) = self.count.finish() {
            fields.push(Field::new(
                consts::HISTOGRAM_COUNT,
                array.data_type().clone(),
                false,
            ));
            columns.push(array);
        }

        if let Some(array) = self.sum.finish() {
            fields.push(Field::new(
                consts::HISTOGRAM_SUM,
                array.data_type().clone(),
                true,
            ));
            columns.push(array);
        }

        if let Some(array) = self.scale.finish() {
            fields.push(Field::new(
                consts::EXP_HISTOGRAM_SCALE,
                array.data_type().clone(),
                false,
            ));
            columns.push(array);
        }

        if let Some(array) = self.zero_count.finish() {
            fields.push(Field::new(
                consts::EXP_HISTOGRAM_ZERO_COUNT,
                array.data_type().clone(),
                false,
            ));
            columns.push(array);
        }

        if let Some(array) = self.positive.finish().transpose()? {
            fields.push(Field::new(
                consts::EXP_HISTOGRAM_POSITIVE,
                array.data_type().clone(),
                false,
            ));
            columns.push(Arc::new(array));
        }

        if let Some(array) = self.negative.finish().transpose()? {
            fields.push(Field::new(
                consts::EXP_HISTOGRAM_NEGATIVE,
                array.data_type().clone(),
                false,
            ));
            columns.push(Arc::new(array));
        }

        if let Some(array) = self.flags.finish() {
            fields.push(Field::new(consts::FLAGS, array.data_type().clone(), false));
            columns.push(array);
        }

        if let Some(array) = self.min.finish() {
            fields.push(Field::new(
                consts::HISTOGRAM_MIN,
                array.data_type().clone(),
                true,
            ));
            columns.push(array);
        }

        if let Some(array) = self.max.finish() {
            fields.push(Field::new(
                consts::HISTOGRAM_MAX,
                array.data_type().clone(),
                true,
            ));
            columns.push(array);
        }

        RecordBatch::try_new(Arc::new(Schema::new(fields)), check(columns))
    }
}

/// Record batch builder for ExponentialHistogram Buckets.
///
/// There are no nulls here at all; at the protobuf level, `Buckets` is required but `prost`
/// represents this as an `Option<Buckets>`, so we handle cases of missing bucket data using an
/// offset of zero and an empty counts slice.
pub struct BucketsRecordBatchBuilder {
    offset: Int32ArrayBuilder,
    bucket_counts: LargeListBuilder<PrimitiveBuilder<UInt64Type>>,
}

impl BucketsRecordBatchBuilder {
    /// Create a new instance of `BucketsRecordBatchBuilder`
    #[must_use]
    pub fn new() -> Self {
        Self {
            offset: Int32ArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                default_values_optional: false,
            }),
            bucket_counts: LargeListBuilder::new(PrimitiveBuilder::new()),
        }
    }

    /// Append a new value.
    pub fn append(&mut self, val: Option<(i32, impl Iterator<Item = u64>)>) {
        match val {
            Some((offset, bucket_counts)) => {
                self.offset.append_value(&offset);
                self.bucket_counts.append_value(bucket_counts.map(Some));
            }
            None => {
                self.offset.append_value(&0);
                self.bucket_counts.append(true);
            }
        }
    }

    /// Construct an OTAP ExponentialHistogramDataPointsBuckets RecordBatch from the builders.
    pub fn finish(&mut self) -> Option<Result<StructArray, ArrowError>> {
        let mut fields = Vec::with_capacity(2);
        let mut columns = Vec::with_capacity(2);

        if let Some(array) = self.offset.finish() {
            fields.push(Field::new(
                consts::EXP_HISTOGRAM_OFFSET,
                array.data_type().clone(),
                false,
            ));
            columns.push(array);
        }

        let array = no_nulls(self.bucket_counts.finish());
        if !array.is_empty() {
            fields.push(Field::new(
                consts::EXP_HISTOGRAM_BUCKET_COUNTS,
                array.data_type().clone(),
                false,
            ));
            columns.push(Arc::new(array));
        }

        let length = columns.first()?.len();
        Some(StructArray::try_new_with_length(
            Fields::from(fields),
            columns,
            None,
            length,
        ))
    }
}
