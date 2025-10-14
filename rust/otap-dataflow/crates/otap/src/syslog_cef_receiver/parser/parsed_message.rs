// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::syslog_cef_receiver::parser::{
    cef::CefMessage, rfc3164::Rfc3164Message, rfc5424::Rfc5424Message,
};
use chrono::{DateTime, Datelike, Local, NaiveDateTime, TimeZone, Utc};
use otel_arrow_rust::encode::record::attributes::StrKeysAttributesRecordBatchBuilder;
use std::borrow::Cow;

// Common attribute key constants for both RFC5424 and RFC3164 messages
const SYSLOG_FACILITY: &str = "syslog.facility";
const SYSLOG_SEVERITY: &str = "syslog.severity";
const SYSLOG_HOST_NAME: &str = "syslog.host_name";

// Attribute key constants for RFC5424 messages
const SYSLOG_VERSION: &str = "syslog.version";
const SYSLOG_APP_NAME: &str = "syslog.app_name";
const SYSLOG_PROCESS_ID: &str = "syslog.process_id";
const SYSLOG_MSG_ID: &str = "syslog.msg_id";
const SYSLOG_STRUCTURED_DATA: &str = "syslog.structured_data";
const SYSLOG_MESSAGE: &str = "syslog.message";

// Attribute key constants for RFC3164 messages
const SYSLOG_TAG: &str = "syslog.tag";
const SYSLOG_CONTENT: &str = "syslog.content";

// Attribute key constants for CEF messages
const CEF_VERSION: &str = "cef.version";
const CEF_DEVICE_VENDOR: &str = "cef.device_vendor";
const CEF_DEVICE_PRODUCT: &str = "cef.device_product";
const CEF_DEVICE_VERSION: &str = "cef.device_version";
const CEF_DEVICE_EVENT_CLASS_ID: &str = "cef.device_event_class_id";
const CEF_NAME: &str = "cef.name";
const CEF_SEVERITY: &str = "cef.severity";

/// Enum to represent different parsed message types
#[derive(Debug, Clone, PartialEq)]
pub enum ParsedSyslogMessage<'a> {
    /// RFC 5424 formatted message
    Rfc5424(Rfc5424Message<'a>),
    /// RFC 3164 formatted message
    Rfc3164(Rfc3164Message<'a>),
    /// Raw CEF message (without syslog header)
    Cef(CefMessage<'a>),
    /// CEF message with RFC 3164 syslog header
    CefWithRfc3164(Rfc3164Message<'a>, CefMessage<'a>),
    /// CEF message with RFC 5424 syslog header
    CefWithRfc5424(Rfc5424Message<'a>, CefMessage<'a>),
}

impl ParsedSyslogMessage<'_> {
    /// Returns the original input received by the receiver
    pub(crate) fn input(&self) -> Cow<'_, str> {
        match self {
            ParsedSyslogMessage::Rfc5424(msg) => String::from_utf8_lossy(msg.input),
            ParsedSyslogMessage::Rfc3164(msg) => String::from_utf8_lossy(msg.input),
            ParsedSyslogMessage::Cef(msg) => String::from_utf8_lossy(msg.input),
            ParsedSyslogMessage::CefWithRfc3164(msg, _) => String::from_utf8_lossy(msg.input),
            ParsedSyslogMessage::CefWithRfc5424(msg, _) => String::from_utf8_lossy(msg.input),
        }
    }

    /// Returns the time when the event occurred.
    // Value is UNIX Epoch time in nanoseconds since 00:00:00 UTC on 1 January 1970.
    pub(crate) fn timestamp(&self) -> Option<u64> {
        match self {
            ParsedSyslogMessage::Rfc5424(msg) | ParsedSyslogMessage::CefWithRfc5424(msg, _) => {
                msg.timestamp.and_then(|ts| {
                    std::str::from_utf8(ts).ok().and_then(|timestamp_str| {
                        // RFC 5424 timestamps are in ISO 8601 format (e.g., "2003-10-11T22:14:15.003Z")
                        // Try to parse as RFC 3339 (ISO 8601)
                        DateTime::parse_from_rfc3339(timestamp_str)
                            .ok()
                            .map(|dt| dt.timestamp_nanos_opt().unwrap_or(0) as u64)
                    })
                })
            }
            ParsedSyslogMessage::Rfc3164(msg) | ParsedSyslogMessage::CefWithRfc3164(msg, _) => {
                msg.timestamp.and_then(|ts| {
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
                })
            }
            ParsedSyslogMessage::Cef(_) => None,
        }
    }

    /// Returns the severity level of the log message.
    pub(crate) fn severity(&self) -> Option<(i32, &str)> {
        match self {
            ParsedSyslogMessage::Rfc5424(msg) | ParsedSyslogMessage::CefWithRfc5424(msg, _) => {
                Some(Self::to_otel_severity(msg.priority.severity))
            }
            ParsedSyslogMessage::Rfc3164(msg) | ParsedSyslogMessage::CefWithRfc3164(msg, _) => msg
                .priority
                .as_ref()
                .map(|p| Self::to_otel_severity(p.severity)),
            ParsedSyslogMessage::Cef(_) => None,
        }
    }

    /// Adds attributes to the log record attributes Arrow record batch.
    #[must_use]
    pub(crate) fn add_attributes_to_arrow(
        &self,
        log_attributes_arrow_records: &mut StrKeysAttributesRecordBatchBuilder<u16>,
    ) -> u16 {
        let mut attributes_count = 0;

        match self {
            ParsedSyslogMessage::CefWithRfc5424(syslog_msg, cef_msg) => {
                // Add syslog RFC5424 attributes
                attributes_count +=
                    self.add_rfc5424_attributes(syslog_msg, log_attributes_arrow_records);
                // Add CEF attributes
                attributes_count += self.add_cef_attributes(cef_msg, log_attributes_arrow_records);
                attributes_count
            }
            ParsedSyslogMessage::CefWithRfc3164(syslog_msg, cef_msg) => {
                // Add syslog RFC3164 attributes
                attributes_count +=
                    self.add_rfc3164_attributes(syslog_msg, log_attributes_arrow_records);
                // Add CEF attributes
                attributes_count += self.add_cef_attributes(cef_msg, log_attributes_arrow_records);
                attributes_count
            }
            ParsedSyslogMessage::Rfc5424(msg) => {
                self.add_rfc5424_attributes(msg, log_attributes_arrow_records)
            }
            ParsedSyslogMessage::Rfc3164(msg) => {
                self.add_rfc3164_attributes(msg, log_attributes_arrow_records)
            }
            ParsedSyslogMessage::Cef(msg) => {
                self.add_cef_attributes(msg, log_attributes_arrow_records)
            }
        }
    }

    // Extract the attribute adding logic into helper methods to avoid duplication
    fn add_rfc5424_attributes(
        &self,
        msg: &Rfc5424Message<'_>,
        log_attributes_arrow_records: &mut StrKeysAttributesRecordBatchBuilder<u16>,
    ) -> u16 {
        let mut attributes_count = 3; // version, facility, and severity are always present

        log_attributes_arrow_records.append_key(SYSLOG_VERSION);
        log_attributes_arrow_records
            .any_values_builder
            .append_int(msg.version.into());

        log_attributes_arrow_records.append_key(SYSLOG_FACILITY);
        log_attributes_arrow_records
            .any_values_builder
            .append_int(msg.priority.facility.into());

        log_attributes_arrow_records.append_key(SYSLOG_SEVERITY);
        log_attributes_arrow_records
            .any_values_builder
            .append_int(msg.priority.severity.into());

        if let Some(hostname) = msg.hostname {
            log_attributes_arrow_records.append_key(SYSLOG_HOST_NAME);
            log_attributes_arrow_records
                .any_values_builder
                .append_str(hostname);
            attributes_count += 1;
        }

        if let Some(appname) = msg.app_name {
            log_attributes_arrow_records.append_key(SYSLOG_APP_NAME);
            log_attributes_arrow_records
                .any_values_builder
                .append_str(appname);
            attributes_count += 1;
        }

        if let Some(proc_id) = msg.proc_id {
            log_attributes_arrow_records.append_key(SYSLOG_PROCESS_ID);
            log_attributes_arrow_records.any_values_builder.append_int(
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
                .any_values_builder
                .append_str(msg_id);
            attributes_count += 1;
        }

        if let Some(structured_data) = &msg.structured_data {
            log_attributes_arrow_records.append_key(SYSLOG_STRUCTURED_DATA);
            log_attributes_arrow_records
                .any_values_builder
                .append_str(structured_data);
            attributes_count += 1;
        }

        if let Some(message) = msg.message {
            log_attributes_arrow_records.append_key(SYSLOG_MESSAGE);
            log_attributes_arrow_records
                .any_values_builder
                .append_str(message);
            attributes_count += 1;
        }

        attributes_count
    }

    fn add_rfc3164_attributes(
        &self,
        msg: &Rfc3164Message<'_>,
        log_attributes_arrow_records: &mut StrKeysAttributesRecordBatchBuilder<u16>,
    ) -> u16 {
        let mut attributes_count = 0;

        // Only add facility and severity if they were present in the original message
        if let Some(priority) = msg.priority.as_ref() {
            log_attributes_arrow_records.append_key(SYSLOG_FACILITY);
            log_attributes_arrow_records
                .any_values_builder
                .append_int(priority.facility.into());
            attributes_count += 1;

            log_attributes_arrow_records.append_key(SYSLOG_SEVERITY);
            log_attributes_arrow_records
                .any_values_builder
                .append_int(priority.severity.into());
            attributes_count += 1;
        }

        if let Some(hostname) = msg.hostname {
            log_attributes_arrow_records.append_key(SYSLOG_HOST_NAME);
            log_attributes_arrow_records
                .any_values_builder
                .append_str(hostname);
            attributes_count += 1;
        }

        if let Some(tag) = msg.tag {
            log_attributes_arrow_records.append_key(SYSLOG_TAG);
            log_attributes_arrow_records
                .any_values_builder
                .append_str(tag);
            attributes_count += 1;
        }

        if let Some(content) = msg.content {
            log_attributes_arrow_records.append_key(SYSLOG_CONTENT);
            log_attributes_arrow_records
                .any_values_builder
                .append_str(content);
            attributes_count += 1;
        }

        attributes_count
    }

    fn add_cef_attributes(
        &self,
        msg: &CefMessage<'_>,
        log_attributes_arrow_records: &mut StrKeysAttributesRecordBatchBuilder<u16>,
    ) -> u16 {
        let mut attributes_count = 7; // version, device_vendor, device_product, device_version, device_event_class_id, name, and severity are always present

        log_attributes_arrow_records.append_key(CEF_VERSION);
        log_attributes_arrow_records
            .any_values_builder
            .append_int(msg.version.into());

        log_attributes_arrow_records.append_key(CEF_DEVICE_VENDOR);
        log_attributes_arrow_records
            .any_values_builder
            .append_str(msg.device_vendor);

        log_attributes_arrow_records.append_key(CEF_DEVICE_PRODUCT);
        log_attributes_arrow_records
            .any_values_builder
            .append_str(msg.device_product);

        log_attributes_arrow_records.append_key(CEF_DEVICE_VERSION);
        log_attributes_arrow_records
            .any_values_builder
            .append_str(msg.device_version);

        log_attributes_arrow_records.append_key(CEF_DEVICE_EVENT_CLASS_ID);
        log_attributes_arrow_records
            .any_values_builder
            .append_str(msg.device_event_class_id);

        log_attributes_arrow_records.append_key(CEF_NAME);
        log_attributes_arrow_records
            .any_values_builder
            .append_str(msg.name);

        log_attributes_arrow_records.append_key(CEF_SEVERITY);
        log_attributes_arrow_records
            .any_values_builder
            .append_str(msg.severity);

        let mut extensions_iter = msg.parse_extensions();
        while let Some((key, value)) = extensions_iter.next_extension() {
            log_attributes_arrow_records.append_key(std::str::from_utf8(key).unwrap_or_default());
            log_attributes_arrow_records
                .any_values_builder
                .append_str(value);
            attributes_count += 1;
        }

        attributes_count
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
    use crate::syslog_cef_receiver::parser::parse;

    #[test]
    fn test_parsed_syslog_message_timestamp_rfc5424() {
        let input = b"<34>1 2003-10-11T22:14:15.003Z mymachine.example.com su - ID47 - 'su root' failed for lonvick on /dev/pts/8";
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

    #[test]
    fn test_cef_with_rfc5424_header() {
        // Test CEF message embedded in RFC 5424 syslog
        let input = b"<134>1 2024-10-09T12:34:56.789Z firewall.example.com CEF - - CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232";
        let result = parse(input).unwrap();

        match result {
            ParsedSyslogMessage::CefWithRfc5424(syslog, cef) => {
                // Verify all syslog header fields
                assert_eq!(syslog.priority.facility, 16); // 134 >> 3
                assert_eq!(syslog.priority.severity, 6); // 134 & 0x07
                assert_eq!(syslog.version, 1);
                assert_eq!(syslog.hostname, Some(&b"firewall.example.com"[..]));
                assert_eq!(syslog.app_name, Some(&b"CEF"[..]));
                assert_eq!(syslog.proc_id, None); // Should be None for "-"
                assert_eq!(syslog.msg_id, None); // Should be None for "-"
                assert_eq!(syslog.structured_data, None); // No structured data in this message

                // Verify timestamp
                assert!(syslog.timestamp.is_some());
                assert_eq!(syslog.timestamp, Some(&b"2024-10-09T12:34:56.789Z"[..]));

                // Verify the message field contains the exact CEF message
                assert!(syslog.message.is_some());
                assert_eq!(syslog.message, Some(&b"CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232"[..]));

                // Verify all CEF fields
                assert_eq!(cef.version, 0);
                assert_eq!(cef.device_vendor, &b"Security"[..]);
                assert_eq!(cef.device_product, &b"threatmanager"[..]);
                assert_eq!(cef.device_version, &b"1.0"[..]);
                assert_eq!(cef.device_event_class_id, &b"100"[..]);
                assert_eq!(cef.name, &b"worm successfully stopped"[..]);
                assert_eq!(cef.severity, &b"10"[..]);

                // Verify extensions using collect_all()
                let extensions = cef.parse_extensions().collect_all();
                assert_eq!(extensions.len(), 3);
                assert_eq!(extensions[0].0.as_slice(), b"src");
                assert_eq!(extensions[0].1.as_slice(), b"10.0.0.1");
                assert_eq!(extensions[1].0.as_slice(), b"dst");
                assert_eq!(extensions[1].1.as_slice(), b"2.1.2.2");
                assert_eq!(extensions[2].0.as_slice(), b"spt");
                assert_eq!(extensions[2].1.as_slice(), b"1232");

                // Verify input field is preserved
                assert_eq!(syslog.input, input);
                assert_eq!(cef.input, syslog.message.unwrap());
            }
            _ => panic!("Expected CefWithRfc5424, got {:?}", result),
        }
    }

    #[test]
    fn test_cef_with_rfc3164_header() {
        // Test CEF message embedded in RFC 3164 syslog
        let input = b"<34>Oct 11 22:14:15 firewall CEF: CEF:0|Vendor|Product|2.0|signature-123|Intrusion detected|7|act=blocked src=192.168.1.100";
        let result = parse(input).unwrap();

        match result {
            ParsedSyslogMessage::CefWithRfc3164(syslog, cef) => {
                // Verify all syslog header fields
                assert!(syslog.priority.is_some());
                assert_eq!(syslog.priority.as_ref().unwrap().facility, 4); // 34 >> 3
                assert_eq!(syslog.priority.as_ref().unwrap().severity, 2); // 34 & 0x07

                // Verify timestamp
                assert!(syslog.timestamp.is_some());
                assert_eq!(syslog.timestamp, Some(&b"Oct 11 22:14:15"[..]));

                // Verify hostname and tag
                assert_eq!(syslog.hostname, Some(&b"firewall"[..]));
                assert_eq!(syslog.tag, Some(&b"CEF"[..]));

                // Verify content contains the exact CEF message
                assert!(syslog.content.is_some());
                assert_eq!(syslog.content, Some(&b"CEF:0|Vendor|Product|2.0|signature-123|Intrusion detected|7|act=blocked src=192.168.1.100"[..]));

                // Verify all CEF fields
                assert_eq!(cef.version, 0);
                assert_eq!(cef.device_vendor, &b"Vendor"[..]);
                assert_eq!(cef.device_product, &b"Product"[..]);
                assert_eq!(cef.device_version, &b"2.0"[..]);
                assert_eq!(cef.device_event_class_id, &b"signature-123"[..]);
                assert_eq!(cef.name, &b"Intrusion detected"[..]);
                assert_eq!(cef.severity, &b"7"[..]);

                // Verify extensions using collect_all()
                let extensions = cef.parse_extensions().collect_all();
                assert_eq!(extensions.len(), 2);
                assert_eq!(extensions[0].0.as_slice(), b"act");
                assert_eq!(extensions[0].1.as_slice(), b"blocked");
                assert_eq!(extensions[1].0.as_slice(), b"src");
                assert_eq!(extensions[1].1.as_slice(), b"192.168.1.100");

                // Verify input field is preserved
                assert_eq!(syslog.input, input);
                assert_eq!(cef.input, syslog.content.unwrap());
            }
            _ => panic!("Expected CefWithRfc3164, got {:?}", result),
        }
    }

    #[test]
    fn test_cef_with_rfc3164_header_no_priority() {
        // Test CEF message embedded in RFC 3164 syslog without priority
        // This is a valid format according to CEF specification
        let input = b"Sep 29 08:26:10 host CEF:1|Security|threatmanager|1.0|100|worm successfully stopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232";
        let result = parse(input).unwrap();

        match result {
            ParsedSyslogMessage::CefWithRfc3164(syslog, cef) => {
                // Verify syslog header fields
                assert!(syslog.priority.is_none()); // No priority in this format

                // Verify timestamp
                assert!(syslog.timestamp.is_some());
                assert_eq!(syslog.timestamp, Some(&b"Sep 29 08:26:10"[..]));

                // Verify hostname and tag (parser extracts "CEF" as tag from "CEF:1|...")
                assert_eq!(syslog.hostname, Some(&b"host"[..]));
                assert_eq!(syslog.tag, Some(&b"CEF"[..])); // Tag should be "CEF"

                // Verify content contains the exact CEF message
                assert!(syslog.content.is_some());
                assert_eq!(syslog.content, Some(&b"CEF:1|Security|threatmanager|1.0|100|worm successfully stopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232"[..]));

                // Verify all CEF fields
                assert_eq!(cef.version, 1); // Note: CEF version 1 in this example
                assert_eq!(cef.device_vendor, &b"Security"[..]);
                assert_eq!(cef.device_product, &b"threatmanager"[..]);
                assert_eq!(cef.device_version, &b"1.0"[..]);
                assert_eq!(cef.device_event_class_id, &b"100"[..]);
                assert_eq!(cef.name, &b"worm successfully stopped"[..]);
                assert_eq!(cef.severity, &b"10"[..]);

                // Verify extensions using collect_all()
                let extensions = cef.parse_extensions().collect_all();
                assert_eq!(extensions.len(), 3);
                assert_eq!(extensions[0].0.as_slice(), b"src");
                assert_eq!(extensions[0].1.as_slice(), b"10.0.0.1");
                assert_eq!(extensions[1].0.as_slice(), b"dst");
                assert_eq!(extensions[1].1.as_slice(), b"2.1.2.2");
                assert_eq!(extensions[2].0.as_slice(), b"spt");
                assert_eq!(extensions[2].1.as_slice(), b"1232");

                // Verify input field is preserved
                assert_eq!(syslog.input, input);
                assert_eq!(cef.input, syslog.content.unwrap());
            }
            _ => panic!("Expected CefWithRfc3164, got {:?}", result),
        }
    }
}
