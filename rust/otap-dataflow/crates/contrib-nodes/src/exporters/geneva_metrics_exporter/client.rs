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
    #[cfg(test)]
    scripted: Option<ScriptedRequests>,
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
    #[error("input publication deadline exhausted")]
    DeadlineExceeded,
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
            Self::DeadlineExceeded | Self::Request(_) => true,
            Self::Response { status } => {
                self.is_unauthorized()
                    || matches!(
                        *status,
                        StatusCode::REQUEST_TIMEOUT | StatusCode::TOO_MANY_REQUESTS
                    )
                    || status.is_server_error()
            }
        }
    }

    pub(crate) fn is_refusal(&self) -> bool {
        matches!(self, Self::Response { status } if status.is_client_error() && *status != StatusCode::REQUEST_TIMEOUT)
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
            #[cfg(test)]
            scripted: None,
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
        #[cfg(test)]
        if let Some(scripted) = &self.scripted {
            return (scripted.0)();
        }
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

/// Test-only request source that replaces HTTP, letting export-loop tests
/// control when publications resolve without real I/O or wall-clock timeouts.
#[cfg(test)]
struct ScriptedRequests(Box<dyn Fn() -> LocalBoxFuture<'static, Result<(), PublishError>>>);

#[cfg(test)]
impl std::fmt::Debug for ScriptedRequests {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ScriptedRequests")
    }
}

#[cfg(test)]
impl MetricsPublisher {
    /// Creates a publisher whose requests are produced by `request` instead of HTTP.
    pub(crate) fn scripted(
        timeout: Duration,
        request: impl Fn() -> LocalBoxFuture<'static, Result<(), PublishError>> + 'static,
    ) -> Self {
        let mut publisher =
            Self::new("http://127.0.0.1:1", timeout).expect("static test endpoint should be valid");
        publisher.scripted = Some(ScriptedRequests(Box::new(request)));
        publisher
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{body_bytes, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn publish(endpoint: &str, account: &str, packet: Vec<u8>) -> Result<(), PublishError> {
        MetricsPublisher::new(endpoint, Duration::from_secs(10))
            .expect("valid endpoint")
            .publish(
                account,
                packet,
                reqwest::header::AUTHORIZATION,
                HeaderValue::from_static("test-authorization"),
            )
            .await
    }

    /// Scenario: A protocol v6 packet is published through an account endpoint template.
    /// Guarantees: The account is path-encoded and the C++-compatible content headers, authorization header, and packet bytes are sent unchanged.
    #[tokio::test]
    async fn publishes_packet_with_required_headers_and_account_routing() {
        let server = MockServer::start().await;
        let packet = vec![6, 0, 1, 2, 3];
        Mock::given(method("POST"))
            .and(path("/metrics/account%20name"))
            .and(header("content-type", CONTENT_TYPE))
            .and(header(ORIGINAL_CONTENT_SIZE_HEADER, "5"))
            .and(header("authorization", "test-authorization"))
            .and(body_bytes(packet.clone()))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;

        let endpoint = format!("{}/metrics/{{monitoring_account}}", server.uri());
        publish(&endpoint, "account name", packet)
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

            let error = publish(
                &format!("{}/metrics", server.uri()),
                "example-account",
                vec![6, 0],
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

    /// Scenario: A publication fails with HTTP 400, 401, 403, 408, 429, or 5xx.
    /// Guarantees: HTTP 400 is permanent; 401, 403, 408, 429, and 5xx remain retryable independently of refusal classification.
    #[test]
    fn classifies_error_responses() {
        for status in [400, 401, 403, 408, 429, 500, 503] {
            let error = PublishError::Response {
                status: StatusCode::from_u16(status).expect("valid HTTP status"),
            };

            assert_eq!(error.is_retryable(), status != 400, "HTTP {status}");
            assert_eq!(error.is_unauthorized(), matches!(status, 401 | 403));
            assert_eq!(error.is_refusal(), matches!(status, 400 | 401 | 403 | 429));
        }
    }
}
