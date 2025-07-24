use std::{cell::RefCell, collections::HashMap};

use data_engine_expressions::{MapValue, PipelineExpression};

use crate::{
    AttachedRecords, LogLevel, LogMessage, MapValueStorage, OwnedValue, Record,
    RecordSetEngineRecord,
};

pub(crate) struct ExecutionContext<'a, 'b, 'c, TRecord>
where
    'a: 'c,
    TRecord: MapValue + 'static,
{
    log_level: LogLevel,
    log_messages: RefCell<Vec<LogMessage<'c>>>,
    pipeline: &'a PipelineExpression,
    attached_records: Option<&'b dyn AttachedRecords>,
    record: RefCell<TRecord>,
    variables: RefCell<MapValueStorage<OwnedValue>>,
}

impl<'a, 'b, 'c, TRecord: Record + 'static> ExecutionContext<'a, 'b, 'c, TRecord> {
    pub fn new(
        log_level: LogLevel,
        pipeline: &'a PipelineExpression,
        attached_records: Option<&'b dyn AttachedRecords>,
        record: TRecord,
    ) -> ExecutionContext<'a, 'b, 'c, TRecord> {
        Self {
            log_level,
            log_messages: RefCell::new(Vec::new()),
            pipeline,
            attached_records,
            record: RefCell::new(record),
            variables: RefCell::new(MapValueStorage::new(HashMap::new())),
        }
    }

    pub fn is_enabled(&self, log_level: LogLevel) -> bool {
        log_level >= self.log_level
    }

    pub fn log(&self, message: LogMessage<'c>) {
        self.log_messages.borrow_mut().push(message);
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
            self.log_messages.take(),
        )
    }
}
