// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Fixed-width `CURRENT` marker codec.

use crate::DecodeError;
use crate::primitives::{CURRENT_MAGIC, FILELOG_FORMAT_VERSION, Reader, Writer, crc32c};

/// Exact version 1 `CURRENT` marker size.
pub const CURRENT_BYTES: usize = 24;

/// Encodes a `CURRENT` marker selecting `generation`.
#[must_use]
pub fn encode_current(generation: u64) -> Vec<u8> {
    let mut out = Writer::new();
    out.bytes(CURRENT_MAGIC);
    out.u16(FILELOG_FORMAT_VERSION);
    out.u16(0);
    out.u64(generation);
    out.u32(crc32c(out.as_slice()));
    out.finish()
}

/// Decodes and validates one exact version 1 `CURRENT` marker.
pub fn decode_current(bytes: &[u8]) -> Result<u64, DecodeError> {
    if bytes.len() != CURRENT_BYTES {
        return Err(DecodeError::InvalidLength {
            context: "CURRENT marker",
            expected: CURRENT_BYTES,
            actual: bytes.len(),
        });
    }
    let mut input = Reader::new(bytes);
    if input.exact(8)? != CURRENT_MAGIC {
        return Err(DecodeError::BadMagic {
            context: "CURRENT marker",
        });
    }
    let version = input.u16()?;
    if version != FILELOG_FORMAT_VERSION {
        return Err(DecodeError::UnsupportedVersion {
            context: "CURRENT marker",
            found: version,
        });
    }
    let flags = input.u16()?;
    if flags != 0 {
        return Err(DecodeError::ReservedFieldNonZero {
            field: "CURRENT.flags",
            value: u64::from(flags),
        });
    }
    let generation = input.u64()?;
    let stored = input.u32()?;
    let computed = crc32c(&bytes[..20]);
    if stored != computed {
        return Err(DecodeError::ChecksumMismatch {
            context: "CURRENT marker",
            stored,
            computed,
        });
    }
    Ok(generation)
}
