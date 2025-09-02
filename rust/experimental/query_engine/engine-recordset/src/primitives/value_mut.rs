// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::fmt::Debug;

use data_engine_expressions::*;

use crate::{OwnedValue, ResolvedValue};

#[derive(Debug)]
pub enum StaticValueMut<'a> {
    Array(&'a mut (dyn ArrayValueMut + 'static)),
    Map(&'a mut (dyn MapValueMut + 'static)),
    String(&'a mut (dyn StringValueMut + 'static)),
}

impl<'a> AsStaticValue for StaticValueMut<'a> {
    fn to_static_value(&self) -> StaticValue<'_> {
        match self {
            StaticValueMut::Array(a) => StaticValue::Array(*a),
            StaticValueMut::Map(b) => StaticValue::Map(*b),
            StaticValueMut::String(s) => StaticValue::String(*s),
        }
    }
}

pub trait AsStaticValueMut: AsStaticValue {
    fn to_static_value_mut(&mut self) -> Option<StaticValueMut<'_>>;
}

pub enum ValueMutGetResult<'a> {
    Found(&'a mut (dyn AsStaticValueMut + 'static)),
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

pub trait ArrayValueMut: ArrayValue {
    fn get_mut(&mut self, index: usize) -> ValueMutGetResult<'_>;

    fn set(&mut self, index: usize, value: ResolvedValue) -> ValueMutWriteResult;

    fn push(&mut self, value: ResolvedValue) -> ValueMutWriteResult;

    fn insert(&mut self, index: usize, value: ResolvedValue) -> ValueMutWriteResult;

    fn remove(&mut self, index: usize) -> ValueMutRemoveResult;

    fn retain(&mut self, item_callback: &mut dyn IndexValueMutCallback);
}

pub trait IndexValueMutCallback {
    fn next(&mut self, index: usize, value: &mut (dyn AsStaticValueMut + 'static)) -> bool;
}

pub struct IndexValueMutClosureCallback<F>
where
    F: FnMut(usize, &mut (dyn AsStaticValueMut + 'static)) -> bool,
{
    callback: F,
}

impl<F> IndexValueMutClosureCallback<F>
where
    F: FnMut(usize, &mut (dyn AsStaticValueMut + 'static)) -> bool,
{
    pub fn new(callback: F) -> IndexValueMutClosureCallback<F> {
        Self { callback }
    }
}

impl<F> IndexValueMutCallback for IndexValueMutClosureCallback<F>
where
    F: FnMut(usize, &mut (dyn AsStaticValueMut + 'static)) -> bool,
{
    fn next(&mut self, index: usize, value: &mut (dyn AsStaticValueMut + 'static)) -> bool {
        (self.callback)(index, value)
    }
}

pub trait MapValueMut: MapValue {
    fn get_mut(&mut self, key: &str) -> ValueMutGetResult<'_>;

    fn set(&mut self, key: &str, value: ResolvedValue) -> ValueMutWriteResult;

    fn rename(&mut self, from_key: &str, to_key: &str) -> ValueMutWriteResult;

    fn remove(&mut self, key: &str) -> ValueMutRemoveResult;

    fn retain(&mut self, item_callback: &mut dyn KeyValueMutCallback);
}

pub trait KeyValueMutCallback {
    fn next(&mut self, key: &str, value: &mut (dyn AsStaticValueMut + 'static)) -> bool;
}

pub struct KeyValueMutClosureCallback<F>
where
    F: FnMut(&str, &mut (dyn AsStaticValueMut + 'static)) -> bool,
{
    callback: F,
}

impl<F> KeyValueMutClosureCallback<F>
where
    F: FnMut(&str, &mut (dyn AsStaticValueMut + 'static)) -> bool,
{
    pub fn new(callback: F) -> KeyValueMutClosureCallback<F> {
        Self { callback }
    }
}

impl<F> KeyValueMutCallback for KeyValueMutClosureCallback<F>
where
    F: FnMut(&str, &mut (dyn AsStaticValueMut + 'static)) -> bool,
{
    fn next(&mut self, key: &str, value: &mut (dyn AsStaticValueMut + 'static)) -> bool {
        (self.callback)(key, value)
    }
}

pub trait StringValueMut: StringValue {
    fn get_value_mut(&mut self) -> &mut String;
}
