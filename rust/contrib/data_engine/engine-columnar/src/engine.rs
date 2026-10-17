// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![allow(unused)]

use std::{
    cell::RefCell,
    fmt::{Debug, Display, Write},
};

use ahash::AHashSet;
use arrow::array::*;
use otel_arrow_contrib_data_engine_expressions::*;

use crate::{
    engine_diagnostic::{ColumnarEngineDiagnostic, ColumnarEngineDiagnosticLevel},
    *,
};

pub struct ColumnarEngineOptions {
    pub(crate) diagnostic_level: ColumnarEngineDiagnosticLevel,
}

impl Default for ColumnarEngineOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl ColumnarEngineOptions {
    pub fn new() -> ColumnarEngineOptions {
        Self {
            diagnostic_level: ColumnarEngineDiagnosticLevel::Warn,
        }
    }

    pub fn with_diagnostic_level(
        mut self,
        diagnostic_level: ColumnarEngineDiagnosticLevel,
    ) -> ColumnarEngineOptions {
        self.diagnostic_level = diagnostic_level;
        self
    }
}

#[derive(Debug)]
pub struct ColumnarEngine {
    diagnostic_level: ColumnarEngineDiagnosticLevel,
    pipeline: PipelineExpression,
}

impl ColumnarEngine {
    pub fn new(pipeline: PipelineExpression) -> ColumnarEngine {
        Self::new_with_options(pipeline, ColumnarEngineOptions::new())
    }

    pub fn new_with_options(
        mut pipeline: PipelineExpression,
        options: ColumnarEngineOptions,
    ) -> ColumnarEngine {
        for expression in pipeline.get_expressions_mut() {
            if let DataExpression::Discard(d) = expression
                && let Some(predicate) = d.get_predicate_mut()
            {
                // Note: The arrow filter operation used for discard by the
                // engine works inversely to what the expression tree is set up
                // to express so we invert the predicate once at construction.
                predicate.invert();
            }
        }

        Self {
            diagnostic_level: options.diagnostic_level,
            pipeline,
        }
    }

    pub fn get_pipeline(&self) -> &PipelineExpression {
        &self.pipeline
    }

    pub fn begin_batch<const BATCH_SIZE: usize>(&self) -> ColumnarEngineBatch<'_, BATCH_SIZE> {
        let mut batch = ColumnarEngineBatch::new(self);
        batch.initialize();
        batch
    }
}

pub struct ColumnarEngineBatch<'a, const BATCH_SIZE: usize> {
    engine: &'a ColumnarEngine,
    diagnostics: RefCell<Vec<ColumnarEngineDiagnostic<'a>>>,
    included_batches: Vec<[Option<RecordBatch>; BATCH_SIZE]>,
    included_record_count: usize,
    dropped_record_count: usize,
}

impl<'a, const BATCH_SIZE: usize> ColumnarEngineBatch<'a, BATCH_SIZE> {
    pub(crate) fn new(engine: &'a ColumnarEngine) -> ColumnarEngineBatch<'a, BATCH_SIZE> {
        Self {
            engine,
            diagnostics: RefCell::new(Vec::new()),
            included_batches: Vec::new(),
            included_record_count: 0,
            dropped_record_count: 0,
        }
    }

    pub(crate) fn initialize(&mut self) {
        todo!()
    }

    pub fn push_records<TRecordFactory: ColumnarRecordsFactory<BATCH_SIZE>>(
        &mut self,
        factory: &TRecordFactory,
        mut batches: [Option<RecordBatch>; BATCH_SIZE],
    ) {
        todo!()
    }

    pub fn flush(self) -> ColumnarEngineResults<'a, BATCH_SIZE> {
        ColumnarEngineResults {
            pipeline: &self.engine.pipeline,
            included_batches: self.included_batches,
            included_record_count: self.included_record_count,
            dropped_record_count: self.dropped_record_count,
            diagnostics: self.diagnostics.take(),
        }
    }
}

#[derive(Debug)]
pub struct ColumnarEngineResults<'a, const BATCH_SIZE: usize> {
    pub pipeline: &'a PipelineExpression,
    pub included_batches: Vec<[Option<RecordBatch>; BATCH_SIZE]>,
    pub included_record_count: usize,
    pub dropped_record_count: usize,
    pub diagnostics: Vec<ColumnarEngineDiagnostic<'a>>,
}

impl<const BATCH_SIZE: usize> Display for ColumnarEngineResults<'_, BATCH_SIZE> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format_diagnostics(self.pipeline.get_query(), &self.diagnostics, f)
    }
}

fn format_diagnostics(
    query: &str,
    diagnostics: &[ColumnarEngineDiagnostic<'_>],
    output: &mut dyn Write,
) -> std::fmt::Result {
    write_diagnostics(query, diagnostics, true, output)
}

fn write_diagnostics(
    query: &str,
    diagnostics: &[ColumnarEngineDiagnostic<'_>],
    all_lines: bool,
    output: &mut dyn Write,
) -> std::fmt::Result {
    let mut lines: Vec<(&str, Vec<&ColumnarEngineDiagnostic<'_>>)> = Vec::new();

    for line in query.lines() {
        lines.push((line, Vec::new()));
    }

    if lines.is_empty() {
        lines.push(("", Vec::new()));
    }

    for log in diagnostics {
        let location = log.get_expression().get_query_location();
        let (line, _) = location.get_line_and_column_numbers();
        if let Some(l) = lines.get_mut(line - 1) {
            l.1.push(log);
        } else {
            lines[0].1.push(log);
        }
    }

    let mut line_number = 1;
    let mut is_first_line = true;

    for (query_line, messages) in lines.iter_mut() {
        if !all_lines && messages.is_empty() {
            line_number += 1;
            continue;
        }

        if is_first_line {
            is_first_line = false;
        } else {
            output.write_char('\n')?;
        }

        messages.sort_by(|a, b| {
            let l = a
                .get_expression()
                .get_query_location()
                .get_line_and_column_numbers()
                .1;
            let r = b
                .get_expression()
                .get_query_location()
                .get_line_and_column_numbers()
                .1;
            r.cmp(&l)
        });

        let mut diagnostics = Vec::with_capacity(messages.len());
        let mut columns = AHashSet::new();

        for message in messages {
            let mut diagnostic = String::new();

            let (_, column) = message
                .get_expression()
                .get_query_location()
                .get_line_and_column_numbers();

            diagnostic.push_str(&" ".repeat(column + 7));
            diagnostic.push_str("| [");
            diagnostic.push_str(message.get_diagnostic_level().get_name());
            diagnostic.push_str("] ");
            diagnostic.push_str(message.get_expression().get_name());
            diagnostic.push_str(": ");
            diagnostic.push_str(message.get_message());

            diagnostics.push((column, diagnostic));

            columns.insert(column);

            if let Some(nested_diagnostics) = message.get_nested_diagnostics() {
                let mut nested = String::new();
                write_diagnostics(query, nested_diagnostics, false, &mut nested)?;
                for line in nested.lines() {
                    diagnostics.push((column, format!("{}|    {line}", " ".repeat(column + 7))));
                }
            }
        }

        let mut line = String::new();
        line.push_str(query_line);
        for (diagnostic_column, mut diagnostic) in diagnostics {
            line.push('\n');
            for column in &columns {
                if diagnostic_column > *column {
                    diagnostic.replace_range(column + 7..column + 8, "|");
                }
            }
            line.push_str(&diagnostic);
        }

        write!(output, "ln {line_number:>3}: {line}")?;
        line_number += 1;
    }

    Ok(())
}
