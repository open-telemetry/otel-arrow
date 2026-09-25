// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Column writers for [`super::concatenate`].
//!
//! Each output column is written exactly once: a destination buffer is sized
//! for the full output and every selected range of every input is copied (and,
//! when needed, cast and/or ID-remapped) directly into it. There is no
//! intermediate per-input converted array and no final coalescing copy.
//!
//! # Structure
//!
//! - [ValueBuilder] implementations know how to append values of one physical
//!   type (primitive, boolean, bytes, fixed size binary) from a source array,
//!   either by contiguous range or by gathering indices.
//! - [write_native] drives a [ValueBuilder] to produce a plain (non dictionary)
//!   output column from native, dictionary, or missing inputs.
//! - [write_dict] drives a [ValueBuilder] for the dictionary values and builds
//!   output keys directly. Dictionary inputs append their whole values array
//!   and shift their keys; native inputs append their selected values with
//!   sequential keys. No hashing is performed.
//! - [write_struct] recurses into struct children.
//! - [write_fallback] handles anything else (currently only List columns)
//!   via `MutableArrayData`.
//!
//! Every writer honors array offsets (sliced inputs) for both values and
//! nulls, and null buffers are only materialized if a null is encountered.
//!
//! # TODO
//!
//! - TODO(list-writer): Replace [write_fallback] for List columns with a
//!   specialized writer that copies offsets and child values directly.
//! - TODO(dict-dedupe): [write_dict] appends the whole values array of every
//!   dictionary input. Skip values arrays already appended (pointer identity)
//!   and trim unreferenced values for heavily sliced inputs.
//! - TODO(bytes-gather-capacity): Byte capacity for dictionary inputs is sized
//!   from the whole values array, which over-allocates when the column is
//!   gathered into a native output. Size it from the selected keys instead.

use std::ops::Range;
use std::sync::Arc;

use arrow::array::BooleanBufferBuilder;
use arrow::array::{
    Array, ArrayData, ArrayRef, ArrowNativeTypeOp, ArrowPrimitiveType, AsArray, BooleanArray,
    DictionaryArray, FixedSizeBinaryArray, GenericByteArray, MutableArrayData, PrimitiveArray,
    StructArray, make_array,
};
use arrow::buffer::{Buffer, NullBuffer, OffsetBuffer, ScalarBuffer};
use arrow::compute::kernels::cast;
use arrow::datatypes::{
    ArrowDictionaryKeyType, ArrowNativeType, BinaryType, ByteArrayType, DurationNanosecondType,
    Float64Type, Int32Type, Int64Type, TimestampNanosecondType, UInt8Type, UInt16Type, UInt32Type,
    UInt64Type, Utf8Type,
};
use arrow_schema::{ArrowError, DataType, Field, Fields};

use super::plan::{AnyRemap, IdCol, IdRemap, InputPlan};
use crate::error::{Error, Result};
use crate::schema::consts::{ID, PARENT_ID, RESOURCE, SCOPE};

/// One input's contribution to an output column.
#[derive(Clone, Copy)]
pub(crate) struct Input<'a> {
    /// The source column, or `None` if this input does not carry the field.
    pub(crate) column: Option<&'a ArrayRef>,
    /// Number of rows in the source record batch.
    pub(crate) num_rows: usize,
    /// Row selection and ID remaps for this input.
    pub(crate) plan: &'a InputPlan,
}

impl Input<'_> {
    fn selected_rows(&self) -> usize {
        self.plan.selection.count(self.num_rows)
    }

    fn ranges(&self) -> impl Iterator<Item = Range<usize>> + '_ {
        self.plan.selection.ranges(self.num_rows)
    }
}

/// Resolve which ID column (if any) a top-level field corresponds to.
#[must_use]
pub(crate) fn top_level_id_col(name: &str) -> Option<IdCol> {
    match name {
        ID => Some(IdCol::Id),
        PARENT_ID => Some(IdCol::ParentId),
        _ => None,
    }
}

fn struct_child_id_col(struct_name: &str, child_name: &str) -> Option<IdCol> {
    match (struct_name, child_name) {
        (RESOURCE, ID) => Some(IdCol::ResourceId),
        (SCOPE, ID) => Some(IdCol::ScopeId),
        _ => None,
    }
}

/// Write a single output column for `target` from `inputs`.
///
/// `rows` must equal the sum of selected rows across `inputs`.
pub(crate) fn write_column(
    target: &Field,
    inputs: &[Input<'_>],
    rows: usize,
    id_col: Option<IdCol>,
) -> Result<ArrayRef> {
    debug_assert_eq!(rows, inputs.iter().map(Input::selected_rows).sum::<usize>());

    match target.data_type() {
        DataType::Struct(fields) => write_struct(target.name(), fields, inputs, rows),
        DataType::Dictionary(key, value) => match key.as_ref() {
            DataType::UInt8 => dispatch_value::<DictDriver<UInt8Type>>(value, inputs, rows, id_col),
            DataType::UInt16 => {
                dispatch_value::<DictDriver<UInt16Type>>(value, inputs, rows, id_col)
            }
            _ => write_fallback(target.data_type(), inputs, rows),
        },
        value => dispatch_value::<NativeDriver>(value, inputs, rows, id_col),
    }
}

// ---------------------------------------------------------------------------
// Drivers
// ---------------------------------------------------------------------------

/// Abstraction over [write_native] and [write_dict] so value-type dispatch is
/// written once.
trait Driver {
    /// True if the output is dictionary encoded.
    const IS_DICT: bool;

    fn write<B: ValueBuilder>(
        builder: B,
        value_type: &DataType,
        inputs: &[Input<'_>],
        rows: usize,
    ) -> Result<ArrayRef>;
}

struct NativeDriver;
impl Driver for NativeDriver {
    const IS_DICT: bool = false;

    fn write<B: ValueBuilder>(
        builder: B,
        value_type: &DataType,
        inputs: &[Input<'_>],
        rows: usize,
    ) -> Result<ArrayRef> {
        write_native(builder, value_type, inputs, rows)
    }
}

struct DictDriver<K>(std::marker::PhantomData<K>);
impl<K: ArrowDictionaryKeyType> Driver for DictDriver<K> {
    const IS_DICT: bool = true;

    fn write<B: ValueBuilder>(
        builder: B,
        value_type: &DataType,
        inputs: &[Input<'_>],
        rows: usize,
    ) -> Result<ArrayRef> {
        write_dict::<K, B>(builder, value_type, inputs, rows)
    }
}

/// Upper bound on the number of values the builder will receive, used for
/// capacity. For native output this is `rows`; for dictionary output this is
/// the sum of dictionary value lengths plus native selected rows.
fn dict_values_capacity(inputs: &[Input<'_>]) -> usize {
    inputs
        .iter()
        .map(|inp| match inp.column {
            Some(col) => match col.data_type() {
                DataType::Dictionary(_, _) => dict_values(col).map(|v| v.len()).unwrap_or(0),
                _ => inp.selected_rows(),
            },
            None => 0,
        })
        .sum()
}

fn dispatch_value<D: Driver>(
    value_type: &DataType,
    inputs: &[Input<'_>],
    rows: usize,
    id_col: Option<IdCol>,
) -> Result<ArrayRef> {
    let cap = if D::IS_DICT {
        dict_values_capacity(inputs)
    } else {
        rows
    };

    macro_rules! prim {
        ($t:ty) => {
            D::write(
                PrimitiveBuilder::<$t>::new(cap, value_type.clone(), None),
                value_type,
                inputs,
                rows,
            )
        };
    }

    match value_type {
        DataType::UInt8 => prim!(UInt8Type),
        DataType::UInt16 => D::write(
            PrimitiveBuilder::<UInt16Type>::new(cap, value_type.clone(), id_col.map(u16_remap)),
            value_type,
            inputs,
            rows,
        ),
        DataType::UInt32 => D::write(
            PrimitiveBuilder::<UInt32Type>::new(cap, value_type.clone(), id_col.map(u32_remap)),
            value_type,
            inputs,
            rows,
        ),
        DataType::UInt64 => prim!(UInt64Type),
        DataType::Int32 => prim!(Int32Type),
        DataType::Int64 => prim!(Int64Type),
        DataType::Float64 => prim!(Float64Type),
        DataType::Timestamp(arrow_schema::TimeUnit::Nanosecond, _) => {
            prim!(TimestampNanosecondType)
        }
        DataType::Duration(arrow_schema::TimeUnit::Nanosecond) => prim!(DurationNanosecondType),
        DataType::Boolean => D::write(BoolBuilder::new(cap), value_type, inputs, rows),
        DataType::Utf8 => D::write(
            BytesBuilder::<Utf8Type>::new(cap, inputs),
            value_type,
            inputs,
            rows,
        ),
        DataType::Binary => D::write(
            BytesBuilder::<BinaryType>::new(cap, inputs),
            value_type,
            inputs,
            rows,
        ),
        DataType::FixedSizeBinary(w) => {
            D::write(FsbBuilder::new(cap, *w as usize), value_type, inputs, rows)
        }
        _ if D::IS_DICT => Err(Error::UnexpectedRecordBatchState {
            reason: format!("unsupported dictionary value type {value_type:?}"),
        }),
        _ => write_fallback(value_type, inputs, rows),
    }
}

/// Produce a plain output column.
fn write_native<B: ValueBuilder>(
    mut builder: B,
    value_type: &DataType,
    inputs: &[Input<'_>],
    rows: usize,
) -> Result<ArrayRef> {
    let mut nulls = LazyNulls::new(rows);

    for inp in inputs {
        builder.begin_input(inp.plan);
        let Some(col) = inp.column else {
            let n = inp.selected_rows();
            builder.append_default(n);
            nulls.append_null(n);
            continue;
        };

        match col.data_type() {
            DataType::Dictionary(key, _) => match key.as_ref() {
                DataType::UInt8 => gather_dict::<UInt8Type, B>(&mut builder, &mut nulls, inp, col),
                DataType::UInt16 => {
                    gather_dict::<UInt16Type, B>(&mut builder, &mut nulls, inp, col)
                }
                k => return Err(unsupported_key(k)),
            },
            dt => {
                check_value_type(dt, value_type)?;
                let src = B::downcast(col.as_ref());
                for r in inp.ranges() {
                    builder.append_range(&src, r.clone());
                    nulls.append_from(col.nulls(), r);
                }
            }
        }
    }

    builder.finish(nulls.finish())
}

/// Gather values through dictionary keys into a native builder.
fn gather_dict<K: ArrowDictionaryKeyType, B: ValueBuilder>(
    builder: &mut B,
    nulls: &mut LazyNulls,
    inp: &Input<'_>,
    col: &ArrayRef,
) {
    let dict = col.as_dictionary::<K>();
    let values = dict.values();
    let keys = dict.keys().values();
    let values_len = values.len();
    let src = B::downcast(values.as_ref());

    for r in inp.ranges() {
        if values_len == 0 {
            // Every key must be null.
            builder.append_default(r.len());
        } else {
            // Null keys may hold arbitrary values; clamp so the gather is
            // always in bounds. The null buffer masks the result.
            let max = values_len - 1;
            builder.append_gather::<K::Native>(&src, &keys[r.clone()], max);
        }
        // A null value referenced by a valid key is also null in the output.
        match values.nulls() {
            Some(vn) if vn.null_count() > 0 => {
                for k in keys[r.clone()].iter().zip(r.clone()) {
                    let (key, row) = k;
                    let valid = dict.keys().is_valid(row)
                        && key.as_usize() < values_len
                        && vn.is_valid(key.as_usize());
                    if valid {
                        nulls.append_valid(1);
                    } else {
                        nulls.append_null(1);
                    }
                }
            }
            _ => nulls.append_from(dict.nulls(), r),
        }
    }
}

/// Produce a dictionary output column with key type `K`.
fn write_dict<K: ArrowDictionaryKeyType, B: ValueBuilder>(
    mut builder: B,
    value_type: &DataType,
    inputs: &[Input<'_>],
    rows: usize,
) -> Result<ArrayRef> {
    let mut keys: Vec<K::Native> = Vec::with_capacity(rows);
    let mut key_nulls = LazyNulls::new(rows);
    let mut value_nulls = LazyNulls::new(dict_values_capacity(inputs));
    let mut vbase: usize = 0;

    for inp in inputs {
        builder.begin_input(inp.plan);
        let Some(col) = inp.column else {
            let n = inp.selected_rows();
            keys.resize(keys.len() + n, K::Native::usize_as(0));
            key_nulls.append_null(n);
            continue;
        };

        match col.data_type() {
            DataType::Dictionary(key, _) => match key.as_ref() {
                DataType::UInt8 => append_dict_input::<UInt8Type, K, B>(
                    &mut builder,
                    &mut keys,
                    &mut key_nulls,
                    &mut value_nulls,
                    &mut vbase,
                    inp,
                    col,
                    value_type,
                )?,
                DataType::UInt16 => append_dict_input::<UInt16Type, K, B>(
                    &mut builder,
                    &mut keys,
                    &mut key_nulls,
                    &mut value_nulls,
                    &mut vbase,
                    inp,
                    col,
                    value_type,
                )?,
                k => return Err(unsupported_key(k)),
            },
            dt => {
                check_value_type(dt, value_type)?;
                let src = B::downcast(col.as_ref());
                for r in inp.ranges() {
                    let n = r.len();
                    builder.append_range(&src, r.clone());
                    value_nulls.append_valid(n);
                    keys.extend((vbase..vbase + n).map(K::Native::usize_as));
                    key_nulls.append_from(col.nulls(), r);
                    vbase += n;
                }
            }
        }
    }

    // The schema selection bounds the total number of physical values so
    // that they fit the chosen key type. Guard anyway rather than wrapping.
    if vbase > 0 && K::Native::from_usize(vbase - 1).is_none() {
        return Err(Error::Batching {
            source: ArrowError::DictionaryKeyOverflowError,
        });
    }

    let values = builder.finish(value_nulls.finish())?;
    let keys = PrimitiveArray::<K>::new(ScalarBuffer::from(keys), key_nulls.finish());
    debug_assert!(DictionaryArray::<K>::try_new(keys.clone(), values.clone()).is_ok());

    // SAFETY: every valid key was produced as `vbase + k` where `k` is a
    // valid index into the values appended for that input (either a source
    // dictionary key, which the source array guarantees to be in bounds, or
    // a sequential index into the values just appended). Null keys may hold
    // arbitrary values, which Arrow permits.
    #[allow(unsafe_code)]
    let dict = unsafe { DictionaryArray::<K>::new_unchecked(keys, values) };
    Ok(Arc::new(dict))
}

#[allow(clippy::too_many_arguments)]
fn append_dict_input<Ks: ArrowDictionaryKeyType, K: ArrowDictionaryKeyType, B: ValueBuilder>(
    builder: &mut B,
    keys: &mut Vec<K::Native>,
    key_nulls: &mut LazyNulls,
    value_nulls: &mut LazyNulls,
    vbase: &mut usize,
    inp: &Input<'_>,
    col: &ArrayRef,
    value_type: &DataType,
) -> Result<()> {
    let dict = col.as_dictionary::<Ks>();
    let values = dict.values();
    check_value_type(values.data_type(), value_type)?;

    // Append the whole values array. This keeps key rewriting a pure add.
    let values_len = values.len();
    let src = B::downcast(values.as_ref());
    builder.append_range(&src, 0..values_len);
    value_nulls.append_from(values.nulls(), 0..values_len);

    let src_keys = dict.keys().values();
    let base = *vbase;
    for r in inp.ranges() {
        keys.extend(
            src_keys[r.clone()]
                .iter()
                .map(|k| K::Native::usize_as(k.as_usize() + base)),
        );
        key_nulls.append_from(dict.nulls(), r);
    }
    *vbase += values_len;
    Ok(())
}

/// Produce a struct output column, recursing into its children.
fn write_struct(
    struct_name: &str,
    fields: &Fields,
    inputs: &[Input<'_>],
    rows: usize,
) -> Result<ArrayRef> {
    let mut nulls = LazyNulls::new(rows);
    let structs: Vec<Option<&StructArray>> = inputs
        .iter()
        .map(|inp| inp.column.map(|c| c.as_struct()))
        .collect();

    for (inp, s) in inputs.iter().zip(&structs) {
        match s {
            Some(s) => {
                for r in inp.ranges() {
                    nulls.append_from(s.nulls(), r);
                }
            }
            None => nulls.append_null(inp.selected_rows()),
        }
    }

    let mut children = Vec::with_capacity(fields.len());
    let mut child_inputs: Vec<Input<'_>> = Vec::with_capacity(inputs.len());
    for field in fields.iter() {
        child_inputs.clear();
        child_inputs.extend(inputs.iter().zip(&structs).map(|(inp, s)| Input {
            column: s.and_then(|s| s.column_by_name(field.name())),
            num_rows: inp.num_rows,
            plan: inp.plan,
        }));
        let id_col = struct_child_id_col(struct_name, field.name());
        children.push(write_column(field, &child_inputs, rows, id_col)?);
    }

    let array = StructArray::try_new_with_length(fields.clone(), children, nulls.finish(), rows)
        .map_err(|source| Error::Batching { source })?;
    Ok(Arc::new(array))
}

/// Generic fallback using `MutableArrayData`. Used for List columns. Inputs of
/// a different type are cast to `target` first.
fn write_fallback(target: &DataType, inputs: &[Input<'_>], rows: usize) -> Result<ArrayRef> {
    let mut casted: Vec<ArrayRef> = Vec::with_capacity(inputs.len());
    for inp in inputs {
        if let Some(col) = inp.column {
            let col = if col.data_type() == target {
                Arc::clone(col)
            } else {
                cast(col.as_ref(), target).map_err(|source| Error::Batching { source })?
            };
            casted.push(col);
        }
    }

    let datas: Vec<ArrayData> = casted.iter().map(|a| a.to_data()).collect();
    let refs: Vec<&ArrayData> = datas.iter().collect();
    if refs.is_empty() {
        return Ok(arrow::array::new_null_array(target, rows));
    }

    let mut mutable = MutableArrayData::new(refs, true, rows);
    let mut src_idx = 0;
    for inp in inputs {
        match inp.column {
            Some(_) => {
                for r in inp.ranges() {
                    mutable.extend(src_idx, r.start, r.end);
                }
                src_idx += 1;
            }
            None => mutable.extend_nulls(inp.selected_rows()),
        }
    }

    Ok(make_array(mutable.freeze()))
}

fn check_value_type(actual: &DataType, expected: &DataType) -> Result<()> {
    if actual == expected {
        Ok(())
    } else {
        Err(Error::ColumnDataTypeMismatch {
            name: String::new(),
            expect: expected.clone(),
            actual: actual.clone(),
        })
    }
}

fn unsupported_key(k: &DataType) -> Error {
    Error::UnsupportedDictionaryKeyType {
        expect_oneof: vec![DataType::UInt8, DataType::UInt16],
        actual: k.clone(),
    }
}

fn dict_values(col: &ArrayRef) -> Option<&ArrayRef> {
    match col.data_type() {
        DataType::Dictionary(k, _) => match k.as_ref() {
            DataType::UInt8 => Some(col.as_dictionary::<UInt8Type>().values()),
            DataType::UInt16 => Some(col.as_dictionary::<UInt16Type>().values()),
            _ => None,
        },
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Null buffer
// ---------------------------------------------------------------------------

/// A null buffer builder that allocates only once a null is seen.
pub(crate) struct LazyNulls {
    len: usize,
    capacity: usize,
    builder: Option<BooleanBufferBuilder>,
}

impl LazyNulls {
    pub(crate) fn new(capacity: usize) -> Self {
        Self {
            len: 0,
            capacity,
            builder: None,
        }
    }

    fn materialize(&mut self) -> &mut BooleanBufferBuilder {
        let (len, capacity) = (self.len, self.capacity);
        self.builder.get_or_insert_with(|| {
            let mut b = BooleanBufferBuilder::new(capacity.max(len));
            b.append_n(len, true);
            b
        })
    }

    pub(crate) fn append_valid(&mut self, n: usize) {
        if let Some(b) = self.builder.as_mut() {
            b.append_n(n, true);
        }
        self.len += n;
    }

    pub(crate) fn append_null(&mut self, n: usize) {
        if n == 0 {
            return;
        }
        self.materialize().append_n(n, false);
        self.len += n;
    }

    /// Append validity for `range` (logical indices) of a source null buffer.
    pub(crate) fn append_from(&mut self, nulls: Option<&NullBuffer>, range: Range<usize>) {
        match nulls {
            Some(n) if n.null_count() > 0 => {
                let inner = n.inner();
                let offset = inner.offset();
                let len = range.len();
                self.materialize()
                    .append_packed_range(offset + range.start..offset + range.end, inner.values());
                self.len += len;
            }
            _ => self.append_valid(range.len()),
        }
    }

    pub(crate) fn finish(self) -> Option<NullBuffer> {
        self.builder
            .map(|mut b| NullBuffer::new(b.finish()))
            .filter(|n| n.null_count() > 0)
    }
}

// ---------------------------------------------------------------------------
// Value builders
// ---------------------------------------------------------------------------

/// Appends values of a single physical type into a pre-sized destination.
trait ValueBuilder {
    /// Typed view of a source array.
    type Src<'a>;

    fn downcast(array: &dyn Array) -> Self::Src<'_>;

    /// Called before each input is processed.
    fn begin_input(&mut self, _plan: &InputPlan) {}

    /// Append logical indices `range` of `src`.
    fn append_range(&mut self, src: &Self::Src<'_>, range: Range<usize>);

    /// Append `src[min(k, max)]` for each dictionary key `k` in `keys`.
    ///
    /// Keys are clamped to `max` (the last valid index) because null keys may
    /// hold arbitrary values; the null buffer masks those rows.
    fn append_gather<K: ArrowNativeType>(&mut self, src: &Self::Src<'_>, keys: &[K], max: usize);

    /// Append `n` placeholder values (masked by nulls).
    fn append_default(&mut self, n: usize);

    fn finish(self, nulls: Option<NullBuffer>) -> Result<ArrayRef>;
}

/// Extracts the typed remap for an ID column from an [InputPlan].
type RemapFn<T> = fn(&InputPlan, IdCol) -> Option<IdRemap<T>>;

fn u16_remap(col: IdCol) -> (IdCol, RemapFn<u16>) {
    (col, |plan, col| match plan.remap(col) {
        Some(AnyRemap::U16(r)) => Some(r.clone()),
        _ => None,
    })
}

fn u32_remap(col: IdCol) -> (IdCol, RemapFn<u32>) {
    (col, |plan, col| match plan.remap(col) {
        Some(AnyRemap::U32(r)) => Some(r.clone()),
        _ => None,
    })
}

/// Split dictionary keys into maximal runs of consecutive value indices and
/// call `f` with each run as a range of value indices, in key order.
///
/// Keys are clamped to `max` (see [ValueBuilder::append_gather]). A run of
/// length 1 is a single value.
#[inline]
fn for_each_key_run<K: ArrowNativeType>(keys: &[K], max: usize, mut f: impl FnMut(Range<usize>)) {
    let mut iter = keys.iter().map(|k| k.as_usize().min(max));
    let Some(first) = iter.next() else {
        return;
    };
    let mut start = first;
    let mut end = first + 1;
    for k in iter {
        if k == end {
            end += 1;
        } else {
            f(start..end);
            start = k;
            end = k + 1;
        }
    }
    f(start..end);
}

struct PrimitiveBuilder<T: ArrowPrimitiveType> {
    values: Vec<T::Native>,
    data_type: DataType,
    remap_source: Option<(IdCol, RemapFn<T::Native>)>,
    remap: IdRemap<T::Native>,
}

impl<T: ArrowPrimitiveType> PrimitiveBuilder<T> {
    fn new(
        capacity: usize,
        data_type: DataType,
        remap_source: Option<(IdCol, RemapFn<T::Native>)>,
    ) -> Self {
        Self {
            values: Vec::with_capacity(capacity),
            data_type,
            remap_source,
            remap: IdRemap::Identity,
        }
    }
}

impl<T: ArrowPrimitiveType> ValueBuilder for PrimitiveBuilder<T> {
    type Src<'a> = &'a [T::Native];

    fn downcast(array: &dyn Array) -> Self::Src<'_> {
        array.as_primitive::<T>().values()
    }

    fn begin_input(&mut self, plan: &InputPlan) {
        if let Some((col, f)) = self.remap_source {
            self.remap = f(plan, col).unwrap_or(IdRemap::Identity);
        }
    }

    #[inline]
    fn append_range(&mut self, src: &Self::Src<'_>, range: Range<usize>) {
        match &self.remap {
            IdRemap::Identity => self.values.extend_from_slice(&src[range]),
            IdRemap::Offset(d) => {
                let d = *d;
                self.values
                    .extend(src[range].iter().map(|v| v.add_wrapping(d)));
            }
            IdRemap::Replace(buf) => self.values.extend_from_slice(&buf[range]),
        }
    }

    #[inline]
    fn append_gather<K: ArrowNativeType>(&mut self, src: &Self::Src<'_>, keys: &[K], max: usize) {
        let indices = keys.iter().map(|k| k.as_usize().min(max));
        match &self.remap {
            IdRemap::Identity => self.values.extend(indices.map(|i| src[i])),
            IdRemap::Offset(d) => {
                let d = *d;
                self.values.extend(indices.map(|i| src[i].add_wrapping(d)));
            }
            IdRemap::Replace(buf) => self.values.extend(indices.map(|i| buf[i])),
        }
    }

    fn append_default(&mut self, n: usize) {
        self.values
            .resize(self.values.len() + n, T::Native::default());
    }

    fn finish(self, nulls: Option<NullBuffer>) -> Result<ArrayRef> {
        let array = PrimitiveArray::<T>::new(ScalarBuffer::from(self.values), nulls)
            .with_data_type(self.data_type);
        Ok(Arc::new(array))
    }
}

struct BoolBuilder {
    values: BooleanBufferBuilder,
}

impl BoolBuilder {
    fn new(capacity: usize) -> Self {
        Self {
            values: BooleanBufferBuilder::new(capacity),
        }
    }
}

impl ValueBuilder for BoolBuilder {
    type Src<'a> = &'a BooleanArray;

    fn downcast(array: &dyn Array) -> Self::Src<'_> {
        array.as_boolean()
    }

    fn append_range(&mut self, src: &Self::Src<'_>, range: Range<usize>) {
        let bits = src.values();
        let offset = bits.offset();
        self.values
            .append_packed_range(offset + range.start..offset + range.end, bits.values());
    }

    fn append_gather<K: ArrowNativeType>(&mut self, src: &Self::Src<'_>, keys: &[K], max: usize) {
        let bits = src.values();
        for k in keys {
            self.values.append(bits.value(k.as_usize().min(max)));
        }
    }

    fn append_default(&mut self, n: usize) {
        self.values.append_n(n, false);
    }

    fn finish(mut self, nulls: Option<NullBuffer>) -> Result<ArrayRef> {
        Ok(Arc::new(BooleanArray::new(self.values.finish(), nulls)))
    }
}

struct BytesBuilder<T: ByteArrayType<Offset = i32>> {
    offsets: Vec<i32>,
    data: Vec<u8>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: ByteArrayType<Offset = i32>> BytesBuilder<T> {
    fn new(capacity: usize, inputs: &[Input<'_>]) -> Self {
        // Exact byte capacity for native inputs; dictionary inputs are either
        // gathered (unknown, grows) or appended whole (exact).
        let mut bytes = 0usize;
        for inp in inputs {
            let Some(col) = inp.column else { continue };
            match col.data_type() {
                DataType::Dictionary(_, _) => {
                    if let Some(v) = dict_values(col) {
                        let o = v.as_bytes::<T>().value_offsets();
                        bytes += (o[o.len() - 1] - o[0]) as usize;
                    }
                }
                _ => {
                    let o = col.as_bytes::<T>().value_offsets();
                    for r in inp.ranges() {
                        bytes += (o[r.end] - o[r.start]) as usize;
                    }
                }
            }
        }

        let mut offsets = Vec::with_capacity(capacity + 1);
        offsets.push(0);
        Self {
            offsets,
            data: Vec::with_capacity(bytes),
            _phantom: std::marker::PhantomData,
        }
    }

    #[inline]
    fn last(&self) -> i32 {
        // safety: always contains the initial 0
        *self.offsets.last().expect("non-empty offsets")
    }
}

impl<T: ByteArrayType<Offset = i32>> ValueBuilder for BytesBuilder<T> {
    type Src<'a> = &'a GenericByteArray<T>;

    fn downcast(array: &dyn Array) -> Self::Src<'_> {
        array.as_bytes::<T>()
    }

    fn append_range(&mut self, src: &Self::Src<'_>, range: Range<usize>) {
        let o = src.value_offsets();
        let start = o[range.start];
        let end = o[range.end];
        self.data
            .extend_from_slice(&src.value_data()[start as usize..end as usize]);
        // Offsets may overflow i32 only if the output exceeds 2GiB; this is
        // checked in `finish`.
        let delta = self.last().wrapping_sub(start);
        self.offsets.extend(
            o[range.start + 1..=range.end]
                .iter()
                .map(|x| x.wrapping_add(delta)),
        );
    }

    fn append_gather<K: ArrowNativeType>(&mut self, src: &Self::Src<'_>, keys: &[K], max: usize) {
        // Runs of consecutive keys reference contiguous values, so each run
        // is copied like `append_range`: one memcpy for the bytes and one
        // shifted copy of the offsets.
        for_each_key_run(keys, max, |run| self.append_range(src, run));
    }

    fn append_default(&mut self, n: usize) {
        let last = self.last();
        self.offsets.resize(self.offsets.len() + n, last);
    }

    fn finish(self, nulls: Option<NullBuffer>) -> Result<ArrayRef> {
        if i32::try_from(self.data.len()).is_err() {
            return Err(Error::Batching {
                source: ArrowError::OffsetOverflowError(self.data.len()),
            });
        }

        let offsets = ScalarBuffer::from(self.offsets);
        let values = Buffer::from_vec(self.data);
        #[cfg(debug_assertions)]
        {
            let checked = OffsetBuffer::new(offsets.clone());
            debug_assert!(
                GenericByteArray::<T>::try_new(checked, values.clone(), nulls.clone()).is_ok()
            );
        }

        // SAFETY: offsets start at 0, are monotonically non-decreasing (each
        // appended value has non-negative length), and the last offset equals
        // `values.len()` which fits in i32 (checked above). Every value was
        // copied whole from a valid source array of the same type `T`, so for
        // Utf8 each value is valid UTF-8 and boundaries fall on char
        // boundaries.
        #[allow(unsafe_code)]
        let array = unsafe {
            let offsets = OffsetBuffer::new_unchecked(offsets);
            GenericByteArray::<T>::new_unchecked(offsets, values, nulls)
        };
        Ok(Arc::new(array))
    }
}

struct FsbBuilder {
    width: usize,
    data: Vec<u8>,
}

impl FsbBuilder {
    fn new(capacity: usize, width: usize) -> Self {
        Self {
            width,
            data: Vec::with_capacity(capacity * width),
        }
    }
}

impl ValueBuilder for FsbBuilder {
    type Src<'a> = &'a FixedSizeBinaryArray;

    fn downcast(array: &dyn Array) -> Self::Src<'_> {
        array.as_fixed_size_binary()
    }

    fn append_range(&mut self, src: &Self::Src<'_>, range: Range<usize>) {
        let w = self.width;
        self.data
            .extend_from_slice(&src.value_data()[range.start * w..range.end * w]);
    }

    fn append_gather<K: ArrowNativeType>(&mut self, src: &Self::Src<'_>, keys: &[K], max: usize) {
        // Runs of consecutive keys reference contiguous values: one memcpy per
        // run.
        for_each_key_run(keys, max, |run| self.append_range(src, run));
    }

    fn append_default(&mut self, n: usize) {
        self.data.resize(self.data.len() + n * self.width, 0);
    }

    fn finish(self, nulls: Option<NullBuffer>) -> Result<ArrayRef> {
        let array =
            FixedSizeBinaryArray::try_new(self.width as i32, Buffer::from_vec(self.data), nulls)
                .map_err(|source| Error::Batching { source })?;
        Ok(Arc::new(array))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::otap::transform::concatenate::plan::Selection;
    use arrow::array::{
        BinaryArray, Float64Array, ListArray, StringArray, UInt8Array, UInt16Array, UInt32Array,
    };
    use arrow::datatypes::Int32Type as I32;

    fn plan_all() -> InputPlan {
        InputPlan::default()
    }

    fn plan_ranges(ranges: Vec<Range<usize>>) -> InputPlan {
        InputPlan {
            selection: Selection::Ranges(ranges),
            ..Default::default()
        }
    }

    fn plan_remap(col: IdCol, remap: AnyRemap) -> InputPlan {
        let mut plan = InputPlan::default();
        plan.remaps[col as usize] = Some(remap);
        plan
    }

    fn dict_u8(keys: Vec<Option<u8>>, values: ArrayRef) -> ArrayRef {
        Arc::new(DictionaryArray::<UInt8Type>::new(
            UInt8Array::from(keys),
            values,
        ))
    }

    fn dict_u16(keys: Vec<Option<u16>>, values: ArrayRef) -> ArrayRef {
        Arc::new(DictionaryArray::<UInt16Type>::new(
            UInt16Array::from(keys),
            values,
        ))
    }

    fn write(
        target: DataType,
        cols: &[(Option<ArrayRef>, usize, InputPlan)],
        id_col: Option<IdCol>,
    ) -> ArrayRef {
        let field = Field::new("f", target, true);
        let inputs: Vec<Input<'_>> = cols
            .iter()
            .map(|(c, n, p)| Input {
                column: c.as_ref(),
                num_rows: *n,
                plan: p,
            })
            .collect();
        let rows = inputs.iter().map(Input::selected_rows).sum();
        let out = write_column(&field, &inputs, rows, id_col).unwrap();
        assert_eq!(out.len(), rows);
        assert_eq!(out.data_type(), field.data_type());
        out.to_data().validate_full().unwrap();
        out
    }

    /// Decode any (possibly dictionary) column to its logical value array.
    fn logical(array: &ArrayRef) -> ArrayRef {
        match array.data_type() {
            DataType::Dictionary(_, v) => cast(array.as_ref(), v).unwrap(),
            _ => Arc::clone(array),
        }
    }

    fn utf8(v: Vec<Option<&str>>) -> ArrayRef {
        Arc::new(StringArray::from(v))
    }

    /// Scenario: every source encoding (native, Dict(u8), Dict(u16), missing)
    /// is written into each target encoding (native, Dict(u8), Dict(u16)) for
    /// a Utf8 column.
    /// Guarantees: the logical values and nulls of the output match the
    /// concatenation of the inputs regardless of input/output encoding.
    #[test]
    fn test_encoding_matrix_utf8() {
        let native = utf8(vec![Some("a"), None, Some("b")]);
        let d8 = dict_u8(
            vec![Some(1), None, Some(0)],
            utf8(vec![Some("x"), Some("y")]),
        );
        let d16 = dict_u16(vec![Some(0), Some(0)], utf8(vec![Some("z")]));
        let expected = utf8(vec![
            Some("a"),
            None,
            Some("b"),
            Some("y"),
            None,
            Some("x"),
            Some("z"),
            Some("z"),
            None,
            None,
        ]);

        for target in [
            DataType::Utf8,
            DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
            DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
        ] {
            let out = write(
                target.clone(),
                &[
                    (Some(native.clone()), 3, plan_all()),
                    (Some(d8.clone()), 3, plan_all()),
                    (Some(d16.clone()), 2, plan_all()),
                    (None, 2, plan_all()),
                ],
                None,
            );
            assert_eq!(logical(&out).as_ref(), expected.as_ref(), "{target:?}");
        }
    }

    /// Scenario: primitive, boolean, binary and fixed size binary columns with
    /// sliced (non-zero offset) sources and nulls are concatenated.
    /// Guarantees: writers honor array offsets for both values and null
    /// buffers.
    #[test]
    fn test_sliced_sources() {
        let prim: ArrayRef = Arc::new(Float64Array::from(vec![
            Some(0.0),
            None,
            Some(2.0),
            Some(3.0),
            None,
        ]));
        let out = write(
            DataType::Float64,
            &[
                (Some(prim.slice(1, 3)), 3, plan_all()),
                (Some(prim.slice(3, 2)), 2, plan_all()),
            ],
            None,
        );
        let expected: ArrayRef = Arc::new(Float64Array::from(vec![
            None,
            Some(2.0),
            Some(3.0),
            Some(3.0),
            None,
        ]));
        assert_eq!(out.as_ref(), expected.as_ref());

        let b: ArrayRef = Arc::new(BooleanArray::from(vec![
            Some(true),
            None,
            Some(false),
            Some(true),
            Some(true),
            Some(false),
            None,
            Some(true),
            Some(false),
            Some(true),
        ]));
        let out = write(
            DataType::Boolean,
            &[
                (Some(b.slice(3, 7)), 7, plan_all()),
                (Some(b.slice(1, 3)), 3, plan_all()),
            ],
            None,
        );
        let mut expected: Vec<Option<bool>> = b.as_boolean().slice(3, 7).iter().collect();
        expected.extend(b.as_boolean().slice(1, 3).iter());
        let expected: ArrayRef = Arc::new(BooleanArray::from(expected));
        assert_eq!(out.as_ref(), expected.as_ref());

        let bin: ArrayRef = Arc::new(BinaryArray::from(vec![
            Some(&b"aa"[..]),
            None,
            Some(b"ccc"),
            Some(b""),
            Some(b"e"),
        ]));
        let out = write(
            DataType::Binary,
            &[
                (Some(bin.slice(2, 3)), 3, plan_all()),
                (Some(bin.slice(0, 2)), 2, plan_all()),
            ],
            None,
        );
        let expected: ArrayRef = Arc::new(BinaryArray::from(vec![
            Some(&b"ccc"[..]),
            Some(b""),
            Some(b"e"),
            Some(b"aa"),
            None,
        ]));
        assert_eq!(out.as_ref(), expected.as_ref());

        let fsb: ArrayRef = Arc::new(
            FixedSizeBinaryArray::try_from_sparse_iter_with_size(
                vec![Some([1u8, 1]), None, Some([3, 3]), Some([4, 4])].into_iter(),
                2,
            )
            .unwrap(),
        );
        let out = write(
            DataType::FixedSizeBinary(2),
            &[
                (Some(fsb.slice(1, 3)), 3, plan_all()),
                (None, 1, plan_all()),
            ],
            None,
        );
        let expected: ArrayRef = Arc::new(
            FixedSizeBinaryArray::try_from_sparse_iter_with_size(
                vec![None, Some([3u8, 3]), Some([4, 4]), None].into_iter(),
                2,
            )
            .unwrap(),
        );
        assert_eq!(out.as_ref(), expected.as_ref());
    }

    /// Scenario: inputs carry `Selection::Ranges` for native, dictionary, and
    /// struct columns.
    /// Guarantees: only rows inside the selected ranges are written, in order,
    /// and nulls follow their rows.
    #[test]
    fn test_selection_ranges() {
        let native: ArrayRef = Arc::new(UInt8Array::from(vec![
            Some(0),
            Some(1),
            None,
            Some(3),
            Some(4),
        ]));
        let out = write(
            DataType::UInt8,
            &[(Some(native.clone()), 5, plan_ranges(vec![0..1, 2..4]))],
            None,
        );
        let expected: ArrayRef = Arc::new(UInt8Array::from(vec![Some(0), None, Some(3)]));
        assert_eq!(out.as_ref(), expected.as_ref());

        let dict = dict_u8(
            vec![Some(0), Some(1), None, Some(0)],
            utf8(vec![Some("a"), Some("b")]),
        );
        for target in [
            DataType::Utf8,
            DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
        ] {
            let out = write(
                target,
                &[(
                    Some(dict.clone()),
                    4,
                    plan_ranges(std::iter::once(1..3).collect()),
                )],
                None,
            );
            assert_eq!(logical(&out).as_ref(), utf8(vec![Some("b"), None]).as_ref());
        }

        let child = Field::new("c", DataType::UInt8, true);
        let s: ArrayRef = Arc::new(StructArray::new(
            vec![child.clone()].into(),
            vec![native],
            Some(NullBuffer::from(vec![true, false, true, true, true])),
        ));
        let target = DataType::Struct(vec![child].into());
        let out = write(target, &[(Some(s), 5, plan_ranges(vec![1..2, 3..5]))], None);
        let out = out.as_struct();
        assert!(out.is_null(0));
        assert!(out.is_valid(1));
        let expected: ArrayRef = Arc::new(UInt8Array::from(vec![Some(1), Some(3), Some(4)]));
        assert_eq!(out.column(0).as_ref(), expected.as_ref());
    }

    /// Scenario: a struct target has children that are absent from some
    /// inputs, and one input lacks the struct entirely.
    /// Guarantees: missing children and missing structs are written as nulls
    /// of the correct length without affecting present data.
    #[test]
    fn test_missing_struct_children() {
        let a = Field::new("a", DataType::Int32, true);
        let b = Field::new("b", DataType::Utf8, true);
        let s1: ArrayRef = Arc::new(StructArray::new(
            vec![a.clone()].into(),
            vec![Arc::new(PrimitiveArray::<I32>::from(vec![1, 2]))],
            None,
        ));
        let s2: ArrayRef = Arc::new(StructArray::new(
            vec![b.clone()].into(),
            vec![utf8(vec![Some("x")])],
            None,
        ));
        let target = DataType::Struct(vec![a, b].into());
        let out = write(
            target,
            &[
                (Some(s1), 2, plan_all()),
                (Some(s2), 1, plan_all()),
                (None, 2, plan_all()),
            ],
            None,
        );
        let out = out.as_struct();
        assert_eq!(out.null_count(), 2);
        let expected_a: ArrayRef = Arc::new(PrimitiveArray::<I32>::from(vec![
            Some(1),
            Some(2),
            None,
            None,
            None,
        ]));
        assert_eq!(out.column(0).as_ref(), expected_a.as_ref());
        assert_eq!(
            out.column(1).as_ref(),
            utf8(vec![None, None, Some("x"), None, None]).as_ref()
        );
    }

    /// Scenario: inputs without nulls are written; then a null appears only in
    /// the last input.
    /// Guarantees: no null buffer is allocated when no input has nulls, and
    /// earlier rows are backfilled as valid when the first null appears late.
    #[test]
    fn test_lazy_nulls() {
        let a: ArrayRef = Arc::new(UInt32Array::from(vec![1, 2]));
        let out = write(
            DataType::UInt32,
            &[
                (Some(a.clone()), 2, plan_all()),
                (Some(a.clone()), 2, plan_all()),
            ],
            None,
        );
        assert!(out.nulls().is_none());

        let b: ArrayRef = Arc::new(UInt32Array::from(vec![None, Some(9)]));
        let out = write(
            DataType::UInt32,
            &[(Some(a), 2, plan_all()), (Some(b), 2, plan_all())],
            None,
        );
        let expected: ArrayRef = Arc::new(UInt32Array::from(vec![Some(1), Some(2), None, Some(9)]));
        assert_eq!(out.as_ref(), expected.as_ref());
    }

    /// Scenario: ID remaps (Offset including wraparound on a null slot, and
    /// Replace) are applied to a native u16 id column and a dictionary u32
    /// parent_id column.
    /// Guarantees: remaps apply to values (dictionary values for dictionary
    /// columns) in both native and dictionary output, null slots never panic,
    /// and nulls are preserved.
    #[test]
    fn test_id_remaps() {
        let ids: ArrayRef = Arc::new(UInt16Array::from(vec![Some(5), None, Some(6)]));
        let out = write(
            DataType::UInt16,
            &[
                (
                    Some(ids.clone()),
                    3,
                    plan_remap(
                        IdCol::Id,
                        AnyRemap::U16(IdRemap::Offset(0u16.wrapping_sub(5))),
                    ),
                ),
                (
                    Some(ids),
                    3,
                    plan_remap(
                        IdCol::Id,
                        AnyRemap::U16(IdRemap::Replace(vec![10u16, 0, 11].into())),
                    ),
                ),
            ],
            Some(IdCol::Id),
        );
        let expected: ArrayRef = Arc::new(UInt16Array::from(vec![
            Some(0),
            None,
            Some(1),
            Some(10),
            None,
            Some(11),
        ]));
        assert_eq!(out.as_ref(), expected.as_ref());

        let values: ArrayRef = Arc::new(UInt32Array::from(vec![100, 200]));
        let pid = dict_u8(vec![Some(1), Some(0), Some(1)], values);
        let plan = plan_remap(IdCol::ParentId, AnyRemap::U32(IdRemap::Offset(7)));
        let expected: ArrayRef = Arc::new(UInt32Array::from(vec![207, 107, 207]));
        for target in [
            DataType::UInt32,
            DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::UInt32)),
        ] {
            let out = write(
                target,
                &[(Some(pid.clone()), 3, plan.clone())],
                Some(IdCol::ParentId),
            );
            assert_eq!(logical(&out).as_ref(), expected.as_ref());
        }
    }

    /// Scenario: a remap is present in the plan for `id` but the column being
    /// written is a non-ID column of the same type.
    /// Guarantees: remaps are only applied to the ID column they target.
    #[test]
    fn test_remap_not_applied_to_other_columns() {
        let a: ArrayRef = Arc::new(UInt32Array::from(vec![1, 2]));
        let out = write(
            DataType::UInt32,
            &[(
                Some(a.clone()),
                2,
                plan_remap(IdCol::Id, AnyRemap::U32(IdRemap::Offset(10))),
            )],
            None,
        );
        assert_eq!(out.as_ref(), a.as_ref());
    }

    /// Scenario: List columns are concatenated via the generic fallback with
    /// a selection and a missing input.
    /// Guarantees: the fallback honors selections and null-pads missing
    /// inputs.
    #[test]
    fn test_list_fallback() {
        let list: ArrayRef = Arc::new(ListArray::from_iter_primitive::<Float64Type, _, _>(vec![
            Some(vec![Some(1.0)]),
            Some(vec![Some(2.0), Some(3.0)]),
            None,
        ]));
        let target = list.data_type().clone();
        let out = write(
            target,
            &[
                (Some(list), 3, plan_ranges(std::iter::once(1..3).collect())),
                (None, 1, plan_all()),
            ],
            None,
        );
        let expected: ArrayRef =
            Arc::new(ListArray::from_iter_primitive::<Float64Type, _, _>(vec![
                Some(vec![Some(2.0), Some(3.0)]),
                None,
                None,
            ]));
        assert_eq!(out.as_ref(), expected.as_ref());
    }

    /// Scenario: dictionary inputs whose keys contain sequential runs,
    /// descending and repeated keys, null keys with out-of-range values, and
    /// a sliced key array are gathered into native Utf8 and FixedSizeBinary
    /// outputs, with and without a row selection.
    /// Guarantees: run-coalesced gathering produces exactly the logical
    /// values of the inputs, in row order, with nulls preserved.
    #[test]
    fn test_dict_gather_runs() {
        let values: Vec<String> = (0..12).map(|i| format!("v{i}{}", "x".repeat(i))).collect();
        let str_values: ArrayRef = Arc::new(StringArray::from(values.clone()));
        let fsb_values: ArrayRef = Arc::new(
            FixedSizeBinaryArray::try_from_iter((0u8..12).map(|i| [i, i.wrapping_mul(7)])).unwrap(),
        );

        // Runs [2,3,4], [9], descending [8,7], repeats [5,5], a null key that
        // holds an out-of-range value, then a run to the end [10,11].
        let key_data: Vec<Option<u8>> = vec![
            Some(2),
            Some(3),
            Some(4),
            Some(9),
            Some(8),
            Some(7),
            Some(5),
            Some(5),
            None,
            Some(10),
            Some(11),
        ];
        let mut keys = UInt8Array::from(key_data.clone());
        // Give the null slot an out-of-range key value.
        let (dt, mut raw, nulls) = keys.into_parts();
        let mut v = raw.to_vec();
        v[8] = 200;
        raw = v.into();
        keys = UInt8Array::new(raw, nulls).with_data_type(dt);

        for (values, target) in [
            (str_values, DataType::Utf8),
            (fsb_values, DataType::FixedSizeBinary(2)),
        ] {
            let dict: ArrayRef = Arc::new(DictionaryArray::<UInt8Type>::new(
                keys.clone(),
                values.clone(),
            ));
            let expected_full = cast(dict.as_ref(), &target).unwrap();

            // Whole input.
            let out = write(
                target.clone(),
                &[(Some(dict.clone()), 11, plan_all())],
                None,
            );
            assert_eq!(out.as_ref(), expected_full.as_ref(), "{target:?} full");

            // Sliced keys (non-zero key offset) plus a selection.
            let sliced = dict.slice(1, 9);
            let out = write(
                target.clone(),
                &[(Some(sliced.clone()), 9, plan_ranges(vec![0..3, 5..9]))],
                None,
            );
            let exp = cast(sliced.as_ref(), &target).unwrap();
            let exp = arrow::compute::concat(&[&exp.slice(0, 3), &exp.slice(5, 4)]).unwrap();
            assert_eq!(out.as_ref(), exp.as_ref(), "{target:?} sliced");
        }
    }

    /// Scenario: a dictionary source has a null in its values array that is
    /// referenced by a valid key, gathered into a native output.
    /// Guarantees: the row is null in the output.
    #[test]
    fn test_dict_null_value_gather() {
        let d = dict_u8(vec![Some(0), Some(1)], utf8(vec![None, Some("a")]));
        let out = write(DataType::Utf8, &[(Some(d), 2, plan_all())], None);
        assert_eq!(out.as_ref(), utf8(vec![None, Some("a")]).as_ref());
    }
}
