// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::{cell::Ref, fmt::Display};

use data_engine_expressions::*;
use regex::Regex;

use crate::{execution_context::ExecutionContext, value_expressions::get_borrow_source, *};

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

    /// A sequence of arrays
    Sequence(Sequence<'a>),
}

impl<'a> ResolvedValue<'a> {
    pub(crate) fn copy_if_borrowed_from_target<'b, T: Record>(
        &mut self,
        execution_context: &ExecutionContext<'b, '_, T>,
        target: &'b MutableValueExpression,
    ) {
        let v = match self {
            ResolvedValue::Borrowed(b, v) => Some((b, v.to_value())),
            ResolvedValue::Slice(s) => {
                return s.copy_if_borrowed_from_target(execution_context, target);
            }
            ResolvedValue::List(l) => {
                return l.copy_if_borrowed_from_target(execution_context, target);
            }
            ResolvedValue::Sequence(s) => {
                return s.copy_if_borrowed_from_target(execution_context, target);
            }
            _ => None,
        };

        if let Some((s, v)) = v {
            let writing_while_holding_borrow = match get_borrow_source(execution_context, target) {
                Some(BorrowSource::Source) => {
                    matches!(s, BorrowSource::Source)
                }
                Some(BorrowSource::Variable) => {
                    matches!(s, BorrowSource::Variable)
                }
                None => false,
            };

            if writing_while_holding_borrow {
                execution_context.add_diagnostic_if_enabled(
                    RecordSetEngineDiagnosticLevel::Verbose,
                    target,
                    || format!("Copied the resolved value '{v}' into temporary storage because the value came from the mutable target"));
                *self = ResolvedValue::Computed(v.into());
            }
        }
    }

    pub(crate) fn try_resolve_string(self) -> Result<ResolvedStringValue<'a>, Self> {
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
            ResolvedValue::Sequence(_) => panic!(),
        }
    }

    pub(crate) fn try_resolve_regex(self) -> Result<ResolvedRegexValue<'a>, Self> {
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
            ResolvedValue::Sequence(_) => panic!(),
        }
    }

    pub(crate) fn try_resolve_array(self) -> Result<ResolvedArrayValue<'a>, Self> {
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
                match Ref::filter_map(Ref::clone(&b), |v| {
                    if let StaticValue::Array(s) = v.to_static_value() {
                        Some(s)
                    } else {
                        None
                    }
                }) {
                    Ok(v) => Ok(ResolvedArrayValue::Borrowed(BorrowedArrayValue {
                        source: s,
                        orig: b,
                        value: v,
                    })),
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
            ResolvedValue::Sequence(s) => Ok(ResolvedArrayValue::Sequence(s)),
        }
    }

    pub(crate) fn try_resolve_map(self) -> Result<ResolvedMapValue<'a>, Self> {
        if self.get_value_type() != ValueType::Map {
            return Err(self);
        }

        match self {
            ResolvedValue::Value(v) => {
                if let Value::Map(s) = v {
                    Ok(ResolvedMapValue::Value(s))
                } else {
                    panic!()
                }
            }
            ResolvedValue::Borrowed(s, b) => {
                match Ref::filter_map(Ref::clone(&b), |v| {
                    if let StaticValue::Map(s) = v.to_static_value() {
                        Some(s)
                    } else {
                        None
                    }
                }) {
                    Ok(v) => Ok(ResolvedMapValue::Borrowed(BorrowedMapValue {
                        source: s,
                        _orig: b,
                        value: v,
                    })),
                    Err(_) => panic!(),
                }
            }
            ResolvedValue::Computed(o) => {
                if let OwnedValue::Map(s) = o {
                    Ok(ResolvedMapValue::Computed(s))
                } else {
                    panic!()
                }
            }
            ResolvedValue::Slice(_) => panic!(),
            ResolvedValue::List(_) => panic!(),
            ResolvedValue::Sequence(_) => panic!(),
        }
    }

    pub(crate) fn select<'b, TRecord: Record>(
        self,
        execution_context: &ExecutionContext<'b, '_, TRecord>,
        expression: &'b dyn Expression,
        selector: Value,
    ) -> Result<Option<ResolvedValue<'a>>, ExpressionError> {
        match selector {
            Value::Integer(index) => {
                match self.try_resolve_array() {
                    Ok(array_value) => {
                        let mut index = index.get_value();
                        let length = array_value.len() as i64;
                        if index < 0 {
                            index += length;
                        }
                        if index < 0 || index >= length {
                            execution_context.add_diagnostic_if_enabled(
                                RecordSetEngineDiagnosticLevel::Warn,
                                expression,
                                || format!("Array index '{index}' specified in accessor expression is invalid"),
                            );
                            return Ok(None);
                        }

                        let mut item = None;

                        array_value.take((index as usize).into(), |_, v| Ok(v), &mut |v| {
                            item = Some(v)
                        })?;

                        if let Some(v) = item {
                            execution_context.add_diagnostic_if_enabled(
                                RecordSetEngineDiagnosticLevel::Verbose,
                                expression,
                                || format!("Resolved {} value for index '{index}' specified in accessor expression", ResolvedValue::Value(v.to_value())),
                            );
                            Ok(Some(v))
                        } else {
                            unreachable!() // Closure not executed
                        }
                    }
                    Err(orig) => {
                        execution_context.add_diagnostic_if_enabled(
                            RecordSetEngineDiagnosticLevel::Warn,
                            expression,
                            || format!("Could not search for array index '{}' specified in select expression because current node is a '{}' value", index.get_value(), orig.get_value_type()),
                        );
                        Ok(None)
                    }
                }
            }
            Value::String(key) => {
                match self.try_resolve_map() {
                    Ok(map_value) => {
                        let key = key.get_value();

                        if !map_value.contains_key(key) {
                            execution_context.add_diagnostic_if_enabled(
                                RecordSetEngineDiagnosticLevel::Warn,
                                expression,
                                || format!("Map key '{key}' specified in accessor expression could not be found"),
                            );
                            return Ok(None);
                        }

                        let mut item = None;

                        map_value.take(&[key], |_, v| Ok(v), &mut |v| item = Some(v))?;

                        if let Some(v) = item {
                            execution_context.add_diagnostic_if_enabled(
                                RecordSetEngineDiagnosticLevel::Verbose,
                                expression,
                                || format!("Resolved {} value for key '{key}' specified in accessor expression", ResolvedValue::Value(v.to_value())),
                            );
                            Ok(Some(v))
                        } else {
                            unreachable!() // Closure not executed
                        }
                    }
                    Err(orig) => {
                        execution_context.add_diagnostic_if_enabled(
                            RecordSetEngineDiagnosticLevel::Warn,
                            expression,
                            || format!("Could not search for map key '{}' specified in select expression because current node is a '{}' value", key.get_value(), orig.get_value_type()),
                        );
                        Ok(None)
                    }
                }
            }
            v => {
                execution_context.add_diagnostic_if_enabled(
                    RecordSetEngineDiagnosticLevel::Warn,
                    expression,
                    || format!("Unexpected scalar expression with '{}' value type encountered in select expression", v.get_value_type()),
                );
                Ok(None)
            }
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
            ResolvedValue::Sequence(s) => Value::Array(&s).into(),
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
            ResolvedValue::Sequence(_) => ValueType::Array,
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
            ResolvedValue::Sequence(s) => Value::Array(s),
        }
    }
}

impl Display for ResolvedValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[")?;
        match self.to_value() {
            Value::Null => f.write_str("Null"),
            Value::Array(a) => {
                write!(f, "Array(Count={})", a.len())
            }
            Value::Map(m) => {
                write!(f, "Map(Count={})", m.len())
            }
            Value::String(s) => {
                f.write_str("String(")?;
                let v = s.get_value();
                if v.len() <= 32 {
                    f.write_str(serde_json::to_string(&v).unwrap().as_str())?;
                } else {
                    write!(
                        f,
                        "{}",
                        serde_json::to_string(&format!("{}...", &v[..32]))
                            .unwrap()
                            .as_str()
                    )?;
                }
                f.write_str(")")
            }
            v => {
                write!(f, "{}(", v.get_value_type())?;
                v.fmt(f)?;
                f.write_str(")")
            }
        }?;
        f.write_str("]")
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

    pub fn copy_if_borrowed_from_target<'b, T: Record>(
        &mut self,
        execution_context: &ExecutionContext<'b, '_, T>,
        target: &'b MutableValueExpression,
    ) {
        for value in &mut self.values {
            value.copy_if_borrowed_from_target(execution_context, target);
        }
    }
}

impl ArrayValue for List<'_> {
    fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    fn len(&self) -> usize {
        self.values.len()
    }

    fn get(&self, index: usize) -> Option<&dyn AsValue> {
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
    pub fn copy_if_borrowed_from_target<'b, T: Record>(
        &mut self,
        execution_context: &ExecutionContext<'b, '_, T>,
        target: &'b MutableValueExpression,
    ) {
        match self {
            Slice::Array(a) => a
                .inner_value
                .copy_if_borrowed_from_target(execution_context, target),
            Slice::String(s) => s
                .inner_value
                .copy_if_borrowed_from_target(execution_context, target),
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
    pub(crate) fn new(
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

    fn get(&self, index: usize) -> Option<&dyn AsValue> {
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
    pub(crate) fn from_char_range(
        inner_value: ResolvedStringValue<'a>,
        range_start_inclusive: usize,
        range_end_exclusive: usize,
    ) -> StringSlice<'a> {
        // Note: Slice of a str returns raw utf8 bytes. Chars can take 1 to 4
        // bytes. In order to correctly slice the str as chars we have to find
        // the correct byte indices to do the slicing
        let count = range_end_exclusive - range_start_inclusive;
        let mut chars = inner_value
            .get_value()
            .char_indices()
            .skip(range_start_inclusive)
            .take(count);

        if let Some(first) = chars.next() {
            if let Some(last) = chars.last() {
                let mut buf = [0; 4];
                let encoded = last.1.encode_utf8(&mut buf);

                Self {
                    inner_value,
                    range_start_inclusive: first.0,
                    range_end_exclusive: last.0 + encoded.len(),
                }
            } else {
                let mut buf = [0; 4];
                let encoded = first.1.encode_utf8(&mut buf);

                Self {
                    inner_value,
                    range_start_inclusive: first.0,
                    range_end_exclusive: first.0 + encoded.len(),
                }
            }
        } else {
            Self {
                inner_value,
                range_start_inclusive: 0,
                range_end_exclusive: 0,
            }
        }
    }

    pub(crate) fn from_byte_range(
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
        let value = self.inner_value.get_value();

        &value[self.range_start_inclusive..self.range_end_exclusive]
    }
}

#[derive(Debug)]
pub struct Sequence<'a> {
    values: Vec<ResolvedArrayValue<'a>>,
}

impl<'a> Sequence<'a> {
    pub(crate) fn new(values: Vec<ResolvedArrayValue<'a>>) -> Sequence<'a> {
        Self { values }
    }

    pub fn copy_if_borrowed_from_target<'b, T: Record>(
        &mut self,
        execution_context: &ExecutionContext<'b, '_, T>,
        target: &'b MutableValueExpression,
    ) {
        for value in &mut self.values {
            value.copy_if_borrowed_from_target(execution_context, target);
        }
    }
}

impl ArrayValue for Sequence<'_> {
    fn is_empty(&self) -> bool {
        for v in &self.values {
            if !v.is_empty() {
                return false;
            }
        }

        true
    }

    fn len(&self) -> usize {
        let mut len = 0;
        for v in &self.values {
            len += v.len();
        }

        len
    }

    fn get(&self, mut index: usize) -> Option<&dyn AsValue> {
        for v in &self.values {
            let end = v.len();
            if index < end {
                return v.get(index);
            }
            index -= end;
        }

        None
    }

    fn get_static(&self, _: usize) -> Result<Option<&(dyn AsStaticValue + 'static)>, String> {
        Err("Sequence does not support static access".to_string())
    }

    fn get_item_range(
        &self,
        range: ArrayRange,
        item_callback: &mut dyn IndexValueCallback,
    ) -> bool {
        let len = self.len();

        let mut start = range.get_start_range_inclusize().unwrap_or(0);
        let mut end = range.get_end_range_exclusive().unwrap_or(len);
        let mut index = 0;

        if end > len {
            panic!(
                "range end index {} out of range for slice of length {}",
                end, len
            )
        }

        for v in &self.values {
            let len = usize::min(end, v.len());
            if len == 0 {
                continue;
            }
            if start < len {
                let range = (start..len).into();
                if !v.get_item_range(
                    range,
                    &mut IndexValueClosureCallback::new(|_, v| {
                        let r = item_callback.next(index, v);
                        index += 1;
                        r
                    }),
                ) {
                    return false;
                }
                start = 0;
            } else {
                start -= len;
            }
            end -= len;
        }

        true
    }
}

#[derive(Debug)]
pub(crate) enum ResolvedStringValue<'a> {
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
    pub fn copy_if_borrowed_from_target<'b, T: Record>(
        &mut self,
        execution_context: &ExecutionContext<'b, '_, T>,
        target: &'b MutableValueExpression,
    ) {
        let v = match self {
            ResolvedStringValue::Borrowed(b, v) => Some((b, v)),
            ResolvedStringValue::Slice(s) => {
                return s
                    .inner_value
                    .copy_if_borrowed_from_target(execution_context, target);
            }
            _ => None,
        };

        if let Some((s, v)) = v {
            let writing_while_holding_borrow = match get_borrow_source(execution_context, target) {
                Some(BorrowSource::Source) => {
                    matches!(s, BorrowSource::Source)
                }
                Some(BorrowSource::Variable) => {
                    matches!(s, BorrowSource::Variable)
                }
                None => false,
            };

            if writing_while_holding_borrow {
                execution_context.add_diagnostic_if_enabled(
                    RecordSetEngineDiagnosticLevel::Verbose,
                    target,
                    || format!("Copied the resolved string value '{}' into temporary storage because the value came from the mutable target", v.get_value()));
                *self =
                    ResolvedStringValue::Computed(StringValueStorage::new(v.get_value().into()));
            }
        }
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
pub(crate) enum ResolvedRegexValue<'a> {
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
pub(crate) enum ResolvedArrayValue<'a> {
    /// A value resolved from the expression tree or an attached record
    Value(&'a dyn ArrayValue),

    /// A value borrowed from the record being modified by the engine
    Borrowed(BorrowedArrayValue<'a>),

    /// A value computed by the engine as the result of a dynamic expression
    Computed(ArrayValueStorage<OwnedValue>),

    /// A slice of items from an array
    Slice(Box<ArraySlice<'a>>),

    /// A list of resolved values
    List(List<'a>),

    /// A sequence of arrays
    Sequence(Sequence<'a>),
}

#[derive(Debug)]
pub(crate) struct BorrowedArrayValue<'a> {
    source: BorrowSource,
    orig: Ref<'a, dyn AsStaticValue + 'static>,
    value: Ref<'a, dyn ArrayValue + 'static>,
}

impl<'a> ResolvedArrayValue<'a> {
    pub fn copy_if_borrowed_from_target<'b, T: Record>(
        &mut self,
        execution_context: &ExecutionContext<'b, '_, T>,
        target: &'b MutableValueExpression,
    ) {
        let v = match self {
            ResolvedArrayValue::Borrowed(b) => Some((&b.source, &b.value)),
            ResolvedArrayValue::Slice(s) => {
                return s
                    .inner_value
                    .copy_if_borrowed_from_target(execution_context, target);
            }
            ResolvedArrayValue::List(l) => {
                return l.copy_if_borrowed_from_target(execution_context, target);
            }
            ResolvedArrayValue::Sequence(s) => {
                return s.copy_if_borrowed_from_target(execution_context, target);
            }
            _ => None,
        };

        if let Some((s, v)) = v {
            let writing_while_holding_borrow = match get_borrow_source(execution_context, target) {
                Some(BorrowSource::Source) => {
                    matches!(s, BorrowSource::Source)
                }
                Some(BorrowSource::Variable) => {
                    matches!(s, BorrowSource::Variable)
                }
                None => false,
            };

            if writing_while_holding_borrow {
                execution_context.add_diagnostic_if_enabled(
                    RecordSetEngineDiagnosticLevel::Verbose,
                    target,
                    || "Copied the resolved array value into temporary storage because the value came from the mutable target".into());
                *self = ResolvedArrayValue::Computed((&**v).into());
            }
        }
    }

    pub fn take<FConvert, FTake, R>(
        self,
        range: ArrayRange,
        convert: FConvert,
        mut take: FTake,
    ) -> Result<(), ExpressionError>
    where
        FConvert: Fn(usize, ResolvedValue<'a>) -> Result<R, ExpressionError>,
        FTake: FnMut(R),
    {
        let start = range.get_start_range_inclusize().unwrap_or(0);

        match self {
            ResolvedArrayValue::Value(a) => {
                let end = range.get_end_range_exclusive().unwrap_or(a.len());
                for i in start..end {
                    let v = (convert)(
                        i,
                        ResolvedValue::Value(
                            a.get(i).expect("Array index was not found").to_value(),
                        ),
                    )?;
                    (take)(v);
                }
                Ok(())
            }
            ResolvedArrayValue::Borrowed(b) => {
                let a = &b.value;
                let end = range.get_end_range_exclusive().unwrap_or(a.len());
                for i in start..end {
                    match Ref::filter_map(Ref::clone(a), |a| {
                        a.get_static(i)
                            .expect("Borrowed array does not implement get_static")
                    }) {
                        Ok(v) => {
                            let v = (convert)(i, ResolvedValue::Borrowed(b.source.clone(), v))?;
                            (take)(v);
                        }
                        Err(_) => panic!("Array index was not found"),
                    }
                }
                Ok(())
            }
            ResolvedArrayValue::Computed(mut a) => {
                let end = range.get_end_range_exclusive().unwrap_or(a.len());
                for (i, v) in a.drain((start..end).into()).enumerate() {
                    let v = (convert)(i, ResolvedValue::Computed(v))?;
                    (take)(v);
                }
                Ok(())
            }
            ResolvedArrayValue::Slice(s) => {
                let start = s.range_start_inclusive + start;
                let end =
                    s.range_start_inclusive + range.get_end_range_exclusive().unwrap_or(s.len());
                s.inner_value.take((start..end).into(), convert, take)?;
                Ok(())
            }
            ResolvedArrayValue::List(mut l) => {
                let end = range.get_end_range_exclusive().unwrap_or(l.len());
                for (i, v) in l.values.drain(start..end).enumerate() {
                    (take)((convert)(i, v)?);
                }
                Ok(())
            }
            ResolvedArrayValue::Sequence(mut s) => {
                let end = range.get_end_range_exclusive().unwrap_or(s.len());
                for (i, v) in s.values.drain(start..end).enumerate() {
                    let r = match v {
                        ResolvedArrayValue::Value(a) => ResolvedValue::Value(Value::Array(a)),
                        ResolvedArrayValue::Borrowed(b) => {
                            ResolvedValue::Borrowed(b.source, b.orig)
                        }
                        ResolvedArrayValue::Computed(a) => {
                            ResolvedValue::Computed(OwnedValue::Array(a))
                        }
                        ResolvedArrayValue::Slice(a) => ResolvedValue::Slice(Slice::Array(*a)),
                        ResolvedArrayValue::List(l) => ResolvedValue::List(l),
                        ResolvedArrayValue::Sequence(s) => ResolvedValue::Sequence(s),
                    };

                    (take)((convert)(i, r)?);
                }
                Ok(())
            }
        }
    }

    pub fn into_vec<F, R>(self, range: ArrayRange, convert: F) -> Result<Vec<R>, ExpressionError>
    where
        F: Fn(usize, ResolvedValue<'a>) -> Result<R, ExpressionError>,
    {
        let mut values = Vec::with_capacity(self.len());
        self.take(range, convert, &mut |i| values.push(i))?;
        Ok(values)
    }

    fn get_array(&self) -> &dyn ArrayValue {
        match self {
            ResolvedArrayValue::Value(v) => *v,
            ResolvedArrayValue::Borrowed(b) => &*b.value,
            ResolvedArrayValue::Computed(c) => c,
            ResolvedArrayValue::Slice(s) => &**s,
            ResolvedArrayValue::List(l) => l,
            ResolvedArrayValue::Sequence(s) => s,
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

    fn get(&self, index: usize) -> Option<&dyn AsValue> {
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

#[derive(Debug)]
pub(crate) enum ResolvedMapValue<'a> {
    /// A value resolved from the expression tree or an attached record
    Value(&'a dyn MapValue),

    /// A value borrowed from the record being modified by the engine
    Borrowed(BorrowedMapValue<'a>),

    /// A value computed by the engine as the result of a dynamic expression
    Computed(MapValueStorage<OwnedValue>),
}

#[derive(Debug)]
pub(crate) struct BorrowedMapValue<'a> {
    source: BorrowSource,
    // Note: orig is not currently used but left in the code to support future
    // needing to return a resolved map as a static value. See usage on
    // BorrowedArrayValue for example
    _orig: Ref<'a, dyn AsStaticValue + 'static>,
    value: Ref<'a, dyn MapValue + 'static>,
}

impl<'a> ResolvedMapValue<'a> {
    /*pub fn copy_if_borrowed_from_target<'b, T: Record>(
        &mut self,
        execution_context: &ExecutionContext<'b, '_, T>,
        target: &'b MutableValueExpression,
    ) {
        let v = match self {
            ResolvedMapValue::Borrowed(b) => Some((&b.source, &b.value)),
            _ => None,
        };

        if let Some((s, v)) = v {
            let writing_while_holding_borrow = match get_borrow_source(execution_context, target) {
                Some(BorrowSource::Source) => {
                    matches!(s, BorrowSource::Source)
                }
                Some(BorrowSource::Variable) => {
                    matches!(s, BorrowSource::Variable)
                }
                None => false,
            };

            if writing_while_holding_borrow {
                execution_context.add_diagnostic_if_enabled(
                    RecordSetEngineDiagnosticLevel::Verbose,
                    target,
                    || "Copied the resolved map value into temporary storage because the value came from the mutable target".into());
                *self = ResolvedMapValue::Computed((&**v).into());
            }
        }
    }*/

    pub fn take<FConvert, FTake, R>(
        self,
        keys: &[&str],
        convert: FConvert,
        mut take: FTake,
    ) -> Result<(), ExpressionError>
    where
        FConvert: Fn(&str, ResolvedValue<'a>) -> Result<R, ExpressionError>,
        FTake: FnMut(R),
    {
        match self {
            ResolvedMapValue::Value(m) => {
                for key in keys {
                    let v = (convert)(
                        key,
                        ResolvedValue::Value(m.get(key).expect("Map key was not found").to_value()),
                    )?;
                    (take)(v);
                }
                Ok(())
            }
            ResolvedMapValue::Borrowed(b) => {
                let m = &b.value;
                for key in keys {
                    match Ref::filter_map(Ref::clone(m), |m| {
                        m.get_static(key)
                            .expect("Borrowed map does not implement get_static")
                    }) {
                        Ok(v) => {
                            let v = (convert)(key, ResolvedValue::Borrowed(b.source.clone(), v))?;
                            (take)(v);
                        }
                        Err(_) => panic!("Map key was not found"),
                    }
                }
                Ok(())
            }
            ResolvedMapValue::Computed(mut m) => {
                let values = m.get_values_mut();

                for key in keys {
                    match values.remove(*key) {
                        Some(v) => {
                            let v = (convert)(key, ResolvedValue::Computed(v))?;
                            (take)(v);
                        }
                        None => panic!("Map key was not found"),
                    }
                }
                Ok(())
            }
        }
    }

    fn get_map(&self) -> &dyn MapValue {
        match self {
            ResolvedMapValue::Value(v) => *v,
            ResolvedMapValue::Borrowed(b) => &*b.value,
            ResolvedMapValue::Computed(c) => c,
        }
    }
}

impl MapValue for ResolvedMapValue<'_> {
    fn is_empty(&self) -> bool {
        self.get_map().is_empty()
    }

    fn len(&self) -> usize {
        self.get_map().len()
    }

    fn contains_key(&self, key: &str) -> bool {
        self.get_map().contains_key(key)
    }

    fn get(&self, key: &str) -> Option<&dyn AsValue> {
        self.get_map().get(key)
    }

    fn get_static(&self, key: &str) -> Result<Option<&(dyn AsStaticValue + 'static)>, String> {
        self.get_map().get_static(key)
    }

    fn get_items(&self, item_callback: &mut dyn KeyValueCallback) -> bool {
        self.get_map().get_items(item_callback)
    }
}

impl AsValue for ResolvedMapValue<'_> {
    fn get_value_type(&self) -> ValueType {
        ValueType::Map
    }

    fn to_value(&self) -> Value<'_> {
        Value::Map(self.get_map())
    }
}

impl Display for ResolvedMapValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_value().fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequence_get_item_range() {
        fn run_test(v: &Sequence, range: ArrayRange, expected: &[Value]) {
            let mut items = 0;

            assert!(v.get_item_range(
                range,
                &mut IndexValueClosureCallback::new(|i, v| {
                    assert_eq!(expected[i], v);
                    items += 1;
                    true
                }),
            ));

            assert_eq!(expected.len(), items);
        }

        let sequence = Sequence::new(vec![
            ResolvedArrayValue::Computed(ArrayValueStorage::new(vec![
                OwnedValue::Integer(IntegerValueStorage::new(0)),
                OwnedValue::Integer(IntegerValueStorage::new(1)),
                OwnedValue::Integer(IntegerValueStorage::new(2)),
            ])),
            ResolvedArrayValue::Computed(ArrayValueStorage::new(vec![
                OwnedValue::Integer(IntegerValueStorage::new(3)),
                OwnedValue::Integer(IntegerValueStorage::new(4)),
                OwnedValue::Integer(IntegerValueStorage::new(5)),
            ])),
        ]);

        run_test(&sequence, (0..0).into(), &[]);
        run_test(
            &sequence,
            (0..1).into(),
            &[OwnedValue::Integer(IntegerValueStorage::new(0)).to_value()],
        );
        run_test(
            &sequence,
            (0..=2).into(),
            &[
                OwnedValue::Integer(IntegerValueStorage::new(0)).to_value(),
                OwnedValue::Integer(IntegerValueStorage::new(1)).to_value(),
                OwnedValue::Integer(IntegerValueStorage::new(2)).to_value(),
            ],
        );
        run_test(
            &sequence,
            (1..=1).into(),
            &[OwnedValue::Integer(IntegerValueStorage::new(1)).to_value()],
        );
        run_test(
            &sequence,
            (2..=3).into(),
            &[
                OwnedValue::Integer(IntegerValueStorage::new(2)).to_value(),
                OwnedValue::Integer(IntegerValueStorage::new(3)).to_value(),
            ],
        );
        run_test(
            &sequence,
            (2..).into(),
            &[
                OwnedValue::Integer(IntegerValueStorage::new(2)).to_value(),
                OwnedValue::Integer(IntegerValueStorage::new(3)).to_value(),
                OwnedValue::Integer(IntegerValueStorage::new(4)).to_value(),
                OwnedValue::Integer(IntegerValueStorage::new(5)).to_value(),
            ],
        );
        run_test(
            &sequence,
            (3..=5).into(),
            &[
                OwnedValue::Integer(IntegerValueStorage::new(3)).to_value(),
                OwnedValue::Integer(IntegerValueStorage::new(4)).to_value(),
                OwnedValue::Integer(IntegerValueStorage::new(5)).to_value(),
            ],
        );
        run_test(
            &sequence,
            (..).into(),
            &[
                OwnedValue::Integer(IntegerValueStorage::new(0)).to_value(),
                OwnedValue::Integer(IntegerValueStorage::new(1)).to_value(),
                OwnedValue::Integer(IntegerValueStorage::new(2)).to_value(),
                OwnedValue::Integer(IntegerValueStorage::new(3)).to_value(),
                OwnedValue::Integer(IntegerValueStorage::new(4)).to_value(),
                OwnedValue::Integer(IntegerValueStorage::new(5)).to_value(),
            ],
        );
        run_test(&sequence, (10..).into(), &[]);
    }

    #[test]
    #[should_panic]
    fn test_sequence_get_item_range_panic() {
        fn run_test(v: &Sequence, range: ArrayRange, expected: &[Value]) {
            v.get_item_range(
                range,
                &mut IndexValueClosureCallback::new(|i, v| {
                    assert_eq!(expected[i], v);
                    true
                }),
            );
        }

        let sequence = Sequence::new(vec![
            ResolvedArrayValue::Computed(ArrayValueStorage::new(vec![
                OwnedValue::Integer(IntegerValueStorage::new(0)),
                OwnedValue::Integer(IntegerValueStorage::new(1)),
                OwnedValue::Integer(IntegerValueStorage::new(2)),
            ])),
            ResolvedArrayValue::Computed(ArrayValueStorage::new(vec![
                OwnedValue::Integer(IntegerValueStorage::new(3)),
                OwnedValue::Integer(IntegerValueStorage::new(4)),
                OwnedValue::Integer(IntegerValueStorage::new(5)),
            ])),
        ]);

        run_test(&sequence, (..10).into(), &[]);
    }
}
