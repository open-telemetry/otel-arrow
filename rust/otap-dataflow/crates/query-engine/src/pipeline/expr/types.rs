// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Utilities for identifying and coercing expression types

use std::ops::BitAnd;

use crate::pipeline::expr::VALUE_COLUMN_NAME;
use crate::pipeline::planner::DataPointContext;
use crate::pipeline::{SignalContext, SignalKind};

use arrow::datatypes::{DataType, TimeUnit};
use datafusion::logical_expr::{Expr, cast};
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_dfe_pdata::schema::{UTC_TIME_ZONE, consts};

/// Identifier of the logical type of some expression/column.
///
/// Note: This is different than the actual Arrow DataType. In many OTAP columns, the type
/// could use dictionary encoding so for example a column with the type variant
/// ExprLogicalType::String may have arrow DataType Dictionary<u8/16, Utf8> or simply Utf8.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ExprLogicalType {
    /// This type represents the type of an expression involving attribute value whose
    /// concrete type could not be determined by static analysis of the expression. The actual
    /// type may be one of String, Int64, Float64, Boolean, or Binary
    AnyValue,

    /// The type of an expression that involves an AnyValue that is at least known to be
    /// numeric. The actual type may be one of Int64 or Float64
    AnyValueNumeric,

    /// This type represents the value of an integer expression whose concrete type has not yet
    /// been determined. i.e when parsing, we may receive an expression such as `1`, and wil
    /// consider the type to be this generic unknown Int type until such time as it is used in
    /// conjunction with an expr that a known type is expected. For example `1 + severity_number`
    /// would result in static scalar `1`'s type being resolved to Int32, because that is the type
    /// of severity number.
    AnyInt,

    Boolean,
    Binary,
    FixedSizeBinary(usize),
    Float64,
    Int32,
    Int64,
    UInt8,
    UInt32,
    UInt64,
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

    /// Returns true if the logical type represents an unambiguous single type. This will return
    /// false if the type could be resolved to multiple different types, such is the case with
    /// variants `AnyValue`, `AnyValueNumeric` and `AnyInt`
    pub fn is_concrete(&self) -> bool {
        !matches!(self, Self::AnyValue | Self::AnyValueNumeric | Self::AnyInt)
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
            Self::Binary => DataType::Binary,
            Self::Boolean => DataType::Boolean,
            Self::FixedSizeBinary(len) => DataType::FixedSizeBinary(*len as i32),
            Self::Float64 => DataType::Float64,
            Self::Int32 => DataType::Int32,
            Self::Int64 => DataType::Int64,
            Self::String => DataType::Utf8,
            Self::TimestampNanosecond => {
                DataType::Timestamp(TimeUnit::Nanosecond, Some(UTC_TIME_ZONE.into()))
            }
            Self::DurationNanoSecond => DataType::Duration(TimeUnit::Nanosecond),
            Self::UInt64 => DataType::UInt64,
            Self::UInt32 => DataType::UInt32,
            Self::UInt8 => DataType::UInt8,

            // These types can actually be more than one arrow type, so return None
            Self::AnyValue | Self::AnyValueNumeric | Self::AnyInt => return None,
        })
    }
}

/// Bitmask representing one or more signals.
///
/// This is used to validate if a given field is valid is valid of a type of signal
// #[derive(std::ops::bit::Bi)]
#[repr(u8)]
#[derive(Clone, Copy, PartialEq)]
enum SignalMask {
    Logs = 0b001,
    Metrics = 0b010,
    Traces = 0b100,
    LogsTraces = 0b101,
    MetricsTraces = 0b110,
    All = 0b111,
}

impl SignalMask {
    fn from_sig_kind(kind: &SignalKind) -> Self {
        match kind {
            SignalKind::Logs => Self::Logs,
            SignalKind::Metrics(_) => Self::Metrics,
            SignalKind::Traces => Self::Traces,
        }
    }

    /// returns `true` if the signal represented by context is valid for the mask
    fn is_valid(self, context: &SignalContext) -> bool {
        match context {
            SignalContext::All => self == SignalMask::All,
            SignalContext::Single(kind) => self & Self::from_sig_kind(kind) > 0,
        }
    }
}

impl BitAnd for SignalMask {
    type Output = u8;

    fn bitand(self, rhs: Self) -> Self::Output {
        self as u8 & rhs as u8
    }
}

/// Return the type for the field from the root OTAP record batch.
///
/// Returns None if the field is not known in the OTAP data model.
pub fn root_field_type(
    field_name: &str,
    signal_context: &SignalContext,
) -> Option<ExprLogicalType> {
    let (field_type, validity_sig_mask) = match field_name {
        // common fields
        consts::SCHEMA_URL => (ExprLogicalType::String, SignalMask::All),

        // logs/traces common fields
        consts::DROPPED_ATTRIBUTES_COUNT => (ExprLogicalType::UInt32, SignalMask::LogsTraces),
        consts::TRACE_ID => (ExprLogicalType::FixedSizeBinary(16), SignalMask::LogsTraces),
        consts::SPAN_ID => (ExprLogicalType::FixedSizeBinary(8), SignalMask::LogsTraces),
        consts::FLAGS => (ExprLogicalType::UInt32, SignalMask::LogsTraces),

        // metrics/trace common fields
        consts::NAME => (ExprLogicalType::String, SignalMask::MetricsTraces),

        // // logs fields
        consts::TIME_UNIX_NANO => (ExprLogicalType::TimestampNanosecond, SignalMask::Logs),
        consts::OBSERVED_TIME_UNIX_NANO => (ExprLogicalType::TimestampNanosecond, SignalMask::Logs),
        consts::SEVERITY_NUMBER => (ExprLogicalType::Int32, SignalMask::Logs),
        consts::SEVERITY_TEXT => (ExprLogicalType::String, SignalMask::Logs),
        consts::EVENT_NAME => (ExprLogicalType::String, SignalMask::Logs),
        consts::BODY => (ExprLogicalType::AnyValue, SignalMask::Logs),

        // traces fields
        consts::START_TIME_UNIX_NANO => (ExprLogicalType::TimestampNanosecond, SignalMask::Traces),
        consts::DURATION_TIME_UNIX_NANO => {
            (ExprLogicalType::DurationNanoSecond, SignalMask::Traces)
        }
        consts::TRACE_STATE => (ExprLogicalType::String, SignalMask::Traces),
        consts::PARENT_SPAN_ID => (ExprLogicalType::FixedSizeBinary(8), SignalMask::Traces),
        consts::KIND => (ExprLogicalType::Int32, SignalMask::Traces),
        consts::DROPPED_EVENTS_COUNT => (ExprLogicalType::UInt32, SignalMask::Traces),
        consts::DROPPED_LINKS_COUNT => (ExprLogicalType::UInt32, SignalMask::Traces),
        // TODO span "status" struct field not yet supported by this query-engine

        // // metric fields
        consts::METRIC_TYPE => (ExprLogicalType::UInt8, SignalMask::Metrics),
        consts::DESCRIPTION => (ExprLogicalType::String, SignalMask::Metrics),
        consts::UNIT => (ExprLogicalType::String, SignalMask::Metrics),
        consts::AGGREGATION_TEMPORALITY => (ExprLogicalType::Int32, SignalMask::Metrics),
        consts::IS_MONOTONIC => (ExprLogicalType::Boolean, SignalMask::Metrics),
        _ => return None,
    };

    // ensure the field is valid for the signal
    validity_sig_mask
        .is_valid(signal_context)
        .then_some(field_type)
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq)]
enum DpMask {
    Number = 0b0001,
    Histogram = 0b0010,
    ExpHistogram = 0b0100,
    Summary = 0b1000,
    HistExpHist = 0b0110,
    HistExpHistSummary = 0b1110,
    All = 0b1111,
}

impl DpMask {
    fn from_dp_type(dp_type: &MetricDataPointType) -> Self {
        match dp_type {
            MetricDataPointType::NumberDataPoint => Self::Number,
            MetricDataPointType::HistogramDataPoint => Self::Histogram,
            MetricDataPointType::ExponentialHistogramDataPoint => Self::ExpHistogram,
            MetricDataPointType::SummaryDataPoint => Self::Summary,
        }
    }

    /// returns `true` if the data point type represented by context is valid for the mask
    fn is_valid(self, context: &DataPointContext) -> bool {
        match context {
            DataPointContext::All => self == DpMask::All,
            DataPointContext::Single(dp_type) => self & Self::from_dp_type(dp_type) > 0,
        }
    }
}

impl BitAnd for DpMask {
    type Output = u8;

    fn bitand(self, rhs: Self) -> Self::Output {
        self as u8 & rhs as u8
    }
}

pub fn data_point_field_type(
    field_name: &str,
    dp_context: &DataPointContext,
) -> Option<ExprLogicalType> {
    let (field_type, validity_sig_mask) = match field_name {
        // all data point common fields
        consts::TIME_UNIX_NANO => (ExprLogicalType::TimestampNanosecond, DpMask::All),
        consts::START_TIME_UNIX_NANO => (ExprLogicalType::TimestampNanosecond, DpMask::All),
        consts::FLAGS => (ExprLogicalType::UInt32, DpMask::All),

        // number data point fields
        consts::INT_VALUE => (ExprLogicalType::Int64, DpMask::Number),
        consts::DOUBLE_VALUE => (ExprLogicalType::Float64, DpMask::Number),

        // histogram/exp hist/summary data point common fields
        consts::SUM => (ExprLogicalType::Float64, DpMask::HistExpHistSummary),
        consts::COUNT => (ExprLogicalType::UInt64, DpMask::HistExpHistSummary),

        // summary data points
        // TODO quantile struct field not supported by query-engine
        // TODO value list field not supported by query-engine

        // histogram / exp hist common fields
        consts::HISTOGRAM_MIN => (ExprLogicalType::Float64, DpMask::HistExpHist),
        consts::HISTOGRAM_MAX => (ExprLogicalType::Float64, DpMask::HistExpHist),

        // histogram data point fields
        // TODO bucket counts list fields not yet supported by query-engine
        // TODO explicit bounds list field not yet supported by query-engine

        // exponential histogram data point fields
        consts::EXP_HISTOGRAM_SCALE => (ExprLogicalType::Int32, DpMask::ExpHistogram),
        consts::EXP_HISTOGRAM_ZERO_COUNT => (ExprLogicalType::UInt64, DpMask::ExpHistogram),
        consts::EXP_HISTOGRAM_ZERO_THRESHOLD => (ExprLogicalType::Float64, DpMask::ExpHistogram),
        // TODO exp histogram positive/negative struct fields not yet supported by query-engine
        _ => return None,
    };

    validity_sig_mask.is_valid(dp_context).then_some(field_type)
}

/// Return the type for an attribute pipeline field.
///
/// Only `key` and `value` are valid fields when operating inside `apply attributes { ... }`.
/// Returns `None` if the field is not valid for attribute pipelines.
pub fn attribute_field_type(field_name: &str) -> Option<ExprLogicalType> {
    match field_name {
        consts::ATTRIBUTE_KEY => Some(ExprLogicalType::String),
        VALUE_COLUMN_NAME => Some(ExprLogicalType::AnyValue),
        _ => None,
    }
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
pub fn cast_expr(expr: &mut Expr, data_type: DataType) {
    *expr = cast(std::mem::take(expr), data_type)
}

/// Attempt to determine the type of the result of an arithmetic expression performed on the passed
/// left and right arguments.
///
/// This function will also coerce either the left or right side into a type that is compatible
/// with the other side for arithmetic operation, by adding casts in the logical expression tree.
///
/// Type coercion rules for integer arithmetic:
/// - Same types: No coercion (e.g., UInt8 + UInt8 -> UInt8)
/// - Both unsigned: Coerce to larger bit width (e.g., UInt8 + UInt32 -> UInt32)
/// - Both signed: Coerce to larger bit width (e.g., Int32 + Int64 -> Int64)
/// - Mixed signedness with UInt32: Always use Int64 to avoid overflow (e.g., UInt32 + Int32 -> Int64)
/// - Mixed signedness without UInt32: Use signed type with larger width (e.g., UInt8 + Int32 -> Int32)
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
    left_expr: &mut Expr,
    left_type: &mut ExprLogicalType,
    right_expr: &mut Expr,
    right_type: &mut ExprLogicalType,
) -> Option<ExprLogicalType> {
    // TODO - need to update the rules here when we support date/time/duration arithmetic
    match &*left_type {
        ExprLogicalType::AnyValue | ExprLogicalType::AnyValueNumeric => {
            // The left side of the arithmetic operation is an AnyValue, or AnyValue numeric. The
            // only way the arithmetic will succeed at runtime is if the left side is either Int or
            // Double variant of AnyValue.
            //
            // We proceed assuming the left side is one of these types, and return a type only if
            // the right side is, or can be converted to, a type that can successfully do
            // arithmetic arithmetic operation with one of these possible types...

            match &*right_type {
                ExprLogicalType::AnyValue | ExprLogicalType::AnyValueNumeric => {
                    // we're adding two AnyValues, but we don't know they're types. We'll have to
                    // assume the types can be added, and let it produce a runtime error if types
                    // were not compatible. The evaluation will succeed if both sides are either
                    // Int or Double.
                    *left_type = ExprLogicalType::AnyValueNumeric;
                    *right_type = ExprLogicalType::AnyValueNumeric;
                    Some(ExprLogicalType::AnyValueNumeric)
                }

                // If the right side is one of our expected AnyValue variants, we know the
                // expression will only succeed if the left side was the same type. No need to
                // coerce the expressions, but we've discovered what the type of the result will be
                ExprLogicalType::Float64 => {
                    *left_type = ExprLogicalType::Float64;
                    Some(ExprLogicalType::Float64)
                }
                ExprLogicalType::Int64 => {
                    *left_type = ExprLogicalType::Int64;
                    Some(ExprLogicalType::Int64)
                }

                ExprLogicalType::AnyInt => {
                    // default type scalar int is int64, and the only type for AnyValue that is int
                    //  like is int64. We don't need to massage the input types, but we've
                    // identified what the expression output of the expression assuming evaluation
                    // succeeds
                    *left_type = ExprLogicalType::Int64;
                    Some(ExprLogicalType::Int64)
                }

                other if other.is_integer() => {
                    // TODO - this is probably controversial. We might want to force users to do an
                    // explicit cast when adding different integer types.
                    //
                    // we have a different type of integer value. automatically cast it to int64 so
                    // addition will succeed
                    *left_type = ExprLogicalType::Int64;
                    cast_expr(right_expr, DataType::Int64);
                    *right_type = ExprLogicalType::Int64;

                    Some(ExprLogicalType::Int64)
                }

                _ => {
                    // other types cannot be added to AnyValue
                    None
                }
            }
        }
        ExprLogicalType::AnyInt => match &*right_type {
            // The left side is a scalar int type. We initialize these to be an int64 in the
            // expression planner, but this is just a placeholder until if/when we know the
            // actual type that will be required.
            ExprLogicalType::Int64 => {
                // nothing to do, types are already aligned
                Some(ExprLogicalType::Int64)
            }
            ExprLogicalType::AnyValue | ExprLogicalType::AnyValueNumeric => {
                // coerce any value into the integer variant
                *right_type = ExprLogicalType::Int64;
                Some(ExprLogicalType::Int64)
            }
            right_int_type if right_int_type.is_integer() => {
                // safety: this should always return Some because we can always determine the
                // logical arrow data type for integer types
                let arrow_data_type = right_int_type.datatype().expect("single data type");
                cast_expr(left_expr, arrow_data_type);
                *left_type = right_int_type.clone();
                Some(right_int_type.clone())
            }
            _ => {
                // other types cannot be integer types
                None
            }
        },
        ExprLogicalType::Float64 => match &*right_type {
            ExprLogicalType::Float64 => {
                // nothing to do, types already aligned
                Some(ExprLogicalType::Float64)
            }
            ExprLogicalType::AnyValue | ExprLogicalType::AnyValueNumeric => {
                // coerce any value into the integer variant
                *right_type = ExprLogicalType::Float64;
                Some(ExprLogicalType::Float64)
            }
            _ => {
                // other types cannot be float types
                None
            }
        },
        left_int_type if left_int_type.is_integer() => match &*right_type {
            ExprLogicalType::AnyValue => {
                // cast the left side to int64, as this is the only integer type that the AnyValue
                // type can take on
                cast_expr(left_expr, DataType::Int64);
                *left_type = ExprLogicalType::Int64;
                *right_type = ExprLogicalType::Int64;
                Some(ExprLogicalType::Int64)
            }
            ExprLogicalType::AnyInt => {
                // safety: this should always return Some because we can always determine the
                // logical arrow data type for integer types
                let arrow_data_type = left_int_type.datatype().expect("single data type");
                cast_expr(right_expr, arrow_data_type);
                *right_type = left_int_type.clone();
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
                    cast_expr(left_expr, target_datatype.clone());
                    *left_type = coerced_type.clone();

                    cast_expr(right_expr, target_datatype);
                    *right_type = coerced_type.clone();

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

/// identifier of metric data point type
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub enum MetricDataPointType {
    NumberDataPoint,
    HistogramDataPoint,
    ExponentialHistogramDataPoint,
    SummaryDataPoint,
}

impl MetricDataPointType {
    /// Get the OTAP payload type containing data points of this metric type
    pub fn payload_type(&self) -> ArrowPayloadType {
        match self {
            Self::SummaryDataPoint => ArrowPayloadType::SummaryDataPoints,
            Self::ExponentialHistogramDataPoint => ArrowPayloadType::ExpHistogramDataPoints,
            Self::HistogramDataPoint => ArrowPayloadType::HistogramDataPoints,
            Self::NumberDataPoint => ArrowPayloadType::NumberDataPoints,
        }
    }

    /// return the [`ArrowPayloadType`] associated with data points of this data point type
    pub fn dp_attrs_payload_type(&self) -> ArrowPayloadType {
        match self {
            Self::SummaryDataPoint => ArrowPayloadType::SummaryDpAttrs,
            Self::ExponentialHistogramDataPoint => ArrowPayloadType::ExpHistogramDpAttrs,
            Self::HistogramDataPoint => ArrowPayloadType::HistogramDpAttrs,
            Self::NumberDataPoint => ArrowPayloadType::NumberDpAttrs,
        }
    }

    /// return the [`ArrowPayloadType`] associated with exemplars of this data point type
    pub fn exemplar_payload_type(&self) -> Option<ArrowPayloadType> {
        match self {
            Self::ExponentialHistogramDataPoint => Some(ArrowPayloadType::ExpHistogramDpExemplars),
            Self::HistogramDataPoint => Some(ArrowPayloadType::HistogramDpExemplars),
            Self::NumberDataPoint => Some(ArrowPayloadType::NumberDpExemplars),
            Self::SummaryDataPoint => None,
        }
    }

    /// return the [`ArrowPayloadType`] of attributes of exemplars of this data point type
    pub fn exemplar_attr_payload_type(&self) -> Option<ArrowPayloadType> {
        match self {
            Self::ExponentialHistogramDataPoint => {
                Some(ArrowPayloadType::ExpHistogramDpExemplarAttrs)
            }
            Self::HistogramDataPoint => Some(ArrowPayloadType::HistogramDpExemplarAttrs),
            Self::NumberDataPoint => Some(ArrowPayloadType::NumberDpExemplarAttrs),
            Self::SummaryDataPoint => None,
        }
    }

    /// returns an iterator of all the types of metric data points
    pub fn all() -> impl Iterator<Item = Self> {
        [
            MetricDataPointType::NumberDataPoint,
            MetricDataPointType::HistogramDataPoint,
            MetricDataPointType::ExponentialHistogramDataPoint,
            MetricDataPointType::SummaryDataPoint,
        ]
        .into_iter()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use datafusion::logical_expr::Expr;

    /// Test helper pair: a logical expression and its type.
    struct TestExpr {
        logical_expr: Expr,
        expr_type: ExprLogicalType,
    }

    fn test_expr(expr_type: ExprLogicalType) -> TestExpr {
        TestExpr {
            expr_type,
            logical_expr: Expr::default(),
        }
    }

    #[test]
    fn test_coerce_arithmetic_left_any_value_right_any_value() {
        let mut left_expr = test_expr(ExprLogicalType::AnyValue);
        let mut right_expr = test_expr(ExprLogicalType::AnyValue);
        let result = coerce_arithmetic(
            &mut left_expr.logical_expr,
            &mut left_expr.expr_type,
            &mut right_expr.logical_expr,
            &mut right_expr.expr_type,
        );
        assert_eq!(result, Some(ExprLogicalType::AnyValueNumeric));
        assert_eq!(left_expr.expr_type, ExprLogicalType::AnyValueNumeric);
        assert_eq!(right_expr.expr_type, ExprLogicalType::AnyValueNumeric);
    }

    #[test]
    fn test_coerce_arithmetic_left_any_value_right_float64() {
        let mut left_expr = test_expr(ExprLogicalType::AnyValue);
        let mut right_expr = test_expr(ExprLogicalType::Float64);
        let result = coerce_arithmetic(
            &mut left_expr.logical_expr,
            &mut left_expr.expr_type,
            &mut right_expr.logical_expr,
            &mut right_expr.expr_type,
        );
        assert_eq!(result, Some(ExprLogicalType::Float64));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Float64);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Float64);
    }

    #[test]
    fn test_coerce_arithmetic_left_any_value_right_int64() {
        let mut left_expr = test_expr(ExprLogicalType::AnyValue);
        let mut right_expr = test_expr(ExprLogicalType::Int64);
        let result = coerce_arithmetic(
            &mut left_expr.logical_expr,
            &mut left_expr.expr_type,
            &mut right_expr.logical_expr,
            &mut right_expr.expr_type,
        );
        assert_eq!(result, Some(ExprLogicalType::Int64));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Int64);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Int64);
    }

    #[test]
    fn test_coerce_arithmetic_left_any_value_right_scalar_int() {
        let mut left_expr = test_expr(ExprLogicalType::AnyValue);
        let mut right_expr = test_expr(ExprLogicalType::AnyInt);
        let result = coerce_arithmetic(
            &mut left_expr.logical_expr,
            &mut left_expr.expr_type,
            &mut right_expr.logical_expr,
            &mut right_expr.expr_type,
        );
        assert_eq!(result, Some(ExprLogicalType::Int64));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Int64);
        assert_eq!(right_expr.expr_type, ExprLogicalType::AnyInt);
    }

    #[test]
    fn test_coerce_arithmetic_left_any_value_right_int32() {
        let mut left_expr = test_expr(ExprLogicalType::AnyValue);
        let mut right_expr = test_expr(ExprLogicalType::Int32);
        let result = coerce_arithmetic(
            &mut left_expr.logical_expr,
            &mut left_expr.expr_type,
            &mut right_expr.logical_expr,
            &mut right_expr.expr_type,
        );
        assert_eq!(result, Some(ExprLogicalType::Int64));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Int64);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Int64);
    }

    #[test]
    fn test_coerce_arithmetic_left_any_value_right_uint32() {
        let mut left_expr = test_expr(ExprLogicalType::AnyValue);
        let mut right_expr = test_expr(ExprLogicalType::UInt32);
        let result = coerce_arithmetic(
            &mut left_expr.logical_expr,
            &mut left_expr.expr_type,
            &mut right_expr.logical_expr,
            &mut right_expr.expr_type,
        );
        assert_eq!(result, Some(ExprLogicalType::Int64));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Int64);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Int64);
    }

    #[test]
    fn test_coerce_arithmetic_left_any_value_right_string() {
        let mut left_expr = test_expr(ExprLogicalType::AnyValue);
        let mut right_expr = test_expr(ExprLogicalType::String);
        let result = coerce_arithmetic(
            &mut left_expr.logical_expr,
            &mut left_expr.expr_type,
            &mut right_expr.logical_expr,
            &mut right_expr.expr_type,
        );
        assert_eq!(result, None);
    }

    #[test]
    fn test_coerce_arithmetic_left_scalar_int_right_int64() {
        let mut left_expr = test_expr(ExprLogicalType::AnyInt);
        let mut right_expr = test_expr(ExprLogicalType::Int64);
        let result = coerce_arithmetic(
            &mut left_expr.logical_expr,
            &mut left_expr.expr_type,
            &mut right_expr.logical_expr,
            &mut right_expr.expr_type,
        );
        assert_eq!(result, Some(ExprLogicalType::Int64));
        assert_eq!(left_expr.expr_type, ExprLogicalType::AnyInt);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Int64);
    }

    #[test]
    fn test_coerce_arithmetic_left_scalar_int_right_any_value() {
        let mut left_expr = test_expr(ExprLogicalType::AnyInt);
        let mut right_expr = test_expr(ExprLogicalType::AnyValue);
        let result = coerce_arithmetic(
            &mut left_expr.logical_expr,
            &mut left_expr.expr_type,
            &mut right_expr.logical_expr,
            &mut right_expr.expr_type,
        );
        assert_eq!(result, Some(ExprLogicalType::Int64));
        assert_eq!(left_expr.expr_type, ExprLogicalType::AnyInt);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Int64);
    }

    #[test]
    fn test_coerce_arithmetic_left_scalar_int_right_int32() {
        let mut left_expr = test_expr(ExprLogicalType::AnyInt);
        let mut right_expr = test_expr(ExprLogicalType::Int32);
        let result = coerce_arithmetic(
            &mut left_expr.logical_expr,
            &mut left_expr.expr_type,
            &mut right_expr.logical_expr,
            &mut right_expr.expr_type,
        );
        assert_eq!(result, Some(ExprLogicalType::Int32));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Int32);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Int32);
    }

    #[test]
    fn test_coerce_arithmetic_left_scalar_int_right_uint32() {
        let mut left_expr = test_expr(ExprLogicalType::AnyInt);
        let mut right_expr = test_expr(ExprLogicalType::UInt32);
        let result = coerce_arithmetic(
            &mut left_expr.logical_expr,
            &mut left_expr.expr_type,
            &mut right_expr.logical_expr,
            &mut right_expr.expr_type,
        );
        assert_eq!(result, Some(ExprLogicalType::UInt32));
        assert_eq!(left_expr.expr_type, ExprLogicalType::UInt32);
        assert_eq!(right_expr.expr_type, ExprLogicalType::UInt32);
    }

    #[test]
    fn test_coerce_arithmetic_left_scalar_int_right_float64() {
        let mut left_expr = test_expr(ExprLogicalType::AnyInt);
        let mut right_expr = test_expr(ExprLogicalType::Float64);
        let result = coerce_arithmetic(
            &mut left_expr.logical_expr,
            &mut left_expr.expr_type,
            &mut right_expr.logical_expr,
            &mut right_expr.expr_type,
        );
        assert_eq!(result, None);
    }

    #[test]
    fn test_coerce_arithmetic_left_float64_right_float64() {
        let mut left_expr = test_expr(ExprLogicalType::Float64);
        let mut right_expr = test_expr(ExprLogicalType::Float64);
        let result = coerce_arithmetic(
            &mut left_expr.logical_expr,
            &mut left_expr.expr_type,
            &mut right_expr.logical_expr,
            &mut right_expr.expr_type,
        );
        assert_eq!(result, Some(ExprLogicalType::Float64));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Float64);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Float64);
    }

    #[test]
    fn test_coerce_arithmetic_left_float64_right_any_value() {
        let mut left_expr = test_expr(ExprLogicalType::Float64);
        let mut right_expr = test_expr(ExprLogicalType::AnyValue);
        let result = coerce_arithmetic(
            &mut left_expr.logical_expr,
            &mut left_expr.expr_type,
            &mut right_expr.logical_expr,
            &mut right_expr.expr_type,
        );
        assert_eq!(result, Some(ExprLogicalType::Float64));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Float64);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Float64);
    }

    #[test]
    fn test_coerce_arithmetic_left_float64_right_int64() {
        let mut left_expr = test_expr(ExprLogicalType::Float64);
        let mut right_expr = test_expr(ExprLogicalType::Int64);
        let result = coerce_arithmetic(
            &mut left_expr.logical_expr,
            &mut left_expr.expr_type,
            &mut right_expr.logical_expr,
            &mut right_expr.expr_type,
        );
        assert_eq!(result, None);
    }

    #[test]
    fn test_coerce_arithmetic_left_int64_right_int64() {
        let mut left_expr = test_expr(ExprLogicalType::Int64);
        let mut right_expr = test_expr(ExprLogicalType::Int64);
        let result = coerce_arithmetic(
            &mut left_expr.logical_expr,
            &mut left_expr.expr_type,
            &mut right_expr.logical_expr,
            &mut right_expr.expr_type,
        );
        assert_eq!(result, Some(ExprLogicalType::Int64));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Int64);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Int64);
    }

    #[test]
    fn test_coerce_arithmetic_left_int64_right_any_value() {
        let mut left_expr = test_expr(ExprLogicalType::Int64);
        let mut right_expr = test_expr(ExprLogicalType::AnyValue);
        let result = coerce_arithmetic(
            &mut left_expr.logical_expr,
            &mut left_expr.expr_type,
            &mut right_expr.logical_expr,
            &mut right_expr.expr_type,
        );
        assert_eq!(result, Some(ExprLogicalType::Int64));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Int64);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Int64);
    }

    #[test]
    fn test_coerce_arithmetic_left_int64_right_scalar_int() {
        let mut left_expr = test_expr(ExprLogicalType::Int64);
        let mut right_expr = test_expr(ExprLogicalType::AnyInt);
        let result = coerce_arithmetic(
            &mut left_expr.logical_expr,
            &mut left_expr.expr_type,
            &mut right_expr.logical_expr,
            &mut right_expr.expr_type,
        );
        assert_eq!(result, Some(ExprLogicalType::Int64));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Int64);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Int64);
    }

    #[test]
    fn test_coerce_arithmetic_left_int64_right_int32() {
        let mut left_expr = test_expr(ExprLogicalType::Int64);
        let mut right_expr = test_expr(ExprLogicalType::Int32);
        let result = coerce_arithmetic(
            &mut left_expr.logical_expr,
            &mut left_expr.expr_type,
            &mut right_expr.logical_expr,
            &mut right_expr.expr_type,
        );
        assert_eq!(result, Some(ExprLogicalType::Int64));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Int64);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Int64);
    }

    #[test]
    fn test_coerce_arithmetic_left_int32_right_int32() {
        let mut left_expr = test_expr(ExprLogicalType::Int32);
        let mut right_expr = test_expr(ExprLogicalType::Int32);
        let result = coerce_arithmetic(
            &mut left_expr.logical_expr,
            &mut left_expr.expr_type,
            &mut right_expr.logical_expr,
            &mut right_expr.expr_type,
        );
        assert_eq!(result, Some(ExprLogicalType::Int32));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Int32);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Int32);
    }

    #[test]
    fn test_coerce_arithmetic_left_int32_right_uint32() {
        let mut left_expr = test_expr(ExprLogicalType::Int32);
        let mut right_expr = test_expr(ExprLogicalType::UInt32);
        let result = coerce_arithmetic(
            &mut left_expr.logical_expr,
            &mut left_expr.expr_type,
            &mut right_expr.logical_expr,
            &mut right_expr.expr_type,
        );
        assert_eq!(result, Some(ExprLogicalType::Int64));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Int64);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Int64);
    }

    #[test]
    fn test_coerce_arithmetic_left_int32_right_any_value() {
        let mut left_expr = test_expr(ExprLogicalType::Int32);
        let mut right_expr = test_expr(ExprLogicalType::AnyValue);
        let result = coerce_arithmetic(
            &mut left_expr.logical_expr,
            &mut left_expr.expr_type,
            &mut right_expr.logical_expr,
            &mut right_expr.expr_type,
        );
        assert_eq!(result, Some(ExprLogicalType::Int64));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Int64);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Int64);
    }

    #[test]
    fn test_coerce_arithmetic_left_int32_right_scalar_int() {
        let mut left_expr = test_expr(ExprLogicalType::Int32);
        let mut right_expr = test_expr(ExprLogicalType::AnyInt);
        let result = coerce_arithmetic(
            &mut left_expr.logical_expr,
            &mut left_expr.expr_type,
            &mut right_expr.logical_expr,
            &mut right_expr.expr_type,
        );
        assert_eq!(result, Some(ExprLogicalType::Int32));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Int32);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Int32);
    }

    #[test]
    fn test_coerce_arithmetic_left_int32_right_float64() {
        let mut left_expr = test_expr(ExprLogicalType::Int32);
        let mut right_expr = test_expr(ExprLogicalType::Float64);
        let result = coerce_arithmetic(
            &mut left_expr.logical_expr,
            &mut left_expr.expr_type,
            &mut right_expr.logical_expr,
            &mut right_expr.expr_type,
        );
        assert_eq!(result, None);
    }

    #[test]
    fn test_coerce_arithmetic_left_uint32_right_uint32() {
        let mut left_expr = test_expr(ExprLogicalType::UInt32);
        let mut right_expr = test_expr(ExprLogicalType::UInt32);
        let result = coerce_arithmetic(
            &mut left_expr.logical_expr,
            &mut left_expr.expr_type,
            &mut right_expr.logical_expr,
            &mut right_expr.expr_type,
        );
        assert_eq!(result, Some(ExprLogicalType::UInt32));
        assert_eq!(left_expr.expr_type, ExprLogicalType::UInt32);
        assert_eq!(right_expr.expr_type, ExprLogicalType::UInt32);
    }

    #[test]
    fn test_coerce_arithmetic_left_uint8_right_uint8() {
        let mut left_expr = test_expr(ExprLogicalType::UInt8);
        let mut right_expr = test_expr(ExprLogicalType::UInt8);
        let result = coerce_arithmetic(
            &mut left_expr.logical_expr,
            &mut left_expr.expr_type,
            &mut right_expr.logical_expr,
            &mut right_expr.expr_type,
        );
        assert_eq!(result, Some(ExprLogicalType::UInt8));
        assert_eq!(left_expr.expr_type, ExprLogicalType::UInt8);
        assert_eq!(right_expr.expr_type, ExprLogicalType::UInt8);
    }

    #[test]
    fn test_coerce_arithmetic_left_string_right_string() {
        let mut left_expr = test_expr(ExprLogicalType::String);
        let mut right_expr = test_expr(ExprLogicalType::String);
        let result = coerce_arithmetic(
            &mut left_expr.logical_expr,
            &mut left_expr.expr_type,
            &mut right_expr.logical_expr,
            &mut right_expr.expr_type,
        );
        assert_eq!(result, None);
    }

    #[test]
    fn test_coerce_arithmetic_left_boolean_right_boolean() {
        let mut left_expr = test_expr(ExprLogicalType::Boolean);
        let mut right_expr = test_expr(ExprLogicalType::Boolean);
        let result = coerce_arithmetic(
            &mut left_expr.logical_expr,
            &mut left_expr.expr_type,
            &mut right_expr.logical_expr,
            &mut right_expr.expr_type,
        );
        assert_eq!(result, None);
    }

    #[test]
    fn test_coerce_arithmetic_left_any_value_numeric_right_any_value_numeric() {
        let mut left_expr = test_expr(ExprLogicalType::AnyValueNumeric);
        let mut right_expr = test_expr(ExprLogicalType::AnyValueNumeric);
        let result = coerce_arithmetic(
            &mut left_expr.logical_expr,
            &mut left_expr.expr_type,
            &mut right_expr.logical_expr,
            &mut right_expr.expr_type,
        );
        assert_eq!(result, Some(ExprLogicalType::AnyValueNumeric));
        assert_eq!(left_expr.expr_type, ExprLogicalType::AnyValueNumeric);
        assert_eq!(right_expr.expr_type, ExprLogicalType::AnyValueNumeric);
    }

    #[test]
    fn test_coerce_arithmetic_left_any_value_numeric_right_float64() {
        let mut left_expr = test_expr(ExprLogicalType::AnyValueNumeric);
        let mut right_expr = test_expr(ExprLogicalType::Float64);
        let result = coerce_arithmetic(
            &mut left_expr.logical_expr,
            &mut left_expr.expr_type,
            &mut right_expr.logical_expr,
            &mut right_expr.expr_type,
        );
        assert_eq!(result, Some(ExprLogicalType::Float64));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Float64);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Float64);
    }

    #[test]
    fn test_coerce_arithmetic_left_any_value_numeric_right_int64() {
        let mut left_expr = test_expr(ExprLogicalType::AnyValueNumeric);
        let mut right_expr = test_expr(ExprLogicalType::Int64);
        let result = coerce_arithmetic(
            &mut left_expr.logical_expr,
            &mut left_expr.expr_type,
            &mut right_expr.logical_expr,
            &mut right_expr.expr_type,
        );
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
        let result = coerce_arithmetic(
            &mut left_expr.logical_expr,
            &mut left_expr.expr_type,
            &mut right_expr.logical_expr,
            &mut right_expr.expr_type,
        );
        assert_eq!(result, Some(ExprLogicalType::UInt32));
        assert_eq!(left_expr.expr_type, ExprLogicalType::UInt32);
        assert_eq!(right_expr.expr_type, ExprLogicalType::UInt32);
    }

    #[test]
    fn test_coerce_arithmetic_uint8_plus_int32() {
        // Unsigned + signed, coerce to signed with same width (Int32)
        let mut left_expr = test_expr(ExprLogicalType::UInt8);
        let mut right_expr = test_expr(ExprLogicalType::Int32);
        let result = coerce_arithmetic(
            &mut left_expr.logical_expr,
            &mut left_expr.expr_type,
            &mut right_expr.logical_expr,
            &mut right_expr.expr_type,
        );
        assert_eq!(result, Some(ExprLogicalType::Int32));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Int32);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Int32);
    }

    #[test]
    fn test_coerce_arithmetic_uint32_plus_int32() {
        // Unsigned + signed with same width, need to upsize to avoid overflow (Int64)
        let mut left_expr = test_expr(ExprLogicalType::UInt32);
        let mut right_expr = test_expr(ExprLogicalType::Int32);
        let result = coerce_arithmetic(
            &mut left_expr.logical_expr,
            &mut left_expr.expr_type,
            &mut right_expr.logical_expr,
            &mut right_expr.expr_type,
        );
        assert_eq!(result, Some(ExprLogicalType::Int64));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Int64);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Int64);
    }

    #[test]
    fn test_coerce_arithmetic_int32_plus_uint32() {
        // Signed + unsigned with same width (reverse order), should give same result
        let mut left_expr = test_expr(ExprLogicalType::Int32);
        let mut right_expr = test_expr(ExprLogicalType::UInt32);
        let result = coerce_arithmetic(
            &mut left_expr.logical_expr,
            &mut left_expr.expr_type,
            &mut right_expr.logical_expr,
            &mut right_expr.expr_type,
        );
        assert_eq!(result, Some(ExprLogicalType::Int64));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Int64);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Int64);
    }

    #[test]
    fn test_coerce_arithmetic_uint8_plus_int64() {
        // Small unsigned + large signed, coerce to larger signed (Int64)
        let mut left_expr = test_expr(ExprLogicalType::UInt8);
        let mut right_expr = test_expr(ExprLogicalType::Int64);
        let result = coerce_arithmetic(
            &mut left_expr.logical_expr,
            &mut left_expr.expr_type,
            &mut right_expr.logical_expr,
            &mut right_expr.expr_type,
        );
        assert_eq!(result, Some(ExprLogicalType::Int64));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Int64);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Int64);
    }

    #[test]
    fn test_coerce_arithmetic_uint32_plus_int64() {
        // Unsigned 32 + signed 64, coerce to larger signed (Int64)
        let mut left_expr = test_expr(ExprLogicalType::UInt32);
        let mut right_expr = test_expr(ExprLogicalType::Int64);
        let result = coerce_arithmetic(
            &mut left_expr.logical_expr,
            &mut left_expr.expr_type,
            &mut right_expr.logical_expr,
            &mut right_expr.expr_type,
        );
        assert_eq!(result, Some(ExprLogicalType::Int64));
        assert_eq!(left_expr.expr_type, ExprLogicalType::Int64);
        assert_eq!(right_expr.expr_type, ExprLogicalType::Int64);
    }
}
