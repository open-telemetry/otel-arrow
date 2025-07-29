use std::{collections::HashMap, fmt::Debug, mem};

use chrono::{DateTime, FixedOffset};
use data_engine_expressions::*;
use regex::Regex;

use crate::*;

pub trait ValueSource<T>: AsValueMut {
    fn from_owned(value: OwnedValue) -> T;

    fn to_owned(self) -> OwnedValue;
}

#[derive(Debug, Clone)]
pub struct ValueStorage<T> {
    value: T,
}

impl<T> ValueStorage<T> {
    pub fn new(value: T) -> ValueStorage<T> {
        Self { value }
    }
}

impl BooleanValue for ValueStorage<bool> {
    fn get_value(&self) -> bool {
        self.value
    }
}

impl AsValue for ValueStorage<bool> {
    fn get_value_type(&self) -> ValueType {
        ValueType::Boolean
    }

    fn to_value(&self) -> Value {
        Value::Boolean(self)
    }
}

impl IntegerValue for ValueStorage<i64> {
    fn get_value(&self) -> i64 {
        self.value
    }
}

impl AsValue for ValueStorage<i64> {
    fn get_value_type(&self) -> ValueType {
        ValueType::Integer
    }

    fn to_value(&self) -> Value {
        Value::Integer(self)
    }
}

impl IntegerValue for ValueStorage<i32> {
    fn get_value(&self) -> i64 {
        self.value as i64
    }
}

impl AsValue for ValueStorage<i32> {
    fn get_value_type(&self) -> ValueType {
        ValueType::Integer
    }

    fn to_value(&self) -> Value {
        Value::Integer(self)
    }
}

impl IntegerValue for ValueStorage<u32> {
    fn get_value(&self) -> i64 {
        self.value as i64
    }
}

impl AsValue for ValueStorage<u32> {
    fn get_value_type(&self) -> ValueType {
        ValueType::Integer
    }

    fn to_value(&self) -> Value {
        Value::Integer(self)
    }
}

impl IntegerValue for ValueStorage<u8> {
    fn get_value(&self) -> i64 {
        self.value as i64
    }
}

impl AsValue for ValueStorage<u8> {
    fn get_value_type(&self) -> ValueType {
        ValueType::Integer
    }

    fn to_value(&self) -> Value {
        Value::Integer(self)
    }
}

impl DateTimeValue for ValueStorage<DateTime<FixedOffset>> {
    fn get_value(&self) -> DateTime<FixedOffset> {
        self.value
    }
}

impl AsValue for ValueStorage<DateTime<FixedOffset>> {
    fn get_value_type(&self) -> ValueType {
        ValueType::DateTime
    }

    fn to_value(&self) -> Value {
        Value::DateTime(self)
    }
}

impl DoubleValue for ValueStorage<f64> {
    fn get_value(&self) -> f64 {
        self.value
    }
}

impl AsValue for ValueStorage<f64> {
    fn get_value_type(&self) -> ValueType {
        ValueType::Double
    }

    fn to_value(&self) -> Value {
        Value::Double(self)
    }
}

impl DoubleValue for ValueStorage<f32> {
    fn get_value(&self) -> f64 {
        self.value as f64
    }
}

impl AsValue for ValueStorage<f32> {
    fn get_value_type(&self) -> ValueType {
        ValueType::Double
    }

    fn to_value(&self) -> Value {
        Value::Double(self)
    }
}

impl RegexValue for ValueStorage<Regex> {
    fn get_value(&self) -> &Regex {
        &self.value
    }
}

impl AsValue for ValueStorage<Regex> {
    fn get_value_type(&self) -> ValueType {
        ValueType::Regex
    }

    fn to_value(&self) -> Value {
        Value::Regex(self)
    }
}

impl StringValue for ValueStorage<String> {
    fn get_value(&self) -> &str {
        &self.value
    }
}

impl AsValue for ValueStorage<String> {
    fn get_value_type(&self) -> ValueType {
        ValueType::String
    }

    fn to_value(&self) -> Value {
        Value::String(self)
    }
}

impl StringValueMut for ValueStorage<String> {
    fn get_value_mut(&mut self) -> &mut String {
        &mut self.value
    }
}

impl AsValueMut for ValueStorage<String> {
    fn to_value_mut(&mut self) -> Option<ValueMut> {
        Some(ValueMut::String(self))
    }
}

#[derive(Debug, Clone)]
pub struct ArrayValueStorage<T: ValueSource<T>> {
    values: Vec<T>,
}

impl<T: ValueSource<T>> ArrayValueStorage<T> {
    pub fn new(values: Vec<T>) -> ArrayValueStorage<T> {
        Self { values }
    }

    pub fn into<TTarget: ValueSource<TTarget>>(mut self) -> ArrayValueStorage<TTarget> {
        ArrayValueStorage::<TTarget>::new(
            self.values
                .drain(..)
                .map(|v| TTarget::from_owned(v.to_owned()))
                .collect(),
        )
    }
}

impl<T: ValueSource<T> + 'static> ArrayValue for ArrayValueStorage<T> {
    fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    fn len(&self) -> usize {
        self.values.len()
    }

    fn get(&self, index: usize) -> Option<&(dyn AsValue + 'static)> {
        self.values.get(index).map(|v| v as &dyn AsValue)
    }

    fn get_items(&self, item_callback: &mut dyn IndexValueCallback) -> bool {
        for (index, value) in self.values.iter().enumerate() {
            if !item_callback.next(index, value.to_value()) {
                return false;
            }
        }

        true
    }
}

impl<T: ValueSource<T> + 'static> AsValue for ArrayValueStorage<T> {
    fn get_value_type(&self) -> ValueType {
        ValueType::Array
    }

    fn to_value(&self) -> Value {
        Value::Array(self)
    }
}

impl<T: ValueSource<T> + 'static> ArrayValueMut for ArrayValueStorage<T> {
    fn get_mut(&mut self, index: usize) -> ValueMutGetResult {
        if let Some(v) = self.values.get_mut(index) {
            if v.to_value_mut().is_some() {
                ValueMutGetResult::Found(v)
            } else {
                ValueMutGetResult::NotSupported(format!(
                    "Cannot mutate '{:?}' value at index '{index}'",
                    v.get_value_type()
                ))
            }
        } else {
            ValueMutGetResult::NotFound
        }
    }

    fn set(&mut self, index: usize, value: ResolvedValue) -> ValueMutWriteResult {
        match self.values.get_mut(index) {
            Some(v) => {
                let old = mem::replace(v, value.convert());
                ValueMutWriteResult::Updated(old.to_owned())
            }
            None => ValueMutWriteResult::NotFound,
        }
    }

    fn push(&mut self, value: ResolvedValue) -> ValueMutWriteResult {
        self.values.push(value.convert());

        ValueMutWriteResult::Created
    }

    fn insert(&mut self, index: usize, value: ResolvedValue) -> ValueMutWriteResult {
        if index > self.values.len() {
            return ValueMutWriteResult::NotFound;
        }

        self.values.insert(index, value.convert());

        ValueMutWriteResult::Created
    }

    fn remove(&mut self, index: usize) -> ValueMutRemoveResult {
        if index >= self.values.len() {
            return ValueMutRemoveResult::NotFound;
        }

        let old = self.values.remove(index);

        ValueMutRemoveResult::Removed(old.to_owned())
    }

    fn retain(&mut self, item_callback: &mut dyn IndexValueMutCallback) {
        let mut index = 0;
        self.values.retain_mut(|v| {
            let r = item_callback.next(index, InnerValue::ValueMut(v));
            index += 1;
            r
        });
    }
}

impl<T: ValueSource<T> + 'static> AsValueMut for ArrayValueStorage<T> {
    fn to_value_mut(&mut self) -> Option<ValueMut> {
        Some(ValueMut::Array(self))
    }
}

#[derive(Debug, Clone)]
pub struct MapValueStorage<T: ValueSource<T>> {
    values: HashMap<Box<str>, T>,
}

impl<T: ValueSource<T>> MapValueStorage<T> {
    pub fn new(values: HashMap<Box<str>, T>) -> MapValueStorage<T> {
        Self { values }
    }

    pub fn into<TTarget: ValueSource<TTarget>>(mut self) -> MapValueStorage<TTarget> {
        MapValueStorage::<TTarget>::new(HashMap::from_iter(
            self.values
                .drain()
                .map(|(k, v)| (k, TTarget::from_owned(v.to_owned()))),
        ))
    }
}

impl<T: ValueSource<T> + 'static> MapValue for MapValueStorage<T> {
    fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    fn len(&self) -> usize {
        self.values.len()
    }

    fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    fn get(&self, key: &str) -> Option<&(dyn AsValue + 'static)> {
        self.values.get(key).map(|v| v as &dyn AsValue)
    }

    fn get_items(&self, item_callback: &mut dyn KeyValueCallback) -> bool {
        for (key, value) in self.values.iter() {
            if !item_callback.next(key, value.to_value()) {
                return false;
            }
        }

        true
    }
}

impl<T: ValueSource<T> + 'static> AsValue for MapValueStorage<T> {
    fn get_value_type(&self) -> ValueType {
        ValueType::Map
    }

    fn to_value(&self) -> Value {
        Value::Map(self)
    }
}

impl<T: ValueSource<T> + 'static> MapValueMut for MapValueStorage<T> {
    fn get_mut(&mut self, key: &str) -> ValueMutGetResult {
        match self.values.get_mut(key) {
            Some(v) => {
                if v.to_value_mut().is_some() {
                    ValueMutGetResult::Found(v)
                } else {
                    ValueMutGetResult::NotSupported(format!(
                        "Cannot mutate '{:?}' value at key '{key}'",
                        v.get_value_type()
                    ))
                }
            }
            None => ValueMutGetResult::NotFound,
        }
    }

    fn set(&mut self, key: &str, value: ResolvedValue) -> ValueMutWriteResult {
        match self.values.insert(key.into(), value.convert()) {
            Some(old) => ValueMutWriteResult::Updated(old.to_owned()),
            None => ValueMutWriteResult::Created,
        }
    }

    fn rename(&mut self, from_key: &str, to_key: &str) -> ValueMutWriteResult {
        match self.values.remove(from_key) {
            Some(v) => match self.values.insert(to_key.into(), v) {
                Some(old) => ValueMutWriteResult::Updated(old.to_owned()),
                None => ValueMutWriteResult::Created,
            },
            None => ValueMutWriteResult::NotFound,
        }
    }

    fn remove(&mut self, key: &str) -> ValueMutRemoveResult {
        match self.values.remove(key) {
            Some(old) => ValueMutRemoveResult::Removed(old.to_owned()),
            None => ValueMutRemoveResult::NotFound,
        }
    }

    fn retain(&mut self, item_callback: &mut dyn KeyValueMutCallback) {
        self.values
            .retain(|k, v| item_callback.next(k, InnerValue::ValueMut(v)));
    }
}

impl<T: ValueSource<T> + 'static> AsValueMut for MapValueStorage<T> {
    fn to_value_mut(&mut self) -> Option<ValueMut> {
        Some(ValueMut::Map(self))
    }
}
