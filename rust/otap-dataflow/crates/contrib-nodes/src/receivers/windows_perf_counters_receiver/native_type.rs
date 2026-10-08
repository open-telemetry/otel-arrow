// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Native counter types and their sample requirements, independent of PDH calls.
//!
//! Definitions follow Windows SDK 10.0.26100.0 `winperf.h`:
//! <https://github.com/microsoft/win32metadata/blob/main/generation/WinSDK/RecompiledIdlHeaders/um/winperf.h>.

const PERF_SIZE_DWORD: u32 = 0x0000_0000;
const PERF_SIZE_LARGE: u32 = 0x0000_0100;
const PERF_TYPE_NUMBER: u32 = 0x0000_0000;
const PERF_TYPE_COUNTER: u32 = 0x0000_0400;
const PERF_NUMBER_HEX: u32 = 0x0000_0000;
const PERF_NUMBER_DECIMAL: u32 = 0x0001_0000;
const PERF_COUNTER_RATE: u32 = 0x0001_0000;
const PERF_COUNTER_FRACTION: u32 = 0x0002_0000;
const PERF_COUNTER_QUEUELEN: u32 = 0x0005_0000;
const PERF_COUNTER_PRECISION: u32 = 0x0007_0000;
const PERF_TIMER_100NS: u32 = 0x0010_0000;
const PERF_OBJECT_TIMER: u32 = 0x0020_0000;
const PERF_DELTA_COUNTER: u32 = 0x0040_0000;
const PERF_DELTA_BASE: u32 = 0x0080_0000;
const PERF_INVERSE_COUNTER: u32 = 0x0100_0000;
const PERF_DISPLAY_NO_SUFFIX: u32 = 0x0000_0000;
const PERF_DISPLAY_PER_SEC: u32 = 0x1000_0000;
const PERF_DISPLAY_PERCENT: u32 = 0x2000_0000;
const PERF_DISPLAY_SECONDS: u32 = 0x3000_0000;
const PERF_DISPLAY_NOSHOW: u32 = 0x4000_0000;

const PERF_COUNTER_RAWCOUNT: u32 =
    PERF_SIZE_DWORD | PERF_TYPE_NUMBER | PERF_NUMBER_DECIMAL | PERF_DISPLAY_NO_SUFFIX;
const PERF_COUNTER_LARGE_RAWCOUNT: u32 =
    PERF_SIZE_LARGE | PERF_TYPE_NUMBER | PERF_NUMBER_DECIMAL | PERF_DISPLAY_NO_SUFFIX;
const PERF_COUNTER_RAWCOUNT_HEX: u32 =
    PERF_SIZE_DWORD | PERF_TYPE_NUMBER | PERF_NUMBER_HEX | PERF_DISPLAY_NO_SUFFIX;
const PERF_COUNTER_LARGE_RAWCOUNT_HEX: u32 =
    PERF_SIZE_LARGE | PERF_TYPE_NUMBER | PERF_NUMBER_HEX | PERF_DISPLAY_NO_SUFFIX;
const PERF_COUNTER_COUNTER: u32 = PERF_SIZE_DWORD
    | PERF_TYPE_COUNTER
    | PERF_COUNTER_RATE
    | PERF_DELTA_COUNTER
    | PERF_DISPLAY_PER_SEC;
const PERF_COUNTER_BULK_COUNT: u32 = PERF_SIZE_LARGE
    | PERF_TYPE_COUNTER
    | PERF_COUNTER_RATE
    | PERF_DELTA_COUNTER
    | PERF_DISPLAY_PER_SEC;
const PERF_COUNTER_QUEUELEN_TYPE: u32 =
    PERF_SIZE_DWORD | PERF_TYPE_COUNTER | PERF_COUNTER_QUEUELEN | PERF_DELTA_COUNTER;
const PERF_COUNTER_LARGE_QUEUELEN_TYPE: u32 =
    PERF_SIZE_LARGE | PERF_TYPE_COUNTER | PERF_COUNTER_QUEUELEN | PERF_DELTA_COUNTER;
const PERF_COUNTER_100NS_QUEUELEN_TYPE: u32 = PERF_SIZE_LARGE
    | PERF_TYPE_COUNTER
    | PERF_COUNTER_QUEUELEN
    | PERF_TIMER_100NS
    | PERF_DELTA_COUNTER;
const PERF_COUNTER_OBJ_TIME_QUEUELEN_TYPE: u32 = PERF_SIZE_LARGE
    | PERF_TYPE_COUNTER
    | PERF_COUNTER_QUEUELEN
    | PERF_OBJECT_TIMER
    | PERF_DELTA_COUNTER;
const PERF_COUNTER_TIMER: u32 = PERF_SIZE_LARGE
    | PERF_TYPE_COUNTER
    | PERF_COUNTER_RATE
    | PERF_DELTA_COUNTER
    | PERF_DISPLAY_PERCENT;
const PERF_COUNTER_TIMER_INV: u32 = PERF_COUNTER_TIMER | PERF_INVERSE_COUNTER;
const PERF_100NSEC_TIMER: u32 = PERF_COUNTER_TIMER | PERF_TIMER_100NS;
const PERF_100NSEC_TIMER_INV: u32 = PERF_100NSEC_TIMER | PERF_INVERSE_COUNTER;
const PERF_OBJ_TIME_TIMER: u32 = PERF_COUNTER_TIMER | PERF_OBJECT_TIMER;
const PERF_PRECISION_SYSTEM_TIMER: u32 = PERF_SIZE_LARGE
    | PERF_TYPE_COUNTER
    | PERF_COUNTER_PRECISION
    | PERF_DELTA_COUNTER
    | PERF_DISPLAY_PERCENT;
const PERF_PRECISION_100NS_TIMER: u32 = PERF_PRECISION_SYSTEM_TIMER | PERF_TIMER_100NS;
const PERF_PRECISION_OBJECT_TIMER: u32 = PERF_PRECISION_SYSTEM_TIMER | PERF_OBJECT_TIMER;
const PERF_COUNTER_DELTA: u32 =
    PERF_SIZE_DWORD | PERF_TYPE_COUNTER | PERF_DELTA_COUNTER | PERF_DISPLAY_NO_SUFFIX;
const PERF_COUNTER_LARGE_DELTA: u32 =
    PERF_SIZE_LARGE | PERF_TYPE_COUNTER | PERF_DELTA_COUNTER | PERF_DISPLAY_NO_SUFFIX;
const PERF_SAMPLE_COUNTER: u32 = PERF_SIZE_DWORD
    | PERF_TYPE_COUNTER
    | PERF_COUNTER_RATE
    | PERF_DELTA_COUNTER
    | PERF_DISPLAY_NO_SUFFIX;
const PERF_RAW_FRACTION: u32 =
    PERF_SIZE_DWORD | PERF_TYPE_COUNTER | PERF_COUNTER_FRACTION | PERF_DISPLAY_PERCENT;
const PERF_LARGE_RAW_FRACTION: u32 =
    PERF_SIZE_LARGE | PERF_TYPE_COUNTER | PERF_COUNTER_FRACTION | PERF_DISPLAY_PERCENT;
const PERF_SAMPLE_FRACTION: u32 = PERF_SIZE_DWORD
    | PERF_TYPE_COUNTER
    | PERF_COUNTER_FRACTION
    | PERF_DELTA_COUNTER
    | PERF_DELTA_BASE
    | PERF_DISPLAY_PERCENT;
const PERF_AVERAGE_TIMER: u32 =
    PERF_SIZE_DWORD | PERF_TYPE_COUNTER | PERF_COUNTER_FRACTION | PERF_DISPLAY_SECONDS;
const PERF_AVERAGE_BULK: u32 =
    PERF_SIZE_LARGE | PERF_TYPE_COUNTER | PERF_COUNTER_FRACTION | PERF_DISPLAY_NOSHOW;

/// Native sample requirements, not the configured OpenTelemetry metric kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CounterKind {
    /// A raw count requiring only the current sample.
    Direct,
    /// A fraction requiring the current numerator and a positive current base.
    RawFraction,
    /// A PDH formula requiring successive samples (including private-clock timers).
    CalculatedTwoSample,
    /// A fraction or average requiring successive samples and advancing base values.
    CalculatedTwoSampleWithBase,
}

/// Metadata outside the supported native-type allowlist.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error(
    "unsupported native counter type 0x{native_type:08X} for {path}; \
     supported types are direct raw counts, rates, deltas, queue lengths, \
     single-counter timers, fractions, and averages"
)]
pub(super) struct UnsupportedNativeType {
    /// Counter path to identify the unsupported observation source.
    pub(super) path: String,
    /// Full native type, retained for diagnostics rather than masked to a family.
    pub(super) native_type: u32,
}

/// Classify exact SDK types; unknown flag combinations must not inherit support.
///
/// Standalone bases, text and no-data types are not observations. Elapsed-time
/// and multi-timer support remains undecided and is deliberately excluded.
pub(super) fn classify_native_type(
    path: &str,
    native_type: u32,
) -> Result<CounterKind, UnsupportedNativeType> {
    match native_type {
        PERF_COUNTER_RAWCOUNT
        | PERF_COUNTER_LARGE_RAWCOUNT
        | PERF_COUNTER_RAWCOUNT_HEX
        | PERF_COUNTER_LARGE_RAWCOUNT_HEX => Ok(CounterKind::Direct),
        PERF_COUNTER_COUNTER
        | PERF_COUNTER_BULK_COUNT
        | PERF_COUNTER_QUEUELEN_TYPE
        | PERF_COUNTER_LARGE_QUEUELEN_TYPE
        | PERF_COUNTER_100NS_QUEUELEN_TYPE
        | PERF_COUNTER_OBJ_TIME_QUEUELEN_TYPE
        | PERF_COUNTER_TIMER
        | PERF_COUNTER_TIMER_INV
        | PERF_100NSEC_TIMER
        | PERF_100NSEC_TIMER_INV
        | PERF_OBJ_TIME_TIMER
        | PERF_PRECISION_SYSTEM_TIMER
        | PERF_PRECISION_100NS_TIMER
        | PERF_PRECISION_OBJECT_TIMER
        | PERF_COUNTER_DELTA
        | PERF_COUNTER_LARGE_DELTA
        | PERF_SAMPLE_COUNTER => Ok(CounterKind::CalculatedTwoSample),
        PERF_SAMPLE_FRACTION | PERF_AVERAGE_TIMER | PERF_AVERAGE_BULK => {
            Ok(CounterKind::CalculatedTwoSampleWithBase)
        }
        PERF_RAW_FRACTION | PERF_LARGE_RAW_FRACTION => Ok(CounterKind::RawFraction),
        _ => Err(UnsupportedNativeType {
            path: path.to_owned(),
            native_type,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::{CounterKind, classify_native_type};

    /// Scenario: SDK metadata describes DWORD/LARGE raw counts in decimal or hexadecimal.
    /// Guarantees: All four types are direct values, including the valid zero-valued type code.
    #[test]
    fn classifies_direct_counts() {
        for native_type in [0x0001_0000, 0x0001_0100, 0x0000_0000, 0x0000_0100] {
            assert_eq!(
                classify_native_type("direct", native_type).unwrap(),
                CounterKind::Direct,
                "native type 0x{native_type:08X}"
            );
        }
    }

    /// Scenario: SDK metadata describes rates, queues, ordinary/private-clock timers and deltas.
    /// Guarantees: Every supported type requires two samples, including inverse and object timers.
    #[test]
    fn classifies_two_sample_counters() {
        for native_type in [
            0x1041_0400, // PERF_COUNTER_COUNTER
            0x1041_0500, // PERF_COUNTER_BULK_COUNT
            0x0045_0400, // PERF_COUNTER_QUEUELEN_TYPE
            0x0045_0500, // PERF_COUNTER_LARGE_QUEUELEN_TYPE
            0x0055_0500, // PERF_COUNTER_100NS_QUEUELEN_TYPE
            0x0065_0500, // PERF_COUNTER_OBJ_TIME_QUEUELEN_TYPE
            0x2041_0500, // PERF_COUNTER_TIMER
            0x2141_0500, // PERF_COUNTER_TIMER_INV
            0x2051_0500, // PERF_100NSEC_TIMER
            0x2151_0500, // PERF_100NSEC_TIMER_INV
            0x2061_0500, // PERF_OBJ_TIME_TIMER
            0x2047_0500, // PERF_PRECISION_SYSTEM_TIMER
            0x2057_0500, // PERF_PRECISION_100NS_TIMER
            0x2067_0500, // PERF_PRECISION_OBJECT_TIMER
            0x0040_0400, // PERF_COUNTER_DELTA
            0x0040_0500, // PERF_COUNTER_LARGE_DELTA
            0x0041_0400, // PERF_SAMPLE_COUNTER
        ] {
            assert_eq!(
                classify_native_type("calculated", native_type).unwrap(),
                CounterKind::CalculatedTwoSample,
                "native type 0x{native_type:08X}"
            );
        }
    }

    /// Scenario: SDK metadata describes raw fractions, sample fractions and timer/bulk averages.
    /// Guarantees: Current-sample fractions remain distinct from formulas needing advancing bases.
    #[test]
    fn classifies_fractions_and_averages() {
        for native_type in [0x2002_0400, 0x2002_0500] {
            assert_eq!(
                classify_native_type("raw fraction", native_type).unwrap(),
                CounterKind::RawFraction,
                "native type 0x{native_type:08X}"
            );
        }
        for native_type in [0x20C2_0400, 0x3002_0400, 0x4002_0500] {
            assert_eq!(
                classify_native_type("delta/base", native_type).unwrap(),
                CounterKind::CalculatedTwoSampleWithBase,
                "native type 0x{native_type:08X}"
            );
        }
    }

    /// Scenario: A standalone sample, average, raw, large/precision or multi base is selected.
    /// Guarantees: Bases are rejected with their path and complete hexadecimal native type.
    #[test]
    fn rejects_standalone_bases_with_diagnostics() {
        let path = r"\Object\Counter Base";
        for native_type in [
            0x4003_0401, // PERF_SAMPLE_BASE
            0x4003_0402, // PERF_AVERAGE_BASE
            0x4003_0403, // PERF_RAW_BASE
            0x4003_0500, // PERF_LARGE_RAW_BASE / PERF_PRECISION_TIMESTAMP
            0x4203_0500, // PERF_COUNTER_MULTI_BASE
        ] {
            let error = classify_native_type(path, native_type).unwrap_err();
            assert_eq!(error.path, path);
            assert_eq!(error.native_type, native_type);
            let message = error.to_string();
            assert!(message.contains(path), "{message}");
            assert!(
                message.contains(&format!("0x{native_type:08X}")),
                "{message}"
            );
            assert!(message.contains("supported types are"), "{message}");
        }
    }

    /// Scenario: Metadata describes nonnumeric types or elapsed/multi-timer families without policy.
    /// Guarantees: No-data, text, histogram and undecided timer types do not become observations.
    #[test]
    fn rejects_non_observations_and_undecided_families() {
        for native_type in [
            0x0000_0B00, // PERF_COUNTER_TEXT
            0x4000_0200, // PERF_COUNTER_NODATA
            0x8000_0000, // PERF_COUNTER_HISTOGRAM_TYPE
            0x3024_0500, // PERF_ELAPSED_TIME
            0x2241_0500, // PERF_COUNTER_MULTI_TIMER
            0x2341_0500, // PERF_COUNTER_MULTI_TIMER_INV
            0x2251_0500, // PERF_100NSEC_MULTI_TIMER
            0x2351_0500, // PERF_100NSEC_MULTI_TIMER_INV
        ] {
            let error = classify_native_type("unsupported", native_type).unwrap_err();
            assert_eq!(error.native_type, native_type);
        }
    }

    /// Scenario: An unknown native type shares family bits with otherwise supported SDK types.
    /// Guarantees: Reserved bits and unrecognized combinations cannot bypass the exact allowlist.
    #[test]
    fn rejects_unknown_types_and_flag_combinations() {
        for native_type in [
            0xFFFF_FFFF,
            0x0001_0001, // raw count with a reserved low bit
            0x1041_0401, // rate with a reserved low bit
            0x2041_0501, // timer with a reserved low bit
            0x2002_0401, // fraction with a reserved low bit
            0x3002_0401, // average with a reserved low bit
            0x0001_0200, // number with zero-length size
        ] {
            let error = classify_native_type("unknown", native_type).unwrap_err();
            assert_eq!(error.native_type, native_type);
        }
    }
}
