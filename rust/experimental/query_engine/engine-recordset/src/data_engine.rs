use std::{cell::RefCell, collections::HashMap, marker::PhantomData};

use crate::{
    Summary,
    data::{
        data_record::*, data_record_batch::*,
        data_record_resolver_cache::DataRecordAnyValueResolverCache,
    },
    data_expressions::data_expression::*,
    error::Error,
    execution_context::*,
    expression::*,
    pipeline_expression::PipelineExpression,
    summary::Summaries,
};

pub struct DataEngine {
    resolver_cache: DataRecordAnyValueResolverCache,
}

impl Default for DataEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl DataEngine {
    pub fn new() -> DataEngine {
        Self {
            resolver_cache: DataRecordAnyValueResolverCache::new(),
        }
    }

    pub fn register<T: DataRecord>(&mut self) -> Result<(), Error> {
        self.resolver_cache.register::<T>()
    }

    pub fn begin_batch<'a, TRecord: DataRecord, TItem: DataEngineItem<TRecord>>(
        &self,
        pipeline: &'a PipelineExpression,
    ) -> DataEngineBatch<'a, '_, TRecord, TItem> {
        let state = _DataEngineState::<TItem> {
            pipeline,
            summaries: Summaries::new(),
            resolver_cache: &self.resolver_cache,
            attached_data_record_names: TItem::get_attached_data_record_names(),
            items: Vec::new(),
            included_record_count: 0,
            dropped_record_count: 0,
        };

        DataEngineBatch {
            state,
            marker: PhantomData,
        }
    }

    pub fn process_complete_batch<
        TRecord: DataRecord,
        TItem: DataEngineItem<TRecord>,
        TBatch: DataRecordBatch<TItem>,
    >(
        &self,
        pipeline: &PipelineExpression,
        batch: &mut TBatch,
    ) -> Result<DataEngineExecutionResults<TItem>, Error> {
        let mut b = self.begin_batch(pipeline);

        batch.drain(&mut b.state, Self::process_item)?;

        Ok(b.complete())
    }

    fn process_item<TRecord: DataRecord, TItem: DataEngineItem<TRecord>>(
        state: &mut _DataEngineState<'_, '_, TItem>,
        mut item: TItem,
    ) -> Result<(), Error> {
        let data_record_index = state.items.len();

        let messages = RefCell::new(HashMap::new());

        let execution_context = DataRecordExecutionContext::new(
            &item,
            Some(data_record_index),
            item.get_data_record(),
            None,
            &messages,
            &state.summaries,
            state.resolver_cache,
        );

        execution_context.add_message_for_expression_id(
            0,
            ExpressionMessage::info(format!(
                "Evaluating expression {}",
                state.pipeline.get_hash().get_hex()
            )),
        );

        execution_context.add_message_for_expression_id(
            0,
            ExpressionMessage::info(format!("DataRecord: {:?}", item.get_data_record().borrow())),
        );

        for name in state.attached_data_record_names {
            execution_context.add_message_for_expression_id(
                0,
                ExpressionMessage::info(format!(
                    "AttachedDataRecord '{name}': {:?}",
                    item.get_attached_data_record(name)
                )),
            );
        }

        let external_summary_result = execution_context.get_external_summary_index();
        if external_summary_result.is_err() {
            execution_context.add_message_for_expression_id(
                0,
                ExpressionMessage::warn(format!(
                    "Ignored external summary on data record due to error: {}",
                    external_summary_result.unwrap_err()
                )),
            );
        } else {
            let external_summary_index = external_summary_result.unwrap();
            if external_summary_index.is_some() {
                let summary_index = external_summary_index.unwrap();

                let external_count = execution_context
                    .get_summaries()
                    .include_in_summary(summary_index);

                execution_context.get_summaries().get_summary(summary_index, |v| {
                    execution_context.add_message_for_expression_id(
                        0,
                        ExpressionMessage::info(
                            format!("Updated externally included record count ({external_count}) for summary_id '{}'", v.expect("Summary could not be found").get_id())));
                });

                execution_context.set_summary_index(summary_index);
            }
        }

        let result = state.pipeline.evaluate(&execution_context)?;

        if let DataExpressionResult::Drop(expression_id) = result {
            execution_context.add_message_for_expression_id(
                expression_id,
                ExpressionMessage::info("Dropped record".to_string()),
            );

            let mut output = String::new();

            execution_context.write_debug_comments_for_expression_id(0, &mut output, "");

            state
                .pipeline
                .write_debug(&execution_context, "pipeline: ", 0, &mut output);

            println!("{output}");

            state.dropped_record_count += 1;
            if !item.on_dropped() {
                state.items.push(DataRecordState {
                    item: Some(item),
                    result: DataRecordProcessResult::Drop,
                });
            }

            return Ok(());
        }

        if let DataExpressionResult::Include(expression_id) = result {
            execution_context.add_message_for_expression_id(
                expression_id,
                ExpressionMessage::info(format!(
                    "Included record: {:?}",
                    item.get_data_record().borrow()
                )),
            );

            let mut output = String::new();

            execution_context.write_debug_comments_for_expression_id(0, &mut output, "");

            state
                .pipeline
                .write_debug(&execution_context, "pipeline: ", 0, &mut output);

            println!("{output}");

            state.included_record_count += 1;
            state.items.push(DataRecordState {
                item: Some(item),
                result: DataRecordProcessResult::Include,
            });

            return Ok(());
        }

        Err(Error::ExpressionError(
            state.pipeline.get_id(),
            Error::InvalidOperation("Unexpected result from pipeline").into(),
        ))
    }
}

pub struct DataEngineBatch<'a, 'b, TRecord: DataRecord, TItem: DataEngineItem<TRecord>> {
    state: _DataEngineState<'a, 'b, TItem>,
    marker: PhantomData<TRecord>,
}

impl<TRecord: DataRecord, TItem: DataEngineItem<TRecord>> DataEngineBatch<'_, '_, TRecord, TItem> {
    pub fn register_summary(&mut self, summary: Summary) {
        self.state.register_summary(summary);
    }

    pub fn push(&mut self, item: TItem) -> Result<(), Error> {
        let state = &mut self.state;

        DataEngine::process_item(state, item)
    }

    pub fn complete(self) -> DataEngineExecutionResults<TItem> {
        let mut state = self.state;

        let summaries = state.summaries.get_summaries();
        let mut summary_results: Vec<Summary> = Vec::with_capacity(summaries.len());

        for mut summary_state in summaries {
            for dropped_data_record in summary_state.get_data_records_dropped_after_being_included()
            {
                let data_record_state = state
                    .items
                    .get_mut(*dropped_data_record)
                    .expect("DataRecord to drop could not be found");
                let data_record = data_record_state
                    .item
                    .as_mut()
                    .expect("DataRecord not found on state");
                let consumed = data_record.on_dropped();
                data_record_state.result = DataRecordProcessResult::Drop;
                state.dropped_record_count += 1;
                state.included_record_count -= 1;
                println!("Dropped record replaced in summary reservoir: {data_record:?}");
                if consumed {
                    data_record_state.item = None;
                }
            }

            for included_data_record in summary_state.get_included_data_records() {
                let data_record_state = state
                    .items
                    .get_mut(*included_data_record)
                    .expect("DataRecord to drop could not be found");
                let data_record = data_record_state
                    .item
                    .as_ref()
                    .expect("DataRecord was not found on state")
                    .get_data_record();
                data_record
                    .borrow_mut()
                    .set_summary_id(summary_state.get_summary().get_id());
                println!("Applied summary to record: {:?}", data_record.borrow());
            }

            if summary_state.get_expected_externally_included_record_count()
                != summary_state.get_externally_included_record_count()
            {
                println!(
                    "External summary record count mismatch expected {} found {}",
                    summary_state.get_expected_externally_included_record_count(),
                    summary_state.get_externally_included_record_count()
                );
            }

            let externally_included_record_count =
                summary_state.get_externally_included_record_count();
            let included_record_count = summary_state.get_included_data_record_count();
            summary_state
                .get_summary_mut()
                .add_externally_included_record_count(externally_included_record_count);

            println!(
                "Processed summary externally_included_record_count: {externally_included_record_count}, locally_included_record_count: {included_record_count}: {:?}",
                summary_state.get_summary()
            );

            summary_results.push(summary_state.into());
        }

        DataEngineExecutionResults::new(
            summary_results,
            state.items,
            state.included_record_count,
            state.dropped_record_count,
        )
    }
}

pub trait DataEngineItem<T: DataRecord>: AttachedDataRecords {
    fn get_attached_data_record_names() -> &'static [&'static str];

    fn get_data_record(&self) -> &RefCell<T>;

    fn on_dropped(&mut self) -> bool;
}

pub struct DataEngineExecutionResults<T> {
    summaries: Vec<Summary>,
    items: Vec<DataRecordState<T>>,
    included_record_count: u32,
    dropped_record_count: u32,
}

impl<T> DataEngineExecutionResults<T> {
    fn new(
        summaries: Vec<Summary>,
        items: Vec<DataRecordState<T>>,
        included_record_count: u32,
        dropped_record_count: u32,
    ) -> DataEngineExecutionResults<T> {
        Self {
            summaries,
            items,
            included_record_count,
            dropped_record_count,
        }
    }

    pub fn get_summaries(&self) -> &Vec<Summary> {
        &self.summaries
    }

    pub fn get_included_record_count(&self) -> u32 {
        self.included_record_count
    }

    pub fn get_dropped_record_count(&self) -> u32 {
        self.dropped_record_count
    }

    pub fn drain_included_records(&mut self) -> Drain<'_, T> {
        Drain {
            items: &mut self.items,
            index: 0,
            included: true,
        }
    }

    pub fn drain_dropped_records(&mut self) -> Drain<'_, T> {
        Drain {
            items: &mut self.items,
            index: 0,
            included: false,
        }
    }
}

pub struct Drain<'a, T> {
    items: &'a mut Vec<DataRecordState<T>>,
    index: usize,
    included: bool,
}

impl<T> Iterator for Drain<'_, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.index >= self.items.len() {
                return None;
            }

            let state = self
                .items
                .get_mut(self.index)
                .expect("DataRecord was not found");

            self.index += 1;

            if state.item.is_none() {
                continue;
            }

            match state.result {
                DataRecordProcessResult::Include => {
                    if self.included {
                        return Some(state.item.take().unwrap());
                    }
                }
                DataRecordProcessResult::Drop => {
                    if !self.included {
                        return Some(state.item.take().unwrap());
                    }
                }
            }
        }
    }
}

pub trait DataEngineState {
    fn register_summary(&mut self, summary: Summary);
}

struct _DataEngineState<'a, 'b, T> {
    pipeline: &'a PipelineExpression,
    summaries: Summaries,
    resolver_cache: &'b DataRecordAnyValueResolverCache,
    attached_data_record_names: &'static [&'static str],
    items: Vec<DataRecordState<T>>,
    included_record_count: u32,
    dropped_record_count: u32,
}

impl<T> DataEngineState for _DataEngineState<'_, '_, T> {
    fn register_summary(&mut self, summary: Summary) {
        let summary_id: String = summary.get_id().into();

        let summary_info = self.summaries.register_summary(summary);

        println!("Summary '{summary_id}' registered: {summary_info:?}");
    }
}

struct DataRecordState<T> {
    item: Option<T>,
    result: DataRecordProcessResult,
}

enum DataRecordProcessResult {
    Include,
    Drop,
}
