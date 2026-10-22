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
) -> EncodeResult {
    // add attributes
    if let Some(attrs_arrays) = resource_attrs_arrays
        && let Some(res_id) = resource_arrays.id.value_at(index)
    {
        for attr_index in
            ChildIndexIter::new(res_id, &attrs_arrays.parent_id, resource_attrs_cursor)
        {
            result_buf.encode_len_delimited(RESOURCE_ATTRIBUTES, |result_buf| {
                encode_key_value(attrs_arrays, attr_index, result_buf)
            })?;
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
) -> EncodeResult {
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
            result_buf.encode_len_delimited(INSTRUMENTATION_SCOPE_ATTRIBUTES, |result_buf| {
                encode_key_value(attr_arrays, attr_index, result_buf)
            })?;
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

/// Suffix appended to truncated strings.
pub const TRUNCATION_SUFFIX: &[u8] = "[...]".as_bytes();

/// Indicating success. This is an alias so that Result<()> can
/// map into EncodeSuccess; it enables Iterator.try_for_each
/// expressions to be used inside encode functions.
pub type EncodeSuccess = ();

/// Returned when a encode operation fails. Users explicitly choose
/// EncodeResult or TruncateResult to indicate whether they handle
/// truncation.
#[derive(Debug, Clone)]
pub enum EncodeFailure {
    /// Dropped due to space limit. If a TruncateResult, it means no
    /// valid data could be written in the available space.
    Dropped,
    /// Invalid data (e.g., CBOR decode failure) in OTAP/OTLP encoding.
    /// We do not save this error, on purpose, since there are very few
    /// cases where it can arise and the cost is not negligible.
    Invalid,
}

impl std::error::Error for EncodeFailure {}

impl fmt::Display for EncodeFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dropped => write!(f, "dropped"),
            Self::Invalid => write!(f, "invalid"),
        }
    }
}

impl From<Error> for EncodeFailure {
    fn from(err: Error) -> Self {
        match err {
            Error::InternalBufferFull => Self::Dropped,
            _ => Self::Invalid,
        }
    }
}

/// Truncation status.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Truncation {
    /// No truncation.
    None,
    /// Some truncation
    Some,
}

// EncodeSuccess equals Truncation::None.
impl From<EncodeSuccess> for Truncation {
    fn from(_: EncodeSuccess) -> Self {
        Truncation::None
    }
}

/// Result type for encoding methods without truncation.
pub type EncodeResult = Result<EncodeSuccess, EncodeFailure>;

/// Result type for encoding methods with truncation.
pub type TruncateResult = Result<Truncation, EncodeFailure>;

const fn tag_width_limit(tagw: usize) -> usize {
    1 << (7 * tagw)
}

const MAX_TAG_WIDTH: usize = 4;

/// The maximum supported protocol buffer size limit (1 << 28) = 256MiB.
pub const MAX_OTLP_SIZE_LIMIT: usize = tag_width_limit(MAX_TAG_WIDTH);

/// Compute the number of bytes needed to encode a varint value.
#[inline]
const fn varint_len(value: u64) -> usize {
    // Each byte encodes 7 bits; we need ceil(bits_needed / 7), minimum 1.
    if value == 0 {
        return 1;
    }
    let bits = 64 - value.leading_zeros() as usize;
    bits.div_ceil(7)
}

/// Truncate a string to at most `max_bytes` bytes at a UTF-8 character boundary.
fn truncate_utf8(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

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

    /// Encode a string field, truncating with a `[...]` suffix if it won't fit.
    fn encode_string_truncating(&mut self, tag: u64, val: &str) -> TruncateResult {
        let full_len = val.len();
        let tag_bytes = varint_len((tag << 3) | wire_types::LEN);

        // Check if the full string fits.
        let full_field_size = tag_bytes + varint_len(full_len as u64) + full_len;
        if self.len() + full_field_size <= self.limit() {
            return Ok(self.encode_string(tag, val)?.into());
        }

        let suffix_len = TRUNCATION_SUFFIX.len();
        let avail = self.remaining();

        // Minimum encoding is: tag + 1-byte length varint + suffix bytes.
        let min_overhead = tag_bytes + 1 + suffix_len;
        if avail < min_overhead {
            return Err(EncodeFailure::Dropped);
        }

        // After the tag, we have `len_and_payload` bytes for the
        // length varint plus the payload (content + suffix). The
        // `min_overhead` check above guarantees there is room for the
        // varint and the suffix, so `content_len` is non-negative.
        let len_and_payload = avail - tag_bytes;
        let len_varint_size = varint_len(len_and_payload as u64);
        debug_assert!(
            len_and_payload >= len_varint_size + suffix_len,
            "min_overhead check guarantees varint + suffix fit",
        );
        let content_len = len_and_payload - len_varint_size - suffix_len;

        // Truncate at a UTF-8 character boundary.
        let truncated_str = truncate_utf8(val, content_len);
        let payload_len = truncated_str.len() + suffix_len;

        // Pre-validated: each write fits within the limit.
        self.encode_field_tag(tag, wire_types::LEN)
            .expect("pre-validated fit");
        self.encode_varint(payload_len as u64)
            .expect("pre-validated fit");
        self.try_extend_slice(truncated_str.as_bytes())
            .expect("pre-validated fit");
        self.try_extend_slice(TRUNCATION_SUFFIX)
            .expect("pre-validated fit");
        Ok(Truncation::Some)
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
    use crate::proto::opentelemetry::common::v1::AnyValue;
    use crate::proto::opentelemetry::common::v1::any_value::Value::StringValue;
    use crate::{
        arrays::MaybeDictArrayAccessor,
        proto::OtlpProtoMessage,
        proto::consts::wire_types,
        schema::consts,
        testing::{fixtures::*, round_trip::*},
    };
    use arrow::{
        array::{RecordBatch, StructArray, UInt16Array},
        datatypes::{DataType, Field, Fields, Schema, UInt16Type},
    };
    use prost::Message;
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

    /// Scenario: atomic encoding writes partial content before failing, then another call succeeds.
    /// Guarantees: failure restores the original bytes and length while success retains new bytes.
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
        let res: EncodeResult = buf.try_atomic(|b| b.encode_string(3, "ok"));
        assert!(res.is_ok());
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

    /// Scenario: a length-delimited callback writes five bytes and reports successful truncation.
    /// Guarantees: the length is patched to five and a subsequent field can still be appended.
    #[test]
    fn encode_len_delimited_partial_patches_length_on_err() {
        let mut buf = StackProtoBuffer::<128>::default();
        // Pre-write a marker field so we can detect mis-parsing of trailing data.
        buf.encode_string(7, "marker").unwrap();
        let len_before = buf.len();

        let r: Truncation = buf
            .encode_len_delimited(1, |b| {
                b.extend_from_slice(b"abcde")?;
                Ok::<_, EncodeFailure>(Truncation::Some)
            })
            .unwrap();
        assert_eq!(r, Truncation::Some);

        // Read back: tag for field 1 (LEN), then patched length, then 5 bytes.
        let bytes = &buf.as_ref()[len_before..];
        assert_eq!(bytes[0], (1 << 3) | wire_types::LEN as u8);
        // Placeholder width 1 expected (small remaining after marker).
        let payload_len = bytes[1] as usize;
        assert_eq!(payload_len, 5, "length patched to written-bytes count");
        assert_eq!(&bytes[2..2 + payload_len], b"abcde");

        // Append another field after the partial one and confirm it parses.
        buf.encode_string(8, "after").unwrap();
        // Find "after" by scanning -- it must be intact at the end.
        assert!(
            buf.as_ref().windows(5).any(|w| w == b"after"),
            "subsequent field is intact when partial used"
        );
    }

    /// Scenario: strings fully fit, require truncation, or cannot fit in a bounded buffer.
    /// Guarantees: full writes have no suffix, truncation adds `[...]`, and hard failure changes no bytes.
    #[test]
    fn encode_string_truncating_three_outcomes() {
        // (a) Full fit -> Ok(false), no suffix.
        let mut buf = StackProtoBuffer::<64>::default();
        let r = buf.encode_string_truncating(1, "hi");
        assert!(matches!(r, Ok(Truncation::None)));
        assert!(!buf.as_ref().windows(5).any(|w| w == TRUNCATION_SUFFIX));

        // (b) Truncated -> Ok(true), output ends with suffix.
        let mut buf = StackProtoBuffer::<32>::default();
        let long = "X".repeat(1000);
        let r = buf.encode_string_truncating(1, &long);
        assert!(matches!(r, Ok(Truncation::Some)));
        assert!(buf.as_ref().ends_with(TRUNCATION_SUFFIX));

        // (c) Hard fail -> Dropped, buffer unchanged.
        let mut buf = StackProtoBuffer::<4>::default();
        // Pre-fill so remaining < min_overhead.
        buf.extend_from_slice(b"abcd").unwrap();
        let snapshot: Vec<u8> = buf.as_ref().to_vec();
        let r = buf.encode_string_truncating(1, &long);
        assert!(matches!(r, Err(EncodeFailure::Dropped)));
        assert_eq!(
            buf.as_ref(),
            snapshot.as_slice(),
            "no partial bytes on hard fail"
        );
    }

    /// Scenario: a long string is truncated while encoding an AnyValue into a bounded buffer.
    /// Guarantees: prost decodes a string containing the original prefix and the `[...]` suffix.
    #[test]
    fn encode_string_truncating_round_trips_via_prost() {
        let mut buf = StackProtoBuffer::<48>::default();
        let long = "Y".repeat(500);
        let r = buf.encode_string_truncating(1, &long);
        assert!(matches!(r, Ok(Truncation::Some)));

        // Decode as AnyValue (field 1 = string_value).
        let av = AnyValue::decode(buf.to_bytes()).expect("valid wire bytes");
        let s = match av.value {
            Some(StringValue(s)) => s,
            other => panic!("unexpected AnyValue variant: {:?}", other),
        };
        assert!(
            s.ends_with("[...]"),
            "decoded value should end with truncation suffix"
        );
        assert!(s.starts_with("YY"));
    }

    /// Scenario: a string of four-byte UTF-8 characters is truncated at every possible byte budget.
    /// Guarantees: each result is a character-aligned prefix within budget, or the full input if it fits.
    #[test]
    fn truncate_utf8_returns_char_boundary_prefix() {
        // 4-byte UTF-8 characters (U+1F600, 4 bytes each).
        let s: String = std::iter::repeat_n('\u{1F600}', 5).collect();
        assert_eq!(s.len(), 20);

        // For every possible max_bytes, the result must be:
        //   - a valid &str prefix of `s` (implicit: no panic on slicing)
        //   - of length <= max_bytes
        //   - of length >= max_bytes - 3 (since chars are at most 4 bytes)
        for max_bytes in 0..=s.len() {
            let out = truncate_utf8(&s, max_bytes);
            assert!(out.len() <= max_bytes, "max_bytes={max_bytes}");
            assert!(
                max_bytes < 4 || out.len() >= max_bytes - 3,
                "max_bytes={max_bytes}, out.len()={}",
                out.len()
            );
            assert!(s.starts_with(out));
            // out.len() must be a multiple of 4 (each char is 4 bytes).
            assert_eq!(out.len() % 4, 0, "max_bytes={max_bytes}");
        }

        // Above the input length, returns the whole string.
        assert_eq!(truncate_utf8(&s, 1000), s.as_str());
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
}
