// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::{collections::HashMap, fmt::Debug, mem};

use chrono::{DateTime, FixedOffset, TimeDelta};
use data_engine_expressions::*;
use regex::Regex;

use crate::*;

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
    fn to_static_value(&self) -> StaticValue<'_> {
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
    fn to_static_value(&self) -> StaticValue<'_> {
        StaticValue::DateTime(self)
    }
}

impl AsStaticValueMut for DateTimeValueStorage {
    fn to_static_value_mut(&mut self) -> Option<StaticValueMut<'_>> {
        None
    }
}

pub trait DoubleValueSource<T>: Into<f64> + Clone + Debug + 'static {}

impl<T: Into<f64> + Clone + Debug + 'static> DoubleValueSource<T> for T {}

#[derive(Debug, Clone)]
pub struct DoubleValueStorage<T: DoubleValueSource<T>> {
    value: T,
}

impl<T: DoubleValueSource<T>> DoubleValueStorage<T> {
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

impl<T: DoubleValueSource<T>> DoubleValue for DoubleValueStorage<T> {
    fn get_value(&self) -> f64 {
        self.value.clone().into()
    }
}

impl<T: DoubleValueSource<T>> AsStaticValue for DoubleValueStorage<T> {
    fn to_static_value(&self) -> StaticValue<'_> {
        StaticValue::Double(self)
    }
}

impl<T: DoubleValueSource<T>> AsStaticValueMut for DoubleValueStorage<T> {
    fn to_static_value_mut(&mut self) -> Option<StaticValueMut<'_>> {
        None
    }
}

pub trait IntegerValueSource<T>: Into<i64> + Clone + Debug + 'static {}

impl<T: Into<i64> + Clone + Debug + 'static> IntegerValueSource<T> for T {}

#[derive(Debug, Clone)]
pub struct IntegerValueStorage<T: IntegerValueSource<T>> {
    value: T,
}

impl<T: IntegerValueSource<T>> IntegerValueStorage<T> {
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

impl<T: IntegerValueSource<T>> IntegerValue for IntegerValueStorage<T> {
    fn get_value(&self) -> i64 {
        self.value.clone().into()
    }
}

impl<T: IntegerValueSource<T>> AsStaticValue for IntegerValueStorage<T> {
    fn to_static_value(&self) -> StaticValue<'_> {
        StaticValue::Integer(self)
    }
}

impl<T: IntegerValueSource<T>> AsStaticValueMut for IntegerValueStorage<T> {
    fn to_static_value_mut(&mut self) -> Option<StaticValueMut<'_>> {
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
    fn to_static_value(&self) -> StaticValue<'_> {
        StaticValue::Regex(self)
    }
}

impl AsStaticValueMut for RegexValueStorage {
    fn to_static_value_mut(&mut self) -> Option<StaticValueMut<'_>> {
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
    fn to_static_value(&self) -> StaticValue<'_> {
        StaticValue::String(self)
    }
}

impl AsStaticValueMut for StringValueStorage {
    fn to_static_value_mut(&mut self) -> Option<StaticValueMut<'_>> {
        Some(StaticValueMut::String(self))
    }
}

#[derive(Debug, Clone)]
pub struct TimeSpanValueStorage {
    value: TimeDelta,
}

impl TimeSpanValueStorage {
    pub fn new(value: TimeDelta) -> TimeSpanValueStorage {
        Self { value }
    }

    pub fn get_raw_value(&self) -> &TimeDelta {
        &self.value
    }

    pub fn get_raw_value_mut(&mut self) -> &mut TimeDelta {
        &mut self.value
    }
}

impl TimeSpanValue for TimeSpanValueStorage {
    fn get_value(&self) -> TimeDelta {
        self.value
    }
}

impl AsStaticValue for TimeSpanValueStorage {
    fn to_static_value(&self) -> StaticValue<'_> {
        StaticValue::TimeSpan(self)
    }
}

pub trait EnumerableValueSource<T>:
    AsStaticValue + AsStaticValueMut + Into<OwnedValue> + From<OwnedValue> + 'static
{
}

impl<T: AsStaticValue + AsStaticValueMut + Into<OwnedValue> + From<OwnedValue> + 'static>
    EnumerableValueSource<T> for T
{
}

#[derive(Debug, Clone)]
pub struct ArrayValueStorage<T: EnumerableValueSource<T>> {
    values: Vec<T>,
}

impl<T: EnumerableValueSource<T>> ArrayValueStorage<T> {
    pub fn new(values: Vec<T>) -> ArrayValueStorage<T> {
        Self { values }
    }

    pub fn into<TTarget: EnumerableValueSource<TTarget>>(mut self) -> ArrayValueStorage<TTarget> {
        ArrayValueStorage::<TTarget>::new(self.values.drain(..).map(|v| v.into().into()).collect())
    }

    pub fn get_values(&self) -> &[T] {
        &self.values
    }

    pub fn get_values_mut(&mut self) -> &mut Vec<T> {
        &mut self.values
    }

    pub fn drain(&mut self, range: ArrayRange) -> std::vec::Drain<'_, T> {
        let start = range.get_start_range_inclusize().unwrap_or(0);
        let end = range.get_end_range_exclusive().unwrap_or(self.values.len());
        self.values.drain(start..end)
    }
}

impl<T: EnumerableValueSource<T>> ArrayValue for ArrayValueStorage<T> {
    fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    fn len(&self) -> usize {
        self.values.len()
    }

    fn get(&self, index: usize) -> Option<&(dyn AsValue + 'static)> {
        self.values.get(index).map(|v| v as &dyn AsValue)
    }

    fn get_static(&self, index: usize) -> Result<Option<&(dyn AsStaticValue + 'static)>, String> {
        Ok(self.values.get(index).map(|v| v as &dyn AsStaticValue))
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

impl<T: EnumerableValueSource<T>> AsStaticValue for ArrayValueStorage<T> {
    fn to_static_value(&self) -> StaticValue<'_> {
        StaticValue::Array(self)
    }
}

impl<T: EnumerableValueSource<T>> AsStaticValueMut for ArrayValueStorage<T> {
    fn to_static_value_mut(&mut self) -> Option<StaticValueMut<'_>> {
        Some(StaticValueMut::Array(self))
    }
}

impl<T: EnumerableValueSource<T>> ArrayValueMut for ArrayValueStorage<T> {
    fn get_mut(&mut self, index: usize) -> ValueMutGetResult<'_> {
        if let Some(v) = self.values.get_mut(index) {
            ValueMutGetResult::Found(v)
        } else {
            ValueMutGetResult::NotFound
        }
    }

    fn set(&mut self, index: usize, value: ResolvedValue) -> ValueMutWriteResult {
        match self.values.get_mut(index) {
            Some(v) => {
                let old = mem::replace(v, T::from(value.into()));
                ValueMutWriteResult::Updated(old.into())
            }
            None => ValueMutWriteResult::NotFound,
        }
    }

    fn push(&mut self, value: ResolvedValue) -> ValueMutWriteResult {
        self.values.push(T::from(value.into()));

        ValueMutWriteResult::Created
    }

    fn insert(&mut self, index: usize, value: ResolvedValue) -> ValueMutWriteResult {
        if index > self.values.len() {
            return ValueMutWriteResult::NotFound;
        }

        self.values.insert(index, T::from(value.into()));

        ValueMutWriteResult::Created
    }

    fn remove(&mut self, index: usize) -> ValueMutRemoveResult {
        if index >= self.values.len() {
            return ValueMutRemoveResult::NotFound;
        }

        let old = self.values.remove(index);

        ValueMutRemoveResult::Removed(old.into())
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
pub struct MapValueStorage<T: EnumerableValueSource<T>> {
    values: HashMap<Box<str>, T>,
}

impl<T: EnumerableValueSource<T>> MapValueStorage<T> {
    pub fn new(values: HashMap<Box<str>, T>) -> MapValueStorage<T> {
        Self { values }
    }

    pub fn get_values(&self) -> &HashMap<Box<str>, T> {
        &self.values
    }

    pub fn get_values_mut(&mut self) -> &mut HashMap<Box<str>, T> {
        &mut self.values
    }

    pub fn take_values(self) -> HashMap<Box<str>, T> {
        self.values
    }

    pub fn into<TTarget: EnumerableValueSource<TTarget>>(mut self) -> MapValueStorage<TTarget> {
        MapValueStorage::<TTarget>::new(HashMap::from_iter(
            self.values.drain().map(|(k, v)| (k, v.into().into())),
        ))
    }
}

impl<T: EnumerableValueSource<T>> MapValue for MapValueStorage<T> {
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

impl<T: EnumerableValueSource<T>> AsStaticValue for MapValueStorage<T> {
    fn to_static_value(&self) -> StaticValue<'_> {
        StaticValue::Map(self)
    }
}

impl<T: EnumerableValueSource<T>> AsStaticValueMut for MapValueStorage<T> {
    fn to_static_value_mut(&mut self) -> Option<StaticValueMut<'_>> {
        Some(StaticValueMut::Map(self))
    }
}

impl<T: EnumerableValueSource<T>> MapValueMut for MapValueStorage<T> {
    fn get_mut(&mut self, key: &str) -> ValueMutGetResult<'_> {
        match self.values.get_mut(key) {
            Some(v) => ValueMutGetResult::Found(v),
            None => ValueMutGetResult::NotFound,
        }
    }

    fn set(&mut self, key: &str, value: ResolvedValue) -> ValueMutWriteResult {
        match self.values.insert(key.into(), T::from(value.into())) {
            Some(old) => ValueMutWriteResult::Updated(old.into()),
            None => ValueMutWriteResult::Created,
        }
    }

    fn rename(&mut self, from_key: &str, to_key: &str) -> ValueMutWriteResult {
        match self.values.remove(from_key) {
            Some(v) => match self.values.insert(to_key.into(), v) {
                Some(old) => ValueMutWriteResult::Updated(old.into()),
                None => ValueMutWriteResult::Created,
            },
            None => ValueMutWriteResult::NotFound,
        }
    }

    fn remove(&mut self, key: &str) -> ValueMutRemoveResult {
        match self.values.remove(key) {
            Some(old) => ValueMutRemoveResult::Removed(old.into()),
            None => ValueMutRemoveResult::NotFound,
        }
    }

    fn retain(&mut self, item_callback: &mut dyn KeyValueMutCallback) {
        self.values.retain(|k, v| item_callback.next(k, v));
    }
}

impl<T: EnumerableValueSource<T>> Record for MapValueStorage<T> {
    fn get_diagnostic_level(&self) -> Option<RecordSetEngineDiagnosticLevel> {
        None
    }
}
