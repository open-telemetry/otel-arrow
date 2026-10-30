// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![allow(unused)]

use std::fmt::{Display, Write};

use otel_arrow_contrib_data_engine_expressions::*;

use crate::*;

#[derive(Debug)]
pub(crate) enum ResolvedScalarValue<'a, 'b>
where
    'a: 'b,
{
    Single(ValueOrRef<'a>),
    Dictionary(Dictionary<'a>),
    Table(&'b dyn RecordTable<'a>),
}

impl<'a, 'b> ResolvedScalarValue<'a, 'b> {
    pub fn new_null() -> ResolvedScalarValue<'a, 'b> {
        ResolvedScalarValue::Single(ValueOrRef::Null)
    }

    pub fn new_int(value: i64) -> ResolvedScalarValue<'a, 'b> {
        ResolvedScalarValue::Single(ValueOrRef::Integer(value))
    }

    pub fn new_from_value(value: Value<'a>) -> ResolvedScalarValue<'a, 'b> {
        ResolvedScalarValue::Single(match value {
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
        })
    }

    pub fn map_into<FSingle, FDictionary, FTable, FRet>(
        self,
        when_single: FSingle,
        when_dictionary: FDictionary,
        when_table: FTable,
    ) -> FRet
    where
        FSingle: FnOnce(ValueOrRef<'a>) -> FRet,
        FDictionary: FnOnce(Dictionary<'a>) -> FRet,
        FTable: FnOnce(&'b dyn RecordTable) -> FRet,
    {
        match self {
            ResolvedScalarValue::Single(single) => when_single(single),
            ResolvedScalarValue::Dictionary(dictionary) => when_dictionary(dictionary),
            ResolvedScalarValue::Table(table) => when_table(table),
        }
    }

    pub fn map_into_with_state<TState, FSingle, FDictionary, FTable, FRet>(
        self,
        state: TState,
        when_single: FSingle,
        when_dictionary: FDictionary,
        when_table: FTable,
    ) -> FRet
    where
        FSingle: FnOnce(TState, ValueOrRef<'a>) -> FRet,
        FDictionary: FnOnce(TState, Dictionary<'a>) -> FRet,
        FTable: FnOnce(TState, &'b dyn RecordTable) -> FRet,
    {
        match self {
            ResolvedScalarValue::Single(single) => when_single(state, single),
            ResolvedScalarValue::Dictionary(dictionary) => when_dictionary(state, dictionary),
            ResolvedScalarValue::Table(table) => when_table(state, table),
        }
    }

    pub fn try_into_dictionary(
        self,
        key_data_type: DictionaryKeyType,
        key_count: usize,
    ) -> Result<Dictionary<'a>, ()> {
        match self {
            ResolvedScalarValue::Single(s) => Ok(s.into_dictionary(key_data_type, key_count)),
            ResolvedScalarValue::Dictionary(d) => Ok(d),
            ResolvedScalarValue::Table(_) => Err(()),
        }
    }
}

impl ResolvedScalarValue<'_, '_> {
    pub fn try_get_key_info(
        values: &[&ResolvedScalarValue<'_, '_>],
    ) -> Result<(DictionaryKeyType, usize), ()> {
        for value in values {
            if let ResolvedScalarValue::Dictionary(d) = value {
                let keys = d.keys();
                return Ok((keys.data_type(), keys.len()));
            }
        }

        Err(())
    }
}

impl Display for ResolvedScalarValue<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolvedScalarValue::Table(t) => t.fmt(f),
            ResolvedScalarValue::Single(s) => {
                f.write_char('[')?;
                s.to_value().diagnostic_fmt(f)?;
                f.write_char(']')
            }
            ResolvedScalarValue::Dictionary(d) => d.diagnostic_fmt(f),
        }
    }
}
