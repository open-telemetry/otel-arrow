// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::{cell::Ref, fmt::Display};

use data_engine_expressions::*;
use regex::Regex;

use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub enum BorrowSource {
    Source,
    Variable,
}

#[derive(Debug)]
pub enum ResolvedValue<'a> {
    /// A value resolved from the expression tree or an attached record
    Value(Value<'a>),

    /// A value borrowed from the source being modified by the engine or
    /// borrowed from a variable
    Borrowed(BorrowSource, Ref<'a, dyn AsStaticValue + 'static>),

    /// A value computed by the engine as the result of a dynamic expression
    Computed(OwnedValue),

    /// A slice of characters from a string or items from an array
    Slice(Slice<'a>),

    /// A list of resolved values
    List(List<'a>),
}

impl<'a> ResolvedValue<'a> {
    pub fn copy_if_borrowed_from_target(&mut self, target: &MutableValueExpression) -> bool {
        let v = match self {
            ResolvedValue::Borrowed(b, v) => Some((b, v.to_value())),
            ResolvedValue::Slice(s) => {
                return s.copy_if_borrowed_from_target(target);
            }
            ResolvedValue::List(l) => {
                return l.copy_if_borrowed_from_target(target);
            }
            _ => None,
        };

        if let Some((s, v)) = v {
            let writing_while_holding_borrow = match target {
                MutableValueExpression::Source(_) => {
                    matches!(s, BorrowSource::Source)
                }
                MutableValueExpression::Variable(_) => {
                    matches!(s, BorrowSource::Variable)
                }
            };

            if writing_while_holding_borrow {
                *self = ResolvedValue::Computed(v.into());
                return true;
            }
        }

        false
    }

    pub fn try_resolve_string(self) -> Result<ResolvedStringValue<'a>, Self> {
        if self.get_value_type() != ValueType::String {
            return Err(self);
        }

        match self {
            ResolvedValue::Value(v) => {
                if let Value::String(s) = v {
                    Ok(ResolvedStringValue::Value(s))
                } else {
                    panic!()
                }
            }
            ResolvedValue::Borrowed(s, b) => {
                match Ref::filter_map(b, |v| {
                    if let StaticValue::String(s) = v.to_static_value() {
                        Some(s)
                    } else {
                        None
                    }
                }) {
                    Ok(v) => Ok(ResolvedStringValue::Borrowed(s, v)),
                    Err(_) => panic!(),
                }
            }
            ResolvedValue::Computed(o) => {
                if let OwnedValue::String(s) = o {
                    Ok(ResolvedStringValue::Computed(s))
                } else {
                    panic!()
                }
            }
            ResolvedValue::Slice(s) => match s {
                Slice::Array(_) => panic!(),
                Slice::String(s) => Ok(ResolvedStringValue::Slice(s.into())),
            },
            ResolvedValue::List(_) => panic!(),
        }
    }

    pub fn try_resolve_regex(self) -> Result<ResolvedRegexValue<'a>, Self> {
        if self.get_value_type() != ValueType::Regex {
            return Err(self);
        }

        match self {
            ResolvedValue::Value(v) => {
                if let Value::Regex(s) = v {
                    Ok(ResolvedRegexValue::Value(s))
                } else {
                    panic!()
                }
            }
            ResolvedValue::Borrowed(_, b) => {
                match Ref::filter_map(b, |v| {
                    if let StaticValue::Regex(r) = v.to_static_value() {
                        Some(r)
                    } else {
                        None
                    }
                }) {
                    Ok(v) => Ok(ResolvedRegexValue::Borrowed(v)),
                    Err(_) => panic!(),
                }
            }
            ResolvedValue::Computed(o) => {
                if let OwnedValue::Regex(s) = o {
                    Ok(ResolvedRegexValue::Computed(s))
                } else {
                    panic!()
                }
            }
            ResolvedValue::Slice(_) => panic!(),
            ResolvedValue::List(_) => panic!(),
        }
    }

    pub fn try_resolve_array(self) -> Result<ResolvedArrayValue<'a>, Self> {
        if self.get_value_type() != ValueType::Array {
            return Err(self);
        }

        match self {
            ResolvedValue::Value(v) => {
                if let Value::Array(s) = v {
                    Ok(ResolvedArrayValue::Value(s))
                } else {
                    panic!()
                }
            }
            ResolvedValue::Borrowed(s, b) => {
                match Ref::filter_map(b, |v| {
                    if let StaticValue::Array(s) = v.to_static_value() {
                        Some(s)
                    } else {
                        None
                    }
                }) {
                    Ok(v) => Ok(ResolvedArrayValue::Borrowed(s, v)),
                    Err(_) => panic!(),
                }
            }
            ResolvedValue::Computed(o) => {
                if let OwnedValue::Array(s) = o {
                    Ok(ResolvedArrayValue::Computed(s))
                } else {
                    panic!()
                }
            }
            ResolvedValue::Slice(s) => match s {
                Slice::Array(a) => Ok(ResolvedArrayValue::Slice(a.into())),
                Slice::String(_) => panic!(),
            },
            ResolvedValue::List(l) => Ok(ResolvedArrayValue::List(l)),
        }
    }
}

impl From<ResolvedValue<'_>> for OwnedValue {
    fn from(val: ResolvedValue<'_>) -> Self {
        match val {
            ResolvedValue::Value(v) => v.into(),
            ResolvedValue::Borrowed(_, b) => b.to_value().into(),
            ResolvedValue::Computed(o) => o,
            ResolvedValue::Slice(s) => match &s {
                Slice::Array(a) => Value::Array(a).into(),
                Slice::String(s) => Value::String(s).into(),
            },
            ResolvedValue::List(l) => Value::Array(&l).into(),
        }
    }
}

impl AsValue for ResolvedValue<'_> {
    fn get_value_type(&self) -> ValueType {
        match self {
            ResolvedValue::Value(v) => v.get_value_type(),
            ResolvedValue::Borrowed(_, b) => b.get_value_type(),
            ResolvedValue::Computed(c) => c.get_value_type(),
            ResolvedValue::Slice(s) => match s {
                Slice::Array(_) => ValueType::Array,
                Slice::String(_) => ValueType::String,
            },
            ResolvedValue::List(_) => ValueType::Array,
        }
    }

    fn to_value(&self) -> Value<'_> {
        match self {
            ResolvedValue::Value(v) => v.clone(),
            ResolvedValue::Borrowed(_, b) => b.to_value(),
            ResolvedValue::Computed(c) => c.to_value(),
            ResolvedValue::Slice(s) => match s {
                Slice::Array(a) => Value::Array(a),
                Slice::String(s) => Value::String(s),
            },
            ResolvedValue::List(l) => Value::Array(l),
        }
    }
}

impl Display for ResolvedValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_value().fmt(f)
    }
}

#[derive(Debug)]
pub struct List<'a> {
    values: Vec<ResolvedValue<'a>>,
}

impl<'a> List<'a> {
    pub fn new(values: Vec<ResolvedValue<'a>>) -> List<'a> {
        Self { values }
    }

    pub fn copy_if_borrowed_from_target(&mut self, target: &MutableValueExpression) -> bool {
        let mut copied = false;

        for value in &mut self.values {
            if value.copy_if_borrowed_from_target(target) {
                copied = true;
            }
        }

        copied
    }
}

impl ArrayValue for List<'_> {
    fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    fn len(&self) -> usize {
        self.values.len()
    }

    fn get(&self, index: usize) -> Option<&(dyn AsValue)> {
        self.values.get(index).map(|v| v as &dyn AsValue)
    }

    fn get_static(&self, _: usize) -> Result<Option<&(dyn AsStaticValue + 'static)>, String> {
        Err("List does not support static access".to_string())
    }

    fn get_item_range(
        &self,
        range: ArrayRange,
        item_callback: &mut dyn IndexValueCallback,
    ) -> bool {
        for (index, value) in range.get_slice(&self.values).iter().enumerate() {
            if !item_callback.next(index, value.to_value()) {
                return false;
            }
        }

        true
    }
}

#[derive(Debug)]
pub enum Slice<'a> {
    Array(ArraySlice<'a>),
    String(StringSlice<'a>),
}

impl Slice<'_> {
    pub fn copy_if_borrowed_from_target(&mut self, target: &MutableValueExpression) -> bool {
        match self {
            Slice::Array(a) => a.inner_value.copy_if_borrowed_from_target(target),
            Slice::String(s) => s.inner_value.copy_if_borrowed_from_target(target),
        }
    }
}

#[derive(Debug)]
pub struct ArraySlice<'a> {
    inner_value: ResolvedArrayValue<'a>,
    range_start_inclusive: usize,
    range_end_exclusive: usize,
}

impl<'a> ArraySlice<'a> {
    pub fn new(
        inner_value: ResolvedArrayValue<'a>,
        range_start_inclusive: usize,
        range_end_exclusive: usize,
    ) -> ArraySlice<'a> {
        Self {
            inner_value,
            range_start_inclusive,
            range_end_exclusive,
        }
    }
}

impl ArrayValue for ArraySlice<'_> {
    fn is_empty(&self) -> bool {
        self.range_end_exclusive - self.range_start_inclusive > 0
    }

    fn len(&self) -> usize {
        self.range_end_exclusive - self.range_start_inclusive
    }

    fn get(&self, index: usize) -> Option<&(dyn AsValue)> {
        self.inner_value.get(self.range_start_inclusive + index)
    }

    fn get_static(&self, index: usize) -> Result<Option<&(dyn AsStaticValue + 'static)>, String> {
        self.inner_value
            .get_static(self.range_start_inclusive + index)
    }

    fn get_item_range(
        &self,
        range: ArrayRange,
        item_callback: &mut dyn IndexValueCallback,
    ) -> bool {
        let start = range
            .get_start_range_inclusize()
            .map(|v| v + self.range_start_inclusive)
            .unwrap_or(self.range_start_inclusive);
        let end = range
            .get_end_range_exclusive()
            .map(|v| v + self.range_start_inclusive)
            .unwrap_or(self.range_end_exclusive);

        if end > self.range_end_exclusive {
            panic!(
                "range end index {} out of range for slice of length {}",
                range.get_end_range_exclusive().unwrap_or(0),
                self.range_end_exclusive - self.range_start_inclusive
            )
        }

        self.inner_value
            .get_item_range((start..end).into(), item_callback)
    }
}

#[derive(Debug)]
pub struct StringSlice<'a> {
    inner_value: ResolvedStringValue<'a>,
    range_start_inclusive: usize,
    range_end_exclusive: usize,
}

impl<'a> StringSlice<'a> {
    pub fn new(
        inner_value: ResolvedStringValue<'a>,
        range_start_inclusive: usize,
        range_end_exclusive: usize,
    ) -> StringSlice<'a> {
        Self {
            inner_value,
            range_start_inclusive,
            range_end_exclusive,
        }
    }
}

impl StringValue for StringSlice<'_> {
    fn get_value(&self) -> &str {
        // Note: Slice of a str returns raw utf8 bytes. Chars can take 1 to 4
        // bytes. In order to correctly slice the str as chars we have to find
        // the correct byte indices to do the slicing
        let count = self.range_end_exclusive - self.range_start_inclusive;
        let mut chars = self
            .inner_value
            .get_value()
            .char_indices()
            .skip(self.range_start_inclusive)
            .take(count);

        let value = self.inner_value.get_value();

        if let Some(first) = chars.next() {
            if let Some(last) = chars.last() {
                let mut buf = [0; 4];
                let encoded = last.1.encode_utf8(&mut buf);

                &value[first.0..(last.0 + encoded.len())]
            } else {
                let mut buf = [0; 4];
                let encoded = first.1.encode_utf8(&mut buf);

                &value[first.0..(first.0 + encoded.len())]
            }
        } else {
            &value[0..0]
        }
    }
}

#[derive(Debug)]
pub enum ResolvedStringValue<'a> {
    /// A value resolved from the expression tree or an attached record
    Value(&'a dyn StringValue),

    /// A value borrowed from the record being modified by the engine
    Borrowed(BorrowSource, Ref<'a, dyn StringValue + 'static>),

    /// A value computed by the engine as the result of a dynamic expression
    Computed(StringValueStorage),

    /// A slice of characters from a string
    Slice(Box<StringSlice<'a>>),
}

impl ResolvedStringValue<'_> {
    pub fn copy_if_borrowed_from_target(&mut self, target: &MutableValueExpression) -> bool {
        let v = match self {
            ResolvedStringValue::Borrowed(b, v) => Some((b, v)),
            ResolvedStringValue::Slice(s) => {
                return s.inner_value.copy_if_borrowed_from_target(target);
            }
            _ => None,
        };

        if let Some((s, v)) = v {
            let writing_while_holding_borrow = match target {
                MutableValueExpression::Source(_) => {
                    matches!(s, BorrowSource::Source)
                }
                MutableValueExpression::Variable(_) => {
                    matches!(s, BorrowSource::Variable)
                }
            };

            if writing_while_holding_borrow {
                *self =
                    ResolvedStringValue::Computed(StringValueStorage::new(v.get_value().into()));
                return true;
            }
        }

        false
    }
}

impl StringValue for ResolvedStringValue<'_> {
    fn get_value(&self) -> &str {
        match self {
            ResolvedStringValue::Value(s) => s.get_value(),
            ResolvedStringValue::Borrowed(_, b) => b.get_value(),
            ResolvedStringValue::Computed(v) => v.get_raw_value(),
            ResolvedStringValue::Slice(s) => s.get_value(),
        }
    }
}

impl AsValue for ResolvedStringValue<'_> {
    fn get_value_type(&self) -> ValueType {
        ValueType::String
    }

    fn to_value(&self) -> Value<'_> {
        match self {
            ResolvedStringValue::Value(v) => Value::String(*v),
            ResolvedStringValue::Borrowed(_, b) => Value::String(&**b),
            ResolvedStringValue::Computed(c) => Value::String(c),
            ResolvedStringValue::Slice(s) => Value::String(&**s),
        }
    }
}

impl Display for ResolvedStringValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_value().fmt(f)
    }
}

#[derive(Debug)]
pub enum ResolvedRegexValue<'a> {
    /// A value resolved from the expression tree or an attached record
    Value(&'a dyn RegexValue),

    /// A value borrowed from the record being modified by the engine
    Borrowed(Ref<'a, dyn RegexValue + 'static>),

    /// A value computed by the engine as the result of a dynamic expression
    Computed(RegexValueStorage),
}

impl RegexValue for ResolvedRegexValue<'_> {
    fn get_value(&self) -> &Regex {
        match self {
            ResolvedRegexValue::Value(s) => s.get_value(),
            ResolvedRegexValue::Borrowed(b) => b.get_value(),
            ResolvedRegexValue::Computed(v) => v.get_raw_value(),
        }
    }
}

impl AsValue for ResolvedRegexValue<'_> {
    fn get_value_type(&self) -> ValueType {
        ValueType::String
    }

    fn to_value(&self) -> Value<'_> {
        match self {
            ResolvedRegexValue::Value(v) => Value::Regex(*v),
            ResolvedRegexValue::Borrowed(b) => Value::Regex(&**b),
            ResolvedRegexValue::Computed(c) => Value::Regex(c),
        }
    }
}

impl Display for ResolvedRegexValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_value().fmt(f)
    }
}

#[derive(Debug)]
pub enum ResolvedArrayValue<'a> {
    /// A value resolved from the expression tree or an attached record
    Value(&'a dyn ArrayValue),

    /// A value borrowed from the record being modified by the engine
    Borrowed(BorrowSource, Ref<'a, dyn ArrayValue + 'static>),

    /// A value computed by the engine as the result of a dynamic expression
    Computed(ArrayValueStorage<OwnedValue>),

    /// A slice of items from an array
    Slice(Box<ArraySlice<'a>>),

    /// A list of resolved values
    List(List<'a>),
}

impl ResolvedArrayValue<'_> {
    pub fn copy_if_borrowed_from_target(&mut self, target: &MutableValueExpression) -> bool {
        let v = match self {
            ResolvedArrayValue::Borrowed(b, v) => Some((b, v)),
            ResolvedArrayValue::Slice(s) => {
                return s.inner_value.copy_if_borrowed_from_target(target);
            }
            ResolvedArrayValue::List(l) => {
                return l.copy_if_borrowed_from_target(target);
            }
            _ => None,
        };

        if let Some((s, v)) = v {
            let writing_while_holding_borrow = match target {
                MutableValueExpression::Source(_) => {
                    matches!(s, BorrowSource::Source)
                }
                MutableValueExpression::Variable(_) => {
                    matches!(s, BorrowSource::Variable)
                }
            };

            if writing_while_holding_borrow {
                *self = ResolvedArrayValue::Computed((&**v).into());
                return true;
            }
        }

        false
    }

    fn get_array(&self) -> &dyn ArrayValue {
        match self {
            ResolvedArrayValue::Value(v) => *v,
            ResolvedArrayValue::Borrowed(_, b) => &**b,
            ResolvedArrayValue::Computed(c) => c,
            ResolvedArrayValue::Slice(s) => &**s,
            ResolvedArrayValue::List(l) => l,
        }
    }
}

impl ArrayValue for ResolvedArrayValue<'_> {
    fn is_empty(&self) -> bool {
        self.get_array().is_empty()
    }

    fn len(&self) -> usize {
        self.get_array().len()
    }

    fn get(&self, index: usize) -> Option<&(dyn AsValue)> {
        self.get_array().get(index)
    }

    fn get_static(&self, index: usize) -> Result<Option<&(dyn AsStaticValue + 'static)>, String> {
        self.get_array().get_static(index)
    }

    fn get_item_range(
        &self,
        range: ArrayRange,
        item_callback: &mut dyn IndexValueCallback,
    ) -> bool {
        self.get_array().get_item_range(range, item_callback)
    }
}

impl AsValue for ResolvedArrayValue<'_> {
    fn get_value_type(&self) -> ValueType {
        ValueType::Array
    }

    fn to_value(&self) -> Value<'_> {
        Value::Array(self.get_array())
    }
}

impl Display for ResolvedArrayValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_value().fmt(f)
    }
}
