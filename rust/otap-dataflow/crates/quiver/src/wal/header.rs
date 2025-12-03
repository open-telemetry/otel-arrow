// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::io::{Read, Seek, SeekFrom, Write};

use super::{WAL_MAGIC, WalError};

pub(crate) const WAL_VERSION: u16 = 1;
pub(crate) const WAL_HEADER_LEN: usize = WAL_MAGIC.len() + 2 + 2 + 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct WalHeader {
    pub segment_cfg_hash: [u8; 16],
}

impl WalHeader {
    pub fn new(segment_cfg_hash: [u8; 16]) -> Self {
        Self { segment_cfg_hash }
    }

    pub fn encode(&self) -> [u8; WAL_HEADER_LEN] {
        let mut buf = [0u8; WAL_HEADER_LEN];
        let mut cursor = 0;
        buf[cursor..cursor + WAL_MAGIC.len()].copy_from_slice(WAL_MAGIC);
        cursor += WAL_MAGIC.len();

        buf[cursor..cursor + 2].copy_from_slice(&WAL_VERSION.to_le_bytes());
        cursor += 2;

        buf[cursor..cursor + 2].copy_from_slice(&0u16.to_le_bytes());
        cursor += 2;

        buf[cursor..cursor + 16].copy_from_slice(&self.segment_cfg_hash);
        buf
    }

    pub fn write_to(&self, file: &mut (impl Write + Seek)) -> Result<(), WalError> {
        let _ = file.seek(SeekFrom::Start(0))?;
        file.write_all(&self.encode())?;
        file.flush()?;
        Ok(())
    }

    pub fn read_from(file: &mut (impl Read + Seek)) -> Result<Self, WalError> {
        let _ = file.seek(SeekFrom::Start(0))?;
        let mut buf = [0u8; WAL_HEADER_LEN];
        file.read_exact(&mut buf)?;
        Self::decode(&buf)
    }

    pub fn decode(buf: &[u8]) -> Result<Self, WalError> {
        if buf.len() < WAL_HEADER_LEN {
            return Err(WalError::InvalidHeader("buffer too short"));
        }
        let mut cursor = 0;
        if &buf[cursor..cursor + WAL_MAGIC.len()] != WAL_MAGIC {
            return Err(WalError::InvalidHeader("magic mismatch"));
        }
        cursor += WAL_MAGIC.len();

        let version = u16::from_le_bytes([buf[cursor], buf[cursor + 1]]);
        cursor += 2;
        if version != WAL_VERSION {
            return Err(WalError::InvalidHeader("unsupported version"));
        }

        // Reserved field; must be zero today.
        if buf[cursor] != 0 || buf[cursor + 1] != 0 {
            return Err(WalError::InvalidHeader("reserved bits non-zero"));
        }
        cursor += 2;

        let mut segment_cfg_hash = [0u8; 16];
        segment_cfg_hash.copy_from_slice(&buf[cursor..cursor + 16]);

        Ok(Self { segment_cfg_hash })
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
    fn encode_decode_roundtrip_preserves_segment_hash() {
        let header = WalHeader::new(sample_hash());
        let encoded = header.encode();
        let decoded = WalHeader::decode(&encoded).expect("decode should succeed");
        assert_eq!(decoded.segment_cfg_hash, sample_hash());
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
    fn decode_rejects_reserved_bits() {
        let header = WalHeader::new(sample_hash());
        let mut encoded = header.encode();
        let reserved_start = WAL_MAGIC.len() + 2;
        encoded[reserved_start] = 1;
        let err = WalHeader::decode(&encoded).unwrap_err();
        assert!(matches!(
            err,
            WalError::InvalidHeader("reserved bits non-zero")
        ));
    }

    #[test]
    fn decode_rejects_short_buffer() {
        let mut buf = vec![0u8; WAL_HEADER_LEN - 1];
        buf.copy_from_slice(&WalHeader::new(sample_hash()).encode()[..WAL_HEADER_LEN - 1]);
        let err = WalHeader::decode(&buf).unwrap_err();
        assert!(matches!(err, WalError::InvalidHeader("buffer too short")));
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
}
