// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::super::model::{SampleFailure, ScaleError};
use super::counter_type::UnsupportedNativeType;
use std::sync::Arc;
use windows_sys::Win32::System::Performance::PDH_INVALID_HANDLE;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in super::super) enum PdhOperation {
    AddEnglishCounter,
    AddCounter,
    CollectQueryData,
    ExpandWildcardPath,
    ExpandWildcardPathSize,
    FormattedCounterDataStatus,
    GetCounterInfo,
    GetCounterInfoSize,
    GetFormattedCounterValue,
    GetRawCounterValue,
    OpenQuery,
    ParseCounterPath,
    ParseCounterPathSize,
    RawCounterDataStatus,
    RemoveCounter,
}

impl PdhOperation {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::AddEnglishCounter => "PdhAddEnglishCounterW",
            Self::AddCounter => "PdhAddCounterW",
            Self::CollectQueryData => "PdhCollectQueryData",
            Self::ExpandWildcardPath => "PdhExpandWildCardPathW",
            Self::ExpandWildcardPathSize => "PdhExpandWildCardPathW(size)",
            Self::FormattedCounterDataStatus => "formatted counter CStatus",
            Self::GetCounterInfo => "PdhGetCounterInfoW",
            Self::GetCounterInfoSize => "PdhGetCounterInfoW(size)",
            Self::GetFormattedCounterValue => "PdhGetFormattedCounterValue",
            Self::GetRawCounterValue => "PdhGetRawCounterValue",
            Self::OpenQuery => "PdhOpenQueryW",
            Self::ParseCounterPath => "PdhParseCounterPathW",
            Self::ParseCounterPathSize => "PdhParseCounterPathW(size)",
            Self::RawCounterDataStatus => "raw counter CStatus",
            Self::RemoveCounter => "PdhRemoveCounter",
        }
    }
}

impl std::fmt::Display for PdhOperation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, thiserror::Error)]
pub(in super::super) enum Error {
    #[error("{operation} for {path} failed with PDH status 0x{status:08X}")]
    Pdh {
        operation: PdhOperation,
        path: String,
        status: u32,
    },
    /// A counter's CStatus failure, separate from a PDH API return code.
    #[error("{operation} for {path} failed with counter data status 0x{status:08X}")]
    CounterData {
        operation: PdhOperation,
        path: String,
        status: u32,
    },
    #[error(transparent)]
    UnsupportedType(#[from] UnsupportedNativeType),
    #[error("counter setup for {path} failed: {message}")]
    CounterSetup {
        path: String,
        message: String,
        permanent: bool,
        cleanup_failed: bool,
    },
    #[error("invalid performance-counter token for {path}")]
    InvalidCounterToken { path: String },
    #[error("invalid performance-counter sample: {0}")]
    InvalidSample(&'static str),
    #[error("performance-counter timestamp read failed: {0}")]
    Timestamp(&'static str),
    #[error("no usable performance counters or recoverable availability work: {detail}")]
    NoUsableCounters { detail: String },
    #[error("counter calculation for {path} failed: {message}")]
    Calculation { path: String, message: String },
    #[error("counter scaling for {path} failed: {source}")]
    Scale { path: String, source: ScaleError },
    #[error("wildcard lookup cleanup for {path} failed: {message}")]
    WildcardCleanup {
        path: String,
        message: String,
        lookup_failed: bool,
    },
    #[error("PDH worker object-list {stage} failed with status 0x{status:08X}")]
    WorkerObjectRefresh { stage: &'static str, status: u32 },
    #[error("PDH worker stop was requested")]
    WorkerStopRequested,
}

impl Error {
    pub(super) fn is_stop(&self) -> bool {
        matches!(self, Self::WorkerStopRequested)
    }

    pub(super) fn is_query_collection_failure(&self) -> bool {
        matches!(
            self,
            Self::Pdh {
                operation: PdhOperation::CollectQueryData,
                ..
            }
        )
    }

    pub(super) fn requires_immediate_query_rebuild(&self) -> bool {
        matches!(
            self,
            Self::CounterSetup {
                cleanup_failed: true,
                ..
            }
        )
    }

    pub(super) fn is_invalid_counter_handle_read(&self) -> bool {
        matches!(
            self,
            Self::Pdh {
                operation:
                    PdhOperation::GetRawCounterValue | PdhOperation::GetFormattedCounterValue,
                status,
                ..
            } if *status == PDH_INVALID_HANDLE
        )
    }

    pub(super) fn reason(&self) -> &'static str {
        match self {
            Self::Pdh { operation, .. } | Self::CounterData { operation, .. } => operation.as_str(),
            Self::UnsupportedType(_) => "unsupported_type",
            Self::CounterSetup { .. } => "counter_setup",
            Self::InvalidCounterToken { .. } => "invalid_counter_token",
            Self::InvalidSample(_) => "invalid_sample",
            Self::Timestamp(_) => "timestamp",
            Self::NoUsableCounters { .. } => "no_usable_counters",
            Self::Calculation { .. } => "calculation",
            Self::Scale { .. } => "scale",
            Self::WildcardCleanup { .. } => "wildcard_lookup_cleanup",
            Self::WorkerObjectRefresh { .. } => "worker_object_refresh",
            Self::WorkerStopRequested => "worker_stop_requested",
        }
    }

    pub(super) fn bounded_detail(&self) -> String {
        match self {
            Self::Pdh { status, .. } => format!("PDH status 0x{status:08X}"),
            Self::CounterData { status, .. } => {
                format!("counter data status 0x{status:08X}")
            }
            Self::UnsupportedType(error) => {
                format!("unsupported native type 0x{:08X}", error.native_type)
            }
            Self::CounterSetup { message, .. } => message.clone(),
            Self::InvalidCounterToken { .. } => self.to_string(),
            Self::InvalidSample(message) | Self::Timestamp(message) => (*message).to_owned(),
            Self::Calculation { message, .. } => message.clone(),
            Self::Scale { source, .. } => source.to_string(),
            other => other.to_string(),
        }
    }

    fn path(&self) -> Option<&str> {
        match self {
            Self::Pdh { path, .. }
            | Self::CounterData { path, .. }
            | Self::CounterSetup { path, .. }
            | Self::WildcardCleanup { path, .. }
            | Self::InvalidCounterToken { path }
            | Self::Calculation { path, .. }
            | Self::Scale { path, .. } => Some(path),
            Self::UnsupportedType(error) => Some(&error.path),
            _ => None,
        }
    }

    pub(super) fn is_permanent_counter_error(&self) -> bool {
        matches!(
            self,
            Self::UnsupportedType(_)
                | Self::CounterSetup {
                    permanent: true,
                    ..
                }
        )
    }

    pub(super) fn with_cleanup_failure(path: &str, primary: Self, cleanup: Self) -> Self {
        let permanent = primary.is_permanent_counter_error();
        let message = format!("{primary}; {cleanup}");
        Self::CounterSetup {
            path: path.to_owned(),
            message,
            permanent,
            cleanup_failed: true,
        }
    }
}

pub(super) fn sample_failure(counter_index: usize, error: &Error) -> SampleFailure {
    SampleFailure {
        counter_index,
        path: Arc::from(error.path().unwrap_or("<worker>")),
        reason: error.reason(),
        error: error.bounded_detail(),
        diagnostic_only: false,
    }
}
