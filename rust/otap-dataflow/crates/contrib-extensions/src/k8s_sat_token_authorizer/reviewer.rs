// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Kubernetes client construction, `TokenReview` execution, and
//! `SubjectAccessReview` (RBAC) checks.

use std::collections::BTreeMap;

use k8s_openapi::api::authentication::v1::{TokenReview, TokenReviewSpec};
use k8s_openapi::api::authorization::v1::{
    ResourceAttributes, SubjectAccessReview, SubjectAccessReviewSpec,
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
}

impl Reviewer {
    /// Constructs a reviewer from an in-cluster (or kubeconfig) client
    /// configuration.
    ///
    /// Uses [`kube::Client::try_default`], which infers in-cluster config from
    /// the projected service-account token and cluster CA when running inside a
    /// pod, and otherwise falls back to the local kubeconfig.
    pub(crate) async fn try_new(audiences: Vec<String>) -> Result<Self, Error> {
        // kube talks to the API server over rustls, which needs a process-wide
        // crypto provider installed. Mirror the other auth extensions and ensure
        // one is present before any TLS handshake.
        otap_df_otap::crypto::ensure_crypto_provider();
        let client = kube::Client::try_default().await.map_err(|source| {
            otel_warn!(
                "k8s_sat_token_authorizer.client_init_failed",
                error = %source,
                message = "kubernetes client init failed; failing closed"
            );
            Error::ClientInit { source }
        })?;
        Ok(Self { client, audiences })
    }

    /// Submits a `TokenReview` for `token` and maps the response to a
    /// [`ReviewOutcome`].
    ///
    /// Returns [`Error`] only when no decision could be reached (the request
    /// failed or the response carried no status); a non-authenticated token is a
    /// normal [`ReviewOutcome::Unauthenticated`], not an error.
    pub(crate) async fn review(&self, token: &str) -> Result<ReviewOutcome, Error> {
        let api: Api<TokenReview> = Api::all(self.client.clone());
        let review = TokenReview {
            spec: TokenReviewSpec {
                token: token.to_owned(),
                audiences: Some(self.audiences.clone()),
            },
            ..Default::default()
        };

        let response = api
            .create(&PostParams::default(), &review)
            .await
            .map_err(|source| {
                otel_warn!(
                    "k8s_sat_token_authorizer.token_review_failed",
                    error = %source,
                    message = "TokenReview request failed; failing closed"
                );
                Error::TokenReview { source }
            })?;

        let status = response.status.ok_or_else(|| {
            otel_warn!(
                "k8s_sat_token_authorizer.token_review_no_status",
                message = "TokenReview response carried no status; failing closed"
            );
            Error::MissingStatus
        })?;

        if status.authenticated.unwrap_or(false) {
            let user = status.user.unwrap_or_default();
            Ok(ReviewOutcome::Authenticated(AuthenticatedUser {
                username: user.username,
                uid: user.uid,
                groups: user.groups.unwrap_or_default(),
                extra: user.extra.unwrap_or_default(),
                audiences: status.audiences.unwrap_or_default(),
            }))
        } else {
            Ok(ReviewOutcome::Unauthenticated {
                error: status.error,
            })
        }
    }

    /// Submits a `SubjectAccessReview` asking whether `user` may perform the
    /// action described by `attrs`, and maps the response to an
    /// [`AccessOutcome`].
    ///
    /// Returns [`Error`] only when no decision could be reached (the request
    /// failed or the response carried no status); an RBAC "not allowed" is a
    /// normal [`AccessOutcome::Denied`], not an error.
    pub(crate) async fn check_access(
        &self,
        user: &AuthenticatedUser,
        attrs: &ResourceAttributesConfig,
    ) -> Result<AccessOutcome, Error> {
        let api: Api<SubjectAccessReview> = Api::all(self.client.clone());
        let review = SubjectAccessReview {
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
        };

        let response = api
            .create(&PostParams::default(), &review)
            .await
            .map_err(|source| {
                otel_warn!(
                    "k8s_sat_token_authorizer.access_review_failed",
                    error = %source,
                    message = "SubjectAccessReview request failed; failing closed"
                );
                Error::SubjectAccessReview { source }
            })?;

        let status = response.status.ok_or_else(|| {
            otel_warn!(
                "k8s_sat_token_authorizer.access_review_no_status",
                message = "SubjectAccessReview response carried no status; failing closed"
            );
            Error::MissingStatus
        })?;

        // `allowed` grants; an explicit `denied` overrides. Anything else (not
        // allowed, or an evaluation error) is a deny -- callers fail closed.
        if status.allowed && !status.denied.unwrap_or(false) {
            Ok(AccessOutcome::Allowed)
        } else {
            let reason = status.reason.or(status.evaluation_error);
            Ok(AccessOutcome::Denied { reason })
        }
    }
}
