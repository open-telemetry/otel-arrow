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
use otel_arrow_dfe_pdata_views::views::common::{AnyValueView, AttributeView, Str, ValueType};

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
    /// A composite value (array or key-value list) still CBOR-encoded in the `ser` column.
    Serialized(&'a [u8]),
}

impl<'a> AnyValueView<'a> for OtapAnyValueView<'a> {
    type KeyValue = OtapAttributeView<'a>;
    type ArrayIter<'arr>
        = CborArrayIter<'a>
    where
        Self: 'arr;
    type KeyValueIter<'kv>
        = CborMapIter<'a>
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
            Self::Serialized(bytes) => cbor_value_type(bytes),
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
        match *self {
            Self::Serialized(bytes) if cbor_value_type(bytes) == ValueType::Array => {
                Some(CborArrayIter::new(bytes))
            }
            _ => None,
        }
    }

    fn as_kvlist(&self) -> Option<Self::KeyValueIter<'_>> {
        match *self {
            Self::Serialized(bytes) if cbor_value_type(bytes) == ValueType::KeyValueList => {
                Some(CborMapIter::new(bytes))
            }
            _ => None,
        }
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

/// Iterator over attributes with u32 parent_id (event attrs, link attrs per OTAP spec sec.5.4.1).
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
            .unwrap_or(OtapAnyValueView::Str(b"")),
        AttributeValueType::Int => anyval
            .attr_int
            .as_ref()
            .and_then(|accessor| accessor.value_at(row_idx))
            .map(OtapAnyValueView::Int)
            .unwrap_or(OtapAnyValueView::Int(0)),
        AttributeValueType::Double => anyval
            .attr_double
            .and_then(|arr| {
                if arr.is_valid(row_idx) {
                    Some(OtapAnyValueView::Double(arr.value(row_idx)))
                } else {
                    None
                }
            })
            .unwrap_or(OtapAnyValueView::Double(0.0)),
        AttributeValueType::Bool => anyval
            .attr_bool
            .and_then(|arr| {
                if arr.is_valid(row_idx) {
                    Some(OtapAnyValueView::Bool(arr.value(row_idx)))
                } else {
                    None
                }
            })
            .unwrap_or(OtapAnyValueView::Bool(false)),
        AttributeValueType::Bytes => anyval
            .attr_bytes
            .as_ref()
            .and_then(|accessor| accessor.slice_at(row_idx))
            .map(OtapAnyValueView::Bytes)
            .unwrap_or(OtapAnyValueView::Bytes(b"")),
        AttributeValueType::Map | AttributeValueType::Slice => anyval
            .attr_ser
            .as_ref()
            .and_then(|accessor| accessor.slice_at(row_idx))
            .map(OtapAnyValueView::Serialized)
            .unwrap_or(OtapAnyValueView::Empty),
        _ => OtapAnyValueView::Empty,
    }
}

// ===== Serialized (CBOR) composite value reader =====
//
// Map and Slice attributes store their value as CBOR in the `ser` column. These helpers read that
// CBOR without copying, so text and bytes point back into the buffer, like the OTLP RawAnyValue
// view does for protobuf. serde_cbor writes indefinite-length maps and arrays, so both are handled.

const CBOR_MAJOR_UINT: u8 = 0;
const CBOR_MAJOR_NINT: u8 = 1;
const CBOR_MAJOR_BYTES: u8 = 2;
const CBOR_MAJOR_TEXT: u8 = 3;
const CBOR_MAJOR_ARRAY: u8 = 4;
const CBOR_MAJOR_MAP: u8 = 5;
const CBOR_MAJOR_SIMPLE: u8 = 7;
const CBOR_INDEFINITE: u8 = 31;
const CBOR_BREAK: u8 = 0xff;
const CBOR_NULL: u8 = 0xf6;
// Maximum CBOR nesting depth scanned while measuring item boundaries. Matches serde_cbor's own
// recursion limit.
const CBOR_MAX_DEPTH: usize = 128;

/// Read the argument (length or immediate value) that follows a CBOR head byte, returning the
/// argument and the number of bytes the head occupies. Indefinite heads are handled by callers.
fn cbor_arg(buf: &[u8]) -> Option<(u64, usize)> {
    match *buf.first()? & 0x1f {
        info @ 0..=23 => Some((info as u64, 1)),
        24 => Some((*buf.get(1)? as u64, 2)),
        25 => Some((
            u16::from_be_bytes(buf.get(1..3)?.try_into().ok()?) as u64,
            3,
        )),
        26 => Some((
            u32::from_be_bytes(buf.get(1..5)?.try_into().ok()?) as u64,
            5,
        )),
        27 => Some((u64::from_be_bytes(buf.get(1..9)?.try_into().ok()?), 9)),
        _ => None,
    }
}

/// Whether the CBOR item at `buf[0]` uses indefinite-length encoding (as serde_cbor writes).
fn cbor_is_indefinite(buf: &[u8]) -> bool {
    matches!(buf.first().copied(), Some(b) if (b & 0x1f) == CBOR_INDEFINITE)
}

/// Total byte length of the single CBOR item that starts at `buf[0]`.
fn cbor_item_len(buf: &[u8]) -> Option<usize> {
    cbor_item_len_at_depth(buf, 0)
}

fn cbor_item_len_at_depth(buf: &[u8], depth: usize) -> Option<usize> {
    if depth > CBOR_MAX_DEPTH {
        return None;
    }
    let first = *buf.first()?;
    let major = first >> 5;
    let info = first & 0x1f;

    if info == CBOR_INDEFINITE {
        return match major {
            CBOR_MAJOR_ARRAY | CBOR_MAJOR_MAP => {
                let mut pos = 1;
                loop {
                    if *buf.get(pos)? == CBOR_BREAK {
                        return pos.checked_add(1);
                    }
                    pos = pos.checked_add(cbor_item_len_at_depth(buf.get(pos..)?, depth + 1)?)?;
                    if major == CBOR_MAJOR_MAP {
                        pos =
                            pos.checked_add(cbor_item_len_at_depth(buf.get(pos..)?, depth + 1)?)?;
                    }
                }
            }
            _ => None,
        };
    }

    let (arg, head) = cbor_arg(buf)?;
    match major {
        CBOR_MAJOR_UINT | CBOR_MAJOR_NINT | CBOR_MAJOR_SIMPLE => Some(head),
        CBOR_MAJOR_BYTES | CBOR_MAJOR_TEXT => head.checked_add(usize::try_from(arg).ok()?),
        CBOR_MAJOR_ARRAY => {
            let mut pos = head;
            for _ in 0..arg {
                pos = pos.checked_add(cbor_item_len_at_depth(buf.get(pos..)?, depth + 1)?)?;
            }
            Some(pos)
        }
        CBOR_MAJOR_MAP => {
            let mut pos = head;
            for _ in 0..arg {
                pos = pos.checked_add(cbor_item_len_at_depth(buf.get(pos..)?, depth + 1)?)?;
                pos = pos.checked_add(cbor_item_len_at_depth(buf.get(pos..)?, depth + 1)?)?;
            }
            Some(pos)
        }
        _ => None,
    }
}

/// The `ValueType` of the CBOR item that starts at `buf[0]`. Kept consistent with
/// `cbor_to_any_value` so values it cannot represent are reported as `Empty`.
fn cbor_value_type(buf: &[u8]) -> ValueType {
    let Some(first) = buf.first().copied() else {
        return ValueType::Empty;
    };
    match first >> 5 {
        // Only advertise Int64 when the value fits an i64, matching cbor_to_any_value.
        CBOR_MAJOR_UINT | CBOR_MAJOR_NINT => cbor_arg(buf)
            .and_then(|(v, _)| i64::try_from(v).ok())
            .map_or(ValueType::Empty, |_| ValueType::Int64),
        CBOR_MAJOR_BYTES => ValueType::Bytes,
        CBOR_MAJOR_TEXT => ValueType::String,
        CBOR_MAJOR_ARRAY => ValueType::Array,
        CBOR_MAJOR_MAP => ValueType::KeyValueList,
        CBOR_MAJOR_SIMPLE => match first & 0x1f {
            20 | 21 => ValueType::Bool,
            26 | 27 => ValueType::Double,
            _ => ValueType::Empty,
        },
        _ => ValueType::Empty,
    }
}

/// Borrow the length-prefixed byte/text payload of the CBOR item at `buf[0]`.
fn cbor_slice(buf: &[u8]) -> Option<&[u8]> {
    let (len, head) = cbor_arg(buf)?;
    let len = usize::try_from(len).ok()?;
    buf.get(head..head.checked_add(len)?)
}

/// Decode the single CBOR item at `buf[0]` into an `OtapAnyValueView`. Nested maps and arrays are
/// returned as `Serialized` so they decode lazily on the next `as_kvlist`/`as_array` call.
fn cbor_to_any_value(buf: &[u8]) -> OtapAnyValueView<'_> {
    let Some(first) = buf.first().copied() else {
        return OtapAnyValueView::Empty;
    };
    match first >> 5 {
        CBOR_MAJOR_UINT => cbor_arg(buf)
            .and_then(|(v, _)| i64::try_from(v).ok())
            .map(OtapAnyValueView::Int)
            .unwrap_or(OtapAnyValueView::Empty),
        CBOR_MAJOR_NINT => cbor_arg(buf)
            .and_then(|(v, _)| i64::try_from(v).ok())
            .and_then(|n| (-1i64).checked_sub(n))
            .map(OtapAnyValueView::Int)
            .unwrap_or(OtapAnyValueView::Empty),
        CBOR_MAJOR_BYTES => cbor_slice(buf)
            .map(OtapAnyValueView::Bytes)
            .unwrap_or(OtapAnyValueView::Empty),
        CBOR_MAJOR_TEXT => cbor_slice(buf)
            .map(OtapAnyValueView::Str)
            .unwrap_or(OtapAnyValueView::Empty),
        CBOR_MAJOR_ARRAY | CBOR_MAJOR_MAP => cbor_item_len(buf)
            .and_then(|len| buf.get(..len))
            .map(OtapAnyValueView::Serialized)
            .unwrap_or(OtapAnyValueView::Empty),
        CBOR_MAJOR_SIMPLE => match first & 0x1f {
            20 => OtapAnyValueView::Bool(false),
            21 => OtapAnyValueView::Bool(true),
            26 => buf
                .get(1..5)
                .and_then(|b| b.try_into().ok())
                .map(|b| OtapAnyValueView::Double(f32::from_be_bytes(b) as f64))
                .unwrap_or(OtapAnyValueView::Empty),
            27 => buf
                .get(1..9)
                .and_then(|b| b.try_into().ok())
                .map(|b| OtapAnyValueView::Double(f64::from_be_bytes(b)))
                .unwrap_or(OtapAnyValueView::Empty),
            _ => OtapAnyValueView::Empty,
        },
        _ => OtapAnyValueView::Empty,
    }
}

/// Iterator over the elements of a CBOR-encoded array in the `ser` column.
pub struct CborArrayIter<'a> {
    buf: &'a [u8],
    pos: usize,
    remaining: Option<u64>,
}

impl<'a> CborArrayIter<'a> {
    fn new(buf: &'a [u8]) -> Self {
        if cbor_is_indefinite(buf) {
            Self {
                buf,
                pos: 1,
                remaining: None,
            }
        } else {
            let (count, head) = cbor_arg(buf).unwrap_or((0, 1));
            Self {
                buf,
                pos: head,
                remaining: Some(count),
            }
        }
    }
}

impl<'a> Iterator for CborArrayIter<'a> {
    type Item = OtapAnyValueView<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.remaining {
            Some(0) => return None,
            None => {
                if *self.buf.get(self.pos)? == CBOR_BREAK {
                    return None;
                }
            }
            Some(_) => {}
        }
        let item = self.buf.get(self.pos..)?;
        let len = cbor_item_len(item)?;
        let value = cbor_to_any_value(item.get(..len)?);
        self.pos += len;
        if let Some(remaining) = self.remaining.as_mut() {
            *remaining -= 1;
        }
        Some(value)
    }
}

/// Iterator over the entries of a CBOR-encoded key-value list in the `ser` column.
pub struct CborMapIter<'a> {
    buf: &'a [u8],
    pos: usize,
    remaining: Option<u64>,
}

impl<'a> CborMapIter<'a> {
    fn new(buf: &'a [u8]) -> Self {
        if cbor_is_indefinite(buf) {
            Self {
                buf,
                pos: 1,
                remaining: None,
            }
        } else {
            let (count, head) = cbor_arg(buf).unwrap_or((0, 1));
            Self {
                buf,
                pos: head,
                remaining: Some(count),
            }
        }
    }
}

impl<'a> Iterator for CborMapIter<'a> {
    type Item = OtapAttributeView<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.remaining {
            Some(0) => return None,
            None => {
                if *self.buf.get(self.pos)? == CBOR_BREAK {
                    return None;
                }
            }
            Some(_) => {}
        }
        let key_buf = self.buf.get(self.pos..)?;
        let key_len = cbor_item_len(key_buf)?;
        // Only text keys are supported. Any other key type would corrupt the name, so fail closed.
        let key = match key_buf.first().copied() {
            Some(first) if first >> 5 == CBOR_MAJOR_TEXT => cbor_slice(key_buf.get(..key_len)?)?,
            Some(CBOR_NULL) => &[],
            _ => return None,
        };
        self.pos += key_len;

        let value_buf = self.buf.get(self.pos..)?;
        let value_len = cbor_item_len(value_buf)?;
        let value = cbor_to_any_value(value_buf.get(..value_len)?);
        self.pos += value_len;

        if let Some(remaining) = self.remaining.as_mut() {
            *remaining -= 1;
        }
        Some(OtapAttributeView { key, value })
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
        && let Some(values) = dict_array
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

    index
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::UInt8Array;

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

    /// Scenario: An OTAP attribute row has a scalar type while its optional value column is omitted.
    /// Guarantees: get_attribute_value returns the corresponding type default instead of Empty.
    #[test]
    fn test_omitted_scalar_column_uses_type_default() {
        fn type_only(attr_type: &UInt8Array) -> AnyValueArrays<'_> {
            AnyValueArrays {
                attr_type,
                attr_str: None,
                attr_int: None,
                attr_double: None,
                attr_bool: None,
                attr_bytes: None,
                attr_ser: None,
            }
        }

        let int_type = UInt8Array::from(vec![AttributeValueType::Int as u8]);
        let int_arr = type_only(&int_type);
        let v = get_attribute_value(&int_arr, 0);
        assert_eq!(v.value_type(), ValueType::Int64);
        assert_eq!(v.as_int64(), Some(0));

        let double_type = UInt8Array::from(vec![AttributeValueType::Double as u8]);
        let double_arr = type_only(&double_type);
        let v = get_attribute_value(&double_arr, 0);
        assert_eq!(v.value_type(), ValueType::Double);
        assert_eq!(v.as_double(), Some(0.0));

        let bool_type = UInt8Array::from(vec![AttributeValueType::Bool as u8]);
        let bool_arr = type_only(&bool_type);
        let v = get_attribute_value(&bool_arr, 0);
        assert_eq!(v.value_type(), ValueType::Bool);
        assert_eq!(v.as_bool(), Some(false));

        let str_type = UInt8Array::from(vec![AttributeValueType::Str as u8]);
        let str_arr = type_only(&str_type);
        let v = get_attribute_value(&str_arr, 0);
        assert_eq!(v.value_type(), ValueType::String);
        assert_eq!(v.as_string(), Some(b"".as_slice()));

        let bytes_type = UInt8Array::from(vec![AttributeValueType::Bytes as u8]);
        let bytes_arr = type_only(&bytes_type);
        let v = get_attribute_value(&bytes_arr, 0);
        assert_eq!(v.value_type(), ValueType::Bytes);
        assert_eq!(v.as_bytes(), Some(b"".as_slice()));
    }

    /// Scenario: A key-value list attribute stored as CBOR in the `ser` column.
    /// Guarantees: `as_kvlist` returns the real entries instead of an empty view.
    #[test]
    fn test_serialized_kvlist_decodes() {
        // Indefinite CBOR map {"k": "v"} as written into the `ser` column.
        let cbor = [0xbf, 0x61, b'k', 0x61, b'v', 0xff];
        let value = OtapAnyValueView::Serialized(&cbor);
        assert_eq!(value.value_type(), ValueType::KeyValueList);

        let entries: Vec<_> = value.as_kvlist().expect("kvlist").collect();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].key(), b"k".as_slice());
        let entry_value = entries[0].value().expect("entry has a value");
        assert_eq!(entry_value.as_string(), Some(b"v".as_slice()));
    }

    /// Scenario: An array attribute stored as CBOR in the `ser` column.
    /// Guarantees: `as_array` returns each element instead of an empty view.
    #[test]
    fn test_serialized_array_decodes() {
        // Indefinite CBOR array [1, "a"] as written into the `ser` column.
        let cbor = [0x9f, 0x01, 0x61, b'a', 0xff];
        let value = OtapAnyValueView::Serialized(&cbor);
        assert_eq!(value.value_type(), ValueType::Array);

        let items: Vec<_> = value.as_array().expect("array").collect();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].as_int64(), Some(1));
        assert_eq!(items[1].as_string(), Some(b"a".as_slice()));
    }

    /// Scenario: CBOR integers at and beyond the i64 range are decoded.
    /// Guarantees: i64::MIN/MAX decode exactly and the first value past each end is rejected as Empty rather than wrapping.
    #[test]
    fn test_cbor_integer_bounds_reject_out_of_range() {
        // Unsigned i64::MAX.
        let max = [0x1b, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
        assert_eq!(cbor_to_any_value(&max).as_int64(), Some(i64::MAX));

        // Unsigned i64::MAX + 1 does not fit an i64.
        let over = [0x1b, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert_eq!(cbor_to_any_value(&over).value_type(), ValueType::Empty);

        // Negative i64::MIN.
        let min = [0x3b, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
        assert_eq!(cbor_to_any_value(&min).as_int64(), Some(i64::MIN));

        // One below i64::MIN does not fit an i64.
        let under = [0x3b, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert_eq!(cbor_to_any_value(&under).value_type(), ValueType::Empty);
    }

    /// Scenario: values that cbor_to_any_value cannot represent are queried for their type.
    /// Guarantees: cbor_value_type reports Empty so it agrees with the decoded value.
    #[test]
    fn test_cbor_value_type_matches_decoding() {
        // Unsigned i64::MAX + 1 cannot be an i64.
        let over = [0x1b, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert_eq!(cbor_value_type(&over), ValueType::Empty);
        assert_eq!(cbor_to_any_value(&over).value_type(), ValueType::Empty);

        // An in-range integer is still Int64.
        assert_eq!(cbor_value_type(&[0x0a]), ValueType::Int64);

        // Half precision floats are classified but never decoded, so they are Empty.
        let half = [0xf9, 0x3c, 0x00];
        assert_eq!(cbor_value_type(&half), ValueType::Empty);
        assert_eq!(cbor_to_any_value(&half).value_type(), ValueType::Empty);

        // A 64-bit float is still Double.
        let double = [0xfb, 0x3f, 0xf0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert_eq!(cbor_value_type(&double), ValueType::Double);
        assert_eq!(cbor_to_any_value(&double).as_double(), Some(1.0));
    }

    /// Scenario: a serialized CBOR map uses a non-text (integer) key.
    /// Guarantees: the entry is dropped instead of yielding a corrupted attribute name.
    #[test]
    fn test_cbor_non_text_map_key_is_rejected() {
        // Definite map {1: 2}.
        let cbor = [0xa1, 0x01, 0x02];
        let value = OtapAnyValueView::Serialized(&cbor);
        assert_eq!(value.value_type(), ValueType::KeyValueList);
        assert!(value.as_kvlist().expect("kvlist").next().is_none());
    }

    /// Scenario: a valid but pathologically deep CBOR value is scanned.
    /// Guarantees: boundary scanning stops at the depth cap and fails closed rather than
    /// recursing until the worker stack overflows.
    #[test]
    fn test_cbor_deep_nesting_fails_closed() {
        // More nested single-element arrays than CBOR_MAX_DEPTH, wrapping a final integer.
        let mut deep = vec![0x81u8; CBOR_MAX_DEPTH + 50];
        deep.push(0x00);
        let value = OtapAnyValueView::Serialized(&deep);
        assert_eq!(value.value_type(), ValueType::Array);
        assert!(value.as_array().expect("array").next().is_none());
    }
}
