// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared mapping from a pipeline [`NackMsg`](otel_arrow_dfe_engine::control::NackMsg)
//! to the wire status codes returned by the OTLP receivers.
//!
//! Keeping this decision in one place guarantees the OTLP/HTTP and OTLP/gRPC
//! receivers report the same pipeline outcome with equivalent status codes.
//! The gRPC-code to HTTP-status pairing follows the OTLP failure conventions:
//! <https://github.com/open-telemetry/opentelemetry-proto/blob/main/docs/specification.md#failures>

use http::StatusCode;
use otel_arrow_dfe_engine::control::NackCause;
use tonic::Status;

/// Coarse classification of a NACK used to pick a response status code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NackClass {
    /// Permanent failure caused by the request itself; the client must change
    /// the request or its configuration. Non-retryable.
    ClientError,
    /// Permanent failure that is not the client's fault. Non-retryable.
    ServerError,
    /// Transient failure; the client may retry.
    Retryable,
}

/// Classifies a NACK from its permanence and cause.
///
/// A transient NACK is always retryable. A permanent NACK is a client error
/// only when it is explicitly classified as [`NackCause::Rejected`]; every
/// other permanent NACK is treated as a server-side failure.
pub(crate) const fn classify_nack(permanent: bool, cause: NackCause) -> NackClass {
    if permanent {
        match cause {
            NackCause::Rejected => NackClass::ClientError,
            NackCause::Unspecified
            | NackCause::RouteFull
            | NackCause::RouteClosed
            | NackCause::NodeShutdown => NackClass::ServerError,
        }
    } else {
        NackClass::Retryable
    }
}

impl NackClass {
    /// Numeric gRPC status code for the OTLP status payload.
    pub(crate) const fn grpc_code(self) -> i32 {
        match self {
            NackClass::ClientError => 3,  // INVALID_ARGUMENT
            NackClass::ServerError => 13, // INTERNAL
            NackClass::Retryable => 14,   // UNAVAILABLE
        }
    }

    /// HTTP status code for the OTLP/HTTP response.
    pub(crate) const fn http_status(self) -> StatusCode {
        match self {
            NackClass::ClientError => StatusCode::BAD_REQUEST,
            NackClass::ServerError => StatusCode::INTERNAL_SERVER_ERROR,
            NackClass::Retryable => StatusCode::SERVICE_UNAVAILABLE,
        }
    }

    /// Builds the tonic [`Status`] for the OTLP/gRPC response.
    pub(crate) fn to_tonic_status(self, message: String) -> Status {
        match self {
            NackClass::ClientError => Status::invalid_argument(message),
            NackClass::ServerError => Status::internal(message),
            NackClass::Retryable => Status::unavailable(message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonic::Code;

    /// Scenario: a permanent NACK explicitly classified as a client rejection.
    /// Guarantees: it maps to a non-retryable client error across both transports.
    #[test]
    fn rejected_permanent_nack_is_a_client_error() {
        let class = classify_nack(true, NackCause::Rejected);
        assert_eq!(class, NackClass::ClientError);
        assert_eq!(class.grpc_code(), 3);
        assert_eq!(class.http_status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            class.to_tonic_status(String::new()).code(),
            Code::InvalidArgument
        );
    }

    /// Scenario: a permanent NACK with any non-rejection cause.
    /// Guarantees: it maps to a non-retryable server error across both transports.
    #[test]
    fn other_permanent_nack_is_a_server_error() {
        for cause in [
            NackCause::Unspecified,
            NackCause::RouteFull,
            NackCause::RouteClosed,
            NackCause::NodeShutdown,
        ] {
            let class = classify_nack(true, cause);
            assert_eq!(class, NackClass::ServerError);
            assert_eq!(class.grpc_code(), 13);
            assert_eq!(class.http_status(), StatusCode::INTERNAL_SERVER_ERROR);
            assert_eq!(class.to_tonic_status(String::new()).code(), Code::Internal);
        }
    }

    /// Scenario: a transient NACK regardless of cause.
    /// Guarantees: it maps to a retryable status across both transports.
    #[test]
    fn transient_nack_is_retryable() {
        let class = classify_nack(false, NackCause::Rejected);
        assert_eq!(class, NackClass::Retryable);
        assert_eq!(class.grpc_code(), 14);
        assert_eq!(class.http_status(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(
            class.to_tonic_status(String::new()).code(),
            Code::Unavailable
        );
    }
}
