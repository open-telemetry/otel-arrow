// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use core::str;

use crate::syslog_cef_receiver::parser::ParseError;

/// RFC 5424 message structure
#[derive(Debug, Clone, PartialEq)]
pub struct Rfc5424Message<'a> {
    pub(super) priority: crate::syslog_cef_receiver::parser::Priority,
    pub(super) version: u8,
    pub(super) timestamp: Option<&'a [u8]>,
    pub(super) hostname: Option<&'a [u8]>,
    pub(super) app_name: Option<&'a [u8]>,
    pub(super) proc_id: Option<&'a [u8]>,
    pub(super) msg_id: Option<&'a [u8]>,
    pub(super) structured_data: Option<&'a [u8]>,
    pub(super) message: Option<&'a [u8]>,
    pub(super) input: &'a [u8],
}

/// Parse an RFC 5424 syslog message
pub fn parse_rfc5424<'a>(input: &'a [u8]) -> Result<Rfc5424Message<'a>, ParseError> {
    // Check for empty input
    if input.is_empty() {
        return Err(ParseError::EmptyInput);
    }

    let (priority, mut remaining) = crate::syslog_cef_receiver::parser::parse_priority(input)?;

    // Check if we have anything after priority
    if remaining.is_empty() {
        return Err(ParseError::InvalidVersion);
    }

    // Parse version
    let version_end = remaining
        .iter()
        .position(|&b| b == b' ')
        .ok_or(ParseError::InvalidVersion)?;

    if version_end == 0 {
        return Err(ParseError::InvalidVersion);
    }

    let version_bytes = &remaining[..version_end];
    let version_str = str::from_utf8(version_bytes).map_err(|_| ParseError::InvalidUtf8)?;
    let version: u8 = version_str
        .parse()
        .map_err(|_| ParseError::InvalidVersion)?;

    // Safe slice: we know version_end < remaining.len()
    remaining = &remaining[version_end + 1..];

    // Helper function to parse next field
    let parse_field = |s: &'a [u8]| -> (&'a [u8], &'a [u8]) {
        if let Some(pos) = s.iter().position(|&b| b == b' ') {
            let field = &s[..pos];
            let rest = if pos + 1 < s.len() {
                &s[pos + 1..]
            } else {
                b""
            };
            (if field == b"-" { b"" } else { field }, rest)
        } else {
            (if s == b"-" { b"" } else { s }, b"")
        }
    };

    // Parse timestamp
    let (timestamp, rest) = parse_field(remaining);
    let timestamp = if timestamp.is_empty() {
        None
    } else {
        Some(timestamp)
    };
    remaining = rest;

    // Parse hostname
    let (hostname, rest) = parse_field(remaining);
    let hostname = if hostname.is_empty() {
        None
    } else {
        Some(hostname)
    };
    remaining = rest;

    // Parse app-name
    let (app_name, rest) = parse_field(remaining);
    let app_name = if app_name.is_empty() {
        None
    } else {
        Some(app_name)
    };
    remaining = rest;

    // Parse procid
    let (proc_id, rest) = parse_field(remaining);
    let proc_id = if proc_id.is_empty() {
        None
    } else {
        Some(proc_id)
    };
    remaining = rest;

    // Parse msgid
    let (msg_id, rest) = parse_field(remaining);
    let msg_id = if msg_id.is_empty() {
        None
    } else {
        Some(msg_id)
    };
    remaining = rest;

    // Parse structured data and message
    let (structured_data, mut message) = if remaining.is_empty() {
        (None, None)
    } else if remaining.starts_with(b"-") {
        let msg_start = remaining
            .iter()
            .position(|&b| b == b' ')
            .map(|i| i + 1)
            .unwrap_or(remaining.len());
        let message = if msg_start < remaining.len() {
            Some(&remaining[msg_start..])
        } else {
            None
        };
        (None, message)
    } else if remaining.starts_with(b"[") {
        // Find end of all consecutive structured data elements
        let mut bracket_count = 0;
        let mut end_pos = 0;
        let mut i = 0;
        let mut in_quotes = false;
        let mut escaped = false;

        while i < remaining.len() {
            let byte = remaining[i];

            if escaped {
                escaped = false;
                // Skip this character - it's escaped
                i += 1;
                continue;
            } else if byte == b'\\' && in_quotes {
                // Only treat backslash as escape character inside quotes
                escaped = true;
            } else if byte == b'"' {
                in_quotes = !in_quotes;
            } else if !in_quotes {
                if byte == b'[' {
                    bracket_count += 1;
                } else if byte == b']' {
                    bracket_count -= 1;
                    if bracket_count == 0 {
                        // Found end of current SD-ELEMENT, check if there's another one
                        let mut j = i + 1;

                        // Skip any whitespace
                        while j < remaining.len() && remaining[j] == b' ' {
                            j += 1;
                        }

                        // If next non-space character is '[', continue parsing more SD-ELEMENTs
                        if j < remaining.len() && remaining[j] == b'[' {
                            i = j - 1; // Will be incremented at end of loop
                        } else {
                            // No more SD-ELEMENTs, this is the end
                            end_pos = i + 1;
                            break;
                        }
                    }
                }
            }
            i += 1;
        }

        // If we didn't find a closing bracket, treat the rest as structured data
        if end_pos == 0 && bracket_count > 0 {
            // Unclosed structured data - take everything
            (Some(remaining), None)
        } else if end_pos > 0 {
            let sd = &remaining[..end_pos];
            let msg_start = if end_pos < remaining.len() && remaining[end_pos] == b' ' {
                end_pos + 1
            } else {
                end_pos
            };
            let message = if msg_start < remaining.len() {
                Some(&remaining[msg_start..])
            } else {
                None
            };
            (Some(sd), message)
        } else {
            (None, Some(remaining))
        }
    } else {
        (None, Some(remaining))
    };

    // Strip UTF-8 BOM if present at the beginning of the message
    // UTF-8 BOM is the byte sequence 0xEF 0xBB 0xBF
    message = message.map(|msg| {
        if msg.len() >= 3 && msg.starts_with(&[0xEF, 0xBB, 0xBF]) {
            &msg[3..]
        } else {
            msg
        }
    });

    Ok(Rfc5424Message {
        priority,
        version,
        timestamp,
        hostname,
        app_name,
        proc_id,
        msg_id,
        structured_data,
        message,
        input,
    })
}

#[cfg(test)]
mod tests {
    use crate::syslog_cef_receiver::parser::*;

    #[test]
    fn test_rfc5424_parsing() {
        let input = b"<34>1 2003-10-11T22:14:15.003Z mymachine.example.com su - ID47 - 'su root' failed for lonvick on /dev/pts/8";
        let result = parse_rfc5424(input).unwrap();

        assert_eq!(result.priority.facility, 4);
        assert_eq!(result.priority.severity, 2);
        assert_eq!(result.version, 1);
        assert_eq!(
            result.timestamp,
            Some(b"2003-10-11T22:14:15.003Z".as_slice())
        );
        assert_eq!(result.hostname, Some(b"mymachine.example.com".as_slice()));
        assert_eq!(result.app_name, Some(b"su".as_slice()));
        assert_eq!(result.proc_id, None);
        assert_eq!(result.msg_id, Some(b"ID47".as_slice()));
        assert_eq!(result.structured_data, None);
        assert_eq!(
            result.message,
            Some(b"'su root' failed for lonvick on /dev/pts/8".as_slice())
        );
    }

    #[test]
    fn test_rfc5424_with_actual_utf8_bom() {
        // Test with actual UTF-8 BOM (0xEF 0xBB 0xBF) at start of message
        // This represents Example 1 from RFC 5424 Section 6.5
        let mut input =
            b"<34>1 2003-10-11T22:14:15.003Z mymachine.example.com su - ID47 - ".to_vec();
        input.extend_from_slice(&[0xEF, 0xBB, 0xBF]); // UTF-8 BOM
        input.extend_from_slice(b"'su root' failed for lonvick on /dev/pts/8");

        let result = parse_rfc5424(&input).unwrap();

        assert_eq!(result.priority.facility, 4);
        assert_eq!(result.priority.severity, 2);
        assert_eq!(result.version, 1);
        assert_eq!(
            result.timestamp,
            Some(b"2003-10-11T22:14:15.003Z".as_slice())
        );
        assert_eq!(result.hostname, Some(b"mymachine.example.com".as_slice()));
        assert_eq!(result.app_name, Some(b"su".as_slice()));
        assert_eq!(result.proc_id, None);
        assert_eq!(result.msg_id, Some(b"ID47".as_slice()));
        assert_eq!(result.structured_data, None);
        // Message should have BOM stripped
        assert_eq!(
            result.message,
            Some(b"'su root' failed for lonvick on /dev/pts/8".as_slice())
        );
    }

    #[test]
    fn test_structured_data_rfc5424() {
        let input = b"<165>1 2003-08-24T05:14:15.000003-07:00 192.0.2.1 myproc 8710 - [exampleSDID@32473 iut=\"3\" eventSource=\"Application\" eventID=\"1011\"] An application event log entry";
        let result = parse_rfc5424(input).unwrap();

        assert_eq!(
            result.structured_data,
            Some(
                b"[exampleSDID@32473 iut=\"3\" eventSource=\"Application\" eventID=\"1011\"]"
                    .as_slice()
            )
        );
        assert_eq!(
            result.message,
            Some(b"An application event log entry".as_slice())
        );
    }

    #[test]
    fn test_structured_data_with_actual_utf8_bom() {
        // Test with actual UTF-8 BOM after structured data
        // This represents Example 4 from RFC 5424 Section 6.5
        let mut input = b"<165>1 2003-08-24T05:14:15.000003-07:00 192.0.2.1 myproc 8710 - [exampleSDID@32473 iut=\"3\" eventSource=\"Application\" eventID=\"1011\"] ".to_vec();
        input.extend_from_slice(&[0xEF, 0xBB, 0xBF]); // UTF-8 BOM
        input.extend_from_slice(b"An application event log entry");

        let result = parse_rfc5424(&input).unwrap();

        assert_eq!(
            result.structured_data,
            Some(
                b"[exampleSDID@32473 iut=\"3\" eventSource=\"Application\" eventID=\"1011\"]"
                    .as_slice()
            )
        );
        // Message should have BOM stripped
        assert_eq!(
            result.message,
            Some(b"An application event log entry".as_slice())
        );
    }

    #[test]
    fn test_rfc5424_minimal_message() {
        // Minimal valid RFC 5424 message
        let input = b"<34>1 - - - - - - ";
        let result = parse_rfc5424(input).unwrap();

        assert_eq!(result.version, 1);
        assert_eq!(result.timestamp, None);
        assert_eq!(result.hostname, None);
        assert_eq!(result.app_name, None);
        assert_eq!(result.proc_id, None);
        assert_eq!(result.msg_id, None);
        assert_eq!(result.structured_data, None);
        assert_eq!(result.message, None);
    }

    #[test]
    fn test_rfc5424_only_structured_data() {
        let input = b"<34>1 2003-10-11T22:14:15.003Z host app proc msgid [id@123 key=\"value\"]";
        let result = parse_rfc5424(input).unwrap();

        assert_eq!(result.priority.facility, 4);
        assert_eq!(result.priority.severity, 2);
        assert_eq!(result.version, 1);
        assert_eq!(
            result.timestamp,
            Some(b"2003-10-11T22:14:15.003Z".as_slice())
        );
        assert_eq!(result.hostname, Some(b"host".as_slice()));
        assert_eq!(result.app_name, Some(b"app".as_slice()));
        assert_eq!(result.proc_id, Some(b"proc".as_slice()));
        assert_eq!(result.msg_id, Some(b"msgid".as_slice()));
        assert_eq!(
            result.structured_data,
            Some(b"[id@123 key=\"value\"]".as_slice())
        );
        assert_eq!(result.message, None);
    }

    #[test]
    fn test_rfc5424_multiple_structured_data() {
        let input = b"<34>1 - - - - - [id1@123 key1=\"val1\"][id2@456 key2=\"val2\"] Message text";
        let result = parse_rfc5424(input).unwrap();

        assert_eq!(result.priority.facility, 4);
        assert_eq!(result.priority.severity, 2);
        assert_eq!(result.version, 1);
        assert_eq!(result.timestamp, None);
        assert_eq!(result.hostname, None);
        assert_eq!(result.app_name, None);
        assert_eq!(result.proc_id, None);
        assert_eq!(result.msg_id, None);
        // Now correctly captures all structured data elements
        assert_eq!(
            result.structured_data,
            Some(b"[id1@123 key1=\"val1\"][id2@456 key2=\"val2\"]".as_slice())
        );
        assert_eq!(result.message, Some(b"Message text".as_slice()));
    }

    #[test]
    fn test_rfc5424_multiple_structured_data_with_spaces() {
        let input = b"<34>1 - - - - - [id1@123 key1=\"val1\"] [id2@456 key2=\"val2\"] [id3@789 key3=\"val3\"] Message text";
        let result = parse_rfc5424(input).unwrap();

        assert_eq!(result.priority.facility, 4);
        assert_eq!(result.priority.severity, 2);
        assert_eq!(result.version, 1);
        assert_eq!(result.timestamp, None);
        assert_eq!(result.hostname, None);
        assert_eq!(result.app_name, None);
        assert_eq!(result.proc_id, None);
        assert_eq!(result.msg_id, None);
        // Captures all structured data elements including spaces
        assert_eq!(
            result.structured_data,
            Some(
                b"[id1@123 key1=\"val1\"] [id2@456 key2=\"val2\"] [id3@789 key3=\"val3\"]"
                    .as_slice()
            )
        );
        assert_eq!(result.message, Some(b"Message text".as_slice()));
    }

    #[test]
    fn test_rfc5424_escaped_characters_in_structured_data() {
        let input = b"<34>1 - - - - - [id@123 key=\"val\\\"ue with \\] and \\\\ chars\"] Message";
        let result = parse_rfc5424(input);
        // This should handle escaped quotes, brackets, and backslashes
        assert!(result.is_ok());

        if let Ok(msg) = result {
            assert_eq!(msg.priority.facility, 4);
            assert_eq!(msg.priority.severity, 2);
            assert_eq!(msg.version, 1);
            assert_eq!(msg.timestamp, None);
            assert_eq!(msg.hostname, None);
            assert_eq!(msg.app_name, None);
            assert_eq!(msg.proc_id, None);
            assert_eq!(msg.msg_id, None);
            // Parser correctly handles escaped characters in structured data
            // The escaped bracket \] doesn't end the structured data element
            assert_eq!(
                msg.structured_data,
                Some(b"[id@123 key=\"val\\\"ue with \\] and \\\\ chars\"]".as_slice())
            );
            assert_eq!(msg.message, Some(b"Message".as_slice()));
        }
    }

    #[test]
    fn test_rfc5424_field_length_limits() {
        // Test with very long hostname (over 255 chars)
        let long_hostname = "a".repeat(300);
        let input =
            format!("<34>1 2003-10-11T22:14:15.003Z {long_hostname} app proc msgid - Message");
        let result = parse_rfc5424(input.as_bytes());
        // Should either truncate or reject based on RFC compliance level desired
        assert!(result.is_ok());

        if let Ok(msg) = result {
            assert_eq!(msg.priority.facility, 4);
            assert_eq!(msg.priority.severity, 2);
            assert_eq!(msg.version, 1);
            assert_eq!(msg.timestamp, Some(b"2003-10-11T22:14:15.003Z".as_slice()));
            assert_eq!(msg.hostname, Some(long_hostname.as_bytes()));
            assert_eq!(msg.app_name, Some(b"app".as_slice()));
            assert_eq!(msg.proc_id, Some(b"proc".as_slice()));
            assert_eq!(msg.msg_id, Some(b"msgid".as_slice()));
            assert_eq!(msg.structured_data, None);
            assert_eq!(msg.message, Some(b"Message".as_slice()));
        }
    }

    #[test]
    fn test_rfc5424_invalid_characters() {
        // Test hostname with invalid characters
        let input = b"<34>1 2003-10-11T22:14:15.003Z host[name] app proc msgid - Message";
        let result = parse_rfc5424(input);
        // Should handle or reject invalid characters in hostname
        assert!(result.is_ok()); // Current implementation is permissive

        if let Ok(msg) = result {
            assert_eq!(msg.priority.facility, 4);
            assert_eq!(msg.priority.severity, 2);
            assert_eq!(msg.version, 1);
            assert_eq!(msg.timestamp, Some(b"2003-10-11T22:14:15.003Z".as_slice()));
            assert_eq!(msg.hostname, Some(b"host[name]".as_slice()));
            assert_eq!(msg.app_name, Some(b"app".as_slice()));
            assert_eq!(msg.proc_id, Some(b"proc".as_slice()));
            assert_eq!(msg.msg_id, Some(b"msgid".as_slice()));
            assert_eq!(msg.structured_data, None);
            assert_eq!(msg.message, Some(b"Message".as_slice()));
        }
    }

    #[test]
    fn test_rfc5424_multi_digit_version() {
        let input = b"<34>10 2003-10-11T22:14:15.003Z host app proc msgid - Message";
        let result = parse_rfc5424(input).unwrap();

        assert_eq!(result.priority.facility, 4);
        assert_eq!(result.priority.severity, 2);
        assert_eq!(result.version, 10);
        assert_eq!(
            result.timestamp,
            Some(b"2003-10-11T22:14:15.003Z".as_slice())
        );
        assert_eq!(result.hostname, Some(b"host".as_slice()));
        assert_eq!(result.app_name, Some(b"app".as_slice()));
        assert_eq!(result.proc_id, Some(b"proc".as_slice()));
        assert_eq!(result.msg_id, Some(b"msgid".as_slice()));
        assert_eq!(result.structured_data, None);
        assert_eq!(result.message, Some(b"Message".as_slice()));
    }

    #[test]
    fn test_byte_slice_to_string_conversion() {
        // Test showing how consumers can convert byte slices to strings when needed
        let input = b"<34>1 2003-10-11T22:14:15.003Z mymachine.example.com su - ID47 - 'su root' failed for lonvick on /dev/pts/8";
        let result = parse_rfc5424(input).unwrap();

        // Test direct access to byte slices
        assert_eq!(
            result.timestamp,
            Some(b"2003-10-11T22:14:15.003Z".as_slice())
        );
        assert_eq!(result.hostname, Some(b"mymachine.example.com".as_slice()));

        // Test conversion to strings using std::str::from_utf8
        assert_eq!(
            std::str::from_utf8(result.timestamp.unwrap()).unwrap(),
            "2003-10-11T22:14:15.003Z"
        );
        assert_eq!(
            std::str::from_utf8(result.hostname.unwrap()).unwrap(),
            "mymachine.example.com"
        );
        assert_eq!(std::str::from_utf8(result.app_name.unwrap()).unwrap(), "su");
        assert_eq!(std::str::from_utf8(result.msg_id.unwrap()).unwrap(), "ID47");
        assert_eq!(
            std::str::from_utf8(result.message.unwrap()).unwrap(),
            "'su root' failed for lonvick on /dev/pts/8"
        );
    }

    // Edge case tests to ensure no panic occurs
    #[test]
    fn test_empty_input() {
        let input = b"";
        let result = parse_rfc5424(input);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ParseError::EmptyInput);
    }

    #[test]
    fn test_priority_only() {
        let input = b"<34>";
        let result = parse_rfc5424(input);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ParseError::InvalidVersion);
    }

    #[test]
    fn test_priority_and_version_no_space() {
        let input = b"<34>1";
        let result = parse_rfc5424(input);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ParseError::InvalidVersion);
    }

    #[test]
    fn test_malformed_structured_data_unclosed() {
        let input = b"<34>1 - - - - - [id@123 key=\"value\" ";
        let result = parse_rfc5424(input);
        assert!(result.is_ok());

        let msg = result.unwrap();
        assert_eq!(msg.priority.facility, 4);
        assert_eq!(msg.priority.severity, 2);
        assert_eq!(msg.version, 1);
        assert_eq!(msg.timestamp, None);
        assert_eq!(msg.hostname, None);
        assert_eq!(msg.app_name, None);
        assert_eq!(msg.proc_id, None);
        assert_eq!(msg.msg_id, None);
        // Unclosed structured data should be captured as is
        assert_eq!(
            msg.structured_data,
            Some(b"[id@123 key=\"value\" ".as_slice())
        );
        assert_eq!(msg.message, None);
    }

    #[test]
    fn test_structured_data_with_escaped_bracket() {
        let input = b"<34>1 - - - - - [id@123 key=\"val\\]ue\"] Message";
        let result = parse_rfc5424(input);
        assert!(result.is_ok());

        let msg = result.unwrap();
        assert_eq!(msg.priority.facility, 4);
        assert_eq!(msg.priority.severity, 2);
        assert_eq!(msg.version, 1);
        assert_eq!(msg.timestamp, None);
        assert_eq!(msg.hostname, None);
        assert_eq!(msg.app_name, None);
        assert_eq!(msg.proc_id, None);
        assert_eq!(msg.msg_id, None);
        // Parser correctly handles escaped bracket within quotes
        assert_eq!(
            msg.structured_data,
            Some(b"[id@123 key=\"val\\]ue\"]".as_slice())
        );
        assert_eq!(msg.message, Some(b"Message".as_slice()));
    }

    #[test]
    fn test_single_bracket() {
        let input = b"<34>1 - - - - - [ Message";
        let result = parse_rfc5424(input);
        assert!(result.is_ok());

        let msg = result.unwrap();
        assert_eq!(msg.priority.facility, 4);
        assert_eq!(msg.priority.severity, 2);
        assert_eq!(msg.version, 1);
        assert_eq!(msg.timestamp, None);
        assert_eq!(msg.hostname, None);
        assert_eq!(msg.app_name, None);
        assert_eq!(msg.proc_id, None);
        assert_eq!(msg.msg_id, None);
        // Unclosed bracket - everything after '[' is treated as structured data
        assert_eq!(msg.structured_data, Some(b"[ Message".as_slice()));
        assert_eq!(msg.message, None);
    }

    #[test]
    fn test_version_followed_by_eof() {
        let input = b"<34>1 ";
        let result = parse_rfc5424(input);
        assert!(result.is_ok());

        let msg = result.unwrap();
        assert_eq!(msg.priority.facility, 4);
        assert_eq!(msg.priority.severity, 2);
        assert_eq!(msg.version, 1);
        // All fields should be None when missing
        assert_eq!(msg.timestamp, None);
        assert_eq!(msg.hostname, None);
        assert_eq!(msg.app_name, None);
        assert_eq!(msg.proc_id, None);
        assert_eq!(msg.msg_id, None);
        assert_eq!(msg.structured_data, None);
        assert_eq!(msg.message, None);
    }

    #[test]
    fn test_incomplete_fields() {
        let input = b"<34>1 2003";
        let result = parse_rfc5424(input);
        assert!(result.is_ok());

        let msg = result.unwrap();
        assert_eq!(msg.priority.facility, 4);
        assert_eq!(msg.priority.severity, 2);
        assert_eq!(msg.version, 1);
        // "2003" is treated as timestamp (even if incomplete)
        assert_eq!(msg.timestamp, Some(b"2003".as_slice()));
        // Rest of fields are missing
        assert_eq!(msg.hostname, None);
        assert_eq!(msg.app_name, None);
        assert_eq!(msg.proc_id, None);
        assert_eq!(msg.msg_id, None);
        assert_eq!(msg.structured_data, None);
        assert_eq!(msg.message, None);
    }

    #[test]
    fn test_structured_data_edge_cases() {
        // Test with empty structured data '[]'
        let input = b"<34>1 - - - - - [] Message";
        let result = parse_rfc5424(input);
        assert!(result.is_ok());

        let msg = result.unwrap();
        assert_eq!(msg.priority.facility, 4);
        assert_eq!(msg.priority.severity, 2);
        assert_eq!(msg.version, 1);
        assert_eq!(msg.timestamp, None);
        assert_eq!(msg.hostname, None);
        assert_eq!(msg.app_name, None);
        assert_eq!(msg.proc_id, None);
        assert_eq!(msg.msg_id, None);
        assert_eq!(msg.structured_data, Some(b"[]".as_slice()));
        assert_eq!(msg.message, Some(b"Message".as_slice()));

        // Test with nested brackets (invalid but shouldn't panic)
        let input2 = b"<34>1 - - - - - [id@123 [nested] key=\"value\"] Message";
        let result2 = parse_rfc5424(input2);
        assert!(result2.is_ok());

        let msg2 = result2.unwrap();
        assert_eq!(msg2.priority.facility, 4);
        assert_eq!(msg2.priority.severity, 2);
        assert_eq!(msg2.version, 1);
        assert_eq!(msg2.timestamp, None);
        assert_eq!(msg2.hostname, None);
        assert_eq!(msg2.app_name, None);
        assert_eq!(msg2.proc_id, None);
        assert_eq!(msg2.msg_id, None);
        // Parser treats nested brackets as part of the structured data
        assert_eq!(
            msg2.structured_data,
            Some(b"[id@123 [nested] key=\"value\"]".as_slice())
        );
        assert_eq!(msg2.message, Some(b"Message".as_slice()));
    }

    #[test]
    fn test_extreme_whitespace() {
        let input = b"<34>1                             -     -    -   -   -    Message";
        let result = parse_rfc5424(input);
        assert!(result.is_ok());

        let msg = result.unwrap();
        assert_eq!(msg.priority.facility, 4);
        assert_eq!(msg.priority.severity, 2);
        assert_eq!(msg.version, 1);
        // When parse_field encounters leading spaces, it finds the first space at position 0
        // This creates an empty field, which becomes None
        assert_eq!(msg.timestamp, None);
        assert_eq!(msg.hostname, None);
        assert_eq!(msg.app_name, None);
        assert_eq!(msg.proc_id, None);
        assert_eq!(msg.msg_id, None);
        assert_eq!(msg.structured_data, None);
        // The parser consumes one space for each field parsed (5 fields before structured data)
        // So the message starts with fewer spaces than the original
        assert_eq!(
            msg.message,
            Some(b"                       -     -    -   -   -    Message".as_slice())
        );
    }
}
