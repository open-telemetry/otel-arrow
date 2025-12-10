// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::{cell::*, collections::HashMap};

use crate::*;

use data_engine_expressions::*;

#[cfg(test)]
use crate::TestRecord;
#[cfg(test)]
use crate::{
    scalars::{
        execute_argument_scalar_expression, execute_scalar_expression,
        execute_source_scalar_expression, execute_variable_scalar_expression,
    },
    value_expressions::execute_mutable_value_expression,
};

pub(crate) struct ExecutionContext<'a, 'b, TRecord>
where
    TRecord: MapValue + 'static,
{
    diagnostic_level: RecordSetEngineDiagnosticLevel,
    diagnostics: RefCell<Vec<RecordSetEngineDiagnostic<'a>>>,
    pipeline: &'a PipelineExpression,
    variables: ExecutionContextVariables<'b>,
    summaries: &'b Summaries<'a>,
    attached_records: Option<&'b dyn AttachedRecords>,
    record: Option<RefCell<TRecord>>,
    arguments: Option<&'b dyn ExecutionContextArguments>,
}

impl<'a, 'b, TRecord: Record + 'static> ExecutionContext<'a, 'b, TRecord> {
    pub fn new(
        diagnostic_level: RecordSetEngineDiagnosticLevel,
        pipeline: &'a PipelineExpression,
        global_variables: &'b RefCell<MapValueStorage<OwnedValue>>,
        summaries: &'b Summaries<'a>,
        attached_records: Option<&'b dyn AttachedRecords>,
        record: Option<TRecord>,
        arguments: Option<&'b dyn ExecutionContextArguments>,
    ) -> ExecutionContext<'a, 'b, TRecord> {
        Self {
            diagnostic_level,
            diagnostics: RefCell::new(Vec::new()),
            pipeline,
            attached_records,
            record: record.map(|v| RefCell::new(v)),
            variables: ExecutionContextVariables::new(global_variables),
            summaries,
            arguments,
        }
    }

    #[cfg(test)]
    pub fn create_scope(
        &self,
        arguments: Option<&'b dyn ExecutionContextArguments>,
    ) -> ExecutionContext<'a, 'b, MapValueStorage<OwnedValue>> {
        ExecutionContext::<MapValueStorage<OwnedValue>>::new(
            self.diagnostic_level.clone(),
            self.pipeline,
            self.get_variables().global_variables,
            self.summaries,
            None,
            None,
            arguments,
        )
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

    pub fn add_diagnostic(&self, diagnostic: RecordSetEngineDiagnostic<'a>) {
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

    pub fn get_arguments(&self) -> Option<&dyn ExecutionContextArguments> {
        self.arguments
    }

    pub fn take_diagnostics(self) -> Vec<RecordSetEngineDiagnostic<'a>> {
        self.diagnostics.take()
    }

    pub fn consume_into_record(self) -> RecordSetEngineRecord<'a, TRecord> {
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

        let var = Ref::filter_map(vars, |v| {
            v.get_static(name)
                .expect("Static access not supported by underlying map")
        });

        if let Ok(v) = var {
            return Some(v);
        }

        Ref::filter_map(self.global_variables.borrow(), |v| {
            v.get_static(name)
                .expect("Static access not supported by underlying map")
        })
        .ok()
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

pub trait ExecutionContextArguments {
    fn get_argument_borrow_source(&self, id: usize) -> Option<BorrowSource>;

    fn get_argument(&self, id: usize) -> Result<ResolvedValue<'_>, ExpressionError>;

    fn get_argument_mut(
        &self,
        id: usize,
    ) -> Result<ResolvedMutableArgument<'_, '_>, ExpressionError>;
}

#[cfg(test)]
pub struct ExecutionContextArgumentContainer<'a, 'b, 'c, TRecord>
where
    TRecord: Record + 'static,
{
    pub parent_execution_context: &'c ExecutionContext<'a, 'b, TRecord>,
    pub arguments: &'a [InvokeFunctionArgument],
}

#[cfg(test)]
impl<'a, 'b, 'c, TRecord> ExecutionContextArguments
    for ExecutionContextArgumentContainer<'a, 'b, 'c, TRecord>
where
    TRecord: Record + 'static,
{
    fn get_argument_borrow_source(&self, id: usize) -> Option<BorrowSource> {
        let argument = self
            .arguments
            .get(id)
            .unwrap_or_else(|| panic!("Argument for id '{id}' was not found"));

        match argument {
            InvokeFunctionArgument::Scalar(s) => match s {
                ScalarExpression::Source(_) => Some(BorrowSource::Source),
                ScalarExpression::Variable(_) => Some(BorrowSource::Variable),
                ScalarExpression::Argument(a) => self
                    .parent_execution_context
                    .get_arguments()
                    .expect("Arguments were not found")
                    .get_argument_borrow_source(a.get_argument_id()),
                _ => None,
            },
            InvokeFunctionArgument::MutableValue(m) => match m {
                MutableValueExpression::Source(_) => Some(BorrowSource::Source),
                MutableValueExpression::Variable(_) => Some(BorrowSource::Variable),
                MutableValueExpression::Argument(a) => self
                    .parent_execution_context
                    .get_arguments()
                    .expect("Arguments were not found")
                    .get_argument_borrow_source(a.get_argument_id()),
            },
        }
    }

    fn get_argument(&self, id: usize) -> Result<ResolvedValue<'_>, ExpressionError> {
        let argument = self
            .arguments
            .get(id)
            .unwrap_or_else(|| panic!("Argument for id '{id}' was not found"));

        match argument {
            InvokeFunctionArgument::Scalar(s) => {
                execute_scalar_expression(self.parent_execution_context, s)
            }
            InvokeFunctionArgument::MutableValue(m) => {
                // Note: In this branch mutable values are retured as immutables which is totally fine
                match m {
                    MutableValueExpression::Argument(a) => {
                        execute_argument_scalar_expression(self.parent_execution_context, a)
                    }
                    MutableValueExpression::Source(s) => {
                        execute_source_scalar_expression(self.parent_execution_context, m, s)
                    }
                    MutableValueExpression::Variable(v) => {
                        execute_variable_scalar_expression(self.parent_execution_context, m, v)
                    }
                }
            }
        }
    }

    fn get_argument_mut(
        &self,
        id: usize,
    ) -> Result<ResolvedMutableArgument<'_, '_>, ExpressionError> {
        let argument = self
            .arguments
            .get(id)
            .unwrap_or_else(|| panic!("Argument for id '{id}' was not found"));

        match argument {
            InvokeFunctionArgument::Scalar(s) => Err(ExpressionError::NotSupported(
                s.get_query_location().clone(),
                format!("Argument for id '{id}' cannot be mutated"),
            )),
            InvokeFunctionArgument::MutableValue(m) => {
                let value = execute_mutable_value_expression(self.parent_execution_context, m)?;

                Ok(ResolvedMutableArgument { source: m, value })
            }
        }
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

    pub fn create_execution_context(&mut self) -> ExecutionContext<'_, 'a, TestRecord> {
        ExecutionContext::new(
            RecordSetEngineDiagnosticLevel::Verbose,
            &self.pipeline,
            &self.global_variables,
            &self.summaries,
            self.attached_records
                .as_ref()
                .map(|v| v as &dyn AttachedRecords),
            self.record.take(),
            None,
        )
    }
}
