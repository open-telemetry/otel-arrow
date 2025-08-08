// SPDX-License-Identifier: Apache-2.0
// Copyright The OpenTelemetry Authors

/// RFC 3164 message structure
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Rfc3164Message<'a> {
    pub(super) priority: crate::parser::Priority,
    pub(super) timestamp: Option<&'a [u8]>,
    pub(super) hostname: Option<&'a [u8]>,
    pub(super) tag: Option<&'a [u8]>,
    pub(super) content: Option<&'a [u8]>,
    pub(super) message: Option<&'a [u8]>,
    pub(super) input: &'a [u8],
}

/// Parse an RFC 3164 syslog message
pub(super) fn parse_rfc3164(input: &[u8]) -> Result<Rfc3164Message<'_>, crate::parser::ParseError> {
    let (priority, mut remaining) = crate::parser::parse_priority(input)?;

    // Parse timestamp (optional)
    let (timestamp, rest) = if remaining.len() >= 15 {
        // Try to parse timestamp (MMM dd HH:MM:SS format)
        let potential_ts = &remaining[..15];
        if remaining.len() > 3
            && remaining[3] == b' '
            && remaining.len() > 6
            && remaining[6] == b' '
        {
            (Some(potential_ts), &remaining[16..])
        } else {
            (None, remaining)
        }
    } else {
        (None, remaining)
    };

    remaining = rest;

    // Parse hostname and tag
    let (hostname, tag, content, message) =
        if let Some(colon_pos) = remaining.iter().position(|&b| b == b':') {
            let before_colon = &remaining[..colon_pos];
            let after_colon = &remaining[colon_pos + 1..];

            if let Some(space_pos) = before_colon.iter().rposition(|&b| b == b' ') {
                // hostname tag: message
                let hostname = Some(&before_colon[..space_pos]);
                let tag = Some(&before_colon[space_pos + 1..]);
                let message = if !after_colon.is_empty() && after_colon[0] == b' ' {
                    Some(&after_colon[1..])
                } else {
                    Some(after_colon)
                };
                (hostname, tag, None, message)
            } else {
                // tag: message (no hostname)
                let tag = Some(before_colon);
                let message = if !after_colon.is_empty() && after_colon[0] == b' ' {
                    Some(&after_colon[1..])
                } else {
                    Some(after_colon)
                };
                (None, tag, None, message)
            }
        } else {
            // No colon, treat as content
            (None, None, Some(remaining), None)
        };

    Ok(Rfc3164Message {
        priority,
        timestamp,
        hostname,
        tag,
        content,
        message,
        input,
    })
}

#[cfg(test)]
mod tests {
    use crate::parser::*;

    #[test]
    fn test_rfc3164_parsing() {
        let input = b"<34>Oct 11 22:14:15 mymachine su: 'su root' failed for lonvick on /dev/pts/8";
        let result = parse_rfc3164(input).unwrap();

        assert_eq!(result.priority.facility, 4);
        assert_eq!(result.priority.severity, 2);
        assert_eq!(result.timestamp, Some(b"Oct 11 22:14:15".as_slice()));
        assert_eq!(result.hostname, Some(b"mymachine".as_slice()));
        assert_eq!(result.tag, Some(b"su".as_slice()));
        assert_eq!(result.content, None);
        assert_eq!(
            result.message,
            Some(b"'su root' failed for lonvick on /dev/pts/8".as_slice())
        );
    }

    #[test]
    fn test_rfc3164_without_timestamp() {
        let input = b"<34>hostname tag: message content";
        let result = parse_rfc3164(input).unwrap();

        assert_eq!(result.priority.facility, 4);
        assert_eq!(result.priority.severity, 2);
        assert_eq!(result.timestamp, None);
        assert_eq!(result.hostname, Some(b"hostname".as_slice()));
        assert_eq!(result.tag, Some(b"tag".as_slice()));
        assert_eq!(result.content, None);
        assert_eq!(result.message, Some(b"message content".as_slice()));
    }

    #[test]
    fn test_rfc3164_content_only() {
        let input = b"<34>This is just content without colon";
        let result = parse_rfc3164(input).unwrap();

        assert_eq!(result.priority.facility, 4);
        assert_eq!(result.priority.severity, 2);
        assert_eq!(result.timestamp, None);
        assert_eq!(result.hostname, None);
        assert_eq!(result.tag, None);
        assert_eq!(
            result.content,
            Some(b"This is just content without colon".as_slice())
        );
        assert_eq!(result.message, None);
    }
}
