use std::{
    fmt::{Display, Write},
    time::SystemTime,
};

use data_engine_expressions::*;

use crate::{
    execution_context::ExecutionContext, logical_expressions::execute_logical_expression,
    transform::transform_expressions::execute_transform_expression, *,
};

pub struct RecordSetEngineOptions {
    pub(crate) diagnostic_level: RecordSetEngineDiagnosticLevel,
}

impl Default for RecordSetEngineOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl RecordSetEngineOptions {
    pub fn new() -> RecordSetEngineOptions {
        Self {
            diagnostic_level: RecordSetEngineDiagnosticLevel::Warn,
        }
    }

    pub fn with_diagnostic_level(
        mut self,
        diagnostic_level: RecordSetEngineDiagnosticLevel,
    ) -> RecordSetEngineOptions {
        self.diagnostic_level = diagnostic_level;
        self
    }
}

pub struct RecordSetEngine {
    diagnostic_level: RecordSetEngineDiagnosticLevel,
}

impl Default for RecordSetEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl RecordSetEngine {
    pub fn new() -> RecordSetEngine {
        Self::new_with_options(RecordSetEngineOptions::new())
    }

    pub fn new_with_options(options: RecordSetEngineOptions) -> RecordSetEngine {
        Self {
            diagnostic_level: options.diagnostic_level,
        }
    }

    pub fn begin_batch<'a, 'b, 'c, TRecord: Record + 'static>(
        &'b self,
        pipeline: &'a PipelineExpression,
    ) -> RecordSetEngineBatch<'a, 'b, 'c, TRecord>
    where
        'a: 'c,
        'b: 'c,
    {
        RecordSetEngineBatch::new(pipeline, self)
    }
}

pub struct RecordSetEngineBatch<'a, 'b, 'c, TRecord: Record> {
    pipeline: &'a PipelineExpression,
    engine: &'b RecordSetEngine,
    included_records: Vec<RecordSetEngineRecord<'a, 'c, TRecord>>,
}

impl<'a, 'b, 'c, TRecord: Record + 'static> RecordSetEngineBatch<'a, 'b, 'c, TRecord>
where
    'a: 'c,
    'b: 'c,
{
    pub(crate) fn new(
        pipeline: &'a PipelineExpression,
        engine: &'b RecordSetEngine,
    ) -> RecordSetEngineBatch<'a, 'b, 'c, TRecord> {
        Self {
            engine,
            pipeline,
            included_records: Vec::new(),
        }
    }

    pub fn push_records<TRecords: RecordSet<TRecord>>(
        &mut self,
        records: &mut TRecords,
    ) -> Vec<RecordSetEngineRecord<'a, 'c, TRecord>> {
        let mut dropped_records = Vec::new();

        records.drain(&mut |attached_records, record| match self
            .process_record(attached_records, record)
        {
            RecordSetEngineResult::Drop(d) => dropped_records.push(d),
            RecordSetEngineResult::Include(i) => self.included_records.push(i),
        });

        dropped_records
    }

    pub fn flush(self) -> RecordSetEngineResults<'a, 'c, TRecord> {
        RecordSetEngineResults::new(self.pipeline, self.included_records, Vec::new())
    }

    fn process_record<'d>(
        &self,
        attached_records: Option<&'d dyn AttachedRecords>,
        record: TRecord,
    ) -> RecordSetEngineResult<'a, 'c, TRecord> {
        let diagnostic_level = record
            .get_diagnostic_level()
            .unwrap_or(self.engine.diagnostic_level.clone());

        let execution_context =
            ExecutionContext::new(diagnostic_level, self.pipeline, attached_records, record);

        if execution_context.is_diagnostic_level_enabled(RecordSetEngineDiagnosticLevel::Verbose) {
            for (constant_id, constant) in self.pipeline.get_constants().iter().enumerate() {
                execution_context.add_diagnostic(RecordSetEngineDiagnostic::new(
                    RecordSetEngineDiagnosticLevel::Verbose,
                    constant,
                    format!("Constant defined with id '{constant_id}'"),
                ));
            }
        }

        for expression in self.pipeline.get_expressions() {
            match expression {
                DataExpression::Discard(d) => {
                    if let Some(predicate) = d.get_predicate() {
                        match execute_logical_expression(&execution_context, predicate) {
                            Ok(logical_result) => {
                                if !logical_result {
                                    execution_context.add_diagnostic_if_enabled(
                                        RecordSetEngineDiagnosticLevel::Verbose,
                                        d,
                                        || "Record included".into(),
                                    );
                                    continue;
                                }
                            }
                            Err(e) => {
                                execution_context.add_diagnostic_if_enabled(
                                    RecordSetEngineDiagnosticLevel::Error,
                                    d,
                                    || e.to_string(),
                                );
                                break;
                            }
                        }
                    }

                    execution_context.add_diagnostic_if_enabled(
                        RecordSetEngineDiagnosticLevel::Info,
                        d,
                        || "Record dropped".into(),
                    );

                    return RecordSetEngineResult::Drop(execution_context.into());
                }
                DataExpression::Transform(t) => {
                    match execute_transform_expression(&execution_context, t) {
                        Ok(_) => {}
                        Err(e) => {
                            execution_context.add_diagnostic_if_enabled(
                                RecordSetEngineDiagnosticLevel::Error,
                                t,
                                || e.to_string(),
                            );
                            break;
                        }
                    }
                }
            }
        }

        RecordSetEngineResult::Include(execution_context.into())
    }
}

pub trait RecordSet<TRecord: Record> {
    fn drain<F>(&mut self, action: &mut F)
    where
        F: FnMut(Option<&dyn AttachedRecords>, TRecord);
}

pub trait Record: MapValueMut {
    fn get_timestamp(&self) -> Option<SystemTime>;

    fn get_observed_timestamp(&self) -> Option<SystemTime>;

    fn get_diagnostic_level(&self) -> Option<RecordSetEngineDiagnosticLevel>;
}

pub trait AttachedRecords {
    fn get_attached_record(&self, name: &str) -> Option<&(dyn MapValue + 'static)>;
}

pub enum RecordSetEngineResult<'a, 'b, TRecord: Record> {
    Drop(RecordSetEngineRecord<'a, 'b, TRecord>),
    Include(RecordSetEngineRecord<'a, 'b, TRecord>),
}

#[derive(Debug)]
pub struct RecordSetEngineRecord<'a, 'b, TRecord: Record> {
    pipeline: &'a PipelineExpression,
    record: TRecord,
    diagnostics: Vec<RecordSetEngineDiagnostic<'b>>,
}

impl<'a, 'b, TRecord: Record> RecordSetEngineRecord<'a, 'b, TRecord> {
    pub(crate) fn new(
        pipeline: &'a PipelineExpression,
        record: TRecord,
        diagnostics: Vec<RecordSetEngineDiagnostic<'b>>,
    ) -> RecordSetEngineRecord<'a, 'b, TRecord> {
        Self {
            pipeline,
            record,
            diagnostics,
        }
    }

    pub fn take_record(self) -> TRecord {
        self.record
    }

    pub fn get_diagnostics(&self) -> &Vec<RecordSetEngineDiagnostic<'b>> {
        &self.diagnostics
    }
}

impl<'a, 'b, 'c, TRecord: Record> From<ExecutionContext<'a, 'b, 'c, TRecord>>
    for RecordSetEngineRecord<'a, 'c, TRecord>
{
    fn from(val: ExecutionContext<'a, 'b, 'c, TRecord>) -> Self {
        val.consume_into_record()
    }
}

impl<TRecord: Record> Display for RecordSetEngineRecord<'_, '_, TRecord> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut lines: Vec<(&str, Vec<&RecordSetEngineDiagnostic<'_>>)> = Vec::new();

        for line in self.pipeline.get_query().lines() {
            lines.push((line, Vec::new()));
        }

        for log in &self.diagnostics {
            let location = log.get_expression().get_query_location();
            let (line, _) = location.get_line_and_column_numbers();
            lines[line - 1].1.push(log);
        }

        let mut line_number = 1;

        for (query_line, messages) in lines.iter_mut() {
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

            let mut line = String::new();
            line.push_str(query_line);
            for message in messages {
                line.push('\n');
                let (_, column) = message
                    .get_expression()
                    .get_query_location()
                    .get_line_and_column_numbers();

                line.push_str(&" ".repeat(column + 7));
                line.push('[');
                line.push_str(message.get_diagnostic_level().get_name());
                line.push_str("] ");
                line.push_str(message.get_expression().get_name());
                line.push_str(": ");
                line.push_str(message.get_message());
            }
            if line_number > 1 {
                f.write_char('\n')?;
            }
            write!(f, "ln {line_number:>3}: {line}")?;
            line_number += 1;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct RecordSetEngineResults<'a, 'b, TRecord: Record> {
    pipeline: &'a PipelineExpression,
    pub included_records: Vec<RecordSetEngineRecord<'a, 'b, TRecord>>,
    pub dropped_records: Vec<RecordSetEngineRecord<'a, 'b, TRecord>>,
}

impl<'a, 'b, TRecord: Record> RecordSetEngineResults<'a, 'b, TRecord> {
    pub(crate) fn new(
        pipeline: &'a PipelineExpression,
        included_records: Vec<RecordSetEngineRecord<'a, 'b, TRecord>>,
        dropped_records: Vec<RecordSetEngineRecord<'a, 'b, TRecord>>,
    ) -> RecordSetEngineResults<'a, 'b, TRecord> {
        Self {
            pipeline,
            included_records,
            dropped_records,
        }
    }

    pub fn get_pipeline(&self) -> &PipelineExpression {
        self.pipeline
    }
}
