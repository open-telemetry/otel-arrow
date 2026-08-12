// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Kubernetes client construction, `TokenReview` execution, and
//! `SubjectAccessReview` (RBAC) checks.

use std::collections::BTreeMap;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use k8s_openapi::api::authentication::v1::{TokenReview, TokenReviewSpec, TokenReviewStatus};
use k8s_openapi::api::authorization::v1::{
    ResourceAttributes, SubjectAccessReview, SubjectAccessReviewSpec, SubjectAccessReviewStatus,
};
use kube::api::{Api, PostParams};

use super::config::ResourceAttributesConfig;
use otap_df_telemetry::otel_warn;

use super::error::Error;

/// The authenticated identity the API server returned for a token.
///
/// Carries the full subject so an RBAC `SubjectAccessReview` can be evaluated
/// against the exact user, groups, and extra attributes Kubernetes recognizes.
#[derive(Debug, Clone, Default)]
pub(crate) struct AuthenticatedUser {
    /// Service-account username (`system:serviceaccount:<ns>:<name>`), if any.
    pub(crate) username: Option<String>,
    /// Opaque unique identifier of the subject, if any.
    pub(crate) uid: Option<String>,
    /// Groups the subject belongs to.
    pub(crate) groups: Vec<String>,
    /// Additional identity attributes the authenticator attached.
    pub(crate) extra: BTreeMap<String, Vec<String>>,
    /// The audiences the token was confirmed valid for.
    pub(crate) audiences: Vec<String>,
}

/// The outcome of a single `TokenReview` call, before admission.
#[derive(Debug, Clone)]
pub(crate) enum ReviewOutcome {
    /// The API server authenticated the token; carries the full subject.
    Authenticated(AuthenticatedUser),
    /// The API server did not authenticate the token (missing, malformed,
    /// expired, wrong audience, or untrusted). Carries the API server's error
    /// message for logs only, when present.
    Unauthenticated {
        /// Human-readable reason from the API server, for logs only.
        error: Option<String>,
    },
}

/// The outcome of a `SubjectAccessReview` (RBAC) check.
#[derive(Debug, Clone)]
pub(crate) enum AccessOutcome {
    /// RBAC permits the action.
    Allowed,
    /// RBAC does not permit the action. Carries the API server's reason for logs
    /// only, when present.
    Denied {
        /// Human-readable reason from the API server, for logs only.
        reason: Option<String>,
    },
}

/// Wraps a Kubernetes client plus the audiences requested on every review.
#[derive(Clone)]
pub(crate) struct Reviewer {
    client: kube::Client,
    audiences: Vec<String>,
    request_timeout: Duration,
}

impl Reviewer {
    /// Constructs a reviewer from an in-cluster (or kubeconfig) client
    /// configuration.
    ///
    /// Uses [`kube::Client::try_default`], which infers in-cluster config from
    /// the projected service-account token and cluster CA when running inside a
    /// pod, and otherwise falls back to the local kubeconfig.
    pub(crate) async fn try_new(
        audiences: Vec<String>,
        request_timeout: Duration,
    ) -> Result<Self, Error> {
        // kube talks to the API server over rustls, which needs a process-wide
        // crypto provider installed. Mirror the other auth extensions and ensure
        // one is present before any TLS handshake.
        otap_df_otap::crypto::ensure_crypto_provider();
        let client = kube::Client::try_default().await.map_err(|source| {
            otel_warn!(
                "k8s_service_account_token_auth.client_init_failed",
                error = %source,
                message = "kubernetes client init failed; failing closed"
            );
            Error::ClientInit {
                source: Arc::new(source),
            }
        })?;
        Ok(Self {
            client,
            audiences,
            request_timeout,
        })
    }

    /// Submits a `TokenReview` for `token` and maps the response to a
    /// [`ReviewOutcome`].
    ///
    /// Returns [`Error`] only when no decision could be reached (the request
    /// failed, timed out, or the response carried no status); a
    /// non-authenticated token is a normal [`ReviewOutcome::Unauthenticated`],
    /// not an error.
    pub(crate) async fn review(&self, token: &str) -> Result<ReviewOutcome, Error> {
        let api: Api<TokenReview> = Api::all(self.client.clone());
        let review = token_review_request(token, &self.audiences);

        let response = match tokio::time::timeout(
            self.request_timeout,
            api.create(&PostParams::default(), &review),
        )
        .await
        {
            Ok(Ok(response)) => response,
            Ok(Err(source)) => {
                otel_warn!(
                    "k8s_service_account_token_auth.token_review_failed",
                    error = %source,
                    message = "TokenReview request failed; failing closed"
                );
                return Err(Error::TokenReview {
                    source: Arc::new(source),
                });
            }
            Err(_) => {
                otel_warn!(
                    "k8s_service_account_token_auth.token_review_timeout",
                    timeout = ?self.request_timeout,
                    message = "TokenReview request timed out; failing closed"
                );
                return Err(Error::TokenReviewTimeout {
                    timeout: self.request_timeout,
                });
            }
        };

        let status = response.status.ok_or_else(|| {
            otel_warn!(
                "k8s_service_account_token_auth.token_review_no_status",
                message = "TokenReview response carried no status; failing closed"
            );
            Error::MissingStatus
        })?;

        Ok(review_outcome(status))
    }

    /// Submits a `SubjectAccessReview` asking whether `user` may perform the
    /// action described by `attrs`, and maps the response to an
    /// [`AccessOutcome`].
    ///
    /// Returns [`Error`] only when no decision could be reached (the request
    /// failed, timed out, or the response carried no status); an RBAC "not
    /// allowed" is a normal [`AccessOutcome::Denied`], not an error.
    pub(crate) async fn check_access(
        &self,
        user: &AuthenticatedUser,
        attrs: &ResourceAttributesConfig,
    ) -> Result<AccessOutcome, Error> {
        let api: Api<SubjectAccessReview> = Api::all(self.client.clone());
        let review = access_review_request(user, attrs);

        let response = match tokio::time::timeout(
            self.request_timeout,
            api.create(&PostParams::default(), &review),
        )
        .await
        {
            Ok(Ok(response)) => response,
            Ok(Err(source)) => {
                otel_warn!(
                    "k8s_service_account_token_auth.access_review_failed",
                    error = %source,
                    message = "SubjectAccessReview request failed; failing closed"
                );
                return Err(Error::SubjectAccessReview {
                    source: Arc::new(source),
                });
            }
            Err(_) => {
                otel_warn!(
                    "k8s_service_account_token_auth.access_review_timeout",
                    timeout = ?self.request_timeout,
                    message = "SubjectAccessReview request timed out; failing closed"
                );
                return Err(Error::SubjectAccessReviewTimeout {
                    timeout: self.request_timeout,
                });
            }
        };

        let status = response.status.ok_or_else(|| {
            otel_warn!(
                "k8s_service_account_token_auth.access_review_no_status",
                message = "SubjectAccessReview response carried no status; failing closed"
            );
            Error::MissingStatus
        })?;

        Ok(access_outcome(status))
    }
}

/// The Kubernetes review calls the decision flow depends on.
///
/// Implemented by [`Reviewer`] in production and by fakes in unit tests, so
/// [`Core::decide`] can run against canned API-server responses.
pub(crate) trait KubeReviews {
    /// Authenticates `token` via `TokenReview`.
    fn review(&self, token: &str) -> impl Future<Output = Result<ReviewOutcome, Error>>;

    /// Asks whether `user` may perform the action described by `attrs`.
    fn check_access(
        &self,
        user: &AuthenticatedUser,
        attrs: &ResourceAttributesConfig,
    ) -> impl Future<Output = Result<AccessOutcome, Error>>;
}

impl KubeReviews for Reviewer {
    async fn review(&self, token: &str) -> Result<ReviewOutcome, Error> {
        Reviewer::review(self, token).await
    }

    async fn check_access(
        &self,
        user: &AuthenticatedUser,
        attrs: &ResourceAttributesConfig,
    ) -> Result<AccessOutcome, Error> {
        Reviewer::check_access(self, user, attrs).await
    }
}

/// Builds the `TokenReview` submitted for `token`.
///
/// Always requests the configured `audiences`, so the API server only ever
/// confirms an audience this authorizer governs.
pub(crate) fn token_review_request(token: &str, audiences: &[String]) -> TokenReview {
    TokenReview {
        spec: TokenReviewSpec {
            token: token.to_owned(),
            audiences: Some(audiences.to_vec()),
        },
        ..Default::default()
    }
}

/// Maps a `TokenReviewStatus` to a [`ReviewOutcome`].
///
/// Anything short of an explicit `authenticated: true` is
/// [`ReviewOutcome::Unauthenticated`], so a status that omits the flag fails
/// closed.
pub(crate) fn review_outcome(status: TokenReviewStatus) -> ReviewOutcome {
    if status.authenticated.unwrap_or(false) {
        let user = status.user.unwrap_or_default();
        ReviewOutcome::Authenticated(AuthenticatedUser {
            username: user.username,
            uid: user.uid,
            groups: user.groups.unwrap_or_default(),
            extra: user.extra.unwrap_or_default(),
            audiences: status.audiences.unwrap_or_default(),
        })
    } else {
        ReviewOutcome::Unauthenticated {
            error: status.error,
        }
    }
}

/// Builds the `SubjectAccessReview` asking whether `user` may perform the
/// action described by `attrs`.
///
/// Forwards the full subject (username, uid, groups, extra) so the API server
/// evaluates RBAC against the exact identity `TokenReview` authenticated.
pub(crate) fn access_review_request(
    user: &AuthenticatedUser,
    attrs: &ResourceAttributesConfig,
) -> SubjectAccessReview {
    SubjectAccessReview {
        spec: SubjectAccessReviewSpec {
            user: user.username.clone(),
            uid: user.uid.clone(),
            groups: Some(user.groups.clone()),
            extra: Some(user.extra.clone()),
            resource_attributes: Some(ResourceAttributes {
                group: attrs.group.clone(),
                version: attrs.version.clone(),
                resource: Some(attrs.resource.clone()),
                verb: Some(attrs.verb.clone()),
                namespace: attrs.namespace.clone(),
                name: attrs.name.clone(),
                subresource: attrs.subresource.clone(),
                ..Default::default()
            }),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Maps a `SubjectAccessReviewStatus` to an [`AccessOutcome`].
///
/// `allowed` grants; an explicit `denied` overrides it. Anything else (not
/// allowed, or an evaluation error) is a deny, so callers fail closed.
pub(crate) fn access_outcome(status: SubjectAccessReviewStatus) -> AccessOutcome {
    if status.allowed && !status.denied.unwrap_or(false) {
        AccessOutcome::Allowed
    } else {
        let reason = status.reason.or(status.evaluation_error);
        AccessOutcome::Denied { reason }
    }
}
