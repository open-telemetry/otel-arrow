// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use otap_df_pdata_views::views::common::{
    AnyValueView, AttributeView, InstrumentationScopeView, ValueType,
};
use otap_df_pdata_views::views::resource::ResourceView;
use serde::ser::{Error as _, SerializeMap, SerializeSeq};
use serde::{Serialize, Serializer};
use std::cell::RefCell;
use std::marker::PhantomData;

pub(super) struct Utf8<'a>(pub(super) &'a [u8]);

impl Serialize for Utf8<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = std::str::from_utf8(self.0).map_err(S::Error::custom)?;
        serializer.serialize_str(value)
    }
}

pub(super) struct ProtoDouble(pub(super) f64);

impl Serialize for ProtoDouble {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.0.is_nan() {
            serializer.serialize_str("NaN")
        } else if self.0 == f64::INFINITY {
            serializer.serialize_str("Infinity")
        } else if self.0 == f64::NEG_INFINITY {
            serializer.serialize_str("-Infinity")
        } else {
            serializer.serialize_f64(self.0)
        }
    }
}

pub(super) struct ProtoI64(pub(super) i64);

impl Serialize for ProtoI64 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(&self.0)
    }
}

pub(super) struct ProtoU64(pub(super) u64);

impl Serialize for ProtoU64 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(&self.0)
    }
}

pub(super) struct ResourceJson<'a, R: ResourceView>(pub(super) &'a R);

impl<R: ResourceView> Serialize for ResourceJson<'_, R> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        if self.0.attributes().next().is_some() {
            map.serialize_entry("attributes", &AttributesJson(self.0))?;
        }
        let dropped = self.0.dropped_attributes_count();
        if dropped != 0 {
            map.serialize_entry("droppedAttributesCount", &dropped)?;
        }
        map.end()
    }
}

struct AttributesJson<'a, A: AttributesView>(pub(super) &'a A);

trait AttributesView {
    type Attribute<'a>: AttributeView
    where
        Self: 'a;
    type Iter<'a>: Iterator<Item = Self::Attribute<'a>>
    where
        Self: 'a;
    fn json_attributes(&self) -> Self::Iter<'_>;
}

impl<T: ResourceView> AttributesView for T {
    type Attribute<'a>
        = T::Attribute<'a>
    where
        Self: 'a;
    type Iter<'a>
        = T::AttributesIter<'a>
    where
        Self: 'a;

    fn json_attributes(&self) -> Self::Iter<'_> {
        self.attributes()
    }
}

impl<A: AttributesView> Serialize for AttributesJson<'_, A> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(None)?;
        for attribute in self.0.json_attributes() {
            sequence.serialize_element(&AttributeJson(attribute))?;
        }
        sequence.end()
    }
}

pub(super) struct AttributeIterJson<I>(RefCell<Option<I>>);

impl<I> AttributeIterJson<I> {
    pub(super) const fn new(iter: I) -> Self {
        Self(RefCell::new(Some(iter)))
    }
}

impl<I, A> Serialize for AttributeIterJson<I>
where
    I: Iterator<Item = A>,
    A: AttributeView,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(None)?;
        let mut iter = self
            .0
            .borrow_mut()
            .take()
            .ok_or_else(|| S::Error::custom("attribute iterator serialized more than once"))?;
        for attribute in &mut iter {
            sequence.serialize_element(&AttributeJson(attribute))?;
        }
        sequence.end()
    }
}

pub(super) struct AttributeJson<A: AttributeView>(pub(super) A);

impl<A: AttributeView> Serialize for AttributeJson<A> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        if !self.0.key().is_empty() {
            map.serialize_entry("key", &Utf8(self.0.key()))?;
        }
        if let Some(value) = self.0.value() {
            map.serialize_entry("value", &AnyValueJson::new(value))?;
        }
        map.end()
    }
}

pub(super) struct AnyValueJson<'a, V: AnyValueView<'a>>(V, PhantomData<&'a ()>);

impl<'a, V: AnyValueView<'a>> AnyValueJson<'a, V> {
    pub(super) const fn new(value: V) -> Self {
        Self(value, PhantomData)
    }
}

impl<'a, V: AnyValueView<'a>> Serialize for AnyValueJson<'a, V> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        match self.0.value_type() {
            ValueType::Empty => {}
            ValueType::String => map.serialize_entry(
                "stringValue",
                &Utf8(
                    self.0
                        .as_string()
                        .ok_or_else(|| S::Error::custom("missing string AnyValue"))?,
                ),
            )?,
            ValueType::Bool => map.serialize_entry(
                "boolValue",
                &self
                    .0
                    .as_bool()
                    .ok_or_else(|| S::Error::custom("missing bool AnyValue"))?,
            )?,
            ValueType::Int64 => map.serialize_entry(
                "intValue",
                &ProtoI64(
                    self.0
                        .as_int64()
                        .ok_or_else(|| S::Error::custom("missing int AnyValue"))?,
                ),
            )?,
            ValueType::Double => map.serialize_entry(
                "doubleValue",
                &ProtoDouble(
                    self.0
                        .as_double()
                        .ok_or_else(|| S::Error::custom("missing double AnyValue"))?,
                ),
            )?,
            ValueType::Array => {
                map.serialize_entry("arrayValue", &ArrayValueJson(&self.0, PhantomData))?
            }
            ValueType::KeyValueList => {
                map.serialize_entry("kvlistValue", &KeyValueListJson(&self.0, PhantomData))?
            }
            ValueType::Bytes => map.serialize_entry(
                "bytesValue",
                &BASE64_STANDARD.encode(
                    self.0
                        .as_bytes()
                        .ok_or_else(|| S::Error::custom("missing bytes AnyValue"))?,
                ),
            )?,
        }
        map.end()
    }
}

struct ArrayValueJson<'borrow, 'data, V: AnyValueView<'data>>(&'borrow V, PhantomData<&'data ()>);

impl<'data, V: AnyValueView<'data>> Serialize for ArrayValueJson<'_, 'data, V> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let values = self
            .0
            .as_array()
            .ok_or_else(|| S::Error::custom("missing array AnyValue"))?;
        let mut map = serializer.serialize_map(None)?;
        if self
            .0
            .as_array()
            .is_some_and(|mut values| values.next().is_some())
        {
            map.serialize_entry("values", &AnyValueIterJson::new(values))?;
        }
        map.end()
    }
}

struct AnyValueIterJson<I>(RefCell<Option<I>>);

impl<I> AnyValueIterJson<I> {
    const fn new(iter: I) -> Self {
        Self(RefCell::new(Some(iter)))
    }
}

impl<'a, I, V> Serialize for AnyValueIterJson<I>
where
    I: Iterator<Item = V>,
    V: AnyValueView<'a>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(None)?;
        let mut iter = self
            .0
            .borrow_mut()
            .take()
            .ok_or_else(|| S::Error::custom("AnyValue iterator serialized more than once"))?;
        for value in &mut iter {
            sequence.serialize_element(&AnyValueJson::new(value))?;
        }
        sequence.end()
    }
}

struct KeyValueListJson<'borrow, 'data, V: AnyValueView<'data>>(&'borrow V, PhantomData<&'data ()>);

impl<'data, V: AnyValueView<'data>> Serialize for KeyValueListJson<'_, 'data, V> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let values = self
            .0
            .as_kvlist()
            .ok_or_else(|| S::Error::custom("missing kvlist AnyValue"))?;
        let mut map = serializer.serialize_map(None)?;
        if self
            .0
            .as_kvlist()
            .is_some_and(|mut values| values.next().is_some())
        {
            map.serialize_entry("values", &AttributeIterJson::new(values))?;
        }
        map.end()
    }
}

pub(super) struct ScopeJson<'a, I: InstrumentationScopeView>(pub(super) &'a I);

impl<I: InstrumentationScopeView> Serialize for ScopeJson<'_, I> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        if let Some(name) = self.0.name().filter(|value| !value.is_empty()) {
            map.serialize_entry("name", &Utf8(name))?;
        }
        if let Some(version) = self.0.version().filter(|value| !value.is_empty()) {
            map.serialize_entry("version", &Utf8(version))?;
        }
        if self.0.attributes().next().is_some() {
            map.serialize_entry("attributes", &AttributeIterJson::new(self.0.attributes()))?;
        }
        let dropped = self.0.dropped_attributes_count();
        if dropped != 0 {
            map.serialize_entry("droppedAttributesCount", &dropped)?;
        }
        map.end()
    }
}
