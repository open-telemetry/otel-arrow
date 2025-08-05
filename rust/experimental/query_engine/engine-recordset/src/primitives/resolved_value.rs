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
}

#[derive(Debug, Clone, PartialEq)]
pub enum BorrowSource {
    Source,
    Variable,
}

impl<'a> ResolvedValue<'a> {
    pub fn copy_if_borrowed_from_target(&mut self, target: &MutableValueExpression) -> bool {
        if let ResolvedValue::Borrowed(s, v) = self {
            let writing_while_holding_borrow = match target {
                MutableValueExpression::Source(_) => {
                    matches!(s, BorrowSource::Source)
                }
                MutableValueExpression::Variable(_) => {
                    matches!(s, BorrowSource::Variable)
                }
            };

            if writing_while_holding_borrow {
                *self = ResolvedValue::Computed(v.to_value().into());
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
            ResolvedValue::Borrowed(_, b) => {
                match Ref::filter_map(b, |v| {
                    if let StaticValue::String(s) = v.to_static_value() {
                        Some(s)
                    } else {
                        None
                    }
                }) {
                    Ok(v) => Ok(ResolvedStringValue::Borrowed(v)),
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
        }
    }
}

impl From<ResolvedValue<'_>> for OwnedValue {
    fn from(val: ResolvedValue<'_>) -> Self {
        match val {
            ResolvedValue::Value(v) => v.into(),
            ResolvedValue::Borrowed(_, b) => b.to_value().into(),
            ResolvedValue::Computed(o) => o,
        }
    }
}

impl AsValue for ResolvedValue<'_> {
    fn get_value_type(&self) -> ValueType {
        match self {
            ResolvedValue::Value(v) => v.get_value_type(),
            ResolvedValue::Borrowed(_, b) => b.get_value_type(),
            ResolvedValue::Computed(c) => c.get_value_type(),
        }
    }

    fn to_value(&self) -> Value {
        match self {
            ResolvedValue::Value(v) => v.clone(),
            ResolvedValue::Borrowed(_, b) => b.to_value(),
            ResolvedValue::Computed(c) => c.to_value(),
        }
    }
}

impl Display for ResolvedValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_value().fmt(f)
    }
}

#[derive(Debug)]
pub enum ResolvedStringValue<'a> {
    /// A value resolved from the expression tree or an attached record
    Value(&'a dyn StringValue),

    /// A value borrowed from the record being modified by the engine
    Borrowed(Ref<'a, dyn StringValue + 'static>),

    /// A value computed by the engine as the result of a dynamic expression
    Computed(StringValueStorage),
}

impl StringValue for ResolvedStringValue<'_> {
    fn get_value(&self) -> &str {
        match self {
            ResolvedStringValue::Value(s) => s.get_value(),
            ResolvedStringValue::Borrowed(b) => b.get_value(),
            ResolvedStringValue::Computed(v) => v.get_raw_value(),
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
            ResolvedStringValue::Borrowed(b) => Value::String(&**b),
            ResolvedStringValue::Computed(c) => Value::String(c),
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
