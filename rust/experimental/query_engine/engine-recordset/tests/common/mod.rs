use std::{cell::RefCell, collections::HashMap, mem::replace, time::SystemTime};

use data_engine_recordset::{data::*, primitives::*, *};

pub fn create_data_engine() -> Result<DataEngine, Error> {
    let mut data_engine = DataEngine::new();

    data_engine.register::<TestResource>()?;
    data_engine.register::<TestInstrumentationScope>()?;
    data_engine.register::<TestLogRecord>()?;

    Ok(data_engine)
}

pub fn unwrap_results(
    mut results: DataEngineExecutionResults<TestLogRecordBatchItem>,
) -> (Vec<TestLogRecord>, Vec<TestLogRecord>) {
    (
        results
            .drain_included_records()
            .map(|i| i.unwrap_log_record())
            .collect(),
        results
            .drain_dropped_records()
            .map(|i| i.unwrap_log_record())
            .collect(),
    )
}

pub trait DataRecordWithAttributes: DataRecord {
    fn get_attributes(&self) -> Option<&AnyValue>;

    fn get_attributes_mut(&mut self) -> Option<&mut AnyValue>;

    fn set_attributes(&mut self, value: Option<AnyValue>) -> Option<AnyValue>;

    fn create_attribute_resolver(path: &ValuePath) -> DataRecordAnyValueResolver<Self>
    where
        Self: Sized,
    {
        if path.is_value_selector() {
            return create_map_value_resolver(
                path,
                &|x: &Self| x.get_attributes(),
                &|x| x.get_attributes_mut(),
                &|x, v| x.set_attributes(v),
            );
        }

        let (root, remaining_path) = path.extract_root_property_key().unwrap();

        if root == "@attributes" {
            return create_map_value_resolver(
                &remaining_path,
                &|x: &Self| x.get_attributes(),
                &|x| x.get_attributes_mut(),
                &|x, v| x.set_attributes(v),
            );
        }

        create_map_value_resolver(
            path,
            &|x: &Self| x.get_attributes(),
            &|x| x.get_attributes_mut(),
            &|x, v| x.set_attributes(v),
        )
    }
}

#[derive(Debug)]
pub struct TestLogRecordBatch<'a> {
    summaries: Vec<Summary>,
    resource: Option<&'a TestResource>,
    instrumentation_scope: Option<&'a TestInstrumentationScope>,
    records: Vec<TestLogRecordBatchItem<'a>>,
}

impl<'a> TestLogRecordBatch<'a> {
    pub fn new() -> TestLogRecordBatch<'a> {
        Self {
            summaries: Vec::new(),
            resource: None,
            instrumentation_scope: None,
            records: Vec::new(),
        }
    }

    pub fn new_with_attached(
        resource: &'a TestResource,
        instrumentation_scope: &'a TestInstrumentationScope,
    ) -> TestLogRecordBatch<'a> {
        Self {
            summaries: Vec::new(),
            resource: Some(resource),
            instrumentation_scope: Some(instrumentation_scope),
            records: Vec::new(),
        }
    }

    pub fn add_summary(&mut self, summary: Summary) {
        self.summaries.push(summary);
    }

    pub fn add_log_record(&mut self, log_record: TestLogRecord) {
        self.records.push(TestLogRecordBatchItem::new(
            self.resource,
            self.instrumentation_scope,
            log_record,
        ))
    }
}

impl<'a> DataRecordBatch<TestLogRecordBatchItem<'a>> for TestLogRecordBatch<'a> {
    fn drain<S: DataEngineState, F>(&mut self, state: &mut S, action: F) -> Result<(), Error>
    where
        F: Fn(&mut S, TestLogRecordBatchItem<'a>) -> Result<(), Error>,
    {
        for summary in self.summaries.drain(..) {
            state.register_summary(summary);
        }

        for record in self.records.drain(..) {
            action(state, record)?;
        }

        Ok(())
    }
}

impl Default for TestLogRecordBatch<'_> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct TestLogRecordBatchItem<'a> {
    resource: Option<&'a dyn DataRecord>,
    instrumentation_scope: Option<&'a dyn DataRecord>,
    log_record: RefCell<TestLogRecord>,
}

impl<'a> TestLogRecordBatchItem<'a> {
    fn new(
        resource: Option<&'a TestResource>,
        instrumentation_scope: Option<&'a TestInstrumentationScope>,
        log_record: TestLogRecord,
    ) -> TestLogRecordBatchItem<'a> {
        Self {
            resource: resource.map(|v| v as &dyn DataRecord),
            instrumentation_scope: instrumentation_scope.map(|v| v as &dyn DataRecord),
            log_record: log_record.into(),
        }
    }

    pub fn unwrap_log_record(self) -> TestLogRecord {
        self.log_record.into_inner()
    }
}

impl DataEngineItem<TestLogRecord> for TestLogRecordBatchItem<'_> {
    fn get_attached_data_record_names() -> &'static [&'static str] {
        static TEST_LOG_RECORD_ATTACHED_DATA_RECORD_NAMES: [&str; 1] = ["resource"];

        &TEST_LOG_RECORD_ATTACHED_DATA_RECORD_NAMES
    }

    fn get_data_record(&self) -> &RefCell<TestLogRecord> {
        &self.log_record
    }

    fn on_dropped(&mut self) -> bool {
        true
    }
}

impl AttachedDataRecords for TestLogRecordBatchItem<'_> {
    fn get_attached_data_record(&self, name: &str) -> Option<&dyn DataRecord> {
        match name {
            "resource" => self.resource,
            "instrumentation_scope" => self.instrumentation_scope,
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TestResource {
    attributes: Option<AnyValue>,
}

impl TestResource {
    pub fn new() -> TestResource {
        Self {
            attributes: Some(AnyValue::new_map_value(HashMap::new())),
        }
    }

    pub fn set_attribute(&mut self, key: &str, value: AnyValue) {
        if let AnyValue::MapValue(map_value) =
            &mut self.attributes.as_mut().expect("attributes not found")
        {
            map_value.insert(key, value);
            return;
        }

        panic!()
    }
}

impl DataRecord for TestResource {
    fn get_any_value_resolver_for_path(path: &ValuePath) -> DataRecordAnyValueResolver<Self> {
        if path.is_value_selector() {
            return DataRecordAnyValueResolver::new_no_op();
        }

        Self::create_attribute_resolver(path)
    }

    fn get_timestamp(&self) -> Option<SystemTime> {
        None
    }

    fn get_observed_timestamp(&self) -> Option<SystemTime> {
        None
    }

    fn get_summary_id(&self) -> Option<&str> {
        None
    }

    fn set_summary_id(&mut self, _: &str) {
        panic!("Summary isn't supported on TestResource")
    }

    fn clear(&mut self) {
        self.attributes = None
    }
}

impl DataRecordWithAttributes for TestResource {
    fn get_attributes(&self) -> Option<&AnyValue> {
        self.attributes.as_ref()
    }

    fn get_attributes_mut(&mut self) -> Option<&mut AnyValue> {
        self.attributes.as_mut()
    }

    fn set_attributes(&mut self, value: Option<AnyValue>) -> Option<AnyValue> {
        replace(&mut self.attributes, value)
    }
}

impl Default for TestResource {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct TestInstrumentationScope {
    name: Option<AnyValue>,
    attributes: Option<AnyValue>,
}

impl TestInstrumentationScope {
    pub fn new(name: &str) -> TestInstrumentationScope {
        Self {
            name: Some(AnyValue::new_string_value(name)),
            attributes: Some(AnyValue::new_map_value(HashMap::new())),
        }
    }

    pub fn set_attribute(&mut self, key: &str, value: AnyValue) {
        if let AnyValue::MapValue(map_value) =
            &mut self.attributes.as_mut().expect("attributes not found")
        {
            map_value.insert(key, value);
            return;
        }

        panic!()
    }

    fn get_any_value_resolver_for_path_and_key(
        key: &str,
        path: &ValuePath,
    ) -> DataRecordAnyValueResolver<Self> {
        if key == "@name" {
            create_string_value_resolver(
                path,
                &|s: &TestInstrumentationScope| s.name.as_ref(),
                &|s: &mut TestInstrumentationScope, v: Option<AnyValue>| replace(&mut s.name, v),
            )
        } else {
            Self::create_attribute_resolver(path)
        }
    }
}

impl DataRecord for TestInstrumentationScope {
    fn get_any_value_resolver_for_path(path: &ValuePath) -> DataRecordAnyValueResolver<Self> {
        match path.extract_root_property_key() {
            Some((key, path)) => {
                TestInstrumentationScope::get_any_value_resolver_for_path_and_key(key, &path)
            }
            None => DataRecordAnyValueResolver::new_no_op(),
        }
    }

    fn get_timestamp(&self) -> Option<SystemTime> {
        None
    }

    fn get_observed_timestamp(&self) -> Option<SystemTime> {
        None
    }

    fn get_summary_id(&self) -> Option<&str> {
        None
    }

    fn set_summary_id(&mut self, _: &str) {
        panic!("Summary isn't supported on TestInstrumentationScope")
    }

    fn clear(&mut self) {
        self.attributes = None;
        self.name = None;
    }
}

impl DataRecordWithAttributes for TestInstrumentationScope {
    fn get_attributes(&self) -> Option<&AnyValue> {
        self.attributes.as_ref()
    }

    fn get_attributes_mut(&mut self) -> Option<&mut AnyValue> {
        self.attributes.as_mut()
    }

    fn set_attributes(&mut self, value: Option<AnyValue>) -> Option<AnyValue> {
        replace(&mut self.attributes, value)
    }
}

#[derive(Debug)]
pub struct TestLogRecord {
    timestamp: SystemTime,
    observed_timestamp: SystemTime,
    body: Option<AnyValue>,
    attributes: Option<AnyValue>,
    summary_id: Option<Box<str>>,
}

impl TestLogRecord {
    pub fn new() -> TestLogRecord {
        TestLogRecord::new_with_timestamp(SystemTime::now())
    }

    pub fn new_with_timestamp(timestamp: SystemTime) -> TestLogRecord {
        Self {
            timestamp,
            observed_timestamp: SystemTime::now(),
            body: None,
            attributes: Some(AnyValue::new_map_value(HashMap::new())),
            summary_id: None,
        }
    }

    pub fn set_attribute(&mut self, key: &str, value: AnyValue) {
        if let AnyValue::MapValue(map_value) =
            &mut self.attributes.as_mut().expect("attributes not found")
        {
            map_value.insert(key, value);
            return;
        }

        panic!()
    }

    pub fn get_attribute(&self, key: &str) -> Option<&AnyValue> {
        if let AnyValue::MapValue(map_value) =
            self.attributes.as_ref().expect("attributes not found")
        {
            return map_value.get(key);
        }

        panic!()
    }

    pub fn get_attributes_map(&self) -> &MapValueData {
        if let AnyValue::MapValue(map_value) =
            self.attributes.as_ref().expect("attributes not found")
        {
            return map_value;
        }

        panic!()
    }

    fn get_any_value_resolver_for_path_and_key(
        key: &str,
        path: ValuePath,
    ) -> DataRecordAnyValueResolver<Self> {
        if key == "@body" {
            DataRecordAnyValueResolver::new(
                path,
                |path, data_record: &TestLogRecord| match data_record.body.as_ref() {
                    Some(body) => path.read(body),
                    None => DataRecordReadAnyValueResult::NotFound,
                },
                |path, data_record, v| {
                    if path.is_value_selector() {
                        let old_value = data_record.body.replace(v);

                        if old_value.is_none() {
                            return DataRecordSetAnyValueResult::Created;
                        }

                        DataRecordSetAnyValueResult::Updated(old_value.unwrap())
                    } else {
                        match data_record.body.as_mut() {
                            Some(body) => path.set(body, v),
                            None => DataRecordSetAnyValueResult::NotFound,
                        }
                    }
                },
                |path, data_record| {
                    if path.is_value_selector() {
                        match data_record.body.take() {
                            Some(body) => DataRecordRemoveAnyValueResult::Removed(body),
                            None => DataRecordRemoveAnyValueResult::NotFound,
                        }
                    } else {
                        match data_record.body.as_mut() {
                            Some(body) => path.remove(body),
                            None => DataRecordRemoveAnyValueResult::NotFound,
                        }
                    }
                },
            )
        } else {
            Self::create_attribute_resolver(&path.with_root_property_key(key))
        }
    }
}

impl DataRecord for TestLogRecord {
    fn get_any_value_resolver_for_path(path: &ValuePath) -> DataRecordAnyValueResolver<Self> {
        match path.extract_root_property_key() {
            Some((key, path)) => TestLogRecord::get_any_value_resolver_for_path_and_key(key, path),
            None => DataRecordAnyValueResolver::new_no_op(),
        }
    }

    fn get_timestamp(&self) -> Option<SystemTime> {
        Some(self.timestamp)
    }

    fn get_observed_timestamp(&self) -> Option<SystemTime> {
        Some(self.observed_timestamp)
    }

    fn get_summary_id(&self) -> Option<&str> {
        self.summary_id.as_ref().map(|v| v.as_ref())
    }

    fn set_summary_id(&mut self, summary_id: &str) {
        self.summary_id = Some(summary_id.into())
    }

    fn clear(&mut self) {
        self.timestamp = SystemTime::UNIX_EPOCH;
        self.observed_timestamp = SystemTime::UNIX_EPOCH;
        self.body = None;
        self.attributes = None;
    }
}

impl DataRecordWithAttributes for TestLogRecord {
    fn get_attributes(&self) -> Option<&AnyValue> {
        self.attributes.as_ref()
    }

    fn get_attributes_mut(&mut self) -> Option<&mut AnyValue> {
        self.attributes.as_mut()
    }

    fn set_attributes(&mut self, value: Option<AnyValue>) -> Option<AnyValue> {
        replace(&mut self.attributes, value)
    }
}

impl Default for TestLogRecord {
    fn default() -> Self {
        Self::new()
    }
}
