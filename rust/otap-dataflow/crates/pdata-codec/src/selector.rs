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
