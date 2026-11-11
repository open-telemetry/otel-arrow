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

#[derive(Debug, Default)]
struct IdHashAccum {
    dense: Vec<(u64, u64)>,
    max_dense_id: Option<usize>,
    sparse: HashMap<u32, (u64, u64)>,
}

impl IdHashAccum {
    fn clear(&mut self) {
        if let Some(max_id) = self.max_dense_id.take() {
            let end = (max_id + 1).min(self.dense.len());
            self.dense[..end].fill((0, 0));
        }
        self.sparse.clear();
    }

    #[inline]
    fn add(&mut self, parent_id: u32, lo: u64, hi: u64) {
        let idx = parent_id as usize;
        if idx < DENSE_ID_LIMIT {
            if idx >= self.dense.len() {
                let next_len = (idx + 1).next_power_of_two().min(DENSE_ID_LIMIT);
                self.dense.resize(next_len, (0, 0));
            }
            self.max_dense_id = Some(self.max_dense_id.map_or(idx, |cur| cur.max(idx)));
            let slot = &mut self.dense[idx];
            slot.0 = slot.0.wrapping_add(lo);
            slot.1 = slot.1.wrapping_add(hi);
        } else {
            let slot = self.sparse.entry(parent_id).or_insert((0, 0));
            slot.0 = slot.0.wrapping_add(lo);
            slot.1 = slot.1.wrapping_add(hi);
        }
    }

    #[inline]
    fn get(&self, parent_id: u32) -> u128 {
        let idx = parent_id as usize;
        let (lo, hi) = if idx < DENSE_ID_LIMIT {
            self.dense.get(idx).copied().unwrap_or((0, 0))
        } else {
            self.sparse.get(&parent_id).copied().unwrap_or((0, 0))
        };
        ((hi as u128) << 64) | (lo as u128)
    }
}

/// Order-independent 128-bit multiset attribute hasher for OTAP `Attributes` record batches.
///
/// Pre-hashes dictionary key/value arrays once per batch so per-row hashing reduces to array
/// lookups and 128-bit integer mixing, while producing identical hashes regardless of whether
/// columns are dictionary-encoded, plain, or omitted due to default values.
#[derive(Debug, Default)]
pub struct AttrHasher {
    accum: IdHashAccum,
    key_dict_hashes: Vec<u128>,
    str_dict_hashes: Vec<u128>,
}

impl AttrHasher {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    #[must_use]
    pub fn get(&self, parent_id: u32) -> u128 {
        self.accum.get(parent_id)
    }

    pub fn populate<P>(&mut self, attrs_rb: Option<&RecordBatch>) -> Result<(), PdataError>
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

/// Wrapper storing mark-and-sweep liveness and adjusted `start_time_ns` alongside per-type state.
#[derive(Debug, Clone)]
pub struct SeriesEntry<S> {
    pub mark: bool,
    pub start_time_ns: i64,
    pub data: S,
}

#[derive(Debug, Clone, Copy)]
pub struct NumberState {
    pub ref_double: f64,
    pub ref_int: i64,
    pub prev_double: f64,
    pub prev_int: i64,
}

/// Shared `count` + `sum` baseline and previous-point tracker for `Summary`, `Histogram`, and
/// `ExponentialHistogram`.
#[derive(Debug, Clone, Copy)]
pub struct CountSumState {
    pub ref_count: u64,
    pub ref_sum: f64,
    pub prev_count: u64,
    pub prev_sum: f64,
}

impl CountSumState {
    #[inline]
    #[must_use]
    pub fn new(count: u64, sum: f64) -> Self {
        Self {
            ref_count: count,
            ref_sum: sum,
            prev_count: count,
            prev_sum: sum,
        }
    }

    #[inline]
    #[must_use]
    pub fn decreased(&self, count: u64, sum: f64) -> bool {
        count < self.prev_count || sum < self.prev_sum
    }

    #[inline]
    pub fn reset(&mut self, count: u64, sum: f64) {
        self.ref_count = 0;
        self.ref_sum = 0.0;
        self.prev_count = count;
        self.prev_sum = sum;
    }

    #[inline]
    pub fn advance(&mut self, count: u64, sum: f64) {
        self.prev_count = count;
        self.prev_sum = sum;
    }
}

#[derive(Debug, Clone)]
pub struct HistogramState {
    pub cs: CountSumState,
    pub explicit_bounds_bits: Vec<u64>,
    pub ref_buckets: Vec<u64>,
    pub prev_buckets: Vec<u64>,
}

#[derive(Debug, Clone, Default)]
pub struct ExpBucketsState {
    pub offset: i32,
    pub counts: Vec<u64>,
}

impl ExpBucketsState {
    #[inline]
    #[must_use]
    pub fn new(offset: i32, counts: &[u64]) -> Self {
        Self {
            offset,
            counts: counts.to_vec(),
        }
    }

    #[inline]
    pub fn update(&mut self, offset: i32, counts: &[u64]) {
        self.offset = offset;
        self.counts.clear();
        self.counts.extend_from_slice(counts);
    }

    #[inline]
    pub fn reset_zero(&mut self, offset: i32, len: usize) {
        self.offset = offset;
        self.counts.clear();
        self.counts.resize(len, 0);
    }
}

#[derive(Debug, Clone)]
pub struct ExpHistogramState {
    pub scale: i32,
    pub cs: CountSumState,
    pub ref_zero_count: u64,
    pub prev_zero_count: u64,
    pub ref_positive: ExpBucketsState,
    pub prev_positive: ExpBucketsState,
    pub ref_negative: ExpBucketsState,
    pub prev_negative: ExpBucketsState,
}

/// Per-metric series map (`metric_name -> dp_attr_hash -> SeriesEntry<S>`).
#[derive(Debug)]
pub struct SeriesTable<S>(HashMap<String, HashMap<u128, SeriesEntry<S>>>);

impl<S> Default for SeriesTable<S> {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

impl<S> SeriesTable<S> {
    #[inline]
    pub fn series_mut(&mut self, metric_name: &str) -> &mut HashMap<u128, SeriesEntry<S>> {
        match self.0.entry_ref(metric_name) {
            EntryRef::Occupied(entry) => entry.into_mut(),
            EntryRef::Vacant(entry) => entry.insert(HashMap::new()),
        }
    }

    fn sweep(&mut self) -> bool {
        self.0.retain(|_, series_map| {
            series_map.retain(|_, entry| std::mem::replace(&mut entry.mark, false));
            !series_map.is_empty()
        });
        !self.0.is_empty()
    }
}

#[derive(Debug, Default)]
pub struct ResourceSeriesMap {
    pub mark: bool,
    pub numbers: SeriesTable<NumberState>,
    pub summaries: SeriesTable<CountSumState>,
    pub histograms: SeriesTable<HistogramState>,
    pub exp_histograms: SeriesTable<ExpHistogramState>,
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
pub struct SubtractInitialCache {
    gc_interval: Duration,
    last_gc: Instant,
    resources: HashMap<u128, ResourceSeriesMap>,
}

impl SubtractInitialCache {
    #[must_use]
    pub fn new(gc_interval: Duration, now: Instant) -> Self {
        Self {
            gc_interval,
            last_gc: now,
            resources: HashMap::new(),
        }
    }

    #[inline]
    pub fn resource_mut(&mut self, resource_hash: u128) -> &mut ResourceSeriesMap {
        let res = self.resources.entry(resource_hash).or_default();
        res.mark = true;
        res
    }

    pub fn maybe_gc(&mut self, now: Instant) {
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
        self.last_gc = now;
    }

    #[cfg(test)]
    pub(crate) fn resource_count(&self) -> usize {
        self.resources.len()
    }
}
