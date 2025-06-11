use std::{any::Any, cell::RefCell, mem::replace};

use crate::{Error, ValuePath, execution_context::ExecutionContext, primitives::AnyValue};

use super::DataRecord;

pub(crate) trait DynamicDataRecordAnyValueResolver: Any {
    fn read_value(
        &self,
        expression_id: usize,
        execution_context: &dyn ExecutionContext,
        path: &ValuePath,
        data_record: &dyn DataRecord,
        action: &mut dyn DataRecordAnyValueReadCallback,
    ) -> Result<(), Error>;
}

pub struct DataRecordAnyValueResolver<T: DataRecord> {
    path: ValuePath,
    read_value_fn:
        Box<dyn for<'a, 'b> Fn(&'a ValuePath, &'b T) -> DataRecordReadAnyValueResult<'b>>,
    read_value_mut_fn:
        Box<dyn for<'a, 'b> Fn(&'a ValuePath, &'b mut T) -> DataRecordReadMutAnyValueResult<'b>>,
    set_value_fn: Box<dyn Fn(&ValuePath, &mut T, AnyValue) -> DataRecordSetAnyValueResult>,
    remove_value_fn: Box<dyn Fn(&ValuePath, &mut T) -> DataRecordRemoveAnyValueResult>,
}

impl<T: DataRecord> DataRecordAnyValueResolver<T> {
    pub fn new(
        path: ValuePath,
        read_value: impl for<'a, 'b> Fn(&'a ValuePath, &'b T) -> DataRecordReadAnyValueResult<'b>
        + 'static,
        read_value_mut: impl for<'a, 'b> Fn(
            &'a ValuePath,
            &'b mut T,
        ) -> DataRecordReadMutAnyValueResult<'b>
        + 'static,
        set_value: impl Fn(&ValuePath, &mut T, AnyValue) -> DataRecordSetAnyValueResult + 'static,
        remove_value: impl Fn(&ValuePath, &mut T) -> DataRecordRemoveAnyValueResult + 'static,
    ) -> DataRecordAnyValueResolver<T> {
        Self {
            path,
            read_value_fn: Box::new(read_value),
            read_value_mut_fn: Box::new(read_value_mut),
            set_value_fn: Box::new(set_value),
            remove_value_fn: Box::new(remove_value),
        }
    }

    pub fn new_no_op() -> DataRecordAnyValueResolver<T> {
        DataRecordAnyValueResolver::new(
            ValuePath::new("").unwrap(),
            |_, _| DataRecordReadAnyValueResult::NotFound,
            |_, _| DataRecordReadMutAnyValueResult::NotFound,
            |_, _, _| DataRecordSetAnyValueResult::NotFound,
            |_, _| DataRecordRemoveAnyValueResult::NotFound,
        )
    }

    pub(crate) fn read_value<F>(&self, data_record: &RefCell<T>, action: F)
    where
        F: FnOnce(DataRecordReadAnyValueResult),
    {
        let borrow = data_record.borrow();

        let result = (self.read_value_fn)(&self.path, &borrow);

        action(result);
    }

    pub(crate) fn read_value_direct<F>(&self, data_record: &T, action: F)
    where
        F: FnOnce(DataRecordReadAnyValueResult),
    {
        let result = (self.read_value_fn)(&self.path, data_record);

        action(result);
    }

    pub(crate) fn read_value_mut<F>(&self, data_record: &RefCell<T>, action: F)
    where
        F: FnOnce(DataRecordReadMutAnyValueResult),
    {
        let mut borrow = data_record.borrow_mut();

        let result = (self.read_value_mut_fn)(&self.path, &mut borrow);

        action(result);
    }

    pub(crate) fn set_value(
        &self,
        data_record: &RefCell<T>,
        value: AnyValue,
    ) -> DataRecordSetAnyValueResult {
        let mut borrow = data_record.borrow_mut();

        return (self.set_value_fn)(&self.path, &mut borrow, value);
    }

    pub(crate) fn remove_value(&self, data_record: &RefCell<T>) -> DataRecordRemoveAnyValueResult {
        let mut borrow = data_record.borrow_mut();

        return (self.remove_value_fn)(&self.path, &mut borrow);
    }
}

#[derive(Debug)]
pub enum DataRecordReadAnyValueResult<'a> {
    NotFound,
    Found(&'a AnyValue),
}

#[derive(Debug)]
pub enum DataRecordReadMutAnyValueResult<'a> {
    NotFound,
    NotSupported(&'static str),
    Found(&'a mut AnyValue),
}

#[derive(Debug)]
pub enum DataRecordSetAnyValueResult {
    NotFound,
    NotSupported(&'static str),
    Created,
    Updated(AnyValue),
}

#[derive(Debug)]
pub enum DataRecordRemoveAnyValueResult {
    NotFound,
    NotSupported(&'static str),
    Removed(AnyValue),
}

pub(crate) trait DataRecordAnyValueReadCallback {
    fn invoke_once(&mut self, result: DataRecordReadAnyValueResult);
}

pub(crate) struct DataRecordAnyValueReadClosureCallback<F>
where
    F: FnOnce(DataRecordReadAnyValueResult),
{
    callback: Option<F>,
}

impl<F> DataRecordAnyValueReadClosureCallback<F>
where
    F: FnOnce(DataRecordReadAnyValueResult),
{
    pub fn new(callback: F) -> DataRecordAnyValueReadClosureCallback<F> {
        Self {
            callback: Some(callback),
        }
    }
}

impl<F> DataRecordAnyValueReadCallback for DataRecordAnyValueReadClosureCallback<F>
where
    F: FnOnce(DataRecordReadAnyValueResult),
{
    fn invoke_once(&mut self, result: DataRecordReadAnyValueResult) {
        let callback = replace(&mut self.callback, None);
        if !callback.is_none() {
            (callback.unwrap())(result);
        }
    }
}

pub(crate) trait DataRecordAnyValueReadMutCallback {
    fn invoke_once(&mut self, result: DataRecordReadMutAnyValueResult);
}

pub(crate) struct DataRecordAnyValueReadMutClosureCallback<F>
where
    F: FnOnce(DataRecordReadMutAnyValueResult),
{
    callback: Option<F>,
}

impl<F> DataRecordAnyValueReadMutClosureCallback<F>
where
    F: FnOnce(DataRecordReadMutAnyValueResult),
{
    pub fn new(callback: F) -> DataRecordAnyValueReadMutClosureCallback<F> {
        Self {
            callback: Some(callback),
        }
    }
}

impl<F> DataRecordAnyValueReadMutCallback for DataRecordAnyValueReadMutClosureCallback<F>
where
    F: FnOnce(DataRecordReadMutAnyValueResult),
{
    fn invoke_once(&mut self, result: DataRecordReadMutAnyValueResult) {
        let callback = replace(&mut self.callback, None);
        if !callback.is_none() {
            (callback.unwrap())(result);
        }
    }
}
