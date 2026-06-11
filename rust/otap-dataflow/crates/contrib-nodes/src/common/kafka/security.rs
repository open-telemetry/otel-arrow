// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared security configuration helpers for Kafka receiver and exporter.
//!
//! Both the Kafka receiver and exporter need to:
//! 1. Determine the security protocol from TLS + Auth configuration
//! 2. Apply TLS certificate paths to the rdkafka `ClientConfig`
//! 3. Apply SASL mechanism settings for authentication
//! 4. Optionally construct an AWS MSK IAM client context (requires the `aws`
//!    feature)
//!
//! This module extracts those common operations so they are defined once and
//! used by both components, ensuring consistent behavior.

#[cfg(feature = "aws")]
use std::borrow::Cow;

#[cfg(feature = "aws")]
use aws_config::Region;
use rdkafka::ClientConfig;

use super::TlsConfig;
use super::auth::Auth;
#[cfg(feature = "aws")]
use super::auth::SaslMechanism;
#[cfg(feature = "aws")]
use super::aws::AwsMskAuthClientContext;

/// Resolves the Kafka security protocol string from TLS and Auth configuration.
///
/// The mapping follows the rdkafka/librdkafka convention:
///
/// | TLS      | Auth                    | Protocol         |
/// |----------|-------------------------|------------------|
/// | present  | present                 | `SASL_SSL`       |
/// | present  | absent                  | `SSL`            |
/// | absent   | AWS MSK IAM OAUTHBEARER | `SASL_SSL`       |
/// | absent   | other SASL              | `SASL_PLAINTEXT` |
/// | absent   | absent                  | `PLAINTEXT`      |
///
/// AWS MSK IAM always requires SSL even without explicit TLS config because
/// the MSK broker enforces encrypted connections.
#[must_use]
pub fn resolve_security_protocol(tls: Option<&TlsConfig>, auth: Option<&Auth>) -> &'static str {
    match (tls, auth) {
        (Some(_), Some(_)) => "SASL_SSL",
        (Some(_), None) => "SSL",
        #[cfg(feature = "aws")]
        (None, Some(Auth::Sasl(sasl)))
            if sasl.mechanism() == SaslMechanism::AwsMskIamOauthbearer =>
        {
            "SASL_SSL"
        }
        (None, Some(_)) => "SASL_PLAINTEXT",
        (None, None) => "PLAINTEXT",
    }
}

/// Applies SASL-specific settings to an rdkafka [`ClientConfig`].
///
/// Supports:
/// - **PLAIN / SCRAM-SHA-256 / SCRAM-SHA-512**: sets `sasl.mechanism`,
///   `sasl.username`, and `sasl.password`.
/// - **AWS MSK IAM OAUTHBEARER**: sets `sasl.mechanism` to `OAUTHBEARER`
///   (requires the `aws` feature).
pub fn apply_sasl_config(auth: Option<&Auth>, config: &mut ClientConfig) {
    if let Some(Auth::Sasl(sasl_auth)) = auth {
        let mechanism = sasl_auth.mechanism();

        _ = config.set("sasl.mechanism", mechanism.as_rdkafka_str());

        if mechanism.is_username_password() {
            if let Some(username) = sasl_auth.username() {
                _ = config.set("sasl.username", username);
            }
            if let Some(password) = sasl_auth.password() {
                _ = config.set("sasl.password", password);
            }
        }
    }
}

/// Constructs an [`AwsMskAuthClientContext`] if the auth configuration
/// specifies AWS MSK IAM authentication.
///
/// Returns `None` if auth is not configured or uses a non-AWS-MSK mechanism.
/// Both the receiver and exporter use this to decide whether to pass a custom
/// client context to the rdkafka consumer/producer.
///
/// As a defense-in-depth measure, this checks both the mechanism **and** the
/// presence of the `aws_msk` block. Even if a misconfigured `SaslAuth` has an
/// `aws_msk` block with a non-MSK mechanism (which `validate()` now rejects),
/// this function will not create an OAUTHBEARER context for it.
#[cfg(feature = "aws")]
#[must_use]
pub fn build_aws_msk_context(auth: Option<&Auth>) -> Option<AwsMskAuthClientContext> {
    if let Some(Auth::Sasl(sasl_auth)) = auth {
        if sasl_auth.mechanism() == SaslMechanism::AwsMskIamOauthbearer {
            if let Some(aws_msk) = sasl_auth.aws_msk() {
                let region = Region::new(Cow::Owned(aws_msk.region().to_owned()));
                return Some(AwsMskAuthClientContext::new(region));
            }
        }
    }
    None
}

impl TlsConfig {
    /// Applies TLS certificate paths and verification settings to an rdkafka
    /// [`ClientConfig`].
    ///
    /// Only sets the librdkafka properties for fields that are present:
    /// - `ssl.ca.location` (when `ca_file` is set)
    /// - `ssl.certificate.location` (when `cert_file` is set)
    /// - `ssl.key.location` (when `key_file` is set)
    /// - `ssl.key.password` (when `key_password` is set)
    /// - `enable.ssl.certificate.verification` (disabled when `insecure` is true)
    pub fn apply_to_client_config(&self, config: &mut ClientConfig) {
        if let Some(ca) = self.ca_file() {
            _ = config.set("ssl.ca.location", ca);
        }
        if let Some(cert) = self.cert_file() {
            _ = config.set("ssl.certificate.location", cert);
        }
        if let Some(key) = self.key_file() {
            _ = config.set("ssl.key.location", key);
        }
        if let Some(key_pw) = self.key_password() {
            _ = config.set("ssl.key.password", key_pw);
        }
        _ = config.set(
            "enable.ssl.certificate.verification",
            if self.insecure() { "false" } else { "true" },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::kafka::auth::{SaslAuth, SaslMechanism};

    fn make_tls() -> TlsConfig {
        TlsConfig::new(
            "ca.pem".to_string(),
            "cert.pem".to_string(),
            "key.pem".to_string(),
            None,
            false,
        )
    }

    #[cfg(feature = "aws")]
    fn make_aws_msk_auth() -> Auth {
        Auth::new_aws_msk_iam("us-east-1".to_string())
    }

    #[cfg(feature = "aws")]
    fn make_generic_sasl_auth() -> Auth {
        Auth::Sasl(SaslAuth::new(SaslMechanism::Plain, None))
    }

    #[cfg(not(feature = "aws"))]
    fn make_generic_sasl_auth() -> Auth {
        Auth::Sasl(SaslAuth::new(SaslMechanism::Plain))
    }

    fn make_plain_auth() -> Auth {
        Auth::new_sasl(
            SaslMechanism::Plain,
            "myuser".to_string(),
            "mypass".to_string(),
        )
    }

    fn make_scram_sha256_auth() -> Auth {
        Auth::new_sasl(
            SaslMechanism::ScramSha256,
            "myuser".to_string(),
            "mypass".to_string(),
        )
    }

    fn make_scram_sha512_auth() -> Auth {
        Auth::new_sasl(
            SaslMechanism::ScramSha512,
            "myuser".to_string(),
            "mypass".to_string(),
        )
    }

    // ── resolve_security_protocol ───────────────────────────

    #[cfg(feature = "aws")]
    #[test]
    fn protocol_tls_and_auth() {
        let tls = make_tls();
        let auth = make_aws_msk_auth();
        assert_eq!(
            resolve_security_protocol(Some(&tls), Some(&auth)),
            "SASL_SSL"
        );
    }

    #[test]
    fn protocol_tls_only() {
        let tls = make_tls();
        assert_eq!(resolve_security_protocol(Some(&tls), None), "SSL");
    }

    #[cfg(feature = "aws")]
    #[test]
    fn protocol_aws_msk_without_tls() {
        let auth = make_aws_msk_auth();
        assert_eq!(resolve_security_protocol(None, Some(&auth)), "SASL_SSL");
    }

    #[test]
    fn protocol_generic_sasl_without_tls() {
        let auth = make_generic_sasl_auth();
        assert_eq!(
            resolve_security_protocol(None, Some(&auth)),
            "SASL_PLAINTEXT"
        );
    }

    #[test]
    fn protocol_plain_sasl_without_tls() {
        let auth = make_plain_auth();
        assert_eq!(
            resolve_security_protocol(None, Some(&auth)),
            "SASL_PLAINTEXT"
        );
    }

    #[test]
    fn protocol_scram_sasl_with_tls() {
        let tls = make_tls();
        let auth = make_scram_sha512_auth();
        assert_eq!(
            resolve_security_protocol(Some(&tls), Some(&auth)),
            "SASL_SSL"
        );
    }

    #[test]
    fn protocol_no_tls_no_auth() {
        assert_eq!(resolve_security_protocol(None, None), "PLAINTEXT");
    }

    // ── apply_sasl_config ───────────────────────────────────

    #[cfg(feature = "aws")]
    #[test]
    fn sasl_config_sets_oauthbearer_for_aws_msk() {
        let auth = make_aws_msk_auth();
        let mut config = ClientConfig::new();
        apply_sasl_config(Some(&auth), &mut config);

        assert_eq!(config.get("sasl.mechanism"), Some("OAUTHBEARER"));
        // Debug logging is now configured via the first-class `debug` field,
        // not automatically injected by apply_sasl_config.
        assert_eq!(config.get("debug"), None);
    }

    #[test]
    fn sasl_config_sets_plain_mechanism_and_credentials() {
        let auth = make_plain_auth();
        let mut config = ClientConfig::new();
        apply_sasl_config(Some(&auth), &mut config);

        assert_eq!(config.get("sasl.mechanism"), Some("PLAIN"));
        assert_eq!(config.get("sasl.username"), Some("myuser"));
        assert_eq!(config.get("sasl.password"), Some("mypass"));
        assert_eq!(config.get("debug"), None);
    }

    #[test]
    fn sasl_config_sets_scram_sha256_mechanism_and_credentials() {
        let auth = make_scram_sha256_auth();
        let mut config = ClientConfig::new();
        apply_sasl_config(Some(&auth), &mut config);

        assert_eq!(config.get("sasl.mechanism"), Some("SCRAM-SHA-256"));
        assert_eq!(config.get("sasl.username"), Some("myuser"));
        assert_eq!(config.get("sasl.password"), Some("mypass"));
    }

    #[test]
    fn sasl_config_sets_scram_sha512_mechanism_and_credentials() {
        let auth = make_scram_sha512_auth();
        let mut config = ClientConfig::new();
        apply_sasl_config(Some(&auth), &mut config);

        assert_eq!(config.get("sasl.mechanism"), Some("SCRAM-SHA-512"));
        assert_eq!(config.get("sasl.username"), Some("myuser"));
        assert_eq!(config.get("sasl.password"), Some("mypass"));
    }

    #[test]
    fn sasl_config_sets_mechanism_without_credentials_for_generic_sasl() {
        // A SaslAuth constructed via `new()` with no username/password.
        let auth = make_generic_sasl_auth();
        let mut config = ClientConfig::new();
        apply_sasl_config(Some(&auth), &mut config);

        assert_eq!(config.get("sasl.mechanism"), Some("PLAIN"));
        assert_eq!(config.get("sasl.username"), None);
        assert_eq!(config.get("sasl.password"), None);
    }

    #[test]
    fn sasl_config_noop_when_none() {
        let mut config = ClientConfig::new();
        apply_sasl_config(None, &mut config);

        assert_eq!(config.get("sasl.mechanism"), None);
    }

    // ── build_aws_msk_context ───────────────────────────────

    #[cfg(feature = "aws")]
    #[test]
    fn aws_msk_context_from_aws_auth() {
        let auth = make_aws_msk_auth();
        assert!(build_aws_msk_context(Some(&auth)).is_some());
    }

    #[cfg(feature = "aws")]
    #[test]
    fn aws_msk_context_none_for_generic_sasl() {
        let auth = make_generic_sasl_auth();
        assert!(build_aws_msk_context(Some(&auth)).is_none());
    }

    #[cfg(feature = "aws")]
    #[test]
    fn aws_msk_context_none_for_plain_auth() {
        let auth = make_plain_auth();
        assert!(build_aws_msk_context(Some(&auth)).is_none());
    }

    #[cfg(feature = "aws")]
    #[test]
    fn aws_msk_context_none_when_no_auth() {
        assert!(build_aws_msk_context(None).is_none());
    }

    #[cfg(feature = "aws")]
    #[test]
    fn aws_msk_context_none_for_plain_with_aws_msk_block() {
        // Defense-in-depth: even if a misconfigured SaslAuth has an aws_msk
        // block with a non-MSK mechanism, the context builder must not create
        // an OAUTHBEARER context.
        let json = r#"{
            "sasl": {
                "mechanism": "PLAIN",
                "username": "user",
                "password": "pass",
                "aws_msk": { "region": "us-east-1" }
            }
        }"#;
        let auth: Auth = serde_json::from_str(json).expect("valid JSON");
        assert!(build_aws_msk_context(Some(&auth)).is_none());
    }

    // ── TlsConfig::apply_to_client_config ───────────────────

    #[test]
    fn tls_applies_cert_paths() {
        let tls = make_tls();
        let mut config = ClientConfig::new();
        tls.apply_to_client_config(&mut config);

        assert_eq!(config.get("ssl.ca.location"), Some("ca.pem"));
        assert_eq!(config.get("ssl.certificate.location"), Some("cert.pem"));
        assert_eq!(config.get("ssl.key.location"), Some("key.pem"));
        assert_eq!(config.get("ssl.key.password"), None);
        assert_eq!(
            config.get("enable.ssl.certificate.verification"),
            Some("true")
        );
    }

    #[test]
    fn tls_insecure_disables_verification() {
        let tls = TlsConfig::new(
            "ca.pem".to_string(),
            "cert.pem".to_string(),
            "key.pem".to_string(),
            None,
            true,
        );
        let mut config = ClientConfig::new();
        tls.apply_to_client_config(&mut config);

        assert_eq!(
            config.get("enable.ssl.certificate.verification"),
            Some("false")
        );
    }

    #[test]
    fn tls_ca_only_skips_cert_and_key() {
        let tls = TlsConfig {
            ca_file: Some("ca.pem".to_string()),
            cert_file: None,
            key_file: None,
            key_password: None,
            insecure: false,
        };
        let mut config = ClientConfig::new();
        tls.apply_to_client_config(&mut config);

        assert_eq!(config.get("ssl.ca.location"), Some("ca.pem"));
        assert_eq!(config.get("ssl.certificate.location"), None);
        assert_eq!(config.get("ssl.key.location"), None);
        assert_eq!(
            config.get("enable.ssl.certificate.verification"),
            Some("true")
        );
    }

    #[test]
    fn tls_empty_block_enables_ssl_only() {
        let tls = TlsConfig {
            ca_file: None,
            cert_file: None,
            key_file: None,
            key_password: None,
            insecure: false,
        };
        let mut config = ClientConfig::new();
        tls.apply_to_client_config(&mut config);

        assert_eq!(config.get("ssl.ca.location"), None);
        assert_eq!(config.get("ssl.certificate.location"), None);
        assert_eq!(config.get("ssl.key.location"), None);
        assert_eq!(
            config.get("enable.ssl.certificate.verification"),
            Some("true")
        );
    }

    #[test]
    fn tls_key_password_is_set_when_present() {
        let tls = TlsConfig {
            ca_file: Some("ca.pem".to_string()),
            cert_file: Some("cert.pem".to_string()),
            key_file: Some("key.pem".to_string()),
            key_password: Some("secret".to_string()),
            insecure: false,
        };
        let mut config = ClientConfig::new();
        tls.apply_to_client_config(&mut config);

        assert_eq!(config.get("ssl.key.password"), Some("secret"));
    }
}
