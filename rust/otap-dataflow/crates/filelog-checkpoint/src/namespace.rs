// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Portable version-one namespace component mapping.

/// Validate and encode the exact ID without normalization or case folding.
pub fn namespace_component(id: &str) -> Result<String, crate::EncodeError> {
    if id.is_empty()
        || id.len() > 127
        || !id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
    {
        return Err(crate::EncodeError::InvalidFieldValue {
            field: "checkpoint_id",
            reason: "checkpoint ID must be 1..=127 ASCII alphanumeric, underscore, hyphen, or dot bytes",
        });
    }
    let mut component = String::with_capacity(id.len() * 2);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in id.bytes() {
        component.push(char::from(HEX[usize::from(byte >> 4)]));
        component.push(char::from(HEX[usize::from(byte & 15)]));
    }
    Ok(component)
}
