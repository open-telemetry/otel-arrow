// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![allow(unused)]

use std::{cell::RefCell, collections::HashMap, fmt::Display};

use arrow::{array::*, compute::kernels::filter, datatypes::*};
use otel_arrow_contrib_data_engine_expressions::*;
use roaring::RoaringBitmap;

use crate::{execution_context::*, resolved_value::*, scalars::execute_scalar_expression, *};

#[derive(Default)]
pub(crate) struct ExecutionContextState<'pipeline> {
    pipeline: Option<&'pipeline PipelineExpression>,
    records: Option<TestRecords<'pipeline>>,
}

impl<'pipeline> ExecutionContextState<'pipeline> {
    pub fn new() -> ExecutionContextState<'pipeline> {
        Default::default()
    }

    pub fn with_pipeline(mut self, pipeline: &'pipeline PipelineExpression) -> Self {
        self.pipeline = Some(pipeline);
        self
    }

    pub fn with_records(mut self, records: TestRecords<'pipeline>) -> Self {
        self.records = Some(records);
        self
    }
}

pub(crate) fn run_scalar_expression_test<FValidate>(
    records: TestRecords,
    expression: ScalarExpression,
    validate: FValidate,
) where
    for<'a, 'b> FValidate: FnOnce(ResolvedScalarValue<'a, 'b>),
{
    run_scalar_expression_test_with_state(
        ExecutionContextState::new().with_records(records),
        expression,
        validate,
    )
}

pub(crate) fn run_scalar_expression_test_with_state<FValidate>(
    state: ExecutionContextState<'_>,
    expression: ScalarExpression,
    validate: FValidate,
) where
    for<'a, 'b> FValidate: FnOnce(ResolvedScalarValue<'a, 'b>),
{
    let d = RefCell::new(vec![]);

    let mut local_pipeline = None;

    let p = state.pipeline.unwrap_or_else(|| {
        local_pipeline = Some(Default::default());
        local_pipeline.as_ref().expect("has pipeline")
    });

    let ec = ExecutionContext::new(ColumnarEngineDiagnosticLevel::Verbose, &d, p, state.records);

    let result = execute_scalar_expression(&ec, &expression);

    println!("{ec}");

    validate(result)
}

pub(crate) fn build_dictionary(
    keys: Vec<Option<u16>>,
    values: Vec<ValueOrRef<'static>>,
) -> Dictionary<'static> {
    let mut key_builder = PrimitiveBuilder::<UInt16Type>::new();

    for key in keys {
        match key {
            None => key_builder.append_null(),
            Some(k) => key_builder.append_value(k),
        }
    }

    let keys = key_builder.finish();

    Dictionary::new(keys.into(), DictionaryValueArray::Vec(values.into())).expect("valid")
}

pub(crate) struct TestRecordsFactory {}

impl ColumnarRecordsFactory<2> for TestRecordsFactory {
    type Records<'pipeline, 'record> = TestRecords<'pipeline>;
    type State<'pipeline> = TestRecords<'pipeline>;

    fn create<'pipeline, 'record>(
        &self,
        _state: Option<Self::State<'pipeline>>,
        _batches: &'record [Option<RecordBatch>; 2],
    ) -> Self::Records<'pipeline, 'record> {
        todo!()
    }

    fn filter<'pipeline>(
        &self,
        _state: &mut Self::State<'pipeline>,
        batches: &mut [Option<RecordBatch>; 2],
        filter: &BooleanArray,
    ) {
        if let Some(records) = &batches[0] {
            batches[0] = Some(filter::filter_record_batch(records, filter).unwrap());

            if let Some(attached_records) = &batches[1] {
                batches[1] = Some(filter::filter_record_batch(attached_records, filter).unwrap());
            }

            return;
        }

        batches[1] = None;
    }

    fn apply<'pipeline, T: ColumnarEngineDiagnosticReceiver<'pipeline>>(
        &self,
        _diagnostic_receiver: &T,
        _expression: &'pipeline dyn Expression,
        _state: &mut Self::State<'pipeline>,
        _batches: &mut [Option<RecordBatch>; 2],
    ) {
        todo!()
    }
}

#[derive(Debug)]
pub(crate) struct TestRecords<'pipeline> {
    ids: Option<PrimitiveArray<Int64Type>>,
    values: Option<HashMap<Box<str>, Dictionary<'pipeline>>>,
    attached_records: Option<HashMap<Box<str>, TestRecords<'pipeline>>>,
}

impl<'pipeline> TestRecords<'pipeline> {
    pub fn new() -> TestRecords<'pipeline> {
        Self {
            ids: None,
            values: None,
            attached_records: None,
        }
    }

    pub fn with_ids(mut self, ids: PrimitiveArray<Int64Type>) -> Self {
        self.ids = Some(ids);
        self
    }

    pub fn with_values(mut self, values: HashMap<Box<str>, Dictionary<'pipeline>>) -> Self {
        self.values = Some(values);
        self
    }

    pub fn with_attached_records(
        mut self,
        attached_records: HashMap<Box<str>, TestRecords<'pipeline>>,
    ) -> Self {
        self.attached_records = Some(attached_records);
        self
    }
}

impl<'pipeline> ColumnarRecords<'pipeline> for TestRecords<'pipeline> {
    fn get_diagnostic_level(&self) -> Option<ColumnarEngineDiagnosticLevel> {
        None
    }

    fn get_key_data_type(&self) -> DictionaryKeyType {
        DictionaryKeyType::UInt16
    }

    fn len(&self) -> usize {
        if let Some(ids) = self.ids.as_ref() {
            ids.len()
        } else if let Some(values) = self.values.as_ref()
            && !values.is_empty()
        {
            values.iter().next().expect("has value").1.len()
        } else {
            0
        }
    }

    fn get_attached_records(&self, name: &str) -> Option<&dyn RecordTable<'pipeline>> {
        self.attached_records
            .as_ref()
            .and_then(|a| a.get(name).map(|v| v as &dyn RecordTable))
    }

    fn set_values<T: ColumnarEngineDiagnosticReceiver<'pipeline>>(
        &mut self,
        _diagnostic_receiver: &T,
        _expression: &'pipeline dyn Expression,
        _root: &ColumnarEngineSelectionPath<'pipeline>,
        _path: &[ColumnarEngineSelectionPath<'pipeline>],
        _key_filter: Option<&RoaringBitmap>,
        _values: Dictionary<'pipeline>,
    ) -> ColumnarRecordsWriteResult {
        todo!()
    }
}

impl<'pipeline> RecordTable<'pipeline> for TestRecords<'pipeline> {
    fn get_values(&self, key: &str) -> Option<RecordTableValue<'pipeline, '_>> {
        self.values
            .as_ref()
            .and_then(|v| v.get(key).map(|v| RecordTableValue::Dictionary(v.clone())))
    }
}

impl Display for TestRecords<'_> {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
