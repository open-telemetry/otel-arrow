// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use data_engine_expressions::Expression;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum RecordSetEngineDiagnosticLevel {
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
        }
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
}
