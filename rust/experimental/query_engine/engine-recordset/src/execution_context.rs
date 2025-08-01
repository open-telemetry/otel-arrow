use std::{cell::RefCell, collections::HashMap};

use data_engine_expressions::{Expression, MapValue, PipelineExpression};

use crate::{
    AttachedRecords, MapValueStorage, OwnedValue, Record, RecordSetEngineDiagnostic,
    RecordSetEngineDiagnosticLevel, RecordSetEngineRecord,
};

pub(crate) struct ExecutionContext<'a, 'b, 'c, TRecord>
where
    'a: 'c,
    TRecord: MapValue + 'static,
{
    diagnostic_level: RecordSetEngineDiagnosticLevel,
    diagnostics: RefCell<Vec<RecordSetEngineDiagnostic<'c>>>,
    pipeline: &'a PipelineExpression,
    attached_records: Option<&'b dyn AttachedRecords>,
    record: RefCell<TRecord>,
    variables: RefCell<MapValueStorage<OwnedValue>>,
}

impl<'a, 'b, 'c, TRecord: Record + 'static> ExecutionContext<'a, 'b, 'c, TRecord> {
    pub fn new(
        diagnostic_level: RecordSetEngineDiagnosticLevel,
        pipeline: &'a PipelineExpression,
        attached_records: Option<&'b dyn AttachedRecords>,
        record: TRecord,
    ) -> ExecutionContext<'a, 'b, 'c, TRecord> {
        Self {
            diagnostic_level,
            diagnostics: RefCell::new(Vec::new()),
            pipeline,
            attached_records,
            record: RefCell::new(record),
            variables: RefCell::new(MapValueStorage::new(HashMap::new())),
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

    pub fn get_record(&self) -> &RefCell<TRecord> {
        &self.record
    }

    pub fn get_variables(&self) -> &RefCell<MapValueStorage<OwnedValue>> {
        &self.variables
    }

    pub fn consume_into_record(self) -> RecordSetEngineRecord<'a, 'c, TRecord> {
        RecordSetEngineRecord::new(
            self.pipeline,
            self.record.into_inner(),
            self.diagnostics.take(),
        )
    }
}
