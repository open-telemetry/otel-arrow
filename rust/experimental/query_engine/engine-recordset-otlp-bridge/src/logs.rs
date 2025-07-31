use std::{collections::HashMap, mem, time::SystemTime};

use data_engine_expressions::*;
use data_engine_recordset2::*;

use crate::{attached_records::OtlpAttachedRecords, *};

impl RecordSet<LogRecord> for ExportLogsServiceRequest {
    fn drain<F>(&mut self, action: &mut F)
    where
        F: FnMut(Option<&dyn AttachedRecords>, LogRecord),
    {
        for (resource_id, resource_log) in self.resource_logs.iter_mut().enumerate() {
            let resource = resource_log.resource.as_ref().map(|v| v as &dyn MapValue);

            for (scope_id, scope_log) in resource_log.scope_logs.iter_mut().enumerate() {
                let scope = scope_log
                    .instrumentation_scope
                    .as_ref()
                    .map(|v| v as &dyn MapValue);

                for mut log_record in scope_log.log_records.drain(..) {
                    log_record.resource_id = Some(resource_id);
                    log_record.scope_id = Some(scope_id);

                    (action)(Some(&OtlpAttachedRecords::new(resource, scope)), log_record)
                }
            }
        }
    }
}

impl Record for LogRecord {
    fn get_timestamp(&self) -> Option<SystemTime> {
        self.timestamp.as_ref().map(|v| v.get_value().into())
    }

    fn get_observed_timestamp(&self) -> Option<SystemTime> {
        self.observed_timestamp
            .as_ref()
            .map(|v| v.get_value().into())
    }

    fn get_diagnostic_level(&self) -> Option<RecordSetEngineDiagnosticLevel> {
        self.diagnostic_level.clone()
    }
}

impl AsValue for LogRecord {
    fn get_value_type(&self) -> ValueType {
        ValueType::Map
    }

    fn to_value(&self) -> Value {
        Value::Map(self)
    }
}

impl MapValue for LogRecord {
    fn is_empty(&self) -> bool {
        false
    }

    fn len(&self) -> usize {
        (self.timestamp.is_some() as usize)
            + (self.observed_timestamp.is_some() as usize)
            + (self.severity_number.is_some() as usize)
            + (self.severity_text.is_some() as usize)
            + (self.body.is_some() as usize)
            + (self.trace_id.is_some() as usize)
            + (self.span_id.is_some() as usize)
            + (self.event_name.is_some() as usize)
            + (self.flags.is_some() as usize)
            + 1
    }

    fn contains_key(&self, key: &str) -> bool {
        match key {
            "Attributes" => true,
            "Timestamp" => self.timestamp.is_some(),
            "ObservedTimestamp" => self.observed_timestamp.is_some(),
            "SeverityNumber" => self.severity_number.is_some(),
            "SeverityText" => self.severity_text.is_some(),
            "Body" => self.body.is_some(),
            "TraceId" => self.trace_id.is_some(),
            "SpanId" => self.span_id.is_some(),
            "TraceFlags" => self.flags.is_some(),
            "EventName" => self.event_name.is_some(),
            _ => false,
        }
    }

    fn get(&self, key: &str) -> Option<&(dyn AsValue + 'static)> {
        match key {
            "Attributes" => Some(&self.attributes as &dyn AsValue),
            "Timestamp" => self.timestamp.as_ref().map(|v| v as &dyn AsValue),
            "ObservedTimestamp" => self.observed_timestamp.as_ref().map(|v| v as &dyn AsValue),
            "SeverityNumber" => self.severity_number.as_ref().map(|v| v as &dyn AsValue),
            "SeverityText" => self.severity_text.as_ref().map(|v| v as &dyn AsValue),
            "Body" => self.body.as_ref().map(|v| v as &dyn AsValue),
            "TraceId" => self.trace_id.as_ref().map(|v| v as &dyn AsValue),
            "SpanId" => self.span_id.as_ref().map(|v| v as &dyn AsValue),
            "TraceFlags" => self.flags.as_ref().map(|v| v as &dyn AsValue),
            "EventName" => self.event_name.as_ref().map(|v| v as &dyn AsValue),
            _ => None,
        }
    }

    fn get_items(&self, item_callback: &mut dyn KeyValueCallback) -> bool {
        if let Some(v) = &self.timestamp {
            if !item_callback.next("Timestamp", Value::DateTime(v)) {
                return false;
            }
        }
        if let Some(v) = &self.observed_timestamp {
            if !item_callback.next("ObservedTimestamp", Value::DateTime(v)) {
                return false;
            }
        }
        if let Some(v) = &self.severity_number {
            if !item_callback.next("SeverityNumber", Value::Integer(v)) {
                return false;
            }
        }
        if let Some(v) = &self.severity_text {
            if !item_callback.next("SeverityText", Value::String(v)) {
                return false;
            }
        }
        if let Some(v) = &self.body {
            if !item_callback.next("Body", v.to_value()) {
                return false;
            }
        }
        if !item_callback.next("Attributes", Value::Map(&self.attributes)) {
            return false;
        }
        if let Some(v) = &self.flags {
            if !item_callback.next("TraceFlags", Value::Integer(v)) {
                return false;
            }
        }
        if let Some(v) = &self.trace_id {
            if !item_callback.next("TraceId", Value::Array(v)) {
                return false;
            }
        }
        if let Some(v) = &self.span_id {
            if !item_callback.next("SpanId", Value::Array(v)) {
                return false;
            }
        }
        if let Some(v) = &self.event_name {
            if !item_callback.next("EventName", Value::String(v)) {
                return false;
            }
        }

        true
    }
}

impl AsValueMut for LogRecord {
    fn to_value_mut(&mut self) -> Option<ValueMut> {
        Some(ValueMut::Map(self))
    }
}

impl MapValueMut for LogRecord {
    fn get_mut(&mut self, key: &str) -> ValueMutGetResult {
        match key {
            "Attributes" => ValueMutGetResult::Found(&mut self.attributes),
            "Timestamp" => ValueMutGetResult::NotSupported(
                "Timestamp cannot be modified in place on LogRecord".into(),
            ),
            "ObservedTimestamp" => ValueMutGetResult::NotSupported(
                "ObservedTimestamp cannot be modified in place on LogRecord".into(),
            ),
            "SeverityNumber" => ValueMutGetResult::NotSupported(
                "SeverityNumber cannot be modified in place on LogRecord".into(),
            ),
            "SeverityText" => match &mut self.severity_text {
                Some(s) => ValueMutGetResult::Found(s),
                None => ValueMutGetResult::NotFound,
            },
            "Body" => match &mut self.body {
                Some(b) => ValueMutGetResult::Found(b as &mut dyn AsValueMut),
                None => ValueMutGetResult::NotFound,
            },
            "TraceId" => match &mut self.trace_id {
                Some(t) => ValueMutGetResult::Found(t),
                None => ValueMutGetResult::NotFound,
            },
            "SpanId" => match &mut self.span_id {
                Some(s) => ValueMutGetResult::Found(s),
                None => ValueMutGetResult::NotFound,
            },
            "TraceFlags" => ValueMutGetResult::NotSupported(
                "TraceFlags cannot be modified in place on LogRecord".into(),
            ),
            "EventName" => match &mut self.event_name {
                Some(e) => ValueMutGetResult::Found(e),
                None => ValueMutGetResult::NotFound,
            },
            _ => ValueMutGetResult::NotFound,
        }
    }

    fn set(&mut self, key: &str, value: ResolvedValue) -> ValueMutWriteResult {
        let value_type = value.get_value_type();

        match key {
            "Attributes" => {
                if let AnyValue::Native(OtlpAnyValue::KvlistValue(k)) = value.convert() {
                    let old = mem::replace(&mut self.attributes, k);
                    return ValueMutWriteResult::Updated(OwnedValue::Map(old.into()));
                }

                ValueMutWriteResult::NotSupported(format!(
                    "Attributes cannot be set to type '{value_type:?}' on LogRecord"
                ))
            }
            "Timestamp" => {
                if let AnyValue::Extended(ExtendedValue::DateTime(d)) = value.convert() {
                    return match self.timestamp.replace(d) {
                        Some(old) => ValueMutWriteResult::Updated(OwnedValue::DateTime(old)),
                        None => ValueMutWriteResult::Created,
                    };
                }

                ValueMutWriteResult::NotSupported(format!(
                    "Timestamp cannot be set to type '{value_type:?}' on LogRecord"
                ))
            }
            "ObservedTimestamp" => {
                if let AnyValue::Extended(ExtendedValue::DateTime(d)) = value.convert() {
                    return match self.observed_timestamp.replace(d) {
                        Some(old) => ValueMutWriteResult::Updated(OwnedValue::DateTime(old)),
                        None => ValueMutWriteResult::Created,
                    };
                }

                ValueMutWriteResult::NotSupported(format!(
                    "ObservedTimestamp cannot be set to type '{value_type:?}' on LogRecord"
                ))
            }
            "SeverityNumber" => {
                if let AnyValue::Native(OtlpAnyValue::IntValue(i)) = value.convert() {
                    let value = i.get_value();
                    if value >= i32::MIN as i64 && value <= i32::MAX as i64 {
                        return match self
                            .severity_number
                            .replace(ValueStorage::new(value as i32))
                        {
                            Some(old) => ValueMutWriteResult::Updated(OwnedValue::Integer(
                                ValueStorage::new(old.get_value()),
                            )),
                            None => ValueMutWriteResult::Created,
                        };
                    }
                }

                ValueMutWriteResult::NotSupported(format!(
                    "SeverityNumber cannot be set to type '{value_type:?}' on LogRecord"
                ))
            }
            "SeverityText" => {
                if let AnyValue::Native(OtlpAnyValue::StringValue(s)) = value.convert() {
                    return match self.severity_text.replace(s) {
                        Some(old) => ValueMutWriteResult::Updated(OwnedValue::String(old)),
                        None => ValueMutWriteResult::Created,
                    };
                }

                ValueMutWriteResult::NotSupported(format!(
                    "SeverityText cannot be set to type '{value_type:?}' on LogRecord"
                ))
            }
            "Body" => match self.body.replace(value.convert()) {
                Some(old) => ValueMutWriteResult::Updated(old.to_owned()),
                None => ValueMutWriteResult::Created,
            },
            "TraceId" => {
                if let AnyValue::Native(OtlpAnyValue::BytesValue(b)) = value.convert() {
                    return match self.trace_id.replace(b) {
                        Some(old) => ValueMutWriteResult::Updated(
                            AnyValue::Native(OtlpAnyValue::BytesValue(old)).to_owned(),
                        ),
                        None => ValueMutWriteResult::Created,
                    };
                }

                ValueMutWriteResult::NotSupported(format!(
                    "TraceId cannot be set to type '{value_type:?}' on LogRecord"
                ))
            }
            "SpanId" => {
                if let AnyValue::Native(OtlpAnyValue::BytesValue(b)) = value.convert() {
                    return match self.span_id.replace(b) {
                        Some(old) => ValueMutWriteResult::Updated(
                            AnyValue::Native(OtlpAnyValue::BytesValue(old)).to_owned(),
                        ),
                        None => ValueMutWriteResult::Created,
                    };
                }

                ValueMutWriteResult::NotSupported(format!(
                    "SpanId cannot be set to type '{value_type:?}' on LogRecord"
                ))
            }
            "TraceFlags" => {
                if let AnyValue::Native(OtlpAnyValue::IntValue(i)) = value.convert() {
                    let value = i.get_value();
                    if value >= u32::MIN as i64 && value <= u32::MAX as i64 {
                        return match self.flags.replace(ValueStorage::new(value as u32)) {
                            Some(old) => ValueMutWriteResult::Updated(OwnedValue::Integer(
                                ValueStorage::new(old.get_value()),
                            )),
                            None => ValueMutWriteResult::Created,
                        };
                    }
                }

                ValueMutWriteResult::NotSupported(format!(
                    "TraceFlags cannot be set to type '{value_type:?}' on LogRecord"
                ))
            }
            "EventName" => {
                if let AnyValue::Native(OtlpAnyValue::StringValue(s)) = value.convert() {
                    return match self.event_name.replace(s) {
                        Some(old) => ValueMutWriteResult::Updated(OwnedValue::String(old)),
                        None => ValueMutWriteResult::Created,
                    };
                }

                ValueMutWriteResult::NotSupported(format!(
                    "EventName cannot be set to type '{value_type:?}' on LogRecord"
                ))
            }
            _ => ValueMutWriteResult::NotFound,
        }
    }

    fn rename(&mut self, _: &str, _: &str) -> ValueMutWriteResult {
        ValueMutWriteResult::NotSupported("Fields of LogRecord can't be renamed".into())
    }

    fn remove(&mut self, key: &str) -> ValueMutRemoveResult {
        match key {
            "Attributes" => {
                let old = mem::replace(&mut self.attributes, MapValueStorage::new(HashMap::new()));
                ValueMutRemoveResult::Removed(OwnedValue::Map(old.into()))
            }
            "Timestamp" => match self.timestamp.take() {
                Some(old) => ValueMutRemoveResult::Removed(OwnedValue::DateTime(old)),
                None => ValueMutRemoveResult::NotFound,
            },
            "ObservedTimestamp" => match self.observed_timestamp.take() {
                Some(old) => ValueMutRemoveResult::Removed(OwnedValue::DateTime(old)),
                None => ValueMutRemoveResult::NotFound,
            },
            "SeverityNumber" => match self.severity_number.take() {
                Some(old) => ValueMutRemoveResult::Removed(OwnedValue::Integer(ValueStorage::new(
                    old.get_value(),
                ))),
                None => ValueMutRemoveResult::NotFound,
            },
            "SeverityText" => match self.severity_text.take() {
                Some(old) => ValueMutRemoveResult::Removed(OwnedValue::String(old)),
                None => ValueMutRemoveResult::NotFound,
            },
            "Body" => match self.body.take() {
                Some(old) => ValueMutRemoveResult::Removed(old.to_owned()),
                None => ValueMutRemoveResult::NotFound,
            },
            "TraceId" => match self.trace_id.take() {
                Some(old) => ValueMutRemoveResult::Removed(
                    AnyValue::Native(OtlpAnyValue::BytesValue(old)).to_owned(),
                ),
                None => ValueMutRemoveResult::NotFound,
            },
            "SpanId" => match self.span_id.take() {
                Some(old) => ValueMutRemoveResult::Removed(
                    AnyValue::Native(OtlpAnyValue::BytesValue(old)).to_owned(),
                ),
                None => ValueMutRemoveResult::NotFound,
            },
            "TraceFlags" => match self.flags.take() {
                Some(old) => ValueMutRemoveResult::Removed(OwnedValue::Integer(ValueStorage::new(
                    old.get_value(),
                ))),
                None => ValueMutRemoveResult::NotFound,
            },
            "EventName" => match self.event_name.take() {
                Some(old) => ValueMutRemoveResult::Removed(OwnedValue::String(old)),
                None => ValueMutRemoveResult::NotFound,
            },
            _ => ValueMutRemoveResult::NotFound,
        }
    }

    fn retain(&mut self, item_callback: &mut dyn KeyValueMutCallback) {
        if let Some(v) = &self.timestamp {
            if !item_callback.next("Timestamp", InnerValue::Value(v)) {
                self.timestamp = None;
            }
        }
        if let Some(v) = &self.observed_timestamp {
            if !item_callback.next("ObservedTimestamp", InnerValue::Value(v)) {
                self.observed_timestamp = None;
            }
        }
        if let Some(v) = &self.severity_number {
            if !item_callback.next("SeverityNumber", InnerValue::Value(v)) {
                self.severity_number = None;
            }
        }
        if let Some(v) = &mut self.severity_text {
            if !item_callback.next("SeverityText", InnerValue::ValueMut(v)) {
                self.severity_text = None;
            }
        }
        if let Some(v) = &mut self.body {
            if !item_callback.next("Body", InnerValue::ValueMut(v)) {
                self.body = None;
            }
        }
        if !item_callback.next("Attributes", InnerValue::ValueMut(&mut self.attributes)) {
            self.attributes = MapValueStorage::new(HashMap::new());
        }
        if let Some(v) = &self.flags {
            if !item_callback.next("TraceFlags", InnerValue::Value(v)) {
                self.flags = None;
            }
        }
        if let Some(v) = &mut self.trace_id {
            if !item_callback.next("TraceId", InnerValue::ValueMut(v)) {
                self.trace_id = None;
            }
        }
        if let Some(v) = &mut self.span_id {
            if !item_callback.next("SpanId", InnerValue::ValueMut(v)) {
                self.span_id = None;
            }
        }
        if let Some(v) = &mut self.event_name {
            if !item_callback.next("EventName", InnerValue::ValueMut(v)) {
                self.event_name = None;
            }
        }
    }
}
