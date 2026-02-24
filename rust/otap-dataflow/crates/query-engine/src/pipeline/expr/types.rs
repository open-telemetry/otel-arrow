// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Utilities for dealing with the types of expressions

use crate::pipeline::expr::LogicalDomainExpr;
use arrow::datatypes::DataType;
use datafusion::logical_expr::cast;
use otap_df_pdata::otap::filter::AnyValue;

pub enum ExprLogicalType {
    AnyValue,
    AnyValueNumberic,
    Boolean,
    Binary,
    Float64,
    Int32,
    Int64,
    UInt32,

    String,
    // TODO the rest
    // TODO - null?
}

// TODO comments on what this is doing
pub fn coerce_arithmetic(
    left: &mut LogicalDomainExpr,
    right: &mut LogicalDomainExpr,
) -> ExprLogicalType {
    match left.expr_type {
        ExprLogicalType::AnyValue => match right.expr_type {
            // just assume we're adding either int64 or float64
            ExprLogicalType::AnyValue => ExprLogicalType::AnyValueNumberic,

            // TODO - should this be any int that gets coerced?
            ExprLogicalType::UInt32 => {
                left.expr_type = ExprLogicalType::Int64;

                // assume coerce to int64
                right.logical_expr = cast(right.logical_expr.clone(), DataType::Int64);
                right.expr_type = ExprLogicalType::Int64;

                ExprLogicalType::Int64
            }

            _ => {
                todo!()
            }
        },
        // TODO this can probably be ExprLogicalType::UInt32 | ExprLogicalType::UInt64 ... etc.
        ExprLogicalType::UInt32 => {
            match right.expr_type {
                // TODO this can probably be ExprLogicalType::AnyValue | ExprLogicalType::AnyValueNumberic
                ExprLogicalType::AnyValue => {
                    // cast to the bigger data_type
                    // TODO - crappy we gotta clone, can we use std::mem::replace?
                    left.logical_expr = cast(left.logical_expr.clone(), DataType::Int64);
                    left.expr_type = ExprLogicalType::Int64;

                    right.expr_type = ExprLogicalType::Int64;
                    // TODO - not sure if we need the cast right? This will just
                    // blow up at runtime if the type isn't int64 I guess ...

                    ExprLogicalType::Int64
                }
                _ => {
                    todo!()
                }
            }
        }
        _ => {
            todo!()
        }
    }
}
