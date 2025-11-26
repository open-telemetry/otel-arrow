// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Zero-copy view implementation for OTAP Arrow RecordBatches (Logs)
//!
//! This module provides a direct view over OTAP Arrow columnar data without any conversion
//! to OTLP bytes or Prost messages, enabling true zero-copy iteration.

use std::collections::BTreeMap;

use arrow::array::{Array, RecordBatch, StructArray, UInt16Array};

#[cfg(test)]
use arrow::array::{TimestampNanosecondArray, UInt32Array};

use crate::arrays::{MaybeDictArrayAccessor, NullableArrayAccessor};
use crate::error::Error;
use crate::otap::OtapArrowRecords;
use crate::otlp::attributes::{Attribute16Arrays, AttributeValueType};
use crate::otlp::logs::{LogBodyArrays, LogsArrays};
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use crate::schema::consts;
use crate::schema::{SpanId, TraceId};
use crate::views::common::{AnyValueView, AttributeView, InstrumentationScopeView, Str, ValueType};
use crate::views::logs::{LogRecordView, LogsDataView, ResourceLogsView, ScopeLogsView};
use crate::views::resource::ResourceView;

use std::ops::Range;

/// Represents a group of rows, either contiguous or scattered.
///
/// When rows are contiguous (common case), we use a Range to avoid allocating a Vec.
/// When rows are scattered, we store the actual indices.
#[derive(Debug, Clone)]
pub(crate) enum RowGroup {
    Contiguous(Range<usize>),
    Scattered(Vec<usize>),
}

impl RowGroup {
    /// Returns an iterator over the indices in this group
    #[must_use]
    pub fn iter(&self) -> RowGroupIter<'_> {
        match self {
            RowGroup::Contiguous(range) => RowGroupIter::Contiguous(range.clone()),
            RowGroup::Scattered(indices) => RowGroupIter::Scattered(indices.iter()),
        }
    }

    /// Returns the number of rows in the group
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        match self {
            RowGroup::Contiguous(range) => range.len(),
            RowGroup::Scattered(indices) => indices.len(),
        }
    }

    /// Returns true if the group is empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Iterator over indices in a RowGroup
pub(crate) enum RowGroupIter<'a> {
    Contiguous(Range<usize>),
    Scattered(std::slice::Iter<'a, usize>),
}

impl<'a> Iterator for RowGroupIter<'a> {
    type Item = usize;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            RowGroupIter::Contiguous(range) => range.next(),
            RowGroupIter::Scattered(iter) => iter.next().copied(),
        }
    }
}

/// Zero-copy view over OTAP logs Arrow RecordBatches
pub struct OtapLogsView<'a> {
    columns: LogsArrays<'a>,                       // ~220 bytes
    resource_attrs: Option<Attribute16Arrays<'a>>, // ~168 bytes when present
    scope_attrs: Option<Attribute16Arrays<'a>>,    // ~168 bytes when present
    log_attrs: Option<Attribute16Arrays<'a>>,      // ~168 bytes when present

    // Pre-computed indices for hierarchy reconstruction and attribute matching.
    //
    // These are built once at construction to avoid O(N*M) scans during iteration.
    // Without these, we'd need to scan the entire logs batch for each resource/scope
    // to find matching rows, and scan all attributes for each log record.
    //
    // Memory allocation: Stores IDs and row indices only, not actual data.
    // Example: 1000 logs with 4 log attributes each allocates ~50 KB for these
    // index structures (resource/scope attributes add negligibly since there are
    // typically only 1-10 resources/scopes). The actual attribute strings (~200 KB)
    // remain in the Arrow RecordBatch (zero-copy).
    //
    // TODO: Consider HashMap instead of BTreeMap for log_attrs_map when log count
    // is large (>1000).
    //
    // Current choice (BTreeMap): Optimizes for memory efficiency over lookup speed.
    // HashMap requires extra capacity beyond the actual entry count to maintain low
    // collision rates, which adds overhead when multiplied across many batches in a
    // high-performance pipeline. BTreeMap stores data more compactly. For resource/
    // scope maps (1-10 entries), this is clearly better. For log_attrs_map, we trade
    // O(log n) lookups for lower memory, where O(log n) on u16 keys is still fast.
    //
    // Future consideration: If profiling shows (1) batches consistently have 1000+
    // logs, AND (2) attribute lookup is a bottleneck, then HashMap's O(1) might
    // justify the memory cost. Requires measuring typical log count and attributes-
    // per-log in production workloads.
    //
    // Resource ID -> List of rows (RowGroup)
    resource_groups: Vec<(u16, RowGroup)>,
    // Resource ID -> List of (Scope ID, RowGroup)
    scope_groups_map: BTreeMap<u16, Vec<(u16, RowGroup)>>,
    // Parent ID -> List of attribute row indices (for O(log n) attribute lookup)
    resource_attrs_map: BTreeMap<u16, Vec<usize>>,
    scope_attrs_map: BTreeMap<u16, Vec<usize>>,
    log_attrs_map: BTreeMap<u16, Vec<usize>>,
}

impl<'a> OtapLogsView<'a> {
    /// Create a new OTAP logs view from Arrow RecordBatches
    #[inline]
    pub(crate) fn new(
        logs_batch: &'a RecordBatch,
        resource_attrs: Option<&'a RecordBatch>,
        scope_attrs: Option<&'a RecordBatch>,
        log_attrs: Option<&'a RecordBatch>,
    ) -> Result<Self, Error> {
        // 1. Cache columns for O(1) access
        let columns = LogsArrays::try_from(logs_batch)?;

        // 2. Pre-compute resource grouping
        let resource_groups = group_by_resource_id(logs_batch);

        // 3. Pre-compute scope grouping per resource
        let mut scope_groups_map = BTreeMap::new();
        for (resource_id, row_indices) in &resource_groups {
            let scope_groups = group_by_scope_id(logs_batch, row_indices);
            let _ = scope_groups_map.insert(*resource_id, scope_groups);
        }

        // 4. Pre-compute attribute mappings
        let resource_attrs_map = if let Some(batch) = resource_attrs {
            build_attribute_index(batch)
        } else {
            BTreeMap::new()
        };

        let scope_attrs_map = if let Some(batch) = scope_attrs {
            build_attribute_index(batch)
        } else {
            BTreeMap::new()
        };

        let log_attrs_map = if let Some(batch) = log_attrs {
            build_attribute_index(batch)
        } else {
            BTreeMap::new()
        };

        Ok(Self {
            columns,
            resource_attrs: resource_attrs
                .map(Attribute16Arrays::try_from)
                .transpose()?,
            scope_attrs: scope_attrs.map(Attribute16Arrays::try_from).transpose()?,
            log_attrs: log_attrs.map(Attribute16Arrays::try_from).transpose()?,
            resource_groups,
            scope_groups_map,
            resource_attrs_map,
            scope_attrs_map,
            log_attrs_map,
        })
    }
}

impl<'a> TryFrom<&'a OtapArrowRecords> for OtapLogsView<'a> {
    type Error = Error;

    fn try_from(records: &'a OtapArrowRecords) -> Result<Self, Self::Error> {
        let logs_batch = records
            .get(ArrowPayloadType::Logs)
            .ok_or(Error::RecordBatchNotFound {
                payload_type: ArrowPayloadType::Logs,
            })?;

        let resource_attrs = records.get(ArrowPayloadType::ResourceAttrs);
        let scope_attrs = records.get(ArrowPayloadType::ScopeAttrs);
        let log_attrs = records.get(ArrowPayloadType::LogAttrs);

        Self::new(logs_batch, resource_attrs, scope_attrs, log_attrs)
    }
}

impl<'a> LogsDataView for OtapLogsView<'a> {
    type ResourceLogs<'res>
        = OtapResourceLogsView<'res>
    where
        Self: 'res;

    type ResourcesIter<'res>
        = OtapResourceLogsIter<'res>
    where
        Self: 'res;

    #[inline]
    fn resources(&self) -> Self::ResourcesIter<'_> {
        OtapResourceLogsIter::new(self)
    }
}

/// Iterator over distinct resource groups in OTAP logs
pub struct OtapResourceLogsIter<'a> {
    view: &'a OtapLogsView<'a>,
    resource_groups: std::slice::Iter<'a, (u16, RowGroup)>,
}

impl<'a> OtapResourceLogsIter<'a> {
    #[inline]
    fn new(view: &'a OtapLogsView<'a>) -> Self {
        // Iterate over pre-computed resource groups stored in the view
        Self {
            view,
            resource_groups: view.resource_groups.iter(),
        }
    }
}

impl<'a> Iterator for OtapResourceLogsIter<'a> {
    type Item = OtapResourceLogsView<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let (resource_id, row_indices) = self.resource_groups.next()?;

        Some(OtapResourceLogsView {
            view: self.view,
            resource_id: *resource_id,
            row_indices,
        })
    }
}

/// View of a single ResourceLogs in OTAP format
pub struct OtapResourceLogsView<'a> {
    view: &'a OtapLogsView<'a>,
    resource_id: u16,
    #[allow(dead_code)]
    row_indices: &'a RowGroup,
}

impl<'a> ResourceLogsView for OtapResourceLogsView<'a> {
    type Resource<'res>
        = OtapResourceView<'res>
    where
        Self: 'res;

    type ScopeLogs<'scp>
        = OtapScopeLogsView<'scp>
    where
        Self: 'scp;

    type ScopesIter<'scp>
        = OtapScopeLogsIter<'scp>
    where
        Self: 'scp;

    #[inline]
    fn resource(&self) -> Option<Self::Resource<'_>> {
        Some(OtapResourceView {
            view: self.view,
            resource_id: self.resource_id,
        })
    }

    #[inline]
    fn scopes(&self) -> Self::ScopesIter<'_> {
        OtapScopeLogsIter::new(self)
    }

    #[inline]
    fn schema_url(&self) -> Option<Str<'_>> {
        // TODO: Implement schema_url lookup from logs batch
        None
    }
}

/// Iterator over scope logs within a resource
pub struct OtapScopeLogsIter<'a> {
    view: &'a OtapLogsView<'a>,
    scope_groups: std::slice::Iter<'a, (u16, RowGroup)>,
}

impl<'a> OtapScopeLogsIter<'a> {
    #[inline]
    fn new(resource_view: &'a OtapResourceLogsView<'a>) -> Self {
        // Use pre-computed scope groups for this resource
        let scope_groups = resource_view
            .view
            .scope_groups_map
            .get(&resource_view.resource_id)
            .map(|v| v.iter())
            .unwrap_or_default();

        Self {
            view: resource_view.view,
            scope_groups,
        }
    }
}

impl<'a> Iterator for OtapScopeLogsIter<'a> {
    type Item = OtapScopeLogsView<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let (scope_id, row_indices) = self.scope_groups.next()?;

        Some(OtapScopeLogsView {
            view: self.view,
            scope_id: *scope_id,
            row_indices,
        })
    }
}

/// View of a single ScopeLogs in OTAP format
pub struct OtapScopeLogsView<'a> {
    view: &'a OtapLogsView<'a>,
    scope_id: u16,
    row_indices: &'a RowGroup, // Rows belonging to this scope
}

impl<'a> ScopeLogsView for OtapScopeLogsView<'a> {
    type Scope<'scp>
        = OtapInstrumentationScopeView<'scp>
    where
        Self: 'scp;

    type LogRecord<'rec>
        = OtapLogRecordView<'rec>
    where
        Self: 'rec;

    type LogRecordsIter<'rec>
        = OtapLogRecordsIter<'rec>
    where
        Self: 'rec;

    #[inline]
    fn scope(&self) -> Option<Self::Scope<'_>> {
        Some(OtapInstrumentationScopeView {
            view: self.view,
            scope_id: self.scope_id,
        })
    }

    #[inline]
    fn log_records(&self) -> Self::LogRecordsIter<'_> {
        OtapLogRecordsIter::new(self.view, self.row_indices)
    }

    #[inline]
    fn schema_url(&self) -> Option<Str<'_>> {
        // get_schema_url_for_scope(self.view.logs_batch, self.scope_id)
        // TODO: Implement schema_url lookup for scope
        None
    }
}

/// Iterator over log records within a scope
pub struct OtapLogRecordsIter<'a> {
    view: &'a OtapLogsView<'a>,
    row_indices: RowGroupIter<'a>,
}

impl<'a> OtapLogRecordsIter<'a> {
    #[inline]
    fn new(view: &'a OtapLogsView<'a>, row_indices: &'a RowGroup) -> Self {
        Self {
            view,
            row_indices: row_indices.iter(),
        }
    }
}

impl<'a> Iterator for OtapLogRecordsIter<'a> {
    type Item = OtapLogRecordView<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let row_idx = self.row_indices.next()?;

        Some(OtapLogRecordView {
            view: self.view,
            row_idx,
        })
    }
}

/// View of a single LogRecord in OTAP format
pub struct OtapLogRecordView<'a> {
    view: &'a OtapLogsView<'a>,
    row_idx: usize,
}

impl<'a> LogRecordView for OtapLogRecordView<'a> {
    type Attribute<'att>
        = OtapAttributeView<'att>
    where
        Self: 'att;

    type AttributeIter<'att>
        = OtapAttributeIter<'att>
    where
        Self: 'att;

    type Body<'bod>
        = OtapAnyValueView<'bod>
    where
        Self: 'bod;

    #[inline]
    fn time_unix_nano(&self) -> Option<u64> {
        self.view.columns.time_unix_nano.as_ref().and_then(|col| {
            if col.is_valid(self.row_idx) {
                Some(col.value(self.row_idx) as u64)
            } else {
                None
            }
        })
    }

    #[inline]
    fn observed_time_unix_nano(&self) -> Option<u64> {
        self.view
            .columns
            .observed_time_unix_nano
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
    fn severity_number(&self) -> Option<i32> {
        self.view
            .columns
            .severity_number
            .as_ref()
            .and_then(|col| col.value_at(self.row_idx))
    }

    #[inline]
    fn severity_text(&self) -> Option<Str<'_>> {
        self.view
            .columns
            .severity_text
            .as_ref()
            .and_then(|col| col.str_at(self.row_idx).map(|s| s.as_bytes()))
    }

    #[inline]
    fn body(&self) -> Option<Self::Body<'_>> {
        // Extract body value from the cached body columns
        self.view
            .columns
            .body
            .as_ref()
            .and_then(|cols| get_body_from_struct(cols, self.row_idx))
    }

    #[inline]
    fn attributes(&self) -> Self::AttributeIter<'_> {
        OtapAttributeIter::new(self.view, self.row_idx)
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
    fn trace_id(&self) -> Option<&TraceId> {
        self.view.columns.trace_id.as_ref().and_then(|col| {
            col.slice_at(self.row_idx)
                .and_then(|bytes| bytes.try_into().ok())
        })
    }

    #[inline]
    fn span_id(&self) -> Option<&SpanId> {
        self.view.columns.span_id.as_ref().and_then(|col| {
            col.slice_at(self.row_idx)
                .and_then(|bytes| bytes.try_into().ok())
        })
    }

    #[inline]
    fn event_name(&self) -> Option<Str<'_>> {
        self.view
            .columns
            .event_name
            .as_ref()
            .and_then(|col| col.str_at(self.row_idx).map(|s| s.as_bytes()))
    }
}

// ===== Resource and Scope Views =====

/// View of a Resource in OTAP format
pub struct OtapResourceView<'a> {
    view: &'a OtapLogsView<'a>,
    resource_id: u16,
}

impl<'a> ResourceView for OtapResourceView<'a> {
    type Attribute<'att>
        = OtapAttributeView<'att>
    where
        Self: 'att;

    type AttributesIter<'att>
        = OtapAttributeIter<'att>
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

        OtapAttributeIter {
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

/// View of an InstrumentationScope in OTAP format
pub struct OtapInstrumentationScopeView<'a> {
    view: &'a OtapLogsView<'a>,
    scope_id: u16,
}

impl<'a> InstrumentationScopeView for OtapInstrumentationScopeView<'a> {
    type Attribute<'att>
        = OtapAttributeView<'att>
    where
        Self: 'att;

    type AttributeIter<'att>
        = OtapAttributeIter<'att>
    where
        Self: 'att;

    #[inline]
    fn name(&self) -> Option<Str<'_>> {
        None // TODO: implement - get from scope table
    }

    #[inline]
    fn version(&self) -> Option<Str<'_>> {
        None // TODO: implement - get from scope table
    }

    #[inline]
    fn attributes(&self) -> Self::AttributeIter<'_> {
        let matching_rows = self
            .view
            .scope_attrs_map
            .get(&self.scope_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        OtapAttributeIter {
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

// ===== Attribute Views =====

/// Attribute view for OTAP format
pub struct OtapAttributeView<'a> {
    key: &'a [u8],
    value: OtapAnyValueView<'a>,
}

impl<'a> AttributeView for OtapAttributeView<'a> {
    type Val<'val>
        = OtapAnyValueView<'val>
    where
        Self: 'val;

    #[inline]
    fn key(&self) -> Str<'_> {
        self.key
    }

    #[inline]
    fn value(&self) -> Option<Self::Val<'_>> {
        Some(self.value)
    }
}

/// AnyValue view for OTAP format
#[derive(Copy, Clone, Debug)]
pub enum OtapAnyValueView<'a> {
    /// Empty/null value
    Empty,
    /// String value (UTF-8 bytes)
    Str(&'a [u8]),
    /// Integer value (64-bit signed)
    Int(i64),
    /// Double-precision floating point value
    Double(f64),
    /// Boolean value
    Bool(bool),
    /// Raw bytes value
    Bytes(&'a [u8]),
    // TODO: Add Array and Map variants when needed
}

impl<'a> AnyValueView<'a> for OtapAnyValueView<'a> {
    type KeyValue = OtapAttributeView<'a>;
    type ArrayIter<'arr>
        = std::iter::Empty<Self>
    where
        Self: 'arr;
    type KeyValueIter<'kv>
        = std::iter::Empty<Self::KeyValue>
    where
        Self: 'kv;

    fn value_type(&self) -> ValueType {
        match self {
            Self::Empty => ValueType::Empty,
            Self::Str(_) => ValueType::String,
            Self::Int(_) => ValueType::Int64,
            Self::Double(_) => ValueType::Double,
            Self::Bool(_) => ValueType::Bool,
            Self::Bytes(_) => ValueType::Bytes,
        }
    }

    fn as_string(&self) -> Option<Str<'a>> {
        match self {
            Self::Str(s) => Some(*s),
            _ => None,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    fn as_int64(&self) -> Option<i64> {
        match self {
            Self::Int(i) => Some(*i),
            _ => None,
        }
    }

    fn as_double(&self) -> Option<f64> {
        match self {
            Self::Double(d) => Some(*d),
            _ => None,
        }
    }

    fn as_bytes(&self) -> Option<&'a [u8]> {
        match self {
            Self::Bytes(b) => Some(*b),
            _ => None,
        }
    }

    fn as_array(&self) -> Option<Self::ArrayIter<'_>> {
        // TODO: Implement array iteration from CBOR-serialized data
        None
    }

    fn as_kvlist(&self) -> Option<Self::KeyValueIter<'_>> {
        // TODO: Implement map iteration from CBOR-serialized data
        None
    }
}

// ===== Attribute Iterators =====

/// Iterator over log record attributes
/// Generic iterator over attributes (used for log, resource, and scope attributes)
pub struct OtapAttributeIter<'a> {
    attrs: Option<&'a Attribute16Arrays<'a>>,
    matching_rows: &'a [usize],
    current_idx: usize,
}

impl<'a> OtapAttributeIter<'a> {
    #[inline]
    fn new(view: &'a OtapLogsView<'a>, log_row_idx: usize) -> Self {
        // Get log ID from logs batch
        let log_id = get_log_id(view.columns.id, log_row_idx);

        let matching_rows = if let Some(id) = log_id {
            view.log_attrs_map
                .get(&id)
                .map(|v| v.as_slice())
                .unwrap_or(&[])
        } else {
            &[]
        };

        Self {
            attrs: view.log_attrs.as_ref(),
            matching_rows,
            current_idx: 0,
        }
    }
}

impl<'a> Iterator for OtapAttributeIter<'a> {
    type Item = OtapAttributeView<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let attrs_cols = self.attrs?;

        // Loop to skip invalid attributes (e.g., null keys) instead of terminating iteration
        loop {
            if self.current_idx >= self.matching_rows.len() {
                return None;
            }

            let attr_row_idx = self.matching_rows[self.current_idx];
            self.current_idx += 1;

            // If key is invalid (null), skip this attribute and continue to next
            if let Some(key) = get_attribute_key(attrs_cols, attr_row_idx) {
                let value = get_attribute_value(attrs_cols, attr_row_idx);
                return Some(OtapAttributeView { key, value });
            }
            // If key was None, loop continues to next attribute
        }
    }
}

// ===== Helper Functions =====

/// Build an inverted index from parent_id to list of attribute row indices
fn build_attribute_index(batch: &RecordBatch) -> BTreeMap<u16, Vec<usize>> {
    let parent_id_col = match batch.column_by_name(consts::PARENT_ID) {
        Some(col) => col,
        None => return BTreeMap::new(),
    };

    let mut index: BTreeMap<u16, Vec<usize>> = BTreeMap::new();

    // Handle dictionary-encoded parent_id (common in OTAP)
    if let Ok(accessor) = MaybeDictArrayAccessor::<UInt16Array>::try_new(parent_id_col) {
        for i in 0..batch.num_rows() {
            if let Some(pid) = accessor.value_at(i) {
                index.entry(pid).or_default().push(i);
            }
        }
        return index;
    }

    // Handle direct UInt16 array
    if let Some(parent_id_array) = parent_id_col.as_any().downcast_ref::<UInt16Array>() {
        for i in 0..batch.num_rows() {
            if parent_id_array.is_valid(i) {
                let pid = parent_id_array.value(i);
                index.entry(pid).or_default().push(i);
            }
        }
    }

    index
}

/// Group logs by resource ID
fn group_by_resource_id(logs_batch: &RecordBatch) -> Vec<(u16, RowGroup)> {
    let num_rows = logs_batch.num_rows();
    if num_rows == 0 {
        return Vec::new();
    }

    let resource_col = match logs_batch.column_by_name(consts::RESOURCE) {
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

    // Optimized grouping:
    // We iterate through the rows and build groups.
    // Since we iterate in order (0..num_rows), we can detect contiguous ranges easily.
    // We use a BTreeMap to map ID -> GroupBuilder to handle interleaved resources if any.

    enum GroupBuilder {
        Contiguous { start: usize, count: usize },
        Scattered(Vec<usize>),
    }

    let mut builders: BTreeMap<u16, GroupBuilder> = BTreeMap::new();

    // Handle dictionary-encoded resource.id (common in OTAP for repeated values)
    if let Ok(accessor) = MaybeDictArrayAccessor::<UInt16Array>::try_new(id_col) {
        for i in 0..num_rows {
            if let Some(id) = accessor.value_at(i) {
                let _ = builders
                    .entry(id)
                    .and_modify(|builder| match builder {
                        GroupBuilder::Contiguous { start, count } => {
                            if *start + *count == i {
                                // Still contiguous
                                *count += 1;
                            } else {
                                // Break in continuity, switch to Scattered
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
        // Fallback: Handle direct UInt16Array (not dictionary-encoded)
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
        // Unsupported array type
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

/// Helper to group logs by scope ID within a set of rows
fn group_by_scope_id(logs_batch: &RecordBatch, row_indices: &RowGroup) -> Vec<(u16, RowGroup)> {
    let scope_col = match logs_batch.column_by_name(consts::SCOPE) {
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

    // Handle dictionary-encoded scope.id (common in OTAP for repeated values)
    if let Ok(accessor) = MaybeDictArrayAccessor::<UInt16Array>::try_new(id_col) {
        // Iterate over the parent group's indices
        for row_idx in row_indices.iter() {
            if let Some(scope_id) = accessor.value_at(row_idx) {
                let _ = builders
                    .entry(scope_id)
                    .and_modify(|builder| match builder {
                        GroupBuilder::Contiguous { start, count } => {
                            if *start + *count == row_idx {
                                // Still contiguous
                                *count += 1;
                            } else {
                                // Break in continuity, switch to Scattered
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
        // Fallback: Handle direct UInt16Array (not dictionary-encoded)
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
        // Unsupported array type
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

// Unused helpers removed

fn get_body_from_struct<'a>(
    cols: &'a LogBodyArrays<'a>,
    row_idx: usize,
) -> Option<OtapAnyValueView<'a>> {
    let anyval = &cols.anyval_arrays;
    let type_array = &anyval.attr_type;

    if !type_array.is_valid(row_idx) {
        return Some(OtapAnyValueView::Empty);
    }

    let value_type = AttributeValueType::try_from(type_array.value(row_idx)).ok()?;

    // Extract the appropriate value field based on type
    match value_type {
        AttributeValueType::Str => Some(
            anyval
                .attr_str
                .as_ref()
                .and_then(|accessor| accessor.str_at(row_idx))
                .map(|s| OtapAnyValueView::Str(s.as_bytes()))
                .unwrap_or(OtapAnyValueView::Empty),
        ),
        AttributeValueType::Int => Some(
            anyval
                .attr_int
                .as_ref()
                .and_then(|accessor| accessor.value_at(row_idx))
                .map(OtapAnyValueView::Int)
                .unwrap_or(OtapAnyValueView::Empty),
        ),
        AttributeValueType::Double => Some(
            anyval
                .attr_double
                .and_then(|arr| {
                    if arr.is_valid(row_idx) {
                        Some(OtapAnyValueView::Double(arr.value(row_idx)))
                    } else {
                        None
                    }
                })
                .unwrap_or(OtapAnyValueView::Empty),
        ),
        AttributeValueType::Bool => Some(
            anyval
                .attr_bool
                .and_then(|arr| {
                    if arr.is_valid(row_idx) {
                        Some(OtapAnyValueView::Bool(arr.value(row_idx)))
                    } else {
                        None
                    }
                })
                .unwrap_or(OtapAnyValueView::Empty),
        ),
        AttributeValueType::Bytes => Some(
            anyval
                .attr_bytes
                .as_ref()
                .and_then(|accessor| accessor.slice_at(row_idx))
                .map(OtapAnyValueView::Bytes)
                .unwrap_or(OtapAnyValueView::Empty),
        ),
        _ => {
            // For other types (Map, Slice), return empty for now
            Some(OtapAnyValueView::Empty)
        }
    }
}

/// Helper to get log ID from logs batch for attribute matching
fn get_log_id(id_array: Option<&UInt16Array>, row_idx: usize) -> Option<u16> {
    let array = id_array?;
    if array.is_valid(row_idx) {
        Some(array.value(row_idx))
    } else {
        None
    }
}

/// Extract attribute key from an attribute batch row
fn get_attribute_key<'a>(cols: &'a Attribute16Arrays<'a>, row_idx: usize) -> Option<&'a [u8]> {
    cols.attr_key.str_at(row_idx).map(|s| s.as_bytes())
}

/// Extract attribute value with type information from an attribute batch row
fn get_attribute_value<'a>(
    cols: &'a Attribute16Arrays<'a>,
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

    // Extract the appropriate value based on type
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

#[allow(dead_code)]
fn get_schema_url_for_resource<'a>(_batch: &'a RecordBatch, _resource_id: u16) -> Option<Str<'a>> {
    // TODO: Implement schema_url lookup for resource
    // Would need to look up in a separate resource table or nested struct
    None
}

#[allow(dead_code)]
fn get_schema_url_for_scope<'a>(_batch: &'a RecordBatch, _scope_id: u16) -> Option<Str<'a>> {
    // TODO: Implement schema_url lookup for scope
    // Would need to look up in a separate scope table or nested struct
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{ArrayRef, Int32Array, Int64Array, StringArray, UInt8Array, UInt16Array};
    use arrow::datatypes::{DataType, Field, Schema, TimeUnit};
    use std::sync::Arc;

    /// Helper to create a logs batch with optional ID column
    fn create_test_logs_batch_impl(include_id: bool) -> RecordBatch {
        // Define schema matching OTAP logs structure
        let resource_field = Field::new(
            "resource",
            DataType::Struct(vec![Field::new("id", DataType::UInt16, false)].into()),
            false,
        );

        let scope_field = Field::new(
            "scope",
            DataType::Struct(vec![Field::new("id", DataType::UInt16, false)].into()),
            false,
        );

        // Build schema conditionally
        let mut fields = Vec::new();
        if include_id {
            fields.push(Field::new("id", DataType::UInt16, false));
        }
        fields.extend([
            resource_field,
            scope_field,
            Field::new(
                "time_unix_nano",
                DataType::Timestamp(TimeUnit::Nanosecond, None),
                true,
            ),
            Field::new(
                "observed_time_unix_nano",
                DataType::Timestamp(TimeUnit::Nanosecond, None),
                true,
            ),
            Field::new("severity_number", DataType::Int32, true),
            Field::new("severity_text", DataType::Utf8, true),
            Field::new(
                "body",
                DataType::Struct(
                    vec![
                        Field::new("type", DataType::UInt8, false),
                        Field::new("str", DataType::Utf8, true),
                    ]
                    .into(),
                ),
                true,
            ),
            Field::new("flags", DataType::UInt32, true),
            Field::new("event_name", DataType::Utf8, true),
        ]);

        let schema = Arc::new(Schema::new(fields));

        // Create test data (3 log records)
        let id_array = UInt16Array::from(vec![1, 2, 3]);

        // Resource structs (all from resource_id=1)
        let resource_id_array = UInt16Array::from(vec![1, 1, 1]);
        let resource_struct = StructArray::from(vec![(
            Arc::new(Field::new("id", DataType::UInt16, false)),
            Arc::new(resource_id_array) as ArrayRef,
        )]);

        // Scope structs (logs 1-2 from scope_id=10, log 3 from scope_id=11)
        let scope_id_array = UInt16Array::from(vec![10, 10, 11]);
        let scope_struct = StructArray::from(vec![(
            Arc::new(Field::new("id", DataType::UInt16, false)),
            Arc::new(scope_id_array) as ArrayRef,
        )]);

        let time_array = TimestampNanosecondArray::from(vec![
            Some(1000000000),
            Some(2000000000),
            Some(3000000000),
        ]);

        let observed_time_array = TimestampNanosecondArray::from(vec![
            Some(1000000100),
            Some(2000000100),
            Some(3000000100),
        ]);

        let severity_array = Int32Array::from(vec![Some(9), Some(17), Some(13)]); // INFO, ERROR, WARN
        let severity_text_array =
            StringArray::from(vec![Some("INFO"), Some("ERROR"), Some("WARN")]);

        // Body struct (mimicking AnyValue)
        let body_type_array = UInt8Array::from(vec![1, 1, 1]); // 1 = Str
        let body_str_array = StringArray::from(vec![
            Some("Log message 1"),
            Some("Error occurred"),
            Some("Warning message"),
        ]);

        let body_struct = StructArray::from(vec![
            (
                Arc::new(Field::new("type", DataType::UInt8, false)),
                Arc::new(body_type_array) as ArrayRef,
            ),
            (
                Arc::new(Field::new("str", DataType::Utf8, true)),
                Arc::new(body_str_array) as ArrayRef,
            ),
        ]);

        let flags_array = UInt32Array::from(vec![Some(1), Some(1), Some(0)]);
        let event_name_array =
            StringArray::from(vec![Some("event1"), Some("event2"), Some("event3")]);

        // Build columns conditionally
        let mut columns: Vec<ArrayRef> = Vec::new();
        if include_id {
            columns.push(Arc::new(id_array) as ArrayRef);
        }
        columns.extend([
            Arc::new(resource_struct) as ArrayRef,
            Arc::new(scope_struct) as ArrayRef,
            Arc::new(time_array) as ArrayRef,
            Arc::new(observed_time_array) as ArrayRef,
            Arc::new(severity_array) as ArrayRef,
            Arc::new(severity_text_array) as ArrayRef,
            Arc::new(body_struct) as ArrayRef,
            Arc::new(flags_array) as ArrayRef,
            Arc::new(event_name_array) as ArrayRef,
        ]);

        RecordBatch::try_new(schema, columns).unwrap()
    }

    fn create_test_logs_batch() -> RecordBatch {
        create_test_logs_batch_impl(true)
    }

    /// Helper to create resource attributes batch
    fn create_test_resource_attrs() -> RecordBatch {
        let schema = Arc::new(Schema::new(vec![
            Field::new("parent_id", DataType::UInt16, false),
            Field::new("key", DataType::Utf8, false),
            Field::new("type", DataType::UInt8, false),
            Field::new("str", DataType::Utf8, true),
        ]));

        // 2 attributes for resource_id=1
        let parent_id = UInt16Array::from(vec![1, 1]);
        let keys = StringArray::from(vec!["service.name", "host.name"]);
        let types = UInt8Array::from(vec![1, 1]); // Str type
        let str_values = StringArray::from(vec![Some("test-service"), Some("test-host")]);

        RecordBatch::try_new(
            schema,
            vec![
                Arc::new(parent_id),
                Arc::new(keys),
                Arc::new(types),
                Arc::new(str_values),
            ],
        )
        .unwrap()
    }

    /// Helper to create log attributes batch
    fn create_test_log_attrs() -> RecordBatch {
        let schema = Arc::new(Schema::new(vec![
            Field::new("parent_id", DataType::UInt16, false),
            Field::new("key", DataType::Utf8, false),
            Field::new("type", DataType::UInt8, false),
            Field::new("str", DataType::Utf8, true),
        ]));

        // Attributes for log records
        let parent_id = UInt16Array::from(vec![1, 1, 2]); // 2 attrs for log 1, 1 for log 2
        let keys = StringArray::from(vec!["user.id", "request.id", "error.code"]);
        let types = UInt8Array::from(vec![1, 1, 1]); // All string
        let str_values = StringArray::from(vec![Some("user123"), Some("req-abc"), Some("E500")]);

        RecordBatch::try_new(
            schema,
            vec![
                Arc::new(parent_id),
                Arc::new(keys),
                Arc::new(types),
                Arc::new(str_values),
            ],
        )
        .unwrap()
    }

    #[test]
    fn test_create_otap_logs_view() {
        // Create test data
        let logs_batch = create_test_logs_batch();
        let resource_attrs = create_test_resource_attrs();
        let log_attrs = create_test_log_attrs();

        // Create view
        let logs_view = OtapLogsView::new(
            &logs_batch,
            Some(&resource_attrs),
            None, // no scope attrs
            Some(&log_attrs),
        )
        .unwrap();

        // Verify we can iterate
        let mut resource_count = 0;
        let mut scope_count = 0;
        let mut log_count = 0;

        for resource_logs in logs_view.resources() {
            resource_count += 1;

            // Verify resource attributes
            if let Some(resource) = resource_logs.resource() {
                let attr_count = resource.attributes().count();
                assert_eq!(attr_count, 2, "Expected 2 resource attributes");
            }

            for scope_logs in resource_logs.scopes() {
                scope_count += 1;

                for log_record in scope_logs.log_records() {
                    log_count += 1;

                    // Verify basic log fields are accessible
                    let _ = log_record.time_unix_nano();
                    let _ = log_record.observed_time_unix_nano();
                    let _ = log_record.severity_number();

                    // Verify body
                    if let Some(body) = log_record.body() {
                        if let Some(s) = body.as_string() {
                            let s = std::str::from_utf8(s).unwrap();
                            assert!(
                                s == "Log message 1"
                                    || s == "Error occurred"
                                    || s == "Warning message",
                                "Unexpected body: {}",
                                s
                            );
                        } else {
                            panic!("Body should be a string");
                        }
                    } else {
                        panic!("Body should be present");
                    }
                }
            }
        }

        assert_eq!(resource_count, 1, "Expected 1 resource group");
        assert_eq!(scope_count, 2, "Expected 2 scope groups");
        assert_eq!(log_count, 3, "Expected 3 log records");
    }

    #[test]
    fn test_try_from_otap_arrow_records() {
        // Create test data
        let logs_batch = create_test_logs_batch();
        let resource_attrs = create_test_resource_attrs();
        let log_attrs = create_test_log_attrs();

        // Create OtapArrowRecords
        let mut otap_records = OtapArrowRecords::Logs(Default::default());
        otap_records.set(ArrowPayloadType::Logs, logs_batch.clone());
        otap_records.set(ArrowPayloadType::ResourceAttrs, resource_attrs.clone());
        otap_records.set(ArrowPayloadType::LogAttrs, log_attrs.clone());

        // Test TryFrom implementation
        let logs_view = OtapLogsView::try_from(&otap_records)
            .expect("Should build logs view from OtapArrowRecords");

        // Verify the view is usable by counting logs
        let mut log_count = 0;
        for resource_logs in logs_view.resources() {
            for scope_logs in resource_logs.scopes() {
                for _log_record in scope_logs.log_records() {
                    log_count += 1;
                }
            }
        }

        assert_eq!(log_count, 3, "Expected 3 logs through view");
    }

    #[test]
    fn test_try_from_requires_logs_batch() {
        // Empty OtapArrowRecords without logs batch
        let otap_records = OtapArrowRecords::Logs(Default::default());

        // Should fail with RecordBatchNotFound error
        let result = OtapLogsView::try_from(&otap_records);

        assert!(result.is_err());
        if let Err(err) = result {
            assert!(matches!(err, Error::RecordBatchNotFound { .. }));
        }
    }

    #[test]
    fn test_zero_copy_behavior() {
        // Verify that creating views doesn't copy the RecordBatch data
        let logs_batch = create_test_logs_batch();

        // Get pointer to original data
        let original_ptr = logs_batch
            .column(0)
            .as_any()
            .downcast_ref::<UInt16Array>()
            .unwrap()
            .values()
            .as_ptr();

        // Create view
        let logs_view = OtapLogsView::new(&logs_batch, None, None, None).unwrap();

        // Iterate through view and access data
        for resource_logs in logs_view.resources() {
            for scope_logs in resource_logs.scopes() {
                for _log_record in scope_logs.log_records() {
                    // Just iterating, accessing the data
                }
            }
        }

        // Verify pointer hasn't changed (no copy occurred)
        let after_ptr = logs_batch
            .column(0)
            .as_any()
            .downcast_ref::<UInt16Array>()
            .unwrap()
            .values()
            .as_ptr();
        assert_eq!(
            original_ptr, after_ptr,
            "Data was copied! Zero-copy violated"
        );

        // The view should just hold references and indices
        // Size check removed as we now store cached columns and indices
        // assert_eq!(
        //     std::mem::size_of_val(&logs_view),
        //     std::mem::size_of::<&RecordBatch>() * 4, // 4 reference fields
        //     "View size should be just 4 pointers"
        // );
    }

    #[test]
    fn test_non_contiguous_resource_grouping() {
        // Test that scattered (non-contiguous) data is correctly detected and stored as Scattered
        // Pattern: Resource A, Resource B, Resource A creates non-contiguous grouping for Resource A
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::UInt16, false),
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
        ]));

        let id_array = UInt16Array::from(vec![1, 2, 3]);
        let resource_id_array = UInt16Array::from(vec![1, 2, 1]); // A, B, A pattern
        let resource_struct = StructArray::from(vec![(
            Arc::new(Field::new("id", DataType::UInt16, false)),
            Arc::new(resource_id_array) as ArrayRef,
        )]);

        let scope_id_array = UInt16Array::from(vec![10, 10, 10]);
        let scope_struct = StructArray::from(vec![(
            Arc::new(Field::new("id", DataType::UInt16, false)),
            Arc::new(scope_id_array) as ArrayRef,
        )]);

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(id_array),
                Arc::new(resource_struct),
                Arc::new(scope_struct),
            ],
        )
        .unwrap();

        let view = OtapLogsView::new(&batch, None, None, None).unwrap();

        // Verify resource grouping
        let resource_groups = &view.resource_groups;
        assert_eq!(resource_groups.len(), 2, "Should have 2 resource groups");

        // Resource 1 should be Scattered (indices [0, 2])
        let (res_id, row_group) = &resource_groups[0];
        assert_eq!(*res_id, 1);
        match row_group {
            RowGroup::Scattered(indices) => {
                assert_eq!(
                    *indices,
                    vec![0, 2],
                    "Resource 1 should have scattered indices [0, 2]"
                );
            }
            RowGroup::Contiguous(_) => {
                panic!("Resource 1 should be Scattered, not Contiguous");
            }
        }

        // Resource 2 should be Contiguous (index [1])
        let (res_id, row_group) = &resource_groups[1];
        assert_eq!(*res_id, 2);
        match row_group {
            RowGroup::Contiguous(range) => {
                assert_eq!(range, &(1..2), "Resource 2 should be contiguous range 1..2");
            }
            RowGroup::Scattered(_) => {
                panic!("Resource 2 should be Contiguous, not Scattered");
            }
        }

        // Verify iteration works correctly
        let mut resource_count = 0;
        let mut total_logs = 0;
        for resource_logs in view.resources() {
            resource_count += 1;
            for scope_logs in resource_logs.scopes() {
                for _log in scope_logs.log_records() {
                    total_logs += 1;
                }
            }
        }

        assert_eq!(resource_count, 2, "Should iterate over 2 resources");
        assert_eq!(total_logs, 3, "Should iterate over all 3 log records");
    }
    #[test]
    fn test_contiguous_optimization() {
        // Test that contiguous data is correctly detected as Contiguous
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::UInt16, false),
            Field::new(
                "resource",
                DataType::Struct(vec![Field::new("id", DataType::UInt16, false)].into()),
                false,
            ),
        ]));

        // 100 rows, all resource 1
        let id_array = UInt16Array::from_iter_values(0..100);
        let resource_id_array = UInt16Array::from_iter_values(std::iter::repeat_n(1, 100));
        let resource_struct = StructArray::from(vec![(
            Arc::new(Field::new("id", DataType::UInt16, false)),
            Arc::new(resource_id_array) as ArrayRef,
        )]);

        let batch =
            RecordBatch::try_new(schema, vec![Arc::new(id_array), Arc::new(resource_struct)])
                .unwrap();

        let logs_view = OtapLogsView::new(&batch, None, None, None).unwrap();

        // Should have 1 group, Contiguous(0..100)
        assert_eq!(logs_view.resource_groups.len(), 1);
        let (id, group) = &logs_view.resource_groups[0];
        assert_eq!(*id, 1);

        let debug_str = format!("{:?}", group);
        assert!(debug_str.contains("Contiguous"), "Should be Contiguous");
        assert!(debug_str.contains("0..100"), "Should cover full range");
    }

    /// Test that log attributes with dictionary-encoded keys and values are correctly accessible
    ///
    /// This test validates:
    /// - Attributes linked to log records with id=0 are accessible (0 is a valid ID)
    /// - Dictionary-encoded attribute keys (names) are properly decoded
    /// - Dictionary-encoded attribute values (strings and integers) are properly decoded
    #[test]
    fn test_log_attributes_with_dictionary_encoding() {
        use arrow::array::{DictionaryArray, TimestampNanosecondArray};
        use arrow::datatypes::{UInt8Type, UInt16Type};

        // Create a simple logs batch with one log record (id=0)
        let logs_schema = Arc::new(Schema::new(vec![
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
                "time_unix_nano",
                DataType::Timestamp(TimeUnit::Nanosecond, None),
                false,
            ),
            Field::new(
                "observed_time_unix_nano",
                DataType::Timestamp(TimeUnit::Nanosecond, None),
                false,
            ),
        ]));

        let id_array = UInt16Array::from(vec![0]);
        let resource_id_array = UInt16Array::from(vec![0]);
        let resource_struct = StructArray::from(vec![(
            Arc::new(Field::new("id", DataType::UInt16, false)),
            Arc::new(resource_id_array) as ArrayRef,
        )]);
        let scope_id_array = UInt16Array::from(vec![0]);
        let scope_struct = StructArray::from(vec![(
            Arc::new(Field::new("id", DataType::UInt16, false)),
            Arc::new(scope_id_array) as ArrayRef,
        )]);
        let time_array = TimestampNanosecondArray::from(vec![1_000_000_000]);
        let observed_time_array = TimestampNanosecondArray::from(vec![1_000_000_000]);

        let logs_batch = RecordBatch::try_new(
            logs_schema,
            vec![
                Arc::new(id_array),
                Arc::new(resource_struct),
                Arc::new(scope_struct),
                Arc::new(time_array),
                Arc::new(observed_time_array),
            ],
        )
        .unwrap();

        // Create log attributes batch with dictionary-encoded keys and values
        // This mimics the OTAP syslog data structure
        let log_attrs_schema = Arc::new(Schema::new(vec![
            Field::new("parent_id", DataType::UInt16, false),
            Field::new(
                "key",
                DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                false,
            ),
            Field::new("type", DataType::UInt8, false),
            Field::new(
                "str",
                DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                true,
            ),
            Field::new(
                "int",
                DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Int64)),
                true,
            ),
        ]));

        // Create dictionary-encoded keys (attribute names)
        let keys_dict = StringArray::from(vec!["attr.string", "attr.int", "attr.zero_id"]);
        let keys_indices = UInt8Array::from(vec![0, 1, 2]);
        let keys_array =
            DictionaryArray::<UInt8Type>::try_new(keys_indices, Arc::new(keys_dict)).unwrap();

        // Create dictionary-encoded string values
        let str_dict = StringArray::from(vec!["test_value"]);
        let str_indices = UInt16Array::from(vec![Some(0), None, Some(0)]);
        let str_array =
            DictionaryArray::<UInt16Type>::try_new(str_indices, Arc::new(str_dict)).unwrap();

        // Create dictionary-encoded int values
        let int_dict = Int64Array::from(vec![42, 0]);
        let int_indices = UInt16Array::from(vec![None, Some(0), Some(1)]);
        let int_array =
            DictionaryArray::<UInt16Type>::try_new(int_indices, Arc::new(int_dict)).unwrap();

        let log_attrs_batch = RecordBatch::try_new(
            log_attrs_schema,
            vec![
                Arc::new(UInt16Array::from(vec![0, 0, 0])), // parent_id = 0 for all
                Arc::new(keys_array),
                Arc::new(UInt8Array::from(vec![1, 2, 2])), // type: Str=1, Int=2
                Arc::new(str_array),
                Arc::new(int_array),
            ],
        )
        .unwrap();

        // Create view with log attributes
        let logs_view = OtapLogsView::new(&logs_batch, None, None, Some(&log_attrs_batch)).unwrap();

        // Iterate and verify attributes
        let mut attr_count = 0;
        let mut found_string_attr = false;
        let mut found_int_attr = false;
        let mut found_zero_id_attr = false;

        for resource_logs in logs_view.resources() {
            for scope_logs in resource_logs.scopes() {
                for log_record in scope_logs.log_records() {
                    for attr in log_record.attributes() {
                        attr_count += 1;
                        let key_str = std::str::from_utf8(attr.key()).unwrap();

                        match key_str {
                            "attr.string" => {
                                found_string_attr = true;
                                assert_eq!(
                                    attr.value()
                                        .unwrap()
                                        .as_string()
                                        .map(|s| std::str::from_utf8(s).unwrap()),
                                    Some("test_value"),
                                    "String attribute value mismatch"
                                );
                            }
                            "attr.int" => {
                                found_int_attr = true;
                                assert_eq!(
                                    attr.value().unwrap().as_int64(),
                                    Some(42),
                                    "Int attribute value mismatch"
                                );
                            }
                            "attr.zero_id" => {
                                found_zero_id_attr = true;
                                // Validates that attributes with parent_id=0 are accessible (0 is valid)
                                assert_eq!(
                                    attr.value().unwrap().as_int64(),
                                    Some(0),
                                    "Zero ID attribute value mismatch"
                                );
                            }
                            _ => panic!("Unexpected attribute key: {}", key_str),
                        }
                    }
                }
            }
        }

        assert_eq!(attr_count, 3, "Expected 3 attributes");
        assert!(found_string_attr, "String attribute not found");
        assert!(found_int_attr, "Int attribute not found");
        assert!(found_zero_id_attr, "Attribute with parent_id=0 not found");
    }

    /// Helper to create a logs batch WITHOUT ID column (for when there are no attributes)
    fn create_test_logs_batch_no_id() -> RecordBatch {
        create_test_logs_batch_impl(false)
    }

    #[test]
    fn test_missing_attributes_iterator() {
        // Verify that when attribute batches are None, the iterators correctly
        // return empty without panicking
        // Use batch WITHOUT ID column since there are no attributes to link to
        let logs_batch = create_test_logs_batch_no_id();

        // Create view with no attributes
        let logs_view = OtapLogsView::new(&logs_batch, None, None, None).unwrap();

        let mut log_count = 0;
        for resource_logs in logs_view.resources() {
            // Resource attributes should be empty
            if let Some(resource) = resource_logs.resource() {
                let attr_count = resource.attributes().count();
                assert_eq!(
                    attr_count, 0,
                    "Expected 0 resource attributes when batch is None"
                );
            }

            for scope_logs in resource_logs.scopes() {
                // Scope attributes should be empty
                if let Some(scope) = scope_logs.scope() {
                    let scope_attr_count = scope.attributes().count();
                    assert_eq!(
                        scope_attr_count, 0,
                        "Expected 0 scope attributes when batch is None"
                    );
                }

                for log_record in scope_logs.log_records() {
                    log_count += 1;

                    // Log attributes should be empty
                    let log_attr_count = log_record.attributes().count();
                    assert_eq!(
                        log_attr_count, 0,
                        "Expected 0 log attributes when batch is None"
                    );
                }
            }
        }

        assert_eq!(log_count, 3, "Should still iterate through all 3 logs");
    }

    #[test]
    fn test_dictionary_encoded_resource_and_scope_ids() {
        use arrow::array::DictionaryArray;
        use arrow::datatypes::UInt8Type;

        // Test that dictionary-encoded resource.id and scope.id columns work correctly
        // This is a common optimization in Arrow for columns with repeated values
        // Note: MaybeDictArrayAccessor supports UInt8/UInt16 key types

        // Create dictionary-encoded resource IDs: [0, 0, 1] (indices) -> [10, 20] (values)
        let resource_id_keys = UInt8Array::from(vec![0, 0, 1]);
        let resource_id_values = UInt16Array::from(vec![10, 20]);
        let resource_id_dict =
            DictionaryArray::<UInt8Type>::try_new(resource_id_keys, Arc::new(resource_id_values))
                .unwrap();

        let resource_struct = StructArray::from(vec![(
            Arc::new(Field::new(
                "id",
                DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::UInt16)),
                false,
            )),
            Arc::new(resource_id_dict) as ArrayRef,
        )]);

        // Create dictionary-encoded scope IDs: [0, 1, 1] (indices) -> [100, 200] (values)
        let scope_id_keys = UInt8Array::from(vec![0, 1, 1]);
        let scope_id_values = UInt16Array::from(vec![100, 200]);
        let scope_id_dict =
            DictionaryArray::<UInt8Type>::try_new(scope_id_keys, Arc::new(scope_id_values))
                .unwrap();

        let scope_struct = StructArray::from(vec![(
            Arc::new(Field::new(
                "id",
                DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::UInt16)),
                false,
            )),
            Arc::new(scope_id_dict) as ArrayRef,
        )]);

        // Create minimal log data
        let time_array = TimestampNanosecondArray::from(vec![1_000_000, 2_000_000, 3_000_000]);

        let body_type_array = UInt8Array::from(vec![1, 1, 1]); // Str type
        let body_str_array = StringArray::from(vec!["log1", "log2", "log3"]);
        let body_struct = StructArray::from(vec![
            (
                Arc::new(Field::new("type", DataType::UInt8, false)),
                Arc::new(body_type_array) as ArrayRef,
            ),
            (
                Arc::new(Field::new("str", DataType::Utf8, true)),
                Arc::new(body_str_array) as ArrayRef,
            ),
        ]);

        let schema = Arc::new(Schema::new(vec![
            Field::new(
                "resource",
                DataType::Struct(
                    vec![Field::new(
                        "id",
                        DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::UInt16)),
                        false,
                    )]
                    .into(),
                ),
                false,
            ),
            Field::new(
                "scope",
                DataType::Struct(
                    vec![Field::new(
                        "id",
                        DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::UInt16)),
                        false,
                    )]
                    .into(),
                ),
                false,
            ),
            Field::new(
                "time_unix_nano",
                DataType::Timestamp(TimeUnit::Nanosecond, None),
                true,
            ),
            Field::new(
                "body",
                DataType::Struct(
                    vec![
                        Field::new("type", DataType::UInt8, false),
                        Field::new("str", DataType::Utf8, true),
                    ]
                    .into(),
                ),
                true,
            ),
        ]));

        let logs_batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(resource_struct) as ArrayRef,
                Arc::new(scope_struct) as ArrayRef,
                Arc::new(time_array) as ArrayRef,
                Arc::new(body_struct) as ArrayRef,
            ],
        )
        .unwrap();

        // Create view with dictionary-encoded IDs
        let logs_view = OtapLogsView::new(&logs_batch, None, None, None)
            .expect("Should create view with dictionary-encoded IDs");

        // Verify correct grouping by resource
        let mut resource_count = 0;
        let mut scope_count = 0;
        let mut log_count = 0;

        for resource_logs in logs_view.resources() {
            resource_count += 1;
            for scope_logs in resource_logs.scopes() {
                scope_count += 1;
                for _log in scope_logs.log_records() {
                    log_count += 1;
                }
            }
        }

        // Expected: 2 resources (ID 10 and 20), 3 scopes total, 3 logs
        assert_eq!(
            resource_count, 2,
            "Should have 2 resource groups from dictionary-encoded resource.id"
        );
        assert_eq!(
            scope_count, 3,
            "Should have 3 scope groups from dictionary-encoded scope.id"
        );
        assert_eq!(log_count, 3, "Should iterate all 3 logs");

        // Verify the grouping is correct by checking log distribution
        let resources: Vec<_> = logs_view.resources().collect();

        // First resource (ID 10) should have 2 logs
        let first_resource_logs: usize = resources[0]
            .scopes()
            .map(|scope| scope.log_records().count())
            .sum();
        assert_eq!(
            first_resource_logs, 2,
            "First resource should have 2 logs (indices 0, 1)"
        );

        // Second resource (ID 20) should have 1 log
        let second_resource_logs: usize = resources[1]
            .scopes()
            .map(|scope| scope.log_records().count())
            .sum();
        assert_eq!(
            second_resource_logs, 1,
            "Second resource should have 1 log (index 2)"
        );
    }
}
