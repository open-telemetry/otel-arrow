// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// TODO(4482): This file is disorganized and should be separated into smaller
// pieces; at least 50% of the file is encoder support but the other half
// quite separate. Reorganize this in a separate PR.

use crate::arrays::{
    ByteArrayAccessor, Int64ArrayAccessor, MaybeDictArrayAccessor, NullableArrayAccessor,
    StringArrayAccessor, StructColumnAccessor, get_bool_array_opt, get_f64_array_opt, get_u8_array,
};
use crate::error::{Error, Result as PDataResult};
use crate::otlp::attributes::{Attribute16Arrays, encode_attribute};
use crate::proto::consts::field_num::common::{
    INSTRUMENTATION_DROPPED_ATTRIBUTES_COUNT, INSTRUMENTATION_SCOPE_ATTRIBUTES,
    INSTRUMENTATION_SCOPE_NAME, INSTRUMENTATION_SCOPE_VERSION,
};
use crate::proto::consts::field_num::resource::{
    RESOURCE_ATTRIBUTES, RESOURCE_DROPPED_ATTRIBUTES_COUNT,
};
use crate::proto::consts::wire_types;
use crate::proto::opentelemetry::common::v1::{AnyValue, any_value::Value};
use crate::schema::consts;
use arrow::array::{
    Array, ArrowPrimitiveType, BooleanArray, Float64Array, PrimitiveArray, RecordBatch,
    StructArray, UInt8Array, UInt16Array, UInt32Array,
};
use arrow::datatypes::{DataType, Field, Fields};
use otel_arrow_dfe_config::ConversionOptions;

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
) -> PDataResult<()> {
    // add attributes
    if let Some(attrs_arrays) = resource_attrs_arrays
        && let Some(res_id) = resource_arrays.id.value_at(index)
    {
        for attr_index in
            ChildIndexIter::new(res_id, &attrs_arrays.parent_id, resource_attrs_cursor)
        {
            encode_attribute(RESOURCE_ATTRIBUTES, attrs_arrays, attr_index, result_buf)?;
        }
    }

    if let Some(col) = resource_arrays.dropped_attributes_count
        && let Some(val) = col.value_at(index)
    {
        result_buf.encode_field_tag(RESOURCE_DROPPED_ATTRIBUTES_COUNT, wire_types::VARINT)?;
        result_buf.encode_varint(val as u64)?;
    }

    Ok(())
}

impl<'a> TryFrom<&'a RecordBatch> for ResourceArrays<'a> {
    type Error = Error;

    fn try_from(rb: &'a RecordBatch) -> PDataResult<Self> {
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

    fn try_from(rb: &'a RecordBatch) -> PDataResult<Self> {
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
) -> PDataResult<()> {
    if let Some(col) = &scope_arrays.name
        && let Some(val) = col.str_at(index)
    {
        result_buf.encode_string(INSTRUMENTATION_SCOPE_NAME, val)?;
    }

    if let Some(col) = &scope_arrays.version
        && let Some(val) = col.str_at(index)
    {
        result_buf.encode_string(INSTRUMENTATION_SCOPE_VERSION, val)?;
    }

    if let Some(attr_arrays) = scope_attrs_arrays
        && let Some(scope_id) = scope_arrays.id.value_at(index)
    {
        for attr_index in ChildIndexIter::new(scope_id, &attr_arrays.parent_id, scope_attrs_cursor)
        {
            encode_attribute(
                INSTRUMENTATION_SCOPE_ATTRIBUTES,
                attr_arrays,
                attr_index,
                result_buf,
            )?;
        }
    }

    if let Some(col) = scope_arrays.dropped_attributes_count
        && let Some(val) = col.value_at(index)
    {
        result_buf
            .encode_field_tag(INSTRUMENTATION_DROPPED_ATTRIBUTES_COUNT, wire_types::VARINT)?;
        result_buf.encode_varint(val as u64)?;
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

/// Borrowed column accessors for an OTAP `AnyValue` representation, extracted from either
/// an attribute [`RecordBatch`] or a nested [`StructArray`] such as `Logs.body`.
///
/// Value columns are optional because OTAP encoders omit columns whose values are all null or
/// all defaults (`default_values_optional`).
///
/// When extracted from a nullable [`StructArray`] (such as `Logs.body`), `AnyValueArrays`
/// borrows only the child columns and does not carry the parent struct's validity bitmap;
/// callers must check `struct_array.is_valid(row)` before reading row `row`.
pub struct AnyValueArrays<'a> {
    /// Required `type` discriminant column (`AttributeValueType` values).
    pub attr_type: &'a UInt8Array,
    /// Optional UTF-8 string value column (`str`), native or dictionary-encoded.
    pub attr_str: Option<StringArrayAccessor<'a>>,
    /// Optional 64-bit signed integer value column (`int`), native or dictionary-encoded.
    pub attr_int: Option<Int64ArrayAccessor<'a>>,
    /// Optional 64-bit floating-point value column (`double`).
    pub attr_double: Option<&'a Float64Array>,
    /// Optional boolean value column (`bool`).
    pub attr_bool: Option<&'a BooleanArray>,
    /// Optional raw byte value column (`bytes`), native or dictionary-encoded.
    pub attr_bytes: Option<ByteArrayAccessor<'a>>,
    /// Optional CBOR-serialized `Map`/`Slice` value column (`ser`), native or dictionary-encoded.
    pub attr_ser: Option<ByteArrayAccessor<'a>>,
}

impl<'a> TryFrom<&'a RecordBatch> for AnyValueArrays<'a> {
    type Error = Error;

    fn try_from(rb: &'a RecordBatch) -> PDataResult<Self> {
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

impl<'a> TryFrom<&'a StructArray> for AnyValueArrays<'a> {
    type Error = Error;

    fn try_from(struct_array: &'a StructArray) -> PDataResult<Self> {
        let column_accessor = StructColumnAccessor::new(struct_array);

        Ok(Self {
            attr_type: column_accessor.primitive_column(consts::ATTRIBUTE_TYPE)?,
            attr_str: column_accessor.string_column_op(consts::ATTRIBUTE_STR)?,
            attr_int: column_accessor.int64_column_op(consts::ATTRIBUTE_INT)?,
            attr_double: column_accessor.primitive_column_op(consts::ATTRIBUTE_DOUBLE)?,
            attr_bool: column_accessor.bool_column_op(consts::ATTRIBUTE_BOOL)?,
            attr_bytes: column_accessor.byte_array_column_op(consts::ATTRIBUTE_BYTES)?,
            attr_ser: column_accessor.byte_array_column_op(consts::ATTRIBUTE_SER)?,
        })
    }
}

/// Returned when a buffer write cannot fit within the size limit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EncodeFailure {
    /// Dropped due to space limit.
    Dropped,
}

impl std::error::Error for EncodeFailure {}

impl fmt::Display for EncodeFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dropped => write!(f, "dropped"),
        }
    }
}

/// Result type for encoding methods that can fail only because of a size limit.
pub type EncodeResult<T = ()> = Result<T, EncodeFailure>;

const fn tag_width_limit(tagw: usize) -> usize {
    1 << (7 * tagw)
}

const MAX_TAG_WIDTH: usize = 4;

/// The maximum supported protocol buffer size limit (1 << 28) = 256MiB.
pub const MAX_OTLP_SIZE_LIMIT: usize = tag_width_limit(MAX_TAG_WIDTH);

/// Build an `N`-byte zero-valued varint placeholder at compile time.
const fn make_len_placeholder<const N: usize>() -> [u8; N] {
    let mut arr = [0x80u8; N];
    arr[N - 1] = 0x00;
    arr
}

/// Trait for buffers that support bounded protobuf encoding.
///
/// The trait exposes a small set of storage primitives; all higher-level
/// encoding methods are provided as default methods so that two concrete
/// storage strategies (heap-backed [`ProtoBuffer`] and stack-backed
/// [`StackProtoBuffer`]) can share the encoding logic.
///
/// Write methods return [`EncodeResult`], enabling `?` propagation through
/// recursive protobuf encoding. Callers are responsible for counting
/// dropped fields. The [`try_atomic`](Self::try_atomic) method provides
/// atomic field-level writes: either the entire field is written, or the
/// buffer is rolled back to its previous state.
///
/// TODO(4482): This interface is too wide: it feaures public methods
/// as well as ~5 internal methods that could become either private
/// free functions or part of an internal storage trait.
pub trait BoundedBuf {
    /// Current write position (encoded byte count).
    fn len(&self) -> usize;

    /// Maximum byte count the buffer will accept.
    fn limit(&self) -> usize;

    /// Set a new write limit.
    /// TODO(4482): Can be a private storage trait and pub fn on the impls.
    fn set_limit(&mut self, limit: usize);

    /// Append bytes if they fit within the limit; otherwise return `Err(Dropped)`
    /// without writing any bytes.
    /// TODO(4482): Note that extend_from_slice() is a deprecated alias for
    //// this method.
    fn try_extend_slice(&mut self, slice: &[u8]) -> EncodeResult;

    /// Truncate the buffer to the given position. Used for rollback.
    /// TODO(4482): Can be an private storage trait.
    fn truncate(&mut self, pos: usize);

    /// Borrow the encoded bytes.
    fn as_slice(&self) -> &[u8];

    /// Borrow the encoded bytes mutably (used by [`patch_len_placeholder`]).
    /// TODO(4482): Can be a private storage trait.
    fn as_mut_slice(&mut self) -> &mut [u8];

    /// Reset the buffer to empty (preserves any underlying allocation).
    fn clear(&mut self);

    /// Returns true if no bytes have been written.
    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of bytes remaining before the limit.
    #[inline]
    fn remaining(&self) -> usize {
        self.limit().saturating_sub(self.len())
    }

    /// TODO(4482): This is an alias, and it is harmful because Vec's
    /// extend_from_slice() is infallible, however a wide impact fix.
    #[inline]
    fn extend_from_slice(&mut self, slice: &[u8]) -> EncodeResult {
        self.try_extend_slice(slice)
    }

    /// Encodes a varint containing type (3 bits) and tag value.
    #[inline]
    fn encode_field_tag(&mut self, tag: u64, wire_type: u64) -> EncodeResult {
        let key = (tag << 3) | wire_type;
        self.encode_varint(key)
    }

    /// An unsigned varint encoding.
    #[inline(always)]
    fn encode_varint(&mut self, value: u64) -> EncodeResult {
        if value < tag_width_limit(1) as u64 {
            return self.try_extend_slice(&[value as u8]);
        }
        if value < tag_width_limit(2) as u64 {
            return self.try_extend_slice(&[((value & 0x7F) | 0x80) as u8, (value >> 7) as u8]);
        }
        let mut tmp = [0u8; 10];
        let mut i = 0;
        let mut v = value;
        while v >= 0x80 {
            tmp[i] = ((v & 0x7F) | 0x80) as u8;
            i += 1;
            v >>= 7;
        }
        tmp[i] = v as u8;
        i += 1;
        self.try_extend_slice(&tmp[..i])
    }

    /// Encodes the signed varint type (e.g. sint32, sint64) using zig-zag encoding.
    /// <https://protobuf.dev/programming-guides/encoding/#signed-ints>
    #[inline]
    fn encode_sint32(&mut self, value: i32) -> EncodeResult {
        self.encode_varint(((value << 1) ^ (value >> 31)) as u64)
    }

    /// Encode a string field by tag number.
    #[inline]
    fn encode_string(&mut self, tag: u64, val: &str) -> EncodeResult {
        self.encode_field_tag(tag, wire_types::LEN)?;
        self.encode_varint(val.len() as u64)?;
        self.try_extend_slice(val.as_bytes())
    }

    /// Encode a bytes field by tag number.
    #[inline]
    fn encode_bytes(&mut self, tag: u64, val: &[u8]) -> EncodeResult {
        self.encode_field_tag(tag, wire_types::LEN)?;
        self.encode_varint(val.len() as u64)?;
        self.try_extend_slice(val)
    }

    /// Returns the narrowest placeholder width sufficient for this
    /// buffer's remaining space.
    /// TODO(4482): Can be a private free function.
    #[inline]
    fn placeholder_width(&self) -> usize {
        let rem = self.remaining();
        if rem < tag_width_limit(1) {
            1
        } else if rem < tag_width_limit(2) {
            2
        } else if rem < tag_width_limit(3) {
            3
        } else {
            4
        }
    }

    /// Encode a length-delimited field with variable-width placeholder. Does not
    /// roll-backup; supports truncation or not.
    #[inline]
    fn encode_len_delimited<T, E: From<EncodeFailure>>(
        &mut self,
        tag: u64,
        f: impl FnOnce(&mut Self) -> Result<T, E>,
    ) -> Result<T, E>
    where
        Self: Sized,
    {
        match self.placeholder_width() {
            1 => self.encode_len_delimited_width::<1, _, E, _>(tag, f),
            2 => self.encode_len_delimited_width::<2, _, E, _>(tag, f),
            3 => self.encode_len_delimited_width::<3, _, E, _>(tag, f),
            _ => self.encode_len_delimited_width::<4, _, E, _>(tag, f),
        }
    }

    /// Implementation of encode_len_delimited with an explicit
    /// placeholder width. Supports TruncateResult or EncodeResult.
    /// TODO(4482): Can be a private free function.
    #[inline]
    fn encode_len_delimited_width<const B: usize, T, E, F>(
        &mut self,
        tag: u64,
        f: F,
    ) -> Result<T, E>
    where
        E: From<EncodeFailure>,
        F: FnOnce(&mut Self) -> Result<T, E>,
        Self: Sized,
    {
        self.encode_field_tag(tag, wire_types::LEN)?;
        let len_start_pos = self.len();
        self.try_extend_slice(&make_len_placeholder::<B>())?;
        let ok = f(self)?;
        let len = self.len() - len_start_pos - B;
        patch_len_placeholder::<B, _>(self, len, len_start_pos);
        Ok(ok)
    }

    /// Run a closure with a temporarily reduced limit.
    ///
    /// The buffer's `limit` is set to `min(current_limit, len() + max_remaining)`
    /// for the duration of the closure, and restored afterward. This lets
    /// callers cap how many bytes a single nested encoding may consume,
    /// without touching the underlying buffer's outer limit.
    #[inline]
    fn with_max_remaining<R>(&mut self, max_remaining: usize, f: impl FnOnce(&mut Self) -> R) -> R
    where
        Self: Sized,
    {
        let saved = self.limit();
        let scoped = self.len().saturating_add(max_remaining).min(saved);
        self.set_limit(scoped);
        let result = f(self);
        self.set_limit(saved);
        result
    }

    /// Try to encode a complete field atomically.
    ///
    /// If the closure returns an error, the buffer is rolled back to
    /// its state before the call and the error is returned.
    #[inline]
    fn try_atomic<T, E>(&mut self, f: impl FnOnce(&mut Self) -> Result<T, E>) -> Result<T, E>
    where
        Self: Sized,
    {
        let cp = self.len();
        match f(self) {
            Ok(t) => Ok(t),
            Err(e) => {
                self.truncate(cp);
                Err(e)
            }
        }
    }
}

/// Heap-backed protobuf encoding buffer.
///
/// Backed by a `Vec<u8>`. Use [`with_capacity`](Self::with_capacity) or
/// [`with_capacity_and_limit`](Self::with_capacity_and_limit) to pre-allocate
/// when the maximum size is known to avoid Vec growth on the hot path.
#[derive(Debug)]
pub struct ProtoBuffer {
    buffer: Vec<u8>,
    limit: usize,
}

impl Default for ProtoBuffer {
    fn default() -> Self {
        Self {
            buffer: Vec::new(),
            limit: MAX_OTLP_SIZE_LIMIT,
        }
    }
}

impl ProtoBuffer {
    /// Construct a new buffer applying [`ConversionOptions`].
    #[must_use]
    pub fn new(opts: ConversionOptions) -> Self {
        let mut s = Self::default();
        if let Some(limit) = opts.otlp_size_limit {
            s.limit = MAX_OTLP_SIZE_LIMIT.min(limit.get());
        }
        s
    }

    /// Construct an unbounded buffer with starting allocation of `capacity` bytes.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(MAX_OTLP_SIZE_LIMIT.min(capacity)),
            limit: MAX_OTLP_SIZE_LIMIT,
        }
    }

    /// Pre-allocate `capacity` bytes and bound writes to `limit`.
    ///
    /// Used by callers that know the bound up front (e.g. self-tracing log
    /// records) to avoid both Vec growth and per-write storage-discriminator
    /// branches that would arise from a SmallVec-like inline option.
    #[must_use]
    pub fn with_capacity_and_limit(capacity: usize, limit: usize) -> Self {
        let limit = MAX_OTLP_SIZE_LIMIT.min(limit);
        let cap = MAX_OTLP_SIZE_LIMIT.min(capacity).min(limit);
        Self {
            buffer: Vec::with_capacity(cap),
            limit,
        }
    }

    /// Returns the current capacity of the underlying allocation.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.buffer.capacity()
    }

    /// Returns the current length in bytes (inherent shortcut for [`BoundedBuf::len`]).
    #[must_use]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Returns true if the buffer has no bytes (inherent shortcut for [`BoundedBuf::is_empty`]).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Reset the buffer length to zero, keeping allocated capacity.
    /// Inherent shortcut for [`BoundedBuf::clear`].
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Consume the buffer and return its bytes as a `Bytes` (zero-copy).
    #[must_use]
    pub fn into_bytes(self) -> Bytes {
        Bytes::from(self.buffer)
    }

    /// Consume the buffer and return its inner `Vec<u8>`.
    #[must_use]
    pub fn into_inner(self) -> Vec<u8> {
        self.buffer
    }

    /// Take encoded bytes, returning them as `Bytes`, plus the previous capacity
    /// so the caller can re-allocate a same-size buffer for reuse.
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

    /// Clears the buffer and bounds the allocation retained for later encodes.
    pub fn retain_capacity(&mut self, maximum: usize) {
        self.buffer.clear();
        if self.buffer.capacity() > maximum {
            self.buffer = Vec::with_capacity(maximum.min(self.limit));
        }
    }
}

impl BoundedBuf for ProtoBuffer {
    #[inline]
    fn len(&self) -> usize {
        self.buffer.len()
    }
    #[inline]
    fn limit(&self) -> usize {
        self.limit
    }
    #[inline]
    fn set_limit(&mut self, limit: usize) {
        self.limit = limit;
    }
    #[inline]
    fn try_extend_slice(&mut self, slice: &[u8]) -> EncodeResult {
        if self.buffer.len() + slice.len() <= self.limit {
            self.buffer.extend_from_slice(slice);
            Ok(())
        } else {
            Err(EncodeFailure::Dropped)
        }
    }
    #[inline]
    fn truncate(&mut self, pos: usize) {
        self.buffer.truncate(pos);
    }
    #[inline]
    fn as_slice(&self) -> &[u8] {
        &self.buffer
    }
    #[inline]
    fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.buffer
    }
    #[inline]
    fn clear(&mut self) {
        self.buffer.clear();
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

/// Stack-backed protobuf encoding buffer with fixed `N`-byte capacity.
///
/// Performs zero allocations: storage is an inline `[u8; N]` array with a
/// running cursor. Used by zero-allocation paths like
/// `raw_error!`/`StackLogRecord` where a small bounded buffer is sufficient.
pub struct StackProtoBuffer<const N: usize> {
    buf: [u8; N],
    pos: usize,
    limit: usize,
}

impl<const N: usize> StackProtoBuffer<N> {
    /// Borrow the encoded bytes (zero-copy).
    #[must_use]
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        // SAFETY: pos is always <= N and tracks initialized bytes.
        &self.buf[..self.pos]
    }

    /// Copy encoded bytes into a `Bytes` (allocates).
    #[must_use]
    pub fn to_bytes(&self) -> Bytes {
        Bytes::copy_from_slice(self.as_slice())
    }
}

impl<const N: usize> Default for StackProtoBuffer<N> {
    fn default() -> Self {
        Self {
            buf: [0u8; N],
            pos: 0,
            limit: N,
        }
    }
}

impl<const N: usize> BoundedBuf for StackProtoBuffer<N> {
    #[inline]
    fn len(&self) -> usize {
        self.pos
    }
    #[inline]
    fn limit(&self) -> usize {
        self.limit
    }
    #[inline]
    fn set_limit(&mut self, limit: usize) {
        self.limit = limit.min(N);
    }
    #[inline]
    fn try_extend_slice(&mut self, slice: &[u8]) -> EncodeResult {
        let new_pos = self.pos + slice.len();
        if new_pos <= self.limit {
            self.buf[self.pos..new_pos].copy_from_slice(slice);
            self.pos = new_pos;
            Ok(())
        } else {
            Err(EncodeFailure::Dropped)
        }
    }
    #[inline]
    fn truncate(&mut self, pos: usize) {
        if pos < self.pos {
            self.pos = pos;
        }
    }
    #[inline]
    fn as_slice(&self) -> &[u8] {
        &self.buf[..self.pos]
    }
    #[inline]
    fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.buf[..self.pos]
    }
    #[inline]
    fn clear(&mut self) {
        self.pos = 0;
    }
}

impl<const N: usize> AsRef<[u8]> for StackProtoBuffer<N> {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl<const N: usize> AsMut<[u8]> for StackProtoBuffer<N> {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.buf[..self.pos]
    }
}

/// Patch a previously written length placeholder with the actual length.
#[inline]
fn patch_len_placeholder<const N: usize, B: BoundedBuf>(
    buf: &mut B,
    len: usize,
    len_start_pos: usize,
) {
    let slice = buf.as_mut_slice();
    for i in 0..N {
        slice[len_start_pos + i] += ((len >> (i * 7)) & 0x7f) as u8;
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

    /// Position the cursor at the first sorted index whose parent-id value is `>= target`.
    ///
    /// The cursor's `sorted_indices` are ordered by the child's parent-id column (ascending,
    /// with nulls sorted last), so this seeks to the start of `target`'s equal-range.
    ///
    /// Seeking makes child attribute lookups independent of the order in which parents request
    /// their children. This matters because the root record visitation order is
    /// `(resource_id, scope_id, id)`, which is not necessarily ascending in `id` alone: e.g. after
    /// `extend attributes[...]` fills null root ids with `max + 1`, an earlier-visited scope can
    /// carry a *larger* id than a later-visited one. A purely forward cursor would then skip
    /// (permanently consume) the later, lower-id parent's child rows, silently dropping all of its
    /// attributes. Each parent id is queried at most once per batch, so re-seeking never re-yields
    /// rows for an already-emitted parent.
    ///
    /// Fast path: parents are *usually* visited in ascending id order, in which case the cursor is
    /// already at (or before) `target`'s range and [`ChildIndexIter::next`] can scan forward to it
    /// without a search. We therefore only pay for the `O(log n)` binary search when a *backward*
    /// jump is required - i.e. `target` is smaller than a parent we have already passed - which is
    /// exactly the non-monotonic case the seek exists to handle. This keeps the common,
    /// already-sorted case at the same cost as a plain forward cursor.
    fn seek_to_parent<T: ArrowPrimitiveType>(
        &mut self,
        target: T::Native,
        parent_id_col: &MaybeDictArrayAccessor<'_, PrimitiveArray<T>>,
    ) where
        T::Native: PartialOrd,
    {
        // If the most recently consumed row's parent id is `<= target`, then `target`'s rows (if
        // any) are at or after `curr_index`, so a forward scan suffices -- no search needed. Nulls
        // (value_at == None) fall through to the search, as does the very first seek of a batch.
        if self.curr_index > 0
            && let Some(prev) = parent_id_col.value_at(self.sorted_indices[self.curr_index - 1])
        {
            // `target >= prev`: forward scan suffices. Incomparable values (partial_cmp ==
            // None) fall through to the search as a safe default.
            if matches!(
                target.partial_cmp(&prev),
                Some(Ordering::Greater | Ordering::Equal)
            ) {
                return;
            }
        }

        self.curr_index =
            self.sorted_indices
                .partition_point(|&idx| match parent_id_col.value_at(idx) {
                    Some(v) => v < target,
                    // nulls sort last, so treat them as `>= target` (never part of a match range)
                    None => false,
                });
    }
}

/// Minimum batch length at which [`BatchSorter`] switches from comparison sorting to stable LSD
/// radix sort. Below this threshold, `sort_unstable_by_key` with a row-index tie-breaker has lower
/// setup overhead than counting 256-entry byte histograms.
const RADIX_SORT_THRESHOLD: usize = 64;

/// Unsigned integer key used by [`BatchSorter`]'s hybrid stable LSD radix sort.
trait RadixKey: Copy + Default + Ord {
    const BYTES: usize;
    fn bit_xor(self, other: Self) -> Self;
    fn bit_or(self, other: Self) -> Self;
    fn byte_at(self, byte_idx: usize) -> u8;
}

macro_rules! impl_radix_key {
    ($($t:ty),*) => {
        $(
            impl RadixKey for $t {
                const BYTES: usize = std::mem::size_of::<$t>();

                #[inline(always)]
                fn bit_xor(self, other: Self) -> Self {
                    self ^ other
                }

                #[inline(always)]
                fn bit_or(self, other: Self) -> Self {
                    self | other
                }

                #[inline(always)]
                fn byte_at(self, byte_idx: usize) -> u8 {
                    (self >> (byte_idx * 8)) as u8
                }
            }
        )*
    };
}

impl_radix_key!(u16, u32, u64);

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
    ) -> PDataResult<()> {
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

        cursor.reset();

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
                Self::sort_non_null_tuples(&mut self.u64_ids, cursor);
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
                Self::sort_non_null_tuples(&mut self.u32_ids, cursor);
            }
            // Single ID column -- sort directly by u16 value.
            [Some(ids), None, None] => {
                self.init_cursor_for_u16_id_column(&MaybeDictArrayAccessor::Native(ids), cursor);
            }
            // No ID columns -- visit in natural order.
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
        <T as ArrowPrimitiveType>::Native: RadixKey,
    {
        cursor.reset();

        if let MaybeDictArrayAccessor::Native(ids) = ids
            && ids.null_count() == 0
        {
            let values = ids.values();
            if values.is_sorted() {
                cursor.sorted_indices.extend(0..values.len());
                return;
            }
            if values.len() >= RADIX_SORT_THRESHOLD {
                Self::radix_sort_native_slice(values, sort_ids_tmp, cursor);
                return;
            }
        }

        let len = ids.len();
        // DictionaryArray::null_count() counts only null keys; check values().null_count() too so
        // valid keys referencing null dictionary entries also take the null path.
        let has_nulls = match ids {
            MaybeDictArrayAccessor::Native(arr) => arr.null_count() > 0,
            MaybeDictArrayAccessor::Dictionary8(d) => {
                d.null_count() > 0 || d.values().null_count() > 0
            }
            MaybeDictArrayAccessor::Dictionary16(d) => {
                d.null_count() > 0 || d.values().null_count() > 0
            }
        };

        if has_nulls && len >= RADIX_SORT_THRESHOLD {
            // Place valid row indices in `sort_ids_tmp[..valid_pos].0`, keep each row's ID value
            // at its original row slot `sort_ids_tmp[row].1`, and collect null row indices from
            // the tail of `cursor.sorted_indices` (reversed afterwards to preserve input order).
            sort_ids_tmp.clear();
            sort_ids_tmp.resize(len, (0, T::Native::default()));
            cursor.sorted_indices.resize(len, 0);

            let mut valid_pos = 0;
            let mut null_pos = len;
            for i in 0..len {
                if let Some(val) = ids.value_at(i) {
                    sort_ids_tmp[valid_pos].0 = i;
                    sort_ids_tmp[i].1 = val;
                    valid_pos += 1;
                } else {
                    null_pos -= 1;
                    cursor.sorted_indices[null_pos] = i;
                }
            }
            cursor.sorted_indices[valid_pos..len].reverse();

            Self::radix_sort_valid_prefix(sort_ids_tmp, valid_pos, true, cursor);
            return;
        }

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

        if !has_nulls {
            Self::sort_non_null_tuples(sort_ids_tmp, cursor);
        } else {
            // Small batch with nulls: sort nulls last, breaking ties by input row index. Check
            // `value_at().is_some()` so dictionary keys pointing to null values sort as nulls.
            sort_ids_tmp.sort_unstable_by(|(ia, a), (ib, b)| {
                match (ids.value_at(*ia).is_some(), ids.value_at(*ib).is_some()) {
                    (true, true) => a.cmp(b).then_with(|| ia.cmp(ib)),
                    (true, false) => Ordering::Less,
                    (false, true) => Ordering::Greater,
                    (false, false) => ia.cmp(ib),
                }
            });
            cursor
                .sorted_indices
                .extend(sort_ids_tmp.iter().map(|(i, _)| *i));
        }
    }

    fn sort_non_null_tuples<K: RadixKey>(
        tuples: &mut [(usize, K)],
        cursor: &mut SortedBatchCursor,
    ) {
        let len = tuples.len();
        if len < RADIX_SORT_THRESHOLD {
            // Small batch: include the original row index as a tie-breaker so unstable sorting
            // stays allocation-free while preserving relative row order for equal IDs.
            tuples.sort_unstable_by_key(|&(i, v)| (v, i));
            cursor.sorted_indices.extend(tuples.iter().map(|(i, _)| *i));
        } else {
            cursor.sorted_indices.resize(len, 0);
            Self::radix_sort_valid_prefix(tuples, len, false, cursor);
        }
    }

    /// Stably sorts a contiguous non-null `values` slice into `cursor.sorted_indices` using LSD
    /// radix sort over only the non-constant bytes of `K`.
    ///
    /// A single active pass reads keys directly from `values` without touching `scratch`. When
    /// multiple active passes are required, `(idx, val)` pairs are ping-ponged through
    /// `scratch[..len]` so subsequent passes read keys sequentially without indirect gathers.
    fn radix_sort_native_slice<K: RadixKey>(
        values: &[K],
        scratch: &mut Vec<(usize, K)>,
        cursor: &mut SortedBatchCursor,
    ) {
        let len = values.len();
        cursor.sorted_indices.resize(len, 0);
        if len <= 1 {
            return;
        }

        let first = values[0];
        let mut diff = K::default();
        let mut counts0 = [0usize; 256];
        for &val in values {
            diff = diff.bit_or(val.bit_xor(first));
            counts0[val.byte_at(0) as usize] += 1;
        }

        let mut active_bytes = [0usize; 8];
        let mut num_passes = 0;
        for b in 0..K::BYTES {
            if diff.byte_at(b) != 0 {
                active_bytes[num_passes] = b;
                num_passes += 1;
            }
        }

        if num_passes == 0 {
            for (dst, slot) in cursor.sorted_indices.iter_mut().enumerate() {
                *slot = dst;
            }
            return;
        }

        if num_passes >= 2 && scratch.len() < len {
            scratch.resize(len, (0, K::default()));
        }

        for (pass, &byte_idx) in active_bytes[..num_passes].iter().enumerate() {
            let dest_is_cursor = (num_passes - 1 - pass).is_multiple_of(2);

            let counts = if byte_idx == 0 {
                counts0
            } else {
                let mut c = [0usize; 256];
                for &val in values {
                    c[val.byte_at(byte_idx) as usize] += 1;
                }
                c
            };

            let mut offsets = [0usize; 256];
            let mut sum = 0;
            for (offset, &count) in offsets.iter_mut().zip(&counts) {
                *offset = sum;
                sum += count;
            }

            if pass == 0 {
                if dest_is_cursor {
                    let dst = &mut cursor.sorted_indices[..len];
                    for (idx, &val) in values.iter().enumerate() {
                        let bucket = val.byte_at(byte_idx) as usize;
                        dst[offsets[bucket]] = idx;
                        offsets[bucket] += 1;
                    }
                } else {
                    let dst = &mut scratch[..len];
                    for (idx, &val) in values.iter().enumerate() {
                        let bucket = val.byte_at(byte_idx) as usize;
                        dst[offsets[bucket]] = (idx, val);
                        offsets[bucket] += 1;
                    }
                }
            } else if dest_is_cursor {
                let dst = &mut cursor.sorted_indices[..len];
                for &(idx, val) in &scratch[..len] {
                    let bucket = val.byte_at(byte_idx) as usize;
                    dst[offsets[bucket]] = idx;
                    offsets[bucket] += 1;
                }
            } else {
                let dst = &mut scratch[..len];
                for &idx in &cursor.sorted_indices[..len] {
                    let val = values[idx];
                    let bucket = val.byte_at(byte_idx) as usize;
                    dst[offsets[bucket]] = (idx, val);
                    offsets[bucket] += 1;
                }
            }
        }
    }

    /// Stably sorts the `valid_len` non-null row indices into `cursor.sorted_indices[..valid_len]`
    /// using LSD radix sort over only the non-constant bytes of `K`.
    ///
    /// ID values remain fixed at `tuples[row].1`; passes alternate row indices between
    /// `tuples[..valid_len].0` and `cursor.sorted_indices[..valid_len]` so that the final pass
    /// always writes directly into `cursor.sorted_indices[..valid_len]`.
    fn radix_sort_valid_prefix<K: RadixKey>(
        tuples: &mut [(usize, K)],
        valid_len: usize,
        has_nulls: bool,
        cursor: &mut SortedBatchCursor,
    ) {
        if valid_len <= 1 {
            if valid_len == 1 {
                cursor.sorted_indices[0] = if has_nulls { tuples[0].0 } else { 0 };
            }
            return;
        }

        // Single scan to detect whether the valid prefix is already sorted, which bytes vary,
        // and the byte-0 histogram.
        let mut diff = K::default();
        let mut is_sorted = true;
        let mut counts0 = [0usize; 256];
        if !has_nulls {
            let first = tuples[0].1;
            let mut prev = first;
            counts0[first.byte_at(0) as usize] += 1;
            for &(_, val) in &tuples[1..valid_len] {
                diff = diff.bit_or(val.bit_xor(first));
                is_sorted &= prev <= val;
                prev = val;
                counts0[val.byte_at(0) as usize] += 1;
            }
        } else {
            let first = tuples[tuples[0].0].1;
            let mut prev = first;
            counts0[first.byte_at(0) as usize] += 1;
            for &(idx, _) in &tuples[1..valid_len] {
                let val = tuples[idx].1;
                diff = diff.bit_or(val.bit_xor(first));
                is_sorted &= prev <= val;
                prev = val;
                counts0[val.byte_at(0) as usize] += 1;
            }
        }

        if is_sorted {
            if !has_nulls {
                for (dst, slot) in cursor.sorted_indices[..valid_len].iter_mut().enumerate() {
                    *slot = dst;
                }
            } else {
                for (slot, &(idx, _)) in cursor.sorted_indices[..valid_len]
                    .iter_mut()
                    .zip(&tuples[..valid_len])
                {
                    *slot = idx;
                }
            }
            return;
        }

        let mut active_bytes = [0usize; 8];
        let mut num_passes = 0;
        for b in 0..K::BYTES {
            if diff.byte_at(b) != 0 {
                active_bytes[num_passes] = b;
                num_passes += 1;
            }
        }

        // Ensure the final pass (`pass == num_passes - 1`) always scatters into
        // `cursor.sorted_indices[..valid_len]`. When `num_passes` is even and `has_nulls` is true,
        // pass 0 scatters into `tuples[..valid_len].0`, so copy the initial valid indices into
        // `cursor.sorted_indices[..valid_len]` first.
        if has_nulls && num_passes.is_multiple_of(2) {
            for (slot, &(idx, _)) in cursor.sorted_indices[..valid_len]
                .iter_mut()
                .zip(&tuples[..valid_len])
            {
                *slot = idx;
            }
        }

        for (pass, &byte_idx) in active_bytes[..num_passes].iter().enumerate() {
            let dest_is_cursor = (num_passes - 1 - pass).is_multiple_of(2);

            let counts = if byte_idx == 0 {
                counts0
            } else {
                let mut c = [0usize; 256];
                if !has_nulls {
                    for &(_, val) in &tuples[..valid_len] {
                        c[val.byte_at(byte_idx) as usize] += 1;
                    }
                } else if dest_is_cursor {
                    for &(idx, _) in &tuples[..valid_len] {
                        c[tuples[idx].1.byte_at(byte_idx) as usize] += 1;
                    }
                } else {
                    for &idx in &cursor.sorted_indices[..valid_len] {
                        c[tuples[idx].1.byte_at(byte_idx) as usize] += 1;
                    }
                }
                c
            };

            let mut offsets = [0usize; 256];
            let mut sum = 0;
            for (offset, &count) in offsets.iter_mut().zip(&counts) {
                *offset = sum;
                sum += count;
            }

            if pass == 0 && !has_nulls {
                if dest_is_cursor {
                    let dst = &mut cursor.sorted_indices[..valid_len];
                    for (idx, &(_, val)) in tuples[..valid_len].iter().enumerate() {
                        let bucket = val.byte_at(byte_idx) as usize;
                        dst[offsets[bucket]] = idx;
                        offsets[bucket] += 1;
                    }
                } else {
                    for idx in 0..valid_len {
                        let bucket = tuples[idx].1.byte_at(byte_idx) as usize;
                        let pos = offsets[bucket];
                        offsets[bucket] += 1;
                        tuples[pos].0 = idx;
                    }
                }
            } else if dest_is_cursor {
                let dst = &mut cursor.sorted_indices[..valid_len];
                for i in 0..valid_len {
                    let idx = tuples[i].0;
                    let bucket = tuples[idx].1.byte_at(byte_idx) as usize;
                    dst[offsets[bucket]] = idx;
                    offsets[bucket] += 1;
                }
            } else {
                let src = &cursor.sorted_indices[..valid_len];
                for &idx in src {
                    let bucket = tuples[idx].1.byte_at(byte_idx) as usize;
                    let pos = offsets[bucket];
                    offsets[bucket] += 1;
                    tuples[pos].0 = idx;
                }
            }
        }
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
    T::Native: PartialOrd,
{
    pub fn new(
        parent_id: T::Native,
        parent_id_col: &'a MaybeDictArrayAccessor<'a, PrimitiveArray<T>>,
        cursor: &'a mut SortedBatchCursor,
    ) -> Self {
        // Seek to the start of this parent's child rows via binary search, so that lookups do
        // not depend on parents being visited in ascending-id order. See `seek_to_parent`.
        cursor.seek_to_parent(parent_id, parent_id_col);
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
    use super::*;
    use crate::{
        arrays::MaybeDictArrayAccessor,
        proto::OtlpProtoMessage,
        schema::consts,
        testing::{fixtures::*, round_trip::*},
    };
    use arrow::{
        array::{RecordBatch, StructArray, UInt16Array},
        datatypes::{DataType, Field, Fields, Schema, UInt16Type},
    };
    use std::sync::Arc;

    /// Scenario: child rows have shuffled parent IDs and parents are visited in ascending order.
    /// Guarantees: each parent yields only its matching row indices, and exhausted iteration ends.
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

    /// Scenario: parents are visited in non-ascending ID order against a shared child cursor.
    /// Guarantees: visiting a higher ID does not consume a lower ID's child rows.
    ///
    /// Regression test: parents may be visited in *non-ascending* id order against a shared
    /// cursor. The root visitation order is `(resource_id, scope_id, id)`, so `id` alone is not
    /// always ascending (e.g. after `extend attributes[...]` fills null root ids with `max + 1`).
    /// A purely forward cursor would consume rows for a lower id while seeking a higher one and
    /// then return nothing for the lower id, silently dropping its attributes. Each parent must
    /// find its own child rows regardless of visitation order.
    #[test]
    fn test_child_index_iter_non_monotonic_parent_order() {
        let mut cursor = SortedBatchCursor::new();
        // rows: parent_id 0 -> row0, parent_id 2 -> row1, parent_id 1 -> row2, parent_id 2 -> row3
        let tmp = UInt16Array::from_iter_values(vec![0, 2, 1, 2]);
        let parent_ids = MaybeDictArrayAccessor::Native(&tmp);
        BatchSorter::new().init_cursor_for_u16_id_column(&parent_ids, &mut cursor);

        // Visit parents in the order 0, 2, 1 (mirrors the `[0, 2, 1]` root-id sequence produced
        // by null-id fill). The middle visit (id 2) must not consume id 1's row.
        {
            let mut id_0_iter = ChildIndexIter::<UInt16Type>::new(0, &parent_ids, &mut cursor);
            assert_eq!(id_0_iter.next(), Some(0));
            assert_eq!(id_0_iter.next(), None);
        }
        {
            let mut id_2_iter = ChildIndexIter::<UInt16Type>::new(2, &parent_ids, &mut cursor);
            assert_eq!(id_2_iter.next(), Some(1));
            assert_eq!(id_2_iter.next(), Some(3));
            assert_eq!(id_2_iter.next(), None);
        }
        {
            // Before the fix this returned None because id 2's visit consumed past row2.
            let mut id_1_iter = ChildIndexIter::<UInt16Type>::new(1, &parent_ids, &mut cursor);
            assert_eq!(id_1_iter.next(), Some(2));
            assert_eq!(id_1_iter.next(), None);
        }
    }

    /// Scenario: no child rows belong to a parent ID between two populated parent IDs.
    /// Guarantees: the missing parent yields no rows without consuming the next parent's rows.
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

    /// Scenario: shuffled child rows include a null parent ID.
    /// Guarantees: null IDs are skipped and each non-null parent yields all its matching rows.
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

    /// Scenario: the same root batch is sorted twice using one batch sorter.
    /// Guarantees: the second sort reuses the existing ID vector allocation.
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

    /// Scenario: logs with populated resource and scope metadata round-trip through OTAP.
    /// Guarantees: log records and their resource and scope metadata remain semantically equivalent.
    #[test]
    fn test_logs_with_full_resource_and_scope() {
        test_logs_round_trip(logs_with_full_resource_and_scope());
    }

    /// Scenario: logs with no resource metadata round-trip through OTAP.
    /// Guarantees: log records and scope metadata survive without requiring resource metadata.
    #[test]
    fn test_logs_with_no_resource() {
        test_logs_round_trip(logs_with_no_resource());
    }

    /// Scenario: logs with no scope metadata round-trip through OTAP.
    /// Guarantees: log records and resource metadata survive without requiring scope metadata.
    #[test]
    fn test_logs_with_no_scope() {
        test_logs_round_trip(log_with_no_scope());
    }

    /// Scenario: logs without record attributes round-trip through OTAP.
    /// Guarantees: record fields remain semantically equivalent without introducing attributes.
    #[test]
    fn test_logs_with_no_attributes() {
        test_logs_round_trip(logs_with_no_attributes());
    }

    /// Scenario: logs with neither resource nor scope metadata round-trip through OTAP.
    /// Guarantees: log record contents survive without requiring either metadata envelope.
    #[test]
    fn test_logs_with_no_resource_no_scope() {
        test_logs_round_trip(logs_with_no_resource_no_scope());
    }

    /// Scenario: several logs share a scope with no resource metadata and round-trip through OTAP.
    /// Guarantees: all records retain their distinct timestamps and severities.
    #[test]
    fn test_logs_multiple_records_no_resource() {
        test_logs_round_trip(logs_multiple_records_no_resource());
    }

    /// Scenario: logs span multiple named scopes with no resource metadata.
    /// Guarantees: an OTAP round trip preserves each record's contents and scope association.
    #[test]
    fn test_logs_multiple_scopes_no_resource() {
        test_logs_round_trip(logs_multiple_scopes_no_resource());
    }

    /// Scenario: logs span multiple resource envelopes with populated and empty scope metadata.
    /// Guarantees: an OTAP round trip preserves the semantic contents of every log record.
    #[test]
    fn test_logs_multiple_resources_mixed_content() {
        test_logs_round_trip(logs_multiple_resources_mixed_content());
    }

    /// Scenario: a logs scope contains no records and round-trips through OTAP.
    /// Guarantees: dropping the empty envelope does not introduce any log records.
    #[test]
    fn test_logs_with_empty_log_records() {
        test_logs_round_trip(logs_with_empty_log_records());
    }

    /// Scenario: a log record has an explicitly empty string body and round-trips through OTAP.
    /// Guarantees: the record retains an empty string body rather than losing the body value.
    #[test]
    fn test_logs_with_body_empty_string() {
        test_logs_round_trip(logs_with_body_empty_string());
    }

    //
    // Traces Tests
    //

    /// Scenario: traces with populated resource and scope metadata round-trip through OTAP.
    /// Guarantees: spans and their resource and scope metadata remain semantically equivalent.
    #[test]
    fn test_traces_with_full_resource_and_scope() {
        test_traces_round_trip(traces_with_full_resource_and_scope());
    }

    /// Scenario: traces with no resource metadata round-trip through OTAP.
    /// Guarantees: spans and scope metadata survive without requiring resource metadata.
    #[test]
    fn test_traces_with_no_resource() {
        test_traces_round_trip(traces_with_no_resource());
    }

    /// Scenario: traces with no scope metadata round-trip through OTAP.
    /// Guarantees: spans and resource metadata survive without requiring scope metadata.
    #[test]
    fn test_traces_with_no_scope() {
        test_traces_round_trip(traces_with_no_scope());
    }

    /// Scenario: traces without span attributes round-trip through OTAP.
    /// Guarantees: span fields remain semantically equivalent without introducing attributes.
    #[test]
    fn test_traces_with_no_attributes() {
        test_traces_round_trip(traces_with_no_attributes());
    }

    /// Scenario: traces with neither resource nor scope metadata round-trip through OTAP.
    /// Guarantees: span contents survive without requiring either metadata envelope.
    #[test]
    fn test_traces_with_no_resource_no_scope() {
        test_traces_round_trip(traces_with_no_resource_no_scope());
    }

    /// Scenario: several spans share a scope with no resource metadata and round-trip through OTAP.
    /// Guarantees: all spans retain their distinct identifiers, names, kinds, and timestamps.
    #[test]
    fn test_traces_multiple_spans_no_resource() {
        test_traces_round_trip(traces_multiple_spans_no_resource());
    }

    /// Scenario: spans belong to multiple named scopes with no resource metadata.
    /// Guarantees: an OTAP round trip preserves each span's contents and scope association.
    #[test]
    fn test_traces_multiple_scopes_no_resource() {
        test_traces_round_trip(traces_multiple_scopes_no_resource());
    }

    /// Scenario: spans span multiple resource envelopes with populated and empty scope metadata.
    /// Guarantees: an OTAP round trip preserves the semantic contents of every span.
    #[test]
    fn test_traces_multiple_resources_mixed_content() {
        test_traces_round_trip(traces_multiple_resources_mixed_content());
    }

    //
    // Metrics Tests
    //

    /// Scenario: a sum metric with populated resource and scope metadata round-trips through OTAP.
    /// Guarantees: metric data and resource and scope metadata remain semantically equivalent.
    #[test]
    fn test_metrics_sum_with_full_resource_and_scope() {
        test_metrics_round_trip(metrics_sum_with_full_resource_and_scope());
    }

    /// Scenario: a sum metric with no resource metadata round-trips through OTAP.
    /// Guarantees: metric data and scope metadata survive without requiring resource metadata.
    #[test]
    fn test_metrics_sum_with_no_resource() {
        test_metrics_round_trip(metrics_sum_with_no_resource());
    }

    /// Scenario: a sum metric with no scope metadata round-trips through OTAP.
    /// Guarantees: metric data and resource metadata survive without requiring scope metadata.
    #[test]
    fn test_metrics_sum_with_no_scope() {
        test_metrics_round_trip(metrics_sum_with_no_scope());
    }

    /// Scenario: a sum metric with neither resource nor scope metadata round-trips through OTAP.
    /// Guarantees: metric data survives without requiring either metadata envelope.
    #[test]
    fn test_metrics_sum_with_no_resource_no_scope() {
        test_metrics_round_trip(metrics_sum_with_no_resource_no_scope());
    }

    /// Scenario: a named sum metric contains no data points and round-trips through OTAP.
    /// Guarantees: the metric and its sum properties survive with an empty data point list.
    #[test]
    fn test_metrics_sum_with_no_data_points() {
        test_metrics_round_trip(metrics_sum_with_no_data_points());
    }

    /// Scenario: multiple sum metrics with different aggregation properties share an empty resource.
    /// Guarantees: an OTAP round trip preserves each metric's values, temporality, and monotonicity.
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

    /// Scenario: logs data contains no resource envelopes.
    /// Guarantees: OTAP encoding produces a batch with zero items.
    #[test]
    fn test_empty_logs() {
        assert_empty_batch(empty_logs().into());
    }

    /// Scenario: a logs resource contains no scope envelopes.
    /// Guarantees: the empty resource encodes to an OTAP batch with zero items.
    #[test]
    fn test_logs_with_empty_scope_logs() {
        assert_empty_batch(logs_with_empty_scope_logs().into());
    }

    /// Scenario: traces data contains no resource envelopes.
    /// Guarantees: OTAP encoding produces a batch with zero items.
    #[test]
    fn test_empty_traces() {
        assert_empty_batch(empty_traces().into());
    }

    /// Scenario: a traces resource contains no scope envelopes.
    /// Guarantees: the empty resource encodes to an OTAP batch with zero items.
    #[test]
    fn test_traces_with_empty_scope_spans() {
        assert_empty_batch(traces_with_empty_scope_spans().into());
    }

    /// Scenario: a traces scope contains no spans.
    /// Guarantees: the empty scope encodes to an OTAP batch with zero items.
    #[test]
    fn test_traces_with_empty_spans() {
        assert_empty_batch(traces_with_empty_spans().into());
    }

    /// Scenario: metrics data contains no resource envelopes.
    /// Guarantees: OTAP encoding produces a batch with zero items.
    #[test]
    fn test_empty_metrics() {
        assert_empty_batch(empty_metrics().into());
    }

    /// Scenario: a metrics resource contains no scope envelopes.
    /// Guarantees: the empty resource encodes to an OTAP batch with zero items.
    #[test]
    fn test_metrics_with_no_scope_metrics() {
        assert_empty_batch(metrics_with_no_scope_metrics().into());
    }

    /// Scenario: a metrics scope contains no metrics.
    /// Guarantees: the empty scope encodes to an OTAP batch with zero items.
    #[test]
    fn test_metrics_with_no_metrics() {
        assert_empty_batch(metrics_with_no_metrics().into());
    }

    /// Scenario: the same nested string is encoded with 127-byte, 128-byte, and default buffers.
    /// Guarantees: length placeholders use one, two, and four bytes respectively.
    #[test]
    fn test_encode_len_delimited_auto_width() {
        // A small inline buffer (127 bytes) should use a 1-byte placeholder.
        let mut buf = StackProtoBuffer::<127>::default();
        buf.encode_len_delimited(1, |buf: &mut StackProtoBuffer<127>| -> EncodeResult {
            buf.encode_string(1, "hi")
        })
        .unwrap();
        // tag(1) + placeholder(1) + inner_tag(1) + inner_len_varint(1) + "hi"(2) = 6
        assert_eq!(buf.len(), 6);

        // A 128-byte inline buffer should use a 2-byte placeholder (128 >= 2^7).
        let mut buf = StackProtoBuffer::<128>::default();
        buf.encode_len_delimited(1, |buf: &mut StackProtoBuffer<128>| -> EncodeResult {
            buf.encode_string(1, "hi")
        })
        .unwrap();
        // tag(1) + placeholder(2) + inner_tag(1) + inner_len_varint(1) + "hi"(2) = 7
        assert_eq!(buf.len(), 7);

        // A default (256MiB limit) buffer should use a 4-byte placeholder.
        let mut buf = ProtoBuffer::default();
        buf.encode_len_delimited(1, |buf| -> EncodeResult { buf.encode_string(1, "hi") })
            .unwrap();
        // tag(1) + placeholder(4) + inner_tag(1) + inner_len_varint(1) + "hi"(2) = 9
        assert_eq!(buf.len(), 9);
    }

    /// Scenario: atomic encoding fails after writing, then a length-delimited callback returns a count.
    /// Guarantees: failure restores the original bytes; success retains bytes and the callback's count.
    #[test]
    fn try_atomic_rolls_back_buffer_on_error() {
        let mut buf = ProtoBuffer::default();
        buf.encode_string(1, "before").unwrap();
        let snapshot: Vec<u8> = buf.as_ref().to_vec();
        let snapshot_len = buf.len();

        let res: EncodeResult = buf.try_atomic(|b| {
            // Write several bytes successfully, then fail.
            b.encode_string(2, "partial-content")?;
            b.encode_varint(0xDEAD_BEEF)?;
            Err(EncodeFailure::Dropped)
        });
        assert!(matches!(res, Err(EncodeFailure::Dropped)));
        assert_eq!(buf.len(), snapshot_len, "len must be restored");
        assert_eq!(buf.as_ref(), snapshot.as_slice(), "bytes must be identical");

        // try_atomic on Ok preserves all writes.
        let res: EncodeResult<usize> = buf.try_atomic(|b| {
            b.encode_len_delimited(3, |b| {
                b.encode_string(1, "ok")?;
                Ok(b.len() - snapshot_len)
            })
        });
        assert_eq!(res.unwrap(), buf.len() - snapshot_len);
        assert!(buf.len() > snapshot_len);
    }

    /// Scenario: a length-delimited field overflows an eight-byte buffer during its inner encoding.
    /// Guarantees: failure leaves the length unpatched, while an atomic wrapper rolls back all bytes.
    #[test]
    fn encode_len_delimited_does_not_patch_on_inner_err() {
        let mut buf = StackProtoBuffer::<8>::default();
        let r: EncodeResult = buf.encode_len_delimited(1, |b| {
            // Write enough partial content to force overflow before completion.
            b.encode_string(1, "AAAAAAAAAAAAAAAAAAAA")?;
            Ok(())
        });
        assert!(matches!(r, Err(EncodeFailure::Dropped)));
        // tag(1) + placeholder(1) was written, plus partial content bytes.
        // The placeholder is unpatched (still 0x00), so a parser would read
        // the body as 0 bytes and continue at the partial content as garbage.
        assert!(buf.len() >= 2, "tag + placeholder were written");
        assert_eq!(buf.as_ref()[1], 0x00, "placeholder remained unpatched");

        // The safe pattern: wrap in try_atomic so partial bytes roll back.
        let mut buf = StackProtoBuffer::<8>::default();
        let r: EncodeResult = buf.try_atomic(|b| {
            b.encode_len_delimited(1, |b| b.encode_string(1, "AAAAAAAAAAAAAAAAAAAA"))
        });
        assert!(matches!(r, Err(EncodeFailure::Dropped)));
        assert_eq!(buf.len(), 0, "try_atomic rolled back partial bytes");
    }

    /// Scenario: a buffer temporarily restricts its remaining capacity to eight bytes without writing.
    /// Guarantees: the scoped budget applies and the original limit and remaining capacity are restored.
    #[test]
    fn with_max_remaining_restores_outer_limit() {
        let mut buf = StackProtoBuffer::<64>::default();
        let outer_limit = buf.limit();
        let outer_remaining = buf.remaining();

        let inner_seen = buf.with_max_remaining(8, |b| {
            assert!(b.remaining() <= 8, "scoped limit applied");
            b.remaining()
        });
        assert_eq!(inner_seen, 8.min(outer_remaining));

        assert_eq!(buf.limit(), outer_limit, "outer limit restored");
        assert_eq!(buf.remaining(), outer_remaining);
    }

    /// Scenario: a ten-byte remaining-capacity scope is nested inside a fifty-byte scope.
    /// Guarantees: each budget applies locally and unwinding restores each enclosing limit in turn.
    #[test]
    fn with_max_remaining_nests_correctly() {
        let mut buf = StackProtoBuffer::<128>::default();
        let outer = buf.limit();

        buf.with_max_remaining(50, |b| {
            let mid = b.limit();
            assert!(mid <= b.len() + 50);
            b.with_max_remaining(10, |b| {
                assert!(b.remaining() <= 10);
            });
            // After inner returns, mid-level limit must be restored, not outer.
            assert_eq!(b.limit(), mid, "inner restoration goes to enclosing scope");
        });
        assert_eq!(buf.limit(), outer, "fully restored after both scopes");
    }

    /// Scenario: a bounded buffer receives a scoped remaining-capacity request of usize::MAX.
    /// Guarantees: the request does not overflow or raise the limit beyond the outer buffer limit.
    #[test]
    fn with_max_remaining_saturates_and_honors_outer() {
        let mut buf = StackProtoBuffer::<32>::default();
        let outer = buf.limit();

        // usize::MAX should saturate without overflow and never exceed outer.
        buf.with_max_remaining(usize::MAX, |b| {
            assert_eq!(b.limit(), outer, "scoped limit clamped to outer");
        });
        assert_eq!(buf.limit(), outer);
    }

    /// Scenario: Construct `AnyValueArrays` from attribute `RecordBatch`es with full dictionary/native columns, minimal `type`-only columns, and missing or mistyped `type` and optional value columns.
    /// Guarantees: All 7 value columns borrow with lifetime `'a` (outliving temporary `AnyValueArrays`), dictionary accessors expose `inner()` and `values()`, omitted optional columns are `None`, and missing or mistyped columns return the corresponding `Error`.
    #[test]
    fn any_value_arrays_try_from_record_batch() {
        use crate::otlp::attributes::AttributeValueType;
        use arrow::array::{ArrayRef, BinaryArray, DictionaryArray, Int64Array, StringArray};
        use arrow::datatypes::{Schema, UInt8Type};
        use std::sync::Arc;

        let str_dict: DictionaryArray<UInt8Type> = vec![Some("hello"), None].into_iter().collect();
        let full_rb = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(consts::ATTRIBUTE_STR, str_dict.data_type().clone(), true),
                Field::new(consts::ATTRIBUTE_INT, DataType::Int64, true),
                Field::new(consts::ATTRIBUTE_DOUBLE, DataType::Float64, true),
                Field::new(consts::ATTRIBUTE_BOOL, DataType::Boolean, true),
                Field::new(consts::ATTRIBUTE_BYTES, DataType::Binary, true),
                Field::new(consts::ATTRIBUTE_SER, DataType::Binary, true),
            ])),
            vec![
                Arc::new(UInt8Array::from(vec![
                    AttributeValueType::Str as u8,
                    AttributeValueType::Int as u8,
                ])),
                Arc::new(str_dict) as ArrayRef,
                Arc::new(Int64Array::from(vec![None, Some(42)])),
                Arc::new(Float64Array::from(vec![Some(1.5), None])),
                Arc::new(BooleanArray::from(vec![Some(true), None])),
                Arc::new(BinaryArray::from_opt_vec(vec![
                    Some(b"ab".as_slice()),
                    None,
                ])),
                Arc::new(BinaryArray::from_opt_vec(vec![
                    None,
                    Some(b"cd".as_slice()),
                ])),
            ],
        )
        .unwrap();

        // Verify returned slices carry the backing RecordBatch lifetime `'a`, outliving a local
        // `AnyValueArrays` value.
        let (borrowed_str, borrowed_bytes): (Option<&str>, Option<&[u8]>) = {
            let arrays = AnyValueArrays::try_from(&full_rb).unwrap();
            assert_eq!(arrays.attr_type.value(0), AttributeValueType::Str as u8);
            assert_eq!(arrays.attr_int.as_ref().unwrap().value_at(1), Some(42));
            assert_eq!(arrays.attr_double.unwrap().value_at(0), Some(1.5));
            assert_eq!(arrays.attr_bool.unwrap().value_at(0), Some(true));
            assert_eq!(
                arrays.attr_ser.as_ref().unwrap().slice_at(1),
                Some(b"cd".as_slice())
            );
            match arrays.attr_str.as_ref().unwrap() {
                MaybeDictArrayAccessor::Dictionary8(dict) => {
                    assert_eq!(dict.inner().len(), 2);
                    assert_eq!(dict.values().value(0), "hello");
                }
                _ => panic!("expected Dictionary8 string accessor"),
            }
            (
                arrays.attr_str.as_ref().and_then(|c| c.str_at(0)),
                arrays.attr_bytes.as_ref().and_then(|c| c.slice_at(0)),
            )
        };
        assert_eq!(borrowed_str, Some("hello"));
        assert_eq!(borrowed_bytes, Some(b"ab".as_slice()));

        let minimal_rb = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new(
                consts::ATTRIBUTE_TYPE,
                DataType::UInt8,
                false,
            )])),
            vec![Arc::new(UInt8Array::from(vec![
                AttributeValueType::Empty as u8,
            ]))],
        )
        .unwrap();
        let minimal = AnyValueArrays::try_from(&minimal_rb).unwrap();
        assert!(minimal.attr_str.is_none());
        assert!(minimal.attr_int.is_none());
        assert!(minimal.attr_double.is_none());
        assert!(minimal.attr_bool.is_none());
        assert!(minimal.attr_bytes.is_none());
        assert!(minimal.attr_ser.is_none());

        let missing_type_rb = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new(
                consts::ATTRIBUTE_INT,
                DataType::Int64,
                true,
            )])),
            vec![Arc::new(Int64Array::from(vec![Some(1)]))],
        )
        .unwrap();
        assert!(matches!(
            AnyValueArrays::try_from(&missing_type_rb),
            Err(Error::ColumnNotFound { .. })
        ));

        let mistyped_type_rb = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new(
                consts::ATTRIBUTE_TYPE,
                DataType::Utf8,
                false,
            )])),
            vec![Arc::new(StringArray::from(vec!["not_u8"]))],
        )
        .unwrap();
        assert!(matches!(
            AnyValueArrays::try_from(&mistyped_type_rb),
            Err(Error::ColumnDataTypeMismatch { .. })
        ));

        let mistyped_opt_rb = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(consts::ATTRIBUTE_STR, DataType::Int64, true),
            ])),
            vec![
                Arc::new(UInt8Array::from(vec![AttributeValueType::Str as u8])),
                Arc::new(Int64Array::from(vec![Some(9)])),
            ],
        )
        .unwrap();
        assert!(matches!(
            AnyValueArrays::try_from(&mistyped_opt_rb),
            Err(Error::InvalidListArray { .. })
        ));
    }

    /// Scenario: Construct `AnyValueArrays` from a `StructArray` (such as `Logs.body`) with dictionary-encoded and native child columns, omitted optional columns, and missing or mistyped child fields.
    /// Guarantees: Struct child columns (including `UInt16` dictionary-encoded `str`) borrow accurately with lifetime `'a`, all 6 omitted optional fields are `None`, and missing or mistyped `type` / optional fields return the corresponding `Error`.
    #[test]
    fn any_value_arrays_try_from_struct_array() {
        use crate::otlp::attributes::AttributeValueType;
        use arrow::array::{ArrayRef, BinaryArray, DictionaryArray, Int64Array, StringArray};
        use arrow::datatypes::UInt16Type;
        use std::sync::Arc;

        let str_dict: DictionaryArray<UInt16Type> = vec![Some("body")].into_iter().collect();
        let full_struct = StructArray::new(
            Fields::from(vec![
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(consts::ATTRIBUTE_STR, str_dict.data_type().clone(), true),
                Field::new(consts::ATTRIBUTE_INT, DataType::Int64, true),
                Field::new(consts::ATTRIBUTE_DOUBLE, DataType::Float64, true),
                Field::new(consts::ATTRIBUTE_BOOL, DataType::Boolean, true),
                Field::new(consts::ATTRIBUTE_BYTES, DataType::Binary, true),
                Field::new(consts::ATTRIBUTE_SER, DataType::Binary, true),
            ]),
            vec![
                Arc::new(UInt8Array::from(vec![AttributeValueType::Str as u8])) as ArrayRef,
                Arc::new(str_dict) as ArrayRef,
                Arc::new(Int64Array::from(vec![Some(7)])),
                Arc::new(Float64Array::from(vec![Some(2.25)])),
                Arc::new(BooleanArray::from(vec![Some(false)])),
                Arc::new(BinaryArray::from_opt_vec(vec![Some(b"xy".as_slice())])),
                Arc::new(BinaryArray::from_opt_vec(vec![Some(b"zz".as_slice())])),
            ],
            None,
        );
        let borrowed_body: Option<&str> = {
            let arrays = AnyValueArrays::try_from(&full_struct).unwrap();
            assert_eq!(arrays.attr_type.value(0), AttributeValueType::Str as u8);
            assert_eq!(arrays.attr_int.as_ref().unwrap().value_at(0), Some(7));
            assert_eq!(arrays.attr_double.unwrap().value_at(0), Some(2.25));
            assert_eq!(arrays.attr_bool.unwrap().value_at(0), Some(false));
            assert_eq!(
                arrays.attr_bytes.as_ref().unwrap().slice_at(0),
                Some(b"xy".as_slice())
            );
            assert_eq!(
                arrays.attr_ser.as_ref().unwrap().slice_at(0),
                Some(b"zz".as_slice())
            );
            arrays.attr_str.as_ref().and_then(|c| c.str_at(0))
        };
        assert_eq!(borrowed_body, Some("body"));

        let minimal_struct = StructArray::new(
            Fields::from(vec![Field::new(
                consts::ATTRIBUTE_TYPE,
                DataType::UInt8,
                false,
            )]),
            vec![Arc::new(UInt8Array::from(vec![AttributeValueType::Str as u8])) as ArrayRef],
            None,
        );
        let minimal = AnyValueArrays::try_from(&minimal_struct).unwrap();
        assert!(minimal.attr_str.is_none());
        assert!(minimal.attr_int.is_none());
        assert!(minimal.attr_double.is_none());
        assert!(minimal.attr_bool.is_none());
        assert!(minimal.attr_bytes.is_none());
        assert!(minimal.attr_ser.is_none());

        let missing_type_struct = StructArray::new(
            Fields::from(vec![Field::new(
                consts::ATTRIBUTE_STR,
                DataType::Utf8,
                true,
            )]),
            vec![Arc::new(StringArray::from(vec![Some("orphan")])) as ArrayRef],
            None,
        );
        assert!(matches!(
            AnyValueArrays::try_from(&missing_type_struct),
            Err(Error::ColumnNotFound { .. })
        ));

        let mistyped_type_struct = StructArray::new(
            Fields::from(vec![Field::new(
                consts::ATTRIBUTE_TYPE,
                DataType::Utf8,
                false,
            )]),
            vec![Arc::new(StringArray::from(vec!["bad"])) as ArrayRef],
            None,
        );
        assert!(matches!(
            AnyValueArrays::try_from(&mistyped_type_struct),
            Err(Error::ColumnDataTypeMismatch { .. })
        ));

        let mistyped_opt_struct = StructArray::new(
            Fields::from(vec![
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(consts::ATTRIBUTE_BOOL, DataType::Int64, true),
            ]),
            vec![
                Arc::new(UInt8Array::from(vec![AttributeValueType::Bool as u8])) as ArrayRef,
                Arc::new(Int64Array::from(vec![Some(1)])) as ArrayRef,
            ],
            None,
        );
        assert!(matches!(
            AnyValueArrays::try_from(&mistyped_opt_struct),
            Err(Error::ColumnDataTypeMismatch { .. })
        ));
    }

    /// Scenario: an attribute parent_id column has initial per-parent rows followed by
    /// column-major appended rows across 64 parents (matching `upsert_attributes` output).
    /// Guarantees: `BatchSorter` preserves the relative input row order for every `parent_id`.
    #[test]
    fn batch_sorter_preserves_row_order_for_equal_ids() {
        let num_parents = 64u32;
        let mut raw_ids = Vec::with_capacity((num_parents as usize) * 16);
        // Initial two attributes per parent_id in parent-grouped order.
        for pid in 0..num_parents {
            raw_ids.push(pid);
            raw_ids.push(pid);
        }
        // Fourteen upserted attributes appended one attribute column at a time across all parents.
        for _ in 0..14 {
            for pid in 0..num_parents {
                raw_ids.push(pid);
            }
        }

        let ids_array = UInt32Array::from(raw_ids.clone());
        let accessor = MaybeDictArrayAccessor::Native(&ids_array);
        let mut sorter = BatchSorter::new();
        let mut cursor = SortedBatchCursor::new();
        sorter.init_cursor_for_u32_id_column(&accessor, &mut cursor);

        let mut expected: Vec<usize> = (0..raw_ids.len()).collect();
        expected.sort_by_key(|&i| raw_ids[i]);
        assert_eq!(cursor.sorted_indices, expected);

        for pid in 0..num_parents {
            let child_indices: Vec<usize> =
                ChildIndexIter::new(pid, &accessor, &mut cursor).collect();
            assert_eq!(child_indices.len(), 16);
            assert!(
                child_indices.is_sorted(),
                "child row indices for parent_id={pid} must preserve input order, got {child_indices:?}"
            );
        }
    }

    /// Scenario: native `u32` and `u16` ID arrays are already sorted in non-decreasing order with
    /// no nulls across empty, single-element, small, and `>= RADIX_SORT_THRESHOLD` multi-byte
    /// inputs.
    /// Guarantees: `BatchSorter` populates identity indices directly without touching scratch
    /// buffers, even when `len >= RADIX_SORT_THRESHOLD`.
    #[test]
    fn batch_sorter_native_sorted_fast_path() {
        let mut sorter = BatchSorter::new();
        let mut cursor = SortedBatchCursor::new();

        let u32_cases: Vec<Vec<u32>> = vec![
            vec![],
            vec![42],
            vec![0, 0, 1, 1, 2, 2, 5],
            (0..128u32).map(|i| i * 3).collect(),
        ];
        for values in u32_cases {
            let ids_array = UInt32Array::from(values);
            let accessor = MaybeDictArrayAccessor::Native(&ids_array);
            sorter.init_cursor_for_u32_id_column(&accessor, &mut cursor);

            assert_eq!(
                cursor.sorted_indices,
                (0..ids_array.len()).collect::<Vec<_>>()
            );
            assert!(
                sorter.u32_ids.is_empty(),
                "already-sorted native u32 ID column should bypass sort scratch buffer"
            );
        }

        let u16_cases: Vec<Vec<u16>> = vec![vec![], vec![7], (0..128u16).map(|i| i * 3).collect()];
        for values in u16_cases {
            let ids_array = UInt16Array::from(values);
            let accessor = MaybeDictArrayAccessor::Native(&ids_array);
            sorter.init_cursor_for_u16_id_column(&accessor, &mut cursor);

            assert_eq!(
                cursor.sorted_indices,
                (0..ids_array.len()).collect::<Vec<_>>()
            );
            assert!(
                sorter.u16_ids.is_empty(),
                "already-sorted native u16 ID column should bypass sort scratch buffer"
            );
        }
    }

    /// Scenario: batches at `RADIX_SORT_THRESHOLD` (64) and larger (1024) with 1, 2, 3, and 4
    /// active radix passes (including constant byte 0 and `u32::MAX` MSB values, plus alternating
    /// parent IDs `[0, 1, 0, 1, ...]`).
    /// Guarantees: hybrid stable LSD radix sort orders parent IDs ascending and preserves input
    /// row order within every parent ID.
    #[test]
    fn batch_sorter_radix_sort_preserves_order_across_byte_patterns() {
        let patterns: &[&[u32]] = &[
            // 1 active pass in byte 0 (alternating 2 parents).
            &[0, 1],
            // 1 active pass in byte 1 only (byte 0 is constant 0).
            &[512, 256, 0, 256],
            // 2 active passes in bytes 0 and 1.
            &[300, 1, 256, 0, 300, 1],
            // 2 active passes in bytes 1 and 2 only (byte 0 is constant 0).
            &[0x0002_0100, 0x0001_0200, 0, 0x0002_0100],
            // 3 active passes in bytes 0, 1, and 2.
            &[70_000, 300, 1, 0, 70_000, 300],
            // 4 active passes across all bytes including unsigned MSB and u32::MAX.
            &[u32::MAX, 0x8000_0000, 0x0102_0304, 0, u32::MAX, 0x8000_0000],
        ];

        let mut sorter = BatchSorter::new();
        let mut cursor = SortedBatchCursor::new();

        for len in [RADIX_SORT_THRESHOLD, 1024] {
            for pattern in patterns {
                let raw_ids: Vec<u32> = (0..len).map(|i| pattern[i % pattern.len()]).collect();
                let ids_array = UInt32Array::from(raw_ids.clone());
                let accessor = MaybeDictArrayAccessor::Native(&ids_array);

                sorter.init_cursor_for_u32_id_column(&accessor, &mut cursor);

                let mut expected: Vec<usize> = (0..raw_ids.len()).collect();
                expected.sort_by_key(|&i| raw_ids[i]);
                assert_eq!(cursor.sorted_indices, expected);
            }
        }
    }

    /// Scenario: large nullable ID columns (>= RADIX_SORT_THRESHOLD) with odd and even numbers of
    /// active radix passes.
    /// Guarantees: valid rows are stably sorted in ascending ID order and null rows are placed at
    /// the end in their original input order.
    #[test]
    fn batch_sorter_radix_sort_with_nulls() {
        let patterns: &[&[Option<u32>]] = &[
            // 1 active pass (odd) with nulls.
            &[Some(2), None, Some(0), Some(2), None, Some(1), Some(0)],
            // 2 active passes (even) with nulls.
            &[
                Some(300),
                None,
                Some(1),
                Some(300),
                None,
                Some(256),
                Some(1),
            ],
        ];

        let mut sorter = BatchSorter::new();
        let mut cursor = SortedBatchCursor::new();

        for pattern in patterns {
            let raw_ids: Vec<Option<u32>> = (0..256).map(|i| pattern[i % pattern.len()]).collect();
            let ids_array = UInt32Array::from(raw_ids.clone());
            let accessor = MaybeDictArrayAccessor::Native(&ids_array);

            cursor.reset();
            sorter.init_cursor_for_u32_id_column(&accessor, &mut cursor);

            let mut expected: Vec<usize> = (0..raw_ids.len()).collect();
            expected.sort_by_key(|&i| (raw_ids[i].is_none(), raw_ids[i].unwrap_or(0)));
            assert_eq!(cursor.sorted_indices, expected);
        }
    }

    /// Scenario: dictionary-encoded ID columns (Dictionary8 and Dictionary16) contain null entries
    /// in the dictionary values array, both with and without key-level nulls, below and above
    /// `RADIX_SORT_THRESHOLD`.
    /// Guarantees: `BatchSorter` never panics on logical null counts exceeding key null counts,
    /// stably sorts valid rows ascending, and places all logical nulls last in input order.
    #[test]
    fn batch_sorter_dictionary_value_nulls_sorted_last_without_panic() {
        use arrow::array::{ArrayRef, DictionaryArray};
        use arrow::datatypes::{UInt8Type, UInt16Type};

        let values = Arc::new(UInt32Array::from(vec![Some(300), None, Some(1), Some(256)]));
        let mut sorter = BatchSorter::new();
        let mut cursor = SortedBatchCursor::new();

        for len in [16usize, RADIX_SORT_THRESHOLD - 1, RADIX_SORT_THRESHOLD, 128] {
            for include_key_nulls in [false, true] {
                let raw_keys: Vec<Option<u8>> = (0..len)
                    .map(|i| match i % 5 {
                        0 if include_key_nulls => None,
                        0 => Some(0),
                        1 => Some(1), // Valid key pointing to a null dictionary value.
                        2 => Some(2),
                        3 => Some(3),
                        _ => Some(0),
                    })
                    .collect();

                let expected_vals: Vec<Option<u32>> = raw_keys
                    .iter()
                    .map(|k| k.and_then(|idx| values.value_at(idx as usize)))
                    .collect();
                let mut expected: Vec<usize> = (0..len).collect();
                expected
                    .sort_by_key(|&i| (expected_vals[i].is_none(), expected_vals[i].unwrap_or(0)));

                // Dictionary8
                let dict8 = DictionaryArray::<UInt8Type>::try_new(
                    UInt8Array::from(raw_keys.clone()),
                    values.clone(),
                )
                .unwrap();
                let dict8_ref: ArrayRef = Arc::new(dict8);
                let accessor8 = MaybeDictArrayAccessor::<UInt32Array>::try_new(&dict8_ref).unwrap();
                cursor.reset();
                sorter.init_cursor_for_u32_id_column(&accessor8, &mut cursor);
                assert_eq!(cursor.sorted_indices, expected);

                // Dictionary16
                let keys16: Vec<Option<u16>> = raw_keys.iter().map(|k| k.map(u16::from)).collect();
                let dict16 = DictionaryArray::<UInt16Type>::try_new(
                    UInt16Array::from(keys16),
                    values.clone(),
                )
                .unwrap();
                let dict16_ref: ArrayRef = Arc::new(dict16);
                let accessor16 =
                    MaybeDictArrayAccessor::<UInt32Array>::try_new(&dict16_ref).unwrap();
                cursor.reset();
                sorter.init_cursor_for_u32_id_column(&accessor16, &mut cursor);
                assert_eq!(cursor.sorted_indices, expected);
            }
        }
    }

    /// Scenario: a `SortedBatchCursor` is initialized and advanced across multiple batches of
    /// varying sizes (radix unsorted, sorted native fast path, small unsorted) without calling
    /// `cursor.reset()` between initializations.
    /// Guarantees: `BatchSorter` resets both `sorted_indices` and `curr_index` uniformly on every
    /// path.
    #[test]
    fn batch_sorter_resets_cursor_on_reinit() {
        let mut sorter = BatchSorter::new();
        let mut cursor = SortedBatchCursor::new();

        // 1. Large unsorted batch (>= RADIX_SORT_THRESHOLD), advanced past the end.
        let large_unsorted = UInt32Array::from((0..80u32).map(|i| (i % 3) ^ 1).collect::<Vec<_>>());
        sorter.init_cursor_for_u32_id_column(
            &MaybeDictArrayAccessor::Native(&large_unsorted),
            &mut cursor,
        );
        while !cursor.finished() {
            cursor.advance();
        }
        assert!(cursor.finished());

        // 2. Large sorted fast-path batch without calling cursor.reset().
        let large_sorted = UInt32Array::from((0..64u32).collect::<Vec<_>>());
        sorter.init_cursor_for_u32_id_column(
            &MaybeDictArrayAccessor::Native(&large_sorted),
            &mut cursor,
        );
        assert_eq!(cursor.curr_index(), Some(0));
        assert_eq!(cursor.sorted_indices, (0..64).collect::<Vec<_>>());
        cursor.advance();

        // 3. Small unsorted batch (< RADIX_SORT_THRESHOLD) without calling cursor.reset().
        let small = UInt32Array::from(vec![2u32, 0, 1, 0]);
        sorter.init_cursor_for_u32_id_column(&MaybeDictArrayAccessor::Native(&small), &mut cursor);
        assert_eq!(cursor.curr_index(), Some(1));
        assert_eq!(cursor.sorted_indices, vec![1, 3, 2, 0]);
    }

    /// Scenario: `init_cursor_for_root_batch` sorts 3-column (`u64`-packed), 2-column
    /// (`u32`-packed), and 1-column (`u16`) root batches both below and at/above
    /// `RADIX_SORT_THRESHOLD`, including already-sorted large batches and odd/even active byte
    /// pass counts with duplicate composite keys.
    /// Guarantees: `radix_sort_valid_prefix` (`has_nulls == false`) and small-batch comparison
    /// sorting match a stable reference sort on `(resource_id, scope_id, id)`.
    #[test]
    fn batch_sorter_root_batch_radix_and_small_sort() {
        let id_field = Fields::from(vec![Field::new(consts::ID, DataType::UInt16, false)]);
        let mut sorter = BatchSorter::new();
        let mut cursor = SortedBatchCursor::new();

        // 3-column (u64 packed): small (< 64), exact threshold (64), large unsorted (256) with
        // both even (6 active bytes) and odd (3 active bytes) pass counts, and already-sorted (128).
        let three_col_cases: Vec<(Vec<u16>, Vec<u16>, Vec<u16>)> = vec![
            // Small batch (len = 63) with duplicates.
            (
                (0..63).map(|i| (i % 3) as u16).collect(),
                (0..63).map(|i| ((i * 2) % 4) as u16).collect(),
                (0..63).map(|i| (i % 5) as u16).collect(),
            ),
            // Exact threshold (len = 64), 3 active bytes (odd pass count: bytes 0, 2, 4).
            (
                (0..64).map(|i| ((i * 5) % 7) as u16).collect(),
                (0..64).map(|i| ((i * 3) % 5) as u16).collect(),
                (0..64).map(|i| ((i * 7) % 11) as u16).collect(),
            ),
            // Large batch (len = 256), all 6 packed bytes active (even pass count: bytes 0..6).
            (
                (0..256).map(|i| ((i * 257) % 700) as u16).collect(),
                (0..256).map(|i| ((i * 521) % 900) as u16).collect(),
                (0..256).map(|i| ((i * 1031) % 1200) as u16).collect(),
            ),
            // Already-sorted large batch (len = 128) hitting `is_sorted` in `radix_sort_valid_prefix`.
            (
                (0..128).map(|i| (i / 32) as u16).collect(),
                (0..128).map(|i| ((i % 32) / 8) as u16).collect(),
                (0..128).map(|i| (i % 8) as u16).collect(),
            ),
        ];

        for (res_ids, scope_ids, row_ids) in three_col_cases {
            let rb = RecordBatch::try_new(
                Arc::new(Schema::new(vec![
                    Field::new(consts::RESOURCE, DataType::Struct(id_field.clone()), false),
                    Field::new(consts::SCOPE, DataType::Struct(id_field.clone()), false),
                    Field::new(consts::ID, DataType::UInt16, false),
                ])),
                vec![
                    Arc::new(StructArray::new(
                        id_field.clone(),
                        vec![Arc::new(UInt16Array::from(res_ids.clone()))],
                        None,
                    )),
                    Arc::new(StructArray::new(
                        id_field.clone(),
                        vec![Arc::new(UInt16Array::from(scope_ids.clone()))],
                        None,
                    )),
                    Arc::new(UInt16Array::from(row_ids.clone())),
                ],
            )
            .unwrap();

            sorter.init_cursor_for_root_batch(&rb, &mut cursor).unwrap();
            let mut expected: Vec<usize> = (0..res_ids.len()).collect();
            expected.sort_by_key(|&i| (res_ids[i], scope_ids[i], row_ids[i]));
            assert_eq!(cursor.sorted_indices, expected);
        }

        // 2-column (u32 packed): small (63), unsorted large (128, even 4 active bytes),
        // unsorted large (128, odd 3 active bytes), and already-sorted large (128).
        let two_col_cases: Vec<(Vec<u16>, Vec<u16>)> = vec![
            (
                (0..63).map(|i| ((i * 3) % 4) as u16).collect(),
                (0..63).map(|i| ((i * 5) % 6) as u16).collect(),
            ),
            (
                (0..128).map(|i| ((i * 313) % 800) as u16).collect(),
                (0..128).map(|i| ((i * 509) % 900) as u16).collect(),
            ),
            (
                (0..128).map(|i| ((i * 7) % 5) as u16).collect(),
                (0..128).map(|i| ((i * 313) % 800) as u16).collect(),
            ),
            (
                (0..128).map(|i| (i / 16) as u16).collect(),
                (0..128).map(|i| (i % 16) as u16).collect(),
            ),
        ];

        for (res_ids, scope_ids) in two_col_cases {
            let rb = RecordBatch::try_new(
                Arc::new(Schema::new(vec![
                    Field::new(consts::RESOURCE, DataType::Struct(id_field.clone()), false),
                    Field::new(consts::SCOPE, DataType::Struct(id_field.clone()), false),
                ])),
                vec![
                    Arc::new(StructArray::new(
                        id_field.clone(),
                        vec![Arc::new(UInt16Array::from(res_ids.clone()))],
                        None,
                    )),
                    Arc::new(StructArray::new(
                        id_field.clone(),
                        vec![Arc::new(UInt16Array::from(scope_ids.clone()))],
                        None,
                    )),
                ],
            )
            .unwrap();

            sorter.init_cursor_for_root_batch(&rb, &mut cursor).unwrap();
            let mut expected: Vec<usize> = (0..res_ids.len()).collect();
            expected.sort_by_key(|&i| (res_ids[i], scope_ids[i]));
            assert_eq!(cursor.sorted_indices, expected);
        }

        // 1-column (u16) root batch at < 64 and >= 64.
        for len in [63usize, 128] {
            let row_ids: Vec<u16> = (0..len).map(|i| ((i * 263) % 700) as u16).collect();
            let rb = RecordBatch::try_new(
                Arc::new(Schema::new(vec![Field::new(
                    consts::ID,
                    DataType::UInt16,
                    false,
                )])),
                vec![Arc::new(UInt16Array::from(row_ids.clone()))],
            )
            .unwrap();

            sorter.init_cursor_for_root_batch(&rb, &mut cursor).unwrap();
            let mut expected: Vec<usize> = (0..len).collect();
            expected.sort_by_key(|&i| row_ids[i]);
            assert_eq!(cursor.sorted_indices, expected);
        }
    }

    /// Scenario: `Dictionary8` and `Dictionary16` ID columns (`u16` and `u32`) are sorted at
    /// `< 64` (63), exact threshold (64), and large (256) lengths for already-sorted non-null,
    /// unsorted non-null (odd and even active passes), and key-null inputs.
    /// Guarantees: dictionary columns produce exact stable sorted indices across all paths.
    #[test]
    fn batch_sorter_dictionary_columns_sorted_and_unsorted() {
        use arrow::array::{ArrayRef, DictionaryArray};
        use arrow::datatypes::{UInt8Type, UInt16Type};

        let mut sorter = BatchSorter::new();
        let mut cursor = SortedBatchCursor::new();

        // u32 dictionaries: 1-pass values, 2-pass values, and already-sorted key patterns.
        let u32_value_sets: &[&[u32]] = &[&[3, 1, 2, 0], &[70_000, 300, 1, 0]];
        for &vals in u32_value_sets {
            let values = Arc::new(UInt32Array::from(vals.to_vec()));
            for len in [RADIX_SORT_THRESHOLD - 1, RADIX_SORT_THRESHOLD, 256] {
                for mode in 0..3 {
                    let raw_keys: Vec<Option<u8>> = match mode {
                        // Already sorted in non-decreasing value order (value index 3 is min=0).
                        0 => (0..len)
                            .map(|i| {
                                let ordered = [3u8, 2, 1, 0];
                                Some(ordered[(i * 4 / len).min(3)])
                            })
                            .collect(),
                        // Unsorted non-null.
                        1 => (0..len).map(|i| Some((i % vals.len()) as u8)).collect(),
                        // Unsorted with key-level nulls.
                        _ => (0..len)
                            .map(|i| {
                                if i % 7 == 0 {
                                    None
                                } else {
                                    Some((i % vals.len()) as u8)
                                }
                            })
                            .collect(),
                    };

                    let expected_vals: Vec<Option<u32>> = raw_keys
                        .iter()
                        .map(|k| k.map(|idx| vals[idx as usize]))
                        .collect();
                    let mut expected: Vec<usize> = (0..len).collect();
                    expected.sort_by_key(|&i| {
                        (expected_vals[i].is_none(), expected_vals[i].unwrap_or(0))
                    });

                    let dict8: ArrayRef = Arc::new(
                        DictionaryArray::<UInt8Type>::try_new(
                            UInt8Array::from(raw_keys.clone()),
                            values.clone(),
                        )
                        .unwrap(),
                    );
                    let acc8 = MaybeDictArrayAccessor::<UInt32Array>::try_new(&dict8).unwrap();
                    sorter.init_cursor_for_u32_id_column(&acc8, &mut cursor);
                    assert_eq!(cursor.sorted_indices, expected);

                    let keys16: Vec<Option<u16>> =
                        raw_keys.iter().map(|k| k.map(u16::from)).collect();
                    let dict16: ArrayRef = Arc::new(
                        DictionaryArray::<UInt16Type>::try_new(
                            UInt16Array::from(keys16),
                            values.clone(),
                        )
                        .unwrap(),
                    );
                    let acc16 = MaybeDictArrayAccessor::<UInt32Array>::try_new(&dict16).unwrap();
                    sorter.init_cursor_for_u32_id_column(&acc16, &mut cursor);
                    assert_eq!(cursor.sorted_indices, expected);
                }
            }
        }

        // u16 Dictionary8 and Dictionary16 at len = 64 (2 active bytes with u16::MAX).
        let u16_vals = [u16::MAX, 300, 1, 0];
        let u16_values = Arc::new(UInt16Array::from(u16_vals.to_vec()));
        let raw_keys: Vec<Option<u8>> = (0..64)
            .map(|i| {
                if i % 9 == 0 {
                    None
                } else {
                    Some((i % 4) as u8)
                }
            })
            .collect();
        let expected_vals: Vec<Option<u16>> = raw_keys
            .iter()
            .map(|k| k.map(|idx| u16_vals[idx as usize]))
            .collect();
        let mut expected: Vec<usize> = (0..64).collect();
        expected.sort_by_key(|&i| (expected_vals[i].is_none(), expected_vals[i].unwrap_or(0)));

        let dict8: ArrayRef = Arc::new(
            DictionaryArray::<UInt8Type>::try_new(
                UInt8Array::from(raw_keys.clone()),
                u16_values.clone(),
            )
            .unwrap(),
        );
        let acc8 = MaybeDictArrayAccessor::<UInt16Array>::try_new(&dict8).unwrap();
        sorter.init_cursor_for_u16_id_column(&acc8, &mut cursor);
        assert_eq!(cursor.sorted_indices, expected);

        let dict16: ArrayRef = Arc::new(
            DictionaryArray::<UInt16Type>::try_new(
                UInt16Array::from(
                    raw_keys
                        .iter()
                        .map(|k| k.map(u16::from))
                        .collect::<Vec<_>>(),
                ),
                u16_values,
            )
            .unwrap(),
        );
        let acc16 = MaybeDictArrayAccessor::<UInt16Array>::try_new(&dict16).unwrap();
        sorter.init_cursor_for_u16_id_column(&acc16, &mut cursor);
        assert_eq!(cursor.sorted_indices, expected);
    }

    /// Scenario: large nullable ID columns (`>= RADIX_SORT_THRESHOLD`) cover all-null (`valid_len
    /// == 0`), single-valid at a non-zero row (`valid_len == 1`), already-sorted and all-equal
    /// valid prefixes with interspersed nulls, plus 3-pass, 4-pass, and constant-byte-0 2-pass
    /// patterns.
    /// Guarantees: every branch of `radix_sort_valid_prefix` (`has_nulls == true`) preserves valid
    /// ascending order and keeps null rows at the tail in input order.
    #[test]
    fn batch_sorter_radix_sort_null_edge_cases() {
        let mut sorter = BatchSorter::new();
        let mut cursor = SortedBatchCursor::new();

        let mut cases: Vec<Vec<Option<u32>>> = Vec::new();
        // 1. All-null (valid_len == 0).
        cases.push(vec![None; RADIX_SORT_THRESHOLD]);
        // 2. Single valid row at non-zero index 37 (valid_len == 1).
        let mut single_valid = vec![None; RADIX_SORT_THRESHOLD];
        single_valid[37] = Some(42);
        cases.push(single_valid);
        // 3. Already-sorted valid prefix with interspersed nulls.
        cases.push(
            (0..RADIX_SORT_THRESHOLD)
                .map(|i| {
                    if i % 4 == 1 {
                        None
                    } else {
                        Some((i / 4) as u32)
                    }
                })
                .collect(),
        );
        // 4. All-equal valid values with interspersed nulls.
        cases.push(
            (0..RADIX_SORT_THRESHOLD)
                .map(|i| if i % 3 == 0 { None } else { Some(7) })
                .collect(),
        );
        // 5. 3 active passes (bytes 0, 1, 2) with nulls.
        let p3 = [Some(70_000), None, Some(300), Some(1), Some(0), None];
        cases.push((0..128).map(|i| p3[i % p3.len()]).collect());
        // 6. 4 active passes (bytes 0, 1, 2, 3 including MSB / u32::MAX) with nulls.
        let p4 = [
            Some(u32::MAX),
            None,
            Some(0x8000_0000),
            Some(0x0102_0304),
            Some(0),
            None,
        ];
        cases.push((0..128).map(|i| p4[i % p4.len()]).collect());
        // 7. 2 active passes skipping byte 0 (bytes 1 and 2 only, hitting
        // `byte_idx != 0 && has_nulls && !dest_is_cursor`).
        let p_hi = [
            Some(0x0002_0100),
            None,
            Some(0x0001_0200),
            Some(0x0002_0100),
            Some(0),
            None,
        ];
        cases.push((0..128).map(|i| p_hi[i % p_hi.len()]).collect());

        for raw_ids in cases {
            let ids_array = UInt32Array::from(raw_ids.clone());
            let accessor = MaybeDictArrayAccessor::Native(&ids_array);
            sorter.init_cursor_for_u32_id_column(&accessor, &mut cursor);

            let mut expected: Vec<usize> = (0..raw_ids.len()).collect();
            expected.sort_by_key(|&i| (raw_ids[i].is_none(), raw_ids[i].unwrap_or(0)));
            assert_eq!(cursor.sorted_indices, expected);
        }
    }

    /// Scenario: small batches just below `RADIX_SORT_THRESHOLD` (`len = 63`, beyond insertion
    /// sort cutoffs) with duplicate IDs and multiple nulls, plus `u16` radix batches at `len = 64`
    /// and `256` (1-pass and 2-pass including `u16::MAX`, non-null and nullable).
    /// Guarantees: small-batch tie-breaking preserves relative order for equal valid IDs and
    /// multiple nulls, and `RadixKey for u16` sorts large `u16` columns stably.
    #[test]
    fn batch_sorter_small_batch_stability_and_u16_radix() {
        let mut sorter = BatchSorter::new();
        let mut cursor = SortedBatchCursor::new();
        let small_len = RADIX_SORT_THRESHOLD - 1;

        // Small non-null u32 and u16 batches (len = 63) with heavy duplicate keys.
        let small_u32: Vec<u32> = (0..small_len).map(|i| ((i * 3) % 5) as u32).collect();
        let arr_u32 = UInt32Array::from(small_u32.clone());
        sorter
            .init_cursor_for_u32_id_column(&MaybeDictArrayAccessor::Native(&arr_u32), &mut cursor);
        let mut expected_u32: Vec<usize> = (0..small_len).collect();
        expected_u32.sort_by_key(|&i| small_u32[i]);
        assert_eq!(cursor.sorted_indices, expected_u32);

        let small_u16: Vec<u16> = (0..small_len).map(|i| ((i * 5) % 4) as u16).collect();
        let arr_u16 = UInt16Array::from(small_u16.clone());
        sorter
            .init_cursor_for_u16_id_column(&MaybeDictArrayAccessor::Native(&arr_u16), &mut cursor);
        let mut expected_u16: Vec<usize> = (0..small_len).collect();
        expected_u16.sort_by_key(|&i| small_u16[i]);
        assert_eq!(cursor.sorted_indices, expected_u16);

        // Small nullable u32 and u16 batches (len = 63) with many None rows to exercise
        // `(false, false) => ia.cmp(ib)`.
        let small_null_u32: Vec<Option<u32>> = (0..small_len)
            .map(|i| {
                if i % 3 == 0 {
                    None
                } else {
                    Some(((i * 7) % 4) as u32)
                }
            })
            .collect();
        let arr_null_u32 = UInt32Array::from(small_null_u32.clone());
        sorter.init_cursor_for_u32_id_column(
            &MaybeDictArrayAccessor::Native(&arr_null_u32),
            &mut cursor,
        );
        let mut expected_null_u32: Vec<usize> = (0..small_len).collect();
        expected_null_u32
            .sort_by_key(|&i| (small_null_u32[i].is_none(), small_null_u32[i].unwrap_or(0)));
        assert_eq!(cursor.sorted_indices, expected_null_u32);

        let small_null_u16: Vec<Option<u16>> = (0..small_len)
            .map(|i| {
                if i % 4 == 1 {
                    None
                } else {
                    Some(((i * 3) % 4) as u16)
                }
            })
            .collect();
        let arr_null_u16 = UInt16Array::from(small_null_u16.clone());
        sorter.init_cursor_for_u16_id_column(
            &MaybeDictArrayAccessor::Native(&arr_null_u16),
            &mut cursor,
        );
        let mut expected_null_u16: Vec<usize> = (0..small_len).collect();
        expected_null_u16
            .sort_by_key(|&i| (small_null_u16[i].is_none(), small_null_u16[i].unwrap_or(0)));
        assert_eq!(cursor.sorted_indices, expected_null_u16);

        // Large u16 radix sort (len = 64 and 256): 1-pass and 2-pass with u16::MAX, non-null and
        // nullable.
        let u16_patterns: &[&[u16]] = &[&[3, 1, 0, 2, 1], &[u16::MAX, 300, 0x8000, 1, 0, u16::MAX]];
        for len in [RADIX_SORT_THRESHOLD, 256] {
            for pattern in u16_patterns {
                let raw: Vec<u16> = (0..len).map(|i| pattern[i % pattern.len()]).collect();
                let arr = UInt16Array::from(raw.clone());
                sorter.init_cursor_for_u16_id_column(
                    &MaybeDictArrayAccessor::Native(&arr),
                    &mut cursor,
                );
                let mut expected: Vec<usize> = (0..len).collect();
                expected.sort_by_key(|&i| raw[i]);
                assert_eq!(cursor.sorted_indices, expected);

                let raw_opt: Vec<Option<u16>> = (0..len)
                    .map(|i| {
                        if i % 5 == 0 {
                            None
                        } else {
                            Some(pattern[i % pattern.len()])
                        }
                    })
                    .collect();
                let arr_opt = UInt16Array::from(raw_opt.clone());
                sorter.init_cursor_for_u16_id_column(
                    &MaybeDictArrayAccessor::Native(&arr_opt),
                    &mut cursor,
                );
                let mut expected_opt: Vec<usize> = (0..len).collect();
                expected_opt.sort_by_key(|&i| (raw_opt[i].is_none(), raw_opt[i].unwrap_or(0)));
                assert_eq!(cursor.sorted_indices, expected_opt);
            }
        }
    }

    /// Scenario: a single `BatchSorter` sorts a 1-pass native batch (128 rows), then 2-pass native
    /// batches that grow (64 -> 256 rows) and shrink (256 -> 80 rows).
    /// Guarantees: 1-pass native radix sorting leaves `u32_ids` untouched, multi-pass growth
    /// succeeds, and shrinking reuses the existing `u32_ids` heap buffer without reading stale tail
    /// entries.
    #[test]
    fn batch_sorter_native_radix_scratch_reuse() {
        let mut sorter = BatchSorter::new();
        let mut cursor = SortedBatchCursor::new();

        // 1-pass native radix sort should not touch scratch buffer at all.
        let one_pass: Vec<u32> = (0..128).map(|i| (i % 4) as u32 ^ 2).collect();
        let arr_1p = UInt32Array::from(one_pass.clone());
        sorter.init_cursor_for_u32_id_column(&MaybeDictArrayAccessor::Native(&arr_1p), &mut cursor);
        let mut expected_1p: Vec<usize> = (0..one_pass.len()).collect();
        expected_1p.sort_by_key(|&i| one_pass[i]);
        assert_eq!(cursor.sorted_indices, expected_1p);
        assert!(
            sorter.u32_ids.is_empty(),
            "single-pass native radix sort should not allocate scratch buffer"
        );

        // 2-pass at len = 64, then grow to len = 256, then shrink to len = 80 without reallocation.
        let pattern = [300u32, 1, 256, 0, 300, 1];
        for len in [RADIX_SORT_THRESHOLD, 256] {
            let raw: Vec<u32> = (0..len).map(|i| pattern[i % pattern.len()]).collect();
            let arr = UInt32Array::from(raw.clone());
            sorter
                .init_cursor_for_u32_id_column(&MaybeDictArrayAccessor::Native(&arr), &mut cursor);
            let mut expected: Vec<usize> = (0..len).collect();
            expected.sort_by_key(|&i| raw[i]);
            assert_eq!(cursor.sorted_indices, expected);
        }

        let ptr_after_256 = sorter.u32_ids.as_ptr();
        let raw_80: Vec<u32> = (0..80).map(|i| pattern[i % pattern.len()]).collect();
        let arr_80 = UInt32Array::from(raw_80.clone());
        sorter.init_cursor_for_u32_id_column(&MaybeDictArrayAccessor::Native(&arr_80), &mut cursor);
        let mut expected_80: Vec<usize> = (0..80).collect();
        expected_80.sort_by_key(|&i| raw_80[i]);
        assert_eq!(cursor.sorted_indices, expected_80);
        assert_eq!(
            sorter.u32_ids.as_ptr(),
            ptr_after_256,
            "shrinking batch size should reuse existing scratch allocation"
        );
    }
}
