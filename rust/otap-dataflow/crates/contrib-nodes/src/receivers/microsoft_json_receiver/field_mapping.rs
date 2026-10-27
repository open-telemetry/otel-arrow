// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Endpoint compilation. No listener, global state, or background reload tasks.

#![allow(dead_code)] // The receiver remains non-operational until the runtime phase.

use super::bounded::{Budget, Limits, MappingError, inspect, read_config};
use super::config::{
    Config, EndpointConfig, EventAction, FieldRule, MappingTarget, OnMissing, ScopeField,
    ScopeSplit, ValueTransform,
};
use super::ikey_filter::{self, IkeyValidator};
use std::collections::{BTreeMap, HashSet};
use std::path::Path;

#[derive(Debug, Clone)]
pub(crate) struct CompiledRule {
    pub path: Vec<String>,
    pub rule: FieldRule,
    allocation_work: usize,
}

impl CompiledRule {
    pub fn matches(&self, path: &[String], budget: &mut Budget) -> Result<bool, MappingError> {
        let pattern_bytes = self.path.iter().map(String::len).sum::<usize>();
        budget.charge(pattern_bytes + path.iter().map(String::len).sum::<usize>() + 1)?;
        let parts = path.iter().map(|key| key.split('.').count()).sum::<usize>();
        // Account for every DP row, string comparison and boundary pass before
        // allocating scratch buffers or entering the multiplicative loop.
        let row_work = pattern_bytes.saturating_add(4 * (self.path.len() + 1));
        budget
            .charge(row_work.saturating_mul(parts.saturating_add(path.len()).saturating_add(1)))?;
        let mut current = vec![false; self.path.len() + 1];
        let mut next = current.clone();
        let mut boundary = current.clone();
        current[0] = true;
        for key in path {
            boundary.fill(false);
            for part in key.split('.') {
                next.fill(false);
                for (index, expected) in self.path.iter().enumerate() {
                    if current[index] {
                        if expected == "*" {
                            // A wildcard also captures the remainder of a literal key.
                            boundary[index + 1] = true;
                            next[index + 1] = true;
                        } else if expected == part {
                            next[index + 1] = true;
                        }
                    }
                }
                std::mem::swap(&mut current, &mut next);
            }
            for (matched, end) in current.iter_mut().zip(&boundary) {
                *matched |= end;
            }
        }
        Ok(current[self.path.len()])
    }
}

#[derive(Debug)]
pub(crate) struct CompiledOverride {
    pub pattern: String,
    pub action: EventAction,
    pub rules: Vec<CompiledRule>,
}

#[derive(Debug)]
pub(crate) struct CompiledEndpoint {
    pub config: EndpointConfig,
    pub rules: Vec<CompiledRule>,
    pub overrides: Vec<CompiledOverride>,
    pub ikey_validator: Option<IkeyValidator>,
    pub env_resource_attrs: BTreeMap<String, serde_json::Value>,
    pub limits: Limits,
}

/// Load all routes at startup, atomically. No partial endpoint set on failure.
pub(crate) fn load_endpoints(
    config: &Config,
    config_dir: &Path,
) -> Result<BTreeMap<String, CompiledEndpoint>, MappingError> {
    config
        .validate()
        .map_err(|e| MappingError::Config(e.to_string()))?;
    if config.endpoints.len() > 64 {
        return Err(MappingError::Limit("endpoint count"));
    }
    let mut result = BTreeMap::new();
    let mut paths: Vec<_> = config.endpoints.iter().collect();
    paths.sort_by_key(|(path, _)| *path);
    for (route, reference) in paths {
        let path = ikey_filter::resolve_config_path(&reference.config_file, config_dir)
            .map_err(MappingError::File)?;
        let yaml = read_config(&path)?;
        let endpoint = serde_yaml::from_str(&yaml)
            .map_err(|e| MappingError::Config(format!("{}: {e}", path.display())))?;
        let compiled =
            CompiledEndpoint::compile(&endpoint, Some(config_dir), &|name| std::env::var(name))?;
        if compiled
            .config
            .max_request_body_size
            .is_some_and(|limit| limit > config.max_request_body_size)
        {
            return Err(MappingError::Config(format!(
                "route {route}: endpoint body limit exceeds receiver limit"
            )));
        }
        _ = result.insert(route.clone(), compiled);
    }
    Ok(result)
}

impl CompiledEndpoint {
    /// The environment resolver is injected so tests never mutate process state.
    pub fn compile(
        config: &EndpointConfig,
        config_dir: Option<&Path>,
        environment: &impl Fn(&str) -> Result<String, std::env::VarError>,
    ) -> Result<Self, MappingError> {
        Self::with_limits(config, config_dir, environment, Limits::default())
    }

    pub fn with_limits(
        config: &EndpointConfig,
        config_dir: Option<&Path>,
        environment: &impl Fn(&str) -> Result<String, std::env::VarError>,
        limits: Limits,
    ) -> Result<Self, MappingError> {
        limits.validate()?;
        if config.body_format.0.is_empty()
            || config.max_request_body_size == Some(0)
            || config.max_event_size == Some(0)
            || config.unmapped.attribute_key.is_empty()
        {
            return Err(MappingError::Config(
                "empty body formats, attribute key or zero byte limit".into(),
            ));
        }
        let mut budget = Budget::new(limits.work);
        let rules = flatten(&config.rules, limits, &mut budget)?;
        validate_event_name_rules(&rules)?;
        let mut overrides = Vec::new();
        let mut count = rules.len();
        let mut entries: Vec<_> = config.event_overrides.iter().collect();
        entries.sort_by_key(|(pattern, _)| *pattern);
        for (pattern, entry) in entries {
            validate_pattern(pattern)?;
            budget.charge(pattern.len())?;
            let additions = flatten(&entry.rules, limits, &mut budget)?;
            if entry.inherit_rules && count.saturating_add(rules.len()) > limits.rules {
                return Err(MappingError::Limit("compiled rules"));
            }
            if entry.inherit_rules {
                for rule in &rules {
                    budget.charge(rule.allocation_work)?;
                }
            }
            let mut merged = if entry.inherit_rules {
                rules.clone()
            } else {
                Vec::new()
            };
            for addition in additions {
                let addition_bytes = addition.path.iter().map(String::len).sum::<usize>();
                let mut matching = None;
                for (index, candidate) in merged.iter().enumerate() {
                    budget.charge(
                        addition_bytes
                            + addition.path.len()
                            + candidate.path.iter().map(String::len).sum::<usize>()
                            + candidate.path.len()
                            + 1,
                    )?;
                    if candidate.path == addition.path {
                        matching = Some(index);
                        break;
                    }
                }
                if let Some(index) = matching {
                    let existing = &mut merged[index];
                    if addition.rule.target.is_some() {
                        let required = addition
                            .rule
                            .required
                            .clone()
                            .or(existing.rule.required.clone());
                        *existing = addition;
                        existing.rule.required = required;
                    } else {
                        let overlay = addition.rule;
                        if overlay.required.is_some() {
                            existing.rule.required = overlay.required;
                        }
                        if overlay.allowed_keys.is_some() {
                            existing.rule.allowed_keys = overlay.allowed_keys;
                        }
                        if overlay.validate.is_some() {
                            existing.rule.validate = overlay.validate;
                        }
                        if !overlay.also.is_empty() {
                            existing.rule.also = overlay.also;
                        }
                    }
                } else {
                    if count.saturating_add(merged.len()) >= limits.rules {
                        return Err(MappingError::Limit("compiled rules"));
                    }
                    merged.push(addition);
                }
            }
            count = count
                .checked_add(merged.len())
                .ok_or(MappingError::Limit("compiled rules"))?;
            if count > limits.rules {
                return Err(MappingError::Limit("compiled rules"));
            }
            validate_event_name_rules(&merged)?;
            overrides.push(CompiledOverride {
                pattern: pattern.clone(),
                action: entry.action.clone(),
                rules: merged,
            });
        }
        // Exact overrides precede globs; overlapping globs use the longest prefix.
        overrides.sort_by(|a, b| {
            a.pattern
                .ends_with('*')
                .cmp(&b.pattern.ends_with('*'))
                .then_with(|| b.pattern.len().cmp(&a.pattern.len()))
                .then_with(|| a.pattern.cmp(&b.pattern))
        });
        for pattern in &config.drop_events {
            validate_pattern(pattern)?;
        }
        for mapping in &config.header_mappings {
            if mapping.header.is_empty()
                || !mapping.header.bytes().all(is_header_token)
                || !matches!(
                    mapping.target,
                    MappingTarget::Attribute { .. } | MappingTarget::Resource { .. }
                )
            {
                return Err(MappingError::Config(
                    "header mappings require a valid name and an attribute/resource target".into(),
                ));
            }
            validate_target(&mapping.target, &mut budget)?;
        }
        let (ikey_validator, _) = match &config.ikey_filter {
            Some(source) => {
                ikey_filter::compile_source(source, config_dir).map_err(MappingError::Config)?
            }
            None => (None, None),
        };
        if let Some(validator) = &ikey_validator {
            budget.charge(validator.comparison_work())?;
        }
        let mut env_resource_attrs = BTreeMap::new();
        let mut environment_keys: Vec<_> = config.env_resource_attrs.iter().collect();
        environment_keys.sort();
        let mut destinations = HashSet::new();
        for (name, key) in environment_keys {
            if name.is_empty() || key.is_empty() || !destinations.insert(key) {
                return Err(MappingError::Config("environment mappings require unique non-empty attribute keys and variable names".into()));
            }
            match environment(name) {
                Ok(value) if !value.is_empty() => {
                    budget.charge(key.len() + value.len())?;
                    _ = env_resource_attrs.insert(key.clone(), value.into());
                }
                Ok(_) | Err(std::env::VarError::NotPresent) => {}
                Err(e) => {
                    return Err(MappingError::Config(format!(
                        "environment variable {name}: {e}"
                    )));
                }
            }
        }
        let mut config = config.clone();
        if config.allowed_methods.is_empty() {
            config.allowed_methods.push("POST".into());
        }
        for method in &mut config.allowed_methods {
            if method.is_empty() || !method.bytes().all(is_header_token) {
                return Err(MappingError::Config("invalid HTTP method".into()));
            }
            method.make_ascii_uppercase();
        }
        for content_type in &mut config.accepted_content_types {
            *content_type = content_type
                .split(';')
                .next()
                .unwrap_or("")
                .trim()
                .to_ascii_lowercase();
            if !content_type.split_once('/').is_some_and(|(kind, subtype)| {
                !kind.is_empty()
                    && !subtype.is_empty()
                    && kind.bytes().all(is_header_token)
                    && subtype.bytes().all(is_header_token)
            }) {
                return Err(MappingError::Config("invalid accepted content type".into()));
            }
        }
        config.cors.cache_control = config
            .cors
            .cache_control
            .replace("<max-age>", &config.cors.max_age_seconds.to_string());
        let mut query_headers = std::collections::HashMap::new();
        for (parameter, header) in config.query_string_headers {
            if parameter.is_empty() || header.is_empty() || !header.bytes().all(is_header_token) {
                return Err(MappingError::Config(
                    "invalid query-string header promotion".into(),
                ));
            }
            if query_headers
                .insert(parameter.to_ascii_lowercase(), header.to_ascii_lowercase())
                .is_some()
            {
                return Err(MappingError::Config(
                    "duplicate case-insensitive query-string parameter".into(),
                ));
            }
        }
        config.query_string_headers = query_headers;
        Ok(Self {
            config,
            rules,
            overrides,
            ikey_validator,
            env_resource_attrs,
            limits,
        })
    }
}

fn validate_event_name_rules(rules: &[CompiledRule]) -> Result<(), MappingError> {
    let mut names = 0;
    for rule in rules {
        if rule
            .rule
            .also
            .iter()
            .any(|also| matches!(also.target, MappingTarget::EventName { .. }))
        {
            return Err(MappingError::Config(
                "event_name must be a primary target, not an additional target".into(),
            ));
        }
        if matches!(rule.rule.target, Some(MappingTarget::EventName { .. })) {
            names += 1;
            if names > 1 || rule.path.iter().any(|segment| segment == "*") {
                return Err(MappingError::Config(
                    "each mapping requires at most one exact event_name target".into(),
                ));
            }
        }
    }
    Ok(())
}

fn is_header_token(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&byte)
}

pub(crate) fn pattern_matches(pattern: &str, name: &str) -> bool {
    pattern
        .strip_suffix('*')
        .map_or(pattern == name, |prefix| name.starts_with(prefix))
}

fn validate_pattern(pattern: &str) -> Result<(), MappingError> {
    if pattern.is_empty() || pattern.strip_suffix('*').unwrap_or(pattern).contains('*') {
        return Err(MappingError::Config(format!(
            "invalid event pattern `{pattern}`"
        )));
    }
    Ok(())
}

fn flatten(
    source: &[FieldRule],
    limits: Limits,
    budget: &mut Budget,
) -> Result<Vec<CompiledRule>, MappingError> {
    fn append(
        source: &[FieldRule],
        prefix: &[String],
        result: &mut Vec<CompiledRule>,
        limits: Limits,
        budget: &mut Budget,
    ) -> Result<(), MappingError> {
        for rule in source {
            if rule.target.is_none()
                && rule.children.as_ref().is_none_or(Vec::is_empty)
                && rule.validate.is_none()
                && rule.allowed_keys.is_none()
                && rule.required.is_none()
                && rule.also.is_empty()
            {
                return Err(MappingError::Config(
                    "targetless leaf rules require validation or mapping metadata".into(),
                ));
            }
            let before = budget.remaining();
            budget.charge(rule.path.len() + 1)?;
            if result.len() >= limits.rules {
                return Err(MappingError::Limit("compiled rules"));
            }
            let mut path = prefix.to_vec();
            for part in rule.path.split('.') {
                if part.is_empty() || (part.contains('*') && part != "*") {
                    return Err(MappingError::Config(format!(
                        "invalid field path `{}`",
                        rule.path
                    )));
                }
                if path.len() >= limits.depth {
                    return Err(MappingError::Limit("rule depth"));
                }
                path.push(part.to_owned());
            }
            if path.len() > limits.depth {
                return Err(MappingError::Limit("rule depth"));
            }
            budget.charge(path.iter().map(String::len).sum::<usize>())?;
            if let Some(required) = &rule.required {
                if path.iter().any(|p| p == "*") {
                    return Err(MappingError::Config(
                        "required wildcard paths are ambiguous".into(),
                    ));
                }
                if matches!(required.on_missing, OnMissing::Default)
                    && required.default_value.is_none()
                {
                    return Err(MappingError::Config(
                        "required default action needs default_value".into(),
                    ));
                }
                if let Some(value) = &required.default_value {
                    inspect(value, limits, budget)?;
                }
            }
            if let Some(target) = &rule.target {
                validate_target(target, budget)?;
            }
            if let Some(keys) = &rule.allowed_keys {
                for key in keys {
                    budget.charge(key.len() + 1)?;
                }
            }
            budget.charge(rule.also.len())?;
            for also in &rule.also {
                validate_target(&also.target, budget)?;
                if let Some(transform) = &also.transform {
                    match transform {
                        ValueTransform::SplitTake { delimiter, .. } if delimiter.is_empty() => {
                            return Err(MappingError::Config(
                                "split delimiter must not be empty".into(),
                            ));
                        }
                        ValueTransform::Format { template } => budget.charge(template.len())?,
                        ValueTransform::SplitTake { delimiter, .. } => {
                            budget.charge(delimiter.len())?
                        }
                        _ => {}
                    }
                }
            }
            let flat = FieldRule {
                path: rule.path.clone(),
                target: rule.target.clone(),
                validate: rule.validate.clone(),
                allowed_keys: rule.allowed_keys.clone(),
                required: rule.required.clone(),
                also: rule.also.clone(),
                children: None,
            };
            result.push(CompiledRule {
                path: path.clone(),
                rule: flat,
                allocation_work: before - budget.remaining(),
            });
            if let Some(children) = &rule.children {
                append(children, &path, result, limits, budget)?;
            }
        }
        Ok(())
    }
    let mut result = Vec::new();
    append(source, &[], &mut result, limits, budget)?;
    Ok(result)
}

fn validate_target(target: &MappingTarget, budget: &mut Budget) -> Result<(), MappingError> {
    match target {
        MappingTarget::EventName {
            prefix,
            split_last,
            strip_segment,
        } => {
            if let Some(prefix) = prefix {
                budget.charge(prefix.len())?;
            }
            if split_last.as_deref() == Some("") {
                return Err(MappingError::Config(
                    "event-name split delimiter must not be empty".into(),
                ));
            }
            if let Some(strip) = strip_segment
                && let Some(path) = strip.match_field()
                && (path.is_empty() || path.split('.').any(|p| p.is_empty() || p.contains('*')))
            {
                return Err(MappingError::Config(
                    "strip_segment needs an exact match_field path".into(),
                ));
            }
        }
        MappingTarget::Attribute { key, prefix } | MappingTarget::Resource { key, prefix } => {
            if key.as_deref() == Some("") {
                return Err(MappingError::Config(
                    "attribute key must not be empty".into(),
                ));
            }
            if let Some(key) = key {
                budget.charge(key.len())?;
            }
            if let Some(prefix) = prefix {
                budget.charge(prefix.len())?;
            }
        }
        MappingTarget::Scope {
            split: Some(_),
            field: ScopeField::Name | ScopeField::Version,
        } => {
            return Err(MappingError::Config(
                "scope split requires name_version".into(),
            ));
        }
        MappingTarget::Scope {
            split: Some(ScopeSplit::LastChar(delimiter)),
            ..
        } if delimiter.is_empty() => {
            return Err(MappingError::Config(
                "scope split delimiter must not be empty".into(),
            ));
        }
        MappingTarget::Severity { map } => {
            for entry in map.values() {
                budget.charge(entry.text.len() + 1)?;
                if !(0..=24).contains(&entry.number) {
                    return Err(MappingError::Config(
                        "severity numbers must be in 0..=24".into(),
                    ));
                }
            }
        }
        _ => {}
    }
    Ok(())
}
