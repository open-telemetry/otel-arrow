// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

/// Parser for Common Event Format (CEF) messages
pub mod cef;
/// Parser for the unified representation of parsed syslog messages
pub mod parsed_message;
/// Parser for syslog messages in RFC3164 format
pub mod rfc3164;
/// Parser for syslog messages in RFC5424 format
pub mod rfc5424;

use self::cef::parse_cef;
use self::parsed_message::ParsedSyslogMessage;
use self::rfc3164::parse_rfc3164_with_priority;
use self::rfc5424::parse_rfc5424_with_priority;

/// Priority structure containing facility and severity
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Priority {
    pub facility: u8,
    pub severity: u8,
}

/// Error types that can occur during parsing
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    /// Error indicating that the input is empty
    EmptyInput,
    /// Error parsing priority value
    /// RFC5424 messages are expected to start with "<Priority>"
    InvalidPriority,
    /// Error parsing version number for RFC5424 messages
    InvalidVersion,
    /// Error indicating that CEF content is empty
    EmptyCEFContent,
    /// Error parsing CEF message
    InvalidCef,
    /// Error parsing UTF-8 strings
    InvalidUtf8,
    /// Error indicating that the format of the syslog message is unknown
    UnknownFormat,
}

/// Parse a syslog message from bytes, automatically detecting the format
pub fn parse(input: &[u8]) -> Result<ParsedSyslogMessage<'_>, ParseError> {
    if input.is_empty() {
        return Err(ParseError::EmptyInput);
    }

    // Try pure CEF first - it's the simplest check
    if input.starts_with(b"CEF:") {
        if let Ok(cef_msg) = parse_cef(input) {
            return Ok(ParsedSyslogMessage::Cef(cef_msg));
        }
    }

    // Parse priority once — both RFC 5424 and RFC 3164 start with <priority>.
    // Messages without a valid priority prefix skip straight to RFC 3164 (no-PRI path).
    if input.starts_with(b"<") {
        if let Ok((priority, remaining)) = parse_priority(input) {
            // Try RFC 5424 first (has version number after priority)
            if let Ok(rfc5424_msg) =
                parse_rfc5424_with_priority(priority.clone(), remaining, input)
            {
                // Check if the message contains CEF
                if let Some(msg) = rfc5424_msg.message {
                    if msg.starts_with(b"CEF:") {
                        if let Ok(cef_msg) = parse_cef(msg) {
                            return Ok(ParsedSyslogMessage::CefWithRfc5424(rfc5424_msg, cef_msg));
                        }
                    }
                }
                return Ok(ParsedSyslogMessage::Rfc5424(rfc5424_msg));
            }

            // Fall through to RFC 3164 with the already-parsed priority
            if let Ok(rfc3164_msg) =
                parse_rfc3164_with_priority(Some(priority), remaining, input)
            {
                return try_rfc3164_cef(rfc3164_msg, input);
            }
        } else {
            // Invalid PRI format — RFC 3164 no-PRI path (entire input as content)
            if let Ok(rfc3164_msg) = parse_rfc3164_with_priority(None, input, input) {
                return try_rfc3164_cef(rfc3164_msg, input);
            }
        }
    } else {
        // No '<' prefix — RFC 3164 no-PRI path
        if let Ok(rfc3164_msg) = parse_rfc3164_with_priority(None, input, input) {
            return try_rfc3164_cef(rfc3164_msg, input);
        }
    }

    Err(ParseError::UnknownFormat)
}

/// Given a parsed RFC 3164 message, check for embedded CEF content and return
/// the appropriate `ParsedSyslogMessage` variant.
fn try_rfc3164_cef<'a>(
    rfc3164_msg: rfc3164::Rfc3164Message<'a>,
    input: &'a [u8],
) -> Result<ParsedSyslogMessage<'a>, ParseError> {
    // Check if the content contains CEF
    if let Some(content) = rfc3164_msg.content {
        if content.starts_with(b"CEF:") {
            if let Ok(cef_msg) = parse_cef(content) {
                return Ok(ParsedSyslogMessage::CefWithRfc3164(rfc3164_msg, cef_msg));
            }
        }
    }

    // Special case: If tag is "CEF", the full CEF message spans from "CEF:" in the input
    // This handles the case where RFC3164 parser splits "CEF:1|..." into tag="CEF" and content="1|..."
    if rfc3164_msg.tag == Some(&b"CEF"[..]) && rfc3164_msg.content.is_some() {
        // Find where "CEF:" appears in the original input after the hostname
        if let Some(hostname) = rfc3164_msg.hostname {
            // Find hostname position in input
            if let Some(hostname_pos) =
                input.windows(hostname.len()).position(|w| w == hostname)
            {
                // Look for "CEF:" after hostname position
                let search_start = hostname_pos + hostname.len();
                let after_hostname = &input[search_start..];

                // Find "CEF:" in the remaining input
                if let Some(cef_pos) = after_hostname.windows(4).position(|w| w == b"CEF:") {
                    // The CEF message starts at this position and goes to the end
                    let cef_message = &after_hostname[cef_pos..];

                    if let Ok(cef_msg) = parse_cef(cef_message) {
                        // Update the rfc3164 content to point to the full CEF message
                        let mut modified_rfc3164 = rfc3164_msg;
                        modified_rfc3164.content = Some(cef_message);
                        return Ok(ParsedSyslogMessage::CefWithRfc3164(
                            modified_rfc3164,
                            cef_msg,
                        ));
                    }
                }
            }
        }
    }

    Ok(ParsedSyslogMessage::Rfc3164(rfc3164_msg))
}

/// Parse priority from the beginning of a syslog message
#[allow(clippy::manual_range_contains)]
pub(super) fn parse_priority(input: &[u8]) -> Result<(Priority, &[u8]), ParseError> {
    if input.is_empty() || input[0] != b'<' {
        return Err(ParseError::InvalidPriority);
    }

    let end = input
        .iter()
        .position(|&b| b == b'>')
        .ok_or(ParseError::InvalidPriority)?;

    if end < 2 || end > 4 {
        return Err(ParseError::InvalidPriority);
    }

    let priority_bytes = &input[1..end];

    // RFC 3164 Section 4.3.3: Check for leading zeros which make PRI "unidentifiable"
    // Example: <00> is invalid, <0> is valid
    if priority_bytes.len() > 1 && priority_bytes[0] == b'0' {
        return Err(ParseError::InvalidPriority);
    }

    // Parse 1-3 ASCII digits directly without str::from_utf8 + str::parse overhead.
    let mut priority_num: u8 = 0;
    for &b in priority_bytes {
        if !b.is_ascii_digit() {
            return Err(ParseError::InvalidPriority);
        }
        priority_num = priority_num
            .checked_mul(10)
            .and_then(|v| v.checked_add(b - b'0'))
            .ok_or(ParseError::InvalidPriority)?;
    }

    // RFC 3164: Maximum valid priority is 191 (facility 23, severity 7)
    if priority_num > 191 {
        return Err(ParseError::InvalidPriority);
    }

    let facility = priority_num >> 3; // Upper 5 bits are facility. This is equivalent to priority_num / 8
    let severity = priority_num & 0x07; // Lower 3 bits are severity. This is equivalent to priority_num % 8

    Ok((Priority { facility, severity }, &input[end + 1..]))
}

/// Benchmark support — exposes internal parser helpers for benchmarking only.
///
/// This module is **not** part of the public API and is gated behind the `bench`
/// Cargo feature. Items here may change or be removed without notice.
#[cfg(feature = "bench")]
#[doc(hidden)]
pub mod bench_support {
    use super::ParseError;
    use super::cef::CefMessage;
    use super::parsed_message::ParsedSyslogMessage;

    /// Auto-detect format and parse a syslog message.
    pub fn parse(input: &[u8]) -> Result<ParsedSyslogMessage<'_>, ParseError> {
        super::parse(input)
    }

    /// Extract the timestamp from a parsed message.
    #[must_use]
    pub fn timestamp(msg: &ParsedSyslogMessage<'_>) -> Option<u64> {
        msg.timestamp()
    }

    /// Opaque wrapper around the internal CEF extensions iterator.
    pub struct CefExtensionsIter<'a>(super::cef::CefExtensionsIter<'a>);

    impl<'a> CefExtensionsIter<'a> {
        /// Returns the next key-value extension pair, or `None` when exhausted.
        pub fn next_extension(&mut self) -> Option<(&[u8], &[u8])> {
            self.0.next_extension()
        }
    }

    /// Start iterating over CEF extensions.
    #[must_use]
    pub const fn parse_extensions<'a>(msg: &'a CefMessage<'a>) -> CefExtensionsIter<'a> {
        CefExtensionsIter(msg.parse_extensions())
    }
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
