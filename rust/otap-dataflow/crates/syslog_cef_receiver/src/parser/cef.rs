// SPDX-License-Identifier: Apache-2.0
// Copyright The OpenTelemetry Authors

/// CEF message structure
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CefMessage<'a> {
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
    pub(crate) fn parse_extensions(&self) -> impl Iterator<Item = (&[u8], &[u8])> {
        parse_cef_extensions(self.extensions).into_iter()
    }
}

/// Parse a CEF message
pub(super) fn parse_cef(input: &[u8]) -> Result<CefMessage<'_>, super::ParseError> {
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

fn parse_cef_extensions(extensions_bytes: &[u8]) -> Vec<(&[u8], &[u8])> {
    let mut extensions = Vec::new();
    let mut key_start = 0;
    let mut in_value = false;
    let mut key_end = 0;
    let mut i = 0;

    while i < extensions_bytes.len() {
        let byte = extensions_bytes[i];

        if !in_value && byte == b'=' {
            key_end = i;
            in_value = true;
        } else if in_value && byte == b' ' {
            // Check if next sequence starts a new key (contains =)
            let mut j = i + 1;
            // Skip spaces
            while j < extensions_bytes.len() && extensions_bytes[j] == b' ' {
                j += 1;
            }

            if j < extensions_bytes.len() {
                let remaining = &extensions_bytes[j..];
                if remaining.contains(&b'=') {
                    let key = &extensions_bytes[key_start..key_end];
                    let value = &extensions_bytes[key_end + 1..i];
                    extensions.push((key, value));

                    key_start = j;
                    i = j;
                    in_value = false;
                    continue;
                }
            }
        }
        i += 1;
    }

    // Add the last key-value pair
    if in_value && key_end > key_start && key_end < extensions_bytes.len() {
        let key = &extensions_bytes[key_start..key_end];
        let value = &extensions_bytes[key_end + 1..];
        extensions.push((key, value));
    }

    extensions
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
}
