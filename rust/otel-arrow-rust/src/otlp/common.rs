// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::arrays::{
    ByteArrayAccessor, Int64ArrayAccessor, MaybeDictArrayAccessor, NullableArrayAccessor,
    StringArrayAccessor, StructColumnAccessor, get_bool_array_opt, get_f64_array_opt,
    get_required_array, get_u8_array,
};
use crate::error::{self, Error, Result};
use crate::otlp::attributes::{Attribute16Arrays, encode_key_value};
use crate::proto::consts::field_num::common::{
    INSTRUMENTATION_DROPPED_ATTRIBUTES_COUNT, INSTRUMENTATION_SCOPE_ATTRIBUTES,
    INSTRUMENTATION_SCOPE_NAME, INSTRUMENTATION_SCOPE_VERSION,
};
use crate::proto::consts::field_num::resource::{
    RESOURCE_ATTRIBUTES, RESOURCE_DROPPED_ATTRIBUTES_COUNT,
};
use crate::proto::consts::wire_types;
use crate::proto::opentelemetry::common::v1::{AnyValue, any_value::Value};
use crate::proto_encode_len_delimited_unknown_size;
use crate::schema::consts;
use arrow::array::{
    Array, ArrowPrimitiveType, BooleanArray, Float64Array, PrimitiveArray, RecordBatch,
    StructArray, UInt8Array, UInt16Array, UInt32Array,
};
use arrow::datatypes::{DataType, Field, Fields};
use arrow::row::{Row, RowConverter, SortField};
use snafu::OptionExt;
use std::cmp::Ordering;
use std::fmt;
use std::fmt::Write;
use std::sync::LazyLock;

pub(in crate::otlp) struct ResourceArrays<'a> {
    pub id: &'a UInt16Array,
    pub dropped_attributes_count: Option<&'a UInt32Array>,
    pub schema_url: Option<StringArrayAccessor<'a>>,
}

static RESOURCE_ARRAY_DATA_TYPE: LazyLock<DataType> = LazyLock::new(|| {
    DataType::Struct(Fields::from(vec![
        Field::new(consts::ID, DataType::UInt16, true),
        Field::new(consts::DROPPED_ATTRIBUTES_COUNT, DataType::UInt32, true),
        Field::new(
            consts::SCHEMA_URL,
            DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
            true,
        ),
    ]))
});

impl ResourceArrays<'_> {
    fn data_type() -> &'static DataType {
        &RESOURCE_ARRAY_DATA_TYPE
    }
}

pub(crate) fn proto_encode_resource(
    index: usize,
    resource_arrays: &ResourceArrays<'_>,
    resource_attrs_arrays: Option<&Attribute16Arrays<'_>>,
    resource_attrs_cursor: &mut SortedBatchCursor,
    result_buf: &mut ProtoBuffer,
) -> Result<()> {
    // add attributes
    if let Some(attrs_arrays) = resource_attrs_arrays {
        if let Some(res_id) = resource_arrays.id.value_at(index) {
            for attr_index in
                ChildIndexIter::new(res_id, &attrs_arrays.parent_id, resource_attrs_cursor)
            {
                proto_encode_len_delimited_unknown_size!(
                    RESOURCE_ATTRIBUTES,
                    encode_key_value(attrs_arrays, attr_index, result_buf)?,
                    result_buf
                );
            }
        }
    }

    if let Some(col) = resource_arrays.dropped_attributes_count {
        if let Some(val) = col.value_at(index) {
            result_buf.encode_field_tag(RESOURCE_DROPPED_ATTRIBUTES_COUNT, wire_types::VARINT);
            result_buf.encode_varint(val as u64);
        }
    }

    Ok(())
}

impl<'a> TryFrom<&'a RecordBatch> for ResourceArrays<'a> {
    type Error = Error;

    fn try_from(rb: &'a RecordBatch) -> Result<Self> {
        let struct_array = get_required_array(rb, consts::RESOURCE)?;
        let struct_array = struct_array
            .as_any()
            .downcast_ref::<StructArray>()
            .with_context(|| error::ColumnDataTypeMismatchSnafu {
                name: consts::RESOURCE,
                actual: struct_array.data_type().clone(),
                expect: Self::data_type().clone(),
            })?;

        let struct_col_accessor = StructColumnAccessor::new(struct_array);

        Ok(Self {
            id: struct_col_accessor.primitive_column(consts::ID)?,
            dropped_attributes_count: struct_col_accessor
                .primitive_column_op(consts::DROPPED_ATTRIBUTES_COUNT)?,
            schema_url: struct_col_accessor.string_column_op(consts::SCHEMA_URL)?,
        })
    }
}

pub(in crate::otlp) struct ScopeArrays<'a> {
    pub name: Option<StringArrayAccessor<'a>>,
    pub version: Option<StringArrayAccessor<'a>>,
    pub dropped_attributes_count: Option<&'a UInt32Array>,
    pub id: Option<&'a UInt16Array>,
}

pub static SCOPE_ARRAY_DATA_TYPE: LazyLock<DataType> = LazyLock::new(|| {
    DataType::Struct(Fields::from(vec![
        Field::new(
            consts::NAME,
            DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
            true,
        ),
        Field::new(consts::VERSION, DataType::Utf8, true),
        Field::new(consts::DROPPED_ATTRIBUTES_COUNT, DataType::UInt32, true),
        Field::new(consts::ID, DataType::UInt16, true),
    ]))
});

impl ScopeArrays<'_> {
    fn data_type() -> &'static DataType {
        &SCOPE_ARRAY_DATA_TYPE
    }
}

impl<'a> TryFrom<&'a RecordBatch> for ScopeArrays<'a> {
    type Error = Error;

    fn try_from(rb: &'a RecordBatch) -> Result<Self> {
        let struct_array = get_required_array(rb, consts::SCOPE)?;
        let scope_array = struct_array
            .as_any()
            .downcast_ref::<StructArray>()
            .with_context(|| error::ColumnDataTypeMismatchSnafu {
                name: consts::RESOURCE,
                actual: struct_array.data_type().clone(),
                expect: Self::data_type().clone(),
            })?;
        let struct_col_accessor = StructColumnAccessor::new(scope_array);

        Ok(Self {
            name: struct_col_accessor.string_column_op(consts::NAME)?,
            version: struct_col_accessor.string_column_op(consts::VERSION)?,
            dropped_attributes_count: struct_col_accessor
                .primitive_column_op(consts::DROPPED_ATTRIBUTES_COUNT)?,
            id: struct_col_accessor.primitive_column_op(consts::ID)?,
        })
    }
}

pub(crate) fn proto_encode_instrumentation_scope(
    index: usize,
    scope_arrays: &ScopeArrays<'_>,
    scope_attrs_arrays: Option<&Attribute16Arrays<'_>>,
    scope_attrs_cursor: &mut SortedBatchCursor,
    result_buf: &mut ProtoBuffer,
) -> Result<()> {
    if let Some(col) = &scope_arrays.name {
        if let Some(val) = col.str_at(index) {
            result_buf.encode_string(INSTRUMENTATION_SCOPE_NAME, val);
        }
    }

    if let Some(col) = &scope_arrays.version {
        if let Some(val) = col.str_at(index) {
            result_buf.encode_string(INSTRUMENTATION_SCOPE_VERSION, val);
        }
    }

    if let Some(attr_arrays) = scope_attrs_arrays {
        if let Some(scope_id) = scope_arrays.id.value_at(index) {
            for attr_index in
                ChildIndexIter::new(scope_id, &attr_arrays.parent_id, scope_attrs_cursor)
            {
                proto_encode_len_delimited_unknown_size!(
                    INSTRUMENTATION_SCOPE_ATTRIBUTES,
                    encode_key_value(attr_arrays, attr_index, result_buf)?,
                    result_buf
                );
            }
        }
    }

    if let Some(col) = scope_arrays.dropped_attributes_count {
        if let Some(val) = col.value_at(index) {
            result_buf
                .encode_field_tag(INSTRUMENTATION_DROPPED_ATTRIBUTES_COUNT, wire_types::VARINT);
            result_buf.encode_varint(val as u64);
        }
    }

    Ok(())
}

// display implementation to use for debug processor
impl fmt::Display for AnyValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(value) = &self.value {
            match value {
                Value::StringValue(string) => {
                    write!(f, "{string}")?;
                }
                Value::BoolValue(bool) => {
                    write!(f, "{bool}")?;
                }
                Value::IntValue(int) => {
                    write!(f, "{int}")?;
                }
                Value::DoubleValue(double) => {
                    write!(f, "{double}")?;
                }
                Value::ArrayValue(array) => {
                    let values = &array.values;
                    write!(f, "{values:?}")?;
                }
                Value::KvlistValue(kvlist) => {
                    let mut kv_string = String::new();
                    for kv in kvlist.values.iter() {
                        if let Some(value) = &kv.value {
                            _ = write!(
                                &mut kv_string,
                                "{key}={value} ",
                                key = kv.key,
                                value = value
                            );
                        }
                    }
                    write!(f, "{kv_string}")?;
                }
                Value::BytesValue(bytes) => {
                    if let Ok(byte_string) = String::from_utf8(bytes.to_vec()) {
                        write!(f, "{byte_string}")?;
                    }
                    write!(f, "")?;
                }
            }
        } else {
            write!(f, "")?;
        }
        Ok(())
    }
}

pub(crate) struct AnyValueArrays<'a> {
    pub attr_type: &'a UInt8Array,
    pub attr_str: Option<StringArrayAccessor<'a>>,
    pub attr_int: Option<Int64ArrayAccessor<'a>>,
    pub attr_double: Option<&'a Float64Array>,
    pub attr_bool: Option<&'a BooleanArray>,
    pub attr_bytes: Option<ByteArrayAccessor<'a>>,
    pub attr_ser: Option<ByteArrayAccessor<'a>>,
}

impl<'a> TryFrom<&'a RecordBatch> for AnyValueArrays<'a> {
    type Error = Error;

    fn try_from(rb: &'a RecordBatch) -> Result<Self> {
        let attr_type = get_u8_array(rb, consts::ATTRIBUTE_TYPE)?;
        let attr_str = rb
            .column_by_name(consts::ATTRIBUTE_STR)
            .map(StringArrayAccessor::try_new)
            .transpose()?;
        let attr_int = rb
            .column_by_name(consts::ATTRIBUTE_INT)
            .map(Int64ArrayAccessor::try_new)
            .transpose()?;
        let attr_double = get_f64_array_opt(rb, consts::ATTRIBUTE_DOUBLE)?;
        let attr_bool = get_bool_array_opt(rb, consts::ATTRIBUTE_BOOL)?;
        let attr_bytes = rb
            .column_by_name(consts::ATTRIBUTE_BYTES)
            .map(ByteArrayAccessor::try_new)
            .transpose()?;
        let attr_ser = rb
            .column_by_name(consts::ATTRIBUTE_SER)
            .map(ByteArrayAccessor::try_new)
            .transpose()?;

        Ok(Self {
            attr_type,
            attr_str,
            attr_int,
            attr_double,
            attr_bool,
            attr_bytes,
            attr_ser,
        })
    }
}

/// mutable buffer for encoding protobuf bytes
#[derive(Debug, Default)]
pub struct ProtoBuffer {
    buffer: Vec<u8>,
}

impl ProtoBuffer {
    #[must_use]
    pub const fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.buffer
    }

    pub fn encode_field_tag(&mut self, field_number: u64, wire_type: u64) {
        let key = (field_number << 3) | wire_type;
        self.encode_varint(key);
    }

    #[inline]
    pub fn encode_varint(&mut self, value: u64) {
        // Fast path for single byte (very common)
        if value < 0x80 {
            self.buffer.push(value as u8);
            return;
        }

        // Fast path for two bytes (common)
        if value < 0x4000 {
            self.buffer
                .extend_from_slice(&[((value & 0x7F) | 0x80) as u8, (value >> 7) as u8]);
            return;
        }

        let mut v = value;
        while v >= 0x80 {
            self.buffer.push(((v & 0x7F) | 0x80) as u8);
            v >>= 7;
        }
        self.buffer.push(v as u8);
    }

    /// encodes the signed varint type (e.g. sint32, sint64, etc.) using zig-zag encoding
    /// https://protobuf.dev/programming-guides/encoding/#signed-ints
    #[inline]
    pub fn encode_sint32(&mut self, value: i32) {
        self.encode_varint(((value << 1) ^ (value >> 31)) as u64);
    }

    pub fn extend_from_slice(&mut self, slice: &[u8]) {
        self.buffer.extend_from_slice(slice);
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.buffer.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn encode_string(&mut self, field_tag: u64, val: &str) {
        self.encode_field_tag(field_tag, wire_types::LEN);
        self.encode_varint(val.len() as u64);
        self.extend_from_slice(val.as_bytes());
    }

    pub fn encode_bytes(&mut self, field_tag: u64, val: &[u8]) {
        self.encode_field_tag(field_tag, wire_types::LEN);
        self.encode_varint(val.len() as u64);
        self.extend_from_slice(val);
    }
}

impl AsRef<[u8]> for ProtoBuffer {
    fn as_ref(&self) -> &[u8] {
        &self.buffer
    }
}

impl AsMut<[u8]> for ProtoBuffer {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.buffer
    }
}

/// Helper for encoding with unknown length. Usage:
/// ```ignore
/// proto_encode_len_delimited_unknown_size!(
///     1, // field tag
///     encode_some_nested_field(&mut result_buf), // fills in the child body
///     result_buf
/// )
/// ```
///
/// Our proto encoding algorithm tries to encode in a single pass over the OTAP data, but it will
/// not know the size of the nested child messages a priori. Because the length fields are encoded
/// in a varint, we don't know how many bytes we need to set aside for the length before we start
/// appending the encoded child.
///
/// The workaround is that we set aside a fixed length number of bytes, and create a zero-padded
/// varint. For example, the varint 5, encoded in 4 bytes would be. Observe that in all bytes, the
/// continuation bit (msb) is set:
/// ```text
/// 0x85 0x80 0x80 0x00
/// ```
///
/// Note: this is less efficient from a space perspective, so there's a tradeoff being made here
/// between encoded size and CPU needed to compute the size of the length.
///
/// TODO: currently we're always allocating 4 byte. This may often be too much but we over-allocate
/// to be safe. Eventually we should maybe allow a size hint here and allocate fewer bytes.
///
#[macro_export]
macro_rules! proto_encode_len_delimited_unknown_size {
    ($field_tag: expr, $encode_fn:expr, $buf:expr) => {{
        let num_bytes = 4; // placeholder length
        $buf.encode_field_tag($field_tag, $crate::proto::consts::wire_types::LEN);
        let len_start_pos = $buf.len();
        $crate::otlp::common::encode_len_placeholder($buf);
        $encode_fn;
        let len = $buf.len() - len_start_pos - num_bytes;
        $crate::otlp::common::patch_len_placeholder($buf, num_bytes, len, len_start_pos);
    }};
}

pub(crate) fn encode_len_placeholder(buf: &mut ProtoBuffer) {
    buf.buffer.extend_from_slice(&[0x80, 0x80, 0x80, 0x00]);
}

pub(crate) fn patch_len_placeholder(
    buf: &mut ProtoBuffer,
    num_bytes: usize,
    len: usize,
    len_start_pos: usize,
) {
    for i in 0..num_bytes {
        buf.buffer[len_start_pos + i] += ((len >> (i * 7)) & 0x7f) as u8;
    }
}

/// Used to iterate over OTAP [`RecordBatch`] in a particular order.
///
/// There are certain use cases where we want to visit all the rows in some record batch that are
/// associated with some logical OTel message. For example, visit all the rows for a given
/// `ResourceLog` in a `Logs` OTAP batch, followed by all logs for the next resource, etc.
/// Similar for attributes -- visit the `LogAttrs` record batch in order of the parent_id column
/// to iterate through attributes for the logical `LogRecord` message.
///
/// This is the data-structure that allows to traverse an associated record batch in this natural
/// order by calling the `curr_index()` and `advance()` methods.
///
/// The motivation behind using this cursor is that it will hopefully be more efficient to
/// initialize this than sorting the entire [`RecordBatch`].
#[derive(Debug)]
pub(crate) struct SortedBatchCursor {
    sorted_indices: Vec<usize>,
    curr_index: usize,
}

impl SortedBatchCursor {
    /// Create a new instance of [`SortedBatchCursor`]
    pub fn new() -> Self {
        Self {
            sorted_indices: Vec::new(),
            curr_index: 0,
        }
    }

    /// Reset the cursor. Typically this would be called before the cursor is then reinitialized.
    pub fn reset(&mut self) {
        self.sorted_indices.clear();
        self.curr_index = 0;
    }

    /// Get the current index to visit
    pub fn curr_index(&self) -> Option<usize> {
        (!self.finished()).then(|| self.sorted_indices[self.curr_index])
    }

    /// Advance the cursor
    pub fn advance(&mut self) {
        self.curr_index += 1;
    }

    /// Check if the cursor has finished. This will return true once we've iterated to the end of
    /// the record batch that was used to initialize this cursor.
    pub fn finished(&self) -> bool {
        self.curr_index >= self.sorted_indices.len()
    }
}

/// This is used to initialize the [`SortedBatchCursor`]. It does this by sorting the IDs and then
/// filling in the cursors indices to visit based on the sorted ID column.
pub(crate) struct BatchSorter {
    row_converter: RowConverter,

    // when we sort the record batch, to it's indices we put the ID columns into these Vecs before
    // transferring the indices to the cursor. We keep these on instance of the [`BatchSorter`] so
    // we can reuse the allocations for multiple batches.
    rows: Vec<(usize, Row<'static>)>,
    u16_ids: Vec<(usize, u16)>,
    u32_ids: Vec<(usize, u32)>,
}

impl BatchSorter {
    pub fn new() -> Self {
        // safety: these datatypes are sortable
        let row_converter = RowConverter::new(vec![
            SortField::new(DataType::UInt16),
            SortField::new(DataType::UInt16),
        ])
        .expect("can create row converter");

        Self {
            row_converter,
            rows: Vec::new(),
            u16_ids: Vec::new(),
            u32_ids: Vec::new(),
        }
    }

    /// This helper function exists so we can the heap allocation of the `rows` vec associated with
    /// this [`BatchSorter`]. We effectively just shuffle the vec between having the lifetime of
    /// the borrowed rows while we're sorting, and the static lifetime when we're done
    fn reuse_rows_vec<'a, 'b>(mut v: Vec<(usize, Row<'a>)>) -> Vec<(usize, Row<'b>)> {
        v.clear();
        // there's a compiler optimization that happens here where it will just recognize that the
        // vec we're creating has the same type/alignment as the source, and reuse the allocation
        v.into_iter().map(|_| unreachable!()).collect()
    }

    /// Initializes the cursor to visit the root [`RecordBatch`] in the OTAP batch in order of
    /// ascending Resource ID & Scope ID. This allows the caller to easily iterate the root
    /// [`RecordBatch`] in a depth-first traversal for each Resource -> Scope -> Log/Span/etc.
    pub fn init_cursor_for_root_batch(
        &mut self,
        record_batch: &RecordBatch,
        cursor: &mut SortedBatchCursor,
    ) -> Result<()> {
        let mut sort_columns_idx = 0;
        let mut sort_columns = [None, None];
        for col_name in [consts::RESOURCE, consts::SCOPE] {
            if let Some(resource_col) = record_batch.column_by_name(col_name) {
                let resource_col = resource_col
                    .as_any()
                    .downcast_ref::<StructArray>()
                    .with_context(|| error::ColumnDataTypeMismatchSnafu {
                        name: col_name,
                        expect: DataType::Struct(Fields::empty()),
                        actual: resource_col.data_type().clone(),
                    })?;
                if let Some(resource_ids) = resource_col.column_by_name(consts::ID) {
                    sort_columns[sort_columns_idx] = Some(resource_ids.clone());
                    sort_columns_idx += 1;
                }
            }
        }

        match sort_columns {
            // use row-sorter if we need to sort by both columns
            [Some(resource_ids), Some(scope_ids)] => {
                let rows = self
                    .row_converter
                    .convert_columns(&[resource_ids, scope_ids])
                    .map_err(|e| {
                        error::UnexpectedRecordBatchStateSnafu {
                            reason: format!(
                                "unexpected resource/scope ID columns for sorting: {e:?}"
                            ),
                        }
                        .build()
                    })?;
                let mut sort = Self::reuse_rows_vec(std::mem::take(&mut self.rows));
                sort.extend(rows.iter().enumerate());
                sort.sort_unstable_by(|(_, a), (_, b)| a.cmp(b));
                // populate the cursor
                cursor.sorted_indices.extend(sort.iter().map(|(i, _)| *i));

                // save the rows vec allocation for next batch needs sorting
                self.rows = Self::reuse_rows_vec(sort);
            }

            //there's only one ID column, so we'll visit in the order of this column.
            [Some(ids), None] => {
                let ids = ids
                    .as_any()
                    .downcast_ref::<UInt16Array>()
                    .with_context(|| error::ColumnDataTypeMismatchSnafu {
                        name: consts::ID,
                        expect: DataType::UInt16,
                        actual: ids.data_type().clone(),
                    })?;
                self.init_cursor_for_u16_id_column(&MaybeDictArrayAccessor::Native(ids), cursor);
            }

            // no scope/resource ID columns....
            // just configure cursor to visit the root record batch in order
            [None, None] => {
                cursor.sorted_indices.extend(0..record_batch.num_rows());
            }

            _ => unreachable!(),
        }

        Ok(())
    }

    pub fn init_cursor_for_u16_id_column(
        &mut self,
        ids: &MaybeDictArrayAccessor<'_, UInt16Array>,
        cursor: &mut SortedBatchCursor,
    ) {
        Self::init_cursor_for_ids_column(&mut self.u16_ids, ids, cursor);
    }

    pub fn init_cursor_for_u32_id_column(
        &mut self,
        ids: &MaybeDictArrayAccessor<'_, UInt32Array>,
        cursor: &mut SortedBatchCursor,
    ) {
        Self::init_cursor_for_ids_column(&mut self.u32_ids, ids, cursor);
    }

    fn init_cursor_for_ids_column<T: ArrowPrimitiveType>(
        sort_ids_tmp: &mut Vec<(usize, T::Native)>,
        ids: &MaybeDictArrayAccessor<'_, PrimitiveArray<T>>,
        cursor: &mut SortedBatchCursor,
    ) where
        <T as ArrowPrimitiveType>::Native: Ord,
    {
        sort_ids_tmp.clear();

        match ids {
            MaybeDictArrayAccessor::Native(ids) => {
                sort_ids_tmp.extend(ids.values().iter().copied().enumerate());
            }
            MaybeDictArrayAccessor::Dictionary16(ids) => {
                sort_ids_tmp.extend(
                    (0..ids.len())
                        .map(|i| ids.value_at(i).unwrap_or_default())
                        .enumerate(),
                );
            }
            MaybeDictArrayAccessor::Dictionary8(ids) => {
                sort_ids_tmp.extend(
                    (0..ids.len())
                        .map(|i| ids.value_at(i).unwrap_or_default())
                        .enumerate(),
                );
            }
        }

        if ids.null_count() == 0 {
            // fast path, no null IDs
            sort_ids_tmp.sort_unstable_by_key(|&(_, value)| value);
        } else {
            // sort nulls last
            sort_ids_tmp.sort_unstable_by(|(ia, a), (ib, b)| {
                match (ids.is_valid(*ia), ids.is_valid(*ib)) {
                    (true, true) => a.cmp(b),
                    (true, false) => Ordering::Less,
                    (false, true) => Ordering::Greater,
                    (false, false) => Ordering::Equal,
                }
            });
        }

        cursor
            .sorted_indices
            .extend(sort_ids_tmp.iter().map(|(i, _)| *i));
    }
}

/// Iterates the indices of some child record batch
pub(crate) struct ChildIndexIter<'a, T: ArrowPrimitiveType> {
    pub parent_id: T::Native,
    pub parent_id_col: &'a MaybeDictArrayAccessor<'a, PrimitiveArray<T>>,
    pub cursor: &'a mut SortedBatchCursor,
}

impl<'a, T> ChildIndexIter<'a, T>
where
    T: ArrowPrimitiveType,
{
    pub fn new(
        parent_id: T::Native,
        parent_id_col: &'a MaybeDictArrayAccessor<'a, PrimitiveArray<T>>,
        cursor: &'a mut SortedBatchCursor,
    ) -> Self {
        Self {
            parent_id,
            parent_id_col,
            cursor,
        }
    }
}

impl<T> Iterator for ChildIndexIter<'_, T>
where
    T: ArrowPrimitiveType,
{
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        // advance the cursor until we either find the parent ID we're looking for, or pass it
        while !self.cursor.finished() {
            // safety: we've just checked that cursor is not finished
            let index = self.cursor.curr_index().expect("cursor not finished");

            if let Some(curr_parent_id) = self.parent_id_col.value_at(index) {
                if curr_parent_id < self.parent_id {
                    self.cursor.advance();
                }

                if curr_parent_id == self.parent_id {
                    self.cursor.advance();
                    return Some(index);
                }

                if curr_parent_id > self.parent_id {
                    return None;
                }
            } else {
                // skip the null values
                self.cursor.advance();
            }
        }

        // we've iterated the cursor until the end and didn't find what we were looking for:
        None
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use arrow::{
        array::{RecordBatch, StructArray, UInt16Array},
        datatypes::{DataType, Field, Fields, Schema, UInt16Type},
    };

    use crate::{
        arrays::MaybeDictArrayAccessor,
        otlp::common::{BatchSorter, ChildIndexIter, SortedBatchCursor},
        schema::consts,
    };

    #[test]
    fn test_child_index_iter_shuffled_order() {
        let mut cursor = SortedBatchCursor::new();
        let tmp = UInt16Array::from_iter_values(vec![2, 1, 2, 0]);
        let parent_ids = MaybeDictArrayAccessor::Native(&tmp);
        BatchSorter::new().init_cursor_for_u16_id_column(&parent_ids, &mut cursor);
        assert_eq!(cursor.sorted_indices, vec![3, 1, 0, 2]);

        {
            let mut id_0_iter = ChildIndexIter::new(0, &parent_ids, &mut cursor);
            assert_eq!(id_0_iter.next(), Some(3));
            assert_eq!(id_0_iter.next(), None)
        }

        {
            let mut id_1_iter = ChildIndexIter::new(1, &parent_ids, &mut cursor);
            assert_eq!(id_1_iter.next(), Some(1));
            assert_eq!(id_1_iter.next(), None)
        }

        {
            let mut id_2_iter = ChildIndexIter::new(2, &parent_ids, &mut cursor);
            assert_eq!(id_2_iter.next(), Some(0));
            assert_eq!(id_2_iter.next(), Some(2));
            assert_eq!(id_2_iter.next(), None)
        }

        {
            // check we don't try to iterate past the end
            let mut id_3_iter = ChildIndexIter::new(3, &parent_ids, &mut cursor);
            assert_eq!(id_3_iter.next(), None)
        }
    }

    #[test]
    fn test_child_index_iter_with_skipped_values() {
        let mut cursor = SortedBatchCursor::new();
        let tmp = UInt16Array::from_iter_values(vec![0, 2, 0, 2]);
        let parent_ids = MaybeDictArrayAccessor::Native(&tmp);
        BatchSorter::new().init_cursor_for_u16_id_column(&parent_ids, &mut cursor);
        assert_eq!(cursor.sorted_indices, vec![0, 2, 1, 3]);

        {
            let mut id_0_iter = ChildIndexIter::<UInt16Type>::new(0, &parent_ids, &mut cursor);
            assert_eq!(id_0_iter.next(), Some(0));
            assert_eq!(id_0_iter.next(), Some(2));
            assert_eq!(id_0_iter.next(), None)
        }

        {
            let mut id_1_iter = ChildIndexIter::<UInt16Type>::new(1, &parent_ids, &mut cursor);
            assert_eq!(id_1_iter.next(), None)
        }

        {
            let mut id_2_iter = ChildIndexIter::<UInt16Type>::new(2, &parent_ids, &mut cursor);
            assert_eq!(id_2_iter.next(), Some(1));
            assert_eq!(id_2_iter.next(), Some(3));
            assert_eq!(id_2_iter.next(), None)
        }
    }

    #[test]
    fn test_child_index_iter_with_nulls() {
        let mut cursor = SortedBatchCursor::new();
        let tmp = UInt16Array::from_iter(vec![Some(0), Some(2), None, Some(0), Some(1)]);
        let parent_ids = MaybeDictArrayAccessor::Native(&tmp);
        BatchSorter::new().init_cursor_for_u16_id_column(&parent_ids, &mut cursor);
        assert_eq!(cursor.sorted_indices, vec![0, 3, 4, 1, 2]);

        {
            let mut id_0_iter = ChildIndexIter::<UInt16Type>::new(0, &parent_ids, &mut cursor);
            assert_eq!(id_0_iter.next(), Some(0));
            assert_eq!(id_0_iter.next(), Some(3));
            assert_eq!(id_0_iter.next(), None)
        }

        {
            let mut id_1_iter = ChildIndexIter::<UInt16Type>::new(1, &parent_ids, &mut cursor);
            assert_eq!(id_1_iter.next(), Some(4));
            assert_eq!(id_1_iter.next(), None)
        }

        {
            let mut id_2_iter = ChildIndexIter::<UInt16Type>::new(2, &parent_ids, &mut cursor);
            assert_eq!(id_2_iter.next(), Some(1));
            assert_eq!(id_2_iter.next(), None)
        }
    }

    #[test]
    fn test_batch_sorter_reuse_rows_alloc() {
        // test that we're able to reuse the batch sorter's 'rows' heap allocation
        // between sortings -- e.g. we're trying to test the functionality of
        // BatchSorter::reuse_rows

        let struct_fields = Fields::from(vec![Field::new(consts::ID, DataType::UInt16, true)]);
        let record_batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(
                    consts::RESOURCE,
                    DataType::Struct(struct_fields.clone()),
                    false,
                ),
                Field::new(
                    consts::SCOPE,
                    DataType::Struct(struct_fields.clone()),
                    false,
                ),
            ])),
            vec![
                Arc::new(StructArray::new(
                    struct_fields.clone(),
                    vec![Arc::new(UInt16Array::from_iter_values(vec![2, 1, 2, 0]))],
                    None,
                )),
                Arc::new(StructArray::new(
                    struct_fields,
                    vec![Arc::new(UInt16Array::from_iter_values(vec![1, 0, 1, 0]))],
                    None,
                )),
            ],
        )
        .unwrap();

        let mut cursor = SortedBatchCursor::new();
        let mut batch_sorter = BatchSorter::new();
        // call once ot allocate the vec
        batch_sorter
            .init_cursor_for_root_batch(&record_batch, &mut cursor)
            .unwrap();

        // the vec should have enough capacity not to get reallocated and we reuse it
        let rows_ptr_before = batch_sorter.rows.as_ptr();
        batch_sorter
            .init_cursor_for_root_batch(&record_batch, &mut cursor)
            .unwrap();
        assert_eq!(rows_ptr_before, batch_sorter.rows.as_ptr());
    }
}
