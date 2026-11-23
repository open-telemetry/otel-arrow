// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Masked 4-stage `LogTransformPolicy` execution (`1. Rename -> 2. Add -> 3. Remove -> 4. Redact`)
//! over OTAP `OtapArrowRecords`, including nested CBOR map mutations, `TraceId`/`SpanId`
//! add/remove, and direct flat Arrow builders.

use std::borrow::Cow;
use std::sync::Arc;

use ahash::{AHashMap, AHashSet};
use arrow::array::{
    Array, ArrayBuilder, ArrayRef, BinaryBuilder, BooleanBuilder, DictionaryArray,
    FixedSizeBinaryBuilder, Float64Builder, Int32Array, Int64Builder, NullBufferBuilder,
    RecordBatch, StringArray, StringBuilder, StructArray, UInt8Builder, UInt16Array, UInt16Builder,
};
use arrow::datatypes::{DataType, Field, Schema, UInt8Type, UInt16Type};
use otel_arrow_dfe_pdata::arrays::NullableArrayAccessor;
use otel_arrow_dfe_pdata::otap::OtapArrowRecords;
use otel_arrow_dfe_pdata::otlp::attributes::AttributeValueType;
use otel_arrow_dfe_pdata::otlp::common::AnyValueArrays;
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_dfe_pdata::schema::{FieldExt, consts};
use regex_automata::meta::Regex as MetaRegex;

use crate::bitset::DenseBitset;
use crate::model::{
    AttributePath, LogAdd, LogAttributeSelector, LogRecordField, LogRemove, LogRename,
    LogTargetSelector, PolicyError, ScalarValue, apply_regex_redaction,
};
#[cfg(test)]
use crate::model::{LogTransformPolicy, compile_redact_regex};
use crate::otap_ops::{
    ExtractedValue, MAX_CBOR_DEPTH, ParentIdColumn, decode_cbor_value, extract_anyval,
    extract_fixed_bytes_from_array, extract_i64_from_array, is_valid_single_cbor_item,
    parse_cbor_map_entries,
};

/// Pre-compiled transform policy stage operations ready for global 4-stage execution.
#[derive(Debug, Clone)]
pub struct CompiledTransformPolicy {
    /// Stage 1: Rename operations.
    pub rename: Vec<LogRename>,
    /// Stage 2: Add operations.
    pub add: Vec<LogAdd>,
    /// Stage 3: Remove operations.
    pub remove: Vec<LogRemove>,
    /// Stage 4: Redact operations with pre-compiled optional `MetaRegex`.
    pub redact: Vec<CompiledLogRedact>,
}

/// Pre-compiled `LogRedact` operation.
#[derive(Debug, Clone)]
pub struct CompiledLogRedact {
    /// Target selector.
    pub target: LogTargetSelector,
    /// Effective replacement string (`"[REDACTED]"` if unset).
    pub replacement: String,
    /// Compiled bounded RE2-compatible regex if `pattern.regex` was set.
    pub regex: Option<MetaRegex>,
}

#[cfg(test)]
impl CompiledTransformPolicy {
    /// Compiles a `LogTransformPolicy` with its `policy_id`.
    pub fn compile(_policy_id: String, tx: &LogTransformPolicy) -> Result<Self, PolicyError> {
        let mut redact = Vec::with_capacity(tx.redact.len());
        for r in &tx.redact {
            let compiled_re = match &r.regex {
                Some(pat) => Some(compile_redact_regex(pat)?),
                None => None,
            };
            redact.push(CompiledLogRedact {
                target: r.target.clone(),
                replacement: r.effective_replacement().to_owned(),
                regex: compiled_re,
            });
        }
        Ok(Self {
            rename: tx.rename.clone(),
            add: tx.add.clone(),
            remove: tx.remove.clone(),
            redact,
        })
    }
}

/// Mutable value cell inside a touched attribute table or `logs.body` column.
/// Preserves empty `AnyValue` entries and raw CBOR `ATTRIBUTE_SER` bytes (`Map` / `Slice`)
/// untouched when sibling flat attributes or rows are mutated.
#[derive(Debug, Clone, PartialEq)]
enum CellValue {
    Empty,
    Scalar(ScalarValue),
    RawCbor { vtype: u8, bytes: Vec<u8> },
}

// ============================================================================
// Raw-slice RFC 8949 CBOR encoder & nested map editor
// ============================================================================

fn encode_cbor_head(major: u8, val: u64, out: &mut Vec<u8>) {
    let prefix = major << 5;
    if val <= 23 {
        out.push(prefix | val as u8);
    } else if val <= u64::from(u8::MAX) {
        out.push(prefix | 24);
        out.push(val as u8);
    } else if val <= u64::from(u16::MAX) {
        out.push(prefix | 25);
        out.extend_from_slice(&(val as u16).to_be_bytes());
    } else if val <= u64::from(u32::MAX) {
        out.push(prefix | 26);
        out.extend_from_slice(&(val as u32).to_be_bytes());
    } else {
        out.push(prefix | 27);
        out.extend_from_slice(&val.to_be_bytes());
    }
}

fn encode_cbor_text(s: &str, out: &mut Vec<u8>) {
    encode_cbor_head(3, s.len() as u64, out);
    out.extend_from_slice(s.as_bytes());
}

fn encode_cbor_bytes(b: &[u8], out: &mut Vec<u8>) {
    encode_cbor_head(2, b.len() as u64, out);
    out.extend_from_slice(b);
}

fn encode_cbor_int(i: i64, out: &mut Vec<u8>) {
    if i >= 0 {
        encode_cbor_head(0, i as u64, out);
    } else {
        let mag = (-1i128 - i128::from(i)) as u64;
        encode_cbor_head(1, mag, out);
    }
}

fn encode_cbor_cell(val: &CellValue, out: &mut Vec<u8>) {
    match val {
        CellValue::Empty => out.push(0xf6),
        CellValue::Scalar(ScalarValue::String(s)) => encode_cbor_text(s, out),
        CellValue::Scalar(ScalarValue::Int(i)) => encode_cbor_int(*i, out),
        CellValue::Scalar(ScalarValue::Double(d)) => {
            out.push(0xfb);
            out.extend_from_slice(&d.to_bits().to_be_bytes());
        }
        CellValue::Scalar(ScalarValue::Bool(b)) => out.push(if *b { 0xf5 } else { 0xf4 }),
        CellValue::Scalar(ScalarValue::Bytes(b)) => encode_cbor_bytes(b, out),
        CellValue::RawCbor { bytes, .. } => out.extend_from_slice(bytes),
    }
}

/// Encodes `leaf` wrapped inside nested 1-entry RFC 8949 CBOR maps for every segment in `path`.
fn encode_cbor_nested_map(path: &[String], leaf: &CellValue) -> Vec<u8> {
    let mut out = Vec::with_capacity(path.len() * 16 + 16);
    for seg in path {
        out.push(0xa1); // definite-length map of 1 entry (major 5, ai 1)
        encode_cbor_text(seg, &mut out);
    }
    encode_cbor_cell(leaf, &mut out);
    out
}

/// Decodes a single raw CBOR value slice into a `CellValue`, preserving nested `Map` and `Slice`
/// subtrees as `RawCbor` and CBOR `Null` (`0xf6`) as `CellValue::Empty`.
fn decode_cbor_raw_to_cell(raw: &[u8]) -> Option<CellValue> {
    let &first = raw.first()?;
    let major = first >> 5;
    if major == 5 {
        return is_valid_single_cbor_item(raw).then(|| CellValue::RawCbor {
            vtype: AttributeValueType::Map as u8,
            bytes: raw.to_vec(),
        });
    }
    if major == 4 {
        return is_valid_single_cbor_item(raw).then(|| CellValue::RawCbor {
            vtype: AttributeValueType::Slice as u8,
            bytes: raw.to_vec(),
        });
    }
    if first == 0xf6 && raw.len() == 1 {
        return Some(CellValue::Empty);
    }
    match decode_cbor_value(raw)? {
        ExtractedValue::Str(s) => Some(CellValue::Scalar(ScalarValue::String(s.into_owned()))),
        ExtractedValue::Int(i) => Some(CellValue::Scalar(ScalarValue::Int(i))),
        ExtractedValue::Double(d) => Some(CellValue::Scalar(ScalarValue::Double(d))),
        ExtractedValue::Bool(b) => Some(CellValue::Scalar(ScalarValue::Bool(b))),
        ExtractedValue::Bytes(b) => Some(CellValue::Scalar(ScalarValue::Bytes(b.into_owned()))),
        ExtractedValue::Array(_) | ExtractedValue::Map(_) => None,
    }
}

fn cbor_map_get(bytes: &[u8], path: &[String], depth: usize) -> Option<CellValue> {
    let (seg, rest) = path.split_first()?;
    if depth >= MAX_CBOR_DEPTH {
        return None;
    }
    let entries = parse_cbor_map_entries(bytes)?;
    let (_, _, raw_val) = entries
        .into_iter()
        .find(|(k, _, _)| k.as_deref() == Some(seg.as_str()))?;
    if rest.is_empty() {
        decode_cbor_raw_to_cell(raw_val)
    } else {
        cbor_map_get(raw_val, rest, depth + 1)
    }
}

fn cbor_map_can_write(bytes: &[u8], path: &[String], upsert: bool, depth: usize) -> bool {
    let Some((seg, rest)) = path.split_first() else {
        return false;
    };
    if depth + path.len() > MAX_CBOR_DEPTH {
        return false;
    }
    let Some(entries) = parse_cbor_map_entries(bytes) else {
        return false;
    };
    let found = entries
        .into_iter()
        .find(|(k, _, _)| k.as_deref() == Some(seg.as_str()));
    match (found, rest.is_empty()) {
        (Some(_), true) => upsert,
        (None, true) => true,
        (Some((_, _, raw_val)), false) => cbor_map_can_write(raw_val, rest, upsert, depth + 1),
        (None, false) => true,
    }
}

/// Sets `path` inside a CBOR map, creating missing intermediate maps and copying all untouched
/// sibling keys and values byte-for-byte. Returns `None` if the operation is a fail-open no-op
/// (non-map intermediate, `upsert == false` on existing leaf, or depth exceeded).
fn cbor_map_set(
    bytes: &[u8],
    path: &[String],
    value: &CellValue,
    upsert: bool,
    depth: usize,
) -> Option<Vec<u8>> {
    let (seg, rest) = path.split_first()?;
    if depth + path.len() > MAX_CBOR_DEPTH {
        return None;
    }
    let entries = parse_cbor_map_entries(bytes)?;
    let match_idx = entries
        .iter()
        .position(|(k, _, _)| k.as_deref() == Some(seg.as_str()));

    let (encoded_new_key, new_val): (Option<Vec<u8>>, Vec<u8>) = match (match_idx, rest.is_empty())
    {
        (Some(_), true) => {
            if !upsert {
                return None;
            }
            let mut buf = Vec::new();
            encode_cbor_cell(value, &mut buf);
            (None, buf)
        }
        (None, true) => {
            let mut kbuf = Vec::new();
            encode_cbor_text(seg, &mut kbuf);
            let mut vbuf = Vec::new();
            encode_cbor_cell(value, &mut vbuf);
            (Some(kbuf), vbuf)
        }
        (Some(idx), false) => {
            let child = cbor_map_set(entries[idx].2, rest, value, upsert, depth + 1)?;
            (None, child)
        }
        (None, false) => {
            let mut kbuf = Vec::new();
            encode_cbor_text(seg, &mut kbuf);
            (Some(kbuf), encode_cbor_nested_map(rest, value))
        }
    };

    let mut out = Vec::with_capacity(bytes.len() + new_val.len() + 16);
    if let Some(idx) = match_idx {
        // Count surviving entries (replace matched key at `idx`, drop any duplicate text keys of `seg`).
        let count = entries
            .iter()
            .enumerate()
            .filter(|(i, (k, _, _))| *i == idx || k.as_deref() != Some(seg.as_str()))
            .count();
        encode_cbor_head(5, count as u64, &mut out);
        for (i, (k, raw_k, raw_v)) in entries.iter().enumerate() {
            if i == idx {
                out.extend_from_slice(raw_k);
                out.extend_from_slice(&new_val);
            } else if k.as_deref() != Some(seg.as_str()) {
                out.extend_from_slice(raw_k);
                out.extend_from_slice(raw_v);
            }
        }
    } else {
        let new_key = encoded_new_key?;
        encode_cbor_head(5, (entries.len() + 1) as u64, &mut out);
        for (_, raw_k, raw_v) in &entries {
            out.extend_from_slice(raw_k);
            out.extend_from_slice(raw_v);
        }
        out.extend_from_slice(&new_key);
        out.extend_from_slice(&new_val);
    }
    Some(out)
}

/// Removes `path` from a CBOR map, copying all sibling keys and values byte-for-byte.
/// Returns `Some((removed_value, updated_map_bytes))` if found, or `None` if absent/non-map.
fn cbor_map_remove(bytes: &[u8], path: &[String], depth: usize) -> Option<(CellValue, Vec<u8>)> {
    let (seg, rest) = path.split_first()?;
    if depth >= MAX_CBOR_DEPTH {
        return None;
    }
    let entries = parse_cbor_map_entries(bytes)?;
    let idx = entries
        .iter()
        .position(|(k, _, _)| k.as_deref() == Some(seg.as_str()))?;

    if rest.is_empty() {
        let removed = decode_cbor_raw_to_cell(entries[idx].2)?;
        let count = entries
            .iter()
            .filter(|(k, _, _)| k.as_deref() != Some(seg.as_str()))
            .count();
        let mut out = Vec::with_capacity(bytes.len());
        encode_cbor_head(5, count as u64, &mut out);
        for (k, raw_k, raw_v) in &entries {
            if k.as_deref() != Some(seg.as_str()) {
                out.extend_from_slice(raw_k);
                out.extend_from_slice(raw_v);
            }
        }
        Some((removed, out))
    } else {
        let (removed, new_child) = cbor_map_remove(entries[idx].2, rest, depth + 1)?;
        let count = entries
            .iter()
            .enumerate()
            .filter(|(i, (k, _, _))| *i == idx || k.as_deref() != Some(seg.as_str()))
            .count();
        let mut out = Vec::with_capacity(bytes.len());
        encode_cbor_head(5, count as u64, &mut out);
        for (i, (k, raw_k, raw_v)) in entries.iter().enumerate() {
            if i == idx {
                out.extend_from_slice(raw_k);
                out.extend_from_slice(&new_child);
            } else if k.as_deref() != Some(seg.as_str()) {
                out.extend_from_slice(raw_k);
                out.extend_from_slice(raw_v);
            }
        }
        Some((removed, out))
    }
}

/// Parses a strictly lowercase canonical hex identifier (`32` chars -> `16` bytes for `TraceId`,
/// `16` chars -> `8` bytes for `SpanId`) per `policy.md` ("Add").
fn parse_lowercase_hex_id<const N: usize>(s: &str) -> Option<[u8; N]> {
    if s.len() != N * 2 || !s.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')) {
        return None;
    }
    let mut out = [0u8; N];
    hex::decode_to_slice(s, &mut out).ok()?;
    Some(out)
}

// ============================================================================
// Direct Arrow AnyValue column builders & zero-copy editors
// ============================================================================

struct AnyValBuilders {
    types: UInt8Builder,
    strs: StringBuilder,
    ints: Int64Builder,
    doubles: Float64Builder,
    bools: BooleanBuilder,
    bytes: BinaryBuilder,
    ser: BinaryBuilder,
}

impl AnyValBuilders {
    fn with_capacity(rows: usize) -> Self {
        Self {
            types: UInt8Builder::with_capacity(rows),
            strs: StringBuilder::with_capacity(rows, rows * 8),
            ints: Int64Builder::with_capacity(rows),
            doubles: Float64Builder::with_capacity(rows),
            bools: BooleanBuilder::with_capacity(rows),
            bytes: BinaryBuilder::with_capacity(rows, 0),
            ser: BinaryBuilder::with_capacity(rows, 0),
        }
    }

    #[inline]
    fn append_empty(&mut self) {
        self.types.append_value(AttributeValueType::Empty as u8);
        self.strs.append_null();
        self.ints.append_null();
        self.doubles.append_null();
        self.bools.append_null();
        self.bytes.append_null();
        self.ser.append_null();
    }

    #[inline]
    fn append_str(&mut self, s: &str) {
        self.types.append_value(AttributeValueType::Str as u8);
        self.strs.append_value(s);
        self.ints.append_null();
        self.doubles.append_null();
        self.bools.append_null();
        self.bytes.append_null();
        self.ser.append_null();
    }

    #[inline]
    fn append_int(&mut self, i: i64) {
        self.types.append_value(AttributeValueType::Int as u8);
        self.strs.append_null();
        self.ints.append_value(i);
        self.doubles.append_null();
        self.bools.append_null();
        self.bytes.append_null();
        self.ser.append_null();
    }

    #[inline]
    fn append_double(&mut self, d: f64) {
        self.types.append_value(AttributeValueType::Double as u8);
        self.strs.append_null();
        self.ints.append_null();
        self.doubles.append_value(d);
        self.bools.append_null();
        self.bytes.append_null();
        self.ser.append_null();
    }

    #[inline]
    fn append_bool(&mut self, b: bool) {
        self.types.append_value(AttributeValueType::Bool as u8);
        self.strs.append_null();
        self.ints.append_null();
        self.doubles.append_null();
        self.bools.append_value(b);
        self.bytes.append_null();
        self.ser.append_null();
    }

    #[inline]
    fn append_bytes(&mut self, b: &[u8]) {
        self.types.append_value(AttributeValueType::Bytes as u8);
        self.strs.append_null();
        self.ints.append_null();
        self.doubles.append_null();
        self.bools.append_null();
        self.bytes.append_value(b);
        self.ser.append_null();
    }

    #[inline]
    fn append_cbor(&mut self, vtype: u8, raw: &[u8]) {
        self.types.append_value(vtype);
        self.strs.append_null();
        self.ints.append_null();
        self.doubles.append_null();
        self.bools.append_null();
        self.bytes.append_null();
        self.ser.append_value(raw);
    }

    fn append_cell(&mut self, cell: &CellValue) {
        match cell {
            CellValue::Empty => self.append_empty(),
            CellValue::Scalar(ScalarValue::String(s)) => self.append_str(s),
            CellValue::Scalar(ScalarValue::Int(i)) => self.append_int(*i),
            CellValue::Scalar(ScalarValue::Double(d)) => self.append_double(*d),
            CellValue::Scalar(ScalarValue::Bool(b)) => self.append_bool(*b),
            CellValue::Scalar(ScalarValue::Bytes(b)) => self.append_bytes(b),
            CellValue::RawCbor { vtype, bytes } => self.append_cbor(*vtype, bytes),
        }
    }

    /// Streams an unmodified row directly from borrowed `AnyValueArrays` with zero intermediate
    /// `String` or `Vec<u8>` allocations.
    fn append_from_arrays(&mut self, arrays: &AnyValueArrays<'_>, row: usize) {
        if !arrays.attr_type.is_valid(row) {
            self.append_empty();
            return;
        }
        match AttributeValueType::try_from(arrays.attr_type.value(row)) {
            Ok(AttributeValueType::Str) => {
                let s = arrays
                    .attr_str
                    .as_ref()
                    .and_then(|c| c.str_at(row))
                    .unwrap_or("");
                self.append_str(s);
            }
            Ok(AttributeValueType::Int) => {
                let i = arrays
                    .attr_int
                    .as_ref()
                    .and_then(|c| c.value_at(row))
                    .unwrap_or(0);
                self.append_int(i);
            }
            Ok(AttributeValueType::Double) => {
                let d = arrays
                    .attr_double
                    .and_then(|da| da.is_valid(row).then(|| da.value(row)))
                    .unwrap_or(0.0);
                self.append_double(d);
            }
            Ok(AttributeValueType::Bool) => {
                if let Some(ba) = arrays.attr_bool
                    && ba.is_valid(row)
                {
                    self.append_bool(ba.value(row));
                } else {
                    self.append_empty();
                }
            }
            Ok(AttributeValueType::Bytes) => {
                let b = arrays
                    .attr_bytes
                    .as_ref()
                    .and_then(|c| c.slice_at(row))
                    .unwrap_or(&[]);
                self.append_bytes(b);
            }
            Ok(vtype @ (AttributeValueType::Map | AttributeValueType::Slice)) => {
                if let Some(raw) = arrays.attr_ser.as_ref().and_then(|c| c.slice_at(row)) {
                    self.append_cbor(vtype as u8, raw);
                } else {
                    self.append_empty();
                }
            }
            Ok(AttributeValueType::Empty) | Err(_) => self.append_empty(),
        }
    }

    fn finish_fields_and_arrays(mut self) -> (Vec<Arc<Field>>, Vec<ArrayRef>) {
        let fields = vec![
            Arc::new(Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false)),
            Arc::new(Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true)),
            Arc::new(Field::new(consts::ATTRIBUTE_INT, DataType::Int64, true)),
            Arc::new(Field::new(
                consts::ATTRIBUTE_DOUBLE,
                DataType::Float64,
                true,
            )),
            Arc::new(Field::new(consts::ATTRIBUTE_BOOL, DataType::Boolean, true)),
            Arc::new(Field::new(consts::ATTRIBUTE_BYTES, DataType::Binary, true)),
            Arc::new(Field::new(consts::ATTRIBUTE_SER, DataType::Binary, true)),
        ];
        let arrays: Vec<ArrayRef> = vec![
            Arc::new(self.types.finish()),
            Arc::new(self.strs.finish()),
            Arc::new(self.ints.finish()),
            Arc::new(self.doubles.finish()),
            Arc::new(self.bools.finish()),
            Arc::new(self.bytes.finish()),
            Arc::new(self.ser.finish()),
        ];
        (fields, arrays)
    }
}

/// Materializes an owned `CellValue` on demand from a single row of `AnyValueArrays`.
fn materialize_cell_from_arrays(arrays: &AnyValueArrays<'_>, row: usize) -> Option<CellValue> {
    if !arrays.attr_type.is_valid(row) {
        return None;
    }
    let vtype = AttributeValueType::try_from(arrays.attr_type.value(row)).ok()?;
    match vtype {
        AttributeValueType::Empty => Some(CellValue::Empty),
        AttributeValueType::Map | AttributeValueType::Slice => arrays
            .attr_ser
            .as_ref()
            .and_then(|c| c.slice_at(row))
            .map(|raw| CellValue::RawCbor {
                vtype: vtype as u8,
                bytes: raw.to_vec(),
            }),
        _ => match extract_anyval(arrays, row) {
            Some(ExtractedValue::Str(s)) => {
                Some(CellValue::Scalar(ScalarValue::String(s.into_owned())))
            }
            Some(ExtractedValue::Int(i)) => Some(CellValue::Scalar(ScalarValue::Int(i))),
            Some(ExtractedValue::Double(d)) => Some(CellValue::Scalar(ScalarValue::Double(d))),
            Some(ExtractedValue::Bool(b)) => Some(CellValue::Scalar(ScalarValue::Bool(b))),
            Some(ExtractedValue::Bytes(b)) => {
                Some(CellValue::Scalar(ScalarValue::Bytes(b.into_owned())))
            }
            _ => None,
        },
    }
}

/// Pre-resolved UTF-8 string column accessor (native `StringArray` or `UInt8`/`UInt16` dictionary)
/// checking both key validity and dictionary value validity.
enum StrColAccessor<'a> {
    Native(&'a StringArray),
    Dict8 {
        arr: &'a DictionaryArray<UInt8Type>,
        values: &'a StringArray,
    },
    Dict16 {
        arr: &'a DictionaryArray<UInt16Type>,
        values: &'a StringArray,
    },
}

impl<'a> StrColAccessor<'a> {
    fn try_from_array(col: &'a ArrayRef) -> Option<Self> {
        if let Some(sa) = col.as_any().downcast_ref::<StringArray>() {
            return Some(Self::Native(sa));
        }
        if let Some(arr) = col.as_any().downcast_ref::<DictionaryArray<UInt8Type>>()
            && let Some(values) = arr.values().as_any().downcast_ref::<StringArray>()
        {
            return Some(Self::Dict8 { arr, values });
        }
        if let Some(arr) = col.as_any().downcast_ref::<DictionaryArray<UInt16Type>>()
            && let Some(values) = arr.values().as_any().downcast_ref::<StringArray>()
        {
            return Some(Self::Dict16 { arr, values });
        }
        None
    }

    #[inline]
    fn str_at(&self, row: usize) -> Option<&'a str> {
        match self {
            Self::Native(sa) => sa.is_valid(row).then(|| sa.value(row)),
            Self::Dict8 { arr, values } => {
                if !arr.is_valid(row) {
                    return None;
                }
                let k = arr.keys().value(row) as usize;
                (k < values.len() && values.is_valid(k)).then(|| values.value(k))
            }
            Self::Dict16 { arr, values } => {
                if !arr.is_valid(row) {
                    return None;
                }
                let k = arr.keys().value(row) as usize;
                (k < values.len() && values.is_valid(k)).then(|| values.value(k))
            }
        }
    }

    fn dict_values(&self) -> Option<&'a StringArray> {
        match self {
            Self::Native(_) => None,
            Self::Dict8 { values, .. } | Self::Dict16 { values, .. } => Some(values),
        }
    }

    #[inline]
    fn valid_dict_idx(&self, row: usize) -> Option<usize> {
        match self {
            Self::Native(_) => None,
            Self::Dict8 { arr, values } => {
                if !arr.is_valid(row) {
                    return None;
                }
                let k = arr.keys().value(row) as usize;
                (k < values.len() && values.is_valid(k)).then_some(k)
            }
            Self::Dict16 { arr, values } => {
                if !arr.is_valid(row) {
                    return None;
                }
                let k = arr.keys().value(row) as usize;
                (k < values.len() && values.is_valid(k)).then_some(k)
            }
        }
    }
}

enum OrigBody<'a> {
    Absent,
    Struct {
        sa: &'a StructArray,
        arrays: AnyValueArrays<'a>,
    },
    StrCol(StrColAccessor<'a>),
}

/// Zero-copy sparse overlay editor for `logs.body`, borrowing untouched rows from the input batch
/// and writing directly into flat Arrow builders when rebuilding.
struct BodyEditor<'a> {
    orig: OrigBody<'a>,
    num_rows: usize,
    overrides: AHashMap<usize, Option<CellValue>>,
    dirty: bool,
}

impl<'a> BodyEditor<'a> {
    fn new(root: &'a RecordBatch, num_rows: usize) -> Self {
        let orig = match root.column_by_name(consts::BODY) {
            None => OrigBody::Absent,
            Some(col) => {
                if let Some(sa) = col.as_any().downcast_ref::<StructArray>() {
                    match AnyValueArrays::try_from(sa) {
                        Ok(arrays) => OrigBody::Struct { sa, arrays },
                        Err(_) => OrigBody::Absent,
                    }
                } else if let Some(acc) = StrColAccessor::try_from_array(col) {
                    OrigBody::StrCol(acc)
                } else {
                    OrigBody::Absent
                }
            }
        };
        Self {
            orig,
            num_rows,
            overrides: AHashMap::new(),
            dirty: false,
        }
    }

    fn get(&self, row: usize) -> Option<CellValue> {
        if let Some(ov) = self.overrides.get(&row) {
            return ov.clone();
        }
        match &self.orig {
            OrigBody::Absent => None,
            OrigBody::Struct { sa, arrays } => {
                if !sa.is_valid(row) {
                    return None;
                }
                let cell = materialize_cell_from_arrays(arrays, row)?;
                // Body Empty discriminant is treated as absent for RecordField::Body reads
                // (consistent with `extract_anyval_from_struct` returning `None` for Empty).
                if matches!(cell, CellValue::Empty) {
                    None
                } else {
                    Some(cell)
                }
            }
            OrigBody::StrCol(acc) => acc
                .str_at(row)
                .map(|s| CellValue::Scalar(ScalarValue::String(s.to_owned()))),
        }
    }

    fn is_present(&self, row: usize) -> bool {
        if let Some(ov) = self.overrides.get(&row) {
            return matches!(ov, Some(c) if !matches!(c, CellValue::Empty));
        }
        match &self.orig {
            OrigBody::Absent => false,
            OrigBody::Struct { sa, arrays } => {
                if !sa.is_valid(row) || !arrays.attr_type.is_valid(row) {
                    return false;
                }
                match AttributeValueType::try_from(arrays.attr_type.value(row)) {
                    Ok(
                        AttributeValueType::Str
                        | AttributeValueType::Int
                        | AttributeValueType::Double
                        | AttributeValueType::Bytes,
                    ) => true,
                    Ok(AttributeValueType::Bool) => {
                        arrays.attr_bool.is_some_and(|ba| ba.is_valid(row))
                    }
                    Ok(AttributeValueType::Map | AttributeValueType::Slice) => arrays
                        .attr_ser
                        .as_ref()
                        .and_then(|c| c.slice_at(row))
                        .is_some(),
                    Ok(AttributeValueType::Empty) | Err(_) => false,
                }
            }
            OrigBody::StrCol(acc) => acc.str_at(row).is_some(),
        }
    }

    fn set(&mut self, row: usize, val: CellValue) {
        let _ = self.overrides.insert(row, Some(val));
        self.dirty = true;
    }

    fn remove(&mut self, row: usize) -> Option<CellValue> {
        let prev = self.get(row)?;
        let _ = self.overrides.insert(row, None);
        self.dirty = true;
        Some(prev)
    }

    fn get_body_field(&self, row: usize, path: &[String]) -> Option<CellValue> {
        if let Some(ov) = self.overrides.get(&row) {
            return match ov {
                Some(CellValue::RawCbor { vtype, bytes })
                    if *vtype == AttributeValueType::Map as u8 =>
                {
                    cbor_map_get(bytes, path, 0)
                }
                _ => None,
            };
        }
        match &self.orig {
            OrigBody::Struct { sa, arrays } => {
                if !sa.is_valid(row)
                    || !arrays.attr_type.is_valid(row)
                    || arrays.attr_type.value(row) != AttributeValueType::Map as u8
                {
                    return None;
                }
                let raw = arrays.attr_ser.as_ref()?.slice_at(row)?;
                cbor_map_get(raw, path, 0)
            }
            _ => None,
        }
    }

    fn can_write_body_field(&self, row: usize, path: &[String], upsert: bool) -> bool {
        if path.is_empty() || path.len() > MAX_CBOR_DEPTH {
            return false;
        }
        if let Some(ov) = self.overrides.get(&row) {
            return match ov {
                None | Some(CellValue::Empty) => true,
                Some(CellValue::RawCbor { vtype, bytes })
                    if *vtype == AttributeValueType::Map as u8 =>
                {
                    cbor_map_can_write(bytes, path, upsert, 0)
                }
                _ => false,
            };
        }
        match &self.orig {
            OrigBody::Absent => true,
            OrigBody::Struct { sa, arrays } => {
                if !sa.is_valid(row) || !arrays.attr_type.is_valid(row) {
                    return true;
                }
                match AttributeValueType::try_from(arrays.attr_type.value(row)) {
                    Ok(AttributeValueType::Empty) => true,
                    Ok(AttributeValueType::Map) => arrays
                        .attr_ser
                        .as_ref()
                        .and_then(|c| c.slice_at(row))
                        .is_none_or(|raw| cbor_map_can_write(raw, path, upsert, 0)),
                    _ => false,
                }
            }
            OrigBody::StrCol(acc) => acc.str_at(row).is_none(),
        }
    }

    fn set_body_field(
        &mut self,
        row: usize,
        path: &[String],
        val: &CellValue,
        upsert: bool,
    ) -> bool {
        if path.is_empty() || path.len() > MAX_CBOR_DEPTH {
            return false;
        }
        match self.get(row) {
            None | Some(CellValue::Empty) => {
                let bytes = encode_cbor_nested_map(path, val);
                self.set(
                    row,
                    CellValue::RawCbor {
                        vtype: AttributeValueType::Map as u8,
                        bytes,
                    },
                );
                true
            }
            Some(CellValue::RawCbor { vtype, bytes }) if vtype == AttributeValueType::Map as u8 => {
                let Some(new_bytes) = cbor_map_set(&bytes, path, val, upsert, 0) else {
                    return false;
                };
                self.set(
                    row,
                    CellValue::RawCbor {
                        vtype: AttributeValueType::Map as u8,
                        bytes: new_bytes,
                    },
                );
                true
            }
            _ => false,
        }
    }

    fn remove_body_field(&mut self, row: usize, path: &[String]) -> Option<CellValue> {
        let Some(CellValue::RawCbor { vtype, bytes }) = self.get(row) else {
            return None;
        };
        if vtype != AttributeValueType::Map as u8 {
            return None;
        }
        let (removed, new_bytes) = cbor_map_remove(&bytes, path, 0)?;
        self.set(
            row,
            CellValue::RawCbor {
                vtype: AttributeValueType::Map as u8,
                bytes: new_bytes,
            },
        );
        Some(removed)
    }

    fn any_present(&self) -> bool {
        (0..self.num_rows).any(|r| self.is_present(r))
    }

    fn build_struct_array(&self) -> ArrayRef {
        let mut builders = AnyValBuilders::with_capacity(self.num_rows);
        let mut validity = NullBufferBuilder::new(self.num_rows);

        for r in 0..self.num_rows {
            if let Some(ov) = self.overrides.get(&r) {
                match ov {
                    None => {
                        validity.append_null();
                        builders.append_empty();
                    }
                    Some(cell) => {
                        validity.append_non_null();
                        builders.append_cell(cell);
                    }
                }
                continue;
            }
            match &self.orig {
                OrigBody::Absent => {
                    validity.append_null();
                    builders.append_empty();
                }
                OrigBody::Struct { sa, arrays } => {
                    if !sa.is_valid(r) {
                        validity.append_null();
                        builders.append_empty();
                    } else {
                        let valid_type = arrays
                            .attr_type
                            .is_valid(r)
                            .then(|| AttributeValueType::try_from(arrays.attr_type.value(r)).ok())
                            .flatten();
                        match valid_type {
                            None | Some(AttributeValueType::Empty) => {
                                validity.append_null();
                                builders.append_empty();
                            }
                            Some(AttributeValueType::Map | AttributeValueType::Slice)
                                if arrays
                                    .attr_ser
                                    .as_ref()
                                    .and_then(|c| c.slice_at(r))
                                    .is_none() =>
                            {
                                validity.append_null();
                                builders.append_empty();
                            }
                            Some(AttributeValueType::Bool)
                                if !arrays.attr_bool.is_some_and(|ba| ba.is_valid(r)) =>
                            {
                                validity.append_null();
                                builders.append_empty();
                            }
                            Some(_) => {
                                validity.append_non_null();
                                builders.append_from_arrays(arrays, r);
                            }
                        }
                    }
                }
                OrigBody::StrCol(acc) => {
                    if let Some(s) = acc.str_at(r) {
                        validity.append_non_null();
                        builders.append_str(s);
                    } else {
                        validity.append_null();
                        builders.append_empty();
                    }
                }
            }
        }

        let (fields, arrays) = builders.finish_fields_and_arrays();
        Arc::new(StructArray::new(fields.into(), arrays, validity.finish()))
    }
}

/// Zero-copy sparse overlay editor for root UTF-8 string fields (`severity_text`, `event_name`).
struct RootStrEditor<'a> {
    orig: Option<StrColAccessor<'a>>,
    num_rows: usize,
    overrides: AHashMap<usize, Option<String>>,
    dirty: bool,
}

impl<'a> RootStrEditor<'a> {
    fn new(root: &'a RecordBatch, col_name: &str, num_rows: usize) -> Self {
        let orig = root
            .column_by_name(col_name)
            .and_then(StrColAccessor::try_from_array);
        Self {
            orig,
            num_rows,
            overrides: AHashMap::new(),
            dirty: false,
        }
    }

    fn get(&self, row: usize) -> Option<String> {
        if let Some(ov) = self.overrides.get(&row) {
            return ov.clone();
        }
        let s = self.orig.as_ref()?.str_at(row)?;
        (!s.is_empty()).then(|| s.to_owned())
    }

    fn is_present(&self, row: usize) -> bool {
        if let Some(ov) = self.overrides.get(&row) {
            return ov.is_some();
        }
        self.orig
            .as_ref()
            .and_then(|acc| acc.str_at(row))
            .is_some_and(|s| !s.is_empty())
    }

    fn set(&mut self, row: usize, s: String) {
        let normalized = (!s.is_empty()).then_some(s);
        let _ = self.overrides.insert(row, normalized);
        self.dirty = true;
    }

    fn remove(&mut self, row: usize) -> Option<String> {
        let prev = self.get(row)?;
        let _ = self.overrides.insert(row, None);
        self.dirty = true;
        Some(prev)
    }

    fn any_present(&self) -> bool {
        (0..self.num_rows).any(|r| self.is_present(r))
    }

    fn build_array(&self) -> ArrayRef {
        let mut builder = StringBuilder::with_capacity(self.num_rows, self.num_rows * 8);
        for r in 0..self.num_rows {
            if let Some(ov) = self.overrides.get(&r) {
                match ov {
                    Some(s) => builder.append_value(s),
                    None => builder.append_null(),
                }
            } else if let Some(s) = self.orig.as_ref().and_then(|a| a.str_at(r))
                && !s.is_empty()
            {
                builder.append_value(s);
            } else {
                builder.append_null();
            }
        }
        Arc::new(builder.finish())
    }
}

/// Entry stored in `FlatAttrsEditor`'s overlay for a touched `(parent_id, key)` pair.
enum AttrEntry {
    /// Points to an existing unmodified row in `orig_rb` (zero value cloning).
    /// When `moved == false`, the row is still emitted in-place during the `orig_rb` pass.
    /// When `moved == true`, `row` was marked superseded (flat-renamed to a new key) and is
    /// streamed from the overlay under its new key.
    OrigRow { row: usize, moved: bool },
    /// Newly added or mutated value.
    Owned(CellValue),
}

struct OrigAttrs<'a> {
    rb: &'a RecordBatch,
    parent_ids: ParentIdColumn<'a>,
    keys: StrColAccessor<'a>,
    arrays: Option<AnyValueArrays<'a>>,
}

/// Direct flat Arrow builder & sparse overlay editor for `LogAttrs`.
///
/// Indexes only `(parent_id, key)` pairs whose `parent_id` belongs to a matched transform row AND
/// whose top-level `key` appears in `touched_keys`. All untouched attributes stream directly from
/// the input `RecordBatch` into Arrow column builders with zero intermediate `BTreeMap` trees or
/// heap `String`/`Vec<u8>` allocations.
struct FlatAttrsEditor<'a> {
    orig: Option<OrigAttrs<'a>>,
    orig_superseded: DenseBitset,
    overlay: AHashMap<(u16, Cow<'a, str>), Option<AttrEntry>>,
    dirty: bool,
}

impl<'a> FlatAttrsEditor<'a> {
    fn new(
        rb_opt: Option<&'a RecordBatch>,
        touched_pids: &DenseBitset,
        touched_keys: &AHashSet<&str>,
    ) -> Self {
        let Some(rb) = rb_opt else {
            return Self {
                orig: None,
                orig_superseded: DenseBitset::new(),
                overlay: AHashMap::new(),
                dirty: false,
            };
        };
        let Some(key_col) = rb.column_by_name(consts::ATTRIBUTE_KEY) else {
            return Self {
                orig: None,
                orig_superseded: DenseBitset::new(),
                overlay: AHashMap::new(),
                dirty: false,
            };
        };
        let Some(keys) = StrColAccessor::try_from_array(key_col) else {
            return Self {
                orig: None,
                orig_superseded: DenseBitset::new(),
                overlay: AHashMap::new(),
                dirty: false,
            };
        };
        let Some(parent_ids) = ParentIdColumn::from_batch(rb) else {
            return Self {
                orig: None,
                orig_superseded: DenseBitset::new(),
                overlay: AHashMap::new(),
                dirty: false,
            };
        };
        let arrays = AnyValueArrays::try_from(rb).ok();

        let mut orig_superseded = DenseBitset::new();
        let mut overlay: AHashMap<(u16, Cow<'a, str>), Option<AttrEntry>> = AHashMap::new();

        if !touched_pids.is_empty() && !touched_keys.is_empty() {
            // Fast dictionary pre-filter: if key_col is dictionary-encoded, check touched_keys
            // once per dictionary entry instead of hashing every attribute row.
            let dict_watched: Option<Vec<bool>> = keys.dict_values().map(|vals| {
                (0..vals.len())
                    .map(|i| vals.is_valid(i) && touched_keys.contains(vals.value(i)))
                    .collect()
            });
            let any_watched = dict_watched
                .as_ref()
                .is_none_or(|v| v.iter().any(|&hit| hit));

            if any_watched {
                for r in 0..rb.num_rows() {
                    let k: &'a str = if let Some(ref watched) = dict_watched {
                        let Some(di) = keys.valid_dict_idx(r) else {
                            continue;
                        };
                        if !watched[di] {
                            continue;
                        }
                        keys.dict_values().expect("dict_values").value(di)
                    } else {
                        let Some(s) = keys.str_at(r) else {
                            continue;
                        };
                        if !touched_keys.contains(s) {
                            continue;
                        }
                        s
                    };
                    let Some(pid) = parent_ids.value_at(r) else {
                        continue;
                    };
                    if !touched_pids.contains(u32::from(pid)) {
                        continue;
                    }
                    let key_pair = (pid, Cow::Borrowed(k));
                    if let Some(Some(AttrEntry::OrigRow { row: prev_r, .. })) = overlay.insert(
                        key_pair,
                        Some(AttrEntry::OrigRow {
                            row: r,
                            moved: false,
                        }),
                    ) {
                        // Duplicate key for the same parent_id: supersede earlier row (last-write-wins).
                        orig_superseded.insert(prev_r as u32);
                    }
                }
            }
        }

        Self {
            orig: Some(OrigAttrs {
                rb,
                parent_ids,
                keys,
                arrays,
            }),
            orig_superseded,
            overlay,
            dirty: false,
        }
    }

    /// Populates `used` with all `parent_id` values currently present in `orig` or `overlay` so
    /// newly allocated `log.id` values never collide with existing attribute rows.
    fn collect_used_parent_ids(&self, used: &mut DenseBitset) {
        if let Some(ref orig) = self.orig {
            for r in 0..orig.rb.num_rows() {
                if let Some(pid) = orig.parent_ids.value_at(r) {
                    used.insert(u32::from(pid));
                }
            }
        }
        for &(pid, _) in self.overlay.keys() {
            used.insert(u32::from(pid));
        }
    }

    fn contains_flat(&self, pid: u16, key: &str) -> bool {
        matches!(self.overlay.get(&(pid, Cow::Borrowed(key))), Some(Some(_)))
    }

    fn materialize_entry(&self, entry: &AttrEntry) -> CellValue {
        match entry {
            AttrEntry::Owned(v) => v.clone(),
            AttrEntry::OrigRow { row, .. } => self
                .orig
                .as_ref()
                .and_then(|o| o.arrays.as_ref())
                .and_then(|arrays| materialize_cell_from_arrays(arrays, *row))
                .unwrap_or(CellValue::Empty),
        }
    }

    fn get_flat(&self, pid: u16, key: &str) -> Option<CellValue> {
        let entry = self.overlay.get(&(pid, Cow::Borrowed(key)))?.as_ref()?;
        Some(self.materialize_entry(entry))
    }

    fn set_flat(&mut self, pid: u16, key: String, val: CellValue) {
        let k = (pid, Cow::Owned(key));
        if let Some(Some(AttrEntry::OrigRow { row, moved: false })) = self.overlay.get(&k) {
            self.orig_superseded.insert(*row as u32);
        }
        let _ = self.overlay.insert(k, Some(AttrEntry::Owned(val)));
        self.dirty = true;
    }

    fn remove_flat(&mut self, pid: u16, key: &str) -> Option<CellValue> {
        let k = (pid, Cow::Owned(key.to_owned()));
        let prev = self.overlay.get_mut(&k)?.take()?;
        if let AttrEntry::OrigRow { row, moved: false } = prev {
            self.orig_superseded.insert(row as u32);
        }
        self.dirty = true;
        Some(self.materialize_entry(&prev))
    }

    /// Zero-copy flat-to-flat rename: preserves unmodified `OrigRow` values without cloning strings
    /// or CBOR buffers.
    fn rename_flat(&mut self, pid: u16, from_key: &str, to_key: &str, upsert: bool) {
        if from_key == to_key || !self.contains_flat(pid, from_key) {
            return;
        }
        if !upsert && self.contains_flat(pid, to_key) {
            return;
        }
        let fk = (pid, Cow::Owned(from_key.to_owned()));
        let Some(Some(src_entry)) = self.overlay.get_mut(&fk).map(Option::take) else {
            return;
        };
        let moved_entry = match src_entry {
            AttrEntry::OrigRow { row, .. } => {
                self.orig_superseded.insert(row as u32);
                AttrEntry::OrigRow { row, moved: true }
            }
            other => other,
        };
        let tk = (pid, Cow::Owned(to_key.to_owned()));
        if let Some(Some(AttrEntry::OrigRow { row, moved: false })) = self.overlay.get(&tk) {
            self.orig_superseded.insert(*row as u32);
        }
        let _ = self.overlay.insert(tk, Some(moved_entry));
        self.dirty = true;
    }

    fn get_path(&self, pid: u16, path: &[String]) -> Option<CellValue> {
        let (head, tail) = path.split_first()?;
        let top = self.get_flat(pid, head)?;
        if tail.is_empty() {
            return Some(top);
        }
        match top {
            CellValue::RawCbor { vtype, bytes } if vtype == AttributeValueType::Map as u8 => {
                cbor_map_get(&bytes, tail, 0)
            }
            _ => None,
        }
    }

    fn can_write_path(&self, pid: u16, path: &[String], upsert: bool) -> bool {
        let Some((head, tail)) = path.split_first() else {
            return false;
        };
        if tail.is_empty() {
            return upsert || !self.contains_flat(pid, head);
        }
        if tail.len() > MAX_CBOR_DEPTH {
            return false;
        }
        match self.get_flat(pid, head) {
            None => true,
            Some(CellValue::RawCbor { vtype, bytes }) if vtype == AttributeValueType::Map as u8 => {
                cbor_map_can_write(&bytes, tail, upsert, 0)
            }
            _ => false,
        }
    }

    fn set_path(&mut self, pid: u16, path: &[String], val: CellValue, upsert: bool) -> bool {
        let Some((head, tail)) = path.split_first() else {
            return false;
        };
        if tail.is_empty() {
            if !upsert && self.contains_flat(pid, head) {
                return false;
            }
            self.set_flat(pid, head.clone(), val);
            return true;
        }
        if tail.len() > MAX_CBOR_DEPTH {
            return false;
        }
        match self.get_flat(pid, head) {
            None => {
                let bytes = encode_cbor_nested_map(tail, &val);
                self.set_flat(
                    pid,
                    head.clone(),
                    CellValue::RawCbor {
                        vtype: AttributeValueType::Map as u8,
                        bytes,
                    },
                );
                true
            }
            Some(CellValue::RawCbor { vtype, bytes }) if vtype == AttributeValueType::Map as u8 => {
                let Some(new_bytes) = cbor_map_set(&bytes, tail, &val, upsert, 0) else {
                    return false;
                };
                self.set_flat(
                    pid,
                    head.clone(),
                    CellValue::RawCbor {
                        vtype: AttributeValueType::Map as u8,
                        bytes: new_bytes,
                    },
                );
                true
            }
            _ => false,
        }
    }

    fn remove_path(&mut self, pid: u16, path: &[String]) -> Option<CellValue> {
        let (head, tail) = path.split_first()?;
        if tail.is_empty() {
            return self.remove_flat(pid, head);
        }
        let Some(CellValue::RawCbor { vtype, bytes }) = self.get_flat(pid, head) else {
            return None;
        };
        if vtype != AttributeValueType::Map as u8 {
            return None;
        }
        let (removed, new_bytes) = cbor_map_remove(&bytes, tail, 0)?;
        self.set_flat(
            pid,
            head.clone(),
            CellValue::RawCbor {
                vtype: AttributeValueType::Map as u8,
                bytes: new_bytes,
            },
        );
        Some(removed)
    }

    /// Streams surviving untouched original rows and mutated/moved overlay entries directly into
    /// flat Arrow builders. Returns `Ok(None)` when every attribute was removed (so the caller
    /// removes `LogAttrs` instead of attaching an empty 0-row batch).
    fn finish_record_batch(self) -> Result<Option<RecordBatch>, PolicyError> {
        let mut dirty_overlay: Vec<(u16, &str, &AttrEntry)> = self
            .overlay
            .iter()
            .filter_map(|((pid, key), opt)| match opt {
                Some(entry @ (AttrEntry::Owned(_) | AttrEntry::OrigRow { moved: true, .. })) => {
                    Some((*pid, key.as_ref(), entry))
                }
                _ => None,
            })
            .collect();
        dirty_overlay.sort_unstable_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(b.1)));

        let orig_rows = self.orig.as_ref().map_or(0, |o| o.rb.num_rows());
        let est_rows = orig_rows
            .saturating_sub(self.orig_superseded.len() as usize)
            .saturating_add(dirty_overlay.len());

        let mut pid_builder = UInt16Builder::with_capacity(est_rows);
        let mut key_builder = StringBuilder::with_capacity(est_rows, est_rows * 12);
        let mut val_builders = AnyValBuilders::with_capacity(est_rows);

        if let Some(ref orig) = self.orig {
            for r in 0..orig.rb.num_rows() {
                if self.orig_superseded.contains(r as u32) {
                    continue;
                }
                let Some(pid) = orig.parent_ids.value_at(r) else {
                    continue;
                };
                let Some(k) = orig.keys.str_at(r) else {
                    continue;
                };
                pid_builder.append_value(pid);
                key_builder.append_value(k);
                if let Some(ref arrays) = orig.arrays {
                    val_builders.append_from_arrays(arrays, r);
                } else {
                    val_builders.append_empty();
                }
            }
        }

        for (pid, key, entry) in dirty_overlay {
            pid_builder.append_value(pid);
            key_builder.append_value(key);
            match entry {
                AttrEntry::Owned(cell) => val_builders.append_cell(cell),
                AttrEntry::OrigRow { row, .. } => {
                    if let Some(ref orig) = self.orig
                        && let Some(ref arrays) = orig.arrays
                    {
                        val_builders.append_from_arrays(arrays, *row);
                    } else {
                        val_builders.append_empty();
                    }
                }
            }
        }

        if pid_builder.is_empty() {
            return Ok(None);
        }

        let (val_fields, val_arrays) = val_builders.finish_fields_and_arrays();
        let mut fields = Vec::with_capacity(2 + val_fields.len());
        fields.push(Arc::new(
            Field::new(consts::PARENT_ID, DataType::UInt16, false).with_plain_encoding(),
        ));
        fields.push(Arc::new(Field::new(
            consts::ATTRIBUTE_KEY,
            DataType::Utf8,
            false,
        )));
        fields.extend(val_fields);

        let mut arrays: Vec<ArrayRef> = Vec::with_capacity(2 + val_arrays.len());
        arrays.push(Arc::new(pid_builder.finish()));
        arrays.push(Arc::new(key_builder.finish()));
        arrays.extend(val_arrays);

        let rb = RecordBatch::try_new(Arc::new(Schema::new(fields)), arrays)
            .map_err(|e| PolicyError::Evaluation(e.to_string()))?;
        Ok(Some(rb))
    }
}

/// Executes all matching `CompiledTransformPolicy` stages in official `opentelemetry-proto#865`
/// order (`1. Rename -> 2. Add -> 3. Remove -> 4. Redact`) directly on `OtapArrowRecords` without
/// splitting and re-concatenating batches via `concatenate::concatenate`.
///
/// `policy_matches` must be aligned with `transforms` (which are pre-sorted by ascending
/// `policy_id`).
pub fn apply_spec_transforms_masked(
    batch: &mut OtapArrowRecords,
    transforms: &[CompiledTransformPolicy],
    policy_matches: &[DenseBitset],
) -> Result<(), PolicyError> {
    if transforms.is_empty() || policy_matches.iter().all(DenseBitset::is_empty) {
        return Ok(());
    }
    let Some(root) = batch.get(ArrowPayloadType::Logs).cloned() else {
        return Ok(());
    };
    let num_rows = root.num_rows();
    if num_rows == 0 {
        return Ok(());
    }

    // Identify which root columns and which LogAttrs keys/rows are touched by active policies.
    let mut touches_root_body = false;
    let mut touches_sev_text = false;
    let mut touches_sev_num = false;
    let mut touches_event_name = false;
    let mut touches_trace_id = false;
    let mut touches_span_id = false;
    let mut touches_log_attrs = false;
    let mut touched_attr_keys: AHashSet<&str> = AHashSet::new();
    let mut touched_attr_rows = DenseBitset::new();

    for (tx, mask) in transforms.iter().zip(policy_matches.iter()) {
        if mask.is_empty() {
            continue;
        }
        let mut policy_touches_attrs = false;
        for op in &tx.rename {
            for sel in [&op.from, &op.to] {
                match sel {
                    LogAttributeSelector::LogAttribute(p) => {
                        if let Some(first) = p.path.first() {
                            let _ = touched_attr_keys.insert(first.as_str());
                            policy_touches_attrs = true;
                        }
                    }
                    LogAttributeSelector::BodyField(_) => touches_root_body = true,
                }
            }
        }
        for (target, is_redact) in tx
            .add
            .iter()
            .map(|op| (&op.target, false))
            .chain(tx.remove.iter().map(|op| (&op.target, false)))
            .chain(tx.redact.iter().map(|op| (&op.target, true)))
        {
            match target {
                LogTargetSelector::RecordField(LogRecordField::Body) => touches_root_body = true,
                LogTargetSelector::RecordField(LogRecordField::SeverityText) => {
                    touches_sev_text = true;
                }
                LogTargetSelector::RecordField(LogRecordField::SeverityNumber) if !is_redact => {
                    touches_sev_num = true;
                }
                LogTargetSelector::RecordField(LogRecordField::EventName) => {
                    touches_event_name = true;
                }
                LogTargetSelector::RecordField(LogRecordField::TraceId) if !is_redact => {
                    touches_trace_id = true;
                }
                LogTargetSelector::RecordField(LogRecordField::SpanId) if !is_redact => {
                    touches_span_id = true;
                }
                LogTargetSelector::RecordField(_) => {}
                LogTargetSelector::LogAttribute(p) => {
                    if let Some(first) = p.path.first() {
                        let _ = touched_attr_keys.insert(first.as_str());
                        policy_touches_attrs = true;
                    }
                }
                LogTargetSelector::BodyField(_) => touches_root_body = true,
            }
        }
        if policy_touches_attrs {
            touches_log_attrs = true;
            touched_attr_rows.or_assign(mask);
        }
    }

    let mut body_editor = if touches_root_body {
        Some(BodyEditor::new(&root, num_rows))
    } else {
        None
    };
    let mut sev_text_editor = if touches_sev_text {
        Some(RootStrEditor::new(&root, consts::SEVERITY_TEXT, num_rows))
    } else {
        None
    };
    let mut sev_num_vals: Vec<Option<i32>> = if touches_sev_num {
        extract_root_i32_vec(&root, consts::SEVERITY_NUMBER, num_rows)
    } else {
        Vec::new()
    };
    let mut event_name_editor = if touches_event_name {
        Some(RootStrEditor::new(&root, consts::EVENT_NAME, num_rows))
    } else {
        None
    };
    let mut trace_id_vals: Vec<Option<[u8; 16]>> = if touches_trace_id {
        extract_root_fixed_id_vec::<16>(&root, consts::TRACE_ID, num_rows)
    } else {
        Vec::new()
    };
    let mut span_id_vals: Vec<Option<[u8; 8]>> = if touches_span_id {
        extract_root_fixed_id_vec::<8>(&root, consts::SPAN_ID, num_rows)
    } else {
        Vec::new()
    };

    // Read existing log ID column directly without allocating a Vec copy up front.
    let existing_log_ids: Option<&UInt16Array> = root
        .column_by_name(consts::ID)
        .and_then(|c| c.as_any().downcast_ref::<UInt16Array>());

    let mut touched_pids = DenseBitset::new();
    if touches_log_attrs && let Some(id_arr) = existing_log_ids {
        for row_u32 in touched_attr_rows.iter() {
            let r = row_u32 as usize;
            if r < id_arr.len() && id_arr.is_valid(r) {
                touched_pids.insert(u32::from(id_arr.value(r)));
            }
        }
    }

    let orig_log_attrs = if touches_log_attrs {
        batch.get(ArrowPayloadType::LogAttrs).cloned()
    } else {
        None
    };
    let mut attrs_editor = if touches_log_attrs {
        Some(FlatAttrsEditor::new(
            orig_log_attrs.as_ref(),
            &touched_pids,
            &touched_attr_keys,
        ))
    } else {
        None
    };

    // Lazily materialized only when a transform writes a LogAttribute to a row that had no log.id.
    let mut mut_log_ids: Option<Vec<Option<u16>>> = None;
    let mut used_log_ids: Option<DenseBitset> = None;
    let mut next_log_id_candidate: u32 = 0;

    let mut sev_num_dirty = false;
    let mut trace_id_dirty = false;
    let mut span_id_dirty = false;

    macro_rules! get_log_id {
        ($row:expr) => {{
            let r: usize = $row;
            if let Some(ref ids) = mut_log_ids {
                ids[r]
            } else {
                existing_log_ids.and_then(|a| a.is_valid(r).then(|| a.value(r)))
            }
        }};
    }

    macro_rules! get_or_alloc_log_id {
        ($row:expr) => {{
            let r: usize = $row;
            if let Some(id) = get_log_id!(r) {
                Some(id)
            } else {
                let ids = mut_log_ids.get_or_insert_with(|| {
                    if let Some(arr) = existing_log_ids {
                        (0..num_rows)
                            .map(|i| arr.is_valid(i).then(|| arr.value(i)))
                            .collect()
                    } else {
                        vec![None; num_rows]
                    }
                });
                let used = used_log_ids.get_or_insert_with(|| {
                    let mut bm = DenseBitset::new();
                    if let Some(arr) = existing_log_ids {
                        for i in 0..num_rows {
                            if arr.is_valid(i) {
                                bm.insert(u32::from(arr.value(i)));
                            }
                        }
                    }
                    if let Some(ref editor) = attrs_editor {
                        editor.collect_used_parent_ids(&mut bm);
                    }
                    bm
                });
                while next_log_id_candidate <= u32::from(u16::MAX)
                    && used.contains(next_log_id_candidate)
                {
                    next_log_id_candidate += 1;
                }
                if next_log_id_candidate <= u32::from(u16::MAX) {
                    let new_id = next_log_id_candidate as u16;
                    used.insert(next_log_id_candidate);
                    next_log_id_candidate += 1;
                    ids[r] = Some(new_id);
                    Some(new_id)
                } else {
                    None
                }
            }
        }};
    }

    macro_rules! get_target {
        ($row:expr, $target:expr) => {{
            let r: usize = $row;
            match $target {
                LogTargetSelector::RecordField(LogRecordField::Body) => {
                    body_editor.as_ref().and_then(|e| e.get(r))
                }
                LogTargetSelector::RecordField(LogRecordField::SeverityText) => sev_text_editor
                    .as_ref()
                    .and_then(|e| e.get(r))
                    .map(|s| CellValue::Scalar(ScalarValue::String(s))),
                LogTargetSelector::RecordField(LogRecordField::SeverityNumber) => {
                    sev_num_vals[r].map(|v| CellValue::Scalar(ScalarValue::Int(i64::from(v))))
                }
                LogTargetSelector::RecordField(LogRecordField::EventName) => event_name_editor
                    .as_ref()
                    .and_then(|e| e.get(r))
                    .map(|s| CellValue::Scalar(ScalarValue::String(s))),
                LogTargetSelector::RecordField(LogRecordField::TraceId) => {
                    trace_id_vals[r].map(|b| CellValue::Scalar(ScalarValue::String(hex::encode(b))))
                }
                LogTargetSelector::RecordField(LogRecordField::SpanId) => {
                    span_id_vals[r].map(|b| CellValue::Scalar(ScalarValue::String(hex::encode(b))))
                }
                LogTargetSelector::RecordField(LogRecordField::Unspecified) => None,
                LogTargetSelector::LogAttribute(AttributePath { path }) => get_log_id!(r)
                    .and_then(|pid| attrs_editor.as_ref().and_then(|ed| ed.get_path(pid, path))),
                LogTargetSelector::BodyField(AttributePath { path }) => {
                    body_editor.as_ref().and_then(|e| e.get_body_field(r, path))
                }
            }
        }};
    }

    macro_rules! remove_target {
        ($row:expr, $target:expr) => {{
            let r: usize = $row;
            match $target {
                LogTargetSelector::RecordField(LogRecordField::Body) => {
                    body_editor.as_mut().and_then(|e| e.remove(r))
                }
                LogTargetSelector::RecordField(LogRecordField::SeverityText) => sev_text_editor
                    .as_mut()
                    .and_then(|e| e.remove(r))
                    .map(|s| CellValue::Scalar(ScalarValue::String(s))),
                LogTargetSelector::RecordField(LogRecordField::SeverityNumber) => {
                    let removed = sev_num_vals[r]
                        .take()
                        .map(|v| CellValue::Scalar(ScalarValue::Int(i64::from(v))));
                    if removed.is_some() {
                        sev_num_dirty = true;
                    }
                    removed
                }
                LogTargetSelector::RecordField(LogRecordField::EventName) => event_name_editor
                    .as_mut()
                    .and_then(|e| e.remove(r))
                    .map(|s| CellValue::Scalar(ScalarValue::String(s))),
                LogTargetSelector::RecordField(LogRecordField::TraceId) => {
                    let removed = trace_id_vals[r]
                        .take()
                        .map(|b| CellValue::Scalar(ScalarValue::String(hex::encode(b))));
                    if removed.is_some() {
                        trace_id_dirty = true;
                    }
                    removed
                }
                LogTargetSelector::RecordField(LogRecordField::SpanId) => {
                    let removed = span_id_vals[r]
                        .take()
                        .map(|b| CellValue::Scalar(ScalarValue::String(hex::encode(b))));
                    if removed.is_some() {
                        span_id_dirty = true;
                    }
                    removed
                }
                LogTargetSelector::RecordField(LogRecordField::Unspecified) => None,
                LogTargetSelector::LogAttribute(AttributePath { path }) => {
                    get_log_id!(r).and_then(|pid| {
                        attrs_editor
                            .as_mut()
                            .and_then(|ed| ed.remove_path(pid, path))
                    })
                }
                LogTargetSelector::BodyField(AttributePath { path }) => body_editor
                    .as_mut()
                    .and_then(|e| e.remove_body_field(r, path)),
            }
        }};
    }

    macro_rules! set_target {
        ($row:expr, $target:expr, $val:expr, $upsert:expr) => {{
            let r: usize = $row;
            let v: CellValue = $val;
            let upsert: bool = $upsert;
            match $target {
                LogTargetSelector::RecordField(LogRecordField::Body) => {
                    if let Some(ref mut e) = body_editor {
                        if !upsert && e.is_present(r) {
                            false
                        } else {
                            e.set(r, v);
                            true
                        }
                    } else {
                        false
                    }
                }
                LogTargetSelector::RecordField(LogRecordField::SeverityText) => {
                    if let (Some(e), CellValue::Scalar(ScalarValue::String(s))) =
                        (&mut sev_text_editor, v)
                    {
                        if !upsert && e.is_present(r) {
                            false
                        } else {
                            e.set(r, s);
                            true
                        }
                    } else {
                        false
                    }
                }
                LogTargetSelector::RecordField(LogRecordField::SeverityNumber) => {
                    if !upsert && sev_num_vals[r].is_some() {
                        false
                    } else if let CellValue::Scalar(ScalarValue::Int(i)) = v
                        && let Ok(i32_val) = i32::try_from(i)
                    {
                        sev_num_vals[r] = (i32_val != 0).then_some(i32_val);
                        sev_num_dirty = true;
                        true
                    } else {
                        false
                    }
                }
                LogTargetSelector::RecordField(LogRecordField::EventName) => {
                    if let (Some(e), CellValue::Scalar(ScalarValue::String(s))) =
                        (&mut event_name_editor, v)
                    {
                        if !upsert && e.is_present(r) {
                            false
                        } else {
                            e.set(r, s);
                            true
                        }
                    } else {
                        false
                    }
                }
                LogTargetSelector::RecordField(LogRecordField::TraceId) => {
                    if !upsert && trace_id_vals[r].is_some() {
                        false
                    } else if let CellValue::Scalar(ScalarValue::String(s)) = v
                        && let Some(bytes) = parse_lowercase_hex_id::<16>(&s)
                    {
                        trace_id_vals[r] = (!bytes.iter().all(|&b| b == 0)).then_some(bytes);
                        trace_id_dirty = true;
                        true
                    } else {
                        false
                    }
                }
                LogTargetSelector::RecordField(LogRecordField::SpanId) => {
                    if !upsert && span_id_vals[r].is_some() {
                        false
                    } else if let CellValue::Scalar(ScalarValue::String(s)) = v
                        && let Some(bytes) = parse_lowercase_hex_id::<8>(&s)
                    {
                        span_id_vals[r] = (!bytes.iter().all(|&b| b == 0)).then_some(bytes);
                        span_id_dirty = true;
                        true
                    } else {
                        false
                    }
                }
                LogTargetSelector::RecordField(LogRecordField::Unspecified) => false,
                LogTargetSelector::LogAttribute(AttributePath { path }) => {
                    let can_write = match get_log_id!(r) {
                        Some(pid) => attrs_editor
                            .as_ref()
                            .is_some_and(|ed| ed.can_write_path(pid, path, upsert)),
                        None => !path.is_empty() && path.len() <= MAX_CBOR_DEPTH + 1,
                    };
                    if can_write && let Some(pid) = get_or_alloc_log_id!(r) {
                        attrs_editor
                            .as_mut()
                            .is_some_and(|ed| ed.set_path(pid, path, v, upsert))
                    } else {
                        false
                    }
                }
                LogTargetSelector::BodyField(AttributePath { path }) => body_editor
                    .as_mut()
                    .is_some_and(|e| e.set_body_field(r, path, &v, upsert)),
            }
        }};
    }

    // STAGE 1: RENAME (across all matching policies in ascending Policy.id order)
    for (tx, mask) in transforms.iter().zip(policy_matches.iter()) {
        if mask.is_empty() || tx.rename.is_empty() {
            continue;
        }
        for op in &tx.rename {
            if op.from == op.to {
                continue;
            }
            // Fast zero-copy path for flat LogAttribute -> LogAttribute rename.
            if let (
                LogAttributeSelector::LogAttribute(AttributePath { path: from_path }),
                LogAttributeSelector::LogAttribute(AttributePath { path: to_path }),
            ) = (&op.from, &op.to)
                && from_path.len() == 1
                && to_path.len() == 1
            {
                let from_key = &from_path[0];
                let to_key = &to_path[0];
                for row_u32 in mask.iter() {
                    let row = row_u32 as usize;
                    let Some(pid) = get_log_id!(row) else {
                        continue;
                    };
                    if let Some(ref mut ed) = attrs_editor {
                        ed.rename_flat(pid, from_key, to_key, op.upsert);
                    }
                }
                continue;
            }

            // General atomic rename supporting nested LogAttribute paths and BodyField across all
            // 4 selector pairs: validate `to` writability before removing `from` (fail-open).
            for row_u32 in mask.iter() {
                let row = row_u32 as usize;
                let src_val = match &op.from {
                    LogAttributeSelector::LogAttribute(AttributePath { path }) => get_log_id!(row)
                        .and_then(|pid| {
                            attrs_editor.as_ref().and_then(|ed| ed.get_path(pid, path))
                        }),
                    LogAttributeSelector::BodyField(AttributePath { path }) => body_editor
                        .as_ref()
                        .and_then(|e| e.get_body_field(row, path)),
                };
                let Some(src_val) = src_val else {
                    continue;
                };

                let dest_pid = match &op.to {
                    LogAttributeSelector::LogAttribute(AttributePath { path }) => {
                        let writable = match get_log_id!(row) {
                            Some(pid) => attrs_editor
                                .as_ref()
                                .is_some_and(|ed| ed.can_write_path(pid, path, op.upsert)),
                            None => !path.is_empty() && path.len() <= MAX_CBOR_DEPTH + 1,
                        };
                        if !writable {
                            continue;
                        }
                        let Some(pid) = get_or_alloc_log_id!(row) else {
                            continue;
                        };
                        Some(pid)
                    }
                    LogAttributeSelector::BodyField(AttributePath { path }) => {
                        if !body_editor
                            .as_ref()
                            .is_some_and(|e| e.can_write_body_field(row, path, op.upsert))
                        {
                            continue;
                        }
                        None
                    }
                };

                // Remove `from`, then write `to`.
                match &op.from {
                    LogAttributeSelector::LogAttribute(AttributePath { path }) => {
                        if let Some(pid) = get_log_id!(row)
                            && let Some(ref mut ed) = attrs_editor
                        {
                            let _ = ed.remove_path(pid, path);
                        }
                    }
                    LogAttributeSelector::BodyField(AttributePath { path }) => {
                        if let Some(ref mut e) = body_editor {
                            let _ = e.remove_body_field(row, path);
                        }
                    }
                }
                match &op.to {
                    LogAttributeSelector::LogAttribute(AttributePath { path }) => {
                        if let (Some(pid), Some(ed)) = (dest_pid, &mut attrs_editor) {
                            let _ = ed.set_path(pid, path, src_val, op.upsert);
                        }
                    }
                    LogAttributeSelector::BodyField(AttributePath { path }) => {
                        if let Some(ref mut e) = body_editor {
                            let _ = e.set_body_field(row, path, &src_val, op.upsert);
                        }
                    }
                }
            }
        }
    }

    // STAGE 2: ADD (across all matching policies in ascending Policy.id order)
    for (tx, mask) in transforms.iter().zip(policy_matches.iter()) {
        if mask.is_empty() || tx.add.is_empty() {
            continue;
        }
        for op in &tx.add {
            for row_u32 in mask.iter() {
                let row = row_u32 as usize;
                let _ = set_target!(
                    row,
                    &op.target,
                    CellValue::Scalar(op.value.clone()),
                    op.upsert
                );
            }
        }
    }

    // STAGE 3: REMOVE (across all matching policies in ascending Policy.id order)
    for (tx, mask) in transforms.iter().zip(policy_matches.iter()) {
        if mask.is_empty() || tx.remove.is_empty() {
            continue;
        }
        for op in &tx.remove {
            for row_u32 in mask.iter() {
                let row = row_u32 as usize;
                let _ = remove_target!(row, &op.target);
            }
        }
    }

    // STAGE 4: REDACT (across all matching policies in ascending Policy.id order)
    for (tx, mask) in transforms.iter().zip(policy_matches.iter()) {
        if mask.is_empty() || tx.redact.is_empty() {
            continue;
        }
        for op in &tx.redact {
            // Per policy.md ("Redact"), targeting non-string typed record fields yields no change.
            if matches!(
                op.target,
                LogTargetSelector::RecordField(
                    LogRecordField::SeverityNumber
                        | LogRecordField::TraceId
                        | LogRecordField::SpanId
                        | LogRecordField::Unspecified
                )
            ) {
                continue;
            }
            for row_u32 in mask.iter() {
                let row = row_u32 as usize;
                let Some(cur_val) = get_target!(row, &op.target) else {
                    continue;
                };
                match &op.regex {
                    None => {
                        // Whole-value mask (converts value to replacement string per `policy.md` "Redact")
                        let _ = set_target!(
                            row,
                            &op.target,
                            CellValue::Scalar(ScalarValue::String(op.replacement.clone())),
                            true
                        );
                    }
                    Some(re) => {
                        if let CellValue::Scalar(ScalarValue::String(s)) = cur_val
                            && let Some(redacted) = apply_regex_redaction(re, &s, &op.replacement)
                        {
                            let _ = set_target!(
                                row,
                                &op.target,
                                CellValue::Scalar(ScalarValue::String(redacted)),
                                true
                            );
                        }
                    }
                }
            }
        }
    }

    let body_dirty = body_editor.as_ref().is_some_and(|e| e.dirty);
    let sev_text_dirty = sev_text_editor.as_ref().is_some_and(|e| e.dirty);
    let event_name_dirty = event_name_editor.as_ref().is_some_and(|e| e.dirty);

    // Rebuild modified root columns only when a root field or log.id actually changed.
    if body_dirty
        || sev_text_dirty
        || sev_num_dirty
        || event_name_dirty
        || trace_id_dirty
        || span_id_dirty
        || mut_log_ids.is_some()
    {
        let schema = root.schema();
        let mut new_fields: Vec<Arc<Field>> = Vec::with_capacity(schema.fields().len() + 7);
        let mut new_cols: Vec<ArrayRef> = Vec::with_capacity(schema.fields().len() + 7);
        let mut wrote_id = false;
        let mut wrote_body = false;
        let mut wrote_sev_text = false;
        let mut wrote_sev_num = false;
        let mut wrote_event_name = false;
        let mut wrote_trace_id = false;
        let mut wrote_span_id = false;

        for (idx, field) in schema.fields().iter().enumerate() {
            match field.name().as_str() {
                consts::ID if mut_log_ids.is_some() => {
                    if let Some(ref ids) = mut_log_ids {
                        let arr: ArrayRef = Arc::new(UInt16Array::from(ids.clone()));
                        new_fields.push(Arc::new(
                            Field::new(consts::ID, DataType::UInt16, true).with_plain_encoding(),
                        ));
                        new_cols.push(arr);
                        wrote_id = true;
                    }
                }
                consts::BODY if body_dirty => {
                    if let Some(ref ed) = body_editor {
                        let arr = ed.build_struct_array();
                        new_fields.push(Arc::new(Field::new(
                            consts::BODY,
                            arr.data_type().clone(),
                            true,
                        )));
                        new_cols.push(arr);
                        wrote_body = true;
                    }
                }
                consts::SEVERITY_TEXT if sev_text_dirty => {
                    if let Some(ref ed) = sev_text_editor {
                        let arr = ed.build_array();
                        new_fields.push(Arc::new(Field::new(
                            consts::SEVERITY_TEXT,
                            DataType::Utf8,
                            true,
                        )));
                        new_cols.push(arr);
                        wrote_sev_text = true;
                    }
                }
                consts::SEVERITY_NUMBER if sev_num_dirty => {
                    let arr: ArrayRef = Arc::new(Int32Array::from(sev_num_vals.clone()));
                    new_fields.push(Arc::new(Field::new(
                        consts::SEVERITY_NUMBER,
                        DataType::Int32,
                        true,
                    )));
                    new_cols.push(arr);
                    wrote_sev_num = true;
                }
                consts::EVENT_NAME if event_name_dirty => {
                    if let Some(ref ed) = event_name_editor {
                        let arr = ed.build_array();
                        new_fields.push(Arc::new(Field::new(
                            consts::EVENT_NAME,
                            DataType::Utf8,
                            true,
                        )));
                        new_cols.push(arr);
                        wrote_event_name = true;
                    }
                }
                consts::TRACE_ID if trace_id_dirty => {
                    let arr = build_fixed_id_array::<16>(&trace_id_vals);
                    new_fields.push(Arc::new(Field::new(
                        consts::TRACE_ID,
                        DataType::FixedSizeBinary(16),
                        true,
                    )));
                    new_cols.push(arr);
                    wrote_trace_id = true;
                }
                consts::SPAN_ID if span_id_dirty => {
                    let arr = build_fixed_id_array::<8>(&span_id_vals);
                    new_fields.push(Arc::new(Field::new(
                        consts::SPAN_ID,
                        DataType::FixedSizeBinary(8),
                        true,
                    )));
                    new_cols.push(arr);
                    wrote_span_id = true;
                }
                _ => {
                    new_fields.push(field.clone());
                    new_cols.push(root.column(idx).clone());
                }
            }
        }
        if let Some(ids) = mut_log_ids
            && !wrote_id
        {
            let arr: ArrayRef = Arc::new(UInt16Array::from(ids));
            new_fields.push(Arc::new(
                Field::new(consts::ID, DataType::UInt16, true).with_plain_encoding(),
            ));
            new_cols.push(arr);
        }
        if body_dirty
            && !wrote_body
            && let Some(ref ed) = body_editor
            && ed.any_present()
        {
            let arr = ed.build_struct_array();
            new_fields.push(Arc::new(Field::new(
                consts::BODY,
                arr.data_type().clone(),
                true,
            )));
            new_cols.push(arr);
        }
        if sev_text_dirty
            && !wrote_sev_text
            && let Some(ref ed) = sev_text_editor
            && ed.any_present()
        {
            let arr = ed.build_array();
            new_fields.push(Arc::new(Field::new(
                consts::SEVERITY_TEXT,
                DataType::Utf8,
                true,
            )));
            new_cols.push(arr);
        }
        if sev_num_dirty && !wrote_sev_num && sev_num_vals.iter().any(Option::is_some) {
            let arr: ArrayRef = Arc::new(Int32Array::from(sev_num_vals));
            new_fields.push(Arc::new(Field::new(
                consts::SEVERITY_NUMBER,
                DataType::Int32,
                true,
            )));
            new_cols.push(arr);
        }
        if event_name_dirty
            && !wrote_event_name
            && let Some(ref ed) = event_name_editor
            && ed.any_present()
        {
            let arr = ed.build_array();
            new_fields.push(Arc::new(Field::new(
                consts::EVENT_NAME,
                DataType::Utf8,
                true,
            )));
            new_cols.push(arr);
        }
        if trace_id_dirty && !wrote_trace_id && trace_id_vals.iter().any(Option::is_some) {
            let arr = build_fixed_id_array::<16>(&trace_id_vals);
            new_fields.push(Arc::new(Field::new(
                consts::TRACE_ID,
                DataType::FixedSizeBinary(16),
                true,
            )));
            new_cols.push(arr);
        }
        if span_id_dirty && !wrote_span_id && span_id_vals.iter().any(Option::is_some) {
            let arr = build_fixed_id_array::<8>(&span_id_vals);
            new_fields.push(Arc::new(Field::new(
                consts::SPAN_ID,
                DataType::FixedSizeBinary(8),
                true,
            )));
            new_cols.push(arr);
        }
        let new_root = RecordBatch::try_new(
            Arc::new(Schema::new_with_metadata(
                new_fields,
                schema.metadata().clone(),
            )),
            new_cols,
        )
        .map_err(|e| PolicyError::Evaluation(e.to_string()))?;
        batch
            .set(ArrowPayloadType::Logs, new_root)
            .map_err(|e| PolicyError::Evaluation(e.to_string()))?;
    }

    if let Some(editor) = attrs_editor
        && editor.dirty
    {
        match editor.finish_record_batch()? {
            None => {
                let _ = batch.remove(ArrowPayloadType::LogAttrs);
            }
            Some(rb) => {
                batch
                    .set(ArrowPayloadType::LogAttrs, rb)
                    .map_err(|e| PolicyError::Evaluation(e.to_string()))?;
            }
        }
    }

    Ok(())
}

fn extract_root_i32_vec(root: &RecordBatch, col_name: &str, num_rows: usize) -> Vec<Option<i32>> {
    let mut out = vec![None; num_rows];
    let Some(col) = root.column_by_name(col_name) else {
        return out;
    };
    for (r, slot) in out.iter_mut().enumerate().take(num_rows) {
        if let Some(v) = extract_i64_from_array(col, r)
            && v != 0
            && let Ok(v32) = i32::try_from(v)
        {
            *slot = Some(v32);
        }
    }
    out
}

fn extract_root_fixed_id_vec<const N: usize>(
    root: &RecordBatch,
    col_name: &str,
    num_rows: usize,
) -> Vec<Option<[u8; N]>> {
    let mut out = vec![None; num_rows];
    let Some(col) = root.column_by_name(col_name) else {
        return out;
    };
    for (r, slot) in out.iter_mut().enumerate().take(num_rows) {
        if let Some(raw) = extract_fixed_bytes_from_array(col, r)
            && let Ok(bytes) = <[u8; N]>::try_from(raw)
            && !bytes.iter().all(|&b| b == 0)
        {
            *slot = Some(bytes);
        }
    }
    out
}

fn build_fixed_id_array<const N: usize>(vals: &[Option<[u8; N]>]) -> ArrayRef {
    let mut builder = FixedSizeBinaryBuilder::with_capacity(vals.len(), N as i32);
    for opt in vals {
        match opt {
            Some(bytes) => {
                let _ = builder.append_value(bytes);
            }
            None => builder.append_null(),
        }
    }
    Arc::new(builder.finish())
}
