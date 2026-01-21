// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::syslog_cef_receiver::parser;

/// RFC 3164 message structure
#[derive(Debug, Clone, PartialEq)]
pub struct Rfc3164Message<'a> {
    pub(super) priority: Option<parser::Priority>,
    pub(super) timestamp: Option<&'a [u8]>,
    pub(super) hostname: Option<&'a [u8]>,
    pub(super) tag: Option<&'a [u8]>,
    pub(super) app_name: Option<&'a [u8]>,
    pub(super) proc_id: Option<&'a [u8]>,
    pub(super) content: Option<&'a [u8]>,
    pub(super) input: &'a [u8],
}

/// Parse an RFC 3164 syslog message from bytes, automatically detecting the format
///
/// This parser identifies and extracts fields from syslog messages but does not
/// act as a relay. Messages without valid PRI headers are accepted and parsed
/// for their content, but no default values are assigned. The calling code must
/// decide how to handle missing fields.
///
/// # Behavior for RFC 3164 messages without PRI:
/// - The message is parsed for any identifiable fields (timestamp, hostname, etc.)
/// - No default priority is assigned
/// - The entire message may be treated as content if no structure is found
///
/// # TAG Parsing
/// The TAG field in RFC 3164 often contains the application name and optionally
/// a process ID in the format `appname[pid]`. This parser extracts:
/// - `app_name`: The part before `[` (or the entire TAG if no `[` is present)
/// - `proc_id`: The numeric content between `[` and `]` (only if it's a valid number)
pub fn parse_rfc3164(input: &[u8]) -> Result<Rfc3164Message<'_>, parser::ParseError> {
    if input.is_empty() {
        return Err(parser::ParseError::EmptyInput);
    }

    // RFC 3164 Section 4.3: Check if we have a valid PRI
    let (priority, mut remaining) = if input.starts_with(b"<") {
        // Try to parse the PRI
        match parser::parse_priority(input) {
            Ok((pri, rest)) => (Some(pri), rest),
            Err(_) => {
                // Invalid PRI format, treat entire input as content
                (None, input)
            }
        }
    } else {
        // No PRI at all (doesn't start with '<')
        (None, input)
    };

    // Parse timestamp (optional)
    let (timestamp, rest) = if remaining.len() >= 15 {
        // Try to parse timestamp (MMM dd HH:MM:SS format)
        let potential_ts = &remaining[..15];
        // Safe bounds checking
        if remaining.len() > 6 && remaining.get(3) == Some(&b' ') && remaining.get(6) == Some(&b' ')
        {
            // Safe slicing - check if we have at least 16 bytes before slicing
            let rest = if remaining.len() > 15 {
                &remaining[16..]
            } else {
                &remaining[15..]
            };
            (Some(potential_ts), rest)
        } else {
            (None, remaining)
        }
    } else {
        (None, remaining)
    };

    remaining = rest;

    // Parse hostname, tag, and content according to RFC 3164
    let (hostname, tag, content) =
        if let Some(colon_pos) = remaining.iter().position(|&b| b == b':') {
            let before_colon = &remaining[..colon_pos];
            let after_colon = &remaining[colon_pos + 1..];

            // RFC 3164: Content is everything after "TAG: " (note the space)
            // When there's a TAG, the text after tag: is the CONTENT
            // When there's no TAG, the entire MSG part is the CONTENT
            // Safe bounds checking using get()
            let content = if after_colon.first() == Some(&b' ') {
                Some(&after_colon[1..])
            } else {
                Some(after_colon)
            };

            if let Some(space_pos) = before_colon.iter().rposition(|&b| b == b' ') {
                // Format: hostname TAG: CONTENT
                (
                    Some(&before_colon[..space_pos]),
                    Some(&before_colon[space_pos + 1..]),
                    content,
                )
            } else {
                // Format: TAG: CONTENT (no hostname)
                (None, Some(before_colon), content)
            }
        } else {
            // No colon found - entire remaining text is CONTENT (no TAG)
            (None, None, Some(remaining))
        };

    // Parse app_name and proc_id from the TAG field
    // TAG format is typically: "appname" or "appname[pid]"
    let (app_name, proc_id) = parse_tag_components(tag);

    Ok(Rfc3164Message {
        priority,
        timestamp,
        hostname,
        tag,
        app_name,
        proc_id,
        content,
        input,
    })
}

/// Parse app_name and proc_id from a TAG field.
///
/// The TAG field in RFC 3164 often follows the format `appname[pid]`.
/// This function extracts:
/// - `app_name`: Everything before the `[` character (or the entire TAG if no `[`)
/// - `proc_id`: The content between `[` and `]`, only if it's a valid numeric value
///
/// # Examples
/// - `"su"` -> app_name=`"su"`, proc_id=None
/// - `"sshd[5678]"` -> app_name=`"sshd"`, proc_id=`"5678"`
/// - `"app[worker-1]"` -> app_name=`"app"`, proc_id=None (non-numeric)
/// - `"app[]"` -> app_name=`"app"`, proc_id=None (empty)
fn parse_tag_components(tag: Option<&[u8]>) -> (Option<&[u8]>, Option<&[u8]>) {
    let Some(tag) = tag else {
        return (None, None);
    };

    if tag.is_empty() {
        return (None, None);
    }

    // Find the position of '[' in the tag
    let Some(bracket_pos) = tag.iter().position(|&b| b == b'[') else {
        // No bracket found, entire tag is the app_name
        return (Some(tag), None);
    };

    // Extract app_name (everything before '[')
    let app_name = if bracket_pos > 0 {
        Some(&tag[..bracket_pos])
    } else {
        None
    };

    // Find the closing bracket
    let Some(close_bracket_pos) = tag.iter().position(|&b| b == b']') else {
        // No closing bracket, treat entire tag as app_name (malformed)
        return (Some(tag), None);
    };

    // Extract proc_id (content between '[' and ']')
    if close_bracket_pos > bracket_pos + 1 {
        let proc_id_bytes = &tag[bracket_pos + 1..close_bracket_pos];
        // Only accept numeric proc_id values
        if proc_id_bytes.iter().all(|&b| b.is_ascii_digit()) {
            return (app_name, Some(proc_id_bytes));
        }
    }

    (app_name, None)
}

#[cfg(test)]
mod tests {
    use crate::syslog_cef_receiver::parser::*;

    #[test]
    fn test_rfc3164_parsing() {
        let input = b"<34>Oct 11 22:14:15 mymachine su: 'su root' failed for lonvick on /dev/pts/8";
        let result = parse_rfc3164(input).unwrap();

        let priority = result.priority.unwrap();
        assert_eq!(priority.facility, 4);
        assert_eq!(priority.severity, 2);
        assert_eq!(result.timestamp, Some(b"Oct 11 22:14:15".as_slice()));
        assert_eq!(result.hostname, Some(b"mymachine".as_slice()));
        assert_eq!(result.tag, Some(b"su".as_slice()));
        assert_eq!(result.app_name, Some(b"su".as_slice()));
        assert_eq!(result.proc_id, None);
        assert_eq!(
            result.content,
            Some(b"'su root' failed for lonvick on /dev/pts/8".as_slice())
        );
    }

    #[test]
    fn test_rfc3164_without_timestamp() {
        let input = b"<34>hostname tag: message content";
        let result = parse_rfc3164(input).unwrap();

        let priority = result.priority.unwrap();
        assert_eq!(priority.facility, 4);
        assert_eq!(priority.severity, 2);
        assert_eq!(result.timestamp, None);
        assert_eq!(result.hostname, Some(b"hostname".as_slice()));
        assert_eq!(result.tag, Some(b"tag".as_slice()));
        assert_eq!(result.app_name, Some(b"tag".as_slice()));
        assert_eq!(result.proc_id, None);
        assert_eq!(result.content, Some(b"message content".as_slice()));
    }

    #[test]
    fn test_rfc3164_content_only() {
        let input = b"<34>This is just content without colon";
        let result = parse_rfc3164(input).unwrap();

        let priority = result.priority.unwrap();
        assert_eq!(priority.facility, 4);
        assert_eq!(priority.severity, 2);
        assert_eq!(result.timestamp, None);
        assert_eq!(result.hostname, None);
        assert_eq!(result.tag, None);
        assert_eq!(
            result.content,
            Some(b"This is just content without colon".as_slice())
        );
    }

    #[test]
    fn test_rfc3164_valid_priority_zero() {
        // <0> is a valid priority (facility=0, severity=0)
        let input = b"<0>Test message with priority zero";
        let result = parse_rfc3164(input).unwrap();

        // Priority should be parsed successfully
        assert!(result.priority.is_some());
        let priority = result.priority.unwrap();
        assert_eq!(priority.facility, 0);
        assert_eq!(priority.severity, 0);
        assert_eq!(result.timestamp, None);
        assert_eq!(result.hostname, None);
        assert_eq!(result.tag, None);
        assert_eq!(
            result.content,
            Some(b"Test message with priority zero".as_slice())
        );
    }

    #[test]
    fn test_rfc3164_no_pri() {
        // RFC 3164 Section 4.3.3: Example 2 "Use the BFG!"
        let input = b"Use the BFG!";
        let result = parse_rfc3164(input).unwrap();

        assert!(result.priority.is_none());
        assert_eq!(result.timestamp, None);
        assert_eq!(result.hostname, None);
        assert_eq!(result.tag, None);
        assert_eq!(result.content, Some(b"Use the BFG!".as_slice()));
    }

    #[test]
    fn test_rfc3164_invalid_pri() {
        // RFC 3164 Section 4.3.3: Unidentifiable PRI like "<00>"
        let input = b"<00>Test message";
        let result = parse_rfc3164(input).unwrap();

        // Priority should be None and entire input treated as content
        assert!(result.priority.is_none());
        assert_eq!(result.timestamp, None);
        assert_eq!(result.hostname, None);
        assert_eq!(result.tag, None);
        assert_eq!(result.content, Some(b"<00>Test message".as_slice()));

        let input = b"<999Test message";
        let result = parse_rfc3164(input).unwrap();

        // Priority should be None and entire input treated as content
        assert!(result.priority.is_none());
        assert_eq!(result.timestamp, None);
        assert_eq!(result.hostname, None);
        assert_eq!(result.tag, None);
        assert_eq!(result.content, Some(b"<999Test message".as_slice()));

        let input = b"<abc> Test message";
        let result = parse_rfc3164(input).unwrap();

        // Priority should be None and entire input treated as content
        assert!(result.priority.is_none());
        assert_eq!(result.timestamp, None);
        assert_eq!(result.hostname, None);
        assert_eq!(result.tag, None);
        assert_eq!(result.content, Some(b"<abc> Test message".as_slice()));

        let input = b"<> Test message";
        let result = parse_rfc3164(input).unwrap();

        // Priority should be None and entire input treated as content
        assert!(result.priority.is_none());
        assert_eq!(result.timestamp, None);
        assert_eq!(result.hostname, None);
        assert_eq!(result.tag, None);
        assert_eq!(result.content, Some(b"<> Test message".as_slice()));
    }

    #[test]
    fn test_rfc3164_no_pri_with_timestamp_like_content() {
        // Message that looks like it might have a timestamp but no PRI
        let input = b"Oct 11 22:14:15 mymachine su: test message";
        let result = parse_rfc3164(input).unwrap();

        // Priority should be None
        assert!(result.priority.is_none());
        // The timestamp-looking part should be parsed as timestamp
        assert_eq!(result.timestamp, Some(b"Oct 11 22:14:15".as_slice()));
        assert_eq!(result.hostname, Some(b"mymachine".as_slice()));
        assert_eq!(result.tag, Some(b"su".as_slice()));
        assert_eq!(result.content, Some(b"test message".as_slice()));
    }

    // Edge case tests to ensure no panics occur
    #[test]
    fn test_empty_input() {
        let input = b"";
        let result = parse_rfc3164(input);

        // Empty input should return an error, not Ok
        assert!(result.is_err());
        assert!(matches!(result, Err(ParseError::EmptyInput)));
    }

    #[test]
    fn test_timestamp_parsing_with_short_input() {
        let input = b"<34>Oct";
        let result = parse_rfc3164(input).unwrap();

        let priority = result.priority.unwrap();
        assert_eq!(priority.facility, 4);
        assert_eq!(priority.severity, 2);
        assert_eq!(result.timestamp, None); // Too short to be a valid timestamp
        assert_eq!(result.hostname, None);
        assert_eq!(result.tag, None);
        assert_eq!(result.content, Some(b"Oct".as_slice())); // Treated as content
    }

    #[test]
    fn test_timestamp_parsing_with_exact_3_bytes() {
        let input = b"<34>Oct ";
        let result = parse_rfc3164(input).unwrap();

        let priority = result.priority.unwrap();
        assert_eq!(priority.facility, 4);
        assert_eq!(priority.severity, 2);
        assert_eq!(result.timestamp, None); // Too short to be a valid timestamp
        assert_eq!(result.hostname, None);
        assert_eq!(result.tag, None);
        assert_eq!(result.content, Some(b"Oct ".as_slice())); // Treated as content
    }

    #[test]
    fn test_after_colon_empty() {
        let input = b"<34>hostname tag:";
        let result = parse_rfc3164(input).unwrap();

        let priority = result.priority.unwrap();
        assert_eq!(priority.facility, 4);
        assert_eq!(priority.severity, 2);
        assert_eq!(result.timestamp, None);
        assert_eq!(result.hostname, Some(b"hostname".as_slice()));
        assert_eq!(result.tag, Some(b"tag".as_slice()));
        assert_eq!(result.content, Some(b"".as_slice())); // Empty content after colon
    }

    #[test]
    fn test_timestamp_boundary_at_15() {
        let input = b"<34>Oct 11 22:14:15";
        let result = parse_rfc3164(input).unwrap();

        let priority = result.priority.unwrap();
        assert_eq!(priority.facility, 4);
        assert_eq!(priority.severity, 2);
        assert_eq!(result.timestamp, Some(b"Oct 11 22:14:15".as_slice()));
        assert_eq!(result.hostname, None); // No content after timestamp
        assert_eq!(result.tag, None);
        assert_eq!(result.content, Some(b"".as_slice())); // Empty content
    }

    #[test]
    fn test_tag_with_proc_id() {
        let input = b"<34>Oct 11 22:14:15 mymachine sshd[5678]: Connection accepted";
        let result = parse_rfc3164(input).unwrap();

        let priority = result.priority.unwrap();
        assert_eq!(priority.facility, 4);
        assert_eq!(priority.severity, 2);
        assert_eq!(result.timestamp, Some(b"Oct 11 22:14:15".as_slice()));
        assert_eq!(result.hostname, Some(b"mymachine".as_slice()));
        assert_eq!(result.tag, Some(b"sshd[5678]".as_slice()));
        assert_eq!(result.app_name, Some(b"sshd".as_slice()));
        assert_eq!(result.proc_id, Some(b"5678".as_slice()));
        assert_eq!(result.content, Some(b"Connection accepted".as_slice()));
    }

    #[test]
    fn test_tag_with_non_numeric_proc_id() {
        let input = b"<34>Oct 11 22:14:15 mymachine app[worker-1]: Task started";
        let result = parse_rfc3164(input).unwrap();

        let priority = result.priority.unwrap();
        assert_eq!(priority.facility, 4);
        assert_eq!(priority.severity, 2);
        assert_eq!(result.timestamp, Some(b"Oct 11 22:14:15".as_slice()));
        assert_eq!(result.hostname, Some(b"mymachine".as_slice()));
        assert_eq!(result.tag, Some(b"app[worker-1]".as_slice()));
        assert_eq!(result.app_name, Some(b"app".as_slice()));
        assert_eq!(result.proc_id, None); // Non-numeric proc_id should be None
        assert_eq!(result.content, Some(b"Task started".as_slice()));
    }

    #[test]
    fn test_tag_with_empty_brackets() {
        let input = b"<34>hostname app[]: message";
        let result = parse_rfc3164(input).unwrap();

        let priority = result.priority.unwrap();
        assert_eq!(priority.facility, 4);
        assert_eq!(priority.severity, 2);
        assert_eq!(result.timestamp, None);
        assert_eq!(result.hostname, Some(b"hostname".as_slice()));
        assert_eq!(result.tag, Some(b"app[]".as_slice()));
        assert_eq!(result.app_name, Some(b"app".as_slice()));
        assert_eq!(result.proc_id, None); // Empty brackets should result in None
        assert_eq!(result.content, Some(b"message".as_slice()));
    }

    #[test]
    fn test_tag_with_unclosed_bracket() {
        let input = b"<34>hostname app[123: message";
        let result = parse_rfc3164(input).unwrap();

        let priority = result.priority.unwrap();
        assert_eq!(priority.facility, 4);
        assert_eq!(priority.severity, 2);
        assert_eq!(result.timestamp, None);
        assert_eq!(result.hostname, Some(b"hostname".as_slice()));
        assert_eq!(result.tag, Some(b"app[123".as_slice()));
        assert_eq!(result.app_name, Some(b"app[123".as_slice())); // Treat entire tag as app_name
        assert_eq!(result.proc_id, None);
        assert_eq!(result.content, Some(b"message".as_slice()));
    }

    #[test]
    fn test_tag_with_only_brackets() {
        let input = b"<34>hostname [123]: message";
        let result = parse_rfc3164(input).unwrap();

        let priority = result.priority.unwrap();
        assert_eq!(priority.facility, 4);
        assert_eq!(priority.severity, 2);
        assert_eq!(result.timestamp, None);
        assert_eq!(result.hostname, Some(b"hostname".as_slice()));
        assert_eq!(result.tag, Some(b"[123]".as_slice()));
        assert_eq!(result.app_name, None); // No app_name before bracket
        assert_eq!(result.proc_id, Some(b"123".as_slice()));
        assert_eq!(result.content, Some(b"message".as_slice()));
    }

    #[test]
    fn test_parse_tag_components_directly() {
        // Test the helper function directly
        use super::parse_tag_components;

        // Simple app name
        assert_eq!(
            parse_tag_components(Some(b"su")),
            (Some(b"su".as_slice()), None)
        );

        // App name with numeric proc_id
        assert_eq!(
            parse_tag_components(Some(b"sshd[5678]")),
            (Some(b"sshd".as_slice()), Some(b"5678".as_slice()))
        );

        // App name with non-numeric proc_id
        assert_eq!(
            parse_tag_components(Some(b"app[worker-1]")),
            (Some(b"app".as_slice()), None)
        );

        // Empty brackets
        assert_eq!(
            parse_tag_components(Some(b"app[]")),
            (Some(b"app".as_slice()), None)
        );

        // Only brackets with number
        assert_eq!(
            parse_tag_components(Some(b"[123]")),
            (None, Some(b"123".as_slice()))
        );

        // None tag
        assert_eq!(parse_tag_components(None), (None, None));

        // Empty tag
        assert_eq!(parse_tag_components(Some(b"")), (None, None));

        // Unclosed bracket
        assert_eq!(
            parse_tag_components(Some(b"app[123")),
            (Some(b"app[123".as_slice()), None)
        );

        // Extra content after closing bracket (ignored)
        assert_eq!(
            parse_tag_components(Some(b"app[123]extra")),
            (Some(b"app".as_slice()), Some(b"123".as_slice()))
        );
    }
}
