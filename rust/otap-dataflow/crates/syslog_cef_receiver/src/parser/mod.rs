// SPDX-License-Identifier: Apache-2.0
// Copyright The OpenTelemetry Authors

use core::str;

pub(crate) mod cef;
pub(crate) mod parsed_message;
pub(crate) mod rfc3164;
pub(crate) mod rfc5424;

use crate::parser::cef::parse_cef;
use crate::parser::parsed_message::ParsedSyslogMessage;
use crate::parser::rfc3164::parse_rfc3164;
use crate::parser::rfc5424::parse_rfc5424;

/// Priority structure containing facility and severity
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Priority {
    pub facility: u8,
    pub severity: u8,
}

/// Error types that can occur during parsing
#[derive(Debug, Clone, PartialEq)]
pub(super) enum ParseError {
    /// Error parsing priority value
    /// RFC5424 and RFC3164 messages are expected to start with "<Priority>"
    InvalidPriority,
    /// Error parsing version number for RFC5424 messages
    InvalidVersion,
    /// Error parsing CEF message
    InvalidCef,
    /// Error parsing UTF-8 strings
    InvalidUtf8,
}

/// Parse a syslog message from bytes, automatically detecting the format
pub(super) fn parse(input: &[u8]) -> Result<ParsedSyslogMessage<'_>, ParseError> {
    // Check if it's a CEF message first
    if input.starts_with(b"CEF:") {
        return parse_cef(input).map(ParsedSyslogMessage::Cef);
    }

    // Try RFC 5424 first
    if let Ok(msg) = parse_rfc5424(input) {
        return Ok(ParsedSyslogMessage::Rfc5424(msg));
    }

    // Fallback to RFC 3164
    parse_rfc3164(input).map(ParsedSyslogMessage::Rfc3164)
}

/// Parse priority from the beginning of a syslog message
pub(super) fn parse_priority(input: &[u8]) -> Result<(Priority, &[u8]), ParseError> {
    if input.is_empty() || input[0] != b'<' {
        return Err(ParseError::InvalidPriority);
    }

    let end = input
        .iter()
        .position(|&b| b == b'>')
        .ok_or(ParseError::InvalidPriority)?;
    let priority_bytes = &input[1..end];
    let priority_str = str::from_utf8(priority_bytes).map_err(|_| ParseError::InvalidUtf8)?;
    let priority_num: u8 = priority_str
        .parse()
        .map_err(|_| ParseError::InvalidPriority)?;

    let facility = priority_num >> 3;
    let severity = priority_num & 0x07;

    Ok((Priority { facility, severity }, &input[end + 1..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_auto_detection() {
        // Test RFC 5424
        let input = b"<34>1 2003-10-11T22:14:15.003Z host app - - - Test";
        let result = parse(input).unwrap();
        assert!(matches!(result, ParsedSyslogMessage::Rfc5424(_)));

        // Test RFC 3164
        let input = b"<34>Oct 11 22:14:15 host tag: message";
        let result = parse(input).unwrap();
        assert!(matches!(result, ParsedSyslogMessage::Rfc3164(_)));

        // Test CEF
        let input = b"CEF:0|Security|threatmanager|1.0|100|test|10|";
        let result = parse(input).unwrap();
        assert!(matches!(result, ParsedSyslogMessage::Cef(_)));
    }

    #[test]
    fn test_priority_parsing_extremes() {
        // Test minimum priority (0)
        let input = b"<0>1 - - - - - - Test message";
        let result = parse(input).unwrap();

        if let ParsedSyslogMessage::Rfc5424(msg) = result {
            assert_eq!(msg.priority.facility, 0);
            assert_eq!(msg.priority.severity, 0);
        }

        // Test maximum priority (191)
        let input = b"<191>1 - - - - - - Test message";
        let result = parse(input).unwrap();

        if let ParsedSyslogMessage::Rfc5424(msg) = result {
            assert_eq!(msg.priority.facility, 23);
            assert_eq!(msg.priority.severity, 7);
        }
    }
}
