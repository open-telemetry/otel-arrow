// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Columnar OTAP (`OtapArrowRecords`) inspection and zero-copy CBOR path navigation.

use std::borrow::Cow;
use std::cell::OnceCell;

#[cfg(test)]
use arrow::array::BinaryArray;
use arrow::array::{
    Array, ArrayRef, BooleanArray, DictionaryArray, FixedSizeBinaryArray, Int32Array, Int64Array,
    RecordBatch, StringArray, StructArray, UInt16Array, UInt32Array,
};
use arrow::datatypes::{UInt8Type, UInt16Type};
use otel_arrow_dfe_pdata::arrays::{MaybeDictArrayAccessor, NullableArrayAccessor};
use otel_arrow_dfe_pdata::otap::OtapArrowRecords;
use otel_arrow_dfe_pdata::otap::filter::{IdBitmapPool, filter_otap_batch};
use otel_arrow_dfe_pdata::otlp::attributes::AttributeValueType;
use otel_arrow_dfe_pdata::otlp::common::AnyValueArrays;
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_dfe_pdata::schema::consts;

use crate::bitset::DenseBitset;

#[cfg(test)]
use crate::model::{LogFieldSelector, LogRecordField, ScopeField};
use crate::model::{PolicyError, ScalarValue, eq_i64_f64};
pub use crate::transform::{
    CompiledLogRedact, CompiledTransformPolicy, apply_spec_transforms_masked,
};

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
pub(crate) const MAX_CBOR_DEPTH: usize = 64;

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

/// Returns `true` if `bytes` contains exactly one well-formed RFC 8949 CBOR item within
/// [`MAX_CBOR_DEPTH`] with no trailing bytes.
#[must_use]
pub(crate) fn is_valid_single_cbor_item(bytes: &[u8]) -> bool {
    let mut cursor = CborCursor::new(bytes);
    cursor.skip_value(0).is_ok() && cursor.pos == bytes.len()
}

/// Splits a top-level RFC 8949 CBOR map into `(optional_text_key, raw_key_slice, raw_val_slice)`
/// entries, validating both definite-length and indefinite-length maps and rejecting trailing bytes.
#[must_use]
pub(crate) fn parse_cbor_map_entries<'a>(
    bytes: &'a [u8],
) -> Option<Vec<(Option<Cow<'a, str>>, &'a [u8], &'a [u8])>> {
    let mut cursor = CborCursor::new(bytes);
    let ib = cursor.read_byte().ok()?;
    if (ib >> 5) != 5 {
        return None;
    }
    let entries = match cursor.read_arg(ib & 0x1f).ok()? {
        CborArg::Val(pairs) => {
            if pairs > (cursor.remaining_len() / 2) as u64 {
                return None;
            }
            let cap = usize::try_from(pairs).ok()?;
            let mut out = Vec::with_capacity(cap);
            for _ in 0..pairs {
                let k_start = cursor.pos;
                let kib = cursor.peek_byte()?;
                let key_text = if (kib >> 5) == 3 {
                    let _ = cursor.read_byte().ok()?;
                    Some(cursor.read_text_item(kib & 0x1f).ok()?)
                } else {
                    cursor.skip_value(1).ok()?;
                    None
                };
                let k_end = cursor.pos;
                let v_start = cursor.pos;
                cursor.skip_value(1).ok()?;
                let v_end = cursor.pos;
                out.push((key_text, &bytes[k_start..k_end], &bytes[v_start..v_end]));
            }
            out
        }
        CborArg::Indefinite => {
            let mut out = Vec::new();
            loop {
                match cursor.peek_byte()? {
                    0xff => {
                        cursor.pos += 1;
                        break;
                    }
                    kib => {
                        let k_start = cursor.pos;
                        let key_text = if (kib >> 5) == 3 {
                            let _ = cursor.read_byte().ok()?;
                            Some(cursor.read_text_item(kib & 0x1f).ok()?)
                        } else {
                            cursor.skip_value(1).ok()?;
                            None
                        };
                        let k_end = cursor.pos;
                        let v_start = cursor.pos;
                        cursor.skip_value(1).ok()?;
                        let v_end = cursor.pos;
                        out.push((key_text, &bytes[k_start..k_end], &bytes[v_start..v_end]));
                    }
                }
            }
            out
        }
    };
    if cursor.pos != bytes.len() {
        return None;
    }
    Some(entries)
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
