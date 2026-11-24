// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::client::{
    CONTENT_TYPE, ORIGINAL_CONTENT_SIZE_HEADER, PublishError, PublisherBuildError,
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use otel_arrow_dfe_engine::capability::auth::bearer_token_provider::TOKEN_USABLE_MARGIN;
use reqwest::header::{HeaderName, HeaderValue};
use reqwest::{Client, StatusCode, Url};
use serde::Deserialize;
use std::cell::RefCell;
use std::time::{Duration, Instant, SystemTime};

const MAX_EXCHANGE_RESPONSE_BYTES: usize = 64 * 1024;

#[derive(Debug)]
pub(super) struct TokenExchange {
    exchange_origin: Url,
    exchange_endpoint: Url,
    routing_id: String,
    monitoring_account: String,
    cached: RefCell<Option<CachedPublication>>,
}

#[derive(Clone, Debug)]
struct CachedPublication {
    endpoint: Url,
    authorization: HeaderValue,
    expires_at: Instant,
}

impl CachedPublication {
    fn is_usable(&self, now: Instant) -> bool {
        self.expires_at
            .checked_duration_since(now)
            .is_some_and(|remaining| remaining > TOKEN_USABLE_MARGIN)
    }
}

#[derive(Deserialize)]
struct PublicationTokenClaims {
    exp: u64,
}

#[derive(Deserialize)]
struct TokenExchangeResponse {
    #[serde(rename = "gigEndpoint")]
    publication_endpoint: String,
    #[serde(rename = "gigAuthToken")]
    publication_token: String,
}

impl TokenExchange {
    pub(super) fn new(
        exchange_origin: Url,
        monitoring_account: &str,
    ) -> Result<Self, PublisherBuildError> {
        let routing_id = exchange_origin
            .host_str()
            .and_then(|host| host.split('.').next())
            .filter(|routing_id| !routing_id.is_empty())
            .ok_or(PublisherBuildError::MissingRoutingId)?
            .to_owned();
        let mut exchange_endpoint = exchange_origin.clone();
        let _ = exchange_endpoint
            .path_segments_mut()
            .map_err(|_| PublisherBuildError::InvalidEndpoint(exchange_origin.to_string()))?
            .clear()
            .extend([
                "user-api",
                "v2",
                "authorization",
                "gig",
                "monitoringAccount",
                monitoring_account,
            ]);
        Ok(Self {
            exchange_origin,
            exchange_endpoint,
            routing_id,
            monitoring_account: monitoring_account.to_owned(),
            cached: RefCell::default(),
        })
    }

    pub(super) fn has_usable_credentials(&self) -> bool {
        self.refresh_deadline().is_some()
    }

    pub(super) fn refresh_deadline(&self) -> Option<Instant> {
        self.cached
            .borrow()
            .as_ref()
            .and_then(|cached| cached.expires_at.checked_sub(TOKEN_USABLE_MARGIN))
            .filter(|deadline| *deadline > Instant::now())
    }

    pub(super) async fn publish(
        &self,
        client: &Client,
        packet: Vec<u8>,
        source_auth: Option<(HeaderName, HeaderValue)>,
    ) -> Result<(), PublishError> {
        let (endpoint, authorization) = self.credentials(client, source_auth).await?;
        let original_size = packet.len();
        let response = client
            .post(endpoint)
            .header(reqwest::header::CONTENT_TYPE, CONTENT_TYPE)
            .header(ORIGINAL_CONTENT_SIZE_HEADER, original_size)
            .header(reqwest::header::AUTHORIZATION, authorization)
            .body(packet)
            .send()
            .await
            .map_err(PublishError::request)?;
        let status = response.status();
        if matches!(status, StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN) {
            *self.cached.borrow_mut() = None;
        }
        if status == StatusCode::OK {
            Ok(())
        } else {
            Err(PublishError::ExchangedTokenPublicationResponse { status })
        }
    }

    async fn credentials(
        &self,
        client: &Client,
        source_auth: Option<(HeaderName, HeaderValue)>,
    ) -> Result<(Url, HeaderValue), PublishError> {
        if let Some(cached) = self
            .cached
            .borrow()
            .as_ref()
            .filter(|cached| cached.is_usable(Instant::now()))
        {
            return Ok((cached.endpoint.clone(), cached.authorization.clone()));
        }
        *self.cached.borrow_mut() = None;
        let (auth_header_name, auth_header_value) =
            source_auth.ok_or(PublishError::SourceTokenUnavailable)?;
        let mut response = client
            .get(self.exchange_endpoint.clone())
            .header(auth_header_name, auth_header_value)
            .send()
            .await
            .map_err(PublishError::request)?;
        if response.status() != StatusCode::OK {
            return Err(PublishError::TokenExchangeResponse {
                status: response.status(),
            });
        }
        let mut body = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(PublishError::request)? {
            if chunk.len() > MAX_EXCHANGE_RESPONSE_BYTES - body.len() {
                return Err(PublishError::ExchangeResponseTooLarge {
                    limit: MAX_EXCHANGE_RESPONSE_BYTES,
                });
            }
            body.extend_from_slice(&chunk);
        }
        let exchange: TokenExchangeResponse = serde_json::from_slice(&body)
            .map_err(|source| PublishError::InvalidExchangeResponse(Box::new(source)))?;
        let publication_endpoint = exchange.publication_endpoint.trim();
        if publication_endpoint.is_empty() {
            return Err(PublishError::MissingExchangeField {
                field: "publication endpoint",
            });
        }
        let publication_token = exchange.publication_token.trim();
        if publication_token.is_empty() {
            return Err(PublishError::MissingExchangeField {
                field: "publication token",
            });
        }
        let publication_endpoint = build_publication_endpoint(
            &self.exchange_origin,
            publication_endpoint,
            &self.monitoring_account,
            &self.routing_id,
        )?;
        let mut publication_auth = HeaderValue::from_str(&format!("Bearer {publication_token}"))
            .map_err(|_| PublishError::InvalidPublicationToken)?;
        publication_auth.set_sensitive(true);
        if let Some(expires_at) =
            publication_token_expiry(publication_token, SystemTime::now(), Instant::now())
        {
            let cached = CachedPublication {
                endpoint: publication_endpoint.clone(),
                authorization: publication_auth.clone(),
                expires_at,
            };
            if cached.is_usable(Instant::now()) {
                *self.cached.borrow_mut() = Some(cached);
            } else {
                otel_debug!(
                    "geneva_metrics_exporter.token_exchange.uncacheable",
                    message = "Publication token expires within the usability margin; it will not be cached"
                );
            }
        } else {
            otel_debug!(
                "geneva_metrics_exporter.token_exchange.uncacheable",
                message = "Publication token has no usable JWT expiry; it will not be cached"
            );
        }
        Ok((publication_endpoint, publication_auth))
    }
}

#[cfg(test)]
impl TokenExchange {
    pub(super) fn set_cached_expiry_for_test(&self, expires_at: Instant) {
        *self.cached.borrow_mut() = Some(CachedPublication {
            endpoint: self.exchange_origin.clone(),
            authorization: HeaderValue::from_static("Bearer test-publication-token"),
            expires_at,
        });
    }
}

// JWT claims only bound cache reuse; the ingestion service authenticates the token.
fn publication_token_expiry(
    token: &str,
    now: SystemTime,
    monotonic_now: Instant,
) -> Option<Instant> {
    let mut parts = token.split('.');
    let _ = parts.next()?;
    let payload = parts.next()?;
    let _ = parts.next()?;
    if parts.next().is_some() {
        return None;
    }
    let claims = URL_SAFE_NO_PAD.decode(payload).ok()?;
    let claims: PublicationTokenClaims = serde_json::from_slice(&claims).ok()?;
    let expires_at = SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs(claims.exp))?;
    monotonic_now.checked_add(expires_at.duration_since(now).ok()?)
}

fn build_publication_endpoint(
    exchange_origin: &Url,
    publication_endpoint: &str,
    monitoring_account: &str,
    routing_id: &str,
) -> Result<Url, PublishError> {
    let mut endpoint = if publication_endpoint.contains("://") {
        Url::parse(publication_endpoint)
    } else {
        Url::parse(&format!(
            "{}://{publication_endpoint}",
            exchange_origin.scheme()
        ))
    }
    .map_err(|error| PublishError::InvalidPublicationEndpoint(error.to_string()))?;
    if !matches!(endpoint.scheme(), "http" | "https") {
        return Err(PublishError::InvalidPublicationEndpoint(
            "publication endpoint scheme must be HTTP or HTTPS".to_owned(),
        ));
    }
    if exchange_origin.scheme() == "https" && endpoint.scheme() != "https" {
        return Err(PublishError::InvalidPublicationEndpoint(
            "publication endpoint must use HTTPS".to_owned(),
        ));
    }
    if endpoint.host_str().is_none() {
        return Err(PublishError::InvalidPublicationEndpoint(
            "publication endpoint must have a hostname".to_owned(),
        ));
    }
    endpoint.set_path("/api/v1/ingestion/ingest");
    endpoint.set_query(None);
    endpoint.set_fragment(None);
    let _ = endpoint
        .query_pairs_mut()
        .append_pair("accountId", monitoring_account)
        .append_pair("stampId", routing_id);
    Ok(endpoint)
}

#[cfg(test)]
mod tests {
    use super::super::client::MetricsPublisher;
    use super::*;
    use serde_json::json;
    use std::time::Duration;
    use wiremock::matchers::{body_bytes, header, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn exchange_response(server: &MockServer, token: &str) -> ResponseTemplate {
        ResponseTemplate::new(200).set_body_json(json!({
            "gigEndpoint": server.uri().trim_start_matches("http://"),
            "gigAuthToken": token,
        }))
    }

    fn publication_token(expires_at: SystemTime) -> String {
        let exp = expires_at
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("test expiry follows the Unix epoch")
            .as_secs();
        let claims = URL_SAFE_NO_PAD.encode(json!({"exp": exp}).to_string());
        format!("e30.{claims}.signature")
    }

    async fn publish_with_token_exchange(publisher: &MetricsPublisher) -> Result<(), PublishError> {
        publisher
            .publish(
                vec![6, 0],
                Some((
                    reqwest::header::AUTHORIZATION,
                    HeaderValue::from_static("Bearer source-token"),
                )),
            )
            .await
    }

    /// Scenario: A returned endpoint is a bare hostname or an absolute HTTPS URL with an existing route.
    /// Guarantees: Both forms use HTTPS and replace any old path, query, and fragment with the publication route.
    #[test]
    fn builds_https_publication_endpoints() {
        let origin = Url::parse("https://exchange.example.test").expect("valid exchange origin");
        for endpoint in [
            "publication.example.test",
            "https://publication.example.test/old-path?old-query=1#old-fragment",
        ] {
            let endpoint =
                build_publication_endpoint(&origin, endpoint, "example-account", "exchange")
                    .expect("valid publication endpoint");
            assert_eq!(
                endpoint.as_str(),
                "https://publication.example.test/api/v1/ingestion/ingest?accountId=example-account&stampId=exchange"
            );
        }
    }

    /// Scenario: A returned endpoint downgrades HTTPS, uses an unsupported scheme, or has no hostname.
    /// Guarantees: Invalid endpoints fail explicitly instead of producing a publication URL.
    #[test]
    fn rejects_invalid_publication_endpoints() {
        for (origin, endpoint) in [
            (
                "https://exchange.example.test",
                "http://publication.example.test",
            ),
            (
                "http://exchange.example.test",
                "ftp://publication.example.test",
            ),
            ("https://exchange.example.test", "https://"),
        ] {
            let origin = Url::parse(origin).expect("valid exchange origin");
            assert!(
                matches!(
                    build_publication_endpoint(&origin, endpoint, "example-account", "exchange"),
                    Err(PublishError::InvalidPublicationEndpoint(_))
                ),
                "endpoint {endpoint} must be rejected for origin {origin}"
            );
        }
    }

    /// Scenario: An HTTPS exchange returns HTTP or FTP publication URLs containing userinfo, a signed query, and a fragment.
    /// Guarantees: Non-HTTPS endpoints are rejected, and Display/Debug diagnostics omit private URL contents.
    #[test]
    fn publication_endpoint_validation_rejects_non_https_urls() {
        let origin = Url::parse("https://exchange.example.test").expect("valid exchange origin");
        for scheme in ["http", "ftp"] {
            let endpoint = format!(
                "{scheme}://private-user:private-password@private-host.example.test/private-path?sig=private-signature#private-fragment"
            );
            let error =
                build_publication_endpoint(&origin, &endpoint, "example-account", "exchange")
                    .expect_err("publication endpoint must be rejected");
            assert!(matches!(error, PublishError::InvalidPublicationEndpoint(_)));
            for diagnostic in [error.to_string(), format!("{error:?}")] {
                for marker in [
                    "private-user",
                    "private-password",
                    "private-host",
                    "private-path",
                    "private-signature",
                    "private-fragment",
                ] {
                    assert!(!diagnostic.contains(marker), "diagnostic exposed {marker}");
                }
            }
        }
    }

    /// Scenario: Chunked exchange responses contain valid JSON exactly at or one byte over the size limit.
    /// Guarantees: The limit is inclusive and oversized responses fail without publishing, even without Content-Length.
    #[tokio::test]
    async fn enforces_exchange_response_size_limit() {
        let publication = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&publication)
            .await;
        for response_size in [MAX_EXCHANGE_RESPONSE_BYTES, MAX_EXCHANGE_RESPONSE_BYTES + 1] {
            let mut body = json!({
                "gigEndpoint": publication.uri(),
                "gigAuthToken": "publication-token",
                "padding": "",
            });
            body["padding"] = json!("x".repeat(response_size - body.to_string().len()));
            let home = MockServer::start().await;
            Mock::given(method("GET"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_raw(body.to_string(), "application/json")
                        .insert_header("transfer-encoding", "chunked"),
                )
                .expect(1)
                .mount(&home)
                .await;
            let publisher = MetricsPublisher::new_token_exchange(
                &home.uri(),
                "example-account",
                Duration::from_secs(10),
            )
            .expect("valid exchange origin");
            let result = publish_with_token_exchange(&publisher).await;
            if response_size == MAX_EXCHANGE_RESPONSE_BYTES {
                result.expect("response at the limit should permit publication");
            } else {
                let error = result.expect_err("oversized response must fail");
                assert!(matches!(
                    error,
                    PublishError::ExchangeResponseTooLarge { limit }
                        if limit == MAX_EXCHANGE_RESPONSE_BYTES
                ));
                assert!(error.is_retryable());
                assert!(!error.is_unauthorized());
            }
        }
    }

    /// Scenario: Two packets receive a JWT or an opaque token, and publication succeeds or returns a server error.
    /// Guarantees: JWT credentials and escaped account routing are reused even after transient errors; opaque tokens repeat the same exchange URL without changing publication headers or bytes.
    #[tokio::test]
    async fn caches_only_credentials_with_known_expiry() {
        let account = "account /%?#";
        for (token, exchanges, status) in [
            (
                publication_token(SystemTime::now() + Duration::from_secs(3600)),
                1,
                200,
            ),
            ("publication-token".to_owned(), 2, 200),
            (
                publication_token(SystemTime::now() + Duration::from_secs(3600)),
                1,
                500,
            ),
        ] {
            let publication = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/api/v1/ingestion/ingest"))
                .and(query_param("accountId", account))
                .and(query_param("stampId", "127"))
                .and(header("authorization", format!("Bearer {token}")))
                .and(header("content-type", CONTENT_TYPE))
                .and(header(ORIGINAL_CONTENT_SIZE_HEADER, "2"))
                .and(body_bytes(vec![6, 0]))
                .respond_with(ResponseTemplate::new(status))
                .expect(2)
                .mount(&publication)
                .await;
            let home = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path(
                    "/user-api/v2/authorization/gig/monitoringAccount/account%20%2F%25%3F%23",
                ))
                .and(header("authorization", "Bearer source-token"))
                .respond_with(exchange_response(&publication, &token))
                .expect(exchanges)
                .mount(&home)
                .await;
            let publisher =
                MetricsPublisher::new_token_exchange(&home.uri(), account, Duration::from_secs(10))
                    .expect("valid exchange origin");
            for _ in 0..2 {
                let result = publish_with_token_exchange(&publisher).await;
                if status == 200 {
                    result.expect("Geneva publication should succeed");
                } else {
                    assert!(matches!(
                        result,
                        Err(PublishError::ExchangedTokenPublicationResponse { status: actual })
                            if actual.as_u16() == status
                    ));
                }
            }
        }
    }

    /// Scenario: A JWT's absolute expiry is converted to a monotonic deadline or cannot be decoded.
    /// Guarantees: Cached credentials become unusable at the monotonic safety margin; past, missing, malformed or overflowing expiry metadata produces no deadline.
    #[test]
    fn respects_publication_token_expiry() {
        let wall_now = SystemTime::UNIX_EPOCH + Duration::from_secs(100);
        let now = Instant::now();
        let mut cached = CachedPublication {
            endpoint: Url::parse("https://example.test").expect("valid URL"),
            authorization: HeaderValue::from_static("test-token"),
            expires_at: now,
        };
        for remaining in [0, 29, 30, 31] {
            cached.expires_at = now + Duration::from_secs(remaining);
            let token = publication_token(wall_now + Duration::from_secs(remaining));
            assert_eq!(
                publication_token_expiry(&token, wall_now, now),
                Some(cached.expires_at)
            );
            assert_eq!(cached.is_usable(now), remaining > 30);
        }
        assert!(cached.is_usable(now));
        assert!(!cached.is_usable(now + Duration::from_secs(1)));
        assert!(!cached.is_usable(cached.expires_at + Duration::from_secs(1)));
        let past = publication_token(SystemTime::UNIX_EPOCH);
        let overflowing = format!(
            "e30.{}.signature",
            URL_SAFE_NO_PAD.encode(json!({"exp": u64::MAX}).to_string())
        );
        for token in [
            "opaque",
            "a.invalid.c",
            "a.e30.c",
            "a.e30.c.extra",
            &past,
            &overflowing,
        ] {
            assert_eq!(publication_token_expiry(token, wall_now, now), None);
        }
    }

    /// Scenario: A previously cached credential expires and the exchange returns a replacement endpoint and token.
    /// Guarantees: Expiry triggers a new exchange and replaces the credential and route together without using stale data.
    #[tokio::test]
    async fn refreshes_expired_credentials_and_routing() {
        otel_arrow_dfe_otap::crypto::ensure_crypto_provider();
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("test HTTP client should build");
        let home = MockServer::start().await;
        let old_endpoint = MockServer::start().await;
        let new_endpoint = MockServer::start().await;
        let old_token = publication_token(SystemTime::now() + Duration::from_secs(3600));
        let new_token = publication_token(SystemTime::now() + Duration::from_secs(7200));
        let exchange = TokenExchange::new(
            Url::parse(&home.uri()).expect("valid exchange origin"),
            "example-account",
        )
        .expect("valid token exchange");
        for (endpoint, token) in [(&old_endpoint, &old_token), (&new_endpoint, &new_token)] {
            Mock::given(method("GET"))
                .respond_with(exchange_response(endpoint, token))
                .expect(1)
                .mount(&home)
                .await;
            let (actual_endpoint, authorization) = exchange
                .credentials(
                    &client,
                    Some((
                        reqwest::header::AUTHORIZATION,
                        HeaderValue::from_static("Bearer source-token"),
                    )),
                )
                .await
                .expect("credentials should be exchanged");
            assert_eq!(
                actual_endpoint.origin().ascii_serialization(),
                endpoint.uri()
            );
            assert_eq!(authorization, format!("Bearer {token}"));
            assert!(authorization.is_sensitive());
            exchange
                .cached
                .borrow_mut()
                .as_mut()
                .expect("credential should be cached")
                .expires_at = Instant::now();
            home.verify().await;
            home.reset().await;
        }
    }

    /// Scenario: Cached credentials expire, their refresh fails, and a later exchange succeeds.
    /// Guarantees: Refresh failure is returned without publishing to the stale endpoint, and recovery caches fresh credentials for subsequent packets.
    #[tokio::test]
    async fn recovers_from_failed_refresh_without_publishing_stale_credentials() {
        otel_arrow_dfe_otap::crypto::ensure_crypto_provider();
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("test HTTP client should build");
        let home = MockServer::start().await;
        let stale_endpoint = MockServer::start().await;
        let fresh_endpoint = MockServer::start().await;
        let token = publication_token(SystemTime::now() + Duration::from_secs(3600));
        let fresh_token = publication_token(SystemTime::now() + Duration::from_secs(7200));
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200))
            .expect(0)
            .mount(&stale_endpoint)
            .await;
        Mock::given(method("POST"))
            .and(header("authorization", format!("Bearer {fresh_token}")))
            .respond_with(ResponseTemplate::new(200))
            .expect(2)
            .mount(&fresh_endpoint)
            .await;
        Mock::given(method("GET"))
            .respond_with(exchange_response(&stale_endpoint, &token))
            .expect(1)
            .mount(&home)
            .await;
        let exchange = TokenExchange::new(
            Url::parse(&home.uri()).expect("valid exchange origin"),
            "example-account",
        )
        .expect("valid token exchange");
        let _ = exchange
            .credentials(
                &client,
                Some((
                    reqwest::header::AUTHORIZATION,
                    HeaderValue::from_static("Bearer source-token"),
                )),
            )
            .await
            .expect("initial credentials should be cached");
        exchange
            .cached
            .borrow_mut()
            .as_mut()
            .expect("credential should be cached")
            .expires_at = Instant::now();
        home.verify().await;
        home.reset().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&home)
            .await;
        let publish = || {
            exchange.publish(
                &client,
                vec![6, 0],
                Some((
                    reqwest::header::AUTHORIZATION,
                    HeaderValue::from_static("Bearer source-token"),
                )),
            )
        };
        let error = publish()
            .await
            .expect_err("refresh failure must be returned");
        assert!(matches!(
            error,
            PublishError::TokenExchangeResponse {
                status: StatusCode::INTERNAL_SERVER_ERROR
            }
        ));
        home.verify().await;
        home.reset().await;
        Mock::given(method("GET"))
            .respond_with(exchange_response(&fresh_endpoint, &fresh_token))
            .expect(1)
            .mount(&home)
            .await;
        for _ in 0..2 {
            publish()
                .await
                .expect("publishing should recover with fresh cached credentials");
        }
    }

    /// Scenario: Cached publication credentials reach their expiry margin without a ready source token.
    /// Guarantees: Usable credentials need no source token, but expiry fails explicitly without attempting another exchange.
    #[tokio::test]
    async fn requires_source_token_only_after_cached_credentials_expire() {
        let home = MockServer::start().await;
        let publication = MockServer::start().await;
        let token = publication_token(SystemTime::now() + Duration::from_secs(3600));
        Mock::given(method("GET"))
            .respond_with(exchange_response(&publication, &token))
            .expect(1)
            .mount(&home)
            .await;
        otel_arrow_dfe_otap::crypto::ensure_crypto_provider();
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("test HTTP client should build");
        let exchange = TokenExchange::new(
            Url::parse(&home.uri()).expect("valid exchange origin"),
            "example-account",
        )
        .expect("valid token exchange");
        let expected = exchange
            .credentials(
                &client,
                Some((
                    reqwest::header::AUTHORIZATION,
                    HeaderValue::from_static("Bearer source-token"),
                )),
            )
            .await
            .expect("initial exchange should succeed");
        assert!(exchange.has_usable_credentials());
        assert_eq!(
            exchange
                .credentials(&client, None)
                .await
                .expect("cached credentials need no source token"),
            expected
        );
        exchange
            .cached
            .borrow_mut()
            .as_mut()
            .expect("cached credentials")
            .expires_at = Instant::now() + TOKEN_USABLE_MARGIN;
        assert!(!exchange.has_usable_credentials());
        let error = exchange
            .credentials(&client, None)
            .await
            .expect_err("expiry requires a source token");
        assert!(matches!(error, PublishError::SourceTokenUnavailable));
        assert!(error.is_retryable());
        assert!(!error.is_unauthorized());
        assert!(exchange.cached.borrow().is_none());
        home.verify().await;
    }

    /// Scenario: The exchange returns malformed JSON or a missing or empty publication endpoint or token.
    /// Guarantees: Invalid credentials produce an explicit exchange error and no publication request.
    #[tokio::test]
    async fn rejects_invalid_exchange_responses_without_publishing() {
        let publication = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200))
            .expect(0)
            .mount(&publication)
            .await;
        for (scenario, body) in [
            ("malformed JSON", "{".to_owned()),
            (
                "missing endpoint",
                json!({"gigAuthToken": "publication-token"}).to_string(),
            ),
            (
                "missing token",
                json!({"gigEndpoint": publication.uri()}).to_string(),
            ),
            (
                "empty endpoint",
                json!({"gigEndpoint": "", "gigAuthToken": "publication-token"}).to_string(),
            ),
            (
                "empty token",
                json!({"gigEndpoint": publication.uri(), "gigAuthToken": ""}).to_string(),
            ),
        ] {
            let home = MockServer::start().await;
            Mock::given(method("GET"))
                .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
                .expect(1)
                .mount(&home)
                .await;
            let publisher = MetricsPublisher::new_token_exchange(
                &home.uri(),
                "example-account",
                Duration::from_secs(10),
            )
            .expect("valid exchange origin");
            let error = publish_with_token_exchange(&publisher)
                .await
                .expect_err(scenario);
            assert!(
                matches!(
                    error,
                    PublishError::InvalidExchangeResponse(_)
                        | PublishError::MissingExchangeField { .. }
                ),
                "{scenario}: {error}"
            );
        }
    }

    /// Scenario: Geneva rejects cached publication credentials with HTTP 401 or 403.
    /// Guarantees: The rejected cache entry is cleared without an internal retry or source-token invalidation, and the next publication exchanges and caches fresh credentials.
    #[tokio::test]
    async fn invalidates_rejected_publication_credentials_without_retrying() {
        for status in [401, 403] {
            let old_token = publication_token(SystemTime::now() + Duration::from_secs(3600));
            let new_token = publication_token(SystemTime::now() + Duration::from_secs(7200));
            let publication = MockServer::start().await;
            Mock::given(method("POST"))
                .and(header("authorization", format!("Bearer {old_token}")))
                .respond_with(ResponseTemplate::new(status))
                .expect(1)
                .mount(&publication)
                .await;
            let home = MockServer::start().await;
            Mock::given(method("GET"))
                .respond_with(exchange_response(&publication, &old_token))
                .expect(1)
                .mount(&home)
                .await;
            let publisher = MetricsPublisher::new_token_exchange(
                &home.uri(),
                "example-account",
                Duration::from_secs(10),
            )
            .expect("valid exchange origin");
            let error = publish_with_token_exchange(&publisher)
                .await
                .expect_err("publication rejection must fail");
            assert!(matches!(
                error,
                PublishError::ExchangedTokenPublicationResponse {
                    status: actual
                } if actual.as_u16() == status
            ));
            assert!(!error.is_unauthorized());
            assert!(error.is_retryable());
            home.verify().await;
            home.reset().await;
            Mock::given(method("GET"))
                .respond_with(exchange_response(&publication, &new_token))
                .expect(1)
                .mount(&home)
                .await;
            Mock::given(method("POST"))
                .and(header("authorization", format!("Bearer {new_token}")))
                .respond_with(ResponseTemplate::new(200))
                .expect(2)
                .mount(&publication)
                .await;
            for _ in 0..2 {
                publish_with_token_exchange(&publisher)
                    .await
                    .expect("publication should use refreshed credentials");
            }
        }
    }

    /// Scenario: The exchange returns an already expired publication JWT twice.
    /// Guarantees: Expiry metadata only controls cache reuse; each packet exchanges credentials and leaves authentication to the ingestion service.
    #[tokio::test]
    async fn does_not_cache_expired_publication_tokens() {
        let publication = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200))
            .expect(2)
            .mount(&publication)
            .await;
        let home = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(exchange_response(
                &publication,
                &publication_token(SystemTime::UNIX_EPOCH),
            ))
            .expect(2)
            .mount(&home)
            .await;
        let publisher = MetricsPublisher::new_token_exchange(
            &home.uri(),
            "example-account",
            Duration::from_secs(10),
        )
        .expect("valid exchange origin");
        for _ in 0..2 {
            publish_with_token_exchange(&publisher)
                .await
                .expect("the ingestion service authenticates freshly exchanged credentials");
        }
    }

    /// Scenario: The Geneva exchange returns HTTP 401 or 403 before publication.
    /// Guarantees: Only 401 is classified for source-auth invalidation; both statuses retain the current runtime's retryable refusal classification.
    #[tokio::test]
    async fn classifies_token_exchange_authorization_failures() {
        for status in [401, 403] {
            let home = MockServer::start().await;
            Mock::given(method("GET"))
                .respond_with(ResponseTemplate::new(status))
                .expect(1)
                .mount(&home)
                .await;
            let publisher = MetricsPublisher::new_token_exchange(
                &home.uri(),
                "example-account",
                Duration::from_secs(10),
            )
            .expect("valid exchange origin");
            let error = publish_with_token_exchange(&publisher)
                .await
                .expect_err("exchange should fail");
            assert!(matches!(
                error,
                PublishError::TokenExchangeResponse { status: actual }
                    if actual.as_u16() == status
            ));
            assert_eq!(
                error.to_string(),
                format!(
                    "Geneva token exchange returned HTTP {}",
                    StatusCode::from_u16(status).expect("valid HTTP status")
                )
            );
            assert_eq!(error.is_unauthorized(), status == 401);
            assert!(error.is_retryable());
            assert!(error.is_refusal());
        }
    }
}
