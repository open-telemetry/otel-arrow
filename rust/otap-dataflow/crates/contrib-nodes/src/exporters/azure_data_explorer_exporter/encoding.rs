// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! JSON encoding helpers for Azure Data Explorer request rows.

use bytes::BytesMut;

const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

pub(super) trait JsonBuf {
    fn push_byte(&mut self, byte: u8);
    fn push_slice(&mut self, bytes: &[u8]);
}

impl JsonBuf for Vec<u8> {
    #[inline]
    fn push_byte(&mut self, byte: u8) {
        self.push(byte);
    }

    #[inline]
    fn push_slice(&mut self, bytes: &[u8]) {
        self.extend_from_slice(bytes);
    }
}

impl JsonBuf for BytesMut {
    #[inline]
    fn push_byte(&mut self, byte: u8) {
        use bytes::BufMut;
        self.put_u8(byte);
    }

    #[inline]
    fn push_slice(&mut self, bytes: &[u8]) {
        self.extend_from_slice(bytes);
    }
}

#[inline]
fn escape_json_string(input: &[u8], output: &mut impl JsonBuf) {
    let lossy = String::from_utf8_lossy(input);
    let text = lossy.as_bytes();
    let mut start = 0;
    for (index, &byte) in text.iter().enumerate() {
        match byte {
            b'"' | b'\\' | b'\n' | b'\r' | b'\t' | 0..=0x1f => {
                if start < index {
                    output.push_slice(&text[start..index]);
                }
                match byte {
                    b'"' => output.push_slice(b"\\\""),
                    b'\\' => output.push_slice(b"\\\\"),
                    b'\n' => output.push_slice(b"\\n"),
                    b'\r' => output.push_slice(b"\\r"),
                    b'\t' => output.push_slice(b"\\t"),
                    _ => {
                        output.push_slice(b"\\u00");
                        output.push_byte(HEX_CHARS[(byte >> 4) as usize]);
                        output.push_byte(HEX_CHARS[(byte & 0x0f) as usize]);
                    }
                }
                start = index + 1;
            }
            _ => {}
        }
    }
    if start < text.len() {
        output.push_slice(&text[start..]);
    }
}

#[inline]
pub(super) fn write_json_string(input: &[u8], output: &mut impl JsonBuf) {
    output.push_byte(b'"');
    escape_json_string(input, output);
    output.push_byte(b'"');
}

#[inline]
pub(super) fn write_json_hex(bytes: &[u8], output: &mut impl JsonBuf) {
    output.push_byte(b'"');
    for &byte in bytes {
        output.push_byte(HEX_CHARS[(byte >> 4) as usize]);
        output.push_byte(HEX_CHARS[(byte & 0x0f) as usize]);
    }
    output.push_byte(b'"');
}

#[cfg(test)]
mod tests {
    use super::{write_json_hex, write_json_string};

    /// Scenario: ADX JSON escaping receives quotes, controls, Unicode, and malformed UTF-8.
    /// Guarantees: output is valid JSON and matches serde's lossy-string encoding.
    #[test]
    fn json_string_encoding_is_valid_and_lossy() {
        for input in [
            b"simple".as_slice(),
            b"quote\" slash\\ newline\n".as_slice(),
            b"\x00\x1f\x7f".as_slice(),
            b"valid\xffsuffix".as_slice(),
        ] {
            let mut output = Vec::new();
            write_json_string(input, &mut output);
            let expected = serde_json::to_vec(&String::from_utf8_lossy(input)).expect("serialize");
            assert_eq!(output, expected);
        }
    }

    /// Scenario: ADX hexadecimal encoding receives arbitrary bytes.
    /// Guarantees: output is a lowercase hexadecimal JSON string.
    #[test]
    fn hexadecimal_json_encoding_is_lowercase() {
        let mut output = Vec::new();
        write_json_hex(&[0x00, 0xab, 0xff], &mut output);

        assert_eq!(output, br#""00abff""#);
    }
}
