// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::receivers::syslog_cef_receiver::parser::{
    cef::CefMessage, rfc3164::Rfc3164Message, rfc5424::Rfc5424Message,
};
use chrono::{DateTime, Datelike, Local, NaiveDate, NaiveDateTime, TimeZone};
use otel_arrow_dfe_pdata::encode::record::attributes::StrKeysAttributesRecordBatchBuilder;

// Common attribute key constants for both RFC5424 and RFC3164 messages
const SYSLOG_FACILITY: &str = "syslog.facility";
const SYSLOG_SEVERITY: &str = "syslog.severity";
const SYSLOG_HOST_NAME: &str = "syslog.host_name";

// Attribute key constants for RFC5424 messages
const SYSLOG_VERSION: &str = "syslog.version";
const SYSLOG_APP_NAME: &str = "syslog.app_name";
const SYSLOG_PROCESS_ID: &str = "syslog.process_id";
const SYSLOG_PROCESS_ID_STR: &str = "syslog.process_id_str";
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

// Attribute key constant for detected input format
const INPUT_FORMAT: &str = "input.format";

/// Decodes a fixed-width ASCII field without allocating or accepting signs.
#[inline]
fn parse_two_digits(input: &[u8]) -> Option<u32> {
    let [tens, ones] = input else {
        return None;
    };
    if !tens.is_ascii_digit() || !ones.is_ascii_digit() {
        return None;
    }

    Some(u32::from(tens - b'0') * 10 + u32::from(ones - b'0'))
}

/// Fast path for the usual 15-byte RFC 3164 timestamp, with calendar validation.
/// Noncanonical inputs and leap seconds retain the legacy parser as a fallback.
#[inline]
fn parse_rfc3164_naive_timestamp(input: &[u8], year: i32) -> Option<NaiveDateTime> {
    if input.len() != 15
        || input[3] != b' '
        || input[6] != b' '
        || input[9] != b':'
        || input[12] != b':'
    {
        return None;
    }

    let month = match &input[..3] {
        b"Jan" => 1,
        b"Feb" => 2,
        b"Mar" => 3,
        b"Apr" => 4,
        b"May" => 5,
        b"Jun" => 6,
        b"Jul" => 7,
        b"Aug" => 8,
        b"Sep" => 9,
        b"Oct" => 10,
        b"Nov" => 11,
        b"Dec" => 12,
        _ => return None,
    };

    let day = match input[4] {
        b' ' => {
            let digit = input[5];
            if !digit.is_ascii_digit() {
                return None;
            }
            u32::from(digit - b'0')
        }
        b'0'..=b'9' => parse_two_digits(&input[4..6])?,
        _ => return None,
    };
    let hour = parse_two_digits(&input[7..9])?;
    let minute = parse_two_digits(&input[10..12])?;
    let second = parse_two_digits(&input[13..15])?;

    NaiveDate::from_ymd_opt(year, month, day)?.and_hms_opt(hour, minute, second)
}

/// Preserves Chrono's acceptance of uncommon forms such as mixed-case months,
/// whitespace-padded time fields, and leap seconds without penalizing normal input.
#[cold]
#[inline(never)]
fn parse_rfc3164_naive_timestamp_fallback(input: &[u8], year: i32) -> Option<NaiveDateTime> {
    let timestamp = std::str::from_utf8(input).ok()?;
    NaiveDateTime::parse_from_str(&format!("{year} {timestamp}"), "%Y %b %d %H:%M:%S").ok()
}

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
    /// Returns whether the message was fully parsed with all expected structure intact.
    ///
    /// This is used to determine whether to set the log body:
    /// - If fully parsed, body is left empty (all data is in attributes)
    /// - If not fully parsed, body contains the original input for debugging
    ///
    /// A message is considered "fully parsed" when:
    /// - RFC 5424: Always true if parsing succeeded (required fields must be present)
    /// - RFC 3164: True if priority was successfully parsed (indicates standard syslog format)
    /// - CEF: Always true if parsing succeeded (all 7 required headers must be present)
    /// - Combined formats: Both components must be fully parsed
    pub(crate) const fn is_fully_parsed(&self) -> bool {
        match self {
            // RFC 5424 parsing requires priority and version, so if Ok it's fully parsed
            ParsedSyslogMessage::Rfc5424(_) => true,
            // RFC 3164 is fully parsed only if priority was successfully extracted
            ParsedSyslogMessage::Rfc3164(msg) => msg.priority.is_some(),
            // CEF parsing requires all 7 header fields, so if Ok it's fully parsed
            ParsedSyslogMessage::Cef(_) => true,
            // Combined: RFC 3164 must have priority for it to be fully parsed
            ParsedSyslogMessage::CefWithRfc3164(msg, _) => msg.priority.is_some(),
            // Combined: RFC 5424 is always fully parsed if Ok
            ParsedSyslogMessage::CefWithRfc5424(_, _) => true,
        }
    }

    /// Returns the original input received by the receiver
    pub(crate) const fn input(&self) -> &[u8] {
        match self {
            ParsedSyslogMessage::Rfc5424(msg) => msg.input,
            ParsedSyslogMessage::Rfc3164(msg) => msg.input,
            ParsedSyslogMessage::Cef(msg) => msg.input,
            ParsedSyslogMessage::CefWithRfc3164(msg, _) => msg.input,
            ParsedSyslogMessage::CefWithRfc5424(msg, _) => msg.input,
        }
    }

    /// Returns the detected input format as a string.
    ///
    /// This is emitted as the `input.format` log attribute so that downstream
    /// processors can filter or route records by format.
    pub(crate) const fn format(&self) -> &'static str {
        match self {
            ParsedSyslogMessage::Rfc5424(_) => "rfc5424",
            ParsedSyslogMessage::Rfc3164(_) => "rfc3164",
            ParsedSyslogMessage::Cef(_) => "cef",
            ParsedSyslogMessage::CefWithRfc3164(_, _) => "cef_rfc3164",
            ParsedSyslogMessage::CefWithRfc5424(_, _) => "cef_rfc5424",
        }
    }

    /// Returns the time when the event occurred.
    // Value is UNIX Epoch time in nanoseconds since 00:00:00 UTC on 1 January 1970.
    pub(crate) fn timestamp(&self) -> Option<u64> {
        self.timestamp_inner(None)
    }

    /// Uses the receiver's local year for yearless RFC 3164 timestamps.
    /// RFC 5424 and raw CEF keep their existing timestamp semantics.
    pub(crate) fn timestamp_with_year(&self, received_local_year: i32) -> Option<u64> {
        self.timestamp_inner(Some(received_local_year))
    }

    fn timestamp_inner(&self, received_local_year: Option<i32>) -> Option<u64> {
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
                    // Encoded messages retain the receipt year. Only the eager
                    // compatibility path infers it from the clock here. Resolve
                    // local time for the event date, preserving DST behavior.
                    let year = received_local_year.unwrap_or_else(|| Local::now().year());
                    let naive = parse_rfc3164_naive_timestamp(ts, year)
                        .or_else(|| parse_rfc3164_naive_timestamp_fallback(ts, year))?;
                    Local
                        .from_local_datetime(&naive)
                        .single()
                        .map(|local| local.timestamp_nanos_opt().unwrap_or(0) as u64)
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
        let mut attributes_count: u16 = match self {
            ParsedSyslogMessage::CefWithRfc5424(syslog_msg, cef_msg) => {
                // Add syslog RFC5424 attributes
                let mut count =
                    self.add_rfc5424_attributes(syslog_msg, log_attributes_arrow_records);
                // Add CEF attributes
                count += self.add_cef_attributes(cef_msg, log_attributes_arrow_records);
                count
            }
            ParsedSyslogMessage::CefWithRfc3164(syslog_msg, cef_msg) => {
                // Add syslog RFC3164 attributes
                let mut count =
                    self.add_rfc3164_attributes(syslog_msg, log_attributes_arrow_records);
                // Add CEF attributes
                count += self.add_cef_attributes(cef_msg, log_attributes_arrow_records);
                count
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
        };

        // Always emit the detected input format attribute
        log_attributes_arrow_records.append_key(INPUT_FORMAT);
        log_attributes_arrow_records
            .any_values_builder
            .append_str(self.format().as_bytes());
        attributes_count += 1;

        attributes_count
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
            // Always store the original string value (per RFC 5424: PROCID = 1*128PRINTUSASCII)
            log_attributes_arrow_records.append_key(SYSLOG_PROCESS_ID_STR);
            log_attributes_arrow_records
                .any_values_builder
                .append_str(proc_id);
            attributes_count += 1;

            // Additionally store as integer if parseable
            if let Ok(proc_id_int) = std::str::from_utf8(proc_id)
                .unwrap_or_default()
                .parse::<i64>()
            {
                log_attributes_arrow_records.append_key(SYSLOG_PROCESS_ID);
                log_attributes_arrow_records
                    .any_values_builder
                    .append_int(proc_id_int);
                attributes_count += 1;
            }
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

        if let Some(app_name) = msg.app_name {
            log_attributes_arrow_records.append_key(SYSLOG_APP_NAME);
            log_attributes_arrow_records
                .any_values_builder
                .append_str(app_name);
            attributes_count += 1;
        }

        if let Some(proc_id) = msg.proc_id {
            // RFC 3164 proc_id is always numeric (parser filters to numeric-only),
            // so we store only as integer
            log_attributes_arrow_records.append_key(SYSLOG_PROCESS_ID);
            log_attributes_arrow_records.any_values_builder.append_int(
                std::str::from_utf8(proc_id)
                    .unwrap_or_default()
                    .parse::<i64>()
                    .unwrap_or(0),
            );
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
    use crate::receivers::syslog_cef_receiver::parser::parse;

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

    /// Scenario: A plain RFC 3164 message contains a valid fixed-format timestamp.
    /// Guarantees: Timestamp conversion uses the current year and local timezone.
    #[test]
    fn test_parsed_syslog_message_timestamp_rfc3164() {
        let input = b"<34>Oct 11 22:14:15 mymachine su: 'su root' failed for lonvick on /dev/pts/8";
        let result = parse(input).unwrap();

        let timestamp_nanos = result.timestamp().unwrap();
        let current_year = Local::now().year();
        let expected_naive = NaiveDate::from_ymd_opt(current_year, 10, 11)
            .unwrap()
            .and_hms_opt(22, 14, 15)
            .unwrap();
        let expected_nanos = Local
            .from_local_datetime(&expected_naive)
            .single()
            .unwrap()
            .timestamp_nanos_opt()
            .unwrap() as u64;
        assert_eq!(timestamp_nanos, expected_nanos);
    }

    /// Scenario: RFC 3164 dates use supported day padding and every month abbreviation.
    /// Guarantees: The direct decoder accepts space-padded, zero-padded, and two-digit days.
    #[test]
    fn test_parse_rfc3164_naive_timestamp_accepts_days_and_months() {
        let day_cases: &[(&[u8], u32)] = &[
            (b"Jan  1 00:00:00", 1),
            (b"Jan 01 00:00:00", 1),
            (b"Jan 31 00:00:00", 31),
        ];
        for (input, expected_day) in day_cases {
            let parsed = parse_rfc3164_naive_timestamp(input, 2024).unwrap();
            assert_eq!(parsed.date().day(), *expected_day);
        }

        let month_cases: &[(&[u8], u32)] = &[
            (b"Jan 01 00:00:00", 1),
            (b"Feb 01 00:00:00", 2),
            (b"Mar 01 00:00:00", 3),
            (b"Apr 01 00:00:00", 4),
            (b"May 01 00:00:00", 5),
            (b"Jun 01 00:00:00", 6),
            (b"Jul 01 00:00:00", 7),
            (b"Aug 01 00:00:00", 8),
            (b"Sep 01 00:00:00", 9),
            (b"Oct 01 00:00:00", 10),
            (b"Nov 01 00:00:00", 11),
            (b"Dec 01 00:00:00", 12),
        ];
        for (input, expected_month) in month_cases {
            let parsed = parse_rfc3164_naive_timestamp(input, 2024).unwrap();
            assert_eq!(parsed.date().month(), *expected_month);
        }
    }

    /// Scenario: RFC 3164 input contains leap dates and out-of-range calendar days.
    /// Guarantees: The direct decoder applies Gregorian calendar validation for the supplied year.
    #[test]
    fn test_parse_rfc3164_naive_timestamp_validates_calendar_dates() {
        assert!(parse_rfc3164_naive_timestamp(b"Feb 29 00:00:00", 2024).is_some());
        assert!(parse_rfc3164_naive_timestamp(b"Feb 29 00:00:00", 2023).is_none());
        assert!(parse_rfc3164_naive_timestamp(b"Jan 00 00:00:00", 2024).is_none());
        assert!(parse_rfc3164_naive_timestamp(b"Jan 32 00:00:00", 2024).is_none());
        assert!(parse_rfc3164_naive_timestamp(b"Apr 31 00:00:00", 2024).is_none());
    }

    /// Scenario: RFC 3164 input has invalid length, fields, digits, separators, or time values.
    /// Guarantees: Malformed fixed-format timestamps return `None` without accepting partial input.
    #[test]
    fn test_parse_rfc3164_naive_timestamp_rejects_malformed_input() {
        let invalid: &[&[u8]] = &[
            b"",
            b"Jan 01 00:00:00x",
            b"Jan 01 00:00:0",
            b"J\xffn 01 00:00:00",
            b"Jan  \xff 00:00:00",
            b"Jan 01 00:00:\xff0",
            b"Jax 01 00:00:00",
            b"Jan-01 00:00:00",
            b"Jan 01-00:00:00",
            b"Jan 01 00-00:00",
            b"Jan 01 00:00-00",
            b"Jan  x 00:00:00",
            b"Jan 0x 00:00:00",
            b"Jan 01 0x:00:00",
            b"Jan 01 00:0x:00",
            b"Jan 01 00:00:0x",
            b"Jan 01 24:00:00",
            b"Jan 01 00:60:00",
            b"Jan 01 00:00:60",
        ];
        for input in invalid {
            assert!(
                parse_rfc3164_naive_timestamp(input, 2024).is_none(),
                "accepted malformed timestamp: {input:?}"
            );
        }
    }

    /// Scenario: An RFC 3164 message has timestamp-shaped bytes with an invalid month.
    /// Guarantees: Message parsing still succeeds while timestamp conversion returns `None`.
    #[test]
    fn test_malformed_rfc3164_timestamp_does_not_reject_message() {
        let result = parse(b"<34>Jax 01 00:00:00 host tag: message").unwrap();

        assert!(matches!(result, ParsedSyslogMessage::Rfc3164(_)));
        assert_eq!(result.timestamp(), None);
    }

    /// Scenario: A CEF event is wrapped by an RFC 3164 syslog header.
    /// Guarantees: The combined representation uses the same local timestamp conversion as RFC 3164.
    #[test]
    fn test_parsed_syslog_message_timestamp_cef_with_rfc3164() {
        let input = b"<134>Oct 11 22:14:15 host CEF:0|Security|threatmanager|1.0|100|test|10|";
        let result = parse(input).unwrap();
        assert!(matches!(result, ParsedSyslogMessage::CefWithRfc3164(_, _)));

        let current_year = Local::now().year();
        let expected_naive = NaiveDate::from_ymd_opt(current_year, 10, 11)
            .unwrap()
            .and_hms_opt(22, 14, 15)
            .unwrap();
        let expected_nanos = Local
            .from_local_datetime(&expected_naive)
            .single()
            .unwrap()
            .timestamp_nanos_opt()
            .unwrap() as u64;
        assert_eq!(result.timestamp(), Some(expected_nanos));
    }

    /// Scenario: RFC 3164 and wrapped CEF contain normal, unusual, or malformed timestamps.
    /// Guarantees: The fast path and fallback preserve the former Chrono timestamp results.
    #[test]
    fn test_rfc3164_timestamp_preserves_legacy_acceptance() {
        let cases: &[(&[u8], bool)] = &[
            (b"Jan 01 01:02:03", true),
            (b"jAn 01 01:02:03", true),
            (b"JAN 01 01:02:03", true),
            (b"Jan \t1 01:02:03", true),
            (b"Jan  1  1: 2: 3", true),
            (b"Jun 30 23:59:60", true),
            (b"Jan 01 01:02:61", false),
            (b"Jan 00 01:02:03", false),
            (b"Jax 01 01:02:03", false),
            (b"J\xffn 01 01:02:03", false),
        ];
        let year = Local::now().year();
        for &(timestamp, accepted) in cases {
            let naive = std::str::from_utf8(timestamp).ok().and_then(|text| {
                NaiveDateTime::parse_from_str(&format!("{year} {text}"), "%Y %b %d %H:%M:%S").ok()
            });
            assert_eq!(
                naive.is_some(),
                accepted,
                "legacy acceptance: {timestamp:?}"
            );
            let expected = naive
                .and_then(|dt| Local.from_local_datetime(&dt).single())
                .map(|dt| dt.timestamp_nanos_opt().unwrap_or(0) as u64);
            for body in ["tag: message", "CEF:0|Security|product|1.0|100|test|10|"] {
                let mut input = b"<34>".to_vec();
                input.extend_from_slice(timestamp);
                input.extend_from_slice(format!(" host {body}").as_bytes());
                let parsed = parse(&input).expect("timestamp failure must not reject a message");
                assert_eq!(parsed.timestamp(), expected, "timestamp: {timestamp:?}");
            }
        }
    }

    /// Scenario: Deferred RFC 3164 and wrapped CEF decoding infers a year for February 29.
    /// Guarantees: Canonical and mixed-case dates use the receipt year for leap-day validation.
    #[test]
    fn test_receipt_year_controls_leap_day_validation() {
        let expected = Local
            .with_ymd_and_hms(2024, 2, 29, 12, 0, 0)
            .single()
            .unwrap()
            .timestamp_nanos_opt()
            .unwrap() as u64;
        for month in ["Feb", "fEb"] {
            for body in ["tag: message", "CEF:0|Security|product|1.0|100|test|10|"] {
                let input = format!("<34>{month} 29 12:00:00 host {body}");
                let parsed = parse(input.as_bytes()).unwrap();
                assert_eq!(parsed.timestamp_with_year(2024), Some(expected));
                assert_eq!(parsed.timestamp_with_year(2023), None);
            }
        }
    }

    /// Scenario: RFC 3164 and wrapped CEF timestamps straddle local DST transitions.
    /// Guarantees: Gaps and ambiguous times have no timestamp; adjacent times resolve normally.
    #[cfg(unix)]
    #[test]
    fn test_rfc3164_local_dst_transitions() {
        use chrono::NaiveDate;

        const CHILD: &str = "OTEL_SYSLOG_DST_TEST_CHILD";
        if std::env::var_os(CHILD).is_none() {
            // A subprocess avoids mutating process-wide TZ while other tests run.
            let module = module_path!()
                .split_once("::")
                .expect("crate-qualified module")
                .1;
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .arg("--exact")
                .arg(format!("{module}::test_rfc3164_local_dst_transitions"))
                .env(CHILD, "1")
                .env("TZ", "EST5EDT,M3.2.0/2,M11.1.0/2")
                .output()
                .expect("run isolated local-timezone test");
            assert!(output.status.success(), "child failed: {output:?}");
            assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
            return;
        }

        let year = Local::now().year();
        let sunday = |month: u32, week: u32| {
            let first = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
            let offset = (7 - first.weekday().num_days_from_sunday()) % 7 + 7 * (week - 1);
            first + chrono::Duration::days(i64::from(offset))
        };
        let spring = sunday(3, 2);
        let fall = sunday(11, 1);
        let gap = spring.and_hms_opt(2, 30, 0).unwrap();
        let ambiguous = fall.and_hms_opt(1, 30, 0).unwrap();
        assert!(matches!(
            Local.from_local_datetime(&gap),
            chrono::LocalResult::None
        ));
        assert!(matches!(
            Local.from_local_datetime(&ambiguous),
            chrono::LocalResult::Ambiguous(_, _)
        ));
        for (date, hour, resolved) in [
            (spring, 1, true),
            (spring, 2, false),
            (spring, 3, true),
            (fall, 0, true),
            (fall, 1, false),
            (fall, 2, true),
        ] {
            let naive = date.and_hms_opt(hour, 30, 0).unwrap();
            let expected = Local
                .from_local_datetime(&naive)
                .single()
                .map(|dt| dt.timestamp_nanos_opt().unwrap() as u64);
            assert_eq!(expected.is_some(), resolved);
            for body in ["tag: message", "CEF:0|Security|product|1.0|100|test|10|"] {
                let input = format!("<34>{} host {body}", naive.format("%b %e %H:%M:%S"));
                let parsed = parse(input.as_bytes()).unwrap();
                assert_eq!(parsed.timestamp(), expected, "local timestamp: {naive}");
                assert_eq!(
                    parsed.timestamp_with_year(year),
                    expected,
                    "receipt timestamp: {naive}"
                );
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

        let input_bytes = result.input();
        assert_eq!(
            input_bytes,
            b"<34>1 2003-10-11T22:14:15.003Z host app - - - Test message"
        );
    }

    #[test]
    fn test_is_fully_parsed() {
        // RFC 5424 with valid priority should be fully parsed
        let input = b"<34>1 2003-10-11T22:14:15.003Z host app - - - Test message";
        let result = parse(input).unwrap();
        assert!(result.is_fully_parsed(), "RFC 5424 should be fully parsed");

        // RFC 3164 with valid priority should be fully parsed
        let input = b"<34>Oct 11 22:14:15 mymachine su: 'su root' failed";
        let result = parse(input).unwrap();
        assert!(
            result.is_fully_parsed(),
            "RFC 3164 with priority should be fully parsed"
        );

        // RFC 3164 without priority (no "<") should NOT be fully parsed
        let input = b"Oct 11 22:14:15 mymachine su: 'su root' failed";
        let result = parse(input).unwrap();
        assert!(
            !result.is_fully_parsed(),
            "RFC 3164 without priority should NOT be fully parsed"
        );

        // RFC 3164 with invalid priority should NOT be fully parsed
        let input = b"<00>Oct 11 22:14:15 mymachine su: message";
        let result = parse(input).unwrap();
        assert!(
            !result.is_fully_parsed(),
            "RFC 3164 with invalid priority should NOT be fully parsed"
        );

        // CEF should be fully parsed
        let input = b"CEF:0|Security|threatmanager|1.0|100|test|10|";
        let result = parse(input).unwrap();
        assert!(result.is_fully_parsed(), "CEF should be fully parsed");

        // CEF with RFC 5424 header should be fully parsed
        let input = b"<134>1 2024-10-09T12:34:56.789Z host CEF - - CEF:0|Security|threatmanager|1.0|100|test|10|";
        let result = parse(input).unwrap();
        assert!(
            result.is_fully_parsed(),
            "CEF with RFC 5424 should be fully parsed"
        );

        // CEF with RFC 3164 header (with priority) should be fully parsed
        let input = b"<134>Oct 11 22:14:15 host CEF:0|Security|threatmanager|1.0|100|test|10|";
        let result = parse(input).unwrap();
        assert!(
            result.is_fully_parsed(),
            "CEF with RFC 3164 (with priority) should be fully parsed"
        );

        // CEF with RFC 3164 header (without priority) should NOT be fully parsed
        let input = b"Oct 11 22:14:15 host CEF:0|Security|threatmanager|1.0|100|test|10|";
        let result = parse(input).unwrap();
        assert!(
            !result.is_fully_parsed(),
            "CEF with RFC 3164 (without priority) should NOT be fully parsed"
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

    #[test]
    fn test_cef_with_rfc3164_priority_no_hostname() {
        // CEF message with priority but no timestamp or hostname -- the RFC 3164
        // parser should fall through to the tag/content path directly.
        let input = b"<34>CEF:0|Security|threatmanager|1.0|100|worm stopped|10|src=10.0.0.1";
        let result = parse(input).unwrap();

        match result {
            ParsedSyslogMessage::CefWithRfc3164(syslog, cef) => {
                assert!(syslog.priority.is_some());
                assert_eq!(syslog.priority.as_ref().unwrap().facility, 4);
                assert_eq!(syslog.priority.as_ref().unwrap().severity, 2);

                // No timestamp or hostname expected
                assert_eq!(syslog.timestamp, None);
                assert_eq!(syslog.hostname, None);

                // Tag should be "CEF" (RFC 3164 parser splits "CEF:0|..." at the colon)
                assert_eq!(syslog.tag, Some(&b"CEF"[..]));

                // Content should be the full CEF message
                assert!(syslog.content.is_some());
                assert!(
                    syslog.content.unwrap().starts_with(b"CEF:0|"),
                    "content should start with CEF:0|, got: {:?}",
                    std::str::from_utf8(syslog.content.unwrap())
                );

                // Verify CEF fields
                assert_eq!(cef.version, 0);
                assert_eq!(cef.device_vendor, &b"Security"[..]);
                assert_eq!(cef.device_product, &b"threatmanager"[..]);
                assert_eq!(cef.name, &b"worm stopped"[..]);

                let extensions = cef.parse_extensions().collect_all();
                assert_eq!(extensions.len(), 1);
                assert_eq!(extensions[0].0.as_slice(), b"src");
                assert_eq!(extensions[0].1.as_slice(), b"10.0.0.1");

                assert_eq!(syslog.input, input);
            }
            _ => panic!("Expected CefWithRfc3164, got {:?}", result),
        }
    }

    #[test]
    fn test_parsed_syslog_message_format() {
        // RFC 5424
        let input = b"<34>1 2003-10-11T22:14:15.003Z host app - - - Test message";
        let result = parse(input).unwrap();
        assert_eq!(result.format(), "rfc5424");

        // RFC 3164
        let input = b"<34>Oct 11 22:14:15 mymachine su: 'su root' failed";
        let result = parse(input).unwrap();
        assert_eq!(result.format(), "rfc3164");

        // Pure CEF
        let input = b"CEF:0|Security|threatmanager|1.0|100|test|10|";
        let result = parse(input).unwrap();
        assert_eq!(result.format(), "cef");

        // CEF with RFC 5424 header
        let input = b"<134>1 2024-10-09T12:34:56.789Z host CEF - - CEF:0|Security|threatmanager|1.0|100|test|10|";
        let result = parse(input).unwrap();
        assert_eq!(result.format(), "cef_rfc5424");

        // CEF with RFC 3164 header
        let input = b"<134>Oct 11 22:14:15 host CEF:0|Security|threatmanager|1.0|100|test|10|";
        let result = parse(input).unwrap();
        assert_eq!(result.format(), "cef_rfc3164");
    }
}
