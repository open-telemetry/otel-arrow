// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Crash-safe WAL cursor persistence.
//!
//! The cursor sidecar is a tiny file (`quiver.wal.cursor`) that tracks how
//! much of the WAL has been durably consumed. It survives crashes so restarts
//! can resume from the last known safe position without rescanning the entire log.
//!
//! # Format (v1, 24 bytes, variable-width support)
//!
//! ```text
//! ┌────────────┬─────────┬──────────┬─────────────────┬──────────┐
//! │ magic (8)  │ ver (2) │ size (2) │ wal_position (8)│ crc (4)  │
//! └────────────┴─────────┴──────────┴─────────────────┴──────────┘
//! ```
//!
//! The `size` field stores the total encoded size, enabling future versions
//! to add fields while maintaining backward compatibility. A v1 reader can
//! safely skip unknown trailing bytes in a v2+ file by using the size field.
//!
//! Writes use atomic rename (`write_to` → `.tmp` → `rename`) plus parent
//! directory fsync to ensure durability across power loss.

use std::io::SeekFrom;
use std::path::{Path, PathBuf};

use crc32fast::Hasher;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

use super::writer::sync_parent_dir;
use super::{WalError, WalResult};

#[cfg(test)]
use super::writer::test_support::{self as writer_test_support, CrashInjection};

pub(crate) const CURSOR_SIDECAR_MAGIC: &[u8; 8] = b"QUIVER\0T";
pub(crate) const CURSOR_SIDECAR_VERSION: u16 = 1;

/// Minimum prefix needed to read size: magic (8) + version (2) + size (2) = 12 bytes
const SIDECAR_MIN_LEN: usize = 12;

/// Size of v1 sidecar: magic (8) + version (2) + size (2) + wal_position (8) + crc (4) = 24 bytes
const SIDECAR_V1_LEN: usize = 24;

/// On-disk metadata describing the consumer cursor position in the logical stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CursorSidecar {
    /// Position in the WAL stream that has been durably consumed by readers.
    /// This is a logical position that spans all WAL files (past and present).
    pub wal_position: u64,
}

impl CursorSidecar {
    pub fn new(wal_position: u64) -> Self {
        Self { wal_position }
    }

    /// Returns the encoded size for the current version.
    pub fn encoded_len(&self) -> usize {
        SIDECAR_V1_LEN
    }

    pub fn encode(&self) -> Vec<u8> {
        let size = self.encoded_len();
        let mut buf = vec![0u8; size];
        let mut cursor = 0;

        buf[cursor..cursor + CURSOR_SIDECAR_MAGIC.len()].copy_from_slice(CURSOR_SIDECAR_MAGIC);
        cursor += CURSOR_SIDECAR_MAGIC.len();

        buf[cursor..cursor + 2].copy_from_slice(&CURSOR_SIDECAR_VERSION.to_le_bytes());
        cursor += 2;

        buf[cursor..cursor + 2].copy_from_slice(&(size as u16).to_le_bytes());
        cursor += 2;

        buf[cursor..cursor + 8].copy_from_slice(&self.wal_position.to_le_bytes());
        cursor += 8;

        let crc = compute_crc(&buf[..cursor]);
        buf[cursor..cursor + 4].copy_from_slice(&crc.to_le_bytes());
        buf
    }

    pub fn decode(buf: &[u8]) -> WalResult<Self> {
        if buf.len() < SIDECAR_MIN_LEN {
            return Err(WalError::InvalidCursorSidecar("buffer too short"));
        }

        // Read magic
        let mut cursor = 0;
        if &buf[cursor..cursor + CURSOR_SIDECAR_MAGIC.len()] != CURSOR_SIDECAR_MAGIC {
            return Err(WalError::InvalidCursorSidecar("magic mismatch"));
        }
        cursor += CURSOR_SIDECAR_MAGIC.len();

        // Read version
        let version = u16::from_le_bytes([buf[cursor], buf[cursor + 1]]);
        cursor += 2;

        // Read size - allows handling future versions with additional fields
        let size = u16::from_le_bytes([buf[cursor], buf[cursor + 1]]) as usize;
        cursor += 2;

        // Validate we have enough data for the declared size
        if buf.len() < size {
            return Err(WalError::InvalidCursorSidecar(
                "buffer shorter than declared size",
            ));
        }

        // For v1, we know the exact expected size
        if version == CURSOR_SIDECAR_VERSION {
            if size != SIDECAR_V1_LEN {
                return Err(WalError::InvalidCursorSidecar("invalid v1 size"));
            }
        } else if version > CURSOR_SIDECAR_VERSION {
            // Future version - we can still try to read v1 fields if size is >= v1 size
            if size < SIDECAR_V1_LEN {
                return Err(WalError::InvalidCursorSidecar(
                    "future version too small for v1 fields",
                ));
            }
            // Log or warn about reading a newer version file with v1 reader
            // For now, proceed to read the v1-compatible fields
        } else {
            return Err(WalError::InvalidCursorSidecar("unsupported version"));
        }

        // Read wal_position (v1 field)
        let wal_position = u64::from_le_bytes([
            buf[cursor],
            buf[cursor + 1],
            buf[cursor + 2],
            buf[cursor + 3],
            buf[cursor + 4],
            buf[cursor + 5],
            buf[cursor + 6],
            buf[cursor + 7],
        ]);
        // cursor += 8; -- not needed, we use crc_offset below

        // CRC covers everything except the CRC itself (last 4 bytes)
        let crc_offset = size - 4;
        let stored_crc = u32::from_le_bytes([
            buf[crc_offset],
            buf[crc_offset + 1],
            buf[crc_offset + 2],
            buf[crc_offset + 3],
        ]);
        let computed_crc = compute_crc(&buf[..crc_offset]);
        if stored_crc != computed_crc {
            return Err(WalError::InvalidCursorSidecar("crc mismatch"));
        }

        Ok(Self { wal_position })
    }

    /// Reads the size field from a file to determine total sidecar length.
    async fn read_size(file: &mut File) -> WalResult<usize> {
        let mut prefix = [0u8; SIDECAR_MIN_LEN];
        let _ = file.read_exact(&mut prefix).await?;

        // Validate magic
        if &prefix[..CURSOR_SIDECAR_MAGIC.len()] != CURSOR_SIDECAR_MAGIC {
            return Err(WalError::InvalidCursorSidecar("magic mismatch"));
        }

        // Read size from offset 10 (magic 8 + version 2)
        let size = u16::from_le_bytes([prefix[10], prefix[11]]) as usize;
        if size < SIDECAR_MIN_LEN {
            return Err(WalError::InvalidCursorSidecar("size too small"));
        }

        // Seek back to beginning for full read
        let _ = file.seek(SeekFrom::Start(0)).await?;
        Ok(size)
    }

    /// Reads the cursor sidecar from a file.
    pub async fn read_from(path: &Path) -> WalResult<Self> {
        let mut file = OpenOptions::new().read(true).open(path).await?;
        let size = Self::read_size(&mut file).await?;
        let mut buf = vec![0u8; size];
        let _ = file.read_exact(&mut buf).await?;
        Self::decode(&buf)
    }

    /// Writes the cursor sidecar to a file using atomic rename.
    pub async fn write_to(path: &Path, value: &Self) -> WalResult<()> {
        let tmp_path = temporary_path(path);
        {
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&tmp_path)
                .await?;
            let encoded = value.encode();
            file.write_all(&encoded).await?;
            file.flush().await?;
            file.sync_data().await?;
        }
        #[cfg(test)]
        if writer_test_support::take_crash(CrashInjection::BeforeSidecarRename) {
            return Err(WalError::InjectedCrash(
                "crash injected before cursor sidecar rename",
            ));
        }
        tokio::fs::rename(&tmp_path, path).await?;
        sync_parent_dir(path).await?;
        Ok(())
    }

    /// Reads the cursor sidecar from a file synchronously.
    ///
    /// Used by tests that verify cursor state after async writes.
    #[cfg(test)]
    pub fn read_from_sync(path: &Path) -> WalResult<Self> {
        use std::io::Read;
        let mut file = std::fs::File::open(path)?;
        let size = Self::read_size_sync(&mut file)?;
        let mut buf = vec![0u8; size];
        file.read_exact(&mut buf)?;
        Self::decode(&buf)
    }

    /// Reads the size field from a file synchronously.
    #[cfg(test)]
    fn read_size_sync(file: &mut std::fs::File) -> WalResult<usize> {
        use std::io::{Read, Seek};
        let mut prefix = [0u8; SIDECAR_MIN_LEN];
        file.read_exact(&mut prefix)?;

        // Validate magic
        if &prefix[..CURSOR_SIDECAR_MAGIC.len()] != CURSOR_SIDECAR_MAGIC {
            return Err(WalError::InvalidCursorSidecar("magic mismatch"));
        }

        // Read size from offset 10 (magic 8 + version 2)
        let size = u16::from_le_bytes([prefix[10], prefix[11]]) as usize;
        if size < SIDECAR_MIN_LEN {
            return Err(WalError::InvalidCursorSidecar("size too small"));
        }

        // Seek back to beginning for full read
        let _ = file.seek(SeekFrom::Start(0))?;
        Ok(size)
    }

    /// Writes the cursor sidecar to a file synchronously using atomic rename.
    ///
    /// Used by tests that need to create cursor sidecar files for testing.
    #[cfg(test)]
    pub fn write_to_sync(path: &Path, value: &Self) -> WalResult<()> {
        use std::io::Write;
        let tmp_path = temporary_path(path);
        {
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&tmp_path)?;
            let encoded = value.encode();
            file.write_all(&encoded)?;
            file.flush()?;
            file.sync_data()?;
        }
        std::fs::rename(&tmp_path, path)?;
        // Sync parent directory for durability (on Unix)
        #[cfg(unix)]
        {
            if let Some(parent) = path.parent() {
                if let Ok(dir) = std::fs::File::open(parent) {
                    let _ = dir.sync_data();
                }
            }
        }
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

    fn sample_sidecar() -> CursorSidecar {
        CursorSidecar::new(128)
    }

    #[test]
    fn encode_decode_roundtrip() {
        let value = sample_sidecar();
        let encoded = value.encode();
        let decoded = CursorSidecar::decode(&encoded).expect("decode");
        assert_eq!(decoded, value);
    }

    #[test]
    fn decode_rejects_magic_mismatch() {
        let mut encoded = sample_sidecar().encode();
        encoded[0] ^= 0xFF;
        let err = CursorSidecar::decode(&encoded).unwrap_err();
        assert!(matches!(
            err,
            WalError::InvalidCursorSidecar("magic mismatch")
        ));
    }

    #[test]
    fn decode_rejects_crc_mismatch() {
        let mut encoded = sample_sidecar().encode();
        let last = encoded.len() - 1;
        encoded[last] ^= 0xFF;
        let err = CursorSidecar::decode(&encoded).unwrap_err();
        assert!(matches!(
            err,
            WalError::InvalidCursorSidecar("crc mismatch")
        ));
    }

    #[tokio::test]
    async fn write_and_read_sidecar() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("quiver.wal.cursor");
        let value = sample_sidecar();
        CursorSidecar::write_to(&path, &value).await.expect("write");
        let loaded = CursorSidecar::read_from(&path).await.expect("read");
        assert_eq!(loaded, value);
    }

    #[test]
    fn forward_compatible_with_larger_future_version() {
        // Simulate a "v2" sidecar with extra fields after wal_position.
        // The v1 reader should still be able to read the wal_position.
        let wal_position: u64 = 0x1234_5678_9ABC_DEF0;
        let future_version: u16 = 2; // Newer version
        let future_size: u16 = 32; // 24 bytes of v1 + 8 bytes of new field

        let mut buf = vec![0u8; future_size as usize];
        let mut cursor = 0;

        // Magic
        buf[cursor..cursor + 8].copy_from_slice(CURSOR_SIDECAR_MAGIC);
        cursor += 8;

        // Version (2 = future)
        buf[cursor..cursor + 2].copy_from_slice(&future_version.to_le_bytes());
        cursor += 2;

        // Size
        buf[cursor..cursor + 2].copy_from_slice(&future_size.to_le_bytes());
        cursor += 2;

        // WAL position (v1 field)
        buf[cursor..cursor + 8].copy_from_slice(&wal_position.to_le_bytes());
        cursor += 8;

        // Hypothetical new v2 field
        buf[cursor..cursor + 8].copy_from_slice(&0xDEAD_BEEF_u64.to_le_bytes());
        cursor += 8;

        // CRC at the end (covers everything except CRC itself)
        let crc = compute_crc(&buf[..cursor]);
        buf[cursor..cursor + 4].copy_from_slice(&crc.to_le_bytes());

        // V1 reader should successfully decode and extract wal_position
        let decoded = CursorSidecar::decode(&buf).expect("should decode future version");
        assert_eq!(decoded.wal_position, wal_position);
    }

    #[test]
    fn rejects_size_smaller_than_v1() {
        // A future version claims a smaller size than v1 - reject
        let mut buf = vec![0u8; 20]; // Too small
        buf[0..8].copy_from_slice(CURSOR_SIDECAR_MAGIC);
        buf[8..10].copy_from_slice(&2u16.to_le_bytes()); // version 2
        buf[10..12].copy_from_slice(&16u16.to_le_bytes()); // size 16 (too small for v1)

        let err = CursorSidecar::decode(&buf).unwrap_err();
        assert!(matches!(
            err,
            WalError::InvalidCursorSidecar("future version too small for v1 fields")
        ));
    }

    #[test]
    fn rejects_invalid_v1_size() {
        // v1 must have exact size of 24
        let mut buf = vec![0u8; 32];
        buf[0..8].copy_from_slice(CURSOR_SIDECAR_MAGIC);
        buf[8..10].copy_from_slice(&1u16.to_le_bytes()); // version 1
        buf[10..12].copy_from_slice(&32u16.to_le_bytes()); // size 32 (wrong for v1)

        let err = CursorSidecar::decode(&buf).unwrap_err();
        assert!(matches!(
            err,
            WalError::InvalidCursorSidecar("invalid v1 size")
        ));
    }
}
