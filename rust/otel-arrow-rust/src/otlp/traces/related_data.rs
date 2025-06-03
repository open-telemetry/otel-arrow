// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::error;
use crate::otap::OtapBatch;
use crate::otlp::attributes::store::{Attribute16Store, Attribute32Store};
use crate::otlp::traces::span_event_store::{SpanEventsStore, span_events_store_from_record_batch};
use crate::otlp::traces::span_links_store::{SpanLinksStore, span_links_store_from_record_batch};
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;

pub struct RelatedData {
    pub(crate) span_id: u16,
    pub(crate) res_attr_map_store: Option<Attribute16Store>,
    pub(crate) scope_attr_map_store: Option<Attribute16Store>,
    pub(crate) span_attr_map_store: Option<Attribute16Store>,
    #[allow(unused)]
    pub(crate) span_event_attr_map_store: Option<Attribute32Store>,
    #[allow(unused)]
    pub(crate) span_link_attr_map_store: Option<Attribute32Store>,
    pub(crate) span_events_store: Option<SpanEventsStore>,
    pub(crate) span_links_store: Option<SpanLinksStore>,
}

impl RelatedData {
    pub fn span_id_from_delta(&mut self, delta: u16) -> u16 {
        self.span_id += delta;
        self.span_id
    }
}

impl TryFrom<&OtapBatch> for RelatedData {
    type Error = error::Error;

    fn try_from(value: &OtapBatch) -> Result<Self, Self::Error> {
        let res_attr_map_store = value
            .get(ArrowPayloadType::ResourceAttrs)
            .map(Attribute16Store::try_from)
            .transpose()?;
        let scope_attr_map_store = value
            .get(ArrowPayloadType::ScopeAttrs)
            .map(Attribute16Store::try_from)
            .transpose()?;
        let span_attr_map_store = value
            .get(ArrowPayloadType::SpanAttrs)
            .map(Attribute16Store::try_from)
            .transpose()?;
        let mut span_event_attr_map_store = value
            .get(ArrowPayloadType::SpanEventAttrs)
            .map(Attribute32Store::try_from)
            .transpose()?;
        let mut span_link_attr_map_store = value
            .get(ArrowPayloadType::SpanLinkAttrs)
            .map(Attribute32Store::try_from)
            .transpose()?;
        let span_events_store = value
            .get(ArrowPayloadType::SpanEvents)
            .map(|rb| span_events_store_from_record_batch(rb, span_event_attr_map_store.as_mut()))
            .transpose()?;
        let span_links_store = value
            .get(ArrowPayloadType::SpanLinks)
            .map(|rb| span_links_store_from_record_batch(rb, span_link_attr_map_store.as_mut()))
            .transpose()?;

        Ok(Self {
            span_id: 0,
            res_attr_map_store,
            scope_attr_map_store,
            span_attr_map_store,
            span_event_attr_map_store,
            span_link_attr_map_store,
            span_events_store,
            span_links_store,
        })
    }
}
