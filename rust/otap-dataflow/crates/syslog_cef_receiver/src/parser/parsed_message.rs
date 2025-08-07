// SPDX-License-Identifier: Apache-2.0
// Copyright The OpenTelemetry Authors

use crate::parser::{cef::CefMessage, rfc3164::Rfc3164Message, rfc5424::Rfc5424Message};
use chrono::{DateTime, Datelike, Local, NaiveDateTime, TimeZone, Utc};
use otel_arrow_rust::encode::record::attributes::AttributesRecordBatchBuilder;
use std::borrow::Cow;

// Common attribute key constants for both RFC5424 and RFC3164 messages
const SYSLOG_FACILITY: &str = "syslog.facility";
const SYSLOG_SEVERITY: &str = "syslog.severity";
const SYSLOG_HOST_NAME: &str = "syslog.host_name";
const SYSLOG_MESSAGE: &str = "syslog.message";

// Attribute key constants for RFC5424 messages
const SYSLOG_VERSION: &str = "syslog.version";
const SYSLOG_APP_NAME: &str = "syslog.app_name";
const SYSLOG_PROCESS_ID: &str = "syslog.process_id";
const SYSLOG_MSG_ID: &str = "syslog.msg_id";
const SYSLOG_STRUCTURED_DATA: &str = "syslog.structured_data";

// Attribute key constants for RFC3164 messages
const SYSLOG_TAG: &str = "syslog.tag";
const SYSLOG_CONTENT: &str = "syslog.content";

// Attribute key constants for CEF messages
const CEF_VERSION: &str = "cef.version";
const CEF_DEVICE_VENDOR: &str = "cef.device_vendor";
const CEF_DEVICE_PRODUCT: &str = "cef.device_product";
const CEF_DEVICE_VERSION: &str = "cef.device_version";
const CEF_SIGNATURE_ID: &str = "cef.signature_id";
const CEF_NAME: &str = "cef.name";
const CEF_SEVERITY: &str = "cef.severity";

/// Enum to represent different parsed message types
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ParsedSyslogMessage<'a> {
    /// RFC 5424 formatted message
    Rfc5424(Rfc5424Message<'a>),
    /// RFC 3164 formatted message
    Rfc3164(Rfc3164Message<'a>),
    /// CEF formatted message
    Cef(CefMessage<'a>),
}

impl ParsedSyslogMessage<'_> {
    /// Returns the original input received by the receiver
    pub(crate) fn input(&self) -> Cow<'_, str> {
        match self {
            ParsedSyslogMessage::Rfc5424(msg) => String::from_utf8_lossy(msg.input),
            ParsedSyslogMessage::Rfc3164(msg) => String::from_utf8_lossy(msg.input),
            ParsedSyslogMessage::Cef(msg) => String::from_utf8_lossy(msg.input),
        }
    }

    /// Returns the time when the event occurred.
    // Value is UNIX Epoch time in nanoseconds since 00:00:00 UTC on 1 January 1970.
    pub(crate) fn timestamp(&self) -> Option<u64> {
        match self {
            ParsedSyslogMessage::Rfc5424(msg) => msg.timestamp.and_then(|ts| {
                std::str::from_utf8(ts).ok().and_then(|timestamp_str| {
                    // RFC 5424 timestamps are in ISO 8601 format (e.g., "2003-10-11T22:14:15.003Z")
                    // Try to parse as RFC 3339 (ISO 8601)
                    DateTime::parse_from_rfc3339(timestamp_str)
                        .ok()
                        .map(|dt| dt.timestamp_nanos_opt().unwrap_or(0) as u64)
                })
            }),
            ParsedSyslogMessage::Rfc3164(msg) => msg.timestamp.and_then(|ts| {
                std::str::from_utf8(ts).ok().and_then(|timestamp_str| {
                    // RFC 3164 format: "Oct 11 22:14:15"
                    // We need to assume the current year and local timezone
                    let current_year = Local::now().year();

                    // Parse the timestamp with assumed year
                    let full_timestamp = format!("{current_year} {timestamp_str}");

                    // Try to parse with format "%Y %b %d %H:%M:%S"
                    if let Ok(naive_dt) =
                        NaiveDateTime::parse_from_str(&full_timestamp, "%Y %b %d %H:%M:%S")
                    {
                        // Convert to local timezone, then to UTC
                        if let Some(local_dt) = Local.from_local_datetime(&naive_dt).single() {
                            return Some(
                                local_dt
                                    .with_timezone(&Utc)
                                    .timestamp_nanos_opt()
                                    .unwrap_or(0) as u64,
                            );
                        }
                    }

                    None
                })
            }),
            ParsedSyslogMessage::Cef(_) => None, // CEF does not have a timestamp field
        }
    }

    /// Returns the severity level of the log message.
    pub(crate) fn severity(&self) -> Option<(i32, &str)> {
        match self {
            ParsedSyslogMessage::Rfc5424(msg) => {
                Some(Self::to_otel_severity(msg.priority.severity))
            }
            ParsedSyslogMessage::Rfc3164(msg) => {
                Some(Self::to_otel_severity(msg.priority.severity))
            }
            ParsedSyslogMessage::Cef(_) => {
                // CEF does not have a severity field, return None
                None
            }
        }
    }

    /// Adds attributes to the log record attributes Arrow record batch.
    /// Returns the number of attributes added.
    #[must_use]
    pub(crate) fn add_attribues_to_arrow(
        &self,
        log_attributes_arrow_records: &mut AttributesRecordBatchBuilder<u16>,
    ) -> u16 {
        let mut attributes_count = 0;
        match self {
            ParsedSyslogMessage::Rfc5424(msg) => {
                attributes_count += 3; // version, facility, and severity are always present

                log_attributes_arrow_records.append_key(SYSLOG_VERSION);
                log_attributes_arrow_records.append_int(msg.version.into());

                log_attributes_arrow_records.append_key(SYSLOG_FACILITY);
                log_attributes_arrow_records.append_int(msg.priority.facility.into());

                log_attributes_arrow_records.append_key(SYSLOG_SEVERITY);
                log_attributes_arrow_records.append_int(msg.priority.severity.into());

                if let Some(hostname) = msg.hostname {
                    log_attributes_arrow_records.append_key(SYSLOG_HOST_NAME);
                    log_attributes_arrow_records
                        .append_str(std::str::from_utf8(hostname).unwrap_or_default());
                    attributes_count += 1;
                }

                if let Some(appname) = msg.app_name {
                    log_attributes_arrow_records.append_key(SYSLOG_APP_NAME);
                    log_attributes_arrow_records
                        .append_str(std::str::from_utf8(appname).unwrap_or_default());
                    attributes_count += 1;
                }

                if let Some(proc_id) = msg.proc_id {
                    log_attributes_arrow_records.append_key(SYSLOG_PROCESS_ID);
                    log_attributes_arrow_records.append_int(
                        std::str::from_utf8(proc_id)
                            .unwrap_or_default()
                            .parse::<i64>()
                            .unwrap_or(0),
                    );
                    attributes_count += 1;
                }

                if let Some(msg_id) = msg.msg_id {
                    log_attributes_arrow_records.append_key(SYSLOG_MSG_ID);
                    log_attributes_arrow_records
                        .append_str(std::str::from_utf8(msg_id).unwrap_or_default());
                    attributes_count += 1;
                }

                if let Some(structured_data) = &msg.structured_data {
                    log_attributes_arrow_records.append_key(SYSLOG_STRUCTURED_DATA);
                    log_attributes_arrow_records
                        .append_str(std::str::from_utf8(structured_data).unwrap_or_default());
                    attributes_count += 1;
                }

                if let Some(message) = msg.message {
                    log_attributes_arrow_records.append_key(SYSLOG_MESSAGE);
                    log_attributes_arrow_records
                        .append_str(std::str::from_utf8(message).unwrap_or_default());
                    attributes_count += 1;
                }

                attributes_count
            }
            ParsedSyslogMessage::Rfc3164(msg) => {
                attributes_count += 2; // facility and severity are always present

                log_attributes_arrow_records.append_key(SYSLOG_FACILITY);
                log_attributes_arrow_records.append_int(msg.priority.facility.into());

                log_attributes_arrow_records.append_key(SYSLOG_SEVERITY);
                log_attributes_arrow_records.append_int(msg.priority.severity.into());

                if let Some(hostname) = msg.hostname {
                    log_attributes_arrow_records.append_key(SYSLOG_HOST_NAME);
                    log_attributes_arrow_records
                        .append_str(std::str::from_utf8(hostname).unwrap_or_default());
                    attributes_count += 1;
                }

                if let Some(tag) = msg.tag {
                    log_attributes_arrow_records.append_key(SYSLOG_TAG);
                    log_attributes_arrow_records
                        .append_str(std::str::from_utf8(tag).unwrap_or_default());
                    attributes_count += 1;
                }

                if let Some(content) = msg.content {
                    log_attributes_arrow_records.append_key(SYSLOG_CONTENT);
                    log_attributes_arrow_records
                        .append_str(std::str::from_utf8(content).unwrap_or_default());
                    attributes_count += 1;
                }

                if let Some(message) = msg.message {
                    log_attributes_arrow_records.append_key(SYSLOG_MESSAGE);
                    log_attributes_arrow_records
                        .append_str(std::str::from_utf8(message).unwrap_or_default());
                    attributes_count += 1;
                }

                attributes_count
            }
            ParsedSyslogMessage::Cef(msg) => {
                attributes_count += 7; // version, device_vendor, device_product, device_version, signature_id, name, and severity are always present

                log_attributes_arrow_records.append_key(CEF_VERSION);
                log_attributes_arrow_records.append_int(msg.version.into());

                log_attributes_arrow_records.append_key(CEF_DEVICE_VENDOR);
                log_attributes_arrow_records
                    .append_str(std::str::from_utf8(msg.device_vendor).unwrap_or_default());

                log_attributes_arrow_records.append_key(CEF_DEVICE_PRODUCT);
                log_attributes_arrow_records
                    .append_str(std::str::from_utf8(msg.device_product).unwrap_or_default());

                log_attributes_arrow_records.append_key(CEF_DEVICE_VERSION);
                log_attributes_arrow_records
                    .append_str(std::str::from_utf8(msg.device_version).unwrap_or_default());

                log_attributes_arrow_records.append_key(CEF_SIGNATURE_ID);
                log_attributes_arrow_records
                    .append_str(std::str::from_utf8(msg.signature_id).unwrap_or_default());

                log_attributes_arrow_records.append_key(CEF_NAME);
                log_attributes_arrow_records
                    .append_str(std::str::from_utf8(msg.name).unwrap_or_default());

                log_attributes_arrow_records.append_key(CEF_SEVERITY);
                log_attributes_arrow_records
                    .append_str(std::str::from_utf8(msg.severity).unwrap_or_default());

                for (key, value) in msg.parse_extensions() {
                    log_attributes_arrow_records
                        .append_key(std::str::from_utf8(key).unwrap_or_default());
                    log_attributes_arrow_records
                        .append_str(std::str::from_utf8(value).unwrap_or_default());
                    attributes_count += 1;
                }

                attributes_count
            }
        }
    }

    /// Follows the severity number mapping mentioned in the Data Model Appendix B in the logs specification:
    /// https://github.com/open-telemetry/opentelemetry-specification/blob/v1.47.0/specification/logs/data-model-appendix.md#appendix-b-severitynumber-example-mappings
    const fn to_otel_severity(syslog_severity: u8) -> (i32, &'static str) {
        match syslog_severity {
            0 => (21, "FATAL"),      // Emergency -> SEVERITY_NUMBER_FATAL
            1 => (19, "ERROR3"),     // Alert -> SEVERITY_NUMBER_ERROR3
            2 => (18, "ERROR2"),     // Critical -> SEVERITY_NUMBER_ERROR2
            3 => (17, "ERROR"),      // Error -> SEVERITY_NUMBER_ERROR
            4 => (13, "WARN"),       // Warning -> SEVERITY_NUMBER_WARN
            5 => (10, "INFO2"),      // Notice -> SEVERITY_NUMBER_INFO2
            6 => (9, "INFO"),        // Informational -> SEVERITY_NUMBER_INFO
            7 => (5, "DEBUG"),       // Debug -> SEVERITY_NUMBER_DEBUG
            _ => (0, "UNSPECIFIED"), // Unknown severity -> SEVERITY_NUMBER_UNSPECIFIED
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn test_parsed_syslog_message_timestamp_rfc5424() {
        let input = b"<34>1 2003-10-11T22:14:15.003Z mymachine.example.com su - ID47 - BOM'su root' failed for lonvick on /dev/pts/8";
        let result = parse(input).unwrap();

        // Test the ParsedSyslogMessage::timestamp method
        let timestamp_nanos = result.timestamp().unwrap();
        // Parse the expected timestamp and convert to nanoseconds for comparison
        let expected_dt = DateTime::parse_from_rfc3339("2003-10-11T22:14:15.003Z").unwrap();
        let expected_nanos = expected_dt.timestamp_nanos_opt().unwrap() as u64;
        assert_eq!(timestamp_nanos, expected_nanos);
    }

    #[test]
    fn test_parsed_syslog_message_timestamp_rfc3164() {
        let input = b"<34>Oct 11 22:14:15 mymachine su: 'su root' failed for lonvick on /dev/pts/8";
        let result = parse(input).unwrap();

        // Test the ParsedSyslogMessage::timestamp method
        let timestamp_nanos = result.timestamp().unwrap();
        // For RFC 3164, we expect the current year to be used since it's not specified
        let current_year = Local::now().year();
        let full_timestamp = format!("{current_year} Oct 11 22:14:15");
        if let Ok(naive_dt) = NaiveDateTime::parse_from_str(&full_timestamp, "%Y %b %d %H:%M:%S") {
            if let Some(local_dt) = Local.from_local_datetime(&naive_dt).single() {
                let expected_nanos = local_dt
                    .with_timezone(&Utc)
                    .timestamp_nanos_opt()
                    .unwrap_or(0) as u64;
                assert_eq!(timestamp_nanos, expected_nanos);
            }
        }
    }

    #[test]
    fn test_parsed_syslog_message_severity() {
        // Test RFC 5424 severity mapping
        let input = b"<34>1 - - - - - - Test message";
        let result = parse(input).unwrap();
        let (severity_num, severity_text) = result.severity().unwrap();
        assert_eq!(severity_num, 18); // Critical -> ERROR2
        assert_eq!(severity_text, "ERROR2");

        // Test RFC 3164 severity mapping
        let input = b"<36>Oct 11 22:14:15 host tag: message";
        let result = parse(input).unwrap();
        let (severity_num, severity_text) = result.severity().unwrap();
        assert_eq!(severity_num, 13); // Warning -> WARN
        assert_eq!(severity_text, "WARN");

        // Test CEF (should return None)
        let input = b"CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|10|";
        let result = parse(input).unwrap();
        assert!(result.severity().is_none());
    }

    #[test]
    fn test_parsed_syslog_message_input() {
        let input = b"<34>1 2003-10-11T22:14:15.003Z host app - - - Test message";
        let result = parse(input).unwrap();

        let input_str = result.input();
        assert_eq!(
            input_str,
            "<34>1 2003-10-11T22:14:15.003Z host app - - - Test message"
        );
    }
}
