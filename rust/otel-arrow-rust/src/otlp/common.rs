// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::arrays::{NullableArrayAccessor, StringArrayAccessor};
use crate::error;
use crate::proto::opentelemetry::common::v1::InstrumentationScope;
use crate::schema::consts;
use arrow::array::{Array, ArrayRef, RecordBatch, StructArray, UInt16Array, UInt32Array};
use arrow::datatypes::{DataType, Field, Fields};
use snafu::OptionExt;

pub(in crate::otlp) struct ResourceArrays<'a> {
    pub id: &'a UInt16Array,
    pub dropped_attributes_count: Option<&'a UInt32Array>,
    pub schema_url: Option<StringArrayAccessor<'a>>,
}

impl ResourceArrays<'_> {
    fn data_type() -> DataType {
        DataType::Struct(Fields::from(vec![
            Field::new(consts::ID, DataType::UInt16, true),
            Field::new(consts::DROPPED_ATTRIBUTES_COUNT, DataType::UInt32, true),
            Field::new(
                consts::SCHEMA_URL,
                DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                true,
            ),
        ]))
    }
}

impl<'a> TryFrom<&'a RecordBatch> for ResourceArrays<'a> {
    type Error = error::Error;

    fn try_from(rb: &'a RecordBatch) -> Result<Self, Self::Error> {
        let struct_array = Downcaster {
            name: consts::RESOURCE,
            source: rb,
            array: |rb: &'a RecordBatch| rb.column_by_name(consts::RESOURCE),
            expect_type: Self::data_type,
        }
        .downcast::<StructArray>()?;

        let id_array = Downcaster {
            name: consts::ID,
            source: struct_array,
            array: |s: &'a StructArray| s.column_by_name(consts::ID),
            expect_type: || DataType::UInt16,
        }
        .downcast::<UInt16Array>()?;

        let dropped_attributes_count = struct_array
            .column_by_name(consts::DROPPED_ATTRIBUTES_COUNT)
            .map(|a| {
                a.as_any().downcast_ref::<UInt32Array>().context(
                    error::ColumnDataTypeMismatchSnafu {
                        name: consts::DROPPED_ATTRIBUTES_COUNT,
                        expect: DataType::UInt32,
                        actual: a.data_type().clone(),
                    },
                )
            })
            .transpose()?;

        let schema_url = struct_array
            .column_by_name(consts::SCHEMA_URL)
            .map(StringArrayAccessor::try_new)
            .transpose()?;

        Ok(Self {
            id: id_array,
            dropped_attributes_count,
            schema_url,
        })
    }
}

pub(in crate::otlp) struct ScopeArrays<'a> {
    pub name: Option<StringArrayAccessor<'a>>,
    pub version: Option<StringArrayAccessor<'a>>,
    pub dropped_attributes_count: Option<&'a UInt32Array>,
    pub id: Option<&'a UInt16Array>,
}

impl ScopeArrays<'_> {
    fn data_type() -> DataType {
        DataType::Struct(Fields::from(vec![
            Field::new(
                consts::NAME,
                DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                true,
            ),
            Field::new(consts::VERSION, DataType::Utf8, true),
            Field::new(consts::DROPPED_ATTRIBUTES_COUNT, DataType::UInt32, true),
            Field::new(consts::ID, DataType::UInt16, true),
        ]))
    }

    pub fn create_instrumentation_scope(&self, idx: usize) -> InstrumentationScope {
        InstrumentationScope {
            name: self.name.value_at(idx).unwrap_or_default(),
            version: self.version.value_at_or_default(idx),
            dropped_attributes_count: self.dropped_attributes_count.value_at_or_default(idx),
            attributes: vec![],
        }
    }
}

impl<'a> TryFrom<&'a RecordBatch> for ScopeArrays<'a> {
    type Error = error::Error;

    fn try_from(rb: &'a RecordBatch) -> Result<Self, Self::Error> {
        let scope_array = Downcaster {
            name: consts::SCOPE,
            source: rb,
            array: |rb: &'a RecordBatch| rb.column_by_name(consts::SCOPE),
            expect_type: Self::data_type,
        }
        .downcast::<StructArray>()?;

        let name = scope_array
            .column_by_name(consts::NAME)
            .map(StringArrayAccessor::try_new)
            .transpose()?;

        let version = scope_array
            .column_by_name(consts::VERSION)
            .map(StringArrayAccessor::try_new)
            .transpose()?;

        let dropped_attributes_count = scope_array
            .column_by_name(consts::DROPPED_ATTRIBUTES_COUNT)
            .map(|a| {
                a.as_any().downcast_ref::<UInt32Array>().context(
                    error::ColumnDataTypeMismatchSnafu {
                        name: consts::DROPPED_ATTRIBUTES_COUNT,
                        expect: DataType::UInt32,
                        actual: a.data_type().clone(),
                    },
                )
            })
            .transpose()?;

        let id = scope_array
            .column_by_name(consts::ID)
            .map(|a| {
                a.as_any().downcast_ref::<UInt16Array>().with_context(|| {
                    error::ColumnDataTypeMismatchSnafu {
                        name: consts::ID,
                        expect: DataType::UInt16,
                        actual: a.data_type().clone(),
                    }
                })
            })
            .transpose()?;

        Ok(Self {
            name,
            version,
            dropped_attributes_count,
            id,
        })
    }
}

struct Downcaster<S, F> {
    // name is the column name, e.g. crate::schema::consts::RESOURCE
    name: &'static str,
    // source is a record batch (determines lifetime of the downcast array)
    source: S,
    // array is a function returning an optional array ref e.g., `|s| s.column_by_name(name)`
    array: F,
    // expect_type is used when downcasting fails, included in the result context.
    expect_type: fn() -> DataType,
}

impl<'a, S, F> Downcaster<S, F> {
    fn downcast<'s, A>(self) -> error::Result<&'a A>
    where
        A: Array + 'static,
        F: Fn(S) -> Option<&'a ArrayRef>,
        S: 'a,
    {
        let array =
            (self.array)(self.source).context(error::ColumnNotFoundSnafu { name: self.name })?;
        array
            .as_any()
            .downcast_ref::<A>()
            .with_context(|| error::ColumnDataTypeMismatchSnafu {
                name: self.name,
                expect: (self.expect_type)(),
                actual: array.data_type().clone(),
            })
    }
}
