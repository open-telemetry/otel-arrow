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
use xxhash_rust::xxh3::xxh3_128;

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
        if idx < DENSE_ID_LIMIT {
            if idx >= self.dense.len() {
                let next_len = (idx + 1).next_power_of_two().min(DENSE_ID_LIMIT);
                self.dense.resize(next_len, [0, 0]);
            }
            self.max_dense_id = Some(self.max_dense_id.map_or(idx, |cur| cur.max(idx)));
            let slot = &mut self.dense[idx];
            slot[0] = slot[0].wrapping_add(lo);
            slot[1] = slot[1].wrapping_add(hi);
        } else {
            let slot = self.sparse.entry(parent_id).or_insert([0, 0]);
            slot[0] = slot[0].wrapping_add(lo);
            slot[1] = slot[1].wrapping_add(hi);
        }
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
            MAP_TY | SLICE_TY => xxh3_128(
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

enum IntCol<'a> {
    Plain {
        vals: &'a [i64],
        nulls: Option<&'a NullBuffer>,
    },
    Dict8 {
        keys: &'a [u8],
        key_nulls: Option<&'a NullBuffer>,
        vals: &'a [i64],
        val_nulls: Option<&'a NullBuffer>,
    },
    Dict16 {
        keys: &'a [u16],
        key_nulls: Option<&'a NullBuffer>,
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
        if let Some(d) = col.as_any().downcast_ref::<DictionaryArray<UInt8Type>>()
            && let Some(v) = d.values().as_any().downcast_ref::<Int64Array>()
        {
            return Ok(Self::Dict8 {
                keys: d.keys().values().as_ref(),
                key_nulls: d.keys().nulls(),
                vals: v.values().as_ref(),
                val_nulls: v.nulls(),
            });
        }
        if let Some(d) = col.as_any().downcast_ref::<DictionaryArray<UInt16Type>>()
            && let Some(v) = d.values().as_any().downcast_ref::<Int64Array>()
        {
            return Ok(Self::Dict16 {
                keys: d.keys().values().as_ref(),
                key_nulls: d.keys().nulls(),
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
            Self::Dict8 {
                keys,
                key_nulls,
                vals,
                val_nulls,
            } => {
                if key_nulls.is_none_or(|n| n.is_valid(row)) {
                    let idx = keys[row] as usize;
                    if val_nulls.is_none_or(|n| n.is_valid(idx)) {
                        vals[idx]
                    } else {
                        0
                    }
                } else {
                    0
                }
            }
            Self::Dict16 {
                keys,
                key_nulls,
                vals,
                val_nulls,
            } => {
                if key_nulls.is_none_or(|n| n.is_valid(row)) {
                    let idx = keys[row] as usize;
                    if val_nulls.is_none_or(|n| n.is_valid(idx)) {
                        vals[idx]
                    } else {
                        0
                    }
                } else {
                    0
                }
            }
            Self::Fallback(acc) => acc.value_at(row).unwrap_or(0),
        }
    }
}

enum StrColHash<'a> {
    Dict8 {
        keys: &'a [u8],
        nulls: Option<&'a NullBuffer>,
        hashes: &'a [u128],
    },
    Dict16 {
        keys: &'a [u16],
        nulls: Option<&'a NullBuffer>,
        hashes: &'a [u128],
    },
    Plain(MaybeDictArrayAccessor<'a, StringArray>),
}

impl<'a> StrColHash<'a> {
    fn try_new(col: &'a ArrayRef, buf: &'a mut Vec<u128>) -> Result<Self, PdataError> {
        if let Some(d) = col.as_any().downcast_ref::<DictionaryArray<UInt8Type>>()
            && let Some(vals) = d.values().as_any().downcast_ref::<StringArray>()
        {
            fill_str_hashes(vals, buf);
            return Ok(Self::Dict8 {
                keys: d.keys().values().as_ref(),
                nulls: d.keys().nulls(),
                hashes: buf.as_slice(),
            });
        }
        if let Some(d) = col.as_any().downcast_ref::<DictionaryArray<UInt16Type>>()
            && let Some(vals) = d.values().as_any().downcast_ref::<StringArray>()
        {
            fill_str_hashes(vals, buf);
            return Ok(Self::Dict16 {
                keys: d.keys().values().as_ref(),
                nulls: d.keys().nulls(),
                hashes: buf.as_slice(),
            });
        }
        Ok(Self::Plain(MaybeDictArrayAccessor::<StringArray>::try_new(
            col,
        )?))
    }

    #[inline(always)]
    fn hash_at(&self, row: usize, default_hash: u128) -> u128 {
        match self {
            Self::Dict8 {
                keys,
                nulls,
                hashes,
            } => {
                if nulls.is_none_or(|n| n.is_valid(row)) {
                    hashes[keys[row] as usize]
                } else {
                    default_hash
                }
            }
            Self::Dict16 {
                keys,
                nulls,
                hashes,
            } => {
                if nulls.is_none_or(|n| n.is_valid(row)) {
                    hashes[keys[row] as usize]
                } else {
                    default_hash
                }
            }
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
        // NOTE: `f64::NEG_INFINITY` marks an unset `prev_sum` (for null or `NaN` sums) so a later
        // finite sum never triggers a false `< prev_sum` reset or poisons subtraction.
        let (ref_sum, prev_sum) = match sum {
            Some(v) if !v.is_nan() => (v, v),
            _ => (0.0, f64::NEG_INFINITY),
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

impl HistogramState {
    #[inline]
    #[must_use]
    pub(super) fn new(count: u64, sum: Option<f64>, bounds_hash: u64, cur_buckets: &[u64]) -> Self {
        let b_len = cur_buckets.len();
        let mut buf = Vec::with_capacity(b_len * 2);
        buf.extend_from_slice(cur_buckets);
        buf.extend_from_slice(cur_buckets);
        Self {
            cs: CountSumState::new(count, sum),
            bounds_hash,
            buckets: buf.into_boxed_slice(),
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
        let p_len = pos_counts.len();
        let n_len = neg_counts.len();
        let mut buf = Vec::with_capacity((p_len + n_len) * 2);
        buf.extend_from_slice(pos_counts);
        buf.extend_from_slice(pos_counts);
        buf.extend_from_slice(neg_counts);
        buf.extend_from_slice(neg_counts);
        Self {
            cs: CountSumState::new(count, sum),
            ref_zero_count: zero_count,
            prev_zero_count: zero_count,
            buckets: buf.into_boxed_slice(),
            ref_pos_offset: pos_offset,
            prev_pos_offset: pos_offset,
            ref_neg_offset: neg_offset,
            prev_neg_offset: neg_offset,
            scale,
            ref_pos_len: p_len as u32,
            prev_pos_len: p_len as u32,
            ref_neg_len: n_len as u32,
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
        let end = start + self.prev_pos_len as usize;
        (self.prev_pos_offset, &self.buckets[start..end])
    }

    #[inline]
    #[must_use]
    pub(super) fn ref_neg(&self) -> (i32, &[u64]) {
        let start = self.ref_pos_len as usize + self.prev_pos_len as usize;
        let end = start + self.ref_neg_len as usize;
        (self.ref_neg_offset, &self.buckets[start..end])
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
        let total = pos_counts.len() + neg_counts.len();
        if self.buckets.len() == total {
            self.buckets[..pos_counts.len()].copy_from_slice(pos_counts);
            self.buckets[pos_counts.len()..].copy_from_slice(neg_counts);
        } else {
            let mut buf = Vec::with_capacity(total);
            buf.extend_from_slice(pos_counts);
            buf.extend_from_slice(neg_counts);
            self.buckets = buf.into_boxed_slice();
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
        let rp = self.ref_pos_len as usize;
        let pp = self.prev_pos_len as usize;
        let rn = self.ref_neg_len as usize;
        let pn = self.buckets.len() - (rp + pp + rn);
        if pos_counts.len() == pp && neg_counts.len() == pn {
            self.buckets[rp..rp + pp].copy_from_slice(pos_counts);
            self.buckets[rp + pp + rn..].copy_from_slice(neg_counts);
        } else {
            let mut buf = Vec::with_capacity(rp + pos_counts.len() + rn + neg_counts.len());
            buf.extend_from_slice(&self.buckets[..rp]);
            buf.extend_from_slice(pos_counts);
            buf.extend_from_slice(&self.buckets[rp + pp..rp + pp + rn]);
            buf.extend_from_slice(neg_counts);
            self.prev_pos_len = pos_counts.len() as u32;
            self.buckets = buf.into_boxed_slice();
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

    /// Scenario: Populating AttrHasher with both dense and sparse (>= 65_536) parent IDs and then clearing on the next populate.
    /// Guarantees: Sparse parent IDs hash identically to dense IDs for the same attributes and clear resets both dense and sparse storage.
    #[test]
    fn test_attr_hasher_sparse_parent_ids_and_clear() {
        let mut hasher = AttrHasher::new();
        let dense_id = 7u32;
        let sparse_id = (DENSE_ID_LIMIT as u32) + 42;
        let schema = Arc::new(Schema::new(vec![
            Field::new(consts::PARENT_ID, DataType::UInt32, false),
            Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
            Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
            Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
        ]));
        let rb = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(UInt32Array::from(vec![dense_id, sparse_id])),
                Arc::new(StringArray::from(vec!["env", "env"])),
                Arc::new(UInt8Array::from(vec![
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                ])),
                Arc::new(StringArray::from(vec![Some("prod"), Some("prod")])),
            ],
        )
        .unwrap();

        hasher.populate::<UInt32Type>(Some(&rb)).unwrap();
        let dense_hash = hasher.get(dense_id);
        let sparse_hash = hasher.get(sparse_id);
        assert_ne!(dense_hash, [0, 0]);
        assert_eq!(dense_hash, sparse_hash);
        assert_eq!(hasher.get(sparse_id + 1), [0, 0]);

        hasher.populate::<UInt32Type>(None).unwrap();
        assert_eq!(hasher.get(dense_id), [0, 0]);
        assert_eq!(hasher.get(sparse_id), [0, 0]);
    }

    /// Scenario: Hashing identical attributes encoded as Plain, Dict8, and Dict16 across string keys, string values, and integer values, plus nullable parent_id/type/dict entries and omitted default columns.
    /// Guarantees: AttrHasher produces identical hashes across Plain/Dict8/Dict16 encodings and matches omitted optional columns against explicit canonical default values.
    #[test]
    fn test_attr_hasher_encoding_equivalence_nulls_and_omitted_defaults() {
        let mut h_plain = AttrHasher::new();
        let mut h_dict8 = AttrHasher::new();
        let mut h_dict16 = AttrHasher::new();

        // Plain batch with 1 string attr and 1 int attr for parent_id = 1.
        let plain_rb = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, true),
                Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
                Field::new(consts::ATTRIBUTE_INT, DataType::Int64, true),
            ])),
            vec![
                Arc::new(UInt16Array::from(vec![Some(1), Some(1), None])),
                Arc::new(StringArray::from(vec!["k_s", "k_i", "ignored"])),
                Arc::new(UInt8Array::from(vec![
                    AttributeValueType::Str as u8,
                    AttributeValueType::Int as u8,
                    AttributeValueType::Str as u8,
                ])),
                Arc::new(StringArray::from(vec![Some("v1"), None, Some("x")])),
                Arc::new(Int64Array::from(vec![None, Some(99), None])),
            ],
        )
        .unwrap();
        h_plain.populate::<UInt16Type>(Some(&plain_rb)).unwrap();

        // Dict8 batch for the same attributes.
        let d8_keys = DictionaryArray::<UInt8Type>::try_new(
            UInt8Array::from(vec![0, 1]),
            Arc::new(StringArray::from(vec!["k_s", "k_i"])),
        )
        .unwrap();
        let d8_strs = DictionaryArray::<UInt8Type>::try_new(
            UInt8Array::from(vec![Some(0), None]),
            Arc::new(StringArray::from(vec![Some("v1")])),
        )
        .unwrap();
        let d8_ints = DictionaryArray::<UInt8Type>::try_new(
            UInt8Array::from(vec![None, Some(0)]),
            Arc::new(Int64Array::from(vec![Some(99)])),
        )
        .unwrap();
        let dict8_rb = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false),
                Field::new(consts::ATTRIBUTE_KEY, d8_keys.data_type().clone(), false),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(consts::ATTRIBUTE_STR, d8_strs.data_type().clone(), true),
                Field::new(consts::ATTRIBUTE_INT, d8_ints.data_type().clone(), true),
            ])),
            vec![
                Arc::new(UInt16Array::from(vec![1, 1])),
                Arc::new(d8_keys),
                Arc::new(UInt8Array::from(vec![
                    AttributeValueType::Str as u8,
                    AttributeValueType::Int as u8,
                ])),
                Arc::new(d8_strs),
                Arc::new(d8_ints),
            ],
        )
        .unwrap();
        h_dict8.populate::<UInt16Type>(Some(&dict8_rb)).unwrap();

        // Dict16 batch for the same attributes.
        let d16_keys = DictionaryArray::<UInt16Type>::try_new(
            UInt16Array::from(vec![0, 1]),
            Arc::new(StringArray::from(vec!["k_s", "k_i"])),
        )
        .unwrap();
        let d16_strs = DictionaryArray::<UInt16Type>::try_new(
            UInt16Array::from(vec![Some(0), None]),
            Arc::new(StringArray::from(vec![Some("v1")])),
        )
        .unwrap();
        let d16_ints = DictionaryArray::<UInt16Type>::try_new(
            UInt16Array::from(vec![None, Some(0)]),
            Arc::new(Int64Array::from(vec![Some(99)])),
        )
        .unwrap();
        let dict16_rb = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false),
                Field::new(consts::ATTRIBUTE_KEY, d16_keys.data_type().clone(), false),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(consts::ATTRIBUTE_STR, d16_strs.data_type().clone(), true),
                Field::new(consts::ATTRIBUTE_INT, d16_ints.data_type().clone(), true),
            ])),
            vec![
                Arc::new(UInt16Array::from(vec![1, 1])),
                Arc::new(d16_keys),
                Arc::new(UInt8Array::from(vec![
                    AttributeValueType::Str as u8,
                    AttributeValueType::Int as u8,
                ])),
                Arc::new(d16_strs),
                Arc::new(d16_ints),
            ],
        )
        .unwrap();
        h_dict16.populate::<UInt16Type>(Some(&dict16_rb)).unwrap();

        assert_eq!(h_plain.get(1), h_dict8.get(1));
        assert_eq!(h_plain.get(1), h_dict16.get(1));

        // Omitted optional columns vs explicit canonical defaults (str="", bool=false, bytes=b"", map=b"", empty/null type).
        let mut h_omitted = AttrHasher::new();
        let mut h_explicit = AttrHasher::new();
        let types_col = UInt8Array::from(vec![
            Some(AttributeValueType::Str as u8),
            Some(AttributeValueType::Bool as u8),
            Some(AttributeValueType::Bytes as u8),
            Some(AttributeValueType::Map as u8),
            None,
        ]);
        let keys_col = StringArray::from(vec!["s", "b", "by", "m", "e"]);
        let pids_col = UInt16Array::from(vec![3; 5]);

        let omitted_rb = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false),
                Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, true),
            ])),
            vec![
                Arc::new(pids_col.clone()),
                Arc::new(keys_col.clone()),
                Arc::new(types_col.clone()),
            ],
        )
        .unwrap();
        let explicit_rb = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false),
                Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, true),
                Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
                Field::new(consts::ATTRIBUTE_BOOL, DataType::Boolean, true),
                Field::new(consts::ATTRIBUTE_BYTES, DataType::Binary, true),
                Field::new(consts::ATTRIBUTE_SER, DataType::Binary, true),
            ])),
            vec![
                Arc::new(pids_col),
                Arc::new(keys_col),
                Arc::new(types_col),
                Arc::new(StringArray::from(vec![Some(""), None, None, None, None])),
                Arc::new(BooleanArray::from(vec![
                    None,
                    Some(false),
                    None,
                    None,
                    None,
                ])),
                Arc::new(BinaryArray::from_opt_vec(vec![
                    None,
                    None,
                    Some(b"".as_slice()),
                    None,
                    None,
                ])),
                Arc::new(BinaryArray::from_opt_vec(vec![
                    None,
                    None,
                    None,
                    Some(b"".as_slice()),
                    None,
                ])),
            ],
        )
        .unwrap();

        h_omitted.populate::<UInt16Type>(Some(&omitted_rb)).unwrap();
        h_explicit
            .populate::<UInt16Type>(Some(&explicit_rb))
            .unwrap();
        assert_eq!(h_omitted.get(3), h_explicit.get(3));
    }

    /// Scenario: Constructing and querying ExpHistogramState with 32_768 positive buckets and a scale > i16::MAX, and shrinking SubtractInitialCache after a cardinality spike.
    /// Guarantees: Bucket indexing never overflows u16, scale is preserved without truncation, and GC shrinks map capacity after sweeping evicted resources.
    #[test]
    fn test_exp_histogram_large_bucket_count_and_gc_shrink() {
        let large_pos = vec![1u64; 32_768];
        let large_neg = vec![7u64, 9u64];
        let mut st = ExpHistogramState::new(
            70_000,
            32_784,
            Some(100.0),
            0,
            5,
            &large_pos,
            -3,
            &large_neg,
        );
        assert_eq!(st.scale, 70_000);
        assert_eq!(st.ref_pos().1.len(), 32_768);
        assert_eq!(st.prev_pos().1.len(), 32_768);
        assert_eq!(st.ref_neg(), (-3, large_neg.as_slice()));
        assert_eq!(st.prev_neg(), (-3, large_neg.as_slice()));

        st.update_prev(1, 5, &large_pos, -3, &[8, 10]);
        assert_eq!(st.prev_neg(), (-3, &[8u64, 10u64][..]));

        // Cardinality spike followed by two GC sweeps shrinks the resource map capacity.
        let t0 = Instant::now();
        let mut cache = SubtractInitialCache::new(Duration::from_secs(10), t0);
        for i in 0u64..200 {
            let res = cache.resource_mut([i, 1]);
            let _ = res.numbers.series_mut("m").insert(
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
}
