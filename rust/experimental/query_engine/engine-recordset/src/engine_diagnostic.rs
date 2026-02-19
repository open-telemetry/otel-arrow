// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use data_engine_expressions::Expression;

#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RecordSetEngineDiagnosticLevel {
    #[default]
    Verbose = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
}

impl RecordSetEngineDiagnosticLevel {
    pub fn get_name(&self) -> &str {
        match self {
            RecordSetEngineDiagnosticLevel::Verbose => "Verbose",
            RecordSetEngineDiagnosticLevel::Info => "Info",
            RecordSetEngineDiagnosticLevel::Warn => "Warn",
            RecordSetEngineDiagnosticLevel::Error => "Error",
        }
    }

    pub fn from_usize(n: usize) -> Option<RecordSetEngineDiagnosticLevel> {
        match n {
            0 => Some(RecordSetEngineDiagnosticLevel::Verbose),
            1 => Some(RecordSetEngineDiagnosticLevel::Info),
            2 => Some(RecordSetEngineDiagnosticLevel::Warn),
            3 => Some(RecordSetEngineDiagnosticLevel::Error),
            _ => None,
        }
    }
}

impl From<RecordSetEngineDiagnosticLevel> for tracing::Level {
    fn from(level: RecordSetEngineDiagnosticLevel) -> Self {
        match level {
            RecordSetEngineDiagnosticLevel::Verbose => tracing::Level::DEBUG,
            RecordSetEngineDiagnosticLevel::Info => tracing::Level::INFO,
            RecordSetEngineDiagnosticLevel::Warn => tracing::Level::WARN,
            RecordSetEngineDiagnosticLevel::Error => tracing::Level::ERROR,
        }
    }
}

impl From<tracing::Level> for RecordSetEngineDiagnosticLevel {
    fn from(level: tracing::Level) -> Self {
        match level {
            tracing::Level::TRACE | tracing::Level::DEBUG => {
                RecordSetEngineDiagnosticLevel::Verbose
            }
            tracing::Level::INFO => RecordSetEngineDiagnosticLevel::Info,
            tracing::Level::WARN => RecordSetEngineDiagnosticLevel::Warn,
            tracing::Level::ERROR => RecordSetEngineDiagnosticLevel::Error,
        }
    }
}

impl FromStr for RecordSetEngineDiagnosticLevel {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Verbose" | "verbose" => Ok(RecordSetEngineDiagnosticLevel::Verbose),
            "Info" | "info" => Ok(RecordSetEngineDiagnosticLevel::Verbose),
            "Warn" | "warn" => Ok(RecordSetEngineDiagnosticLevel::Warn),
            "Error" | "error" => Ok(RecordSetEngineDiagnosticLevel::Error),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub struct RecordSetEngineDiagnostic<'a> {
    diagnostic_level: RecordSetEngineDiagnosticLevel,
    expression: &'a dyn Expression,
    message: String,
    nested_diagnostics: Option<Vec<RecordSetEngineDiagnostic<'a>>>,
}

impl<'a> RecordSetEngineDiagnostic<'a> {
    pub(crate) fn new(
        diagnostic_level: RecordSetEngineDiagnosticLevel,
        expression: &'a dyn Expression,
        message: String,
    ) -> RecordSetEngineDiagnostic<'a> {
        Self {
            diagnostic_level,
            expression,
            message,
            nested_diagnostics: None,
        }
    }

    pub(crate) fn with_nested_diagnostics(
        mut self,
        nested_diagnostics: Vec<RecordSetEngineDiagnostic<'a>>,
    ) -> RecordSetEngineDiagnostic<'a> {
        self.nested_diagnostics = Some(nested_diagnostics);
        self
    }

    pub fn get_diagnostic_level(&self) -> RecordSetEngineDiagnosticLevel {
        self.diagnostic_level.clone()
    }

    pub fn get_expression(&self) -> &dyn Expression {
        self.expression
    }

    pub fn get_message(&self) -> &str {
        &self.message
    }

    pub fn get_nested_diagnostics(&self) -> Option<&[RecordSetEngineDiagnostic<'a>]> {
        self.nested_diagnostics.as_ref().map(|v| &v[..])
    }
}
