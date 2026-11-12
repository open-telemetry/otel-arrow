// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Translation from `policy-rs` types to query-engine expression AST nodes.
//!
//! Converts [`TypedCheck`] and [`ExistenceCheck`] structures from the
//! compiled policy snapshot into [`LogicalExpression`] AST nodes that the
//! query-engine's [`PipelineExpr`] can plan and evaluate.

use otel_arrow_contrib_data_engine_expressions::{
    BooleanScalarExpression, DoubleScalarExpression, EqualToLogicalExpression,
    GreaterThanLogicalExpression, GreaterThanOrEqualToLogicalExpression, IntegerScalarExpression,
    LogicalExpression, NotLogicalExpression, NullScalarExpression, QueryLocation,
    ScalarExpression, SourceScalarExpression, StaticScalarExpression, StringScalarExpression,
    ValueAccessor,
};
use policy_rs::LogFieldSelector;
use policy_rs::engine::{
    CompiledNumericValue, CompiledTypedMatcher, CompiledValue, ExistenceCheck, LogSignal,
    TypedCheck,
};
use policy_rs::proto::tero::policy::v1::LogField;

/// Translate a [`TypedCheck`] into a [`LogicalExpression`].
///
/// Maps each `CompiledTypedMatcher` variant to the corresponding AST node:
/// - `Equals(Bool/Int/Double)` -> `EqualTo`
/// - `Gt` -> `GreaterThan`
/// - `Gte` -> `GreaterThanOrEqualTo`
/// - `Lt` -> `GreaterThan` with swapped operands
/// - `Lte` -> `GreaterThanOrEqualTo` with swapped operands
///
/// # Panics
///
/// Does not panic. Unsupported matchers (e.g., `Equals(Bytes)`) return `None`.
pub(crate) fn typed_check_to_logical(
    check: &TypedCheck<LogSignal>,
) -> Option<LogicalExpression> {

    let field_expr = field_selector_to_scalar(&check.field);

    let expr = match &check.matcher {
        CompiledTypedMatcher::Equals(value) => match value {
            CompiledValue::Bool(v) => Some(LogicalExpression::EqualTo(
                EqualToLogicalExpression::new(
                    loc(),
                    field_expr,
                    ScalarExpression::Static(StaticScalarExpression::Boolean(
                        BooleanScalarExpression::new(loc(), *v),
                    )),
                    false,
                ),
            )),
            CompiledValue::Int(v) => Some(LogicalExpression::EqualTo(
                EqualToLogicalExpression::new(
                    loc(),
                    field_expr,
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(loc(), *v),
                    )),
                    false,
                ),
            )),
            CompiledValue::Double(v) => Some(LogicalExpression::EqualTo(
                EqualToLogicalExpression::new(
                    loc(),
                    field_expr,
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(loc(), *v),
                    )),
                    false,
                ),
            )),
            // TODO: support Equals(Bytes) -- requires hex/bytes comparison
            CompiledValue::Bytes(_) => None,
        },
        CompiledTypedMatcher::Gt(nv) => Some(LogicalExpression::GreaterThan(
            GreaterThanLogicalExpression::new(loc(), field_expr, numeric_value_to_scalar(nv)),
        )),
        CompiledTypedMatcher::Gte(nv) => Some(LogicalExpression::GreaterThanOrEqualTo(
            GreaterThanOrEqualToLogicalExpression::new(
                loc(),
                field_expr,
                numeric_value_to_scalar(nv),
            ),
        )),
        // Lt(threshold): field < threshold  =>  threshold > field  (swap operands)
        CompiledTypedMatcher::Lt(nv) => Some(LogicalExpression::GreaterThan(
            GreaterThanLogicalExpression::new(loc(), numeric_value_to_scalar(nv), field_expr),
        )),
        // Lte(threshold): field <= threshold  =>  threshold >= field  (swap operands)
        CompiledTypedMatcher::Lte(nv) => Some(LogicalExpression::GreaterThanOrEqualTo(
            GreaterThanOrEqualToLogicalExpression::new(
                loc(),
                numeric_value_to_scalar(nv),
                field_expr,
            ),
        )),
    };

    // Apply negation if needed
    match (expr, check.is_negated) {
        (Some(e), true) => Some(LogicalExpression::Not(NotLogicalExpression::new(loc(), e))),
        (Some(e), false) => Some(e),
        (None, _) => None,
    }
}

/// Translate an [`ExistenceCheck`] into a [`LogicalExpression`].
///
/// - `should_exist: true` -> `NOT(field == null)` (field is not null)
/// - `should_exist: false` -> `field == null` (field is null)
pub(crate) fn existence_check_to_logical(
    check: &ExistenceCheck<LogSignal>,
) -> LogicalExpression {
    let field_expr = field_selector_to_scalar(&check.field);

    let is_null = LogicalExpression::EqualTo(EqualToLogicalExpression::new(
        loc(),
        field_expr,
        ScalarExpression::Static(StaticScalarExpression::Null(NullScalarExpression::new(
            loc(),
        ))),
        false,
    ));

    let base = if check.should_exist {
        // "field exists" = "field is NOT null"
        LogicalExpression::Not(NotLogicalExpression::new(loc(), is_null))
    } else {
        // "field does not exist" = "field IS null"
        is_null
    };

    // Apply negation if needed
    if check.is_negated {
        LogicalExpression::Not(NotLogicalExpression::new(loc(), base))
    } else {
        base
    }
}

/// Convert a [`LogFieldSelector`] to a [`ScalarExpression`] representing
/// field access in the query-engine's AST.
fn field_selector_to_scalar(field: &LogFieldSelector) -> ScalarExpression {
    let selectors = match field {
        LogFieldSelector::Simple(log_field) => {
            vec![str_selector(simple_field_name(log_field))]
        }
        LogFieldSelector::LogAttribute(path) => {
            let mut sels = vec![str_selector("attributes")];
            for segment in path {
                sels.push(str_selector(segment));
            }
            sels
        }
        LogFieldSelector::ResourceAttribute(path) => {
            let mut sels = vec![str_selector("resource"), str_selector("attributes")];
            for segment in path {
                sels.push(str_selector(segment));
            }
            sels
        }
        LogFieldSelector::ScopeAttribute(path) => {
            let mut sels = vec![
                str_selector("instrumentation_scope"),
                str_selector("attributes"),
            ];
            for segment in path {
                sels.push(str_selector(segment));
            }
            sels
        }
    };

    ScalarExpression::Source(SourceScalarExpression::new(
        loc(),
        ValueAccessor::new_with_selectors(selectors),
    ))
}

/// Map a `LogField` enum variant to its OTAP Arrow column name.
fn simple_field_name(field: &LogField) -> &'static str {
    match field {
        LogField::Body => "body",
        LogField::SeverityText => "severity_text",
        LogField::TraceId => "trace_id",
        LogField::SpanId => "span_id",
        LogField::EventName => "event_name",
        // Schema URLs are struct fields on resource/scope, but the policy spec
        // treats them as simple fields. The query-engine accesses them as
        // resource.schema_url / instrumentation_scope.schema_url. For now we
        // map them to the top-level schema_url column since both resource and
        // scope share the same column name in the root record batch.
        // TODO: verify this mapping handles the resource vs scope distinction
        LogField::ResourceSchemaUrl | LogField::ScopeSchemaUrl => "schema_url",
        LogField::Unspecified => "body", // fallback, should not occur
    }
}

/// Convert a `CompiledNumericValue` to a `ScalarExpression` literal.
fn numeric_value_to_scalar(nv: &CompiledNumericValue) -> ScalarExpression {
    match nv {
        CompiledNumericValue::Int(v) => ScalarExpression::Static(
            StaticScalarExpression::Integer(IntegerScalarExpression::new(loc(), *v)),
        ),
        CompiledNumericValue::Double(v) => ScalarExpression::Static(
            StaticScalarExpression::Double(DoubleScalarExpression::new(loc(), *v)),
        ),
    }
}

/// Helper: create a string selector for a `ValueAccessor` path segment.
fn str_selector(s: &str) -> ScalarExpression {
    ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
        loc(),
        s,
    )))
}

/// Convenience: create a fake `QueryLocation` for programmatic AST construction.
fn loc() -> QueryLocation {
    QueryLocation::new_fake()
}
