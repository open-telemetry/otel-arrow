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

pub fn span_links_store_from_record_batch(
    rb: &RecordBatch,
    attr_store: Option<&mut Attribute32Store>,
) -> error::Result<SpanLinksStore> {
    let mut store = SpanLinksStore::default();
    let mut iterator = SpanLinkIterator::try_new(rb, attr_store)?;

    while let Some(batch) = iterator.next() {
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

impl<'a> Iterator for SpanLinkIterator<'a> {
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
