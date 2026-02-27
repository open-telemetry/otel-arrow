// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Utilities for dealing with the types of expressions
//!
//! TODO more fleshed out docs

use crate::pipeline::expr::LogicalDomainExpr;
use arrow::datatypes::{DataType, TimeUnit};
use datafusion::logical_expr::cast;
use otap_df_pdata::schema::consts;

/// TODO comments on what this is
#[derive(Clone, Debug, PartialEq)]
pub enum ExprLogicalType {
    AnyValue,
    AnyValueNumeric,
    Boolean,
    Binary,
    FixedSizeBinary(usize),
    Float64,
    Int32,
    Int64,
    UInt8,
    UInt32,
    ScalarInt,
    String,
    TimestampNanosecond,
}

impl ExprLogicalType {
    fn is_integer(&self) -> bool {
        matches!(self, Self::Int32 | Self::Int64 | Self::UInt8 | Self::UInt32)
    }

    /// return the datatype associated with this type. returns None if the type
    /// is not associated with a single datatype, such as with AnyValue* and ScalarInt
    fn datatype(&self) -> Option<DataType> {
        Some(match self {
            Self::Binary => DataType::Binary,
            Self::Boolean => DataType::Boolean,
            Self::FixedSizeBinary(len) => DataType::FixedSizeBinary(*len as i32),
            Self::Float64 => DataType::Float64,
            Self::Int32 => DataType::Int32,
            Self::Int64 => DataType::Int64,
            Self::String => DataType::Utf8,
            Self::TimestampNanosecond => DataType::Timestamp(TimeUnit::Nanosecond, None),
            Self::UInt32 => DataType::UInt32,
            Self::UInt8 => DataType::UInt8,

            // These types can actually be more than one arrow type, so return None
            Self::AnyValue | Self::AnyValueNumeric | Self::ScalarInt => return None,
        })
    }
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

/// Adds a cast logical expression to cast the value of the expression to the passed data type.
///
/// This is used when coercing the input types for expression operations.
fn cast_expr(expr: &mut LogicalDomainExpr, data_type: DataType) {
    expr.logical_expr = cast(std::mem::take(&mut expr.logical_expr), data_type)
}

/// Attempt to coerce the types of the passed expressions into types that can be arguments to an
/// arithmetic expression
///
/// Note, the current rules are that ... (TODO)
///
/// This function may add casts to the passed expressions in order to coerce the arguments to the
/// correct type
///
/// Returns the type that the arithmetic operation will produce if it were to evaluate successfully.
/// This returns None if it can be detected that arithmetic can be performed on the passed types.
pub fn coerce_arithmetic(
    left: &mut LogicalDomainExpr,
    right: &mut LogicalDomainExpr,
) -> Option<ExprLogicalType> {
    match &left.expr_type {
        ExprLogicalType::AnyValue | ExprLogicalType::AnyValueNumeric => {
            // The left side of the arithmetic operation is an AnyValue, or AnyValue numeric. The
            // only way the arithmetic will succeed at runtime is if the left side is either Int or
            // Double variant of AnyValue.
            //
            // We proceed assuming the left side is one of these types, and return a type only if
            // the right side is, or can be converted to, a type that can successfully do
            // arithmetic arithmetic operation with one of these possible types...

            match &right.expr_type {
                ExprLogicalType::AnyValue | ExprLogicalType::AnyValueNumeric => {
                    // we're adding two AnyValues, but we don't know they're types. We'll have to
                    // assume the types can be added, and let it produce a runtime error if types
                    // were not compatible. The evaluation will succeed if both sides are either
                    // Int or Double.
                    left.expr_type = ExprLogicalType::AnyValueNumeric;
                    right.expr_type = ExprLogicalType::AnyValueNumeric;
                    Some(ExprLogicalType::AnyValueNumeric)
                }

                // If the right side is one of our expected AnyValue variants, we know the
                // expression will only succeed if the left side was the same type. No need to
                // coerce the expressions, but we've discovered what the type of the result will be
                ExprLogicalType::Float64 => {
                    left.expr_type = ExprLogicalType::Float64;
                    Some(ExprLogicalType::Float64)
                }
                ExprLogicalType::Int64 => {
                    left.expr_type = ExprLogicalType::Int64;
                    Some(ExprLogicalType::Int64)
                }

                ExprLogicalType::ScalarInt => {
                    // default type scalar int is int64, and the only type for AnyValue that is int
                    //  like is int64. We don't need to massage the input types, but we've
                    // identified what the expression output of the expression assuming evaluation
                    // succeeds
                    left.expr_type = ExprLogicalType::Int64;
                    Some(ExprLogicalType::Int64)
                }

                other if other.is_integer() => {
                    // TODO - this is probably controversial. We might want to force users to do an
                    // explicit cast when adding different integer types.
                    //
                    // we have a different type of integer value. automatically cast it to int64 so
                    // addition will succeed
                    left.expr_type = ExprLogicalType::Int64;
                    cast_expr(right, DataType::Int64);
                    right.expr_type = ExprLogicalType::Int64;

                    Some(ExprLogicalType::Int64)
                }

                _ => {
                    // other types cannot be added to AnyValue
                    None
                }
            }
        }
        ExprLogicalType::ScalarInt => match &right.expr_type {
            // The left side is a scalar int type. We initialize these to be an int64 in the
            // expression planner, but this is just a placeholder until if/when we know the
            // actual type that will be required.
            ExprLogicalType::Int64 => {
                // nothing to do, types are already aligned
                Some(ExprLogicalType::Int64)
            }
            ExprLogicalType::AnyValue | ExprLogicalType::AnyValueNumeric => {
                // coerce any value into the integer variant
                right.expr_type = ExprLogicalType::Int64;
                Some(ExprLogicalType::Int64)
            }
            right_int_type if right_int_type.is_integer() => {
                // safety: this should always return Some because we can always determine the
                // logical arrow data type for integer types
                let arrow_data_type = right_int_type.datatype().expect("single data type");
                cast_expr(left, arrow_data_type);
                left.expr_type = right_int_type.clone();
                Some(right_int_type.clone())
            }
            _ => {
                // other types cannot be integer types
                None
            }
        },
        ExprLogicalType::Float64 => match &right.expr_type {
            ExprLogicalType::Float64 => {
                // nothing to do, types already aligned
                Some(ExprLogicalType::Float64)
            }
            ExprLogicalType::AnyValue | ExprLogicalType::AnyValueNumeric => {
                // coerce any value into the integer variant
                right.expr_type = ExprLogicalType::Float64;
                Some(ExprLogicalType::Float64)
            }
            _ => {
                // other types cannot be float types
                None
            }
        },
        left_int_type if left_int_type.is_integer() => match &right.expr_type {
            ExprLogicalType::AnyValue => {
                // cast the left side to int64, as this is the only integer type that the AnyValue
                // type can take on
                cast_expr(left, DataType::Int64);
                left.expr_type = ExprLogicalType::Int64;
                right.expr_type = ExprLogicalType::Int64;
                Some(ExprLogicalType::Int64)
            }
            ExprLogicalType::ScalarInt => {
                // safety: this should always return Some because we can always determine the
                // logical arrow data type for integer types
                let arrow_data_type = left_int_type.datatype().expect("single data type");
                cast_expr(right, arrow_data_type);
                right.expr_type = left_int_type.clone();
                Some(left_int_type.clone())
            }
            right_int_type if right_int_type.is_integer() => {
                if *left_int_type == *right_int_type {
                    // nothing to do, types already equal
                    Some(left_int_type.clone())
                } else {
                    // TODO - this probably isn't the right thing to do long term. For now just
                    // cast both sides to int64
                    cast_expr(left, DataType::Int64);
                    left.expr_type = ExprLogicalType::Int64;

                    cast_expr(right, DataType::Int64);
                    right.expr_type = ExprLogicalType::Int64;

                    Some(ExprLogicalType::Int64)
                }
            }
            _ => {
                // other types can't be treated as integers
                None
            }
        },

        // other types cannot be used as argument to arithmetic
        _ => None,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use datafusion::logical_expr::Expr;

    use crate::pipeline::expr::{
        DataDomainId, LogicalDataDomain, LogicalDomainExpr, LogicalExprDataSource,
    };

    fn test_expr(expr_type: ExprLogicalType) -> LogicalDomainExpr {
        LogicalDomainExpr {
            expr_type,

            // rest of fields are just placeholder values
            logical_expr: Expr::default(),
            source: LogicalExprDataSource::DataSource(LogicalDataDomain {
                domain_id: DataDomainId::StaticScalar,
            }),
            requires_dict_downcast: false,
        }
    }

    #[test]
    fn test_coerce_arithmetic_left_any_value_right_any_value() {
        let mut left_expr = test_expr(ExprLogicalType::AnyValue);
        let mut right_expr = test_expr(ExprLogicalType::AnyValue);
        let result = coerce_arithmetic(&mut left_expr, &mut right_expr);
        assert_eq!(result, Some(ExprLogicalType::AnyValueNumeric));
        assert_eq!(left_expr.expr_type, ExprLogicalType::AnyValueNumeric);
        assert_eq!(right_expr.expr_type, ExprLogicalType::AnyValueNumeric);
    }

    #[test]
    fn test_coerce_arithmetic_left_any_value_right_float64() {
        let mut left_expr = test_expr(ExprLogicalType::AnyValue);
        let mut right_expr = test_expr(ExprLogicalType::Float64);
        let result = coerce_arithmetic(&mut left_expr, &mut right_expr);
        assert_eq!(result, Some(ExprLogicalType::Float64));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Float64);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Float64);
    }

    #[test]
    fn test_coerce_arithmetic_left_any_value_right_int64() {
        let mut left_expr = test_expr(ExprLogicalType::AnyValue);
        let mut right_expr = test_expr(ExprLogicalType::Int64);
        let result = coerce_arithmetic(&mut left_expr, &mut right_expr);
        assert_eq!(result, Some(ExprLogicalType::Int64));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Int64);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Int64);
    }

    #[test]
    fn test_coerce_arithmetic_left_any_value_right_scalar_int() {
        let mut left_expr = test_expr(ExprLogicalType::AnyValue);
        let mut right_expr = test_expr(ExprLogicalType::ScalarInt);
        let result = coerce_arithmetic(&mut left_expr, &mut right_expr);
        assert_eq!(result, Some(ExprLogicalType::Int64));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Int64);
        assert_eq!(right_expr.expr_type, ExprLogicalType::ScalarInt);
    }

    #[test]
    fn test_coerce_arithmetic_left_any_value_right_int32() {
        let mut left_expr = test_expr(ExprLogicalType::AnyValue);
        let mut right_expr = test_expr(ExprLogicalType::Int32);
        let result = coerce_arithmetic(&mut left_expr, &mut right_expr);
        assert_eq!(result, Some(ExprLogicalType::Int64));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Int64);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Int64);
    }

    #[test]
    fn test_coerce_arithmetic_left_any_value_right_uint32() {
        let mut left_expr = test_expr(ExprLogicalType::AnyValue);
        let mut right_expr = test_expr(ExprLogicalType::UInt32);
        let result = coerce_arithmetic(&mut left_expr, &mut right_expr);
        assert_eq!(result, Some(ExprLogicalType::Int64));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Int64);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Int64);
    }

    #[test]
    fn test_coerce_arithmetic_left_any_value_right_string() {
        let mut left_expr = test_expr(ExprLogicalType::AnyValue);
        let mut right_expr = test_expr(ExprLogicalType::String);
        let result = coerce_arithmetic(&mut left_expr, &mut right_expr);
        assert_eq!(result, None);
    }

    #[test]
    fn test_coerce_arithmetic_left_scalar_int_right_int64() {
        let mut left_expr = test_expr(ExprLogicalType::ScalarInt);
        let mut right_expr = test_expr(ExprLogicalType::Int64);
        let result = coerce_arithmetic(&mut left_expr, &mut right_expr);
        assert_eq!(result, Some(ExprLogicalType::Int64));
        assert_eq!(left_expr.expr_type, ExprLogicalType::ScalarInt);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Int64);
    }

    #[test]
    fn test_coerce_arithmetic_left_scalar_int_right_any_value() {
        let mut left_expr = test_expr(ExprLogicalType::ScalarInt);
        let mut right_expr = test_expr(ExprLogicalType::AnyValue);
        let result = coerce_arithmetic(&mut left_expr, &mut right_expr);
        assert_eq!(result, Some(ExprLogicalType::Int64));
        assert_eq!(left_expr.expr_type, ExprLogicalType::ScalarInt);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Int64);
    }

    #[test]
    fn test_coerce_arithmetic_left_scalar_int_right_int32() {
        let mut left_expr = test_expr(ExprLogicalType::ScalarInt);
        let mut right_expr = test_expr(ExprLogicalType::Int32);
        let result = coerce_arithmetic(&mut left_expr, &mut right_expr);
        assert_eq!(result, Some(ExprLogicalType::Int32));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Int32);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Int32);
    }

    #[test]
    fn test_coerce_arithmetic_left_scalar_int_right_uint32() {
        let mut left_expr = test_expr(ExprLogicalType::ScalarInt);
        let mut right_expr = test_expr(ExprLogicalType::UInt32);
        let result = coerce_arithmetic(&mut left_expr, &mut right_expr);
        assert_eq!(result, Some(ExprLogicalType::UInt32));
        assert_eq!(left_expr.expr_type, ExprLogicalType::UInt32);
        assert_eq!(right_expr.expr_type, ExprLogicalType::UInt32);
    }

    #[test]
    fn test_coerce_arithmetic_left_scalar_int_right_float64() {
        let mut left_expr = test_expr(ExprLogicalType::ScalarInt);
        let mut right_expr = test_expr(ExprLogicalType::Float64);
        let result = coerce_arithmetic(&mut left_expr, &mut right_expr);
        assert_eq!(result, None);
    }

    #[test]
    fn test_coerce_arithmetic_left_float64_right_float64() {
        let mut left_expr = test_expr(ExprLogicalType::Float64);
        let mut right_expr = test_expr(ExprLogicalType::Float64);
        let result = coerce_arithmetic(&mut left_expr, &mut right_expr);
        assert_eq!(result, Some(ExprLogicalType::Float64));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Float64);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Float64);
    }

    #[test]
    fn test_coerce_arithmetic_left_float64_right_any_value() {
        let mut left_expr = test_expr(ExprLogicalType::Float64);
        let mut right_expr = test_expr(ExprLogicalType::AnyValue);
        let result = coerce_arithmetic(&mut left_expr, &mut right_expr);
        assert_eq!(result, Some(ExprLogicalType::Float64));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Float64);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Float64);
    }

    #[test]
    fn test_coerce_arithmetic_left_float64_right_int64() {
        let mut left_expr = test_expr(ExprLogicalType::Float64);
        let mut right_expr = test_expr(ExprLogicalType::Int64);
        let result = coerce_arithmetic(&mut left_expr, &mut right_expr);
        assert_eq!(result, None);
    }

    #[test]
    fn test_coerce_arithmetic_left_int64_right_int64() {
        let mut left_expr = test_expr(ExprLogicalType::Int64);
        let mut right_expr = test_expr(ExprLogicalType::Int64);
        let result = coerce_arithmetic(&mut left_expr, &mut right_expr);
        assert_eq!(result, Some(ExprLogicalType::Int64));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Int64);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Int64);
    }

    #[test]
    fn test_coerce_arithmetic_left_int64_right_any_value() {
        let mut left_expr = test_expr(ExprLogicalType::Int64);
        let mut right_expr = test_expr(ExprLogicalType::AnyValue);
        let result = coerce_arithmetic(&mut left_expr, &mut right_expr);
        assert_eq!(result, Some(ExprLogicalType::Int64));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Int64);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Int64);
    }

    #[test]
    fn test_coerce_arithmetic_left_int64_right_scalar_int() {
        let mut left_expr = test_expr(ExprLogicalType::Int64);
        let mut right_expr = test_expr(ExprLogicalType::ScalarInt);
        let result = coerce_arithmetic(&mut left_expr, &mut right_expr);
        assert_eq!(result, Some(ExprLogicalType::Int64));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Int64);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Int64);
    }

    #[test]
    fn test_coerce_arithmetic_left_int64_right_int32() {
        let mut left_expr = test_expr(ExprLogicalType::Int64);
        let mut right_expr = test_expr(ExprLogicalType::Int32);
        let result = coerce_arithmetic(&mut left_expr, &mut right_expr);
        assert_eq!(result, Some(ExprLogicalType::Int64));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Int64);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Int64);
    }

    #[test]
    fn test_coerce_arithmetic_left_int32_right_int32() {
        let mut left_expr = test_expr(ExprLogicalType::Int32);
        let mut right_expr = test_expr(ExprLogicalType::Int32);
        let result = coerce_arithmetic(&mut left_expr, &mut right_expr);
        assert_eq!(result, Some(ExprLogicalType::Int32));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Int32);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Int32);
    }

    #[test]
    fn test_coerce_arithmetic_left_int32_right_uint32() {
        let mut left_expr = test_expr(ExprLogicalType::Int32);
        let mut right_expr = test_expr(ExprLogicalType::UInt32);
        let result = coerce_arithmetic(&mut left_expr, &mut right_expr);
        assert_eq!(result, Some(ExprLogicalType::Int64));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Int64);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Int64);
    }

    #[test]
    fn test_coerce_arithmetic_left_int32_right_any_value() {
        let mut left_expr = test_expr(ExprLogicalType::Int32);
        let mut right_expr = test_expr(ExprLogicalType::AnyValue);
        let result = coerce_arithmetic(&mut left_expr, &mut right_expr);
        assert_eq!(result, Some(ExprLogicalType::Int64));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Int64);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Int64);
    }

    #[test]
    fn test_coerce_arithmetic_left_int32_right_scalar_int() {
        let mut left_expr = test_expr(ExprLogicalType::Int32);
        let mut right_expr = test_expr(ExprLogicalType::ScalarInt);
        let result = coerce_arithmetic(&mut left_expr, &mut right_expr);
        assert_eq!(result, Some(ExprLogicalType::Int32));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Int32);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Int32);
    }

    #[test]
    fn test_coerce_arithmetic_left_int32_right_float64() {
        let mut left_expr = test_expr(ExprLogicalType::Int32);
        let mut right_expr = test_expr(ExprLogicalType::Float64);
        let result = coerce_arithmetic(&mut left_expr, &mut right_expr);
        assert_eq!(result, None);
    }

    #[test]
    fn test_coerce_arithmetic_left_uint32_right_uint32() {
        let mut left_expr = test_expr(ExprLogicalType::UInt32);
        let mut right_expr = test_expr(ExprLogicalType::UInt32);
        let result = coerce_arithmetic(&mut left_expr, &mut right_expr);
        assert_eq!(result, Some(ExprLogicalType::UInt32));
        assert_eq!(left_expr.expr_type, ExprLogicalType::UInt32);
        assert_eq!(right_expr.expr_type, ExprLogicalType::UInt32);
    }

    #[test]
    fn test_coerce_arithmetic_left_uint8_right_uint8() {
        let mut left_expr = test_expr(ExprLogicalType::UInt8);
        let mut right_expr = test_expr(ExprLogicalType::UInt8);
        let result = coerce_arithmetic(&mut left_expr, &mut right_expr);
        assert_eq!(result, Some(ExprLogicalType::UInt8));
        assert_eq!(left_expr.expr_type, ExprLogicalType::UInt8);
        assert_eq!(right_expr.expr_type, ExprLogicalType::UInt8);
    }

    #[test]
    fn test_coerce_arithmetic_left_string_right_string() {
        let mut left_expr = test_expr(ExprLogicalType::String);
        let mut right_expr = test_expr(ExprLogicalType::String);
        let result = coerce_arithmetic(&mut left_expr, &mut right_expr);
        assert_eq!(result, None);
    }

    #[test]
    fn test_coerce_arithmetic_left_boolean_right_boolean() {
        let mut left_expr = test_expr(ExprLogicalType::Boolean);
        let mut right_expr = test_expr(ExprLogicalType::Boolean);
        let result = coerce_arithmetic(&mut left_expr, &mut right_expr);
        assert_eq!(result, None);
    }

    #[test]
    fn test_coerce_arithmetic_left_any_value_numeric_right_any_value_numeric() {
        let mut left_expr = test_expr(ExprLogicalType::AnyValueNumeric);
        let mut right_expr = test_expr(ExprLogicalType::AnyValueNumeric);
        let result = coerce_arithmetic(&mut left_expr, &mut right_expr);
        assert_eq!(result, Some(ExprLogicalType::AnyValueNumeric));
        assert_eq!(left_expr.expr_type, ExprLogicalType::AnyValueNumeric);
        assert_eq!(right_expr.expr_type, ExprLogicalType::AnyValueNumeric);
    }

    #[test]
    fn test_coerce_arithmetic_left_any_value_numeric_right_float64() {
        let mut left_expr = test_expr(ExprLogicalType::AnyValueNumeric);
        let mut right_expr = test_expr(ExprLogicalType::Float64);
        let result = coerce_arithmetic(&mut left_expr, &mut right_expr);
        assert_eq!(result, Some(ExprLogicalType::Float64));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Float64);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Float64);
    }

    #[test]
    fn test_coerce_arithmetic_left_any_value_numeric_right_int64() {
        let mut left_expr = test_expr(ExprLogicalType::AnyValueNumeric);
        let mut right_expr = test_expr(ExprLogicalType::Int64);
        let result = coerce_arithmetic(&mut left_expr, &mut right_expr);
        assert_eq!(result, Some(ExprLogicalType::Int64));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Int64);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Int64);
    }
}
