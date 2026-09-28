// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OTLP attribute selection and Geneva metric context construction.

use std::cmp::Ordering;
use std::str::{self, Utf8Error};

use otel_arrow_dfe_pdata_views::views::common::{
    AnyValueView, AttributeView, InstrumentationScopeView,
};
use otel_arrow_dfe_pdata_views::views::metrics::ScopeMetricsView;

use super::super::encoder::Dimension;
use super::{CardinalityOverflow, Config, PointContext, ResourceContext};

const MAX_DIMENSIONS: usize = 74;
pub(super) const MAX_DIMENSION_NAME_UTF16_UNITS: usize = 512;
pub(super) const MAX_DIMENSION_VALUE_UTF16_UNITS: usize = 1024;

pub(super) const ACCOUNT_ATTRIBUTE: &str = "_microsoft_metrics_account";
const PREVIOUS_ACCOUNT_ATTRIBUTE: &str = "microsoft_metrics_account";
pub(super) const NAMESPACE_ATTRIBUTE: &str = "_microsoft_metrics_namespace";
const PREVIOUS_NAMESPACE_ATTRIBUTE: &str = "microsoft_metrics_namespace";
const CARDINALITY_OVERFLOW_ATTRIBUTE: &str = "otel.metric.overflow";

pub(super) fn default_resource_context(config: &Config) -> ResourceContext {
    ResourceContext {
        monitoring_account: config.monitoring_account.clone(),
        namespace: config.metric_namespace.clone(),
        original_dimensions: Vec::new(),
        dimensions: Vec::new(),
    }
}

pub(super) fn resource_context<A>(
    attributes: impl IntoIterator<Item = A>,
    config: &Config,
) -> Result<ResourceContext, Utf8Error>
where
    A: AttributeView,
{
    let mut monitoring_account = None;
    let mut namespace = None;
    let mut dimensions = Vec::new();
    for attribute in attributes {
        match str::from_utf8(attribute.key())? {
            ACCOUNT_ATTRIBUTE | PREVIOUS_ACCOUNT_ATTRIBUTE => {
                monitoring_account = Some(routing_value(&attribute)?);
            }
            NAMESPACE_ATTRIBUTE | PREVIOUS_NAMESPACE_ATTRIBUTE => {
                namespace = Some(routing_value(&attribute)?);
            }
            key if selected_attribute(key, &config.resource_attributes) => {
                dimensions.push(attribute_dimension(&attribute, true)?);
            }
            _ => {}
        }
    }
    Ok(ResourceContext {
        monitoring_account: monitoring_account
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| config.monitoring_account.clone()),
        namespace: namespace
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| config.metric_namespace.clone()),
        original_dimensions: dimensions.clone(),
        dimensions,
    })
}

pub(super) fn point_context<A>(
    attributes: impl IntoIterator<Item = A>,
    resource: &ResourceContext,
    scope_namespace: &str,
    scope_dimensions: &[Dimension],
    config: &Config,
    metric_name: Option<&str>,
) -> Result<(Option<PointContext>, Option<CardinalityOverflow>), Utf8Error>
where
    A: AttributeView,
{
    let mut monitoring_account = None;
    let mut namespace = None;
    let mut point_dimensions = Vec::new();
    let mut dimensions_valid = metric_name.is_some();
    let mut cardinality_overflow = false;
    for attribute in attributes {
        match str::from_utf8(attribute.key())? {
            ACCOUNT_ATTRIBUTE => {
                monitoring_account = Some(routing_value(&attribute)?);
            }
            NAMESPACE_ATTRIBUTE => {
                namespace = Some(routing_value(&attribute)?);
            }
            CARDINALITY_OVERFLOW_ATTRIBUTE => {
                cardinality_overflow = attribute
                    .value()
                    .and_then(|value| value.as_bool())
                    .unwrap_or(false);
            }
            _ if dimensions_valid
                && !add_dimension(&mut point_dimensions, &attribute, false, true)? =>
            {
                dimensions_valid = false;
                point_dimensions.clear();
            }
            _ => {}
        }
    }
    let dimensions = if dimensions_valid {
        merge_dimensions(
            point_dimensions,
            &resource.dimensions,
            scope_dimensions,
            config.honor_resource_attributes,
            config.honor_scope_attributes,
        )
        .filter(|dimensions| dimensions_within_limits(dimensions))
        .map(|mut dimensions| {
            dimensions.sort_unstable_by(compare_dimensions);
            dimensions
        })
    } else {
        None
    };
    if dimensions.is_none() && !cardinality_overflow {
        return Ok((None, None));
    }

    let monitoring_account =
        monitoring_account.unwrap_or_else(|| resource.monitoring_account.clone());
    let namespace = namespace.unwrap_or_else(|| scope_namespace.to_string());
    if let Some(dimensions) = dimensions {
        let overflow = cardinality_overflow.then(|| CardinalityOverflow {
            monitoring_account: monitoring_account.clone(),
            namespace: namespace.clone(),
            metric_name: metric_name.unwrap_or_default().to_string(),
        });
        return Ok((
            Some(PointContext {
                monitoring_account,
                namespace,
                dimensions,
            }),
            overflow,
        ));
    }
    Ok((
        None,
        Some(CardinalityOverflow {
            monitoring_account,
            namespace,
            metric_name: metric_name.unwrap_or_default().to_string(),
        }),
    ))
}

fn add_dimension<A>(
    dimensions: &mut Vec<Dimension>,
    attribute: &A,
    string_only: bool,
    overwrite_duplicate: bool,
) -> Result<bool, Utf8Error>
where
    A: AttributeView,
{
    let dimension = attribute_dimension(attribute, string_only)?;
    Ok(set_dimension(dimensions, dimension, overwrite_duplicate))
}

fn attribute_dimension<A>(attribute: &A, string_only: bool) -> Result<Dimension, Utf8Error>
where
    A: AttributeView,
{
    let value = attribute.value().map_or_else(
        || Ok(String::new()),
        |value| {
            if let Some(value) = value.as_string() {
                return Ok(str::from_utf8(value)?.to_string());
            }
            if !string_only {
                if let Some(value) = value.as_bool() {
                    return Ok(if value { "True" } else { "False" }.to_string());
                }
                if let Some(value) = value.as_int64() {
                    return Ok(value.to_string());
                }
                if let Some(value) = value.as_double() {
                    return Ok(format_double_dimension(value));
                }
            }
            Ok(String::new())
        },
    )?;
    Ok(Dimension {
        name: str::from_utf8(attribute.key())?.to_string(),
        value,
    })
}

fn set_dimension(
    dimensions: &mut Vec<Dimension>,
    dimension: Dimension,
    overwrite_duplicate: bool,
) -> bool {
    if let Some(existing) = dimensions
        .iter_mut()
        .find(|existing| existing.name.eq_ignore_ascii_case(&dimension.name))
    {
        if overwrite_duplicate && !existing.value.eq_ignore_ascii_case(&dimension.value) {
            existing.value = dimension.value;
        }
        return true;
    }
    if dimensions.len() == MAX_DIMENSIONS {
        return false;
    }
    dimensions.push(dimension);
    true
}

pub(super) fn selected_scope_dimensions<S>(
    scope_metrics: &S,
    config: &Config,
) -> Result<Vec<Dimension>, Utf8Error>
where
    S: ScopeMetricsView,
{
    let Some(scope) = scope_metrics
        .scope()
        .filter(|scope| scope.name().is_some_and(|name| !name.is_empty()))
    else {
        return Ok(Vec::new());
    };
    let scope_name = str::from_utf8(scope.name().unwrap_or_default())?;
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
            .filter(|selection| selection.name == scope_name)
            .flat_map(|selection| selection.keys.iter())
            .map(String::as_str)
            .collect::<Vec<_>>()
    };
    let mut dimensions = Vec::new();
    for attribute in scope.attributes() {
        let key = str::from_utf8(attribute.key())?;
        if key != NAMESPACE_ATTRIBUTE
            && selected_keys
                .iter()
                .any(|candidate| *candidate == "*" || *candidate == key)
        {
            dimensions.push(attribute_dimension(&attribute, true)?);
        }
    }
    Ok(dimensions)
}

pub(super) fn apply_scope_resource_overrides(
    resource_dimensions: &mut [Dimension],
    original_resource_dimensions: &[Dimension],
    scope_dimensions: &[Dimension],
    honor_resource: bool,
    honor_scope: bool,
) {
    if honor_resource || honor_scope {
        return;
    }
    for scope_dimension in scope_dimensions {
        let Some((index, original)) = original_resource_dimensions
            .iter()
            .enumerate()
            .find(|(_, resource_dimension)| resource_dimension.name == scope_dimension.name)
        else {
            continue;
        };
        resource_dimensions[index].value = if scope_dimension.value.is_empty() {
            original.value.clone()
        } else {
            scope_dimension.value.clone()
        };
    }
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
    mut point: Vec<Dimension>,
    resource: &[Dimension],
    scope: &[Dimension],
    honor_resource: bool,
    honor_scope: bool,
) -> Option<Vec<Dimension>> {
    for dimension in resource {
        if !set_dimension(&mut point, dimension.clone(), honor_resource) {
            return None;
        }
    }
    for dimension in scope {
        if !set_dimension(&mut point, dimension.clone(), honor_scope) {
            return None;
        }
    }
    Some(point)
}

fn dimensions_within_limits(dimensions: &[Dimension]) -> bool {
    dimensions.len() <= MAX_DIMENSIONS
        && dimensions.iter().all(|dimension| {
            dimension.name.encode_utf16().count() <= MAX_DIMENSION_NAME_UTF16_UNITS
                && dimension.value.encode_utf16().count() <= MAX_DIMENSION_VALUE_UTF16_UNITS
        })
}

fn compare_dimensions(left: &Dimension, right: &Dimension) -> Ordering {
    compare_ascii_case_insensitive(&left.name, &right.name)
        .then_with(|| compare_ascii_case_insensitive(&left.value, &right.value))
}

fn compare_ascii_case_insensitive(left: &str, right: &str) -> Ordering {
    // ME compares UTF-16 wide strings and folds ASCII under its default C locale.
    left.encode_utf16()
        .map(ascii_lowercase_utf16)
        .cmp(right.encode_utf16().map(ascii_lowercase_utf16))
}

fn ascii_lowercase_utf16(code_unit: u16) -> u16 {
    match code_unit {
        0x41..=0x5a => code_unit + 0x20,
        _ => code_unit,
    }
}

pub(super) fn attribute_string<A>(
    attributes: impl IntoIterator<Item = A>,
    key: &str,
) -> Result<Option<String>, Utf8Error>
where
    A: AttributeView,
{
    let mut matched = None;
    for attribute in attributes {
        if str::from_utf8(attribute.key())? == key {
            matched = Some(routing_value(&attribute)?);
        }
    }
    Ok(matched.filter(|value| !value.is_empty()))
}

fn string_value<A>(attribute: &A) -> Result<Option<String>, Utf8Error>
where
    A: AttributeView,
{
    let Some(value) = attribute.value() else {
        return Ok(None);
    };
    let Some(value) = value.as_string() else {
        return Ok(None);
    };
    Ok(Some(str::from_utf8(value)?.to_string()))
}

fn routing_value<A>(attribute: &A) -> Result<String, Utf8Error>
where
    A: AttributeView,
{
    Ok(string_value(attribute)?.unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{
        AnyValue, InstrumentationScope, KeyValue, any_value,
    };
    use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::ScopeMetrics;
    use otel_arrow_dfe_pdata::views::otlp::proto::common::KeyValueIter;
    use otel_arrow_dfe_pdata::views::otlp::proto::metrics::ObjScopeMetrics;
    use otel_arrow_dfe_pdata::views::otlp::proto::wrappers::Wraps;

    use super::super::ResourceContext;
    use super::*;
    use crate::exporters::geneva_metrics_exporter::{Config, ScopeAttributes, encoder::Dimension};

    fn resource_context(attributes: &[KeyValue], config: &Config) -> ResourceContext {
        super::resource_context(KeyValueIter::new(attributes.iter()), config)
            .expect("test attributes should contain valid UTF-8")
    }

    fn point_context(
        attributes: &[KeyValue],
        resource: &ResourceContext,
        scope_namespace: &str,
        scope_dimensions: &[Dimension],
        config: &Config,
    ) -> Option<PointContext> {
        super::point_context(
            KeyValueIter::new(attributes.iter()),
            resource,
            scope_namespace,
            scope_dimensions,
            config,
            Some("metric"),
        )
        .expect("test attributes should contain valid UTF-8")
        .0
    }

    fn overflow_diagnostic(
        attributes: &[KeyValue],
        resource: &ResourceContext,
        scope_namespace: &str,
        metric_name: &str,
    ) -> Option<CardinalityOverflow> {
        super::point_context(
            KeyValueIter::new(attributes.iter()),
            resource,
            scope_namespace,
            &[],
            &config(),
            Some(metric_name),
        )
        .expect("test attributes should contain valid UTF-8")
        .1
    }

    fn selected_scope_dimensions(scope_metrics: &ScopeMetrics, config: &Config) -> Vec<Dimension> {
        super::selected_scope_dimensions(&ObjScopeMetrics::new(scope_metrics), config)
            .expect("test attributes should contain valid UTF-8")
    }

    fn attribute_string(attributes: &[KeyValue], key: &str) -> Option<String> {
        super::attribute_string(KeyValueIter::new(attributes.iter()), key)
            .expect("test attributes should contain valid UTF-8")
    }

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
            original_dimensions: dimensions.clone(),
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
            let merged = merge_dimensions(
                point.to_vec(),
                &resource,
                &scope,
                honor_resource,
                honor_scope,
            )
            .expect("one merged dimension should be valid");

            assert_eq!(merged.len(), 1);
            assert_eq!(merged[0].name, "SHARED");
            assert_eq!(merged[0].value, expected);
        }
    }

    /// Scenario: Unhonored scope dimensions use exact-case resource matching before event enrichment.
    /// Guarantees: Exact spelling replaces the resource value while a case-only name variant does not.
    #[test]
    fn applies_exact_case_scope_pre_override() {
        let original = vec![dimension("region", "resource")];
        let mut exact = original.clone();
        apply_scope_resource_overrides(
            &mut exact,
            &original,
            &[dimension("region", "scope")],
            false,
            false,
        );
        let mut case_variant = original.clone();
        apply_scope_resource_overrides(
            &mut case_variant,
            &original,
            &[dimension("REGION", "scope")],
            false,
            false,
        );

        assert_eq!(
            merge_dimensions(Vec::new(), &exact, &[], false, false),
            Some(vec![dimension("region", "scope")])
        );
        assert_eq!(
            merge_dimensions(Vec::new(), &case_variant, &[], false, false),
            Some(vec![dimension("region", "resource")])
        );
    }

    /// Scenario: A later scope repeats an exact resource key with an empty value.
    /// Guarantees: FE clears the earlier scope override and restores the original resource value.
    #[test]
    fn empty_scope_pre_override_restores_resource_value() {
        let original = vec![dimension("region", "resource")];
        let mut effective = original.clone();
        apply_scope_resource_overrides(
            &mut effective,
            &original,
            &[dimension("region", "scope"), dimension("region", "")],
            false,
            false,
        );

        assert_eq!(effective, original);
    }

    /// Scenario: One scope overrides a resource key and the next scope omits that key.
    /// Guarantees: The resource override remains effective until a later exact-case scope value replaces or clears it.
    #[test]
    fn persists_scope_pre_override_across_scopes() {
        let original = vec![dimension("region", "resource")];
        let mut effective = original.clone();
        apply_scope_resource_overrides(
            &mut effective,
            &original,
            &[dimension("region", "scope-one")],
            false,
            false,
        );
        apply_scope_resource_overrides(
            &mut effective,
            &original,
            &[dimension("other", "scope-two")],
            false,
            false,
        );

        assert_eq!(effective, vec![dimension("region", "scope-one")]);
    }

    /// Scenario: Selected resource and scope attributes contain case variants of the same key.
    /// Guarantees: Raw parent occurrences remain available for FE's exact-case pre-override pass.
    #[test]
    fn retains_parent_dimension_occurrences_until_merge() {
        let mut mapping_config = config();
        mapping_config.resource_attributes = vec!["*".to_string()];
        mapping_config.scope_attributes = vec![ScopeAttributes {
            name: "meter".to_string(),
            keys: vec!["*".to_string()],
        }];
        let resource = resource_context(
            &[
                string_attribute("REGION", "first-resource"),
                string_attribute("region", "second-resource"),
            ],
            &mapping_config,
        );
        let scope = scope_metrics(
            "meter",
            vec![
                string_attribute("REGION", "first-scope"),
                string_attribute("region", "second-scope"),
            ],
        );
        let scope_dimensions = selected_scope_dimensions(&scope, &mapping_config);

        assert_eq!(
            resource.dimensions,
            vec![
                dimension("REGION", "first-resource"),
                dimension("region", "second-resource"),
            ]
        );
        assert_eq!(
            scope_dimensions,
            vec![
                dimension("REGION", "first-scope"),
                dimension("region", "second-scope"),
            ]
        );
    }

    /// Scenario: An honored resource value differs from the existing point value only by casing.
    /// Guarantees: ME's case-insensitive duplicate check preserves the first serialized value.
    #[test]
    fn preserves_point_value_when_honored_resource_differs_only_by_case() {
        let point = [dimension("REGION", "WEST")];
        let resource = [dimension("region", "west")];

        let merged = merge_dimensions(point.to_vec(), &resource, &[], true, false)
            .expect("one merged dimension should be valid");

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

    /// Scenario: Dimension names include ASCII case variants and a Unicode character with a similar fold.
    /// Guarantees: Sorting uses the same ASCII case semantics as duplicate detection and then compares values.
    #[test]
    fn sorts_dimensions_with_ascii_case_folding() {
        assert_eq!(
            compare_dimensions(&dimension("REGION", "east"), &dimension("region", "WEST")),
            Ordering::Less
        );
        assert_eq!(
            compare_dimensions(&dimension("k", "value"), &dimension("\u{212a}", "value")),
            Ordering::Less
        );
    }

    /// Scenario: A supplementary-plane dimension name is compared with a BMP private-use name.
    /// Guarantees: Dimension ordering follows ME's UTF-16 code-unit order rather than UTF-8 byte order.
    #[test]
    fn sorts_dimension_names_by_utf16_code_units() {
        assert_eq!(
            compare_dimensions(
                &dimension("\u{10000}", "value"),
                &dimension("\u{e000}", "value")
            ),
            Ordering::Less
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

    /// Scenario: The dimension accumulator reaches 74 unique names, then receives a duplicate and a new name.
    /// Guarantees: Duplicate updates remain valid while the first excess unique dimension is rejected without growing the vector.
    #[test]
    fn bounds_dimension_accumulation_at_limit() {
        let mut dimensions = (0..MAX_DIMENSIONS)
            .map(|index| dimension(&format!("dimension-{index}"), "first"))
            .collect::<Vec<_>>();

        assert!(set_dimension(
            &mut dimensions,
            dimension("DIMENSION-0", "second"),
            true,
        ));
        assert_eq!(dimensions[0].value, "second");
        assert!(!set_dimension(
            &mut dimensions,
            dimension("dimension-overflow", "value"),
            true,
        ));
        assert_eq!(dimensions.len(), MAX_DIMENSIONS);
    }

    /// Scenario: A point attribute name exceeds the Geneva UTF-16 character limit.
    /// Guarantees: The invalid dimension causes only that point context to be rejected.
    #[test]
    fn rejects_oversized_point_dimension_name() {
        let point = point_context(
            &[string_attribute(
                &"n".repeat(MAX_DIMENSION_NAME_UTF16_UNITS + 1),
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
                &"v".repeat(MAX_DIMENSION_VALUE_UTF16_UNITS + 1),
            )],
            &resource_with_dimensions(Vec::new()),
            "scope-namespace",
            &[],
            &config(),
        );

        assert!(point.is_none());
    }

    /// Scenario: Dimension names and values fit the scalar-value limit but exceed the Geneva UTF-16 limit.
    /// Guarantees: Non-BMP characters count as two UTF-16 code units and reject the point before encoding.
    #[test]
    fn rejects_dimensions_above_utf16_limits() {
        let oversized_name = "\u{10000}".repeat(MAX_DIMENSION_NAME_UTF16_UNITS / 2 + 1);
        let oversized_value = "\u{10000}".repeat(MAX_DIMENSION_VALUE_UTF16_UNITS / 2 + 1);

        for attributes in [
            vec![string_attribute(&oversized_name, "value")],
            vec![string_attribute("name", &oversized_value)],
        ] {
            assert!(
                point_context(
                    &attributes,
                    &resource_with_dimensions(Vec::new()),
                    "scope-namespace",
                    &[],
                    &config(),
                )
                .is_none()
            );
        }
    }

    /// Scenario: Oversized resource and scope values are replaced by valid point values.
    /// Guarantees: Dimension limits apply after ME precedence, so discarded parent values do not reject the point.
    #[test]
    fn accepts_valid_point_overrides_for_oversized_parent_values() {
        let oversized = "v".repeat(MAX_DIMENSION_VALUE_UTF16_UNITS + 1);
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
