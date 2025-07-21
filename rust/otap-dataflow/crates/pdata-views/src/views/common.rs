// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains backend-agnostic, zero-copy view traits for common OTLP message types
//! such as `AnyValue`, `KeyValue`, and `InstrumentationScope`.
//!
//! It also contains common helper types and structs such as `ValSlice` and `Str` which do not have
//! an analogous proto message, but are available as common return types for other View trait
//! implementations

/// All current implementations only use borrowed strings from the underlying data.
/// If lossy UTF-8 support is needed in the future, this can be reverted to `Cow<'src, str>`.
pub type Str<'src> = &'src str;

/// Trace IDs are 16 binary bytes.
pub type TraceId = [u8; 16];

/// Span IDs are 8 binary bytes.
pub type SpanId = [u8; 8];

/// View for AnyValue
pub trait AnyValueView<'val> {
    /// The `AttributeView` type associated with this impl of the `AnyValueView` trait.
    /// This type will be used to access the value if the value type is kvlist
    type KeyValue: AttributeView;

    /// The associated value iterator type for this impl of the trait. The iterator will yield
    /// borrowed references that must live as long as the lifetime 'att. This will be used if the
    /// value_type is array
    type ArrayIter<'arr>: Iterator<Item = Self>
    where
        Self: 'arr;

    /// The associated attribute iterator type for this trait. The iterator will yield
    /// borrowed references that must live as long as the lifetime 'kv. This will be used if the
    /// value type is kvlist.
    type KeyValueIter<'kv>: Iterator<Item = Self::KeyValue>
    where
        Self: 'kv;

    /// The value type of this value. Care should be taken to implement this method in a way such
    /// that it accurately identifies the underlying type such that callers can expect to be able
    /// to unwrap the various as_<type> methods. For example, if this returns `ValueType::String`
    /// then it should be safe for a caller to also call as_string().expect("...")
    fn value_type(&self) -> ValueType;

    /* ---------- scalar ---------- */
    /// Get the value as an &str. returns None if the value_type is not `String`
    fn as_string(&self) -> Option<Str<'_>>;
    /// Get the value as a boolean. returns `None` if value_type is not `Bool`
    fn as_bool(&self) -> Option<bool>;
    /// Get the value as int64. returns `None` if the value_type is not an `Int64`
    fn as_int64(&self) -> Option<i64>;
    /// Get the value as double. returns `None` if the value_type is not an `Double`
    fn as_double(&self) -> Option<f64>;
    /// Get the value as bytes. returns `None` if the value_type is not an `Bytes`
    fn as_bytes(&self) -> Option<&[u8]>;

    /* ---------- composite ---------- */
    /// Get the value as an array. Returns `None` if the value_type is not `Array`
    fn as_array(&self) -> Option<Self::ArrayIter<'_>>;

    /// Get the value as a list of kv pairs. Returns `None` if the value type is not `KeyValueList`
    fn as_kvlist(&self) -> Option<Self::KeyValueIter<'_>>;

    /* ---------- helper ---------- */
    /// helper method to determine if the underlying value is a scalar.
    #[inline(always)]
    fn is_scalar(&self) -> bool {
        !matches!(
            self.value_type(),
            ValueType::Array | ValueType::KeyValueList
        )
    }
}

/// Enum representing the type of some AnyValue
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ValueType {
    /// the value is Empty/Null
    Empty,

    /// the type of the value is a String
    String,

    /// the type of the value is a Bool
    Bool,

    /// the type of the value is an Int64
    Int64,

    /// the type of the value is a Double precision floating point number
    Double,

    /// the value is an array of o of values
    Array,

    /// the value is an array of key-value pairs (possibly representing a map)
    KeyValueList,

    /// the value is an array of bytes
    Bytes,
}

/// View for Key-Value attribute
pub trait AttributeView {
    /// The `AnyValueView` trait associated with this impl of `AttributeView`
    type Val<'val>: AnyValueView<'val>
    where
        Self: 'val;

    /// access the key of this attribute
    fn key(&self) -> Str<'_>;

    /// access the value of the attribute. This will return `None` if the value is "empty"
    fn value(&self) -> Option<Self::Val<'_>>;
}

/// View for the instrumentation scope
pub trait InstrumentationScopeView {
    /// The `AttributeView` type associated with this impl of the `InstrumentationScopeView` trait
    /// for accessing this Scope's attributes
    type Attribute<'att>: AttributeView
    where
        Self: 'att;

    /// The associated attribute iterator type for this impl of the trait. The iterator will yield
    /// borrowed references that must live as long as the lifetime 'att
    type AttributeIter<'att>: Iterator<Item = Self::Attribute<'att>>
    where
        Self: 'att;

    /// access the name of the scope. This should return `None` if the name is not known
    fn name(&self) -> Option<Str<'_>>;

    /// the version of the scope. This should return `None` if the version is not known
    fn version(&self) -> Option<Str<'_>>;

    /// Access the scope's attributes
    fn attributes(&self) -> Self::AttributeIter<'_>;

    /// Access this scope's dropped attributes.The value is 0 when no attributes were dropped.
    fn dropped_attributes_count(&self) -> u32;
}
