// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

/// CEF message structure
#[derive(Debug, Clone, PartialEq)]
pub struct CefMessage<'a> {
    pub(super) version: u8,
    pub(super) device_vendor: &'a [u8],
    pub(super) device_product: &'a [u8],
    pub(super) device_version: &'a [u8],
    pub(super) signature_id: &'a [u8],
    pub(super) name: &'a [u8],
    pub(super) severity: &'a [u8],
    pub(super) extensions: &'a [u8],
    pub(super) input: &'a [u8],
}

impl CefMessage<'_> {
    /// Parse and iterate over the extensions as key-value pairs
    pub(crate) fn parse_extensions(&self) -> CefExtensionsIter<'_> {
        CefExtensionsIter::new(self.extensions)
    }
}

/// Parse a CEF message
pub fn parse_cef(input: &[u8]) -> Result<CefMessage<'_>, super::ParseError> {
    if !input.starts_with(b"CEF:") {
        return Err(super::ParseError::InvalidCef);
    }

    let content = &input[4..];

    // Find up to 8 pipe-separated parts (7 required + 1 optional extensions)
    let mut parts: [Option<&[u8]>; 8] = [None; 8];
    let mut parts_count = 0;
    let mut start = 0;
    let mut pipe_count = 0;

    for (i, &byte) in content.iter().enumerate() {
        if byte == b'|' {
            parts[parts_count] = Some(&content[start..i]);
            parts_count += 1;
            start = i + 1;
            pipe_count += 1;
            if pipe_count == 7 {
                // After 7 pipes, the rest is extensions
                if start < content.len() {
                    parts[parts_count] = Some(&content[start..]);
                    parts_count += 1;
                }
                break;
            }
        }
    }

    // Add the last part if we didn't reach 7 pipes
    if pipe_count < 7 && start < content.len() {
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
        Some(signature_id),
        Some(name),
        Some(severity),
    ) = (
        parts[0], parts[1], parts[2], parts[3], parts[4], parts[5], parts[6],
    )
    else {
        return Err(super::ParseError::InvalidCef);
    };

    let version: u8 = match version_bytes {
        b"0" => 0,
        b"1" => 1,
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
        signature_id,
        name,
        severity,
        extensions,
        input,
    })
}

/// Iterator for CEF extensions that parses on-demand
pub(crate) struct CefExtensionsIter<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> CefExtensionsIter<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }
}

impl<'a> Iterator for CefExtensionsIter<'a> {
    type Item = (&'a [u8], &'a [u8]);

    fn next(&mut self) -> Option<Self::Item> {
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
            return None;
        }

        let key_end = self.pos;
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

            if self.data[self.pos] == b'\\' {
                // Next character is escaped
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
        let value = &self.data[value_start..self.pos];

        // Move position to start of next key
        while self.pos < self.data.len() && self.data[self.pos] == b' ' {
            self.pos += 1;
        }

        Some((key, value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cef_parsing() {
        let input = b"CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232";
        let result = parse_cef(input).unwrap();

        assert_eq!(result.version, 0);
        assert_eq!(result.device_vendor, b"Security".as_slice());
        assert_eq!(result.device_product, b"threatmanager".as_slice());
        assert_eq!(result.device_version, b"1.0".as_slice());
        assert_eq!(result.signature_id, b"100".as_slice());
        assert_eq!(result.name, b"worm successfully stopped".as_slice());
        assert_eq!(result.severity, b"10".as_slice());

        let extensions: Vec<_> = result.parse_extensions().collect();
        assert_eq!(extensions.len(), 3);
        assert_eq!(extensions[0], (b"src".as_slice(), b"10.0.0.1".as_slice()));
        assert_eq!(extensions[1], (b"dst".as_slice(), b"2.1.2.2".as_slice()));
        assert_eq!(extensions[2], (b"spt".as_slice(), b"1232".as_slice()));
    }

    #[test]
    fn test_cef_without_extensions() {
        let input = b"CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|10|";
        let result = parse_cef(input).unwrap();

        assert_eq!(result.version, 0);
        assert_eq!(result.device_vendor, b"Security".as_slice());
        assert_eq!(result.device_product, b"threatmanager".as_slice());
        assert_eq!(result.device_version, b"1.0".as_slice());
        assert_eq!(result.signature_id, b"100".as_slice());
        assert_eq!(result.name, b"worm successfully stopped".as_slice());
        assert_eq!(result.severity, b"10".as_slice());

        let extensions: Vec<_> = result.parse_extensions().collect();
        assert_eq!(extensions.len(), 0);
    }

    #[test]
    fn test_cef_extensions_with_spaces_in_values() {
        let input = b"CEF:0|V|P|1.0|100|name|10|msg=This is a message with spaces src=10.0.0.1";
        let result = parse_cef(input).unwrap();

        let extensions: Vec<_> = result.parse_extensions().collect();
        assert_eq!(extensions.len(), 2);
        assert_eq!(
            extensions[0],
            (
                b"msg".as_slice(),
                b"This is a message with spaces".as_slice()
            )
        );
        assert_eq!(extensions[1], (b"src".as_slice(), b"10.0.0.1".as_slice()));
    }

    #[test]
    fn test_cef_extensions_with_equals_in_values() {
        let input = b"CEF:0|V|P|1.0|100|name|10|equation=a=b+c src=10.0.0.1";
        let result = parse_cef(input).unwrap();

        let extensions: Vec<_> = result.parse_extensions().collect();
        assert_eq!(extensions.len(), 2);
        assert_eq!(extensions[0], (b"equation".as_slice(), b"a=b+c".as_slice()));
        assert_eq!(extensions[1], (b"src".as_slice(), b"10.0.0.1".as_slice()));
    }

    #[test]
    fn test_cef_extensions_empty_value() {
        let input = b"CEF:0|V|P|1.0|100|name|10|empty= src=10.0.0.1";
        let result = parse_cef(input).unwrap();

        let extensions: Vec<_> = result.parse_extensions().collect();
        assert_eq!(extensions.len(), 2);
        assert_eq!(extensions[0], (b"empty".as_slice(), b"".as_slice()));
        assert_eq!(extensions[1], (b"src".as_slice(), b"10.0.0.1".as_slice()));
    }

    #[test]
    fn test_cef_extensions_trailing_spaces() {
        let input = b"CEF:0|V|P|1.0|100|name|10|value=has trailing spaces   next=value";
        let result = parse_cef(input).unwrap();

        let extensions: Vec<_> = result.parse_extensions().collect();
        assert_eq!(extensions.len(), 2);
        assert_eq!(
            extensions[0],
            (b"value".as_slice(), b"has trailing spaces".as_slice())
        );
        assert_eq!(extensions[1], (b"next".as_slice(), b"value".as_slice()));
    }

    #[test]
    fn test_cef_extensions_escaped_characters() {
        let input = b"CEF:0|V|P|1.0|100|name|10|msg=escaped\\=equals src=10.0.0.1";
        let result = parse_cef(input).unwrap();

        let extensions: Vec<_> = result.parse_extensions().collect();
        assert_eq!(extensions.len(), 2);
        // Note: The escaped sequence is preserved in the raw bytes
        assert_eq!(
            extensions[0],
            (b"msg".as_slice(), b"escaped\\=equals".as_slice())
        );
        assert_eq!(extensions[1], (b"src".as_slice(), b"10.0.0.1".as_slice()));
    }
}
