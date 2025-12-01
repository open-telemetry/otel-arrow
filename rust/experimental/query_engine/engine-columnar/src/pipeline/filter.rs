// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;
use std::sync::Arc;

use arrow::array::{
    Array, BooleanArray, BooleanBufferBuilder, RecordBatch, StructArray, UInt16Array,
};
use arrow::buffer::BooleanBuffer;
use arrow::compute::{and, filter_record_batch, not, or};
use async_trait::async_trait;
use data_engine_expressions::{LogicalExpression, ScalarExpression};
use datafusion::common::DFSchema;
use datafusion::common::cast::as_boolean_array;
use datafusion::config::ConfigOptions;
use datafusion::execution::TaskContext;
use datafusion::execution::context::SessionContext;
use datafusion::functions::core::expr_ext::FieldAccessor;
use datafusion::logical_expr::{BinaryExpr, Expr, Operator, col, lit};
use datafusion::physical_expr::{PhysicalExprRef, create_physical_expr};
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::otap::filter::build_uint16_id_filter;
use otap_df_pdata::otap::{Logs, Metrics, Traces};
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otap_df_pdata::schema::consts;
use roaring::RoaringBitmap;

use crate::error::{Error, Result};
use crate::pipeline::PipelineStage;
use crate::pipeline::planner::{
    AttributesIdentifier, BinaryArg, ColumnAccessor, try_attrs_value_filter_from_literal,
    try_static_scalar_to_literal,
};

/// A compositional tree structure for combining expressions with boolean operators.
///
/// Represents logical combinations of base values using Not, And, and Or operations,
/// forming a tree that can be evaluated or transformed.
pub enum Composite<T> {
    Base(T),
    Not(Box<Self>),
    And(LeftRight<Box<Self>>),
    Or(LeftRight<Box<Self>>),
}

/// A pair of values representing left and right operands of a binary operation.
type LeftRight<T> = (T, T);

/// A helper trait that can be used to transform a composite logical plan into a composite
/// executable plan.
pub trait ToExec {
    type ExecutablePlan;

    fn to_exec(
        &self,
        // TODO this might not be needed in this trait anymore
        session_ctx: &SessionContext,
        otap_batch: &OtapArrowRecords,
    ) -> Result<Self::ExecutablePlan>;
}

impl<T, B> Composite<T>
where
    T: ToExec<ExecutablePlan = B>,
{
    /// transform the composite logical plan into a composite executable plan by calling `to_exec`
    /// on the base type
    pub fn to_exec(
        &self,
        session_ctx: &SessionContext,
        otap_batch: &OtapArrowRecords,
    ) -> Result<Composite<B>> {
        match self {
            Self::Base(inner) => {
                let exec = inner.to_exec(session_ctx, otap_batch)?;
                Ok(Composite::Base(exec))
            }
            Self::Not(inner) => {
                let exec = inner.to_exec(session_ctx, otap_batch)?;
                Ok(Composite::Not(Box::new(exec)))
            }
            Self::And((left, right)) => {
                let left_exec = left.to_exec(session_ctx, otap_batch)?;
                let right_exec = right.to_exec(session_ctx, otap_batch)?;
                Ok(Composite::And((Box::new(left_exec), Box::new(right_exec))))
            }
            Self::Or((left, right)) => {
                let left_exec = left.to_exec(session_ctx, otap_batch)?;
                let right_exec = right.to_exec(session_ctx, otap_batch)?;
                Ok(Composite::Or((Box::new(left_exec), Box::new(right_exec))))
            }
        }
    }
}

/// helper for creating the Composite type from the base type
impl<T> From<T> for Composite<T> {
    fn from(value: T) -> Self {
        Self::Base(value)
    }
}

/// A logical plan for filtering data across root batch's columns and attributes.
///
/// Supports two types of filters that can be applied independently or together:
/// - `source_filter`: Filters on regular columns in the source data
/// - `attribute_filter`: Filters on key-value attribute pairs
///
/// When both types of filters are present, the resulting execution of the plan will be the
/// intersection of the two filters.
///
/// Can be constructed from either a DataFusion `Expr` (for root batch's filters) or an
/// `AttributesFilterPlan` (for attribute filters), and can be composed into boolean expressions
/// using `Composite`.
pub struct FilterPlan {
    /// filters that will be applied to the root record batch
    source_filter: Option<Expr>,

    /// filters that will be applied to the attributes record batch in order fo filter the
    /// rows of the root batch
    attribute_filter: Option<Composite<AttributesFilterPlan>>,
}

impl From<Expr> for FilterPlan {
    fn from(expr: Expr) -> Self {
        Self {
            source_filter: Some(expr),
            attribute_filter: None,
        }
    }
}

impl From<AttributesFilterPlan> for FilterPlan {
    fn from(attrs_filter: AttributesFilterPlan) -> Self {
        Self {
            source_filter: None,
            attribute_filter: Some(attrs_filter.into()),
        }
    }
}

impl FilterPlan {
    /// Try to create a [`FilterPlan`] representing a comparison between the left scalar expression
    /// and right scalar expression using the given binary_op for comparison.
    ///
    /// The scalar expression on either side can represent:
    /// - an attribute (e.g. attributes["x"], resource.attributes["x"], etc.)
    /// - a column (e.g. severity_text, event_name, etc.)
    /// - a column nested within a struct (e.g. resource.schema_url, instrumentation_scope.name, etc.)
    /// - a literal (e.g. "a", 1234, true, etc.)
    ///
    fn try_from_binary_expr(
        left_expr: &ScalarExpression,
        binary_op: Operator,
        right_expr: &ScalarExpression,
    ) -> Result<Self> {
        let left_arg = BinaryArg::try_from(left_expr)?;
        let right_arg = BinaryArg::try_from(right_expr)?;

        // TODO there are several branches below which are not yet supported supported
        // - comparing two literals. e.g "a" == "b"
        // - comparing non-literal left with non-literal right. e.g.
        //   - severity_text == event_name
        //   - attributes["x"] == severity_text
        //   - etc.

        match left_arg {
            BinaryArg::Column(left_column) => match left_column {
                ColumnAccessor::ColumnName(left_col_name) => match right_arg {
                    BinaryArg::Literal(right_lit) => {
                        // left = column & right = literal
                        Ok(FilterPlan::from(Expr::BinaryExpr(BinaryExpr::new(
                            Box::new(col(left_col_name)),
                            binary_op,
                            Box::new(try_static_scalar_to_literal(&right_lit)?),
                        ))))
                    }
                    _ => Err(Error::NotYetSupportedError {
                        message: "comparing left column with non-literal right in filter.".into(),
                    }),
                },
                ColumnAccessor::StructCol(left_struct_name, left_struct_field) => match right_arg {
                    BinaryArg::Literal(right_lit) => {
                        // left = struct col & right = literal
                        Ok(FilterPlan::from(Expr::BinaryExpr(BinaryExpr::new(
                            Box::new(col(left_struct_name).field(left_struct_field)),
                            binary_op,
                            Box::new(try_static_scalar_to_literal(&right_lit)?),
                        ))))
                    }
                    _ => Err(Error::NotYetSupportedError {
                        message: "comparing left struct column with non-literal right in filter"
                            .into(),
                    }),
                },
                ColumnAccessor::Attributes(attrs_identifier, attrs_key) => {
                    match right_arg {
                        // left = attribute & right = literal
                        BinaryArg::Literal(right_lit) => {
                            Ok(FilterPlan::from(AttributesFilterPlan {
                                attrs_identifier,
                                filter: col(consts::ATTRIBUTE_KEY).eq(lit(attrs_key)).and(
                                    Expr::BinaryExpr(try_attrs_value_filter_from_literal(
                                        &right_lit, binary_op,
                                    )?),
                                ),
                            }))
                        }
                        _ => Err(Error::NotYetSupportedError {
                            message: "comparing left attribute with non-literal right in filter"
                                .into(),
                        }),
                    }
                }
            },
            BinaryArg::Literal(left_lit) => match right_arg {
                BinaryArg::Literal(_right_lit) => Err(Error::NotYetSupportedError {
                    message: "comparing literals in filter".into(),
                }),
                BinaryArg::Column(right_column) => match right_column {
                    ColumnAccessor::ColumnName(right_col_name) => {
                        // left = literal & right = column
                        Ok(FilterPlan::from(Expr::BinaryExpr(BinaryExpr::new(
                            Box::new(try_static_scalar_to_literal(&left_lit)?),
                            binary_op,
                            Box::new(col(right_col_name)),
                        ))))
                    }
                    ColumnAccessor::StructCol(right_struct_name, right_struct_field) => {
                        // left = literal & right = struct col
                        Ok(FilterPlan::from(Expr::BinaryExpr(BinaryExpr::new(
                            Box::new(try_static_scalar_to_literal(&left_lit)?),
                            binary_op,
                            Box::new(col(right_struct_name).field(right_struct_field)),
                        ))))
                    }
                    ColumnAccessor::Attributes(attrs_identifier, attrs_key) => {
                        // left = literal & right = attribute
                        Ok(FilterPlan::from(AttributesFilterPlan {
                            attrs_identifier,
                            filter: col(consts::ATTRIBUTE_KEY).eq(lit(attrs_key)).and(
                                Expr::BinaryExpr(try_attrs_value_filter_from_literal(
                                    &left_lit, binary_op,
                                )?),
                            ),
                        }))
                    }
                },
            },
        }
    }
}

impl TryFrom<&LogicalExpression> for Composite<FilterPlan> {
    type Error = Error;

    fn try_from(logical_expr: &LogicalExpression) -> Result<Self> {
        match logical_expr {
            LogicalExpression::EqualTo(equals_to_expr) => FilterPlan::try_from_binary_expr(
                equals_to_expr.get_left(),
                Operator::Eq,
                equals_to_expr.get_right(),
            )
            .map(|plan| plan.into()),
            LogicalExpression::GreaterThan(gt_expr) => FilterPlan::try_from_binary_expr(
                gt_expr.get_left(),
                Operator::Gt,
                gt_expr.get_right(),
            )
            .map(|plan| plan.into()),
            LogicalExpression::GreaterThanOrEqualTo(geq_expr) => FilterPlan::try_from_binary_expr(
                geq_expr.get_left(),
                Operator::GtEq,
                geq_expr.get_right(),
            )
            .map(|plan| plan.into()),
            LogicalExpression::And(and_expr) => {
                let left = Self::try_from(and_expr.get_left())?;
                let right = Self::try_from(and_expr.get_right())?;
                Ok(Self::And((Box::new(left), Box::new(right))))
            }
            LogicalExpression::Or(or_expr) => {
                let left = Self::try_from(or_expr.get_left())?;
                let right = Self::try_from(or_expr.get_right())?;
                Ok(Self::Or((Box::new(left), Box::new(right))))
            }
            LogicalExpression::Not(not_expr) => {
                let inner = Self::try_from(not_expr.get_inner_expression())?;
                Ok(Self::Not(Box::new(inner)))
            }

            // TODO add support for these expressions eventually
            LogicalExpression::Matches(_)
            | LogicalExpression::Contains(_)
            | LogicalExpression::Scalar(_) => Err(Error::NotYetSupportedError {
                message: format!("Logical expression not yet supported {logical_expr:?}"),
            }),
        }
    }
}

impl ToExec for FilterPlan {
    type ExecutablePlan = FilterExec;
    fn to_exec(
        &self,
        session_ctx: &SessionContext,
        otap_batch: &OtapArrowRecords,
    ) -> Result<Self::ExecutablePlan> {
        let physical_expr = self
            .source_filter
            .as_ref()
            .map(|expr| AdaptivePhysicalExprExec::new(expr.clone()));

        let attrs_filter = self
            .attribute_filter
            .as_ref()
            .map(|attr_filter| attr_filter.to_exec(session_ctx, otap_batch))
            .transpose()?;

        Ok(FilterExec {
            predicate: physical_expr,
            attributes_filter: attrs_filter,
        })
    }
}

/// A logical plan for filtering attributes.
#[derive(Clone)]
struct AttributesFilterPlan {
    /// The filtering expression that  will be applied to the attributes
    ///
    /// Note that the expression should be constructed based on the otap data-model, not using some
    /// abstract notion of an attribute column. This means, say we're filtering for
    /// `attributes["x"] == "y"` that we'd expect a filter expr like
    /// `col("key").eq(lit("x")).and(col("str").eq(lit("y")))`
    filter: Expr,

    /// The identifier of which attributes will be considered when filtering
    attrs_identifier: AttributesIdentifier,
}

impl ToExec for AttributesFilterPlan {
    type ExecutablePlan = AttributeFilterExec;

    fn to_exec(
        &self,
        _session_ctx: &SessionContext,
        otap_batch: &OtapArrowRecords,
    ) -> Result<Self::ExecutablePlan> {
        let attrs_payload_type = match self.attrs_identifier {
            AttributesIdentifier::Root => match otap_batch.root_payload_type() {
                ArrowPayloadType::Logs => ArrowPayloadType::LogAttrs,
                ArrowPayloadType::Spans => ArrowPayloadType::SpanAttrs,
                _ => ArrowPayloadType::MetricAttrs,
            },
            AttributesIdentifier::NonRoot(payload_type) => payload_type,
        };

        Ok(AttributeFilterExec {
            filter: AdaptivePhysicalExprExec::new(self.filter.clone()),
            payload_type: attrs_payload_type,
        })
    }
}

fn to_physical_exprs(
    expr: &Expr,
    record_batch: &RecordBatch,
    session_ctx: &SessionContext,
) -> Result<PhysicalExprRef> {
    let df_schema = DFSchema::from_unqualified_fields(
        record_batch.schema().fields.clone(),
        Default::default(),
    )?;
    let physical_expr =
        create_physical_expr(expr, &df_schema, session_ctx.state().execution_props())?;

    Ok(physical_expr)
}

pub struct FilterExec {
    predicate: Option<AdaptivePhysicalExprExec>,
    attributes_filter: Option<Composite<AttributeFilterExec>>,
}

impl FilterExec {
    /// execute the filter expression. returns a selection vector for the passed record batch
    fn execute(
        &mut self,
        otap_batch: &OtapArrowRecords,
        session_ctx: &SessionContext,
    ) -> Result<BooleanArray> {
        let root_rb = match otap_batch.root_record_batch() {
            Some(rb) => rb,
            None => {
                // The caller should be responsible for checking if the batch is empty (e.g. if
                // there's no root batch), which means nothing could be filtered out so it should
                // not be calling this method. If we encounter this, return an error
                return Err(Error::ExecutionError {
                    cause: "root batch not present in execution".into(),
                });
            }
        };

        let mut selection_vec = match self.predicate.as_mut() {
            Some(predicate) => predicate.evaluate_filter(root_rb, session_ctx)?,

            // TODO -- we might be able to optimize this method by not allocating this here
            // and instead returning None
            None => BooleanArray::new(BooleanBuffer::new_set(root_rb.num_rows()), None),
        };

        if let Some(attrs_filter) = &mut self.attributes_filter {
            let id_mask = attrs_filter.execute(otap_batch, session_ctx, false)?;

            let id_col =
                get_id_col_from_parent(root_rb, attrs_filter.payload_type())?.ok_or_else(|| {
                    // TODO we should handle if the ID column is missing from the root record batch.
                    // This would happen if there were no attributes. If someone tries to filter if
                    // there are no attributes, the correct behaviour is to treat every record as if
                    // it was null. See the comment below about null handling ...
                    Error::ExecutionError {
                        cause: "ID column not found on root record batch".into(),
                    }
                })?;

            let mut attrs_selection_vec_builder = BooleanBufferBuilder::new(root_rb.num_rows());

            // we append to the selection vector in contiguous segments rather than doing it 1-by-1
            // for each value, as this is a faster way to build up the BooleanBuffer
            let mut segment_validity = false;
            let mut segment_len = 0usize;

            for index in 0..id_col.len() {
                let row_validity = if id_col.is_valid(index) {
                    id_mask.contains(id_col.value(index) as u32)
                } else {
                    // TODO -- here the record has no attributes, so the row should be filtered out
                    // in all cases except for the attribute predicate was something like
                    // `attributes["x"] == null`.
                    //
                    // Generally speaking, we need to eventually add a way to inspect the attribute
                    // filter's predicate to determine if the attribute not being present means
                    // passes. But b/c we don't currently support any predicates like this, we just
                    // assume no attr means the row gets filtered out
                    //
                    false
                };

                if segment_validity != row_validity {
                    if segment_len > 0 {
                        attrs_selection_vec_builder.append_n(segment_len, segment_validity);
                    }
                    segment_validity = row_validity;
                    segment_len = 0;
                }

                segment_len += 1;
            }

            // append the last segment
            if segment_len > 0 {
                attrs_selection_vec_builder.append_n(segment_len, segment_validity);
            }

            // update the result selection_vec to be the intersection of what's already filtered
            // and the attributes filters
            selection_vec = and(
                &selection_vec,
                &BooleanArray::new(attrs_selection_vec_builder.finish(), None),
            )?;
        }

        Ok(selection_vec)
    }
}

impl Composite<FilterExec> {
    fn execute(
        &mut self,
        otap_batch: &OtapArrowRecords,
        session_ctx: &SessionContext,
    ) -> Result<BooleanArray> {
        match self {
            Self::Base(filter) => filter.execute(otap_batch, session_ctx),
            Self::Not(filter) => Ok(not(&filter.execute(otap_batch, session_ctx)?)?),
            Self::And((left, right)) => {
                let left_result = left.execute(otap_batch, session_ctx)?;
                let right_result = right.execute(otap_batch, session_ctx)?;
                Ok(and(&left_result, &right_result)?)
            }
            Self::Or((left, right)) => {
                let left_result = left.execute(otap_batch, session_ctx)?;
                let right_result = right.execute(otap_batch, session_ctx)?;
                Ok(or(&left_result, &right_result)?)
            }
        }
    }
}

struct AttributeFilterExec {
    filter: AdaptivePhysicalExprExec,
    payload_type: ArrowPayloadType,
}

impl AttributeFilterExec {
    /// execute the filter on the attributes. This returns a bitmap of parent_ids that were
    /// selected by the filter. Conversely if invert = `true` it creates a bitmap of parent_ids
    /// not selected by the filter.
    fn execute(
        &mut self,
        otap_batch: &OtapArrowRecords,
        session_ctx: &SessionContext,
        inverted: bool,
    ) -> Result<RoaringBitmap> {
        let record_batch = match otap_batch.get(self.payload_type) {
            Some(rb) => rb,
            None => {
                // if there are no attributes, then nothing can match the filter so just return
                // empty ID mask
                return Ok(RoaringBitmap::new());
            }
        };

        let selection_vec = self.filter.evaluate_filter(record_batch, session_ctx)?;
        let parent_id_col = get_parent_id_column(record_batch)?;

        // create a bitmap containing the parent_ids that passed the filter predicate
        let id_mask = parent_id_col
            .iter()
            .enumerate()
            .filter_map(|(index, parent_id)| {
                selection_vec
                    .value(index)
                    .then(|| {
                        // the parent_id column _should_ be non-nullable, so we could maybe call
                        // `expect` here, but `map` is probably safer just in case there is a null
                        // for some unexpected reason
                        parent_id.map(|i| i as u32)
                    })
                    .flatten()
            })
            .collect();

        if !inverted {
            return Ok(id_mask);
        }

        // create an id_mask that is an inversion of the parent_ids selected by the filter
        let id_mask = parent_id_col
            .iter()
            .flatten()
            .map(|parent_id| parent_id as u32)
            .filter(|parent_id| !id_mask.contains(*parent_id))
            .collect();

        Ok(id_mask)
    }
}

impl Composite<AttributeFilterExec> {
    /// Executes the base filter, and combines the the parent_id to using the logical expression
    /// defined by the composite tree. The reason we do here, instead of say combining everything
    /// in `Composite<FilterExec>`, is that this saves us from doing additional conversions between
    /// the parent_id bitmap to a selection vector for the parent record batch.
    fn execute(
        &mut self,
        otap_batch: &OtapArrowRecords,
        session_ctx: &SessionContext,
        inverted: bool,
    ) -> Result<RoaringBitmap> {
        match self {
            Self::Base(filter) => filter.execute(otap_batch, session_ctx, inverted),
            Self::Not(filter) => filter.execute(otap_batch, session_ctx, !inverted),
            Self::And((left, right)) => {
                let left_result = left.execute(otap_batch, session_ctx, inverted)?;
                let right_result = right.execute(otap_batch, session_ctx, inverted)?;
                Ok(if inverted {
                    // not (A and B) = (not A) or (not B)
                    left_result | right_result
                } else {
                    left_result & right_result
                })
            }
            Self::Or((left, right)) => {
                let left_result = left.execute(otap_batch, session_ctx, inverted)?;
                let right_result = right.execute(otap_batch, session_ctx, inverted)?;
                Ok(if inverted {
                    // not (A or B) = (not A) and (not B)
                    left_result & right_result
                } else {
                    left_result | right_result
                })
            }
        }
    }

    fn payload_type(&self) -> ArrowPayloadType {
        match self {
            Self::Base(filter) => filter.payload_type,
            Self::Not(filter) => filter.payload_type(),

            // All children should be for the same payload type, so we just traverse one side
            // of the tree.
            Self::And((left, _)) => left.payload_type(),
            Self::Or((left, _)) => left.payload_type(),
        }
    }
}

/// This is responsible for evaluating a  [`PhysicalExpr`](datafusion::physical_expr::PhysicalExpr)
/// while adapting to "schema" changes that may be encountered between evaluations.
///
/// OTAP data might have minor changes the "schema" between batches.
/// - Optional payload type's [`RecordBatch`]s may change presence
/// - Optional columns within a [`RecordBatch`] may change presence
/// - The type of a column may change between Dict<u8, V>, Dict<16, V>, and the native array type
/// - The order of columns may change
///
struct AdaptivePhysicalExprExec {
    /// The physical expression that should be evaluated for each batch. This is initialized lazily
    /// when the first non-`None` batch is passed to `evaluate`
    physical_expr: Option<PhysicalExprRef>,

    /// The original logical plan used to produce the [`PhysicalExpr`]
    logical_expr: Expr,
}

impl AdaptivePhysicalExprExec {
    fn new(logical_expr: Expr) -> Self {
        Self {
            physical_expr: None,
            logical_expr,
        }
    }

    fn evaluate_filter(
        &mut self,
        record_batch: &RecordBatch,
        session_ctx: &SessionContext,
    ) -> Result<BooleanArray> {
        // lazily initialize the physical expr if not already initialized
        if self.physical_expr.is_none() {
            let physical_expr = to_physical_exprs(&self.logical_expr, record_batch, session_ctx)?;
            self.physical_expr = Some(physical_expr);
        }

        // safety: this is already initialized
        let predicate = self.physical_expr.as_ref().expect("initialized");

        // evaluate the predicate
        let arr = predicate
            .evaluate(record_batch)?
            .into_array(record_batch.num_rows())?;

        // ensure it actually evaluated to a boolean expression, and if so return that as selection vec
        as_boolean_array(&arr)
            .cloned()
            .map_err(|_| Error::ExecutionError {
                cause: format!(
                    "Cannot create selection vector from non-boolean predicates. Found {}",
                    arr.data_type()
                ),
            })
    }
}

/// helper function for getting the ID column associated with the parent_id in the child record
/// batch which is identified by the passed payload type
// TODO - this currently only supports the root record batch. When we support additional signal
// types with more deeply nested tree of payload types, we'll need to correct the logic here
fn get_id_col_from_parent(
    root_rb: &RecordBatch,
    child_payload_type: ArrowPayloadType,
) -> Result<Option<&UInt16Array>> {
    match child_payload_type {
        ArrowPayloadType::ResourceAttrs => root_rb
            .column_by_name(consts::RESOURCE)
            .and_then(|arr| arr.as_any().downcast_ref::<StructArray>())
            .and_then(|arr| arr.column_by_name(consts::ID)),
        ArrowPayloadType::ScopeAttrs => root_rb
            .column_by_name(consts::SCOPE)
            .and_then(|arr| arr.as_any().downcast_ref::<StructArray>())
            .and_then(|arr| arr.column_by_name(consts::ID)),
        _ => root_rb.column_by_name(consts::ID),
    }
    .map(|id_col| {
        id_col
            .as_any()
            .downcast_ref::<UInt16Array>()
            .ok_or_else(|| Error::ExecutionError {
                cause: format!(
                    "unexpected type for ID column. Expected u16 found {}",
                    id_col.data_type()
                ),
            })
    })
    .transpose()
}

fn get_parent_id_column(record_batch: &RecordBatch) -> Result<&UInt16Array> {
    // get the parent ID column
    let parent_id_arr = record_batch
        .column_by_name(consts::PARENT_ID)
        .ok_or_else(|| Error::ExecutionError {
            cause: "parent_id column not found on child batch".into(),
        })?;

    let parent_id_ints = parent_id_arr
        .as_any()
        .downcast_ref::<UInt16Array>()
        .ok_or_else(|| Error::ExecutionError {
            cause: format!(
                "unexpected parent_id type for child record batch. Expected u16 found {:?}",
                parent_id_arr.data_type()
            ),
        })?;

    Ok(parent_id_ints)
}

pub struct FilterPipelineStage {
    filter_exec: Composite<FilterExec>,
}

impl FilterPipelineStage {
    pub fn new(filter_exec: Composite<FilterExec>) -> Self {
        Self { filter_exec }
    }

    /// After filtering has been applied to the parent record batch, go into the child record batch
    /// and remove rows with parent_id pointing to parents that were filtered out
    fn filter_child_batch(
        &self,
        otap_batch: &mut OtapArrowRecords,
        payload_type: ArrowPayloadType,
    ) -> Result<()> {
        let root_rb = match otap_batch.root_record_batch() {
            Some(rb) => rb,
            None => {
                // if the root record batch is missing, it we must have an empty OTAP batch
                // hence nothing to do
                return Ok(());
            }
        };

        let child_rb = match otap_batch.get(payload_type) {
            Some(rb) => rb,
            None => {
                // if child batch doesn't exist, then there are no records to filter
                return Ok(());
            }
        };

        let id_col = get_id_col_from_parent(root_rb, payload_type)?.ok_or_else(||
            // this would be considered an unexpected state for this batch. We have a child
            // record batch that is supposed to have it's parent_id pointing to an ID column
            // on the root batch which does not exist
            Error::ExecutionError {
                cause: format!(
                    "Invalid batch - ID column not found on root batch {:?}",
                    otap_batch.root_payload_type()
                )
            })?;

        // build the selection vector for the child record batch. This uses common code shared
        // with the filter processor
        let id_mask: HashSet<u16> = id_col.iter().flatten().collect();
        let child_parent_ids =
            child_rb
                .column_by_name(consts::PARENT_ID)
                .ok_or_else(|| Error::ExecutionError {
                    cause: "parent_id column not found on child batch".into(),
                })?;

        let child_selection_vec =
            build_uint16_id_filter(child_parent_ids, id_mask).map_err(|e| {
                Error::ExecutionError {
                    cause: format!("error filtering child batch {:?}", e),
                }
            })?;

        // create the new child record batch from rows that were selected and update the OTAP batch
        let new_child_rb = filter_record_batch(child_rb, &child_selection_vec).map_err(|e| {
            Error::ExecutionError {
                cause: format!("error filtering child batch {:?}", e),
            }
        })?;
        otap_batch.set(payload_type, new_child_rb);

        Ok(())
    }
}

#[async_trait]
impl PipelineStage for FilterPipelineStage {
    async fn execute(
        &mut self,
        mut otap_batch: OtapArrowRecords,
        session_context: &SessionContext,
        _config_options: &ConfigOptions,
        _task_context: Arc<TaskContext>,
    ) -> Result<OtapArrowRecords> {
        let root_batch = match otap_batch.root_record_batch() {
            Some(rb) => rb,
            None => {
                // if batch is empty, no filtering to do
                return Ok(otap_batch);
            }
        };

        // TODO -- we need to check here if the schema has changed or if any attribute
        // record batches have gone missing and if so, replan the internal FilterExec

        let selection_vec = self.filter_exec.execute(&otap_batch, session_context)?;

        // check if nothing was filtered
        if selection_vec.true_count() == root_batch.num_rows() {
            // Nothing was filtered out, return original batch
            return Ok(otap_batch);
        }

        // check if the filter removed all records
        if selection_vec.false_count() == root_batch.num_rows() {
            // here we return an empty OTAP batch with the same signal type
            return Ok(match otap_batch.root_payload_type() {
                ArrowPayloadType::Logs => OtapArrowRecords::Logs(Logs::default()),
                ArrowPayloadType::Spans => OtapArrowRecords::Traces(Traces::default()),
                _ => OtapArrowRecords::Metrics(Metrics::default()),
            });
        }

        // take the rows from the root batch that were selected
        let new_root_batch = filter_record_batch(root_batch, &selection_vec)?;

        // replace the root batch
        otap_batch.set(otap_batch.root_payload_type(), new_root_batch);

        // update the child batches after filtering has been applied to parent
        match otap_batch.root_payload_type() {
            ArrowPayloadType::Logs => {
                self.filter_child_batch(&mut otap_batch, ArrowPayloadType::LogAttrs)?;
                self.filter_child_batch(&mut otap_batch, ArrowPayloadType::ScopeAttrs)?;
                self.filter_child_batch(&mut otap_batch, ArrowPayloadType::ResourceAttrs)?;
            }
            signal_type => {
                return Err(Error::NotYetSupportedError {
                    message: format!(
                        "signal type {:?} not yet supported by FilterPipelineStage",
                        signal_type
                    ),
                });
            }
        };

        Ok(otap_batch)
    }
}

#[cfg(test)]
mod test {
    use crate::pipeline::Pipeline;

    use super::*;

    use arrow::array::{RecordBatch, StringArray};
    use arrow::datatypes::{DataType, Field, Schema};
    use data_engine_kql_parser::{KqlParser, Parser};
    use otap_df_pdata::otap::Logs;
    use otap_df_pdata::proto::OtlpProtoMessage;
    use otap_df_pdata::proto::opentelemetry::common::v1::{
        AnyValue, InstrumentationScope, KeyValue,
    };
    use otap_df_pdata::proto::opentelemetry::logs::v1::{
        LogRecord, LogsData, ResourceLogs, ScopeLogs,
    };
    use otap_df_pdata::proto::opentelemetry::resource::v1::Resource;
    use otap_df_pdata::testing::round_trip::otlp_to_otap;
    use otap_df_pdata::{OtapPayload, OtlpProtoBytes};
    use prost::Message;

    use crate::pipeline::test::{to_logs_data, to_otap};

    /// helper function for converting [`OtapArrowRecords`] to [`LogsData`]
    pub fn otap_to_logs_data(otap_batch: OtapArrowRecords) -> LogsData {
        let otap_payload: OtapPayload = otap_batch.into();
        let otlp_bytes: OtlpProtoBytes = otap_payload.try_into().unwrap();
        LogsData::decode(otlp_bytes.as_bytes()).unwrap()
    }

    pub async fn exec_logs_pipeline(kql_expr: &str, logs_data: LogsData) -> LogsData {
        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(logs_data));
        let pipeline_expr = KqlParser::parse(kql_expr).unwrap();
        let mut pipeline = Pipeline::new(pipeline_expr);
        let result = pipeline.execute(otap_batch.clone()).await.unwrap();
        otap_to_logs_data(result)
    }

    #[tokio::test]
    async fn test_simple_filter() {
        let otap_batch = to_otap(vec![
            LogRecord::build()
                .severity_text("TRACE")
                .event_name("1")
                .finish(),
            LogRecord::build()
                .severity_text("INFO")
                .event_name("2")
                .finish(),
            LogRecord::build()
                .severity_text("ERROR")
                .event_name("3")
                .finish(),
        ]);

        let pipeline_expr = KqlParser::parse("logs | where severity_text == \"ERROR\"").unwrap();
        let mut pipeline = Pipeline::new(pipeline_expr);
        let result = pipeline.execute(otap_batch.clone()).await.unwrap();
        let expected = to_otap(vec![
            LogRecord::build()
                .severity_text("ERROR")
                .event_name("3")
                .finish(),
        ]);
        assert_eq!(result, expected);

        // test same filter where the literal is on the left and column name on the right
        let pipeline_expr = KqlParser::parse("logs | where \"ERROR\" == severity_text").unwrap();
        let mut pipeline = Pipeline::new(pipeline_expr);
        let result = pipeline.execute(otap_batch.clone()).await.unwrap();
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_simple_attrs_filter() {
        let otap_batch = to_otap(vec![
            LogRecord::build()
                .event_name("1")
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("a"))])
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("b"))])
                .event_name("2")
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("c"))])
                .event_name("3")
                .finish(),
        ]);

        let expected = vec![
            LogRecord::build()
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("b"))])
                .event_name("2")
                .finish(),
        ];

        let pipeline_expr = KqlParser::parse("logs | where attributes[\"x\"] == \"b\"").unwrap();
        let mut pipeline = Pipeline::new(pipeline_expr);
        let result = pipeline.execute(otap_batch.clone()).await.unwrap();
        let result_otlp = otap_to_logs_data(result);
        pretty_assertions::assert_eq!(
            &result_otlp.resource_logs[0].scope_logs[0].log_records,
            &expected,
        );

        // test same filter where the literal is on the left and the attribute is on the right
        let pipeline_expr = KqlParser::parse("logs | where \"b\" == attributes[\"x\"]").unwrap();
        let mut pipeline = Pipeline::new(pipeline_expr);
        let result = pipeline.execute(otap_batch.clone()).await.unwrap();
        let result_otlp = otap_to_logs_data(result);
        pretty_assertions::assert_eq!(
            &result_otlp.resource_logs[0].scope_logs[0].log_records,
            &expected,
        )
    }

    #[tokio::test]
    async fn test_filter_by_resources() {
        let input = LogsData {
            resource_logs: vec![
                ResourceLogs {
                    schema_url: "schema1".into(),
                    resource: Some(Resource {
                        attributes: vec![KeyValue::new("x", AnyValue::new_string("a"))],
                        ..Default::default()
                    }),
                    scope_logs: vec![ScopeLogs {
                        scope: Some(InstrumentationScope::default()),
                        log_records: vec![LogRecord::build().event_name("r1.e1").finish()],
                        ..Default::default()
                    }],
                },
                ResourceLogs {
                    schema_url: "schema2".into(),
                    resource: Some(Resource {
                        attributes: vec![KeyValue::new("x", AnyValue::new_string("b"))],
                        ..Default::default()
                    }),
                    scope_logs: vec![ScopeLogs {
                        scope: Some(InstrumentationScope::default()),
                        log_records: vec![LogRecord::build().event_name("r2.e1").finish()],
                        ..Default::default()
                    }],
                },
            ],
        };

        // test filter by resource properties
        let result = exec_logs_pipeline(
            "logs | where resource.schema_url == \"schema1\"",
            input.clone(),
        )
        .await;
        assert_eq!(
            result,
            LogsData {
                resource_logs: vec![input.resource_logs[0].clone()],
            }
        );

        // test same as above, but with the literal on the right
        let result = exec_logs_pipeline(
            "logs | where \"schema2\" == resource.schema_url",
            input.clone(),
        )
        .await;
        assert_eq!(
            result,
            LogsData {
                resource_logs: vec![input.resource_logs[1].clone()],
            }
        );

        // test filter by resource attributes
        let result = exec_logs_pipeline(
            "logs | where resource.attributes[\"x\"] == \"a\"",
            input.clone(),
        )
        .await;
        assert_eq!(
            result,
            LogsData {
                resource_logs: vec![input.resource_logs[0].clone()],
            }
        );
    }

    #[tokio::test]
    async fn test_filter_by_scope() {
        let scope_logs = vec![
            ScopeLogs {
                scope: Some(
                    InstrumentationScope::build()
                        .name("name1")
                        .attributes(vec![KeyValue::new("x", AnyValue::new_string("a"))])
                        .finish(),
                ),
                log_records: vec![LogRecord::build().event_name("r1.e1").finish()],
                ..Default::default()
            },
            ScopeLogs {
                scope: Some(
                    InstrumentationScope::build()
                        .name("name2")
                        .attributes(vec![KeyValue::new("x", AnyValue::new_string("b"))])
                        .finish(),
                ),
                log_records: vec![LogRecord::build().event_name("r1.e1").finish()],
                ..Default::default()
            },
        ];

        let input = LogsData {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource::default()),
                scope_logs: scope_logs.clone(),
                ..Default::default()
            }],
        };

        // test filter by resource properties
        let result = exec_logs_pipeline(
            "logs | where instrumentation_scope.name == \"name1\"",
            input.clone(),
        )
        .await;
        assert_eq!(
            result,
            LogsData {
                resource_logs: vec![ResourceLogs {
                    resource: Some(Resource::default()),
                    scope_logs: vec![scope_logs[0].clone()],
                    ..Default::default()
                }],
            }
        );

        // test same as above, but with the literal on the right
        let result = exec_logs_pipeline(
            "logs | where \"name2\" == instrumentation_scope.name",
            input.clone(),
        )
        .await;
        assert_eq!(
            result,
            LogsData {
                resource_logs: vec![ResourceLogs {
                    resource: Some(Resource::default()),
                    scope_logs: vec![scope_logs[1].clone()],
                    ..Default::default()
                }],
            }
        );

        // test filter by resource attributes
        let result = exec_logs_pipeline(
            "logs | where instrumentation_scope.attributes[\"x\"] == \"a\"",
            input.clone(),
        )
        .await;
        assert_eq!(
            result,
            LogsData {
                resource_logs: vec![ResourceLogs {
                    resource: Some(Resource::default()),
                    scope_logs: vec![scope_logs[0].clone()],
                    ..Default::default()
                }],
            }
        );
    }

    #[tokio::test]
    async fn test_filter_with_and() {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .severity_text("ERROR")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("a")),
                    KeyValue::new("y", AnyValue::new_string("d")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("2")
                .severity_text("ERROR")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("b")),
                    KeyValue::new("y", AnyValue::new_string("e")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("3")
                .severity_text("ERROR")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("c")),
                    KeyValue::new("y", AnyValue::new_string("f")),
                ])
                .finish(),
        ];
        let otap_batch = to_otap(log_records.clone());

        // check simple filter "and" properties
        let pipeline_expr =
            KqlParser::parse("logs | where severity_text == \"ERROR\" and event_name == \"2\"")
                .unwrap();
        let mut pipeline = Pipeline::new(pipeline_expr);
        let result = pipeline.execute(otap_batch.clone()).await.unwrap();
        let result_otlp = otap_to_logs_data(result);
        pretty_assertions::assert_eq!(
            &result_otlp.resource_logs[0].scope_logs[0].log_records,
            &[log_records[1].clone()],
        );

        // check simple filter "and" with attributes
        let pipeline_expr = KqlParser::parse(
            "logs | where severity_text == \"ERROR\" and attributes[\"x\"] == \"c\"",
        )
        .unwrap();
        let mut pipeline = Pipeline::new(pipeline_expr);
        let result = pipeline.execute(otap_batch.clone()).await.unwrap();
        let result_otlp = otap_to_logs_data(result);
        pretty_assertions::assert_eq!(
            &result_otlp.resource_logs[0].scope_logs[0].log_records,
            &[log_records[2].clone()],
        );

        // check simple filter "and" two attributes
        let pipeline_expr = KqlParser::parse(
            "logs | where attributes[\"y\"] == \"d\" and attributes[\"x\"] == \"a\"",
        )
        .unwrap();
        let mut pipeline = Pipeline::new(pipeline_expr);
        let result = pipeline.execute(otap_batch.clone()).await.unwrap();
        let result_otlp = otap_to_logs_data(result);
        pretty_assertions::assert_eq!(
            &result_otlp.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone()],
        );
    }

    #[tokio::test]
    async fn test_filter_with_or() {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .severity_text("INFO")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("a")),
                    KeyValue::new("y", AnyValue::new_string("d")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("2")
                .severity_text("ERROR")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("b")),
                    KeyValue::new("y", AnyValue::new_string("e")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("3")
                .severity_text("DEBUG")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("c")),
                    KeyValue::new("y", AnyValue::new_string("f")),
                ])
                .finish(),
        ];
        let otap_batch = to_otap(log_records.clone());

        // check simple filter "or" with properties predicates
        let pipeline_expr = KqlParser::parse(
            "logs | where severity_text == \"INFO\" or severity_text == \"ERROR\"",
        )
        .unwrap();
        let mut pipeline = Pipeline::new(pipeline_expr);
        let result = pipeline.execute(otap_batch.clone()).await.unwrap();
        let result_otlp = otap_to_logs_data(result);
        pretty_assertions::assert_eq!(
            &result_otlp.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[1].clone()],
        );

        // check simple filter "or" with mixed attributes/properties predicates
        let pipeline_expr = KqlParser::parse(
            "logs | where severity_text == \"ERROR\" or attributes[\"x\"] == \"c\"",
        )
        .unwrap();
        let mut pipeline = Pipeline::new(pipeline_expr);
        let result = pipeline.execute(otap_batch.clone()).await.unwrap();
        let result_otlp = otap_to_logs_data(result);
        pretty_assertions::assert_eq!(
            &result_otlp.resource_logs[0].scope_logs[0].log_records,
            &[log_records[1].clone(), log_records[2].clone()],
        );

        // check simple filter "or" two attributes predicates
        let pipeline_expr = KqlParser::parse(
            "logs | where attributes[\"x\"] == \"a\" or attributes[\"y\"] == \"e\"",
        )
        .unwrap();
        let mut pipeline = Pipeline::new(pipeline_expr);
        let result = pipeline.execute(otap_batch.clone()).await.unwrap();
        let result_otlp = otap_to_logs_data(result);
        pretty_assertions::assert_eq!(
            &result_otlp.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[1].clone()],
        );
    }

    #[tokio::test]
    async fn test_filter_with_not() {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .severity_text("INFO")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("a")),
                    KeyValue::new("y", AnyValue::new_string("d")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("2")
                .severity_text("ERROR")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("b")),
                    KeyValue::new("y", AnyValue::new_string("e")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("3")
                .severity_text("DEBUG")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("c")),
                    KeyValue::new("y", AnyValue::new_string("f")),
                ])
                .finish(),
        ];

        // check simple filter "not" with properties predicate
        let result = exec_logs_pipeline(
            "logs | where not(severity_text == \"INFO\")",
            to_logs_data(log_records.clone()),
        )
        .await;
        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[1].clone(), log_records[2].clone()],
        );

        // check simple filter "not" with attributes predicate
        let result = exec_logs_pipeline(
            "logs | where not(attributes[\"x\"] == \"b\")",
            to_logs_data(log_records.clone()),
        )
        .await;
        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[2].clone()],
        );
    }

    #[tokio::test]
    async fn test_filter_not_and() {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .severity_text("INFO")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("a")),
                    KeyValue::new("y", AnyValue::new_string("d")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("2")
                .severity_text("ERROR")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("b")),
                    KeyValue::new("y", AnyValue::new_string("e")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("3")
                .severity_text("DEBUG")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("c")),
                    KeyValue::new("y", AnyValue::new_string("f")),
                ])
                .finish(),
        ];

        // check simple inverted "and" filter with properties predicates
        let result = exec_logs_pipeline(
            "logs | where not(severity_text == \"INFO\" and event_name == \"1\")",
            to_logs_data(log_records.clone()),
        )
        .await;
        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[1].clone(), log_records[2].clone()],
        );

        // check simple inverted "and" filter with attributes predicates
        let result = exec_logs_pipeline(
            "logs | where not(attributes[\"x\"] == \"b\" and attributes[\"y\"] == \"e\")",
            to_logs_data(log_records.clone()),
        )
        .await;
        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[2].clone()],
        );

        // check simple inverted "and" filter with mixed attributes & properties predicates
        // check simple inverted "and" filter with attributes predicates
        let result = exec_logs_pipeline(
            "logs | where not(attributes[\"x\"] == \"c\" and severity_text == \"DEBUG\")",
            to_logs_data(log_records.clone()),
        )
        .await;
        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[1].clone()],
        );
    }

    #[tokio::test]
    async fn test_filter_not_or() {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .severity_text("INFO")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("a")),
                    KeyValue::new("y", AnyValue::new_string("d")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("2")
                .severity_text("ERROR")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("b")),
                    KeyValue::new("y", AnyValue::new_string("e")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("3")
                .severity_text("DEBUG")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("c")),
                    KeyValue::new("y", AnyValue::new_string("f")),
                ])
                .finish(),
        ];

        // check simple inverted "or" filter with properties predicates
        let result = exec_logs_pipeline(
            "logs | where not(severity_text == \"INFO\" or event_name == \"2\")",
            to_logs_data(log_records.clone()),
        )
        .await;
        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[2].clone()],
        );

        // check simple inverted "or" filter with attributes predicates
        let result = exec_logs_pipeline(
            "logs | where not(attributes[\"x\"] == \"b\" or attributes[\"y\"] == \"f\")",
            to_logs_data(log_records.clone()),
        )
        .await;
        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone()],
        );

        // check simple inverted "or" filter with mixed attributes & properties predicates
        let result = exec_logs_pipeline(
            "logs | where not(attributes[\"x\"] == \"c\" or severity_text == \"INFO\")",
            to_logs_data(log_records.clone()),
        )
        .await;
        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[1].clone()],
        );
    }

    #[tokio::test]
    async fn test_filter_numeric_comparison_binary_operators() {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .severity_text("INFO")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("a")),
                    KeyValue::new("z", AnyValue::new_int(1)),
                ])
                .finish(),
            LogRecord::build()
                .event_name("2")
                .severity_text("ERROR")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("b")),
                    KeyValue::new("z", AnyValue::new_int(2)),
                ])
                .finish(),
            LogRecord::build()
                .event_name("3")
                .severity_text("DEBUG")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("c")),
                    KeyValue::new("z", AnyValue::new_int(3)),
                ])
                .finish(),
        ];

        let result = exec_logs_pipeline(
            "logs | where attributes[\"z\"] > 2",
            to_logs_data(log_records.clone()),
        )
        .await;
        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[2].clone()],
        );

        let result = exec_logs_pipeline(
            "logs | where attributes[\"z\"] >= 2",
            to_logs_data(log_records.clone()),
        )
        .await;
        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[1].clone(), log_records[2].clone()],
        );

        let result = exec_logs_pipeline(
            "logs | where attributes[\"z\"] < 2",
            to_logs_data(log_records.clone()),
        )
        .await;
        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone()],
        );

        let result = exec_logs_pipeline(
            "logs | where attributes[\"z\"] <= 2",
            to_logs_data(log_records.clone()),
        )
        .await;
        pretty_assertions::assert_eq!(
            &result.resource_logs[0].scope_logs[0].log_records,
            &[log_records[0].clone(), log_records[1].clone()],
        );
    }

    #[tokio::test]
    async fn test_filter_nomatch() {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .severity_text("INFO")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("a")),
                    KeyValue::new("y", AnyValue::new_string("d")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("2")
                .severity_text("ERROR")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("b")),
                    KeyValue::new("y", AnyValue::new_string("e")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("3")
                .severity_text("DEBUG")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("c")),
                    KeyValue::new("y", AnyValue::new_string("f")),
                ])
                .finish(),
        ];

        let pipeline_expr = KqlParser::parse("logs | where event_name == \"5\"").unwrap();
        let mut pipeline = Pipeline::new(pipeline_expr);
        let result = pipeline
            .execute(to_otap(log_records.clone()))
            .await
            .unwrap();
        // assert it's equal to empty batch because there were no matches
        assert_eq!(result, OtapArrowRecords::Logs(Logs::default()));

        // assert we have the correct behaviour when filtering by attributes as well
        let pipeline_expr = KqlParser::parse("logs | where attributes[\"a\"] == \"1234\"").unwrap();
        let mut pipeline = Pipeline::new(pipeline_expr);
        let result = pipeline
            .execute(to_otap(log_records.clone()))
            .await
            .unwrap();
        assert_eq!(result, OtapArrowRecords::Logs(Logs::default()))
    }

    // TODO ignored as this test is not current passing b/c we've not implemented the correct
    // planning logic for when the root record batch is missing
    #[ignore]
    #[tokio::test]
    async fn test_empty_batch() {
        let input = OtapArrowRecords::Logs(Logs::default());
        let pipeline_expr = KqlParser::parse("logs | where event_name == \"5\"").unwrap();
        let mut pipeline = Pipeline::new(pipeline_expr);
        let result = pipeline.execute(input.clone()).await.unwrap();
        assert_eq!(result, input);
    }

    // TODO ignored as this test is currently not passing b/c we've not implemented the correct
    // planning logic for when filtering by attributes if the attrs record batches are missing
    #[ignore]
    #[tokio::test]
    async fn test_filter_no_attrs() {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .severity_text("INFO")
                .finish(),
            LogRecord::build()
                .event_name("2")
                .severity_text("INFO")
                .finish(),
            LogRecord::build()
                .event_name("3")
                .severity_text("INFO")
                .finish(),
        ];

        // check that if there are no attributes to filter by then, we get the empty batch
        let pipeline_expr = KqlParser::parse("logs | where attributes[\"a\"] == \"1234\"").unwrap();
        let mut pipeline = Pipeline::new(pipeline_expr);
        let result = pipeline
            .execute(to_otap(log_records.clone()))
            .await
            .unwrap();
        assert_eq!(result, OtapArrowRecords::Logs(Logs::default()));

        // check that the same result happens when filtering by resource and scope attrs
        let pipeline_expr =
            KqlParser::parse("logs | where resource.attributes[\"a\"] == \"1234\"").unwrap();
        let mut pipeline = Pipeline::new(pipeline_expr);
        let result = pipeline
            .execute(to_otap(log_records.clone()))
            .await
            .unwrap();
        assert_eq!(result, OtapArrowRecords::Logs(Logs::default()));

        // check that the same result happens when filtering by resource and scope attrs
        let pipeline_expr =
            KqlParser::parse("logs | where instrumentation_scope.attributes[\"a\"] == \"1234\"")
                .unwrap();
        let mut pipeline = Pipeline::new(pipeline_expr);
        let result = pipeline
            .execute(to_otap(log_records.clone()))
            .await
            .unwrap();
        assert_eq!(result, OtapArrowRecords::Logs(Logs::default()));
    }

    #[tokio::test]
    async fn test_filter_all_match() {
        let log_records = vec![
            LogRecord::build()
                .event_name("1")
                .severity_text("INFO")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("a")),
                    KeyValue::new("y", AnyValue::new_string("d")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("2")
                .severity_text("INFO")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("b")),
                    KeyValue::new("y", AnyValue::new_string("e")),
                ])
                .finish(),
            LogRecord::build()
                .event_name("3")
                .severity_text("INFO")
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_string("c")),
                    KeyValue::new("y", AnyValue::new_string("f")),
                ])
                .finish(),
        ];

        // assert the behaviour is correct when nothing is filtered out
        let otap_input = to_otap(log_records);
        let pipeline_expr = KqlParser::parse("logs | where severity_text == \"INFO\"").unwrap();
        let mut pipeline = Pipeline::new(pipeline_expr);
        let result = pipeline.execute(otap_input.clone()).await.unwrap();

        assert_eq!(result, otap_input)
    }

    #[tokio::test]
    async fn test_composite_attributes_filter() {
        // Currently our plans don't construct the Composite<AttributeFilterExec> and until we have
        // that planning in place, we are just invoking it manually to ensure it actually works.

        let mut otap_batch = OtapArrowRecords::Logs(Logs::default());

        let attrs_rb = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new("parent_id", DataType::UInt16, false),
                Field::new("key", DataType::Utf8, false),
                Field::new("str", DataType::Utf8, true),
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values([0, 0, 1, 1, 2, 2])),
                Arc::new(StringArray::from_iter_values([
                    "x", "y", "x", "y", "x", "y",
                ])),
                Arc::new(StringArray::from_iter_values([
                    "a", "d", "b", "e", "c", "f",
                ])),
            ],
        )
        .unwrap();

        otap_batch.set(ArrowPayloadType::LogAttrs, attrs_rb);

        let session_ctx = Pipeline::create_session_context();

        let filter_x_eq_a = AttributesFilterPlan {
            filter: col("key").eq(lit("x")).and(col("str").eq(lit("a"))),
            attrs_identifier: AttributesIdentifier::Root,
        };

        let filter_y_eq_d = AttributesFilterPlan {
            filter: col("key").eq(lit("y")).and(col("str").eq(lit("d"))),
            attrs_identifier: AttributesIdentifier::Root,
        };

        let filter_x_eq_b = AttributesFilterPlan {
            filter: col("key").eq(lit("x")).and(col("str").eq(lit("b"))),
            attrs_identifier: AttributesIdentifier::Root,
        };

        // test simple filter
        let mut filter_exec: Composite<AttributeFilterExec> =
            Composite::<AttributesFilterPlan>::from(filter_x_eq_a.clone())
                .to_exec(&session_ctx, &otap_batch)
                .unwrap();
        assert_eq!(
            filter_exec
                .execute(&otap_batch, &session_ctx, false)
                .unwrap(),
            RoaringBitmap::from_iter([0])
        );

        // test simple not filter
        filter_exec = Composite::Not(Box::new(filter_x_eq_a.clone().into()))
            .to_exec(&session_ctx, &otap_batch)
            .unwrap();
        assert_eq!(
            filter_exec
                .execute(&otap_batch, &session_ctx, false)
                .unwrap(),
            RoaringBitmap::from_iter([1, 2])
        );

        // test "and" filter
        filter_exec = Composite::And((
            Box::new(filter_x_eq_a.clone().into()),
            Box::new(filter_y_eq_d.clone().into()),
        ))
        .to_exec(&session_ctx, &otap_batch)
        .unwrap();
        assert_eq!(
            filter_exec
                .execute(&otap_batch, &session_ctx, false)
                .unwrap(),
            RoaringBitmap::from_iter([0])
        );

        // test inverted "and" filter
        filter_exec = Composite::Not(Box::new(Composite::And((
            Box::new(filter_x_eq_a.clone().into()),
            Box::new(filter_y_eq_d.clone().into()),
        ))))
        .to_exec(&session_ctx, &otap_batch)
        .unwrap();
        assert_eq!(
            filter_exec
                .execute(&otap_batch, &session_ctx, false)
                .unwrap(),
            RoaringBitmap::from_iter([1, 2])
        );

        // test "or" filter
        filter_exec = Composite::Or((
            Box::new(filter_x_eq_a.clone().into()),
            Box::new(filter_x_eq_b.clone().into()),
        ))
        .to_exec(&session_ctx, &otap_batch)
        .unwrap();
        assert_eq!(
            filter_exec
                .execute(&otap_batch, &session_ctx, false)
                .unwrap(),
            RoaringBitmap::from_iter([0, 1])
        );

        // test inverted "or" filter
        filter_exec = Composite::Not(Box::new(Composite::Or((
            Box::new(filter_x_eq_a.clone().into()),
            Box::new(filter_x_eq_b.clone().into()),
        ))))
        .to_exec(&session_ctx, &otap_batch)
        .unwrap();
        assert_eq!(
            filter_exec
                .execute(&otap_batch, &session_ctx, false)
                .unwrap(),
            RoaringBitmap::from_iter([2])
        );
    }
}
