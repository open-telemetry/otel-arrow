// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains builders for record batches for metrics.

use std::sync::Arc;

use arrow::{
    array::{Array, RecordBatch, StructArray},
    datatypes::{Field, Fields, Schema},
    error::ArrowError,
};

use crate::{
    encode::record::{
        array::{
            ArrayAppend, ArrayAppendNulls, ArrayAppendStr, ArrayOptions, CheckedArrayAppendSlice,
            DurationNanosecondArrayBuilder, FixedSizeBinaryArrayBuilder, Int32ArrayBuilder,
            StringArrayBuilder, TimestampNanosecondArrayBuilder, UInt16ArrayBuilder,
            UInt32ArrayBuilder, dictionary::DictionaryOptions,
        },
        logs::{ResourceBuilder, ScopeBuilder},
    },
    schema::{FieldExt, SpanId, TraceId, consts},
};

/// Record batch builder for traces
pub struct TracesRecordBatchBuilder {
    id: UInt16ArrayBuilder,

    /// the builder for the resource struct for this metric record batch
    pub resource: ResourceBuilder,

    /// the builder for the scope struct for this metric record batch
    pub scope: ScopeBuilder,

    schema_url: StringArrayBuilder,
    start_time_unix_nano: TimestampNanosecondArrayBuilder,
    duration_time_unix_nano: DurationNanosecondArrayBuilder,
    trace_id: FixedSizeBinaryArrayBuilder,
    span_id: FixedSizeBinaryArrayBuilder,
    trace_state: StringArrayBuilder,
    parent_span_id: FixedSizeBinaryArrayBuilder,
    name: StringArrayBuilder,
    kind: Int32ArrayBuilder,
    dropped_attributes_count: UInt32ArrayBuilder,
    dropped_events_count: UInt32ArrayBuilder,
    dropped_links_count: UInt32ArrayBuilder,

    /// the builder for the Status StructArray
    pub status: StatusRecordBatchBuilder,
}

impl TracesRecordBatchBuilder {
    /// Create a new instance of `Traces`
    #[must_use]
    pub fn new() -> Self {
        Self {
            id: UInt16ArrayBuilder::new(ArrayOptions {
                optional: false,
                dictionary_options: None,
                ..Default::default()
            }),
            resource: ResourceBuilder::new(),
            scope: ScopeBuilder::new(),
            schema_url: StringArrayBuilder::new(ArrayOptions {
                optional: false,
                dictionary_options: Some(DictionaryOptions::dict8()),
                ..Default::default()
            }),
            start_time_unix_nano: TimestampNanosecondArrayBuilder::new(ArrayOptions {
                optional: false,
                dictionary_options: None,
                ..Default::default()
            }),
            duration_time_unix_nano: DurationNanosecondArrayBuilder::new(ArrayOptions {
                optional: false,
                dictionary_options: Some(DictionaryOptions::dict8()),
                ..Default::default()
            }),
            trace_id: FixedSizeBinaryArrayBuilder::new_with_args(
                ArrayOptions {
                    optional: false,
                    dictionary_options: None,
                    ..Default::default()
                },
                16,
            ),
            span_id: FixedSizeBinaryArrayBuilder::new_with_args(
                ArrayOptions {
                    optional: false,
                    dictionary_options: None,
                    ..Default::default()
                },
                8,
            ),
            trace_state: StringArrayBuilder::new(ArrayOptions {
                optional: false,
                dictionary_options: Some(DictionaryOptions::dict8()),
                ..Default::default()
            }),
            parent_span_id: FixedSizeBinaryArrayBuilder::new_with_args(
                ArrayOptions {
                    optional: false,
                    dictionary_options: None,
                    ..Default::default()
                },
                8,
            ),
            name: StringArrayBuilder::new(ArrayOptions {
                optional: false,
                dictionary_options: Some(DictionaryOptions::dict8()),
                ..Default::default()
            }),
            kind: Int32ArrayBuilder::new(ArrayOptions {
                optional: false,
                dictionary_options: Some(DictionaryOptions::dict8()),
                ..Default::default()
            }),
            dropped_attributes_count: UInt32ArrayBuilder::new(ArrayOptions {
                optional: false,
                dictionary_options: None,
                ..Default::default()
            }),
            dropped_events_count: UInt32ArrayBuilder::new(ArrayOptions {
                optional: false,
                dictionary_options: None,
                ..Default::default()
            }),
            dropped_links_count: UInt32ArrayBuilder::new(ArrayOptions {
                optional: false,
                dictionary_options: None,
                ..Default::default()
            }),
            status: StatusRecordBatchBuilder::new(),
        }
    }

    /// Append a value to the `id` array.
    pub fn append_id(&mut self, val: Option<u16>) {
        match val {
            Some(val) => self.id.append_value(&val),
            None => self.id.append_null(),
        }
    }

    /// Append a value to the `schema_url` array.
    pub fn append_schema_url(&mut self, val: Option<&str>) {
        match val {
            Some(val) => self.schema_url.append_str(val),
            None => self.schema_url.append_null(),
        }
    }

    /// Append a value to the `schema_url` array `n` times.
    pub fn append_schema_url_n(&mut self, val: Option<&str>, n: usize) {
        match val {
            Some(val) => self.schema_url.append_str_n(val, n),
            None => self.schema_url.append_nulls(n),
        }
    }

    /// Append a value to the `start_time_unix_nano` array.
    pub fn append_start_time_unix_nano(&mut self, val: i64) {
        self.start_time_unix_nano.append_value(&val);
    }

    /// Append a value to the `duration_time_unix_nano` array.
    pub fn append_duration_time_unix_nano(&mut self, val: i64) {
        self.duration_time_unix_nano.append_value(&val);
    }

    /// Append a value to the `trace_id` array.
    pub fn append_trace_id(&mut self, val: TraceId) -> Result<(), ArrowError> {
        self.trace_id.append_slice(&val)
    }

    /// Append a value to the `span_id` array.
    pub fn append_span_id(&mut self, val: SpanId) -> Result<(), ArrowError> {
        self.span_id.append_slice(&val)
    }

    /// Append a value to the `trace_state` array.
    pub fn append_trace_state(&mut self, val: Option<&str>) {
        match val {
            Some(val) => self.trace_state.append_str(val),
            None => self.trace_state.append_null(),
        }
    }

    /// Append a value to the `parent_span_id` array.
    pub fn append_parent_span_id(&mut self, val: Option<SpanId>) -> Result<(), ArrowError> {
        match val {
            Some(val) => self.parent_span_id.append_slice(&val),
            None => {
                self.parent_span_id.append_null();
                Ok(())
            }
        }
    }

    /// Append a value to the `name` array.
    pub fn append_name(&mut self, val: &str) {
        self.name.append_str(val);
    }

    /// Append a value to the `kind` array.
    pub fn append_kind(&mut self, val: Option<i32>) {
        match val {
            Some(val) => self.kind.append_value(&val),
            None => self.kind.append_null(),
        }
    }

    /// Append a value to the `dropped_attributes_count` array.
    pub fn append_dropped_attributes_count(&mut self, val: Option<u32>) {
        match val {
            Some(val) => self.dropped_attributes_count.append_value(&val),
            None => self.dropped_attributes_count.append_null(),
        }
    }

    /// Append a value to the `dropped_events_count` array.
    pub fn append_dropped_events_count(&mut self, val: Option<u32>) {
        match val {
            Some(val) => self.dropped_events_count.append_value(&val),
            None => self.dropped_events_count.append_null(),
        }
    }

    /// Append a value to the `dropped_links_count` array.
    pub fn append_dropped_links_count(&mut self, val: Option<u32>) {
        match val {
            Some(val) => self.dropped_links_count.append_value(&val),
            None => self.dropped_links_count.append_null(),
        }
    }

    /// Construct an OTAP Traces RecordBatch from the builders.
    pub fn finish(&mut self) -> Result<RecordBatch, ArrowError> {
        let mut fields = Vec::with_capacity(13);
        let mut columns = Vec::with_capacity(13);

        // SAFETY: `expect` is safe here because `AdaptiveArrayBuilder` guarantees that for
        // non-optional arrays, `finish()` will always return an array, even if it is empty.
        let array = self.id.finish().expect("finish returns `Some(array)`");
        fields.push(Field::new(consts::ID, array.data_type().clone(), true).with_plain_encoding());
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

        // SAFETY: `expect` is safe here because `AdaptiveArrayBuilder` guarantees that for
        // non-optional arrays, `finish()` will always return an array, even if it is empty.
        let array = self
            .schema_url
            .finish()
            .expect("finish returns `Some(array)`");
        fields.push(Field::new(
            consts::SCHEMA_URL,
            array.data_type().clone(),
            true,
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
            false,
        ));
        columns.push(array);

        // SAFETY: `expect` is safe here because `AdaptiveArrayBuilder` guarantees that for
        // non-optional arrays, `finish()` will always return an array, even if it is empty.
        let array = self
            .duration_time_unix_nano
            .finish()
            .expect("finish returns `Some(array)`");
        fields.push(Field::new(
            consts::DURATION_TIME_UNIX_NANO,
            array.data_type().clone(),
            false,
        ));
        columns.push(array);

        // SAFETY: `expect` is safe here because `AdaptiveArrayBuilder` guarantees that for
        // non-optional arrays, `finish()` will always return an array, even if it is empty.
        let array = self
            .trace_id
            .finish()
            .expect("finish returns `Some(array)`");
        fields.push(Field::new(
            consts::TRACE_ID,
            array.data_type().clone(),
            false,
        ));
        columns.push(array);

        // SAFETY: `expect` is safe here because `AdaptiveArrayBuilder` guarantees that for
        // non-optional arrays, `finish()` will always return an array, even if it is empty.
        let array = self.span_id.finish().expect("finish returns `Some(array)`");
        fields.push(Field::new(
            consts::SPAN_ID,
            array.data_type().clone(),
            false,
        ));
        columns.push(array);

        // SAFETY: `expect` is safe here because `AdaptiveArrayBuilder` guarantees that for
        // non-optional arrays, `finish()` will always return an array, even if it is empty.
        let array = self
            .trace_state
            .finish()
            .expect("finish returns `Some(array)`");
        fields.push(Field::new(
            consts::TRACE_STATE,
            array.data_type().clone(),
            true,
        ));
        columns.push(array);

        // SAFETY: `expect` is safe here because `AdaptiveArrayBuilder` guarantees that for
        // non-optional arrays, `finish()` will always return an array, even if it is empty.
        let array = self
            .parent_span_id
            .finish()
            .expect("finish returns `Some(array)`");
        fields.push(Field::new(
            consts::PARENT_SPAN_ID,
            array.data_type().clone(),
            true,
        ));
        columns.push(array);

        // SAFETY: `expect` is safe here because `AdaptiveArrayBuilder` guarantees that for
        // non-optional arrays, `finish()` will always return an array, even if it is empty.
        let array = self.name.finish().expect("finish returns `Some(array)`");
        fields.push(Field::new(consts::NAME, array.data_type().clone(), false));
        columns.push(array);

        // SAFETY: `expect` is safe here because `AdaptiveArrayBuilder` guarantees that for
        // non-optional arrays, `finish()` will always return an array, even if it is empty.
        let array = self.kind.finish().expect("finish returns `Some(array)`");
        fields.push(Field::new(consts::KIND, array.data_type().clone(), true));
        columns.push(array);

        // SAFETY: `expect` is safe here because `AdaptiveArrayBuilder` guarantees that for
        // non-optional arrays, `finish()` will always return an array, even if it is empty.
        let array = self
            .dropped_attributes_count
            .finish()
            .expect("finish returns `Some(array)`");
        fields.push(Field::new(
            consts::DROPPED_ATTRIBUTES_COUNT,
            array.data_type().clone(),
            true,
        ));
        columns.push(array);

        // SAFETY: `expect` is safe here because `AdaptiveArrayBuilder` guarantees that for
        // non-optional arrays, `finish()` will always return an array, even if it is empty.
        let array = self
            .dropped_events_count
            .finish()
            .expect("finish returns `Some(array)`");
        fields.push(Field::new(
            consts::DROPPED_EVENTS_COUNT,
            array.data_type().clone(),
            true,
        ));
        columns.push(array);

        // SAFETY: `expect` is safe here because `AdaptiveArrayBuilder` guarantees that for
        // non-optional arrays, `finish()` will always return an array, even if it is empty.
        let array = self
            .dropped_links_count
            .finish()
            .expect("finish returns `Some(array)`");
        fields.push(Field::new(
            consts::DROPPED_LINKS_COUNT,
            array.data_type().clone(),
            true,
        ));
        columns.push(array);

        let array = self.status.finish()?;
        fields.push(Field::new(consts::STATUS, array.data_type().clone(), true));
        columns.push(Arc::new(array));

        RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
    }
}

/// Record batch builder for events
pub struct EventsRecordBatchBuilder {
    id: UInt32ArrayBuilder,
    parent_id: UInt16ArrayBuilder,
    time_unix_nano: TimestampNanosecondArrayBuilder,
    name: StringArrayBuilder,
    dropped_attributes_count: UInt32ArrayBuilder,
}

impl EventsRecordBatchBuilder {
    /// Create a new instance of `Events`
    #[must_use]
    pub fn new() -> Self {
        Self {
            id: UInt32ArrayBuilder::new(ArrayOptions {
                optional: false,
                dictionary_options: None,
                ..Default::default()
            }),
            parent_id: UInt16ArrayBuilder::new(ArrayOptions {
                optional: false,
                dictionary_options: None,
                ..Default::default()
            }),
            time_unix_nano: TimestampNanosecondArrayBuilder::new(ArrayOptions {
                optional: false,
                dictionary_options: None,
                ..Default::default()
            }),
            name: StringArrayBuilder::new(ArrayOptions {
                optional: false,
                dictionary_options: Some(DictionaryOptions::dict8()),
                ..Default::default()
            }),
            dropped_attributes_count: UInt32ArrayBuilder::new(ArrayOptions {
                optional: false,
                dictionary_options: None,
                ..Default::default()
            }),
        }
    }

    /// Append a value to the `id` array.
    pub fn append_id(&mut self, val: Option<u32>) {
        match val {
            Some(val) => self.id.append_value(&val),
            None => self.id.append_null(),
        }
    }

    /// Append a value to the `parent_id` array.
    pub fn append_parent_id(&mut self, val: u16) {
        self.parent_id.append_value(&val);
    }

    /// Append a value to the `time_unix_nano` array.
    pub fn append_time_unix_nano(&mut self, val: Option<i64>) {
        match val {
            Some(val) => self.time_unix_nano.append_value(&val),
            None => self.time_unix_nano.append_null(),
        }
    }

    /// Append a value to the `name` array.
    pub fn append_name(&mut self, val: &str) {
        self.name.append_str(val);
    }

    /// Append a value to the `dropped_attributes_count` array.
    pub fn append_dropped_attributes_count(&mut self, val: Option<u32>) {
        match val {
            Some(val) => self.dropped_attributes_count.append_value(&val),
            None => self.dropped_attributes_count.append_null(),
        }
    }

    /// Construct an OTAP Events RecordBatch from the builders.
    pub fn finish(&mut self) -> Result<RecordBatch, ArrowError> {
        let mut fields = Vec::with_capacity(5);
        let mut columns = Vec::with_capacity(5);

        // SAFETY: `expect` is safe here because `AdaptiveArrayBuilder` guarantees that for
        // non-optional arrays, `finish()` will always return an array, even if it is empty.
        let array = self.id.finish().expect("finish returns `Some(array)`");
        fields.push(Field::new(consts::ID, array.data_type().clone(), true).with_plain_encoding());
        columns.push(array);

        // SAFETY: `expect` is safe here because `AdaptiveArrayBuilder` guarantees that for
        // non-optional arrays, `finish()` will always return an array, even if it is empty.
        let array = self
            .parent_id
            .finish()
            .expect("finish returns `Some(array)`");
        fields.push(
            Field::new(consts::PARENT_ID, array.data_type().clone(), false).with_plain_encoding(),
        );
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
            true,
        ));
        columns.push(array);

        // SAFETY: `expect` is safe here because `AdaptiveArrayBuilder` guarantees that for
        // non-optional arrays, `finish()` will always return an array, even if it is empty.
        let array = self.name.finish().expect("finish returns `Some(array)`");
        fields.push(Field::new(consts::NAME, array.data_type().clone(), false));
        columns.push(array);

        // SAFETY: `expect` is safe here because `AdaptiveArrayBuilder` guarantees that for
        // non-optional arrays, `finish()` will always return an array, even if it is empty.
        let array = self
            .dropped_attributes_count
            .finish()
            .expect("finish returns `Some(array)`");
        fields.push(Field::new(
            consts::DROPPED_ATTRIBUTES_COUNT,
            array.data_type().clone(),
            true,
        ));
        columns.push(array);

        RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
    }
}

/// Record batch builder for links
pub struct LinksRecordBatchBuilder {
    id: UInt32ArrayBuilder,
    parent_id: UInt16ArrayBuilder,
    trace_id: FixedSizeBinaryArrayBuilder,
    span_id: FixedSizeBinaryArrayBuilder,
    trace_state: StringArrayBuilder,
    dropped_attributes_count: UInt32ArrayBuilder,
}

impl LinksRecordBatchBuilder {
    /// Create a new instance of `Links`
    #[must_use]
    pub fn new() -> Self {
        Self {
            id: UInt32ArrayBuilder::new(ArrayOptions {
                optional: false,
                dictionary_options: None,
                ..Default::default()
            }),
            parent_id: UInt16ArrayBuilder::new(ArrayOptions {
                optional: false,
                dictionary_options: None,
                ..Default::default()
            }),
            trace_id: FixedSizeBinaryArrayBuilder::new_with_args(
                ArrayOptions {
                    optional: false,
                    dictionary_options: Some(DictionaryOptions::dict8()),
                    ..Default::default()
                },
                16,
            ),
            span_id: FixedSizeBinaryArrayBuilder::new_with_args(
                ArrayOptions {
                    optional: false,
                    dictionary_options: Some(DictionaryOptions::dict8()),
                    ..Default::default()
                },
                8,
            ),
            trace_state: StringArrayBuilder::new(ArrayOptions {
                optional: false,
                dictionary_options: Some(DictionaryOptions::dict8()),
                ..Default::default()
            }),
            dropped_attributes_count: UInt32ArrayBuilder::new(ArrayOptions {
                optional: false,
                dictionary_options: None,
                ..Default::default()
            }),
        }
    }

    /// Append a value to the `id` array.
    pub fn append_id(&mut self, val: Option<u32>) {
        match val {
            Some(val) => self.id.append_value(&val),
            None => self.id.append_null(),
        }
    }

    /// Append a value to the `parent_id` array.
    pub fn append_parent_id(&mut self, val: u16) {
        self.parent_id.append_value(&val);
    }

    /// Append a value to the `trace_id` array.
    pub fn append_trace_id(&mut self, val: Option<TraceId>) -> Result<(), ArrowError> {
        match val {
            Some(val) => self.trace_id.append_slice(&val),
            None => {
                self.trace_id.append_null();
                Ok(())
            }
        }
    }

    /// Append a value to the `span_id` array.
    pub fn append_span_id(&mut self, val: Option<SpanId>) -> Result<(), ArrowError> {
        match val {
            Some(val) => self.span_id.append_slice(&val),
            None => {
                self.span_id.append_null();
                Ok(())
            }
        }
    }

    /// Append a value to the `trace_state` array.
    pub fn append_trace_state(&mut self, val: Option<&str>) {
        match val {
            Some(val) => self.trace_state.append_str(val),
            None => self.trace_state.append_null(),
        }
    }

    /// Append a value to the `dropped_attributes_count` array.
    pub fn append_dropped_attributes_count(&mut self, val: Option<u32>) {
        match val {
            Some(val) => self.dropped_attributes_count.append_value(&val),
            None => self.dropped_attributes_count.append_null(),
        }
    }

    /// Construct an OTAP Links RecordBatch from the builders.
    pub fn finish(&mut self) -> Result<RecordBatch, ArrowError> {
        let mut fields = Vec::with_capacity(6);
        let mut columns = Vec::with_capacity(6);

        // SAFETY: `expect` is safe here because `AdaptiveArrayBuilder` guarantees that for
        // non-optional arrays, `finish()` will always return an array, even if it is empty.
        let array = self.id.finish().expect("finish returns `Some(array)`");
        fields.push(Field::new(consts::ID, array.data_type().clone(), true).with_plain_encoding());
        columns.push(array);

        // SAFETY: `expect` is safe here because `AdaptiveArrayBuilder` guarantees that for
        // non-optional arrays, `finish()` will always return an array, even if it is empty.
        let array = self
            .parent_id
            .finish()
            .expect("finish returns `Some(array)`");
        fields.push(
            Field::new(consts::PARENT_ID, array.data_type().clone(), false).with_plain_encoding(),
        );
        columns.push(array);

        // SAFETY: `expect` is safe here because `AdaptiveArrayBuilder` guarantees that for
        // non-optional arrays, `finish()` will always return an array, even if it is empty.
        let array = self
            .trace_id
            .finish()
            .expect("finish returns `Some(array)`");
        fields.push(Field::new(
            consts::TRACE_ID,
            array.data_type().clone(),
            true,
        ));
        columns.push(array);

        // SAFETY: `expect` is safe here because `AdaptiveArrayBuilder` guarantees that for
        // non-optional arrays, `finish()` will always return an array, even if it is empty.
        let array = self.span_id.finish().expect("finish returns `Some(array)`");
        fields.push(Field::new(consts::SPAN_ID, array.data_type().clone(), true));
        columns.push(array);

        // SAFETY: `expect` is safe here because `AdaptiveArrayBuilder` guarantees that for
        // non-optional arrays, `finish()` will always return an array, even if it is empty.
        let array = self
            .trace_state
            .finish()
            .expect("finish returns `Some(array)`");
        fields.push(Field::new(
            consts::TRACE_STATE,
            array.data_type().clone(),
            true,
        ));
        columns.push(array);

        // SAFETY: `expect` is safe here because `AdaptiveArrayBuilder` guarantees that for
        // non-optional arrays, `finish()` will always return an array, even if it is empty.
        let array = self
            .dropped_attributes_count
            .finish()
            .expect("finish returns `Some(array)`");
        fields.push(Field::new(
            consts::DROPPED_ATTRIBUTES_COUNT,
            array.data_type().clone(),
            true,
        ));
        columns.push(array);

        RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
    }
}

/// Record batch builder for status
pub struct StatusRecordBatchBuilder {
    code: Int32ArrayBuilder,
    status_message: StringArrayBuilder,
}

impl StatusRecordBatchBuilder {
    /// Create a new instance of `Status`
    #[must_use]
    pub fn new() -> Self {
        Self {
            code: Int32ArrayBuilder::new(ArrayOptions {
                optional: false,
                dictionary_options: Some(DictionaryOptions::dict8()),
                ..Default::default()
            }),
            status_message: StringArrayBuilder::new(ArrayOptions {
                optional: false,
                dictionary_options: Some(DictionaryOptions::dict8()),
                ..Default::default()
            }),
        }
    }

    /// Append a value to the `code` array.
    pub fn append_code(&mut self, val: Option<i32>) {
        match val {
            Some(val) => self.code.append_value(&val),
            None => self.code.append_null(),
        }
    }

    /// Append a value to the `status_message` array.
    pub fn append_status_message(&mut self, val: Option<&str>) {
        match val {
            Some(val) => self.status_message.append_str(val),
            None => self.status_message.append_null(),
        }
    }

    /// Construct an OTAP Status RecordBatch from the builders.
    pub fn finish(&mut self) -> Result<StructArray, ArrowError> {
        let mut fields = Vec::with_capacity(2);
        let mut columns = Vec::with_capacity(2);

        // SAFETY: `expect` is safe here because `AdaptiveArrayBuilder` guarantees that for
        // non-optional arrays, `finish()` will always return an array, even if it is empty.
        let array = self.code.finish().expect("finish returns `Some(array)`");
        let length = array.len();
        fields.push(Field::new(
            consts::STATUS_CODE,
            array.data_type().clone(),
            true,
        ));
        columns.push(array);

        // SAFETY: `expect` is safe here because `AdaptiveArrayBuilder` guarantees that for
        // non-optional arrays, `finish()` will always return an array, even if it is empty.
        let array = self
            .status_message
            .finish()
            .expect("finish returns `Some(array)`");
        fields.push(Field::new(
            consts::STATUS_MESSAGE,
            array.data_type().clone(),
            true,
        ));
        columns.push(array);

        StructArray::try_new_with_length(Fields::from(fields), columns, None, length)
    }
}
