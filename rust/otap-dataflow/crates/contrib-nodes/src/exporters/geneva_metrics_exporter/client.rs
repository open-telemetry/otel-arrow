// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use reqwest::{Client, StatusCode};
use std::time::Duration;
use thiserror::Error;

const CONTENT_TYPE: &str = "application/x-mepacket";
const ORIGINAL_CONTENT_SIZE_HEADER: &str = "OriginalContentSize";

#[derive(Clone, Debug)]
pub(crate) struct MetricsPublisher {
    client: Client,
    endpoint: String,
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
    #[error("Geneva metrics bearer token cannot be represented as an HTTP header")]
    InvalidAuthorizationHeader(#[source] reqwest::header::InvalidHeaderValue),
    #[error("Geneva metrics request failed: {0}")]
    Request(#[source] Box<reqwest::Error>),
    #[error("Geneva metrics endpoint returned HTTP {status}")]
    Response { status: StatusCode },
}

impl PublishError {
    pub(crate) fn is_retryable(&self) -> bool {
        match self {
            Self::InvalidAuthorizationHeader(_) => false,
            Self::Request(_) => true,
            Self::Response { status } => {
                *status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error()
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
            .build()
            .map_err(|error| PublisherBuildError::Client(Box::new(error)))?;
        Ok(Self {
            client,
            endpoint: endpoint.to_owned(),
        })
    }

    pub(crate) async fn publish(
        &self,
        monitoring_account: &str,
        packet: Vec<u8>,
        bearer_token: &str,
    ) -> Result<(), PublishError> {
        let original_size = packet.len();
        let endpoint = self.endpoint.replace(
            "{monitoring_account}",
            &urlencoding::encode(monitoring_account),
        );
        let mut request = self
            .client
            .post(endpoint)
            .header(reqwest::header::CONTENT_TYPE, CONTENT_TYPE)
            .header(ORIGINAL_CONTENT_SIZE_HEADER, original_size)
            .body(packet);
        let mut authorization =
            reqwest::header::HeaderValue::from_str(&format!("Bearer {bearer_token}"))
                .map_err(PublishError::InvalidAuthorizationHeader)?;
        authorization.set_sensitive(true);
        request = request.header(reqwest::header::AUTHORIZATION, authorization);
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{body_bytes, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

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
            .and(header("authorization", "Bearer test-token"))
            .and(body_bytes(packet.clone()))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;

        let publisher =
            MetricsPublisher::new(&format!("{}/metrics", server.uri()), Duration::from_secs(1))
                .expect("publisher should be created");

        publisher
            .publish("example-account", packet, "test-token")
            .await
            .expect("publication should succeed");
    }

    /// Scenario: The endpoint rejects a packet with a client error.
    /// Guarantees: HTTP 4xx responses other than throttling are classified as permanent.
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
            .publish("example-account", vec![6, 0], "test-token")
            .await
            .expect_err("publication should fail");

        assert!(!error.is_retryable());
    }

    /// Scenario: The endpoint is throttled or unavailable.
    /// Guarantees: HTTP 429 and 5xx responses are classified as retryable.
    #[tokio::test]
    async fn classifies_transient_responses_as_retryable() {
        for status in [429, 500, 503] {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .respond_with(ResponseTemplate::new(status))
                .mount(&server)
                .await;
            let publisher = MetricsPublisher::new(&server.uri(), Duration::from_secs(1))
                .expect("valid endpoint");

            let error = publisher
                .publish("example-account", vec![6, 0], "test-token")
                .await
                .expect_err("publication should fail");

            assert!(error.is_retryable(), "HTTP {status} should be retryable");
        }
    }

    /// Scenario: An authenticated account publication uses an endpoint template.
    /// Guarantees: The account is path-encoded and the bearer credential is sent in a sensitive Authorization header.
    #[tokio::test]
    async fn publishes_with_bearer_token_and_account_routing() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/metrics/account%20name"))
            .and(header("authorization", "Bearer test-token"))
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
            .publish("account name", vec![6, 0], "test-token")
            .await
            .expect("authenticated publication should succeed");
    }
}
