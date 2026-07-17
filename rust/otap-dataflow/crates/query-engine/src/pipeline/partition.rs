// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Utilities for splitting based on the result of some expression.
//!
//! For example, we may wish to split an OTAP batch on some attribute/resource attribute,
//! or some computed value, like
//! `sha256(concat(resource.attributes["k8s.namespace.name"], resource.attributes["service.name"]))`
//!
//! The main public entrypoint for this module is the [`Partitioner`] type.

use std::cmp::Ordering;
use std::ops::Range;

use arrow::array::{
    AnyDictionaryArray, Array, ArrayRef, ArrowNativeTypeOp, AsArray, BooleanArray, DynComparator,
    StructArray, UInt8Array, make_comparator,
};
use arrow::buffer::{BooleanBuffer, MutableBuffer, NullBuffer};
use arrow::compute::SortOptions;
use arrow::datatypes::DataType;
use arrow::util::bit_util;
use data_engine_expressions::{PipelineFunction, ScalarExpression};
use datafusion::execution::context::SessionContext;
use datafusion::logical_expr::ColumnarValue;
use datafusion::scalar::ScalarValue;
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::otap::filter::{IdBitmapPool, filter_otap_batch};
use otap_df_pdata::otlp::attributes::AttributeValueType;
use otap_df_pdata::schema::consts;

use crate::error::{Error, Result};
use crate::pipeline::Pipeline;
use crate::pipeline::expr::ScopedExpr;
use crate::pipeline::expr::eval::align_value_to_root;
use crate::pipeline::expr::planner::ExprPlanner;
use crate::pipeline::project::anyval::is_any_value_data_type;

/// Produces partitioned record batches by the results of some evaluated expression.
///
/// Usage:
///
/// ```rust,ignore
/// let (scalar_expr, functions) = OplParser::parse_expr_with_options(
///     "attributes[\"x\"]",
///     default_parser_options()
/// )?;
/// let mut partitioner = Partitioner::try_new(scalar_expr, functions)?;
/// let partitions = partitioner.partition(otap_batch)?.into_iter().collect::<Vec<_>>();
/// ```
///
/// The intention is that this type can be reused for each OTAP batch that must be partitioned in
/// order that any internal heap allocations can be reused across batches.
pub struct Partitioner {
    /// the expression that will be evaluated to partition incoming OTAP batches
    expr: ScopedExpr,

    /// datafusion [`SessionContext`] - used to evaluate expression
    session_ctx: SessionContext,

    /// coalescer for ranges of partitioned rows that have the same partition value.
    range_coalescer: PartitionRangeCoalescer,

    /// ID bitmap pool - used when taking rows that belong to the same partition
    id_bitmap_pool: IdBitmapPool,

    /// Pool of reusable buffers for [`ArrayComparator`] group id mappings.
    group_id_pool: GroupIdPool,

    /// Reusable buffer of partition results
    partitions: Vec<Partition>,
}

impl Partitioner {
    /// Create a new instance of [`Partitioner`] that will partition OTAP batches on the evaluation
    /// result of the passed expression.
    ///
    /// If the expression references functions, they must be defined in the vec of passed
    /// functions. [`ScalarExpression`]s produced by parsers implementations will typically create
    /// these function definitions while parsing, so this constructor is intended to be used from
    /// the results of parsing some expression.
    pub fn try_new(
        scalar_expr: ScalarExpression,
        functions: Vec<PipelineFunction>,
    ) -> Result<Self> {
        let expr_planner = ExprPlanner::new();
        let planned_expr = expr_planner.plan_scalar(&scalar_expr, &functions)?;

        Ok(Self {
            expr: planned_expr.expr,
            session_ctx: Pipeline::create_session_context(),
            id_bitmap_pool: IdBitmapPool::new(),
            range_coalescer: PartitionRangeCoalescer::new(),
            group_id_pool: GroupIdPool::new(),
            partitions: Vec::new(),
        })
    }

    /// Evaluates the partition expression against the batch and returns an iterator of partitions.
    pub fn partition(
        &mut self,
        otap_batch: OtapArrowRecords,
    ) -> Result<impl ExactSizeIterator<Item = Partition> + '_> {
        // reset state
        self.partitions.clear();
        self.range_coalescer.clear();

        partition(
            otap_batch,
            &self.session_ctx,
            &mut self.expr,
            &mut self.partitions,
            &mut self.range_coalescer,
            &mut self.id_bitmap_pool,
            &mut self.group_id_pool,
        )?;

        // return iterator of results
        Ok(self.partitions.drain(..))
    }
}

/// Partitioned OTAP batch
///
/// All records in the batch have the same value computed from the partition expression.
pub struct Partition {
    /// value of partition
    pub value: PartitionValue,

    /// telemetry data in partition
    pub batch: OtapArrowRecords,
}

impl Partition {
    fn new(value: PartitionValue, batch: OtapArrowRecords) -> Self {
        Self { value, batch }
    }
}

/// Computed value of partition for partitioned OTAP Data.
#[derive(Debug, PartialEq)]
pub enum PartitionValue {
    /// value of type string
    String(String),

    /// value of type signed int
    Int(i64),

    /// value of type unsigned int
    UInt(u64),

    /// value of type float
    Float(f64),

    /// value of type bool
    Boolean(bool),

    /// value of binary
    Binary(Vec<u8>),

    /// partition value is null
    Null,
}

impl PartitionValue {
    /// Construct a [`PartitionValue`] from the value in the [`ScalarExpression`].
    ///
    /// May return `Err` for types that are not yet supported.
    fn try_from_scalar(scalar: ScalarValue) -> Result<Self> {
        Ok(match scalar {
            ScalarValue::Boolean(Some(b)) => Self::Boolean(b),
            ScalarValue::Binary(Some(b))
            | ScalarValue::BinaryView(Some(b))
            | ScalarValue::FixedSizeBinary(_, Some(b))
            | ScalarValue::LargeBinary(Some(b)) => Self::Binary(b),

            ScalarValue::Date32(Some(t))
            | ScalarValue::Time32Millisecond(Some(t))
            | ScalarValue::Time32Second(Some(t)) => Self::Int(t as i64),
            ScalarValue::Date64(Some(t))
            | ScalarValue::DurationSecond(Some(t))
            | ScalarValue::DurationMillisecond(Some(t))
            | ScalarValue::DurationMicrosecond(Some(t))
            | ScalarValue::DurationNanosecond(Some(t))
            | ScalarValue::Time64Microsecond(Some(t))
            | ScalarValue::Time64Nanosecond(Some(t))
            | ScalarValue::TimestampSecond(Some(t), _)
            | ScalarValue::TimestampMillisecond(Some(t), _)
            | ScalarValue::TimestampMicrosecond(Some(t), _)
            | ScalarValue::TimestampNanosecond(Some(t), _) => Self::Int(t),

            ScalarValue::Float16(Some(f)) => Self::Float(f.to_f64()),
            ScalarValue::Float32(Some(f)) => Self::Float(f as f64),
            ScalarValue::Float64(Some(f)) => Self::Float(f),
            ScalarValue::Int8(Some(i)) => Self::Int(i as i64),
            ScalarValue::Int16(Some(i)) => Self::Int(i as i64),
            ScalarValue::Int32(Some(i)) => Self::Int(i as i64),
            ScalarValue::Int64(Some(i)) => Self::Int(i),

            ScalarValue::UInt8(Some(i)) => Self::UInt(i as u64),
            ScalarValue::UInt16(Some(i)) => Self::UInt(i as u64),
            ScalarValue::UInt32(Some(i)) => Self::UInt(i as u64),
            ScalarValue::UInt64(Some(i)) => Self::UInt(i),

            ScalarValue::Utf8(Some(s))
            | ScalarValue::Utf8View(Some(s))
            | ScalarValue::LargeUtf8(Some(s)) => Self::String(s),

            ScalarValue::Binary(None)
            | ScalarValue::Boolean(None)
            | ScalarValue::BinaryView(None)
            | ScalarValue::Date32(None)
            | ScalarValue::Time32Millisecond(None)
            | ScalarValue::Time32Second(None)
            | ScalarValue::Date64(None)
            | ScalarValue::DurationSecond(None)
            | ScalarValue::DurationMillisecond(None)
            | ScalarValue::DurationMicrosecond(None)
            | ScalarValue::DurationNanosecond(None)
            | ScalarValue::Time64Microsecond(None)
            | ScalarValue::Time64Nanosecond(None)
            | ScalarValue::TimestampSecond(None, _)
            | ScalarValue::TimestampMillisecond(None, _)
            | ScalarValue::TimestampMicrosecond(None, _)
            | ScalarValue::TimestampNanosecond(None, _)
            | ScalarValue::Float16(None)
            | ScalarValue::Float32(None)
            | ScalarValue::Float64(None)
            | ScalarValue::FixedSizeBinary(_, None)
            | ScalarValue::Int8(None)
            | ScalarValue::Int16(None)
            | ScalarValue::Int32(None)
            | ScalarValue::Int64(None)
            | ScalarValue::LargeBinary(None)
            | ScalarValue::LargeUtf8(None)
            | ScalarValue::Null
            | ScalarValue::UInt8(None)
            | ScalarValue::UInt16(None)
            | ScalarValue::UInt32(None)
            | ScalarValue::UInt64(None)
            | ScalarValue::Utf8(None)
            | ScalarValue::Utf8View(None) => Self::Null,

            ScalarValue::Struct(s) => {
                if is_any_value_data_type(&DataType::Struct(s.fields().clone())) {
                    return Self::try_from_anyvalue_struct_arr(s.as_ref(), 0);
                }

                return Err(Error::ExecutionError {
                    cause: format!(
                        "partition value cannot be computed from unsupported datatype {:?}",
                        s.data_type(),
                    ),
                });
            }

            ScalarValue::Dictionary(_, s) | ScalarValue::RunEndEncoded(_, _, s) => {
                Self::try_from_scalar(s.as_ref().clone())?
            }

            ScalarValue::Decimal32(_, _, _)
            | ScalarValue::Decimal64(_, _, _)
            | ScalarValue::Decimal128(_, _, _)
            | ScalarValue::Decimal256(_, _, _)
            | ScalarValue::IntervalDayTime(_)
            | ScalarValue::IntervalMonthDayNano(_)
            | ScalarValue::IntervalYearMonth(_)
            | ScalarValue::FixedSizeList(_)
            | ScalarValue::List(_)
            | ScalarValue::LargeList(_)
            | ScalarValue::Map(_)
            | ScalarValue::Union(_, _, _) => {
                return Err(Error::ExecutionError {
                    cause: format!(
                        "partition value cannot be computed from unsupported datatype {:?}",
                        scalar.data_type()
                    ),
                });
            }
        })
    }

    fn try_from_array_value(arr: &dyn Array, index: usize) -> Result<Self> {
        if is_any_value_data_type(arr.data_type()) {
            // safety: we've checked the type is struct array
            let anyvalue_arr = arr
                .as_any()
                .downcast_ref::<StructArray>()
                .expect("is struct array");
            return Self::try_from_anyvalue_struct_arr(anyvalue_arr, index);
        }

        let scalar_at_index = ScalarValue::try_from_array(arr, index)?;
        Self::try_from_scalar(scalar_at_index)
    }

    /// Construct a partition value from the `AnyValue` by examining the struct representation
    /// at some index.
    ///
    /// Will return an error if the struct array is not the proper representation used by OTAP
    /// to represent `AnyValue`s.
    fn try_from_anyvalue_struct_arr(arr: &StructArray, index: usize) -> Result<Self> {
        if arr.is_null(index) {
            return Ok(Self::Null);
        }

        let Some(type_col) = arr.column_by_name(consts::ATTRIBUTE_TYPE) else {
            return Err(otap_df_pdata::error::Error::ColumnNotFound {
                name: consts::ATTRIBUTE_TYPE.into(),
            }
            .into());
        };

        let Some(type_col) = type_col.as_any().downcast_ref::<UInt8Array>() else {
            return Err(otap_df_pdata::error::Error::ColumnDataTypeMismatch {
                name: consts::ATTRIBUTE_TYPE.into(),
                actual: type_col.data_type().clone(),
                expect: DataType::UInt8,
            }
            .into());
        };

        let attr_type =
            AttributeValueType::try_from(type_col.values()[index]).map_err(|error| {
                otap_df_pdata::error::Error::UnrecognizedAttributeValueType { error }
            })?;

        let values_col = match attr_type {
            AttributeValueType::Bool => arr.column_by_name(consts::ATTRIBUTE_BOOL),
            AttributeValueType::Bytes => arr.column_by_name(consts::ATTRIBUTE_BYTES),
            AttributeValueType::Double => arr.column_by_name(consts::ATTRIBUTE_DOUBLE),
            AttributeValueType::Int => arr.column_by_name(consts::ATTRIBUTE_INT),
            AttributeValueType::Map | AttributeValueType::Slice => {
                arr.column_by_name(consts::ATTRIBUTE_SER)
            }
            AttributeValueType::Str => arr.column_by_name(consts::ATTRIBUTE_STR),
            AttributeValueType::Empty => return Ok(Self::Null),
        };

        match values_col {
            Some(col) => Self::try_from_array_value(col.as_ref(), index),
            None => Ok(Self::Null),
        }
    }
}

/// Partition OTAP batch by the value of the evaluated expression.
///
/// The full partitioning process can be roughly thought of in three phases:
/// 1. determining partition boundaries - e.g. which rows are distinct from their neighbour
/// 2. grouping partition boundaries that have the same value
/// 3. taking all the rows in the same partition.
///
/// However, there may be cases where we return early before performing all these steps
/// (e.g. when we can determine quickly that there will only be one or zero partitions).
///
fn partition(
    otap_batch: OtapArrowRecords,
    session_ctx: &SessionContext,
    expr: &mut ScopedExpr,
    result_partitions: &mut Vec<Partition>,
    range_coalescer: &mut PartitionRangeCoalescer,
    id_bitmap_pool: &mut IdBitmapPool,
    group_id_pool: &mut GroupIdPool,
) -> Result<()> {
    // nothing to evaluate
    if otap_batch.num_items() == 0 {
        return Ok(());
    }

    let eval_result = match expr.execute_as_value(&otap_batch, session_ctx)? {
        Some(result) => {
            // align value to root so we can calculate partitions for the root record batch
            align_value_to_root(result, &otap_batch)?
        }
        None => {
            // the result evaluated to `null` for all rows, which means there is only one
            // partition, so simply return the original batch
            result_partitions.push(Partition::new(PartitionValue::Null, otap_batch));
            return Ok(());
        }
    };

    match eval_result.values {
        ColumnarValue::Array(array) => {
            if is_any_value_data_type(array.data_type()) {
                partition_any_value_struct_array(
                    array.as_struct(),
                    otap_batch,
                    result_partitions,
                    range_coalescer,
                    id_bitmap_pool,
                    group_id_pool,
                )
            } else {
                partition_simple_array(
                    array,
                    otap_batch,
                    result_partitions,
                    range_coalescer,
                    id_bitmap_pool,
                    group_id_pool,
                )
            }
        }
        ColumnarValue::Scalar(scalar) => {
            // the result evaluated to a single value for all rows, meaning there is only one
            // partition, so simply return the original batch
            result_partitions.push(Partition::new(
                PartitionValue::try_from_scalar(scalar)?,
                otap_batch,
            ));

            Ok(())
        }
    }
}

/// Populate the `result` vec with partitions of the OTAP batch based on which rows in the passed
/// array have equivalent values.
fn partition_simple_array(
    array: ArrayRef,
    otap_batch: OtapArrowRecords,
    result: &mut Vec<Partition>,
    range_coalescer: &mut PartitionRangeCoalescer,
    id_bitmap_pool: &mut IdBitmapPool,
    group_id_pool: &mut GroupIdPool,
) -> Result<()> {
    let cmp = ArrayComparator::try_new(&array, group_id_pool)?;
    let boundaries: BooleanBuffer = (0..array.len() - 1)
        .map(|i| !cmp.cmp(i, i + 1).is_eq())
        .collect();

    let result = partition_at_boundaries(
        array.as_ref(),
        boundaries,
        otap_batch,
        result,
        range_coalescer,
        id_bitmap_pool,
        &|i1, i2| Ok(cmp.cmp(i1, i2)),
    );

    cmp.return_to_pool(group_id_pool);
    result
}

/// Populate the `result` vec with partitions of the OTAP batch based on which rows in the passed
/// struct array (which represents a list of `AnyValue`) have equivalent values.
fn partition_any_value_struct_array(
    array: &StructArray,
    otap_batch: OtapArrowRecords,
    result: &mut Vec<Partition>,
    range_coalescer: &mut PartitionRangeCoalescer,
    id_bitmap_pool: &mut IdBitmapPool,
    group_id_pool: &mut GroupIdPool,
) -> Result<()> {
    let comparator = AnyValueStructComparator::try_new(array, group_id_pool)?;
    let boundaries: BooleanBuffer = (0..array.len() - 1)
        .map(|i| comparator.cmp(i, i + 1).map(|ord| !ord.is_eq()))
        .collect::<Result<Vec<bool>>>()?
        .into();

    let result = partition_at_boundaries(
        array,
        boundaries,
        otap_batch,
        result,
        range_coalescer,
        id_bitmap_pool,
        &|i1, i2| comparator.cmp(i1, i2),
    );

    comparator.return_to_pool(group_id_pool);
    result
}

/// Populate the `result` vec with partitions from `boundaries`, which represents indices where
/// the array value is distinct from its next neighbour. The range_coalescer will be used to group
/// boundary-delineated partitions into the same result partition.
///
/// For example, if passed:
/// ```text,ignore
/// boundaries: [false, true, false, true, true]
/// array: ["a", "a", "b", "b", "a"]
/// ```
/// The result would be two partitions with :
/// ```text
/// // partition 1:
/// rows: [0, 1, 4]
/// value: "a"
///
/// // partition 2:
/// rows: [2, 3]
/// value: "b"
/// ```
fn partition_at_boundaries(
    array: &dyn Array,
    boundaries: BooleanBuffer,
    otap_batch: OtapArrowRecords,
    result: &mut Vec<Partition>,
    range_coalescer: &mut PartitionRangeCoalescer,
    id_bitmap_pool: &mut IdBitmapPool,
    cmp: &dyn Fn(usize, usize) -> Result<Ordering>,
) -> Result<()> {
    let num_rows = boundaries.len() + 1;
    for group in range_coalescer.coalesce_groups(num_rows, boundaries, &cmp)? {
        let selection_vec = BooleanArray::from(BooleanBuffer::new(
            group.selection_vec_builder.into(),
            0,
            num_rows,
        ));
        let filtered = filter_otap_batch(&selection_vec, &otap_batch, id_bitmap_pool)?;
        let partition_value =
            PartitionValue::try_from_array_value(array, group.representative_row)?;
        result.push(Partition::new(partition_value, filtered));
    }

    Ok(())
}

/// A pool of reusable `Vec<u32>` buffers used by [`ArrayComparator`] for pre-computed
/// row-to-group-id mappings. This avoids re-allocating these buffers on every batch.
struct GroupIdPool {
    pool: Vec<Vec<u32>>,
}

impl GroupIdPool {
    fn new() -> Self {
        Self { pool: Vec::new() }
    }

    /// Takes a buffer from the pool (reusing an existing allocation) or creates a new one.
    fn take(&mut self) -> Vec<u32> {
        self.pool.pop().unwrap_or_default()
    }

    /// Returns a buffer to the pool so its allocation can be reused in future batches.
    fn give_back(&mut self, mut buf: Vec<u32>) {
        buf.clear();
        self.pool.push(buf);
    }
}

/// A comparator for arrow arrays that optimizes comparisons on dictionary-encoded arrays.
///
/// For non-dictionary arrays, this delegates to arrow's [`make_comparator`] ([`DynComparator`]).
///
/// For dictionary-encoded arrays (common in OTAP, especially `Dictionary(UInt8|UInt16, Utf8)`),
/// this pre-computes a mapping from each row index to a "group id" where rows with equal
/// dictionary values share the same group id. Comparisons then reduce to integer comparisons
/// on the group ids (O(1)) instead of comparing the underlying values (e.g. string comparisons
/// which are O(n) in string length).
///
/// The group id mapping also correctly handles dictionaries with duplicate values mapped to
/// different keys.
enum ArrayComparator {
    /// Non-dictionary array: delegates to arrow's [`DynComparator`].
    Standard(DynComparator),

    /// Dictionary-encoded array: pre-computed row index -> group id mapping.
    ///
    /// Two rows are equal iff they have the same group id. Null rows are assigned
    /// `u32::MAX` as a sentinel group id so that all nulls compare as equal.
    DictGrouped(Vec<u32>),
}

/// Sentinel group id assigned to null rows in [`ArrayComparator::DictGrouped`].
const NULL_GROUP_ID: u32 = u32::MAX;

impl ArrayComparator {
    /// Creates a new comparator for the given array.
    ///
    /// If the array is dictionary-encoded, pre-computes a group id mapping using a buffer
    /// from `pool`. For non-dictionary arrays, falls back to [`make_comparator`].
    fn try_new(array: &ArrayRef, pool: &mut GroupIdPool) -> Result<Self> {
        if let Some(dict) = array.as_any_dictionary_opt() {
            Self::try_new_dict_grouped(dict, array.nulls(), pool)
        } else {
            let cmp = make_comparator(array, array, SortOptions::default())?;
            Ok(Self::Standard(cmp))
        }
    }

    /// Build a [`DictGrouped`](ArrayComparator::DictGrouped) comparator for a dictionary array.
    ///
    /// Steps:
    /// 1. Group the dictionary *values* by equality, producing a `value_index -> group_id` map.
    /// 2. Map each row's dictionary key to its corresponding group id, handling nulls.
    fn try_new_dict_grouped(
        dict: &dyn AnyDictionaryArray,
        nulls: Option<&NullBuffer>,
        pool: &mut GroupIdPool,
    ) -> Result<Self> {
        let values = dict.values();
        let num_values = values.len();

        // Phase 1: group duplicate dictionary values.
        // `value_to_group[v]` is the group id for dictionary value index `v`.
        let value_to_group = if num_values == 0 {
            Vec::new()
        } else {
            let values_cmp = make_comparator(&values, &values, SortOptions::default())?;
            let mut value_to_group = vec![0u32; num_values];
            // representative value index for each group
            let mut group_representatives: Vec<usize> = Vec::new();

            for (v, group_id_slot) in value_to_group.iter_mut().enumerate() {
                let mut found_group = None;
                for (group_id, &rep) in group_representatives.iter().enumerate() {
                    if values_cmp(v, rep).is_eq() {
                        found_group = Some(group_id as u32);
                        break;
                    }
                }
                match found_group {
                    Some(gid) => *group_id_slot = gid,
                    None => {
                        let gid = group_representatives.len() as u32;
                        *group_id_slot = gid;
                        group_representatives.push(v);
                    }
                }
            }
            value_to_group
        };

        // Phase 2: map each row to its group id using the dictionary's normalized keys.
        let normalized_keys = dict.normalized_keys();
        let num_rows = normalized_keys.len();

        let mut row_group_ids = pool.take();
        row_group_ids.resize(num_rows, 0);

        for (row, &key) in normalized_keys.iter().enumerate() {
            let is_null = nulls.is_some_and(|n| n.is_null(row));
            row_group_ids[row] = if is_null {
                NULL_GROUP_ID
            } else {
                value_to_group[key]
            };
        }

        Ok(Self::DictGrouped(row_group_ids))
    }

    /// Compare the values at two row indices for equality/ordering.
    ///
    /// For partitioning, only equality matters. For [`DictGrouped`](ArrayComparator::DictGrouped),
    /// the ordering between non-equal values is stable but not necessarily meaningful
    /// (it reflects group id order, not lexicographic order of the underlying values).
    fn cmp(&self, i: usize, j: usize) -> Ordering {
        match self {
            Self::Standard(cmp) => cmp(i, j),
            Self::DictGrouped(group_ids) => group_ids[i].cmp(&group_ids[j]),
        }
    }

    /// Consumes this comparator and returns the group id buffer (if any) to the pool.
    fn return_to_pool(self, pool: &mut GroupIdPool) {
        if let Self::DictGrouped(buf) = self {
            pool.give_back(buf);
        }
    }
}

/// This type is responsible for grouping boundary-delineated partitions that have the same
/// actual value for the partition expression.
///
/// Partitioning proceeds by determining the boundaries at which each row where the value of the
/// partition key is not equal to its next neighbour. The ranges between these boundaries are then
/// grouped together where ranges have equivalent values, and a selection vector is created
///
/// For example, if we had `boundaries: [false, true, false, true, true]` and partition key values
/// of ["a", "a", "b", "b", "a"] this represents three ranges: `[(0, 2), (2, 3), (3, 4)]`, where
/// the first and third ranges have equivalent values. This type would try to produce the following
/// groups:
/// ```text,ignore
/// // group 1:
/// representative_row: 0
/// selection_vec: [true, true, false, false, true]
///
/// // group 2
/// representative_row: 2
/// selection_vec: [false, false, true, true, false]
/// ```
///
/// The intention with this type is that it can be reused for multiple coalescing operations in
/// order that the internal heap allocation can be reused for many OTAP batches.
struct PartitionRangeCoalescer {
    groups: Vec<CoalescingGroup>,
}

/// Intermediate result used when coalescing partitions into a single partition
struct CoalescingGroup {
    /// index of row representing the value of the group
    representative_row: usize,

    /// mutable selection vec builder for selecting rows belonging to the partition
    selection_vec_builder: MutableBuffer,
}

impl PartitionRangeCoalescer {
    fn new() -> Self {
        Self { groups: Vec::new() }
    }

    fn coalesce_groups(
        &mut self,
        source_len: usize,
        partition_boundaries: BooleanBuffer,
        cmp: &dyn Fn(usize, usize) -> Result<Ordering>,
    ) -> Result<impl IntoIterator<Item = CoalescingGroup>> {
        coalesce_groups(source_len, partition_boundaries, cmp, &mut self.groups)?;

        // return iterator of results
        Ok(self.groups.drain(..))
    }

    fn clear(&mut self) {
        self.groups.clear();
    }
}

/// coalesce partitions delineated by `boundaries` into groups of rows that have equivalent values.
///
/// `boundaries` is a boolean buffer where the rows represent that some index is distinct from its
/// next neighbour. `cmp` is a comparison function used to compare two rows.
fn coalesce_groups(
    source_len: usize,
    partition_boundaries: BooleanBuffer,
    cmp: &dyn Fn(usize, usize) -> Result<Ordering>,
    groups: &mut Vec<CoalescingGroup>,
) -> Result<()> {
    let mut current = 0;
    for idx in partition_boundaries.set_indices() {
        let t = current;
        current = idx + 1;
        append_range_to_groups(source_len, t..current, cmp, groups)?;
    }

    let last = partition_boundaries.len() + 1;
    if current != last {
        append_range_to_groups(source_len, current..last, cmp, groups)?;
    }

    Ok(())
}

/// appends the range of rows to the [`CoalescingGroup`] to which it belongs. This means either
/// setting rows in the selection vec of some existing group that has the same partition value
/// as the rows in the passed range, or creating a new group if no such equivalent group exists.
fn append_range_to_groups(
    source_len: usize,
    range: Range<usize>,
    cmp: &dyn Fn(usize, usize) -> Result<Ordering>,
    groups: &mut Vec<CoalescingGroup>,
) -> Result<()> {
    // try to find a group whose value matches the value of the rows in the passed range
    let mut match_idx = None;
    for (idx, group) in groups.iter().enumerate() {
        if cmp(group.representative_row, range.start)?.is_eq() {
            match_idx = Some(idx);
            break;
        }
    }

    if let Some(idx) = match_idx {
        // set the bits in the selection vec for this group
        set_range_bits(range, &mut groups[idx].selection_vec_builder);
    } else {
        // create a new group
        let mut group = CoalescingGroup {
            representative_row: range.start,
            selection_vec_builder: MutableBuffer::from_len_zeroed(bit_util::ceil(source_len, 8)),
        };
        set_range_bits(range, &mut group.selection_vec_builder);
        groups.push(group)
    }

    Ok(())
}

/// sets all the rows in the range to `1`.
///
/// this expects that the buffer len is at least range.end
fn set_range_bits(range: Range<usize>, bool_buffer: &mut [u8]) {
    let aligned_start_index = bit_util::ceil(range.start, 8) * 8;
    let aligned_end_index = (range.end / 8) * 8;

    if aligned_start_index >= aligned_end_index {
        // range too small to contain a full byte
        for i in range.start..range.end {
            bit_util::set_bit(bool_buffer, i);
        }
        return;
    }

    // set leading partial
    for i in range.start..aligned_start_index {
        bit_util::set_bit(bool_buffer, i);
    }

    // full bytes — memset
    let first_full_byte = aligned_start_index / 8;
    let last_full_byte = aligned_end_index / 8;
    bool_buffer[first_full_byte..last_full_byte].fill(0xFF);

    // set trailing partial
    for i in aligned_end_index..range.end {
        bit_util::set_bit(bool_buffer, i);
    }
}

/// Wrapper around the fields from an AnyValue struct array that will compare the attribute values
/// at two indices. This is functionally equivalent to arrow's [`make_comparator`], but its
/// comparison implementation understands the semantics of how OTAP represents AnyValues.
///
/// Uses [`ArrayComparator`] for each value column, which optimizes comparisons on
/// dictionary-encoded columns by pre-computing group id mappings.
struct AnyValueStructComparator<'a> {
    type_col: &'a UInt8Array,
    struct_nulls: Option<&'a NullBuffer>,
    str_comparator: Option<ArrayComparator>,
    int_comparator: Option<ArrayComparator>,
    float_comparator: Option<ArrayComparator>,
    bool_comparator: Option<ArrayComparator>,
    bytes_comparator: Option<ArrayComparator>,
    ser_comparator: Option<ArrayComparator>,
}

impl<'a> AnyValueStructComparator<'a> {
    fn try_new(
        anyval_struct_arr: &'a StructArray,
        group_id_pool: &mut GroupIdPool,
    ) -> Result<Self> {
        let type_col_arr = anyval_struct_arr
            .column_by_name(consts::ATTRIBUTE_TYPE)
            .ok_or_else(|| otap_df_pdata::error::Error::ColumnNotFound {
                name: consts::ATTRIBUTE_TYPE.into(),
            })?;

        let type_col = type_col_arr
            .as_any()
            .downcast_ref::<UInt8Array>()
            .ok_or_else(|| otap_df_pdata::error::Error::ColumnDataTypeMismatch {
                name: consts::ATTRIBUTE_TYPE.into(),
                actual: type_col_arr.data_type().clone(),
                expect: DataType::UInt8,
            })?;

        Ok(Self {
            type_col,
            struct_nulls: anyval_struct_arr.nulls(),
            str_comparator: anyval_struct_arr
                .column_by_name(consts::ATTRIBUTE_STR)
                .map(|col| ArrayComparator::try_new(col, group_id_pool))
                .transpose()?,
            int_comparator: anyval_struct_arr
                .column_by_name(consts::ATTRIBUTE_INT)
                .map(|col| ArrayComparator::try_new(col, group_id_pool))
                .transpose()?,
            float_comparator: anyval_struct_arr
                .column_by_name(consts::ATTRIBUTE_DOUBLE)
                .map(|col| ArrayComparator::try_new(col, group_id_pool))
                .transpose()?,
            bool_comparator: anyval_struct_arr
                .column_by_name(consts::ATTRIBUTE_BOOL)
                .map(|col| ArrayComparator::try_new(col, group_id_pool))
                .transpose()?,
            bytes_comparator: anyval_struct_arr
                .column_by_name(consts::ATTRIBUTE_BYTES)
                .map(|col| ArrayComparator::try_new(col, group_id_pool))
                .transpose()?,
            ser_comparator: anyval_struct_arr
                .column_by_name(consts::ATTRIBUTE_SER)
                .map(|col| ArrayComparator::try_new(col, group_id_pool))
                .transpose()?,
        })
    }

    fn cmp(&self, i1: usize, i2: usize) -> Result<Ordering> {
        if let Some(nulls) = self.struct_nulls {
            let validity_bitmap = nulls.inner().values();
            let i1_valid = bit_util::get_bit(validity_bitmap, i1);
            let i2_valid = bit_util::get_bit(validity_bitmap, i2);
            if !i1_valid && !i2_valid {
                // both null
                return Ok(Ordering::Equal);
            }
            if i1_valid && !i2_valid {
                return Ok(Ordering::Less);
            }
            if !i1_valid && i2_valid {
                return Ok(Ordering::Greater);
            }
        }

        let type1 = self.type_col.values()[i1];
        let type2 = self.type_col.values()[i2];

        match type1.compare(type2) {
            Ordering::Equal => {
                if type1 == AttributeValueType::Empty as u8 {
                    return Ok(Ordering::Equal);
                }

                if type1 == AttributeValueType::Str as u8 {
                    return Ok(if let Some(cmp) = self.str_comparator.as_ref() {
                        cmp.cmp(i1, i2)
                    } else {
                        Ordering::Equal
                    });
                }

                if type1 == AttributeValueType::Int as u8 {
                    return Ok(if let Some(cmp) = self.int_comparator.as_ref() {
                        cmp.cmp(i1, i2)
                    } else {
                        Ordering::Equal
                    });
                }

                if type1 == AttributeValueType::Double as u8 {
                    return Ok(if let Some(cmp) = self.float_comparator.as_ref() {
                        cmp.cmp(i1, i2)
                    } else {
                        Ordering::Equal
                    });
                }

                if type1 == AttributeValueType::Bool as u8 {
                    return Ok(if let Some(cmp) = self.bool_comparator.as_ref() {
                        cmp.cmp(i1, i2)
                    } else {
                        Ordering::Equal
                    });
                }

                if type1 == AttributeValueType::Bytes as u8 {
                    return Ok(if let Some(cmp) = self.bytes_comparator.as_ref() {
                        cmp.cmp(i1, i2)
                    } else {
                        Ordering::Equal
                    });
                }

                if type1 == AttributeValueType::Slice as u8
                    || type1 == AttributeValueType::Map as u8
                {
                    return Ok(if let Some(cmp) = self.ser_comparator.as_ref() {
                        cmp.cmp(i1, i2)
                    } else {
                        Ordering::Equal
                    });
                }

                Err(Error::ExecutionError {
                    cause: format!("Invalid attribute type enum value {type1:?}"),
                })
            }
            other => Ok(other),
        }
    }

    /// Consumes this comparator and returns all group id buffers to the pool.
    fn return_to_pool(self, pool: &mut GroupIdPool) {
        if let Some(cmp) = self.str_comparator {
            cmp.return_to_pool(pool);
        }
        if let Some(cmp) = self.int_comparator {
            cmp.return_to_pool(pool);
        }
        if let Some(cmp) = self.float_comparator {
            cmp.return_to_pool(pool);
        }
        if let Some(cmp) = self.bool_comparator {
            cmp.return_to_pool(pool);
        }
        if let Some(cmp) = self.bytes_comparator {
            cmp.return_to_pool(pool);
        }
        if let Some(cmp) = self.ser_comparator {
            cmp.return_to_pool(pool);
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use arrow::array::{
        ArrayRef, BinaryArray, BooleanArray, DictionaryArray, Float64Array, Int64Array,
        StringArray, StructArray, UInt8Array, UInt16Array,
    };
    use arrow::buffer::{BooleanBuffer, NullBuffer};
    use arrow::datatypes::{DataType, Field, Fields, UInt8Type, UInt16Type};
    use otap_df_pdata::otlp::attributes::AttributeValueType;
    use otap_df_pdata::proto::OtlpProtoMessage;
    use otap_df_pdata::proto::opentelemetry::common::v1::{
        AnyValue, InstrumentationScope, KeyValue,
    };
    use otap_df_pdata::proto::opentelemetry::logs::v1::{
        LogRecord, LogsData, ResourceLogs, ScopeLogs,
    };
    use otap_df_pdata::proto::opentelemetry::resource::v1::Resource;
    use otap_df_pdata::schema::consts;
    use otap_df_pdata::testing::round_trip::{otap_to_otlp, otlp_to_otap};
    use otap_df_query_engine_languages::opl::parser::OplParser;

    use crate::parser::default_parser_options;
    use crate::pipeline::partition::{
        AnyValueStructComparator, ArrayComparator, GroupIdPool, PartitionValue, Partitioner,
    };

    #[test]
    fn test_partition_logs_by_severity_number() {
        let otap = otlp_to_otap(&OtlpProtoMessage::Logs(LogsData::new(vec![
            ResourceLogs::new(
                Resource::default(),
                vec![ScopeLogs::new(
                    InstrumentationScope::default(),
                    vec![
                        LogRecord::build()
                            .severity_number(13)
                            .event_name("e1")
                            .finish(),
                        LogRecord::build()
                            .severity_number(17)
                            .event_name("e2")
                            .finish(),
                        LogRecord::build()
                            .severity_number(13)
                            .event_name("e3")
                            .finish(),
                        LogRecord::build()
                            .severity_number(17)
                            .event_name("e4")
                            .finish(),
                    ],
                )],
            ),
        ])));

        let (scalar_expr, functions) =
            OplParser::parse_expr_with_options("severity_number", default_parser_options())
                .unwrap();
        let mut partitioner = Partitioner::try_new(scalar_expr, functions).unwrap();
        let partitions = partitioner.partition(otap).unwrap().collect::<Vec<_>>();

        assert_eq!(partitions.len(), 2, "expected 2 partitions");

        assert_eq!(partitions[0].value, PartitionValue::Int(13));
        let OtlpProtoMessage::Logs(partition0_logs) = otap_to_otlp(&partitions[0].batch) else {
            panic!("expected logs");
        };
        assert_eq!(
            partition0_logs,
            LogsData::new(vec![ResourceLogs::new(
                Resource::default(),
                vec![ScopeLogs::new(
                    InstrumentationScope::default(),
                    vec![
                        LogRecord::build()
                            .severity_number(13)
                            .event_name("e1")
                            .finish(),
                        LogRecord::build()
                            .severity_number(13)
                            .event_name("e3")
                            .finish(),
                    ],
                )],
            )])
        );

        assert_eq!(partitions[1].value, PartitionValue::Int(17));
        let OtlpProtoMessage::Logs(partition1_logs) = otap_to_otlp(&partitions[1].batch) else {
            panic!("expected logs");
        };
        assert_eq!(
            partition1_logs,
            LogsData::new(vec![ResourceLogs::new(
                Resource::default(),
                vec![ScopeLogs::new(
                    InstrumentationScope::default(),
                    vec![
                        LogRecord::build()
                            .severity_number(17)
                            .event_name("e2")
                            .finish(),
                        LogRecord::build()
                            .severity_number(17)
                            .event_name("e4")
                            .finish(),
                    ],
                )],
            )])
        );
    }

    #[test]
    fn test_partition_traces_by_span_name() {
        use otap_df_pdata::proto::opentelemetry::trace::v1::{
            ResourceSpans, ScopeSpans, Span, Status, TracesData,
        };

        let otap = otlp_to_otap(&OtlpProtoMessage::Traces(TracesData::new(vec![
            ResourceSpans::new(
                Resource::default(),
                vec![ScopeSpans::new(
                    InstrumentationScope::default(),
                    vec![
                        Span::build()
                            .trace_id(vec![1u8; 16])
                            .span_id(vec![1u8; 8])
                            .name("op-a")
                            .finish(),
                        Span::build()
                            .trace_id(vec![2u8; 16])
                            .span_id(vec![2u8; 8])
                            .name("op-b")
                            .finish(),
                        Span::build()
                            .trace_id(vec![3u8; 16])
                            .span_id(vec![3u8; 8])
                            .name("op-a")
                            .finish(),
                        Span::build()
                            .trace_id(vec![4u8; 16])
                            .span_id(vec![4u8; 8])
                            .name("op-b")
                            .finish(),
                    ],
                )],
            ),
        ])));

        let (scalar_expr, functions) =
            OplParser::parse_expr_with_options("name", default_parser_options()).unwrap();
        let mut partitioner = Partitioner::try_new(scalar_expr, functions).unwrap();
        let partitions = partitioner.partition(otap).unwrap().collect::<Vec<_>>();

        assert_eq!(partitions.len(), 2, "expected 2 partitions");

        assert_eq!(partitions[0].value, PartitionValue::String("op-a".into()));
        let OtlpProtoMessage::Traces(partition0_traces) = otap_to_otlp(&partitions[0].batch) else {
            panic!("expected traces");
        };
        assert_eq!(
            partition0_traces,
            TracesData::new(vec![ResourceSpans::new(
                Resource::default(),
                vec![ScopeSpans::new(
                    InstrumentationScope::default(),
                    vec![
                        Span::build()
                            .trace_id(vec![1u8; 16])
                            .span_id(vec![1u8; 8])
                            .name("op-a")
                            .status(Status::default())
                            .finish(),
                        Span::build()
                            .trace_id(vec![3u8; 16])
                            .span_id(vec![3u8; 8])
                            .name("op-a")
                            .status(Status::default())
                            .finish(),
                    ],
                )],
            )])
        );

        assert_eq!(partitions[1].value, PartitionValue::String("op-b".into()));
        let OtlpProtoMessage::Traces(partition1_traces) = otap_to_otlp(&partitions[1].batch) else {
            panic!("expected traces");
        };
        assert_eq!(
            partition1_traces,
            TracesData::new(vec![ResourceSpans::new(
                Resource::default(),
                vec![ScopeSpans::new(
                    InstrumentationScope::default(),
                    vec![
                        Span::build()
                            .trace_id(vec![2u8; 16])
                            .span_id(vec![2u8; 8])
                            .name("op-b")
                            .status(Status::default())
                            .finish(),
                        Span::build()
                            .trace_id(vec![4u8; 16])
                            .span_id(vec![4u8; 8])
                            .name("op-b")
                            .status(Status::default())
                            .finish(),
                    ],
                )],
            )])
        );
    }

    #[test]
    fn test_partition_using_functions() {
        let input = otlp_to_otap(&OtlpProtoMessage::Logs(LogsData::new(vec![
            ResourceLogs::new(
                Resource::build()
                    .attributes(vec![KeyValue::new("attr1", AnyValue::new_string("506"))]),
                vec![ScopeLogs::new(
                    InstrumentationScope::default(),
                    vec![LogRecord::build().event_name("e1").finish()],
                )],
            ),
            ResourceLogs::new(
                Resource::build()
                    .attributes(vec![KeyValue::new("attr1", AnyValue::new_string("514"))]),
                vec![ScopeLogs::new(
                    InstrumentationScope::default(),
                    vec![LogRecord::build().event_name("e2").finish()],
                )],
            ),
            ResourceLogs::new(
                Resource::build()
                    .attributes(vec![KeyValue::new("attr1", AnyValue::new_string("418"))]),
                vec![ScopeLogs::new(
                    InstrumentationScope::default(),
                    vec![LogRecord::build().event_name("e3").finish()],
                )],
            ),
        ])));

        let (scalar_expr, functions) = OplParser::parse_expr_with_options(
            "substring(resource.attributes[\"attr1\"], 0, 1)",
            default_parser_options(),
        )
        .unwrap();
        let mut partitioner = Partitioner::try_new(scalar_expr, functions).unwrap();
        let partitions = partitioner.partition(input).unwrap().collect::<Vec<_>>();
        assert_eq!(partitions.len(), 2);

        assert_eq!(partitions[0].value, PartitionValue::String("5".into()));
        let OtlpProtoMessage::Logs(partition0_log_records) = otap_to_otlp(&partitions[0].batch)
        else {
            panic!("invalid signal type")
        };

        assert_eq!(
            partition0_log_records,
            LogsData::new(vec![
                ResourceLogs::new(
                    Resource::build()
                        .attributes(vec![KeyValue::new("attr1", AnyValue::new_string("506"))]),
                    vec![ScopeLogs::new(
                        InstrumentationScope::default(),
                        vec![LogRecord::build().event_name("e1").finish()],
                    )],
                ),
                ResourceLogs::new(
                    Resource::build()
                        .attributes(vec![KeyValue::new("attr1", AnyValue::new_string("514"))]),
                    vec![ScopeLogs::new(
                        InstrumentationScope::default(),
                        vec![LogRecord::build().event_name("e2").finish()],
                    )],
                ),
            ])
        );

        assert_eq!(partitions[1].value, PartitionValue::String("4".into()));
        let OtlpProtoMessage::Logs(partition1_log_records) = otap_to_otlp(&partitions[1].batch)
        else {
            panic!("invalid signal type")
        };

        assert_eq!(
            partition1_log_records,
            LogsData::new(vec![ResourceLogs::new(
                Resource::build()
                    .attributes(vec![KeyValue::new("attr1", AnyValue::new_string("418"))]),
                vec![ScopeLogs::new(
                    InstrumentationScope::default(),
                    vec![LogRecord::build().event_name("e3").finish()],
                )],
            )])
        )
    }

    #[test]
    fn test_partition_missing_data_produces_null_partition_value() {
        let otap = otlp_to_otap(&OtlpProtoMessage::Logs(LogsData::new(vec![
            ResourceLogs::new(
                Resource::default(),
                vec![ScopeLogs::new(
                    InstrumentationScope::default(),
                    vec![
                        LogRecord::build()
                            .attributes(vec![KeyValue::new("attrx", AnyValue::new_string("y"))])
                            .event_name("e1")
                            .finish(),
                        LogRecord::build()
                            .severity_number(17)
                            .event_name("e2")
                            .finish(),
                        LogRecord::build()
                            .severity_number(13)
                            .event_name("e3")
                            .finish(),
                        LogRecord::build()
                            .severity_number(17)
                            .event_name("e4")
                            .finish(),
                    ],
                )],
            ),
        ])));

        let (scalar_expr, functions) = OplParser::parse_expr_with_options(
            "attributes[\"some_attr_not_existing\"]",
            default_parser_options(),
        )
        .unwrap();
        let mut partitioner = Partitioner::try_new(scalar_expr, functions).unwrap();
        let partitions = partitioner.partition(otap).unwrap().collect::<Vec<_>>();
        assert_eq!(partitions.len(), 1);
        assert_eq!(partitions[0].value, PartitionValue::Null);
    }

    #[test]
    fn test_partition_anyvalue_struct_array_homogenous_type_no_nulls() {
        let otap = otlp_to_otap(&OtlpProtoMessage::Logs(LogsData::new(vec![
            ResourceLogs::new(
                Resource::default(),
                vec![ScopeLogs::new(
                    InstrumentationScope::default(),
                    vec![
                        LogRecord::build()
                            .event_name("event0")
                            .attributes(vec![KeyValue::new("x", AnyValue::new_string("0"))])
                            .finish(),
                        LogRecord::build()
                            .event_name("event1")
                            .attributes(vec![KeyValue::new("x", AnyValue::new_string("0"))])
                            .finish(),
                        LogRecord::build()
                            .event_name("event2")
                            .attributes(vec![KeyValue::new("x", AnyValue::new_string("1"))])
                            .finish(),
                        LogRecord::build()
                            .event_name("event3")
                            .attributes(vec![KeyValue::new("x", AnyValue::new_string("1"))])
                            .finish(),
                        LogRecord::build()
                            .event_name("event4")
                            .attributes(vec![KeyValue::new("x", AnyValue::new_string("1"))])
                            .finish(),
                        LogRecord::build()
                            .event_name("event5")
                            .attributes(vec![KeyValue::new("x", AnyValue::new_string("2"))])
                            .finish(),
                        LogRecord::build()
                            .event_name("event6")
                            .attributes(vec![KeyValue::new("x", AnyValue::new_string("2"))])
                            .finish(),
                        LogRecord::build()
                            .event_name("event7")
                            .attributes(vec![KeyValue::new("x", AnyValue::new_string("2"))])
                            .finish(),
                        LogRecord::build()
                            .event_name("event8")
                            .attributes(vec![KeyValue::new("x", AnyValue::new_string("2"))])
                            .finish(),
                        LogRecord::build()
                            .event_name("event9")
                            .attributes(vec![KeyValue::new("x", AnyValue::new_string("0"))])
                            .finish(),
                        LogRecord::build()
                            .event_name("event10")
                            .attributes(vec![KeyValue::new("x", AnyValue::new_string("1"))])
                            .finish(),
                        LogRecord::build()
                            .event_name("event11")
                            .attributes(vec![KeyValue::new("x", AnyValue::new_string("2"))])
                            .finish(),
                        LogRecord::build()
                            .event_name("event12")
                            .attributes(vec![KeyValue::new("x", AnyValue::new_string("2"))])
                            .finish(),
                    ],
                )],
            ),
        ])));

        let (scalar_expr, functions) =
            OplParser::parse_expr_with_options("attributes[\"x\"]", default_parser_options())
                .unwrap();
        let mut partitioner = Partitioner::try_new(scalar_expr, functions).unwrap();
        let partitions = partitioner.partition(otap).unwrap().collect::<Vec<_>>();
        assert_eq!(partitions.len(), 3);

        assert_eq!(partitions[0].value, PartitionValue::String("0".into()));
        let OtlpProtoMessage::Logs(partition0_log_records) = otap_to_otlp(&partitions[0].batch)
        else {
            panic!("invalid signal type")
        };
        assert_eq!(
            partition0_log_records,
            LogsData::new(vec![ResourceLogs::new(
                Resource::default(),
                vec![ScopeLogs::new(
                    InstrumentationScope::default(),
                    vec![
                        LogRecord::build()
                            .event_name("event0")
                            .attributes(vec![KeyValue::new("x", AnyValue::new_string("0"))])
                            .finish(),
                        LogRecord::build()
                            .event_name("event1")
                            .attributes(vec![KeyValue::new("x", AnyValue::new_string("0"))])
                            .finish(),
                        LogRecord::build()
                            .event_name("event9")
                            .attributes(vec![KeyValue::new("x", AnyValue::new_string("0"))])
                            .finish(),
                    ],
                )],
            ),])
        );

        assert_eq!(partitions[1].value, PartitionValue::String("1".into()));
        let OtlpProtoMessage::Logs(partition1_log_records) = otap_to_otlp(&partitions[1].batch)
        else {
            panic!("invalid signal type")
        };
        assert_eq!(
            partition1_log_records,
            LogsData::new(vec![ResourceLogs::new(
                Resource::default(),
                vec![ScopeLogs::new(
                    InstrumentationScope::default(),
                    vec![
                        LogRecord::build()
                            .event_name("event2")
                            .attributes(vec![KeyValue::new("x", AnyValue::new_string("1"))])
                            .finish(),
                        LogRecord::build()
                            .event_name("event3")
                            .attributes(vec![KeyValue::new("x", AnyValue::new_string("1"))])
                            .finish(),
                        LogRecord::build()
                            .event_name("event4")
                            .attributes(vec![KeyValue::new("x", AnyValue::new_string("1"))])
                            .finish(),
                        LogRecord::build()
                            .event_name("event10")
                            .attributes(vec![KeyValue::new("x", AnyValue::new_string("1"))])
                            .finish(),
                    ],
                )],
            ),])
        );

        assert_eq!(partitions[2].value, PartitionValue::String("2".into()));
        let OtlpProtoMessage::Logs(partition2_log_records) = otap_to_otlp(&partitions[2].batch)
        else {
            panic!("invalid signal type")
        };
        assert_eq!(
            partition2_log_records,
            LogsData::new(vec![ResourceLogs::new(
                Resource::default(),
                vec![ScopeLogs::new(
                    InstrumentationScope::default(),
                    vec![
                        LogRecord::build()
                            .event_name("event5")
                            .attributes(vec![KeyValue::new("x", AnyValue::new_string("2"))])
                            .finish(),
                        LogRecord::build()
                            .event_name("event6")
                            .attributes(vec![KeyValue::new("x", AnyValue::new_string("2"))])
                            .finish(),
                        LogRecord::build()
                            .event_name("event7")
                            .attributes(vec![KeyValue::new("x", AnyValue::new_string("2"))])
                            .finish(),
                        LogRecord::build()
                            .event_name("event8")
                            .attributes(vec![KeyValue::new("x", AnyValue::new_string("2"))])
                            .finish(),
                        LogRecord::build()
                            .event_name("event11")
                            .attributes(vec![KeyValue::new("x", AnyValue::new_string("2"))])
                            .finish(),
                        LogRecord::build()
                            .event_name("event12")
                            .attributes(vec![KeyValue::new("x", AnyValue::new_string("2"))])
                            .finish(),
                    ],
                )],
            ),])
        );
    }

    #[test]
    fn test_anyval_comparator_type_and_value_logic() {
        let anyval_struct = StructArray::new(
            Fields::from(vec![
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
                Field::new(consts::ATTRIBUTE_INT, DataType::Int64, true),
            ]),
            vec![
                Arc::new(UInt8Array::from_iter_values([
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Int as u8,
                    AttributeValueType::Int as u8,
                    AttributeValueType::Int as u8,
                ])),
                Arc::new(StringArray::from_iter_values(["a", "a", "b", "", "", ""])),
                Arc::new(Int64Array::from_iter_values([0, 0, 0, 0, 0, 1])),
            ],
            None,
        );

        let mut pool = GroupIdPool::new();
        let comparator = AnyValueStructComparator::try_new(&anyval_struct, &mut pool).unwrap();

        assert!(comparator.cmp(0, 1).unwrap().is_eq()); // equivalent strings
        assert!(comparator.cmp(1, 2).unwrap().is_ne()); // non-equivalent strings
        assert!(comparator.cmp(2, 3).unwrap().is_ne()); // non-matching types
        assert!(comparator.cmp(3, 4).unwrap().is_eq()); // matching ints
        assert!(comparator.cmp(4, 5).unwrap().is_ne()); // non-matching ints
    }

    #[test]
    fn test_anyval_comparator_bool() {
        let anyval_struct = StructArray::new(
            Fields::from(vec![
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(consts::ATTRIBUTE_BOOL, DataType::Boolean, true),
            ]),
            vec![
                Arc::new(UInt8Array::from_iter_values([
                    AttributeValueType::Bool as u8,
                    AttributeValueType::Bool as u8,
                    AttributeValueType::Bool as u8,
                    AttributeValueType::Str as u8,
                ])),
                Arc::new(BooleanArray::from(vec![true, true, false, false])),
            ],
            None,
        );

        let mut pool = GroupIdPool::new();
        let comparator = AnyValueStructComparator::try_new(&anyval_struct, &mut pool).unwrap();

        assert!(comparator.cmp(0, 1).unwrap().is_eq());
        assert!(comparator.cmp(1, 2).unwrap().is_ne());
        assert!(comparator.cmp(2, 3).unwrap().is_ne());
    }

    #[test]
    fn test_anyval_comparator_double() {
        let anyval_struct = StructArray::new(
            Fields::from(vec![
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(consts::ATTRIBUTE_DOUBLE, DataType::Float64, true),
            ]),
            vec![
                Arc::new(UInt8Array::from_iter_values([
                    AttributeValueType::Double as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Str as u8,
                ])),
                Arc::new(Float64Array::from_iter_values([0.1, 0.1, 0.0, 0.0])),
            ],
            None,
        );

        let mut pool = GroupIdPool::new();
        let comparator = AnyValueStructComparator::try_new(&anyval_struct, &mut pool).unwrap();
        assert!(comparator.cmp(0, 1).unwrap().is_eq());
        assert!(comparator.cmp(1, 2).unwrap().is_ne());
        assert!(comparator.cmp(2, 3).unwrap().is_ne());
    }

    #[test]
    fn test_anyval_comparator_bytes() {
        let anyval_struct = StructArray::new(
            Fields::from(vec![
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(consts::ATTRIBUTE_BYTES, DataType::Binary, true),
            ]),
            vec![
                Arc::new(UInt8Array::from_iter_values([
                    AttributeValueType::Bytes as u8,
                    AttributeValueType::Bytes as u8,
                    AttributeValueType::Bytes as u8,
                    AttributeValueType::Str as u8,
                ])),
                Arc::new(BinaryArray::from_iter_values([b"0", b"0", b"1", b"1"])),
            ],
            None,
        );

        let mut pool = GroupIdPool::new();
        let comparator = AnyValueStructComparator::try_new(&anyval_struct, &mut pool).unwrap();
        assert!(comparator.cmp(0, 1).unwrap().is_eq());
        assert!(comparator.cmp(1, 2).unwrap().is_ne());
        assert!(comparator.cmp(2, 3).unwrap().is_ne());
    }

    #[test]
    fn test_anyval_comparator_slice_and_map() {
        let anyval_struct = StructArray::new(
            Fields::from(vec![
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(consts::ATTRIBUTE_SER, DataType::Binary, true),
            ]),
            vec![
                Arc::new(UInt8Array::from_iter_values([
                    AttributeValueType::Map as u8,
                    AttributeValueType::Map as u8,
                    AttributeValueType::Map as u8,
                    AttributeValueType::Slice as u8,
                    AttributeValueType::Slice as u8,
                    AttributeValueType::Slice as u8,
                    AttributeValueType::Str as u8,
                ])),
                Arc::new(BinaryArray::from_iter_values([
                    b"0", b"0", b"1", b"1", b"1", b"2", b"b",
                ])),
            ],
            None,
        );

        let mut pool = GroupIdPool::new();
        let comparator = AnyValueStructComparator::try_new(&anyval_struct, &mut pool).unwrap();
        assert!(comparator.cmp(0, 1).unwrap().is_eq());
        assert!(comparator.cmp(1, 2).unwrap().is_ne());
        assert!(comparator.cmp(2, 3).unwrap().is_ne());
        assert!(comparator.cmp(3, 4).unwrap().is_eq());
        assert!(comparator.cmp(4, 5).unwrap().is_ne());
        assert!(comparator.cmp(5, 6).unwrap().is_ne());
    }

    #[test]
    fn test_anyval_comparator_empty() {
        let anyval_struct = StructArray::new(
            Fields::from(vec![Field::new(
                consts::ATTRIBUTE_TYPE,
                DataType::UInt8,
                false,
            )]),
            vec![Arc::new(UInt8Array::from_iter_values([
                AttributeValueType::Empty as u8,
                AttributeValueType::Empty as u8,
                AttributeValueType::Str as u8,
            ]))],
            None,
        );

        let mut pool = GroupIdPool::new();
        let comparator = AnyValueStructComparator::try_new(&anyval_struct, &mut pool).unwrap();
        assert!(comparator.cmp(0, 1).unwrap().is_eq());
        assert!(comparator.cmp(1, 2).unwrap().is_ne());
    }

    #[test]
    fn test_anyval_comparator_missing_columns_treated_as_equal() {
        let anyval_struct = StructArray::new(
            Fields::from(vec![Field::new(
                consts::ATTRIBUTE_TYPE,
                DataType::UInt8,
                false,
            )]),
            vec![Arc::new(UInt8Array::from_iter_values([
                AttributeValueType::Str as u8,
                AttributeValueType::Str as u8,
                AttributeValueType::Int as u8,
                AttributeValueType::Int as u8,
            ]))],
            None,
        );

        // since the values columns are missing, we assume they are all null so effectively
        // the partition has equivalent values
        let mut pool = GroupIdPool::new();
        let comparator = AnyValueStructComparator::try_new(&anyval_struct, &mut pool).unwrap();
        assert!(comparator.cmp(0, 1).unwrap().is_eq());
        assert!(comparator.cmp(1, 2).unwrap().is_ne());
        assert!(comparator.cmp(2, 3).unwrap().is_eq());
    }

    #[test]
    fn test_anyval_comparator_null_values() {
        let anyval_struct = StructArray::new(
            Fields::from(vec![
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
            ]),
            vec![
                Arc::new(UInt8Array::from_iter_values([
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                ])),
                Arc::new(StringArray::from_iter([
                    Some(""),
                    Some(""),
                    None,
                    None,
                    Some(""),
                ])),
            ],
            None,
        );

        // since the values columns are missing, we assume they are all null so effectively
        // the partition has equivalent values
        let mut pool = GroupIdPool::new();
        let comparator = AnyValueStructComparator::try_new(&anyval_struct, &mut pool).unwrap();
        assert!(comparator.cmp(0, 1).unwrap().is_eq());
        assert!(comparator.cmp(1, 2).unwrap().is_ne());
        assert!(comparator.cmp(2, 3).unwrap().is_eq());
        assert!(comparator.cmp(3, 4).unwrap().is_ne());
    }

    #[test]
    fn test_anyval_comparator_null_struct_col() {
        let anyval_struct = StructArray::new(
            Fields::from(vec![
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
            ]),
            vec![
                Arc::new(UInt8Array::from_iter_values([
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                ])),
                Arc::new(StringArray::from_iter([
                    Some(""),
                    Some(""),
                    Some(""),
                    Some("a"),
                    Some("a"),
                ])),
            ],
            Some(NullBuffer::new(BooleanBuffer::from(vec![
                true, true, false, false, true,
            ]))),
        );

        let mut pool = GroupIdPool::new();
        let comparator = AnyValueStructComparator::try_new(&anyval_struct, &mut pool).unwrap();
        assert!(comparator.cmp(0, 1).unwrap().is_eq());
        assert!(comparator.cmp(1, 2).unwrap().is_ne());
        assert!(comparator.cmp(2, 3).unwrap().is_eq());
        assert!(comparator.cmp(3, 4).unwrap().is_ne());
    }

    #[test]
    fn test_array_comparator_dict_encoded_strings() {
        // Dictionary with values: ["foo", "bar", "baz"]
        // Keys: [0, 1, 0, 2, 1]
        // Logical array: ["foo", "bar", "foo", "baz", "bar"]
        let values = StringArray::from(vec!["foo", "bar", "baz"]);
        let keys = UInt8Array::from(vec![0u8, 1, 0, 2, 1]);
        let dict = DictionaryArray::new(keys, Arc::new(values));
        let array: ArrayRef = Arc::new(dict);

        let mut pool = GroupIdPool::new();
        let cmp = ArrayComparator::try_new(&array, &mut pool).unwrap();

        // "foo" == "foo"
        assert!(cmp.cmp(0, 2).is_eq());
        // "bar" == "bar"
        assert!(cmp.cmp(1, 4).is_eq());
        // "foo" != "bar"
        assert!(cmp.cmp(0, 1).is_ne());
        // "foo" != "baz"
        assert!(cmp.cmp(0, 3).is_ne());
        // "bar" != "baz"
        assert!(cmp.cmp(1, 3).is_ne());

        cmp.return_to_pool(&mut pool);
    }

    #[test]
    fn test_array_comparator_dict_with_duplicate_values() {
        // Dictionary with duplicate values: ["foo", "bar", "foo"]
        // Key 0 and key 2 both map to "foo" - this is the key case we're optimizing for.
        // Keys: [0, 1, 2, 0, 1]
        // Logical array: ["foo", "bar", "foo", "foo", "bar"]
        let values = StringArray::from(vec!["foo", "bar", "foo"]);
        let keys = UInt8Array::from(vec![0u8, 1, 2, 0, 1]);
        let dict = DictionaryArray::new(keys, Arc::new(values));
        let array: ArrayRef = Arc::new(dict);

        let mut pool = GroupIdPool::new();
        let cmp = ArrayComparator::try_new(&array, &mut pool).unwrap();

        // row 0 (key=0, "foo") == row 2 (key=2, "foo") -- different keys, same value
        assert!(cmp.cmp(0, 2).is_eq());
        // row 0 (key=0, "foo") == row 3 (key=0, "foo") -- same key
        assert!(cmp.cmp(0, 3).is_eq());
        // row 2 (key=2, "foo") == row 3 (key=0, "foo") -- different keys, same value
        assert!(cmp.cmp(2, 3).is_eq());
        // row 0 ("foo") != row 1 ("bar")
        assert!(cmp.cmp(0, 1).is_ne());
        // row 1 ("bar") == row 4 ("bar")
        assert!(cmp.cmp(1, 4).is_eq());

        cmp.return_to_pool(&mut pool);
    }

    #[test]
    fn test_array_comparator_dict_with_nulls() {
        // Dictionary: values=["a", "b"], keys=[0, null, 1, null, 0]
        let values = StringArray::from(vec!["a", "b"]);
        let keys = UInt8Array::from(vec![Some(0u8), None, Some(1), None, Some(0)]);
        let dict = DictionaryArray::new(keys, Arc::new(values));
        let array: ArrayRef = Arc::new(dict);

        let mut pool = GroupIdPool::new();
        let cmp = ArrayComparator::try_new(&array, &mut pool).unwrap();

        // row 0 ("a") == row 4 ("a")
        assert!(cmp.cmp(0, 4).is_eq());
        // row 1 (null) == row 3 (null)
        assert!(cmp.cmp(1, 3).is_eq());
        // row 0 ("a") != row 1 (null)
        assert!(cmp.cmp(0, 1).is_ne());
        // row 0 ("a") != row 2 ("b")
        assert!(cmp.cmp(0, 2).is_ne());

        cmp.return_to_pool(&mut pool);
    }

    #[test]
    fn test_array_comparator_dict_uint16_keys() {
        // Test with UInt16 key type (common in OTAP for higher-cardinality dicts)
        let values = StringArray::from(vec!["alpha", "beta", "alpha"]);
        let keys = UInt16Array::from(vec![0u16, 1, 2, 0, 1]);
        let dict: DictionaryArray<UInt16Type> = DictionaryArray::new(keys, Arc::new(values));
        let array: ArrayRef = Arc::new(dict);

        let mut pool = GroupIdPool::new();
        let cmp = ArrayComparator::try_new(&array, &mut pool).unwrap();

        // row 0 (key=0, "alpha") == row 2 (key=2, "alpha") -- deduped
        assert!(cmp.cmp(0, 2).is_eq());
        // row 0 (key=0, "alpha") == row 3 (key=0, "alpha")
        assert!(cmp.cmp(0, 3).is_eq());
        // row 0 ("alpha") != row 1 ("beta")
        assert!(cmp.cmp(0, 1).is_ne());
        // row 1 ("beta") == row 4 ("beta")
        assert!(cmp.cmp(1, 4).is_eq());

        cmp.return_to_pool(&mut pool);
    }

    #[test]
    fn test_array_comparator_non_dict_falls_back_to_standard() {
        // Plain (non-dict) string array should use Standard variant
        let array: ArrayRef = Arc::new(StringArray::from(vec!["a", "b", "a", "c"]));

        let mut pool = GroupIdPool::new();
        let cmp = ArrayComparator::try_new(&array, &mut pool).unwrap();

        assert!(cmp.cmp(0, 2).is_eq());
        assert!(cmp.cmp(0, 1).is_ne());
        assert!(cmp.cmp(1, 3).is_ne());

        // Standard variant should not return anything to the pool
        cmp.return_to_pool(&mut pool);
        assert!(pool.pool.is_empty());
    }

    #[test]
    fn test_group_id_pool_reuses_allocations() {
        let mut pool = GroupIdPool::new();
        assert!(pool.pool.is_empty());

        // Take creates a new buffer
        let buf = pool.take();
        assert!(pool.pool.is_empty());

        // Give back returns it
        pool.give_back(buf);
        assert_eq!(pool.pool.len(), 1);

        // Take reuses the allocation
        let buf = pool.take();
        assert!(pool.pool.is_empty());

        // Use it, then return
        let mut buf = buf;
        buf.resize(100, 42);
        pool.give_back(buf);

        // The returned buffer should be cleared but still have capacity
        let buf = pool.take();
        assert!(buf.is_empty());
        assert!(buf.capacity() >= 100);
        pool.give_back(buf);
    }

    #[test]
    fn test_anyval_comparator_dict_encoded_str_column() {
        // Test AnyValueStructComparator with a dict-encoded string column.
        // This is the common case in OTAP: attribute string values are dict-encoded.
        let values = StringArray::from(vec!["hello", "world", "hello"]);
        let keys = UInt8Array::from(vec![0u8, 0, 1, 2, 1]);
        let dict_str: DictionaryArray<UInt8Type> = DictionaryArray::new(keys, Arc::new(values));

        let anyval_struct = StructArray::new(
            Fields::from(vec![
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(
                    consts::ATTRIBUTE_STR,
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                    true,
                ),
            ]),
            vec![
                Arc::new(UInt8Array::from_iter_values([
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                ])),
                Arc::new(dict_str),
            ],
            None,
        );

        let mut pool = GroupIdPool::new();
        let comparator = AnyValueStructComparator::try_new(&anyval_struct, &mut pool).unwrap();

        // row 0 ("hello") == row 1 ("hello") -- same key
        assert!(comparator.cmp(0, 1).unwrap().is_eq());
        // row 0 ("hello") != row 2 ("world")
        assert!(comparator.cmp(0, 2).unwrap().is_ne());
        // row 0 (key=0, "hello") == row 3 (key=2, "hello") -- different keys, same value
        assert!(comparator.cmp(0, 3).unwrap().is_eq());
        // row 2 ("world") == row 4 ("world")
        assert!(comparator.cmp(2, 4).unwrap().is_eq());

        comparator.return_to_pool(&mut pool);
    }
}
