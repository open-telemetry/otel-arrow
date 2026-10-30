// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![allow(unused)]

use std::{cell::RefCell, fmt::Display, ops::Deref};

use otel_arrow_contrib_data_engine_expressions::*;

use crate::{engine_diagnostic::*, *};

pub struct ExecutionContext<'a, 'pipeline, TRecords: ColumnarRecords<'pipeline>> {
    diagnostics: ColumnarEngineDiagnosticReceiverImpl<'a, 'pipeline>,
    pipeline: &'pipeline PipelineExpression,
    records: Option<TRecords>,
}

impl<'a, 'pipeline, TRecords: ColumnarRecords<'pipeline>>
    ExecutionContext<'a, 'pipeline, TRecords>
{
    pub(crate) fn new(
        diagnostic_level: ColumnarEngineDiagnosticLevel,
        diagnostics: &'a RefCell<Vec<ColumnarEngineDiagnostic<'pipeline>>>,
        pipeline: &'pipeline PipelineExpression,
        records: Option<TRecords>,
    ) -> ExecutionContext<'a, 'pipeline, TRecords> {
        Self {
            diagnostics: ColumnarEngineDiagnosticReceiverImpl::new(diagnostic_level, diagnostics),
            pipeline,
            records,
        }
    }

    pub fn is_diagnostic_level_enabled(
        &self,
        diagnostic_level: ColumnarEngineDiagnosticLevel,
    ) -> bool {
        self.diagnostics
            .is_diagnostic_level_enabled(diagnostic_level)
    }

    pub fn add_diagnostic_if_enabled<F>(
        &self,
        diagnostic_level: ColumnarEngineDiagnosticLevel,
        expression: &'pipeline dyn Expression,
        generate_message: F,
    ) where
        F: FnOnce() -> String,
    {
        self.diagnostics
            .add_diagnostic_if_enabled(diagnostic_level, expression, generate_message)
    }

    pub fn add_diagnostic(&self, diagnostic: ColumnarEngineDiagnostic<'pipeline>) {
        self.diagnostics.add_diagnostic(diagnostic)
    }

    pub fn get_pipeline(&self) -> &'pipeline PipelineExpression {
        self.pipeline
    }

    pub fn get_records(&self) -> Option<&TRecords> {
        self.records.as_ref()
    }

    pub fn get_records_mut(&mut self) -> Option<&mut TRecords> {
        self.records.as_mut()
    }

    pub fn into_parts(self) -> Option<TRecords> {
        self.records
    }
}

#[cfg(test)]
impl<'pipeline, TRecords: ColumnarRecords<'pipeline>> Display
    for ExecutionContext<'_, 'pipeline, TRecords>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format_diagnostics(
            self.pipeline.get_query(),
            self.diagnostics.get_diagnostics().deref(),
            f,
        )
    }
}
