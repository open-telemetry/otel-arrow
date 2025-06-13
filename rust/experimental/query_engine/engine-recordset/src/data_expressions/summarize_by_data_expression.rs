use std::{cell::OnceCell, time::SystemTime};

use chrono::{DateTime, Timelike, Utc};

use crate::{
    data::data_record_resolver::*,
    error::Error,
    execution_context::*,
    expression::*,
    logical_expressions::logical_expression::LogicalExpression,
    primitives::any_value::AnyValue,
    summary::*,
    value_expressions::{
        resolve_value_expression::ResolveValueExpression, value_expression::ValueExpressionInternal,
    },
};

use super::data_expression::{DataExpressionInternal, DataExpressionResult};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum SummaryWindow {
    Timestamp(u32),
    ObservedTimestamp(u32),
}

impl SummaryWindow {
    pub fn new_timestamp_based(window_duration_in_seconds: u32) -> SummaryWindow {
        SummaryWindow::Timestamp(window_duration_in_seconds)
    }

    pub fn new_observed_timestamp_based(window_duration_in_seconds: u32) -> SummaryWindow {
        SummaryWindow::ObservedTimestamp(window_duration_in_seconds)
    }

    pub(crate) fn add_hash_bytes(&self, hasher: &mut Hasher) {
        match self {
            SummaryWindow::Timestamp(interval_seconds) => {
                hasher.add_bytes(&[0]);
                hasher.add_bytes(&(*interval_seconds).to_le_bytes());
            }
            SummaryWindow::ObservedTimestamp(interval_seconds) => {
                hasher.add_bytes(&[1]);
                hasher.add_bytes(&(*interval_seconds).to_le_bytes());
            }
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum SummaryReservoir {
    SimpleReservoir(u32),
}

impl SummaryReservoir {
    pub fn new_simple(size: u32) -> SummaryReservoir {
        SummaryReservoir::SimpleReservoir(size)
    }

    pub(crate) fn add_hash_bytes(&self, hasher: &mut Hasher) {
        match self {
            SummaryReservoir::SimpleReservoir(size) => {
                hasher.add_bytes(&[0]);
                hasher.add_bytes(&(*size).to_le_bytes());
            }
        }
    }
}

#[derive(Debug)]
pub struct SummarizeByDataExpression {
    id: usize,
    window: SummaryWindow,
    reservoir: SummaryReservoir,
    values: Vec<ResolveValueExpression>,
    predicate: Option<Box<dyn LogicalExpression>>,
    hash: OnceCell<ExpressionHash>,
}

impl Expression for SummarizeByDataExpression {
    fn get_id(&self) -> usize {
        self.id
    }

    fn get_hash(&self) -> &ExpressionHash {
        self.hash.get_or_init(|| {
            ExpressionHash::new(|h| {
                h.add_bytes(b"summarize_by");
                if self.predicate.is_some() {
                    h.add_bytes(b"predicate:");
                    h.add_bytes(self.predicate.as_ref().unwrap().get_hash().get_bytes());
                }
                h.add_bytes(b"window:");
                self.window.add_hash_bytes(h);
                h.add_bytes(b"reservoir:");
                self.reservoir.add_hash_bytes(h);
                if !self.values.is_empty() {
                    h.add_bytes(b"values:");
                    for value_expr in self.values.iter() {
                        h.add_bytes(value_expr.get_hash().get_bytes());
                    }
                }
            })
        })
    }

    fn write_debug(
        &self,
        execution_context: &dyn ExecutionContext,
        heading: &'static str,
        level: i32,
        output: &mut String,
    ) {
        let padding = "\t".repeat(level as usize);

        output.push_str(&padding);
        output.push_str(heading);
        output.push_str("summarize_by (\n");

        if self.predicate.is_some() {
            self.predicate.as_ref().unwrap().write_debug(
                execution_context,
                "predicate: ",
                level + 1,
                output,
            );

            output.push_str(&padding);
            output.push_str(" ,\n");
        }

        output.push_str(&padding);
        output.push_str(format!("\twindow: {:?}\n", &self.window).as_str());

        output.push_str(&padding);
        output.push_str(" ,\n");

        output.push_str(&padding);
        output.push_str(format!("\treservoir: {:?}\n", &self.reservoir).as_str());

        if !self.values.is_empty() {
            output.push_str(&padding);
            output.push_str(" ,\n");

            output.push_str(&padding);
            output.push_str("\tvalues: [\n");

            let mut is_first = true;
            for value_expr in self.values.iter() {
                if !is_first {
                    output.push_str(&padding);
                    output.push_str("\t ,\n");
                } else {
                    is_first = false;
                }
                value_expr.write_debug(execution_context, "", level + 2, output);
            }

            output.push_str(&padding);
            output.push_str("\t]\n");
        }

        output.push_str(&padding);
        output.push_str(")\n");

        execution_context.write_debug_comments_for_expression(self, output, &padding);
    }
}

impl DataExpressionInternal for SummarizeByDataExpression {
    fn evaluate<'a, 'b>(
        &'a self,
        execution_context: &dyn ExecutionContext<'b>,
    ) -> Result<DataExpressionResult, Error>
    where
        'a: 'b,
    {
        let summary_index = execution_context.get_summary_index();
        if summary_index.is_some() {
            execution_context.add_message_for_expression(
                self,
                ExpressionMessage::info("SummarizeByDataExpression evaluation skipped because the current record has already been summarized".to_string()));

            return Ok(DataExpressionResult::None);
        }

        if self.predicate.is_some()
            && !self
                .predicate
                .as_ref()
                .unwrap()
                .evaluate(execution_context)?
        {
            execution_context.add_message_for_expression(
                self,
                ExpressionMessage::info("SummarizeByDataExpression evaluation skipped".to_string()),
            );

            return Ok(DataExpressionResult::None);
        }

        let i = execution_context.get_data_record_index();
        if i.is_none() {
            execution_context.add_message_for_expression(
                self,
                ExpressionMessage::warn("SummarizeByDataExpression evaluation skipped because the current record doesn't have an index".to_string()));
            return Ok(DataExpressionResult::None);
        }

        let data_record_index = i.unwrap();

        let timestamp: Option<SystemTime>;
        let interval_seconds: i64;
        match self.window {
            SummaryWindow::Timestamp(t) => {
                timestamp = execution_context.get_timestamp();
                interval_seconds = t as i64;
            }
            SummaryWindow::ObservedTimestamp(t) => {
                timestamp = execution_context.get_observed_timestamp();
                interval_seconds = t as i64;
            }
        }

        if timestamp.is_none() {
            execution_context.add_message_for_expression(
                self,
                ExpressionMessage::warn("SummarizeByDataExpression evaluation skipped because the current record doesn't have the requested timestamp".to_string()));
            return Ok(DataExpressionResult::None);
        }

        let timestamp_utc: DateTime<Utc> = timestamp.unwrap().into();
        let seconds_from_midnight = timestamp_utc.num_seconds_from_midnight() as i64;
        let window_start = timestamp_utc.timestamp() - seconds_from_midnight
            + (seconds_from_midnight % interval_seconds);

        let mut grouping: Vec<SummaryGroupKeyValue> = Vec::new();

        for resolved_value in execution_context.get_resolved_values().borrow().iter() {
            grouping.push(SummaryGroupKeyValue::new(
                SummaryGroupKey::new(resolved_value.get_name(), resolved_value.get_path()),
                resolved_value.get_summary_group_value().clone(),
            ));
        }

        for value_expr in &self.values {
            value_expr.read_any_value(execution_context, &mut DataRecordAnyValueReadClosureCallback::new(|v| {
                match v {
                    DataRecordReadAnyValueResult::NotFound => {
                        grouping.push(
                            SummaryGroupKeyValue::new(
                                SummaryGroupKey::new(None, value_expr.get_path().get_raw_value()),
                                SummaryGroupValue::NullValue));

                        execution_context.add_message_for_expression(
                            self,
                            ExpressionMessage::warn("SummarizeByDataExpression value could not be resolved".to_string()));
                    },
                    DataRecordReadAnyValueResult::Found(any_value) => {
                        if let AnyValue::NullValue = any_value {
                            grouping.push(
                                SummaryGroupKeyValue::new(
                                    SummaryGroupKey::new(None, value_expr.get_path().get_raw_value()),
                                    SummaryGroupValue::NullValue));

                            execution_context.add_message_for_expression(
                                self,
                                ExpressionMessage::info(
                                    "SummarizeByDataExpression Null value resolved".to_string()));
                        }
                        else if let AnyValue::StringValue(string_value) = any_value {
                            grouping.push(
                                SummaryGroupKeyValue::new(
                                    SummaryGroupKey::new(None, value_expr.get_path().get_raw_value()),
                                    SummaryGroupValue::StringValue(string_value.get_value().into())));

                            execution_context.add_message_for_expression(
                                self,
                                ExpressionMessage::info(
                                    format!("SummarizeByDataExpression String value resolved: {:?}", string_value.get_value())));
                        }
                        else if let AnyValue::DoubleValue(double_value) = any_value {
                            grouping.push(
                                SummaryGroupKeyValue::new(
                                    SummaryGroupKey::new(None, value_expr.get_path().get_raw_value()),
                                    SummaryGroupValue::DoubleValue(double_value.get_value().to_le_bytes())));

                            execution_context.add_message_for_expression(
                                self,
                                ExpressionMessage::info(
                                    format!("SummarizeByDataExpression Double value resolved: {:?}", double_value.get_value())));
                        }
                        else if let AnyValue::LongValue(long_value) = any_value {
                            grouping.push(
                                SummaryGroupKeyValue::new(
                                    SummaryGroupKey::new(None, value_expr.get_path().get_raw_value()),
                                    SummaryGroupValue::LongValue(long_value.get_value().to_le_bytes())));

                            execution_context.add_message_for_expression(
                                self,
                                ExpressionMessage::info(
                                    format!("SummarizeByDataExpression Long value resolved: {:?}", long_value.get_value())));
                        }
                        else {
                            any_value.as_string_value(|s| {
                                match s {
                                    Some(string_value) => {
                                        grouping.push(
                                            SummaryGroupKeyValue::new(
                                                SummaryGroupKey::new(None, value_expr.get_path().get_raw_value()),
                                                SummaryGroupValue::StringValue(string_value.into())));

                                        execution_context.add_message_for_expression(
                                            self,
                                            ExpressionMessage::info(
                                                format!("SummarizeByDataExpression resolved value converted to String: {:?}", any_value)));
                                    },
                                    None => {
                                        grouping.push(
                                            SummaryGroupKeyValue::new(
                                                SummaryGroupKey::new(None, value_expr.get_path().get_raw_value()),
                                                SummaryGroupValue::NullValue));

                                        execution_context.add_message_for_expression(
                                            self,
                                            ExpressionMessage::warn(
                                                format!("SummarizeByDataExpression resolved value could not be converted into a String: {:?}", any_value)));
                                    },
                                }
                            });
                        }
                    },
                }
            }));
        }

        let lookup = SummaryLookup::new(
            self.window.clone(),
            window_start,
            window_start + interval_seconds,
            self.reservoir.clone(),
            grouping,
        );

        let result = execution_context
            .get_summaries()
            .create_or_update_summary(data_record_index, lookup);

        match result {
            SummaryResult::Include(summary_info) => {
                self.complete_summary_update(execution_context, &summary_info);

                Ok(DataExpressionResult::None)
            }
            SummaryResult::Drop(summary_info) => {
                self.complete_summary_update(execution_context, &summary_info);

                Ok(DataExpressionResult::Drop(self.get_id()))
            }
        }
    }
}

impl SummarizeByDataExpression {
    pub fn new(window: SummaryWindow, reservoir: SummaryReservoir) -> SummarizeByDataExpression {
        Self {
            id: get_next_id(),
            window,
            reservoir,
            values: Vec::new(),
            predicate: None,
            hash: OnceCell::new(),
        }
    }

    pub fn new_with_predicate(
        window: SummaryWindow,
        reservoir: SummaryReservoir,
        predicate: impl LogicalExpression + 'static,
    ) -> SummarizeByDataExpression {
        Self {
            id: get_next_id(),
            window,
            reservoir,
            values: Vec::new(),
            predicate: Some(Box::new(predicate)),
            hash: OnceCell::new(),
        }
    }

    pub fn add_value_expression(&mut self, value: ResolveValueExpression) {
        self.values.push(value);
    }

    fn complete_summary_update(
        &self,
        execution_context: &dyn ExecutionContext,
        summary_info: &SummaryInfo,
    ) {
        execution_context.set_summary_index(summary_info.get_summary_index());

        execution_context
            .get_summaries()
            .get_summary(summary_info.get_summary_index(), |s| {
                let summary = s.expect("Summary not found");

                if summary_info.get_existed() {
                    execution_context.add_message_for_expression(
                        self,
                        ExpressionMessage::info(format!(
                            "SummarizeByDataExpression summary updated: {:?}",
                            summary
                        )),
                    );
                } else {
                    execution_context.add_message_for_expression(
                        self,
                        ExpressionMessage::info(format!(
                            "SummarizeByDataExpression summary established: {:?}",
                            summary
                        )),
                    );
                }
            });
    }
}
