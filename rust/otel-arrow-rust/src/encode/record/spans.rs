// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains builders for record batches for spans.

use std::sync::Arc;

use arrow::{
    array::{Array, RecordBatch},
    datatypes::{Field, Schema},
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
    schema::consts,
};

// These are copied from `pdata-views::views::common` because they're simple and I don't want to
// introduce a cross-cutting dependency.

/// Trace IDs are 16 binary bytes.
pub type TraceId = [u8; 16];

/// Span IDs are 8 binary bytes.
pub type SpanId = [u8; 8];

/// Record batch builder for spans
pub struct SpansRecordBatchBuilder {
    id: UInt16ArrayBuilder,

    /// the builder for the resource struct for this span record batch
    pub resource: ResourceBuilder,

    /// the builder for the scope struct for this span record batch
    pub scope: ScopeBuilder,

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
    status_code: Int32ArrayBuilder,
    status_status_message: StringArrayBuilder,
}

impl SpansRecordBatchBuilder {
    /// Create a new `SpansRecordBatchBuilder`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            id: UInt16ArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                default_values_optional: false,
            }),
            resource: ResourceBuilder::new(),
            scope: ScopeBuilder::new(),

            start_time_unix_nano: TimestampNanosecondArrayBuilder::new(ArrayOptions {
                optional: false,
                dictionary_options: None,
                ..Default::default()
            }),
            duration_time_unix_nano: DurationNanosecondArrayBuilder::new(ArrayOptions {
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
                optional: true,
                dictionary_options: Some(DictionaryOptions::dict8()),
                ..Default::default()
            }),
            parent_span_id: FixedSizeBinaryArrayBuilder::new_with_args(
                ArrayOptions {
                    optional: true,
                    dictionary_options: Some(DictionaryOptions::dict8()),
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
                optional: true,
                dictionary_options: Some(DictionaryOptions::dict8()),
                ..Default::default()
            }),
            dropped_attributes_count: UInt32ArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                ..Default::default()
            }),
            dropped_events_count: UInt32ArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                ..Default::default()
            }),
            dropped_links_count: UInt32ArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                ..Default::default()
            }),

            status_code: Int32ArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: Some(DictionaryOptions::dict8()),
                ..Default::default()
            }),
            status_status_message: StringArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: Some(DictionaryOptions::dict8()),
                ..Default::default()
            }),
        }
    }

    /// append a value to the `id` array
    pub fn append_id(&mut self, val: Option<u16>) {
        if let Some(val) = val {
            self.id.append_value(&val);
        } else {
            self.id.append_null();
        }
    }

    /// append a value to the `start_time_unix_nano` array
    pub fn append_start_time_unix_nano(&mut self, val: Option<i64>) {
        let val = val.unwrap_or(0);
        self.start_time_unix_nano.append_value(&val);
    }

    /// append a span duration
    pub fn append_duration_time_unix_nano(&mut self, val: Option<i64>) {
        let val = val.unwrap_or(0);
        self.duration_time_unix_nano.append_value(&val);
    }

    /// append a value to the `trace_id` array
    pub fn append_trace_id(&mut self, val: Option<&TraceId>) -> Result<(), ArrowError> {
        let id = val.unwrap_or(&[0; 16]);
        self.trace_id.append_slice(id)
    }

    /// append a value to the `span_id` array
    pub fn append_span_id(&mut self, val: Option<&SpanId>) -> Result<(), ArrowError> {
        // FIXME: dealing with Option for `span_id`, `trace_id`, and `name` is not great; the OTLP
        // representation allows empty values ([0; 8], [0; 16], and "" respectively) but the OTAP
        // representation doesn't. Since we can't append_null, we write the empty "invalid" value
        // instead, but perhaps we should raise an error?
        let id = val.unwrap_or(&[0; 8]);
        self.span_id.append_slice(id)
    }

    /// append trace_state
    pub fn append_trace_state(&mut self, val: Option<&str>) {
        if let Some(val) = val {
            self.trace_state.append_str(val);
        } else {
            self.trace_state.append_null();
        }
    }

    /// append a value to the `parent_span_id` array
    pub fn append_parent_span_id(&mut self, val: Option<&SpanId>) -> Result<(), ArrowError> {
        if let Some(val) = val {
            self.parent_span_id.append_slice(val)
        } else {
            self.parent_span_id.append_null();
            Ok(())
        }
    }

    /// append a span name
    pub fn append_name(&mut self, val: Option<&str>) {
        let name = val.unwrap_or("");
        self.name.append_str(name)
    }

    /// append a value to the `severity_number` array
    pub fn append_kind(&mut self, val: Option<i32>) {
        if let Some(val) = val {
            self.kind.append_value(&val);
        } else {
            self.kind.append_null();
        }
    }

    /// append a value to the `dropped_attributes_count` array
    pub fn append_dropped_attributes_count(&mut self, val: u32) {
        self.dropped_attributes_count.append_value(&val);
    }

    /// append a value to the `dropped_events_count` array
    pub fn append_dropped_events_count(&mut self, val: u32) {
        self.dropped_events_count.append_value(&val);
    }

    /// append a value to the `dropped_links_count` array
    pub fn append_dropped_links_count(&mut self, val: u32) {
        self.dropped_links_count.append_value(&val);
    }

    /// append a value to the `status_code` array
    pub fn append_status_code(&mut self, val: Option<i32>) {
        if let Some(val) = val {
            self.status_code.append_value(&val);
        } else {
            self.status_code.append_null();
        }
    }

    /// append a value to the `status_status_message` array
    pub fn append_status_status_message(&mut self, val: Option<&str>) {
        if let Some(val) = val {
            self.status_status_message.append_str(val);
        } else {
            self.status_status_message.append_null();
        }
    }

    /// Construct an OTAP Spans RecordBatch from the builders
    pub fn finish(&mut self) -> Result<RecordBatch, ArrowError> {
        // FIXME: we should consume self here!
        let mut fields = vec![];
        let mut columns = vec![];

        if let Some(array) = self.id.finish() {
            fields.push(Field::new(consts::ID, array.data_type().clone(), true));
            columns.push(array);
        }

        let resources = self.resource.finish()?;
        fields.push(Field::new(
            consts::RESOURCE,
            resources.data_type().clone(),
            true,
        ));
        columns.push(Arc::new(resources));

        let scopes = self.scope.finish()?;
        fields.push(Field::new(consts::SCOPE, scopes.data_type().clone(), true));
        columns.push(Arc::new(scopes));

        if let Some(array) = self.start_time_unix_nano.finish() {
            fields.push(Field::new(
                consts::START_TIME_UNIX_NANO,
                array.data_type().clone(),
                false,
            ));
            columns.push(array);
        }
        if let Some(array) = self.duration_time_unix_nano.finish() {
            fields.push(Field::new(
                consts::DURATION_TIME_UNIX_NANO,
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

        if let Some(array) = self.span_id.finish() {
            fields.push(Field::new(
                consts::SPAN_ID,
                array.data_type().clone(),
                false,
            ));
            columns.push(array);
        }

        if let Some(array) = self.trace_state.finish() {
            fields.push(Field::new(
                consts::TRACE_STATE,
                array.data_type().clone(),
                true,
            ));
            columns.push(array);
        }

        if let Some(array) = self.parent_span_id.finish() {
            fields.push(Field::new(
                consts::PARENT_SPAN_ID,
                array.data_type().clone(),
                true,
            ));
            columns.push(array);
        }
        if let Some(array) = self.name.finish() {
            fields.push(Field::new(consts::NAME, array.data_type().clone(), false));
            columns.push(array);
        }

        if let Some(array) = self.kind.finish() {
            fields.push(Field::new(consts::KIND, array.data_type().clone(), true));
            columns.push(array);
        }

        if let Some(array) = self.dropped_attributes_count.finish() {
            fields.push(Field::new(
                consts::DROPPED_ATTRIBUTES_COUNT,
                array.data_type().clone(),
                true,
            ));
            columns.push(array);
        }

        if let Some(array) = self.dropped_events_count.finish() {
            fields.push(Field::new(
                consts::DROPPED_EVENTS_COUNT,
                array.data_type().clone(),
                true,
            ));
            columns.push(array);
        }

        if let Some(array) = self.dropped_links_count.finish() {
            fields.push(Field::new(
                consts::DROPPED_LINKS_COUNT,
                array.data_type().clone(),
                true,
            ));
            columns.push(array);
        }

        if let Some(array) = self.status_code.finish() {
            fields.push(Field::new(
                consts::STATUS_CODE,
                array.data_type().clone(),
                true,
            ));
            columns.push(array);
        }
        if let Some(array) = self.status_status_message.finish() {
            fields.push(Field::new(
                consts::STATUS_MESSAGE,
                array.data_type().clone(),
                true,
            ));
            columns.push(array);
        }

        RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
    }
}

impl Default for SpansRecordBatchBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for the Events table
pub struct EventsBuilder {
    id: UInt32ArrayBuilder,
    parent_id: UInt16ArrayBuilder,
    time_unix_nano: TimestampNanosecondArrayBuilder,
    name: StringArrayBuilder,
    dropped_attributes_count: UInt32ArrayBuilder,
}

impl EventsBuilder {
    /// Create a new `EventsBuilder`.
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
            time_unix_nano: TimestampNanosecondArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                ..Default::default()
            }),
            name: StringArrayBuilder::new(ArrayOptions {
                optional: false,
                dictionary_options: Some(DictionaryOptions::dict8()),
                ..Default::default()
            }),
            dropped_attributes_count: UInt32ArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                ..Default::default()
            }),
        }
    }

    /// append a value to the `id` array
    pub fn append_id(&mut self, val: Option<u32>) {
        if let Some(val) = val {
            self.id.append_value(&val);
        } else {
            self.id.append_null();
        }
    }

    /// append a value to the `parent_id` array
    pub fn append_parent_id(&mut self, val: u16) {
        self.parent_id.append_value(&val);
    }

    /// append a value to the `time_unix_nano` array
    pub fn append_time_unix_nano(&mut self, val: Option<i64>) {
        if let Some(val) = val {
            self.time_unix_nano.append_value(&val);
        } else {
            self.time_unix_nano.append_null();
        }
    }

    /// append a value to the `name` array
    pub fn append_name(&mut self, val: Option<&str>) {
        // names are required in the event spec
        let name = val.unwrap_or("");
        self.name.append_str(name);
    }

    /// append a value to the `dropped_attributes_count` array
    pub fn append_dropped_attributes_count(&mut self, val: u32) {
        self.dropped_attributes_count.append_value(&val);
    }

    /// Construct an OTAP Events record batch from the array builders
    pub fn finish(&mut self) -> Result<RecordBatch, ArrowError> {
        let mut fields = vec![];
        let mut columns = vec![];

        if let Some(array) = self.id.finish() {
            fields.push(Field::new(consts::ID, array.data_type().clone(), true));
            columns.push(array);
        }

        if let Some(array) = self.parent_id.finish() {
            fields.push(Field::new(
                consts::PARENT_ID,
                array.data_type().clone(),
                false,
            ));
            columns.push(array);
        }

        if let Some(array) = self.time_unix_nano.finish() {
            fields.push(Field::new(
                consts::TIME_UNIX_NANO,
                array.data_type().clone(),
                true,
            ));
            columns.push(array);
        }

        if let Some(array) = self.name.finish() {
            fields.push(Field::new(consts::NAME, array.data_type().clone(), false));
            columns.push(array);
        }

        if let Some(array) = self.dropped_attributes_count.finish() {
            fields.push(Field::new(
                consts::DROPPED_ATTRIBUTES_COUNT,
                array.data_type().clone(),
                true,
            ));
            columns.push(array);
        }

        RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
    }
}

impl Default for EventsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for the Links table
pub struct LinksBuilder {
    id: UInt32ArrayBuilder,
    parent_id: UInt16ArrayBuilder,
    trace_id: FixedSizeBinaryArrayBuilder,
    span_id: FixedSizeBinaryArrayBuilder,
    trace_state: StringArrayBuilder,
    dropped_attributes_count: UInt32ArrayBuilder,
}

impl LinksBuilder {
    /// Create a new instance of the `LinksBuilder`
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
            trace_id: FixedSizeBinaryArrayBuilder::new_with_args(
                ArrayOptions {
                    optional: true,
                    dictionary_options: Some(DictionaryOptions::dict8()),
                    ..Default::default()
                },
                16,
            ),
            span_id: FixedSizeBinaryArrayBuilder::new_with_args(
                ArrayOptions {
                    optional: true,
                    dictionary_options: Some(DictionaryOptions::dict8()),
                    ..Default::default()
                },
                8,
            ),
            trace_state: StringArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: Some(DictionaryOptions::dict8()),
                ..Default::default()
            }),
            dropped_attributes_count: UInt32ArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                ..Default::default()
            }),
        }
    }

    /// append a value to the `id` array
    pub fn append_id(&mut self, val: Option<u32>) {
        if let Some(val) = val {
            self.id.append_value(&val);
        } else {
            self.id.append_null();
        }
    }

    /// append a value to the `parent_id` array
    pub fn append_parent_id(&mut self, val: u16) {
        self.parent_id.append_value(&val);
    }

    /// append a value to the `trace_id` array
    pub fn append_trace_id(&mut self, val: Option<&TraceId>) -> Result<(), ArrowError> {
        if let Some(val) = val {
            self.trace_id.append_slice(val)
        } else {
            self.trace_id.append_null();
            Ok(())
        }
    }

    /// append a value to the `span_id` array
    pub fn append_span_id(&mut self, val: Option<&SpanId>) -> Result<(), ArrowError> {
        if let Some(val) = val {
            self.span_id.append_slice(val)
        } else {
            self.span_id.append_null();
            Ok(())
        }
    }

    /// append trace_state
    pub fn append_trace_state(&mut self, val: Option<&str>) {
        if let Some(val) = val {
            self.trace_state.append_str(val);
        } else {
            self.trace_state.append_null();
        }
    }

    /// append a value to the `dropped_attributes_count` array
    pub fn append_dropped_attributes_count(&mut self, val: u32) {
        self.dropped_attributes_count.append_value(&val);
    }

    /// Construct an OTAP Events record batch from the array builders
    pub fn finish(&mut self) -> Result<RecordBatch, ArrowError> {
        let mut fields = vec![];
        let mut columns = vec![];

        if let Some(array) = self.id.finish() {
            fields.push(Field::new(consts::ID, array.data_type().clone(), true));
            columns.push(array);
        }

        if let Some(array) = self.parent_id.finish() {
            fields.push(Field::new(
                consts::PARENT_ID,
                array.data_type().clone(),
                false,
            ));
            columns.push(array);
        }

        if let Some(array) = self.trace_id.finish() {
            fields.push(Field::new(
                consts::TRACE_ID,
                array.data_type().clone(),
                true,
            ));
            columns.push(array);
        }

        if let Some(array) = self.span_id.finish() {
            fields.push(Field::new(consts::SPAN_ID, array.data_type().clone(), true));
            columns.push(array);
        }

        if let Some(array) = self.trace_state.finish() {
            fields.push(Field::new(
                consts::TRACE_STATE,
                array.data_type().clone(),
                true,
            ));
            columns.push(array);
        }

        if let Some(array) = self.dropped_attributes_count.finish() {
            fields.push(Field::new(
                consts::DROPPED_ATTRIBUTES_COUNT,
                array.data_type().clone(),
                true,
            ));
            columns.push(array);
        }

        RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
    }
}

impl Default for LinksBuilder {
    fn default() -> Self {
        Self::new()
    }
}
