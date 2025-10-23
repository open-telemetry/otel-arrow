// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use data_engine_expressions::{
    BooleanValue, DoubleValue, IntegerValue, StaticScalarExpression, StringValue,
};
use datafusion::logical_expr::{Expr, lit};
use otel_arrow_rust::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_rust::schema::consts;

use crate::error::Result;

pub enum ColumnAccessor {
    ColumnName(String),

    /// column in a nested struct. for example resource.schema_url or instrumentation_scope.name
    StructCol(&'static str, String),

    /// payload type identifies which attributes are being joined
    /// and the string identifies the attribute key
    Attributes(AttributesIdentifier, String),
}

/// Identifier of a batch of attributes
pub enum AttributesIdentifier {
    /// Attributes for the root record type. E.g. LogAttrs for a batch of log records
    Root,

    /// Attributes for something that isn't the root record type. E.g. ScopeAttrs, ResourceAttrs
    NonRoot(ArrowPayloadType),
}

/// try to convert a static scalar expression into a DataFusion Expr::Lit
pub fn try_static_scalar_to_literal(static_scalar: &StaticScalarExpression) -> Result<Expr> {
    let lit_expr = match static_scalar {
        StaticScalarExpression::String(str_val) => lit(str_val.get_value()),
        StaticScalarExpression::Boolean(bool_val) => lit(bool_val.get_value()),
        StaticScalarExpression::Integer(int_val) => lit(int_val.get_value()),
        StaticScalarExpression::Double(float_val) => lit(float_val.get_value()),
        _ => {
            todo!("handle other value types")
        }
    };

    Ok(lit_expr)
}

/// try to get the column from an OTAP batch containing an AnyValue based on the value of some
/// defined static scalar.
pub fn try_static_scalar_to_any_val_column(
    static_scalar: &StaticScalarExpression,
) -> Result<&'static str> {
    let col_name = match static_scalar {
        StaticScalarExpression::Boolean(_) => consts::ATTRIBUTE_BOOL,
        StaticScalarExpression::Double(_) => consts::ATTRIBUTE_DOUBLE,
        StaticScalarExpression::Integer(_) => consts::ATTRIBUTE_INT,
        StaticScalarExpression::String(_) => consts::ATTRIBUTE_STR,
        _ => {
            todo!("handle other attribute columns for binary expr")
        }
    };

    Ok(col_name)
}
