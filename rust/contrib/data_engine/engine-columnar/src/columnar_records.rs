// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::fmt::{Debug, Display};

use arrow::array::*;
use otel_arrow_contrib_data_engine_expressions::Expression;
use roaring::RoaringBitmap;

use crate::{engine_diagnostic::ColumnarEngineDiagnosticLevel, *};

pub trait ColumnarRecordsFactory<const BATCH_SIZE: usize> {
    type Records<'pipeline, 'record>: ColumnarRecords<'pipeline> + Into<Self::State<'pipeline>>;
    type State<'pipeline>;

    fn create<'pipeline, 'record>(
        &self,
        state: Option<Self::State<'pipeline>>,
        batches: &'record [Option<RecordBatch>; BATCH_SIZE],
    ) -> Self::Records<'pipeline, 'record>;

    fn filter<'pipeline>(
        &self,
        state: &mut Self::State<'pipeline>,
        batches: &mut [Option<RecordBatch>; BATCH_SIZE],
        filter: &BooleanArray,
    );

    fn apply<'pipeline, T: ColumnarEngineDiagnosticReceiver<'pipeline>>(
        &self,
        diagnostic_receiver: &T,
        expression: &'pipeline dyn Expression,
        state: &mut Self::State<'pipeline>,
        batches: &mut [Option<RecordBatch>; BATCH_SIZE],
    );
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ColumnarRecordsWriteResult {
    Success,
    PartialSuccess,
    NotFound,
}

#[derive(Debug)]
pub enum ColumnarEngineSelectionPath<'a> {
    Key {
        expression: &'a dyn Expression,
        value: StringValueOrRef<'a>,
    },
    Index {
        expression: &'a dyn Expression,
        value: i64,
    },
    Dictionary {
        expression: &'a dyn Expression,
        value: Dictionary<'a>,
    },
}

pub trait ColumnarRecords<'pipeline>: RecordTable<'pipeline> {
    fn get_diagnostic_level(&self) -> Option<ColumnarEngineDiagnosticLevel>;

    fn get_key_data_type(&self) -> DictionaryKeyType;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn get_attached_records(&self, name: &str) -> Option<&dyn RecordTable<'pipeline>>;

    fn set_values<T: ColumnarEngineDiagnosticReceiver<'pipeline>>(
        &mut self,
        diagnostic_receiver: &T,
        expression: &'pipeline dyn Expression,
        root: &ColumnarEngineSelectionPath<'pipeline>,
        path: &[ColumnarEngineSelectionPath<'pipeline>],
        key_filter: Option<&RoaringBitmap>,
        values: Dictionary<'pipeline>,
    ) -> ColumnarRecordsWriteResult;
}

pub trait RecordTable<'pipeline>: Display + Debug {
    fn get_values(&self, key: &str) -> Option<RecordTableValue<'pipeline, '_>>;
}

#[expect(
    clippy::large_enum_variant,
    reason = "values are transient; boxing Dictionary would add an allocation to each lookup"
)]
#[derive(Debug, Clone)]
pub enum RecordTableValue<'pipeline, 'a> {
    Dictionary(Dictionary<'pipeline>),
    Table(&'a dyn RecordTable<'pipeline>),
}
