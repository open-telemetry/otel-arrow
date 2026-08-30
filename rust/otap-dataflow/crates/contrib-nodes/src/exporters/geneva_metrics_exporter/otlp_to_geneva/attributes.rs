// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OTLP attribute selection and Geneva metric context construction.

use std::cmp::Ordering;

use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{KeyValue, any_value};
use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::ScopeMetrics;

use super::super::encoder::Dimension;
use super::{Config, PointContext, ResourceContext};

const MAX_DIMENSIONS: usize = 74;
pub(super) const MAX_DIMENSION_NAME_CHARS: usize = 512;
const MAX_DIMENSION_VALUE_CHARS: usize = 1024;

pub(super) const ACCOUNT_ATTRIBUTE: &str = "_microsoft_metrics_account";
const PREVIOUS_ACCOUNT_ATTRIBUTE: &str = "microsoft_metrics_account";
pub(super) const NAMESPACE_ATTRIBUTE: &str = "_microsoft_metrics_namespace";
const PREVIOUS_NAMESPACE_ATTRIBUTE: &str = "microsoft_metrics_namespace";

pub(super) fn resource_context(attributes: &[KeyValue], config: &Config) -> ResourceContext {
    let mut monitoring_account = config.monitoring_account.clone();
    let mut namespace = config.metric_namespace.clone();
    let mut dimensions = Vec::new();
    let mut dimensions_valid = true;
    for attribute in attributes {
        match attribute.key.as_str() {
            ACCOUNT_ATTRIBUTE | PREVIOUS_ACCOUNT_ATTRIBUTE => {
                if let Some(value) = string_value(attribute) {
                    monitoring_account = value;
                }
            }
            NAMESPACE_ATTRIBUTE | PREVIOUS_NAMESPACE_ATTRIBUTE => {
                if let Some(value) = string_value(attribute) {
                    namespace = value;
                }
            }
            _ if selected_attribute(&attribute.key, &config.resource_attributes)
                && !add_dimension(
                    &mut dimensions,
                    attribute,
                    true,
                    config.honor_resource_attributes,
                ) =>
            {
                dimensions_valid = false;
                break;
            }
            _ => {}
        }
    }
    ResourceContext {
        monitoring_account,
        namespace,
        dimensions,
        dimensions_valid,
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
                if let Some(value) = string_value(attribute) {
                    monitoring_account = value;
                }
            }
            NAMESPACE_ATTRIBUTE => {
                if let Some(value) = string_value(attribute) {
                    namespace = value;
                }
            }
            _ if !add_dimension(&mut point_dimensions, attribute, false, false) => return None,
            _ => {}
        }
    }
    let mut dimensions = merge_dimensions(
        &point_dimensions,
        &resource.dimensions,
        scope_dimensions,
        config.honor_resource_attributes,
        config.honor_scope_attributes,
    );
    if dimensions.len() > MAX_DIMENSIONS {
        return None;
    }
    dimensions.sort_by(compare_dimensions);
    Some(PointContext {
        monitoring_account,
        namespace,
        dimensions,
    })
}

fn add_dimension(
    dimensions: &mut Vec<Dimension>,
    attribute: &KeyValue,
    string_only: bool,
    overwrite_duplicate: bool,
) -> bool {
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
                Some(any_value::Value::DoubleValue(value)) if !string_only => value.to_string(),
                _ => String::new(),
            });
    if attribute.key.chars().count() > MAX_DIMENSION_NAME_CHARS
        || value.chars().count() > MAX_DIMENSION_VALUE_CHARS
    {
        return false;
    }
    if let Some(existing) = dimensions
        .iter_mut()
        .find(|dimension| dimension.name.eq_ignore_ascii_case(&attribute.key))
    {
        if overwrite_duplicate {
            existing.value = value;
        }
        return true;
    }
    dimensions.push(Dimension {
        name: attribute.key.clone(),
        value,
    });
    true
}

pub(super) fn selected_scope_dimensions(
    scope_metrics: &ScopeMetrics,
    config: &Config,
) -> Option<Vec<Dimension>> {
    let Some(scope) = scope_metrics
        .scope
        .as_ref()
        .filter(|scope| !scope.name.is_empty())
    else {
        return Some(Vec::new());
    };
    let selection = config
        .scope_attributes
        .iter()
        .find(|selection| selection.name == "*")
        .or_else(|| {
            config
                .scope_attributes
                .iter()
                .find(|selection| selection.name == scope.name)
        });
    let Some(selection) = selection else {
        return Some(Vec::new());
    };
    let mut dimensions = Vec::new();
    for attribute in &scope.attributes {
        if attribute.key != NAMESPACE_ATTRIBUTE
            && selected_attribute(&attribute.key, &selection.keys)
            && !add_dimension(
                &mut dimensions,
                attribute,
                true,
                config.honor_scope_attributes,
            )
        {
            return None;
        }
    }
    Some(dimensions)
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
        existing.value.clone_from(&dimension.value);
    } else {
        dimensions.push(dimension.clone());
    }
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
        .and_then(string_value)
}

pub(super) fn string_value(attribute: &KeyValue) -> Option<String> {
    match attribute.value.as_ref()?.value.as_ref()? {
        any_value::Value::StringValue(value) => Some(value.clone()),
        _ => None,
    }
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
            dimensions_valid: true,
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
        assert!(context.dimensions_valid);
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
        assert!(context.dimensions_valid);
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
                dimension("alpha", "second"),
                dimension("Beta", "scope"),
                dimension("zeta", "resource"),
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

        let dimensions = selected_scope_dimensions(&scope, &mapping_config)
            .expect("scope dimensions should be valid");

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

        let dimensions = selected_scope_dimensions(&scope, &mapping_config)
            .expect("scope dimensions should be valid");

        assert_eq!(
            dimensions,
            vec![
                dimension("region", "west"),
                dimension("service", "checkout"),
            ]
        );
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

        let dimensions = selected_scope_dimensions(&scope, &mapping_config)
            .expect("unselected scope should remain valid");

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
