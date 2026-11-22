// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Columnar OTAP (`OtapArrowRecords`) inspection, CBOR path navigation, and masked 4-stage
//! `LogTransformPolicy` execution (`1. Rename -> 2. Add -> 3. Remove -> 4. Redact`).

use std::borrow::Cow;
use std::cell::OnceCell;
use std::collections::BTreeMap;
use std::sync::Arc;

use arrow::array::{
    Array, ArrayRef, BinaryArray, BooleanArray, DictionaryArray, FixedSizeBinaryArray,
    Float64Array, Int32Array, Int64Array, RecordBatch, StringArray, StructArray, UInt8Array,
    UInt16Array, UInt32Array,
};
use arrow::buffer::NullBuffer;
use arrow::datatypes::{DataType, Field, Schema, UInt8Type, UInt16Type};
use otel_arrow_dfe_pdata::arrays::{MaybeDictArrayAccessor, NullableArrayAccessor};
use otel_arrow_dfe_pdata::otap::OtapArrowRecords;
use otel_arrow_dfe_pdata::otap::filter::{IdBitmapPool, filter_otap_batch};
use otel_arrow_dfe_pdata::otlp::attributes::AttributeValueType;
use otel_arrow_dfe_pdata::otlp::common::AnyValueArrays;
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_dfe_pdata::schema::{FieldExt, consts};
use regex_automata::meta::Regex as MetaRegex;

use crate::bitset::DenseBitset;

use crate::model::{
    AttributePath, LogAdd, LogAttributeSelector, LogRecordField, LogRemove, LogRename,
    LogTargetSelector, PolicyError, ScalarValue, apply_regex_redaction, eq_i64_f64,
};
#[cfg(test)]
use crate::model::{LogFieldSelector, LogTransformPolicy, ScopeField, compile_redact_regex};

/// Decoded scalar or array value extracted from an OTAP column or CBOR `ATTRIBUTE_SER` payload.
#[derive(Debug, Clone, PartialEq)]
pub enum ExtractedValue<'a> {
    /// UTF-8 string value.
    Str(Cow<'a, str>),
    /// 64-bit signed integer value.
    Int(i64),
    /// 64-bit float value.
    Double(f64),
    /// Boolean value.
    Bool(bool),
    /// Raw byte slice.
    Bytes(Cow<'a, [u8]>),
    /// Array of scalar values (from CBOR `AttributeValueType::Slice`).
    Array(Vec<ExtractedValue<'a>>),
    /// Map of key-value pairs (from CBOR `AttributeValueType::Map`).
    Map(Vec<(Cow<'a, str>, ExtractedValue<'a>)>),
}

impl<'a> ExtractedValue<'a> {
    /// Converts borrowed data into `'static`.
    #[cfg(test)]
    #[must_use]
    pub fn into_owned(self) -> ExtractedValue<'static> {
        match self {
            Self::Str(s) => ExtractedValue::Str(Cow::Owned(s.into_owned())),
            Self::Int(i) => ExtractedValue::Int(i),
            Self::Double(d) => ExtractedValue::Double(d),
            Self::Bool(b) => ExtractedValue::Bool(b),
            Self::Bytes(b) => ExtractedValue::Bytes(Cow::Owned(b.into_owned())),
            Self::Array(v) => ExtractedValue::Array(v.into_iter().map(Self::into_owned).collect()),
            Self::Map(m) => ExtractedValue::Map(
                m.into_iter()
                    .map(|(k, v)| (Cow::Owned(k.into_owned()), v.into_owned()))
                    .collect(),
            ),
        }
    }

    /// Checks exact equality against a policy `ScalarValue` (`policy.md` "Typed Values").
    #[must_use]
    pub fn equals_scalar(&self, target: &ScalarValue) -> bool {
        match (self, target) {
            (Self::Str(a), ScalarValue::String(b)) => a.as_ref() == b.as_str(),
            (Self::Int(a), ScalarValue::Int(b)) => *a == *b,
            (Self::Double(a), ScalarValue::Double(b)) => !a.is_nan() && !b.is_nan() && *a == *b,
            (Self::Int(a), ScalarValue::Double(b)) => eq_i64_f64(*a, *b),
            (Self::Double(a), ScalarValue::Int(b)) => eq_i64_f64(*b, *a),
            (Self::Bool(a), ScalarValue::Bool(b)) => *a == *b,
            (Self::Bytes(a), ScalarValue::Bytes(b)) => a.as_ref() == b.as_slice(),
            _ => false,
        }
    }

    /// Checks `contains` against a policy `ScalarValue` (`policy.md` "contains").
    #[must_use]
    pub fn contains_scalar(&self, target: &ScalarValue) -> bool {
        match (self, target) {
            (Self::Str(a), ScalarValue::String(b)) => a.contains(b.as_str()),
            (Self::Bytes(a), ScalarValue::Bytes(b)) => {
                if b.is_empty() {
                    true
                } else {
                    memchr::memmem::find(a.as_ref(), b.as_slice()).is_some()
                }
            }
            (Self::Array(elems), scalar) => elems.iter().any(|e| e.equals_scalar(scalar)),
            _ => false,
        }
    }
}

/// Formats a 16-byte `trace_id` or 8-byte `span_id` into lowercase hex, returning `None` if all zeros.
#[cfg(test)]
#[inline]
#[must_use]
pub fn format_non_zero_hex(bytes: &[u8]) -> Option<String> {
    if bytes.iter().all(|&b| b == 0) {
        return None;
    }
    Some(hex::encode(bytes))
}

/// Maximum nesting depth permitted while streaming or decoding CBOR payloads.
const MAX_CBOR_DEPTH: usize = 64;

enum CborArg {
    Val(u64),
    Indefinite,
}

#[derive(Clone, Copy)]
enum CborDecodeMode {
    /// Decode scalars, arrays, and maps in full.
    Full,
    /// Decode scalars and top-level array elements; skip map and nested container interiors.
    EvalWithArrays,
    /// Decode scalars only; skip array and map interiors without allocating.
    EvalScalarsOnly,
}

/// Zero-copy streaming RFC 8949 CBOR cursor over an `ATTRIBUTE_SER` byte slice.
///
/// Navigates map paths directly over `bytes` without allocating skipped sibling keys,
/// values, or subtrees, and borrows definite-length leaf strings and byte slices directly
/// from the Arrow buffer.
struct CborCursor<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> CborCursor<'a> {
    #[inline]
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    #[inline]
    const fn remaining_len(&self) -> usize {
        self.bytes.len().saturating_sub(self.pos)
    }

    #[inline]
    fn peek_byte(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    #[inline]
    fn read_byte(&mut self) -> Result<u8, ()> {
        let &b = self.bytes.get(self.pos).ok_or(())?;
        self.pos += 1;
        Ok(b)
    }

    #[inline]
    fn take_bytes(&mut self, len: usize) -> Result<&'a [u8], ()> {
        let end = self.pos.checked_add(len).ok_or(())?;
        let slice = self.bytes.get(self.pos..end).ok_or(())?;
        self.pos = end;
        Ok(slice)
    }

    fn read_arg(&mut self, ai: u8) -> Result<CborArg, ()> {
        match ai {
            0..=23 => Ok(CborArg::Val(u64::from(ai))),
            24 => Ok(CborArg::Val(u64::from(self.read_byte()?))),
            25 => {
                let b: [u8; 2] = self.take_bytes(2)?.try_into().map_err(|_| ())?;
                Ok(CborArg::Val(u64::from(u16::from_be_bytes(b))))
            }
            26 => {
                let b: [u8; 4] = self.take_bytes(4)?.try_into().map_err(|_| ())?;
                Ok(CborArg::Val(u64::from(u32::from_be_bytes(b))))
            }
            27 => {
                let b: [u8; 8] = self.take_bytes(8)?.try_into().map_err(|_| ())?;
                Ok(CborArg::Val(u64::from_be_bytes(b)))
            }
            31 => Ok(CborArg::Indefinite),
            _ => Err(()),
        }
    }

    fn read_bytes_item(&mut self, ai: u8) -> Result<Cow<'a, [u8]>, ()> {
        match self.read_arg(ai)? {
            CborArg::Val(n) => {
                let len = usize::try_from(n).map_err(|_| ())?;
                Ok(Cow::Borrowed(self.take_bytes(len)?))
            }
            CborArg::Indefinite => {
                let mut buf = Vec::new();
                loop {
                    match self.peek_byte() {
                        Some(0xff) => {
                            self.pos += 1;
                            return Ok(Cow::Owned(buf));
                        }
                        Some(_) => {
                            let cib = self.read_byte()?;
                            if (cib >> 5) != 2 {
                                return Err(());
                            }
                            let CborArg::Val(n) = self.read_arg(cib & 0x1f)? else {
                                return Err(());
                            };
                            let len = usize::try_from(n).map_err(|_| ())?;
                            buf.extend_from_slice(self.take_bytes(len)?);
                        }
                        None => return Err(()),
                    }
                }
            }
        }
    }

    fn read_text_item(&mut self, ai: u8) -> Result<Cow<'a, str>, ()> {
        match self.read_arg(ai)? {
            CborArg::Val(n) => {
                let len = usize::try_from(n).map_err(|_| ())?;
                let slice = self.take_bytes(len)?;
                let s = std::str::from_utf8(slice).map_err(|_| ())?;
                Ok(Cow::Borrowed(s))
            }
            CborArg::Indefinite => {
                let mut buf = String::new();
                loop {
                    match self.peek_byte() {
                        Some(0xff) => {
                            self.pos += 1;
                            return Ok(Cow::Owned(buf));
                        }
                        Some(_) => {
                            let cib = self.read_byte()?;
                            if (cib >> 5) != 3 {
                                return Err(());
                            }
                            let CborArg::Val(n) = self.read_arg(cib & 0x1f)? else {
                                return Err(());
                            };
                            let len = usize::try_from(n).map_err(|_| ())?;
                            let slice = self.take_bytes(len)?;
                            let s = std::str::from_utf8(slice).map_err(|_| ())?;
                            buf.push_str(s);
                        }
                        None => return Err(()),
                    }
                }
            }
        }
    }

    fn skip_string_chunks(&mut self, major: u8, ai: u8) -> Result<(), ()> {
        match self.read_arg(ai)? {
            CborArg::Val(n) => {
                let len = usize::try_from(n).map_err(|_| ())?;
                let slice = self.take_bytes(len)?;
                if major == 3 {
                    let _ = std::str::from_utf8(slice).map_err(|_| ())?;
                }
                Ok(())
            }
            CborArg::Indefinite => loop {
                match self.peek_byte() {
                    Some(0xff) => {
                        self.pos += 1;
                        return Ok(());
                    }
                    Some(_) => {
                        let cib = self.read_byte()?;
                        if (cib >> 5) != major {
                            return Err(());
                        }
                        let CborArg::Val(n) = self.read_arg(cib & 0x1f)? else {
                            return Err(());
                        };
                        let len = usize::try_from(n).map_err(|_| ())?;
                        let slice = self.take_bytes(len)?;
                        if major == 3 {
                            let _ = std::str::from_utf8(slice).map_err(|_| ())?;
                        }
                    }
                    None => return Err(()),
                }
            },
        }
    }

    fn skip_array_body(&mut self, ai: u8, depth: usize) -> Result<(), ()> {
        match self.read_arg(ai)? {
            CborArg::Val(count) => {
                for _ in 0..count {
                    self.skip_value(depth + 1)?;
                }
                Ok(())
            }
            CborArg::Indefinite => loop {
                match self.peek_byte() {
                    Some(0xff) => {
                        self.pos += 1;
                        return Ok(());
                    }
                    Some(_) => self.skip_value(depth + 1)?,
                    None => return Err(()),
                }
            },
        }
    }

    fn skip_map_body(&mut self, ai: u8, depth: usize) -> Result<(), ()> {
        match self.read_arg(ai)? {
            CborArg::Val(pairs) => {
                for _ in 0..pairs {
                    self.skip_value(depth + 1)?;
                    self.skip_value(depth + 1)?;
                }
                Ok(())
            }
            CborArg::Indefinite => loop {
                match self.peek_byte() {
                    Some(0xff) => {
                        self.pos += 1;
                        return Ok(());
                    }
                    Some(_) => {
                        self.skip_value(depth + 1)?;
                        self.skip_value(depth + 1)?;
                    }
                    None => return Err(()),
                }
            },
        }
    }

    fn skip_value(&mut self, depth: usize) -> Result<(), ()> {
        if depth >= MAX_CBOR_DEPTH {
            return Err(());
        }
        let ib = self.read_byte()?;
        let major = ib >> 5;
        let ai = ib & 0x1f;
        match major {
            0 | 1 => match self.read_arg(ai)? {
                CborArg::Val(_) => Ok(()),
                CborArg::Indefinite => Err(()),
            },
            2 | 3 => self.skip_string_chunks(major, ai),
            4 => self.skip_array_body(ai, depth),
            5 => self.skip_map_body(ai, depth),
            6 => match self.read_arg(ai)? {
                CborArg::Val(_) => self.skip_value(depth + 1),
                CborArg::Indefinite => Err(()),
            },
            7 => match ai {
                0..=23 => Ok(()),
                24 => {
                    let v = self.read_byte()?;
                    if v < 32 { Err(()) } else { Ok(()) }
                }
                25 => {
                    let _ = self.take_bytes(2)?;
                    Ok(())
                }
                26 => {
                    let _ = self.take_bytes(4)?;
                    Ok(())
                }
                27 => {
                    let _ = self.take_bytes(8)?;
                    Ok(())
                }
                _ => Err(()),
            },
            _ => Err(()),
        }
    }

    /// Compares a CBOR map key against `target` without allocating, skipping non-text keys.
    fn match_text_key(&mut self, target: &str, depth: usize) -> Result<bool, ()> {
        let ib = self.peek_byte().ok_or(())?;
        if (ib >> 5) != 3 {
            self.skip_value(depth)?;
            return Ok(false);
        }
        let _ = self.read_byte()?;
        let ai = ib & 0x1f;
        match self.read_arg(ai)? {
            CborArg::Val(n) => {
                let len = usize::try_from(n).map_err(|_| ())?;
                let slice = self.take_bytes(len)?;
                let s = std::str::from_utf8(slice).map_err(|_| ())?;
                Ok(s == target)
            }
            CborArg::Indefinite => {
                let target_bytes = target.as_bytes();
                let mut offset = 0usize;
                let mut matches = true;
                loop {
                    match self.peek_byte() {
                        Some(0xff) => {
                            self.pos += 1;
                            return Ok(matches && offset == target_bytes.len());
                        }
                        Some(_) => {
                            let cib = self.read_byte()?;
                            if (cib >> 5) != 3 {
                                return Err(());
                            }
                            let CborArg::Val(n) = self.read_arg(cib & 0x1f)? else {
                                return Err(());
                            };
                            let len = usize::try_from(n).map_err(|_| ())?;
                            let slice = self.take_bytes(len)?;
                            let _ = std::str::from_utf8(slice).map_err(|_| ())?;
                            if matches {
                                if let Some(end) = offset.checked_add(slice.len())
                                    && target_bytes.get(offset..end) == Some(slice)
                                {
                                    offset = end;
                                } else {
                                    matches = false;
                                }
                            }
                        }
                        None => return Err(()),
                    }
                }
            }
        }
    }

    fn decode_value(
        &mut self,
        depth: usize,
        mode: CborDecodeMode,
    ) -> Result<Option<ExtractedValue<'a>>, ()> {
        if depth >= MAX_CBOR_DEPTH {
            return Err(());
        }
        let ib = self.read_byte()?;
        let major = ib >> 5;
        let ai = ib & 0x1f;
        match major {
            0 => match self.read_arg(ai)? {
                CborArg::Val(n) => Ok(i64::try_from(n).ok().map(ExtractedValue::Int)),
                CborArg::Indefinite => Err(()),
            },
            1 => match self.read_arg(ai)? {
                CborArg::Val(n) => {
                    let val = -1i128 - i128::from(n);
                    Ok(i64::try_from(val).ok().map(ExtractedValue::Int))
                }
                CborArg::Indefinite => Err(()),
            },
            2 => Ok(Some(ExtractedValue::Bytes(self.read_bytes_item(ai)?))),
            3 => Ok(Some(ExtractedValue::Str(self.read_text_item(ai)?))),
            4 => {
                if matches!(mode, CborDecodeMode::EvalScalarsOnly) {
                    self.skip_array_body(ai, depth)?;
                    return Ok(Some(ExtractedValue::Array(Vec::new())));
                }
                let elem_mode = match mode {
                    CborDecodeMode::Full => CborDecodeMode::Full,
                    CborDecodeMode::EvalWithArrays | CborDecodeMode::EvalScalarsOnly => {
                        CborDecodeMode::EvalScalarsOnly
                    }
                };
                match self.read_arg(ai)? {
                    CborArg::Val(count) => {
                        let cap = usize::try_from(count)
                            .unwrap_or(usize::MAX)
                            .min(self.remaining_len());
                        let mut out = Vec::with_capacity(cap);
                        for _ in 0..count {
                            if let Some(ev) = self.decode_value(depth + 1, elem_mode)? {
                                out.push(ev);
                            }
                        }
                        Ok(Some(ExtractedValue::Array(out)))
                    }
                    CborArg::Indefinite => {
                        let mut out = Vec::new();
                        loop {
                            match self.peek_byte() {
                                Some(0xff) => {
                                    self.pos += 1;
                                    return Ok(Some(ExtractedValue::Array(out)));
                                }
                                Some(_) => {
                                    if let Some(ev) = self.decode_value(depth + 1, elem_mode)? {
                                        out.push(ev);
                                    }
                                }
                                None => return Err(()),
                            }
                        }
                    }
                }
            }
            5 => {
                if !matches!(mode, CborDecodeMode::Full) {
                    self.skip_map_body(ai, depth)?;
                    return Ok(Some(ExtractedValue::Map(Vec::new())));
                }
                match self.read_arg(ai)? {
                    CborArg::Val(pairs) => {
                        let cap = usize::try_from(pairs)
                            .unwrap_or(usize::MAX)
                            .min(self.remaining_len() / 2);
                        let mut out = Vec::with_capacity(cap);
                        for _ in 0..pairs {
                            let kib = self.peek_byte().ok_or(())?;
                            if (kib >> 5) == 3 {
                                let _ = self.read_byte()?;
                                let key = self.read_text_item(kib & 0x1f)?;
                                if let Some(val) =
                                    self.decode_value(depth + 1, CborDecodeMode::Full)?
                                {
                                    out.push((key, val));
                                }
                            } else {
                                self.skip_value(depth + 1)?;
                                self.skip_value(depth + 1)?;
                            }
                        }
                        Ok(Some(ExtractedValue::Map(out)))
                    }
                    CborArg::Indefinite => {
                        let mut out = Vec::new();
                        loop {
                            match self.peek_byte() {
                                Some(0xff) => {
                                    self.pos += 1;
                                    return Ok(Some(ExtractedValue::Map(out)));
                                }
                                Some(kib) => {
                                    if (kib >> 5) == 3 {
                                        let _ = self.read_byte()?;
                                        let key = self.read_text_item(kib & 0x1f)?;
                                        if let Some(val) =
                                            self.decode_value(depth + 1, CborDecodeMode::Full)?
                                        {
                                            out.push((key, val));
                                        }
                                    } else {
                                        self.skip_value(depth + 1)?;
                                        self.skip_value(depth + 1)?;
                                    }
                                }
                                None => return Err(()),
                            }
                        }
                    }
                }
            }
            6 => match self.read_arg(ai)? {
                CborArg::Val(_) => {
                    self.skip_value(depth + 1)?;
                    Ok(None)
                }
                CborArg::Indefinite => Err(()),
            },
            7 => match ai {
                0..=19 | 22 | 23 => Ok(None),
                20 => Ok(Some(ExtractedValue::Bool(false))),
                21 => Ok(Some(ExtractedValue::Bool(true))),
                24 => {
                    let v = self.read_byte()?;
                    if v < 32 { Err(()) } else { Ok(None) }
                }
                25 => {
                    let b: [u8; 2] = self.take_bytes(2)?.try_into().map_err(|_| ())?;
                    Ok(Some(ExtractedValue::Double(decode_cbor_f16(
                        u16::from_be_bytes(b),
                    ))))
                }
                26 => {
                    let b: [u8; 4] = self.take_bytes(4)?.try_into().map_err(|_| ())?;
                    Ok(Some(ExtractedValue::Double(f64::from(f32::from_be_bytes(
                        b,
                    )))))
                }
                27 => {
                    let b: [u8; 8] = self.take_bytes(8)?.try_into().map_err(|_| ())?;
                    Ok(Some(ExtractedValue::Double(f64::from_be_bytes(b))))
                }
                _ => Err(()),
            },
            _ => Err(()),
        }
    }

    fn extract_path(
        &mut self,
        segments: &[String],
        depth: usize,
        mode: CborDecodeMode,
    ) -> Result<Option<ExtractedValue<'a>>, ()> {
        let Some((seg, rest)) = segments.split_first() else {
            return self.decode_value(depth, mode);
        };
        if depth >= MAX_CBOR_DEPTH {
            return Err(());
        }
        let ib = self.read_byte()?;
        if (ib >> 5) != 5 {
            return Ok(None);
        }
        match self.read_arg(ib & 0x1f)? {
            CborArg::Val(pairs) => {
                for _ in 0..pairs {
                    if self.match_text_key(seg, depth + 1)? {
                        return self.extract_path(rest, depth + 1, mode);
                    }
                    self.skip_value(depth + 1)?;
                }
                Ok(None)
            }
            CborArg::Indefinite => loop {
                match self.peek_byte() {
                    Some(0xff) => {
                        self.pos += 1;
                        return Ok(None);
                    }
                    Some(_) => {
                        if self.match_text_key(seg, depth + 1)? {
                            return self.extract_path(rest, depth + 1, mode);
                        }
                        self.skip_value(depth + 1)?;
                    }
                    None => return Err(()),
                }
            },
        }
    }
}

fn decode_cbor_f16(bits: u16) -> f64 {
    let sign = (bits & 0x8000) != 0;
    let exp = (bits >> 10) & 0x1f;
    let mant = bits & 0x03ff;

    if exp == 0 {
        let val = if mant == 0 {
            0.0
        } else {
            // Subnormal binary16: mant * 2^-24 (f64 biased exponent 1023 - 24 = 999).
            f64::from(mant) * f64::from_bits(999u64 << 52)
        };
        if sign { -val } else { val }
    } else if exp == 31 {
        if mant == 0 {
            if sign {
                f64::NEG_INFINITY
            } else {
                f64::INFINITY
            }
        } else {
            f64::NAN
        }
    } else {
        let f64_sign = u64::from(bits & 0x8000) << 48;
        let f64_exp = (u64::from(exp) + 1008) << 52;
        let f64_mant = u64::from(mant) << 42;
        f64::from_bits(f64_sign | f64_exp | f64_mant)
    }
}

/// Decodes a CBOR byte slice (`ATTRIBUTE_SER`) into a zero-copy `ExtractedValue<'_>`.
#[must_use]
pub fn decode_cbor_value(bytes: &[u8]) -> Option<ExtractedValue<'_>> {
    extract_cbor_path(bytes, &[])
}

/// Streams a CBOR byte slice (`ATTRIBUTE_SER`) along `segments` without allocating skipped
/// sibling keys, values, or subtrees, returning the decoded leaf value.
#[must_use]
pub fn extract_cbor_path<'a>(bytes: &'a [u8], segments: &[String]) -> Option<ExtractedValue<'a>> {
    let mut cursor = CborCursor::new(bytes);
    cursor
        .extract_path(segments, 0, CborDecodeMode::Full)
        .ok()
        .flatten()
}

/// Streams a CBOR byte slice along `segments` for predicate evaluation, skipping map interiors
/// and (when `decode_array_elements` is `false`) array interiors without allocating.
#[must_use]
pub(crate) fn extract_cbor_path_for_eval<'a>(
    bytes: &'a [u8],
    segments: &[String],
    decode_array_elements: bool,
) -> Option<ExtractedValue<'a>> {
    let mode = if decode_array_elements {
        CborDecodeMode::EvalWithArrays
    } else {
        CborDecodeMode::EvalScalarsOnly
    };
    let mut cursor = CborCursor::new(bytes);
    cursor.extract_path(segments, 0, mode).ok().flatten()
}

/// Navigates a nested `AttributePath` (`segments`) inside a decoded `ExtractedValue::Map`.
#[cfg(test)]
#[must_use]
pub fn navigate_extracted_path<'a>(
    mut current: ExtractedValue<'a>,
    segments: &[String],
) -> Option<ExtractedValue<'a>> {
    for seg in segments {
        match current {
            ExtractedValue::Map(entries) => {
                let found = entries.into_iter().find(|(k, _)| k.as_ref() == seg)?.1;
                current = found;
            }
            _ => return None,
        }
    }
    Some(current)
}

/// Lazily indexed parent-to-row mapping for a `Logs` `OtapArrowRecords` batch.
///
/// `LogAttrs` uses a lazily populated `log.id -> row_idx` vector, while `ResourceAttrs` and
/// `ScopeAttrs` are evaluated in resource/scope ID space and checked directly against the
/// `resource.id` / `scope.id` columns without allocating per-ID row bitmaps.
pub struct LogsBatchView<'a> {
    /// Borrowed `OtapArrowRecords`.
    pub records: &'a OtapArrowRecords,
    /// Root `Logs` `RecordBatch`.
    pub root: &'a RecordBatch,
    /// Number of log rows in `root`.
    pub num_rows: usize,
    log_id_to_row: OnceCell<Vec<u32>>,
}

impl<'a> LogsBatchView<'a> {
    /// Constructs a `LogsBatchView` from an `OtapArrowRecords` batch without scanning ID columns
    /// until an active policy inspects `LogAttrs`.
    pub fn try_new(records: &'a OtapArrowRecords) -> Result<Option<Self>, PolicyError> {
        let Some(root) = records.get(ArrowPayloadType::Logs) else {
            return Ok(None);
        };
        let num_rows = root.num_rows();
        if num_rows == 0 {
            return Ok(None);
        }

        Ok(Some(Self {
            records,
            root,
            num_rows,
            log_id_to_row: OnceCell::new(),
        }))
    }

    /// Lazily maps `log.id` (`u16`) -> row index (`0..num_rows`), or `u32::MAX` if unmapped.
    #[must_use]
    pub fn log_id_to_row(&self) -> &[u32] {
        self.log_id_to_row.get_or_init(|| {
            let mut map = Vec::new();
            if let Some(id_col) = self.root.column_by_name(consts::ID)
                && let Some(id_arr) = id_col.as_any().downcast_ref::<UInt16Array>()
            {
                let mut max_id: Option<usize> = None;
                for row in 0..self.num_rows {
                    if id_arr.is_valid(row) {
                        let id = id_arr.value(row) as usize;
                        max_id = Some(max_id.map_or(id, |m| m.max(id)));
                    }
                }
                if let Some(max_id) = max_id {
                    map.resize(max_id + 1, u32::MAX);
                    for row in 0..self.num_rows {
                        if id_arr.is_valid(row) {
                            map[id_arr.value(row) as usize] = row as u32;
                        }
                    }
                }
            }
            map
        })
    }

    /// Borrows the `(resource StructArray, resource.id UInt16Array)` columns if present.
    #[must_use]
    pub fn resource_id_column(&self) -> Option<(&'a StructArray, &'a UInt16Array)> {
        let res_col = self.root.column_by_name(consts::RESOURCE)?;
        let res_struct = res_col.as_any().downcast_ref::<StructArray>()?;
        let res_id_col = res_struct.column_by_name(consts::ID)?;
        let res_id_arr = res_id_col.as_any().downcast_ref::<UInt16Array>()?;
        Some((res_struct, res_id_arr))
    }

    /// Borrows the `(scope StructArray, scope.id UInt16Array)` columns if present.
    #[must_use]
    pub fn scope_id_column(&self) -> Option<(&'a StructArray, &'a UInt16Array)> {
        let scope_col = self.root.column_by_name(consts::SCOPE)?;
        let scope_struct = scope_col.as_any().downcast_ref::<StructArray>()?;
        let scope_id_col = scope_struct.column_by_name(consts::ID)?;
        let scope_id_arr = scope_id_col.as_any().downcast_ref::<UInt16Array>()?;
        Some((scope_struct, scope_id_arr))
    }

    #[cfg(test)]
    pub(crate) fn is_log_index_built(&self) -> bool {
        self.log_id_to_row.get().is_some()
    }
}

#[cfg(test)]
impl<'a> LogsBatchView<'a> {
    /// Extracts the value of `field` at a specific log row `row` (`0..num_rows`), if present.
    #[must_use]
    pub fn extract_field_at_row(
        &self,
        field: &LogFieldSelector,
        row: usize,
    ) -> Option<ExtractedValue<'a>> {
        match field {
            LogFieldSelector::RecordField(rf) => self.extract_record_field_at_row(*rf, row),
            LogFieldSelector::ScopeField(sf) => self.extract_scope_field_at_row(*sf, row),
            LogFieldSelector::BodyField(path) => {
                if path.path.is_empty() {
                    return None;
                }
                let val = self.extract_record_field_at_row(LogRecordField::Body, row)?;
                navigate_extracted_path(val, &path.path)
            }
            LogFieldSelector::LogAttribute(path) => {
                let log_id = self.row_to_log_id(row)?;
                let attrs = self.records.get(ArrowPayloadType::LogAttrs)?;
                extract_attr_from_batch(attrs, log_id, &path.path)
            }
            LogFieldSelector::ResourceAttribute(path) => {
                let res_id = self.row_to_resource_id(row)?;
                let attrs = self.records.get(ArrowPayloadType::ResourceAttrs)?;
                extract_attr_from_batch(attrs, res_id, &path.path)
            }
            LogFieldSelector::ScopeAttribute(path) => {
                let scope_id = self.row_to_scope_id(row)?;
                let attrs = self.records.get(ArrowPayloadType::ScopeAttrs)?;
                extract_attr_from_batch(attrs, scope_id, &path.path)
            }
        }
    }

    /// Returns `log.id` for `row`, or `None` if `id` is null or the `id` column is absent.
    #[inline]
    #[must_use]
    pub fn row_to_log_id(&self, row: usize) -> Option<u16> {
        let id_col = self.root.column_by_name(consts::ID)?;
        let id_arr = id_col.as_any().downcast_ref::<UInt16Array>()?;
        id_arr.is_valid(row).then(|| id_arr.value(row))
    }

    /// Returns `resource.id` for `row`.
    #[inline]
    #[must_use]
    pub fn row_to_resource_id(&self, row: usize) -> Option<u16> {
        let res_col = self.root.column_by_name(consts::RESOURCE)?;
        let res_struct = res_col.as_any().downcast_ref::<StructArray>()?;
        if !res_struct.is_valid(row) {
            return None;
        }
        let id_col = res_struct.column_by_name(consts::ID)?;
        let id_arr = id_col.as_any().downcast_ref::<UInt16Array>()?;
        id_arr.is_valid(row).then(|| id_arr.value(row))
    }

    /// Returns `scope.id` for `row`.
    #[inline]
    #[must_use]
    pub fn row_to_scope_id(&self, row: usize) -> Option<u16> {
        let scope_col = self.root.column_by_name(consts::SCOPE)?;
        let scope_struct = scope_col.as_any().downcast_ref::<StructArray>()?;
        if !scope_struct.is_valid(row) {
            return None;
        }
        let id_col = scope_struct.column_by_name(consts::ID)?;
        let id_arr = id_col.as_any().downcast_ref::<UInt16Array>()?;
        id_arr.is_valid(row).then(|| id_arr.value(row))
    }

    fn extract_record_field_at_row(
        &self,
        rf: LogRecordField,
        row: usize,
    ) -> Option<ExtractedValue<'a>> {
        match rf {
            LogRecordField::Unspecified => None,
            LogRecordField::Body => {
                let body_col = self.root.column_by_name(consts::BODY)?;
                if !body_col.is_valid(row) {
                    return None;
                }
                if let Some(body_struct) = body_col.as_any().downcast_ref::<StructArray>() {
                    extract_anyval_from_struct(body_struct, row)
                } else {
                    extract_str_from_array(body_col, row)
                        .map(|s| ExtractedValue::Str(Cow::Borrowed(s)))
                }
            }
            LogRecordField::SeverityText => {
                let col = self.root.column_by_name(consts::SEVERITY_TEXT)?;
                let s = extract_str_from_array(col, row)?;
                if s.is_empty() {
                    None
                } else {
                    Some(ExtractedValue::Str(Cow::Borrowed(s)))
                }
            }
            LogRecordField::EventName => {
                let col = self.root.column_by_name(consts::EVENT_NAME)?;
                let s = extract_str_from_array(col, row)?;
                if s.is_empty() {
                    None
                } else {
                    Some(ExtractedValue::Str(Cow::Borrowed(s)))
                }
            }
            LogRecordField::SeverityNumber => {
                let col = self.root.column_by_name(consts::SEVERITY_NUMBER)?;
                let v = extract_i64_from_array(col, row)?;
                if v == 0 {
                    None
                } else {
                    Some(ExtractedValue::Int(v))
                }
            }
            LogRecordField::TraceId => {
                let col = self.root.column_by_name(consts::TRACE_ID)?;
                let bytes = extract_fixed_bytes_from_array(col, row)?;
                let hex_str = format_non_zero_hex(bytes)?;
                Some(ExtractedValue::Str(Cow::Owned(hex_str)))
            }
            LogRecordField::SpanId => {
                let col = self.root.column_by_name(consts::SPAN_ID)?;
                let bytes = extract_fixed_bytes_from_array(col, row)?;
                let hex_str = format_non_zero_hex(bytes)?;
                Some(ExtractedValue::Str(Cow::Owned(hex_str)))
            }
        }
    }

    fn extract_scope_field_at_row(&self, sf: ScopeField, row: usize) -> Option<ExtractedValue<'a>> {
        match sf {
            ScopeField::Unspecified => None,
            ScopeField::SchemaUrl => {
                let col = self.root.column_by_name(consts::SCHEMA_URL)?;
                let s = extract_str_from_array(col, row)?;
                (!s.is_empty()).then_some(ExtractedValue::Str(Cow::Borrowed(s)))
            }
            ScopeField::Name => {
                let scope_col = self.root.column_by_name(consts::SCOPE)?;
                let scope_struct = scope_col.as_any().downcast_ref::<StructArray>()?;
                if !scope_struct.is_valid(row) {
                    return None;
                }
                let col = scope_struct.column_by_name(consts::NAME)?;
                let s = extract_str_from_array(col, row)?;
                (!s.is_empty()).then_some(ExtractedValue::Str(Cow::Borrowed(s)))
            }
            ScopeField::Version => {
                let scope_col = self.root.column_by_name(consts::SCOPE)?;
                let scope_struct = scope_col.as_any().downcast_ref::<StructArray>()?;
                if !scope_struct.is_valid(row) {
                    return None;
                }
                let col = scope_struct.column_by_name(consts::VERSION)?;
                let s = extract_str_from_array(col, row)?;
                (!s.is_empty()).then_some(ExtractedValue::Str(Cow::Borrowed(s)))
            }
        }
    }
}

/// Extracts a `&str` at `row` from a `StringArray` or `DictionaryArray<UInt8 | UInt16, StringArray>`.
#[inline]
#[must_use]
pub fn extract_str_from_array(col: &ArrayRef, row: usize) -> Option<&str> {
    if !col.is_valid(row) {
        return None;
    }
    if let Some(sa) = col.as_any().downcast_ref::<StringArray>() {
        return Some(sa.value(row));
    }
    if let Some(da) = col.as_any().downcast_ref::<DictionaryArray<UInt8Type>>() {
        let k = da.keys().value(row) as usize;
        let vals = da.values().as_any().downcast_ref::<StringArray>()?;
        return vals.is_valid(k).then(|| vals.value(k));
    }
    if let Some(da) = col.as_any().downcast_ref::<DictionaryArray<UInt16Type>>() {
        let k = da.keys().value(row) as usize;
        let vals = da.values().as_any().downcast_ref::<StringArray>()?;
        return vals.is_valid(k).then(|| vals.value(k));
    }
    None
}

/// Extracts an `i64` at `row` from `Int32Array`, `Int64Array`, or `DictionaryArray<UInt8 | UInt16, Int32 | Int64>`.
#[inline]
#[must_use]
pub fn extract_i64_from_array(col: &ArrayRef, row: usize) -> Option<i64> {
    if !col.is_valid(row) {
        return None;
    }
    if let Some(a) = col.as_any().downcast_ref::<Int64Array>() {
        return Some(a.value(row));
    }
    if let Some(a) = col.as_any().downcast_ref::<Int32Array>() {
        return Some(i64::from(a.value(row)));
    }
    if let Some(da) = col.as_any().downcast_ref::<DictionaryArray<UInt8Type>>() {
        let k = da.keys().value(row) as usize;
        if let Some(vals) = da.values().as_any().downcast_ref::<Int64Array>() {
            return vals.is_valid(k).then(|| vals.value(k));
        }
        if let Some(vals) = da.values().as_any().downcast_ref::<Int32Array>() {
            return vals.is_valid(k).then(|| i64::from(vals.value(k)));
        }
    }
    if let Some(da) = col.as_any().downcast_ref::<DictionaryArray<UInt16Type>>() {
        let k = da.keys().value(row) as usize;
        if let Some(vals) = da.values().as_any().downcast_ref::<Int64Array>() {
            return vals.is_valid(k).then(|| vals.value(k));
        }
        if let Some(vals) = da.values().as_any().downcast_ref::<Int32Array>() {
            return vals.is_valid(k).then(|| i64::from(vals.value(k)));
        }
    }
    None
}

/// Extracts a `&[u8]` at `row` from a `BinaryArray` or `DictionaryArray<UInt8 | UInt16, BinaryArray>`.
#[cfg(test)]
#[inline]
#[must_use]
pub fn extract_bytes_from_array(col: &ArrayRef, row: usize) -> Option<&[u8]> {
    if !col.is_valid(row) {
        return None;
    }
    if let Some(ba) = col.as_any().downcast_ref::<BinaryArray>() {
        return Some(ba.value(row));
    }
    if let Some(da) = col.as_any().downcast_ref::<DictionaryArray<UInt8Type>>() {
        let k = da.keys().value(row) as usize;
        let vals = da.values().as_any().downcast_ref::<BinaryArray>()?;
        return vals.is_valid(k).then(|| vals.value(k));
    }
    if let Some(da) = col.as_any().downcast_ref::<DictionaryArray<UInt16Type>>() {
        let k = da.keys().value(row) as usize;
        let vals = da.values().as_any().downcast_ref::<BinaryArray>()?;
        return vals.is_valid(k).then(|| vals.value(k));
    }
    None
}

/// Extracts a fixed-size byte slice at `row` from `FixedSizeBinaryArray` or `DictionaryArray`.
#[inline]
#[must_use]
pub fn extract_fixed_bytes_from_array(col: &ArrayRef, row: usize) -> Option<&[u8]> {
    if !col.is_valid(row) {
        return None;
    }
    if let Some(fsb) = col.as_any().downcast_ref::<FixedSizeBinaryArray>() {
        return Some(fsb.value(row));
    }
    if let Some(da) = col.as_any().downcast_ref::<DictionaryArray<UInt8Type>>() {
        let k = da.keys().value(row) as usize;
        let vals = da
            .values()
            .as_any()
            .downcast_ref::<FixedSizeBinaryArray>()?;
        return vals.is_valid(k).then(|| vals.value(k));
    }
    if let Some(da) = col.as_any().downcast_ref::<DictionaryArray<UInt16Type>>() {
        let k = da.keys().value(row) as usize;
        let vals = da
            .values()
            .as_any()
            .downcast_ref::<FixedSizeBinaryArray>()?;
        return vals.is_valid(k).then(|| vals.value(k));
    }
    None
}

/// Extracts an [`ExtractedValue`] at `row` from pre-resolved OTAP [`AnyValueArrays`].
///
/// Callers reading from a nullable [`StructArray`] (such as `logs.body`) must check
/// `struct_arr.is_valid(row)` before calling this helper.
#[inline]
#[must_use]
pub(crate) fn extract_anyval<'a>(
    arrays: &AnyValueArrays<'a>,
    row: usize,
) -> Option<ExtractedValue<'a>> {
    if !arrays.attr_type.is_valid(row) {
        return None;
    }
    let vtype = AttributeValueType::try_from(arrays.attr_type.value(row)).ok()?;
    match vtype {
        AttributeValueType::Empty => None,
        // Per pdata's `AnyValuesRecordsBuilder` (`default_values_optional: true`) and
        // `ResolvedAnyValue::resolve`, omitted or null str/int/double/bytes slots encode
        // their respective type defaults (`""`, `0`, `0.0`, `&[]`).
        AttributeValueType::Str => Some(ExtractedValue::Str(Cow::Borrowed(
            arrays
                .attr_str
                .as_ref()
                .and_then(|col| col.str_at(row))
                .unwrap_or(""),
        ))),
        AttributeValueType::Int => Some(ExtractedValue::Int(
            arrays
                .attr_int
                .as_ref()
                .and_then(|col| col.value_at(row))
                .unwrap_or(0),
        )),
        AttributeValueType::Double => Some(ExtractedValue::Double(
            arrays
                .attr_double
                .and_then(|da| da.is_valid(row).then(|| da.value(row)))
                .unwrap_or(0.0),
        )),
        AttributeValueType::Bool => {
            let ba = arrays.attr_bool?;
            ba.is_valid(row)
                .then(|| ExtractedValue::Bool(ba.value(row)))
        }
        AttributeValueType::Bytes => Some(ExtractedValue::Bytes(Cow::Borrowed(
            arrays
                .attr_bytes
                .as_ref()
                .and_then(|col| col.slice_at(row))
                .unwrap_or(&[]),
        ))),
        AttributeValueType::Map | AttributeValueType::Slice => {
            let bytes = arrays.attr_ser.as_ref()?.slice_at(row)?;
            decode_cbor_value(bytes)
        }
    }
}

/// Extracts an `ExtractedValue` at `row` from a `logs.body` `StructArray`.
#[cfg(test)]
#[must_use]
pub fn extract_anyval_from_struct(
    struct_arr: &StructArray,
    row: usize,
) -> Option<ExtractedValue<'_>> {
    if !struct_arr.is_valid(row) {
        return None;
    }
    let arrays = AnyValueArrays::try_from(struct_arr).ok()?;
    extract_anyval(&arrays, row)
}

/// Extracts an `ExtractedValue` at `row` from an attribute `RecordBatch` (`LogAttrs`, `ResourceAttrs`, `ScopeAttrs`).
#[cfg(test)]
#[must_use]
pub fn extract_anyval_from_attr_batch(rb: &RecordBatch, row: usize) -> Option<ExtractedValue<'_>> {
    let arrays = AnyValueArrays::try_from(rb).ok()?;
    extract_anyval(&arrays, row)
}

/// Pre-resolved `parent_id` column accessor supporting native and `UInt8`/`UInt16`
/// dictionary-encoded `UInt16` and `UInt32` arrays without per-row lookups or downcasts.
pub(crate) enum ParentIdColumn<'a> {
    UInt16(MaybeDictArrayAccessor<'a, UInt16Array>),
    UInt32(MaybeDictArrayAccessor<'a, UInt32Array>),
}

impl<'a> ParentIdColumn<'a> {
    /// Resolves the `parent_id` column once for an attribute `RecordBatch`.
    #[must_use]
    pub(crate) fn from_batch(rb: &'a RecordBatch) -> Option<Self> {
        let col = rb.column_by_name(consts::PARENT_ID)?;
        if let Ok(acc) = MaybeDictArrayAccessor::<UInt16Array>::try_new(col) {
            return Some(Self::UInt16(acc));
        }
        if let Ok(acc) = MaybeDictArrayAccessor::<UInt32Array>::try_new(col) {
            return Some(Self::UInt32(acc));
        }
        None
    }

    /// Reads the `parent_id` as `u16` at `row`, returning `None` if null or out of `u16` range.
    #[inline]
    #[must_use]
    pub(crate) fn value_at(&self, row: usize) -> Option<u16> {
        match self {
            Self::UInt16(acc) => acc.value_at(row),
            Self::UInt32(acc) => acc.value_at(row).and_then(|v| u16::try_from(v).ok()),
        }
    }
}

/// Extracts `parent_id` (`u16`) at `attr_row` from an attribute `RecordBatch`.
#[cfg(test)]
#[inline]
#[must_use]
pub fn extract_parent_id_u16(rb: &RecordBatch, attr_row: usize) -> Option<u16> {
    ParentIdColumn::from_batch(rb)?.value_at(attr_row)
}

#[cfg(test)]
fn extract_attr_from_batch<'a>(
    rb: &'a RecordBatch,
    target_parent_id: u16,
    path: &[String],
) -> Option<ExtractedValue<'a>> {
    let first_key = path.first()?;
    let key_col = rb.column_by_name(consts::ATTRIBUTE_KEY)?;
    let parent_ids = ParentIdColumn::from_batch(rb)?;
    let num_rows = rb.num_rows();
    for r in 0..num_rows {
        if parent_ids.value_at(r) == Some(target_parent_id)
            && extract_str_from_array(key_col, r) == Some(first_key.as_str())
        {
            let val = extract_anyval_from_attr_batch(rb, r)?;
            if path.len() == 1 {
                return Some(val);
            }
            return navigate_extracted_path(val, &path[1..]);
        }
    }
    None
}

/// Applies a boolean row-keep mask (`DenseBitset` over `0..num_rows`) to an `OtapArrowRecords` batch.
pub fn filter_otap_logs_by_bitmap(
    batch: &OtapArrowRecords,
    keep_rows: &DenseBitset,
    num_rows: usize,
    pool: &mut IdBitmapPool,
) -> Result<OtapArrowRecords, PolicyError> {
    if keep_rows.len() as usize == num_rows {
        return Ok(batch.clone());
    }
    let bool_buf = keep_rows.to_boolean_buffer(num_rows);
    let selection = BooleanArray::new(bool_buf, None);
    filter_otap_batch(&selection, batch, pool).map_err(|e| PolicyError::Evaluation(e.to_string()))
}

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

/// Executes all matching `CompiledTransformPolicy` stages in official `opentelemetry-proto#865` order
/// (`1. Rename -> 2. Add -> 3. Remove -> 4. Redact`) directly on `OtapArrowRecords` without
/// splitting and re-concatenating batches via `concatenate::concatenate`.
///
/// `policy_matches` must be aligned with `transforms` (which are pre-sorted by ascending `policy_id`).
// TODO: For production workloads with high transform counts (e.g., W4b at N=1,000), mutate flat
// Arrow builders directly instead of decoding and cloning touched attribute tables into nested
// `BTreeMap`s (`decode_attrs_table` / `encode_attrs_table`).
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

    // Extract mutable per-row state for top-level fields (`body`, `severity_text`, `severity_number`, `event_name`)
    // and per-parent `LogAttrs` map ONLY for columns/tables touched by transforms.
    let mut touches_root_body = false;
    let mut touches_sev_text = false;
    let mut touches_sev_num = false;
    let mut touches_event_name = false;
    let mut touches_log_attrs = false;

    for (tx, mask) in transforms.iter().zip(policy_matches.iter()) {
        if mask.is_empty() {
            continue;
        }
        for op in &tx.rename {
            if matches!(op.from, LogAttributeSelector::LogAttribute(_))
                || matches!(op.to, LogAttributeSelector::LogAttribute(_))
            {
                touches_log_attrs = true;
            }
        }
        let mut mark_target = |t: &LogTargetSelector| match t {
            LogTargetSelector::RecordField(LogRecordField::Body) => touches_root_body = true,
            LogTargetSelector::RecordField(LogRecordField::SeverityText) => touches_sev_text = true,
            LogTargetSelector::RecordField(LogRecordField::SeverityNumber) => {
                touches_sev_num = true
            }
            LogTargetSelector::RecordField(LogRecordField::EventName) => touches_event_name = true,
            LogTargetSelector::RecordField(_) => {}
            LogTargetSelector::LogAttribute(_) => touches_log_attrs = true,
            LogTargetSelector::BodyField(_) => {}
        };
        for op in &tx.add {
            mark_target(&op.target);
        }
        for op in &tx.remove {
            mark_target(&op.target);
        }
        for op in &tx.redact {
            mark_target(&op.target);
        }
    }

    let mut body_vals: Vec<Option<CellValue>> = if touches_root_body {
        extract_root_body_vec(&root, num_rows)
    } else {
        Vec::new()
    };
    let mut sev_text_vals: Vec<Option<String>> = if touches_sev_text {
        extract_root_str_vec(&root, consts::SEVERITY_TEXT, num_rows)
    } else {
        Vec::new()
    };
    let mut sev_num_vals: Vec<Option<i32>> = if touches_sev_num {
        extract_root_i32_vec(&root, consts::SEVERITY_NUMBER, num_rows)
    } else {
        Vec::new()
    };
    let mut event_name_vals: Vec<Option<String>> = if touches_event_name {
        extract_root_str_vec(&root, consts::EVENT_NAME, num_rows)
    } else {
        Vec::new()
    };

    // Read existing log ID column directly without allocating a Vec copy up front.
    let existing_log_ids: Option<&UInt16Array> = root
        .column_by_name(consts::ID)
        .and_then(|c| c.as_any().downcast_ref::<UInt16Array>());

    // Lazily materialized only when a transform adds a LogAttribute to a row that had no log.id.
    let mut mut_log_ids: Option<Vec<Option<u16>>> = None;
    let mut used_log_ids: Option<DenseBitset> = None;
    let mut next_log_id_candidate: u32 = 0;

    let mut log_attrs_map = if touches_log_attrs {
        decode_attrs_table(batch.get(ArrowPayloadType::LogAttrs))
    } else {
        BTreeMap::new()
    };

    let mut body_dirty = false;
    let mut sev_text_dirty = false;
    let mut sev_num_dirty = false;
    let mut event_name_dirty = false;
    let mut log_attrs_dirty = false;

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
                    for &pid in log_attrs_map.keys() {
                        bm.insert(u32::from(pid));
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

    // Context closure for reading/writing/removing a target at `row`.
    // TODO: Support mutating nested CBOR attribute paths (`path.len() > 1`, e.g., renaming or
    // redacting keys inside `AttributeValueType::Map` payloads) via `ciborium::Value` traversal
    // and re-serialization. Multi-segment transform targets are currently no-ops, while untouched
    // CBOR `ATTRIBUTE_SER` payloads are preserved byte-for-byte.
    macro_rules! get_target {
        ($row:expr, $target:expr) => {{
            let r: usize = $row;
            match $target {
                LogTargetSelector::RecordField(LogRecordField::Body) => body_vals[r].clone(),
                LogTargetSelector::RecordField(LogRecordField::SeverityText) => sev_text_vals[r]
                    .clone()
                    .map(|s| CellValue::Scalar(ScalarValue::String(s))),
                LogTargetSelector::RecordField(LogRecordField::SeverityNumber) => {
                    sev_num_vals[r].map(|v| CellValue::Scalar(ScalarValue::Int(i64::from(v))))
                }
                LogTargetSelector::RecordField(LogRecordField::EventName) => event_name_vals[r]
                    .clone()
                    .map(|s| CellValue::Scalar(ScalarValue::String(s))),
                LogTargetSelector::RecordField(_) => None,
                LogTargetSelector::LogAttribute(AttributePath { path }) => {
                    if path.len() == 1 {
                        get_log_id!(r).and_then(|pid| {
                            log_attrs_map
                                .get(&pid)
                                .and_then(|m| m.get(&path[0]).cloned())
                        })
                    } else {
                        None
                    }
                }
                LogTargetSelector::BodyField(_) => None,
            }
        }};
    }

    macro_rules! remove_target {
        ($row:expr, $target:expr) => {{
            let r: usize = $row;
            match $target {
                LogTargetSelector::RecordField(LogRecordField::Body) => {
                    let removed = body_vals[r].take();
                    if removed.is_some() {
                        body_dirty = true;
                    }
                    removed
                }
                LogTargetSelector::RecordField(LogRecordField::SeverityText) => {
                    let removed = sev_text_vals[r]
                        .take()
                        .map(|s| CellValue::Scalar(ScalarValue::String(s)));
                    if removed.is_some() {
                        sev_text_dirty = true;
                    }
                    removed
                }
                LogTargetSelector::RecordField(LogRecordField::SeverityNumber) => {
                    let removed = sev_num_vals[r]
                        .take()
                        .map(|v| CellValue::Scalar(ScalarValue::Int(i64::from(v))));
                    if removed.is_some() {
                        sev_num_dirty = true;
                    }
                    removed
                }
                LogTargetSelector::RecordField(LogRecordField::EventName) => {
                    let removed = event_name_vals[r]
                        .take()
                        .map(|s| CellValue::Scalar(ScalarValue::String(s)));
                    if removed.is_some() {
                        event_name_dirty = true;
                    }
                    removed
                }
                LogTargetSelector::RecordField(_) => None,
                LogTargetSelector::LogAttribute(AttributePath { path }) => {
                    if path.len() == 1 {
                        let removed = get_log_id!(r).and_then(|pid| {
                            log_attrs_map.get_mut(&pid).and_then(|m| m.remove(&path[0]))
                        });
                        if removed.is_some() {
                            log_attrs_dirty = true;
                        }
                        removed
                    } else {
                        None
                    }
                }
                LogTargetSelector::BodyField(_) => None,
            }
        }};
    }

    macro_rules! set_target {
        ($row:expr, $target:expr, $val:expr) => {{
            let r: usize = $row;
            let v: CellValue = $val;
            match $target {
                LogTargetSelector::RecordField(LogRecordField::Body) => {
                    body_vals[r] = Some(v);
                    body_dirty = true;
                    true
                }
                LogTargetSelector::RecordField(LogRecordField::SeverityText) => {
                    if let CellValue::Scalar(ScalarValue::String(s)) = v {
                        sev_text_vals[r] = (!s.is_empty()).then_some(s);
                        sev_text_dirty = true;
                        true
                    } else {
                        false
                    }
                }
                LogTargetSelector::RecordField(LogRecordField::SeverityNumber) => {
                    if let CellValue::Scalar(ScalarValue::Int(i)) = v
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
                    if let CellValue::Scalar(ScalarValue::String(s)) = v {
                        event_name_vals[r] = (!s.is_empty()).then_some(s);
                        event_name_dirty = true;
                        true
                    } else {
                        false
                    }
                }
                LogTargetSelector::RecordField(_) => false,
                LogTargetSelector::LogAttribute(AttributePath { path }) => {
                    if path.len() == 1 {
                        if let Some(pid) = get_or_alloc_log_id!(r) {
                            let _ = log_attrs_map
                                .entry(pid)
                                .or_default()
                                .insert(path[0].clone(), v);
                            log_attrs_dirty = true;
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
                LogTargetSelector::BodyField(_) => false,
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
            let (
                LogAttributeSelector::LogAttribute(AttributePath { path: from_path }),
                LogAttributeSelector::LogAttribute(AttributePath { path: to_path }),
            ) = (&op.from, &op.to)
            else {
                continue;
            };
            if from_path.len() != 1 || to_path.len() != 1 {
                continue;
            }
            let from_key = &from_path[0];
            let to_key = &to_path[0];
            for row_u32 in mask.iter() {
                let row = row_u32 as usize;
                let Some(pid) = get_log_id!(row) else {
                    continue;
                };
                let Some(attrs) = log_attrs_map.get_mut(&pid) else {
                    continue;
                };
                if !attrs.contains_key(from_key) {
                    continue;
                }
                if !op.upsert && attrs.contains_key(to_key) {
                    continue;
                }
                if let Some(src_val) = attrs.remove(from_key) {
                    let _ = attrs.insert(to_key.clone(), src_val);
                    log_attrs_dirty = true;
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
                if !op.upsert && get_target!(row, &op.target).is_some() {
                    continue;
                }
                let _ = set_target!(row, &op.target, CellValue::Scalar(op.value.clone()));
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
                            CellValue::Scalar(ScalarValue::String(op.replacement.clone()))
                        );
                    }
                    Some(re) => {
                        if let CellValue::Scalar(ScalarValue::String(s)) = cur_val
                            && let Some(redacted) = apply_regex_redaction(re, &s, &op.replacement)
                        {
                            let _ = set_target!(
                                row,
                                &op.target,
                                CellValue::Scalar(ScalarValue::String(redacted))
                            );
                        }
                    }
                }
            }
        }
    }

    // Rebuild modified root columns only when a root field or log.id actually changed
    if body_dirty || sev_text_dirty || sev_num_dirty || event_name_dirty || mut_log_ids.is_some() {
        let schema = root.schema();
        let mut new_fields: Vec<Arc<Field>> = Vec::with_capacity(schema.fields().len() + 5);
        let mut new_cols: Vec<ArrayRef> = Vec::with_capacity(schema.fields().len() + 5);
        let mut wrote_id = false;
        let mut wrote_body = false;
        let mut wrote_sev_text = false;
        let mut wrote_sev_num = false;
        let mut wrote_event_name = false;
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
                    let arr = build_body_struct_array(&body_vals);
                    new_fields.push(Arc::new(Field::new(
                        consts::BODY,
                        arr.data_type().clone(),
                        true,
                    )));
                    new_cols.push(arr);
                    wrote_body = true;
                }
                consts::SEVERITY_TEXT if sev_text_dirty => {
                    let arr: ArrayRef = Arc::new(StringArray::from(sev_text_vals.clone()));
                    new_fields.push(Arc::new(Field::new(
                        consts::SEVERITY_TEXT,
                        DataType::Utf8,
                        true,
                    )));
                    new_cols.push(arr);
                    wrote_sev_text = true;
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
                    let arr: ArrayRef = Arc::new(StringArray::from(event_name_vals.clone()));
                    new_fields.push(Arc::new(Field::new(
                        consts::EVENT_NAME,
                        DataType::Utf8,
                        true,
                    )));
                    new_cols.push(arr);
                    wrote_event_name = true;
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
        if body_dirty && !wrote_body && body_vals.iter().any(Option::is_some) {
            let arr = build_body_struct_array(&body_vals);
            new_fields.push(Arc::new(Field::new(
                consts::BODY,
                arr.data_type().clone(),
                true,
            )));
            new_cols.push(arr);
        }
        if sev_text_dirty && !wrote_sev_text && sev_text_vals.iter().any(Option::is_some) {
            let arr: ArrayRef = Arc::new(StringArray::from(sev_text_vals));
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
        if event_name_dirty && !wrote_event_name && event_name_vals.iter().any(Option::is_some) {
            let arr: ArrayRef = Arc::new(StringArray::from(event_name_vals));
            new_fields.push(Arc::new(Field::new(
                consts::EVENT_NAME,
                DataType::Utf8,
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

    if log_attrs_dirty {
        if log_attrs_map.values().all(BTreeMap::is_empty) {
            let _ = batch.remove(ArrowPayloadType::LogAttrs);
        } else {
            let rb = encode_attrs_table(&log_attrs_map)?;
            batch
                .set(ArrowPayloadType::LogAttrs, rb)
                .map_err(|e| PolicyError::Evaluation(e.to_string()))?;
        }
    }

    Ok(())
}

fn extract_root_body_vec(root: &RecordBatch, num_rows: usize) -> Vec<Option<CellValue>> {
    let mut out = vec![None; num_rows];
    let Some(col) = root.column_by_name(consts::BODY) else {
        return out;
    };
    if let Some(sa) = col.as_any().downcast_ref::<StructArray>() {
        let Ok(arrays) = AnyValueArrays::try_from(sa) else {
            return out;
        };
        for (r, slot) in out.iter_mut().enumerate().take(num_rows) {
            if !sa.is_valid(r) {
                continue;
            }
            if arrays.attr_type.is_valid(r)
                && let Ok(vtype @ (AttributeValueType::Map | AttributeValueType::Slice)) =
                    AttributeValueType::try_from(arrays.attr_type.value(r))
            {
                if let Some(raw) = arrays.attr_ser.as_ref().and_then(|sc| sc.slice_at(r)) {
                    *slot = Some(CellValue::RawCbor {
                        vtype: vtype as u8,
                        bytes: raw.to_vec(),
                    });
                }
                continue;
            }
            *slot = match extract_anyval(&arrays, r) {
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
            };
        }
    } else {
        for (r, slot) in out.iter_mut().enumerate().take(num_rows) {
            if let Some(s) = extract_str_from_array(col, r) {
                *slot = Some(CellValue::Scalar(ScalarValue::String(s.to_owned())));
            }
        }
    }
    out
}

fn extract_root_str_vec(
    root: &RecordBatch,
    col_name: &str,
    num_rows: usize,
) -> Vec<Option<String>> {
    let mut out = vec![None; num_rows];
    let Some(col) = root.column_by_name(col_name) else {
        return out;
    };
    for (r, slot) in out.iter_mut().enumerate().take(num_rows) {
        if let Some(s) = extract_str_from_array(col, r)
            && !s.is_empty()
        {
            *slot = Some(s.to_owned());
        }
    }
    out
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

fn build_body_struct_array(vals: &[Option<CellValue>]) -> ArrayRef {
    let len = vals.len();
    let mut types = Vec::with_capacity(len);
    let mut strs: Vec<Option<&str>> = Vec::with_capacity(len);
    let mut ints: Vec<Option<i64>> = Vec::with_capacity(len);
    let mut doubles: Vec<Option<f64>> = Vec::with_capacity(len);
    let mut bools: Vec<Option<bool>> = Vec::with_capacity(len);
    let mut bytes_vec: Vec<Option<&[u8]>> = Vec::with_capacity(len);
    let mut ser_vec: Vec<Option<&[u8]>> = Vec::with_capacity(len);
    let mut validity = Vec::with_capacity(len);

    for v in vals {
        match v {
            None => {
                validity.push(false);
                types.push(AttributeValueType::Empty as u8);
                strs.push(None);
                ints.push(None);
                doubles.push(None);
                bools.push(None);
                bytes_vec.push(None);
                ser_vec.push(None);
            }
            Some(CellValue::Empty) => {
                validity.push(true);
                types.push(AttributeValueType::Empty as u8);
                strs.push(None);
                ints.push(None);
                doubles.push(None);
                bools.push(None);
                bytes_vec.push(None);
                ser_vec.push(None);
            }
            Some(CellValue::Scalar(ScalarValue::String(s))) => {
                validity.push(true);
                types.push(AttributeValueType::Str as u8);
                strs.push(Some(s.as_str()));
                ints.push(None);
                doubles.push(None);
                bools.push(None);
                bytes_vec.push(None);
                ser_vec.push(None);
            }
            Some(CellValue::Scalar(ScalarValue::Int(i))) => {
                validity.push(true);
                types.push(AttributeValueType::Int as u8);
                strs.push(None);
                ints.push(Some(*i));
                doubles.push(None);
                bools.push(None);
                bytes_vec.push(None);
                ser_vec.push(None);
            }
            Some(CellValue::Scalar(ScalarValue::Double(d))) => {
                validity.push(true);
                types.push(AttributeValueType::Double as u8);
                strs.push(None);
                ints.push(None);
                doubles.push(Some(*d));
                bools.push(None);
                bytes_vec.push(None);
                ser_vec.push(None);
            }
            Some(CellValue::Scalar(ScalarValue::Bool(b))) => {
                validity.push(true);
                types.push(AttributeValueType::Bool as u8);
                strs.push(None);
                ints.push(None);
                doubles.push(None);
                bools.push(Some(*b));
                bytes_vec.push(None);
                ser_vec.push(None);
            }
            Some(CellValue::Scalar(ScalarValue::Bytes(b))) => {
                validity.push(true);
                types.push(AttributeValueType::Bytes as u8);
                strs.push(None);
                ints.push(None);
                doubles.push(None);
                bools.push(None);
                bytes_vec.push(Some(b.as_slice()));
                ser_vec.push(None);
            }
            Some(CellValue::RawCbor { vtype, bytes }) => {
                validity.push(true);
                types.push(*vtype);
                strs.push(None);
                ints.push(None);
                doubles.push(None);
                bools.push(None);
                bytes_vec.push(None);
                ser_vec.push(Some(bytes.as_slice()));
            }
        }
    }

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
        Arc::new(UInt8Array::from(types)),
        Arc::new(StringArray::from(strs)),
        Arc::new(Int64Array::from(ints)),
        Arc::new(Float64Array::from(doubles)),
        Arc::new(BooleanArray::from(bools)),
        Arc::new(BinaryArray::from_opt_vec(bytes_vec)),
        Arc::new(BinaryArray::from_opt_vec(ser_vec)),
    ];
    let nulls = NullBuffer::from(validity);
    Arc::new(StructArray::new(fields.into(), arrays, Some(nulls)))
}

// TODO: Replace `BTreeMap` decode/encode with direct flat Arrow builders in production to avoid
// per-attribute heap allocations and string cloning when transform policies touch large batches.
fn decode_attrs_table(rb_opt: Option<&RecordBatch>) -> BTreeMap<u16, BTreeMap<String, CellValue>> {
    let mut map: BTreeMap<u16, BTreeMap<String, CellValue>> = BTreeMap::new();
    let Some(rb) = rb_opt else {
        return map;
    };
    let Some(key_col) = rb.column_by_name(consts::ATTRIBUTE_KEY) else {
        return map;
    };
    let Some(parent_ids) = ParentIdColumn::from_batch(rb) else {
        return map;
    };
    let arrays = AnyValueArrays::try_from(rb).ok();
    for r in 0..rb.num_rows() {
        let Some(pid) = parent_ids.value_at(r) else {
            continue;
        };
        let Some(k) = extract_str_from_array(key_col, r) else {
            continue;
        };
        let cell = if let Some(arrays) = &arrays {
            if arrays.attr_type.is_valid(r) {
                match AttributeValueType::try_from(arrays.attr_type.value(r)) {
                    Ok(AttributeValueType::Empty) => CellValue::Empty,
                    Ok(vtype @ (AttributeValueType::Map | AttributeValueType::Slice)) => {
                        if let Some(raw) = arrays.attr_ser.as_ref().and_then(|sc| sc.slice_at(r)) {
                            CellValue::RawCbor {
                                vtype: vtype as u8,
                                bytes: raw.to_vec(),
                            }
                        } else {
                            CellValue::Empty
                        }
                    }
                    _ => match extract_anyval(arrays, r) {
                        Some(ExtractedValue::Str(s)) => {
                            CellValue::Scalar(ScalarValue::String(s.into_owned()))
                        }
                        Some(ExtractedValue::Int(i)) => CellValue::Scalar(ScalarValue::Int(i)),
                        Some(ExtractedValue::Double(d)) => {
                            CellValue::Scalar(ScalarValue::Double(d))
                        }
                        Some(ExtractedValue::Bool(b)) => CellValue::Scalar(ScalarValue::Bool(b)),
                        Some(ExtractedValue::Bytes(b)) => {
                            CellValue::Scalar(ScalarValue::Bytes(b.into_owned()))
                        }
                        _ => CellValue::Empty,
                    },
                }
            } else {
                CellValue::Empty
            }
        } else {
            CellValue::Empty
        };
        let _ = map.entry(pid).or_default().insert(k.to_owned(), cell);
    }
    map
}

fn encode_attrs_table(
    map: &BTreeMap<u16, BTreeMap<String, CellValue>>,
) -> Result<RecordBatch, PolicyError> {
    let total_rows: usize = map.values().map(BTreeMap::len).sum();
    let mut parent_ids: Vec<u16> = Vec::with_capacity(total_rows);
    let mut keys: Vec<&str> = Vec::with_capacity(total_rows);
    let mut types: Vec<u8> = Vec::with_capacity(total_rows);
    let mut strs: Vec<Option<&str>> = Vec::with_capacity(total_rows);
    let mut ints: Vec<Option<i64>> = Vec::with_capacity(total_rows);
    let mut doubles: Vec<Option<f64>> = Vec::with_capacity(total_rows);
    let mut bools: Vec<Option<bool>> = Vec::with_capacity(total_rows);
    let mut bytes_vec: Vec<Option<&[u8]>> = Vec::with_capacity(total_rows);
    let mut ser_vec: Vec<Option<&[u8]>> = Vec::with_capacity(total_rows);

    for (&pid, kvs) in map {
        for (k, v) in kvs {
            parent_ids.push(pid);
            keys.push(k.as_str());
            match v {
                CellValue::Empty => {
                    types.push(AttributeValueType::Empty as u8);
                    strs.push(None);
                    ints.push(None);
                    doubles.push(None);
                    bools.push(None);
                    bytes_vec.push(None);
                    ser_vec.push(None);
                }
                CellValue::Scalar(ScalarValue::String(s)) => {
                    types.push(AttributeValueType::Str as u8);
                    strs.push(Some(s.as_str()));
                    ints.push(None);
                    doubles.push(None);
                    bools.push(None);
                    bytes_vec.push(None);
                    ser_vec.push(None);
                }
                CellValue::Scalar(ScalarValue::Int(i)) => {
                    types.push(AttributeValueType::Int as u8);
                    strs.push(None);
                    ints.push(Some(*i));
                    doubles.push(None);
                    bools.push(None);
                    bytes_vec.push(None);
                    ser_vec.push(None);
                }
                CellValue::Scalar(ScalarValue::Double(d)) => {
                    types.push(AttributeValueType::Double as u8);
                    strs.push(None);
                    ints.push(None);
                    doubles.push(Some(*d));
                    bools.push(None);
                    bytes_vec.push(None);
                    ser_vec.push(None);
                }
                CellValue::Scalar(ScalarValue::Bool(b)) => {
                    types.push(AttributeValueType::Bool as u8);
                    strs.push(None);
                    ints.push(None);
                    doubles.push(None);
                    bools.push(Some(*b));
                    bytes_vec.push(None);
                    ser_vec.push(None);
                }
                CellValue::Scalar(ScalarValue::Bytes(b)) => {
                    types.push(AttributeValueType::Bytes as u8);
                    strs.push(None);
                    ints.push(None);
                    doubles.push(None);
                    bools.push(None);
                    bytes_vec.push(Some(b.as_slice()));
                    ser_vec.push(None);
                }
                CellValue::RawCbor { vtype, bytes } => {
                    types.push(*vtype);
                    strs.push(None);
                    ints.push(None);
                    doubles.push(None);
                    bools.push(None);
                    bytes_vec.push(None);
                    ser_vec.push(Some(bytes.as_slice()));
                }
            }
        }
    }

    let schema = Arc::new(Schema::new(vec![
        Field::new(consts::PARENT_ID, DataType::UInt16, false).with_plain_encoding(),
        Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
        Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
        Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
        Field::new(consts::ATTRIBUTE_INT, DataType::Int64, true),
        Field::new(consts::ATTRIBUTE_DOUBLE, DataType::Float64, true),
        Field::new(consts::ATTRIBUTE_BOOL, DataType::Boolean, true),
        Field::new(consts::ATTRIBUTE_BYTES, DataType::Binary, true),
        Field::new(consts::ATTRIBUTE_SER, DataType::Binary, true),
    ]));
    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(UInt16Array::from(parent_ids)),
            Arc::new(StringArray::from(keys)),
            Arc::new(UInt8Array::from(types)),
            Arc::new(StringArray::from(strs)),
            Arc::new(Int64Array::from(ints)),
            Arc::new(Float64Array::from(doubles)),
            Arc::new(BooleanArray::from(bools)),
            Arc::new(BinaryArray::from_opt_vec(bytes_vec)),
            Arc::new(BinaryArray::from_opt_vec(ser_vec)),
        ],
    )
    .map_err(|e| PolicyError::Evaluation(e.to_string()))
}
