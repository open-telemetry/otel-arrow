// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Utilities for dealing with the types of expressions

use crate::pipeline::expr::LogicalDomainExpr;
use arrow::{
    datatypes::DataType,
    ipc::{FixedSizeBinary, Utf8},
};
use datafusion::logical_expr::cast;
use otap_df_pdata::{otap::filter::AnyValue, schema::consts};

#[derive(Debug)]
pub enum ExprLogicalType {
    // TODO comments
    AnyValue,
    // TODO comments
    AnyValueNumberic,
    Boolean,
    Binary,
    FixedSizeBinary(usize),
    Float64,
    Int32,
    Int64,
    UInt8,
    UInt32,

    // TODO comment on what this is
    ScalarInt,

    String,

    TimestampNanosecond,
    // TODO the rest
    // TODO - null?
}

/// Return the type for the field from the root OTAP record batch.
///
/// Returns None if the field is not known in the OTAP data model.
pub fn root_field_type(field_name: &str) -> Option<ExprLogicalType> {
    Some(match field_name {
        // common fields
        consts::SCHEMA_URL => ExprLogicalType::String,
        consts::DROPPED_ATTRIBUTES_COUNT => ExprLogicalType::UInt32,

        // logs/traces common fields
        consts::TIME_UNIX_NANO => ExprLogicalType::TimestampNanosecond,
        consts::OBSERVED_TIME_UNIX_NANO => ExprLogicalType::TimestampNanosecond,
        consts::TRACE_ID => ExprLogicalType::FixedSizeBinary(16),
        consts::SPAN_ID => ExprLogicalType::FixedSizeBinary(8),
        consts::FLAGS => ExprLogicalType::UInt32,

        // logs fields
        consts::SEVERITY_NUMBER => ExprLogicalType::Int32,
        consts::SEVERITY_TEXT => ExprLogicalType::String,

        // traces fields
        consts::TRACE_STATE => ExprLogicalType::String,
        consts::PARENT_ID => ExprLogicalType::FixedSizeBinary(8),
        consts::KIND => ExprLogicalType::Int32,
        consts::DROPPED_EVENTS_COUNT => ExprLogicalType::UInt32,
        consts::DROPPED_LINKS_COUNT => ExprLogicalType::UInt32,

        // metric fields
        consts::METRIC_TYPE => ExprLogicalType::UInt8,
        consts::NAME => ExprLogicalType::String,
        consts::DESCRIPTION => ExprLogicalType::String,
        consts::UNIT => ExprLogicalType::String,
        consts::AGGREGATION_TEMPORALITY => ExprLogicalType::Int32,

        _ => return None,
    })
}

/// Return the type from a nested struct field on the root OTAP record batch such as resource/scope
///
/// Returns None if the field is not known in the OTAP data model.
pub fn nested_struct_field_type(field_name: &str) -> Option<ExprLogicalType> {
    Some(match field_name {
        // resource fields
        consts::SCHEMA_URL => ExprLogicalType::String,

        // scope fields
        consts::NAME => ExprLogicalType::String,
        consts::VERSION => ExprLogicalType::String,

        // common fields
        consts::DROPPED_ATTRIBUTES_COUNT => ExprLogicalType::UInt32,

        _ => return None,
    })
}

/// Attempt to coerce the types of the passed expressions into types that can be arguments to an
/// arithmetic expression
///
/// Note, the current rules are that ... (TODO)
///
/// This function may add casts to the passed expressions in order to coerce the arguments to the
/// correct type
///
/// Returns the type of the resulting arithmetic expression, or None if no arithmetic can be
/// performed on the passed types.
pub fn coerce_arithmetic(
    left: &mut LogicalDomainExpr,
    right: &mut LogicalDomainExpr,
) -> Option<ExprLogicalType> {
    match &left.expr_type {
        ExprLogicalType::AnyValue | ExprLogicalType::AnyValueNumberic => match &right.expr_type {
            ExprLogicalType::AnyValue | ExprLogicalType::AnyValueNumberic => {
                // just assume we're adding either int64 or float64
                Some(ExprLogicalType::AnyValueNumberic)
            }

            ExprLogicalType::ScalarInt => {
                // default for scalar int is int64, and the only type for AnyValue that is int like
                // is int64. We don't need to massage the input types, but we've identified what the
                // expression type should be ...
                Some(ExprLogicalType::Int64)
            }

            // TODO - should this be any int that gets coerced?
            ExprLogicalType::Int32 => {
                left.expr_type = ExprLogicalType::Int64;

                // assume coerce to int64
                right.logical_expr = cast(right.logical_expr.clone(), DataType::Int64);
                right.expr_type = ExprLogicalType::Int64;

                Some(ExprLogicalType::Int64)
            }

            other_right => {
                todo!("todo left to other right {other_right:?}")
            }
        },
        // TODO this can probably be ExprLogicalType::UInt32 | ExprLogicalType::UInt64 ... etc.
        ExprLogicalType::Int32 => {
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

                    Some(ExprLogicalType::Int64)
                }
                ExprLogicalType::ScalarInt => {
                    // TODO - we gotta check the value and see if it'll overflow or underflow
                    // or at least test this ...
                    right.logical_expr = cast(right.logical_expr.clone(), DataType::Int32);
                    right.expr_type = ExprLogicalType::Int32;

                    Some(ExprLogicalType::Int32)
                }
                _ => {
                    todo!()
                }
            }
        }
        ExprLogicalType::ScalarInt => match right.expr_type {
            ExprLogicalType::Int32 => {
                left.logical_expr = cast(left.logical_expr.clone(), DataType::Int32);
                left.expr_type = ExprLogicalType::Int32;

                Some(ExprLogicalType::Int32)
            }
            _ => {
                todo!()
            }
        },
        other_left => {
            todo!("no type handler for left {:?}", other_left)
        }
    }
}
