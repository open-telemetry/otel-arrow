// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// Provides read/write helpers for the `truncate.offset` sidecar that tracks the
// earliest WAL byte still required for crash recovery. The sidecar lets new
// processes resume from a known safe offset without rescanning the entire WAL
// and carries a CRC so we can discard corrupted metadata safely.

use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crc32fast::Hasher;

use super::{WalError, WalResult};

#[cfg(test)]
use super::writer::test_support::{self as writer_test_support, CrashInjection};

pub(crate) const TRUNCATE_SIDECAR_MAGIC: &[u8; 8] = b"QUIVER\0T";
pub(crate) const TRUNCATE_SIDECAR_VERSION: u16 = 1;
pub(crate) const TRUNCATE_SIDECAR_LEN: usize = 8 + 2 + 2 + 8 + 8 + 4;

/// On-disk metadata describing the safe truncate point and rotation generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TruncateSidecar {
    pub truncate_offset: u64,
    pub rotation_generation: u64,
}

impl TruncateSidecar {
    pub fn new(truncate_offset: u64, rotation_generation: u64) -> Self {
        Self {
            truncate_offset,
            rotation_generation,
        }
    }

    pub fn encode(&self) -> [u8; TRUNCATE_SIDECAR_LEN] {
        let mut buf = [0u8; TRUNCATE_SIDECAR_LEN];
        let mut cursor = 0;
        buf[cursor..cursor + TRUNCATE_SIDECAR_MAGIC.len()]
            .copy_from_slice(TRUNCATE_SIDECAR_MAGIC);
        cursor += TRUNCATE_SIDECAR_MAGIC.len();

        buf[cursor..cursor + 2].copy_from_slice(&TRUNCATE_SIDECAR_VERSION.to_le_bytes());
        cursor += 2;

        buf[cursor..cursor + 2].copy_from_slice(&0u16.to_le_bytes());
        cursor += 2;

        buf[cursor..cursor + 8].copy_from_slice(&self.truncate_offset.to_le_bytes());
        cursor += 8;

        buf[cursor..cursor + 8].copy_from_slice(&self.rotation_generation.to_le_bytes());
        cursor += 8;

        let crc = compute_crc(&buf[..cursor]);
        buf[cursor..cursor + 4].copy_from_slice(&crc.to_le_bytes());
        buf
    }

    pub fn decode(buf: &[u8]) -> WalResult<Self> {
        if buf.len() < TRUNCATE_SIDECAR_LEN {
            return Err(WalError::InvalidTruncateSidecar("buffer too short"));
        }
        let mut cursor = 0;
        if &buf[cursor..cursor + TRUNCATE_SIDECAR_MAGIC.len()] != TRUNCATE_SIDECAR_MAGIC {
            return Err(WalError::InvalidTruncateSidecar("magic mismatch"));
        }
        cursor += TRUNCATE_SIDECAR_MAGIC.len();

        let version = u16::from_le_bytes([buf[cursor], buf[cursor + 1]]);
        cursor += 2;
        if version != TRUNCATE_SIDECAR_VERSION {
            return Err(WalError::InvalidTruncateSidecar("unsupported version"));
        }

        if buf[cursor] != 0 || buf[cursor + 1] != 0 {
            return Err(WalError::InvalidTruncateSidecar("reserved bits non-zero"));
        }
        cursor += 2;

        let truncate_offset = u64::from_le_bytes([
            buf[cursor],
            buf[cursor + 1],
            buf[cursor + 2],
            buf[cursor + 3],
            buf[cursor + 4],
            buf[cursor + 5],
            buf[cursor + 6],
            buf[cursor + 7],
        ]);
        cursor += 8;

        let rotation_generation = u64::from_le_bytes([
            buf[cursor],
            buf[cursor + 1],
            buf[cursor + 2],
            buf[cursor + 3],
            buf[cursor + 4],
            buf[cursor + 5],
            buf[cursor + 6],
            buf[cursor + 7],
        ]);
        cursor += 8;

        let stored_crc = u32::from_le_bytes([
            buf[cursor],
            buf[cursor + 1],
            buf[cursor + 2],
            buf[cursor + 3],
        ]);
        let computed_crc = compute_crc(&buf[..cursor]);
        if stored_crc != computed_crc {
            return Err(WalError::InvalidTruncateSidecar("crc mismatch"));
        }

        Ok(Self {
            truncate_offset,
            rotation_generation,
        })
    }

    pub fn read_from(path: &Path) -> WalResult<Self> {
        let mut file = OpenOptions::new().read(true).open(path)?;
        let mut buf = [0u8; TRUNCATE_SIDECAR_LEN];
        file.read_exact(&mut buf)?;
        Self::decode(&buf)
    }

    pub fn write_to(path: &Path, value: &Self) -> WalResult<()> {
        let tmp_path = temporary_path(path);
        {
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&tmp_path)?;
            let encoded = value.encode();
            file.write_all(&encoded)?;
            file.flush()?;
            file.sync_data()?;
        }
        #[cfg(test)]
        if writer_test_support::take_crash(CrashInjection::BeforeSidecarRename) {
            return Err(WalError::InjectedCrash(
                "crash injected before truncate sidecar rename",
            ));
        }
        std::fs::rename(&tmp_path, path)?;
        Ok(())
    }
}

fn compute_crc(buf: &[u8]) -> u32 {
    let mut hasher = Hasher::new();
    hasher.update(buf);
    hasher.finalize()
}

fn temporary_path(path: &Path) -> PathBuf {
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    PathBuf::from(tmp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn sample_sidecar() -> TruncateSidecar {
        TruncateSidecar::new(128, 7)
    }

    #[test]
    fn encode_decode_roundtrip() {
        let value = sample_sidecar();
        let encoded = value.encode();
        let decoded = TruncateSidecar::decode(&encoded).expect("decode");
        assert_eq!(decoded, value);
    }

    #[test]
    fn decode_rejects_magic_mismatch() {
        let mut encoded = sample_sidecar().encode();
        encoded[0] ^= 0xFF;
        let err = TruncateSidecar::decode(&encoded).unwrap_err();
        assert!(matches!(err, WalError::InvalidTruncateSidecar("magic mismatch")));
    }

    #[test]
    fn decode_rejects_crc_mismatch() {
        let mut encoded = sample_sidecar().encode();
        let last = encoded.len() - 1;
        encoded[last] ^= 0xFF;
        let err = TruncateSidecar::decode(&encoded).unwrap_err();
        assert!(matches!(err, WalError::InvalidTruncateSidecar("crc mismatch")));
    }

    #[test]
    fn write_and_read_sidecar() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("truncate.offset");
        let value = sample_sidecar();
        TruncateSidecar::write_to(&path, &value).expect("write");
        let loaded = TruncateSidecar::read_from(&path).expect("read");
        assert_eq!(loaded, value);
    }
}
