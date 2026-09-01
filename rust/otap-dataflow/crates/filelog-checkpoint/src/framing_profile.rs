// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Framing-profile canonical serialization and digest.

use sha2::{Digest, Sha256};

use crate::EncodeError;
use crate::primitives::{FRAMING_PATTERN_MAX_BYTES, Writer};

const DOMAIN: &[u8] = b"otel-arrow-filelog-framing-profile-v1\0";

/// Configured source encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FramingEncoding {
    /// UTF-8.
    Utf8,
    /// ASCII.
    Ascii,
    /// Little-endian UTF-16.
    Utf16Le,
    /// Big-endian UTF-16.
    Utf16Be,
    /// Raw bytes.
    Raw,
}

impl FramingEncoding {
    const fn wire(self) -> u8 {
        match self {
            Self::Utf8 => 1,
            Self::Ascii => 2,
            Self::Utf16Le => 3,
            Self::Utf16Be => 4,
            Self::Raw => 5,
        }
    }
}

/// Configured malformed-input behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FramingOnDecodeError {
    /// Preserve exact raw bytes.
    PreserveRaw,
    /// Replace malformed input.
    Replace,
    /// Fail and quarantine.
    Fail,
}

impl FramingOnDecodeError {
    const fn wire(self) -> u8 {
        match self {
            Self::PreserveRaw => 1,
            Self::Replace => 2,
            Self::Fail => 3,
        }
    }
}

/// Configured oversized-record behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaxLogSizeBehavior {
    /// Emit bounded fragments.
    Split,
    /// Emit a bounded prefix and discard the tail.
    Truncate,
}

impl MaxLogSizeBehavior {
    const fn wire(self) -> u8 {
        match self {
            Self::Split => 1,
            Self::Truncate => 2,
        }
    }
}

/// Framing boundary mode and its canonical pattern inputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MultilineMode {
    /// Newline framing with no regex input.
    Newline,
    /// Start-pattern multiline framing.
    StartPattern {
        /// Executable regex-profile version.
        regex_profile_version: u16,
        /// Exact UTF-8 regex source.
        pattern: String,
    },
    /// End-pattern multiline framing.
    EndPattern {
        /// Executable regex-profile version.
        regex_profile_version: u16,
        /// Exact UTF-8 regex source.
        pattern: String,
    },
}

impl MultilineMode {
    fn parts(&self) -> (u8, u16, &[u8]) {
        match self {
            Self::Newline => (0, 0, &[]),
            Self::StartPattern {
                regex_profile_version,
                pattern,
            } => (1, *regex_profile_version, pattern.as_bytes()),
            Self::EndPattern {
                regex_profile_version,
                pattern,
            } => (2, *regex_profile_version, pattern.as_bytes()),
        }
    }
}

/// Complete canonical framing and identity profile input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FramingProfileParams {
    /// Fingerprint recipe version.
    pub fingerprint_profile_version: u16,
    /// Configured fingerprint evidence window.
    pub fingerprint_bytes: u16,
    /// Bytes skipped before fingerprint evidence.
    pub ignored_header_bytes: u32,
    /// Source encoding.
    pub encoding: FramingEncoding,
    /// Malformed-input policy.
    pub on_decode_error: FramingOnDecodeError,
    /// Multiline mode and pattern.
    pub multiline_mode: MultilineMode,
    /// Physical-line bound.
    pub max_line_bytes: u64,
    /// Logical-record bound.
    pub max_record_bytes: u64,
    /// Oversized-record policy.
    pub max_log_size_behavior: MaxLogSizeBehavior,
    /// Multiline physical-line limit.
    pub max_multiline_lines: u32,
    /// Idle flush period in milliseconds; zero disables it.
    pub force_flush_period_millis: u64,
}

impl FramingProfileParams {
    /// Produces the exact version 1 canonical byte sequence.
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, EncodeError> {
        if self.fingerprint_profile_version == 0 {
            return Err(EncodeError::InvalidFieldValue {
                field: "framing_profile.fingerprint_profile_version",
                reason: "must be nonzero",
            });
        }
        if self.fingerprint_bytes < 16 {
            return Err(EncodeError::InvalidFieldValue {
                field: "framing_profile.fingerprint_bytes",
                reason: "must be at least 16",
            });
        }
        let (mode, regex_version, pattern) = self.multiline_mode.parts();
        if mode != 0 && regex_version == 0 {
            return Err(EncodeError::InvalidFieldValue {
                field: "framing_profile.regex_profile_version",
                reason: "pattern modes require a nonzero version",
            });
        }
        if mode != 0 && pattern.is_empty() {
            return Err(EncodeError::RequiredFieldEmpty {
                field: "framing_profile.pattern",
            });
        }
        let mut out = Writer::new();
        out.bytes(DOMAIN);
        out.u16(self.fingerprint_profile_version);
        out.u16(self.fingerprint_bytes);
        out.u32(self.ignored_header_bytes);
        out.u8(self.encoding.wire());
        out.u8(self.on_decode_error.wire());
        out.u8(mode);
        out.u16(regex_version);
        out.var_bytes(
            "framing_profile.pattern",
            pattern,
            FRAMING_PATTERN_MAX_BYTES,
        )?;
        out.u64(self.max_line_bytes);
        out.u64(self.max_record_bytes);
        out.u8(self.max_log_size_behavior.wire());
        out.u32(self.max_multiline_lines);
        out.u64(self.force_flush_period_millis);
        Ok(out.finish())
    }

    /// Computes SHA-256 over the exact canonical bytes.
    pub fn digest(&self) -> Result<[u8; 32], EncodeError> {
        let mut hasher = Sha256::new();
        hasher.update(self.canonical_bytes()?);
        Ok(hasher.finalize().into())
    }
}
