// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Utilities for splitting based on the result of some expression.
//!
//! For example, we may wish to split an OTAP batch on some some attribute/resource attribute,
//! or some computed value, like
//! `sha256(concat(resource.attributes["k8s.namespace.name"], resource.attributes["service.name"]`)`
//!

use std::cmp::Ordering;
use std::ops::Range;

use arrow::array::{Array, ArrayRef, BooleanArray, StructArray, UInt8Array, make_comparator};
use arrow::buffer::{BooleanBuffer, MutableBuffer};
use arrow::compute::SortOptions;
use arrow::datatypes::DataType;
use arrow::util::bit_util;
use datafusion::execution::context::SessionContext;
use datafusion::logical_expr::ColumnarValue;
use datafusion::scalar::ScalarValue;
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::otap::filter::{IdBitmapPool, filter_otap_batch};
use otap_df_pdata::otlp::attributes::AttributeValueType;
use otap_df_pdata::schema::consts;
use otap_df_query_engine_languages::opl::parser::OplParser;

use crate::error::Result;
use crate::parser::default_parser_options;
use crate::pipeline::Pipeline;
use crate::pipeline::expr::ScopedExpr;
use crate::pipeline::expr::eval::align_value_to_root;
use crate::pipeline::expr::planner::ExprPlanner;
use crate::pipeline::project::anyval::is_any_value_data_type;

/// Computed value of partition for partitioned OTAP Data
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
                    // safety: we've checked the type is struct array
                    let anyvalue_arr = s.as_any().downcast_ref::<StructArray>().expect("is struct array");
                    return Self::try_from_anyvalue_struct_arr(anyvalue_arr, 0)
                }

                todo!("handle invalid struct")
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
                todo!("invalid")
            }
        })
    }

    fn try_from_array_value(arr: &dyn Array, index: usize) -> Result<Self> {
        if is_any_value_data_type(arr.data_type()) {
            // safety: we've checked the type is struct array
            let anyvalue_arr = arr.as_any().downcast_ref::<StructArray>().expect("is struct array");
            return Self::try_from_anyvalue_struct_arr(anyvalue_arr, index)
        }

        let scalar_at_index = ScalarValue::try_from_array(arr, index)?;
        Self::try_from_scalar(scalar_at_index)
    }

    fn try_from_anyvalue_struct_arr(arr: &StructArray, index: usize) -> Result<Self> {
        let Some(type_col) = arr.column_by_name(consts::ATTRIBUTE_TYPE) else {
            todo!("invalid struct col")
        };

        let Some(type_col) = type_col.as_any().downcast_ref::<UInt8Array>() else {
            todo!("invalid struct col");
        };

        let type_encoded = type_col.values()[index];
        let Ok(attr_type) = AttributeValueType::try_from(type_encoded) else {
            todo!("invalid type")   
        };

        let values_col = match attr_type {
            AttributeValueType::Bool => arr.column_by_name(consts::ATTRIBUTE_BOOL),
            AttributeValueType::Bytes => arr.column_by_name(consts::ATTRIBUTE_BYTES),
            AttributeValueType::Double => arr.column_by_name(consts::ATTRIBUTE_DOUBLE),
            AttributeValueType::Int => arr.column_by_name(consts::ATTRIBUTE_INT),
            AttributeValueType::Map | AttributeValueType::Slice => arr.column_by_name(consts::ATTRIBUTE_SER),
            AttributeValueType::Str => arr.column_by_name(consts::ATTRIBUTE_STR),
            AttributeValueType::Empty => {
                return Ok(Self::Null)
            }
        };

        match values_col {
            Some(col) => {
                Self::try_from_array_value(col.as_ref(), index)
            },
            None => {
                Ok(Self::Null)
            }
        }
    }
}

/// Partitioned OTAP batch
///
/// All records in the batch have the same value computed from the partition expression.
pub struct Partition {
    /// value of partition
    value: PartitionValue,

    /// telemetry data in partition
    batch: OtapArrowRecords,
}

impl Partition {
    fn new(value: PartitionValue, batch: OtapArrowRecords) -> Self {
        Self { value, batch }
    }
}

/// Produces partitioned record batches by the results of some evaluated expression
pub struct Partitioner {
    expr: ScopedExpr,

    session_ctx: SessionContext,

    range_coalescer: PartitionRangeCoalescer,

    id_bitmap_pool: IdBitmapPool,

    /// reusable buffer of partitions
    partitions: Vec<Partition>,
}

impl Partitioner {
    /// TODO docs
    /// TODO - this is possibly the wrong abstraction due to this having the crate depend on
    /// the parser, consider if maybe we should have this specific constructor take
    /// ScalarExpression and Functions, but move the constructor (or this type) to something
    /// higher level
    pub fn try_new_from_opl_expression(expr: &str) -> Result<Self> {
        let (scalar_expr, functions) =
            OplParser::parse_expr_with_options(expr, default_parser_options()).unwrap(); // TODO no unwrap

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
}

impl Partitioner {
    /// TODO docs
    pub fn partition(
        &mut self,
        otap_batch: OtapArrowRecords,
    ) -> Result<impl IntoIterator<Item = Partition>> {
        self.partitions.clear();

        partition(
            otap_batch,
            &self.session_ctx,
            &mut self.expr,
            &mut self.partitions,
            &mut self.range_coalescer,
            &mut self.id_bitmap_pool,
        )?;

        Ok(self.partitions.drain(..))
    }
}

/// Partition OTAP batch by the value of the evaluated expression
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
                // TODO need to partition any values
                todo!()
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

fn partition_simple_array(
    array: ArrayRef,
    otap_batch: OtapArrowRecords,
    result: &mut Vec<Partition>,
    range_coalescer: &mut PartitionRangeCoalescer,
    id_bitmap_pool: &mut IdBitmapPool,
) -> Result<()> {
    let num_rows = array.len();
    let cmp = make_comparator(&array, &array, SortOptions::default())?;
    let boundaries: BooleanBuffer = (0..array.len() - 1)
        .map(|i| !cmp(i, i + 1).is_eq())
        .collect();

    for group in range_coalescer
        .coalesce_groups(num_rows, boundaries, &cmp)
        .into_iter()
    {
        let selection_vec = BooleanArray::from(BooleanBuffer::new(
            group.selection_vec_builder.into(),
            0,
            num_rows,
        ));
        let filtered = filter_otap_batch(&selection_vec, &otap_batch, id_bitmap_pool)?;
        let partition_value =
            PartitionValue::try_from_array_value(array.as_ref(), group.representative_row)?;
        result.push(Partition::new(partition_value, filtered));
    }

    Ok(())
}

struct PartitionRangeCoalescer {
    groups: Vec<CoalescingGroup>,
}

impl PartitionRangeCoalescer {
    fn new() -> Self {
        Self { groups: Vec::new() }
    }

    fn coalesce_groups(
        &mut self,
        source_len: usize,
        partition_boundaries: BooleanBuffer,
        cmp: &dyn Fn(usize, usize) -> Ordering,
    ) -> impl IntoIterator<Item = CoalescingGroup> {
        coalesce_groups(source_len, partition_boundaries, cmp, &mut self.groups);

        self.groups.drain(..)
    }
}

struct CoalescingGroup {
    representative_row: usize,
    selection_vec_builder: MutableBuffer,
}

fn coalesce_groups(
    source_len: usize,
    partition_boundaries: BooleanBuffer,
    cmp: &dyn Fn(usize, usize) -> Ordering,
    groups: &mut Vec<CoalescingGroup>,
) {
    let mut current = 0;
    for idx in partition_boundaries.set_indices() {
        let t = current;
        current = idx + 1;
        append_range_to_groups(source_len, t..current, cmp, groups);
    }

    let last = partition_boundaries.len() + 1;
    if current != last {
        append_range_to_groups(source_len, current..last, cmp, groups);
    }
}

fn append_range_to_groups(
    source_len: usize,
    range: Range<usize>,
    cmp: &dyn Fn(usize, usize) -> Ordering,
    groups: &mut Vec<CoalescingGroup>,
) {
    let match_group = groups
        .iter_mut()
        .find(|group| cmp(group.representative_row, range.start).is_eq());

    if let Some(group) = match_group {
        set_range_bits(range, &mut group.selection_vec_builder);
    } else {
        let mut group = CoalescingGroup {
            representative_row: range.start,
            selection_vec_builder: MutableBuffer::from_len_zeroed(source_len),
        };
        set_range_bits(range, &mut group.selection_vec_builder);
        groups.push(group)
    }
}

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
    for i in aligned_start_index..range.end {
        bit_util::set_bit(bool_buffer, i);
    }
}

#[cfg(test)]
mod test {
    use datafusion::logical_expr::col;
    use otap_df_pdata::otap::filter::IdBitmapPool;
    use otap_df_pdata::proto::OtlpProtoMessage;
    use otap_df_pdata::proto::opentelemetry::common::v1::InstrumentationScope;
    use otap_df_pdata::proto::opentelemetry::logs::v1::{
        LogRecord, LogsData, ResourceLogs, ScopeLogs,
    };
    use otap_df_pdata::proto::opentelemetry::resource::v1::Resource;
    use otap_df_pdata::schema::consts;
    use otap_df_pdata::testing::round_trip::{otap_to_otlp, otlp_to_otap};

    use crate::pipeline::Pipeline;
    use crate::pipeline::expr::{DataScope, LeafEval, ScopedExpr};
    use crate::pipeline::partition::PartitionRangeCoalescer;

    use super::partition;

    // TODO the tests here also need to assert on the partition values.

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
        let session_ctx = Pipeline::create_session_context();

        let mut expr = ScopedExpr::Eval {
            scope: DataScope::Root,
            eval: LeafEval::new_df_expr(col(consts::SEVERITY_NUMBER), false).unwrap(),
        };

        let mut partitions = Vec::new();
        partition(
            otap,
            &session_ctx,
            &mut expr,
            &mut partitions,
            &mut PartitionRangeCoalescer::new(),
            &mut IdBitmapPool::new(),
        )
        .unwrap();

        assert_eq!(partitions.len(), 2, "expected 2 partitions");

        // Collect (severity_number, event_names) for each partition by round-tripping to OTLP.
        let mut partition_summaries: Vec<(i32, Vec<String>)> = Vec::new();
        for p in &partitions {
            let root_rb = p
                .batch
                .root_record_batch()
                .expect("partition should have root batch");
            assert!(root_rb.num_rows() == 2, "each partition should have 2 rows");

            let OtlpProtoMessage::Logs(logs_data) = otap_to_otlp(&p.batch) else {
                panic!("expected logs");
            };
            let records: Vec<_> = logs_data
                .resource_logs
                .iter()
                .flat_map(|rl| &rl.scope_logs)
                .flat_map(|sl| &sl.log_records)
                .collect();

            let severity = records[0].severity_number;
            let event_names: Vec<String> = records.iter().map(|r| r.event_name.clone()).collect();

            // All records in a partition should share the same severity_number.
            for r in &records {
                assert_eq!(r.severity_number, severity);
            }

            partition_summaries.push((severity, event_names));
        }

        // Sort by severity so assertion order is deterministic.
        partition_summaries.sort_by_key(|(sev, _)| *sev);

        assert_eq!(partition_summaries[0].0, 13);
        assert_eq!(partition_summaries[0].1, vec!["e1", "e3"]);
        assert_eq!(partition_summaries[1].0, 17);
        assert_eq!(partition_summaries[1].1, vec!["e2", "e4"]);
    }

    /// Partition traces by span `name`.
    ///
    /// Input: 4 spans with names ["op-a", "op-b", "op-a", "op-b"].
    /// Expected: 2 partitions -- one with the two "op-a" spans, one with the two "op-b" spans.
    /// This tests partitioning on a string column with non-adjacent equal values in traces.
    #[test]
    fn test_partition_traces_by_span_name() {
        use otap_df_pdata::proto::opentelemetry::trace::v1::{
            ResourceSpans, ScopeSpans, Span, TracesData,
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
        let session_ctx = Pipeline::create_session_context();

        let mut expr = ScopedExpr::Eval {
            scope: DataScope::Root,
            eval: LeafEval::new_df_expr(col(consts::NAME), false).unwrap(),
        };

        let mut partitions = Vec::new();
        partition(
            otap,
            &session_ctx,
            &mut expr,
            &mut partitions,
            &mut PartitionRangeCoalescer::new(),
            &mut IdBitmapPool::new(),
        )
        .unwrap();

        assert_eq!(partitions.len(), 2, "expected 2 partitions");

        // Collect (span_name, span_ids) for each partition by round-tripping to OTLP.
        let mut partition_summaries: Vec<(String, Vec<Vec<u8>>)> = Vec::new();
        for p in &partitions {
            let root_rb = p
                .batch
                .root_record_batch()
                .expect("partition should have root batch");
            assert_eq!(root_rb.num_rows(), 2, "each partition should have 2 rows");

            let OtlpProtoMessage::Traces(traces_data) = otap_to_otlp(&p.batch) else {
                panic!("expected traces");
            };
            let spans: Vec<_> = traces_data
                .resource_spans
                .iter()
                .flat_map(|rs| &rs.scope_spans)
                .flat_map(|ss| &ss.spans)
                .collect();

            let name = spans[0].name.clone();
            let span_ids: Vec<Vec<u8>> = spans.iter().map(|s| s.span_id.clone()).collect();

            // All spans in a partition should share the same name.
            for s in &spans {
                assert_eq!(s.name, name);
            }

            partition_summaries.push((name, span_ids));
        }

        // Sort by name so assertion order is deterministic.
        partition_summaries.sort_by(|(a, _), (b, _)| a.cmp(b));

        assert_eq!(partition_summaries[0].0, "op-a");
        assert_eq!(partition_summaries[0].1, vec![vec![1u8; 8], vec![3u8; 8]]);
        assert_eq!(partition_summaries[1].0, "op-b");
        assert_eq!(partition_summaries[1].1, vec![vec![2u8; 8], vec![4u8; 8]]);
    }
}
