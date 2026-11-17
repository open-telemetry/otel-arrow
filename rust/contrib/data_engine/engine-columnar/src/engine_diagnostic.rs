// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![allow(unused)]

use std::{
    cell::{Ref, RefCell},
    str::FromStr,
};

use otel_arrow_contrib_data_engine_expressions::Expression;

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum ColumnarEngineDiagnosticLevel {
    Verbose = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
}

impl ColumnarEngineDiagnosticLevel {
    pub fn get_name(&self) -> &str {
        match self {
            ColumnarEngineDiagnosticLevel::Verbose => "Verbose",
            ColumnarEngineDiagnosticLevel::Info => "Info",
            ColumnarEngineDiagnosticLevel::Warn => "Warn",
            ColumnarEngineDiagnosticLevel::Error => "Error",
        }
    }

    pub fn from_usize(n: usize) -> Option<ColumnarEngineDiagnosticLevel> {
        match n {
            0 => Some(ColumnarEngineDiagnosticLevel::Verbose),
            1 => Some(ColumnarEngineDiagnosticLevel::Info),
            2 => Some(ColumnarEngineDiagnosticLevel::Warn),
            3 => Some(ColumnarEngineDiagnosticLevel::Error),
            _ => None,
        }
    }
}

impl FromStr for ColumnarEngineDiagnosticLevel {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Verbose" | "verbose" => Ok(ColumnarEngineDiagnosticLevel::Verbose),
            "Info" | "info" => Ok(ColumnarEngineDiagnosticLevel::Info),
            "Warn" | "warn" => Ok(ColumnarEngineDiagnosticLevel::Warn),
            "Error" | "error" => Ok(ColumnarEngineDiagnosticLevel::Error),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub struct ColumnarEngineDiagnostic<'a> {
    diagnostic_level: ColumnarEngineDiagnosticLevel,
    expression: &'a dyn Expression,
    message: String,
    nested_diagnostics: Option<Vec<ColumnarEngineDiagnostic<'a>>>,
}

impl<'a> ColumnarEngineDiagnostic<'a> {
    pub(crate) fn new(
        diagnostic_level: ColumnarEngineDiagnosticLevel,
        expression: &'a dyn Expression,
        message: String,
    ) -> ColumnarEngineDiagnostic<'a> {
        Self {
            diagnostic_level,
            expression,
            message,
            nested_diagnostics: None,
        }
    }

    pub(crate) fn with_nested_diagnostics(
        mut self,
        nested_diagnostics: Vec<ColumnarEngineDiagnostic<'a>>,
    ) -> ColumnarEngineDiagnostic<'a> {
        self.nested_diagnostics = Some(nested_diagnostics);
        self
    }

    pub fn get_diagnostic_level(&self) -> ColumnarEngineDiagnosticLevel {
        self.diagnostic_level
    }

    pub fn get_expression(&self) -> &dyn Expression {
        self.expression
    }

    pub fn get_message(&self) -> &str {
        &self.message
    }

    pub fn get_nested_diagnostics(&self) -> Option<&[ColumnarEngineDiagnostic<'a>]> {
        self.nested_diagnostics.as_ref().map(|v| &v[..])
    }
}

pub trait ColumnarEngineDiagnosticReceiver<'a> {
    fn is_diagnostic_level_enabled(&self, diagnostic_level: ColumnarEngineDiagnosticLevel) -> bool;

    fn add_diagnostic_if_enabled<F>(
        &self,
        diagnostic_level: ColumnarEngineDiagnosticLevel,
        expression: &'a dyn Expression,
        generate_message: F,
    ) where
        F: FnOnce() -> String;

    fn add_diagnostic(&self, diagnostic: ColumnarEngineDiagnostic<'a>);

    fn bind(
        &self,
        expression: &'a dyn Expression,
    ) -> ColumnarEngineBoundDiagnosticReceiver<'a, '_, Self>
    where
        Self: Sized,
    {
        ColumnarEngineBoundDiagnosticReceiver {
            receiver: self,
            expression,
        }
    }
}

/// A diagnostic receiver that attributes messages to a single expression.
pub struct ColumnarEngineBoundDiagnosticReceiver<'a, 'b, R: ColumnarEngineDiagnosticReceiver<'a>> {
    receiver: &'b R,
    expression: &'a dyn Expression,
}

impl<'a, 'b, R: ColumnarEngineDiagnosticReceiver<'a>>
    ColumnarEngineBoundDiagnosticReceiver<'a, 'b, R>
{
    pub fn is_diagnostic_level_enabled(
        &self,
        diagnostic_level: ColumnarEngineDiagnosticLevel,
    ) -> bool {
        self.receiver.is_diagnostic_level_enabled(diagnostic_level)
    }

    pub fn add_diagnostic_if_enabled<F>(
        &self,
        diagnostic_level: ColumnarEngineDiagnosticLevel,
        generate_message: F,
    ) where
        F: FnOnce() -> String,
    {
        self.receiver.add_diagnostic_if_enabled(
            diagnostic_level,
            self.expression,
            generate_message,
        );
    }
}

pub(crate) struct ColumnarEngineDiagnosticReceiverImpl<'a, 'pipeline> {
    diagnostic_level: ColumnarEngineDiagnosticLevel,
    diagnostics: &'a RefCell<Vec<ColumnarEngineDiagnostic<'pipeline>>>,
}

impl<'a, 'pipeline> ColumnarEngineDiagnosticReceiverImpl<'a, 'pipeline> {
    pub fn new(
        diagnostic_level: ColumnarEngineDiagnosticLevel,
        diagnostics: &'a RefCell<Vec<ColumnarEngineDiagnostic<'pipeline>>>,
    ) -> ColumnarEngineDiagnosticReceiverImpl<'a, 'pipeline> {
        Self {
            diagnostic_level,
            diagnostics,
        }
    }

    pub fn get_diagnostics(&self) -> Ref<'_, Vec<ColumnarEngineDiagnostic<'pipeline>>> {
        self.diagnostics.borrow()
    }
}

impl<'a> ColumnarEngineDiagnosticReceiver<'a> for ColumnarEngineDiagnosticReceiverImpl<'_, 'a> {
    fn is_diagnostic_level_enabled(&self, diagnostic_level: ColumnarEngineDiagnosticLevel) -> bool {
        diagnostic_level >= self.diagnostic_level
    }

    fn add_diagnostic_if_enabled<F>(
        &self,
        diagnostic_level: ColumnarEngineDiagnosticLevel,
        expression: &'a dyn Expression,
        generate_message: F,
    ) where
        F: FnOnce() -> String,
    {
        if diagnostic_level >= self.diagnostic_level {
            let diagnostic =
                ColumnarEngineDiagnostic::new(diagnostic_level, expression, generate_message());
            self.diagnostics.borrow_mut().push(diagnostic);
        }
    }

    fn add_diagnostic(&self, diagnostic: ColumnarEngineDiagnostic<'a>) {
        self.diagnostics.borrow_mut().push(diagnostic);
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helpers::run_bound_diagnostic_test_with_level;

    use super::*;

    /// Scenario: An expression-bound receiver filters warnings at each configured diagnostic level.
    /// Guarantees: Disabled messages are never generated, and enabled warnings retain their level, message, and exact expression.
    #[test]
    fn bound_receiver_filters_and_attributes_warnings() {
        for level in [
            ColumnarEngineDiagnosticLevel::Verbose,
            ColumnarEngineDiagnosticLevel::Info,
            ColumnarEngineDiagnosticLevel::Warn,
            ColumnarEngineDiagnosticLevel::Error,
        ] {
            let enabled = level <= ColumnarEngineDiagnosticLevel::Warn;
            run_bound_diagnostic_test_with_level(
                level,
                |bound| {
                    assert_eq!(
                        bound.is_diagnostic_level_enabled(ColumnarEngineDiagnosticLevel::Warn),
                        enabled
                    );
                    bound.add_diagnostic_if_enabled(ColumnarEngineDiagnosticLevel::Warn, || {
                        assert!(enabled, "disabled diagnostic must not generate a message");
                        "warning".into()
                    });
                },
                |diagnostics| {
                    assert_eq!(diagnostics.len(), usize::from(enabled));
                    if enabled {
                        assert_eq!(
                            diagnostics[0].get_diagnostic_level(),
                            ColumnarEngineDiagnosticLevel::Warn
                        );
                        assert_eq!(diagnostics[0].get_message(), "warning");
                    }
                },
            );
        }
    }
}
