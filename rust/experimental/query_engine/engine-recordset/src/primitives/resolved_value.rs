use std::{cell::Ref, fmt::Display};

use data_engine_expressions::*;
use regex::Regex;

use crate::*;

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
    Slice(Option<BorrowSource>, Slice<'a>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BorrowSource {
    Source,
    Variable,
}

impl<'a> ResolvedValue<'a> {
    pub fn get_borrow_source(&self) -> Option<BorrowSource> {
        match self {
            ResolvedValue::Borrowed(b, _) => Some(b.clone()),
            ResolvedValue::Slice(b, _) => b.clone(),
            _ => None,
        }
    }

    pub fn copy_if_borrowed_from_target(&mut self, target: &MutableValueExpression) -> bool {
        let v = match self {
            ResolvedValue::Borrowed(b, v) => Some((b, v.to_value())),
            ResolvedValue::Slice(Some(b), v) => match v {
                Slice::Array(a) => Some((b, Value::Array(a))),
                Slice::String(s) => Some((b, Value::String(s))),
            },
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
            ResolvedValue::Slice(b, s) => match s {
                Slice::Array(_) => panic!(),
                Slice::String(s) => Ok(ResolvedStringValue::Slice(b, s)),
            },
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
            ResolvedValue::Slice(_, _) => panic!(),
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
            ResolvedValue::Slice(b, s) => match s {
                Slice::Array(a) => Ok(ResolvedArrayValue::Slice(b, a.into())),
                Slice::String(_) => panic!(),
            },
        }
    }
}

impl From<ResolvedValue<'_>> for OwnedValue {
    fn from(val: ResolvedValue<'_>) -> Self {
        match val {
            ResolvedValue::Value(v) => v.into(),
            ResolvedValue::Borrowed(_, b) => b.to_value().into(),
            ResolvedValue::Computed(o) => o,
            ResolvedValue::Slice(_, s) => match &s {
                Slice::Array(a) => Value::Array(a).into(),
                Slice::String(s) => Value::String(s).into(),
            },
        }
    }
}

impl AsValue for ResolvedValue<'_> {
    fn get_value_type(&self) -> ValueType {
        match self {
            ResolvedValue::Value(v) => v.get_value_type(),
            ResolvedValue::Borrowed(_, b) => b.get_value_type(),
            ResolvedValue::Computed(c) => c.get_value_type(),
            ResolvedValue::Slice(_, s) => match s {
                Slice::Array(_) => ValueType::Array,
                Slice::String(_) => ValueType::String,
            },
        }
    }

    fn to_value(&self) -> Value {
        match self {
            ResolvedValue::Value(v) => v.clone(),
            ResolvedValue::Borrowed(_, b) => b.to_value(),
            ResolvedValue::Computed(c) => c.to_value(),
            ResolvedValue::Slice(_, s) => match s {
                Slice::Array(a) => Value::Array(a),
                Slice::String(s) => Value::String(s),
            },
        }
    }
}

impl Display for ResolvedValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_value().fmt(f)
    }
}

#[derive(Debug)]
pub enum Slice<'a> {
    Array(ArraySlice<'a>),
    String(StringSlice),
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

    fn get(&self, index: usize) -> Option<&(dyn AsStaticValue + 'static)> {
        self.inner_value.get(self.range_start_inclusive + index)
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
pub struct StringSlice {
    value: String,
}

impl StringSlice {
    pub fn new<'a>(
        inner_value: ResolvedStringValue<'a>,
        range_start_inclusive: usize,
        range_end_exclusive: usize,
    ) -> StringSlice {
        Self {
            value: SliceScalarExpression::slice_string(
                inner_value.get_value(),
                range_start_inclusive,
                range_end_exclusive,
            ),
        }
    }
}

impl StringValue for StringSlice {
    fn get_value(&self) -> &str {
        &self.value
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
    Slice(Option<BorrowSource>, StringSlice),
}

impl ResolvedStringValue<'_> {
    pub fn get_borrow_source(&self) -> Option<BorrowSource> {
        match self {
            ResolvedStringValue::Borrowed(b, _) => Some(b.clone()),
            ResolvedStringValue::Slice(b, _) => b.clone(),
            _ => None,
        }
    }
}

impl StringValue for ResolvedStringValue<'_> {
    fn get_value(&self) -> &str {
        match self {
            ResolvedStringValue::Value(s) => s.get_value(),
            ResolvedStringValue::Borrowed(_, b) => b.get_value(),
            ResolvedStringValue::Computed(v) => v.get_raw_value(),
            ResolvedStringValue::Slice(_, s) => s.get_value(),
        }
    }
}

impl AsValue for ResolvedStringValue<'_> {
    fn get_value_type(&self) -> ValueType {
        ValueType::String
    }

    fn to_value(&self) -> Value {
        match self {
            ResolvedStringValue::Value(v) => Value::String(*v),
            ResolvedStringValue::Borrowed(_, b) => Value::String(&**b),
            ResolvedStringValue::Computed(c) => Value::String(c),
            ResolvedStringValue::Slice(_, s) => Value::String(s),
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

    fn to_value(&self) -> Value {
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
    Slice(Option<BorrowSource>, Box<ArraySlice<'a>>),
}

impl ResolvedArrayValue<'_> {
    pub fn get_borrow_source(&self) -> Option<BorrowSource> {
        match self {
            ResolvedArrayValue::Borrowed(b, _) => Some(b.clone()),
            ResolvedArrayValue::Slice(b, _) => b.clone(),
            _ => None,
        }
    }

    fn get_array(&self) -> &dyn ArrayValue {
        match self {
            ResolvedArrayValue::Value(v) => *v,
            ResolvedArrayValue::Borrowed(_, b) => &**b,
            ResolvedArrayValue::Computed(c) => c,
            ResolvedArrayValue::Slice(_, s) => &**s,
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

    fn get(&self, index: usize) -> Option<&(dyn AsStaticValue + 'static)> {
        self.get_array().get(index)
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

    fn to_value(&self) -> Value {
        Value::Array(self.get_array())
    }
}

impl Display for ResolvedArrayValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_value().fmt(f)
    }
}
