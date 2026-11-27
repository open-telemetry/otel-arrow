// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Per-series state cache and zero-allocation attribute multiset hashing for
//! the `metric_start_time` processor (`subtract_initial_point` strategy).

use std::time::{Duration, Instant};

use arrow::array::{
    Array, ArrayRef, ArrowPrimitiveType, BooleanArray, DictionaryArray, Float64Array, Int64Array,
    PrimitiveArray, RecordBatch, StringArray, UInt8Array,
};
use arrow::buffer::NullBuffer;
use arrow::datatypes::{DataType, UInt8Type, UInt16Type};
use hashbrown::HashMap;
use hashbrown::hash_map::EntryRef;
use otel_arrow_dfe_pdata::arrays::{
    ByteArrayAccessor, MaybeDictArrayAccessor, NullableArrayAccessor, get_required_array,
};
use otel_arrow_dfe_pdata::error::Error as PdataError;
use otel_arrow_dfe_pdata::otlp::attributes::AttributeValueType;
use otel_arrow_dfe_pdata::schema::consts;
use xxhash_rust::xxh3::{Xxh3, xxh3_128};

/// Upper bound for direct flat-vector indexing of `parent_id` values before falling
/// back to the sparse hash map.
const DENSE_ID_LIMIT: usize = 65_536;

const K0: u64 = 0x9E37_79B9_7F4A_7C15;
const K1: u64 = 0xBF58_476D_1CE4_E5B9;
const K2: u64 = 0x94D0_49BB_1331_11EB;
const K3: u64 = 0xD6E8_FEB8_6659_FD93;

/// 128-bit multiset attribute hash stored as `[u64; 2]` (8-byte alignment) rather than `u128`
/// (16-byte alignment) so `hashbrown::HashMap` bucket tuples `(AttrHash, SeriesEntry<S>)` do not
/// waste 8 bytes of tail padding when `size_of::<SeriesEntry<S>>()` is an odd multiple of 8.
pub(super) type AttrHash = [u64; 2];

const MARK_BIT: u64 = 1 << 63;

#[derive(Debug, Default)]
struct IdHashAccum {
    dense: Vec<AttrHash>,
    max_dense_id: Option<usize>,
    sparse: HashMap<u32, AttrHash>,
}

impl IdHashAccum {
    fn clear(&mut self) {
        if let Some(max_id) = self.max_dense_id.take() {
            let end = (max_id + 1).min(self.dense.len());
            self.dense[..end].fill([0, 0]);
        }
        self.sparse.clear();
    }

    #[inline]
    fn add(&mut self, parent_id: u32, lo: u64, hi: u64) {
        let idx = parent_id as usize;
        let slot = if idx < DENSE_ID_LIMIT {
            if idx >= self.dense.len() {
                let next_len = (idx + 1).next_power_of_two().min(DENSE_ID_LIMIT);
                self.dense.resize(next_len, [0, 0]);
            }
            self.max_dense_id = Some(self.max_dense_id.map_or(idx, |cur| cur.max(idx)));
            &mut self.dense[idx]
        } else {
            self.sparse.entry(parent_id).or_insert([0, 0])
        };
        slot[0] = slot[0].wrapping_add(lo);
        slot[1] = slot[1].wrapping_add(hi);
    }

    #[inline]
    fn get(&self, parent_id: u32) -> AttrHash {
        let idx = parent_id as usize;
        if idx < DENSE_ID_LIMIT {
            self.dense.get(idx).copied().unwrap_or([0, 0])
        } else {
            self.sparse.get(&parent_id).copied().unwrap_or([0, 0])
        }
    }
}

/// Order-independent 128-bit multiset attribute hasher for OTAP `Attributes` record batches.
///
/// Pre-hashes dictionary key/value arrays once per batch so per-row hashing reduces to array
/// lookups and 128-bit integer mixing, while producing identical hashes regardless of whether
/// columns are dictionary-encoded, plain, or omitted due to default values.
#[derive(Debug, Default)]
pub(super) struct AttrHasher {
    accum: IdHashAccum,
    key_dict_hashes: Vec<u128>,
    str_dict_hashes: Vec<u128>,
}

impl AttrHasher {
    #[must_use]
    pub(super) fn new() -> Self {
        Self::default()
    }

    #[inline]
    #[must_use]
    pub(super) fn get(&self, parent_id: u32) -> AttrHash {
        self.accum.get(parent_id)
    }

    pub(super) fn populate<P>(&mut self, attrs_rb: Option<&RecordBatch>) -> Result<(), PdataError>
    where
        P: ArrowPrimitiveType,
        P::Native: Into<u32>,
    {
        let Self {
            accum,
            key_dict_hashes,
            str_dict_hashes,
        } = self;
        accum.clear();
        let Some(rb) = attrs_rb else {
            return Ok(());
        };
        let num_rows = rb.num_rows();
        if num_rows == 0 {
            return Ok(());
        }

        let parent_id_arr = get_required_array(rb, consts::PARENT_ID)?;
        let keys = StrColHash::try_new(
            get_required_array(rb, consts::ATTRIBUTE_KEY)?,
            key_dict_hashes,
        )?;
        let type_arr = get_required_array(rb, consts::ATTRIBUTE_TYPE)?;
        let types = type_arr
            .as_any()
            .downcast_ref::<UInt8Array>()
            .ok_or_else(|| PdataError::ColumnDataTypeMismatch {
                name: consts::ATTRIBUTE_TYPE.into(),
                actual: type_arr.data_type().clone(),
                expect: DataType::UInt8,
            })?;
        let hasher = RowHasher {
            type_vals: types.values().as_ref(),
            type_nulls: types.nulls(),
            keys,
            strs: rb
                .column_by_name(consts::ATTRIBUTE_STR)
                .map(|c| StrColHash::try_new(c, str_dict_hashes))
                .transpose()?,
            ints: rb
                .column_by_name(consts::ATTRIBUTE_INT)
                .map(IntCol::try_new)
                .transpose()?,
            doubles: rb
                .column_by_name(consts::ATTRIBUTE_DOUBLE)
                .and_then(|c| c.as_any().downcast_ref::<Float64Array>()),
            bools: rb
                .column_by_name(consts::ATTRIBUTE_BOOL)
                .and_then(|c| c.as_any().downcast_ref::<BooleanArray>()),
            bytes: rb
                .column_by_name(consts::ATTRIBUTE_BYTES)
                .map(ByteArrayAccessor::try_new)
                .transpose()?,
            sers: rb
                .column_by_name(consts::ATTRIBUTE_SER)
                .map(ByteArrayAccessor::try_new)
                .transpose()?,
            empty_str_hash: xxh3_128(b""),
        };

        if let Some(plain) = parent_id_arr.as_any().downcast_ref::<PrimitiveArray<P>>()
            && plain.nulls().is_none()
        {
            for (row, &pid) in plain.values().as_ref().iter().enumerate() {
                let (lo, hi) = hasher.hash_row(row);
                accum.add(pid.into(), lo, hi);
            }
            return Ok(());
        }

        let parent_ids = MaybeDictArrayAccessor::<PrimitiveArray<P>>::try_new(parent_id_arr)?;
        for row in 0..num_rows {
            if let Some(pid) = parent_ids.value_at(row) {
                let (lo, hi) = hasher.hash_row(row);
                accum.add(pid.into(), lo, hi);
            }
        }
        Ok(())
    }
}

struct RowHasher<'a> {
    type_vals: &'a [u8],
    type_nulls: Option<&'a NullBuffer>,
    keys: StrColHash<'a>,
    strs: Option<StrColHash<'a>>,
    ints: Option<IntCol<'a>>,
    doubles: Option<&'a Float64Array>,
    bools: Option<&'a BooleanArray>,
    bytes: Option<ByteArrayAccessor<'a>>,
    sers: Option<ByteArrayAccessor<'a>>,
    empty_str_hash: u128,
}

impl RowHasher<'_> {
    #[inline(always)]
    fn hash_row(&self, row: usize) -> (u64, u64) {
        const STR_TY: u8 = AttributeValueType::Str as u8;
        const INT_TY: u8 = AttributeValueType::Int as u8;
        const DBL_TY: u8 = AttributeValueType::Double as u8;
        const BOOL_TY: u8 = AttributeValueType::Bool as u8;
        const BYTES_TY: u8 = AttributeValueType::Bytes as u8;
        const MAP_TY: u8 = AttributeValueType::Map as u8;
        const SLICE_TY: u8 = AttributeValueType::Slice as u8;

        let val_type = if self.type_nulls.is_none_or(|n| n.is_valid(row)) {
            self.type_vals[row]
        } else {
            0
        };
        let kh = self.keys.hash_at(row, self.empty_str_hash);
        let vh: u128 = match val_type {
            STR_TY => self
                .strs
                .as_ref()
                .map_or(self.empty_str_hash, |c| c.hash_at(row, self.empty_str_hash)),
            INT_TY => mix_scalar(self.ints.as_ref().map_or(0, |c| c.val_at(row)) as u64),
            DBL_TY => mix_scalar(
                self.doubles
                    .and_then(|c| c.is_valid(row).then(|| c.value(row)))
                    .unwrap_or(0.0)
                    .to_bits(),
            ),
            BOOL_TY => mix_scalar(u64::from(
                self.bools
                    .and_then(|c| c.is_valid(row).then(|| c.value(row)))
                    .unwrap_or(false),
            )),
            BYTES_TY => xxh3_128(
                self.bytes
                    .as_ref()
                    .and_then(|c| c.slice_at(row))
                    .unwrap_or(&[]),
            ),
            MAP_TY | SLICE_TY => hash_cbor_bytes(
                self.sers
                    .as_ref()
                    .and_then(|c| c.slice_at(row))
                    .unwrap_or(&[]),
            ),
            _ => 0,
        };
        combine_kv(val_type, kh, vh)
    }
}

const MAX_CBOR_DEPTH: usize = 64;

const CBOR_TAG_UINT: u8 = 1;
const CBOR_TAG_NINT: u8 = 2;
const CBOR_TAG_BYTES: u8 = 3;
const CBOR_TAG_TEXT: u8 = 4;
const CBOR_TAG_ARRAY: u8 = 5;
const CBOR_TAG_MAP: u8 = 6;
const CBOR_TAG_SEMANTIC: u8 = 7;
const CBOR_TAG_SIMPLE: u8 = 8;
const CBOR_TAG_FLOAT: u8 = 9;
const CBOR_TAG_MAP_ENTRY: u8 = 10;
const CBOR_TAG_SEQ_ELEM: u8 = 11;

/// Hashes serialized CBOR attribute values (`Map` / `Slice`) without heap allocations,
/// treating maps recursively as unordered key-value multisets (including maps inside arrays)
/// while preserving array element order. Falls back to `xxh3_128(raw)` for empty, malformed,
/// or excessively nested inputs.
fn hash_cbor_bytes(raw: &[u8]) -> u128 {
    if raw.is_empty() {
        return xxh3_128(b"");
    }
    let mut cur = raw;
    match hash_cbor_item(&mut cur, 0) {
        Some(h) if cur.is_empty() => h,
        _ => xxh3_128(raw),
    }
}

#[inline]
fn pair_to_u128(lo: u64, hi: u64) -> u128 {
    u128::from(lo) | (u128::from(hi) << 64)
}

#[inline]
fn mix_cbor_val(tag: u8, v: u128) -> u128 {
    let (lo, hi) = combine_kv(tag, mix_scalar(u64::from(tag)), v);
    pair_to_u128(lo, hi)
}

#[inline]
fn cbor_read_u8(cur: &mut &[u8]) -> Option<u8> {
    let (&first, rest) = cur.split_first()?;
    *cur = rest;
    Some(first)
}

#[inline]
fn cbor_read_exact<'a>(cur: &mut &'a [u8], n: usize) -> Option<&'a [u8]> {
    if cur.len() < n {
        return None;
    }
    let (head, rest) = cur.split_at(n);
    *cur = rest;
    Some(head)
}

#[inline]
fn cbor_read_arg(cur: &mut &[u8], ai: u8) -> Option<u64> {
    match ai {
        0..=23 => Some(u64::from(ai)),
        24 => Some(u64::from(cbor_read_u8(cur)?)),
        25 => Some(u64::from(u16::from_be_bytes(
            cbor_read_exact(cur, 2)?.try_into().ok()?,
        ))),
        26 => Some(u64::from(u32::from_be_bytes(
            cbor_read_exact(cur, 4)?.try_into().ok()?,
        ))),
        27 => Some(u64::from_be_bytes(
            cbor_read_exact(cur, 8)?.try_into().ok()?,
        )),
        _ => None,
    }
}

fn cbor_hash_bytes_or_text(cur: &mut &[u8], major: u8, ai: u8) -> Option<u128> {
    if ai == 31 {
        let mut hasher = Xxh3::new();
        loop {
            let b = cbor_read_u8(cur)?;
            if b == 0xff {
                return Some(hasher.digest128());
            }
            if (b >> 5) != major || (b & 0x1f) == 31 {
                return None;
            }
            let len = usize::try_from(cbor_read_arg(cur, b & 0x1f)?).ok()?;
            hasher.update(cbor_read_exact(cur, len)?);
        }
    }
    let len = usize::try_from(cbor_read_arg(cur, ai)?).ok()?;
    Some(xxh3_128(cbor_read_exact(cur, len)?))
}

fn cbor_f16_to_f64(bits: u16) -> f64 {
    let sign = u64::from(bits >> 15) << 63;
    let exp = (bits >> 10) & 0x1f;
    let mant = u64::from(bits & 0x03ff);
    let f64_bits = match exp {
        0 if mant == 0 => sign,
        0 => {
            // Subnormal binary16: (-1)^s * mant * 2^-24.
            let val = (mant as f64) * f64::from_bits(0x3e70_0000_0000_0000);
            return if sign != 0 { -val } else { val };
        }
        0x1f => sign | (0x7ffu64 << 52) | (mant << 42),
        _ => sign | ((u64::from(exp) + (1023 - 15)) << 52) | (mant << 42),
    };
    f64::from_bits(f64_bits)
}

fn hash_cbor_item(cur: &mut &[u8], depth: usize) -> Option<u128> {
    if depth >= MAX_CBOR_DEPTH {
        return None;
    }
    let ib = cbor_read_u8(cur)?;
    let major = ib >> 5;
    let ai = ib & 0x1f;
    match major {
        0 => Some(mix_cbor_val(
            CBOR_TAG_UINT,
            mix_scalar(cbor_read_arg(cur, ai)?),
        )),
        1 => Some(mix_cbor_val(
            CBOR_TAG_NINT,
            mix_scalar(cbor_read_arg(cur, ai)?),
        )),
        2 => Some(mix_cbor_val(
            CBOR_TAG_BYTES,
            cbor_hash_bytes_or_text(cur, 2, ai)?,
        )),
        3 => Some(mix_cbor_val(
            CBOR_TAG_TEXT,
            cbor_hash_bytes_or_text(cur, 3, ai)?,
        )),
        4 => {
            let mut acc = mix_cbor_val(CBOR_TAG_ARRAY, 0);
            let mut count = 0u64;
            if ai == 31 {
                loop {
                    if cur.first() == Some(&0xff) {
                        *cur = &cur[1..];
                        break;
                    }
                    let elem = hash_cbor_item(cur, depth + 1)?;
                    let (lo, hi) = combine_kv(CBOR_TAG_SEQ_ELEM, acc ^ mix_scalar(count), elem);
                    acc = pair_to_u128(lo, hi);
                    count = count.checked_add(1)?;
                }
            } else {
                let len = cbor_read_arg(cur, ai)?;
                for i in 0..len {
                    let elem = hash_cbor_item(cur, depth + 1)?;
                    let (lo, hi) = combine_kv(CBOR_TAG_SEQ_ELEM, acc ^ mix_scalar(i), elem);
                    acc = pair_to_u128(lo, hi);
                }
                count = len;
            }
            Some(mix_cbor_val(CBOR_TAG_ARRAY, acc ^ mix_scalar(count)))
        }
        5 => {
            let (mut sum_lo, mut sum_hi) = (0u64, 0u64);
            let mut count = 0u64;
            if ai == 31 {
                loop {
                    if cur.first() == Some(&0xff) {
                        *cur = &cur[1..];
                        break;
                    }
                    let kh = hash_cbor_item(cur, depth + 1)?;
                    let vh = hash_cbor_item(cur, depth + 1)?;
                    let (lo, hi) = combine_kv(CBOR_TAG_MAP_ENTRY, kh, vh);
                    sum_lo = sum_lo.wrapping_add(lo);
                    sum_hi = sum_hi.wrapping_add(hi);
                    count = count.checked_add(1)?;
                }
            } else {
                let len = cbor_read_arg(cur, ai)?;
                for _ in 0..len {
                    let kh = hash_cbor_item(cur, depth + 1)?;
                    let vh = hash_cbor_item(cur, depth + 1)?;
                    let (lo, hi) = combine_kv(CBOR_TAG_MAP_ENTRY, kh, vh);
                    sum_lo = sum_lo.wrapping_add(lo);
                    sum_hi = sum_hi.wrapping_add(hi);
                }
                count = len;
            }
            let (lo, hi) = combine_kv(
                CBOR_TAG_MAP,
                mix_scalar(count),
                pair_to_u128(sum_lo, sum_hi),
            );
            Some(pair_to_u128(lo, hi))
        }
        6 => {
            let tag = cbor_read_arg(cur, ai)?;
            let inner = hash_cbor_item(cur, depth + 1)?;
            let (lo, hi) = combine_kv(CBOR_TAG_SEMANTIC, mix_scalar(tag), inner);
            Some(pair_to_u128(lo, hi))
        }
        7 => match ai {
            0..=23 => Some(mix_cbor_val(CBOR_TAG_SIMPLE, mix_scalar(u64::from(ai)))),
            24 => Some(mix_cbor_val(
                CBOR_TAG_SIMPLE,
                mix_scalar(u64::from(cbor_read_u8(cur)?)),
            )),
            25 => {
                let bits = u16::from_be_bytes(cbor_read_exact(cur, 2)?.try_into().ok()?);
                Some(mix_cbor_val(
                    CBOR_TAG_FLOAT,
                    mix_scalar(cbor_f16_to_f64(bits).to_bits()),
                ))
            }
            26 => {
                let bits = u32::from_be_bytes(cbor_read_exact(cur, 4)?.try_into().ok()?);
                let val = f64::from(f32::from_bits(bits));
                Some(mix_cbor_val(CBOR_TAG_FLOAT, mix_scalar(val.to_bits())))
            }
            27 => {
                let bits = u64::from_be_bytes(cbor_read_exact(cur, 8)?.try_into().ok()?);
                Some(mix_cbor_val(CBOR_TAG_FLOAT, mix_scalar(bits)))
            }
            _ => None,
        },
        _ => None,
    }
}

#[inline(always)]
fn widen_mul(a: u64, b: u64) -> u128 {
    (a as u128).wrapping_mul(b as u128)
}

#[inline(always)]
fn mix_scalar(v: u64) -> u128 {
    let x = (v ^ K0).wrapping_mul(K1);
    widen_mul(x ^ (x >> 32), K2)
}

#[inline(always)]
fn combine_kv(val_type: u8, kh: u128, vh: u128) -> (u64, u64) {
    let type_mix = (val_type as u64).wrapping_add(1).wrapping_mul(K3);
    let a = (kh as u64) ^ ((vh >> 64) as u64).wrapping_add(type_mix);
    let b = ((kh >> 64) as u64) ^ (vh as u64).wrapping_add(type_mix ^ K0);
    let p1 = widen_mul(a ^ K1, b ^ K2);
    let p2 = widen_mul(a.wrapping_add(K3), b.wrapping_add(K1));
    let mut lo = (p1 as u64) ^ ((p2 >> 64) as u64);
    let hi = ((p1 >> 64) as u64) ^ (p2 as u64);
    if lo == 0 && hi == 0 {
        lo = 1;
    }
    (lo, hi)
}

enum DictKeys<'a> {
    U8(&'a [u8], Option<&'a NullBuffer>),
    U16(&'a [u16], Option<&'a NullBuffer>),
}

impl DictKeys<'_> {
    #[inline(always)]
    fn index_at(&self, row: usize) -> Option<usize> {
        match *self {
            Self::U8(keys, nulls) => nulls
                .is_none_or(|n| n.is_valid(row))
                .then(|| keys[row] as usize),
            Self::U16(keys, nulls) => nulls
                .is_none_or(|n| n.is_valid(row))
                .then(|| keys[row] as usize),
        }
    }
}

fn downcast_dict<'a, V: 'static>(col: &'a ArrayRef) -> Option<(DictKeys<'a>, &'a V)> {
    if let Some(d) = col.as_any().downcast_ref::<DictionaryArray<UInt8Type>>()
        && let Some(v) = d.values().as_any().downcast_ref::<V>()
    {
        return Some((
            DictKeys::U8(d.keys().values().as_ref(), d.keys().nulls()),
            v,
        ));
    }
    if let Some(d) = col.as_any().downcast_ref::<DictionaryArray<UInt16Type>>()
        && let Some(v) = d.values().as_any().downcast_ref::<V>()
    {
        return Some((
            DictKeys::U16(d.keys().values().as_ref(), d.keys().nulls()),
            v,
        ));
    }
    None
}

enum IntCol<'a> {
    Plain {
        vals: &'a [i64],
        nulls: Option<&'a NullBuffer>,
    },
    Dict {
        keys: DictKeys<'a>,
        vals: &'a [i64],
        val_nulls: Option<&'a NullBuffer>,
    },
    Fallback(MaybeDictArrayAccessor<'a, Int64Array>),
}

impl<'a> IntCol<'a> {
    fn try_new(col: &'a ArrayRef) -> Result<Self, PdataError> {
        if let Some(arr) = col.as_any().downcast_ref::<Int64Array>() {
            return Ok(Self::Plain {
                vals: arr.values().as_ref(),
                nulls: arr.nulls(),
            });
        }
        if let Some((keys, v)) = downcast_dict::<Int64Array>(col) {
            return Ok(Self::Dict {
                keys,
                vals: v.values().as_ref(),
                val_nulls: v.nulls(),
            });
        }
        Ok(Self::Fallback(
            MaybeDictArrayAccessor::<Int64Array>::try_new(col)?,
        ))
    }

    #[inline(always)]
    fn val_at(&self, row: usize) -> i64 {
        match self {
            Self::Plain { vals, nulls } => {
                if nulls.is_none_or(|n| n.is_valid(row)) {
                    vals[row]
                } else {
                    0
                }
            }
            Self::Dict {
                keys,
                vals,
                val_nulls,
            } => keys
                .index_at(row)
                .filter(|&idx| val_nulls.is_none_or(|n| n.is_valid(idx)))
                .map_or(0, |idx| vals[idx]),
            Self::Fallback(acc) => acc.value_at(row).unwrap_or(0),
        }
    }
}

enum StrColHash<'a> {
    Dict(DictKeys<'a>, &'a [u128]),
    Plain(MaybeDictArrayAccessor<'a, StringArray>),
}

impl<'a> StrColHash<'a> {
    fn try_new(col: &'a ArrayRef, buf: &'a mut Vec<u128>) -> Result<Self, PdataError> {
        if let Some((keys, vals)) = downcast_dict::<StringArray>(col) {
            fill_str_hashes(vals, buf);
            return Ok(Self::Dict(keys, buf.as_slice()));
        }
        Ok(Self::Plain(MaybeDictArrayAccessor::<StringArray>::try_new(
            col,
        )?))
    }

    #[inline(always)]
    fn hash_at(&self, row: usize, default_hash: u128) -> u128 {
        match self {
            Self::Dict(keys, hashes) => keys.index_at(row).map_or(default_hash, |i| hashes[i]),
            Self::Plain(acc) => acc
                .str_at(row)
                .map_or(default_hash, |s| xxh3_128(s.as_bytes())),
        }
    }
}

fn fill_str_hashes(vals: &StringArray, out: &mut Vec<u128>) {
    out.clear();
    out.reserve(vals.len());
    let empty_hash = xxh3_128(b"");
    let nulls = vals.nulls();
    for i in 0..vals.len() {
        let h = if nulls.is_none_or(|n| n.is_valid(i)) {
            xxh3_128(vals.value(i).as_bytes())
        } else {
            empty_hash
        };
        out.push(h);
    }
}

/// Wrapper storing mark-and-sweep liveness packed into bit 63 of `start_time_ns` alongside
/// per-type series state, avoiding 8 bytes of `bool` alignment padding per series.
///
/// NOTE: OTLP timestamps represent non-negative nanoseconds since the Unix epoch (`0..=i64::MAX`,
/// covering 1970-2262), so negative inputs are clamped to `0` before packing `MARK_BIT`.
#[derive(Debug, Clone)]
pub(super) struct SeriesEntry<S> {
    marked_start_ns: u64,
    pub(super) data: S,
}

impl<S> SeriesEntry<S> {
    #[inline]
    #[must_use]
    pub(super) fn new(start_time_ns: i64, data: S) -> Self {
        Self {
            marked_start_ns: (start_time_ns.max(0) as u64) | MARK_BIT,
            data,
        }
    }

    #[inline]
    pub(super) fn mark(&mut self) {
        self.marked_start_ns |= MARK_BIT;
    }

    #[inline]
    fn take_mark(&mut self) -> bool {
        let marked = (self.marked_start_ns & MARK_BIT) != 0;
        self.marked_start_ns &= !MARK_BIT;
        marked
    }

    #[inline]
    #[must_use]
    pub(super) fn start_time_ns(&self) -> i64 {
        (self.marked_start_ns & !MARK_BIT) as i64
    }

    #[inline]
    pub(super) fn set_start_time_ns(&mut self, start_time_ns: i64) {
        self.marked_start_ns = (start_time_ns.max(0) as u64) | MARK_BIT;
    }
}

/// Type-tagged 24-byte state for `Sum` (`NumberDataPoint`), keeping `i64` and `f64` series
/// distinct so a value-type switch triggers a clean reset instead of bit reinterpretation.
#[derive(Debug, Clone, Copy)]
pub(super) enum NumberState {
    Int { ref_val: i64, prev_val: i64 },
    Double { ref_val: f64, prev_val: f64 },
}

/// Shared `count` + `sum` baseline and previous-point tracker for `Summary`, `Histogram`, and
/// `ExponentialHistogram`.
#[derive(Debug, Clone, Copy)]
pub(super) struct CountSumState {
    pub(super) ref_count: u64,
    pub(super) ref_sum: f64,
    prev_count: u64,
    prev_sum: f64,
}

impl CountSumState {
    #[inline]
    #[must_use]
    pub(super) fn new(count: u64, sum: Option<f64>) -> Self {
        // NOTE: `f64::NAN` marks an unknown baseline `ref_sum` (when the initial point
        // omitted `sum` with non-zero observations or reported `NaN`) so subsequent points
        // omit optional `sum` until a reset establishes a zero baseline. `f64::NEG_INFINITY`
        // marks an unset `prev_sum` so a later finite sum never triggers a false `< prev_sum`
        // reset.
        let (ref_sum, prev_sum) = match sum {
            Some(v) if !v.is_nan() => (v, v),
            None if count == 0 => (0.0, f64::NEG_INFINITY),
            _ => (f64::NAN, f64::NEG_INFINITY),
        };
        Self {
            ref_count: count,
            ref_sum,
            prev_count: count,
            prev_sum,
        }
    }

    #[inline]
    #[must_use]
    pub(super) fn has_known_ref_sum(&self) -> bool {
        !self.ref_sum.is_nan()
    }

    #[inline]
    #[must_use]
    pub(super) fn decreased(&self, count: u64, sum: Option<f64>) -> bool {
        count < self.prev_count || sum.is_some_and(|v| !v.is_nan() && v < self.prev_sum)
    }

    #[inline]
    pub(super) fn reset(&mut self, count: u64, sum: Option<f64>) {
        self.ref_count = 0;
        self.ref_sum = 0.0;
        self.prev_count = count;
        self.prev_sum = match sum {
            Some(v) if !v.is_nan() => v,
            _ => f64::NEG_INFINITY,
        };
    }

    #[inline]
    pub(super) fn advance(&mut self, count: u64, sum: Option<f64>) {
        self.prev_count = count;
        if let Some(v) = sum
            && !v.is_nan()
        {
            self.prev_sum = v;
        }
    }
}

/// Compact 56-byte state for `HistogramDataPoint`, storing a 64-bit hash of `explicit_bounds`
/// and packing `[ref_buckets | prev_buckets]` into a single exact-sized `Box<[u64]>`.
#[derive(Debug, Clone)]
pub(super) struct HistogramState {
    pub(super) cs: CountSumState,
    pub(super) bounds_hash: u64,
    pub(super) buckets: Box<[u64]>,
}

fn pack_u64_slices(slices: &[&[u64]]) -> Box<[u64]> {
    let total: usize = slices.iter().map(|s| s.len()).sum();
    let mut buf = Vec::with_capacity(total);
    for s in slices {
        buf.extend_from_slice(s);
    }
    buf.into_boxed_slice()
}

impl HistogramState {
    #[inline]
    #[must_use]
    pub(super) fn new(count: u64, sum: Option<f64>, bounds_hash: u64, cur_buckets: &[u64]) -> Self {
        Self {
            cs: CountSumState::new(count, sum),
            bounds_hash,
            buckets: pack_u64_slices(&[cur_buckets, cur_buckets]),
        }
    }
}

/// Compact 96-byte state for `ExponentialHistogramDataPoint`, packing all four bucket slices
/// (`[ref_pos | prev_pos | ref_neg | prev_neg]`) into a single exact-sized `Box<[u64]>`.
#[derive(Debug, Clone)]
pub(super) struct ExpHistogramState {
    pub(super) cs: CountSumState,
    pub(super) ref_zero_count: u64,
    pub(super) prev_zero_count: u64,
    buckets: Box<[u64]>,
    ref_pos_offset: i32,
    prev_pos_offset: i32,
    ref_neg_offset: i32,
    prev_neg_offset: i32,
    pub(super) scale: i32,
    ref_pos_len: u32,
    prev_pos_len: u32,
    ref_neg_len: u32,
}

impl ExpHistogramState {
    #[inline]
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        scale: i32,
        count: u64,
        sum: Option<f64>,
        zero_count: u64,
        pos_offset: i32,
        pos_counts: &[u64],
        neg_offset: i32,
        neg_counts: &[u64],
    ) -> Self {
        Self {
            cs: CountSumState::new(count, sum),
            ref_zero_count: zero_count,
            prev_zero_count: zero_count,
            buckets: pack_u64_slices(&[pos_counts, pos_counts, neg_counts, neg_counts]),
            ref_pos_offset: pos_offset,
            prev_pos_offset: pos_offset,
            ref_neg_offset: neg_offset,
            prev_neg_offset: neg_offset,
            scale,
            ref_pos_len: pos_counts.len() as u32,
            prev_pos_len: pos_counts.len() as u32,
            ref_neg_len: neg_counts.len() as u32,
        }
    }

    #[inline]
    #[must_use]
    pub(super) fn ref_pos(&self) -> (i32, &[u64]) {
        (
            self.ref_pos_offset,
            &self.buckets[..self.ref_pos_len as usize],
        )
    }

    #[inline]
    #[must_use]
    pub(super) fn prev_pos(&self) -> (i32, &[u64]) {
        let start = self.ref_pos_len as usize;
        (
            self.prev_pos_offset,
            &self.buckets[start..start + self.prev_pos_len as usize],
        )
    }

    #[inline]
    #[must_use]
    pub(super) fn ref_neg(&self) -> (i32, &[u64]) {
        let start = self.ref_pos_len as usize + self.prev_pos_len as usize;
        (
            self.ref_neg_offset,
            &self.buckets[start..start + self.ref_neg_len as usize],
        )
    }

    #[inline]
    #[must_use]
    pub(super) fn prev_neg(&self) -> (i32, &[u64]) {
        let start =
            self.ref_pos_len as usize + self.prev_pos_len as usize + self.ref_neg_len as usize;
        (self.prev_neg_offset, &self.buckets[start..])
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub(super) fn reset_and_update_prev(
        &mut self,
        scale: i32,
        count: u64,
        sum: Option<f64>,
        zero_count: u64,
        pos_offset: i32,
        pos_counts: &[u64],
        neg_offset: i32,
        neg_counts: &[u64],
    ) {
        self.scale = scale;
        self.cs.reset(count, sum);
        self.ref_zero_count = 0;
        self.prev_zero_count = zero_count;
        self.ref_pos_offset = pos_offset;
        self.prev_pos_offset = pos_offset;
        self.ref_neg_offset = neg_offset;
        self.prev_neg_offset = neg_offset;
        self.ref_pos_len = 0;
        self.prev_pos_len = pos_counts.len() as u32;
        self.ref_neg_len = 0;
        if self.buckets.len() == pos_counts.len() + neg_counts.len() {
            self.buckets[..pos_counts.len()].copy_from_slice(pos_counts);
            self.buckets[pos_counts.len()..].copy_from_slice(neg_counts);
        } else {
            self.buckets = pack_u64_slices(&[pos_counts, neg_counts]);
        }
    }

    #[inline]
    pub(super) fn update_prev(
        &mut self,
        zero_count: u64,
        pos_offset: i32,
        pos_counts: &[u64],
        neg_offset: i32,
        neg_counts: &[u64],
    ) {
        self.prev_zero_count = zero_count;
        self.prev_pos_offset = pos_offset;
        self.prev_neg_offset = neg_offset;
        let (rp, pp, rn) = (
            self.ref_pos_len as usize,
            self.prev_pos_len as usize,
            self.ref_neg_len as usize,
        );
        let pn = self.buckets.len() - (rp + pp + rn);
        if pos_counts.len() == pp && neg_counts.len() == pn {
            self.buckets[rp..rp + pp].copy_from_slice(pos_counts);
            self.buckets[rp + pp + rn..].copy_from_slice(neg_counts);
        } else {
            self.prev_pos_len = pos_counts.len() as u32;
            self.buckets = pack_u64_slices(&[
                &self.buckets[..rp],
                pos_counts,
                &self.buckets[rp + pp..rp + pp + rn],
                neg_counts,
            ]);
        }
    }
}

/// Reclaims excess hash-table capacity after mark-and-sweep eviction when a map
/// drops well below its peak allocation.
fn shrink_if_sparse<K: Eq + std::hash::Hash, V>(map: &mut HashMap<K, V>) {
    if map.capacity() > 64 && map.len().saturating_mul(4) < map.capacity() {
        map.shrink_to(map.len().saturating_mul(2));
    }
}

/// Per-metric series map (`metric_name -> dp_attr_hash -> SeriesEntry<S>`).
#[derive(Debug)]
pub(super) struct SeriesTable<S>(HashMap<String, HashMap<AttrHash, SeriesEntry<S>>>);

impl<S> Default for SeriesTable<S> {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

impl<S> SeriesTable<S> {
    #[inline]
    pub(super) fn series_mut(
        &mut self,
        metric_name: &str,
    ) -> &mut HashMap<AttrHash, SeriesEntry<S>> {
        match self.0.entry_ref(metric_name) {
            EntryRef::Occupied(entry) => entry.into_mut(),
            EntryRef::Vacant(entry) => entry.insert(HashMap::new()),
        }
    }

    fn sweep(&mut self) -> bool {
        self.0.retain(|_, series_map| {
            series_map.retain(|_, entry| entry.take_mark());
            shrink_if_sparse(series_map);
            !series_map.is_empty()
        });
        shrink_if_sparse(&mut self.0);
        !self.0.is_empty()
    }
}

#[derive(Debug, Default)]
pub(super) struct ResourceSeriesMap {
    mark: bool,
    pub(super) numbers: SeriesTable<NumberState>,
    pub(super) summaries: SeriesTable<CountSumState>,
    pub(super) histograms: SeriesTable<HistogramState>,
    pub(super) exp_histograms: SeriesTable<ExpHistogramState>,
}

impl ResourceSeriesMap {
    fn sweep(&mut self) -> bool {
        let a = self.numbers.sweep();
        let b = self.summaries.sweep();
        let c = self.histograms.sweep();
        let d = self.exp_histograms.sweep();
        a || b || c || d
    }
}

/// Two-level (`resource_hash -> metric_name -> dp_attr_hash`) state cache with mark-and-sweep GC.
#[derive(Debug)]
pub(super) struct SubtractInitialCache {
    gc_interval: Duration,
    last_gc: Instant,
    resources: HashMap<AttrHash, ResourceSeriesMap>,
}

impl SubtractInitialCache {
    #[must_use]
    pub(super) fn new(gc_interval: Duration, now: Instant) -> Self {
        Self {
            gc_interval,
            last_gc: now,
            resources: HashMap::new(),
        }
    }

    #[inline]
    pub(super) fn resource_mut(&mut self, resource_hash: AttrHash) -> &mut ResourceSeriesMap {
        let res = self.resources.entry(resource_hash).or_default();
        res.mark = true;
        res
    }

    /// Clears cached state before processing if more than two full GC intervals
    /// elapsed without traffic so stale entries are not resurrected.
    pub(super) fn maybe_clear_stale(&mut self, now: Instant) {
        if now.saturating_duration_since(self.last_gc) > self.gc_interval.saturating_mul(2) {
            self.resources.clear();
            shrink_if_sparse(&mut self.resources);
            self.last_gc = now;
        }
    }

    pub(super) fn maybe_gc(&mut self, now: Instant) {
        if now.saturating_duration_since(self.last_gc) <= self.gc_interval {
            return;
        }
        self.resources.retain(|_, res_map| {
            if !res_map.mark {
                return false;
            }
            res_map.mark = false;
            res_map.sweep()
        });
        shrink_if_sparse(&mut self.resources);
        self.last_gc = now;
    }

    #[cfg(test)]
    pub(super) fn resource_count(&self) -> usize {
        self.resources.len()
    }
}

const _: () = {
    assert!(size_of::<(AttrHash, SeriesEntry<NumberState>)>() == 48);
    assert!(size_of::<(AttrHash, SeriesEntry<CountSumState>)>() == 56);
    assert!(size_of::<(AttrHash, SeriesEntry<HistogramState>)>() == 80);
    assert!(size_of::<(AttrHash, SeriesEntry<ExpHistogramState>)>() == 120);
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use arrow::array::{BinaryArray, UInt16Array, UInt32Array};
    use arrow::datatypes::{Field, Schema, UInt32Type};

    fn attr_rb(
        pids: ArrayRef,
        keys: ArrayRef,
        types: ArrayRef,
        extra: Vec<(&str, ArrayRef)>,
    ) -> RecordBatch {
        let mut fields = vec![
            Field::new(consts::PARENT_ID, pids.data_type().clone(), true),
            Field::new(consts::ATTRIBUTE_KEY, keys.data_type().clone(), false),
            Field::new(consts::ATTRIBUTE_TYPE, types.data_type().clone(), true),
        ];
        let mut cols = vec![pids, keys, types];
        for (name, arr) in extra {
            fields.push(Field::new(name, arr.data_type().clone(), true));
            cols.push(arr);
        }
        RecordBatch::try_new(Arc::new(Schema::new(fields)), cols).unwrap()
    }

    /// Scenario: Populating AttrHasher with both dense and sparse (>= 65_536) parent IDs and then clearing on the next populate.
    /// Guarantees: Sparse parent IDs hash identically to dense IDs for the same attributes and clear resets both dense and sparse storage.
    #[test]
    fn test_attr_hasher_sparse_parent_ids_and_clear() {
        let mut hasher = AttrHasher::new();
        let (dense_id, sparse_id) = (7u32, (DENSE_ID_LIMIT as u32) + 42);
        let rb = attr_rb(
            Arc::new(UInt32Array::from(vec![dense_id, sparse_id])),
            Arc::new(StringArray::from(vec!["env", "env"])),
            Arc::new(UInt8Array::from(vec![AttributeValueType::Str as u8; 2])),
            vec![(
                consts::ATTRIBUTE_STR,
                Arc::new(StringArray::from(vec![Some("prod"), Some("prod")])),
            )],
        );

        hasher.populate::<UInt32Type>(Some(&rb)).unwrap();
        assert_ne!(hasher.get(dense_id), [0, 0]);
        assert_eq!(hasher.get(dense_id), hasher.get(sparse_id));
        assert_eq!(hasher.get(sparse_id + 1), [0, 0]);

        hasher.populate::<UInt32Type>(None).unwrap();
        assert_eq!(
            (hasher.get(dense_id), hasher.get(sparse_id)),
            ([0, 0], [0, 0])
        );
    }

    fn dict_attr_batch<K: arrow::datatypes::ArrowDictionaryKeyType>(
        pids: ArrayRef,
        types: ArrayRef,
        k0: K::Native,
        k1: K::Native,
    ) -> RecordBatch {
        let dict = |keys: PrimitiveArray<K>, vals: ArrayRef| -> ArrayRef {
            Arc::new(DictionaryArray::<K>::try_new(keys, vals).unwrap())
        };
        attr_rb(
            pids,
            dict(
                PrimitiveArray::<K>::from_iter_values([k0, k1]),
                Arc::new(StringArray::from(vec!["k_s", "k_i"])),
            ),
            types,
            vec![
                (
                    consts::ATTRIBUTE_STR,
                    dict(
                        PrimitiveArray::<K>::from_iter([Some(k0), None]),
                        Arc::new(StringArray::from(vec![Some("v1")])),
                    ),
                ),
                (
                    consts::ATTRIBUTE_INT,
                    dict(
                        PrimitiveArray::<K>::from_iter([None, Some(k0)]),
                        Arc::new(Int64Array::from(vec![Some(99)])),
                    ),
                ),
            ],
        )
    }

    /// Scenario: Hashing identical attributes encoded as Plain, Dict8, and Dict16 across string keys, string values, and integer values, plus nullable parent_id/type/dict entries and omitted default columns.
    /// Guarantees: AttrHasher produces identical hashes across Plain/Dict8/Dict16 encodings and matches omitted optional columns against explicit canonical default values.
    #[test]
    fn test_attr_hasher_encoding_equivalence_nulls_and_omitted_defaults() {
        let hash_one = |rb: RecordBatch, id: u32| {
            let mut h = AttrHasher::new();
            h.populate::<UInt16Type>(Some(&rb)).unwrap();
            h.get(id)
        };
        let ty2: ArrayRef = Arc::new(UInt8Array::from(vec![
            AttributeValueType::Str as u8,
            AttributeValueType::Int as u8,
        ]));
        let pids2: ArrayRef = Arc::new(UInt16Array::from(vec![1u16, 1]));

        let plain = hash_one(
            attr_rb(
                Arc::new(UInt16Array::from(vec![Some(1), Some(1), None])),
                Arc::new(StringArray::from(vec!["k_s", "k_i", "ignored"])),
                Arc::new(UInt8Array::from(vec![
                    AttributeValueType::Str as u8,
                    AttributeValueType::Int as u8,
                    AttributeValueType::Str as u8,
                ])),
                vec![
                    (
                        consts::ATTRIBUTE_STR,
                        Arc::new(StringArray::from(vec![Some("v1"), None, Some("x")])),
                    ),
                    (
                        consts::ATTRIBUTE_INT,
                        Arc::new(Int64Array::from(vec![None, Some(99), None])),
                    ),
                ],
            ),
            1,
        );
        assert_eq!(
            plain,
            hash_one(
                dict_attr_batch::<UInt8Type>(Arc::clone(&pids2), Arc::clone(&ty2), 0, 1),
                1
            )
        );
        assert_eq!(
            plain,
            hash_one(dict_attr_batch::<UInt16Type>(pids2, ty2, 0, 1), 1)
        );

        let pids5: ArrayRef = Arc::new(UInt16Array::from(vec![3u16; 5]));
        let keys5: ArrayRef = Arc::new(StringArray::from(vec!["s", "b", "by", "m", "e"]));
        let types5: ArrayRef = Arc::new(UInt8Array::from(vec![
            Some(AttributeValueType::Str as u8),
            Some(AttributeValueType::Bool as u8),
            Some(AttributeValueType::Bytes as u8),
            Some(AttributeValueType::Map as u8),
            None,
        ]));
        let omitted = hash_one(
            attr_rb(
                Arc::clone(&pids5),
                Arc::clone(&keys5),
                Arc::clone(&types5),
                vec![],
            ),
            3,
        );
        let explicit = hash_one(
            attr_rb(
                pids5,
                keys5,
                types5,
                vec![
                    (
                        consts::ATTRIBUTE_STR,
                        Arc::new(StringArray::from(vec![Some(""), None, None, None, None])),
                    ),
                    (
                        consts::ATTRIBUTE_BOOL,
                        Arc::new(BooleanArray::from(vec![
                            None,
                            Some(false),
                            None,
                            None,
                            None,
                        ])),
                    ),
                    (
                        consts::ATTRIBUTE_BYTES,
                        Arc::new(BinaryArray::from_opt_vec(vec![
                            None,
                            None,
                            Some(b"".as_slice()),
                            None,
                            None,
                        ])),
                    ),
                    (
                        consts::ATTRIBUTE_SER,
                        Arc::new(BinaryArray::from_opt_vec(vec![
                            None,
                            None,
                            None,
                            Some(b"".as_slice()),
                            None,
                        ])),
                    ),
                ],
            ),
            3,
        );
        assert_eq!(omitted, explicit);
    }

    /// Scenario: Constructing and querying ExpHistogramState with 32_768 positive buckets and a scale > i16::MAX, and shrinking SubtractInitialCache after a cardinality spike.
    /// Guarantees: Bucket indexing never overflows u16, scale is preserved without truncation, and GC shrinks map capacity after sweeping evicted resources.
    #[test]
    fn test_exp_histogram_large_bucket_count_and_gc_shrink() {
        let large_pos = vec![1u64; 32_768];
        let mut st =
            ExpHistogramState::new(70_000, 32_784, Some(100.0), 0, 5, &large_pos, -3, &[7, 9]);
        assert_eq!(st.scale, 70_000);
        assert_eq!(
            (st.ref_pos().1.len(), st.prev_pos().1.len()),
            (32_768, 32_768)
        );
        assert_eq!(st.ref_neg(), (-3, &[7u64, 9][..]));
        assert_eq!(st.prev_neg(), (-3, &[7u64, 9][..]));

        st.update_prev(1, 5, &large_pos, -3, &[8, 10]);
        assert_eq!(st.prev_neg(), (-3, &[8u64, 10][..]));

        let t0 = Instant::now();
        let mut cache = SubtractInitialCache::new(Duration::from_secs(10), t0);
        for i in 0u64..200 {
            let _ = cache.resource_mut([i, 1]).numbers.series_mut("m").insert(
                [i, 2],
                SeriesEntry::new(
                    10,
                    NumberState::Int {
                        ref_val: 1,
                        prev_val: 1,
                    },
                ),
            );
        }
        let peak_cap = cache.resources.capacity();
        assert!(peak_cap >= 200);
        cache.maybe_gc(t0 + Duration::from_secs(11));
        cache.maybe_gc(t0 + Duration::from_secs(22));
        assert_eq!(cache.resource_count(), 0);
        assert!(cache.resources.capacity() < peak_cap);
    }

    /// Scenario: Hashing CBOR maps and arrays across reordered keys, nested maps inside arrays, definite vs indefinite length encodings, float widths, and malformed/deep inputs.
    /// Guarantees: Equivalent CBOR maps hash identically regardless of key order or container length encoding, array element order remains distinct, and malformed or over-nested payloads fall back without panicking.
    #[test]
    fn test_hash_cbor_order_independent_maps_ordered_arrays_and_fallback() {
        // Definite map {"a": 1, "b": 2} vs indefinite map {_ "b": 2, "a": 1}.
        let def_map_ab = [0xa2, 0x61, b'a', 0x01, 0x61, b'b', 0x02];
        let def_map_ba = [0xa2, 0x61, b'b', 0x02, 0x61, b'a', 0x01];
        let indef_map_ba = [0xbf, 0x61, b'b', 0x02, 0x61, b'a', 0x01, 0xff];
        assert_eq!(hash_cbor_bytes(&def_map_ab), hash_cbor_bytes(&def_map_ba));
        assert_eq!(hash_cbor_bytes(&def_map_ab), hash_cbor_bytes(&indef_map_ba));

        // Array containing maps: map key order inside elements is ignored, array order is preserved.
        let arr_m1_m2 = [
            0x9f, 0xbf, 0x61, b'x', 0x01, 0x61, b'y', 0x02, 0xff, 0xbf, 0x61, b'z', 0x03, 0xff,
            0xff,
        ];
        let arr_m1rev_m2 = [
            0x82, 0xa2, 0x61, b'y', 0x02, 0x61, b'x', 0x01, 0xa1, 0x61, b'z', 0x03,
        ];
        let arr_m2_m1 = [
            0x82, 0xa1, 0x61, b'z', 0x03, 0xa2, 0x61, b'x', 0x01, 0x61, b'y', 0x02,
        ];
        assert_eq!(hash_cbor_bytes(&arr_m1_m2), hash_cbor_bytes(&arr_m1rev_m2));
        assert_ne!(hash_cbor_bytes(&arr_m1_m2), hash_cbor_bytes(&arr_m2_m1));

        // Empty map != empty array; float16(1.5) == float32(1.5) == float64(1.5).
        assert_ne!(hash_cbor_bytes(&[0xa0]), hash_cbor_bytes(&[0x80]));
        let f16_1_5 = [0xf9, 0x3e, 0x00];
        let f32_1_5 = [0xfa, 0x3f, 0xc0, 0x00, 0x00];
        let f64_1_5 = [0xfb, 0x3f, 0xf8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert_eq!(hash_cbor_bytes(&f16_1_5), hash_cbor_bytes(&f32_1_5));
        assert_eq!(hash_cbor_bytes(&f16_1_5), hash_cbor_bytes(&f64_1_5));

        // Indefinite chunked text string "ab" matches definite "ab".
        let def_str = [0x62, b'a', b'b'];
        let indef_str = [0x7f, 0x61, b'a', 0x61, b'b', 0xff];
        assert_eq!(hash_cbor_bytes(&def_str), hash_cbor_bytes(&indef_str));

        // Truncated and over-nested payloads fall back to raw xxh3_128 without panicking.
        let truncated = [0xa2, 0x61, b'a'];
        assert_eq!(hash_cbor_bytes(&truncated), xxh3_128(&truncated));
        let mut deep = vec![0x81u8; MAX_CBOR_DEPTH + 4];
        deep.push(0x00);
        assert_eq!(hash_cbor_bytes(&deep), xxh3_128(&deep));
    }
}
