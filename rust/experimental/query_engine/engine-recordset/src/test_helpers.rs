#![cfg(test)]
use std::{collections::HashMap, time::SystemTime};

use chrono::{DateTime, FixedOffset};
use data_engine_expressions::*;

use crate::*;

pub struct TestAttachedRecords {
    records: HashMap<Box<str>, MapValueStorage<OwnedValue>>,
}

impl Default for TestAttachedRecords {
    fn default() -> Self {
        Self::new()
    }
}

impl TestAttachedRecords {
    pub fn new() -> TestAttachedRecords {
        Self {
            records: HashMap::new(),
        }
    }

    pub fn push(&mut self, name: &str, record: MapValueStorage<OwnedValue>) {
        self.records.insert(name.into(), record);
    }
}

impl AttachedRecords for TestAttachedRecords {
    fn get_attached_record(&self, name: &str) -> Option<&(dyn MapValue + 'static)> {
        self.records.get(name).map(|v| v as &dyn MapValue)
    }
}

#[derive(Debug)]
pub struct TestRecordSet {
    records: Vec<TestRecord>,
}

impl TestRecordSet {
    pub fn new(records: Vec<TestRecord>) -> TestRecordSet {
        Self { records }
    }
}

impl RecordSet<TestRecord> for TestRecordSet {
    fn drain<F>(&mut self, action: &mut F)
    where
        F: FnMut(Option<&dyn AttachedRecords>, TestRecord),
    {
        for record in self.records.drain(..) {
            (action)(None, record)
        }
    }
}

#[derive(Debug, Clone)]
pub struct TestRecord {
    values: HashMap<Box<str>, OwnedValue>,
}

impl TestRecord {
    pub fn new() -> TestRecord {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn with_timestamp(self, value: DateTime<FixedOffset>) -> TestRecord {
        let mut values = self.values;
        values.insert(
            "Timestamp".into(),
            OwnedValue::DateTime(DateTimeValueStorage::new(value)),
        );
        Self { values }
    }

    pub fn with_observed_timestamp(self, value: DateTime<FixedOffset>) -> TestRecord {
        let mut values = self.values;
        values.insert(
            "ObservedTimestamp".into(),
            OwnedValue::DateTime(DateTimeValueStorage::new(value)),
        );
        Self { values }
    }

    pub fn with_key_value(self, key: Box<str>, value: OwnedValue) -> TestRecord {
        let mut values = self.values;
        values.insert(key, value);
        Self { values }
    }
}

impl Default for TestRecord {
    fn default() -> Self {
        Self::new()
    }
}

impl Record for TestRecord {
    fn get_timestamp(&self) -> Option<SystemTime> {
        if let Some(OwnedValue::DateTime(d)) = self.values.get("Timestamp") {
            Some((*d.get_raw_value()).into())
        } else {
            None
        }
    }

    fn get_observed_timestamp(&self) -> Option<SystemTime> {
        if let Some(OwnedValue::DateTime(d)) = self.values.get("ObservedTimestamp") {
            Some((*d.get_raw_value()).into())
        } else {
            None
        }
    }

    fn get_diagnostic_level(&self) -> Option<RecordSetEngineDiagnosticLevel> {
        None
    }
}

impl AsStaticValue for TestRecord {
    fn to_static_value(&self) -> StaticValue<'_> {
        StaticValue::Map(self)
    }
}

impl MapValue for TestRecord {
    fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    fn len(&self) -> usize {
        self.values.len()
    }

    fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    fn get(&self, key: &str) -> Option<&(dyn AsStaticValue + 'static)> {
        self.values.get(key).map(|v| v as &dyn AsStaticValue)
    }

    fn get_items(&self, item_callback: &mut dyn KeyValueCallback) -> bool {
        for (k, v) in &self.values {
            if !item_callback.next(k, v.to_value()) {
                return false;
            }
        }

        true
    }
}

impl MapValueMut for TestRecord {
    fn get_mut(&mut self, key: &str) -> ValueMutGetResult<'_> {
        if let Some(v) = self.values.get_mut(key) {
            ValueMutGetResult::Found(v)
        } else {
            ValueMutGetResult::NotFound
        }
    }

    fn set(&mut self, key: &str, value: ResolvedValue) -> ValueMutWriteResult {
        match self.values.insert(key.into(), value.into()) {
            Some(old) => ValueMutWriteResult::Updated(old),
            None => ValueMutWriteResult::Created,
        }
    }

    fn rename(&mut self, from_key: &str, to_key: &str) -> ValueMutWriteResult {
        match self.values.remove(from_key) {
            Some(v) => self.set(to_key, ResolvedValue::Computed(v)),
            None => ValueMutWriteResult::NotFound,
        }
    }

    fn remove(&mut self, key: &str) -> ValueMutRemoveResult {
        match self.values.remove(key) {
            Some(v) => ValueMutRemoveResult::Removed(v),
            None => ValueMutRemoveResult::NotFound,
        }
    }

    fn retain(&mut self, item_callback: &mut dyn KeyValueMutCallback) {
        self.values.retain(|k, v| item_callback.next(k, v));
    }
}
