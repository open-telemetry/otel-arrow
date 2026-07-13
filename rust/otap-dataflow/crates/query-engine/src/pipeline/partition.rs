// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Utilities for splitting based on the result of some expression.
//!
//! For example, we may wish to split an OTAP batch on some some attribute/resource attribute,
//! or some computed value, like
//! `sha256(concat(resource.attributes["k8s.namespace.name"], resource.attributes["service.name"]`)`
//!
//! The main public entrypoint for this module is the [`Partitioner`] type.

use std::cmp::Ordering;
use std::ops::Range;

use arrow::array::{
    Array, ArrayRef, ArrowNativeTypeOp, AsArray, BooleanArray, DynComparator, StructArray,
    UInt8Array, make_comparator,
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
            partitions: Vec::new(),
        })
    }

    /// Evaluates the
    pub fn partition(
        &mut self,
        otap_batch: OtapArrowRecords,
    ) -> Result<impl IntoIterator<Item = Partition>> {
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
                )
            } else {
                partition_simple_array(
                    array,
                    otap_batch,
                    result_partitions,
                    range_coalescer,
                    id_bitmap_pool,
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
) -> Result<()> {
    let cmp = make_comparator(&array, &array, SortOptions::default())?;
    let boundaries: BooleanBuffer = (0..array.len() - 1)
        .map(|i| !cmp(i, i + 1).is_eq())
        .collect();

    partition_at_boundaries(
        array.as_ref(),
        boundaries,
        otap_batch,
        result,
        range_coalescer,
        id_bitmap_pool,
        &|i1, i2| Ok(cmp(i1, i2)),
    )
}

/// Populate the `result` vec with partitions of the OTAP batch based on which rows in the passed
/// struct array (which represents a list of `AnyValue`) have equivalent values.
fn partition_any_value_struct_array(
    array: &StructArray,
    otap_batch: OtapArrowRecords,
    result: &mut Vec<Partition>,
    range_coalescer: &mut PartitionRangeCoalescer,
    id_bitmap_pool: &mut IdBitmapPool,
) -> Result<()> {
    let comparator = AnyValueStructComparator::try_new(array)?;
    let boundaries: BooleanBuffer = (0..array.len() - 1)
        .map(|i| comparator.cmp(i, i + 1).map(|ord| !ord.is_eq()))
        .collect::<Result<Vec<bool>>>()?
        .into();

    partition_at_boundaries(
        array,
        boundaries,
        otap_batch,
        result,
        range_coalescer,
        id_bitmap_pool,
        &|i1, i2| comparator.cmp(i1, i2),
    )
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
            selection_vec_builder: MutableBuffer::from_len_zeroed(source_len),
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
struct AnyValueStructComparator<'a> {
    type_col: &'a UInt8Array,
    struct_nulls: Option<&'a NullBuffer>,
    str_comparator: Option<DynComparator>,
    int_comparator: Option<DynComparator>,
    float_comparator: Option<DynComparator>,
    bool_comparator: Option<DynComparator>,
    bytes_comparator: Option<DynComparator>,
    ser_comparator: Option<DynComparator>,
}

impl<'a> AnyValueStructComparator<'a> {
    fn try_new(anyval_struct_arr: &'a StructArray) -> Result<Self> {
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
                .map(|col| make_comparator(col, col, SortOptions::default()))
                .transpose()?,
            int_comparator: anyval_struct_arr
                .column_by_name(consts::ATTRIBUTE_INT)
                .map(|col| make_comparator(col, col, SortOptions::default()))
                .transpose()?,
            float_comparator: anyval_struct_arr
                .column_by_name(consts::ATTRIBUTE_DOUBLE)
                .map(|col| make_comparator(col, col, SortOptions::default()))
                .transpose()?,
            bool_comparator: anyval_struct_arr
                .column_by_name(consts::ATTRIBUTE_BOOL)
                .map(|col| make_comparator(col, col, SortOptions::default()))
                .transpose()?,
            bytes_comparator: anyval_struct_arr
                .column_by_name(consts::ATTRIBUTE_BYTES)
                .map(|col| make_comparator(col, col, SortOptions::default()))
                .transpose()?,
            ser_comparator: anyval_struct_arr
                .column_by_name(consts::ATTRIBUTE_SER)
                .map(|col| make_comparator(col, col, SortOptions::default()))
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
                        cmp(i1, i2)
                    } else {
                        Ordering::Equal
                    });
                }

                if type1 == AttributeValueType::Int as u8 {
                    return Ok(if let Some(cmp) = self.int_comparator.as_ref() {
                        cmp(i1, i2)
                    } else {
                        Ordering::Equal
                    });
                }

                if type1 == AttributeValueType::Double as u8 {
                    return Ok(if let Some(cmp) = self.float_comparator.as_ref() {
                        cmp(i1, i2)
                    } else {
                        Ordering::Equal
                    });
                }

                if type1 == AttributeValueType::Bool as u8 {
                    return Ok(if let Some(cmp) = self.bool_comparator.as_ref() {
                        cmp(i1, i2)
                    } else {
                        Ordering::Equal
                    });
                }

                if type1 == AttributeValueType::Bytes as u8 {
                    return Ok(if let Some(cmp) = self.bytes_comparator.as_ref() {
                        cmp(i1, i2)
                    } else {
                        Ordering::Equal
                    });
                }

                if type1 == AttributeValueType::Slice as u8
                    || type1 == AttributeValueType::Map as u8
                {
                    return Ok(if let Some(cmp) = self.ser_comparator.as_ref() {
                        cmp(i1, i2)
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
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use arrow::array::{
        BinaryArray, BooleanArray, Float64Array, Int64Array, StringArray, StructArray, UInt8Array,
    };
    use arrow::buffer::{BooleanBuffer, NullBuffer};
    use arrow::datatypes::{DataType, Field, Fields};
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
    use crate::pipeline::partition::{AnyValueStructComparator, PartitionValue, Partitioner};

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
        let partitions = partitioner
            .partition(otap)
            .unwrap()
            .into_iter()
            .collect::<Vec<_>>();

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
        let partitions = partitioner
            .partition(otap)
            .unwrap()
            .into_iter()
            .collect::<Vec<_>>();

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
        let partitions = partitioner
            .partition(otap)
            .unwrap()
            .into_iter()
            .collect::<Vec<_>>();
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

        let comparator = AnyValueStructComparator::try_new(&anyval_struct).unwrap();

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

        let comparator = AnyValueStructComparator::try_new(&anyval_struct).unwrap();

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

        let comparator = AnyValueStructComparator::try_new(&anyval_struct).unwrap();
        assert!(comparator.cmp(0, 1).unwrap().is_eq());
        assert!(comparator.cmp(1, 2).unwrap().is_ne());
        assert!(comparator.cmp(2, 3).unwrap().is_ne());
    }

    #[test]
    fn test_anyval_compartor_bytes() {
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

        let comparator = AnyValueStructComparator::try_new(&anyval_struct).unwrap();
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

        let comparator = AnyValueStructComparator::try_new(&anyval_struct).unwrap();
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

        let comparator = AnyValueStructComparator::try_new(&anyval_struct).unwrap();
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
        let comparator = AnyValueStructComparator::try_new(&anyval_struct).unwrap();
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
        let comparator = AnyValueStructComparator::try_new(&anyval_struct).unwrap();
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

        let comparator = AnyValueStructComparator::try_new(&anyval_struct).unwrap();
        assert!(comparator.cmp(0, 1).unwrap().is_eq());
        assert!(comparator.cmp(1, 2).unwrap().is_ne());
        assert!(comparator.cmp(2, 3).unwrap().is_eq());
        assert!(comparator.cmp(3, 4).unwrap().is_ne());
    }
}
