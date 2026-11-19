// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Format-independent selector subset for nested fields and array iteration.

/// One traversal operation, independent of the source document format.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SelectorStep {
    /// Select a named object field.
    Field(String),
    /// Visit every array element in document order.
    Each,
}

/// A selector compiled once at configuration time.
///
/// Supports `$`, dot-separated ASCII field names, and `[*]`. Filters, recursive
/// descent, numeric indexes, and quoted field names are deliberately unsupported.
#[derive(Clone, Debug)]
pub struct CompiledSelector {
    steps: Box<[SelectorStep]>,
}

/// Invalid or unsupported selector syntax.
#[derive(Debug, thiserror::Error)]
#[error("selector must use $, .field, and [*], with at most 32 steps and 1024 bytes")]
pub struct SelectorError;

impl CompiledSelector {
    /// Validates and compiles the supported selector subset without retaining its text.
    pub fn compile(selector: &str) -> Result<Self, SelectorError> {
        if selector.len() > 1024 || !selector.starts_with('$') {
            return Err(SelectorError);
        }

        let mut remaining = &selector[1..];
        let mut steps = Vec::new();
        while !remaining.is_empty() {
            if steps.len() == 32 {
                return Err(SelectorError);
            }
            if let Some(rest) = remaining.strip_prefix("[*]") {
                steps.push(SelectorStep::Each);
                remaining = rest;
            } else if let Some(rest) = remaining.strip_prefix('.') {
                let length = rest
                    .bytes()
                    .take_while(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
                    .count();
                if length == 0 {
                    return Err(SelectorError);
                }
                steps.push(SelectorStep::Field(rest[..length].to_owned()));
                remaining = &rest[length..];
            } else {
                return Err(SelectorError);
            }
        }
        Ok(Self {
            steps: steps.into_boxed_slice(),
        })
    }

    /// Returns the traversal operations in selection order.
    #[must_use]
    pub fn steps(&self) -> &[SelectorStep] {
        &self.steps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: A selector addresses the root or nested fields across multiple arrays.
    /// Guarantees: Compilation preserves field names, iteration order, and root identity.
    #[test]
    fn compiles_root_and_nested_array_paths() {
        assert!(CompiledSelector::compile("$").unwrap().steps().is_empty());
        assert_eq!(
            CompiledSelector::compile("$.groups[*].events[*].message_1")
                .unwrap()
                .steps(),
            &[
                SelectorStep::Field("groups".into()),
                SelectorStep::Each,
                SelectorStep::Field("events".into()),
                SelectorStep::Each,
                SelectorStep::Field("message_1".into()),
            ]
        );
        assert_eq!(
            CompiledSelector::compile("$[*]").unwrap().steps(),
            &[SelectorStep::Each]
        );
    }

    /// Scenario: Configuration supplies malformed selectors or unsupported JSONPath syntax.
    /// Guarantees: Invalid roots, empty fields, indexes, filters, and quoted names fail at startup.
    #[test]
    fn rejects_unsupported_paths() {
        for path in [
            "",
            "message",
            "@.message",
            "$.",
            "$..message",
            "$.*",
            "$[0]",
            "$['message']",
            "$[?(@.x)]",
            "$.events[",
            "$.field-name",
            "$.field name",
            "$.caf\u{e9}",
        ] {
            assert!(CompiledSelector::compile(path).is_err(), "{path:?}");
        }
    }

    /// Scenario: Selectors reach the byte and traversal-step limits.
    /// Guarantees: Exact limits compile and a single extra byte or step is rejected.
    #[test]
    fn enforces_selector_boundaries() {
        let bytes = format!("$.{}", "x".repeat(1022));
        assert!(CompiledSelector::compile(&bytes).is_ok());
        assert!(CompiledSelector::compile(&(bytes + "x")).is_err());
        let steps = format!("${}", "[*]".repeat(32));
        assert!(CompiledSelector::compile(&steps).is_ok());
        assert!(CompiledSelector::compile(&(steps + "[*]")).is_err());
    }
}
