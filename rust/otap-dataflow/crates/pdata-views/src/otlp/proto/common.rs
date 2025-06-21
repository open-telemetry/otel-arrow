// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains the implementation of the pdata View traits for proto message structs
//! from otlp common.proto.

use std::borrow::Cow;

use otel_arrow_rust::proto::opentelemetry::common::v1::{
    AnyValue, InstrumentationScope, KeyValue, any_value,
};

use crate::views::common::{AnyValueView, AttributeView, InstrumentationScopeView, Str, ValueType};

/* ───────────────────────────── VIEW WRAPPERS (zero-alloc) ────────────── */

/// Lightweight wrapper so that `val()` can return `&Self::Val` without the
/// double-reference gymnastics.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct ObjAny<'a>(pub &'a AnyValue);

/// Lightweight wrapper for a Key-Value pair that implements the `AttributeView` trait
#[derive(Clone, Copy)]
pub struct ObjKeyValue<'a> {
    key: &'a str,
    val: Option<ObjAny<'a>>,
}

impl<'a> ObjKeyValue<'a> {
    /// create a new instance of `ObjKeyValue`
    #[must_use]
    pub fn new(key: &'a str, val: Option<ObjAny<'a>>) -> Self {
        Self { key, val }
    }
}

/// Lightweight wrapper around `InstrumentationScope` that implements the
/// `InstrumentationScopeView` trait
#[derive(Clone, Copy)]
pub struct ObjInstrumentationScope<'a> {
    inner: &'a InstrumentationScope,
}

impl<'a> ObjInstrumentationScope<'a> {
    /// Create a new instance of `ObjInstrumentationScope`
    #[must_use]
    pub fn new(inner: &'a InstrumentationScope) -> Self {
        Self { inner }
    }
}

/* ───────────────────────────── ADAPTER ITERATORS ─────────────────────── */

/// Iterator of Key-Value pairs. Can be used for types that must return an iterator of such
/// type (for example, as an accessor for some message's attributes).
#[derive(Clone)]
pub struct KeyValueIter<'a> {
    it: std::slice::Iter<'a, KeyValue>,
}

impl<'a> KeyValueIter<'a> {
    /// Create a new instance of `KeyValueIter`
    #[must_use]
    pub fn new(it: std::slice::Iter<'a, KeyValue>) -> Self {
        Self { it }
    }
}

impl<'a> Iterator for KeyValueIter<'a> {
    type Item = ObjKeyValue<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.it.next().map(|kv| ObjKeyValue {
            key: &kv.key,
            val: kv.value.as_ref().map(ObjAny),
        })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.it.size_hint()
    }
}

/// Iterator of AnyValues
#[derive(Clone)]
pub struct AnyValueIter<'a> {
    it: std::slice::Iter<'a, AnyValue>,
}

impl<'a> Iterator for AnyValueIter<'a> {
    type Item = ObjAny<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.it.next().map(ObjAny)
    }
}

/* ───────────────────────────── TRAIT IMPLEMENTATIONS ─────────────────── */

impl<'a> AnyValueView<'a> for ObjAny<'a> {
    type KeyValue = ObjKeyValue<'a>;

    type KeyValueIter<'kv>
        = KeyValueIter<'a>
    where
        Self: 'kv;

    type ArrayIter<'att>
        = AnyValueIter<'a>
    where
        Self: 'att;

    fn value_type(&self) -> ValueType {
        match self.0.value.as_ref() {
            Some(val) => val.into(),
            None => ValueType::Empty,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        self.0.value.as_ref().and_then(|v| match v {
            any_value::Value::BoolValue(b) => Some(*b),
            _ => None,
        })
    }

    fn as_bytes(&self) -> Option<&[u8]> {
        self.0.value.as_ref().and_then(|v| match v {
            any_value::Value::BytesValue(b) => Some(b.as_slice()),
            _ => None,
        })
    }

    fn as_double(&self) -> Option<f64> {
        self.0.value.as_ref().and_then(|v| match v {
            any_value::Value::DoubleValue(f) => Some(*f),
            _ => None,
        })
    }

    fn as_int64(&self) -> Option<i64> {
        self.0.value.as_ref().and_then(|v| match v {
            any_value::Value::IntValue(i) => Some(*i),
            _ => None,
        })
    }

    fn as_string(&self) -> Option<Str<'_>> {
        self.0.value.as_ref().and_then(|v| match v {
            any_value::Value::StringValue(s) => Some(Cow::Borrowed(s.as_str())),
            _ => None,
        })
    }

    fn as_array(&self) -> Option<Self::ArrayIter<'_>> {
        if let Some(any_value::Value::ArrayValue(ref vec)) = self.0.value {
            let values = &vec.values;
            Some(AnyValueIter { it: values.iter() })
        } else {
            None
        }
    }

    fn as_kvlist(&self) -> Option<Self::KeyValueIter<'_>> {
        if let Some(any_value::Value::KvlistValue(ref vec)) = self.0.value {
            let vals = &vec.values;
            Some(KeyValueIter { it: vals.iter() })
        } else {
            None
        }
    }
}

impl From<&any_value::Value> for ValueType {
    fn from(value: &any_value::Value) -> Self {
        match value {
            any_value::Value::ArrayValue(_) => ValueType::Array,
            any_value::Value::BoolValue(_) => ValueType::Bool,
            any_value::Value::BytesValue(_) => ValueType::Bytes,
            any_value::Value::DoubleValue(_) => ValueType::Double,
            any_value::Value::IntValue(_) => ValueType::Int64,
            any_value::Value::KvlistValue(_) => ValueType::KeyValueList,
            any_value::Value::StringValue(_) => ValueType::String,
        }
    }
}

impl AttributeView for ObjKeyValue<'_> {
    type Val<'val>
        = ObjAny<'val>
    where
        Self: 'val;

    fn key(&self) -> Str<'_> {
        Cow::Borrowed(self.key)
    }

    fn value(&self) -> Option<&Self::Val<'_>> {
        self.val.as_ref()
    }
}

impl InstrumentationScopeView for ObjInstrumentationScope<'_> {
    type Attribute<'b>
        = ObjKeyValue<'b>
    where
        Self: 'b;
    type AttributeIter<'b>
        = KeyValueIter<'b>
    where
        Self: 'b;

    fn name(&self) -> Option<Str<'_>> {
        if !self.inner.name.is_empty() {
            Some(Cow::Borrowed(self.inner.name.as_str()))
        } else {
            None
        }
    }

    fn version(&self) -> Option<Str<'_>> {
        if !self.inner.version.is_empty() {
            Some(Cow::Borrowed(self.inner.version.as_str()))
        } else {
            None
        }
    }

    fn attributes(&self) -> Self::AttributeIter<'_> {
        KeyValueIter::new(self.inner.attributes.iter())
    }

    fn dropped_attributes_count(&self) -> u32 {
        self.inner.dropped_attributes_count
    }
}
