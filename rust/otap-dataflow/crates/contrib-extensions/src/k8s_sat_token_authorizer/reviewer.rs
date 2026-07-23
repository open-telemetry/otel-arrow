// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Kubernetes client construction and `TokenReview` execution.

use k8s_openapi::api::authentication::v1::{TokenReview, TokenReviewSpec};
use kube::api::{Api, PostParams};

use super::error::Error;

/// The outcome of a single `TokenReview` call, before admission against the
/// configured allow-list.
#[derive(Debug, Clone)]
pub(crate) enum ReviewOutcome {
    /// The API server authenticated the token. Carries the service-account
    /// username (`system:serviceaccount:<ns>:<name>`) and the audiences the
    /// token was confirmed valid for, when present.
    Authenticated {
        /// The authenticated principal's username, if the API server returned
        /// one.
        username: Option<String>,
        /// The audiences the token was confirmed valid for.
        audiences: Vec<String>,
    },
    /// The API server did not authenticate the token (missing, malformed,
    /// expired, wrong audience, or untrusted). Carries the API server's error
    /// message for logs only, when present.
    Unauthenticated {
        /// Human-readable reason from the API server, for logs only.
        error: Option<String>,
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
        let client = kube::Client::try_default()
            .await
            .map_err(|source| Error::ClientInit { source })?;
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
            .map_err(|source| Error::TokenReview { source })?;

        let status = response.status.ok_or(Error::MissingStatus)?;

        if status.authenticated.unwrap_or(false) {
            Ok(ReviewOutcome::Authenticated {
                username: status.user.and_then(|user| user.username),
                audiences: status.audiences.unwrap_or_default(),
            })
        } else {
            Ok(ReviewOutcome::Unauthenticated {
                error: status.error,
            })
        }
    }
}
