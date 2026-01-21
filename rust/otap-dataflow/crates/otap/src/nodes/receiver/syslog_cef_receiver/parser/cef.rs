// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

/// CEF message structure
#[derive(Debug, Clone, PartialEq)]
pub struct CefMessage<'a> {
    pub(super) version: u8,
    pub(super) device_vendor: &'a [u8],
    pub(super) device_product: &'a [u8],
    pub(super) device_version: &'a [u8],
    pub(super) device_event_class_id: &'a [u8],
    pub(super) name: &'a [u8],
    pub(super) severity: &'a [u8],
    pub(super) extensions: &'a [u8],
    pub(super) input: &'a [u8],
}

impl CefMessage<'_> {
    /// Parse and iterate over the extensions as key-value pairs
    pub(super) fn parse_extensions(&self) -> CefExtensionsIter<'_> {
        CefExtensionsIter::new(self.extensions)
    }
}

/// Zero-allocation helper to check if a slice needs unescaping
#[inline]
fn needs_unescaping(data: &[u8]) -> bool {
    let len = data.len();
    if len < 2 {
        return false;
    }

    // Use unchecked indexing since we know i+1 is valid
    for i in 0..len - 1 {
        if data[i] == b'\\' {
            match data[i + 1] {
                b'\\' | b'=' | b'n' | b'r' => return true,
                _ => {}
            }
        }
    }
    false
}

/// Zero-allocation in-place unescaping for CEF extension values
/// Returns the new length after unescaping
#[inline]
const fn unescape_inplace(data: &mut [u8]) -> usize {
    let mut write_pos = 0;
    let mut read_pos = 0;

    while read_pos < data.len() {
        if read_pos + 1 < data.len() && data[read_pos] == b'\\' {
            match data[read_pos + 1] {
                b'\\' => {
                    data[write_pos] = b'\\';
                    read_pos += 2;
                }
                b'=' => {
                    data[write_pos] = b'=';
                    read_pos += 2;
                }
                b'n' => {
                    data[write_pos] = b'\n';
                    read_pos += 2;
                }
                b'r' => {
                    data[write_pos] = b'\r';
                    read_pos += 2;
                }
                _ => {
                    // Not a recognized escape sequence, keep both characters
                    data[write_pos] = data[read_pos];
                    read_pos += 1;
                }
            }
        } else {
            data[write_pos] = data[read_pos];
            read_pos += 1;
        }
        write_pos += 1;
    }

    write_pos
}

/// Parse a CEF message
pub fn parse_cef(input: &[u8]) -> Result<CefMessage<'_>, super::ParseError> {
    if !input.starts_with(b"CEF:") {
        return Err(super::ParseError::InvalidCef);
    }

    let content = &input[4..];

    // Early return if content is empty
    if content.is_empty() {
        return Err(super::ParseError::EmptyCEFContent);
    }

    // Format: CEF:Version|Vendor|Product|Version|EventClassID|Name|Severity|[Extensions]
    // Find up to 8 pipe-separated parts (7 required + 1 optional extensions)
    // 1. Version (required) - CEF version, e.g., "0" or "1"
    // 2. Device Vendor (required) - String identifying the vendor
    // 3. Device Product (required) - String identifying the product
    // 4. Device Version (required) - String identifying the product version
    // 5. Device Event Class ID (required) - Unique identifier for the event type
    // 6. Name (required) - Human-readable description of the event
    // 7. Severity (required) - Severity level of the event
    // 8. Extensions (optional) - Key-value pairs with additional event details
    // Handle escaped pipes in header fields
    let mut parts: [Option<&[u8]>; 8] = [None; 8];
    let mut parts_count = 0;
    let mut start = 0;
    let mut pipe_count = 0;
    let mut i = 0;

    while i < content.len() {
        if content[i] == b'|' {
            // Check if this pipe is escaped (preceded by unescaped backslash)
            let mut escaped = false;
            if i > 0 {
                let mut backslash_count = 0;
                let mut j = i;
                while j > 0 && content[j - 1] == b'\\' {
                    backslash_count += 1;
                    j -= 1;
                }
                // If odd number of backslashes, the pipe is escaped
                escaped = backslash_count % 2 == 1;
            }

            if !escaped {
                parts[parts_count] = Some(&content[start..i]);
                parts_count += 1;
                start = i + 1;
                pipe_count += 1;
                if pipe_count == 7 {
                    // After 7 pipes, the rest is extensions
                    if start < content.len() {
                        parts[parts_count] = Some(&content[start..]);
                        parts_count += 1;
                    } else {
                        // Empty extensions
                        parts[parts_count] = Some(&[]);
                        parts_count += 1;
                    }
                    break;
                }
            }
        }
        i += 1;
    }

    // Add the last part if we didn't reach 7 pipes
    if pipe_count < 7 && start <= content.len() {
        parts[parts_count] = Some(&content[start..]);
        parts_count += 1;
    }

    if parts_count < 7 {
        return Err(super::ParseError::InvalidCef);
    }

    // Extract the 7 required fields using pattern matching
    let (
        Some(version_bytes),
        Some(device_vendor),
        Some(device_product),
        Some(device_version),
        Some(device_event_class_id),
        Some(name),
        Some(severity),
    ) = (
        parts[0], parts[1], parts[2], parts[3], parts[4], parts[5], parts[6],
    )
    else {
        return Err(super::ParseError::InvalidCef);
    };

    // Parse version according to CEF spec: supports "0", "1", "0.x", "1.x" etc.
    // Only the major version number (before the dot) is significant
    let version: u8 = match version_bytes.first() {
        Some(b'0') => 0,
        Some(b'1') => 1,
        _ => return Err(super::ParseError::InvalidCef),
    };

    // Parse extensions if present
    let extensions = if parts_count > 7 {
        parts[7].unwrap_or(&[])
    } else {
        &[]
    };

    Ok(CefMessage {
        version,
        device_vendor,
        device_product,
        device_version,
        device_event_class_id,
        name,
        severity,
        extensions,
        input,
    })
}

/// Iterator for CEF extensions that parses on-demand
pub(super) struct CefExtensionsIter<'a> {
    data: &'a [u8],
    pos: usize,
    // Scratch buffer for unescaping - reused across iterations
    scratch_buffer: Vec<u8>,
}

impl<'a> CefExtensionsIter<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: 0,
            scratch_buffer: Vec::new(), // TODO: This would allocate if the extensions provided in the input have to be unescaped. Could we avoid this allocation?
        }
    }

    pub(super) fn next_extension(&mut self) -> Option<(&[u8], &[u8])> {
        if self.pos >= self.data.len() {
            return None;
        }

        // Skip leading spaces
        while self.pos < self.data.len() && self.data[self.pos] == b' ' {
            self.pos += 1;
        }

        if self.pos >= self.data.len() {
            return None;
        }

        let key_start = self.pos;

        // Find the '=' separator for the key
        while self.pos < self.data.len() && self.data[self.pos] != b'=' {
            self.pos += 1;
        }

        if self.pos >= self.data.len() {
            // No '=' found, invalid extension
            return None;
        }

        let key_end = self.pos;

        // Skip empty keys
        if key_start == key_end {
            self.pos += 1; // Skip the '='
            // Try to find the next extension
            while self.pos < self.data.len() && self.data[self.pos] != b' ' {
                self.pos += 1;
            }
            return self.next_extension();
        }

        self.pos += 1; // Skip '='

        if self.pos >= self.data.len() {
            // Empty value at end
            return Some((&self.data[key_start..key_end], &[]));
        }

        let value_start = self.pos;
        let mut escaped = false;

        // Parse value - handle escaping according to CEF spec
        while self.pos < self.data.len() {
            if escaped {
                // Skip this character, it's escaped
                escaped = false;
                self.pos += 1;
                continue;
            }

            if self.data[self.pos] == b'\\' && self.pos + 1 < self.data.len() {
                // Next character is escaped (only if there is a next character)
                escaped = true;
                self.pos += 1;
                continue;
            }

            // Look for unescaped space that could be a separator
            if self.data[self.pos] == b' ' {
                // Peek ahead to see if this is a key=value pattern
                let mut lookahead = self.pos + 1;

                // Skip spaces
                while lookahead < self.data.len() && self.data[lookahead] == b' ' {
                    lookahead += 1;
                }

                // Check if we have a valid key pattern (alphanumeric followed by =)
                if lookahead < self.data.len() {
                    let mut key_end_pos = lookahead;
                    let mut found_key = false;

                    // A valid key should contain alphanumeric chars and end with =
                    while key_end_pos < self.data.len() {
                        let ch = self.data[key_end_pos];
                        if ch == b'=' && key_end_pos > lookahead {
                            // Found a valid key pattern
                            found_key = true;
                            break;
                        } else if !ch.is_ascii_alphanumeric() && ch != b'_' && ch != b'-' {
                            // Not a valid key character
                            break;
                        }
                        key_end_pos += 1;
                    }

                    if found_key {
                        // This space is a separator
                        break;
                    }
                }
            }

            self.pos += 1;
        }

        let key = &self.data[key_start..key_end];
        let raw_value = &self.data[value_start..self.pos];

        // Move position to start of next key
        while self.pos < self.data.len() && self.data[self.pos] == b' ' {
            self.pos += 1;
        }

        // Handle unescaping efficiently
        let value = if needs_unescaping(raw_value) {
            // Reuse scratch buffer to avoid allocations
            self.scratch_buffer.clear();
            self.scratch_buffer.extend_from_slice(raw_value);
            let new_len = unescape_inplace(&mut self.scratch_buffer);
            self.scratch_buffer.truncate(new_len);
            &self.scratch_buffer[..]
        } else {
            raw_value
        };

        Some((key, value))
    }

    /// Collect all extensions into a Vec, allocating only when necessary
    #[cfg(test)]
    pub(super) fn collect_all(mut self) -> Vec<(Vec<u8>, Vec<u8>)> {
        let mut result = Vec::new();
        while let Some((key, value)) = self.next_extension() {
            result.push((key.to_vec(), value.to_vec()));
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syslog_cef_receiver::parser;

    #[test]
    fn test_cef_parsing() {
        let input = b"CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232";
        let result = parse_cef(input).unwrap();

        assert_eq!(result.version, 0);
        assert_eq!(result.device_vendor, b"Security".as_slice());
        assert_eq!(result.device_product, b"threatmanager".as_slice());
        assert_eq!(result.device_version, b"1.0".as_slice());
        assert_eq!(result.device_event_class_id, b"100".as_slice());
        assert_eq!(result.name, b"worm successfully stopped".as_slice());
        assert_eq!(result.severity, b"10".as_slice());

        let extensions = result.parse_extensions().collect_all();
        assert_eq!(extensions.len(), 3);
        assert_eq!(
            (extensions[0].0.as_slice(), extensions[0].1.as_slice()),
            (b"src".as_slice(), b"10.0.0.1".as_slice())
        );
        assert_eq!(
            (extensions[1].0.as_slice(), extensions[1].1.as_slice()),
            (b"dst".as_slice(), b"2.1.2.2".as_slice())
        );
        assert_eq!(
            (extensions[2].0.as_slice(), extensions[2].1.as_slice()),
            (b"spt".as_slice(), b"1232".as_slice())
        );
    }

    #[test]
    fn test_cef_version_with_minor() {
        // Test version 0.5
        let input = b"CEF:0.5|Security|threatmanager|1.0|100|worm successfully stopped|10|";
        let result = parse_cef(input).unwrap();
        assert_eq!(result.version, 0);

        // Test version 1.2
        let input = b"CEF:1.2|Security|threatmanager|1.0|100|worm successfully stopped|10|";
        let result = parse_cef(input).unwrap();
        assert_eq!(result.version, 1);

        // Test version 0.0
        let input = b"CEF:0.0|Security|threatmanager|1.0|100|worm successfully stopped|10|";
        let result = parse_cef(input).unwrap();
        assert_eq!(result.version, 0);

        // Test invalid major version
        let input = b"CEF:2.0|Security|threatmanager|1.0|100|worm successfully stopped|10|";
        let result = parse_cef(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_cef_without_extensions() {
        let input = b"CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|10|";
        let result = parse_cef(input).unwrap();

        assert_eq!(result.version, 0);
        assert_eq!(result.device_vendor, b"Security".as_slice());
        assert_eq!(result.device_product, b"threatmanager".as_slice());
        assert_eq!(result.device_version, b"1.0".as_slice());
        assert_eq!(result.device_event_class_id, b"100".as_slice());
        assert_eq!(result.name, b"worm successfully stopped".as_slice());
        assert_eq!(result.severity, b"10".as_slice());

        let extensions = result.parse_extensions().collect_all();
        assert_eq!(extensions.len(), 0);
    }

    #[test]
    fn test_cef_extensions_with_spaces_in_values() {
        let input = b"CEF:0|V|P|1.0|100|name|10|msg=This is a message with spaces src=10.0.0.1";
        let result = parse_cef(input).unwrap();

        let extensions = result.parse_extensions().collect_all();
        assert_eq!(extensions.len(), 2);
        assert_eq!(
            (extensions[0].0.as_slice(), extensions[0].1.as_slice()),
            (
                b"msg".as_slice(),
                b"This is a message with spaces".as_slice()
            )
        );
        assert_eq!(
            (extensions[1].0.as_slice(), extensions[1].1.as_slice()),
            (b"src".as_slice(), b"10.0.0.1".as_slice())
        );
    }

    #[test]
    fn test_cef_extensions_with_equals_in_values() {
        let input = b"CEF:0|V|P|1.0|100|name|10|equation=a=b+c src=10.0.0.1";
        let result = parse_cef(input).unwrap();

        let extensions = result.parse_extensions().collect_all();
        assert_eq!(extensions.len(), 2);
        assert_eq!(
            (extensions[0].0.as_slice(), extensions[0].1.as_slice()),
            (b"equation".as_slice(), b"a=b+c".as_slice())
        );
        assert_eq!(
            (extensions[1].0.as_slice(), extensions[1].1.as_slice()),
            (b"src".as_slice(), b"10.0.0.1".as_slice())
        );
    }

    #[test]
    fn test_cef_extensions_empty_value() {
        let input = b"CEF:0|V|P|1.0|100|name|10|empty= src=10.0.0.1";
        let result = parse_cef(input).unwrap();

        let extensions = result.parse_extensions().collect_all();
        assert_eq!(extensions.len(), 2);
        assert_eq!(
            (extensions[0].0.as_slice(), extensions[0].1.as_slice()),
            (b"empty".as_slice(), b"".as_slice())
        );
        assert_eq!(
            (extensions[1].0.as_slice(), extensions[1].1.as_slice()),
            (b"src".as_slice(), b"10.0.0.1".as_slice())
        );
    }

    #[test]
    fn test_cef_extensions_trailing_spaces() {
        let input = b"CEF:0|V|P|1.0|100|name|10|value=has trailing spaces   next=value";
        let result = parse_cef(input).unwrap();

        let extensions = result.parse_extensions().collect_all();
        assert_eq!(extensions.len(), 2);
        assert_eq!(
            (extensions[0].0.as_slice(), extensions[0].1.as_slice()),
            (b"value".as_slice(), b"has trailing spaces".as_slice())
        );
        assert_eq!(
            (extensions[1].0.as_slice(), extensions[1].1.as_slice()),
            (b"next".as_slice(), b"value".as_slice())
        );
    }

    #[test]
    fn test_cef_extensions_escaped_characters() {
        let input = b"CEF:0|V|P|1.0|100|name|10|msg=escaped\\=equals src=10.0.0.1";
        let result = parse_cef(input).unwrap();

        let extensions = result.parse_extensions().collect_all();
        assert_eq!(extensions.len(), 2);
        // Now properly unescaped
        assert_eq!(
            (extensions[0].0.as_slice(), extensions[0].1.as_slice()),
            (b"msg".as_slice(), b"escaped=equals".as_slice())
        );
        assert_eq!(
            (extensions[1].0.as_slice(), extensions[1].1.as_slice()),
            (b"src".as_slice(), b"10.0.0.1".as_slice())
        );
    }

    #[test]
    fn test_header_pipe_escaping() {
        let input =
            b"CEF:0|Security|threatmanager|1.0|100|detected a \\| in message|10|src=10.0.0.1";
        let result = parse_cef(input).unwrap();

        assert_eq!(result.version, 0);
        assert_eq!(result.device_vendor, b"Security".as_slice());
        assert_eq!(result.device_product, b"threatmanager".as_slice());
        assert_eq!(result.name, b"detected a \\| in message".as_slice()); // Raw bytes preserved
        assert_eq!(result.severity, b"10".as_slice());
    }

    #[test]
    fn test_extension_unescaping_comprehensive() {
        let input = b"CEF:0|V|P|1.0|100|name|10|msg=Line1\\nLine2 path=C:\\\\temp equals=a\\=b";
        let result = parse_cef(input).unwrap();

        let extensions = result.parse_extensions().collect_all();
        assert_eq!(extensions.len(), 3);

        assert_eq!(
            (extensions[0].0.as_slice(), extensions[0].1.as_slice()),
            (b"msg".as_slice(), b"Line1\nLine2".as_slice())
        );
        assert_eq!(
            (extensions[1].0.as_slice(), extensions[1].1.as_slice()),
            (b"path".as_slice(), b"C:\\temp".as_slice())
        );
        assert_eq!(
            (extensions[2].0.as_slice(), extensions[2].1.as_slice()),
            (b"equals".as_slice(), b"a=b".as_slice())
        );
    }

    // Edge case tests to ensure no panic occurs
    #[test]
    fn test_malformed_cef_header() {
        // Test with just "CEF:" and nothing else
        let input = b"CEF:";
        let result = parse_cef(input);
        assert!(result.is_err());
        assert!(matches!(result, Err(parser::ParseError::EmptyCEFContent)));

        // Test with incomplete header
        let input = b"CEF:0";
        let result = parse_cef(input);
        assert!(result.is_err());
        assert!(matches!(result, Err(parser::ParseError::InvalidCef)));
    }

    #[test]
    fn test_cef_with_empty_fields() {
        // Test with empty fields between pipes
        let input = b"CEF:0|||||||";
        let result = parse_cef(input).unwrap();
        assert_eq!(result.version, 0);
        assert_eq!(result.device_vendor, b"");
        assert_eq!(result.device_product, b"");
        assert_eq!(result.device_version, b"");
        assert_eq!(result.device_event_class_id, b"");
        assert_eq!(result.name, b"");
        assert_eq!(result.severity, b"");
        assert_eq!(result.extensions, b"");

        let extensions = result.parse_extensions().collect_all();
        assert_eq!(extensions.len(), 0);
    }

    #[test]
    fn test_cef_with_insufficient_fields() {
        // Test with only 4 pipes (5 parts) - missing name and severity
        let input = b"CEF:0|vendor|product|version|id";
        let result = parse_cef(input);
        assert!(result.is_err());
        assert!(matches!(result, Err(parser::ParseError::InvalidCef)));

        // Test with only 5 pipes (6 parts) - missing severity
        let input = b"CEF:0|vendor|product|version|id|name";
        let result = parse_cef(input);
        assert!(result.is_err());
        assert!(matches!(result, Err(parser::ParseError::InvalidCef)));

        // Test with exactly 6 pipes (7 parts) - valid without extensions
        let input = b"CEF:0|vendor|product|version|id|name|10";
        let result = parse_cef(input);
        assert!(result.is_ok());

        // Verify all fields are correctly parsed
        let msg = result.unwrap();
        assert_eq!(msg.version, 0);
        assert_eq!(msg.device_vendor, b"vendor");
        assert_eq!(msg.device_product, b"product");
        assert_eq!(msg.device_version, b"version");
        assert_eq!(msg.device_event_class_id, b"id");
        assert_eq!(msg.name, b"name");
        assert_eq!(msg.severity, b"10");
        assert_eq!(msg.extensions, b""); // No extensions
        assert_eq!(msg.input, input); // Original input preserved

        // Test with 7 pipes (8 parts) - valid with extensions
        let input = b"CEF:0|vendor|product|1.0|eventId|Event Name|5|src=127.0.0.1 dst=192.168.1.1";
        let result = parse_cef(input);
        assert!(result.is_ok());

        let msg = result.unwrap();
        assert_eq!(msg.version, 0);
        assert_eq!(msg.device_vendor, b"vendor");
        assert_eq!(msg.device_product, b"product");
        assert_eq!(msg.device_version, b"1.0");
        assert_eq!(msg.device_event_class_id, b"eventId");
        assert_eq!(msg.name, b"Event Name");
        assert_eq!(msg.severity, b"5");
        assert_eq!(msg.extensions, b"src=127.0.0.1 dst=192.168.1.1");
        assert_eq!(msg.input, input);

        // Verify extensions parse correctly
        let extensions = msg.parse_extensions().collect_all();
        assert_eq!(extensions.len(), 2);
        assert_eq!(extensions[0].0.as_slice(), b"src");
        assert_eq!(extensions[0].1.as_slice(), b"127.0.0.1");
        assert_eq!(extensions[1].0.as_slice(), b"dst");
        assert_eq!(extensions[1].1.as_slice(), b"192.168.1.1");
    }

    #[test]
    fn test_extension_parsing_edge_cases() {
        // Test extension with only equals sign
        let input = b"CEF:0|V|P|1.0|100|name|10|=";
        let result = parse_cef(input).unwrap();
        let extensions = result.parse_extensions().collect_all();
        assert_eq!(extensions.len(), 0); // Should handle gracefully

        // Test extension with multiple equals in a row
        let input = b"CEF:0|V|P|1.0|100|name|10|===value";
        let result = parse_cef(input).unwrap();
        let extensions = result.parse_extensions().collect_all();
        assert_eq!(extensions.len(), 0); // Empty key should be skipped

        // Test extension ending with backslash
        let input = b"CEF:0|V|P|1.0|100|name|10|key=value\\";
        let result = parse_cef(input).unwrap();
        let extensions = result.parse_extensions().collect_all();
        assert_eq!(extensions.len(), 1);
        assert_eq!(
            (extensions[0].0.as_slice(), extensions[0].1.as_slice()),
            (b"key".as_slice(), b"value\\".as_slice())
        );
    }

    #[test]
    fn test_escaped_backslash_at_end() {
        // Test with escaped pipe - single backslash escapes the pipe
        let input = b"CEF:0|V|P|1.0|100|name\\|10|";
        let result = parse_cef(input).unwrap();
        assert_eq!(result.version, 0);
        assert_eq!(result.device_vendor, b"V".as_slice());
        assert_eq!(result.device_product, b"P".as_slice());
        assert_eq!(result.device_version, b"1.0".as_slice());
        assert_eq!(result.device_event_class_id, b"100".as_slice());
        assert_eq!(result.name, b"name\\|10".as_slice()); // The escaped pipe is part of the name field
        assert_eq!(result.severity, b"".as_slice()); // Empty severity field
        assert_eq!(result.extensions, b"".as_slice()); // Empty extensions

        // Test extension value ending with backslash
        let input = b"CEF:0|V|P|1.0|100|name|10|key=val\\";
        let result = parse_cef(input).unwrap();
        assert_eq!(result.version, 0);
        assert_eq!(result.device_vendor, b"V".as_slice());
        assert_eq!(result.device_product, b"P".as_slice());
        assert_eq!(result.device_version, b"1.0".as_slice());
        assert_eq!(result.device_event_class_id, b"100".as_slice());
        assert_eq!(result.name, b"name".as_slice());
        assert_eq!(result.severity, b"10".as_slice());

        let extensions = result.parse_extensions().collect_all();
        assert_eq!(extensions.len(), 1);
        assert_eq!(
            (extensions[0].0.as_slice(), extensions[0].1.as_slice()),
            (b"key".as_slice(), b"val\\".as_slice()) // Trailing backslash preserved
        );
    }

    #[test]
    fn test_very_long_escape_sequences() {
        // Test with many consecutive backslashes
        let input = b"CEF:0|V|P|1.0|100|name|10|key=\\\\\\\\\\\\";
        let result = parse_cef(input).unwrap();
        assert_eq!(result.version, 0);
        assert_eq!(result.device_vendor, b"V".as_slice());
        assert_eq!(result.device_product, b"P".as_slice());

        let extensions = result.parse_extensions().collect_all();
        assert_eq!(extensions.len(), 1);
        assert_eq!(
            (extensions[0].0.as_slice(), extensions[0].1.as_slice()),
            (b"key".as_slice(), b"\\\\\\".as_slice()) // 6 backslashes become 3
        );
    }

    #[test]
    fn test_invalid_utf8_sequences() {
        // Test with invalid UTF-8 bytes
        let mut input = b"CEF:0|V|P|1.0|100|name|10|key=".to_vec();
        input.push(0xFF);
        input.push(0xFE);
        let result = parse_cef(&input).unwrap();

        assert_eq!(result.version, 0);
        assert_eq!(result.device_vendor, b"V".as_slice());
        assert_eq!(result.device_product, b"P".as_slice());
        assert_eq!(result.device_version, b"1.0".as_slice());
        assert_eq!(result.device_event_class_id, b"100".as_slice());
        assert_eq!(result.name, b"name".as_slice());
        assert_eq!(result.severity, b"10".as_slice());

        let extensions = result.parse_extensions().collect_all();
        assert_eq!(extensions.len(), 1);
        assert_eq!(extensions[0].0.as_slice(), b"key".as_slice());
        assert_eq!(extensions[0].1.as_slice(), &[0xFF, 0xFE]); // Raw bytes preserved
    }
}
