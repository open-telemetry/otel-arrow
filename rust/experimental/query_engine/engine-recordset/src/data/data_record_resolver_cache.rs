use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::{HashMap, hash_map::Entry},
};

use crate::{Error, ValuePath, execution_context::*, expression::ExpressionMessage};

use super::{DataRecord, data_record_resolver::*};

pub(crate) struct DataRecordAnyValueResolverCache {
    cache: HashMap<TypeId, Box<dyn DynamicDataRecordAnyValueResolver>>,
}

impl DataRecordAnyValueResolverCache {
    pub fn new() -> DataRecordAnyValueResolverCache {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn register<T: DataRecord>(&mut self) -> Result<(), Error> {
        match self.cache.entry(TypeId::of::<T>()) {
            Entry::Occupied(_) => Err(Error::RegistrationError("DataRecord already registered")),
            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(Box::new(GenericDataRecordAnyValueResolverCache::<T>::new()));
                Ok(())
            }
        }
    }

    pub fn read_value(
        &self,
        expression_id: usize,
        execution_context: &dyn ExecutionContext,
        path: &ValuePath,
        data_record: &dyn DataRecord,
        action: &mut dyn DataRecordAnyValueReadCallback,
    ) -> Result<(), Error> {
        match self.cache.get(&data_record.type_id()) {
            Some(dynamic_resolver) => {
                dynamic_resolver.read_value(
                    expression_id,
                    execution_context,
                    path,
                    data_record,
                    action,
                )?;
                Ok(())
            }
            None => Err(Error::RegistrationError(
                "DataRecord registration could not be found",
            )),
        }
    }

    pub fn invoke_resolver<T: DataRecord, F, R>(
        &self,
        expression_id: usize,
        execution_context: &dyn ExecutionContext,
        path: &ValuePath,
        data_record: &RefCell<T>,
        action: F,
    ) -> Result<R, Error>
    where
        F: FnOnce(&DataRecordAnyValueResolver<T>, &RefCell<T>) -> R,
    {
        match self.cache.get(&TypeId::of::<T>()) {
            Some(dynamic_resolver) => {
                match (dynamic_resolver.as_ref() as &dyn Any)
                    .downcast_ref::<GenericDataRecordAnyValueResolverCache<T>>()
                {
                    Some(concrete_resolver) => Ok(concrete_resolver.invoke_resolver(
                        expression_id,
                        execution_context,
                        path,
                        data_record,
                        action,
                    )),
                    None => Err(Error::RegistrationError(
                        "DataRecord registration type mismatch",
                    )),
                }
            }
            None => Err(Error::RegistrationError(
                "DataRecord registration could not be found",
            )),
        }
    }
}

pub(crate) struct GenericDataRecordAnyValueResolverCache<T: DataRecord> {
    cache: RefCell<HashMap<ValuePath, DataRecordAnyValueResolver<T>>>,
}

impl<T: DataRecord> GenericDataRecordAnyValueResolverCache<T> {
    pub fn new() -> GenericDataRecordAnyValueResolverCache<T> {
        Self {
            cache: RefCell::new(HashMap::new()),
        }
    }

    pub fn invoke_resolver<F, R>(
        &self,
        expression_id: usize,
        execution_context: &dyn ExecutionContext,
        path: &ValuePath,
        data_record: &RefCell<T>,
        action: F,
    ) -> R
    where
        F: FnOnce(&DataRecordAnyValueResolver<T>, &RefCell<T>) -> R,
    {
        {
            let cache_read_borrow = self.cache.borrow();
            let resolver = cache_read_borrow.get(path);
            if let Some(resolver) = &resolver {
                execution_context.add_message_for_expression_id(
                    expression_id,
                    ExpressionMessage::info(format!(
                        "ExecutionContext AnyValue resolver for path '{}' resolved from cache",
                        path.get_raw_value()
                    )),
                );

                return action(resolver, data_record);
            }
        }

        let mut cache_update_borrow = self.cache.borrow_mut();
        let resolver = cache_update_borrow
            .entry(path.clone())
            .or_insert(T::get_any_value_resolver_for_path(path));

        action(resolver, data_record)
    }
}

impl<T: DataRecord> DynamicDataRecordAnyValueResolver
    for GenericDataRecordAnyValueResolverCache<T>
{
    fn read_value(
        &self,
        expression_id: usize,
        execution_context: &dyn ExecutionContext,
        path: &ValuePath,
        data_record: &dyn DataRecord,
        action: &mut dyn DataRecordAnyValueReadCallback,
    ) -> Result<(), Error> {
        {
            let cache_read_borrow = self.cache.borrow();
            let resolver = cache_read_borrow.get(path);
            if resolver.is_some() {
                execution_context.add_message_for_expression_id(
                    expression_id,
                    ExpressionMessage::info(format!(
                        "ExecutionContext AnyValue resolver for path '{}' resolved from cache",
                        path.get_raw_value()
                    )),
                );

                match (data_record as &dyn Any).downcast_ref::<T>() {
                    Some(typed_data_record) => {
                        resolver
                            .as_ref()
                            .unwrap()
                            .read_value_direct(typed_data_record, |r| action.invoke_once(r));
                        return Ok(());
                    }
                    None => {
                        return Err(Error::RegistrationError(
                            "DataRecord registration type mismatch",
                        ));
                    }
                }
            }
        }

        let mut cache_update_borrow = self.cache.borrow_mut();
        let resolver = cache_update_borrow
            .entry(path.clone())
            .or_insert(T::get_any_value_resolver_for_path(path));

        match (data_record as &dyn Any).downcast_ref::<T>() {
            Some(typed_data_record) => {
                resolver.read_value_direct(typed_data_record, |r| action.invoke_once(r));
                Ok(())
            }
            None => Err(Error::RegistrationError(
                "DataRecord registration type mismatch",
            )),
        }
    }
}
