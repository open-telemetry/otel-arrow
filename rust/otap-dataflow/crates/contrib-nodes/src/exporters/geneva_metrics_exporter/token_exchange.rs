// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::client::{
    CONTENT_TYPE, ORIGINAL_CONTENT_SIZE_HEADER, PublishError, PublisherBuildError,
};
use reqwest::header::{HeaderName, HeaderValue};
use reqwest::{Client, StatusCode, Url};
use serde::Deserialize;

const MAX_EXCHANGE_RESPONSE_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug)]
pub(super) struct TokenExchange {
    exchange_origin: Url,
    routing_id: String,
    monitoring_account: String,
}

#[derive(Debug, Deserialize)]
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
        Ok(Self {
            exchange_origin,
            routing_id,
            monitoring_account: monitoring_account.to_owned(),
        })
    }

    pub(super) async fn publish(
        &self,
        client: &Client,
        packet: Vec<u8>,
        auth_header_name: HeaderName,
        auth_header_value: HeaderValue,
    ) -> Result<(), PublishError> {
        let mut exchange_endpoint = self.exchange_origin.clone();
        let _ = exchange_endpoint
            .path_segments_mut()
            .map_err(|_| {
                PublishError::InvalidPublicationEndpoint(self.exchange_origin.to_string())
            })?
            .clear()
            .extend([
                "user-api",
                "v2",
                "authorization",
                "gig",
                "monitoringAccount",
                &self.monitoring_account,
            ]);
        let mut response = client
            .get(exchange_endpoint)
            .header(auth_header_name, auth_header_value)
            .send()
            .await
            .map_err(|error| PublishError::Request(Box::new(error)))?;
        if response.status() != StatusCode::OK {
            return Err(PublishError::Response {
                status: response.status(),
            });
        }
        let mut body = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|error| PublishError::Request(Box::new(error)))?
        {
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
        let original_size = packet.len();
        let response = client
            .post(publication_endpoint)
            .header(reqwest::header::CONTENT_TYPE, CONTENT_TYPE)
            .header(ORIGINAL_CONTENT_SIZE_HEADER, original_size)
            .header(reqwest::header::AUTHORIZATION, publication_auth)
            .body(packet)
            .send()
            .await
            .map_err(|error| PublishError::Request(Box::new(error)))?;
        if response.status() == StatusCode::OK {
            Ok(())
        } else {
            Err(PublishError::ExchangedTokenPublicationResponse {
                status: response.status(),
            })
        }
    }
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
    if !matches!(endpoint.scheme(), "http" | "https")
        || (exchange_origin.scheme() == "https" && endpoint.scheme() != "https")
        || endpoint.host_str().is_none()
    {
        return Err(PublishError::InvalidPublicationEndpoint(
            endpoint.to_string(),
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

    async fn publish_with_token_exchange(publisher: &MetricsPublisher) -> Result<(), PublishError> {
        publisher
            .publish(
                vec![6, 0],
                reqwest::header::AUTHORIZATION,
                HeaderValue::from_static("Bearer source-token"),
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

    /// Scenario: Two packets use the same Azure bearer token and monitoring account.
    /// Guarantees: Each publication exchanges the token and uses the returned credential, route, headers, and packet bytes.
    #[tokio::test]
    async fn exchanges_source_token_for_each_publication() {
        let publication = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/ingestion/ingest"))
            .and(query_param("accountId", "example-account"))
            .and(query_param("stampId", "127"))
            .and(header("authorization", "Bearer publication-token"))
            .and(header("content-type", CONTENT_TYPE))
            .and(header(ORIGINAL_CONTENT_SIZE_HEADER, "2"))
            .and(body_bytes(vec![6, 0]))
            .respond_with(ResponseTemplate::new(200))
            .expect(2)
            .mount(&publication)
            .await;
        let home = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(
                "/user-api/v2/authorization/gig/monitoringAccount/example-account",
            ))
            .and(header("authorization", "Bearer source-token"))
            .respond_with(exchange_response(&publication, "publication-token"))
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
                .expect("Geneva publication should succeed");
        }
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

    /// Scenario: Geneva rejects the exchanged publication token with HTTP 401.
    /// Guarantees: The error is returned without an internal retry or invalidating the source Azure token.
    #[tokio::test]
    async fn returns_publication_unauthorized_without_retrying() {
        let publication = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(401))
            .expect(1)
            .mount(&publication)
            .await;
        let home = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(exchange_response(&publication, "publication-token"))
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
                status: StatusCode::UNAUTHORIZED
            }
        ));
        assert!(!error.is_unauthorized());
        assert!(error.is_retryable());
    }

    /// Scenario: The Geneva exchange returns HTTP 401 or 403 before publication.
    /// Guarantees: Only 401 invalidates source auth; both statuses retain the current runtime's retryable refusal classification.
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
            assert_eq!(error.is_unauthorized(), status == 401);
            assert!(error.is_retryable());
            assert!(error.is_refusal());
        }
    }
}
