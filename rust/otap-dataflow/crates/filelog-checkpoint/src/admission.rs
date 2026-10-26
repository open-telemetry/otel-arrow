// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Portable checkpoint size and recovery admission; no filesystem access.

use crate::{
    CHECKPOINT_HEADER_BYTES, FINGERPRINT_MAX_BYTES, SNAPSHOT_FOOTER_BYTES, SNAPSHOT_HEADER_BYTES,
    SNAPSHOT_MAX_RECORD_FRAME_BYTES, WAL_MAX_TX_FRAME_BYTES,
};

/// Maximum complete checkpoint and either recovery phase, in bytes.
pub const CHECKPOINT_MAX_BYTES: u64 = 1 << 30;

/// A failed arithmetic operation or a calculated value above its ceiling.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum AdmissionError {
    /// Inputs do not describe a supported checkpoint bound.
    #[error("{setting}: {reason}")]
    Invalid {
        /// Setting that failed validation.
        setting: &'static str,
        /// Required constraint.
        reason: &'static str,
    },
    /// A named formula cannot be represented.
    #[error(
        "{formula}: arithmetic overflow; snapshot_bytes={snapshot_bytes}, wal_bytes={wal_bytes}"
    )]
    Overflow {
        /// Formula that overflowed.
        formula: &'static str,
        /// Contributing snapshot bound.
        snapshot_bytes: u64,
        /// Contributing capped WAL bound.
        wal_bytes: u64,
    },
    /// A representable formula exceeds its specified ceiling.
    #[error(
        "{formula}={value} exceeds ceiling={ceiling}; snapshot_bytes={snapshot_bytes}, wal_bytes={wal_bytes}"
    )]
    Ceiling {
        /// Formula that exceeds the ceiling.
        formula: &'static str,
        /// Calculated value.
        value: u64,
        /// Normative maximum.
        ceiling: u64,
        /// Contributing snapshot bound.
        snapshot_bytes: u64,
        /// Contributing capped WAL bound.
        wal_bytes: u64,
    },
}

/// Bounds for the complete file and sequential recovery phases.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CheckpointBounds {
    /// Snapshot section, excluding the container header.
    pub snapshot_bytes: u64,
    /// WAL bytes capped by both configured thresholds.
    pub wal_bytes: u64,
    /// Complete container including its header.
    pub checkpoint_file_bytes: u64,
    /// Conservative snapshot-phase working set.
    pub snapshot_phase_bytes: u64,
    /// Conservative WAL-phase working set.
    pub wal_phase_bytes: u64,
}

/// Maximum snapshot section for a configured population and fingerprint window.
pub fn maximum_snapshot_bytes(tracked: u32, fingerprint: u16) -> Result<u64, AdmissionError> {
    if tracked == 0 || fingerprint < 16 {
        return Err(AdmissionError::Invalid {
            setting: "limits.max_tracked_files/identity.fingerprint_bytes",
            reason: "nonzero tracked population and fingerprint_bytes >= 16 required",
        });
    }
    // All non-fingerprint fields retain their format-defined maximum widths.
    let frame =
        SNAPSHOT_MAX_RECORD_FRAME_BYTES - FINGERPRINT_MAX_BYTES as u64 + u64::from(fingerprint);
    u64::from(tracked)
        .checked_mul(frame)
        .and_then(|n| n.checked_add((SNAPSHOT_HEADER_BYTES + SNAPSHOT_FOOTER_BYTES) as u64))
        .ok_or(AdmissionError::Overflow {
            formula: "maximum_snapshot_bytes",
            snapshot_bytes: 0,
            wal_bytes: 0,
        })
}

impl CheckpointBounds {
    /// Admit the configured population while retaining older fingerprint windows.
    /// The snapshot budget is independent of the selected window once its own
    /// worst case fits. Historical records may use the remaining recovery budget,
    /// up to the format maximum per record, without raising any 1 GiB ceiling.
    pub fn for_population(
        tracked: u32,
        fingerprint: u16,
        wal_threshold: u64,
        transactions: u64,
    ) -> Result<Self, AdmissionError> {
        let current = Self::new(
            maximum_snapshot_bytes(tracked, fingerprint)?,
            wal_threshold,
            transactions,
        )?;
        // Validation above proves all subtractions and this addition fit. Each
        // extra snapshot byte costs one file byte, four snapshot-phase bytes,
        // and three WAL-phase bytes.
        let headroom = (CHECKPOINT_MAX_BYTES - current.checkpoint_file_bytes)
            .min((CHECKPOINT_MAX_BYTES - current.snapshot_phase_bytes) / 4)
            .min((CHECKPOINT_MAX_BYTES - current.wal_phase_bytes) / 3);
        let snapshot =
            maximum_snapshot_bytes(tracked, u16::MAX)?.min(current.snapshot_bytes + headroom);
        Self::new(snapshot, wal_threshold, transactions)
    }

    /// Derive and validate admission from explicit snapshot and WAL limits.
    /// The store uses this same calculation before opening a namespace.
    pub fn new(
        snapshot_bytes: u64,
        wal_threshold: u64,
        transactions: u64,
    ) -> Result<Self, AdmissionError> {
        if snapshot_bytes < (SNAPSHOT_HEADER_BYTES + SNAPSHOT_FOOTER_BYTES) as u64 {
            return Err(AdmissionError::Invalid {
                setting: "snapshot_bytes",
                reason: "must contain the snapshot header and footer",
            });
        }
        if wal_threshold < WAL_MAX_TX_FRAME_BYTES || transactions == 0 {
            return Err(AdmissionError::Invalid {
                setting: "checkpoint.compact_after_bytes/compact_after_transactions",
                reason: "at least one maximum transaction and a nonzero transaction count required",
            });
        }
        // This intermediate saturation preserves the exact mathematical minimum.
        let wal_bytes = transactions
            .saturating_mul(WAL_MAX_TX_FRAME_BYTES)
            .min(wal_threshold);
        let bounded = |formula, n: Option<u64>| {
            let value = n.ok_or(AdmissionError::Overflow {
                formula,
                snapshot_bytes,
                wal_bytes,
            })?;
            if value > CHECKPOINT_MAX_BYTES {
                return Err(AdmissionError::Ceiling {
                    formula,
                    value,
                    ceiling: CHECKPOINT_MAX_BYTES,
                    snapshot_bytes,
                    wal_bytes,
                });
            }
            Ok(value)
        };
        let header = CHECKPOINT_HEADER_BYTES as u64;
        let checkpoint_file_bytes = bounded(
            "checkpoint_file_bytes",
            snapshot_bytes
                .checked_add(header)
                .and_then(|n| n.checked_add(wal_bytes)),
        )?;
        let snapshot_phase_bytes = bounded(
            "snapshot_phase_bytes",
            snapshot_bytes
                .checked_mul(4)
                .and_then(|n| n.checked_add(header)),
        )?;
        let wal_phase_bytes = bounded(
            "wal_phase_bytes",
            snapshot_bytes
                .checked_mul(3)
                .and_then(|n| n.checked_add(wal_bytes))
                .and_then(|n| n.checked_add(4 * WAL_MAX_TX_FRAME_BYTES + header)),
        )?;
        Ok(Self {
            snapshot_bytes,
            wal_bytes,
            checkpoint_file_bytes,
            snapshot_phase_bytes,
            wal_phase_bytes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: Historical evidence exceeds the current window for small and default populations.
    /// Guarantees: Window reductions keep the same snapshot budget without exceeding recovery ceilings.
    #[test]
    fn historical_fingerprints_respect_recovery_ceiling() {
        for (tracked, old_window, wal) in [
            (1, u16::MAX, 64 << 20),
            (10_000, 1000, 64 << 20),
            (10_000, 1000, 700 << 20),
        ] {
            let old = CheckpointBounds::for_population(tracked, old_window, wal, 10_000).unwrap();
            let new = CheckpointBounds::for_population(tracked, 16, wal, 10_000).unwrap();
            assert_eq!(old, new);
            assert!(new.snapshot_bytes >= maximum_snapshot_bytes(tracked, old_window).unwrap());
            assert!(new.checkpoint_file_bytes <= CHECKPOINT_MAX_BYTES);
            assert!(new.snapshot_phase_bytes <= CHECKPOINT_MAX_BYTES);
            assert!(new.wal_phase_bytes <= CHECKPOINT_MAX_BYTES);
            if tracked == 1 {
                assert_eq!(
                    new.snapshot_bytes,
                    maximum_snapshot_bytes(1, u16::MAX).unwrap()
                );
            } else {
                assert!(CheckpointBounds::new(new.snapshot_bytes + 1, wal, 10_000).is_err());
            }
        }
        // Current-window admission must still reject an oversized population.
        assert!(CheckpointBounds::for_population(10_000, u16::MAX, 64 << 20, 10_000).is_err());
    }

    /// Scenario: A recovery formula reaches exactly its ceiling, then exceeds it by one byte.
    /// Guarantees: Equality is accepted and excess has a distinct, value-bearing error.
    #[test]
    fn exact_recovery_ceiling() {
        let snapshot = 84;
        let wal = CHECKPOINT_MAX_BYTES
            - 3 * snapshot
            - 4 * WAL_MAX_TX_FRAME_BYTES
            - CHECKPOINT_HEADER_BYTES as u64;
        let bounds = CheckpointBounds::new(snapshot, wal, u64::MAX).unwrap();
        assert_eq!(bounds.wal_phase_bytes, CHECKPOINT_MAX_BYTES);
        assert!(
            matches!(CheckpointBounds::new(snapshot, wal + 1, u64::MAX), Err(AdmissionError::Ceiling { formula: "wal_phase_bytes", value, .. }) if value == CHECKPOINT_MAX_BYTES + 1)
        );
    }

    /// Scenario: Input sums overflow, or a transaction product exceeds u64 before capping.
    /// Guarantees: Overflow is distinguished from a ceiling; only the capped intermediate may saturate.
    #[test]
    fn arithmetic_and_capped_product() {
        assert!(matches!(
            CheckpointBounds::new(u64::MAX, WAL_MAX_TX_FRAME_BYTES, 1),
            Err(AdmissionError::Overflow {
                formula: "checkpoint_file_bytes",
                ..
            })
        ));
        assert_eq!(
            CheckpointBounds::new(84, WAL_MAX_TX_FRAME_BYTES, u64::MAX)
                .unwrap()
                .wal_bytes,
            WAL_MAX_TX_FRAME_BYTES
        );
        assert_eq!(
            maximum_snapshot_bytes(1, u16::MAX).unwrap(),
            SNAPSHOT_MAX_RECORD_FRAME_BYTES + 84
        );
    }
}
