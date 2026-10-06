// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared bearer authorization evaluation for receiver transports.

// TODO(auth): Create an HTTP-header authorization helper shared by all
// header-based authentication methods.

use http::HeaderMap;
use otel_arrow_dfe_engine::capability::auth::{
    AuthorizedIdentity, AuthzDecision, BearerToken, DenyReason,
};
use otel_arrow_dfe_engine::shared::capability::auth::bearer_token_authorizer::BearerTokenAuthorizer;
use otel_arrow_dfe_telemetry::common_attributes::ReceiverRejectionErrorType;
use std::time::Duration;

/// A protocol-neutral reason an OTLP request was not authorized.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AuthorizationRejection {
    /// The request did not contain one valid bearer credential.
    Unauthenticated,
    /// The credential was valid but not permitted.
    PermissionDenied,
    /// The authorizer could not reach a decision.
    Unavailable,
}

impl AuthorizationRejection {
    /// Returns the bounded receiver rejection category for this outcome.
    pub(crate) const fn error_type(self) -> ReceiverRejectionErrorType {
        match self {
            Self::Unauthenticated => ReceiverRejectionErrorType::Authentication,
            Self::PermissionDenied => ReceiverRejectionErrorType::PermissionDenied,
            Self::Unavailable => ReceiverRejectionErrorType::AuthorizationUnavailable,
        }
    }

    /// Returns a protocol-neutral message safe to return to the caller.
    pub(crate) const fn message(self) -> &'static str {
        match self {
            Self::Unauthenticated => "unauthenticated",
            Self::PermissionDenied => "permission denied",
            Self::Unavailable => "authorization unavailable",
        }
    }
}

/// Evaluates exactly one bearer credential and preserves the authorized identity.
pub(crate) async fn authorize_bearer(
    authorizer: &dyn BearerTokenAuthorizer,
    headers: &HeaderMap,
    timeout: Option<Duration>,
) -> Result<AuthorizedIdentity, AuthorizationRejection> {
    let mut values = headers.get_all(http::header::AUTHORIZATION).iter();
    let Some(header) = values.next() else {
        return Err(AuthorizationRejection::Unauthenticated);
    };
    if values.next().is_some() {
        return Err(AuthorizationRejection::Unauthenticated);
    }

    let header = header
        .to_str()
        .map_err(|_| AuthorizationRejection::Unauthenticated)?;
    let credential =
        BearerToken::from_header_value(header).ok_or(AuthorizationRejection::Unauthenticated)?;

    let decision = if let Some(timeout) = timeout {
        match tokio::time::timeout(timeout, authorizer.authorize(&credential)).await {
            Ok(decision) => decision,
            Err(_) => {
                otel_arrow_dfe_telemetry::otel_warn!(
                    "receiver.authz.timeout",
                    timeout = ?timeout
                );
                return Err(AuthorizationRejection::Unavailable);
            }
        }
    } else {
        authorizer.authorize(&credential).await
    };

    match decision {
        Ok(AuthzDecision::Allow { identity }) => Ok(identity),
        Ok(AuthzDecision::Deny {
            reason: DenyReason::MissingCredential | DenyReason::InvalidCredential,
            ..
        }) => Err(AuthorizationRejection::Unauthenticated),
        Ok(AuthzDecision::Deny {
            reason: DenyReason::NotPermitted,
            ..
        }) => Err(AuthorizationRejection::PermissionDenied),
        // `DenyReason` is non-exhaustive. A future variant is still a
        // definitive denial and must not produce a retryable response.
        Ok(AuthzDecision::Deny { reason, .. }) => {
            otel_arrow_dfe_telemetry::otel_warn!(
                "receiver.authz.unknown_deny_reason",
                reason = ?reason
            );
            Err(AuthorizationRejection::PermissionDenied)
        }
        Err(error) => {
            otel_arrow_dfe_telemetry::otel_warn!(
                "receiver.authz.undetermined",
                error = error.to_string()
            );
            Err(AuthorizationRejection::Unavailable)
        }
    }
}
