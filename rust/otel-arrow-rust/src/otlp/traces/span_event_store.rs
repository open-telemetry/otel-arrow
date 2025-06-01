// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::arrays::{
    NullableArrayAccessor, StringArrayAccessor, UInt32ArrayAccessor,
    get_timestamp_nanosecond_array_opt, get_u16_array_opt, get_u32_array_opt,
};
use crate::error;
use crate::otlp::attributes::store::Attribute32Store;
use crate::otlp::traces::delta_decoder::DeltaDecoder;
use crate::otlp::traces::span_event::SpanEvent;
use crate::proto::opentelemetry::trace::v1::span::Event;
use crate::schema::consts;
use arrow::array::{RecordBatch, TimestampNanosecondArray, UInt16Array, UInt32Array};
use std::collections::HashMap;

#[derive(Default)]
pub struct SpanEventsStore {
    events_by_id: HashMap<u16, Vec<Event>>,
}

impl SpanEventsStore {
    pub(crate) fn event_by_id(&self, id: u16) -> Option<&Vec<Event>> {
        self.events_by_id.get(&id)
    }
}

pub fn span_events_store_from_record_batch(
    rb: &RecordBatch,
    attrs_store: Option<&mut Attribute32Store>,
) -> error::Result<SpanEventsStore> {
    let mut store = SpanEventsStore::default();
    let span_event_iterator = SpanEventIDIterator::try_new(rb, attrs_store)?;
    for span_event in span_event_iterator {
        store
            .events_by_id
            .entry(span_event.parent_id)
            .or_default()
            .push(span_event.event);
    }
    Ok(store)
}

struct SpanEventIDIterator<'a> {
    id: Option<&'a UInt32Array>,
    name: Option<StringArrayAccessor<'a>>,
    parent_id: Option<&'a UInt16Array>,
    time_unix_nano: Option<&'a TimestampNanosecondArray>,
    dropped_attributes_count: Option<UInt32ArrayAccessor<'a>>,
    offset: usize,
    num_rows: usize,
    parent_id_decoder: DeltaDecoder<String, u16>,
    attr_store: Option<&'a mut Attribute32Store>,
}

impl<'a> SpanEventIDIterator<'a> {
    fn try_new(
        rb: &'a RecordBatch,
        attr_store: Option<&'a mut Attribute32Store>,
    ) -> error::Result<Self> {
        let num_rows = rb.num_rows();
        let id = get_u32_array_opt(rb, consts::ID)?;
        let name = rb
            .column_by_name(consts::NAME)
            .map(StringArrayAccessor::try_new)
            .transpose()?;
        let parent_id = get_u16_array_opt(rb, consts::PARENT_ID)?;
        let time_unix_nano = get_timestamp_nanosecond_array_opt(rb, consts::TIME_UNIX_NANO)?;
        let dropped_attributes_count = rb
            .column_by_name(consts::DROPPED_ATTRIBUTES_COUNT)
            .map(UInt32ArrayAccessor::try_new)
            .transpose()?;

        Ok(Self {
            id,
            name,
            parent_id,
            time_unix_nano,
            dropped_attributes_count,
            offset: 0,
            num_rows,
            parent_id_decoder: Default::default(),
            attr_store,
        })
    }
}

impl Iterator for SpanEventIDIterator<'_> {
    type Item = SpanEvent;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.num_rows {
            return None;
        }
        let id = self.id.value_at(self.offset);
        let name = self.name.value_at(self.offset).unwrap_or_default();
        let parent_id = self.parent_id.value_at_or_default(self.offset);
        let decoded_parent_id = self.parent_id_decoder.decode(&name, parent_id);
        let time_unix_nano = self.time_unix_nano.value_at_or_default(self.offset) as u64;
        let dropped_attributes_count = self
            .dropped_attributes_count
            .value_at_or_default(self.offset);
        let attributes = if let Some((id, attr_store)) = id.zip(self.attr_store.as_deref_mut()) {
            // Copy attributes
            attr_store
                .attribute_by_delta_id(id)
                .unwrap_or_default()
                .to_vec()
        } else {
            vec![]
        };

        self.offset += 1;
        Some(SpanEvent {
            parent_id: decoded_parent_id,
            event: Event {
                time_unix_nano,
                name,
                attributes,
                dropped_attributes_count,
            },
        })
    }
}
