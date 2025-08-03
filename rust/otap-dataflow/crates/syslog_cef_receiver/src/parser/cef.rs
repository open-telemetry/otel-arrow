use core::str;

/// CEF message structure
#[derive(Debug, Clone, PartialEq)]
pub(super) struct CefMessage<'a> {
    pub(super) version: u8,
    pub(super) device_vendor: &'a [u8],
    pub(super) device_product: &'a [u8],
    pub(super) device_version: &'a [u8],
    pub(super) signature_id: &'a [u8],
    pub(super) name: &'a [u8],
    pub(super) severity: &'a [u8],
    pub(super) extensions: Vec<(&'a [u8], &'a [u8])>,
    pub(super) input: &'a [u8],
}

/// Parse a CEF message
pub(super) fn parse_cef(input: &[u8]) -> Result<CefMessage<'_>, super::ParseError> {
    if !input.starts_with(b"CEF:") {
        return Err(super::ParseError::InvalidCef);
    }
    
    let content = &input[4..];
    
    // Find up to 8 pipe-separated parts
    let mut parts = Vec::new();
    let mut start = 0;
    let mut pipe_count = 0;
    
    for (i, &byte) in content.iter().enumerate() {
        if byte == b'|' {
            parts.push(&content[start..i]);
            start = i + 1;
            pipe_count += 1;
            if pipe_count == 7 {
                // After 7 pipes, the rest is extensions
                if start < content.len() {
                    parts.push(&content[start..]);
                }
                break;
            }
        }
    }
    
    // Add the last part if we didn't reach 7 pipes
    if pipe_count < 7 && start < content.len() {
        parts.push(&content[start..]);
    }
    
    if parts.len() < 7 {
        return Err(super::ParseError::InvalidCef);
    }
    
    let version_str = str::from_utf8(parts[0]).map_err(|_| super::ParseError::InvalidUtf8)?;
    let version: u8 = version_str.parse().map_err(|_| super::ParseError::InvalidCef)?;
    let device_vendor = parts[1];
    let device_product = parts[2];
    let device_version = parts[3];
    let signature_id = parts[4];
    let name = parts[5];
    let severity = parts[6];
    
    // Parse extensions if present
    let extensions = if parts.len() > 7 {
        parse_cef_extensions(parts[7])
    } else {
        Vec::new()
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
                if remaining.iter().any(|&b| b == b'=') {
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
        assert_eq!(result.extensions.len(), 3);
        assert_eq!(result.extensions[0], (b"src".as_slice(), b"10.0.0.1".as_slice()));
        assert_eq!(result.extensions[1], (b"dst".as_slice(), b"2.1.2.2".as_slice()));
        assert_eq!(result.extensions[2], (b"spt".as_slice(), b"1232".as_slice()));
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
        assert_eq!(result.extensions.len(), 0);
    }
}
