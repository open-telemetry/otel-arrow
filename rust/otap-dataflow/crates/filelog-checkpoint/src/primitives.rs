// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared durable values and bounded byte helpers.

use crc::{CRC_32_ISCSI, Crc};
use sha2::{Digest, Sha256};

use crate::{DecodeError, EncodeError};

/// Version of every version 1 checkpoint artifact header.
pub const FILELOG_FORMAT_VERSION: u16 = 1;
/// Version of the framing-profile canonical serialization and digest recipe.
pub const FRAMING_PROFILE_VERSION: u16 = 1;
pub(crate) const TX_ENVELOPE_VERSION: u16 = 1;
pub(crate) const CURRENT_MAGIC: &[u8; 8] = b"FLOGCUR\0";
pub(crate) const SNAPSHOT_MAGIC: &[u8; 8] = b"FLOGSNP\0";
pub(crate) const SNAPSHOT_FOOTER_MAGIC: &[u8; 8] = b"FLOGSFT\0";
pub(crate) const WAL_MAGIC: &[u8; 8] = b"FLOGWAL\0";
pub(crate) const TX_MAGIC: &[u8; 8] = b"FLOGTXN\0";

const NAMESPACE_DOMAIN: &[u8] = b"otel-arrow-filelog-checkpoint-namespace-v1\0";
const FRONTIER_DOMAIN: &[u8] = b"otel-arrow-filelog-frontier-guard-v1\0";
const EMPTY_FRONTIER_GUARD_DIGEST: [u8; 32] = [
    0xbe, 0x47, 0xd0, 0x23, 0xa0, 0x6e, 0x82, 0xfd, 0x6d, 0xa2, 0xda, 0xa0, 0x63, 0x15, 0x47, 0xd6,
    0xec, 0xa2, 0x97, 0xb7, 0xac, 0x53, 0x2c, 0xba, 0x64, 0x71, 0xab, 0x90, 0x82, 0x9e, 0xc5, 0xb9,
];
const ADVISORY_PATH_DOMAIN: &[u8] = b"otel-arrow-filelog-advisory-path-v1\0";

/// Absolute version 1 fingerprint field maximum.
pub const FINGERPRINT_MAX_BYTES: usize = u16::MAX as usize;
/// Maximum stored advisory-path suffix.
pub const ADVISORY_PATH_STORED_MAX_BYTES: usize = 4096;
const ADVISORY_PATH_TRUNCATED: u8 = 0x01;
/// Maximum administrative audit-reason length.
pub const AUDIT_REASON_MAX_BYTES: usize = 1024;
/// Maximum administrative namespace-ID length.
pub const NAMESPACE_ID_MAX_BYTES: usize = 255;
pub(crate) const FRAMING_PATTERN_MAX_BYTES: usize = 4096;
/// Raw-byte window covered by committed-frontier evidence.
pub const COMMITTED_FRONTIER_GUARD_WINDOW_BYTES: u16 = 64;
pub(crate) const SNAPSHOT_MAX_RECORD_PAYLOAD_BYTES: u64 = 69_854;
pub(crate) const REASON_CODE_RESERVED: u16 = 0;
pub(crate) const QUARANTINE_REASON_RESERVED_V1: u16 = 4;

/// Opaque durable 128-bit file identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FileId(pub(crate) [u8; 16]);

impl FileId {
    /// Constructs an ID from its exact opaque bytes.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// Returns the exact opaque bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

/// Durable lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleState {
    /// File is actively tracked.
    Active,
    /// Rotation finalization completed.
    RotatedFinalized,
    /// Reading is blocked pending administration.
    Quarantined,
}

impl LifecycleState {
    pub(crate) const fn to_wire(self) -> u8 {
        match self {
            Self::Active => 1,
            Self::RotatedFinalized => 2,
            Self::Quarantined => 3,
        }
    }

    pub(crate) fn from_wire(value: u8, field: &'static str) -> Result<Self, DecodeError> {
        match value {
            1 => Ok(Self::Active),
            2 => Ok(Self::RotatedFinalized),
            3 => Ok(Self::Quarantined),
            _ => Err(DecodeError::UnknownDiscriminant {
                field,
                value: u32::from(value),
            }),
        }
    }
}

/// Normalized platform-neutral file locator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Locator {
    /// No locator is available. Version 1 snapshot and registration state
    /// rejects this value, but the discriminant remains part of the codec.
    Unspecified,
    /// POSIX device and inode values widened to `u64`.
    PosixDevIno {
        /// Device identifier.
        dev: u64,
        /// Inode number.
        ino: u64,
    },
    /// Windows volume serial and opaque 128-bit file ID.
    WindowsVolumeFileId {
        /// Full volume serial value.
        volume_serial: u64,
        /// Opaque `FILE_ID_INFO.FileId` bytes.
        file_id: [u8; 16],
    },
}

impl Locator {
    pub(crate) fn write(&self, out: &mut Writer) {
        match *self {
            Self::Unspecified => out.u8(0),
            Self::PosixDevIno { dev, ino } => {
                out.u8(1);
                out.u64(dev);
                out.u64(ino);
            }
            Self::WindowsVolumeFileId {
                volume_serial,
                file_id,
            } => {
                out.u8(2);
                out.u64(volume_serial);
                out.bytes(&file_id);
            }
        }
    }

    pub(crate) fn read(input: &mut Reader<'_>) -> Result<Self, DecodeError> {
        match input.u8()? {
            0 => Ok(Self::Unspecified),
            1 => Ok(Self::PosixDevIno {
                dev: input.u64()?,
                ino: input.u64()?,
            }),
            2 => Ok(Self::WindowsVolumeFileId {
                volume_serial: input.u64()?,
                file_id: input.array()?,
            }),
            value => Err(DecodeError::UnknownDiscriminant {
                field: "locator.kind",
                value: u32::from(value),
            }),
        }
    }
}

/// Durable framing resume state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FramingResume {
    /// The next unit starts a new record.
    Clean,
    /// A split record remains in progress.
    Continuation {
        /// Original record start offset.
        record_start_offset: u64,
        /// Known record end, or zero for scan-to-LF mode.
        record_end_offset: u64,
        /// Index of the next fragment.
        next_fragment_index: u32,
    },
}

impl FramingResume {
    pub(crate) fn write(&self, out: &mut Writer) {
        match *self {
            Self::Clean => out.u8(0),
            Self::Continuation {
                record_start_offset,
                record_end_offset,
                next_fragment_index,
            } => {
                out.u8(1);
                out.u64(record_start_offset);
                out.u64(record_end_offset);
                out.u32(next_fragment_index);
            }
        }
    }

    pub(crate) fn read(input: &mut Reader<'_>) -> Result<Self, DecodeError> {
        match input.u8()? {
            0 => Ok(Self::Clean),
            1 => Ok(Self::Continuation {
                record_start_offset: input.u64()?,
                record_end_offset: input.u64()?,
                next_fragment_index: input.u32()?,
            }),
            value => Err(DecodeError::UnknownDiscriminant {
                field: "framing_resume.kind",
                value: u32::from(value),
            }),
        }
    }

    pub(crate) const fn valid_for_offset(self, offset: u64) -> bool {
        match self {
            Self::Clean => true,
            Self::Continuation {
                record_start_offset,
                record_end_offset,
                next_fragment_index,
            } => {
                next_fragment_index >= 1
                    && record_start_offset < offset
                    && (record_end_offset == 0 || offset < record_end_offset)
            }
        }
    }
}

/// Advisory path encoding kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdvisoryPathKind {
    /// No path is available.
    Unavailable,
    /// Native Unix bytes.
    UnixBytes,
    /// Native UTF-16 code units serialized little-endian.
    WindowsUtf16Le,
}

impl AdvisoryPathKind {
    /// Returns the version 1 discriminant.
    #[must_use]
    pub const fn to_wire(self) -> u8 {
        match self {
            Self::Unavailable => 0,
            Self::UnixBytes => 1,
            Self::WindowsUtf16Le => 2,
        }
    }

    fn from_wire(value: u8) -> Result<Self, DecodeError> {
        match value {
            0 => Ok(Self::Unavailable),
            1 => Ok(Self::UnixBytes),
            2 => Ok(Self::WindowsUtf16Le),
            _ => Err(DecodeError::UnknownDiscriminant {
                field: "advisory_path.path_kind",
                value: u32::from(value),
            }),
        }
    }
}

/// Bounded filelog advisory-path evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvisoryPath {
    kind: AdvisoryPathKind,
    truncated: bool,
    full_path_len: u64,
    stored_path_bytes: Vec<u8>,
    full_path_digest: [u8; 32],
}

impl AdvisoryPath {
    /// Constructs the explicit unavailable value.
    #[must_use]
    pub fn unavailable() -> Self {
        Self {
            kind: AdvisoryPathKind::Unavailable,
            truncated: false,
            full_path_len: 0,
            stored_path_bytes: Vec::new(),
            full_path_digest: advisory_path_digest(AdvisoryPathKind::Unavailable, 0, &[]),
        }
    }

    /// Constructs bounded evidence from complete native Unix path bytes.
    pub fn from_unix_bytes(bytes: &[u8]) -> Result<Self, EncodeError> {
        if bytes.is_empty() {
            return Err(EncodeError::InvalidAdvisoryPath {
                reason: "a present Unix path must be non-empty",
            });
        }
        let full_path_len =
            u64::try_from(bytes.len()).map_err(|_| EncodeError::InvalidAdvisoryPath {
                reason: "path length does not fit u64",
            })?;
        let start = bytes.len().saturating_sub(ADVISORY_PATH_STORED_MAX_BYTES);
        Ok(Self {
            kind: AdvisoryPathKind::UnixBytes,
            truncated: start != 0,
            full_path_len,
            stored_path_bytes: bytes[start..].to_vec(),
            full_path_digest: advisory_path_digest(
                AdvisoryPathKind::UnixBytes,
                full_path_len,
                bytes,
            ),
        })
    }

    /// Constructs bounded evidence from complete Windows UTF-16 code units.
    pub fn from_windows_utf16_units(units: &[u16]) -> Result<Self, EncodeError> {
        if units.is_empty() {
            return Err(EncodeError::InvalidAdvisoryPath {
                reason: "a present Windows path must be non-empty",
            });
        }
        let full_path_len = u64::try_from(units.len())
            .ok()
            .and_then(|len| len.checked_mul(2))
            .ok_or(EncodeError::InvalidAdvisoryPath {
                reason: "path length does not fit u64",
            })?;
        let mut hasher = Sha256::new();
        hasher.update(ADVISORY_PATH_DOMAIN);
        hasher.update([AdvisoryPathKind::WindowsUtf16Le.to_wire()]);
        hasher.update(full_path_len.to_be_bytes());
        for unit in units {
            hasher.update(unit.to_le_bytes());
        }
        let stored_units = units.len().min(ADVISORY_PATH_STORED_MAX_BYTES / 2);
        let mut stored_path_bytes = Vec::with_capacity(stored_units * 2);
        for unit in &units[units.len() - stored_units..] {
            stored_path_bytes.extend_from_slice(&unit.to_le_bytes());
        }
        Ok(Self {
            kind: AdvisoryPathKind::WindowsUtf16Le,
            truncated: stored_units != units.len(),
            full_path_len,
            stored_path_bytes,
            full_path_digest: hasher.finalize().into(),
        })
    }

    /// Returns the path kind.
    #[must_use]
    pub const fn kind(&self) -> AdvisoryPathKind {
        self.kind
    }

    /// Reports whether only the final 4,096 bytes are stored.
    #[must_use]
    pub const fn is_truncated(&self) -> bool {
        self.truncated
    }

    /// Returns the complete native-byte length.
    #[must_use]
    pub const fn full_path_len(&self) -> u64 {
        self.full_path_len
    }

    /// Returns the complete bytes or stored suffix.
    #[must_use]
    pub fn stored_path_bytes(&self) -> &[u8] {
        &self.stored_path_bytes
    }

    /// Returns the digest of the complete native path.
    #[must_use]
    pub const fn full_path_digest(&self) -> &[u8; 32] {
        &self.full_path_digest
    }

    pub(crate) fn write(&self, out: &mut Writer) {
        out.u8(self.kind.to_wire());
        out.u8(u8::from(self.truncated));
        out.u64(self.full_path_len);
        out.u16(self.stored_path_bytes.len() as u16);
        out.bytes(&self.stored_path_bytes);
        out.bytes(&self.full_path_digest);
    }

    pub(crate) fn read(input: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let kind = AdvisoryPathKind::from_wire(input.u8()?)?;
        let flags = input.u8()?;
        if flags & !ADVISORY_PATH_TRUNCATED != 0 {
            return Err(DecodeError::ReservedFieldNonZero {
                field: "advisory_path.path_flags",
                value: u64::from(flags),
            });
        }
        let truncated = flags & ADVISORY_PATH_TRUNCATED != 0;
        let full_path_len = input.u64()?;
        let stored_len = usize::from(input.u16()?);
        if stored_len > ADVISORY_PATH_STORED_MAX_BYTES {
            return Err(DecodeError::LengthExceedsMaximum {
                field: "advisory_path.stored_path_bytes",
                declared: stored_len as u64,
                max: ADVISORY_PATH_STORED_MAX_BYTES as u64,
            });
        }
        let stored_path_bytes = input.exact(stored_len)?.to_vec();
        let full_path_digest = input.array()?;
        let invalid = |field, reason| DecodeError::InvalidAdvisoryPath { field, reason };
        match kind {
            AdvisoryPathKind::Unavailable => {
                if truncated || full_path_len != 0 || stored_len != 0 {
                    return Err(invalid(
                        "advisory_path",
                        "Unavailable requires zero flags and lengths",
                    ));
                }
                if full_path_digest != advisory_path_digest(kind, 0, &[]) {
                    return Err(invalid(
                        "advisory_path.full_path_digest",
                        "Unavailable digest does not match",
                    ));
                }
            }
            AdvisoryPathKind::UnixBytes | AdvisoryPathKind::WindowsUtf16Le => {
                if full_path_len == 0 {
                    return Err(invalid(
                        "advisory_path.full_path_len",
                        "a present path must be non-empty",
                    ));
                }
                if kind == AdvisoryPathKind::WindowsUtf16Le
                    && (!full_path_len.is_multiple_of(2) || !stored_len.is_multiple_of(2))
                {
                    return Err(invalid(
                        "advisory_path",
                        "Windows path lengths must be even",
                    ));
                }
                if full_path_len <= ADVISORY_PATH_STORED_MAX_BYTES as u64 {
                    if truncated || full_path_len != stored_len as u64 {
                        return Err(invalid(
                            "advisory_path",
                            "complete path flags and lengths are inconsistent",
                        ));
                    }
                    if full_path_digest
                        != advisory_path_digest(kind, full_path_len, &stored_path_bytes)
                    {
                        return Err(invalid(
                            "advisory_path.full_path_digest",
                            "complete path digest does not match",
                        ));
                    }
                } else if !truncated || stored_len != ADVISORY_PATH_STORED_MAX_BYTES {
                    return Err(invalid(
                        "advisory_path",
                        "truncated path flags and lengths are inconsistent",
                    ));
                }
            }
        }
        Ok(Self {
            kind,
            truncated,
            full_path_len,
            stored_path_bytes,
            full_path_digest,
        })
    }
}

fn advisory_path_digest(kind: AdvisoryPathKind, len: u64, bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(ADVISORY_PATH_DOMAIN);
    hasher.update([kind.to_wire()]);
    hasher.update(len.to_be_bytes());
    hasher.update(bytes);
    hasher.finalize().into()
}

/// Fixed-width digest evidence immediately preceding committed progress.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommittedFrontierGuard {
    /// Required raw window length.
    pub window_len: u16,
    /// Domain-separated SHA-256 digest.
    pub digest: [u8; 32],
}

impl CommittedFrontierGuard {
    /// Computes a guard from the exact required raw window.
    pub fn compute(offset: u64, window: &[u8]) -> Result<Self, EncodeError> {
        let expected = offset.min(u64::from(COMMITTED_FRONTIER_GUARD_WINDOW_BYTES)) as usize;
        if window.len() != expected {
            return Err(EncodeError::InvalidFieldValue {
                field: "committed_frontier_guard.window_bytes",
                reason: "length must equal min(offset, 64)",
            });
        }
        let window_len = expected as u16;
        let mut hasher = Sha256::new();
        hasher.update(FRONTIER_DOMAIN);
        hasher.update(window_len.to_be_bytes());
        hasher.update(window);
        Ok(Self {
            window_len,
            digest: hasher.finalize().into(),
        })
    }

    /// Computes the canonical empty guard at offset zero.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            window_len: 0,
            digest: EMPTY_FRONTIER_GUARD_DIGEST,
        }
    }

    pub(crate) fn write(&self, out: &mut Writer) {
        out.u16(self.window_len);
        out.bytes(&self.digest);
    }

    pub(crate) fn read(input: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            window_len: input.u16()?,
            digest: input.array()?,
        })
    }

    pub(crate) fn valid_for_offset(&self, offset: u64) -> bool {
        let expected_len = offset.min(u64::from(COMMITTED_FRONTIER_GUARD_WINDOW_BYTES));
        self.window_len as u64 == expected_len
            && (offset != 0 || self.digest == EMPTY_FRONTIER_GUARD_DIGEST)
    }
}

/// Bounded runtime raw-byte window used to derive a frontier guard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommittedFrontierWindow {
    end_offset: u64,
    bytes: Vec<u8>,
}

impl CommittedFrontierWindow {
    /// Constructs a window whose bytes end at `end_offset`.
    pub fn new(end_offset: u64, bytes: Vec<u8>) -> Result<Self, EncodeError> {
        let expected = end_offset.min(u64::from(COMMITTED_FRONTIER_GUARD_WINDOW_BYTES)) as usize;
        if bytes.len() != expected {
            return Err(EncodeError::InvalidFieldValue {
                field: "committed_frontier_window.bytes",
                reason: "length must equal min(end_offset, 64)",
            });
        }
        Ok(Self { end_offset, bytes })
    }

    /// Returns the empty offset-zero window.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            end_offset: 0,
            bytes: Vec::new(),
        }
    }

    /// Returns the window end offset.
    #[must_use]
    pub const fn end_offset(&self) -> u64 {
        self.end_offset
    }

    /// Returns the retained exact bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Computes the corresponding durable guard.
    pub fn guard(&self) -> Result<CommittedFrontierGuard, EncodeError> {
        CommittedFrontierGuard::compute(self.end_offset, &self.bytes)
    }
}

/// Computes the version 1 namespace digest for an exact checkpoint ID.
pub fn namespace_digest(checkpoint_id: &str) -> Result<[u8; 32], EncodeError> {
    let bytes = checkpoint_id.as_bytes();
    if bytes.is_empty() {
        return Err(EncodeError::RequiredFieldEmpty {
            field: "checkpoint_id",
        });
    }
    if bytes.len() > NAMESPACE_ID_MAX_BYTES {
        return Err(EncodeError::FieldTooLong {
            field: "checkpoint_id",
            len: bytes.len(),
            max: NAMESPACE_ID_MAX_BYTES,
        });
    }
    let mut hasher = Sha256::new();
    hasher.update(NAMESPACE_DOMAIN);
    hasher.update((bytes.len() as u16).to_be_bytes());
    hasher.update(bytes);
    Ok(hasher.finalize().into())
}

/// Computes reflected Castagnoli CRC-32C.
#[must_use]
pub fn crc32c(bytes: &[u8]) -> u32 {
    const CRC: Crc<u32> = Crc::<u32>::new(&CRC_32_ISCSI);
    CRC.checksum(bytes)
}

pub(crate) struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    pub(crate) const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    pub(crate) fn remaining(&self) -> usize {
        self.bytes.len() - self.pos
    }

    pub(crate) const fn position(&self) -> usize {
        self.pos
    }

    pub(crate) fn exact(&mut self, len: usize) -> Result<&'a [u8], DecodeError> {
        let end = self
            .pos
            .checked_add(len)
            .ok_or(DecodeError::ArithmeticOverflow {
                context: "reader position + field length",
            })?;
        if end > self.bytes.len() {
            return Err(DecodeError::Truncated {
                needed: len,
                available: self.remaining(),
            });
        }
        let result = &self.bytes[self.pos..end];
        self.pos = end;
        Ok(result)
    }

    pub(crate) fn array<const N: usize>(&mut self) -> Result<[u8; N], DecodeError> {
        let mut result = [0; N];
        result.copy_from_slice(self.exact(N)?);
        Ok(result)
    }

    pub(crate) fn u8(&mut self) -> Result<u8, DecodeError> {
        Ok(self.array::<1>()?[0])
    }

    pub(crate) fn u16(&mut self) -> Result<u16, DecodeError> {
        Ok(u16::from_be_bytes(self.array()?))
    }

    pub(crate) fn u32(&mut self) -> Result<u32, DecodeError> {
        Ok(u32::from_be_bytes(self.array()?))
    }

    pub(crate) fn u64(&mut self) -> Result<u64, DecodeError> {
        Ok(u64::from_be_bytes(self.array()?))
    }

    pub(crate) fn var_bytes(
        &mut self,
        field: &'static str,
        max: usize,
    ) -> Result<&'a [u8], DecodeError> {
        let len = usize::from(self.u16()?);
        if len > max {
            return Err(DecodeError::LengthExceedsMaximum {
                field,
                declared: len as u64,
                max: max as u64,
            });
        }
        self.exact(len)
    }

    pub(crate) fn var_string(
        &mut self,
        field: &'static str,
        max: usize,
    ) -> Result<&'a str, DecodeError> {
        std::str::from_utf8(self.var_bytes(field, max)?)
            .map_err(|_| DecodeError::InvalidUtf8 { field })
    }
}

#[derive(Default)]
pub(crate) struct Writer(Vec<u8>);

impl Writer {
    pub(crate) const fn new() -> Self {
        Self(Vec::new())
    }

    pub(crate) fn bytes(&mut self, bytes: &[u8]) {
        self.0.extend_from_slice(bytes);
    }

    pub(crate) fn u8(&mut self, value: u8) {
        self.0.push(value);
    }

    pub(crate) fn u16(&mut self, value: u16) {
        self.bytes(&value.to_be_bytes());
    }

    pub(crate) fn u32(&mut self, value: u32) {
        self.bytes(&value.to_be_bytes());
    }

    pub(crate) fn u64(&mut self, value: u64) {
        self.bytes(&value.to_be_bytes());
    }

    pub(crate) fn var_bytes(
        &mut self,
        field: &'static str,
        bytes: &[u8],
        max: usize,
    ) -> Result<(), EncodeError> {
        if bytes.len() > max {
            return Err(EncodeError::FieldTooLong {
                field,
                len: bytes.len(),
                max,
            });
        }
        self.u16(bytes.len() as u16);
        self.bytes(bytes);
        Ok(())
    }

    pub(crate) fn as_slice(&self) -> &[u8] {
        &self.0
    }

    pub(crate) fn finish(self) -> Vec<u8> {
        self.0
    }
}

pub(crate) const fn quarantine_reason_reserved(value: u16) -> bool {
    value == REASON_CODE_RESERVED || value == QUARANTINE_REASON_RESERVED_V1
}
