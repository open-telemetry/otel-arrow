// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::token_exchange::TokenExchange;
use futures::future::Either;
#[cfg(test)]
use futures::future::LocalBoxFuture;
use reqwest::header::{HeaderName, HeaderValue};
use reqwest::{Client, StatusCode, Url};
use std::rc::Rc;
use std::time::{Duration, Instant};
use thiserror::Error;

pub(super) const CONTENT_TYPE: &str = "application/x-mepacket";
pub(super) const ORIGINAL_CONTENT_SIZE_HEADER: &str = "OriginalContentSize";

#[derive(Debug)]
pub(crate) struct MetricsPublisher {
    client: Client,
    endpoint: Url,
    timeout: Duration,
    token_exchange: Option<Rc<TokenExchange>>,
    #[cfg(test)]
    scripted: Option<ScriptedRequests>,
}

#[derive(Debug, Error)]
pub(crate) enum PublisherBuildError {
    #[error("invalid Geneva metrics endpoint: {0}")]
    InvalidEndpoint(String),
    #[error("Geneva metrics endpoint does not contain a routing hostname")]
    MissingRoutingId,
    #[error("failed to create the Geneva metrics HTTP client: {0}")]
    Client(#[source] Box<reqwest::Error>),
}

#[derive(Debug, Error)]
#[allow(variant_size_differences)]
pub(crate) enum PublishError {
    #[error("Geneva metrics publication requires a ready source token")]
    SourceTokenUnavailable,
    #[error("input publication deadline exhausted")]
    DeadlineExceeded,
    #[error("Geneva metrics publication request failed: {0}")]
    Request(#[source] Box<reqwest::Error>),
    #[error("Geneva token exchange request failed: {0}")]
    TokenExchangeRequest(#[source] Box<reqwest::Error>),
    #[error("Geneva metrics endpoint returned HTTP {status}")]
    Response { status: StatusCode },
    #[error("Geneva token exchange returned HTTP {status}")]
    TokenExchangeResponse { status: StatusCode },
    #[error("Geneva metrics publication returned HTTP {status}")]
    ExchangedTokenPublicationResponse { status: StatusCode },
    #[error("Geneva token exchange returned invalid JSON")]
    InvalidExchangeResponse(#[source] Box<serde_json::Error>),
    #[error("Geneva token exchange response exceeds {limit} bytes")]
    ExchangeResponseTooLarge { limit: usize },
    #[error("Geneva token exchange did not return {field}")]
    MissingExchangeField { field: &'static str },
    #[error("Geneva endpoint is invalid: {0}")]
    InvalidPublicationEndpoint(String),
    #[error("Geneva token cannot be used as an authorization header")]
    InvalidPublicationToken,
}

impl PublishError {
    pub(super) fn request(error: reqwest::Error) -> Self {
        Self::Request(Box::new(error.without_url()))
    }

    pub(super) fn token_exchange_request(error: reqwest::Error) -> Self {
        Self::TokenExchangeRequest(Box::new(error.without_url()))
    }

    pub(crate) fn is_unauthorized(&self) -> bool {
        // A 403 can indicate permissions; invalidation cannot force an early token refresh.
        matches!(
            self,
            Self::Response {
                status: StatusCode::UNAUTHORIZED
            } | Self::TokenExchangeResponse {
                status: StatusCode::UNAUTHORIZED
            }
        )
    }

    pub(crate) fn is_retryable(&self) -> bool {
        match self {
            Self::SourceTokenUnavailable
            | Self::DeadlineExceeded
            | Self::Request(_)
            | Self::TokenExchangeRequest(_)
            | Self::InvalidExchangeResponse(_)
            | Self::ExchangeResponseTooLarge { .. }
            | Self::MissingExchangeField { .. }
            | Self::InvalidPublicationEndpoint(_)
            | Self::InvalidPublicationToken => true,
            Self::Response { status }
            | Self::TokenExchangeResponse { status }
            | Self::ExchangedTokenPublicationResponse { status } => {
                matches!(
                    *status,
                    StatusCode::UNAUTHORIZED
                        | StatusCode::FORBIDDEN
                        | StatusCode::REQUEST_TIMEOUT
                        | StatusCode::TOO_MANY_REQUESTS
                ) || status.is_server_error()
            }
        }
    }

    pub(crate) fn is_refusal(&self) -> bool {
        matches!(self, Self::Response { status } | Self::TokenExchangeResponse { status } | Self::ExchangedTokenPublicationResponse { status } if status.is_client_error() && *status != StatusCode::REQUEST_TIMEOUT)
    }
}

impl MetricsPublisher {
    pub(crate) fn new(
        endpoint: &str,
        monitoring_account: &str,
        timeout: Duration,
    ) -> Result<Self, PublisherBuildError> {
        otel_arrow_dfe_otap::crypto::ensure_crypto_provider();
        let endpoint = endpoint.trim().replace(
            "{monitoring_account}",
            &urlencoding::encode(monitoring_account),
        );
        let endpoint = Url::parse(&endpoint)
            .map_err(|error| PublisherBuildError::InvalidEndpoint(error.to_string()))?;
        let client = Client::builder()
            .timeout(timeout)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|error| PublisherBuildError::Client(Box::new(error)))?;
        Ok(Self {
            client,
            endpoint,
            timeout,
            token_exchange: None,
            #[cfg(test)]
            scripted: None,
        })
    }

    pub(crate) fn new_token_exchange(
        exchange_origin: &str,
        monitoring_account: &str,
        timeout: Duration,
    ) -> Result<Self, PublisherBuildError> {
        let mut publisher = Self::new(exchange_origin, monitoring_account, timeout)?;
        publisher.token_exchange = Some(Rc::new(TokenExchange::new(
            publisher.endpoint.clone(),
            monitoring_account,
        )?));
        Ok(publisher)
    }

    pub(crate) fn timeout(&self) -> Duration {
        self.timeout
    }

    pub(crate) fn is_ready(&self, source_token_ready: bool) -> bool {
        source_token_ready
            || self
                .token_exchange
                .as_ref()
                .is_some_and(|exchange| exchange.has_usable_credentials())
    }

    pub(crate) fn refresh_deadline(&self, source_deadline: Option<Instant>) -> Option<Instant> {
        source_deadline
            .into_iter()
            .chain(
                self.token_exchange
                    .as_ref()
                    .and_then(|exchange| exchange.refresh_deadline()),
            )
            .min()
    }

    pub(crate) fn publish(
        &self,
        packet: Vec<u8>,
        source_auth: Option<(HeaderName, HeaderValue)>,
    ) -> impl Future<Output = Result<(), PublishError>> + 'static + use<> {
        #[cfg(test)]
        if let Some(scripted) = &self.scripted {
            return Either::Left((scripted.0)());
        }
        let request = if let Some(token_exchange) = self.token_exchange.clone() {
            let client = self.client.clone();
            Either::Left(async move { token_exchange.publish(&client, packet, source_auth).await })
        } else {
            let original_size = packet.len();
            let client = self.client.clone();
            let endpoint = self.endpoint.clone();
            Either::Right(async move {
                let (auth_header_name, auth_header_value) =
                    source_auth.ok_or(PublishError::SourceTokenUnavailable)?;
                let response = client
                    .post(endpoint)
                    .header(reqwest::header::CONTENT_TYPE, CONTENT_TYPE)
                    .header(ORIGINAL_CONTENT_SIZE_HEADER, original_size)
                    .header(auth_header_name, auth_header_value)
                    .body(packet)
                    .send()
                    .await
                    .map_err(PublishError::request)?;

                if response.status() == StatusCode::OK {
                    Ok(())
                } else {
                    Err(PublishError::Response {
                        status: response.status(),
                    })
                }
            })
        };
        #[cfg(test)]
        let request = Either::Right(request);
        request
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
        let mut publisher = Self::new("http://127.0.0.1:1", "example-account", timeout)
            .expect("static test endpoint should be valid");
        publisher.scripted = Some(ScriptedRequests(Box::new(request)));
        publisher
    }

    /// Creates a network-free publisher with a cache deadline and a controlled expiry action.
    pub(crate) fn scripted_with_cached_credentials(
        timeout: Duration,
        usable_until: Instant,
        request: impl Fn() -> LocalBoxFuture<'static, Result<(), PublishError>> + 'static,
    ) -> (Self, impl Fn()) {
        let mut publisher = Self::scripted(timeout, request);
        let exchange = Rc::new(
            TokenExchange::new(publisher.endpoint.clone(), "example-account")
                .expect("static test exchange endpoint should be valid"),
        );
        exchange.set_cached_expiry_for_test(
            usable_until + otel_arrow_dfe_engine::capability::auth::bearer_token_provider::TOKEN_USABLE_MARGIN,
        );
        publisher.token_exchange = Some(exchange.clone());
        (publisher, move || {
            exchange.set_cached_expiry_for_test(Instant::now())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{body_bytes, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn publish(endpoint: &str, account: &str, packet: Vec<u8>) -> Result<(), PublishError> {
        MetricsPublisher::new(endpoint, account, Duration::from_secs(10))
            .expect("valid endpoint")
            .publish(
                packet,
                Some((
                    reqwest::header::AUTHORIZATION,
                    HeaderValue::from_static("test-authorization"),
                )),
            )
            .await
    }

    /// Scenario: Direct bearer publication is attempted without a ready source credential.
    /// Guarantees: Admission remains closed and publication returns a retryable auth error without making a request.
    #[tokio::test]
    async fn direct_bearer_requires_source_token() {
        let publisher = MetricsPublisher::new(
            "http://127.0.0.1:1",
            "example-account",
            Duration::from_secs(10),
        )
        .expect("valid endpoint");
        assert!(!publisher.is_ready(false));
        let error = publisher
            .publish(vec![6, 0], None)
            .await
            .expect_err("direct bearer needs a source token");
        assert!(matches!(error, PublishError::SourceTokenUnavailable));
        assert!(error.is_retryable());
    }

    /// Scenario: A server closes a direct or cached exchanged-token publication connection to a URL containing credentials and a signed query.
    /// Guarantees: Both failures identify publication, remain retryable without source-auth invalidation or refusal, and omit private URL contents in Display and Debug.
    #[tokio::test]
    async fn closed_publication_connection_is_retryable() {
        use tokio::io::AsyncReadExt;

        for token_exchange in [false, true] {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
                .await
                .expect("test listener should bind");
            let address = listener
                .local_addr()
                .expect("listener should have an address");
            let endpoint = format!(
                "http://private-user:private-password@{address}/private-path?sig=private-signature#private-fragment"
            );
            let publisher = if token_exchange {
                let publisher = MetricsPublisher::new_token_exchange(
                    &endpoint,
                    "example-account",
                    Duration::from_secs(60),
                )
                .expect("valid exchange origin");
                publisher
                    .token_exchange
                    .as_ref()
                    .expect("token exchange configured")
                    .set_cached_expiry_for_test(Instant::now() + Duration::from_secs(3600));
                publisher
            } else {
                MetricsPublisher::new(&endpoint, "example-account", Duration::from_secs(60))
                    .expect("valid endpoint")
            };
            let server = async move {
                let (mut connection, _) = listener.accept().await.expect("request should connect");
                let mut byte = [0];
                assert_eq!(
                    connection
                        .read(&mut byte)
                        .await
                        .expect("request should arrive"),
                    1
                );
            };
            let (result, ()) = tokio::time::timeout(Duration::from_secs(60), async {
                tokio::join!(
                    publisher.publish(
                        vec![6, 0],
                        Some((
                            reqwest::header::AUTHORIZATION,
                            HeaderValue::from_static("Bearer source-token"),
                        )),
                    ),
                    server
                )
            })
            .await
            .expect("publication and server should finish");
            let error = result.expect_err("closed connection must fail publication");
            let PublishError::Request(request_error) = &error else {
                panic!("expected a publication request error, got {error:?}");
            };
            assert!(request_error.url().is_none());
            assert!(
                error
                    .to_string()
                    .starts_with("Geneva metrics publication request failed:")
            );
            assert!(error.is_retryable());
            assert!(!error.is_unauthorized());
            assert!(!error.is_refusal());
            for diagnostic in [error.to_string(), format!("{error:?}")] {
                for marker in [
                    "private-user",
                    "private-password",
                    "private-path",
                    "private-signature",
                    "private-fragment",
                ] {
                    assert!(!diagnostic.contains(marker), "diagnostic exposed {marker}");
                }
            }
        }
    }

    /// Scenario: One publisher sends two protocol v6 packets through a fixed account endpoint template.
    /// Guarantees: Both requests use the same path-encoded account, including reserved characters, and preserve content headers, authorization, and packet bytes.
    #[tokio::test]
    async fn publishes_packet_with_required_headers_and_account_routing() {
        let server = MockServer::start().await;
        let packet = vec![6, 0, 1, 2, 3];
        Mock::given(method("POST"))
            .and(path("/metrics/account%20%2F%25%3F%23"))
            .and(header("content-type", CONTENT_TYPE))
            .and(header(ORIGINAL_CONTENT_SIZE_HEADER, "5"))
            .and(header("authorization", "test-authorization"))
            .and(body_bytes(packet.clone()))
            .respond_with(ResponseTemplate::new(200))
            .expect(2)
            .mount(&server)
            .await;

        let endpoint = format!("{}/metrics/{{monitoring_account}}", server.uri());
        let publisher = MetricsPublisher::new(&endpoint, "account /%?#", Duration::from_secs(10))
            .expect("valid endpoint");
        for _ in 0..2 {
            publisher
                .publish(
                    packet.clone(),
                    Some((
                        reqwest::header::AUTHORIZATION,
                        HeaderValue::from_static("test-authorization"),
                    )),
                )
                .await
                .expect("publication should succeed");
        }
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

    /// Scenario: Direct publication, token exchange, or exchanged-token publication fails with HTTP 400, 401, 403, 408, 413, 429, or 5xx.
    /// Guarantees: Retry and refusal classifications agree across all three; only direct publication and exchange 401s are classified for source-auth invalidation.
    #[test]
    fn classifies_error_responses() {
        for status in [400, 401, 403, 408, 413, 429, 500, 503] {
            let status_code = StatusCode::from_u16(status).expect("valid HTTP status");
            for (error, invalidates_source) in [
                (
                    PublishError::Response {
                        status: status_code,
                    },
                    true,
                ),
                (
                    PublishError::TokenExchangeResponse {
                        status: status_code,
                    },
                    true,
                ),
                (
                    PublishError::ExchangedTokenPublicationResponse {
                        status: status_code,
                    },
                    false,
                ),
            ] {
                assert_eq!(
                    error.is_retryable(),
                    !matches!(status, 400 | 413),
                    "{error}"
                );
                assert_eq!(
                    error.is_unauthorized(),
                    invalidates_source && status == 401,
                    "{error}"
                );
                assert_eq!(
                    error.is_refusal(),
                    matches!(status, 400 | 401 | 403 | 413 | 429),
                    "{error}"
                );
            }
        }
    }
}
