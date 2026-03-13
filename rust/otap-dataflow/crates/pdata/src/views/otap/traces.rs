// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Zero-copy view implementation for OTAP Arrow RecordBatches (Traces)
//!
//! This module provides a direct view over OTAP Arrow columnar data for traces/spans
//! without any conversion to OTLP bytes or Prost messages, enabling true zero-copy iteration.
//!
//! The hierarchical structure mirrors the OTLP traces data model:
//! ```text
//! OtapTracesView (TracesView)
//! └── OtapResourceSpansView (ResourceSpansView)
//!     └── OtapScopeSpansView (ScopeSpansView)
//!         └── OtapSpanView (SpanView)
//!             ├── attributes (via OtapAttributeIter)
//!             ├── OtapEventView (EventView)
//!             ├── OtapLinkView (LinkView)
//!             └── OtapStatusView (StatusView)
//! ```

use std::collections::BTreeMap;

use arrow::array::{Array, RecordBatch, StructArray, UInt16Array, UInt32Array};

use crate::arrays::NullableArrayAccessor;
use crate::error::Error;
use crate::otap::OtapArrowRecords;
use crate::otlp::attributes::{Attribute16Arrays, Attribute32Arrays, AttributeValueType};
use crate::otlp::common::{ResourceArrays, ScopeArrays};
use crate::otlp::traces::{
    span_event::SpanEventArrays, span_link::SpanLinkArrays, spans_arrays::SpansArrays,
};
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use crate::schema::consts;
use crate::schema::{SpanId, TraceId};
use otap_df_pdata_views::views::common::{InstrumentationScopeView, Str};
use otap_df_pdata_views::views::resource::ResourceView;
use otap_df_pdata_views::views::trace::{
    EventView, LinkView, ResourceSpansView, ScopeSpansView, SpanView, StatusView, TracesView,
};

use crate::views::otap::common::{
    OtapAnyValueView, OtapAttributeView, RowGroup, RowGroupIter, build_attribute_index,
    build_attribute_index_u32,
};

// ===== Main View =====

/// Zero-copy view over OTAP traces Arrow RecordBatches.
///
/// This struct holds references to the Arrow RecordBatches and pre-computed indices
/// for efficient hierarchical traversal of the trace data.
pub struct OtapTracesView<'a> {
    columns: SpansArrays<'a>,
    resource_columns: ResourceArrays<'a>,
    scope_columns: ScopeArrays<'a>,
    event_columns: Option<SpanEventArrays<'a>>,
    link_columns: Option<SpanLinkArrays<'a>>,

    resource_attrs: Option<Attribute16Arrays<'a>>,
    scope_attrs: Option<Attribute16Arrays<'a>>,
    span_attrs: Option<Attribute16Arrays<'a>>,
    event_attrs: Option<Attribute32Arrays<'a>>,
    link_attrs: Option<Attribute32Arrays<'a>>,

    // Pre-computed indices for hierarchy reconstruction
    resource_groups: Vec<(u16, RowGroup)>,
    scope_groups_map: BTreeMap<u16, Vec<(u16, RowGroup)>>,

    // Pre-computed attribute index maps (parent_id -> list of attr row indices)
    resource_attrs_map: BTreeMap<u16, Vec<usize>>,
    scope_attrs_map: BTreeMap<u16, Vec<usize>>,
    span_attrs_map: BTreeMap<u16, Vec<usize>>,
    // Event and link attributes use u32 parent_id per OTAP spec §5.4.1
    event_attrs_map: BTreeMap<u32, Vec<usize>>,
    link_attrs_map: BTreeMap<u32, Vec<usize>>,

    // Pre-computed event/link index maps (parent span id -> list of event/link row indices)
    events_map: BTreeMap<u16, Vec<usize>>,
    links_map: BTreeMap<u16, Vec<usize>>,
}

impl<'a> OtapTracesView<'a> {
    /// Create a new OTAP traces view from Arrow RecordBatches.
    ///
    /// # Arguments
    /// * `spans_batch` - The main spans record batch
    /// * `resource_attrs` - Optional resource attributes batch
    /// * `scope_attrs` - Optional scope attributes batch
    /// * `span_attrs` - Optional span attributes batch
    /// * `events_batch` - Optional span events batch
    /// * `event_attrs` - Optional event attributes batch
    /// * `links_batch` - Optional span links batch
    /// * `link_attrs` - Optional link attributes batch
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        spans_batch: &'a RecordBatch,
        resource_attrs: Option<&'a RecordBatch>,
        scope_attrs: Option<&'a RecordBatch>,
        span_attrs: Option<&'a RecordBatch>,
        events_batch: Option<&'a RecordBatch>,
        event_attrs: Option<&'a RecordBatch>,
        links_batch: Option<&'a RecordBatch>,
        link_attrs: Option<&'a RecordBatch>,
    ) -> Result<Self, Error> {
        // 1. Cache columns for O(1) access
        let columns = SpansArrays::try_from(spans_batch)?;
        let resource_columns = ResourceArrays::try_from(spans_batch)?;
        let scope_columns = ScopeArrays::try_from(spans_batch)?;

        // 2. Pre-compute resource grouping
        let resource_groups = group_by_resource_id(spans_batch);

        // 3. Pre-compute scope grouping per resource
        let mut scope_groups_map = BTreeMap::new();
        for (resource_id, row_indices) in &resource_groups {
            let scope_groups = group_by_scope_id(spans_batch, row_indices);
            let _ = scope_groups_map.insert(*resource_id, scope_groups);
        }

        // 4. Pre-compute attribute mappings
        let resource_attrs_map = resource_attrs
            .map(build_attribute_index)
            .unwrap_or_default();
        let scope_attrs_map = scope_attrs.map(build_attribute_index).unwrap_or_default();
        let span_attrs_map = span_attrs.map(build_attribute_index).unwrap_or_default();
        let event_attrs_map = event_attrs
            .map(build_attribute_index_u32)
            .unwrap_or_default();
        let link_attrs_map = link_attrs
            .map(build_attribute_index_u32)
            .unwrap_or_default();

        let events_map = events_batch
            .map(crate::views::otap::common::build_u16_index)
            .unwrap_or_default();
        let links_map = links_batch
            .map(crate::views::otap::common::build_u16_index)
            .unwrap_or_default();

        // 6. Cache event/link columns
        let event_columns = events_batch.map(SpanEventArrays::try_from).transpose()?;
        let link_columns = links_batch.map(SpanLinkArrays::try_from).transpose()?;

        Ok(Self {
            columns,
            resource_columns,
            scope_columns,
            event_columns,
            link_columns,
            resource_attrs: resource_attrs
                .map(Attribute16Arrays::try_from)
                .transpose()?,
            scope_attrs: scope_attrs.map(Attribute16Arrays::try_from).transpose()?,
            span_attrs: span_attrs.map(Attribute16Arrays::try_from).transpose()?,
            event_attrs: event_attrs.map(Attribute32Arrays::try_from).transpose()?,
            link_attrs: link_attrs.map(Attribute32Arrays::try_from).transpose()?,

            resource_groups,
            scope_groups_map,
            resource_attrs_map,
            scope_attrs_map,
            span_attrs_map,
            event_attrs_map,
            link_attrs_map,
            events_map,
            links_map,
        })
    }
}

impl<'a> TryFrom<&'a OtapArrowRecords> for OtapTracesView<'a> {
    type Error = Error;

    fn try_from(records: &'a OtapArrowRecords) -> Result<Self, Self::Error> {
        let spans_batch =
            records
                .get(ArrowPayloadType::Spans)
                .ok_or(Error::RecordBatchNotFound {
                    payload_type: ArrowPayloadType::Spans,
                })?;

        let resource_attrs = records.get(ArrowPayloadType::ResourceAttrs);
        let scope_attrs = records.get(ArrowPayloadType::ScopeAttrs);
        let span_attrs = records.get(ArrowPayloadType::SpanAttrs);
        let events_batch = records.get(ArrowPayloadType::SpanEvents);
        let event_attrs = records.get(ArrowPayloadType::SpanEventAttrs);
        let links_batch = records.get(ArrowPayloadType::SpanLinks);
        let link_attrs = records.get(ArrowPayloadType::SpanLinkAttrs);

        Self::new(
            spans_batch,
            resource_attrs,
            scope_attrs,
            span_attrs,
            events_batch,
            event_attrs,
            links_batch,
            link_attrs,
        )
    }
}

// ===== TracesView Implementation =====

impl<'a> TracesView for OtapTracesView<'a> {
    type ResourceSpans<'res>
        = OtapResourceSpansView<'res>
    where
        Self: 'res;

    type ResourcesIter<'res>
        = OtapResourceSpansIter<'res>
    where
        Self: 'res;

    #[inline]
    fn resources(&self) -> Self::ResourcesIter<'_> {
        OtapResourceSpansIter::new(self)
    }
}

// ===== Resource Spans =====

/// Iterator over distinct resource groups in OTAP traces
pub struct OtapResourceSpansIter<'a> {
    view: &'a OtapTracesView<'a>,
    resource_groups: std::slice::Iter<'a, (u16, RowGroup)>,
}

impl<'a> OtapResourceSpansIter<'a> {
    #[inline]
    fn new(view: &'a OtapTracesView<'a>) -> Self {
        Self {
            view,
            resource_groups: view.resource_groups.iter(),
        }
    }
}

impl<'a> Iterator for OtapResourceSpansIter<'a> {
    type Item = OtapResourceSpansView<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let (resource_id, row_indices) = self.resource_groups.next()?;
        Some(OtapResourceSpansView {
            view: self.view,
            resource_id: *resource_id,
            row_indices,
        })
    }
}

/// View of a single ResourceSpans in OTAP format
pub struct OtapResourceSpansView<'a> {
    view: &'a OtapTracesView<'a>,
    resource_id: u16,
    #[allow(dead_code)]
    row_indices: &'a RowGroup,
}

impl<'a> ResourceSpansView for OtapResourceSpansView<'a> {
    type Resource<'res>
        = OtapTraceResourceView<'res>
    where
        Self: 'res;

    type ScopeSpans<'scp>
        = OtapScopeSpansView<'scp>
    where
        Self: 'scp;

    type ScopesIter<'scp>
        = OtapScopeSpansIter<'scp>
    where
        Self: 'scp;

    #[inline]
    fn resource(&self) -> Option<Self::Resource<'_>> {
        Some(OtapTraceResourceView {
            view: self.view,
            resource_id: self.resource_id,
        })
    }

    #[inline]
    fn scopes(&self) -> Self::ScopesIter<'_> {
        OtapScopeSpansIter::new(self)
    }

    #[inline]
    fn schema_url(&self) -> Option<Str<'_>> {
        let first_row = self.row_indices.iter().next()?;
        self.view
            .resource_columns
            .schema_url
            .as_ref()
            .and_then(|c| c.str_at(first_row))
            .map(|s| s.as_bytes())
    }
}

// ===== Scope Spans =====

/// Iterator over scope spans within a resource
pub struct OtapScopeSpansIter<'a> {
    view: &'a OtapTracesView<'a>,
    scope_groups: std::slice::Iter<'a, (u16, RowGroup)>,
}

impl<'a> OtapScopeSpansIter<'a> {
    #[inline]
    fn new(resource_view: &'a OtapResourceSpansView<'a>) -> Self {
        let scope_groups = resource_view
            .view
            .scope_groups_map
            .get(&resource_view.resource_id)
            .map(|v: &Vec<(u16, RowGroup)>| v.iter())
            .unwrap_or_default();

        Self {
            view: resource_view.view,
            scope_groups,
        }
    }
}

impl<'a> Iterator for OtapScopeSpansIter<'a> {
    type Item = OtapScopeSpansView<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let (scope_id, row_indices) = self.scope_groups.next()?;
        Some(OtapScopeSpansView {
            view: self.view,
            scope_id: *scope_id,
            row_indices,
        })
    }
}

/// View of a single ScopeSpans in OTAP format
pub struct OtapScopeSpansView<'a> {
    view: &'a OtapTracesView<'a>,
    scope_id: u16,
    row_indices: &'a RowGroup,
}

impl<'a> ScopeSpansView for OtapScopeSpansView<'a> {
    type Scope<'scp>
        = OtapTraceInstrumentationScopeView<'scp>
    where
        Self: 'scp;

    type Span<'sp>
        = OtapSpanView<'sp>
    where
        Self: 'sp;

    type SpanIter<'sp>
        = OtapSpansIter<'sp>
    where
        Self: 'sp;

    #[inline]
    fn scope(&self) -> Option<Self::Scope<'_>> {
        Some(OtapTraceInstrumentationScopeView {
            view: self.view,
            scope_id: self.scope_id,
        })
    }

    #[inline]
    fn spans(&self) -> Self::SpanIter<'_> {
        OtapSpansIter::new(self.view, self.row_indices)
    }

    #[inline]
    fn schema_url(&self) -> Option<Str<'_>> {
        None
    }
}

// ===== Span Iterator =====

/// Iterator over spans within a scope
pub struct OtapSpansIter<'a> {
    view: &'a OtapTracesView<'a>,
    row_indices: RowGroupIter<'a>,
}

impl<'a> OtapSpansIter<'a> {
    #[inline]
    fn new(view: &'a OtapTracesView<'a>, row_indices: &'a RowGroup) -> Self {
        Self {
            view,
            row_indices: row_indices.iter(),
        }
    }
}

impl<'a> Iterator for OtapSpansIter<'a> {
    type Item = OtapSpanView<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let row_idx = self.row_indices.next()?;
        Some(OtapSpanView {
            view: self.view,
            row_idx,
        })
    }
}

// ===== Span View =====

/// View of a single Span in OTAP format
pub struct OtapSpanView<'a> {
    view: &'a OtapTracesView<'a>,
    row_idx: usize,
}

impl<'a> SpanView for OtapSpanView<'a> {
    type Attribute<'att>
        = OtapAttributeView<'att>
    where
        Self: 'att;

    type AttributeIter<'att>
        = OtapSpanAttributeIter<'att>
    where
        Self: 'att;

    type Event<'ev>
        = OtapEventView<'ev>
    where
        Self: 'ev;

    type EventsIter<'ev>
        = OtapEventsIter<'ev>
    where
        Self: 'ev;

    type Link<'ln>
        = OtapLinkView<'ln>
    where
        Self: 'ln;

    type LinksIter<'ln>
        = OtapLinksIter<'ln>
    where
        Self: 'ln;

    type Status<'st>
        = OtapStatusView<'st>
    where
        Self: 'st;

    #[inline]
    fn trace_id(&self) -> Option<&TraceId> {
        self.view.columns.trace_id.as_ref().and_then(|col| {
            col.slice_at(self.row_idx)
                .and_then(|bytes: &[u8]| bytes.try_into().ok())
        })
    }

    #[inline]
    fn span_id(&self) -> Option<&SpanId> {
        self.view.columns.span_id.as_ref().and_then(|col| {
            col.slice_at(self.row_idx)
                .and_then(|bytes: &[u8]| bytes.try_into().ok())
        })
    }

    #[inline]
    fn trace_state(&self) -> Option<Str<'_>> {
        self.view
            .columns
            .trace_state
            .as_ref()
            .and_then(|col| col.str_at(self.row_idx).map(|s| s.as_bytes()))
    }

    #[inline]
    fn parent_span_id(&self) -> Option<&SpanId> {
        self.view.columns.parent_span_id.as_ref().and_then(|col| {
            col.slice_at(self.row_idx)
                .and_then(|bytes: &[u8]| bytes.try_into().ok())
        })
    }

    #[inline]
    fn flags(&self) -> Option<u32> {
        self.view.columns.flags.as_ref().and_then(|col| {
            if col.is_valid(self.row_idx) {
                Some(col.value(self.row_idx))
            } else {
                None
            }
        })
    }

    #[inline]
    fn name(&self) -> Option<Str<'_>> {
        self.view
            .columns
            .name
            .as_ref()
            .and_then(|col| col.str_at(self.row_idx).map(|s| s.as_bytes()))
    }

    #[inline]
    fn kind(&self) -> i32 {
        self.view
            .columns
            .kind
            .as_ref()
            .and_then(|col| col.value_at(self.row_idx))
            .unwrap_or(0)
    }

    #[inline]
    fn start_time_unix_nano(&self) -> Option<u64> {
        self.view
            .columns
            .start_time_unix_nano
            .as_ref()
            .and_then(|col| {
                if col.is_valid(self.row_idx) {
                    Some(col.value(self.row_idx) as u64)
                } else {
                    None
                }
            })
    }

    #[inline]
    fn end_time_unix_nano(&self) -> Option<u64> {
        // OTAP stores duration rather than end_time, so we compute:
        // end_time = start_time + duration
        let start = self.start_time_unix_nano()?;
        let duration = self.get_duration_nanos()?;
        Some(start.wrapping_add(duration as u64))
    }

    #[inline]
    fn attributes(&self) -> Self::AttributeIter<'_> {
        let span_id = self.get_span_row_id();
        let matching_rows = span_id
            .and_then(|id| self.view.span_attrs_map.get(&id))
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        OtapSpanAttributeIter {
            attrs: self.view.span_attrs.as_ref(),
            matching_rows,
            current_idx: 0,
        }
    }

    #[inline]
    fn dropped_attributes_count(&self) -> u32 {
        self.view
            .columns
            .dropped_attributes_count
            .as_ref()
            .map(|col| {
                if col.is_valid(self.row_idx) {
                    col.value(self.row_idx)
                } else {
                    0
                }
            })
            .unwrap_or(0)
    }

    #[inline]
    fn events(&self) -> Self::EventsIter<'_> {
        let span_id = self.get_span_row_id();
        let matching_rows = span_id
            .and_then(|id| self.view.events_map.get(&id))
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        OtapEventsIter {
            view: self.view,
            matching_rows,
            current_idx: 0,
        }
    }

    #[inline]
    fn dropped_events_count(&self) -> u32 {
        self.view
            .columns
            .dropped_events_count
            .as_ref()
            .map(|col| {
                if col.is_valid(self.row_idx) {
                    col.value(self.row_idx)
                } else {
                    0
                }
            })
            .unwrap_or(0)
    }

    #[inline]
    fn links(&self) -> Self::LinksIter<'_> {
        let span_id = self.get_span_row_id();
        let matching_rows = span_id
            .and_then(|id| self.view.links_map.get(&id))
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        OtapLinksIter {
            view: self.view,
            matching_rows,
            current_idx: 0,
        }
    }

    #[inline]
    fn dropped_links_count(&self) -> u32 {
        self.view
            .columns
            .dropped_links_count
            .as_ref()
            .map(|col| {
                if col.is_valid(self.row_idx) {
                    col.value(self.row_idx)
                } else {
                    0
                }
            })
            .unwrap_or(0)
    }

    #[inline]
    fn status(&self) -> Option<Self::Status<'_>> {
        // Return a status view if we have status columns
        if self.view.columns.status.is_some() {
            Some(OtapStatusView {
                view: self.view,
                row_idx: self.row_idx,
            })
        } else {
            None
        }
    }
}

impl<'a> OtapSpanView<'a> {
    /// Get the span's row ID from the "id" column (used for attribute/event/link matching)
    #[inline]
    fn get_span_row_id(&self) -> Option<u16> {
        let array = &self.view.columns.id;
        if array.is_valid(self.row_idx) {
            Some(array.value(self.row_idx))
        } else {
            None
        }
    }

    /// Get the duration in nanoseconds from the duration column
    #[inline]
    fn get_duration_nanos(&self) -> Option<i64> {
        self.view
            .columns
            .duration_time_unix_nano
            .as_ref()
            .and_then(|col| col.value_at(self.row_idx))
    }
}

// ===== Span Attribute Iterator =====

/// Iterator over span attributes (u16 parent_id).
/// Used for resource, scope, and span attributes.
pub struct OtapSpanAttributeIter<'a> {
    attrs: Option<&'a Attribute16Arrays<'a>>,
    matching_rows: &'a [usize],
    current_idx: usize,
}

impl<'a> Iterator for OtapSpanAttributeIter<'a> {
    type Item = OtapAttributeView<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let attrs_cols = self.attrs?;

        loop {
            if self.current_idx >= self.matching_rows.len() {
                return None;
            }

            let attr_row_idx = self.matching_rows[self.current_idx];
            self.current_idx += 1;

            if let Some(key) = get_attribute_key(attrs_cols, attr_row_idx) {
                let value = get_attribute_value(attrs_cols, attr_row_idx);
                return Some(OtapAttributeView { key, value });
            }
        }
    }
}

/// Iterator over event/link attributes (u32 parent_id).
/// Per OTAP spec §5.4.1, event and link attributes use u32 parent_id.
pub struct OtapU32AttributeIter<'a> {
    attrs: Option<&'a Attribute32Arrays<'a>>,
    matching_rows: &'a [usize],
    current_idx: usize,
}

impl<'a> Iterator for OtapU32AttributeIter<'a> {
    type Item = OtapAttributeView<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let attrs_cols = self.attrs?;

        loop {
            if self.current_idx >= self.matching_rows.len() {
                return None;
            }

            let attr_row_idx = self.matching_rows[self.current_idx];
            self.current_idx += 1;

            if let Some(key) = get_attribute_key(attrs_cols, attr_row_idx) {
                let value = get_attribute_value(attrs_cols, attr_row_idx);
                return Some(OtapAttributeView { key, value });
            }
        }
    }
}

// ===== Event View =====

/// View of a single Event in OTAP format
pub struct OtapEventView<'a> {
    view: &'a OtapTracesView<'a>,
    event_row_idx: usize,
}

impl<'a> EventView for OtapEventView<'a> {
    type Attribute<'att>
        = OtapAttributeView<'att>
    where
        Self: 'att;

    type AttributeIter<'att>
        = OtapU32AttributeIter<'att>
    where
        Self: 'att;

    #[inline]
    fn time_unix_nano(&self) -> Option<u64> {
        self.view
            .event_columns
            .as_ref()
            .and_then(|cols| cols.time_unix_nano.as_ref())
            .and_then(|col| {
                if col.is_valid(self.event_row_idx) {
                    Some(col.value(self.event_row_idx) as u64)
                } else {
                    None
                }
            })
    }

    #[inline]
    fn name(&self) -> Option<Str<'_>> {
        self.view
            .event_columns
            .as_ref()
            .and_then(|cols| cols.name.as_ref())
            .and_then(|col| col.str_at(self.event_row_idx).map(|s| s.as_bytes()))
    }

    #[inline]
    fn attributes(&self) -> Self::AttributeIter<'_> {
        // Event attributes use u32 parent_id per OTAP spec §5.4.1
        let event_id = self.get_event_id();
        let matching_rows = event_id
            .and_then(|id| self.view.event_attrs_map.get(&id))
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        OtapU32AttributeIter {
            attrs: self.view.event_attrs.as_ref(),
            matching_rows,
            current_idx: 0,
        }
    }

    #[inline]
    fn dropped_attributes_count(&self) -> u32 {
        self.view
            .event_columns
            .as_ref()
            .and_then(|cols| cols.dropped_attributes_count.as_ref())
            .map(|col| {
                if col.is_valid(self.event_row_idx) {
                    col.value(self.event_row_idx)
                } else {
                    0
                }
            })
            .unwrap_or(0)
    }
}

impl<'a> OtapEventView<'a> {
    /// Get event ID (u32) from the events batch for attribute matching.
    /// Event attributes are linked via parent_id (u32) in the event_attrs batch.
    #[inline]
    fn get_event_id(&self) -> Option<u32> {
        self.view
            .event_columns
            .as_ref()
            .and_then(|cols| cols.id.as_ref())
            .and_then(|id_col| {
                if id_col.is_valid(self.event_row_idx) {
                    Some(id_col.value(self.event_row_idx))
                } else {
                    None
                }
            })
    }
}

/// Iterator over events for a span
pub struct OtapEventsIter<'a> {
    view: &'a OtapTracesView<'a>,
    matching_rows: &'a [usize],
    current_idx: usize,
}

impl<'a> Iterator for OtapEventsIter<'a> {
    type Item = OtapEventView<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_idx >= self.matching_rows.len() {
            return None;
        }

        let event_row_idx = self.matching_rows[self.current_idx];
        self.current_idx += 1;

        Some(OtapEventView {
            view: self.view,
            event_row_idx,
        })
    }
}

// ===== Link View =====

/// View of a single Link in OTAP format
pub struct OtapLinkView<'a> {
    view: &'a OtapTracesView<'a>,
    link_row_idx: usize,
}

impl<'a> LinkView for OtapLinkView<'a> {
    type Attribute<'att>
        = OtapAttributeView<'att>
    where
        Self: 'att;

    type AttributeIter<'att>
        = OtapU32AttributeIter<'att>
    where
        Self: 'att;

    #[inline]
    fn trace_id(&self) -> Option<&TraceId> {
        self.view
            .link_columns
            .as_ref()
            .and_then(|cols| cols.trace_id.as_ref())
            .and_then(|col| {
                col.slice_at(self.link_row_idx)
                    .and_then(|bytes: &[u8]| bytes.try_into().ok())
            })
    }

    #[inline]
    fn span_id(&self) -> Option<&SpanId> {
        self.view
            .link_columns
            .as_ref()
            .and_then(|cols| cols.span_id.as_ref())
            .and_then(|col| {
                col.slice_at(self.link_row_idx)
                    .and_then(|bytes: &[u8]| bytes.try_into().ok())
            })
    }

    #[inline]
    fn trace_state(&self) -> Option<Str<'_>> {
        self.view
            .link_columns
            .as_ref()
            .and_then(|cols| cols.trace_state.as_ref())
            .and_then(|col| col.str_at(self.link_row_idx).map(|s| s.as_bytes()))
    }

    #[inline]
    fn attributes(&self) -> Self::AttributeIter<'_> {
        // Link attributes use u32 parent_id per OTAP spec §5.4.1
        let link_id = self.get_link_id();
        let matching_rows = link_id
            .and_then(|id| self.view.link_attrs_map.get(&id))
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        OtapU32AttributeIter {
            attrs: self.view.link_attrs.as_ref(),
            matching_rows,
            current_idx: 0,
        }
    }

    #[inline]
    fn dropped_attributes_count(&self) -> u32 {
        self.view
            .link_columns
            .as_ref()
            .and_then(|cols| cols.dropped_attributes_count.as_ref())
            .map(|col| {
                if col.is_valid(self.link_row_idx) {
                    col.value(self.link_row_idx)
                } else {
                    0
                }
            })
            .unwrap_or(0)
    }

    #[inline]
    fn flags(&self) -> Option<u32> {
        self.view
            .link_columns
            .as_ref()
            .and_then(|cols| cols.flags.as_ref())
            .and_then(|col| {
                if col.is_valid(self.link_row_idx) {
                    Some(col.value(self.link_row_idx))
                } else {
                    None
                }
            })
    }
}

impl<'a> OtapLinkView<'a> {
    /// Get link ID (u32) from the links batch for attribute matching.
    /// Link attributes are linked via parent_id (u32) in the link_attrs batch.
    #[inline]
    fn get_link_id(&self) -> Option<u32> {
        self.view
            .link_columns
            .as_ref()
            .and_then(|cols| cols.id.as_ref())
            .and_then(|id_col| {
                if id_col.is_valid(self.link_row_idx) {
                    Some(id_col.value(self.link_row_idx))
                } else {
                    None
                }
            })
    }
}

/// Iterator over links for a span
pub struct OtapLinksIter<'a> {
    view: &'a OtapTracesView<'a>,
    matching_rows: &'a [usize],
    current_idx: usize,
}

impl<'a> Iterator for OtapLinksIter<'a> {
    type Item = OtapLinkView<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_idx >= self.matching_rows.len() {
            return None;
        }

        let link_row_idx = self.matching_rows[self.current_idx];
        self.current_idx += 1;

        Some(OtapLinkView {
            view: self.view,
            link_row_idx,
        })
    }
}

// ===== Status View =====

/// View of a Span's Status in OTAP format.
/// Status is stored as a struct column with "code" and "status_message" sub-fields.
pub struct OtapStatusView<'a> {
    view: &'a OtapTracesView<'a>,
    row_idx: usize,
}

impl<'a> StatusView for OtapStatusView<'a> {
    #[inline]
    fn message(&self) -> Option<Str<'_>> {
        self.view
            .columns
            .status
            .as_ref()
            .and_then(|s| s.message.as_ref())
            .and_then(|col| col.str_at(self.row_idx).map(|s| s.as_bytes()))
    }

    #[inline]
    fn status_code(&self) -> i32 {
        self.view
            .columns
            .status
            .as_ref()
            .and_then(|s| s.code.as_ref())
            .and_then(|col| col.value_at(self.row_idx))
            .unwrap_or(0)
    }
}

// ===== Resource and Scope Views for Traces =====

/// View of a Resource in OTAP traces format
pub struct OtapTraceResourceView<'a> {
    view: &'a OtapTracesView<'a>,
    resource_id: u16,
}

impl<'a> ResourceView for OtapTraceResourceView<'a> {
    type Attribute<'att>
        = OtapAttributeView<'att>
    where
        Self: 'att;

    type AttributesIter<'att>
        = OtapSpanAttributeIter<'att>
    where
        Self: 'att;

    #[inline]
    fn attributes(&self) -> Self::AttributesIter<'_> {
        let matching_rows = self
            .view
            .resource_attrs_map
            .get(&self.resource_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        OtapSpanAttributeIter {
            attrs: self.view.resource_attrs.as_ref(),
            matching_rows,
            current_idx: 0,
        }
    }

    #[inline]
    fn dropped_attributes_count(&self) -> u32 {
        0 // TODO: implement if stored in OTAP
    }
}

/// View of an InstrumentationScope in OTAP traces format
pub struct OtapTraceInstrumentationScopeView<'a> {
    view: &'a OtapTracesView<'a>,
    scope_id: u16,
}

impl<'a> InstrumentationScopeView for OtapTraceInstrumentationScopeView<'a> {
    type Attribute<'att>
        = OtapAttributeView<'att>
    where
        Self: 'att;

    type AttributeIter<'att>
        = OtapSpanAttributeIter<'att>
    where
        Self: 'att;

    #[inline]
    fn name(&self) -> Option<Str<'_>> {
        // Scope name is in the scope struct column; find a row with this scope_id
        let first_row = self.find_first_row_for_scope()?;
        self.view
            .scope_columns
            .name
            .as_ref()
            .and_then(|col| col.str_at(first_row).map(|s| s.as_bytes()))
    }

    #[inline]
    fn version(&self) -> Option<Str<'_>> {
        let first_row = self.find_first_row_for_scope()?;
        self.view
            .scope_columns
            .version
            .as_ref()
            .and_then(|col| col.str_at(first_row).map(|s| s.as_bytes()))
    }

    #[inline]
    fn attributes(&self) -> Self::AttributeIter<'_> {
        let matching_rows = self
            .view
            .scope_attrs_map
            .get(&self.scope_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        OtapSpanAttributeIter {
            attrs: self.view.scope_attrs.as_ref(),
            matching_rows,
            current_idx: 0,
        }
    }

    #[inline]
    fn dropped_attributes_count(&self) -> u32 {
        0 // TODO: implement if stored in OTAP
    }
}

impl<'a> OtapTraceInstrumentationScopeView<'a> {
    /// Find the first row in the spans batch that belongs to this scope.
    /// All rows with the same scope_id share the same scope name/version.
    fn find_first_row_for_scope(&self) -> Option<usize> {
        for scope_list in self.view.scope_groups_map.values() {
            for (sid, row_group) in scope_list {
                if *sid == self.scope_id {
                    return row_group.iter().next(); // type known
                }
            }
        }
        None
    }
}

// ===== Helper Functions =====

/// Group rows by resource ID (same logic as logs)
fn group_by_resource_id(batch: &RecordBatch) -> Vec<(u16, RowGroup)> {
    let num_rows = batch.num_rows();
    if num_rows == 0 {
        return Vec::new();
    }

    let resource_col = match batch.column_by_name(consts::RESOURCE) {
        Some(col) => col,
        None => return Vec::new(),
    };

    let resource_struct = match resource_col.as_any().downcast_ref::<StructArray>() {
        Some(arr) => arr,
        None => return Vec::new(),
    };

    let id_col = match resource_struct.column_by_name(consts::ID) {
        Some(col) => col,
        None => return Vec::new(),
    };

    enum GroupBuilder {
        Contiguous { start: usize, count: usize },
        Scattered(Vec<usize>),
    }

    let mut builders: BTreeMap<u16, GroupBuilder> = BTreeMap::new();

    if let Ok(accessor) = crate::arrays::MaybeDictArrayAccessor::<UInt16Array>::try_new(id_col) {
        for i in 0..num_rows {
            if let Some(id) = accessor.value_at(i) {
                let _ = builders
                    .entry(id)
                    .and_modify(|builder| match builder {
                        GroupBuilder::Contiguous { start, count } => {
                            if *start + *count == i {
                                *count += 1;
                            } else {
                                let mut indices = Vec::with_capacity(*count + 1);
                                indices.extend(*start..(*start + *count));
                                indices.push(i);
                                *builder = GroupBuilder::Scattered(indices);
                            }
                        }
                        GroupBuilder::Scattered(indices) => {
                            indices.push(i);
                        }
                    })
                    .or_insert(GroupBuilder::Contiguous { start: i, count: 1 });
            }
        }
    } else if let Some(id_array) = id_col.as_any().downcast_ref::<UInt16Array>() {
        for i in 0..num_rows {
            if id_array.is_valid(i) {
                let id = id_array.value(i);
                let _ = builders
                    .entry(id)
                    .and_modify(|builder| match builder {
                        GroupBuilder::Contiguous { start, count } => {
                            if *start + *count == i {
                                *count += 1;
                            } else {
                                let mut indices = Vec::with_capacity(*count + 1);
                                indices.extend(*start..(*start + *count));
                                indices.push(i);
                                *builder = GroupBuilder::Scattered(indices);
                            }
                        }
                        GroupBuilder::Scattered(indices) => {
                            indices.push(i);
                        }
                    })
                    .or_insert(GroupBuilder::Contiguous { start: i, count: 1 });
            }
        }
    } else {
        return Vec::new();
    }

    builders
        .into_iter()
        .map(|(id, builder)| {
            let group = match builder {
                GroupBuilder::Contiguous { start, count } => {
                    RowGroup::Contiguous(start..(start + count))
                }
                GroupBuilder::Scattered(indices) => RowGroup::Scattered(indices),
            };
            (id, group)
        })
        .collect()
}

/// Group rows by scope ID within a set of rows (same logic as logs)
fn group_by_scope_id(batch: &RecordBatch, row_indices: &RowGroup) -> Vec<(u16, RowGroup)> {
    let scope_col = match batch.column_by_name(consts::SCOPE) {
        Some(col) => col,
        None => return Vec::new(),
    };

    let scope_struct = match scope_col.as_any().downcast_ref::<StructArray>() {
        Some(arr) => arr,
        None => return Vec::new(),
    };

    let id_col = match scope_struct.column_by_name(consts::ID) {
        Some(col) => col,
        None => return Vec::new(),
    };

    enum GroupBuilder {
        Contiguous { start: usize, count: usize },
        Scattered(Vec<usize>),
    }

    let mut builders: BTreeMap<u16, GroupBuilder> = BTreeMap::new();

    if let Ok(accessor) = crate::arrays::MaybeDictArrayAccessor::<UInt16Array>::try_new(id_col) {
        for row_idx in row_indices.iter() {
            if let Some(scope_id) = accessor.value_at(row_idx) {
                let _ = builders
                    .entry(scope_id)
                    .and_modify(|builder| match builder {
                        GroupBuilder::Contiguous { start, count } => {
                            if *start + *count == row_idx {
                                *count += 1;
                            } else {
                                let mut indices = Vec::with_capacity(*count + 1);
                                indices.extend(*start..(*start + *count));
                                indices.push(row_idx);
                                *builder = GroupBuilder::Scattered(indices);
                            }
                        }
                        GroupBuilder::Scattered(indices) => {
                            indices.push(row_idx);
                        }
                    })
                    .or_insert(GroupBuilder::Contiguous {
                        start: row_idx,
                        count: 1,
                    });
            }
        }
    } else if let Some(scope_id_array) = id_col.as_any().downcast_ref::<UInt16Array>() {
        for row_idx in row_indices.iter() {
            if scope_id_array.is_valid(row_idx) {
                let scope_id = scope_id_array.value(row_idx);
                let _ = builders
                    .entry(scope_id)
                    .and_modify(|builder| match builder {
                        GroupBuilder::Contiguous { start, count } => {
                            if *start + *count == row_idx {
                                *count += 1;
                            } else {
                                let mut indices = Vec::with_capacity(*count + 1);
                                indices.extend(*start..(*start + *count));
                                indices.push(row_idx);
                                *builder = GroupBuilder::Scattered(indices);
                            }
                        }
                        GroupBuilder::Scattered(indices) => {
                            indices.push(row_idx);
                        }
                    })
                    .or_insert(GroupBuilder::Contiguous {
                        start: row_idx,
                        count: 1,
                    });
            }
        }
    } else {
        return Vec::new();
    }

    builders
        .into_iter()
        .map(|(id, builder)| {
            let group = match builder {
                GroupBuilder::Contiguous { start, count } => {
                    RowGroup::Contiguous(start..(start + count))
                }
                GroupBuilder::Scattered(indices) => RowGroup::Scattered(indices),
            };
            (id, group)
        })
        .collect()
}

/// Extract attribute key from an attribute batch row.
/// Generic over ArrowPrimitiveType to work with both u16 and u32 attribute arrays.
fn get_attribute_key<'a, T: arrow::datatypes::ArrowPrimitiveType>(
    cols: &'a crate::otlp::attributes::AttributeArrays<'a, T>,
    row_idx: usize,
) -> Option<&'a [u8]> {
    cols.attr_key.str_at(row_idx).map(|s| s.as_bytes())
}

/// Extract attribute value with type information from an attribute batch row.
/// Generic over ArrowPrimitiveType to work with both u16 and u32 attribute arrays.
fn get_attribute_value<'a, T: arrow::datatypes::ArrowPrimitiveType>(
    cols: &'a crate::otlp::attributes::AttributeArrays<'a, T>,
    row_idx: usize,
) -> OtapAnyValueView<'a> {
    let type_array = &cols.anyval_arrays.attr_type;

    if !type_array.is_valid(row_idx) {
        return OtapAnyValueView::Empty;
    }

    let value_type = match AttributeValueType::try_from(type_array.value(row_idx)) {
        Ok(t) => t,
        Err(_) => return OtapAnyValueView::Empty,
    };

    let anyval = &cols.anyval_arrays;
    match value_type {
        AttributeValueType::Str => anyval
            .attr_str
            .as_ref()
            .and_then(|accessor| accessor.str_at(row_idx))
            .map(|s| OtapAnyValueView::Str(s.as_bytes()))
            .unwrap_or(OtapAnyValueView::Empty),
        AttributeValueType::Int => anyval
            .attr_int
            .as_ref()
            .and_then(|accessor| accessor.value_at(row_idx))
            .map(OtapAnyValueView::Int)
            .unwrap_or(OtapAnyValueView::Empty),
        AttributeValueType::Double => anyval
            .attr_double
            .and_then(|arr| {
                if arr.is_valid(row_idx) {
                    Some(OtapAnyValueView::Double(arr.value(row_idx)))
                } else {
                    None
                }
            })
            .unwrap_or(OtapAnyValueView::Empty),
        AttributeValueType::Bool => anyval
            .attr_bool
            .and_then(|arr| {
                if arr.is_valid(row_idx) {
                    Some(OtapAnyValueView::Bool(arr.value(row_idx)))
                } else {
                    None
                }
            })
            .unwrap_or(OtapAnyValueView::Empty),
        AttributeValueType::Bytes => anyval
            .attr_bytes
            .as_ref()
            .and_then(|accessor| accessor.slice_at(row_idx))
            .map(OtapAnyValueView::Bytes)
            .unwrap_or(OtapAnyValueView::Empty),
        _ => OtapAnyValueView::Empty,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{
        ArrayRef, DurationNanosecondArray, FixedSizeBinaryArray, Int32Array, StringArray,
        TimestampNanosecondArray, UInt16Array, UInt32Array,
    };
    use arrow::datatypes::{DataType, Field, Schema, TimeUnit};
    use std::sync::Arc;

    /// Helper to create a minimal spans batch for testing
    fn create_test_spans_batch() -> RecordBatch {
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::UInt16, true),
            Field::new(
                "resource",
                DataType::Struct(vec![Field::new("id", DataType::UInt16, false)].into()),
                false,
            ),
            Field::new(
                "scope",
                DataType::Struct(vec![Field::new("id", DataType::UInt16, false)].into()),
                false,
            ),
            Field::new(
                "start_time_unix_nano",
                DataType::Timestamp(TimeUnit::Nanosecond, None),
                false,
            ),
            Field::new(
                "duration_time_unix_nano",
                DataType::Duration(TimeUnit::Nanosecond),
                false,
            ),
            Field::new("trace_id", DataType::FixedSizeBinary(16), false),
            Field::new("span_id", DataType::FixedSizeBinary(8), false),
            Field::new("name", DataType::Utf8, false),
            Field::new("kind", DataType::Int32, true),
            Field::new("dropped_attributes_count", DataType::UInt32, true),
            Field::new("dropped_events_count", DataType::UInt32, true),
            Field::new("dropped_links_count", DataType::UInt32, true),
        ]));

        let id_array = UInt16Array::from(vec![0, 1, 2]);

        let resource_id_array = UInt16Array::from(vec![1, 1, 2]);
        let resource_struct = StructArray::from(vec![(
            Arc::new(Field::new("id", DataType::UInt16, false)),
            Arc::new(resource_id_array) as ArrayRef,
        )]);

        let scope_id_array = UInt16Array::from(vec![10, 10, 20]);
        let scope_struct = StructArray::from(vec![(
            Arc::new(Field::new("id", DataType::UInt16, false)),
            Arc::new(scope_id_array) as ArrayRef,
        )]);

        let start_time =
            TimestampNanosecondArray::from(vec![1_000_000_000, 2_000_000_000, 3_000_000_000]);
        let duration = DurationNanosecondArray::from(vec![100_000, 200_000, 300_000]);

        // Create valid trace IDs (16 bytes each)
        let trace_id_data: Vec<u8> = vec![
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, // trace 1
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, // trace 2 (same)
            2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, // trace 3
        ];
        let trace_id_array =
            FixedSizeBinaryArray::try_from_iter(trace_id_data.chunks(16).map(|c| c.to_vec()))
                .unwrap();

        // Create valid span IDs (8 bytes each)
        let span_id_data: Vec<u8> = vec![
            1, 2, 3, 4, 5, 6, 7, 8, // span 1
            2, 3, 4, 5, 6, 7, 8, 9, // span 2
            3, 4, 5, 6, 7, 8, 9, 10, // span 3
        ];
        let span_id_array =
            FixedSizeBinaryArray::try_from_iter(span_id_data.chunks(8).map(|c| c.to_vec()))
                .unwrap();

        let name_array = StringArray::from(vec!["span-1", "span-2", "span-3"]);
        let kind_array = Int32Array::from(vec![1, 2, 3]); // INTERNAL, SERVER, CLIENT
        let dropped_attrs = UInt32Array::from(vec![0, 0, 0]);
        let dropped_events = UInt32Array::from(vec![0, 0, 0]);
        let dropped_links = UInt32Array::from(vec![0, 0, 0]);

        RecordBatch::try_new(
            schema,
            vec![
                Arc::new(id_array) as ArrayRef,
                Arc::new(resource_struct) as ArrayRef,
                Arc::new(scope_struct) as ArrayRef,
                Arc::new(start_time) as ArrayRef,
                Arc::new(duration) as ArrayRef,
                Arc::new(trace_id_array) as ArrayRef,
                Arc::new(span_id_array) as ArrayRef,
                Arc::new(name_array) as ArrayRef,
                Arc::new(kind_array) as ArrayRef,
                Arc::new(dropped_attrs) as ArrayRef,
                Arc::new(dropped_events) as ArrayRef,
                Arc::new(dropped_links) as ArrayRef,
            ],
        )
        .unwrap()
    }

    #[test]
    fn test_create_otap_traces_view() {
        let spans_batch = create_test_spans_batch();
        let view =
            OtapTracesView::new(&spans_batch, None, None, None, None, None, None, None).unwrap();

        let mut resource_count = 0;
        let mut scope_count = 0;
        let mut span_count = 0;

        for resource_spans in view.resources() {
            resource_count += 1;
            for scope_spans in resource_spans.scopes() {
                scope_count += 1;
                for span in scope_spans.spans() {
                    span_count += 1;

                    // Verify span fields are accessible
                    assert!(span.trace_id().is_some());
                    assert!(span.span_id().is_some());
                    assert!(span.name().is_some());
                    assert!(span.start_time_unix_nano().is_some());
                    assert!(span.end_time_unix_nano().is_some());
                }
            }
        }

        assert_eq!(resource_count, 2, "Should have 2 resource groups");
        assert_eq!(scope_count, 2, "Should have 2 scope groups");
        assert_eq!(span_count, 3, "Should iterate all 3 spans");
    }

    #[test]
    fn test_span_fields() {
        let spans_batch = create_test_spans_batch();
        let view =
            OtapTracesView::new(&spans_batch, None, None, None, None, None, None, None).unwrap();

        let resources: Vec<_> = view.resources().collect();

        // First resource (id=1) should have 2 spans
        let mut span_count = 0;
        for s in resources[0].scopes() {
            for span in s.spans() {
                if span_count == 0 {
                    // Check first span's properties
                    let name = span.name().unwrap();
                    assert_eq!(std::str::from_utf8(name).unwrap(), "span-1");

                    // Check kind
                    assert_eq!(span.kind(), 1); // INTERNAL

                    // Check start time
                    assert_eq!(span.start_time_unix_nano(), Some(1_000_000_000));

                    // Check end time = start + duration
                    assert_eq!(span.end_time_unix_nano(), Some(1_000_000_000 + 100_000));
                }
                span_count += 1;
            }
        }
        assert_eq!(span_count, 2);
    }

    #[test]
    fn test_span_status() {
        // Create a batch with status struct
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::UInt16, true),
            Field::new(
                "resource",
                DataType::Struct(vec![Field::new("id", DataType::UInt16, false)].into()),
                false,
            ),
            Field::new(
                "scope",
                DataType::Struct(vec![Field::new("id", DataType::UInt16, false)].into()),
                false,
            ),
            Field::new(
                "start_time_unix_nano",
                DataType::Timestamp(TimeUnit::Nanosecond, None),
                false,
            ),
            Field::new(
                "status",
                DataType::Struct(
                    vec![
                        Field::new("code", DataType::Int32, true),
                        Field::new("status_message", DataType::Utf8, true),
                    ]
                    .into(),
                ),
                true,
            ),
        ]));

        let id_array = UInt16Array::from(vec![0]);
        let resource_struct = StructArray::from(vec![(
            Arc::new(Field::new("id", DataType::UInt16, false)),
            Arc::new(UInt16Array::from(vec![1])) as ArrayRef,
        )]);
        let scope_struct = StructArray::from(vec![(
            Arc::new(Field::new("id", DataType::UInt16, false)),
            Arc::new(UInt16Array::from(vec![1])) as ArrayRef,
        )]);
        let start_time = TimestampNanosecondArray::from(vec![1_000_000_000]);

        let status_code = Int32Array::from(vec![2]); // ERROR
        let status_message = StringArray::from(vec!["something went wrong"]);
        let status_struct = StructArray::from(vec![
            (
                Arc::new(Field::new("code", DataType::Int32, true)),
                Arc::new(status_code) as ArrayRef,
            ),
            (
                Arc::new(Field::new("status_message", DataType::Utf8, true)),
                Arc::new(status_message) as ArrayRef,
            ),
        ]);

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(id_array) as ArrayRef,
                Arc::new(resource_struct) as ArrayRef,
                Arc::new(scope_struct) as ArrayRef,
                Arc::new(start_time) as ArrayRef,
                Arc::new(status_struct) as ArrayRef,
            ],
        )
        .unwrap();

        let view = OtapTracesView::new(&batch, None, None, None, None, None, None, None).unwrap();

        for resource_spans in view.resources() {
            for scope_spans in resource_spans.scopes() {
                for span in scope_spans.spans() {
                    let status = span.status().expect("Should have status");
                    assert_eq!(status.status_code(), 2); // ERROR
                    let msg = status.message().unwrap();
                    assert_eq!(std::str::from_utf8(msg).unwrap(), "something went wrong");
                }
            }
        }
    }

    #[test]
    fn test_missing_optional_columns() {
        // Create a minimal batch with only required columns
        let schema = Arc::new(Schema::new(vec![
            Field::new(
                "resource",
                DataType::Struct(vec![Field::new("id", DataType::UInt16, false)].into()),
                false,
            ),
            Field::new(
                "scope",
                DataType::Struct(vec![Field::new("id", DataType::UInt16, false)].into()),
                false,
            ),
            Field::new(
                "start_time_unix_nano",
                DataType::Timestamp(TimeUnit::Nanosecond, None),
                false,
            ),
        ]));

        let resource_struct = StructArray::from(vec![(
            Arc::new(Field::new("id", DataType::UInt16, false)),
            Arc::new(UInt16Array::from(vec![1])) as ArrayRef,
        )]);
        let scope_struct = StructArray::from(vec![(
            Arc::new(Field::new("id", DataType::UInt16, false)),
            Arc::new(UInt16Array::from(vec![1])) as ArrayRef,
        )]);
        let start_time = TimestampNanosecondArray::from(vec![1_000_000_000]);

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(resource_struct) as ArrayRef,
                Arc::new(scope_struct) as ArrayRef,
                Arc::new(start_time) as ArrayRef,
            ],
        )
        .unwrap();

        let view = OtapTracesView::new(&batch, None, None, None, None, None, None, None).unwrap();

        for resource_spans in view.resources() {
            for scope_spans in resource_spans.scopes() {
                for span in scope_spans.spans() {
                    // All optional fields should return None/default
                    assert!(span.trace_id().is_none());
                    assert!(span.span_id().is_none());
                    assert!(span.trace_state().is_none());
                    assert!(span.parent_span_id().is_none());
                    assert!(span.flags().is_none());
                    assert!(span.name().is_none());
                    assert_eq!(span.kind(), 0);
                    assert!(span.end_time_unix_nano().is_none());
                    assert_eq!(span.dropped_attributes_count(), 0);
                    assert_eq!(span.dropped_events_count(), 0);
                    assert_eq!(span.dropped_links_count(), 0);
                    assert!(span.status().is_none());
                    assert_eq!(span.attributes().count(), 0);
                    assert_eq!(span.events().count(), 0);
                    assert_eq!(span.links().count(), 0);
                }
            }
        }
    }

    #[test]
    fn test_events_iteration() {
        let spans_batch = create_test_spans_batch();

        // Create events batch linked to span id=0
        let events_schema = Arc::new(Schema::new(vec![
            Field::new("parent_id", DataType::UInt16, false),
            Field::new(
                "time_unix_nano",
                DataType::Timestamp(TimeUnit::Nanosecond, None),
                true,
            ),
            Field::new("name", DataType::Utf8, true),
        ]));

        let events_batch = RecordBatch::try_new(
            events_schema,
            vec![
                Arc::new(UInt16Array::from(vec![0, 0, 1])) as ArrayRef, // 2 events for span 0, 1 for span 1
                Arc::new(TimestampNanosecondArray::from(vec![
                    1_100_000, 1_200_000, 2_100_000,
                ])) as ArrayRef,
                Arc::new(StringArray::from(vec!["event-a", "event-b", "event-c"])) as ArrayRef,
            ],
        )
        .unwrap();

        let view = OtapTracesView::new(
            &spans_batch,
            None,
            None,
            None,
            Some(&events_batch),
            None,
            None,
            None,
        )
        .unwrap();

        let mut total_events = 0;
        for resource_spans in view.resources() {
            for scope_spans in resource_spans.scopes() {
                for span in scope_spans.spans() {
                    for event in span.events() {
                        total_events += 1;
                        assert!(event.name().is_some());
                        assert!(event.time_unix_nano().is_some());
                    }
                }
            }
        }

        assert_eq!(total_events, 3, "Should iterate all 3 events");
    }

    #[test]
    fn test_otap_traces_view_integration() {
        let spans_batch = create_test_spans_batch();
        let resource_attrs = None; // we could build complex attrs but basics are fine
        let scope_attrs = None;

        let events_schema = Arc::new(Schema::new(vec![
            Field::new("parent_id", DataType::UInt16, false),
            Field::new(
                "time_unix_nano",
                DataType::Timestamp(TimeUnit::Nanosecond, None),
                true,
            ),
            Field::new("name", DataType::Utf8, true),
        ]));

        let events_batch = RecordBatch::try_new(
            events_schema.clone(),
            vec![
                Arc::new(UInt16Array::from(vec![0])) as ArrayRef,
                Arc::new(TimestampNanosecondArray::from(vec![1_100_000])) as ArrayRef,
                Arc::new(StringArray::from(vec!["integration-event"])) as ArrayRef,
            ],
        )
        .unwrap();

        let links_schema = Arc::new(Schema::new(vec![
            Field::new("parent_id", DataType::UInt16, false),
            Field::new("trace_id", DataType::FixedSizeBinary(16), true),
            Field::new("span_id", DataType::FixedSizeBinary(8), true),
        ]));

        let links_batch = RecordBatch::try_new(
            links_schema.clone(),
            vec![
                Arc::new(UInt16Array::from(vec![0])) as ArrayRef,
                Arc::new(
                    FixedSizeBinaryArray::try_from_sparse_iter_with_size(
                        vec![Some(vec![1; 16])],
                        16,
                    )
                    .unwrap(),
                ) as ArrayRef,
                Arc::new(
                    FixedSizeBinaryArray::try_from_sparse_iter_with_size(vec![Some(vec![2; 8])], 8)
                        .unwrap(),
                ) as ArrayRef,
            ],
        )
        .unwrap();

        let view = OtapTracesView::new(
            &spans_batch,
            resource_attrs,
            scope_attrs,
            None,
            Some(&events_batch),
            None,
            Some(&links_batch),
            None,
        )
        .expect("Failed to create unified integration view");

        let mut span_count = 0;
        let mut event_count = 0;
        let mut link_count = 0;
        for resource in view.resources() {
            assert!(resource.schema_url().is_none()); // because our test default has none
            for scope in resource.scopes() {
                assert!(scope.schema_url().is_none());
                for span in scope.spans() {
                    span_count += 1;
                    event_count += span.events().count();
                    link_count += span.links().count();
                    assert!(span.span_id().is_some());
                }
            }
        }

        assert_eq!(span_count, 3);
        assert_eq!(event_count, 1);
        assert_eq!(link_count, 1);
    }
}
