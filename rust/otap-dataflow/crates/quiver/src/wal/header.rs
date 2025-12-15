// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! WAL file header encoding and validation.
//!
//! Every WAL file starts with a variable-size header (38 bytes for version 1):
//!
//! ```text
//! ┌────────────┬─────────┬──────────────┬──────────────────┬────────────────────┐
//! │ magic (10) │ ver (2) │ hdr_size (2) │ segment_hash (16)│ wal_pos_start (8)  │
//! └────────────┴─────────┴──────────────┴──────────────────┴────────────────────┘
//! ```
//!
//! - **magic**: `b"QUIVER\0WAL"` identifies the file type
//! - **version**: Format version (currently 1)
//! - **header_size**: Total size of the header in bytes (allows future expansion)
//! - **segment_hash**: MD5 of segment configuration; mismatches reject the file
//! - **wal_pos_start**: WAL stream position at the start of this file. Used to
//!   maintain a stable coordinate system across WAL rotations and purges.
//!   For the first WAL file this is 0; after rotation it equals the WAL position
//!   at the end of the previous file.
//!
//! The `header_size` field enables future versions to add fields without breaking
//! backwards compatibility. Readers should read `header_size` bytes total and
//! ignore any unknown trailing fields.

use std::io::{Read, Seek, SeekFrom, Write};

use super::{WAL_MAGIC, WalError};

pub(crate) const WAL_VERSION: u16 = 1;

/// Minimum header size to read magic, version, and header_size fields.
/// Future versions may increase the full header size, but this prefix is stable.
pub(crate) const WAL_HEADER_MIN_LEN: usize = 14; // magic (10) + version (2) + header_size (2)

/// Header size for version 1.
/// Layout: magic (10) + version (2) + header_size (2) + segment_hash (16) +
///         wal_pos_start (8) = 38 bytes
const HEADER_V1_LEN: usize = 38;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct WalHeader {
    pub segment_cfg_hash: [u8; 16],
    /// WAL stream position at the start of this file.
    /// This enables maintaining a stable coordinate system across file rotations.
    /// Entries in this file have WAL positions starting from `wal_position_start`.
    pub wal_position_start: u64,
    /// Actual header size in bytes.
    /// For newly constructed headers, this is the current version's size.
    /// When reading files, this reflects the file's declared header size.
    pub header_size: u16,
}

impl WalHeader {
    pub fn new(segment_cfg_hash: [u8; 16]) -> Self {
        Self {
            segment_cfg_hash,
            wal_position_start: 0,
            header_size: HEADER_V1_LEN as u16,
        }
    }

    pub fn with_base_offset(segment_cfg_hash: [u8; 16], wal_position_start: u64) -> Self {
        Self {
            segment_cfg_hash,
            wal_position_start,
            header_size: HEADER_V1_LEN as u16,
        }
    }

    /// Returns the size in bytes of the encoded header.
    pub fn encoded_len(&self) -> u64 {
        self.header_size as u64
    }

    pub fn encode(&self) -> [u8; HEADER_V1_LEN] {
        let mut buf = [0u8; HEADER_V1_LEN];
        let mut cursor = 0;

        // Magic
        buf[cursor..cursor + WAL_MAGIC.len()].copy_from_slice(WAL_MAGIC);
        cursor += WAL_MAGIC.len();

        // Version
        buf[cursor..cursor + 2].copy_from_slice(&WAL_VERSION.to_le_bytes());
        cursor += 2;

        // Header size (allows readers to skip unknown fields in future versions)
        buf[cursor..cursor + 2].copy_from_slice(&(HEADER_V1_LEN as u16).to_le_bytes());
        cursor += 2;

        // Segment config hash
        buf[cursor..cursor + 16].copy_from_slice(&self.segment_cfg_hash);
        cursor += 16;

        // WAL position start
        buf[cursor..cursor + 8].copy_from_slice(&self.wal_position_start.to_le_bytes());

        buf
    }

    pub fn write_to(&self, file: &mut (impl Write + Seek)) -> Result<(), WalError> {
        let _ = file.seek(SeekFrom::Start(0))?;
        file.write_all(&self.encode())?;
        file.flush()?;
        Ok(())
    }

    /// Reads the header size from a file without consuming the full header.
    /// Returns the header size in bytes.
    pub fn read_header_size(file: &mut (impl Read + Seek)) -> Result<u16, WalError> {
        let _ = file.seek(SeekFrom::Start(0))?;
        let mut buf = [0u8; WAL_HEADER_MIN_LEN];
        file.read_exact(&mut buf)?;

        // Validate magic
        if &buf[0..WAL_MAGIC.len()] != WAL_MAGIC {
            return Err(WalError::InvalidHeader("magic mismatch"));
        }

        // Validate version
        let version = u16::from_le_bytes([buf[WAL_MAGIC.len()], buf[WAL_MAGIC.len() + 1]]);
        if version != WAL_VERSION {
            return Err(WalError::InvalidHeader("unsupported version"));
        }

        // Read header size
        let header_size = u16::from_le_bytes([buf[WAL_MAGIC.len() + 2], buf[WAL_MAGIC.len() + 3]]);

        if (header_size as usize) < WAL_HEADER_MIN_LEN {
            return Err(WalError::InvalidHeader("header size too small"));
        }

        Ok(header_size)
    }

    pub fn read_from(file: &mut (impl Read + Seek)) -> Result<Self, WalError> {
        let _ = file.seek(SeekFrom::Start(0))?;

        // First read minimum header to get the actual header size
        let mut min_buf = [0u8; WAL_HEADER_MIN_LEN];
        file.read_exact(&mut min_buf)?;

        // Validate magic
        if &min_buf[0..WAL_MAGIC.len()] != WAL_MAGIC {
            return Err(WalError::InvalidHeader("magic mismatch"));
        }

        // Validate version
        let version = u16::from_le_bytes([min_buf[WAL_MAGIC.len()], min_buf[WAL_MAGIC.len() + 1]]);
        if version != WAL_VERSION {
            return Err(WalError::InvalidHeader("unsupported version"));
        }

        // Read header size
        let header_size =
            u16::from_le_bytes([min_buf[WAL_MAGIC.len() + 2], min_buf[WAL_MAGIC.len() + 3]]);
        let header_size = header_size as usize;

        if header_size < WAL_HEADER_MIN_LEN {
            return Err(WalError::InvalidHeader("header size too small"));
        }

        // Now read the remaining bytes
        let remaining = header_size - WAL_HEADER_MIN_LEN;
        let mut remaining_buf = vec![0u8; remaining];
        file.read_exact(&mut remaining_buf)?;

        // Combine into full buffer for decoding
        let mut full_buf = Vec::with_capacity(header_size);
        full_buf.extend_from_slice(&min_buf);
        full_buf.extend_from_slice(&remaining_buf);

        Self::decode(&full_buf)
    }

    pub fn decode(buf: &[u8]) -> Result<Self, WalError> {
        if buf.len() < WAL_HEADER_MIN_LEN {
            return Err(WalError::InvalidHeader(
                "buffer too short for minimum header",
            ));
        }

        let mut cursor = 0;

        // Magic
        if &buf[cursor..cursor + WAL_MAGIC.len()] != WAL_MAGIC {
            return Err(WalError::InvalidHeader("magic mismatch"));
        }
        cursor += WAL_MAGIC.len();

        // Version
        let version = u16::from_le_bytes([buf[cursor], buf[cursor + 1]]);
        cursor += 2;
        if version != WAL_VERSION {
            return Err(WalError::InvalidHeader("unsupported version"));
        }

        // Header size
        let header_size = u16::from_le_bytes([buf[cursor], buf[cursor + 1]]) as usize;
        cursor += 2;

        if header_size < WAL_HEADER_MIN_LEN {
            return Err(WalError::InvalidHeader("header size too small"));
        }

        if buf.len() < header_size {
            return Err(WalError::InvalidHeader(
                "buffer shorter than declared header size",
            ));
        }

        // For version 1, we need at least 38 bytes (the minimum for our current fields)
        // magic (10) + version (2) + header_size (2) + segment_hash (16) + wal_pos_start (8) = 38
        const V1_MIN_FIELDS: usize = 10 + 2 + 2 + 16 + 8;
        if header_size < V1_MIN_FIELDS {
            return Err(WalError::InvalidHeader(
                "header too small for required fields",
            ));
        }

        // Segment config hash
        let mut segment_cfg_hash = [0u8; 16];
        segment_cfg_hash.copy_from_slice(&buf[cursor..cursor + 16]);
        cursor += 16;

        // WAL position start
        let wal_position_start = u64::from_le_bytes([
            buf[cursor],
            buf[cursor + 1],
            buf[cursor + 2],
            buf[cursor + 3],
            buf[cursor + 4],
            buf[cursor + 5],
            buf[cursor + 6],
            buf[cursor + 7],
        ]);

        // Remaining bytes up to header_size are reserved/unknown fields - we skip them

        Ok(Self {
            segment_cfg_hash,
            wal_position_start,
            header_size: header_size as u16,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_hash() -> [u8; 16] {
        let mut hash = [0u8; 16];
        for (idx, byte) in hash.iter_mut().enumerate() {
            *byte = (idx as u8).wrapping_mul(3).wrapping_add(1);
        }
        hash
    }

    #[test]
    fn header_size_is_38_bytes() {
        let header = WalHeader::new(sample_hash());
        assert_eq!(header.encoded_len(), 38);
    }

    #[test]
    fn encode_includes_header_size_field() {
        let header = WalHeader::new(sample_hash());
        let encoded = header.encode();

        // Header size is at offset 12 (after magic + version)
        let header_size = u16::from_le_bytes([encoded[12], encoded[13]]);
        assert_eq!(header_size, header.header_size);
    }

    #[test]
    fn encode_decode_roundtrip_preserves_segment_hash() {
        let header = WalHeader::new(sample_hash());
        let encoded = header.encode();
        let decoded = WalHeader::decode(&encoded).expect("decode should succeed");
        assert_eq!(decoded.segment_cfg_hash, sample_hash());
        assert_eq!(decoded.wal_position_start, 0);
    }

    #[test]
    fn encode_decode_roundtrip_preserves_base_offset() {
        let header = WalHeader::with_base_offset(sample_hash(), 123456789);
        let encoded = header.encode();
        let decoded = WalHeader::decode(&encoded).expect("decode should succeed");
        assert_eq!(decoded.segment_cfg_hash, sample_hash());
        assert_eq!(decoded.wal_position_start, 123456789);
    }

    #[test]
    fn decode_rejects_magic_mismatch() {
        let header = WalHeader::new(sample_hash());
        let mut encoded = header.encode();
        encoded[0] ^= 0xFF;
        let err = WalHeader::decode(&encoded).unwrap_err();
        assert!(matches!(err, WalError::InvalidHeader("magic mismatch")));
    }

    #[test]
    fn decode_rejects_short_buffer() {
        let buf = vec![0u8; WAL_HEADER_MIN_LEN - 1];
        let err = WalHeader::decode(&buf).unwrap_err();
        assert!(matches!(
            err,
            WalError::InvalidHeader("buffer too short for minimum header")
        ));
    }

    #[test]
    fn decode_rejects_unsupported_version() {
        let mut encoded = WalHeader::new(sample_hash()).encode();
        let version_idx = WAL_MAGIC.len();
        encoded[version_idx..version_idx + 2].copy_from_slice(&2u16.to_le_bytes());
        let err = WalHeader::decode(&encoded).unwrap_err();
        assert!(matches!(
            err,
            WalError::InvalidHeader("unsupported version")
        ));
    }

    #[test]
    fn decode_rejects_header_size_too_small() {
        let mut encoded = WalHeader::new(sample_hash()).encode();
        // Set header_size to something too small
        let header_size_idx = WAL_MAGIC.len() + 2;
        encoded[header_size_idx..header_size_idx + 2].copy_from_slice(&10u16.to_le_bytes());
        let err = WalHeader::decode(&encoded).unwrap_err();
        assert!(matches!(
            err,
            WalError::InvalidHeader("header size too small")
        ));
    }

    #[test]
    fn decode_accepts_larger_header_size() {
        // Simulate a future version with a larger header
        let header = WalHeader::new(sample_hash());
        let mut encoded = header.encode().to_vec();

        // Extend with some "future" bytes
        encoded.extend_from_slice(&[0xAA; 32]);

        // Update header_size field to reflect the new size
        let new_size = encoded.len() as u16;
        let header_size_idx = WAL_MAGIC.len() + 2;
        encoded[header_size_idx..header_size_idx + 2].copy_from_slice(&new_size.to_le_bytes());

        // Should still decode correctly, ignoring unknown trailing bytes
        let decoded = WalHeader::decode(&encoded).expect("decode should succeed");
        assert_eq!(decoded.segment_cfg_hash, sample_hash());
        assert_eq!(decoded.wal_position_start, 0);
    }

    #[test]
    fn decode_rejects_buffer_shorter_than_declared_size() {
        let header = WalHeader::new(sample_hash());
        let mut encoded = header.encode().to_vec();

        // Claim the header is larger than the buffer
        let header_size_idx = WAL_MAGIC.len() + 2;
        encoded[header_size_idx..header_size_idx + 2].copy_from_slice(&128u16.to_le_bytes());

        let err = WalHeader::decode(&encoded).unwrap_err();
        assert!(matches!(
            err,
            WalError::InvalidHeader("buffer shorter than declared header size")
        ));
    }
}
