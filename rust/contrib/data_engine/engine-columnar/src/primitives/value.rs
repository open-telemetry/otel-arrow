// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::hash::{Hash, Hasher};
use std::rc::Rc;

use chrono::{DateTime, FixedOffset, TimeDelta};
use otel_arrow_contrib_data_engine_expressions::*;
use regex::Regex;

use crate::*;

#[derive(Debug, Clone)]
pub enum ValueOrRef<'a> {
    Array(ArrayValueOrRef<'a>),
    Boolean(bool),
    DateTime(DateTime<FixedOffset>),
    Double(f64),
    Integer(i64),
    Map(MapValueOrRef<'a>),
    Null,
    Regex(RegexValueOrRef<'a>),
    String(StringValueOrRef<'a>),
    TimeSpan(TimeDelta),
}

impl<'a> ValueOrRef<'a> {
    pub fn to_string(&self) -> StringValueOrRef<'a> {
        self.into()
    }
}

impl ValueOrRef<'_> {
    pub fn to_int<T>(&self) -> Option<T>
    where
        T: TryFrom<i64>,
    {
        let v = match self {
            ValueOrRef::Null => {
                return None;
            }
            ValueOrRef::Integer(i) => *i,
            v => match v.to_value().convert_to_integer() {
                None => {
                    return None;
                }
                Some(v) => v,
            },
        };

        v.try_into().ok()
    }
}

#[derive(Debug, Clone)]
pub enum RegexValueOrRef<'a> {
    Ref(&'a Regex),
    Owned(Rc<Regex>),
}

impl RegexValueOrRef<'_> {
    pub fn new_owned(value: Regex) -> RegexValueOrRef<'static> {
        RegexValueOrRef::Owned(value.into())
    }
}

impl<'a> RegexValueOrRef<'a> {
    pub fn new_ref(value: &'a Regex) -> RegexValueOrRef<'a> {
        RegexValueOrRef::Ref(value)
    }
}

impl RegexValue for RegexValueOrRef<'_> {
    fn get_value(&self) -> &Regex {
        match self {
            RegexValueOrRef::Ref(r) => r,
            RegexValueOrRef::Owned(r) => r,
        }
    }
}

impl Hash for ValueOrRef<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            ValueOrRef::String(s) => {
                [0].hash(state);
                s.hash(state);
            }
            ValueOrRef::Integer(i) => {
                [1].hash(state);
                i.get_value().hash(state);
            }
            ValueOrRef::Double(d) => {
                [2].hash(state);
                state.write_u64(d.get_value().to_bits());
            }
            ValueOrRef::Boolean(b) => {
                [3].hash(state);
                b.hash(state);
            }
            ValueOrRef::DateTime(d) => {
                [4].hash(state);
                d.hash(state);
            }
            ValueOrRef::TimeSpan(t) => {
                [5].hash(state);
                t.hash(state);
            }
            ValueOrRef::Regex(r) => {
                [6].hash(state);
                r.get_value().as_str().hash(state);
            }
            ValueOrRef::Array(a) => {
                a.hash(state);
            }
            ValueOrRef::Map(m) => {
                m.hash(state);
            }
            ValueOrRef::Null => [9].hash(state),
        }
    }
}

impl PartialEq for ValueOrRef<'_> {
    fn eq(&self, other: &Self) -> bool {
        match self {
            ValueOrRef::String(s) => {
                if let ValueOrRef::String(other) = other {
                    s == other
                } else {
                    false
                }
            }
            ValueOrRef::Integer(i) => match other {
                ValueOrRef::Integer(r) => *i == *r,
                _ => false,
            },
            ValueOrRef::Double(d) => match other {
                ValueOrRef::Double(r) => d.get_value().to_bits() == r.get_value().to_bits(),
                _ => false,
            },
            ValueOrRef::Boolean(b) => match other {
                ValueOrRef::Boolean(r) => *b == *r,
                _ => false,
            },
            ValueOrRef::DateTime(d) => match other {
                ValueOrRef::DateTime(o) => *d == *o,
                _ => false,
            },
            ValueOrRef::TimeSpan(t) => match other {
                ValueOrRef::TimeSpan(o) => *t == *o,
                _ => false,
            },
            ValueOrRef::Regex(r) => {
                if let ValueOrRef::Regex(other) = other {
                    r.get_value().as_str() == other.get_value().as_str()
                } else {
                    false
                }
            }
            ValueOrRef::Array(a) => {
                if let ValueOrRef::Array(other) = other {
                    a.eq(other)
                } else {
                    false
                }
            }
            ValueOrRef::Map(m) => {
                let m = m.as_map_value();

                if let ValueOrRef::Map(other) = other {
                    let other = other.as_map_value();
                    if m.len() == other.len() {
                        return m.get_items(&mut |k, l| match other.get(k) {
                            None => false,
                            Some(r) => {
                                Into::<ValueOrRef>::into(l)
                                    == Into::<ValueOrRef>::into(r.to_value())
                            }
                        });
                    }
                }

                false
            }
            ValueOrRef::Null => matches!(other, ValueOrRef::Null),
        }
    }
}

impl Eq for ValueOrRef<'_> {}

impl AsValue for ValueOrRef<'_> {
    fn get_value_type(&self) -> ValueType {
        match self {
            ValueOrRef::Array(_) => ValueType::Array,
            ValueOrRef::String(_) => ValueType::String,
            ValueOrRef::Integer(_) => ValueType::Integer,
            ValueOrRef::Double(_) => ValueType::Double,
            ValueOrRef::Boolean(_) => ValueType::Boolean,
            ValueOrRef::DateTime(_) => ValueType::DateTime,
            ValueOrRef::TimeSpan(_) => ValueType::TimeSpan,
            ValueOrRef::Regex(_) => ValueType::Regex,
            ValueOrRef::Map(_) => ValueType::Map,
            ValueOrRef::Null => ValueType::Null,
        }
    }

    fn to_value(&self) -> Value<'_> {
        match self {
            ValueOrRef::String(s) => Value::String(s),
            ValueOrRef::Integer(i) => Value::Integer(i),
            ValueOrRef::Double(d) => Value::Double(d),
            ValueOrRef::Boolean(b) => Value::Boolean(b),
            ValueOrRef::DateTime(d) => Value::DateTime(d),
            ValueOrRef::TimeSpan(t) => Value::TimeSpan(t),
            ValueOrRef::Regex(r) => Value::Regex(r),
            ValueOrRef::Array(ArrayValueOrRef::Ref(a)) => Value::Array(*a),
            ValueOrRef::Map(m) => Value::Map(m.as_map_value()),
            ValueOrRef::Array(a) => Value::Array(a.as_array_value()),
            ValueOrRef::Null => Value::Null,
        }
    }
}

impl<'a> From<Value<'a>> for ValueOrRef<'a> {
    fn from(val: Value<'a>) -> Self {
        match val {
            Value::Array(a) => ValueOrRef::Array(ArrayValueOrRef::Ref(a)),
            Value::Boolean(b) => ValueOrRef::Boolean(b.get_value()),
            Value::DateTime(d) => ValueOrRef::DateTime(d.get_value()),
            Value::Double(d) => ValueOrRef::Double(d.get_value()),
            Value::Integer(i) => ValueOrRef::Integer(i.get_value()),
            Value::Map(m) => ValueOrRef::Map(MapValueOrRef::Ref(m)),
            Value::Null => ValueOrRef::Null,
            Value::Regex(r) => ValueOrRef::Regex(RegexValueOrRef::Ref(r.get_value())),
            Value::String(s) => ValueOrRef::String(StringValueOrRef::Ref(s.get_value())),
            Value::TimeSpan(t) => ValueOrRef::TimeSpan(t.get_value()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: Integer conversion is requested from native integers, convertible strings, invalid strings, and Null.
    /// Guarantees: Successful conversions respect the destination range and failed conversions return None.
    #[test]
    fn integer_conversion_handles_types_and_ranges() {
        assert_eq!(ValueOrRef::Integer(255).to_int::<u8>(), Some(255));
        assert_eq!(ValueOrRef::Integer(256).to_int::<u8>(), None);
        assert_eq!(
            ValueOrRef::String(StringValueOrRef::new_ref("-12")).to_int::<i16>(),
            Some(-12)
        );
        assert_eq!(
            ValueOrRef::String(StringValueOrRef::new_ref("not-a-number")).to_int::<i64>(),
            None
        );
        assert_eq!(ValueOrRef::Null.to_int::<i64>(), None);
    }

    /// Scenario: Floating-point values include NaNs with identical bits and positive and negative zero.
    /// Guarantees: Equality follows the exact bit representation used by hashing.
    #[test]
    fn double_equality_uses_exact_bits() {
        let nan_bits = 0x7ff8_0000_0000_0001;
        assert_eq!(
            ValueOrRef::Double(f64::from_bits(nan_bits)),
            ValueOrRef::Double(f64::from_bits(nan_bits))
        );
        assert_ne!(ValueOrRef::Double(0.0), ValueOrRef::Double(-0.0));
    }

    /// Scenario: A regex is alternately borrowed and retained by an owned reference.
    /// Guarantees: Both forms expose equal values and preserve the Regex value type through AsValue.
    #[test]
    fn regex_references_and_owned_values_are_equivalent() {
        let regex = Regex::new("^otel$").unwrap();
        let borrowed = ValueOrRef::Regex(RegexValueOrRef::new_ref(&regex));
        let owned = ValueOrRef::Regex(RegexValueOrRef::new_owned(regex.clone()));

        assert_eq!(borrowed, owned);
        assert_eq!(borrowed.get_value_type(), ValueType::Regex);
        assert_eq!(borrowed.to_value().convert_to_string().as_ref(), "^otel$");
    }
}
