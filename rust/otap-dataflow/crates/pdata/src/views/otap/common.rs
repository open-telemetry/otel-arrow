// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared types and helpers for OTAP view implementations (logs, traces, metrics).
//!
//! This module contains types that are used by multiple view implementations to avoid
//! code duplication across signal-specific modules.

use std::collections::BTreeMap;
use std::ops::Range;

use arrow::array::{Array, RecordBatch, StructArray, UInt16Array};

use crate::arrays::{MaybeDictArrayAccessor, NullableArrayAccessor, StringArrayAccessor};
use crate::otlp::attributes::{Attribute16Arrays, Attribute32Arrays, AttributeValueType};
use crate::otlp::common::AnyValueArrays;
use crate::schema::consts;
use otap_df_pdata_views::views::common::{AnyValueView, AttributeView, Str, ValueType};

// ===== RowGroup =====

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

impl Iterator for RowGroupIter<'_> {
    type Item = usize;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            RowGroupIter::Contiguous(range) => range.next(),
            RowGroupIter::Scattered(iter) => iter.next().copied(),
        }
    }
}

// ===== AnyValue and Attribute Views =====

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
        None
    }

    fn as_kvlist(&self) -> Option<Self::KeyValueIter<'_>> {
        None
    }
}

/// Attribute view for OTAP format
pub struct OtapAttributeView<'a> {
    pub(crate) key: &'a [u8],
    pub(crate) value: OtapAnyValueView<'a>,
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

// ===== Attribute Iterators =====

/// Iterator over attributes with u16 parent_id (logs, spans, resources, scopes).
pub struct OtapAttributeIter<'a> {
    pub(crate) attrs: Option<&'a Attribute16Arrays<'a>>,
    pub(crate) matching_rows: &'a [usize],
    pub(crate) current_idx: usize,
}

impl<'a> Iterator for OtapAttributeIter<'a> {
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

            if let Some(key) = get_attribute_key(&attrs_cols.attr_key, attr_row_idx) {
                let value = get_attribute_value(&attrs_cols.anyval_arrays, attr_row_idx);
                return Some(OtapAttributeView { key, value });
            }
        }
    }
}

/// Iterator over attributes with u32 parent_id (event attrs, link attrs per OTAP spec §5.4.1).
pub struct Otap32AttributeIter<'a> {
    pub(crate) attrs: Option<&'a Attribute32Arrays<'a>>,
    pub(crate) matching_rows: &'a [usize],
    pub(crate) current_idx: usize,
}

impl<'a> Iterator for Otap32AttributeIter<'a> {
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

            if let Some(key) = get_attribute_key(&attrs_cols.attr_key, attr_row_idx) {
                let value = get_attribute_value(&attrs_cols.anyval_arrays, attr_row_idx);
                return Some(OtapAttributeView { key, value });
            }
        }
    }
}

// ===== Attribute Helper Functions =====

/// Extract attribute key from any attribute arrays.
#[inline(always)]
pub(crate) fn get_attribute_key<'a>(
    attr_key: &'a StringArrayAccessor<'a>,
    row_idx: usize,
) -> Option<&'a [u8]> {
    attr_key.str_at(row_idx).map(|s| s.as_bytes())
}

/// Extract attribute value from any attribute arrays.
#[inline(always)]
pub(crate) fn get_attribute_value<'a>(
    anyval: &'a AnyValueArrays<'a>,
    row_idx: usize,
) -> OtapAnyValueView<'a> {
    let type_array = anyval.attr_type;

    if !type_array.is_valid(row_idx) {
        return OtapAnyValueView::Empty;
    }

    let value_type = match AttributeValueType::try_from(type_array.value(row_idx)) {
        Ok(t) => t,
        Err(_) => return OtapAnyValueView::Empty,
    };

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

// ===== Grouping Helpers =====

/// Build an inverted index from parent_id (u16) to list of row indices.
pub(crate) fn build_u16_index(batch: &RecordBatch) -> BTreeMap<u16, Vec<usize>> {
    let parent_id_col = match batch.column_by_name(consts::PARENT_ID) {
        Some(col) => col,
        None => return BTreeMap::new(),
    };

    let mut index: BTreeMap<u16, Vec<usize>> = BTreeMap::new();

    if let Ok(accessor) = MaybeDictArrayAccessor::<UInt16Array>::try_new(parent_id_col) {
        for i in 0..batch.num_rows() {
            if let Some(pid) = accessor.value_at(i) {
                index.entry(pid).or_default().push(i);
            }
        }
        return index;
    }

    if let Some(parent_id_array) = parent_id_col.as_any().downcast_ref::<UInt16Array>() {
        for i in 0..batch.num_rows() {
            if parent_id_array.is_valid(i) {
                index.entry(parent_id_array.value(i)).or_default().push(i);
            }
        }
    }

    index
}

/// Group rows by resource ID column.
pub(crate) fn group_by_resource_id(batch: &RecordBatch) -> Vec<(u16, RowGroup)> {
    let num_rows = batch.num_rows();
    if num_rows == 0 {
        return Vec::new();
    }

    let resource_struct = batch
        .column_by_name(consts::RESOURCE)
        .and_then(|c| c.as_any().downcast_ref::<StructArray>());

    let id_col = resource_struct.and_then(|s| s.column_by_name(consts::ID));

    match id_col {
        Some(col) => group_by_id_column(col, 0..num_rows),
        None => Vec::new(),
    }
}

/// Group rows by scope ID within a set of rows.
pub(crate) fn group_by_scope_id(
    batch: &RecordBatch,
    row_indices: &RowGroup,
) -> Vec<(u16, RowGroup)> {
    let scope_struct = batch
        .column_by_name(consts::SCOPE)
        .and_then(|c| c.as_any().downcast_ref::<StructArray>());

    let id_col = scope_struct.and_then(|s| s.column_by_name(consts::ID));

    match id_col {
        Some(col) => group_by_id_column(col, row_indices.iter()),
        None => Vec::new(),
    }
}

/// Core grouping logic for any ID column.
fn group_by_id_column(
    id_col: &dyn Array,
    indices: impl Iterator<Item = usize>,
) -> Vec<(u16, RowGroup)> {
    enum GroupBuilder {
        Contiguous { start: usize, count: usize },
        Scattered(Vec<usize>),
    }

    fn insert_idx(builders: &mut BTreeMap<u16, GroupBuilder>, id: u16, i: usize) {
        let _ = builders
            .entry(id)
            .and_modify(|builder| match builder {
                GroupBuilder::Contiguous { start, count } => {
                    if *start + *count == i {
                        *count += 1;
                    } else {
                        let mut v = Vec::with_capacity(*count + 1);
                        v.extend(*start..(*start + *count));
                        v.push(i);
                        *builder = GroupBuilder::Scattered(v);
                    }
                }
                GroupBuilder::Scattered(v) => v.push(i),
            })
            .or_insert(GroupBuilder::Contiguous { start: i, count: 1 });
    }

    let mut builders: BTreeMap<u16, GroupBuilder> = BTreeMap::new();

    // Convert to ArrayRef to handle dictionary cases
    let id_col_arc: arrow::array::ArrayRef =
        std::sync::Arc::new(arrow::array::make_array(id_col.to_data()));
    if let Ok(accessor) = MaybeDictArrayAccessor::<UInt16Array>::try_new(&id_col_arc) {
        for i in indices {
            if let Some(id) = accessor.value_at(i) {
                insert_idx(&mut builders, id, i);
            }
        }
    } else if let Some(id_array) = id_col.as_any().downcast_ref::<UInt16Array>() {
        for i in indices {
            if id_array.is_valid(i) {
                insert_idx(&mut builders, id_array.value(i), i);
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

// ===== Shared Helpers =====

/// Build an inverted index from parent_id to list of row indices.
/// This is used for attribute batches where parent_id links back to the parent entity.
pub(crate) fn build_attribute_index(batch: &RecordBatch) -> BTreeMap<u16, Vec<usize>> {
    let parent_id_col = match batch.column_by_name(consts::PARENT_ID) {
        Some(col) => col,
        None => return BTreeMap::new(),
    };

    let mut index: BTreeMap<u16, Vec<usize>> = BTreeMap::new();

    if let Ok(accessor) = MaybeDictArrayAccessor::<UInt16Array>::try_new(&parent_id_col.clone()) {
        for i in 0..batch.num_rows() {
            if let Some(pid) = accessor.value_at(i) {
                index.entry(pid).or_default().push(i);
            }
        }
        return index;
    }

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

/// Build an inverted index from u32 parent_id to list of row indices.
pub(crate) fn build_attribute_index_u32(batch: &RecordBatch) -> BTreeMap<u32, Vec<usize>> {
    let parent_id_col = match batch.column_by_name(consts::PARENT_ID) {
        Some(col) => col,
        None => return BTreeMap::new(),
    };

    let mut index: BTreeMap<u32, Vec<usize>> = BTreeMap::new();

    if let Some(parent_id_array) = parent_id_col
        .as_any()
        .downcast_ref::<arrow::array::UInt32Array>()
    {
        for i in 0..batch.num_rows() {
            if parent_id_array.is_valid(i) {
                let pid = parent_id_array.value(i);
                index.entry(pid).or_default().push(i);
            }
        }
    } else if let Some(dict_array) = parent_id_col
        .as_any()
        .downcast_ref::<arrow::array::DictionaryArray<arrow::datatypes::UInt8Type>>()
    {
        if let Some(values) = dict_array
            .values()
            .as_any()
            .downcast_ref::<arrow::array::UInt32Array>()
        {
            for i in 0..batch.num_rows() {
                if dict_array.is_valid(i) {
                    let key = dict_array.keys().value(i) as usize;
                    if values.is_valid(key) {
                        let pid = values.value(key);
                        index.entry(pid).or_default().push(i);
                    }
                }
            }
        }
    }

    index
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_row_group_contiguous() {
        let rg = RowGroup::Contiguous(5..8);
        assert_eq!(rg.len(), 3);
        assert!(!rg.is_empty());

        let indices: Vec<usize> = rg.iter().collect();
        assert_eq!(indices, vec![5, 6, 7]);
    }

    #[test]
    fn test_row_group_scattered() {
        let rg = RowGroup::Scattered(vec![1, 5, 9]);
        assert_eq!(rg.len(), 3);
        assert!(!rg.is_empty());

        let indices: Vec<usize> = rg.iter().collect();
        assert_eq!(indices, vec![1, 5, 9]);
    }

    #[test]
    fn test_row_group_empty() {
        let rg = RowGroup::Contiguous(5..5);
        assert_eq!(rg.len(), 0);
        assert!(rg.is_empty());

        let rg2 = RowGroup::Scattered(vec![]);
        assert_eq!(rg2.len(), 0);
        assert!(rg2.is_empty());
    }

    #[test]
    fn test_any_value_view_types() {
        let v_empty = OtapAnyValueView::Empty;
        assert_eq!(v_empty.value_type(), ValueType::Empty);
        assert_eq!(v_empty.as_string(), None);
        assert_eq!(v_empty.as_int64(), None);
        assert_eq!(v_empty.as_double(), None);
        assert_eq!(v_empty.as_bool(), None);
        assert_eq!(v_empty.as_bytes(), None);
        // assert!(v_empty.as_array().unwrap().next().is_none());
        // assert!(v_empty.as_kvlist().unwrap().next().is_none());

        let v_str = OtapAnyValueView::Str(b"hello");
        assert_eq!(v_str.value_type(), ValueType::String);
        assert_eq!(v_str.as_string(), Some(b"hello".as_slice()));
        assert_eq!(v_str.as_int64(), None);

        let v_int = OtapAnyValueView::Int(42);
        assert_eq!(v_int.value_type(), ValueType::Int64);
        assert_eq!(v_int.as_int64(), Some(42));
        assert_eq!(v_int.as_string(), None);

        let v_double = OtapAnyValueView::Double(3.125);
        assert_eq!(v_double.value_type(), ValueType::Double);
        assert_eq!(v_double.as_double(), Some(3.125));

        let v_bool = OtapAnyValueView::Bool(true);
        assert_eq!(v_bool.value_type(), ValueType::Bool);
        assert_eq!(v_bool.as_bool(), Some(true));

        let v_bytes = OtapAnyValueView::Bytes(b"raw");
        assert_eq!(v_bytes.value_type(), ValueType::Bytes);
        assert_eq!(v_bytes.as_bytes(), Some(b"raw".as_slice()));
    }

    #[test]
    fn test_attribute_view() {
        let attr = OtapAttributeView {
            key: b"service.name",
            value: OtapAnyValueView::Str(b"my-service"),
        };

        assert_eq!(attr.key(), b"service.name".as_slice());
        let val = attr.value().unwrap();
        assert_eq!(val.as_string(), Some(b"my-service".as_slice()));
    }
}
