// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use futures::future::LocalBoxFuture;
use reqwest::header::{HeaderName, HeaderValue};
use reqwest::{Client, StatusCode};
use std::time::Duration;
use thiserror::Error;

const CONTENT_TYPE: &str = "application/x-mepacket";
const ORIGINAL_CONTENT_SIZE_HEADER: &str = "OriginalContentSize";

#[derive(Debug)]
pub(crate) struct MetricsPublisher {
    client: Client,
    endpoint: String,
    timeout: Duration,
}

#[derive(Debug, Error)]
pub(crate) enum PublisherBuildError {
    #[error("invalid Geneva metrics endpoint: {0}")]
    InvalidEndpoint(String),
    #[error("failed to create the Geneva metrics HTTP client: {0}")]
    Client(#[source] Box<reqwest::Error>),
}

#[derive(Debug, Error)]
#[allow(variant_size_differences)]
pub(crate) enum PublishError {
    #[error("Geneva metrics request failed: {0}")]
    Request(#[source] Box<reqwest::Error>),
    #[error("Geneva metrics endpoint returned HTTP {status}")]
    Response { status: StatusCode },
}

impl PublishError {
    pub(crate) fn is_unauthorized(&self) -> bool {
        matches!(
            self,
            Self::Response {
                status: StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN
            }
        )
    }

    pub(crate) fn is_retryable(&self) -> bool {
        match self {
            Self::Request(_) => true,
            Self::Response { status } => {
                *status == StatusCode::UNAUTHORIZED
                    || *status == StatusCode::FORBIDDEN
                    || *status == StatusCode::TOO_MANY_REQUESTS
                    || status.is_server_error()
            }
        }
    }
}

impl MetricsPublisher {
    pub(crate) fn new(endpoint: &str, timeout: Duration) -> Result<Self, PublisherBuildError> {
        otel_arrow_dfe_otap::crypto::ensure_crypto_provider();
        let endpoint = endpoint.trim();
        let _endpoint = reqwest::Url::parse(endpoint)
            .map_err(|error| PublisherBuildError::InvalidEndpoint(error.to_string()))?;
        let client = Client::builder()
            .timeout(timeout)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|error| PublisherBuildError::Client(Box::new(error)))?;
        Ok(Self {
            client,
            endpoint: endpoint.to_owned(),
            timeout,
        })
    }

    pub(crate) fn timeout(&self) -> Duration {
        self.timeout
    }

    pub(crate) fn publish(
        &self,
        monitoring_account: &str,
        packet: Vec<u8>,
        auth_header_name: HeaderName,
        auth_header_value: HeaderValue,
    ) -> LocalBoxFuture<'static, Result<(), PublishError>> {
        let original_size = packet.len();
        let endpoint = self.endpoint.replace(
            "{monitoring_account}",
            &urlencoding::encode(monitoring_account),
        );
        let request = self
            .client
            .post(endpoint)
            .header(reqwest::header::CONTENT_TYPE, CONTENT_TYPE)
            .header(ORIGINAL_CONTENT_SIZE_HEADER, original_size)
            .header(auth_header_name, auth_header_value)
            .body(packet);
        Box::pin(async move {
            let response = request
                .send()
                .await
                .map_err(|error| PublishError::Request(Box::new(error)))?;

            if response.status() == StatusCode::OK {
                Ok(())
            } else {
                Err(PublishError::Response {
                    status: response.status(),
                })
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{body_bytes, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn authorization_header() -> HeaderValue {
        HeaderValue::from_static("test-authorization")
    }

    /// Scenario: A protocol v6 packet is published to a healthy endpoint.
    /// Guarantees: The C++-compatible content headers and packet bytes are sent unchanged.
    #[tokio::test]
    async fn publishes_packet_with_required_headers() {
        let server = MockServer::start().await;
        let packet = vec![6, 0, 1, 2, 3];
        Mock::given(method("POST"))
            .and(path("/metrics"))
            .and(header("content-type", CONTENT_TYPE))
            .and(header(ORIGINAL_CONTENT_SIZE_HEADER, "5"))
            .and(header("authorization", "test-authorization"))
            .and(body_bytes(packet.clone()))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;

        let publisher =
            MetricsPublisher::new(&format!("{}/metrics", server.uri()), Duration::from_secs(1))
                .expect("publisher should be created");

        publisher
            .publish(
                "example-account",
                packet,
                reqwest::header::AUTHORIZATION,
                authorization_header(),
            )
            .await
            .expect("publication should succeed");
    }

    /// Scenario: The publication endpoint redirects to a page that would return HTTP 200.
    /// Guarantees: Redirects fail publication with their original status and are never followed.
    #[tokio::test]
    async fn rejects_redirects_without_following_them() {
        for status in [301, 302, 303, 307, 308] {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/metrics"))
                .respond_with(
                    ResponseTemplate::new(status)
                        .insert_header("location", format!("{}/login", server.uri())),
                )
                .expect(1)
                .mount(&server)
                .await;
            Mock::given(path("/login"))
                .respond_with(ResponseTemplate::new(200))
                .expect(0)
                .mount(&server)
                .await;
            let publisher = MetricsPublisher::new(
                &format!("{}/metrics", server.uri()),
                Duration::from_secs(30),
            )
            .expect("valid endpoint");

            let error = publisher
                .publish(
                    "example-account",
                    vec![6, 0],
                    reqwest::header::AUTHORIZATION,
                    authorization_header(),
                )
                .await
                .expect_err("redirect must not count as successful publication");

            assert!(
                matches!(error, PublishError::Response { status: actual } if actual.as_u16() == status),
                "expected original HTTP {status}, got {error}"
            );
            assert!(!error.is_retryable());
            assert!(!error.is_unauthorized());
        }
    }

    /// Scenario: The endpoint rejects a packet with a non-retryable client error.
    /// Guarantees: HTTP 400 responses are classified as permanent.
    #[tokio::test]
    async fn classifies_client_error_as_permanent() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(400))
            .mount(&server)
            .await;
        let publisher =
            MetricsPublisher::new(&server.uri(), Duration::from_secs(1)).expect("valid endpoint");

        let error = publisher
            .publish(
                "example-account",
                vec![6, 0],
                reqwest::header::AUTHORIZATION,
                authorization_header(),
            )
            .await
            .expect_err("publication should fail");

        assert!(!error.is_retryable(), "HTTP 400 should be permanent");
        assert!(!error.is_unauthorized());
    }

    /// Scenario: A bearer credential is rejected, the endpoint is throttled, or the service is unavailable.
    /// Guarantees: HTTP 401, 403, 429, and 5xx responses are classified as retryable.
    #[tokio::test]
    async fn classifies_transient_responses_as_retryable() {
        for status in [401, 403, 429, 500, 503] {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .respond_with(ResponseTemplate::new(status))
                .mount(&server)
                .await;
            let publisher = MetricsPublisher::new(&server.uri(), Duration::from_secs(1))
                .expect("valid endpoint");

            let error = publisher
                .publish(
                    "example-account",
                    vec![6, 0],
                    reqwest::header::AUTHORIZATION,
                    authorization_header(),
                )
                .await
                .expect_err("publication should fail");

            assert!(error.is_retryable(), "HTTP {status} should be retryable");
            assert_eq!(error.is_unauthorized(), matches!(status, 401 | 403));
        }
    }

    /// Scenario: An authenticated account publication uses an endpoint template.
    /// Guarantees: The account is path-encoded and the supplied authorization header is sent.
    #[tokio::test]
    async fn publishes_with_authorization_header_and_account_routing() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/metrics/account%20name"))
            .and(header("authorization", "test-authorization"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;
        let publisher = MetricsPublisher::new(
            &format!("{}/metrics/{{monitoring_account}}", server.uri()),
            Duration::from_secs(1),
        )
        .expect("valid endpoint");

        publisher
            .publish(
                "account name",
                vec![6, 0],
                reqwest::header::AUTHORIZATION,
                authorization_header(),
            )
            .await
            .expect("authenticated publication should succeed");
    }
}
