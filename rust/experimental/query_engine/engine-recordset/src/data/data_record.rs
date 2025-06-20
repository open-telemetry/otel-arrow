use core::fmt::Debug;
use std::{any::Any, time::SystemTime};

use crate::{primitives::any_value::AnyValue, value_path::ValuePath};

use super::data_record_resolver::*;

pub trait DataRecord: Any + Debug {
    fn get_timestamp(&self) -> Option<SystemTime>;

    fn get_observed_timestamp(&self) -> Option<SystemTime>;

    fn get_any_value_resolver_for_path(path: &ValuePath) -> DataRecordAnyValueResolver<Self>
    where
        Self: Sized;

    fn get_summary_id(&self) -> Option<&str>;

    fn set_summary_id(&mut self, summary_id: &str);

    fn clear(&mut self);
}

pub trait AttachedDataRecords: Debug {
    fn get_attached_data_record(&self, name: &str) -> Option<&dyn DataRecord>;
}

pub fn create_string_value_resolver<T: DataRecord, R, S>(
    path: &ValuePath,
    read_action: &'static R,
    set_action: &'static S,
) -> DataRecordAnyValueResolver<T>
where
    R: Fn(&T) -> Option<&AnyValue>,
    S: Fn(&mut T, Option<AnyValue>) -> Option<AnyValue>,
{
    if !path.is_value_selector() {
        return DataRecordAnyValueResolver::new_no_op();
    }

    DataRecordAnyValueResolver::new(
        path.clone(),
        |_, data_record| {
            let root = read_action(data_record);
            match root {
                Some(v) => DataRecordReadAnyValueResult::Found(v),
                None => DataRecordReadAnyValueResult::NotFound,
            }
        },
        move |_, data_record, v| {
            if let AnyValue::StringValue(_) = &v {
                let old_value = set_action(data_record, Some(v));

                if old_value.is_none() {
                    return DataRecordSetAnyValueResult::Created;
                }

                return DataRecordSetAnyValueResult::Updated(old_value.unwrap());
            } else if let AnyValue::NullValue = &v {
                let old_value = set_action(data_record, None);

                if old_value.is_none() {
                    return DataRecordSetAnyValueResult::NotFound;
                }

                return DataRecordSetAnyValueResult::Updated(old_value.unwrap());
            }

            DataRecordSetAnyValueResult::NotSupported("Value was not a String")
        },
        move |_, data_record| {
            let old_value = set_action(data_record, None);

            if old_value.is_none() {
                return DataRecordRemoveAnyValueResult::NotFound;
            }

            DataRecordRemoveAnyValueResult::Removed(old_value.unwrap())
        },
    )
}

pub fn create_map_value_resolver<T: DataRecord, R, M, S>(
    path: &ValuePath,
    read_action: &'static R,
    read_mut_action: &'static M,
    set_action: &'static S,
) -> DataRecordAnyValueResolver<T>
where
    R: Fn(&T) -> Option<&AnyValue>,
    M: Fn(&mut T) -> Option<&mut AnyValue>,
    S: Fn(&mut T, Option<AnyValue>) -> Option<AnyValue>,
{
    DataRecordAnyValueResolver::new(
        path.clone(),
        |path, data_record| {
            let root = read_action(data_record);
            match root {
                Some(v) => path.read(v),
                None => DataRecordReadAnyValueResult::NotFound,
            }
        },
        move |path, data_record, v| {
            if path.is_value_selector() {
                if let AnyValue::MapValue(_) = &v {
                    let old_value = set_action(data_record, Some(v));

                    if old_value.is_none() {
                        return DataRecordSetAnyValueResult::Created;
                    }

                    return DataRecordSetAnyValueResult::Updated(old_value.unwrap());
                } else if let AnyValue::NullValue = &v {
                    let old_value = set_action(data_record, None);

                    if old_value.is_none() {
                        return DataRecordSetAnyValueResult::NotFound;
                    }

                    return DataRecordSetAnyValueResult::Updated(old_value.unwrap());
                }

                DataRecordSetAnyValueResult::NotSupported("Value was not a Map")
            } else {
                let root = read_mut_action(data_record);
                match root {
                    Some(r) => path.set(r, v),
                    None => DataRecordSetAnyValueResult::NotFound,
                }
            }
        },
        move |path, data_record| {
            if path.is_value_selector() {
                let old_value = set_action(data_record, None);

                if old_value.is_none() {
                    return DataRecordRemoveAnyValueResult::NotFound;
                }

                DataRecordRemoveAnyValueResult::Removed(old_value.unwrap())
            } else {
                let root = read_mut_action(data_record);
                match root {
                    Some(r) => path.remove(r),
                    None => DataRecordRemoveAnyValueResult::NotFound,
                }
            }
        },
    )
}
