// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Native result metadata and cursor bindings over the shared source-query contract.

use super::adapter::{Error, Result};
use otel_arrow_dfe_scraper::database::{
    ColumnMetadata, CompiledQuery, CompiledWatermark, ScalarValue, SourceQuery,
};
use std::collections::BTreeSet;
use tokio_postgres::types::Type;

#[derive(Clone)]
pub(crate) struct Plan {
    pub sql: String,
    pub cursor: CursorColumns,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ScalarKind {
    Int64,
    Timestamp,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CursorColumns {
    Snapshot,
    Scalar { index: usize, kind: ScalarKind },
    Composite { timestamp: usize, tie: usize },
}

impl CursorColumns {
    pub fn indices(self) -> Vec<usize> {
        match self {
            Self::Snapshot => vec![],
            Self::Scalar { index, .. } => vec![index],
            Self::Composite { timestamp, tie } => vec![timestamp, tie],
        }
    }

    pub fn timestamp(self) -> Option<usize> {
        match self {
            Self::Composite { timestamp, .. }
            | Self::Scalar {
                index: timestamp,
                kind: ScalarKind::Timestamp,
            } => Some(timestamp),
            Self::Snapshot | Self::Scalar { .. } => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ResultColumn {
    pub name: String,
    pub source_type: String,
    pub nullable: bool,
    pub type_modifier: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ResolvedSchema {
    pub columns: Vec<ResultColumn>,
    pub native_types: Vec<Type>,
}

impl ResolvedSchema {
    pub fn discover(columns: &[tokio_postgres::Column]) -> Result<Self> {
        if !(1..=128).contains(&columns.len()) {
            return Err(Error::Metadata);
        }
        let mut names = BTreeSet::new();
        let mut result = Vec::with_capacity(columns.len());
        let mut native_types = Vec::with_capacity(columns.len());
        for column in columns {
            let native = super::value::native_type(column.type_().name())?;
            let modifier = column.type_modifier();
            if column.name().trim().is_empty()
                || !names.insert(column.name().to_ascii_lowercase())
                || &native != column.type_()
                || modifier < -1
                || (matches!(native, Type::TIMESTAMP | Type::TIMESTAMPTZ)
                    && !(-1..=6).contains(&modifier))
            {
                return Err(Error::Metadata);
            }
            result.push(ResultColumn {
                name: column.name().to_owned(),
                source_type: native.name().to_owned(),
                // RowDescription has no nullability proof for arbitrary expressions/views.
                // Generated predicates surface null cursors for value-level rejection.
                nullable: true,
                type_modifier: modifier,
            });
            native_types.push(native);
        }
        Ok(Self {
            columns: result,
            native_types,
        })
    }

    pub fn metadata(&self) -> Vec<ColumnMetadata> {
        self.columns
            .iter()
            .map(|column| ColumnMetadata {
                name: column.name.clone(),
                source_type: column.source_type.clone(),
                nullable: column.nullable,
            })
            .collect()
    }
}

impl Plan {
    pub fn resolve(
        common: &CompiledQuery,
        source: &SourceQuery,
        schema: &ResolvedSchema,
    ) -> Result<Self> {
        let index = |name: &str| {
            schema
                .columns
                .iter()
                .position(|column| column.name.eq_ignore_ascii_case(name))
                .ok_or(Error::Metadata)
        };
        let cursor = match common.watermark() {
            CompiledWatermark::Snapshot => CursorColumns::Snapshot,
            CompiledWatermark::Scalar(spec) => {
                let index = index(&spec.column)?;
                let kind = match (&spec.initial, &schema.native_types[index]) {
                    (ScalarValue::Int64(_), &Type::INT2 | &Type::INT4 | &Type::INT8) => {
                        ScalarKind::Int64
                    }
                    (ScalarValue::Timestamp(_), &Type::TIMESTAMP | &Type::TIMESTAMPTZ) => {
                        ScalarKind::Timestamp
                    }
                    _ => return Err(Error::Metadata),
                };
                CursorColumns::Scalar { index, kind }
            }
            CompiledWatermark::Composite(spec) => {
                let timestamp = index(&spec.timestamp_column)?;
                let tie = index(&spec.tie_breaker_column)?;
                if timestamp == tie
                    || !matches!(
                        schema.native_types[timestamp],
                        Type::TIMESTAMP | Type::TIMESTAMPTZ
                    )
                    || !matches!(
                        schema.native_types[tie],
                        Type::INT2 | Type::INT4 | Type::INT8
                    )
                {
                    return Err(Error::Metadata);
                }
                CursorColumns::Composite { timestamp, tie }
            }
        };
        if common
            .output()
            .timestamp_column
            .as_deref()
            .is_some_and(|name| {
                !index(name).is_ok_and(|index| {
                    matches!(
                        schema.native_types[index],
                        Type::TIMESTAMP | Type::TIMESTAMPTZ
                    )
                })
            })
        {
            return Err(Error::Metadata);
        }
        let plan = Self {
            sql: source.paged_sql(common, &schema.metadata())?,
            cursor,
        };
        let _ = super::value::parameters(&common.watermark().initial(), &plan, schema)?;
        Ok(plan)
    }
}
