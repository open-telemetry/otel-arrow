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

/// Creates a `SpanEventsStore` from an Arrow RecordBatch containing span event data.
///
/// This function processes a RecordBatch containing span event information and organizes
/// the events into a `SpanEventsStore` where events are grouped by their parent span ID.
///
/// # Arguments
/// * `rb` - The Arrow RecordBatch containing span event data. Expected columns:
///   - `id`: Unique identifier for each event (optional)
///   - `name`: Event name (optional)
///   - `parent_id`: ID of the parent span (optional)
///   - `time_unix_nano`: Timestamp in nanoseconds (optional)
///   - `dropped_attributes_count`: Count of dropped attributes (optional)
/// * `attrs_store` - Optional attribute store for looking up event attributes by ID
///
/// # Returns
/// Returns a `SpanEventsStore` containing all events grouped by parent span ID,
/// or an error if the RecordBatch cannot be processed.
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

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{StringArray, TimestampNanosecondArray, UInt16Array, UInt32Array};
    use arrow::datatypes::{DataType, Field, Schema, TimeUnit};
    use std::sync::Arc;

    fn create_test_record_batch() -> RecordBatch {
        let schema = Arc::new(Schema::new(vec![
            Field::new(consts::ID, DataType::UInt32, true),
            Field::new(consts::NAME, DataType::Utf8, true),
            Field::new(consts::PARENT_ID, DataType::UInt16, true),
            Field::new(
                consts::TIME_UNIX_NANO,
                DataType::Timestamp(TimeUnit::Nanosecond, None),
                true,
            ),
            Field::new(consts::DROPPED_ATTRIBUTES_COUNT, DataType::UInt32, true),
        ]));

        let id_array = Arc::new(UInt32Array::from(vec![Some(1), Some(2), Some(3)]));
        let name_array = Arc::new(StringArray::from(vec![
            Some("event1"),
            Some("event2"),
            Some("event3"),
        ]));
        let parent_id_array = Arc::new(UInt16Array::from(vec![Some(10), Some(10), Some(20)]));
        let time_array = Arc::new(TimestampNanosecondArray::from(vec![
            Some(1000000000),
            Some(2000000000),
            Some(3000000000),
        ]));
        let dropped_count_array = Arc::new(UInt32Array::from(vec![Some(0), Some(1), Some(2)]));

        RecordBatch::try_new(
            schema,
            vec![
                id_array,
                name_array,
                parent_id_array,
                time_array,
                dropped_count_array,
            ],
        )
        .unwrap()
    }

    fn create_empty_record_batch() -> RecordBatch {
        let schema = Arc::new(Schema::new(vec![
            Field::new(consts::ID, DataType::UInt32, true),
            Field::new(consts::NAME, DataType::Utf8, true),
            Field::new(consts::PARENT_ID, DataType::UInt16, true),
            Field::new(
                consts::TIME_UNIX_NANO,
                DataType::Timestamp(TimeUnit::Nanosecond, None),
                true,
            ),
            Field::new(consts::DROPPED_ATTRIBUTES_COUNT, DataType::UInt32, true),
        ]));

        let id_array = Arc::new(UInt32Array::from(Vec::<Option<u32>>::new()));
        let name_array = Arc::new(StringArray::from(Vec::<Option<String>>::new()));
        let parent_id_array = Arc::new(UInt16Array::from(Vec::<Option<u16>>::new()));
        let time_array = Arc::new(TimestampNanosecondArray::from(Vec::<Option<i64>>::new()));
        let dropped_count_array = Arc::new(UInt32Array::from(Vec::<Option<u32>>::new()));

        RecordBatch::try_new(
            schema,
            vec![
                id_array,
                name_array,
                parent_id_array,
                time_array,
                dropped_count_array,
            ],
        )
        .unwrap()
    }

    #[test]
    fn test_span_events_store_default() {
        let store = SpanEventsStore::default();
        assert!(store.event_by_id(1).is_none());
        assert!(store.event_by_id(0).is_none());
    }

    #[test]
    fn test_span_events_store_event_by_id() {
        let mut store = SpanEventsStore::default();
        let event = Event {
            time_unix_nano: 1000000000,
            name: "test_event".to_string(),
            attributes: vec![],
            dropped_attributes_count: 0,
        };

        let _ = store.events_by_id.insert(42, vec![event.clone()]);

        let retrieved_events = store.event_by_id(42);
        assert!(retrieved_events.is_some());
        assert_eq!(retrieved_events.unwrap().len(), 1);
        assert_eq!(retrieved_events.unwrap()[0].name, "test_event");
        assert_eq!(retrieved_events.unwrap()[0].time_unix_nano, 1000000000);

        assert!(store.event_by_id(99).is_none());
    }

    #[test]
    fn test_span_events_store_from_record_batch_basic() {
        let rb = create_test_record_batch();
        let store = span_events_store_from_record_batch(&rb, None).unwrap();

        // Check that events are grouped by parent_id
        let events_for_parent_10 = store.event_by_id(10);
        assert!(events_for_parent_10.is_some());
        assert_eq!(events_for_parent_10.unwrap().len(), 2);

        let events_for_parent_20 = store.event_by_id(20);
        assert!(events_for_parent_20.is_some());
        assert_eq!(events_for_parent_20.unwrap().len(), 1);

        // Verify event details for parent_id 10
        let events = events_for_parent_10.unwrap();
        assert_eq!(events[0].name, "event1");
        assert_eq!(events[0].time_unix_nano, 1000000000);
        assert_eq!(events[0].dropped_attributes_count, 0);

        assert_eq!(events[1].name, "event2");
        assert_eq!(events[1].time_unix_nano, 2000000000);
        assert_eq!(events[1].dropped_attributes_count, 1);

        // Verify event details for parent_id 20
        let events = events_for_parent_20.unwrap();
        assert_eq!(events[0].name, "event3");
        assert_eq!(events[0].time_unix_nano, 3000000000);
        assert_eq!(events[0].dropped_attributes_count, 2);
    }

    #[test]
    fn test_span_events_store_from_empty_record_batch() {
        let rb = create_empty_record_batch();
        let store = span_events_store_from_record_batch(&rb, None).unwrap();

        assert!(store.event_by_id(1).is_none());
        assert!(store.event_by_id(0).is_none());
    }

    #[test]
    fn test_span_event_iterator_creation() {
        let rb = create_test_record_batch();
        let iterator = SpanEventIDIterator::try_new(&rb, None).unwrap();

        assert_eq!(iterator.num_rows, 3);
        assert_eq!(iterator.offset, 0);
        assert!(iterator.id.is_some());
        assert!(iterator.name.is_some());
        assert!(iterator.parent_id.is_some());
        assert!(iterator.time_unix_nano.is_some());
        assert!(iterator.dropped_attributes_count.is_some());
    }

    #[test]
    fn test_span_event_iterator_iteration() {
        let rb = create_test_record_batch();
        let mut iterator = SpanEventIDIterator::try_new(&rb, None).unwrap();

        // First event
        let first_event = iterator.next();
        assert!(first_event.is_some());
        let span_event = first_event.unwrap();
        assert_eq!(span_event.event.name, "event1");
        assert_eq!(span_event.event.time_unix_nano, 1000000000);
        assert_eq!(span_event.event.dropped_attributes_count, 0);

        // Second event
        let second_event = iterator.next();
        assert!(second_event.is_some());
        let span_event = second_event.unwrap();
        assert_eq!(span_event.event.name, "event2");
        assert_eq!(span_event.event.time_unix_nano, 2000000000);
        assert_eq!(span_event.event.dropped_attributes_count, 1);

        // Third event
        let third_event = iterator.next();
        assert!(third_event.is_some());
        let span_event = third_event.unwrap();
        assert_eq!(span_event.event.name, "event3");
        assert_eq!(span_event.event.time_unix_nano, 3000000000);
        assert_eq!(span_event.event.dropped_attributes_count, 2);

        // No more events
        let fourth_event = iterator.next();
        assert!(fourth_event.is_none());
    }

    #[test]
    fn test_span_event_iterator_empty_batch() {
        let rb = create_empty_record_batch();
        let mut iterator = SpanEventIDIterator::try_new(&rb, None).unwrap();

        assert_eq!(iterator.num_rows, 0);
        assert!(iterator.next().is_none());
    }

    #[test]
    fn test_span_events_store_multiple_events_same_parent() {
        let schema = Arc::new(Schema::new(vec![
            Field::new(consts::ID, DataType::UInt32, true),
            Field::new(consts::NAME, DataType::Utf8, true),
            Field::new(consts::PARENT_ID, DataType::UInt16, true),
            Field::new(
                consts::TIME_UNIX_NANO,
                DataType::Timestamp(TimeUnit::Nanosecond, None),
                true,
            ),
            Field::new(consts::DROPPED_ATTRIBUTES_COUNT, DataType::UInt32, true),
        ]));

        // Create batch with multiple events for the same parent
        let id_array = Arc::new(UInt32Array::from(vec![Some(1), Some(2), Some(3), Some(4)]));
        let name_array = Arc::new(StringArray::from(vec![
            Some("event1"),
            Some("event2"),
            Some("event3"),
            Some("event4"),
        ]));
        let parent_id_array = Arc::new(UInt16Array::from(vec![
            Some(100),
            Some(100),
            Some(100),
            Some(100),
        ]));
        let time_array = Arc::new(TimestampNanosecondArray::from(vec![
            Some(1000000000),
            Some(2000000000),
            Some(3000000000),
            Some(4000000000),
        ]));
        let dropped_count_array =
            Arc::new(UInt32Array::from(vec![Some(0), Some(0), Some(0), Some(0)]));

        let rb = RecordBatch::try_new(
            schema,
            vec![
                id_array,
                name_array,
                parent_id_array,
                time_array,
                dropped_count_array,
            ],
        )
        .unwrap();

        let store = span_events_store_from_record_batch(&rb, None).unwrap();

        let events = store.event_by_id(100);
        assert!(events.is_some());
        assert_eq!(events.unwrap().len(), 4);

        let events = events.unwrap();
        assert_eq!(events[0].name, "event1");
        assert_eq!(events[1].name, "event2");
        assert_eq!(events[2].name, "event3");
        assert_eq!(events[3].name, "event4");
    }

    #[test]
    fn test_span_events_store_with_null_values() {
        let schema = Arc::new(Schema::new(vec![
            Field::new(consts::ID, DataType::UInt32, true),
            Field::new(consts::NAME, DataType::Utf8, true),
            Field::new(consts::PARENT_ID, DataType::UInt16, true),
            Field::new(
                consts::TIME_UNIX_NANO,
                DataType::Timestamp(TimeUnit::Nanosecond, None),
                true,
            ),
            Field::new(consts::DROPPED_ATTRIBUTES_COUNT, DataType::UInt32, true),
        ]));

        // Create batch with some null values
        let id_array = Arc::new(UInt32Array::from(vec![None, Some(2)]));
        let name_array = Arc::new(StringArray::from(vec![None, Some("event2")]));
        let parent_id_array = Arc::new(UInt16Array::from(vec![Some(10), None]));
        let time_array = Arc::new(TimestampNanosecondArray::from(vec![None, Some(2000000000)]));
        let dropped_count_array = Arc::new(UInt32Array::from(vec![None, Some(1)]));

        let rb = RecordBatch::try_new(
            schema,
            vec![
                id_array,
                name_array,
                parent_id_array,
                time_array,
                dropped_count_array,
            ],
        )
        .unwrap();

        let store = span_events_store_from_record_batch(&rb, None).unwrap();

        // Should handle null values gracefully
        assert!(!store.events_by_id.is_empty());
    }
}
