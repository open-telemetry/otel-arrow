// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::{
    cell::RefMut,
    ops::{Deref, DerefMut},
};

use data_engine_expressions::*;

use crate::{execution_context::ExecutionContext, *};

#[derive(Debug)]
pub(crate) enum ResolvedValueMut<'a, 'b> {
    Map(RefMut<'a, dyn MapValueMut + 'static>),
    MapKey {
        map: RefMut<'a, dyn MapValueMut + 'static>,
        key: ResolvedStringValue<'b>,
    },
    ArrayIndex {
        array: RefMut<'a, dyn ArrayValueMut + 'static>,
        index: usize,
    },
}

#[derive(Debug)]
pub struct ResolvedMutableArgument<'a, 'b> {
    pub(crate) value: Option<ResolvedValueMut<'a, 'b>>,
}

impl<'a, 'b> AsStaticValueMut for ResolvedMutableArgument<'a, 'b> {
    fn to_static_value_mut(&mut self) -> Option<StaticValueMut<'_>> {
        match &mut self.value {
            Some(ResolvedValueMut::Map(m)) => Some(StaticValueMut::Map(m.deref_mut())),
            Some(ResolvedValueMut::MapKey { map, key }) => match map.get_mut(key.get_value()) {
                ValueMutGetResult::Found(v) => v.to_static_value_mut(),
                _ => None,
            },
            Some(ResolvedValueMut::ArrayIndex { array, index }) => match array.get_mut(*index) {
                ValueMutGetResult::Found(v) => v.to_static_value_mut(),
                _ => None,
            },
            None => None,
        }
    }
}

impl<'a, 'b> AsStaticValue for ResolvedMutableArgument<'a, 'b> {
    fn to_static_value(&self) -> StaticValue<'_> {
        match &self.value {
            Some(ResolvedValueMut::Map(m)) => StaticValue::Map(m.deref()),
            Some(ResolvedValueMut::MapKey { map, key }) => match map.get_static(key.get_value()) {
                Ok(Some(v)) => v.to_static_value(),
                _ => StaticValue::Null,
            },
            Some(ResolvedValueMut::ArrayIndex { array, index }) => match array.get_static(*index) {
                Ok(Some(v)) => v.to_static_value(),
                _ => StaticValue::Null,
            },
            None => StaticValue::Null,
        }
    }
}

pub(crate) fn resolve_map_key_mut<'a, 'b, TRecord: Record>(
    execution_context: &ExecutionContext<'a, '_, TRecord>,
    expression: &'a dyn Expression,
    map: RefMut<'b, dyn MapValueMut + 'static>,
    key: &str,
) -> Option<RefMut<'b, dyn AsStaticValueMut + 'static>> {
    RefMut::filter_map(map, |v| match v.get_mut(key) {
        ValueMutGetResult::Found(v) => {
            execution_context.add_diagnostic_if_enabled(
                RecordSetEngineDiagnosticLevel::Verbose,
                expression,
                || {
                    format!(
                        "Resolved '{}' value for key '{key}' specified in accessor expression",
                        v.get_value_type()
                    )
                },
            );
            Some(v)
        }
        _ => {
            execution_context.add_diagnostic_if_enabled(
                RecordSetEngineDiagnosticLevel::Warn,
                expression,
                || format!("Could not find map key '{key}' specified in accessor expression"),
            );
            None
        }
    })
    .ok()
}

pub(crate) fn resolve_array_index_mut<'a, 'b, TRecord: Record>(
    execution_context: &ExecutionContext<'a, '_, TRecord>,
    expression: &'a dyn Expression,
    array: RefMut<'b, dyn ArrayValueMut + 'static>,
    index: usize,
) -> Option<RefMut<'b, dyn AsStaticValueMut + 'static>> {
    RefMut::filter_map(array, |v| {
        match v.get_mut(index) {
            ValueMutGetResult::Found(v) => {
                execution_context.add_diagnostic_if_enabled(
                    RecordSetEngineDiagnosticLevel::Verbose,
                    expression,
                    || format!("Resolved '{}' value for array index '{index}' specified in accessor expression", v.get_value_type()),
                );
                Some(v)
            }
            _ => {
                execution_context.add_diagnostic_if_enabled(
                    RecordSetEngineDiagnosticLevel::Warn,
                    expression,
                    || format!("Could not find array index '{index}' specified in accessor expression"),
                );
                None
            }
        }
    }).ok()
}
