// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Utilities for identifying and coercing expression types

use crate::pipeline::expr::{ScopedLogicalExpr, VALUE_COLUMN_NAME};
use arrow::datatypes::{DataType, TimeUnit};
use datafusion::logical_expr::cast;
use otap_df_pdata::schema::consts;

/// Identifier of the logical type of some expression/column.
///
/// Note: This is different than the actual Arrow DataType. In many OTAP columns, the type
/// could use dictionary encoding so for example a column with the type variant
/// ExprLogicalType::String may have arrow DataType Dictionary<u8/16, Utf8> or simply Utf8.
#[derive(Clone, Debug, PartialEq)]
pub enum ExprLogicalType {
    /// This type represents the type of an expression involving attribute value whose
    /// concrete type could not be determined by static analysis of the expression. The actual
    /// type may be one of String, Int64, Float64, Boolean, or Binary
    AnyValue,

    /// The type of an expression that involves an AnyValue that is at least known to be
    /// numeric. The actual type may be one of Int64 or Float64
    AnyValueNumeric,

    /// This type represents the value of an integer scalar expression that could not be determined
    /// to be a concrete type. When parsing, we may receive an expression such as `1`, and will
    /// consider the type to be this generic unknown Int type until such time ias it is used in
    /// conjunction with a place that a known type is expected. For example `1 + severity_number`
    /// would result in static scalar `1`'s type being resolved to Int32, because that is the type
    /// of severity number.
    ScalarInt,

    Boolean,
    FixedSizeBinary(usize),
    Float64,
    Int32,
    Int64,
    UInt8,
    UInt32,
    String,
    DurationNanoSecond,
    TimestampNanosecond,
}

impl ExprLogicalType {
    pub fn is_integer(&self) -> bool {
        matches!(self, Self::Int32 | Self::Int64 | Self::UInt8 | Self::UInt32)
    }

    fn is_signed_integer(&self) -> bool {
        matches!(self, Self::Int32 | Self::Int64)
    }

    /// Returns the bit width of integer types
    fn integer_bit_width(&self) -> Option<u8> {
        match self {
            Self::UInt8 => Some(8),
            Self::Int32 | Self::UInt32 => Some(32),
            Self::Int64 => Some(64),
            _ => None,
        }
    }

    /// return the datatype associated with this type. returns None if the type
    /// is not associated with a single datatype, such as with AnyValue* and ScalarInt
    pub fn datatype(&self) -> Option<DataType> {
        Some(match self {
            Self::Boolean => DataType::Boolean,
            Self::FixedSizeBinary(len) => DataType::FixedSizeBinary(*len as i32),
            Self::Float64 => DataType::Float64,
            Self::Int32 => DataType::Int32,
            Self::Int64 => DataType::Int64,
            Self::String => DataType::Utf8,
            Self::TimestampNanosecond => DataType::Timestamp(TimeUnit::Nanosecond, None),
            Self::DurationNanoSecond => DataType::Duration(TimeUnit::Nanosecond),
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
        consts::EVENT_NAME => ExprLogicalType::String,

        // traces fields
        consts::DURATION_TIME_UNIX_NANO => ExprLogicalType::DurationNanoSecond,
        consts::TRACE_STATE => ExprLogicalType::String,
        consts::PARENT_SPAN_ID => ExprLogicalType::FixedSizeBinary(8),
        consts::KIND => ExprLogicalType::Int32,
        consts::DROPPED_EVENTS_COUNT => ExprLogicalType::UInt32,
        consts::DROPPED_LINKS_COUNT => ExprLogicalType::UInt32,

        // metric fields
        consts::METRIC_TYPE => ExprLogicalType::UInt8,
        consts::NAME => ExprLogicalType::String,
        consts::DESCRIPTION => ExprLogicalType::String,
        consts::UNIT => ExprLogicalType::String,
        consts::AGGREGATION_TEMPORALITY => ExprLogicalType::Int32,

        // the virtual attributes "value" column
        VALUE_COLUMN_NAME => ExprLogicalType::AnyValue,

        _ => return None,
    })
}

/// Returns true if the field on the root batch can be a dictionary encoded type
pub fn root_field_supports_dict_encoding(field_name: &str) -> bool {
    // TODO - when we have better support for time arithmetic we should test that this
    // duration type gets coerced into a dictionary during assignment for column with name
    // consts::DURATION_TIME_UNIX_NANO

    matches!(
        field_name,
        consts::SCHEMA_URL
            | consts::TRACE_ID
            | consts::SPAN_ID
            | consts::SEVERITY_NUMBER
            | consts::SEVERITY_TEXT
            | consts::EVENT_NAME
            | consts::TRACE_STATE
            | consts::KIND
            | consts::NAME
            | consts::DESCRIPTION
            | consts::UNIT
            | consts::AGGREGATION_TEMPORALITY
    )
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

/// Coerce two integer types to a common type for arithmetic operations.
/// Rules:
/// - If either type is signed, result is signed
/// - Result has the larger bit width of the two types
/// - Special case: UInt32 + any signed type -> Int64 (to avoid overflow, since UInt32 max > Int32 max)
/// - UInt8 + Int32 -> Int32 (signed wins, larger width sufficient)
/// - UInt8 + UInt32 -> UInt32 (both unsigned, larger width)
fn coerce_integer_types(left: &ExprLogicalType, right: &ExprLogicalType) -> ExprLogicalType {
    let left_signed = left.is_signed_integer();
    let right_signed = right.is_signed_integer();
    let left_width = left.integer_bit_width().expect("left is integer");
    let right_width = right.integer_bit_width().expect("right is integer");

    let any_signed = left_signed || right_signed;
    let has_uint32 =
        matches!(left, ExprLogicalType::UInt32) || matches!(right, ExprLogicalType::UInt32);

    // Special case: if mixing UInt32 with any signed type, must use Int64
    // because UInt32's max value (~4,2 million) doesn't fit in Int32
    if any_signed && has_uint32 {
        return ExprLogicalType::Int64;
    }

    let max_width = left_width.max(right_width);

    match (any_signed, max_width) {
        // If any is signed, use signed type with appropriate width
        (true, w) if w <= 32 => ExprLogicalType::Int32,
        (true, _) => ExprLogicalType::Int64,
        // Both unsigned
        (false, w) if w <= 8 => ExprLogicalType::UInt8,
        (false, w) if w <= 32 => ExprLogicalType::UInt32,
        // Note: we don't have UInt64, so this shouldn't happen with current types
        (false, _) => ExprLogicalType::UInt32,
    }
}

/// Adds a cast logical expression to cast the value of the expression to the passed data type.
///
/// This is used when coercing the input types for expression operations.
fn cast_expr(expr: &mut ScopedLogicalExpr, data_type: DataType) {
    expr.logical_expr = cast(std::mem::take(&mut expr.logical_expr), data_type)
}

/// Attempt to determine the type of the result of an arithmetic expression performed on the passed
/// left and right arguments.
///
/// This function will also coerce either the left or right side into a type that is compatible
/// with the other side for arithmetic operation, by adding casts in the logical expression tree.
///
/// Type coercion rules for integer arithmetic:
/// - Same types: No coercion (e.g., UInt8 + UInt8 → UInt8)
/// - Both unsigned: Coerce to larger bit width (e.g., UInt8 + UInt32 → UInt32)
/// - Both signed: Coerce to larger bit width (e.g., Int32 + Int64 → Int64)
/// - Mixed signedness with UInt32: Always use Int64 to avoid overflow (e.g., UInt32 + Int32 → Int64)
/// - Mixed signedness without UInt32: Use signed type with larger width (e.g., UInt8 + Int32 → Int32)
/// - Unresolved scalar integers: Coerced to match the concrete type on the other side
/// - AnyValue with integers: Coerced to Int64 (the only integer type AnyValue can represent)
///
/// Returns the type the arithmetic operation will produce IF it were to evaluate successfully.
/// This returns None if it can be detected that arithmetic can be performed on the passed types.
///
/// However, also note that just because this function returns Some(type), does not automatically
/// mean that the expression evaluation will succeed. It only indicates that if the expression
/// evaluation were to succeed, its result would be of the returned type.
///
/// For example, consider an expression such as `attributes["x"] + 1`. Because we're adding an
/// integer to an `AnyValue`, and we know the only integer type an AnyValue can take on is Int64,
/// this function will return `Some(Int64)`. However, if at runtime `attributes["x"]` turns out to
/// not be an Int64 type attribute, the expression evaluation will fail.
pub fn coerce_arithmetic(
    left: &mut ScopedLogicalExpr,
    right: &mut ScopedLogicalExpr,
) -> Option<ExprLogicalType> {
    // TODO - need to update the rules here when we support date/time/duration arithmetic
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
                    // Coerce to the appropriate type based on signedness and bit width
                    let coerced_type = coerce_integer_types(left_int_type, right_int_type);

                    // Cast both sides to the coerced type
                    let target_datatype =
                        coerced_type.datatype().expect("integer type has datatype");
                    cast_expr(left, target_datatype.clone());
                    left.expr_type = coerced_type.clone();

                    cast_expr(right, target_datatype);
                    right.expr_type = coerced_type.clone();

                    Some(coerced_type)
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

    use crate::pipeline::expr::{DataScope, LogicalExprDataSource, ScopedLogicalExpr};

    fn test_expr(expr_type: ExprLogicalType) -> ScopedLogicalExpr {
        ScopedLogicalExpr {
            expr_type,

            // rest of fields are just placeholder values
            logical_expr: Expr::default(),
            source: LogicalExprDataSource::DataSource(DataScope::StaticScalar),
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

    // Tests for mixed integer type coercion

    #[test]
    fn test_coerce_arithmetic_uint8_plus_uint32() {
        // Both unsigned, coerce to larger width (UInt32)
        let mut left_expr = test_expr(ExprLogicalType::UInt8);
        let mut right_expr = test_expr(ExprLogicalType::UInt32);
        let result = coerce_arithmetic(&mut left_expr, &mut right_expr);
        assert_eq!(result, Some(ExprLogicalType::UInt32));
        assert_eq!(left_expr.expr_type, ExprLogicalType::UInt32);
        assert_eq!(right_expr.expr_type, ExprLogicalType::UInt32);
    }

    #[test]
    fn test_coerce_arithmetic_uint8_plus_int32() {
        // Unsigned + signed, coerce to signed with same width (Int32)
        let mut left_expr = test_expr(ExprLogicalType::UInt8);
        let mut right_expr = test_expr(ExprLogicalType::Int32);
        let result = coerce_arithmetic(&mut left_expr, &mut right_expr);
        assert_eq!(result, Some(ExprLogicalType::Int32));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Int32);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Int32);
    }

    #[test]
    fn test_coerce_arithmetic_uint32_plus_int32() {
        // Unsigned + signed with same width, need to upsize to avoid overflow (Int64)
        let mut left_expr = test_expr(ExprLogicalType::UInt32);
        let mut right_expr = test_expr(ExprLogicalType::Int32);
        let result = coerce_arithmetic(&mut left_expr, &mut right_expr);
        assert_eq!(result, Some(ExprLogicalType::Int64));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Int64);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Int64);
    }

    #[test]
    fn test_coerce_arithmetic_int32_plus_uint32() {
        // Signed + unsigned with same width (reverse order), should give same result
        let mut left_expr = test_expr(ExprLogicalType::Int32);
        let mut right_expr = test_expr(ExprLogicalType::UInt32);
        let result = coerce_arithmetic(&mut left_expr, &mut right_expr);
        assert_eq!(result, Some(ExprLogicalType::Int64));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Int64);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Int64);
    }

    #[test]
    fn test_coerce_arithmetic_uint8_plus_int64() {
        // Small unsigned + large signed, coerce to larger signed (Int64)
        let mut left_expr = test_expr(ExprLogicalType::UInt8);
        let mut right_expr = test_expr(ExprLogicalType::Int64);
        let result = coerce_arithmetic(&mut left_expr, &mut right_expr);
        assert_eq!(result, Some(ExprLogicalType::Int64));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Int64);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Int64);
    }

    #[test]
    fn test_coerce_arithmetic_uint32_plus_int64() {
        // Unsigned 32 + signed 64, coerce to larger signed (Int64)
        let mut left_expr = test_expr(ExprLogicalType::UInt32);
        let mut right_expr = test_expr(ExprLogicalType::Int64);
        let result = coerce_arithmetic(&mut left_expr, &mut right_expr);
        assert_eq!(result, Some(ExprLogicalType::Int64));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Int64);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Int64);
    }
}
