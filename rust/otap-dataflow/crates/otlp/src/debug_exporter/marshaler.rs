// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTLPMarshaler for converting OTLP messages to structured string reports.
//!

use crate::proto::opentelemetry::{
    collector::{
        logs::v1::ExportLogsServiceRequest, metrics::v1::ExportMetricsServiceRequest,
        profiles::v1development::ExportProfilesServiceRequest,
        trace::v1::ExportTraceServiceRequest,
    },
    common::v1::{AnyValue, any_value::Value},
};
use std::fmt;
use std::fmt::Write;

/// Trait that provides methods to take OTLP messages and extract information from them and generate a report
pub trait OTLPMarshaler {
    /// extract data from logs and generate string report
    fn marshal_logs(&self, logs: ExportLogsServiceRequest) -> String;
    /// extract data from metricss and generate string report
    fn marshal_metrics(&self, metrics: ExportMetricsServiceRequest) -> String;
    /// extract data from traces and generate string report
    fn marshal_traces(&self, traces: ExportTraceServiceRequest) -> String;
    /// extract data from profiles and generate string report
    fn marshal_profiles(&self, profiles: ExportProfilesServiceRequest) -> String;
}

impl fmt::Display for AnyValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(value) = &self.value {
            match value {
                Value::StringValue(string) => {
                    write!(f, "{}", string)?;
                }
                Value::BoolValue(bool) => {
                    write!(f, "{}", bool)?;
                }
                Value::IntValue(int) => {
                    write!(f, "{}", int)?;
                }
                Value::DoubleValue(double) => {
                    write!(f, "{}", double)?;
                }
                Value::ArrayValue(array) => {
                    let values = &array.values;
                    for value in values {
                        write!(f, "{}", value)?;
                    }
                }
                Value::KvlistValue(kvlist) => {
                    let mut kv_string = String::new();
                    for kv in kvlist.values.iter() {
                        if let Some(value) = &kv.value {
                            _ = write!(
                                &mut kv_string,
                                "{key}={value} ",
                                key = kv.key,
                                value = value
                            );
                        }
                    }
                    write!(f, "{}", kv_string)?;
                }
                Value::BytesValue(bytes) => {
                    if let Ok(byte_string) = String::from_utf8(bytes.to_vec()) {
                        write!(f, "{}", byte_string)?;
                    }
                    write!(f, "")?;
                }
            }
        } else {
            write!(f, "")?;
        }
        Ok(())
    }
}
