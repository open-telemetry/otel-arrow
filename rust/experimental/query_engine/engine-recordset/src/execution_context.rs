// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::{cell::*, collections::HashMap};

use data_engine_expressions::{AsStaticValue, Expression, MapValue, PipelineExpression};

#[cfg(test)]
use crate::TestRecord;
use crate::*;

pub(crate) struct ExecutionContext<'a, 'b, 'c, TRecord>
where
    'a: 'c,
    TRecord: MapValue + 'static,
{
    diagnostic_level: RecordSetEngineDiagnosticLevel,
    diagnostics: RefCell<Vec<RecordSetEngineDiagnostic<'c>>>,
    pipeline: &'a PipelineExpression,
    variables: ExecutionContextVariables<'b>,
    summaries: &'b Summaries<'a>,
    attached_records: Option<&'b dyn AttachedRecords>,
    record: Option<RefCell<TRecord>>,
}

impl<'a, 'b, 'c, TRecord: Record + 'static> ExecutionContext<'a, 'b, 'c, TRecord> {
    pub fn new(
        diagnostic_level: RecordSetEngineDiagnosticLevel,
        pipeline: &'a PipelineExpression,
        global_variables: &'b RefCell<MapValueStorage<OwnedValue>>,
        summaries: &'b Summaries<'a>,
        attached_records: Option<&'b dyn AttachedRecords>,
        record: Option<TRecord>,
    ) -> ExecutionContext<'a, 'b, 'c, TRecord> {
        Self {
            diagnostic_level,
            diagnostics: RefCell::new(Vec::new()),
            pipeline,
            attached_records,
            record: record.map(|v| RefCell::new(v)),
            variables: ExecutionContextVariables::new(global_variables),
            summaries,
        }
    }

    pub fn is_diagnostic_level_enabled(
        &self,
        diagnostic_level: RecordSetEngineDiagnosticLevel,
    ) -> bool {
        diagnostic_level >= self.diagnostic_level
    }

    pub fn add_diagnostic_if_enabled<F>(
        &self,
        diagnostic_level: RecordSetEngineDiagnosticLevel,
        expression: &'a dyn Expression,
        generate_message: F,
    ) where
        F: FnOnce() -> String,
    {
        if diagnostic_level >= self.diagnostic_level {
            self.diagnostics
                .borrow_mut()
                .push(RecordSetEngineDiagnostic::new(
                    diagnostic_level,
                    expression,
                    (generate_message)(),
                ));
        }
    }

    pub fn add_diagnostic(&self, diagnostic: RecordSetEngineDiagnostic<'c>) {
        self.diagnostics.borrow_mut().push(diagnostic);
    }

    pub fn get_pipeline(&self) -> &'a PipelineExpression {
        self.pipeline
    }

    pub fn get_attached_records(&self) -> Option<&'b dyn AttachedRecords> {
        self.attached_records
    }

    pub fn get_record(&self) -> Option<&RefCell<TRecord>> {
        self.record.as_ref()
    }

    pub fn get_variables(&self) -> &ExecutionContextVariables<'b> {
        &self.variables
    }

    pub fn get_summaries(&self) -> &Summaries<'a> {
        self.summaries
    }

    pub fn take_diagnostics(self) -> Vec<RecordSetEngineDiagnostic<'c>> {
        self.diagnostics.take()
    }

    pub fn consume_into_record(self) -> RecordSetEngineRecord<'a, 'c, TRecord> {
        RecordSetEngineRecord::new(
            self.pipeline,
            self.record.expect("record wasn't set").into_inner(),
            self.diagnostics.take(),
        )
    }
}

pub(crate) struct ExecutionContextVariables<'a> {
    global_variables: &'a RefCell<MapValueStorage<OwnedValue>>,
    local_variables: RefCell<MapValueStorage<OwnedValue>>,
}

impl<'a> ExecutionContextVariables<'a> {
    pub fn new(global_variables: &'a RefCell<MapValueStorage<OwnedValue>>) -> Self {
        Self {
            global_variables,
            local_variables: RefCell::new(MapValueStorage::new(HashMap::new())),
        }
    }

    pub fn get_global_or_local_variable(
        &self,
        name: &str,
    ) -> Option<Ref<'_, dyn AsStaticValue + 'static>> {
        let vars = self.local_variables.borrow();

        let var = Ref::filter_map(vars, |v| v.get(name));

        if let Ok(v) = var {
            return Some(v);
        }

        Ref::filter_map(self.global_variables.borrow(), |v| v.get(name)).ok()
    }

    #[cfg(test)]
    pub fn get_local_variables(&self) -> Ref<'_, MapValueStorage<OwnedValue>> {
        self.local_variables.borrow()
    }

    pub fn get_local_variables_mut(&self) -> RefMut<'_, MapValueStorage<OwnedValue>> {
        self.local_variables.borrow_mut()
    }

    #[cfg(test)]
    pub fn get_global_variables(&self) -> Ref<'_, MapValueStorage<OwnedValue>> {
        self.global_variables.borrow()
    }
}

#[cfg(test)]
pub struct TestExecutionContext<'a> {
    pipeline: PipelineExpression,
    global_variables: RefCell<MapValueStorage<OwnedValue>>,
    summaries: Summaries<'a>,
    attached_records: Option<TestAttachedRecords>,
    record: Option<TestRecord>,
}

#[cfg(test)]
impl<'a> TestExecutionContext<'a> {
    pub fn new() -> TestExecutionContext<'a> {
        Self {
            pipeline: Default::default(),
            global_variables: RefCell::new(MapValueStorage::new(HashMap::new())),
            summaries: Summaries::new(8192),
            attached_records: None,
            record: None,
        }
    }

    pub fn with_pipeline(mut self, pipeline: PipelineExpression) -> TestExecutionContext<'a> {
        self.pipeline = pipeline;
        self
    }

    pub fn with_attached_records(
        mut self,
        attached_records: TestAttachedRecords,
    ) -> TestExecutionContext<'a> {
        self.attached_records = Some(attached_records);
        self
    }

    pub fn with_record(mut self, record: TestRecord) -> TestExecutionContext<'a> {
        self.record = Some(record);
        self
    }

    pub fn set_global_variable(&self, name: &str, value: ResolvedValue) {
        self.global_variables.borrow_mut().set(name, value);
    }

    pub fn create_execution_context(&mut self) -> ExecutionContext<'_, 'a, '_, TestRecord> {
        ExecutionContext::new(
            RecordSetEngineDiagnosticLevel::Verbose,
            &self.pipeline,
            &self.global_variables,
            &self.summaries,
            self.attached_records
                .as_ref()
                .map(|v| v as &dyn AttachedRecords),
            self.record.take(),
        )
    }
}
