// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Exact bounded PostgreSQL binary codecs. No numeric/JSON float round trips.

use super::{
    adapter::{Error, Result},
    query::{Plan, ResolvedSchema},
};
use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, Timelike, Utc};
use otel_arrow_dfe_scraper::database::{CellValue, CompositeCursor, CursorRow, Row};
use tokio_postgres::types::{FromSql, Type};

const MAX_NUMERIC_CHARS: usize = 16_384;

pub(crate) fn native_type(name: &str) -> Result<Type> {
    Ok(match name {
        "bool" => Type::BOOL,
        "int2" => Type::INT2,
        "int4" => Type::INT4,
        "int8" => Type::INT8,
        "numeric" => Type::NUMERIC,
        "float4" => Type::FLOAT4,
        "float8" => Type::FLOAT8,
        "text" => Type::TEXT,
        "varchar" => Type::VARCHAR,
        "bpchar" => Type::BPCHAR,
        "bytea" => Type::BYTEA,
        "timestamp" => Type::TIMESTAMP,
        "timestamptz" => Type::TIMESTAMPTZ,
        "date" => Type::DATE,
        "uuid" => Type::UUID,
        "json" => Type::JSON,
        "jsonb" => Type::JSONB,
        "interval" => Type::INTERVAL,
        _ => return Err(Error::Metadata),
    })
}

pub(crate) fn cursor_time(text: &str, modifier: i32) -> Result<DateTime<Utc>> {
    let bytes = text.as_bytes();
    if !bytes.iter().enumerate().all(|(i, b)| match i {
        4 | 7 => *b == b'-',
        10 => *b == b'T',
        13 | 16 => *b == b':',
        19 if bytes.len() > 20 => *b == b'.',
        _ if i + 1 == bytes.len() => *b == b'Z',
        _ => b.is_ascii_digit(),
    }) {
        return Err(Error::Value);
    }
    if text.len() < 20
        || text.len() > 27
        || !text.ends_with('Z')
        || text.as_bytes().get(10) != Some(&b'T')
    {
        return Err(Error::Value);
    }
    let dt = DateTime::parse_from_rfc3339(text)
        .map_err(|_| Error::Value)?
        .with_timezone(&Utc);
    validate_timestamp(dt.naive_utc(), modifier)?;
    Ok(dt)
}

pub(crate) fn normalize_cursor_time(text: &str) -> Result<String> {
    if text.len() > 64
        || text.split_once('.').is_some_and(|(_, fraction)| {
            fraction.bytes().take_while(u8::is_ascii_digit).count() > 9
        })
    {
        return Err(Error::Value);
    }
    let timestamp = DateTime::parse_from_rfc3339(text)
        .map(|value| value.with_timezone(&Utc))
        .or_else(|_| {
            NaiveDateTime::parse_from_str(text, "%Y-%m-%d %H:%M:%S%.f").map(|value| value.and_utc())
        })
        .map_err(|_| Error::Value)?;
    validate_timestamp(timestamp.naive_utc(), 6)?;
    timestamp_text(timestamp.naive_utc())
}

fn validate_timestamp(dt: NaiveDateTime, modifier: i32) -> Result<()> {
    let precision = if modifier == -1 { 6 } else { modifier };
    if !(0..=6).contains(&precision)
        || !(1..=9999).contains(&dt.year())
        || dt.nanosecond() >= 1_000_000_000
        || !dt
            .nanosecond()
            .is_multiple_of(10u32.pow(9 - precision as u32))
    {
        return Err(Error::Value);
    }
    Ok(())
}

pub(crate) fn check_tie(value: i64, ty: &str) -> Result<()> {
    let valid = match ty {
        "int2" => i16::try_from(value).is_ok(),
        "int4" => i32::try_from(value).is_ok(),
        "int8" => true,
        _ => false,
    };
    if valid { Ok(()) } else { Err(Error::Value) }
}

struct Raw<'a>(&'a [u8]);
impl<'a> FromSql<'a> for Raw<'a> {
    fn from_sql(
        _: &Type,
        raw: &'a [u8],
    ) -> std::result::Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        Ok(Self(raw))
    }
    fn accepts(_: &Type) -> bool {
        // This private wrapper only borrows bytes. Catalog/prepared-statement
        // validation establishes the types; the cached decoder checks values.
        true
    }
}

fn array<const N: usize>(bytes: &[u8]) -> Result<[u8; N]> {
    bytes.try_into().map_err(|_| Error::Value)
}
fn epoch() -> Result<NaiveDateTime> {
    NaiveDate::from_ymd_opt(2000, 1, 1)
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .ok_or(Error::Value)
}
fn timestamp(bytes: &[u8]) -> Result<NaiveDateTime> {
    timestamp_micros(i64::from_be_bytes(array(bytes)?))
}

fn timestamp_micros(micros: i64) -> Result<NaiveDateTime> {
    if matches!(micros, i64::MIN | i64::MAX) {
        return Err(Error::Value);
    }
    let dt = epoch()?
        .checked_add_signed(chrono::Duration::microseconds(micros))
        .ok_or(Error::Value)?;
    if !(1..=9999).contains(&dt.year()) {
        return Err(Error::Value);
    }
    Ok(dt)
}

fn write_timestamp(text: &mut String, dt: NaiveDateTime) -> Result<()> {
    use std::fmt::Write;
    write!(
        text,
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:06}Z",
        dt.year(),
        dt.month(),
        dt.day(),
        dt.hour(),
        dt.minute(),
        dt.second(),
        dt.nanosecond() / 1000,
    )
    .map_err(|_| Error::Value)
}

fn timestamp_text(dt: NaiveDateTime) -> Result<String> {
    let mut text = String::with_capacity(27);
    write_timestamp(&mut text, dt)?;
    Ok(text)
}

#[derive(Default)]
pub(crate) struct CursorTimestampCache {
    // Sorted rows often share a timestamp. Retain just one native value and its
    // UTC text; include typmod so reuse cannot bypass a stricter precision check.
    last: Option<(i64, i32, NaiveDateTime)>,
    text: String,
}

impl CursorTimestampCache {
    fn decode(&mut self, bytes: &[u8], modifier: i32) -> Result<(NaiveDateTime, &str)> {
        let micros = i64::from_be_bytes(array(bytes)?);
        if let Some((previous, precision, dt)) = &self.last
            && *previous == micros
            && *precision == modifier
        {
            return Ok((*dt, &self.text));
        }
        let dt = timestamp_micros(micros)?;
        validate_timestamp(dt, modifier)?;
        self.text.clear();
        write_timestamp(&mut self.text, dt)?;
        self.last = Some((micros, modifier, dt));
        Ok((dt, &self.text))
    }
}

fn text(bytes: &[u8]) -> Result<&str> {
    std::str::from_utf8(bytes).map_err(|_| Error::Value)
}

/// Appends the leading `take` characters of a zero-padded base-10000 group.
///
/// Rows decode on the per-page hot path, so this avoids the temporary string a
/// width-padded `write!` would allocate for every group of every value.
fn push_group(output: &mut String, digit: u16, take: usize) {
    for position in 0..take.min(4) {
        let power = 10u16.pow(3 - position as u32);
        output.push(char::from(b'0' + (digit / power % 10) as u8));
    }
}

pub(crate) fn numeric(bytes: &[u8]) -> Result<String> {
    // PostgreSQL stores base-10000 groups plus an independent display scale.
    // Render directly to decimal text to preserve large integers and trailing zeros.
    if bytes.len() < 8 {
        return Err(Error::Value);
    }
    let count = i16::from_be_bytes(array(&bytes[..2])?);
    let weight = i16::from_be_bytes(array(&bytes[2..4])?) as i32;
    let sign = u16::from_be_bytes(array(&bytes[4..6])?);
    let scale = u16::from_be_bytes(array(&bytes[6..8])?) as usize;
    if count < 0 || bytes.len() != 8 + count as usize * 2 || !matches!(sign, 0 | 0x4000) {
        return Err(Error::Value);
    }
    let mut digits = Vec::with_capacity(count as usize);
    for chunk in bytes[8..].as_chunks::<2>().0 {
        let digit = u16::from_be_bytes(*chunk);
        if digit >= 10000 {
            return Err(Error::Value);
        }
        digits.push(digit);
    }
    // Digits outside the declared scale must be zero; trimming nonzero digits
    // would silently turn malformed input into a different numeric value.
    for (i, digit) in digits.iter().enumerate() {
        let fractional_end = -(weight - i as i32) * 4;
        if fractional_end > scale as i32 {
            let excess = (fractional_end - scale as i32).min(4) as u32;
            if !(*digit as u32).is_multiple_of(10u32.pow(excess)) {
                return Err(Error::Value);
            }
        }
    }
    let first = digits.iter().position(|d| *d != 0);
    let highest = first.map(|i| weight - i as i32).unwrap_or(-1);
    let integer_len = if highest < 0 {
        1
    } else {
        let d = digits[first.ok_or(Error::Value)?];
        let width = match d {
            1000..=9999 => 4,
            100..=999 => 3,
            10..=99 => 2,
            _ => 1,
        };
        highest as usize * 4 + width
    };
    let length = integer_len + usize::from(scale > 0) + scale + usize::from(sign != 0);
    if length > MAX_NUMERIC_CHARS {
        return Err(Error::Limit);
    }
    let digit_at = |power: i32| -> u16 {
        usize::try_from(weight - power)
            .ok()
            .and_then(|i| digits.get(i))
            .copied()
            .unwrap_or(0)
    };
    use std::fmt::Write;
    let mut output = String::with_capacity(length);
    if sign != 0 {
        output.push('-');
    }
    if highest < 0 {
        output.push('0');
    } else {
        write!(output, "{}", digit_at(highest)).map_err(|_| Error::Value)?;
        for power in (0..highest).rev() {
            push_group(&mut output, digit_at(power), 4);
        }
    }
    if scale > 0 {
        output.push('.');
        for group in 0..scale.div_ceil(4) {
            push_group(
                &mut output,
                digit_at(-(group as i32) - 1),
                scale - group * 4,
            );
        }
    }
    Ok(output)
}

pub(crate) fn decode(ty: &Type, bytes: &[u8]) -> Result<CellValue> {
    Ok(match *ty {
        Type::BOOL => match bytes {
            [0] => CellValue::Bool(false),
            [1] => CellValue::Bool(true),
            _ => return Err(Error::Value),
        },
        Type::INT2 => CellValue::Int64(i16::from_be_bytes(array(bytes)?) as i64),
        Type::INT4 => CellValue::Int64(i32::from_be_bytes(array(bytes)?) as i64),
        Type::INT8 => CellValue::Int64(i64::from_be_bytes(array(bytes)?)),
        Type::NUMERIC => CellValue::Decimal(numeric(bytes)?),
        Type::FLOAT4 | Type::FLOAT8 => {
            let value = if *ty == Type::FLOAT4 {
                f32::from_be_bytes(array(bytes)?) as f64
            } else {
                f64::from_be_bytes(array(bytes)?)
            };
            if !value.is_finite() {
                return Err(Error::Value);
            }
            CellValue::Float64(value)
        }
        Type::TEXT | Type::VARCHAR | Type::BPCHAR => CellValue::String(text(bytes)?.into()),
        Type::BYTEA => CellValue::Bytes(bytes.into()),
        Type::TIMESTAMP => CellValue::Timestamp(timestamp_text(timestamp(bytes)?)?),
        Type::TIMESTAMPTZ => CellValue::TimestampTz(timestamp_text(timestamp(bytes)?)?),
        Type::DATE => {
            let days = i32::from_be_bytes(array(bytes)?);
            let date = epoch()?
                .date()
                .checked_add_signed(chrono::Duration::days(days as i64))
                .ok_or(Error::Value)?;
            if !(1..=9999).contains(&date.year()) {
                return Err(Error::Value);
            }
            // Format directly instead of through a chrono format string, which
            // reparses its pattern for every decoded row.
            use std::fmt::Write;
            let mut text = String::with_capacity(10);
            write!(
                text,
                "{:04}-{:02}-{:02}",
                date.year(),
                date.month(),
                date.day()
            )
            .map_err(|_| Error::Value)?;
            CellValue::String(text)
        }
        Type::UUID => {
            let bytes: [u8; 16] = array(bytes)?;
            const HEX: [u8; 16] = *b"0123456789abcdef";
            let mut value = String::with_capacity(36);
            for (i, byte) in bytes.iter().enumerate() {
                if matches!(i, 4 | 6 | 8 | 10) {
                    value.push('-');
                }
                // Two table lookups per byte instead of a formatter invocation.
                value.push(char::from(HEX[usize::from(byte >> 4)]));
                value.push(char::from(HEX[usize::from(byte & 0x0f)]));
            }
            CellValue::String(value)
        }
        Type::JSON | Type::JSONB => {
            let bytes = if *ty == Type::JSONB {
                if bytes.first() != Some(&1) {
                    return Err(Error::Value);
                }
                &bytes[1..]
            } else {
                bytes
            };
            let text = text(bytes)?;
            // Validate syntax without materializing numbers through a floating-point
            // JSON tree; the original numeric lexemes remain the emitted value.
            let _: &serde_json::value::RawValue =
                serde_json::from_str(text).map_err(|_| Error::Value)?;
            CellValue::String(text.into())
        }
        Type::INTERVAL => {
            if bytes.len() != 16 {
                return Err(Error::Value);
            }
            let micros = i64::from_be_bytes(array(&bytes[..8])?);
            let days = i32::from_be_bytes(array(&bytes[8..12])?);
            let months = i32::from_be_bytes(array(&bytes[12..])?);
            CellValue::Interval(format!("months={months};days={days};microseconds={micros}"))
        }
        _ => return Err(Error::Metadata),
    })
}

pub(crate) type CursorPosition = (NaiveDateTime, i64);

pub(crate) struct DecodedRow {
    pub row: CursorRow,
    // Carry native ordering alongside durable cursor text so the worker need not
    // reparse formatted timestamps for every row's strict-advance check.
    pub position: CursorPosition,
}

pub(crate) fn row(
    native: &tokio_postgres::Row,
    plan: &Plan,
    schema: &ResolvedSchema,
    timestamp_cache: &mut CursorTimestampCache,
    max_raw_bytes: usize,
) -> Result<DecodedRow> {
    // raw_size_bytes measures encoded data, not normalized allocations. Framing
    // bounds receipt; the worker and shared encoder separately admit output bytes.
    if native.len() != schema.columns.len() || native.raw_size_bytes() > max_raw_bytes {
        return Err(Error::Limit);
    }
    let mut values = Vec::with_capacity(native.len());
    let mut cursor_timestamp = None;
    for (i, (col, native_type)) in schema.columns.iter().zip(&schema.native_types).enumerate() {
        let value = match native
            .try_get::<_, Option<Raw<'_>>>(i)
            .map_err(|_| Error::Value)?
        {
            None => {
                if !col.nullable {
                    return Err(Error::Value);
                }
                CellValue::Null
            }
            Some(raw) if i == plan.timestamp => {
                let (dt, text) = timestamp_cache.decode(raw.0, col.type_modifier)?;
                cursor_timestamp = Some(dt);
                match *native_type {
                    Type::TIMESTAMP => CellValue::Timestamp(text.to_owned()),
                    Type::TIMESTAMPTZ => CellValue::TimestampTz(text.to_owned()),
                    _ => return Err(Error::Metadata),
                }
            }
            Some(raw) => decode(native_type, raw.0)?,
        };
        values.push(value);
    }
    let timestamp = match &values[plan.timestamp] {
        CellValue::Timestamp(v) | CellValue::TimestampTz(v) => v.clone(),
        _ => return Err(Error::Value),
    };
    let CellValue::Int64(tie) = values[plan.tie] else {
        return Err(Error::Value);
    };
    Ok(DecodedRow {
        row: CursorRow {
            row: Row { values },
            cursor: CompositeCursor::new(timestamp, tie).into(),
        },
        position: (cursor_timestamp.ok_or(Error::Value)?, tie),
    })
}
