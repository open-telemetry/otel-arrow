// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of [`PipelineStage`] for assigning values

use std::borrow::Cow;
use std::ops::Deref;
use std::sync::Arc;

use arrow::array::{ArrayRef, NullArray, RecordBatch, StringArray};
use arrow::compute::filter_record_batch;
use arrow::compute::kernels::cmp::eq;
use arrow::datatypes::{Field, Schema};
use data_engine_expressions::{
    BinaryMathematicalScalarExpression, MathScalarExpression, ScalarExpression,
    SetTransformExpression, SourceScalarExpression,
};
use datafusion::logical_expr::{BinaryExpr, Expr, Operator, col};
use datafusion::physical_expr::PhysicalExprRef;
use datafusion::physical_plan::PhysicalExpr;
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::arrays::get_required_array;
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otap_df_pdata::schema::consts;

use crate::error::{Error, Result};
use crate::pipeline::filter::FilterProjection;
use crate::pipeline::planner::AttributesIdentifier;

const CHILD_COLUMN_NAME: &str = "child";

pub struct AssignPipelineStage {}

#[derive(Debug, PartialEq)]
enum DataDomainId {
    Root,
    Attributes(AttributesIdentifier, String),
    StaticScalar,
}

struct LogicalDataDomain {
    domain_id: DataDomainId,
}

impl LogicalDataDomain {
    fn can_combine(&self, other: &LogicalDataDomain) -> bool {
        if self.is_scalar() || other.is_scalar() {
            return true;
        }

        if self.domain_id == other.domain_id {
            return true;
        }

        return false;
    }

    fn is_scalar(&self) -> bool {
        self.domain_id == DataDomainId::StaticScalar
    }
}

struct LogicalDomainExpr {
    data_domain: LogicalDataDomain,
    logical_expr: Expr,
    child: Option<Box<LogicalDomainExpr>>,
}

struct AssignmentLogicalPlanner {}

impl AssignmentLogicalPlanner {
    fn plan_set_expr(&mut self, set_expr: &SetTransformExpression) -> Result<()> {
        todo!()
    }

    fn plan_scalar_expr(
        &mut self,
        scalar_expression: &ScalarExpression,
    ) -> Result<LogicalDomainExpr> {
        match scalar_expression {
            ScalarExpression::Source(source_scalar_expr) => {
                todo!()
            }
            ScalarExpression::Static(static_scalar_expr) => {
                todo!()
            }
            ScalarExpression::Math(math_scalar_expr) => match math_scalar_expr {
                MathScalarExpression::Add(binary_math_expr) => {
                    let left = self.plan_scalar_expr(binary_math_expr.get_left_expression())?;
                    let right = self.plan_scalar_expr(binary_math_expr.get_right_expression())?;
                    if left.data_domain.can_combine(&right.data_domain) {
                        let data_domain = if left.data_domain.is_scalar() {
                            left.data_domain
                        } else {
                            right.data_domain
                        };
                        Ok(LogicalDomainExpr {
                            data_domain,
                            logical_expr: Expr::BinaryExpr(BinaryExpr::new(
                                Box::new(left.logical_expr),
                                Operator::Plus,
                                Box::new(right.logical_expr),
                            )),
                            child: None,
                        })
                    } else {
                        Ok(LogicalDomainExpr {
                            data_domain: left.data_domain,
                            logical_expr: Expr::BinaryExpr(BinaryExpr::new(
                                Box::new(left.logical_expr),
                                Operator::Plus,
                                Box::new(col(CHILD_COLUMN_NAME)),
                            )),
                            child: Some(Box::new(right)),
                        })
                    }
                }
                _ => {
                    todo!("other math")
                }
            },
            _ => {
                todo!("handle other scalar expressions or return error")
            }
        }
    }
}

struct AssignmentPhysicalPlanner {}

impl AssignmentPhysicalPlanner {
    fn plan(&self, logical_expr: &LogicalDomainExpr) -> Result<PhysicalDomainExpr> {
        todo!()
    }
}

struct PhysicalDomainExpr {
    data_domain: DataDomainId,
    physical_expr: PhysicalExprRef,

    // TODO - we should rename this type to just "Projection"
    // TODO - since this is only needed for root, should be in domain?
    projection: FilterProjection,

    child: Option<Box<PhysicalDomainExpr>>,
}

impl PhysicalDomainExpr {
    fn execute(&self, otap_batch: &OtapArrowRecords) -> Result<ArrayRef> {
        // TODO - this might not need to be Cow?
        let input = match &self.data_domain {
            DataDomainId::Root => otap_batch.root_record_batch().map(|rb| {
                // TODO no unwrap here on missing columns!
                self.projection.project(rb).unwrap()
            }).map(Cow::Owned),
            DataDomainId::Attributes(attrs_id, key) => {
                let attrs_payload_type = match *attrs_id {
                    AttributesIdentifier::Root => match otap_batch.root_payload_type() {
                        ArrowPayloadType::Logs => ArrowPayloadType::LogAttrs,
                        ArrowPayloadType::Spans => ArrowPayloadType::SpanAttrs,
                        _ => ArrowPayloadType::MetricAttrs,
                    },
                    AttributesIdentifier::NonRoot(paylod_type) => paylod_type,
                };

                match otap_batch.get(attrs_payload_type) {
                    Some(rb) => self.try_project_attrs(rb, key.as_str())?.map(Cow::Owned),
                    None => None,
                }
            }
            DataDomainId::StaticScalar => {
                // TODO - this should probably be a lazy static
                Some(Cow::Owned(RecordBatch::new_empty(Arc::new(Schema::new(
                    Vec::<Field>::new(),
                )))))
            }
        };

        let child = match &self.child {
            Some(child) => Some(child.execute(otap_batch)?),
            None => None,
        };

        match (input, child) {
            (Some(input), Some(child)) => {
                todo!()
            },
            (Some(input), None) => {
                let result = self.physical_expr.evaluate(&input)?;
                Ok(result.to_array(input.num_rows())?)
            },
            (None, Some(child)) => {
                todo!()
            },
            (None, None) => {
                // TODO is this right?
                // Ok(Arc::new(NullArray::new(0)))
                todo!()
            }
        }
    }

    fn try_project_attrs(
        &self,
        record_batch: &RecordBatch,
        key: &str,
    ) -> Result<Option<RecordBatch>> {
        // TODO ugly
        let key_col =
            get_required_array(record_batch, consts::ATTRIBUTE_KEY).map_err(|e| Error::ExecutionError {
                cause: e.to_string(),
            })?;
        let key_mask = eq(key_col, &StringArray::new_scalar(key))?;
        let filtered_batch = filter_record_batch(record_batch, &key_mask)?;
        // TODO:
        // - figure out what the type is
        // - project the type column to something called "value"
        todo!()
    }
}

#[cfg(test)]
mod test {
    use arrow::array::Int64Array;

    use super::*;

    #[test]
    fn test_empty_schema() {
        let x = RecordBatch::new_empty(Arc::new(Schema::new(Vec::<Field>::new())));
    }

    #[test]
    fn test_math_with_nulls() {
        let x = NullArray::new(5);
        let y = Int64Array::from(vec![1, 2, 3, 4, 5]);
        let result = arrow::compute::kernels::numeric::add(&x, &y).unwrap();
        println!("{:?}", result);
    }
}
