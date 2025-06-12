use std::{
    cell::{RefCell, RefMut},
    collections::HashMap,
    time::SystemTime,
};

use crate::{
    ValuePath,
    data::{
        AttachedDataRecords, data_record::*, data_record_resolver::*,
        data_record_resolver_cache::DataRecordAnyValueResolverCache,
    },
    data_expressions::data_expression::*,
    error::Error,
    expression::*,
    primitives::any_value::AnyValue,
    summary::*,
    value_expressions::resolve_value_expression::ResolveValueExpression,
};

pub(crate) trait ExecutionContext<'a> {
    fn evaluate<'b>(
        &self,
        expression: &'b dyn DataExpression,
    ) -> Result<DataExpressionResult, Error>
    where
        'b: 'a;

    fn clear(&self);

    fn read_any_value<'b>(
        &self,
        expression_id: usize,
        resolve_value_expr: &'b ResolveValueExpression,
        action: &mut dyn DataRecordAnyValueReadCallback,
    ) where
        'b: 'a;

    fn read_any_value_from_attached<'b>(
        &self,
        expression_id: usize,
        name: &'b str,
        path: &'b ValuePath,
        action: &mut dyn DataRecordAnyValueReadCallback,
    ) where
        'b: 'a;

    fn set_any_value<'b>(
        &self,
        expression_id: usize,
        resolve_value_expr: &'b ResolveValueExpression,
        value: AnyValue,
    ) -> DataRecordSetAnyValueResult
    where
        'b: 'a;

    fn remove_any_value<'b>(
        &self,
        expression_id: usize,
        resolve_value_expr: &'b ResolveValueExpression,
    ) -> DataRecordRemoveAnyValueResult
    where
        'b: 'a;

    fn get_attached_data_records(&self) -> &dyn AttachedDataRecords;

    fn get_data_record_index(&self) -> Option<usize>;

    fn get_timestamp(&self) -> Option<SystemTime>;

    fn get_observed_timestamp(&self) -> Option<SystemTime>;

    fn get_summaries(&self) -> &Summaries;

    fn get_external_summary_index(&self) -> Result<Option<usize>, Error>;

    fn get_summary_index(&self) -> Option<usize>;

    fn set_summary_index(&self, index: usize);

    fn get_variables(&self) -> &RefCell<HashMap<String, AnyValue>>;

    fn get_resolved_values(&self) -> &RefCell<Vec<DataRecordResolvedValue<'a>>>;

    fn add_message_for_expression(&self, expression: &dyn Expression, message: ExpressionMessage);

    fn add_message_for_expression_id(&self, expression_id: usize, message: ExpressionMessage);

    fn write_debug_comments_for_expression(
        &self,
        expression: &dyn Expression,
        output: &mut String,
        padding: &str,
    );

    fn write_debug_comments_for_expression_id(
        &self,
        expression_id: usize,
        output: &mut String,
        padding: &str,
    );
}

pub(crate) struct DataRecordExecutionContext<'a, T: DataRecord> {
    attached_data_records: &'a dyn AttachedDataRecords,
    data_record_index: Option<usize>,
    data_record: &'a RefCell<T>,
    message_scope: Option<&'a str>,
    messages: &'a RefCell<HashMap<usize, Vec<ExpressionMessage>>>,
    variables: RefCell<HashMap<String, AnyValue>>,
    resolved_values: RefCell<Vec<DataRecordResolvedValue<'a>>>,
    summaries: &'a Summaries,
    summary_index: RefCell<Option<usize>>,
    resolver_cache: &'a DataRecordAnyValueResolverCache,
}

pub(crate) struct DataRecordResolvedValue<'a> {
    name: Option<&'a str>,
    path: &'a str,
    summary_group_value: SummaryGroupValue,
}

impl<'a> DataRecordResolvedValue<'a> {
    pub fn new(
        name: Option<&'a str>,
        path: &'a str,
        value: &AnyValue,
    ) -> DataRecordResolvedValue<'a> {
        Self {
            name,
            path,
            summary_group_value: SummaryGroupValue::new_from_any_value(value),
        }
    }
}

impl DataRecordResolvedValue<'_> {
    pub fn get_name(&self) -> Option<&str> {
        self.name
    }

    pub fn get_path(&self) -> &str {
        self.path
    }

    pub fn get_summary_group_value(&self) -> &SummaryGroupValue {
        &self.summary_group_value
    }
}

impl<'a, T: DataRecord> ExecutionContext<'a> for DataRecordExecutionContext<'a, T> {
    fn evaluate<'b>(
        &self,
        expression: &'b dyn DataExpression,
    ) -> Result<DataExpressionResult, Error>
    where
        'b: 'a,
    {
        let resolved_values_index = self.resolved_values.borrow().len();

        let result = expression.evaluate(self);

        let mut resolved_values = self.resolved_values.borrow_mut();

        if resolved_values.len() > resolved_values_index {
            resolved_values.truncate(resolved_values_index);
        }

        result
    }

    fn clear(&self) {
        self.data_record.borrow_mut().clear();
    }

    fn read_any_value<'b>(
        &self,
        expression_id: usize,
        resolve_value_expr: &'b ResolveValueExpression,
        action: &mut dyn DataRecordAnyValueReadCallback,
    ) where
        'b: 'a,
    {
        let r = self.resolver_cache.invoke_resolver(
            expression_id,
            self,
            resolve_value_expr.get_path(),
            self.data_record,
            |resolver, data_record| {
                resolver.read_value(data_record, |r| {
                    if let DataRecordReadAnyValueResult::Found(v) = r {
                        self.resolved_values
                            .borrow_mut()
                            .push(DataRecordResolvedValue::new(
                                None,
                                resolve_value_expr.get_path().get_raw_value(),
                                v,
                            ));
                    }
                    action.invoke_once(r);
                });
            },
        );

        if r.is_err() {
            self.add_message_for_expression_id(
                expression_id,
                ExpressionMessage::err(format!(
                    "ExecutionContext read operation returned an error: {}",
                    r.unwrap_err()
                )),
            );

            action.invoke_once(DataRecordReadAnyValueResult::NotFound);
        }
    }

    fn read_any_value_from_attached<'b>(
        &self,
        expression_id: usize,
        name: &'b str,
        path: &'b ValuePath,
        action: &mut dyn DataRecordAnyValueReadCallback,
    ) where
        'b: 'a,
    {
        let r = self
            .get_attached_data_records()
            .get_attached_data_record(name);
        if r.is_none() {
            self.add_message_for_expression_id(
                expression_id,
                ExpressionMessage::warn(format!(
                    "ExecutionContext could not find attached data for name {name}"
                )),
            );
            action.invoke_once(DataRecordReadAnyValueResult::NotFound);
            return;
        }

        let r = self.resolver_cache.read_value(
            expression_id,
            self,
            path,
            r.unwrap(),
            &mut DataRecordAnyValueReadClosureCallback::new(|r| {
                if let DataRecordReadAnyValueResult::Found(v) = r {
                    self.resolved_values
                        .borrow_mut()
                        .push(DataRecordResolvedValue::new(
                            Some(name),
                            path.get_raw_value(),
                            v,
                        ));
                }
                action.invoke_once(r);
            }),
        );

        if r.is_err() {
            self.add_message_for_expression_id(
                expression_id,
                ExpressionMessage::err(
                    format!("ExecutionContext found attached data for name {name} but read operation returned an error: {}", r.unwrap_err())));

            action.invoke_once(DataRecordReadAnyValueResult::NotFound);
        }
    }

    fn set_any_value<'b>(
        &self,
        expression_id: usize,
        resolve_value_expr: &'b ResolveValueExpression,
        value: AnyValue,
    ) -> DataRecordSetAnyValueResult
    where
        'b: 'a,
    {
        let r = self.resolver_cache.invoke_resolver(
            expression_id,
            self,
            resolve_value_expr.get_path(),
            self.data_record,
            |resolver, data_record| resolver.set_value(data_record, value),
        );

        if r.is_err() {
            self.add_message_for_expression_id(
                expression_id,
                ExpressionMessage::err(format!(
                    "ExecutionContext set operation returned an error: {}",
                    r.unwrap_err()
                )),
            );

            return DataRecordSetAnyValueResult::NotFound;
        }

        r.unwrap()
    }

    fn remove_any_value<'b>(
        &self,
        expression_id: usize,
        resolve_value_expr: &'b ResolveValueExpression,
    ) -> DataRecordRemoveAnyValueResult
    where
        'b: 'a,
    {
        let r = self.resolver_cache.invoke_resolver(
            expression_id,
            self,
            resolve_value_expr.get_path(),
            self.data_record,
            |resolver, data_record| resolver.remove_value(data_record),
        );

        if r.is_err() {
            self.add_message_for_expression_id(
                expression_id,
                ExpressionMessage::err(format!(
                    "ExecutionContext remove operation returned an error: {}",
                    r.unwrap_err()
                )),
            );

            return DataRecordRemoveAnyValueResult::NotFound;
        }

        r.unwrap()
    }

    fn get_attached_data_records(&self) -> &dyn AttachedDataRecords {
        self.attached_data_records
    }

    fn get_data_record_index(&self) -> Option<usize> {
        self.data_record_index
    }

    fn get_timestamp(&self) -> Option<SystemTime> {
        self.data_record.borrow().get_timestamp()
    }

    fn get_observed_timestamp(&self) -> Option<SystemTime> {
        self.data_record.borrow().get_observed_timestamp()
    }

    fn get_summaries(&self) -> &Summaries {
        self.summaries
    }

    fn get_external_summary_index(&self) -> Result<Option<usize>, Error> {
        let data_record = self.data_record.borrow();
        let s = data_record.get_summary_id();
        if s.is_none() {
            return Ok(None);
        }

        let summary_id = s.unwrap();

        let summary_index = self.get_summaries().get_summary_index(summary_id);

        if summary_index.is_none() {
            return Err(Error::ExternalSummaryNotFound(summary_id.into()));
        }

        Ok(Some(summary_index.unwrap()))
    }

    fn get_summary_index(&self) -> Option<usize> {
        let summary_index = self.summary_index.borrow();
        if summary_index.is_none() {
            None
        } else {
            Some(*summary_index.as_ref().unwrap())
        }
    }

    fn set_summary_index(&self, index: usize) {
        *self.summary_index.borrow_mut() = Some(index);
    }

    fn get_variables(&self) -> &RefCell<HashMap<String, AnyValue>> {
        &self.variables
    }

    fn get_resolved_values(&self) -> &RefCell<Vec<DataRecordResolvedValue<'a>>> {
        &self.resolved_values
    }

    fn add_message_for_expression(&self, expression: &dyn Expression, message: ExpressionMessage) {
        self.add_message_for_expression_id(expression.get_id(), message);
    }

    fn add_message_for_expression_id(&self, expression_id: usize, mut message: ExpressionMessage) {
        if self.message_scope.is_some() {
            message.add_scope(self.message_scope.as_ref().unwrap());
        }

        let mut messages_borrow = self.messages.borrow_mut();

        let expression_messages =
            Self::get_messages_for_expression_id(&mut messages_borrow, expression_id);

        expression_messages.push(message);
    }

    fn write_debug_comments_for_expression(
        &self,
        expression: &dyn Expression,
        output: &mut String,
        padding: &str,
    ) {
        self.write_debug_comments_for_expression_id(expression.get_id(), output, padding);
    }

    fn write_debug_comments_for_expression_id(
        &self,
        expression_id: usize,
        output: &mut String,
        padding: &str,
    ) {
        let messages = self.messages.borrow();
        let messages_for_expression = messages.get(&expression_id);
        if messages_for_expression.is_some() {
            for message in messages_for_expression.unwrap() {
                output.push_str(padding);
                message.write_debug_comment(output);
            }
        }
    }
}

impl<'a, T: DataRecord> DataRecordExecutionContext<'a, T> {
    pub fn new(
        attached_data_records: &'a dyn AttachedDataRecords,
        data_record_index: Option<usize>,
        data_record: &'a RefCell<T>,
        message_scope: Option<&'a str>,
        messages: &'a RefCell<HashMap<usize, Vec<ExpressionMessage>>>,
        summaries: &'a Summaries,
        resolver_cache: &'a DataRecordAnyValueResolverCache,
    ) -> DataRecordExecutionContext<'a, T> {
        Self {
            attached_data_records,
            data_record_index,
            data_record,
            message_scope,
            messages,
            resolved_values: RefCell::new(Vec::new()),
            variables: RefCell::new(HashMap::new()),
            summaries,
            summary_index: RefCell::new(None),
            resolver_cache,
        }
    }

    pub(crate) fn get_messages_for_expression_id<'b>(
        messages: &'b mut RefMut<'a, HashMap<usize, Vec<ExpressionMessage>>>,
        expression_id: usize,
    ) -> &'b mut Vec<ExpressionMessage> {
        let entry = messages.entry(expression_id);

        entry.or_default()
    }
}
