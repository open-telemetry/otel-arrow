use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::{Debug, Display, Write},
};

use data_engine_expressions::*;

use crate::{
    execution_context::*,
    logical_expressions::execute_logical_expression,
    summary::{summary_data_expression::execute_summary_data_expression, *},
    transform::transform_expressions::execute_transform_expression,
    *,
};

pub struct RecordSetEngineOptions {
    pub(crate) diagnostic_level: RecordSetEngineDiagnosticLevel,
    pub(crate) summary_cardinality_limit: usize,
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
            summary_cardinality_limit: 8192,
        }
    }

    pub fn with_diagnostic_level(
        mut self,
        diagnostic_level: RecordSetEngineDiagnosticLevel,
    ) -> RecordSetEngineOptions {
        self.diagnostic_level = diagnostic_level;
        self
    }

    pub fn with_summary_cardinality_limit(
        mut self,
        summary_cardinality_limit: usize,
    ) -> RecordSetEngineOptions {
        self.summary_cardinality_limit = summary_cardinality_limit;
        self
    }
}

pub struct RecordSetEngine {
    diagnostic_level: RecordSetEngineDiagnosticLevel,
    summary_cardinality_limit: usize,
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
            summary_cardinality_limit: options.summary_cardinality_limit,
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
    global_variables: RefCell<MapValueStorage<OwnedValue>>,
    summaries: Summaries,
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
            global_variables: RefCell::new(MapValueStorage::new(HashMap::new())),
            summaries: Summaries::new(engine.summary_cardinality_limit),
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
        RecordSetEngineResults::new(
            self.pipeline,
            self.summaries,
            self.included_records,
            Vec::new(),
        )
    }

    fn process_record<'d>(
        &self,
        attached_records: Option<&'d dyn AttachedRecords>,
        record: TRecord,
    ) -> RecordSetEngineResult<'a, 'c, TRecord> {
        let diagnostic_level = record
            .get_diagnostic_level()
            .unwrap_or(self.engine.diagnostic_level.clone());

        let execution_context = ExecutionContext::new(
            diagnostic_level,
            self.pipeline,
            &self.global_variables,
            &self.summaries,
            attached_records,
            record,
        );

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
                DataExpression::Summary(s) => {
                    match execute_summary_data_expression(&execution_context, s) {
                        Ok(_) => {
                            execution_context.add_diagnostic_if_enabled(
                                RecordSetEngineDiagnosticLevel::Info,
                                s,
                                || "Record summarized and dropped".into(),
                            );

                            return RecordSetEngineResult::Drop(execution_context.into());
                        }
                        Err(e) => {
                            execution_context.add_diagnostic_if_enabled(
                                RecordSetEngineDiagnosticLevel::Error,
                                s,
                                || e.to_string(),
                            );
                            break;
                        }
                    }
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

pub trait RecordSet<TRecord: Record>: Debug {
    fn drain<F>(&mut self, action: &mut F)
    where
        F: FnMut(Option<&dyn AttachedRecords>, TRecord);
}

pub trait Record: MapValueMut + AsStaticValue {
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

    pub fn get_record(&self) -> &TRecord {
        &self.record
    }

    pub fn get_diagnostics(&self) -> &Vec<RecordSetEngineDiagnostic<'b>> {
        &self.diagnostics
    }

    pub fn take_record(self) -> TRecord {
        self.record
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
    pub summaries: Vec<RecordSetEngineSummary>,
    pub included_records: Vec<RecordSetEngineRecord<'a, 'b, TRecord>>,
    pub dropped_records: Vec<RecordSetEngineRecord<'a, 'b, TRecord>>,
}

impl<'a, 'b, TRecord: Record> RecordSetEngineResults<'a, 'b, TRecord> {
    pub(crate) fn new(
        pipeline: &'a PipelineExpression,
        summaries: Summaries,
        included_records: Vec<RecordSetEngineRecord<'a, 'b, TRecord>>,
        dropped_records: Vec<RecordSetEngineRecord<'a, 'b, TRecord>>,
    ) -> RecordSetEngineResults<'a, 'b, TRecord> {
        Self {
            pipeline,
            summaries: summaries.into(),
            included_records,
            dropped_records,
        }
    }

    pub fn get_pipeline(&self) -> &PipelineExpression {
        self.pipeline
    }
}

#[derive(Debug)]
pub struct RecordSetEngineSummary {
    pub summary_id: String,
    pub group_by_values: Vec<(Box<str>, OwnedValue)>,
    pub aggregation_values: HashMap<Box<str>, SummaryAggregation>,
}

impl RecordSetEngineSummary {
    pub fn new(
        summary_id: String,
        group_by_values: Vec<(Box<str>, OwnedValue)>,
        aggregation_values: HashMap<Box<str>, SummaryAggregation>,
    ) -> RecordSetEngineSummary {
        Self {
            summary_id,
            group_by_values,
            aggregation_values,
        }
    }
}

impl From<Summaries> for Vec<RecordSetEngineSummary> {
    fn from(value: Summaries) -> Self {
        let mut values = value.values.borrow_mut();

        let mut results = Vec::with_capacity(values.len());

        for (summary_id, summary) in values.drain() {
            results.push(RecordSetEngineSummary::new(
                summary_id,
                summary.group_by_values,
                summary.aggregation_values,
            ));
        }

        results
    }
}
