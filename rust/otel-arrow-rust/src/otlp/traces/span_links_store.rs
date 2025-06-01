// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::arrays::{
    FixedSizeBinaryArrayAccessor, NullableArrayAccessor, StringArrayAccessor, UInt32ArrayAccessor,
    get_u16_array_opt, get_u32_array_opt,
};
use crate::error;
use crate::otlp::attributes::store::Attribute32Store;
use crate::otlp::traces::delta_decoder::DeltaDecoder;
use crate::otlp::traces::span_link::SpanLink;
use crate::proto::opentelemetry::trace::v1::span::Link;
use crate::schema::consts;
use arrow::array::{RecordBatch, UInt16Array, UInt32Array};
use std::collections::HashMap;

#[derive(Default)]
pub struct SpanLinksStore {
    links_by_id: HashMap<u16, Vec<Link>>,
}

impl SpanLinksStore {
    pub(crate) fn link_by_id(&self, id: u16) -> Option<&Vec<Link>> {
        self.links_by_id.get(&id)
    }
}

/// Creates a `SpanLinksStore` from an Arrow RecordBatch containing span link data.
///
/// This function processes a RecordBatch where each row represents a span link, extracting:
/// - Parent span ID (decoded using delta encoding)
/// - Linked span's trace ID and span ID  
/// - Trace state
/// - Dropped attributes count
/// - Any associated attributes (if an Attribute32Store is provided)
///
/// The links are grouped by their parent span ID in the resulting store for efficient lookup.
///
/// # Parameters
/// - `rb`: The RecordBatch containing span link data with expected schema:
///   - ID (UInt32): Optional attribute ID
///   - TRACE_ID (FixedSizeBinary[16]): Trace ID of linked span
///   - PARENT_ID (UInt16): Delta-encoded parent span ID  
///   - SPAN_ID (FixedSizeBinary[8]): Span ID of linked span
///   - TRACE_STATE (String): Trace state string
///   - DROPPED_ATTRIBUTES_COUNT (UInt32): Count of dropped attributes
/// - `attr_store`: Optional mutable reference to an Attribute32Store for looking up attributes
///
/// # Returns
/// - `Result<SpanLinksStore>`: Contains all span links grouped by parent span ID
///
/// # Errors
/// Returns an error if:
/// - Required columns are missing or have wrong data types
/// - Delta decoding fails
pub fn span_links_store_from_record_batch(
    rb: &RecordBatch,
    attr_store: Option<&mut Attribute32Store>,
) -> error::Result<SpanLinksStore> {
    let mut store = SpanLinksStore::default();
    let iterator = SpanLinkIterator::try_new(rb, attr_store)?;

    for batch in iterator {
        let link = batch?;
        store
            .links_by_id
            .entry(link.parent_id)
            .or_default()
            .push(link.link);
    }
    Ok(store)
}

struct SpanLinkIterator<'a> {
    id: Option<&'a UInt32Array>,
    trace_id: Option<FixedSizeBinaryArrayAccessor<'a>>,
    parent_id: Option<&'a UInt16Array>,
    span_id: Option<FixedSizeBinaryArrayAccessor<'a>>,
    trace_state: Option<StringArrayAccessor<'a>>,
    dropped_attributes_count: Option<UInt32ArrayAccessor<'a>>,
    parent_id_decoder: DeltaDecoder<Vec<u8>, u16>,
    attr_store: Option<&'a mut Attribute32Store>,
    offset: usize,
    num_rows: usize,
}

impl Iterator for SpanLinkIterator<'_> {
    type Item = error::Result<SpanLink>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.num_rows {
            return None;
        }
        let id = self.id.value_at(self.offset);
        let parent_id = self.parent_id.value_at_or_default(self.offset);

        // todo(v0y4g3r): maybe impl value_ref_at to avoid heap alloc.
        let trace_id = self.trace_id.value_at(self.offset).unwrap_or_default();

        let decoded_parent_id = self.parent_id_decoder.decode(&trace_id, parent_id);

        let span_id = self.span_id.value_at(self.offset).unwrap_or_default();

        let trace_state = self.trace_state.value_at_or_default(self.offset);
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
        Some(Ok(SpanLink {
            parent_id: decoded_parent_id,
            link: Link {
                trace_id,
                span_id,
                trace_state,
                attributes,
                dropped_attributes_count,
                flags: 0,
            },
        }))
    }
}

impl<'a> SpanLinkIterator<'a> {
    fn try_new(
        rb: &'a RecordBatch,
        attr_store: Option<&'a mut Attribute32Store>,
    ) -> error::Result<Self> {
        let num_rows = rb.num_rows();
        let id = get_u32_array_opt(rb, consts::ID)?;
        let parent_id = get_u16_array_opt(rb, consts::PARENT_ID)?;
        let trace_id = rb
            .column_by_name(consts::TRACE_ID)
            .map(|a| FixedSizeBinaryArrayAccessor::try_new(a, 16))
            .transpose()?;

        let span_id = rb
            .column_by_name(consts::SPAN_ID)
            .map(|a| FixedSizeBinaryArrayAccessor::try_new(a, 8))
            .transpose()?;
        let trace_state = rb
            .column_by_name(consts::TRACE_STATE)
            .map(StringArrayAccessor::try_new)
            .transpose()?;
        let dropped_attributes_count = rb
            .column_by_name(consts::DROPPED_ATTRIBUTES_COUNT)
            .map(UInt32ArrayAccessor::try_new)
            .transpose()?;

        Ok(Self {
            id,
            trace_id,
            parent_id,
            span_id,
            trace_state,
            dropped_attributes_count,
            parent_id_decoder: DeltaDecoder::new(),
            attr_store,
            offset: 0,
            num_rows,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{FixedSizeBinaryArray, StringArray, UInt16Array, UInt32Array};
    use arrow::datatypes::{DataType, Field, Schema};
    use std::sync::Arc;

    fn create_test_record_batch() -> RecordBatch {
        let schema = Arc::new(Schema::new(vec![
            Field::new(consts::ID, DataType::UInt32, true),
            Field::new(consts::TRACE_ID, DataType::FixedSizeBinary(16), true),
            Field::new(consts::PARENT_ID, DataType::UInt16, true),
            Field::new(consts::SPAN_ID, DataType::FixedSizeBinary(8), true),
            Field::new(consts::TRACE_STATE, DataType::Utf8, true),
            Field::new(consts::DROPPED_ATTRIBUTES_COUNT, DataType::UInt32, true),
        ]));

        // Create test data
        let id_array = Arc::new(UInt32Array::from(vec![Some(1), Some(2), Some(3)]));

        // Create 16-byte trace IDs
        let trace_id_data = vec![vec![1u8; 16], vec![2u8; 16], vec![3u8; 16]];
        let trace_id_array =
            Arc::new(FixedSizeBinaryArray::try_from_iter(trace_id_data.into_iter()).unwrap());

        let parent_id_array = Arc::new(UInt16Array::from(vec![Some(10), Some(10), Some(20)]));

        // Create 8-byte span IDs
        let span_id_data = vec![vec![11u8; 8], vec![12u8; 8], vec![13u8; 8]];
        let span_id_array =
            Arc::new(FixedSizeBinaryArray::try_from_iter(span_id_data.into_iter()).unwrap());

        let trace_state_array = Arc::new(StringArray::from(vec!["state1", "state2", "state3"]));

        let dropped_count_array = Arc::new(UInt32Array::from(vec![Some(0), Some(1), Some(2)]));

        RecordBatch::try_new(schema, vec![
            id_array,
            trace_id_array,
            parent_id_array,
            span_id_array,
            trace_state_array,
            dropped_count_array,
        ])
        .unwrap()
    }

    fn create_empty_record_batch() -> RecordBatch {
        let schema = Arc::new(Schema::new(vec![
            Field::new(consts::ID, DataType::UInt32, true),
            Field::new(consts::TRACE_ID, DataType::FixedSizeBinary(16), true),
            Field::new(consts::PARENT_ID, DataType::UInt16, true),
            Field::new(consts::SPAN_ID, DataType::FixedSizeBinary(8), true),
            Field::new(consts::TRACE_STATE, DataType::Utf8, true),
            Field::new(consts::DROPPED_ATTRIBUTES_COUNT, DataType::UInt32, true),
        ]));

        let id_array = Arc::new(UInt32Array::from(Vec::<Option<u32>>::new()));
        let trace_id_array = Arc::new(
            FixedSizeBinaryArray::try_from_sparse_iter_with_size(
                Vec::<Option<Vec<u8>>>::new().into_iter(),
                16i32,
            )
            .unwrap(),
        );
        let parent_id_array = Arc::new(UInt16Array::from(Vec::<Option<u16>>::new()));
        let span_id_array = Arc::new(
            FixedSizeBinaryArray::try_from_sparse_iter_with_size(
                Vec::<Option<Vec<u8>>>::new().into_iter(),
                8i32,
            )
            .unwrap(),
        );
        let trace_state_array = Arc::new(StringArray::from(Vec::<Option<String>>::new()));
        let dropped_count_array = Arc::new(UInt32Array::from(Vec::<Option<u32>>::new()));

        RecordBatch::try_new(schema, vec![
            id_array,
            trace_id_array,
            parent_id_array,
            span_id_array,
            trace_state_array,
            dropped_count_array,
        ])
        .unwrap()
    }

    fn create_minimal_record_batch() -> RecordBatch {
        let schema = Arc::new(Schema::new(vec![
            Field::new(consts::ID, DataType::UInt32, true),
            Field::new(consts::TRACE_ID, DataType::FixedSizeBinary(16), true),
            Field::new(consts::PARENT_ID, DataType::UInt16, true),
            Field::new(consts::SPAN_ID, DataType::FixedSizeBinary(8), true),
            Field::new(consts::TRACE_STATE, DataType::Utf8, true),
            Field::new(consts::DROPPED_ATTRIBUTES_COUNT, DataType::UInt32, true),
        ]));

        // Single row with minimal data
        let id_array = Arc::new(UInt32Array::from(vec![Some(1)]));
        let trace_id_array =
            Arc::new(FixedSizeBinaryArray::try_from_iter(vec![vec![1u8; 16]].into_iter()).unwrap());
        let parent_id_array = Arc::new(UInt16Array::from(vec![Some(100)]));
        let span_id_array =
            Arc::new(FixedSizeBinaryArray::try_from_iter(vec![vec![2u8; 8]].into_iter()).unwrap());
        let trace_state_array = Arc::new(StringArray::from(vec![Some("minimal")]));
        let dropped_count_array = Arc::new(UInt32Array::from(vec![Some(0)]));

        RecordBatch::try_new(schema, vec![
            id_array,
            trace_id_array,
            parent_id_array,
            span_id_array,
            trace_state_array,
            dropped_count_array,
        ])
        .unwrap()
    }

    #[test]
    fn test_span_links_store_default() {
        let store = SpanLinksStore::default();
        assert!(store.link_by_id(1).is_none());
        assert!(store.link_by_id(0).is_none());
    }

    #[test]
    fn test_span_links_store_link_by_id() {
        let mut store = SpanLinksStore::default();
        let link = Link {
            trace_id: vec![1u8; 16],
            span_id: vec![2u8; 8],
            trace_state: "test_state".to_string(),
            attributes: vec![],
            dropped_attributes_count: 0,
            flags: 0,
        };

        let _ = store.links_by_id.insert(42, vec![link.clone()]);

        let retrieved_links = store.link_by_id(42);
        assert!(retrieved_links.is_some());
        assert_eq!(retrieved_links.unwrap().len(), 1);
        assert_eq!(retrieved_links.unwrap()[0].trace_id, vec![1u8; 16]);
        assert_eq!(retrieved_links.unwrap()[0].span_id, vec![2u8; 8]);
        assert_eq!(retrieved_links.unwrap()[0].trace_state, "test_state");

        assert!(store.link_by_id(99).is_none());
    }

    #[test]
    fn test_span_links_store_from_record_batch_basic() {
        let rb = create_test_record_batch();
        let store = span_links_store_from_record_batch(&rb, None).unwrap();

        // Check that links are grouped by parent_id
        let links_for_parent_10 = store.link_by_id(10);
        assert!(links_for_parent_10.is_some());
        assert_eq!(links_for_parent_10.unwrap().len(), 2);

        let links_for_parent_20 = store.link_by_id(20);
        assert!(links_for_parent_20.is_some());
        assert_eq!(links_for_parent_20.unwrap().len(), 1);

        // Verify link details for parent_id 10
        let links = links_for_parent_10.unwrap();
        assert_eq!(links[0].trace_id, vec![1u8; 16]);
        assert_eq!(links[0].span_id, vec![11u8; 8]);
        assert_eq!(links[0].trace_state, "state1");
        assert_eq!(links[0].dropped_attributes_count, 0);
        assert_eq!(links[0].flags, 0);

        assert_eq!(links[1].trace_id, vec![2u8; 16]);
        assert_eq!(links[1].span_id, vec![12u8; 8]);
        assert_eq!(links[1].trace_state, "state2");
        assert_eq!(links[1].dropped_attributes_count, 1);

        // Verify link details for parent_id 20
        let links = links_for_parent_20.unwrap();
        assert_eq!(links[0].trace_id, vec![3u8; 16]);
        assert_eq!(links[0].span_id, vec![13u8; 8]);
        assert_eq!(links[0].trace_state, "state3");
        assert_eq!(links[0].dropped_attributes_count, 2);
    }

    #[test]
    fn test_span_links_store_from_empty_record_batch() {
        let rb = create_empty_record_batch();
        let store = span_links_store_from_record_batch(&rb, None).unwrap();

        assert!(store.link_by_id(1).is_none());
        assert!(store.link_by_id(0).is_none());
        assert!(store.links_by_id.is_empty());
    }

    #[test]
    fn test_span_links_store_from_minimal_record_batch() {
        let rb = create_minimal_record_batch();
        let store = span_links_store_from_record_batch(&rb, None).unwrap();

        let links = store.link_by_id(100);
        assert!(links.is_some());
        assert_eq!(links.unwrap().len(), 1);

        let link = &links.unwrap()[0];
        assert_eq!(link.trace_id, vec![1u8; 16]);
        assert_eq!(link.span_id, vec![2u8; 8]);
        assert_eq!(link.trace_state, "minimal");
        assert_eq!(link.dropped_attributes_count, 0);
        assert_eq!(link.flags, 0);
        assert!(link.attributes.is_empty());
    }

    #[test]
    fn test_span_link_iterator_creation() {
        let rb = create_test_record_batch();
        let iterator = SpanLinkIterator::try_new(&rb, None).unwrap();

        assert_eq!(iterator.num_rows, 3);
        assert_eq!(iterator.offset, 0);
        assert!(iterator.id.is_some());
        assert!(iterator.trace_id.is_some());
        assert!(iterator.parent_id.is_some());
        assert!(iterator.span_id.is_some());
        assert!(iterator.trace_state.is_some());
        assert!(iterator.dropped_attributes_count.is_some());
    }

    #[test]
    fn test_span_link_iterator_iteration() {
        let rb = create_test_record_batch();
        let mut iterator = SpanLinkIterator::try_new(&rb, None).unwrap();

        // First link
        let first_link = iterator.next();
        assert!(first_link.is_some());
        let span_link = first_link.unwrap().unwrap();
        assert_eq!(span_link.link.trace_id, vec![1u8; 16]);
        assert_eq!(span_link.link.span_id, vec![11u8; 8]);
        assert_eq!(span_link.link.trace_state, "state1");
        assert_eq!(span_link.link.dropped_attributes_count, 0);

        // Second link
        let second_link = iterator.next();
        assert!(second_link.is_some());
        let span_link = second_link.unwrap().unwrap();
        assert_eq!(span_link.link.trace_id, vec![2u8; 16]);
        assert_eq!(span_link.link.span_id, vec![12u8; 8]);
        assert_eq!(span_link.link.trace_state, "state2");
        assert_eq!(span_link.link.dropped_attributes_count, 1);

        // Third link
        let third_link = iterator.next();
        assert!(third_link.is_some());
        let span_link = third_link.unwrap().unwrap();
        assert_eq!(span_link.link.trace_id, vec![3u8; 16]);
        assert_eq!(span_link.link.span_id, vec![13u8; 8]);
        assert_eq!(span_link.link.trace_state, "state3");
        assert_eq!(span_link.link.dropped_attributes_count, 2);

        // No more links
        let fourth_link = iterator.next();
        assert!(fourth_link.is_none());
    }

    #[test]
    fn test_span_link_iterator_empty_batch() {
        let rb = create_empty_record_batch();
        let mut iterator = SpanLinkIterator::try_new(&rb, None).unwrap();

        assert_eq!(iterator.num_rows, 0);
        assert!(iterator.next().is_none());
    }

    #[test]
    fn test_span_links_store_multiple_links_same_parent() {
        let schema = Arc::new(Schema::new(vec![
            Field::new(consts::ID, DataType::UInt32, true),
            Field::new(consts::TRACE_ID, DataType::FixedSizeBinary(16), true),
            Field::new(consts::PARENT_ID, DataType::UInt16, true),
            Field::new(consts::SPAN_ID, DataType::FixedSizeBinary(8), true),
            Field::new(consts::TRACE_STATE, DataType::Utf8, true),
            Field::new(consts::DROPPED_ATTRIBUTES_COUNT, DataType::UInt32, true),
        ]));

        // Create batch with multiple links for the same parent
        let id_array = Arc::new(UInt32Array::from(vec![Some(1), Some(2), Some(3), Some(4)]));

        let trace_id_data = vec![vec![1u8; 16], vec![2u8; 16], vec![3u8; 16], vec![4u8; 16]];
        let trace_id_array =
            Arc::new(FixedSizeBinaryArray::try_from_iter(trace_id_data.into_iter()).unwrap());

        let parent_id_array = Arc::new(UInt16Array::from(vec![
            Some(100),
            Some(100),
            Some(100),
            Some(100),
        ]));

        let span_id_data = vec![vec![11u8; 8], vec![12u8; 8], vec![13u8; 8], vec![14u8; 8]];
        let span_id_array =
            Arc::new(FixedSizeBinaryArray::try_from_iter(span_id_data.into_iter()).unwrap());

        let trace_state_array = Arc::new(StringArray::from(vec![
            Some("state1"),
            Some("state2"),
            Some("state3"),
            Some("state4"),
        ]));

        let dropped_count_array =
            Arc::new(UInt32Array::from(vec![Some(0), Some(0), Some(0), Some(0)]));

        let rb = RecordBatch::try_new(schema, vec![
            id_array,
            trace_id_array,
            parent_id_array,
            span_id_array,
            trace_state_array,
            dropped_count_array,
        ])
        .unwrap();

        let store = span_links_store_from_record_batch(&rb, None).unwrap();

        let links = store.link_by_id(100);
        assert!(links.is_some());
        assert_eq!(links.unwrap().len(), 4);

        let links = links.unwrap();
        assert_eq!(links[0].trace_state, "state1");
        assert_eq!(links[1].trace_state, "state2");
        assert_eq!(links[2].trace_state, "state3");
        assert_eq!(links[3].trace_state, "state4");
    }

    #[test]
    fn test_span_links_store_with_null_values() {
        let schema = Arc::new(Schema::new(vec![
            Field::new(consts::ID, DataType::UInt32, true),
            Field::new(consts::TRACE_ID, DataType::FixedSizeBinary(16), true),
            Field::new(consts::PARENT_ID, DataType::UInt16, true),
            Field::new(consts::SPAN_ID, DataType::FixedSizeBinary(8), true),
            Field::new(consts::TRACE_STATE, DataType::Utf8, true),
            Field::new(consts::DROPPED_ATTRIBUTES_COUNT, DataType::UInt32, true),
        ]));

        // Create batch with some null values
        let id_array = Arc::new(UInt32Array::from(vec![None, Some(2)]));

        let trace_id_data = vec![None, Some(vec![2u8; 16])];
        let trace_id_array = Arc::new(
            FixedSizeBinaryArray::try_from_sparse_iter_with_size(trace_id_data.into_iter(), 16i32)
                .unwrap(),
        );

        let parent_id_array = Arc::new(UInt16Array::from(vec![Some(10), None]));

        let span_id_data = vec![None, Some(vec![12u8; 8])];
        let span_id_array = Arc::new(
            FixedSizeBinaryArray::try_from_sparse_iter_with_size(span_id_data.into_iter(), 8i32)
                .unwrap(),
        );

        let trace_state_array = Arc::new(StringArray::from(vec![None, Some("state2")]));
        let dropped_count_array = Arc::new(UInt32Array::from(vec![None, Some(1)]));

        let rb = RecordBatch::try_new(schema, vec![
            id_array,
            trace_id_array,
            parent_id_array,
            span_id_array,
            trace_state_array,
            dropped_count_array,
        ])
        .unwrap();

        let _ = span_links_store_from_record_batch(&rb, None).unwrap();
    }

    #[test]
    fn test_span_links_store_delta_decoding() {
        let schema = Arc::new(Schema::new(vec![
            Field::new(consts::ID, DataType::UInt32, true),
            Field::new(consts::TRACE_ID, DataType::FixedSizeBinary(16), true),
            Field::new(consts::PARENT_ID, DataType::UInt16, true),
            Field::new(consts::SPAN_ID, DataType::FixedSizeBinary(8), true),
            Field::new(consts::TRACE_STATE, DataType::Utf8, true),
            Field::new(consts::DROPPED_ATTRIBUTES_COUNT, DataType::UInt32, true),
        ]));

        // Test delta decoding with same trace_id
        let id_array = Arc::new(UInt32Array::from(vec![Some(1), Some(2), Some(3)]));

        let trace_id_data = vec![
            Some(vec![1u8; 16]), // Same trace_id
            Some(vec![1u8; 16]), // Same trace_id - should accumulate parent_id
            Some(vec![2u8; 16]), // Different trace_id - should reset
        ];
        let trace_id_array = Arc::new(
            FixedSizeBinaryArray::try_from_sparse_iter_with_size(trace_id_data.into_iter(), 16i32)
                .unwrap(),
        );

        let parent_id_array = Arc::new(UInt16Array::from(vec![Some(10), Some(5), Some(20)]));

        let span_id_data = vec![
            Some(vec![11u8; 8]),
            Some(vec![12u8; 8]),
            Some(vec![13u8; 8]),
        ];
        let span_id_array = Arc::new(
            FixedSizeBinaryArray::try_from_sparse_iter_with_size(span_id_data.into_iter(), 8i32)
                .unwrap(),
        );

        let trace_state_array = Arc::new(StringArray::from(vec![
            Some("state1"),
            Some("state2"),
            Some("state3"),
        ]));

        let dropped_count_array = Arc::new(UInt32Array::from(vec![Some(0), Some(1), Some(2)]));

        let rb = RecordBatch::try_new(schema, vec![
            id_array,
            trace_id_array,
            parent_id_array,
            span_id_array,
            trace_state_array,
            dropped_count_array,
        ])
        .unwrap();

        let store = span_links_store_from_record_batch(&rb, None).unwrap();

        // First link should have parent_id 10
        let links_10 = store.link_by_id(10);
        assert!(links_10.is_some());
        assert_eq!(links_10.unwrap().len(), 1);

        // Second link should have accumulated parent_id (10 + 5 = 15)
        let links_15 = store.link_by_id(15);
        assert!(links_15.is_some());
        assert_eq!(links_15.unwrap().len(), 1);

        // Third link should have reset parent_id to 20 (new trace_id)
        let links_20 = store.link_by_id(20);
        assert!(links_20.is_some());
        assert_eq!(links_20.unwrap().len(), 1);
    }
}
