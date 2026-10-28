// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bounded, transactional JSON-to-log mapping without HTTP or Arrow dependencies.

#![allow(dead_code)] // Runtime integration is deliberately deferred.

use super::config::{
    EventAction, EventNameValidation, ExpectedType, LengthExceeded, MappingTarget, OnMissing,
    ScopeField, ScopeSplit, UnmappedMode, Validation, ValidationFailure, ValueTransform,
    split_dash_semver,
};
use super::error::MappingError;
use super::field_mapping::{CompiledEndpoint, CompiledRule, pattern_matches};
use super::ikey_filter::FilterVerdict;
use otel_arrow_dfe_config::data_limits::{Budget, Limits, inspect_json, json_size};
use serde::Serialize;
use serde_json::{Map, Value};
use std::borrow::Cow;
use std::collections::BTreeMap;

/// Typed intermediate representation consumed by the later Arrow builder.
#[derive(Debug, Default, PartialEq, Serialize)]
pub(crate) struct MappedLog {
    pub event_name: Option<String>,
    pub time_unix_nano: Option<u64>,
    pub observed_time_unix_nano: Option<u64>,
    pub trace_id: Option<[u8; 16]>,
    pub span_id: Option<[u8; 8]>,
    pub flags: Option<u32>,
    pub severity_number: Option<i32>,
    pub severity_text: Option<String>,
    pub scope_name: Option<String>,
    pub scope_version: Option<String>,
    pub attributes: BTreeMap<String, Value>,
    pub resource: BTreeMap<String, Value>,
    pub body: Option<Value>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum DropReason {
    InstrumentationKey,
    EventName,
    Override,
}

#[derive(Debug, PartialEq)]
#[allow(variant_size_differences)] // Accepted data is boxed; drops have no log allocation.
pub(crate) enum MappingOutcome {
    Accepted(Box<MappedLog>),
    Dropped(DropReason),
}

/// Headers have already been decoded by the caller; names are case-insensitive.
#[derive(Debug, Default)]
pub(crate) struct MappingContext {
    pub headers: BTreeMap<String, String>,
    pub origin: Option<String>,
}

impl CompiledEndpoint {
    pub fn map_event(
        &self,
        event: &Value,
        context: &MappingContext,
    ) -> Result<MappingOutcome, MappingError> {
        let mut budget = Budget::new(self.limits.work);
        inspect_json(event, self.limits, &mut budget)?;
        if !event.is_object() {
            return Err(MappingError::Rejected("expected a JSON object".into()));
        }
        let event_limit = self
            .config
            .max_event_size
            .unwrap_or(usize::MAX)
            .min(budget.remaining());
        let event_bytes = json_size(event, event_limit)?;
        budget.charge(event_bytes)?;

        let mut headers = BTreeMap::new();
        for (key, value) in &context.headers {
            budget.charge(key.len() + value.len() + 1)?;
            if headers
                .insert(key.to_ascii_lowercase(), value.as_str())
                .is_some()
            {
                return Err(MappingError::Rejected(
                    "duplicate case-insensitive header".into(),
                ));
            }
        }
        if let Some(origin) = &context.origin {
            budget.charge(origin.len())?;
        }
        // Select overrides from the base event-name transform, before key remapping.
        let mut selector = None;
        for rule in &self.rules {
            budget.charge(path_work(&rule.path))?;
            if let Some(target @ MappingTarget::EventName { .. }) = &rule.rule.target
                && let Some(value) = resolve(event, &rule.path)
            {
                let value = validate(value, rule.rule.validate.as_ref(), &mut budget)?;
                if let Some(value) = value {
                    selector = Some(event_name(target, &value, event, &mut budget, self.limits)?);
                    break;
                }
            }
        }
        for entry in &self.overrides {
            budget.charge(entry.pattern.len() + 1)?;
        }
        for pattern in &self.config.drop_events {
            budget.charge(pattern.len() + 1)?;
        }
        let selected = selector.as_deref().and_then(|name| {
            self.overrides
                .iter()
                .find(|entry| pattern_matches(&entry.pattern, name))
        });
        if let Some(entry) = selected {
            match entry.action {
                EventAction::Drop => return Ok(MappingOutcome::Dropped(DropReason::Override)),
                EventAction::Reject => return Err(MappingError::RejectedPayload),
                EventAction::Accept => {}
            }
        }
        let key = event
            .get("iKey")
            .and_then(Value::as_str)
            .or_else(|| headers.get("apikey").copied());
        let remap = if let Some(validator) = &self.ikey_validator {
            budget.charge(validator.comparison_work())?;
            match validator.check(key, selector.as_deref(), context.origin.as_deref()) {
                FilterVerdict::Accept => None,
                FilterVerdict::AcceptRemap(key) => Some(key),
                FilterVerdict::Ignore => {
                    return Ok(MappingOutcome::Dropped(DropReason::InstrumentationKey));
                }
                FilterVerdict::Reject => {
                    return Err(MappingError::Rejected("instrumentation key policy".into()));
                }
                FilterVerdict::RejectUnauthorized => return Err(MappingError::Unauthorized),
            }
        } else {
            None
        };
        let rules = selected.map_or(self.rules.as_slice(), |entry| entry.rules.as_slice());
        let mut logical = Cow::Borrowed(event);
        // Inject defaults before the single mapping pass, so nested defaults
        // obey the same validation, allowed-key and unmapped policies.
        for rule in rules {
            budget.charge(path_work(&rule.path))?;
            if let Some(required) = &rule.rule.required
                && resolve(&logical, &rule.path).is_none()
            {
                match required.on_missing {
                    OnMissing::Reject => {
                        return Err(MappingError::Rejected(format!(
                            "missing required field {}",
                            rule.path.join(".")
                        )));
                    }
                    OnMissing::Default => {
                        let default = required
                            .default_value
                            .as_ref()
                            .ok_or_else(|| MappingError::Config("missing default value".into()))?;
                        budget.charge(json_size(default, self.limits.output_bytes)?)?;
                        insert_default(logical.to_mut(), &rule.path, default)?;
                        inspect_json(&logical, self.limits, &mut budget)?;
                        budget.charge(json_size(&logical, budget.remaining())?)?;
                    }
                }
            }
        }
        inspect_json(&logical, self.limits, &mut budget)?;
        let mut state = Mapper {
            endpoint: self,
            event,
            budget,
            output: MappedLog::default(),
            remaining: self.limits.output_bytes,
            attributes: 0,
            unmapped: Vec::new(),
            remap: remap.as_deref(),
        };
        // Environment < header mappings < event mappings. Separate namespaces
        // prevent resource attributes from overwriting log attributes.
        state.reserve(json_size(&state.output, state.remaining)?)?;
        for (key, value) in &self.env_resource_attrs {
            state.attribute(true, key, value)?;
        }
        for mapping in &self.config.header_mappings {
            let header = mapping.header.to_ascii_lowercase();
            if let Some(value) = headers.get(&header) {
                let value = if header == "apikey" {
                    remap.as_deref().unwrap_or(value)
                } else {
                    value
                };
                state.target(
                    &mapping.target,
                    &Value::String(value.to_string()),
                    &[header],
                    None,
                )?;
            }
        }
        let (body, _) = state.walk(&logical, &mut Vec::new(), rules, false)?;
        state.output.body = body;
        if self.config.unmapped.mode == UnmappedMode::Attribute && !state.unmapped.is_empty() {
            let paths = Value::Array(std::mem::take(&mut state.unmapped));
            state.attribute(false, &self.config.unmapped.attribute_key, &paths)?;
        }
        if let Some(name) = &state.output.event_name {
            validate_name(name, &self.config.event_name_validation)?;
            // Defaults, overrides or duplicate dotted/nested representations must
            // not emit a name different from the one used for admission policy.
            if (!self.overrides.is_empty() || self.ikey_validator.is_some())
                && selector.as_deref() != Some(name.as_str())
            {
                return Err(MappingError::Rejected(
                    "mapped event name differs from the policy-selected event name".into(),
                ));
            }
            if self
                .config
                .drop_events
                .iter()
                .any(|p| pattern_matches(p, name))
            {
                return Ok(MappingOutcome::Dropped(DropReason::EventName));
            }
        }
        // Every allocation above is charged before emission; this also guards
        // the final representation, including structural overhead.
        _ = json_size(&state.output, self.limits.output_bytes)?;
        Ok(MappingOutcome::Accepted(Box::new(state.output)))
    }
}

struct Mapper<'a> {
    endpoint: &'a CompiledEndpoint,
    event: &'a Value,
    budget: Budget,
    output: MappedLog,
    remaining: usize,
    attributes: usize,
    unmapped: Vec<Value>,
    remap: Option<&'a str>,
}

impl Mapper<'_> {
    fn reserve(&mut self, bytes: usize) -> Result<(), MappingError> {
        self.budget.charge(bytes)?;
        self.remaining = self
            .remaining
            .checked_sub(bytes)
            .ok_or(MappingError::Limit("output bytes"))?;
        Ok(())
    }

    fn attribute(&mut self, resource: bool, key: &str, value: &Value) -> Result<(), MappingError> {
        let exists = if resource {
            self.output.resource.contains_key(key)
        } else {
            self.output.attributes.contains_key(key)
        };
        if !exists {
            if self.attributes >= self.endpoint.limits.attributes {
                return Err(MappingError::Limit("attribute count"));
            }
            self.attributes += 1;
        }
        let bytes = json_size(value, self.remaining)? + json_size(&key, self.remaining)? + 2;
        self.reserve(bytes)?;
        let attributes = if resource {
            &mut self.output.resource
        } else {
            &mut self.output.attributes
        };
        _ = attributes.insert(key.to_owned(), value.clone());
        Ok(())
    }

    fn walk(
        &mut self,
        original: &Value,
        path: &mut Vec<String>,
        rules: &[CompiledRule],
        mut suppress_unmapped: bool,
    ) -> Result<(Option<Value>, Option<Value>), MappingError> {
        self.budget
            .charge(1 + path.iter().map(String::len).sum::<usize>())?;
        let mut logical = Vec::new();
        for part in path.iter().flat_map(|p| p.split('.')) {
            if logical.len() >= self.endpoint.limits.depth {
                return Err(MappingError::Limit("field path depth"));
            }
            logical.push(part.to_owned());
        }
        // Flattened SDK keys must not bypass constraints on virtual parents.
        for (index, container) in rules.iter().enumerate() {
            if let Some(keys) = &container.rule.allowed_keys
                && let Some((child, _)) = container.ancestor_child(path, &mut self.budget)?
                && !allowed_child(keys, child, &mut self.budget)?
            {
                return Err(MappingError::Rejected(
                    "unlisted flattened child key".into(),
                ));
            }
            if matches!(container.rule.target, Some(MappingTarget::Drop))
                && let Some((_, prefix)) = container.ancestor_child(path, &mut self.budget)?
            {
                let mut shadowed = false;
                for earlier in &rules[..index] {
                    if earlier.matches(&prefix, &mut self.budget)? {
                        shadowed = true;
                        break;
                    }
                }
                suppress_unmapped |= !shadowed;
            }
        }
        let replacement;
        let value = if let Some(remap) = self.remap.filter(|_| path.as_slice() == ["iKey"]) {
            replacement = Value::String(remap.to_owned());
            &replacement
        } else {
            original
        };
        let mut matched = None;
        for rule in rules {
            if rule.matches(path, &mut self.budget)? {
                matched = Some(rule);
                break;
            }
        }
        if let Some(compiled) = matched {
            let rule = &compiled.rule;
            if let Some(keys) = &rule.allowed_keys {
                let object = value.as_object().ok_or_else(|| {
                    MappingError::Rejected("allowed_keys requires an object".into())
                })?;
                for key in object.keys() {
                    if !allowed_child(keys, key, &mut self.budget)? {
                        return Err(MappingError::Rejected(format!("unlisted child key {key}")));
                    }
                }
            }
            let validated = validate(value, rule.validate.as_ref(), &mut self.budget)?;
            let Some(validated) = validated else {
                return Ok((None, None));
            };
            let consumed = rule
                .target
                .as_ref()
                .is_some_and(|t| !matches!(t, MappingTarget::Drop))
                || rule
                    .also
                    .iter()
                    .any(|also| !matches!(also.target, MappingTarget::Drop));
            if let Some(target) = &rule.target {
                self.target(target, &validated, path, Some(compiled))?;
            }
            for also in &rule.also {
                let transformed = match &also.transform {
                    Some(transform) => Cow::Owned(
                        transform_value(transform, &validated, self.remaining, &mut self.budget)?
                            .ok_or_else(|| {
                            MappingError::Rejected(format!(
                                "transform cannot be applied at {}",
                                path.join(".")
                            ))
                        })?,
                    ),
                    None => Cow::Borrowed(&validated),
                };
                self.target(&also.target, &transformed, path, Some(compiled))?;
            }
            if consumed {
                if let Value::Object(map) = value {
                    _ = self.children(map, path, rules, true)?;
                }
                let retained = self.retain(path)?;
                return Ok((None, retained));
            }
            if matches!(rule.target, Some(MappingTarget::Drop)) {
                // A dropped container can still have explicitly mapped children.
                if let Value::Object(map) = value {
                    return self.children(map, path, rules, true);
                }
                return Ok((None, None));
            }
        }
        if let Value::Object(map) = value {
            return self.children(map, path, rules, suppress_unmapped);
        }
        if suppress_unmapped {
            Ok((None, None))
        } else {
            self.unmapped(value, path)
        }
    }

    fn retain(&mut self, path: &[String]) -> Result<Option<Value>, MappingError> {
        if !self.endpoint.config.filtered_item_size {
            return Ok(None);
        }
        let mut value = self.event;
        for key in path {
            let Some(child) = value.get(key) else {
                return Ok(None);
            };
            value = child;
        }
        self.budget
            .charge(json_size(value, self.endpoint.limits.output_bytes)?)?;
        Ok(Some(value.clone()))
    }

    fn children(
        &mut self,
        map: &Map<String, Value>,
        path: &mut Vec<String>,
        rules: &[CompiledRule],
        dropped: bool,
    ) -> Result<(Option<Value>, Option<Value>), MappingError> {
        let mut body = Map::new();
        let mut retained = Map::new();
        let mut entries: Vec<_> = map.iter().collect();
        entries.sort_by_key(|(key, _)| *key);
        for (key, value) in entries {
            path.push(key.clone());
            let (child_body, child_retained) = self.walk(value, path, rules, dropped)?;
            _ = path.pop();
            if !dropped && let Some(value) = child_body {
                self.reserve(json_size(key, self.remaining)? + 3)?;
                _ = body.insert(key.clone(), value);
            }
            if let Some(value) = child_retained {
                _ = retained.insert(key.clone(), value);
            }
        }
        if path.is_empty() && self.endpoint.config.filtered_item_size {
            let size = json_size(&retained, self.endpoint.limits.output_bytes)?;
            self.attribute(false, "filtered_item_size", &Value::from(size))?;
        }
        // Preserve empty source objects rather than treating them as absent.
        if map.is_empty() && !path.is_empty() && !dropped {
            return self.unmapped(&Value::Object(Map::new()), path);
        }
        Ok((
            (!body.is_empty()).then_some(Value::Object(body)),
            (!retained.is_empty()).then_some(Value::Object(retained)),
        ))
    }

    fn unmapped(
        &mut self,
        value: &Value,
        path: &[String],
    ) -> Result<(Option<Value>, Option<Value>), MappingError> {
        match self.endpoint.config.unmapped.mode {
            UnmappedMode::Body => {
                self.reserve(json_size(value, self.remaining)?)?;
                Ok((Some(value.clone()), self.retain(path)?))
            }
            UnmappedMode::Attribute => {
                if self.unmapped.len() >= self.endpoint.limits.attributes {
                    return Err(MappingError::Limit("unmapped paths"));
                }
                let path = path.join(".");
                self.reserve(json_size(&path, self.remaining)? + 1)?;
                self.unmapped.push(Value::String(path));
                Ok((None, None))
            }
            UnmappedMode::Drop => Ok((None, None)),
        }
    }

    fn target(
        &mut self,
        target: &MappingTarget,
        value: &Value,
        path: &[String],
        rule: Option<&CompiledRule>,
    ) -> Result<(), MappingError> {
        self.budget.charge(1)?;
        match target {
            MappingTarget::Drop => {}
            MappingTarget::Attribute { key, prefix } | MappingTarget::Resource { key, prefix } => {
                let start = rule
                    .and_then(|rule| rule.path.iter().position(|part| part == "*"))
                    .unwrap_or(0);
                let source = path
                    .iter()
                    .flat_map(|key| key.split('.'))
                    .skip(start)
                    .collect::<Vec<_>>()
                    .join(".");
                let key = format!(
                    "{}{}",
                    prefix.as_deref().unwrap_or(""),
                    key.as_deref().unwrap_or(&source)
                );
                let encoded;
                let value = if value.is_object() || value.is_array() {
                    let size = json_size(value, self.remaining)?;
                    self.budget.charge(size)?;
                    encoded = Value::String(value.to_string());
                    &encoded
                } else {
                    value
                };
                self.attribute(
                    matches!(target, MappingTarget::Resource { .. }),
                    &key,
                    value,
                )?;
            }
            MappingTarget::EventName { .. } => {
                let name = event_name(
                    target,
                    value,
                    self.event,
                    &mut self.budget,
                    self.endpoint.limits,
                )?;
                self.reserve(json_size(&name, self.remaining)?)?;
                self.output.event_name = Some(name);
            }
            MappingTarget::Timestamp | MappingTarget::ObservedTimestamp => {
                self.reserve(20)?;
                let time = value
                    .as_str()
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .and_then(|t| {
                        u64::try_from(t.timestamp())
                            .ok()?
                            .checked_mul(1_000_000_000)?
                            .checked_add(u64::from(t.timestamp_subsec_nanos()))
                    })
                    .ok_or_else(|| {
                        MappingError::Rejected("invalid or out-of-range RFC 3339 timestamp".into())
                    })?;
                if matches!(target, MappingTarget::Timestamp) {
                    self.output.time_unix_nano = Some(time);
                } else {
                    self.output.observed_time_unix_nano = Some(time);
                }
            }
            MappingTarget::TraceId => {
                if let Some(id) = hex_id(value)? {
                    self.reserve(65)?;
                    self.output.trace_id = Some(id);
                }
            }
            MappingTarget::SpanId => {
                if let Some(id) = hex_id(value)? {
                    self.reserve(33)?;
                    self.output.span_id = Some(id);
                }
            }
            MappingTarget::TraceFlags => {
                self.reserve(10)?;
                let flags = value
                    .as_u64()
                    .or_else(|| value.as_str().and_then(|s| s.parse().ok()))
                    .and_then(|n| u32::try_from(n).ok())
                    .ok_or_else(|| MappingError::Rejected("invalid trace flags".into()))?;
                self.output.flags = Some(flags);
            }
            MappingTarget::Severity { map } => {
                let key = scalar(value)
                    .ok_or_else(|| MappingError::Rejected("severity requires a scalar".into()))?;
                let (number, text) = map
                    .get(&key)
                    .map_or((0, "UNSPECIFIED"), |e| (e.number, e.text.as_str()));
                self.reserve(json_size(&text, self.remaining)?)?;
                self.output.severity_number = Some(number);
                self.output.severity_text = Some(text.to_owned());
            }
            MappingTarget::Scope { field, split } => {
                let raw = value
                    .as_str()
                    .ok_or_else(|| MappingError::Rejected("scope requires a string".into()))?;
                self.reserve(json_size(&raw, self.remaining)?)?;
                match field {
                    ScopeField::Name => self.output.scope_name = Some(raw.to_owned()),
                    ScopeField::Version => self.output.scope_version = Some(raw.to_owned()),
                    ScopeField::NameVersion => {
                        let (name, version) = match split {
                            Some(ScopeSplit::LastChar(separator)) => {
                                raw.rsplit_once(separator.as_str()).unwrap_or((raw, ""))
                            }
                            Some(ScopeSplit::DashSemver) => split_dash_semver(raw),
                            None => split_dash_semver(raw),
                        };
                        self.output.scope_name = Some(name.to_owned());
                        self.output.scope_version = Some(version.to_owned());
                    }
                }
            }
        }
        Ok(())
    }
}

pub(super) fn allowed_child(
    keys: &[String],
    key: &str,
    budget: &mut Budget,
) -> Result<bool, MappingError> {
    budget.charge(key.len() + 1)?;
    let head = key.split('.').next();
    for allowed in keys {
        budget.charge(allowed.len().saturating_mul(2).saturating_add(1))?;
        if allowed == key || Some(allowed.as_str()) == head {
            return Ok(true);
        }
    }
    Ok(false)
}

fn validate(
    value: &Value,
    validation: Option<&Validation>,
    budget: &mut Budget,
) -> Result<Option<Value>, MappingError> {
    let Some(validation) = validation else {
        budget.charge(json_size(value, budget.remaining())?)?;
        return Ok(Some(value.clone()));
    };
    let failure = || match validation.on_validation_failure {
        ValidationFailure::Skip => Ok(None),
        ValidationFailure::Reject => Err(MappingError::Rejected("field type validation".into())),
    };
    let converted = match validation.expected_type {
        Some(ExpectedType::String) if !value.is_string() => return failure(),
        Some(ExpectedType::Integer) => match value
            .as_i64()
            .or_else(|| value.as_str().and_then(|s| s.parse().ok()))
        {
            Some(n) => Value::from(n),
            None => return failure(),
        },
        Some(ExpectedType::Float) if value.is_number() => value.clone(),
        Some(ExpectedType::Float) => match value
            .as_f64()
            .or_else(|| value.as_str().and_then(|s| s.parse().ok()))
            .and_then(serde_json::Number::from_f64)
        {
            Some(n) => Value::Number(n),
            None => return failure(),
        },
        Some(ExpectedType::Boolean) if !value.is_boolean() => return failure(),
        Some(ExpectedType::StringCoerce) if !value.is_string() => {
            let size = json_size(value, budget.remaining())?;
            budget.charge(size)?;
            Value::String(value.to_string())
        }
        _ => {
            budget.charge(json_size(value, budget.remaining())?)?;
            value.clone()
        }
    };
    if let (Some(max), Value::String(s)) = (validation.max_length, &converted)
        && s.len() > max
    {
        return match validation.on_length_exceeded {
            LengthExceeded::Reject => Err(MappingError::Rejected("field length limit".into())),
            LengthExceeded::Truncate => {
                let mut end = max;
                while !s.is_char_boundary(end) {
                    end -= 1;
                }
                Ok(Some(Value::String(s[..end].to_owned())))
            }
        };
    }
    Ok(Some(converted))
}

fn scalar(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

fn transform_value(
    transform: &ValueTransform,
    value: &Value,
    limit: usize,
    budget: &mut Budget,
) -> Result<Option<Value>, MappingError> {
    if let ValueTransform::Format { template } = transform {
        let Some(raw) = scalar(value) else {
            return Ok(None);
        };
        let count = template.matches("{value}").count();
        let size = (template.len() - count * "{value}".len())
            .checked_add(
                count
                    .checked_mul(raw.len())
                    .ok_or(MappingError::Limit("transform bytes"))?,
            )
            .ok_or(MappingError::Limit("transform bytes"))?;
        if size > limit {
            return Err(MappingError::Limit("transform bytes"));
        }
        budget.charge(size)?;
    }
    if let ValueTransform::SplitTake { delimiter, index } = transform {
        let Some(raw) = value.as_str() else {
            return Ok(None);
        };
        budget.charge(raw.len())?;
        let part = if *index < 0 {
            raw.rsplit(delimiter).nth(index.unsigned_abs() as usize - 1)
        } else {
            raw.split(delimiter).nth(*index as usize)
        };
        return Ok(part.map(|s| Value::String(s.to_owned())));
    }
    budget.charge(json_size(value, limit)?)?;
    Ok(transform.apply(value))
}

fn event_name(
    target: &MappingTarget,
    value: &Value,
    event: &Value,
    budget: &mut Budget,
    limits: Limits,
) -> Result<String, MappingError> {
    let MappingTarget::EventName {
        prefix,
        split_last,
        strip_segment,
    } = target
    else {
        return Err(MappingError::Config("event-name selector target".into()));
    };
    let raw = value
        .as_str()
        .ok_or_else(|| MappingError::Rejected("event name requires a string".into()))?;
    budget.charge(raw.len())?;
    let mut name = raw.to_owned();
    if let Some(strip) = strip_segment {
        let match_path = strip
            .match_field()
            .map(|p| p.split('.').map(str::to_owned).collect::<Vec<_>>());
        if let Some(path) = &match_path {
            budget.charge(path_work(path))?;
        }
        let field = match_path
            .as_ref()
            .and_then(|path| resolve(event, path))
            .and_then(Value::as_str);
        let mut offset = 0;
        for (index, segment) in raw.split('.').enumerate() {
            if index == strip.index() {
                let end = offset + segment.len();
                if strip.matches_segment(segment, field) {
                    name = if end < raw.len() {
                        format!("{}{}", &raw[..offset], &raw[end + 1..])
                    } else {
                        raw[..offset.saturating_sub(1)].to_owned()
                    };
                }
                break;
            }
            offset += segment.len() + 1;
        }
    }
    if let Some(separator) = split_last
        && let Some((_, tail)) = name.rsplit_once(separator.as_str())
    {
        name = tail.to_owned();
    }
    let prefix = prefix.as_deref().unwrap_or("");
    if prefix.len().saturating_add(name.len()) > limits.output_bytes {
        return Err(MappingError::Limit("event-name bytes"));
    }
    Ok(format!("{prefix}{name}"))
}

fn validate_name(name: &str, validation: &EventNameValidation) -> Result<(), MappingError> {
    let valid = match validation {
        EventNameValidation::None => true,
        EventNameValidation::PrintableAscii => {
            !name.is_empty() && name.bytes().all(|b| (b'!'..=b'~').contains(&b))
        }
        EventNameValidation::OtelSemconv => {
            name.bytes().next().is_some_and(|b| b.is_ascii_lowercase())
                && name
                    .bytes()
                    .last()
                    .is_some_and(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
                && name
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'.' || b == b'_')
                && !name
                    .as_bytes()
                    .windows(2)
                    .any(|p| b"._".contains(&p[0]) && b"._".contains(&p[1]))
        }
    };
    if valid {
        Ok(())
    } else {
        Err(MappingError::Rejected("invalid final event name".into()))
    }
}

fn hex_id<const N: usize>(value: &Value) -> Result<Option<[u8; N]>, MappingError> {
    let raw = value
        .as_str()
        .ok_or_else(|| MappingError::Rejected("identifier requires a string".into()))?;
    if raw.len() != N * 2 || !raw.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(MappingError::Rejected(
            "invalid hexadecimal identifier".into(),
        ));
    }
    let mut id = [0; N];
    for (byte, pair) in id.iter_mut().zip(raw.as_bytes().as_chunks::<2>().0) {
        let hex = |b: u8| {
            if b.is_ascii_digit() {
                b - b'0'
            } else {
                b.to_ascii_lowercase() - b'a' + 10
            }
        };
        *byte = hex(pair[0]) * 16 + hex(pair[1]);
    }
    Ok(id.iter().any(|byte| *byte != 0).then_some(id))
}

/// Literal dotted keys take precedence over nested traversal at each level.
fn resolve<'a>(value: &'a Value, path: &[String]) -> Option<&'a Value> {
    if path.is_empty() {
        return Some(value);
    }
    for length in (1..=path.len()).rev() {
        if let Some(child) = value.get(path[..length].join(".")) {
            return resolve(child, &path[length..]);
        }
    }
    None
}

fn path_work(path: &[String]) -> usize {
    path.iter()
        .map(String::len)
        .sum::<usize>()
        .saturating_add(path.len())
        .saturating_mul(path.len().saturating_add(1))
        .saturating_add(1)
}

fn insert_default(value: &mut Value, path: &[String], default: &Value) -> Result<(), MappingError> {
    if path.is_empty() {
        return Err(MappingError::Config("empty default path".into()));
    }
    let map = value
        .as_object_mut()
        .ok_or_else(|| MappingError::Rejected("default field has a non-object parent".into()))?;
    if path.len() == 1 {
        _ = map.insert(path[0].clone(), default.clone());
        return Ok(());
    }
    // Honor dotted literal parent keys using the same precedence as lookup.
    let length = (1..path.len())
        .rev()
        .find(|n| map.contains_key(&path[..*n].join(".")))
        .unwrap_or(1);
    let key = path[..length].join(".");
    let child = map.entry(key).or_insert_with(|| Value::Object(Map::new()));
    insert_default(child, &path[length..], default)
}
