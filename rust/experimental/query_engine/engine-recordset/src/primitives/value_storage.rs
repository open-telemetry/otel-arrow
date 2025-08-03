use std::{collections::HashMap, fmt::Debug, mem};

use chrono::{DateTime, FixedOffset};
use data_engine_expressions::*;
use regex::Regex;

use crate::*;

pub trait ValueSource<T>: AsStaticValue + AsStaticValueMut {
    fn from_owned(value: OwnedValue) -> T;

    fn to_owned(self) -> OwnedValue;
}

#[derive(Debug, Clone)]
pub struct BooleanValueStorage {
    value: bool,
}

impl BooleanValueStorage {
    pub fn new(value: bool) -> BooleanValueStorage {
        Self { value }
    }

    pub fn get_raw_value(&self) -> &bool {
        &self.value
    }

    pub fn get_raw_value_mut(&mut self) -> &mut bool {
        &mut self.value
    }
}

impl BooleanValue for BooleanValueStorage {
    fn get_value(&self) -> bool {
        self.value
    }
}

impl AsStaticValue for BooleanValueStorage {
    fn to_static_value(&self) -> StaticValue {
        StaticValue::Boolean(self)
    }
}

#[derive(Debug, Clone)]
pub struct DateTimeValueStorage {
    value: DateTime<FixedOffset>,
}

impl DateTimeValueStorage {
    pub fn new(value: DateTime<FixedOffset>) -> DateTimeValueStorage {
        Self { value }
    }

    pub fn get_raw_value(&self) -> &DateTime<FixedOffset> {
        &self.value
    }

    pub fn get_raw_value_mut(&mut self) -> &mut DateTime<FixedOffset> {
        &mut self.value
    }
}

impl DateTimeValue for DateTimeValueStorage {
    fn get_value(&self) -> DateTime<FixedOffset> {
        self.value
    }
}

impl AsStaticValue for DateTimeValueStorage {
    fn to_static_value(&self) -> StaticValue {
        StaticValue::DateTime(self)
    }
}

impl AsStaticValueMut for DateTimeValueStorage {
    fn to_static_value_mut(&mut self) -> Option<StaticValueMut> {
        None
    }
}

#[derive(Debug, Clone)]
pub struct DoubleValueStorage<T: Into<f64> + Clone + Debug + 'static> {
    value: T,
}

impl<T: Into<f64> + Clone + Debug + 'static> DoubleValueStorage<T> {
    pub fn new(value: T) -> DoubleValueStorage<T> {
        Self { value }
    }

    pub fn get_raw_value(&self) -> &T {
        &self.value
    }

    pub fn get_raw_value_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T: Into<f64> + Clone + Debug + 'static> DoubleValue for DoubleValueStorage<T> {
    fn get_value(&self) -> f64 {
        self.value.clone().into()
    }
}

impl<T: Into<f64> + Clone + Debug + 'static> AsStaticValue for DoubleValueStorage<T> {
    fn to_static_value(&self) -> StaticValue {
        StaticValue::Double(self)
    }
}

impl<T: Into<f64> + Clone + Debug + 'static> AsStaticValueMut for DoubleValueStorage<T> {
    fn to_static_value_mut(&mut self) -> Option<StaticValueMut> {
        None
    }
}

#[derive(Debug, Clone)]
pub struct IntegerValueStorage<T: Into<i64> + Clone + Debug + 'static> {
    value: T,
}

impl<T: Into<i64> + Clone + Debug + 'static> IntegerValueStorage<T> {
    pub fn new(value: T) -> IntegerValueStorage<T> {
        Self { value }
    }

    pub fn get_raw_value(&self) -> &T {
        &self.value
    }

    pub fn get_raw_value_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T: Into<i64> + Clone + Debug + 'static> IntegerValue for IntegerValueStorage<T> {
    fn get_value(&self) -> i64 {
        self.value.clone().into()
    }
}

impl<T: Into<i64> + Clone + Debug + 'static> AsStaticValue for IntegerValueStorage<T> {
    fn to_static_value(&self) -> StaticValue {
        StaticValue::Integer(self)
    }
}

impl<T: Into<i64> + Clone + Debug + 'static> AsStaticValueMut for IntegerValueStorage<T> {
    fn to_static_value_mut(&mut self) -> Option<StaticValueMut> {
        None
    }
}

#[derive(Debug, Clone)]
pub struct RegexValueStorage {
    value: Regex,
}

impl RegexValueStorage {
    pub fn new(value: Regex) -> RegexValueStorage {
        Self { value }
    }

    pub fn get_raw_value(&self) -> &Regex {
        &self.value
    }

    pub fn get_raw_value_mut(&mut self) -> &mut Regex {
        &mut self.value
    }
}

impl RegexValue for RegexValueStorage {
    fn get_value(&self) -> &Regex {
        &self.value
    }
}

impl AsStaticValue for RegexValueStorage {
    fn to_static_value(&self) -> StaticValue {
        StaticValue::Regex(self)
    }
}

impl AsStaticValueMut for RegexValueStorage {
    fn to_static_value_mut(&mut self) -> Option<StaticValueMut> {
        None
    }
}

#[derive(Debug, Clone)]
pub struct StringValueStorage {
    value: String,
}

impl StringValueStorage {
    pub fn new(value: String) -> StringValueStorage {
        Self { value }
    }

    pub fn get_raw_value(&self) -> &String {
        &self.value
    }

    pub fn get_raw_value_mut(&mut self) -> &mut String {
        &mut self.value
    }
}

impl StringValue for StringValueStorage {
    fn get_value(&self) -> &str {
        &self.value
    }
}

impl StringValueMut for StringValueStorage {
    fn get_value_mut(&mut self) -> &mut String {
        &mut self.value
    }
}

impl AsStaticValue for StringValueStorage {
    fn to_static_value(&self) -> StaticValue {
        StaticValue::String(self)
    }
}

impl AsStaticValueMut for StringValueStorage {
    fn to_static_value_mut(&mut self) -> Option<StaticValueMut> {
        Some(StaticValueMut::String(self))
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

    pub fn get_values(&self) -> &Vec<T> {
        &self.values
    }

    pub fn get_values_mut(&mut self) -> &mut Vec<T> {
        &mut self.values
    }
}

impl<T: ValueSource<T> + 'static> ArrayValue for ArrayValueStorage<T> {
    fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    fn len(&self) -> usize {
        self.values.len()
    }

    fn get(&self, index: usize) -> Option<&(dyn AsStaticValue + 'static)> {
        self.values.get(index).map(|v| v as &dyn AsStaticValue)
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

impl<T: ValueSource<T> + 'static> AsStaticValue for ArrayValueStorage<T> {
    fn to_static_value(&self) -> StaticValue {
        StaticValue::Array(self)
    }
}

impl<T: ValueSource<T> + 'static> AsStaticValueMut for ArrayValueStorage<T> {
    fn to_static_value_mut(&mut self) -> Option<StaticValueMut> {
        Some(StaticValueMut::Array(self))
    }
}

impl<T: ValueSource<T> + 'static> ArrayValueMut for ArrayValueStorage<T> {
    fn get_mut(&mut self, index: usize) -> ValueMutGetResult {
        if let Some(v) = self.values.get_mut(index) {
            ValueMutGetResult::Found(v)
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
            let r = item_callback.next(index, v);
            index += 1;
            r
        });
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

    pub fn get_values(&self) -> &HashMap<Box<str>, T> {
        &self.values
    }

    pub fn get_values_mut(&mut self) -> &mut HashMap<Box<str>, T> {
        &mut self.values
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

    fn get(&self, key: &str) -> Option<&(dyn AsStaticValue + 'static)> {
        self.values.get(key).map(|v| v as &dyn AsStaticValue)
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

impl<T: ValueSource<T> + 'static> AsStaticValue for MapValueStorage<T> {
    fn to_static_value(&self) -> StaticValue {
        StaticValue::Map(self)
    }
}

impl<T: ValueSource<T> + 'static> AsStaticValueMut for MapValueStorage<T> {
    fn to_static_value_mut(&mut self) -> Option<StaticValueMut> {
        Some(StaticValueMut::Map(self))
    }
}

impl<T: ValueSource<T> + 'static> MapValueMut for MapValueStorage<T> {
    fn get_mut(&mut self, key: &str) -> ValueMutGetResult {
        match self.values.get_mut(key) {
            Some(v) => ValueMutGetResult::Found(v),
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
        self.values.retain(|k, v| item_callback.next(k, v));
    }
}
