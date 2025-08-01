use data_engine_expressions::*;

use crate::{OwnedValue, ResolvedValue};

#[derive(Debug)]
pub enum ValueMut<'a> {
    Array(&'a mut (dyn ArrayValueMut + 'static)),
    Map(&'a mut (dyn MapValueMut + 'static)),
    String(&'a mut (dyn StringValueMut + 'static)),
}

pub trait AsValueMut: AsValue {
    fn to_value_mut(&mut self) -> Option<ValueMut>;
}

impl<'a> AsValue for ValueMut<'a> {
    fn get_value_type(&self) -> ValueType {
        match self {
            ValueMut::Array(_) => ValueType::Array,
            ValueMut::Map(_) => ValueType::Map,
            ValueMut::String(_) => ValueType::String,
        }
    }

    fn to_value(&self) -> Value {
        match self {
            ValueMut::Array(a) => Value::Array(*a),
            ValueMut::Map(b) => Value::Map(*b),
            ValueMut::String(s) => Value::String(*s),
        }
    }
}

#[derive(Debug)]
pub enum InnerValue<'a> {
    Value(&'a (dyn AsValue + 'static)),
    ValueMut(&'a mut (dyn AsValueMut + 'static)),
}

impl AsValue for InnerValue<'_> {
    fn get_value_type(&self) -> ValueType {
        match self {
            InnerValue::Value(v) => v.get_value_type(),
            InnerValue::ValueMut(v) => v.get_value_type(),
        }
    }

    fn to_value(&self) -> Value {
        match self {
            InnerValue::Value(v) => v.to_value(),
            InnerValue::ValueMut(v) => v.to_value(),
        }
    }
}

impl AsValueMut for InnerValue<'_> {
    fn to_value_mut(&mut self) -> Option<ValueMut> {
        match self {
            InnerValue::Value(_) => None,
            InnerValue::ValueMut(v) => v.to_value_mut(),
        }
    }
}

pub enum ValueMutGetResult<'a> {
    Found(&'a mut (dyn AsValueMut + 'static)),
    NotFound,
    NotSupported(String),
}

pub enum ValueMutWriteResult {
    NotFound,
    Created,
    Updated(OwnedValue),
    NotSupported(String),
}

pub enum ValueMutRemoveResult {
    NotFound,
    Removed(OwnedValue),
}

pub trait ArrayValueMut: ArrayValue + AsValueMut {
    fn get_mut(&mut self, index: usize) -> ValueMutGetResult;

    fn set(&mut self, index: usize, value: ResolvedValue) -> ValueMutWriteResult;

    fn push(&mut self, value: ResolvedValue) -> ValueMutWriteResult;

    fn insert(&mut self, index: usize, value: ResolvedValue) -> ValueMutWriteResult;

    fn remove(&mut self, index: usize) -> ValueMutRemoveResult;

    fn retain(&mut self, item_callback: &mut dyn IndexValueMutCallback);
}

pub trait IndexValueMutCallback {
    fn next(&mut self, index: usize, value: InnerValue) -> bool;
}

pub struct IndexValueMutClosureCallback<F>
where
    F: FnMut(usize, InnerValue) -> bool,
{
    callback: F,
}

impl<F> IndexValueMutClosureCallback<F>
where
    F: FnMut(usize, InnerValue) -> bool,
{
    pub fn new(callback: F) -> IndexValueMutClosureCallback<F> {
        Self { callback }
    }
}

impl<F> IndexValueMutCallback for IndexValueMutClosureCallback<F>
where
    F: FnMut(usize, InnerValue) -> bool,
{
    fn next(&mut self, index: usize, value: InnerValue) -> bool {
        (self.callback)(index, value)
    }
}

pub trait MapValueMut: MapValue + AsValueMut {
    fn get_mut(&mut self, key: &str) -> ValueMutGetResult;

    fn set(&mut self, key: &str, value: ResolvedValue) -> ValueMutWriteResult;

    fn rename(&mut self, from_key: &str, to_key: &str) -> ValueMutWriteResult;

    fn remove(&mut self, key: &str) -> ValueMutRemoveResult;

    fn retain(&mut self, item_callback: &mut dyn KeyValueMutCallback);
}

pub trait KeyValueMutCallback {
    fn next(&mut self, key: &str, value: InnerValue) -> bool;
}

pub struct KeyValueMutClosureCallback<F>
where
    F: FnMut(&str, InnerValue) -> bool,
{
    callback: F,
}

impl<F> KeyValueMutClosureCallback<F>
where
    F: FnMut(&str, InnerValue) -> bool,
{
    pub fn new(callback: F) -> KeyValueMutClosureCallback<F> {
        Self { callback }
    }
}

impl<F> KeyValueMutCallback for KeyValueMutClosureCallback<F>
where
    F: FnMut(&str, InnerValue) -> bool,
{
    fn next(&mut self, key: &str, value: InnerValue) -> bool {
        (self.callback)(key, value)
    }
}

pub trait StringValueMut: StringValue + AsValueMut {
    fn get_value_mut(&mut self) -> &mut String;
}
