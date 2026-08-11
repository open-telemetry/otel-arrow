// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The transport-neutral outcome vocabulary returned by an admission gate.

/// Outcome of one weighted admission check.
///
/// The vocabulary is deliberately generic: it says what the caller must *do*,
/// not how the provider reached the verdict. Nothing here mentions token
/// buckets, memory pressure, HTTP status codes, or gRPC codes, so a component
/// maps a decision onto its own protocol without depending on the provider's
/// internals, and a custom provider using a different algorithm can express
/// itself with the same four outcomes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionDecision {
    /// The request is admitted. The caller proceeds normally.
    Admit,
    /// The request is admitted, but an enforcing provider would have thrown it
    /// away.
    ///
    /// Emitted by observe-only configurations. The caller **must** proceed as
    /// if the decision were [`Admit`](Self::Admit); the variant exists so a
    /// component can record a protocol-specific "would refuse" counter and let
    /// an operator size a limit before enabling enforcement.
    WouldThrottle,
    /// The request is rejected, and an identical request may succeed later.
    ///
    /// `retry_after_secs` is the provider's retry hint, already clamped to at
    /// least one second so a component can render it into `Retry-After` or
    /// `grpc-retry-pushback-ms` without re-clamping.
    Throttle {
        /// Seconds a client should wait before retrying. Always `>= 1`.
        retry_after_secs: u32,
    },
    /// The request is rejected permanently: its weight can never fit, no matter
    /// how long the client waits.
    ///
    /// Components must translate this into a non-retryable protocol response
    /// (for example `grpc-retry-pushback-ms: -1`) rather than advertising a
    /// retry hint that can never succeed.
    Oversized,
}

impl AdmissionDecision {
    /// Returns true when the caller must continue processing the request.
    ///
    /// Both [`Admit`](Self::Admit) and [`WouldThrottle`](Self::WouldThrottle)
    /// are admissions; only the telemetry differs.
    #[must_use]
    pub const fn is_admitted(self) -> bool {
        matches!(self, Self::Admit | Self::WouldThrottle)
    }

    /// Returns the retry hint when one applies.
    ///
    /// [`Oversized`](Self::Oversized) deliberately returns `None`: advertising
    /// a retry delay for a request that can never fit misleads clients into
    /// an infinite retry loop.
    #[must_use]
    pub const fn retry_after_secs(self) -> Option<u32> {
        match self {
            Self::Throttle { retry_after_secs } => Some(retry_after_secs),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: a component decides whether to keep processing a request.
    /// Guarantees: observe-only would-throttle counts as an admission, while both
    /// rejection variants do not, so observe-only never drops traffic.
    #[test]
    fn only_rejections_stop_request_processing() {
        assert!(AdmissionDecision::Admit.is_admitted());
        assert!(AdmissionDecision::WouldThrottle.is_admitted());
        assert!(
            !AdmissionDecision::Throttle {
                retry_after_secs: 3
            }
            .is_admitted()
        );
        assert!(!AdmissionDecision::Oversized.is_admitted());
    }

    /// Scenario: a component renders a retry hint onto its protocol response.
    /// Guarantees: only the retryable rejection carries a hint; a permanently
    /// oversized request never advertises a delay that cannot help.
    #[test]
    fn retry_hint_exists_only_for_retryable_rejection() {
        assert_eq!(
            AdmissionDecision::Throttle {
                retry_after_secs: 7
            }
            .retry_after_secs(),
            Some(7)
        );
        assert_eq!(AdmissionDecision::Oversized.retry_after_secs(), None);
        assert_eq!(AdmissionDecision::Admit.retry_after_secs(), None);
        assert_eq!(AdmissionDecision::WouldThrottle.retry_after_secs(), None);
    }
}
