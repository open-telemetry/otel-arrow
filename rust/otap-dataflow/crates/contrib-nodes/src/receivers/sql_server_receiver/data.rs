// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Live result plans and lossless conversion of stock Tiberius values.

use super::{SqlServerError as Error, SqlServerParameter, SqlServerStatement};
use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use otel_arrow_dfe_scraper::database::{
    CellValue, ColumnMetadata, CompiledQuery, CompiledWatermark, CompositeCursor, Cursor,
    CursorRow, Row, ScalarValue,
};
use std::collections::HashSet;
use tiberius::{ColumnData, ColumnType, time};

pub(super) const MAX_COLUMNS: usize = 256;
pub(super) const MAX_METADATA_BYTES: usize = 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ColumnPlan {
    pub metadata: ColumnMetadata,
    pub kind: NativeType,
    pub precision: u8,
    pub scale: u8,
    pub max_length: i16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum NativeType {
    Bit,
    TinyInt,
    SmallInt,
    Int,
    BigInt,
    Decimal,
    Real,
    Float,
    Text,
    Binary,
    Date,
    Time,
    DateTime2,
    DateTimeOffset,
    Guid,
}

impl NativeType {
    pub fn from_system_id(id: i32) -> Result<Self, Error> {
        Ok(match id {
            104 => Self::Bit,
            48 => Self::TinyInt,
            52 => Self::SmallInt,
            56 => Self::Int,
            127 => Self::BigInt,
            106 | 108 => Self::Decimal,
            59 => Self::Real,
            62 => Self::Float,
            167 | 175 | 231 | 239 => Self::Text,
            165 | 173 | 189 => Self::Binary,
            40 => Self::Date,
            41 => Self::Time,
            42 => Self::DateTime2,
            43 => Self::DateTimeOffset,
            36 => Self::Guid,
            // Money is not accepted through Tiberius' f64 decode.
            _ => return Err(Error::Metadata),
        })
    }

    pub fn matches(self, column: ColumnType) -> bool {
        match self {
            Self::Bit => matches!(column, ColumnType::Bit | ColumnType::Bitn),
            Self::TinyInt => column == ColumnType::Int1,
            Self::SmallInt => column == ColumnType::Int2,
            Self::Int => column == ColumnType::Int4,
            Self::BigInt => column == ColumnType::Int8,
            Self::Decimal => matches!(column, ColumnType::Decimaln | ColumnType::Numericn),
            Self::Real => column == ColumnType::Float4,
            Self::Float => column == ColumnType::Float8,
            Self::Text => matches!(
                column,
                ColumnType::BigVarChar
                    | ColumnType::BigChar
                    | ColumnType::NVarchar
                    | ColumnType::NChar
            ),
            Self::Binary => matches!(column, ColumnType::BigBinary | ColumnType::BigVarBin),
            Self::Date => column == ColumnType::Daten,
            Self::Time => column == ColumnType::Timen,
            Self::DateTime2 => column == ColumnType::Datetime2,
            Self::DateTimeOffset => column == ColumnType::DatetimeOffsetn,
            Self::Guid => column == ColumnType::Guid,
        }
    }
}

#[derive(Clone)]
pub(super) struct ResultPlan {
    pub columns: Vec<ColumnPlan>,
    cursor: CursorColumns,
}

#[derive(Clone, Copy)]
enum CursorColumns {
    Snapshot,
    Scalar(usize),
    Composite { timestamp: usize, integer: usize },
}

impl ResultPlan {
    pub fn new(columns: Vec<ColumnPlan>, query: &CompiledQuery) -> Result<Self, Error> {
        let mut names = HashSet::new();
        if columns.is_empty() || columns.len() > MAX_COLUMNS {
            return Err(Error::Metadata);
        }
        for column in &columns {
            if column.metadata.name.is_empty()
                || !names.insert(column.metadata.name.to_ascii_lowercase())
                || (column.kind == NativeType::Decimal
                    && (!(1..=38).contains(&column.precision) || column.scale > column.precision))
                || (matches!(
                    column.kind,
                    NativeType::Time | NativeType::DateTime2 | NativeType::DateTimeOffset
                ) && column.scale > 7)
            {
                return Err(Error::Metadata);
            }
        }
        let key = |name: &str| {
            columns
                .iter()
                .position(|column| column.metadata.name == name && !column.metadata.nullable)
                .ok_or(Error::Metadata)
        };
        let cursor = match query.watermark() {
            CompiledWatermark::Snapshot => CursorColumns::Snapshot,
            CompiledWatermark::Scalar(value) => {
                let index = key(&value.column)?;
                check_scalar(&columns[index], &value.initial)?;
                CursorColumns::Scalar(index)
            }
            CompiledWatermark::Composite(value) => {
                let timestamp = key(&value.timestamp_column)?;
                let integer = key(&value.tie_breaker_column)?;
                check_scalar(
                    &columns[timestamp],
                    &ScalarValue::Timestamp(value.initial.timestamp.clone()),
                )?;
                check_scalar(
                    &columns[integer],
                    &ScalarValue::Int64(value.initial.tie_breaker),
                )?;
                CursorColumns::Composite { timestamp, integer }
            }
        };
        Ok(Self { columns, cursor })
    }

    pub fn metadata(&self) -> Vec<ColumnMetadata> {
        self.columns
            .iter()
            .map(|column| column.metadata.clone())
            .collect()
    }

    pub fn check_columns(&self, columns: &[tiberius::Column]) -> Result<(), Error> {
        if columns.len() != self.columns.len()
            || columns.iter().zip(&self.columns).any(|(actual, expected)| {
                actual.name() != expected.metadata.name
                    || !expected.kind.matches(actual.column_type())
            })
        {
            return Err(Error::Metadata);
        }
        Ok(())
    }

    pub fn row(&self, row: tiberius::Row) -> Result<CursorRow, Error> {
        self.check_columns(row.columns())?;
        let mut values = Vec::with_capacity(self.columns.len());
        for (value, column) in row.into_iter().zip(&self.columns) {
            values.push(convert(value, column)?);
        }
        let cursor = match self.cursor {
            CursorColumns::Snapshot => Cursor::Snapshot,
            CursorColumns::Scalar(index) => match &values[index] {
                CellValue::Int64(value) => Cursor::Scalar(ScalarValue::Int64(*value)),
                CellValue::Timestamp(value) => {
                    Cursor::Scalar(ScalarValue::Timestamp(value.clone()))
                }
                _ => return Err(Error::Cursor),
            },
            CursorColumns::Composite { timestamp, integer } => {
                let (CellValue::Timestamp(timestamp), CellValue::Int64(integer)) =
                    (&values[timestamp], &values[integer])
                else {
                    return Err(Error::Cursor);
                };
                Cursor::Composite(CompositeCursor::new(timestamp.clone(), *integer))
            }
        };
        Ok(CursorRow {
            row: Row { values },
            cursor,
        })
    }

    pub fn request<'a>(
        &self,
        statement: &'a SqlServerStatement,
        query: &CompiledQuery,
        cursor: &Cursor,
    ) -> Result<tiberius::Query<'a>, Error> {
        query
            .watermark()
            .validate_cursor(cursor)
            .map_err(|_| Error::Cursor)?;
        let mut request = tiberius::Query::new(statement.driver_sql());
        for role in statement.parameters() {
            match (role, cursor, self.cursor) {
                (SqlServerParameter::PageLimit, _, _) => {
                    let limit = query.max_rows() + usize::from(matches!(cursor, Cursor::Snapshot));
                    request.bind(i64::try_from(limit).map_err(|_| Error::Cursor)?);
                }
                (
                    SqlServerParameter::ScalarInt64,
                    Cursor::Scalar(value),
                    CursorColumns::Scalar(index),
                )
                | (
                    SqlServerParameter::Timestamp,
                    Cursor::Scalar(value),
                    CursorColumns::Scalar(index),
                ) => {
                    request.bind(bind_scalar(&self.columns[index], value)?);
                }
                (
                    SqlServerParameter::Timestamp,
                    Cursor::Composite(value),
                    CursorColumns::Composite { timestamp, .. },
                ) => {
                    request.bind(bind_scalar(
                        &self.columns[timestamp],
                        &ScalarValue::Timestamp(value.timestamp.clone()),
                    )?);
                }
                (
                    SqlServerParameter::TieBreaker,
                    Cursor::Composite(value),
                    CursorColumns::Composite { integer, .. },
                ) => {
                    request.bind(bind_scalar(
                        &self.columns[integer],
                        &ScalarValue::Int64(value.tie_breaker),
                    )?);
                }
                _ => return Err(Error::Cursor),
            }
        }
        Ok(request)
    }
}

fn check_scalar(column: &ColumnPlan, value: &ScalarValue) -> Result<(), Error> {
    bind_scalar(column, value).map(|_| ())
}

pub(super) fn bind_scalar(
    column: &ColumnPlan,
    value: &ScalarValue,
) -> Result<ColumnData<'static>, Error> {
    Ok(match (column.kind, value) {
        (NativeType::SmallInt, ScalarValue::Int64(value)) => {
            ColumnData::I16(Some(i16::try_from(*value).map_err(|_| Error::Cursor)?))
        }
        (NativeType::Int, ScalarValue::Int64(value)) => {
            ColumnData::I32(Some(i32::try_from(*value).map_err(|_| Error::Cursor)?))
        }
        (NativeType::BigInt, ScalarValue::Int64(value)) => ColumnData::I64(Some(*value)),
        (NativeType::DateTime2, ScalarValue::Timestamp(value)) => {
            let normalized =
                super::config::normalize_timestamp(value).map_err(|_| Error::Cursor)?;
            let instant = DateTime::parse_from_rfc3339(&normalized)
                .map_err(|_| Error::Cursor)?
                .naive_utc();
            if column.scale > 7
                || instant.nanosecond() % 10u32.pow(9 - u32::from(column.scale)) != 0
            {
                return Err(Error::Cursor);
            }
            let days =
                u32::try_from(instant.date().num_days_from_ce() - 1).map_err(|_| Error::Cursor)?;
            let factor = 10u64.pow(u32::from(column.scale));
            let ticks = u64::from(instant.num_seconds_from_midnight()) * factor
                + u64::from(instant.nanosecond()) / 10u64.pow(9 - u32::from(column.scale));
            ColumnData::DateTime2(Some(time::DateTime2::new(
                time::Date::new(days),
                time::Time::new(ticks, column.scale),
            )))
        }
        _ => return Err(Error::Cursor),
    })
}

pub(super) fn convert(value: ColumnData<'static>, column: &ColumnPlan) -> Result<CellValue, Error> {
    let result = match (column.kind, value) {
        (NativeType::TinyInt, ColumnData::U8(value)) => {
            value.map(|v| CellValue::Int64(i64::from(v)))
        }
        (NativeType::SmallInt, ColumnData::I16(value)) => {
            value.map(|v| CellValue::Int64(i64::from(v)))
        }
        (NativeType::Int, ColumnData::I32(value)) => value.map(|v| CellValue::Int64(i64::from(v))),
        (NativeType::BigInt, ColumnData::I64(value)) => value.map(CellValue::Int64),
        (NativeType::Bit, ColumnData::Bit(value)) => value.map(CellValue::Bool),
        (NativeType::Real, ColumnData::F32(value)) => {
            value.map(|v| CellValue::Float64(f64::from(v)))
        }
        (NativeType::Float, ColumnData::F64(value)) => value.map(CellValue::Float64),
        (NativeType::Text, ColumnData::String(value)) => {
            value.map(|v| CellValue::String(v.into_owned()))
        }
        (NativeType::Binary, ColumnData::Binary(value)) => {
            value.map(|v| CellValue::Bytes(v.into_owned()))
        }
        (NativeType::Guid, ColumnData::Guid(value)) => {
            value.map(|v| CellValue::String(v.hyphenated().to_string()))
        }
        (NativeType::Decimal, ColumnData::Numeric(value)) => value
            .map(|value| {
                if value.scale() != column.scale
                    || !(1..=38).contains(&column.precision)
                    || value.value().unsigned_abs() >= 10u128.pow(u32::from(column.precision))
                {
                    return Err(Error::Conversion);
                }
                let scale = usize::from(value.scale());
                let mut text = format!(
                    "{:0width$}",
                    value.value().unsigned_abs(),
                    width = scale + 1
                );
                if scale > 0 {
                    text.insert(text.len() - scale, '.');
                }
                if value.value() < 0 {
                    text.insert(0, '-');
                }
                Ok(CellValue::Decimal(text))
            })
            .transpose()?,
        (NativeType::Date, ColumnData::Date(value)) => value
            .map(|v| date(v).map(|v| CellValue::String(v.to_string())))
            .transpose()?,
        (NativeType::Time, ColumnData::Time(value)) => value
            .map(|v| {
                if v.scale() != column.scale {
                    return Err(Error::Conversion);
                }
                clock(v).map(|v| {
                    CellValue::String(format!(
                        "{}.{:07}",
                        v.format("%H:%M:%S"),
                        v.nanosecond() / 100
                    ))
                })
            })
            .transpose()?,
        (NativeType::DateTime2, ColumnData::DateTime2(value)) => value
            .map(|v| {
                if v.time().scale() != column.scale {
                    return Err(Error::Conversion);
                }
                datetime(v).map(|v| CellValue::Timestamp(timestamp(v)))
            })
            .transpose()?,
        (NativeType::DateTimeOffset, ColumnData::DateTimeOffset(value)) => value
            .map(|v| {
                if v.datetime2().time().scale() != column.scale
                    || !(-840..=840).contains(&v.offset())
                {
                    return Err(Error::Conversion);
                }
                let utc = datetime(v.datetime2())?;
                let local = utc
                    .checked_add_signed(chrono::Duration::minutes(i64::from(v.offset())))
                    .ok_or(Error::Conversion)?;
                if !(1..=9999).contains(&local.year()) {
                    return Err(Error::Conversion);
                }
                let offset = v.offset();
                Ok(CellValue::TimestampTz(format!(
                    "{}.{:07}{}{:02}:{:02}",
                    local.format("%Y-%m-%dT%H:%M:%S"),
                    local.nanosecond() / 100,
                    if offset < 0 { "-" } else { "+" },
                    offset.unsigned_abs() / 60,
                    offset.unsigned_abs() % 60,
                )))
            })
            .transpose()?,
        _ => return Err(Error::Conversion),
    };
    match result {
        Some(CellValue::Float64(value)) if !value.is_finite() => Err(Error::Conversion),
        Some(value) => Ok(value),
        None if column.metadata.nullable => Ok(CellValue::Null),
        None => Err(Error::Conversion),
    }
}

fn date(value: time::Date) -> Result<NaiveDate, Error> {
    let days = i32::try_from(value.days()).map_err(|_| Error::Conversion)?;
    NaiveDate::from_num_days_from_ce_opt(days + 1)
        .filter(|date| (1..=9999).contains(&date.year()))
        .ok_or(Error::Conversion)
}

fn clock(value: time::Time) -> Result<NaiveTime, Error> {
    if value.scale() > 7 {
        return Err(Error::Conversion);
    }
    let factor = 10u64.pow(u32::from(value.scale()));
    let seconds = u32::try_from(value.increments() / factor).map_err(|_| Error::Conversion)?;
    let nanos =
        u32::try_from((value.increments() % factor) * 10u64.pow(9 - u32::from(value.scale())))
            .map_err(|_| Error::Conversion)?;
    NaiveTime::from_num_seconds_from_midnight_opt(seconds, nanos).ok_or(Error::Conversion)
}

fn datetime(value: time::DateTime2) -> Result<NaiveDateTime, Error> {
    Ok(date(value.date())?.and_time(clock(value.time())?))
}

fn timestamp(value: NaiveDateTime) -> String {
    format!(
        "{}.{:07}Z",
        value.format("%Y-%m-%dT%H:%M:%S"),
        value.nanosecond() / 100
    )
}
