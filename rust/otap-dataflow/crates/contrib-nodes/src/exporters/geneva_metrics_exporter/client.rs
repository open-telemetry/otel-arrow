// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use reqwest::{Client, StatusCode};
use std::path::Path;
use std::time::Duration;
use thiserror::Error;

const CONTENT_TYPE: &str = "application/x-mepacket";
const ORIGINAL_CONTENT_SIZE_HEADER: &str = "OriginalContentSize";

pub(crate) struct CertificateIdentity<'a> {
    pub(crate) path: &'a Path,
    pub(crate) password: &'a str,
}

#[derive(Clone, Debug)]
pub(crate) struct MetricsPublisher {
    client: Client,
    endpoint: String,
}

#[derive(Debug, Error)]
pub(crate) enum PublisherBuildError {
    #[error("invalid Geneva metrics endpoint: {0}")]
    InvalidEndpoint(String),
    #[cfg(not(feature = "geneva-metrics-certificate-auth"))]
    #[error(
        "certificate authentication requires the 'geneva-metrics-certificate-auth' build feature"
    )]
    CertificateFeatureDisabled,
    #[cfg(feature = "geneva-metrics-certificate-auth")]
    #[error("failed to read Geneva metrics certificate {path}: {source}")]
    ReadCertificate {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[cfg(feature = "geneva-metrics-certificate-auth")]
    #[error("failed to load the Geneva metrics PKCS#12 certificate: {0}")]
    Certificate(#[source] Box<reqwest::Error>),
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
    pub(crate) fn new(
        endpoint: &str,
        timeout: Duration,
        certificate: Option<CertificateIdentity<'_>>,
    ) -> Result<Self, PublisherBuildError> {
        otel_arrow_dfe_otap::crypto::ensure_crypto_provider();
        let endpoint = endpoint.trim();
        let _endpoint = reqwest::Url::parse(endpoint)
            .map_err(|error| PublisherBuildError::InvalidEndpoint(error.to_string()))?;
        let builder = Client::builder().timeout(timeout);
        #[cfg(feature = "geneva-metrics-certificate-auth")]
        let builder = if let Some(certificate) = certificate {
            let der = std::fs::read(certificate.path).map_err(|source| {
                PublisherBuildError::ReadCertificate {
                    path: certificate.path.display().to_string(),
                    source,
                }
            })?;
            let identity = reqwest::Identity::from_pkcs12_der(&der, certificate.password)
                .map_err(|error| PublisherBuildError::Certificate(Box::new(error)))?;
            builder.tls_backend_native().identity(identity)
        } else {
            builder
        };
        #[cfg(not(feature = "geneva-metrics-certificate-auth"))]
        if let Some(certificate) = certificate {
            let _ = (certificate.path, certificate.password);
            return Err(PublisherBuildError::CertificateFeatureDisabled);
        }
        let client = builder
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
        bearer_token: Option<&str>,
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
        if let Some(token) = bearer_token {
            let mut authorization =
                reqwest::header::HeaderValue::from_str(&format!("Bearer {token}"))
                    .map_err(PublishError::InvalidAuthorizationHeader)?;
            authorization.set_sensitive(true);
            request = request.header(reqwest::header::AUTHORIZATION, authorization);
        }
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
            .and(body_bytes(packet.clone()))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;

        let publisher = MetricsPublisher::new(
            &format!("{}/metrics", server.uri()),
            Duration::from_secs(1),
            None,
        )
        .expect("publisher should be created");

        publisher
            .publish("example-account", packet, None)
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
        let publisher = MetricsPublisher::new(&server.uri(), Duration::from_secs(1), None)
            .expect("valid endpoint");

        let error = publisher
            .publish("example-account", vec![6, 0], None)
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
            let publisher = MetricsPublisher::new(&server.uri(), Duration::from_secs(1), None)
                .expect("valid endpoint");

            let error = publisher
                .publish("example-account", vec![6, 0], None)
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
            None,
        )
        .expect("valid endpoint");

        publisher
            .publish("account name", vec![6, 0], Some("test-token"))
            .await
            .expect("authenticated publication should succeed");
    }

    /// Scenario: Certificate authentication references a missing PKCS#12 file.
    /// Guarantees: Exporter construction fails explicitly before any publication attempt.
    #[cfg(feature = "geneva-metrics-certificate-auth")]
    #[test]
    fn rejects_missing_certificate_file() {
        let missing = Path::new("missing-geneva-metrics-client.p12");
        let error = MetricsPublisher::new(
            "https://example.test/metrics",
            Duration::from_secs(1),
            Some(CertificateIdentity {
                path: missing,
                password: "not-used",
            }),
        )
        .expect_err("missing certificate should fail");

        assert!(matches!(error, PublisherBuildError::ReadCertificate { .. }));
    }

    /// Scenario: Certificate authentication reads a file that is not valid PKCS#12.
    /// Guarantees: Invalid certificate material is rejected during exporter construction.
    #[cfg(feature = "geneva-metrics-certificate-auth")]
    #[test]
    fn rejects_invalid_pkcs12_certificate() {
        let path =
            std::env::temp_dir().join(format!("geneva-metrics-invalid-{}.p12", std::process::id()));
        std::fs::write(&path, b"not a pkcs12 identity").expect("fixture should be written");

        let error = MetricsPublisher::new(
            "https://example.test/metrics",
            Duration::from_secs(1),
            Some(CertificateIdentity {
                path: &path,
                password: "wrong-password",
            }),
        )
        .expect_err("invalid certificate should fail");
        std::fs::remove_file(path).expect("fixture should be removed");

        assert!(matches!(error, PublisherBuildError::Certificate(_)));
    }
}
