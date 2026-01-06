// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::{collections::HashMap, mem};

use data_engine_expressions::*;
use data_engine_recordset::*;

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
    fn get_diagnostic_level(&self) -> Option<RecordSetEngineDiagnosticLevel> {
        self.diagnostic_level.clone()
    }
}

impl AsStaticValue for LogRecord {
    fn to_static_value(&self) -> StaticValue<'_> {
        StaticValue::Map(self)
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
            "Attributes" | "attributes" => true,
            "Timestamp" | "time_unix_nano" => self.timestamp.is_some(),
            "ObservedTimestamp" | "observed_time_unix_nano" => self.observed_timestamp.is_some(),
            "SeverityNumber" | "severity_number" => self.severity_number.is_some(),
            "SeverityText" | "severity_text" => self.severity_text.is_some(),
            "Body" | "body" => self.body.is_some(),
            "TraceId" | "trace_id" => self.trace_id.is_some(),
            "SpanId" | "span_id" => self.span_id.is_some(),
            "TraceFlags" | "flags" => self.flags.is_some(),
            "EventName" | "event_name" => self.event_name.is_some(),
            _ => false,
        }
    }

    fn get(&self, key: &str) -> Option<&dyn AsValue> {
        self.get_static(key).unwrap().map(|v| v as &dyn AsValue)
    }

    fn get_static(&self, key: &str) -> Result<Option<&(dyn AsStaticValue + 'static)>, String> {
        Ok(match key {
            "Attributes" | "attributes" => Some(&self.attributes),
            "Timestamp" | "time_unix_nano" => {
                self.timestamp.as_ref().map(|v| v as &dyn AsStaticValue)
            }
            "ObservedTimestamp" | "observed_time_unix_nano" => self
                .observed_timestamp
                .as_ref()
                .map(|v| v as &dyn AsStaticValue),
            "SeverityNumber" | "severity_number" => self
                .severity_number
                .as_ref()
                .map(|v| v as &dyn AsStaticValue),
            "SeverityText" | "severity_text" => {
                self.severity_text.as_ref().map(|v| v as &dyn AsStaticValue)
            }
            "Body" | "body" => self.body.as_ref().map(|v| v as &dyn AsStaticValue),
            "TraceId" | "trace_id" => self.trace_id.as_ref().map(|v| v as &dyn AsStaticValue),
            "SpanId" | "span_id" => self.span_id.as_ref().map(|v| v as &dyn AsStaticValue),
            "TraceFlags" | "flags" => self.flags.as_ref().map(|v| v as &dyn AsStaticValue),
            "EventName" | "event_name" => self.event_name.as_ref().map(|v| v as &dyn AsStaticValue),
            _ => None,
        })
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

impl AsStaticValueMut for LogRecord {
    fn to_static_value_mut(&mut self) -> Option<StaticValueMut<'_>> {
        Some(StaticValueMut::Map(self))
    }
}

impl MapValueMut for LogRecord {
    fn get_mut(&mut self, key: &str) -> ValueMutGetResult<'_> {
        match key {
            "Attributes" | "attributes" => ValueMutGetResult::Found(&mut self.attributes),
            "Timestamp" | "time_unix_nano" => ValueMutGetResult::NotSupported(
                "Timestamp cannot be modified in place on LogRecord".into(),
            ),
            "ObservedTimestamp" | "observed_time_unix_nano" => ValueMutGetResult::NotSupported(
                "ObservedTimestamp cannot be modified in place on LogRecord".into(),
            ),
            "SeverityNumber" | "severity_number" => ValueMutGetResult::NotSupported(
                "SeverityNumber cannot be modified in place on LogRecord".into(),
            ),
            "SeverityText" | "severity_text" => match &mut self.severity_text {
                Some(s) => ValueMutGetResult::Found(s),
                None => ValueMutGetResult::NotFound,
            },
            "Body" | "body" => match &mut self.body {
                Some(b) => ValueMutGetResult::Found(b),
                None => ValueMutGetResult::NotFound,
            },
            "TraceId" | "trace_id" => match &mut self.trace_id {
                Some(t) => ValueMutGetResult::Found(t),
                None => ValueMutGetResult::NotFound,
            },
            "SpanId" | "span_id" => match &mut self.span_id {
                Some(s) => ValueMutGetResult::Found(s),
                None => ValueMutGetResult::NotFound,
            },
            "TraceFlags" | "flags" => ValueMutGetResult::NotSupported(
                "TraceFlags cannot be modified in place on LogRecord".into(),
            ),
            "EventName" | "event_name" => match &mut self.event_name {
                Some(e) => ValueMutGetResult::Found(e),
                None => ValueMutGetResult::NotFound,
            },
            _ => ValueMutGetResult::NotFound,
        }
    }

    fn set(&mut self, key: &str, value: ResolvedValue) -> ValueMutWriteResult {
        let value_type = value.get_value_type();

        let any_value = Into::<OwnedValue>::into(value).into();

        match key {
            "Attributes" | "attributes" => {
                if let AnyValue::Native(OtlpAnyValue::KvlistValue(k)) = any_value {
                    let old = mem::replace(&mut self.attributes, k);
                    return ValueMutWriteResult::Updated(OwnedValue::Map(old.into()));
                }

                ValueMutWriteResult::NotSupported(format!(
                    "Attributes cannot be set to type '{value_type:?}' on LogRecord"
                ))
            }
            "Timestamp" | "time_unix_nano" => {
                if let AnyValue::Extended(ExtendedValue::DateTime(d)) = any_value {
                    return match self.timestamp.replace(d) {
                        Some(old) => ValueMutWriteResult::Updated(OwnedValue::DateTime(old)),
                        None => ValueMutWriteResult::Created,
                    };
                }

                ValueMutWriteResult::NotSupported(format!(
                    "Timestamp cannot be set to type '{value_type:?}' on LogRecord"
                ))
            }
            "ObservedTimestamp" | "observed_time_unix_nano" => {
                if let AnyValue::Extended(ExtendedValue::DateTime(d)) = any_value {
                    return match self.observed_timestamp.replace(d) {
                        Some(old) => ValueMutWriteResult::Updated(OwnedValue::DateTime(old)),
                        None => ValueMutWriteResult::Created,
                    };
                }

                ValueMutWriteResult::NotSupported(format!(
                    "ObservedTimestamp cannot be set to type '{value_type:?}' on LogRecord"
                ))
            }
            "SeverityNumber" | "severity_number" => {
                if let AnyValue::Native(OtlpAnyValue::IntValue(i)) = any_value {
                    let value = i.get_value();
                    if value >= i32::MIN as i64 && value <= i32::MAX as i64 {
                        return match self
                            .severity_number
                            .replace(IntegerValueStorage::new(value as i32))
                        {
                            Some(old) => ValueMutWriteResult::Updated(OwnedValue::Integer(
                                IntegerValueStorage::new(old.get_value()),
                            )),
                            None => ValueMutWriteResult::Created,
                        };
                    }
                }

                ValueMutWriteResult::NotSupported(format!(
                    "SeverityNumber cannot be set to type '{value_type:?}' on LogRecord"
                ))
            }
            "SeverityText" | "severity_text" => {
                if let AnyValue::Native(OtlpAnyValue::StringValue(s)) = any_value {
                    return match self.severity_text.replace(s) {
                        Some(old) => ValueMutWriteResult::Updated(OwnedValue::String(old)),
                        None => ValueMutWriteResult::Created,
                    };
                }

                ValueMutWriteResult::NotSupported(format!(
                    "SeverityText cannot be set to type '{value_type:?}' on LogRecord"
                ))
            }
            "Body" | "body" => match self.body.replace(any_value) {
                Some(old) => ValueMutWriteResult::Updated(old.into()),
                None => ValueMutWriteResult::Created,
            },
            "TraceId" | "trace_id" => {
                if let AnyValue::Native(OtlpAnyValue::BytesValue(b)) = any_value {
                    return match self.trace_id.replace(b) {
                        Some(old) => ValueMutWriteResult::Updated(
                            AnyValue::Native(OtlpAnyValue::BytesValue(old)).into(),
                        ),
                        None => ValueMutWriteResult::Created,
                    };
                }

                ValueMutWriteResult::NotSupported(format!(
                    "TraceId cannot be set to type '{value_type:?}' on LogRecord"
                ))
            }
            "SpanId" | "span_id" => {
                if let AnyValue::Native(OtlpAnyValue::BytesValue(b)) = any_value {
                    return match self.span_id.replace(b) {
                        Some(old) => ValueMutWriteResult::Updated(
                            AnyValue::Native(OtlpAnyValue::BytesValue(old)).into(),
                        ),
                        None => ValueMutWriteResult::Created,
                    };
                }

                ValueMutWriteResult::NotSupported(format!(
                    "SpanId cannot be set to type '{value_type:?}' on LogRecord"
                ))
            }
            "TraceFlags" | "flags" => {
                if let AnyValue::Native(OtlpAnyValue::IntValue(i)) = any_value {
                    let value = i.get_value();
                    if value >= u32::MIN as i64 && value <= u32::MAX as i64 {
                        return match self.flags.replace(IntegerValueStorage::new(value as u32)) {
                            Some(old) => ValueMutWriteResult::Updated(OwnedValue::Integer(
                                IntegerValueStorage::new(old.get_value()),
                            )),
                            None => ValueMutWriteResult::Created,
                        };
                    }
                }

                ValueMutWriteResult::NotSupported(format!(
                    "TraceFlags cannot be set to type '{value_type:?}' on LogRecord"
                ))
            }
            "EventName" | "event_name" => {
                if let AnyValue::Native(OtlpAnyValue::StringValue(s)) = any_value {
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
            "Attributes" | "attributes" => {
                let old = mem::replace(&mut self.attributes, MapValueStorage::new(HashMap::new()));
                ValueMutRemoveResult::Removed(OwnedValue::Map(old.into()))
            }
            "Timestamp" | "time_unix_nano" => match self.timestamp.take() {
                Some(old) => ValueMutRemoveResult::Removed(OwnedValue::DateTime(old)),
                None => ValueMutRemoveResult::NotFound,
            },
            "ObservedTimestamp" | "observed_time_unix_nano" => match self.observed_timestamp.take()
            {
                Some(old) => ValueMutRemoveResult::Removed(OwnedValue::DateTime(old)),
                None => ValueMutRemoveResult::NotFound,
            },
            "SeverityNumber" | "severity_number" => match self.severity_number.take() {
                Some(old) => ValueMutRemoveResult::Removed(OwnedValue::Integer(
                    IntegerValueStorage::new(old.get_value()),
                )),
                None => ValueMutRemoveResult::NotFound,
            },
            "SeverityText" | "severity_text" => match self.severity_text.take() {
                Some(old) => ValueMutRemoveResult::Removed(OwnedValue::String(old)),
                None => ValueMutRemoveResult::NotFound,
            },
            "Body" | "body" => match self.body.take() {
                Some(old) => ValueMutRemoveResult::Removed(old.into()),
                None => ValueMutRemoveResult::NotFound,
            },
            "TraceId" | "trace_id" => match self.trace_id.take() {
                Some(old) => ValueMutRemoveResult::Removed(
                    AnyValue::Native(OtlpAnyValue::BytesValue(old)).into(),
                ),
                None => ValueMutRemoveResult::NotFound,
            },
            "SpanId" | "span_id" => match self.span_id.take() {
                Some(old) => ValueMutRemoveResult::Removed(
                    AnyValue::Native(OtlpAnyValue::BytesValue(old)).into(),
                ),
                None => ValueMutRemoveResult::NotFound,
            },
            "TraceFlags" | "flags" => match self.flags.take() {
                Some(old) => ValueMutRemoveResult::Removed(OwnedValue::Integer(
                    IntegerValueStorage::new(old.get_value()),
                )),
                None => ValueMutRemoveResult::NotFound,
            },
            "EventName" | "event_name" => match self.event_name.take() {
                Some(old) => ValueMutRemoveResult::Removed(OwnedValue::String(old)),
                None => ValueMutRemoveResult::NotFound,
            },
            _ => ValueMutRemoveResult::NotFound,
        }
    }

    fn retain(&mut self, item_callback: &mut dyn KeyValueMutCallback) {
        if let Some(v) = &mut self.timestamp {
            if !item_callback.next("Timestamp", v) {
                self.timestamp = None;
            }
        }
        if let Some(v) = &mut self.observed_timestamp {
            if !item_callback.next("ObservedTimestamp", v) {
                self.observed_timestamp = None;
            }
        }
        if let Some(v) = &mut self.severity_number {
            if !item_callback.next("SeverityNumber", v) {
                self.severity_number = None;
            }
        }
        if let Some(v) = &mut self.severity_text {
            if !item_callback.next("SeverityText", v) {
                self.severity_text = None;
            }
        }
        if let Some(v) = &mut self.body {
            if !item_callback.next("Body", v) {
                self.body = None;
            }
        }
        if !item_callback.next("Attributes", &mut self.attributes) {
            self.attributes = MapValueStorage::new(HashMap::new());
        }
        if let Some(v) = &mut self.flags {
            if !item_callback.next("TraceFlags", v) {
                self.flags = None;
            }
        }
        if let Some(v) = &mut self.trace_id {
            if !item_callback.next("TraceId", v) {
                self.trace_id = None;
            }
        }
        if let Some(v) = &mut self.span_id {
            if !item_callback.next("SpanId", v) {
                self.span_id = None;
            }
        }
        if let Some(v) = &mut self.event_name {
            if !item_callback.next("EventName", v) {
                self.event_name = None;
            }
        }
    }
}
