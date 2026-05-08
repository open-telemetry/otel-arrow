// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Attribute and `AnyValue` encoders for STEF metrics records.
//!
//! Attribute sets are delta-encoded against the previous record. The optimized path handles
//! repeated keys and unchanged values without rebuilding full attribute state for every point.

use std::borrow::Cow;
use std::rc::Rc;

use super::super::wire::{
    BitWriter, BoolEncoder, BytesEncoder, BytesWriter, Float64Encoder, I64Encoder,
    SharedStringDict, StringEncoder, any_value_column_tree, attributes_column_tree,
};
use super::*;
use crate::otlp::attributes::{AttributeArrays, AttributeValueType};
use crate::otlp::common::AnyValueArrays;
use arrow::array::Array;
use arrow::datatypes::ArrowPrimitiveType;
use otap_df_pdata_views::views::common::{AnyValueScalar, AnyValueView, AttributeView, ValueType};

#[derive(Default)]
pub(super) struct AttributesEncoder {
    header: BytesWriter,
    key: StringEncoder,
    value: AnyValueEncoder,
    last: Vec<DirectAttribute>,
    changed_values: Vec<(usize, DirectAnyValue)>,
}

impl AttributesEncoder {
    pub(super) fn new(attribute_key: SharedStringDict, any_value_string: SharedStringDict) -> Self {
        Self {
            key: StringEncoder::with_dict(attribute_key),
            value: AnyValueEncoder::new(any_value_string),
            ..Self::default()
        }
    }

    pub(super) fn encode_empty(&mut self) {
        self.header.write_uvarint(0b1);
        self.last.clear();
        self.changed_values.clear();
    }

    pub(super) fn encode_view<A>(
        &mut self,
        mut attributes: impl Iterator<Item = A>,
    ) -> Result<(), Error>
    where
        A: AttributeView,
    {
        if self.last.is_empty() || self.last.len() >= 63 {
            return self.encode_full_view(attributes);
        }

        let last_len = self.last.len();
        let mut changed = 0_u64;
        let mut seen = 0_usize;
        self.changed_values.clear();

        while let Some(attribute) = attributes.next() {
            if seen >= last_len {
                return self.encode_full_view_from_seen(seen, Some(attribute), attributes);
            }

            let last = &self.last[seen];
            if attribute.key() != last.key.as_ref().as_bytes() {
                return self.encode_full_view_from_seen(seen, Some(attribute), attributes);
            }

            if let Some(value) =
                direct_any_value_from_view_if_changed(attribute.value(), &last.value)?
            {
                changed |= 1 << seen;
                self.changed_values.push((seen, value));
            }
            seen += 1;
        }

        if seen != last_len {
            return self.encode_full_view_from_seen(seen, None, std::iter::empty::<A>());
        }

        self.header.write_uvarint(changed << 1);
        let mut changed_values = std::mem::take(&mut self.changed_values);
        for (index, value) in changed_values.drain(..) {
            self.value.encode(&value)?;
            self.last[index].value = value;
        }
        self.changed_values = changed_values;
        Ok(())
    }

    pub(super) fn encode_full_view<A>(
        &mut self,
        attributes: impl Iterator<Item = A>,
    ) -> Result<(), Error>
    where
        A: AttributeView,
    {
        self.last.clear();
        for attribute in attributes {
            self.encode_full_attribute(attribute)?;
        }
        self.header
            .write_uvarint((self.last.len() as u64) << 1 | 0b1);
        Ok(())
    }

    pub(super) fn encode_full_view_from_seen<A>(
        &mut self,
        seen: usize,
        current: Option<A>,
        remaining: impl Iterator<Item = A>,
    ) -> Result<(), Error>
    where
        A: AttributeView,
    {
        let old_last = std::mem::take(&mut self.last);
        let changed_values = std::mem::take(&mut self.changed_values);
        let mut changed_values = changed_values.into_iter().peekable();

        self.last.reserve(seen + usize::from(current.is_some()));
        for (index, previous) in old_last.into_iter().take(seen).enumerate() {
            let mut value = previous.value;
            if changed_values
                .peek()
                .is_some_and(|(changed_index, _)| *changed_index == index)
            {
                value = changed_values.next().expect("peeked changed value").1;
            }

            self.key.encode(previous.key.as_ref());
            self.value.encode(&value)?;
            self.last.push(DirectAttribute {
                key: previous.key,
                value,
            });
        }

        if let Some(attribute) = current {
            self.encode_full_attribute(attribute)?;
        }
        for attribute in remaining {
            self.encode_full_attribute(attribute)?;
        }

        self.header
            .write_uvarint((self.last.len() as u64) << 1 | 0b1);
        Ok(())
    }

    pub(super) fn encode_full_attribute<A>(&mut self, attribute: A) -> Result<(), Error>
    where
        A: AttributeView,
    {
        let (key, value) = attribute.key_scalar_value();
        let key = std::str::from_utf8(key).map_err(|_| Error::InvalidUtf8("attribute.key"))?;
        self.key.encode(key);
        let value = self
            .value
            .encode_scalar(value.unwrap_or(AnyValueScalar::Empty))?;
        self.last.push(DirectAttribute {
            key: key.into(),
            value,
        });
        Ok(())
    }

    pub(super) fn encode_otap_attributes<T>(
        &mut self,
        attrs: Option<&AttributeArrays<'_, T>>,
        parent_id: Option<T::Native>,
        cursor: &mut SortedBatchCursor,
    ) -> Result<(), Error>
    where
        T: ArrowPrimitiveType,
    {
        let (Some(attrs), Some(parent_id)) = (attrs, parent_id) else {
            self.encode_empty();
            return Ok(());
        };

        let mut rows = ChildIndexIter::new(parent_id, &attrs.parent_id, cursor);
        if self.last.is_empty() || self.last.len() >= 63 {
            return self.encode_full_otap_attributes(attrs, rows);
        }

        let last_len = self.last.len();
        let mut changed = 0_u64;
        let mut seen = 0_usize;
        self.changed_values.clear();

        while let Some(row) = rows.next() {
            let Some(key) = attrs.attr_key.str_at(row) else {
                continue;
            };
            if seen >= last_len {
                return self.encode_full_otap_attributes_from_seen(attrs, seen, Some(row), rows);
            }

            let last = &self.last[seen];
            if key.as_bytes() != last.key.as_ref().as_bytes() {
                return self.encode_full_otap_attributes_from_seen(attrs, seen, Some(row), rows);
            }

            if let Some(value) =
                direct_any_value_from_otap_if_changed(&attrs.anyval_arrays, row, &last.value)
            {
                changed |= 1 << seen;
                self.changed_values.push((seen, value));
            }
            seen += 1;
        }

        if seen != last_len {
            return self.encode_full_otap_attributes_from_seen(
                attrs,
                seen,
                None,
                std::iter::empty(),
            );
        }

        self.header.write_uvarint(changed << 1);
        let mut changed_values = std::mem::take(&mut self.changed_values);
        for (index, value) in changed_values.drain(..) {
            self.value.encode(&value)?;
            self.last[index].value = value;
        }
        self.changed_values = changed_values;
        Ok(())
    }

    pub(super) fn encode_full_otap_attributes<T>(
        &mut self,
        attrs: &AttributeArrays<'_, T>,
        rows: impl Iterator<Item = usize>,
    ) -> Result<(), Error>
    where
        T: ArrowPrimitiveType,
    {
        self.last.clear();
        for row in rows {
            self.encode_full_otap_attribute(attrs, row)?;
        }
        self.header
            .write_uvarint((self.last.len() as u64) << 1 | 0b1);
        Ok(())
    }

    pub(super) fn encode_full_otap_attributes_from_seen<T>(
        &mut self,
        attrs: &AttributeArrays<'_, T>,
        seen: usize,
        current: Option<usize>,
        remaining: impl Iterator<Item = usize>,
    ) -> Result<(), Error>
    where
        T: ArrowPrimitiveType,
    {
        let old_last = std::mem::take(&mut self.last);
        let changed_values = std::mem::take(&mut self.changed_values);
        let mut changed_values = changed_values.into_iter().peekable();

        self.last.reserve(seen + usize::from(current.is_some()));
        for (index, previous) in old_last.into_iter().take(seen).enumerate() {
            let mut value = previous.value;
            if changed_values
                .peek()
                .is_some_and(|(changed_index, _)| *changed_index == index)
            {
                value = changed_values.next().expect("peeked changed value").1;
            }

            self.key.encode(previous.key.as_ref());
            self.value.encode(&value)?;
            self.last.push(DirectAttribute {
                key: previous.key,
                value,
            });
        }

        if let Some(row) = current {
            self.encode_full_otap_attribute(attrs, row)?;
        }
        for row in remaining {
            self.encode_full_otap_attribute(attrs, row)?;
        }

        self.header
            .write_uvarint((self.last.len() as u64) << 1 | 0b1);
        Ok(())
    }

    pub(super) fn encode_full_otap_attribute<T>(
        &mut self,
        attrs: &AttributeArrays<'_, T>,
        row: usize,
    ) -> Result<(), Error>
    where
        T: ArrowPrimitiveType,
    {
        let Some(key) = attrs.attr_key.str_at(row) else {
            return Ok(());
        };
        self.key.encode(key);
        let value = self.value.encode_otap(&attrs.anyval_arrays, row)?;
        self.last.push(DirectAttribute {
            key: key.into(),
            value,
        });
        Ok(())
    }

    pub(super) fn take_column(&mut self) -> Column {
        let mut column = attributes_column_tree();
        column.data = self.header.take_bytes();
        column.children[0].data = self.key.take_bytes();
        column.children[1] = self.value.take_column();
        column
    }
}

#[inline]
pub(super) fn value_type_name(value_type: ValueType) -> &'static str {
    match value_type {
        ValueType::Empty => "empty",
        ValueType::String => "string",
        ValueType::Bool => "bool",
        ValueType::Int64 => "int",
        ValueType::Double => "double",
        ValueType::Array => "array",
        ValueType::KeyValueList => "kvlist",
        ValueType::Bytes => "bytes",
    }
}

pub(super) fn direct_any_value_from_otap(
    anyval: &AnyValueArrays<'_>,
    row: usize,
) -> DirectAnyValue {
    match otap_attribute_value_type(anyval, row) {
        AttributeValueType::Empty => DirectAnyValue::Empty,
        AttributeValueType::Str => anyval
            .attr_str
            .as_ref()
            .and_then(|accessor| accessor.str_at(row))
            .map_or(DirectAnyValue::Empty, |value| {
                DirectAnyValue::String(value.into())
            }),
        AttributeValueType::Int => anyval
            .attr_int
            .as_ref()
            .and_then(|accessor| accessor.value_at(row))
            .map_or(DirectAnyValue::Empty, DirectAnyValue::Int),
        AttributeValueType::Double => anyval
            .attr_double
            .and_then(|arr| {
                arr.is_valid(row)
                    .then(|| DirectAnyValue::Double(arr.value(row)))
            })
            .unwrap_or(DirectAnyValue::Empty),
        AttributeValueType::Bool => anyval
            .attr_bool
            .and_then(|arr| {
                arr.is_valid(row)
                    .then(|| DirectAnyValue::Bool(arr.value(row)))
            })
            .unwrap_or(DirectAnyValue::Empty),
        AttributeValueType::Bytes => anyval
            .attr_bytes
            .as_ref()
            .and_then(|accessor| accessor.slice_at(row))
            .map_or(DirectAnyValue::Empty, |value| {
                DirectAnyValue::Bytes(value.to_vec())
            }),
        AttributeValueType::Map | AttributeValueType::Slice => DirectAnyValue::Empty,
    }
}

pub(super) fn direct_any_value_from_otap_if_changed(
    anyval: &AnyValueArrays<'_>,
    row: usize,
    last: &DirectAnyValue,
) -> Option<DirectAnyValue> {
    match otap_attribute_value_type(anyval, row) {
        AttributeValueType::Empty => {
            (last != &DirectAnyValue::Empty).then_some(DirectAnyValue::Empty)
        }
        AttributeValueType::Str => {
            let value = anyval
                .attr_str
                .as_ref()
                .and_then(|accessor| accessor.str_at(row));
            match value {
                Some(value)
                    if matches!(
                        last,
                        DirectAnyValue::String(last) if last.as_ref() == value
                    ) =>
                {
                    None
                }
                Some(value) => Some(DirectAnyValue::String(value.into())),
                None => (last != &DirectAnyValue::Empty).then_some(DirectAnyValue::Empty),
            }
        }
        AttributeValueType::Int => {
            let value = anyval
                .attr_int
                .as_ref()
                .and_then(|accessor| accessor.value_at(row));
            match value {
                Some(value) if matches!(last, DirectAnyValue::Int(last) if *last == value) => None,
                Some(value) => Some(DirectAnyValue::Int(value)),
                None => (last != &DirectAnyValue::Empty).then_some(DirectAnyValue::Empty),
            }
        }
        AttributeValueType::Double => {
            let value = anyval
                .attr_double
                .and_then(|arr| arr.is_valid(row).then(|| arr.value(row)));
            match value {
                Some(value) if matches!(last, DirectAnyValue::Double(last) if *last == value) => {
                    None
                }
                Some(value) => Some(DirectAnyValue::Double(value)),
                None => (last != &DirectAnyValue::Empty).then_some(DirectAnyValue::Empty),
            }
        }
        AttributeValueType::Bool => {
            let value = anyval
                .attr_bool
                .and_then(|arr| arr.is_valid(row).then(|| arr.value(row)));
            match value {
                Some(value) if matches!(last, DirectAnyValue::Bool(last) if *last == value) => None,
                Some(value) => Some(DirectAnyValue::Bool(value)),
                None => (last != &DirectAnyValue::Empty).then_some(DirectAnyValue::Empty),
            }
        }
        AttributeValueType::Bytes => {
            let value = anyval
                .attr_bytes
                .as_ref()
                .and_then(|accessor| accessor.slice_at(row));
            match value {
                Some(value) if matches!(last, DirectAnyValue::Bytes(last) if last == value) => None,
                Some(value) => Some(DirectAnyValue::Bytes(value.to_vec())),
                None => (last != &DirectAnyValue::Empty).then_some(DirectAnyValue::Empty),
            }
        }
        AttributeValueType::Map | AttributeValueType::Slice => {
            (last != &DirectAnyValue::Empty).then_some(DirectAnyValue::Empty)
        }
    }
}

pub(super) fn otap_attribute_value_type(
    anyval: &AnyValueArrays<'_>,
    row: usize,
) -> AttributeValueType {
    if !anyval.attr_type.is_valid(row) {
        return AttributeValueType::Empty;
    }
    AttributeValueType::try_from(anyval.attr_type.value(row)).unwrap_or(AttributeValueType::Empty)
}

#[inline]
pub(super) fn direct_any_value_from_view_if_changed<'v, V>(
    value: Option<V>,
    last: &DirectAnyValue,
) -> Result<Option<DirectAnyValue>, Error>
where
    V: AnyValueView<'v>,
{
    let Some(value) = value else {
        return Ok((last != &DirectAnyValue::Empty).then_some(DirectAnyValue::Empty));
    };

    match value.value_type() {
        ValueType::Empty => Ok((last != &DirectAnyValue::Empty).then_some(DirectAnyValue::Empty)),
        ValueType::String => {
            let value = value
                .as_string()
                .ok_or(Error::UnsupportedAttributeValue("string"))?;
            if matches!(last, DirectAnyValue::String(last) if last.as_ref().as_bytes() == value) {
                Ok(None)
            } else {
                let value = std::str::from_utf8(value)
                    .map_err(|_| Error::InvalidUtf8("attribute.value.string"))?;
                Ok(Some(DirectAnyValue::String(value.into())))
            }
        }
        ValueType::Bool => {
            let value = value
                .as_bool()
                .ok_or(Error::UnsupportedAttributeValue("bool"))?;
            Ok(
                (!matches!(last, DirectAnyValue::Bool(last) if *last == value))
                    .then_some(DirectAnyValue::Bool(value)),
            )
        }
        ValueType::Int64 => {
            let value = value
                .as_int64()
                .ok_or(Error::UnsupportedAttributeValue("int"))?;
            Ok(
                (!matches!(last, DirectAnyValue::Int(last) if *last == value))
                    .then_some(DirectAnyValue::Int(value)),
            )
        }
        ValueType::Double => {
            let value = value
                .as_double()
                .ok_or(Error::UnsupportedAttributeValue("double"))?;
            Ok(
                (!matches!(last, DirectAnyValue::Double(last) if *last == value))
                    .then_some(DirectAnyValue::Double(value)),
            )
        }
        ValueType::Array => Err(Error::UnsupportedAttributeValue("array")),
        ValueType::KeyValueList => Err(Error::UnsupportedAttributeValue("kvlist")),
        ValueType::Bytes => {
            let value = value
                .as_bytes()
                .ok_or(Error::UnsupportedAttributeValue("bytes"))?;
            if matches!(last, DirectAnyValue::Bytes(last) if last == value) {
                Ok(None)
            } else {
                Ok(Some(DirectAnyValue::Bytes(value.to_vec())))
            }
        }
    }
}

#[derive(Default)]
pub(super) struct AnyValueEncoder {
    bits: BitWriter,
    string: StringEncoder,
    bool_: BoolEncoder,
    int64: I64Encoder,
    float64: Float64Encoder,
    bytes: BytesEncoder,
}

impl AnyValueEncoder {
    pub(super) fn new(string_dict: SharedStringDict) -> Self {
        Self {
            string: StringEncoder::with_dict(string_dict),
            ..Self::default()
        }
    }

    pub(super) fn encode(&mut self, value: &DirectAnyValue) -> Result<(), Error> {
        match value {
            DirectAnyValue::Empty => self.bits.write_bits(0, 4),
            DirectAnyValue::String(value) => {
                self.bits.write_bits(1, 4);
                self.string.encode(value.as_ref());
            }
            DirectAnyValue::Bool(value) => {
                self.bits.write_bits(2, 4);
                self.bool_.encode(*value);
            }
            DirectAnyValue::Int(value) => {
                self.bits.write_bits(3, 4);
                self.int64.encode(*value);
            }
            DirectAnyValue::Double(value) => {
                self.bits.write_bits(4, 4);
                self.float64.encode(*value);
            }
            DirectAnyValue::Bytes(value) => {
                self.bits.write_bits(7, 4);
                self.bytes.encode(value);
            }
        }
        Ok(())
    }

    pub(super) fn encode_scalar(
        &mut self,
        value: AnyValueScalar<'_>,
    ) -> Result<DirectAnyValue, Error> {
        match value {
            AnyValueScalar::Empty => {
                self.bits.write_bits(0, 4);
                Ok(DirectAnyValue::Empty)
            }
            AnyValueScalar::String(value) => self.encode_scalar_string(value),
            AnyValueScalar::Bool(value) => {
                self.bits.write_bits(2, 4);
                self.bool_.encode(value);
                Ok(DirectAnyValue::Bool(value))
            }
            AnyValueScalar::Int64(value) => {
                self.bits.write_bits(3, 4);
                self.int64.encode(value);
                Ok(DirectAnyValue::Int(value))
            }
            AnyValueScalar::Double(value) => {
                self.bits.write_bits(4, 4);
                self.float64.encode(value);
                Ok(DirectAnyValue::Double(value))
            }
            AnyValueScalar::Bytes(value) => {
                self.bits.write_bits(7, 4);
                self.bytes.encode(value.as_ref());
                Ok(DirectAnyValue::Bytes(value.into_owned()))
            }
            AnyValueScalar::Array => Err(Error::UnsupportedAttributeValue("array")),
            AnyValueScalar::KeyValueList => Err(Error::UnsupportedAttributeValue("kvlist")),
            AnyValueScalar::Invalid(value_type) => Err(Error::UnsupportedAttributeValue(
                value_type_name(value_type),
            )),
        }
    }

    pub(super) fn encode_scalar_string(
        &mut self,
        value: Cow<'_, [u8]>,
    ) -> Result<DirectAnyValue, Error> {
        match value {
            Cow::Borrowed(value) => {
                let value = std::str::from_utf8(value)
                    .map_err(|_| Error::InvalidUtf8("attribute.value.string"))?;
                self.bits.write_bits(1, 4);
                self.string.encode(value);
                Ok(DirectAnyValue::String(value.into()))
            }
            Cow::Owned(value) => {
                let value = String::from_utf8(value)
                    .map_err(|_| Error::InvalidUtf8("attribute.value.string"))?;
                self.bits.write_bits(1, 4);
                self.string.encode(&value);
                Ok(DirectAnyValue::String(Rc::<str>::from(value)))
            }
        }
    }

    pub(super) fn encode_otap(
        &mut self,
        anyval: &AnyValueArrays<'_>,
        row: usize,
    ) -> Result<DirectAnyValue, Error> {
        let value = direct_any_value_from_otap(anyval, row);
        self.encode(&value)?;
        Ok(value)
    }

    pub(super) fn take_column(&mut self) -> Column {
        let mut column = any_value_column_tree();
        column.data = self.bits.take_bytes();
        column.children[0].data = self.string.take_bytes();
        column.children[1].data = self.bool_.take_bytes();
        column.children[2].data = self.int64.take_bytes();
        column.children[3].data = self.float64.take_bytes();
        column.children[6].data = self.bytes.take_bytes();
        column
    }
}

#[derive(Clone, PartialEq)]
pub(super) struct DirectAttribute {
    key: Rc<str>,
    value: DirectAnyValue,
}

impl Default for DirectAttribute {
    fn default() -> Self {
        Self {
            key: empty_rc_str(),
            value: DirectAnyValue::Empty,
        }
    }
}

#[derive(Clone, Default, PartialEq)]
pub(super) enum DirectAnyValue {
    #[default]
    Empty,
    String(Rc<str>),
    Bool(bool),
    Int(i64),
    Double(f64),
    Bytes(Vec<u8>),
}

pub(super) fn empty_rc_str() -> Rc<str> {
    Rc::<str>::from("")
}
