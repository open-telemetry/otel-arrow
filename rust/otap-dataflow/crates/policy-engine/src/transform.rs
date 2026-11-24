// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Masked 4-stage `LogTransformPolicy` execution (`1. Rename -> 2. Add -> 3. Remove -> 4. Redact`)
//! over OTAP `OtapArrowRecords`, including nested CBOR map mutations and lazy direct Arrow builders.

use std::borrow::Cow;
use std::sync::Arc;

use ahash::{AHashMap, AHashSet};
use arrow::array::{
    Array, ArrayBuilder, ArrayRef, BinaryBuilder, BooleanBuilder, Float64Builder, Int32Array,
    Int32Builder, Int64Array, Int64Builder, NullBufferBuilder, RecordBatch, StringArray,
    StringBuilder, StructArray, UInt8Builder, UInt16Array, UInt16Builder,
};
use arrow::datatypes::{DataType, Field, Schema};
use otel_arrow_dfe_pdata::arrays::{MaybeDictArrayAccessor, NullableArrayAccessor};
use otel_arrow_dfe_pdata::otap::OtapArrowRecords;
use otel_arrow_dfe_pdata::otlp::attributes::AttributeValueType;
use otel_arrow_dfe_pdata::otlp::common::AnyValueArrays;
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_dfe_pdata::schema::{FieldExt, consts};
use regex_automata::meta::Regex as MetaRegex;

use crate::bitset::DenseBitset;
use crate::compile::attr_selectors_overlap;
use crate::model::{
    AttributePath, LogAdd, LogAttributeSelector, LogRecordField, LogRemove, LogRename,
    LogTargetSelector, PolicyError, ScalarValue, apply_regex_redaction_with_caps,
};
#[cfg(test)]
use crate::model::{LogTransformPolicy, compile_redact_regex};
use crate::otap_ops::{
    CborPathResult, ExtractedValue, MAX_CBOR_DEPTH, ParentIdColumn, StrColAccessor, byte_slice_at,
    cbor_item_depth, decode_cbor_value, extract_anyval, extract_cbor_path_status,
    is_valid_single_cbor_item, parse_cbor_map_entries,
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

impl CompiledTransformPolicy {
    /// Returns `true` if any operation in this transform policy reads or writes `LogAttrs`.
    #[must_use]
    pub(crate) fn touches_log_attributes(&self) -> bool {
        self.rename.iter().any(|op| {
            matches!(op.from, LogAttributeSelector::LogAttribute(_))
                || matches!(op.to, LogAttributeSelector::LogAttribute(_))
        }) || self
            .remove
            .iter()
            .any(|op| matches!(op.target, LogAttributeSelector::LogAttribute(_)))
            || self
                .add
                .iter()
                .any(|op| matches!(op.target, LogTargetSelector::LogAttribute(_)))
            || self
                .redact
                .iter()
                .any(|op| matches!(op.target, LogTargetSelector::LogAttribute(_)))
    }
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
            let replacement = r.effective_replacement();
            let compiled_re = match &r.regex {
                Some(pat) => Some(compile_redact_regex(pat, replacement.contains('$'))?),
                None => None,
            };
            redact.push(CompiledLogRedact {
                target: r.target.clone(),
                replacement: replacement.to_owned(),
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

impl CellValue {
    /// Returns the CBOR nesting depth (`1..=MAX_CBOR_DEPTH`) required to store this value inside
    /// a CBOR container, or `None` if a `RawCbor` payload is malformed or exceeds `MAX_CBOR_DEPTH`.
    fn cbor_depth(&self) -> Option<usize> {
        match self {
            Self::Empty | Self::Scalar(_) => Some(1),
            Self::RawCbor { bytes, .. } => cbor_item_depth(bytes),
        }
    }
}

#[inline]
fn is_body_cell_absent(cell: &CellValue) -> bool {
    matches!(cell, CellValue::Empty)
        || matches!(cell, CellValue::Scalar(ScalarValue::String(s)) if s.is_empty())
        || matches!(cell, CellValue::RawCbor { bytes, .. } if bytes.as_slice() == [0xf6])
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum WriteMode {
    InsertOnly,
    Upsert,
    UpdateExistingOnly,
}

impl WriteMode {
    #[inline]
    fn from_upsert(upsert: bool) -> Self {
        if upsert {
            Self::Upsert
        } else {
            Self::InsertOnly
        }
    }

    #[inline]
    fn allows(self, exists: bool) -> bool {
        match self {
            Self::InsertOnly => !exists,
            Self::Upsert => true,
            Self::UpdateExistingOnly => exists,
        }
    }
}

#[derive(Clone, Copy)]
enum CborLeaf<'a> {
    Cell(&'a CellValue),
    Scalar(&'a ScalarValue),
}

fn encode_cbor_scalar(val: &ScalarValue, out: &mut Vec<u8>) {
    match val {
        ScalarValue::String(s) => encode_cbor_text(s, out),
        ScalarValue::Int(i) => encode_cbor_int(*i, out),
        ScalarValue::Double(d) => {
            out.push(0xfb);
            out.extend_from_slice(&d.to_bits().to_be_bytes());
        }
        ScalarValue::Bool(b) => out.push(if *b { 0xf5 } else { 0xf4 }),
        ScalarValue::Bytes(b) => encode_cbor_bytes(b, out),
    }
}

fn encode_cbor_leaf(val: CborLeaf<'_>, out: &mut Vec<u8>) {
    match val {
        CborLeaf::Scalar(s) => encode_cbor_scalar(s, out),
        CborLeaf::Cell(CellValue::Empty) => out.push(0xf6),
        CborLeaf::Cell(CellValue::Scalar(s)) => encode_cbor_scalar(s, out),
        CborLeaf::Cell(CellValue::RawCbor { bytes, .. }) => out.extend_from_slice(bytes),
    }
}

fn encode_cbor_nested_map_into(path: &[String], leaf: CborLeaf<'_>, out: &mut Vec<u8>) {
    for seg in path {
        out.push(0xa1); // definite-length map of 1 entry (major 5, ai 1)
        encode_cbor_text(seg, out);
    }
    encode_cbor_leaf(leaf, out);
}

/// Encodes `leaf` wrapped inside nested 1-entry RFC 8949 CBOR maps for every segment in `path`.
fn encode_cbor_nested_map(path: &[String], leaf: CborLeaf<'_>) -> Vec<u8> {
    let mut out = Vec::with_capacity(path.len() * 16 + 16);
    encode_cbor_nested_map_into(path, leaf, &mut out);
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
    if depth + path.len() >= MAX_CBOR_DEPTH {
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

fn cbor_map_can_write(
    bytes: &[u8],
    path: &[String],
    val_depth: usize,
    upsert: bool,
    depth: usize,
) -> bool {
    let Some((seg, rest)) = path.split_first() else {
        return false;
    };
    if depth + path.len() + val_depth > MAX_CBOR_DEPTH {
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
        (Some((_, _, raw_val)), false) => {
            cbor_map_can_write(raw_val, rest, val_depth, upsert, depth + 1)
        }
        (None, false) => true,
    }
}

/// Sets `path` inside a CBOR map, copying all untouched sibling keys and values byte-for-byte
/// without decoding replaced leaves (succeeds even on non-OTLP CBOR values under `Upsert` or
/// `UpdateExistingOnly`). Returns `None` if the operation is a fail-open no-op (non-map
/// intermediate, `InsertOnly` on existing leaf, `UpdateExistingOnly` on missing key, or resulting
/// depth `> MAX_CBOR_DEPTH`).
fn cbor_map_set(
    bytes: &[u8],
    path: &[String],
    value: CborLeaf<'_>,
    val_depth: usize,
    mode: WriteMode,
    depth: usize,
) -> Option<Vec<u8>> {
    let (seg, rest) = path.split_first()?;
    if depth + path.len() + val_depth > MAX_CBOR_DEPTH {
        return None;
    }
    let entries = parse_cbor_map_entries(bytes)?;
    let match_idx = entries
        .iter()
        .position(|(k, _, _)| k.as_deref() == Some(seg.as_str()));

    if let Some(idx) = match_idx {
        let updated_child = if rest.is_empty() {
            if matches!(mode, WriteMode::InsertOnly) {
                return None;
            }
            None
        } else {
            Some(cbor_map_set(
                entries[idx].2,
                rest,
                value,
                val_depth,
                mode,
                depth + 1,
            )?)
        };

        // Count surviving entries (replace matched key at `idx`, drop any duplicate text keys of `seg`).
        let count = entries
            .iter()
            .enumerate()
            .filter(|(i, (k, _, _))| *i == idx || k.as_deref() != Some(seg.as_str()))
            .count();
        let extra = updated_child.as_ref().map_or(16, Vec::len);
        let mut out = Vec::with_capacity(bytes.len() + extra);
        encode_cbor_head(5, count as u64, &mut out);
        for (i, (k, raw_k, raw_v)) in entries.iter().enumerate() {
            if i == idx {
                out.extend_from_slice(raw_k);
                if let Some(ref child) = updated_child {
                    out.extend_from_slice(child);
                } else {
                    encode_cbor_leaf(value, &mut out);
                }
            } else if k.as_deref() != Some(seg.as_str()) {
                out.extend_from_slice(raw_k);
                out.extend_from_slice(raw_v);
            }
        }
        Some(out)
    } else {
        if matches!(mode, WriteMode::UpdateExistingOnly) {
            return None;
        }
        let mut out = Vec::with_capacity(bytes.len() + seg.len() + rest.len() * 16 + 24);
        encode_cbor_head(5, (entries.len() + 1) as u64, &mut out);
        for (_, raw_k, raw_v) in &entries {
            out.extend_from_slice(raw_k);
            out.extend_from_slice(raw_v);
        }
        encode_cbor_text(seg, &mut out);
        if rest.is_empty() {
            encode_cbor_leaf(value, &mut out);
        } else {
            encode_cbor_nested_map_into(rest, value, &mut out);
        }
        Some(out)
    }
}

/// Removes `path` from a CBOR map, copying all sibling keys and values byte-for-byte without
/// decoding the removed value (succeeds even on non-OTLP CBOR values such as tags or `u64 > i64::MAX`).
/// Returns `Some(updated_map_bytes)` if found, or `None` if absent/non-map.
fn cbor_map_remove(bytes: &[u8], path: &[String], depth: usize) -> Option<Vec<u8>> {
    let (seg, rest) = path.split_first()?;
    if depth + path.len() >= MAX_CBOR_DEPTH {
        return None;
    }
    let entries = parse_cbor_map_entries(bytes)?;
    let idx = entries
        .iter()
        .position(|(k, _, _)| k.as_deref() == Some(seg.as_str()))?;

    if rest.is_empty() {
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
        Some(out)
    } else {
        let new_child = cbor_map_remove(entries[idx].2, rest, depth + 1)?;
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
        Some(out)
    }
}

// ============================================================================
// Lazy direct Arrow AnyValue column builders & zero-copy editors
// ============================================================================

/// Lazy Arrow builders for OTAP `AnyValue` columns. Value-type builders are only allocated and
/// emitted in the resulting schema when at least one non-null value of that type is appended.
struct AnyValBuilders {
    len: usize,
    cap: usize,
    types: UInt8Builder,
    strs: Option<StringBuilder>,
    ints: Option<Int64Builder>,
    doubles: Option<Float64Builder>,
    bools: Option<BooleanBuilder>,
    bytes: Option<BinaryBuilder>,
    ser: Option<BinaryBuilder>,
}

impl AnyValBuilders {
    fn with_capacity(rows: usize) -> Self {
        Self {
            len: 0,
            cap: rows,
            types: UInt8Builder::with_capacity(rows),
            strs: None,
            ints: None,
            doubles: None,
            bools: None,
            bytes: None,
            ser: None,
        }
    }

    #[inline]
    fn append_nulls_except(&mut self, active: u8) {
        if active != 1
            && let Some(b) = &mut self.strs
        {
            b.append_null();
        }
        if active != 2
            && let Some(b) = &mut self.ints
        {
            b.append_null();
        }
        if active != 3
            && let Some(b) = &mut self.doubles
        {
            b.append_null();
        }
        if active != 4
            && let Some(b) = &mut self.bools
        {
            b.append_null();
        }
        if active != 5
            && let Some(b) = &mut self.bytes
        {
            b.append_null();
        }
        if active != 6
            && let Some(b) = &mut self.ser
        {
            b.append_null();
        }
        self.len += 1;
    }

    #[inline]
    fn append_empty(&mut self) {
        self.types.append_value(AttributeValueType::Empty as u8);
        self.append_nulls_except(0);
    }

    #[inline]
    fn append_str(&mut self, s: &str) {
        self.types.append_value(AttributeValueType::Str as u8);
        let len = self.len;
        let cap = self.cap;
        let b = self.strs.get_or_insert_with(|| {
            let mut sb = StringBuilder::with_capacity(cap, cap * 8);
            sb.append_nulls(len);
            sb
        });
        b.append_value(s);
        self.append_nulls_except(1);
    }

    #[inline]
    fn append_int(&mut self, i: i64) {
        self.types.append_value(AttributeValueType::Int as u8);
        let len = self.len;
        let cap = self.cap;
        let b = self.ints.get_or_insert_with(|| {
            let mut ib = Int64Builder::with_capacity(cap);
            ib.append_nulls(len);
            ib
        });
        b.append_value(i);
        self.append_nulls_except(2);
    }

    #[inline]
    fn append_double(&mut self, d: f64) {
        self.types.append_value(AttributeValueType::Double as u8);
        let len = self.len;
        let cap = self.cap;
        let b = self.doubles.get_or_insert_with(|| {
            let mut db = Float64Builder::with_capacity(cap);
            db.append_nulls(len);
            db
        });
        b.append_value(d);
        self.append_nulls_except(3);
    }

    #[inline]
    fn append_bool(&mut self, val: bool) {
        self.types.append_value(AttributeValueType::Bool as u8);
        let len = self.len;
        let cap = self.cap;
        let b = self.bools.get_or_insert_with(|| {
            let mut bb = BooleanBuilder::with_capacity(cap);
            bb.append_nulls(len);
            bb
        });
        b.append_value(val);
        self.append_nulls_except(4);
    }

    #[inline]
    fn append_bytes(&mut self, raw: &[u8]) {
        self.types.append_value(AttributeValueType::Bytes as u8);
        let len = self.len;
        let cap = self.cap;
        let b = self.bytes.get_or_insert_with(|| {
            let mut bb = BinaryBuilder::with_capacity(cap, 0);
            bb.append_nulls(len);
            bb
        });
        b.append_value(raw);
        self.append_nulls_except(5);
    }

    #[inline]
    fn append_cbor(&mut self, vtype: u8, raw: &[u8]) {
        self.types.append_value(vtype);
        let len = self.len;
        let cap = self.cap;
        let b = self.ser.get_or_insert_with(|| {
            let mut sb = BinaryBuilder::with_capacity(cap, 0);
            sb.append_nulls(len);
            sb
        });
        b.append_value(raw);
        self.append_nulls_except(6);
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
                    .and_then(|c| StrColAccessor::from_maybe_dict(c).str_at(row))
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
                    .and_then(|c| byte_slice_at(c, row))
                    .unwrap_or(&[]);
                self.append_bytes(b);
            }
            Ok(vtype @ (AttributeValueType::Map | AttributeValueType::Slice)) => {
                if let Some(raw) = arrays.attr_ser.as_ref().and_then(|c| byte_slice_at(c, row)) {
                    self.append_cbor(vtype as u8, raw);
                } else {
                    self.append_empty();
                }
            }
            Ok(AttributeValueType::Empty) | Err(_) => self.append_empty(),
        }
    }

    fn finish_fields_and_arrays(mut self) -> (Vec<Arc<Field>>, Vec<ArrayRef>) {
        let mut fields = Vec::with_capacity(7);
        let mut arrays: Vec<ArrayRef> = Vec::with_capacity(7);

        fields.push(Arc::new(Field::new(
            consts::ATTRIBUTE_TYPE,
            DataType::UInt8,
            false,
        )));
        arrays.push(Arc::new(self.types.finish()));

        if let Some(mut b) = self.strs {
            fields.push(Arc::new(Field::new(
                consts::ATTRIBUTE_STR,
                DataType::Utf8,
                true,
            )));
            arrays.push(Arc::new(b.finish()));
        }
        if let Some(mut b) = self.ints {
            fields.push(Arc::new(Field::new(
                consts::ATTRIBUTE_INT,
                DataType::Int64,
                true,
            )));
            arrays.push(Arc::new(b.finish()));
        }
        if let Some(mut b) = self.doubles {
            fields.push(Arc::new(Field::new(
                consts::ATTRIBUTE_DOUBLE,
                DataType::Float64,
                true,
            )));
            arrays.push(Arc::new(b.finish()));
        }
        if let Some(mut b) = self.bools {
            fields.push(Arc::new(Field::new(
                consts::ATTRIBUTE_BOOL,
                DataType::Boolean,
                true,
            )));
            arrays.push(Arc::new(b.finish()));
        }
        if let Some(mut b) = self.bytes {
            fields.push(Arc::new(Field::new(
                consts::ATTRIBUTE_BYTES,
                DataType::Binary,
                true,
            )));
            arrays.push(Arc::new(b.finish()));
        }
        if let Some(mut b) = self.ser {
            fields.push(Arc::new(Field::new(
                consts::ATTRIBUTE_SER,
                DataType::Binary,
                true,
            )));
            arrays.push(Arc::new(b.finish()));
        }

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
            .and_then(|c| byte_slice_at(c, row))
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

enum OrigBody<'a> {
    Absent,
    Struct {
        sa: &'a StructArray,
        arrays: AnyValueArrays<'a>,
        str_acc: Option<StrColAccessor<'a>>,
    },
    StrCol(StrColAccessor<'a>),
}

/// Logical state of `logs.body` at a row for `BodyField` operations, shared across
/// `can_write_body_field`, `set_body_field`, and `remove_body_field` so they stay 100% consistent.
enum BodyMapState<'a> {
    /// Body is unset, empty `AnyValue`, empty string `""`, or invalid null-Bool/null-Slice/null-Map.
    Absent,
    /// Body holds a CBOR map payload.
    Map(&'a [u8]),
    /// Body holds a non-map present value (e.g. non-empty string, int, bool, bytes, slice).
    OtherPresent,
}

/// Zero-copy sparse overlay editor for `logs.body`, borrowing untouched rows from the input batch
/// and writing directly into lazy Arrow builders when rebuilding.
struct BodyEditor<'a> {
    orig: OrigBody<'a>,
    num_rows: usize,
    overrides: AHashMap<usize, CellValue>,
    dirty: bool,
}

impl<'a> BodyEditor<'a> {
    fn new(root: &'a RecordBatch, num_rows: usize) -> Self {
        let orig = match root.column_by_name(consts::BODY) {
            None => OrigBody::Absent,
            Some(col) => {
                if let Some(sa) = col.as_any().downcast_ref::<StructArray>() {
                    match AnyValueArrays::try_from(sa) {
                        Ok(arrays) => {
                            let str_acc = arrays
                                .attr_str
                                .as_ref()
                                .map(StrColAccessor::from_maybe_dict);
                            OrigBody::Struct {
                                sa,
                                arrays,
                                str_acc,
                            }
                        }
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

    fn map_state(&self, row: usize) -> BodyMapState<'_> {
        if let Some(cell) = self.overrides.get(&row) {
            return match cell {
                c if is_body_cell_absent(c) => BodyMapState::Absent,
                CellValue::RawCbor { vtype, bytes } if *vtype == AttributeValueType::Map as u8 => {
                    BodyMapState::Map(bytes.as_slice())
                }
                _ => BodyMapState::OtherPresent,
            };
        }
        match &self.orig {
            OrigBody::Absent => BodyMapState::Absent,
            OrigBody::Struct {
                sa,
                arrays,
                str_acc,
            } => {
                if !sa.is_valid(row) || !arrays.attr_type.is_valid(row) {
                    return BodyMapState::Absent;
                }
                match AttributeValueType::try_from(arrays.attr_type.value(row)) {
                    Ok(AttributeValueType::Empty) | Err(_) => BodyMapState::Absent,
                    Ok(AttributeValueType::Str) => {
                        if str_acc
                            .as_ref()
                            .and_then(|acc| acc.str_at(row))
                            .is_some_and(|s| !s.is_empty())
                        {
                            BodyMapState::OtherPresent
                        } else {
                            BodyMapState::Absent
                        }
                    }
                    Ok(AttributeValueType::Bool) => {
                        if arrays.attr_bool.is_some_and(|ba| ba.is_valid(row)) {
                            BodyMapState::OtherPresent
                        } else {
                            BodyMapState::Absent
                        }
                    }
                    Ok(AttributeValueType::Map) => {
                        match arrays.attr_ser.as_ref().and_then(|c| byte_slice_at(c, row)) {
                            Some(raw) if raw != [0xf6] => BodyMapState::Map(raw),
                            _ => BodyMapState::Absent,
                        }
                    }
                    Ok(AttributeValueType::Slice) => {
                        match arrays.attr_ser.as_ref().and_then(|c| byte_slice_at(c, row)) {
                            Some(raw) if raw != [0xf6] => BodyMapState::OtherPresent,
                            _ => BodyMapState::Absent,
                        }
                    }
                    Ok(
                        AttributeValueType::Int
                        | AttributeValueType::Double
                        | AttributeValueType::Bytes,
                    ) => BodyMapState::OtherPresent,
                }
            }
            OrigBody::StrCol(acc) => {
                if acc.str_at(row).is_some_and(|s| !s.is_empty()) {
                    BodyMapState::OtherPresent
                } else {
                    BodyMapState::Absent
                }
            }
        }
    }

    /// Borrows `logs.body` as a non-empty UTF-8 string for regex `LogRedact` without cloning
    /// non-string or untouched values, treating empty string `""` as absent per `policy.md`.
    fn get_str(&self, row: usize) -> Option<Cow<'_, str>> {
        if let Some(cell) = self.overrides.get(&row) {
            return match cell {
                CellValue::Scalar(ScalarValue::String(s)) if !s.is_empty() => {
                    Some(Cow::Borrowed(s.as_str()))
                }
                _ => None,
            };
        }
        match &self.orig {
            OrigBody::Absent => None,
            OrigBody::Struct {
                sa,
                arrays,
                str_acc,
            } => {
                if !sa.is_valid(row)
                    || !arrays.attr_type.is_valid(row)
                    || arrays.attr_type.value(row) != AttributeValueType::Str as u8
                {
                    return None;
                }
                str_acc
                    .as_ref()
                    .and_then(|acc| acc.str_at(row))
                    .filter(|s| !s.is_empty())
                    .map(Cow::Borrowed)
            }
            OrigBody::StrCol(acc) => acc.str_at(row).filter(|s| !s.is_empty()).map(Cow::Borrowed),
        }
    }

    fn is_present(&self, row: usize) -> bool {
        !matches!(self.map_state(row), BodyMapState::Absent)
    }

    fn set(&mut self, row: usize, val: CellValue) {
        let _ = self.overrides.insert(row, val);
        self.dirty = true;
    }

    fn get_body_field(&self, row: usize, path: &[String]) -> Option<CellValue> {
        match self.map_state(row) {
            BodyMapState::Map(raw) => cbor_map_get(raw, path, 0),
            BodyMapState::Absent | BodyMapState::OtherPresent => None,
        }
    }

    fn get_body_field_str(&self, row: usize, path: &[String]) -> Option<Cow<'_, str>> {
        match self.map_state(row) {
            BodyMapState::Map(raw) => match extract_cbor_path_status(raw, path, false) {
                CborPathResult::Present(ExtractedValue::Str(s)) => Some(s),
                _ => None,
            },
            BodyMapState::Absent | BodyMapState::OtherPresent => None,
        }
    }

    fn can_write_body_field(
        &self,
        row: usize,
        path: &[String],
        val_depth: usize,
        upsert: bool,
    ) -> bool {
        if path.is_empty() || path.len() + val_depth > MAX_CBOR_DEPTH {
            return false;
        }
        match self.map_state(row) {
            BodyMapState::Absent => true,
            BodyMapState::Map(raw) => cbor_map_can_write(raw, path, val_depth, upsert, 0),
            BodyMapState::OtherPresent => false,
        }
    }

    fn set_body_field_with_depth(
        &mut self,
        row: usize,
        path: &[String],
        val: CborLeaf<'_>,
        val_depth: usize,
        mode: WriteMode,
    ) -> bool {
        if path.is_empty() || path.len() + val_depth > MAX_CBOR_DEPTH {
            return false;
        }
        let new_bytes = match self.map_state(row) {
            BodyMapState::Absent => {
                if matches!(mode, WriteMode::UpdateExistingOnly) {
                    return false;
                }
                encode_cbor_nested_map(path, val)
            }
            BodyMapState::Map(raw) => {
                let Some(updated) = cbor_map_set(raw, path, val, val_depth, mode, 0) else {
                    return false;
                };
                updated
            }
            BodyMapState::OtherPresent => return false,
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

    fn remove_body_field(&mut self, row: usize, path: &[String]) -> bool {
        let new_bytes = match self.map_state(row) {
            BodyMapState::Map(raw) => match cbor_map_remove(raw, path, 0) {
                Some(updated) => updated,
                None => return false,
            },
            BodyMapState::Absent | BodyMapState::OtherPresent => return false,
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

    fn any_present(&self) -> bool {
        (0..self.num_rows).any(|r| self.is_present(r))
    }

    fn build_struct_array(&self) -> ArrayRef {
        let mut builders = AnyValBuilders::with_capacity(self.num_rows);
        let mut validity = NullBufferBuilder::new(self.num_rows);

        for r in 0..self.num_rows {
            if let Some(cell) = self.overrides.get(&r) {
                validity.append_non_null();
                builders.append_cell(cell);
                continue;
            }
            match &self.orig {
                OrigBody::Absent => {
                    validity.append_null();
                    builders.append_empty();
                }
                OrigBody::Struct { sa, arrays, .. } => {
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
                                    .and_then(|c| byte_slice_at(c, r))
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

    fn get_str(&self, row: usize) -> Option<&str> {
        if let Some(ov) = self.overrides.get(&row) {
            return ov.as_deref();
        }
        self.orig.as_ref()?.str_at(row).filter(|s| !s.is_empty())
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

enum OrigSevNum<'a> {
    Absent,
    Int32(MaybeDictArrayAccessor<'a, Int32Array>),
    Int64(MaybeDictArrayAccessor<'a, Int64Array>),
}

impl<'a> OrigSevNum<'a> {
    fn from_column(col: &'a ArrayRef) -> Self {
        if let Ok(acc) = MaybeDictArrayAccessor::<Int32Array>::try_new(col) {
            Self::Int32(acc)
        } else if let Ok(acc) = MaybeDictArrayAccessor::<Int64Array>::try_new(col) {
            Self::Int64(acc)
        } else {
            Self::Absent
        }
    }

    #[inline]
    fn get_non_zero(&self, row: usize) -> Option<i32> {
        match self {
            Self::Absent => None,
            Self::Int32(acc) => acc.value_at(row).filter(|&v| v != 0),
            Self::Int64(acc) => acc
                .value_at(row)
                .filter(|&v| v != 0)
                .and_then(|v| i32::try_from(v).ok()),
        }
    }
}

/// Zero-copy sparse overlay editor for `logs.severity_number` (native or dictionary `Int32`/`Int64`).
struct RootSevNumEditor<'a> {
    orig: OrigSevNum<'a>,
    num_rows: usize,
    overrides: AHashMap<usize, i32>,
    dirty: bool,
}

impl<'a> RootSevNumEditor<'a> {
    fn new(root: &'a RecordBatch, col_name: &str, num_rows: usize) -> Self {
        let orig = root
            .column_by_name(col_name)
            .map(OrigSevNum::from_column)
            .unwrap_or(OrigSevNum::Absent);
        Self {
            orig,
            num_rows,
            overrides: AHashMap::new(),
            dirty: false,
        }
    }

    #[inline]
    fn get(&self, row: usize) -> Option<i32> {
        if let Some(&v) = self.overrides.get(&row) {
            return Some(v);
        }
        self.orig.get_non_zero(row)
    }

    #[inline]
    fn is_present(&self, row: usize) -> bool {
        self.overrides.contains_key(&row) || self.orig.get_non_zero(row).is_some()
    }

    #[inline]
    fn set(&mut self, row: usize, val: i32) {
        let _ = self.overrides.insert(row, val);
        self.dirty = true;
    }

    fn any_present(&self) -> bool {
        !self.overrides.is_empty()
            || (0..self.num_rows).any(|r| self.orig.get_non_zero(r).is_some())
    }

    fn build_array(&self) -> ArrayRef {
        let mut builder = Int32Builder::with_capacity(self.num_rows);
        for r in 0..self.num_rows {
            match self.get(r) {
                Some(v) => builder.append_value(v),
                None => builder.append_null(),
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

enum FlatMapState<'a> {
    Absent,
    Map(&'a [u8]),
    OtherPresent,
}

struct OrigAttrs<'a> {
    rb: &'a RecordBatch,
    parent_ids: ParentIdColumn<'a>,
    keys: StrColAccessor<'a>,
    arrays: Option<AnyValueArrays<'a>>,
}

/// Direct lazy Arrow builder & sparse overlay editor for `LogAttrs`.
///
/// Keys the overlay by `(u16, &'a str)` borrowed directly from `orig_rb` and `transforms` so
/// flat attribute operations incur zero key `String` allocations. Untouched attributes stream
/// directly from the input `RecordBatch` into lazy Arrow column builders.
struct FlatAttrsEditor<'a> {
    orig: Option<OrigAttrs<'a>>,
    orig_superseded: DenseBitset,
    overlay: AHashMap<(u16, &'a str), Option<AttrEntry>>,
    dirty: bool,
}

impl<'a> FlatAttrsEditor<'a> {
    fn new(
        rb_opt: Option<&'a RecordBatch>,
        touched_pids: &DenseBitset,
        touched_keys: &AHashSet<&'a str>,
    ) -> Self {
        let Some(rb) = rb_opt else {
            return Self::empty();
        };
        let Some(key_col) = rb.column_by_name(consts::ATTRIBUTE_KEY) else {
            return Self::empty();
        };
        let Some(keys) = StrColAccessor::try_from_array(key_col) else {
            return Self::empty();
        };
        let Some(parent_ids) = ParentIdColumn::from_batch(rb) else {
            return Self::empty();
        };
        let arrays = AnyValueArrays::try_from(rb).ok();

        let mut orig_superseded = DenseBitset::new();
        let mut overlay: AHashMap<(u16, &'a str), Option<AttrEntry>> = AHashMap::new();

        if !touched_pids.is_empty() && !touched_keys.is_empty() {
            // Fast dictionary pre-filter: if key_col is dictionary-encoded, check touched_keys
            // once per dictionary entry instead of hashing every attribute row.
            let dict_watched: Option<(&'a StringArray, Vec<bool>)> =
                keys.dict_values().map(|vals| {
                    let flags = (0..vals.len())
                        .map(|i| vals.is_valid(i) && touched_keys.contains(vals.value(i)))
                        .collect();
                    (vals, flags)
                });
            let any_watched = dict_watched
                .as_ref()
                .is_none_or(|(_, flags)| flags.iter().any(|&hit| hit));

            if any_watched {
                for r in 0..rb.num_rows() {
                    let k: &'a str = if let Some((vals, ref watched)) = dict_watched {
                        let Some(di) = keys.valid_dict_idx(r) else {
                            continue;
                        };
                        if !watched[di] {
                            continue;
                        }
                        vals.value(di)
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
                    if let Some(Some(AttrEntry::OrigRow { row: prev_r, .. })) = overlay.insert(
                        (pid, k),
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

    fn empty() -> Self {
        Self {
            orig: None,
            orig_superseded: DenseBitset::new(),
            overlay: AHashMap::new(),
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

    fn contains_flat(&self, pid: u16, key: &'a str) -> bool {
        matches!(self.overlay.get(&(pid, key)), Some(Some(_)))
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

    fn flat_map_state(&self, pid: u16, key: &'a str) -> FlatMapState<'_> {
        let Some(Some(entry)) = self.overlay.get(&(pid, key)) else {
            return FlatMapState::Absent;
        };
        match entry {
            AttrEntry::Owned(CellValue::RawCbor { vtype, bytes })
                if *vtype == AttributeValueType::Map as u8 =>
            {
                FlatMapState::Map(bytes.as_slice())
            }
            AttrEntry::Owned(_) => FlatMapState::OtherPresent,
            AttrEntry::OrigRow { row, .. } => {
                if let Some(ref orig) = self.orig
                    && let Some(ref arrays) = orig.arrays
                    && arrays.attr_type.is_valid(*row)
                    && arrays.attr_type.value(*row) == AttributeValueType::Map as u8
                    && let Some(raw) = arrays
                        .attr_ser
                        .as_ref()
                        .and_then(|c| byte_slice_at(c, *row))
                {
                    FlatMapState::Map(raw)
                } else {
                    FlatMapState::OtherPresent
                }
            }
        }
    }

    fn get_flat(&self, pid: u16, key: &'a str) -> Option<CellValue> {
        let entry = self.overlay.get(&(pid, key))?.as_ref()?;
        Some(self.materialize_entry(entry))
    }

    fn set_flat(&mut self, pid: u16, key: &'a str, val: CellValue) {
        let k = (pid, key);
        if let Some(Some(AttrEntry::OrigRow { row, moved: false })) = self.overlay.get(&k) {
            self.orig_superseded.insert(*row as u32);
        }
        let _ = self.overlay.insert(k, Some(AttrEntry::Owned(val)));
        self.dirty = true;
    }

    fn remove_flat(&mut self, pid: u16, key: &'a str) -> bool {
        let Some(prev) = self.overlay.get_mut(&(pid, key)).and_then(Option::take) else {
            return false;
        };
        if let AttrEntry::OrigRow { row, moved: false } = prev {
            self.orig_superseded.insert(row as u32);
        }
        self.dirty = true;
        true
    }

    /// Zero-copy flat-to-flat rename: preserves unmodified `OrigRow` values without cloning strings
    /// or CBOR buffers.
    fn rename_flat(&mut self, pid: u16, from_key: &'a str, to_key: &'a str, upsert: bool) {
        if from_key == to_key || !self.contains_flat(pid, from_key) {
            return;
        }
        if !upsert && self.contains_flat(pid, to_key) {
            return;
        }
        let Some(src_entry) = self
            .overlay
            .get_mut(&(pid, from_key))
            .and_then(Option::take)
        else {
            return;
        };
        let moved_entry = match src_entry {
            AttrEntry::OrigRow { row, .. } => {
                self.orig_superseded.insert(row as u32);
                AttrEntry::OrigRow { row, moved: true }
            }
            other => other,
        };
        let tk = (pid, to_key);
        if let Some(Some(AttrEntry::OrigRow { row, moved: false })) = self.overlay.get(&tk) {
            self.orig_superseded.insert(*row as u32);
        }
        let _ = self.overlay.insert(tk, Some(moved_entry));
        self.dirty = true;
    }

    fn get_path(&self, pid: u16, path: &'a [String]) -> Option<CellValue> {
        let (head, tail) = path.split_first()?;
        if tail.is_empty() {
            return self.get_flat(pid, head.as_str());
        }
        match self.flat_map_state(pid, head.as_str()) {
            FlatMapState::Map(raw) => cbor_map_get(raw, tail, 0),
            FlatMapState::Absent | FlatMapState::OtherPresent => None,
        }
    }

    fn get_path_str(&self, pid: u16, path: &'a [String]) -> Option<Cow<'_, str>> {
        let (head, tail) = path.split_first()?;
        if tail.is_empty() {
            let entry = self.overlay.get(&(pid, head.as_str()))?.as_ref()?;
            return match entry {
                AttrEntry::Owned(CellValue::Scalar(ScalarValue::String(s))) => {
                    Some(Cow::Borrowed(s.as_str()))
                }
                AttrEntry::Owned(_) => None,
                AttrEntry::OrigRow { row, .. } => {
                    let arrays = self.orig.as_ref()?.arrays.as_ref()?;
                    if !arrays.attr_type.is_valid(*row)
                        || arrays.attr_type.value(*row) != AttributeValueType::Str as u8
                    {
                        return None;
                    }
                    let s = arrays
                        .attr_str
                        .as_ref()
                        .and_then(|c| StrColAccessor::from_maybe_dict(c).str_at(*row))
                        .unwrap_or("");
                    Some(Cow::Borrowed(s))
                }
            };
        }
        match self.flat_map_state(pid, head.as_str()) {
            FlatMapState::Map(raw) => match extract_cbor_path_status(raw, tail, false) {
                CborPathResult::Present(ExtractedValue::Str(s)) => Some(s),
                _ => None,
            },
            FlatMapState::Absent | FlatMapState::OtherPresent => None,
        }
    }

    fn can_write_path(&self, pid: u16, path: &'a [String], val_depth: usize, upsert: bool) -> bool {
        let Some((head, tail)) = path.split_first() else {
            return false;
        };
        if tail.is_empty() {
            return val_depth <= MAX_CBOR_DEPTH && (upsert || !self.contains_flat(pid, head));
        }
        if tail.len() + val_depth > MAX_CBOR_DEPTH {
            return false;
        }
        match self.flat_map_state(pid, head.as_str()) {
            FlatMapState::Absent => true,
            FlatMapState::Map(raw) => cbor_map_can_write(raw, tail, val_depth, upsert, 0),
            FlatMapState::OtherPresent => false,
        }
    }

    fn set_nested_cbor_leaf(
        &mut self,
        pid: u16,
        head: &'a str,
        tail: &[String],
        leaf: CborLeaf<'_>,
        val_depth: usize,
        mode: WriteMode,
    ) -> bool {
        if tail.len() + val_depth > MAX_CBOR_DEPTH {
            return false;
        }
        let new_bytes = match self.flat_map_state(pid, head) {
            FlatMapState::Absent => {
                if matches!(mode, WriteMode::UpdateExistingOnly) {
                    return false;
                }
                encode_cbor_nested_map(tail, leaf)
            }
            FlatMapState::Map(raw) => {
                let Some(updated) = cbor_map_set(raw, tail, leaf, val_depth, mode, 0) else {
                    return false;
                };
                updated
            }
            FlatMapState::OtherPresent => return false,
        };
        self.set_flat(
            pid,
            head,
            CellValue::RawCbor {
                vtype: AttributeValueType::Map as u8,
                bytes: new_bytes,
            },
        );
        true
    }

    fn set_path_scalar(
        &mut self,
        pid: u16,
        path: &'a [String],
        val: Cow<'_, ScalarValue>,
        mode: WriteMode,
    ) -> bool {
        let Some((head, tail)) = path.split_first() else {
            return false;
        };
        if tail.is_empty() {
            if !mode.allows(self.contains_flat(pid, head)) {
                return false;
            }
            self.set_flat(pid, head.as_str(), CellValue::Scalar(val.into_owned()));
            return true;
        }
        self.set_nested_cbor_leaf(
            pid,
            head.as_str(),
            tail,
            CborLeaf::Scalar(val.as_ref()),
            1,
            mode,
        )
    }

    fn set_path_with_depth(
        &mut self,
        pid: u16,
        path: &'a [String],
        val: CellValue,
        val_depth: usize,
        mode: WriteMode,
    ) -> bool {
        let Some((head, tail)) = path.split_first() else {
            return false;
        };
        if tail.is_empty() {
            if !mode.allows(self.contains_flat(pid, head)) {
                return false;
            }
            self.set_flat(pid, head.as_str(), val);
            return true;
        }
        self.set_nested_cbor_leaf(
            pid,
            head.as_str(),
            tail,
            CborLeaf::Cell(&val),
            val_depth,
            mode,
        )
    }

    fn remove_path(&mut self, pid: u16, path: &'a [String]) -> bool {
        let Some((head, tail)) = path.split_first() else {
            return false;
        };
        if tail.is_empty() {
            return self.remove_flat(pid, head.as_str());
        }
        let new_bytes = match self.flat_map_state(pid, head.as_str()) {
            FlatMapState::Map(raw) => match cbor_map_remove(raw, tail, 0) {
                Some(updated) => updated,
                None => return false,
            },
            FlatMapState::Absent | FlatMapState::OtherPresent => return false,
        };
        self.set_flat(
            pid,
            head.as_str(),
            CellValue::RawCbor {
                vtype: AttributeValueType::Map as u8,
                bytes: new_bytes,
            },
        );
        true
    }

    /// Streams surviving untouched original rows and mutated/moved overlay entries directly into
    /// lazy Arrow builders. Preserves original schema metadata while removing `SORT_COLUMNS`.
    /// Returns `Ok(None)` when every attribute was removed (so the caller removes `LogAttrs`
    /// instead of attaching an empty 0-row batch).
    fn finish_record_batch(self) -> Result<Option<RecordBatch>, PolicyError> {
        let mut dirty_overlay: Vec<(u16, &str, &AttrEntry)> = self
            .overlay
            .iter()
            .filter_map(|(&(pid, key), opt)| match opt {
                Some(entry @ (AttrEntry::Owned(_) | AttrEntry::OrigRow { moved: true, .. })) => {
                    Some((pid, key, entry))
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

        let mut metadata = self
            .orig
            .as_ref()
            .map(|o| o.rb.schema().metadata().clone())
            .unwrap_or_default();
        let _ = metadata.remove(consts::metadata::SORT_COLUMNS);

        let rb = RecordBatch::try_new(
            Arc::new(Schema::new_with_metadata(fields, metadata)),
            arrays,
        )
        .map_err(|e| PolicyError::Evaluation(e.to_string()))?;
        Ok(Some(rb))
    }
}

/// Mutable columnar working state across all 4 transform stages on one `Logs` batch.
struct TransformState<'a> {
    num_rows: usize,
    body_editor: Option<BodyEditor<'a>>,
    sev_text_editor: Option<RootStrEditor<'a>>,
    sev_num_editor: Option<RootSevNumEditor<'a>>,
    event_name_editor: Option<RootStrEditor<'a>>,
    attrs_editor: Option<FlatAttrsEditor<'a>>,
    existing_log_ids: Option<&'a UInt16Array>,
    mut_log_ids: Option<Vec<Option<u16>>>,
    used_log_ids: Option<DenseBitset>,
    next_log_id_candidate: u32,
}

impl<'a> TransformState<'a> {
    #[inline]
    fn get_log_id(&self, row: usize) -> Option<u16> {
        if let Some(ref ids) = self.mut_log_ids {
            ids[row]
        } else {
            self.existing_log_ids
                .and_then(|a| a.is_valid(row).then(|| a.value(row)))
        }
    }

    fn get_or_alloc_log_id(&mut self, row: usize) -> Option<u16> {
        if let Some(id) = self.get_log_id(row) {
            return Some(id);
        }
        // Find a free u16 ID before materializing `mut_log_ids` so exhaustion never dirties the root batch.
        let used = self.used_log_ids.get_or_insert_with(|| {
            let mut bm = DenseBitset::new();
            if let Some(arr) = self.existing_log_ids {
                for i in 0..self.num_rows {
                    if arr.is_valid(i) {
                        bm.insert(u32::from(arr.value(i)));
                    }
                }
            }
            if let Some(ref editor) = self.attrs_editor {
                editor.collect_used_parent_ids(&mut bm);
            }
            bm
        });
        while self.next_log_id_candidate <= u32::from(u16::MAX)
            && used.contains(self.next_log_id_candidate)
        {
            self.next_log_id_candidate += 1;
        }
        if self.next_log_id_candidate > u32::from(u16::MAX) {
            return None;
        }
        let new_id = self.next_log_id_candidate as u16;
        used.insert(self.next_log_id_candidate);
        self.next_log_id_candidate += 1;

        let num_rows = self.num_rows;
        let existing_log_ids = self.existing_log_ids;
        let ids = self.mut_log_ids.get_or_insert_with(|| {
            if let Some(arr) = existing_log_ids {
                (0..num_rows)
                    .map(|i| arr.is_valid(i).then(|| arr.value(i)))
                    .collect()
            } else {
                vec![None; num_rows]
            }
        });
        ids[row] = Some(new_id);
        Some(new_id)
    }

    fn get_attr_selector(&self, row: usize, sel: &'a LogAttributeSelector) -> Option<CellValue> {
        match sel {
            LogAttributeSelector::LogAttribute(AttributePath { path }) => self
                .get_log_id(row)
                .and_then(|pid| self.attrs_editor.as_ref()?.get_path(pid, path)),
            LogAttributeSelector::BodyField(AttributePath { path }) => {
                self.body_editor.as_ref()?.get_body_field(row, path)
            }
        }
    }

    fn remove_attr_selector(&mut self, row: usize, sel: &'a LogAttributeSelector) -> bool {
        match sel {
            LogAttributeSelector::LogAttribute(AttributePath { path }) => {
                let Some(pid) = self.get_log_id(row) else {
                    return false;
                };
                self.attrs_editor
                    .as_mut()
                    .is_some_and(|ed| ed.remove_path(pid, path))
            }
            LogAttributeSelector::BodyField(AttributePath { path }) => self
                .body_editor
                .as_mut()
                .is_some_and(|e| e.remove_body_field(row, path)),
        }
    }

    fn get_str_for_redact(
        &self,
        row: usize,
        target: &'a LogTargetSelector,
    ) -> Option<Cow<'_, str>> {
        match target {
            LogTargetSelector::RecordField(LogRecordField::Body) => {
                self.body_editor.as_ref()?.get_str(row)
            }
            LogTargetSelector::RecordField(LogRecordField::SeverityText) => self
                .sev_text_editor
                .as_ref()?
                .get_str(row)
                .map(Cow::Borrowed),
            LogTargetSelector::RecordField(LogRecordField::EventName) => self
                .event_name_editor
                .as_ref()?
                .get_str(row)
                .map(Cow::Borrowed),
            LogTargetSelector::RecordField(_) => None,
            LogTargetSelector::LogAttribute(AttributePath { path }) => self
                .get_log_id(row)
                .and_then(|pid| self.attrs_editor.as_ref()?.get_path_str(pid, path)),
            LogTargetSelector::BodyField(AttributePath { path }) => {
                self.body_editor.as_ref()?.get_body_field_str(row, path)
            }
        }
    }

    fn set_target_scalar(
        &mut self,
        row: usize,
        target: &'a LogTargetSelector,
        val: Cow<'_, ScalarValue>,
        mode: WriteMode,
    ) -> bool {
        if matches!(val.as_ref(), ScalarValue::Double(d) if !d.is_finite()) {
            return false;
        }
        match target {
            LogTargetSelector::RecordField(LogRecordField::Body) => {
                let Some(ref mut e) = self.body_editor else {
                    return false;
                };
                if !mode.allows(e.is_present(row)) {
                    return false;
                }
                e.set(row, CellValue::Scalar(val.into_owned()));
                true
            }
            LogTargetSelector::RecordField(LogRecordField::SeverityText) => {
                let Some(ref mut e) = self.sev_text_editor else {
                    return false;
                };
                if !matches!(val.as_ref(), ScalarValue::String(_))
                    || !mode.allows(e.is_present(row))
                {
                    return false;
                }
                let ScalarValue::String(s) = val.into_owned() else {
                    return false;
                };
                e.set(row, s);
                true
            }
            LogTargetSelector::RecordField(LogRecordField::SeverityNumber) => {
                if let (Some(e), ScalarValue::Int(i)) = (&mut self.sev_num_editor, val.as_ref())
                    && (1..=24).contains(i)
                    && mode.allows(e.is_present(row))
                {
                    e.set(row, *i as i32);
                    true
                } else {
                    false
                }
            }
            LogTargetSelector::RecordField(LogRecordField::EventName) => {
                let Some(ref mut e) = self.event_name_editor else {
                    return false;
                };
                if !matches!(val.as_ref(), ScalarValue::String(_))
                    || !mode.allows(e.is_present(row))
                {
                    return false;
                }
                let ScalarValue::String(s) = val.into_owned() else {
                    return false;
                };
                e.set(row, s);
                true
            }
            LogTargetSelector::RecordField(_) => false,
            LogTargetSelector::LogAttribute(AttributePath { path }) => {
                if let Some(pid) = self.get_log_id(row) {
                    return self
                        .attrs_editor
                        .as_mut()
                        .is_some_and(|ed| ed.set_path_scalar(pid, path, val, mode));
                }
                // Missing `log.id` implies no existing attributes; never allocate an ID for update-only.
                if matches!(mode, WriteMode::UpdateExistingOnly)
                    || path.is_empty()
                    || path.len() > MAX_CBOR_DEPTH
                {
                    return false;
                }
                let Some(pid) = self.get_or_alloc_log_id(row) else {
                    return false;
                };
                self.attrs_editor
                    .as_mut()
                    .is_some_and(|ed| ed.set_path_scalar(pid, path, val, mode))
            }
            LogTargetSelector::BodyField(AttributePath { path }) => {
                self.body_editor.as_mut().is_some_and(|e| {
                    e.set_body_field_with_depth(row, path, CborLeaf::Scalar(val.as_ref()), 1, mode)
                })
            }
        }
    }

    fn apply_rename_stage(
        &mut self,
        transforms: &'a [CompiledTransformPolicy],
        policy_matches: &[DenseBitset],
    ) {
        for (tx, mask) in transforms.iter().zip(policy_matches.iter()) {
            if mask.is_empty() || tx.rename.is_empty() {
                continue;
            }
            for op in &tx.rename {
                if op.from == op.to || attr_selectors_overlap(&op.from, &op.to) {
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
                    let from_key = from_path[0].as_str();
                    let to_key = to_path[0].as_str();
                    for row_u32 in mask.iter() {
                        let row = row_u32 as usize;
                        let Some(pid) = self.get_log_id(row) else {
                            continue;
                        };
                        if let Some(ref mut ed) = self.attrs_editor {
                            ed.rename_flat(pid, from_key, to_key, op.upsert);
                        }
                    }
                    continue;
                }

                // General atomic rename supporting nested LogAttribute paths and BodyField across
                // all 4 selector pairs: validate `to` writability (including moved subtree CBOR
                // depth) before removing `from` (fail-open).
                for row_u32 in mask.iter() {
                    let row = row_u32 as usize;
                    let Some(src_val) = self.get_attr_selector(row, &op.from) else {
                        continue;
                    };
                    let Some(val_depth) = src_val.cbor_depth() else {
                        continue;
                    };

                    let dest_pid = match &op.to {
                        LogAttributeSelector::LogAttribute(AttributePath { path }) => {
                            let writable = match self.get_log_id(row) {
                                Some(pid) => self.attrs_editor.as_ref().is_some_and(|ed| {
                                    ed.can_write_path(pid, path, val_depth, op.upsert)
                                }),
                                None => {
                                    let tail_len = path.len().saturating_sub(1);
                                    !path.is_empty()
                                        && ((tail_len == 0 && val_depth <= MAX_CBOR_DEPTH)
                                            || (tail_len > 0
                                                && tail_len + val_depth <= MAX_CBOR_DEPTH))
                                }
                            };
                            if !writable {
                                continue;
                            }
                            let Some(pid) = self.get_or_alloc_log_id(row) else {
                                continue;
                            };
                            Some(pid)
                        }
                        LogAttributeSelector::BodyField(AttributePath { path }) => {
                            if !self.body_editor.as_ref().is_some_and(|e| {
                                e.can_write_body_field(row, path, val_depth, op.upsert)
                            }) {
                                continue;
                            }
                            None
                        }
                    };

                    let _ = self.remove_attr_selector(row, &op.from);
                    let mode = WriteMode::from_upsert(op.upsert);
                    match &op.to {
                        LogAttributeSelector::LogAttribute(AttributePath { path }) => {
                            if let (Some(pid), Some(ed)) = (dest_pid, &mut self.attrs_editor) {
                                let _ = ed.set_path_with_depth(pid, path, src_val, val_depth, mode);
                            }
                        }
                        LogAttributeSelector::BodyField(AttributePath { path }) => {
                            if let Some(ref mut e) = self.body_editor {
                                let _ = e.set_body_field_with_depth(
                                    row,
                                    path,
                                    CborLeaf::Cell(&src_val),
                                    val_depth,
                                    mode,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    fn apply_add_stage(
        &mut self,
        transforms: &'a [CompiledTransformPolicy],
        policy_matches: &[DenseBitset],
    ) {
        for (tx, mask) in transforms.iter().zip(policy_matches.iter()) {
            if mask.is_empty() || tx.add.is_empty() {
                continue;
            }
            for op in &tx.add {
                let mode = WriteMode::from_upsert(op.upsert);
                for row_u32 in mask.iter() {
                    let _ = self.set_target_scalar(
                        row_u32 as usize,
                        &op.target,
                        Cow::Borrowed(&op.value),
                        mode,
                    );
                }
            }
        }
    }

    fn apply_remove_stage(
        &mut self,
        transforms: &'a [CompiledTransformPolicy],
        policy_matches: &[DenseBitset],
    ) {
        for (tx, mask) in transforms.iter().zip(policy_matches.iter()) {
            if mask.is_empty() || tx.remove.is_empty() {
                continue;
            }
            for op in &tx.remove {
                for row_u32 in mask.iter() {
                    let _ = self.remove_attr_selector(row_u32 as usize, &op.target);
                }
            }
        }
    }

    fn apply_redact_stage(
        &mut self,
        transforms: &'a [CompiledTransformPolicy],
        policy_matches: &[DenseBitset],
    ) {
        for (tx, mask) in transforms.iter().zip(policy_matches.iter()) {
            if mask.is_empty() || tx.redact.is_empty() {
                continue;
            }
            for op in &tx.redact {
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
                match &op.regex {
                    None => {
                        let replacement = ScalarValue::String(op.replacement.clone());
                        for row_u32 in mask.iter() {
                            let _ = self.set_target_scalar(
                                row_u32 as usize,
                                &op.target,
                                Cow::Borrowed(&replacement),
                                WriteMode::UpdateExistingOnly,
                            );
                        }
                    }
                    Some(re) => {
                        let mut caps = op.replacement.contains('$').then(|| re.create_captures());
                        for row_u32 in mask.iter() {
                            let row = row_u32 as usize;
                            let redacted = self.get_str_for_redact(row, &op.target).and_then(|s| {
                                apply_regex_redaction_with_caps(
                                    re,
                                    &s,
                                    &op.replacement,
                                    caps.as_mut(),
                                )
                            });
                            if let Some(redacted) = redacted {
                                let _ = self.set_target_scalar(
                                    row,
                                    &op.target,
                                    Cow::Owned(ScalarValue::String(redacted)),
                                    WriteMode::Upsert,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    /// Rebuilds the root `Logs` `RecordBatch` in a single pass if any root column or `log.id`
    /// changed, moving owned `mut_log_ids` without cloning.
    fn rebuild_root_batch(
        &mut self,
        root: &RecordBatch,
    ) -> Result<Option<RecordBatch>, PolicyError> {
        let schema = root.schema();
        let mut updates: Vec<(&'static str, Arc<Field>, ArrayRef)> = Vec::new();

        if let Some(ids) = self.mut_log_ids.take() {
            updates.push((
                consts::ID,
                Arc::new(Field::new(consts::ID, DataType::UInt16, true).with_plain_encoding()),
                Arc::new(UInt16Array::from(ids)),
            ));
        }
        if let Some(ref ed) = self.body_editor
            && ed.dirty
            && (schema.index_of(consts::BODY).is_ok() || ed.any_present())
        {
            let arr = ed.build_struct_array();
            updates.push((
                consts::BODY,
                Arc::new(Field::new(consts::BODY, arr.data_type().clone(), true)),
                arr,
            ));
        }
        if let Some(ref ed) = self.sev_text_editor
            && ed.dirty
            && (schema.index_of(consts::SEVERITY_TEXT).is_ok() || ed.any_present())
        {
            updates.push((
                consts::SEVERITY_TEXT,
                Arc::new(Field::new(consts::SEVERITY_TEXT, DataType::Utf8, true)),
                ed.build_array(),
            ));
        }
        if let Some(ref ed) = self.sev_num_editor
            && ed.dirty
            && (schema.index_of(consts::SEVERITY_NUMBER).is_ok() || ed.any_present())
        {
            updates.push((
                consts::SEVERITY_NUMBER,
                Arc::new(Field::new(consts::SEVERITY_NUMBER, DataType::Int32, true)),
                ed.build_array(),
            ));
        }
        if let Some(ref ed) = self.event_name_editor
            && ed.dirty
            && (schema.index_of(consts::EVENT_NAME).is_ok() || ed.any_present())
        {
            updates.push((
                consts::EVENT_NAME,
                Arc::new(Field::new(consts::EVENT_NAME, DataType::Utf8, true)),
                ed.build_array(),
            ));
        }

        if updates.is_empty() {
            return Ok(None);
        }

        let mut new_fields: Vec<Arc<Field>> =
            Vec::with_capacity(schema.fields().len() + updates.len());
        let mut new_cols: Vec<ArrayRef> = Vec::with_capacity(schema.fields().len() + updates.len());
        let mut consumed = vec![false; updates.len()];

        for (idx, field) in schema.fields().iter().enumerate() {
            if let Some(pos) = updates
                .iter()
                .position(|(name, _, _)| *name == field.name().as_str())
            {
                consumed[pos] = true;
                new_fields.push(updates[pos].1.clone());
                new_cols.push(updates[pos].2.clone());
            } else {
                new_fields.push(field.clone());
                new_cols.push(root.column(idx).clone());
            }
        }
        for ((_, field, arr), was_consumed) in updates.into_iter().zip(consumed) {
            if !was_consumed {
                new_fields.push(field);
                new_cols.push(arr);
            }
        }

        let new_root = RecordBatch::try_new(
            Arc::new(Schema::new_with_metadata(
                new_fields,
                schema.metadata().clone(),
            )),
            new_cols,
        )
        .map_err(|e| PolicyError::Evaluation(e.to_string()))?;
        Ok(Some(new_root))
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
    let mut touches_log_attrs = false;
    let mut touched_attr_keys: AHashSet<&str> = AHashSet::new();
    let mut touched_attr_rows = DenseBitset::new();

    for (tx, mask) in transforms.iter().zip(policy_matches.iter()) {
        if mask.is_empty() {
            continue;
        }
        let mut policy_touches_attrs = false;
        for sel in tx
            .rename
            .iter()
            .flat_map(|op| [&op.from, &op.to])
            .chain(tx.remove.iter().map(|op| &op.target))
        {
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
        for (target, is_redact) in tx
            .add
            .iter()
            .map(|op| (&op.target, false))
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

    let mut state = TransformState {
        num_rows,
        body_editor: touches_root_body.then(|| BodyEditor::new(&root, num_rows)),
        sev_text_editor: touches_sev_text
            .then(|| RootStrEditor::new(&root, consts::SEVERITY_TEXT, num_rows)),
        sev_num_editor: touches_sev_num
            .then(|| RootSevNumEditor::new(&root, consts::SEVERITY_NUMBER, num_rows)),
        event_name_editor: touches_event_name
            .then(|| RootStrEditor::new(&root, consts::EVENT_NAME, num_rows)),
        attrs_editor: touches_log_attrs.then(|| {
            FlatAttrsEditor::new(orig_log_attrs.as_ref(), &touched_pids, &touched_attr_keys)
        }),
        existing_log_ids,
        mut_log_ids: None,
        used_log_ids: None,
        next_log_id_candidate: 0,
    };

    state.apply_rename_stage(transforms, policy_matches);
    state.apply_add_stage(transforms, policy_matches);
    state.apply_remove_stage(transforms, policy_matches);
    state.apply_redact_stage(transforms, policy_matches);

    if let Some(new_root) = state.rebuild_root_batch(&root)? {
        batch
            .set(ArrowPayloadType::Logs, new_root)
            .map_err(|e| PolicyError::Evaluation(e.to_string()))?;
    }

    if let Some(editor) = state.attrs_editor
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
