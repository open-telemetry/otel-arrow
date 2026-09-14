// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OTLP attribute selection and Geneva metric context construction.

use std::cmp::Ordering;

use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{KeyValue, any_value};
use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::ScopeMetrics;

use super::super::encoder::Dimension;
use super::{CardinalityOverflow, Config, PointContext, ResourceContext};

const MAX_DIMENSIONS: usize = 74;
pub(super) const MAX_DIMENSION_NAME_CHARS: usize = 512;
pub(super) const MAX_DIMENSION_VALUE_CHARS: usize = 1024;

pub(super) const ACCOUNT_ATTRIBUTE: &str = "_microsoft_metrics_account";
const PREVIOUS_ACCOUNT_ATTRIBUTE: &str = "microsoft_metrics_account";
pub(super) const NAMESPACE_ATTRIBUTE: &str = "_microsoft_metrics_namespace";
const PREVIOUS_NAMESPACE_ATTRIBUTE: &str = "microsoft_metrics_namespace";
const CARDINALITY_OVERFLOW_ATTRIBUTE: &str = "otel.metric.overflow";

pub(super) fn resource_context(attributes: &[KeyValue], config: &Config) -> ResourceContext {
    let mut monitoring_account = None;
    let mut namespace = None;
    let mut dimensions = Vec::new();
    for attribute in attributes {
        match attribute.key.as_str() {
            ACCOUNT_ATTRIBUTE | PREVIOUS_ACCOUNT_ATTRIBUTE => {
                monitoring_account = Some(routing_value(attribute));
            }
            NAMESPACE_ATTRIBUTE | PREVIOUS_NAMESPACE_ATTRIBUTE => {
                namespace = Some(routing_value(attribute));
            }
            _ if selected_attribute(&attribute.key, &config.resource_attributes) => {
                add_dimension(
                    &mut dimensions,
                    attribute,
                    true,
                    config.honor_resource_attributes,
                );
            }
            _ => {}
        }
    }
    ResourceContext {
        monitoring_account: monitoring_account
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| config.monitoring_account.clone()),
        namespace: namespace
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| config.metric_namespace.clone()),
        dimensions,
    }
}

pub(super) fn point_context(
    attributes: &[KeyValue],
    resource: &ResourceContext,
    scope_namespace: &str,
    scope_dimensions: &[Dimension],
    config: &Config,
) -> Option<PointContext> {
    let mut monitoring_account = resource.monitoring_account.clone();
    let mut namespace = scope_namespace.to_string();
    let mut point_dimensions = Vec::new();
    for attribute in attributes {
        match attribute.key.as_str() {
            ACCOUNT_ATTRIBUTE => {
                monitoring_account = routing_value(attribute);
            }
            NAMESPACE_ATTRIBUTE => {
                namespace = routing_value(attribute);
            }
            CARDINALITY_OVERFLOW_ATTRIBUTE => {}
            _ => add_dimension(&mut point_dimensions, attribute, false, true),
        }
    }
    let mut dimensions = merge_dimensions(
        &point_dimensions,
        &resource.dimensions,
        scope_dimensions,
        config.honor_resource_attributes,
        config.honor_scope_attributes,
    );
    if !dimensions_within_limits(&dimensions) {
        return None;
    }
    dimensions.sort_by(compare_dimensions);
    Some(PointContext {
        monitoring_account,
        namespace,
        dimensions,
    })
}

pub(super) fn overflow_diagnostic(
    attributes: &[KeyValue],
    resource: &ResourceContext,
    scope_namespace: &str,
    metric_name: &str,
) -> Option<CardinalityOverflow> {
    let mut monitoring_account = resource.monitoring_account.clone();
    let mut namespace = scope_namespace.to_string();
    let mut cardinality_overflow = false;
    for attribute in attributes {
        match attribute.key.as_str() {
            ACCOUNT_ATTRIBUTE => {
                monitoring_account = routing_value(attribute);
            }
            NAMESPACE_ATTRIBUTE => {
                namespace = routing_value(attribute);
            }
            CARDINALITY_OVERFLOW_ATTRIBUTE => {
                cardinality_overflow = matches!(
                    attribute
                        .value
                        .as_ref()
                        .and_then(|value| value.value.as_ref()),
                    Some(any_value::Value::BoolValue(true))
                );
            }
            _ => {}
        }
    }
    cardinality_overflow.then(|| CardinalityOverflow {
        monitoring_account,
        namespace,
        metric_name: metric_name.to_string(),
    })
}

fn add_dimension(
    dimensions: &mut Vec<Dimension>,
    attribute: &KeyValue,
    string_only: bool,
    overwrite_duplicate: bool,
) {
    let value =
        attribute
            .value
            .as_ref()
            .map_or_else(String::new, |value| match value.value.as_ref() {
                Some(any_value::Value::StringValue(value)) => value.clone(),
                Some(any_value::Value::BoolValue(value)) if !string_only => {
                    if *value {
                        "True".to_string()
                    } else {
                        "False".to_string()
                    }
                }
                Some(any_value::Value::IntValue(value)) if !string_only => value.to_string(),
                Some(any_value::Value::DoubleValue(value)) if !string_only => {
                    format_double_dimension(*value)
                }
                _ => String::new(),
            });
    if let Some(existing) = dimensions
        .iter_mut()
        .find(|dimension| dimension.name.eq_ignore_ascii_case(&attribute.key))
    {
        if overwrite_duplicate && !existing.value.eq_ignore_ascii_case(&value) {
            existing.value = value;
        }
        return;
    }
    dimensions.push(Dimension {
        name: attribute.key.clone(),
        value,
    });
}

pub(super) fn selected_scope_dimensions(
    scope_metrics: &ScopeMetrics,
    config: &Config,
) -> Vec<Dimension> {
    let Some(scope) = scope_metrics
        .scope
        .as_ref()
        .filter(|scope| !scope.name.is_empty())
    else {
        return Vec::new();
    };
    let wildcard_selection = config
        .scope_attributes
        .iter()
        .find(|selection| selection.name == "*");
    let selected_keys = if let Some(selection) = wildcard_selection {
        selection
            .keys
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
    } else {
        config
            .scope_attributes
            .iter()
            .filter(|selection| selection.name == scope.name)
            .flat_map(|selection| selection.keys.iter())
            .map(String::as_str)
            .collect::<Vec<_>>()
    };
    let mut dimensions = Vec::new();
    for attribute in &scope.attributes {
        if attribute.key != NAMESPACE_ATTRIBUTE
            && selected_keys
                .iter()
                .any(|candidate| *candidate == "*" || *candidate == attribute.key.as_str())
        {
            add_dimension(
                &mut dimensions,
                attribute,
                true,
                config.honor_scope_attributes,
            );
        }
    }
    dimensions
}

fn format_double_dimension(value: f64) -> String {
    if value.is_nan() {
        return "nan".to_string();
    }
    if value == f64::INFINITY {
        return "inf".to_string();
    }
    if value == f64::NEG_INFINITY {
        return "-inf".to_string();
    }
    if value.abs() >= 1e6 {
        let formatted = format!("{value:.6e}");
        let (mantissa, exponent) = formatted
            .split_once('e')
            .expect("scientific formatting always includes an exponent");
        let exponent = exponent
            .parse::<i32>()
            .expect("a formatted f64 exponent always fits i32");
        return format!("{mantissa}e{exponent:+03}");
    }

    let mut formatted = format!("{value:.6}");
    while formatted.len() > 1 && (formatted.ends_with('0') || formatted.ends_with('.')) {
        let _ = formatted.pop();
    }
    formatted
}

fn selected_attribute(key: &str, selected: &[String]) -> bool {
    selected
        .iter()
        .any(|candidate| candidate == "*" || candidate == key)
}

fn merge_dimensions(
    point: &[Dimension],
    resource: &[Dimension],
    scope: &[Dimension],
    honor_resource: bool,
    honor_scope: bool,
) -> Vec<Dimension> {
    let precedence = match (honor_resource, honor_scope) {
        (false, false) => [resource, scope, point],
        (true, false) => [scope, point, resource],
        (false, true) => [resource, point, scope],
        (true, true) => [point, resource, scope],
    };
    let mut merged = Vec::new();
    for dimensions in precedence {
        for dimension in dimensions {
            set_dimension(&mut merged, dimension);
        }
    }
    merged
}

fn set_dimension(dimensions: &mut Vec<Dimension>, dimension: &Dimension) {
    if let Some(existing) = dimensions
        .iter_mut()
        .find(|existing| existing.name.eq_ignore_ascii_case(&dimension.name))
    {
        if !existing.value.eq_ignore_ascii_case(&dimension.value) {
            existing.value.clone_from(&dimension.value);
        }
    } else {
        dimensions.push(dimension.clone());
    }
}

fn dimensions_within_limits(dimensions: &[Dimension]) -> bool {
    dimensions.len() <= MAX_DIMENSIONS
        && dimensions.iter().all(|dimension| {
            dimension.name.chars().count() <= MAX_DIMENSION_NAME_CHARS
                && dimension.value.chars().count() <= MAX_DIMENSION_VALUE_CHARS
        })
}

fn compare_dimensions(left: &Dimension, right: &Dimension) -> Ordering {
    left.name
        .to_lowercase()
        .cmp(&right.name.to_lowercase())
        .then_with(|| left.value.to_lowercase().cmp(&right.value.to_lowercase()))
}

pub(super) fn attribute_string(attributes: &[KeyValue], key: &str) -> Option<String> {
    attributes
        .iter()
        .rev()
        .find(|attribute| attribute.key == key)
        .map(routing_value)
        .filter(|value| !value.is_empty())
}

pub(super) fn string_value(attribute: &KeyValue) -> Option<String> {
    match attribute.value.as_ref()?.value.as_ref()? {
        any_value::Value::StringValue(value) => Some(value.clone()),
        _ => None,
    }
}

fn routing_value(attribute: &KeyValue) -> String {
    string_value(attribute).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{AnyValue, InstrumentationScope};

    use super::super::ResourceContext;
    use super::*;
    use crate::exporters::geneva_metrics_exporter::{Config, ScopeAttributes, encoder::Dimension};

    fn config() -> Config {
        Config {
            monitoring_account: "default-account".to_string(),
            metric_namespace: "default-namespace".to_string(),
            resource_attributes: Vec::new(),
            honor_resource_attributes: false,
            scope_attributes: Vec::new(),
            honor_scope_attributes: false,
            disable_exemplars: false,
        }
    }

    fn attribute(key: &str, value: any_value::Value) -> KeyValue {
        KeyValue {
            key: key.to_string(),
            value: Some(AnyValue { value: Some(value) }),
        }
    }

    fn string_attribute(key: &str, value: &str) -> KeyValue {
        attribute(key, any_value::Value::StringValue(value.to_string()))
    }

    fn double_attribute(key: &str, value: f64) -> KeyValue {
        attribute(key, any_value::Value::DoubleValue(value))
    }

    fn dimension(name: &str, value: &str) -> Dimension {
        Dimension {
            name: name.to_string(),
            value: value.to_string(),
        }
    }

    fn resource_with_dimensions(dimensions: Vec<Dimension>) -> ResourceContext {
        ResourceContext {
            monitoring_account: "resource-account".to_string(),
            namespace: "resource-namespace".to_string(),
            dimensions,
        }
    }

    fn scope_metrics(name: &str, attributes: Vec<KeyValue>) -> ScopeMetrics {
        ScopeMetrics {
            scope: Some(InstrumentationScope {
                name: name.to_string(),
                version: String::new(),
                attributes,
                dropped_attributes_count: 0,
            }),
            metrics: Vec::new(),
            schema_url: String::new(),
        }
    }

    /// Scenario: A resource has no routing or dimension attributes.
    /// Guarantees: Configured account and namespace defaults are retained with a valid empty dimension set.
    #[test]
    fn uses_configured_resource_defaults() {
        let context = resource_context(&[], &config());

        assert_eq!(context.monitoring_account, "default-account");
        assert_eq!(context.namespace, "default-namespace");
        assert!(context.dimensions.is_empty());
    }

    /// Scenario: Resource routing uses either the current or legacy account and namespace attribute names.
    /// Guarantees: Both naming generations override the configured routing defaults.
    #[test]
    fn accepts_current_and_legacy_resource_routing_attributes() {
        let mapping_config = config();
        let legacy = resource_context(
            &[
                string_attribute(PREVIOUS_ACCOUNT_ATTRIBUTE, "legacy-account"),
                string_attribute(PREVIOUS_NAMESPACE_ATTRIBUTE, "legacy-namespace"),
            ],
            &mapping_config,
        );
        let current = resource_context(
            &[
                string_attribute(ACCOUNT_ATTRIBUTE, "current-account"),
                string_attribute(NAMESPACE_ATTRIBUTE, "current-namespace"),
            ],
            &mapping_config,
        );

        assert_eq!(legacy.monitoring_account, "legacy-account");
        assert_eq!(legacy.namespace, "legacy-namespace");
        assert_eq!(current.monitoring_account, "current-account");
        assert_eq!(current.namespace, "current-namespace");
    }

    /// Scenario: Final resource routing attributes are empty or non-string after earlier non-empty values.
    /// Guarantees: ME routing falls back to the configured monitoring account and namespace.
    #[test]
    fn falls_back_from_empty_resource_routing_values() {
        let mapping_config = config();
        let context = resource_context(
            &[
                string_attribute(ACCOUNT_ATTRIBUTE, "ignored-account"),
                attribute(ACCOUNT_ATTRIBUTE, any_value::Value::IntValue(1)),
                string_attribute(NAMESPACE_ATTRIBUTE, "ignored-namespace"),
                string_attribute(NAMESPACE_ATTRIBUTE, ""),
            ],
            &mapping_config,
        );

        assert_eq!(context.monitoring_account, "default-account");
        assert_eq!(context.namespace, "default-namespace");
    }

    /// Scenario: Resource attributes contain selected and unselected string dimensions.
    /// Guarantees: Only explicitly selected resource attributes become Geneva dimensions.
    #[test]
    fn selects_configured_resource_dimensions() {
        let mut mapping_config = config();
        mapping_config.resource_attributes = vec!["region".to_string()];

        let context = resource_context(
            &[
                string_attribute("region", "west"),
                string_attribute("ignored", "value"),
            ],
            &mapping_config,
        );
        assert_eq!(context.dimensions, vec![dimension("region", "west")]);
    }

    /// Scenario: A point supplies destination overrides alongside ordinary dimensions.
    /// Guarantees: Routing attributes update account and namespace without being emitted as dimensions.
    #[test]
    fn applies_point_routing_attributes_without_emitting_them() {
        let resource = resource_with_dimensions(Vec::new());
        let point = point_context(
            &[
                string_attribute(ACCOUNT_ATTRIBUTE, "point-account"),
                string_attribute(NAMESPACE_ATTRIBUTE, "point-namespace"),
                string_attribute("region", "west"),
            ],
            &resource,
            "scope-namespace",
            &[],
            &config(),
        )
        .expect("point context should be valid");

        assert_eq!(point.monitoring_account, "point-account");
        assert_eq!(point.namespace, "point-namespace");
        assert_eq!(point.dimensions, vec![dimension("region", "west")]);
    }

    /// Scenario: Point routing attributes contain non-string OTLP values.
    /// Guarantees: Protobuf string-value semantics apply empty account and namespace overrides instead of retaining parent routing.
    #[test]
    fn applies_empty_point_overrides_for_non_string_routing_values() {
        let point = point_context(
            &[
                string_attribute(ACCOUNT_ATTRIBUTE, "ignored-account"),
                attribute(ACCOUNT_ATTRIBUTE, any_value::Value::IntValue(1)),
                string_attribute(NAMESPACE_ATTRIBUTE, "ignored-namespace"),
                attribute(NAMESPACE_ATTRIBUTE, any_value::Value::BoolValue(true)),
            ],
            &resource_with_dimensions(Vec::new()),
            "scope-namespace",
            &[],
            &config(),
        )
        .expect("point context should be valid");

        assert!(point.monitoring_account.is_empty());
        assert!(point.namespace.is_empty());
        assert!(point.dimensions.is_empty());
    }

    /// Scenario: A scope namespace attribute is empty or non-string.
    /// Guarantees: Scope routing returns no override so the caller can retain the resource or configured namespace.
    #[test]
    fn ignores_empty_scope_namespace_overrides() {
        for attribute in [
            string_attribute(NAMESPACE_ATTRIBUTE, ""),
            attribute(NAMESPACE_ATTRIBUTE, any_value::Value::BoolValue(true)),
        ] {
            assert_eq!(attribute_string(&[attribute], NAMESPACE_ATTRIBUTE), None);
        }
    }

    /// Scenario: Point, resource, and scope dimensions use the same name with different casing.
    /// Guarantees: Every honor-flag combination selects the documented case-insensitive precedence winner.
    #[test]
    fn applies_all_dimension_precedence_modes() {
        let point = [dimension("SHARED", "point")];
        let resource = [dimension("shared", "resource")];
        let scope = [dimension("Shared", "scope")];

        for (honor_resource, honor_scope, expected) in [
            (false, false, "point"),
            (true, false, "resource"),
            (false, true, "scope"),
            (true, true, "scope"),
        ] {
            let merged = merge_dimensions(&point, &resource, &scope, honor_resource, honor_scope);

            assert_eq!(merged.len(), 1);
            assert_eq!(merged[0].value, expected);
        }
    }

    /// Scenario: An honored resource value differs from the existing point value only by casing.
    /// Guarantees: ME's case-insensitive duplicate check preserves the first serialized value.
    #[test]
    fn preserves_point_value_when_honored_resource_differs_only_by_case() {
        let point = [dimension("REGION", "WEST")];
        let resource = [dimension("region", "west")];

        let merged = merge_dimensions(&point, &resource, &[], true, false);

        assert_eq!(merged, point);
    }

    /// Scenario: Point, resource, and scope dimensions arrive in unrelated lexical order.
    /// Guarantees: The merged dimensions are sorted case-insensitively by name and then value.
    #[test]
    fn sorts_merged_dimensions_case_insensitively() {
        let resource = resource_with_dimensions(vec![dimension("zeta", "resource")]);
        let scope = [dimension("Beta", "scope")];
        let point = point_context(
            &[
                string_attribute("alpha", "second"),
                string_attribute("ALPHA", "first"),
            ],
            &resource,
            "scope-namespace",
            &scope,
            &config(),
        )
        .expect("point context should be valid");

        assert_eq!(
            point.dimensions,
            vec![
                dimension("alpha", "first"),
                dimension("Beta", "scope"),
                dimension("zeta", "resource"),
            ]
        );
    }

    /// Scenario: Case-insensitive duplicate point attributes carry different values.
    /// Guarantees: The final OTLP occurrence replaces the earlier value while retaining one dimension.
    #[test]
    fn uses_last_duplicate_point_dimension_value() {
        let point = point_context(
            &[
                string_attribute("region", "east"),
                string_attribute("REGION", "west"),
            ],
            &resource_with_dimensions(Vec::new()),
            "scope-namespace",
            &[],
            &config(),
        )
        .expect("point context should be valid");

        assert_eq!(point.dimensions, vec![dimension("region", "west")]);
    }

    /// Scenario: Duplicate point values differ only by casing.
    /// Guarantees: ME retains the first value instead of replacing it with a case-equivalent value.
    #[test]
    fn preserves_first_case_equivalent_point_dimension_value() {
        let point = point_context(
            &[
                string_attribute("region", "WEST"),
                string_attribute("REGION", "west"),
            ],
            &resource_with_dimensions(Vec::new()),
            "scope-namespace",
            &[],
            &config(),
        )
        .expect("point context should be valid");

        assert_eq!(point.dimensions, vec![dimension("region", "WEST")]);
    }

    /// Scenario: Cardinality-overflow metadata is false or has a non-boolean value.
    /// Guarantees: The reserved attribute is never emitted as a dimension and only boolean true raises the diagnostic marker.
    #[test]
    fn reserves_cardinality_overflow_attribute() {
        for attribute in [
            attribute(
                CARDINALITY_OVERFLOW_ATTRIBUTE,
                any_value::Value::BoolValue(false),
            ),
            string_attribute(CARDINALITY_OVERFLOW_ATTRIBUTE, "true"),
        ] {
            let diagnostic = overflow_diagnostic(
                std::slice::from_ref(&attribute),
                &resource_with_dimensions(Vec::new()),
                "scope-namespace",
                "metric",
            );
            let point = point_context(
                &[attribute],
                &resource_with_dimensions(Vec::new()),
                "scope-namespace",
                &[],
                &config(),
            )
            .expect("point context should be valid");

            assert!(point.dimensions.is_empty());
            assert!(diagnostic.is_none());
        }
    }

    /// Scenario: Double point attributes cover fixed, scientific, and special floating-point values.
    /// Guarantees: Dimension values use ME's six-decimal locale-independent formatting contract.
    #[test]
    fn formats_double_dimensions_like_me() {
        let point = point_context(
            &[
                double_attribute("fixed", 12_345.678_9),
                double_attribute("rounded", 1.234_567_89),
                double_attribute("scientific", 1_234_567_890_000_000.0),
                double_attribute("negative-zero", -0.0),
                double_attribute("nan", f64::NAN),
                double_attribute("positive-infinity", f64::INFINITY),
                double_attribute("negative-infinity", f64::NEG_INFINITY),
            ],
            &resource_with_dimensions(Vec::new()),
            "scope-namespace",
            &[],
            &config(),
        )
        .expect("point context should be valid");

        assert_eq!(
            point.dimensions,
            vec![
                dimension("fixed", "12345.6789"),
                dimension("nan", "nan"),
                dimension("negative-infinity", "-inf"),
                dimension("negative-zero", "-"),
                dimension("positive-infinity", "inf"),
                dimension("rounded", "1.234568"),
                dimension("scientific", "1.234568e+15"),
            ]
        );
    }

    /// Scenario: Merged resource and point attributes exceed the Geneva dimension limit.
    /// Guarantees: A point with more than 74 dimensions is rejected before encoding.
    #[test]
    fn rejects_points_above_dimension_count_limit() {
        let resource = resource_with_dimensions(
            (0..MAX_DIMENSIONS)
                .map(|index| dimension(&format!("resource-{index}"), "value"))
                .collect(),
        );

        let point = point_context(
            &[string_attribute("point", "value")],
            &resource,
            "scope-namespace",
            &[],
            &config(),
        );

        assert!(point.is_none());
    }

    /// Scenario: A point attribute name exceeds the Geneva UTF-16 character limit.
    /// Guarantees: The invalid dimension causes only that point context to be rejected.
    #[test]
    fn rejects_oversized_point_dimension_name() {
        let point = point_context(
            &[string_attribute(
                &"n".repeat(MAX_DIMENSION_NAME_CHARS + 1),
                "value",
            )],
            &resource_with_dimensions(Vec::new()),
            "scope-namespace",
            &[],
            &config(),
        );

        assert!(point.is_none());
    }

    /// Scenario: A point attribute value exceeds the Geneva UTF-16 character limit.
    /// Guarantees: The invalid dimension causes only that point context to be rejected.
    #[test]
    fn rejects_oversized_point_dimension_value() {
        let point = point_context(
            &[string_attribute(
                "name",
                &"v".repeat(MAX_DIMENSION_VALUE_CHARS + 1),
            )],
            &resource_with_dimensions(Vec::new()),
            "scope-namespace",
            &[],
            &config(),
        );

        assert!(point.is_none());
    }

    /// Scenario: Oversized resource and scope values are replaced by valid point values.
    /// Guarantees: Dimension limits apply after ME precedence, so discarded parent values do not reject the point.
    #[test]
    fn accepts_valid_point_overrides_for_oversized_parent_values() {
        let oversized = "v".repeat(MAX_DIMENSION_VALUE_CHARS + 1);
        let mut mapping_config = config();
        mapping_config.resource_attributes = vec!["resource-shared".to_string()];
        mapping_config.scope_attributes = vec![ScopeAttributes {
            name: "meter".to_string(),
            keys: vec!["scope-shared".to_string()],
        }];
        let resource = resource_context(
            &[string_attribute("resource-shared", &oversized)],
            &mapping_config,
        );
        let scope = scope_metrics("meter", vec![string_attribute("scope-shared", &oversized)]);
        let scope_dimensions = selected_scope_dimensions(&scope, &mapping_config);

        let point = point_context(
            &[
                string_attribute("resource-shared", "point-resource"),
                string_attribute("scope-shared", "point-scope"),
            ],
            &resource,
            "scope-namespace",
            &scope_dimensions,
            &mapping_config,
        )
        .expect("overridden oversized values should not reject the point");

        assert_eq!(
            point.dimensions,
            vec![
                dimension("resource-shared", "point-resource"),
                dimension("scope-shared", "point-scope"),
            ]
        );
    }

    /// Scenario: An exact scope selection includes one dimension and the reserved namespace attribute.
    /// Guarantees: Selected dimensions are retained while the namespace routing attribute is never emitted.
    #[test]
    fn selects_exact_scope_dimensions_and_excludes_namespace() {
        let mut mapping_config = config();
        mapping_config.scope_attributes = vec![ScopeAttributes {
            name: "meter".to_string(),
            keys: vec!["region".to_string(), NAMESPACE_ATTRIBUTE.to_string()],
        }];
        let scope = scope_metrics(
            "meter",
            vec![
                string_attribute("region", "west"),
                string_attribute("ignored", "value"),
                string_attribute(NAMESPACE_ATTRIBUTE, "scope-namespace"),
            ],
        );

        let dimensions = selected_scope_dimensions(&scope, &mapping_config);

        assert_eq!(dimensions, vec![dimension("region", "west")]);
    }

    /// Scenario: A wildcard scope selector and wildcard key selector receive an arbitrary scope.
    /// Guarantees: All non-routing string attributes from that scope become dimensions.
    #[test]
    fn applies_wildcard_scope_dimension_selection() {
        let mut mapping_config = config();
        mapping_config.scope_attributes = vec![ScopeAttributes {
            name: "*".to_string(),
            keys: vec!["*".to_string()],
        }];
        let scope = scope_metrics(
            "arbitrary-meter",
            vec![
                string_attribute("region", "west"),
                string_attribute("service", "checkout"),
            ],
        );

        let dimensions = selected_scope_dimensions(&scope, &mapping_config);

        assert_eq!(
            dimensions,
            vec![
                dimension("region", "west"),
                dimension("service", "checkout"),
            ]
        );
    }

    /// Scenario: Multiple scope configuration entries target the same instrumentation scope.
    /// Guarantees: Their selected key sets are unioned so no later configured dimension is lost.
    #[test]
    fn merges_repeated_scope_attribute_selections() {
        let mut mapping_config = config();
        mapping_config.scope_attributes = vec![
            ScopeAttributes {
                name: "meter".to_string(),
                keys: vec!["region".to_string()],
            },
            ScopeAttributes {
                name: "meter".to_string(),
                keys: vec!["service".to_string()],
            },
        ];
        let scope = scope_metrics(
            "meter",
            vec![
                string_attribute("region", "west"),
                string_attribute("service", "checkout"),
            ],
        );

        let dimensions = selected_scope_dimensions(&scope, &mapping_config);

        assert_eq!(
            dimensions,
            vec![
                dimension("region", "west"),
                dimension("service", "checkout"),
            ]
        );
    }

    /// Scenario: Exact scope selectors appear before and after a wildcard scope selector.
    /// Guarantees: The first wildcard replaces exact selections and later entries are ignored like ME configuration loading.
    #[test]
    fn wildcard_scope_selection_replaces_exact_entries() {
        let mut mapping_config = config();
        mapping_config.scope_attributes = vec![
            ScopeAttributes {
                name: "meter".to_string(),
                keys: vec!["exact-before".to_string()],
            },
            ScopeAttributes {
                name: "*".to_string(),
                keys: vec!["global".to_string()],
            },
            ScopeAttributes {
                name: "*".to_string(),
                keys: vec!["ignored-global".to_string()],
            },
            ScopeAttributes {
                name: "meter".to_string(),
                keys: vec!["exact-after".to_string()],
            },
        ];
        let scope = scope_metrics(
            "meter",
            vec![
                string_attribute("exact-before", "value"),
                string_attribute("global", "value"),
                string_attribute("ignored-global", "value"),
                string_attribute("exact-after", "value"),
            ],
        );

        let dimensions = selected_scope_dimensions(&scope, &mapping_config);

        assert_eq!(dimensions, vec![dimension("global", "value")]);
    }

    /// Scenario: No configured scope selector matches the instrumentation scope.
    /// Guarantees: Mapping continues with no scope dimensions rather than rejecting the point.
    #[test]
    fn ignores_unselected_scope_dimensions() {
        let mut mapping_config = config();
        mapping_config.scope_attributes = vec![ScopeAttributes {
            name: "other-meter".to_string(),
            keys: vec!["region".to_string()],
        }];
        let scope = scope_metrics("meter", vec![string_attribute("region", "west")]);

        let dimensions = selected_scope_dimensions(&scope, &mapping_config);

        assert!(dimensions.is_empty());
    }

    /// Scenario: Duplicate routing attributes provide different string values.
    /// Guarantees: Attribute lookup uses the final occurrence, matching OTLP attribute precedence.
    #[test]
    fn uses_last_duplicate_attribute_value() {
        let attributes = vec![
            string_attribute(NAMESPACE_ATTRIBUTE, "first"),
            string_attribute(NAMESPACE_ATTRIBUTE, "last"),
        ];

        assert_eq!(
            attribute_string(&attributes, NAMESPACE_ATTRIBUTE),
            Some("last".to_string())
        );
    }
}
