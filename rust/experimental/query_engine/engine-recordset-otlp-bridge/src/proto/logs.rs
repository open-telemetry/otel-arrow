use std::collections::HashMap;

use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use data_engine_recordset::*;

use crate::{serializer::ProtobufField, *};

#[derive(Debug)]
pub struct ResourceLogs {
    pub resource: Option<Resource>,
    pub scope_logs: Vec<ScopeLogs>,
    pub(crate) extra_fields: Vec<ProtobufField>,
}

impl Default for ResourceLogs {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceLogs {
    pub fn new() -> ResourceLogs {
        Self {
            resource: None,
            scope_logs: Vec::new(),
            extra_fields: Vec::new(),
        }
    }

    pub fn with_resource(mut self, value: Resource) -> ResourceLogs {
        self.resource = Some(value);
        self
    }

    pub fn with_scope_logs(mut self, value: ScopeLogs) -> ResourceLogs {
        self.scope_logs.push(value);
        self
    }
}

#[derive(Debug)]
pub struct ScopeLogs {
    pub instrumentation_scope: Option<InstrumentationScope>,
    pub log_records: Vec<LogRecord>,
    pub(crate) extra_fields: Vec<ProtobufField>,
}

impl Default for ScopeLogs {
    fn default() -> Self {
        Self::new()
    }
}

impl ScopeLogs {
    pub fn new() -> ScopeLogs {
        Self {
            instrumentation_scope: None,
            log_records: Vec::new(),
            extra_fields: Vec::new(),
        }
    }

    pub fn with_instrumentation_scope(mut self, value: InstrumentationScope) -> ScopeLogs {
        self.instrumentation_scope = Some(value);
        self
    }

    pub fn with_log_record(mut self, value: LogRecord) -> ScopeLogs {
        self.log_records.push(value);
        self
    }
}

#[derive(Debug)]
pub struct LogRecord {
    pub(crate) resource_id: Option<usize>,
    pub(crate) scope_id: Option<usize>,
    pub(crate) diagnostic_level: Option<RecordSetEngineDiagnosticLevel>,
    pub timestamp: Option<ValueStorage<DateTime<FixedOffset>>>,
    pub observed_timestamp: Option<ValueStorage<DateTime<FixedOffset>>>,
    pub severity_number: Option<ValueStorage<i32>>,
    pub severity_text: Option<ValueStorage<String>>,
    pub body: Option<AnyValue>,
    pub attributes: MapValueStorage<AnyValue>,
    pub flags: Option<ValueStorage<u32>>,
    pub trace_id: Option<ByteArrayValueStorage>,
    pub span_id: Option<ByteArrayValueStorage>,
    pub event_name: Option<ValueStorage<String>>,
    pub(crate) extra_fields: Vec<ProtobufField>,
}

impl Default for LogRecord {
    fn default() -> Self {
        Self::new()
    }
}

impl LogRecord {
    pub fn new() -> LogRecord {
        Self {
            resource_id: None,
            scope_id: None,
            diagnostic_level: None,
            timestamp: None,
            observed_timestamp: None,
            severity_number: None,
            severity_text: None,
            body: None,
            attributes: MapValueStorage::new(HashMap::new()),
            flags: None,
            trace_id: None,
            span_id: None,
            event_name: None,
            extra_fields: Vec::new(),
        }
    }

    pub fn with_timestamp(mut self, value: DateTime<FixedOffset>) -> LogRecord {
        self.timestamp = Some(ValueStorage::new(value));
        self
    }

    pub fn with_timestamp_unix_nanos(mut self, value: u64) -> LogRecord {
        self.timestamp = Some(ValueStorage::new(Utc.timestamp_nanos(value as i64).into()));
        self
    }

    pub fn with_observed_timestamp(mut self, value: DateTime<FixedOffset>) -> LogRecord {
        self.observed_timestamp = Some(ValueStorage::new(value));
        self
    }

    pub fn with_observed_timestamp_unix_nanos(mut self, value: u64) -> LogRecord {
        self.observed_timestamp = Some(ValueStorage::new(Utc.timestamp_nanos(value as i64).into()));
        self
    }

    pub fn with_severity_number(mut self, value: i32) -> LogRecord {
        self.severity_number = Some(ValueStorage::new(value));
        self
    }

    pub fn with_severity_text(mut self, value: String) -> LogRecord {
        self.severity_text = Some(ValueStorage::new(value));
        self
    }

    pub fn with_body(mut self, value: AnyValue) -> LogRecord {
        self.body = Some(value);
        self
    }

    pub fn with_attribute(mut self, key: &str, value: AnyValue) -> LogRecord {
        if !key.is_empty() {
            self.attributes.get_values_mut().insert(key.into(), value);
        }
        self
    }

    pub fn with_flags(mut self, value: u32) -> LogRecord {
        self.flags = Some(ValueStorage::new(value));
        self
    }

    pub fn with_trace_id(mut self, value: Vec<u8>) -> LogRecord {
        self.trace_id = Some(ByteArrayValueStorage::new(
            value.iter().map(|v| ValueStorage::new(*v)).collect(),
        ));
        self
    }

    pub fn with_span_id(mut self, value: Vec<u8>) -> LogRecord {
        self.span_id = Some(ByteArrayValueStorage::new(
            value.iter().map(|v| ValueStorage::new(*v)).collect(),
        ));
        self
    }

    pub fn with_event_name(mut self, value: String) -> LogRecord {
        self.event_name = Some(ValueStorage::new(value));
        self
    }
}
