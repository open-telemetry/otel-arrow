// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::arrays::{
    ByteArrayAccessor, Int64ArrayAccessor, MaybeDictArrayAccessor, NullableArrayAccessor,
    StringArrayAccessor, StructColumnAccessor, get_bool_array_opt, get_f64_array_opt, get_u8_array,
};
use crate::error::{Error, Result};
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

use bytes::Bytes;
use std::cmp::Ordering;
use std::fmt;
use std::fmt::Write;
use std::sync::LazyLock;

pub(crate) struct ResourceArrays<'a> {
    pub id: Option<&'a UInt16Array>,
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
        // Resource column is optional - if it's missing, return all None values
        let resource_column = rb.column_by_name(consts::RESOURCE);

        let struct_array = if let Some(resource_column) = resource_column {
            resource_column
        } else {
            return Ok(Self {
                id: None,
                dropped_attributes_count: None,
                schema_url: None,
            });
        };

        let struct_array = struct_array
            .as_any()
            .downcast_ref::<StructArray>()
            .ok_or_else(|| Error::ColumnDataTypeMismatch {
                name: consts::RESOURCE.into(),
                actual: struct_array.data_type().clone(),
                expect: Self::data_type().clone(),
            })?;

        let struct_col_accessor = StructColumnAccessor::new(struct_array);

        Ok(Self {
            id: struct_col_accessor.primitive_column_op(consts::ID)?,
            dropped_attributes_count: struct_col_accessor
                .primitive_column_op(consts::DROPPED_ATTRIBUTES_COUNT)?,
            schema_url: struct_col_accessor.string_column_op(consts::SCHEMA_URL)?,
        })
    }
}

pub(crate) struct ScopeArrays<'a> {
    pub name: Option<StringArrayAccessor<'a>>,
    pub version: Option<StringArrayAccessor<'a>>,
    pub dropped_attributes_count: Option<&'a UInt32Array>,
    pub id: Option<&'a UInt16Array>,
}

static SCOPE_ARRAY_DATA_TYPE: LazyLock<DataType> = LazyLock::new(|| {
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
        // Scope column is optional - if it's missing, return all None values
        let scope_column = rb.column_by_name(consts::SCOPE);

        let struct_array = if let Some(scope_column) = scope_column {
            scope_column
        } else {
            return Ok(Self {
                name: None,
                version: None,
                dropped_attributes_count: None,
                id: None,
            });
        };

        let scope_array = struct_array
            .as_any()
            .downcast_ref::<StructArray>()
            .ok_or_else(|| Error::ColumnDataTypeMismatch {
                name: consts::SCOPE.into(),
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

/// Buffer for encoding protobuf bytes.
#[derive(Debug, Default)]
pub struct ProtoBuffer {
    buffer: Vec<u8>,
}

impl ProtoBuffer {
    /// Construct a new, empty protocol buffer.
    #[must_use]
    pub const fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    /// Construct a new buffer with at least the provided capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
        }
    }

    /// Returns a Bytes representation.
    #[must_use]
    pub fn into_bytes(self) -> Bytes {
        Bytes::from(self.buffer)
    }

    /// Encodes a varint containing type (3 bits) and tag value.
    pub fn encode_field_tag(&mut self, field_number: u64, wire_type: u64) {
        let key = (field_number << 3) | wire_type;
        self.encode_varint(key);
    }

    /// An unsigned varint encoding.
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

    /// Append pre-encoded protocol bytes.
    pub fn extend_from_slice(&mut self, slice: &[u8]) {
        self.buffer.extend_from_slice(slice);
    }

    /// Length of the current encoding.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Returns the current capacity of the underlying buffer.
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.buffer.capacity()
    }

    /// Is the buffer empty?
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Reset the buffer.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Encode a string field by tag number.
    pub fn encode_string(&mut self, field_tag: u64, val: &str) {
        self.encode_field_tag(field_tag, wire_types::LEN);
        self.encode_varint(val.len() as u64);
        self.extend_from_slice(val.as_bytes());
    }

    /// Encode a bytes field by tag number.
    pub fn encode_bytes(&mut self, field_tag: u64, val: &[u8]) {
        self.encode_field_tag(field_tag, wire_types::LEN);
        self.encode_varint(val.len() as u64);
        self.extend_from_slice(val);
    }

    /// Take the encoded bytes, returning them as `Bytes`, and reserve the original capacity.
    /// This lets callers reuse the same buffer (growth preserved) without a second temporary.
    pub fn take_into_bytes(&mut self) -> (Bytes, usize) {
        let buffer = std::mem::take(&mut self.buffer);
        let capacity = buffer.capacity();
        (Bytes::from(buffer), capacity)
    }

    /// Ensure the underlying buffer has at least the requested capacity.
    pub fn ensure_capacity(&mut self, capacity: usize) {
        if capacity > self.buffer.capacity() {
            self.buffer.reserve(capacity - self.buffer.capacity());
        }
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

impl std::io::Write for ProtoBuffer {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Helper for encoding with const length.
///
/// Our proto encoding algorithm tries to encode in a single pass over the OTAP data, but it will
/// not know the size of the nested child messages a priori. Because the length fields are encoded
/// in a varint, we don't know how many bytes we need to set aside for the length before we start
/// appending the encoded child.
///
/// An optional fourth argument specifies how many bytes to reserve for the length placeholder.
/// An N-byte placeholder supports sizes up to 2^(8*n-1) bytes: 1=127, 2=16KiB, 3=2MiB, 4=256MiB
#[macro_export]
macro_rules! proto_encode_len_delimited_of_size {
    ($field_tag: expr, $encode_fn:expr, $buf:expr, $placeholder_bytes:literal) => {{
        $buf.encode_field_tag($field_tag, $crate::proto::consts::wire_types::LEN);
        let len_start_pos = $buf.len();
        $crate::otlp::common::encode_len_placeholder::<$placeholder_bytes>($buf);
        $encode_fn;
        let len = $buf.len() - len_start_pos - $placeholder_bytes;
        $crate::otlp::common::patch_len_placeholder::<$placeholder_bytes>($buf, len, len_start_pos);
    }};
}

/// Deprecated form of helper proto_encode_len_delimited_of_size with 4 byte placeholder.
/// Callers should use the _small or _large variations directly.
#[macro_export]
macro_rules! proto_encode_len_delimited_unknown_size {
    ($field_tag: expr, $encode_fn:expr, $buf:expr) => {{ $crate::proto_encode_len_delimited_large!($field_tag, $encode_fn, $buf) }};
}

/// 2-byte placeholder for messages up to 16KiB.
///
/// Use this for encoding into small, bounded buffers.
/// See [`proto_encode_len_delimited_of_size!`] for details.
#[macro_export]
macro_rules! proto_encode_len_delimited_small {
    ($field_tag: expr, $encode_fn:expr, $buf:expr) => {{ $crate::proto_encode_len_delimited_of_size!($field_tag, $encode_fn, $buf, 2) }};
}

/// 4-byte placeholder for messages up to 256MiB.
///
/// Use this for encoding into arbitrary-size messages.
/// See [`proto_encode_len_delimited_of_size!`] for details.
#[macro_export]
macro_rules! proto_encode_len_delimited_large {
    ($field_tag: expr, $encode_fn:expr, $buf:expr) => {{ $crate::proto_encode_len_delimited_of_size!($field_tag, $encode_fn, $buf, 4) }};
}

/// Write an `N`-byte length placeholder for later patching.
///
/// Each byte except the last has its continuation bit (MSB) set, with all
/// value bits zeroed. `N` must be between 1 and 4 inclusive (enforced at
/// compile time).
///
/// Do not call directly — use [`proto_encode_len_delimited_of_size!`].
#[inline]
pub fn encode_len_placeholder<const N: usize>(buf: &mut ProtoBuffer) {
    const { assert!(N >= 1 && N <= 4, "placeholder must be 1-4 bytes") }
    buf.buffer.extend_from_slice(&make_len_placeholder::<N>());
}

/// Build an `N`-byte zero-valued varint placeholder at compile time.
const fn make_len_placeholder<const N: usize>() -> [u8; N] {
    let mut arr = [0x80u8; N];
    arr[N - 1] = 0x00;
    arr
}

/// Patch a previously written length placeholder with the actual length.
///
/// Do not call directly — use [`proto_encode_len_delimited_of_size!`].
#[inline]
pub fn patch_len_placeholder<const N: usize>(
    buf: &mut ProtoBuffer,
    len: usize,
    len_start_pos: usize,
) {
    for i in 0..N {
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
    pub const fn new() -> Self {
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
    pub const fn advance(&mut self) {
        self.curr_index += 1;
    }

    /// Check if the cursor has finished. This will return true once we've iterated to the end of
    /// the record batch that was used to initialize this cursor.
    pub const fn finished(&self) -> bool {
        self.curr_index >= self.sorted_indices.len()
    }
}

/// This is used to initialize the [`SortedBatchCursor`]. It does this by sorting the IDs and then
/// filling in the cursors indices to visit based on the sorted ID column.
pub(crate) struct BatchSorter {
    // when we sort the record batch, to it's indices we put the ID columns into these Vecs before
    // transferring the indices to the cursor. We keep these on instance of the [`BatchSorter`] so
    // we can reuse the allocations for multiple batches.
    u16_ids: Vec<(usize, u16)>,
    u32_ids: Vec<(usize, u32)>,

    // note: in OTAP we don't have u64 ID columns. This vec is used to pack multiple IDs together
    // while doing a multi-column sort
    u64_ids: Vec<(usize, u64)>,
}

impl BatchSorter {
    pub fn new() -> Self {
        Self {
            u16_ids: Vec::new(),
            u32_ids: Vec::new(),
            u64_ids: Vec::new(),
        }
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
        let mut sort_columns: [Option<&UInt16Array>; 3] = [None, None, None];
        for col_name in [consts::RESOURCE, consts::SCOPE] {
            if let Some(struct_col) = record_batch.column_by_name(col_name) {
                let struct_array = struct_col
                    .as_any()
                    .downcast_ref::<StructArray>()
                    .ok_or_else(|| Error::ColumnDataTypeMismatch {
                        name: col_name.into(),
                        expect: DataType::Struct(Fields::empty()),
                        actual: struct_col.data_type().clone(),
                    })?;
                if let Some(ids) = struct_array.column_by_name(consts::ID) {
                    sort_columns[sort_columns_idx] =
                        Some(ids.as_any().downcast_ref::<UInt16Array>().ok_or_else(|| {
                            Error::ColumnDataTypeMismatch {
                                name: col_name.into(),
                                expect: DataType::UInt16,
                                actual: ids.data_type().clone(),
                            }
                        })?);
                    sort_columns_idx += 1;
                }
            }
        }

        if let Some(ids) = record_batch.column_by_name(consts::ID) {
            sort_columns[sort_columns_idx] =
                Some(ids.as_any().downcast_ref::<UInt16Array>().ok_or_else(|| {
                    Error::ColumnDataTypeMismatch {
                        name: consts::ID.into(),
                        expect: DataType::UInt16,
                        actual: ids.data_type().clone(),
                    }
                })?);
        }

        match sort_columns {
            // Pack 3 u16 IDs into a u64 for composite sorting by resource, scope, then row ID.
            [Some(resource_ids), Some(scope_ids), Some(ids)] => {
                self.u64_ids.clear();
                self.u64_ids.extend(
                    resource_ids
                        .values()
                        .iter()
                        .zip(scope_ids.values().iter())
                        .zip(ids.values().iter())
                        .enumerate()
                        .map(|(i, ((r, s), id))| {
                            (i, (*r as u64) << 32 | (*s as u64) << 16 | *id as u64)
                        }),
                );
                self.u64_ids.sort_unstable_by_key(|&(_, v)| v);
                cursor
                    .sorted_indices
                    .extend(self.u64_ids.iter().map(|(i, _)| *i));
            }
            // Pack 2 u16 IDs into a u32 for composite sorting by resource, then scope ID.
            [Some(ids1), Some(ids2), None] => {
                self.u32_ids.clear();
                self.u32_ids.extend(
                    ids1.values()
                        .iter()
                        .zip(ids2.values().iter())
                        .enumerate()
                        .map(|(i, (a, b))| (i, (*a as u32) << 16 | *b as u32)),
                );
                self.u32_ids.sort_unstable_by_key(|&(_, v)| v);
                cursor
                    .sorted_indices
                    .extend(self.u32_ids.iter().map(|(i, _)| *i));
            }
            // Single ID column — sort directly by u16 value.
            [Some(ids), None, None] => {
                self.init_cursor_for_u16_id_column(&MaybeDictArrayAccessor::Native(ids), cursor);
            }
            // No ID columns — visit in natural order.
            [None, None, None] => {
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
    pub const fn new(
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
        proto::OtlpProtoMessage,
        schema::consts,
        testing::{fixtures::*, round_trip::*},
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
    fn test_batch_sorter_reuse_alloc() {
        // test that we're able to reuse the batch sorter's sort vec heap allocation
        // between sortings

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
        // call once to allocate the vec
        batch_sorter
            .init_cursor_for_root_batch(&record_batch, &mut cursor)
            .unwrap();

        // the vec should have enough capacity not to get reallocated and we reuse it
        let ids_ptr_before = batch_sorter.u32_ids.as_ptr();
        batch_sorter
            .init_cursor_for_root_batch(&record_batch, &mut cursor)
            .unwrap();
        assert_eq!(ids_ptr_before, batch_sorter.u32_ids.as_ptr());
    }

    //
    // Logs Tests
    //

    #[test]
    fn test_logs_with_full_resource_and_scope() {
        test_logs_round_trip(logs_with_full_resource_and_scope());
    }

    #[test]
    fn test_logs_with_no_resource() {
        test_logs_round_trip(logs_with_no_resource());
    }

    #[test]
    fn test_logs_with_no_scope() {
        test_logs_round_trip(log_with_no_scope());
    }

    #[test]
    fn test_logs_with_no_attributes() {
        test_logs_round_trip(logs_with_no_attributes());
    }

    #[test]
    fn test_logs_with_no_resource_no_scope() {
        test_logs_round_trip(logs_with_no_resource_no_scope());
    }

    #[test]
    fn test_logs_multiple_records_no_resource() {
        test_logs_round_trip(logs_multiple_records_no_resource());
    }

    #[test]
    fn test_logs_multiple_scopes_no_resource() {
        test_logs_round_trip(logs_multiple_scopes_no_resource());
    }

    #[test]
    fn test_logs_multiple_resources_mixed_content() {
        test_logs_round_trip(logs_multiple_resources_mixed_content());
    }

    #[test]
    fn test_logs_with_empty_log_records() {
        test_logs_round_trip(logs_with_empty_log_records());
    }

    #[test]
    fn test_logs_with_body_empty_string() {
        test_logs_round_trip(logs_with_body_empty_string());
    }

    //
    // Traces Tests
    //

    #[test]
    fn test_traces_with_full_resource_and_scope() {
        test_traces_round_trip(traces_with_full_resource_and_scope());
    }

    #[test]
    fn test_traces_with_no_resource() {
        test_traces_round_trip(traces_with_no_resource());
    }

    #[test]
    fn test_traces_with_no_scope() {
        test_traces_round_trip(traces_with_no_scope());
    }

    #[test]
    fn test_traces_with_no_attributes() {
        test_traces_round_trip(traces_with_no_attributes());
    }

    #[test]
    fn test_traces_with_no_resource_no_scope() {
        test_traces_round_trip(traces_with_no_resource_no_scope());
    }

    #[test]
    fn test_traces_multiple_spans_no_resource() {
        test_traces_round_trip(traces_multiple_spans_no_resource());
    }

    #[test]
    fn test_traces_multiple_scopes_no_resource() {
        test_traces_round_trip(traces_multiple_scopes_no_resource());
    }

    #[test]
    fn test_traces_multiple_resources_mixed_content() {
        test_traces_round_trip(traces_multiple_resources_mixed_content());
    }

    //
    // Metrics Tests
    //

    #[test]
    fn test_metrics_sum_with_full_resource_and_scope() {
        test_metrics_round_trip(metrics_sum_with_full_resource_and_scope());
    }

    #[test]
    fn test_metrics_sum_with_no_resource() {
        test_metrics_round_trip(metrics_sum_with_no_resource());
    }

    #[test]
    fn test_metrics_sum_with_no_scope() {
        test_metrics_round_trip(metrics_sum_with_no_scope());
    }

    #[test]
    fn test_metrics_sum_with_no_resource_no_scope() {
        test_metrics_round_trip(metrics_sum_with_no_resource_no_scope());
    }

    #[test]
    fn test_metrics_sum_with_no_data_points() {
        test_metrics_round_trip(metrics_sum_with_no_data_points());
    }

    #[test]
    fn test_metrics_multiple_sums_no_resource() {
        test_metrics_round_trip(metrics_multiple_sums_no_resource());
    }

    //
    // Empty encoding tests
    //

    /// OpenTelemetry data may contain "empty envelopes". This checks that
    /// they encode to an empty OTAP encoding.
    fn assert_empty_batch(msg: OtlpProtoMessage) {
        let encoded = otlp_to_otap(&msg);
        assert_eq!(encoded.num_items(), 0, "Expected an empty batch");
    }

    #[test]
    fn test_empty_logs() {
        assert_empty_batch(empty_logs().into());
    }

    #[test]
    fn test_logs_with_empty_scope_logs() {
        assert_empty_batch(logs_with_empty_scope_logs().into());
    }

    #[test]
    fn test_empty_traces() {
        assert_empty_batch(empty_traces().into());
    }

    #[test]
    fn test_traces_with_empty_scope_spans() {
        assert_empty_batch(traces_with_empty_scope_spans().into());
    }

    #[test]
    fn test_traces_with_empty_spans() {
        assert_empty_batch(traces_with_empty_spans().into());
    }

    #[test]
    fn test_empty_metrics() {
        assert_empty_batch(empty_metrics().into());
    }

    #[test]
    fn test_metrics_with_no_scope_metrics() {
        assert_empty_batch(metrics_with_no_scope_metrics().into());
    }

    #[test]
    fn test_metrics_with_no_metrics() {
        assert_empty_batch(metrics_with_no_metrics().into());
    }

    #[test]
    fn test_encode_len_placeholder_sizes() {
        use crate::otlp::common::{ProtoBuffer, encode_len_placeholder};

        fn check<const N: usize>() {
            let mut buf = ProtoBuffer::new();
            encode_len_placeholder::<N>(&mut buf);
            assert_eq!(buf.len(), N);
            for i in 0..N - 1 {
                assert_eq!(buf.as_ref()[i], 0x80, "byte {i} for {N}-byte placeholder");
            }
            assert_eq!(buf.as_ref()[N - 1], 0x00);
        }

        check::<1>();
        check::<2>();
        check::<3>();
        check::<4>();
    }

    #[test]
    fn test_patch_len_placeholder_roundtrip() {
        use crate::otlp::common::{ProtoBuffer, encode_len_placeholder, patch_len_placeholder};

        // Verify that encoding+patching with each placeholder size produces
        // valid varints that a standard decoder would read correctly.
        fn check<const N: usize>(test_lengths: &[usize]) {
            let max_len = (1usize << (7 * N)) - 1;
            for &len in test_lengths {
                if len > max_len {
                    continue;
                }
                let mut buf = ProtoBuffer::new();
                let start = buf.len();
                encode_len_placeholder::<N>(&mut buf);
                patch_len_placeholder::<N>(&mut buf, len, start);

                let mut decoded: u64 = 0;
                for i in 0..N {
                    decoded |= ((buf.as_ref()[start + i] & 0x7f) as u64) << (i * 7);
                }
                assert_eq!(decoded, len as u64, "N={N}, expected len={len}");
            }
        }

        let lengths: &[usize] = &[0, 1, 42, 127, 128, 1000, 16383];
        check::<1>(lengths);
        check::<2>(lengths);
        check::<3>(lengths);
        check::<4>(lengths);
    }

    #[test]
    fn test_macro_with_custom_placeholder_size() {
        use crate::otlp::common::ProtoBuffer;

        // Encode a simple string field using the _large and _small variants,
        // then verify the _small output is 2 bytes shorter.
        let mut buf_large = ProtoBuffer::new();
        proto_encode_len_delimited_large!(
            1,
            {
                buf_large.encode_string(1, "hello");
            },
            &mut buf_large
        );

        let mut buf_small = ProtoBuffer::new();
        proto_encode_len_delimited_small!(
            1,
            {
                buf_small.encode_string(1, "hello");
            },
            &mut buf_small
        );

        // The _small variant should be exactly 2 bytes shorter.
        assert_eq!(buf_large.len() - buf_small.len(), 2);

        // Both should decode to valid protobuf with the same payload.
        // Skip the outer tag+len to compare just the inner content.
        // large: 1 byte tag + 4 byte len + payload
        // small: 1 byte tag + 2 byte len + payload
        assert_eq!(&buf_large.as_ref()[5..], &buf_small.as_ref()[3..]);
    }
}
