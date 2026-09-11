// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Compatibility tests with the Go Kafka receiver config surface.

use super::*;

// ---- Compatibility ----

/// Scenario (compatibility): the TLS getter is read on a default config.
/// Guarantees: it returns none, so TLS is off unless configured.
#[test]
fn tls_getter_returns_none_by_default() {
    let cfg: KafkaReceiverConfig = KafkaReceiverConfigBuilder::new("b", "g", "c")
        .with_traces(SignalConfig {
            topics: vec!["t".to_string()],
            ..Default::default()
        })
        .try_into()
        .unwrap();
    assert!(cfg.tls().is_none());
}

/// Scenario (compatibility): TLS is configured via the builder.
/// Guarantees: the TLS block is applied, so the builder configures TLS.
#[test]
fn with_tls_builder_method() {
    let tls = TlsConfig::new(
        "/certs/ca.pem".to_string(),
        "/certs/client.pem".to_string(),
        "/certs/client-key.pem".to_string(),
        None,
        false,
    );
    let cfg: KafkaReceiverConfig = KafkaReceiverConfigBuilder::new("b", "g", "c")
        .with_traces(SignalConfig {
            topics: vec!["t".to_string()],
            ..Default::default()
        })
        .with_tls(tls)
        .try_into()
        .unwrap();
    let tls = cfg.tls().expect("tls should be set");
    assert_eq!(tls.ca_file(), Some("/certs/ca.pem"));
    assert_eq!(tls.cert_file(), Some("/certs/client.pem"));
    assert_eq!(tls.key_file(), Some("/certs/client-key.pem"));
    assert!(!tls.insecure());
}

/// Scenario (compatibility): a config with a TLS block is deserialized.
/// Guarantees: the TLS fields parse, so TLS is configurable via JSON.
#[test]
fn deserialize_config_with_tls() {
    let json = json!({
        "brokers": "kafka:9093",
        "group_id": "g",
        "client_id": "c",
        "traces": {"topics": ["traces"]},
        "tls": {
            "ca_file": "/certs/ca.pem",
            "cert_file": "/certs/client.pem",
            "key_file": "/certs/client-key.pem",
            "insecure": true
        }
    });
    let cfg: KafkaReceiverConfig =
        serde_json::from_value(json).expect("should deserialize config with tls");
    let tls = cfg.tls().expect("tls should be present");
    assert_eq!(tls.ca_file(), Some("/certs/ca.pem"));
    assert_eq!(tls.cert_file(), Some("/certs/client.pem"));
    assert_eq!(tls.key_file(), Some("/certs/client-key.pem"));
    assert!(tls.insecure());
}

/// Scenario (compatibility): a TLS block omits the insecure flag.
/// Guarantees: insecure defaults to false, so certificate verification is on unless
/// explicitly disabled.
#[test]
fn deserialize_config_with_tls_insecure_defaults_false() {
    let json = json!({
        "brokers": "kafka:9093",
        "group_id": "g",
        "client_id": "c",
        "traces": {"topics": ["traces"]},
        "tls": {
            "ca_file": "/certs/ca.pem",
            "cert_file": "/certs/client.pem",
            "key_file": "/certs/client-key.pem"
        }
    });
    let cfg: KafkaReceiverConfig =
        serde_json::from_value(json).expect("should deserialize config with tls");
    let tls = cfg.tls().expect("tls should be present");
    assert!(!tls.insecure());
}

/// Scenario (compatibility): a client config is built with neither TLS nor auth.
/// Guarantees: `security.protocol` is PLAINTEXT, so an unsecured connection is used
/// only when nothing is configured.
#[test]
fn build_client_config_no_tls_no_auth_sets_plaintext() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c");
    let client_config = cfg.build_client_config();
    assert_eq!(client_config.get("security.protocol"), Some("PLAINTEXT"));
    assert_eq!(client_config.get("ssl.ca.location"), None);
}

/// Scenario (compatibility): a client config is built with TLS and no auth.
/// Guarantees: `security.protocol` is SSL and the `ssl.*` properties are set, so
/// server-only TLS is configured.
#[test]
fn build_client_config_tls_only_sets_ssl_protocol() {
    let tls = TlsConfig::new(
        "/certs/ca.pem".to_string(),
        "/certs/client.pem".to_string(),
        "/certs/client-key.pem".to_string(),
        None,
        false,
    );
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_tls(tls);
    let client_config = cfg.build_client_config();
    assert_eq!(client_config.get("security.protocol"), Some("SSL"));
    assert_eq!(client_config.get("ssl.ca.location"), Some("/certs/ca.pem"));
    assert_eq!(
        client_config.get("ssl.certificate.location"),
        Some("/certs/client.pem")
    );
    assert_eq!(
        client_config.get("ssl.key.location"),
        Some("/certs/client-key.pem")
    );
    assert_eq!(
        client_config.get("enable.ssl.certificate.verification"),
        Some("true")
    );
}

/// Scenario (compatibility): TLS is configured with insecure = true.
/// Guarantees: certificate verification is disabled, so insecure TLS is honored when
/// explicitly requested.
#[test]
fn build_client_config_tls_insecure_disables_verification() {
    let tls = TlsConfig::new(
        "/certs/ca.pem".to_string(),
        "/certs/client.pem".to_string(),
        "/certs/client-key.pem".to_string(),
        None,
        true,
    );
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_tls(tls);
    let client_config = cfg.build_client_config();
    assert_eq!(
        client_config.get("enable.ssl.certificate.verification"),
        Some("false")
    );
}

/// Scenario (compatibility): a client config is built with TLS and SASL auth.
/// Guarantees: `security.protocol` is SASL_SSL, so SASL-over-TLS is configured.
#[test]
#[cfg(feature = "aws")]
fn build_client_config_tls_with_sasl_sets_sasl_ssl_protocol() {
    let json = json!({
        "brokers": "kafka:9093",
        "group_id": "g",
        "client_id": "c",
        "traces": {"topics": ["t"]},
        "auth": {
            "sasl": {
                "mechanism": "AWS_MSK_IAM_OAUTHBEARER",
                "aws_msk": {"region": "us-east-1"}
            }
        },
        "tls": {
            "ca_file": "/certs/ca.pem",
            "cert_file": "/certs/client.pem",
            "key_file": "/certs/client-key.pem"
        }
    });
    let cfg: KafkaReceiverConfig = serde_json::from_value(json).unwrap();
    let client_config = cfg.build_client_config();
    assert_eq!(client_config.get("security.protocol"), Some("SASL_SSL"));
    assert_eq!(client_config.get("ssl.ca.location"), Some("/certs/ca.pem"));
    assert_eq!(client_config.get("sasl.mechanism"), Some("OAUTHBEARER"));
}

/// Scenario (compatibility): AWS MSK IAM auth is configured without an explicit TLS
/// block.
/// Guarantees: `security.protocol` is SASL_SSL, so MSK IAM implies TLS.
#[test]
#[cfg(feature = "aws")]
fn build_client_config_aws_msk_without_tls_sets_sasl_ssl() {
    let json = json!({
        "brokers": "kafka:9093",
        "group_id": "g",
        "client_id": "c",
        "traces": {"topics": ["t"]},
        "auth": {
            "sasl": {
                "mechanism": "AWS_MSK_IAM_OAUTHBEARER",
                "aws_msk": {"region": "us-east-1"}
            }
        }
    });
    let cfg: KafkaReceiverConfig = serde_json::from_value(json).unwrap();
    let client_config = cfg.build_client_config();
    assert_eq!(client_config.get("security.protocol"), Some("SASL_SSL"));
    assert_eq!(client_config.get("sasl.mechanism"), Some("OAUTHBEARER"));
    // No TLS config set, so SSL cert fields should not be present
    assert_eq!(client_config.get("ssl.ca.location"), None);
}

/// Scenario (compatibility): generic SASL auth is configured without TLS.
/// Guarantees: `security.protocol` is SASL_PLAINTEXT, so plaintext SASL is used when no
/// TLS is requested.
#[test]
fn build_client_config_sasl_without_msk_and_no_tls_sets_sasl_plaintext() {
    let json = json!({
        "brokers": "kafka:9092",
        "group_id": "g",
        "client_id": "c",
        "traces": {"topics": ["t"]},
        "auth": {
            "sasl": {
                "mechanism": "PLAIN",
                "username": "user",
                "password": "pass"
            }
        }
    });
    let cfg: KafkaReceiverConfig = serde_json::from_value(json).unwrap();
    let client_config = cfg.build_client_config();
    assert_eq!(
        client_config.get("security.protocol"),
        Some("SASL_PLAINTEXT")
    );
    assert_eq!(client_config.get("sasl.mechanism"), Some("PLAIN"));
    assert_eq!(client_config.get("sasl.username"), Some("user"));
    assert_eq!(client_config.get("sasl.password"), Some("pass"));
}

/// Scenario (compatibility): SASL SCRAM-SHA-256 auth is configured with no TLS.
/// Guarantees: `build_client_config` sets `security.protocol` to
/// SASL_PLAINTEXT and `sasl.mechanism` to SCRAM-SHA-256 with the supplied
/// credentials, so the SCRAM-256 mechanism in the auth matrix is wired
/// through to librdkafka correctly.
#[test]
fn build_client_config_scram_sha_256_sets_sasl_plaintext() {
    let json = json!({
        "brokers": "kafka:9092",
        "group_id": "g",
        "client_id": "c",
        "traces": {"topics": ["t"]},
        "auth": {
            "sasl": {
                "mechanism": "SCRAM-SHA-256",
                "username": "user",
                "password": "pass"
            }
        }
    });
    let cfg: KafkaReceiverConfig = serde_json::from_value(json).unwrap();
    let client_config = cfg.build_client_config();
    assert_eq!(
        client_config.get("security.protocol"),
        Some("SASL_PLAINTEXT")
    );
    assert_eq!(client_config.get("sasl.mechanism"), Some("SCRAM-SHA-256"));
    assert_eq!(client_config.get("sasl.username"), Some("user"));
    assert_eq!(client_config.get("sasl.password"), Some("pass"));
}

/// Scenario (compatibility): SASL SCRAM-SHA-512 auth is configured over TLS.
/// Guarantees: `build_client_config` sets `security.protocol` to SASL_SSL
/// (SASL over TLS) and `sasl.mechanism` to SCRAM-SHA-512 alongside the TLS
/// CA path, so the SCRAM-512 mechanism combined with TLS is wired through
/// correctly.
#[test]
fn build_client_config_scram_sha_512_over_tls_sets_sasl_ssl() {
    let json = json!({
        "brokers": "kafka:9093",
        "group_id": "g",
        "client_id": "c",
        "traces": {"topics": ["t"]},
        "tls": {"ca_file": "/certs/ca.pem"},
        "auth": {
            "sasl": {
                "mechanism": "SCRAM-SHA-512",
                "username": "user",
                "password": "pass"
            }
        }
    });
    let cfg: KafkaReceiverConfig = serde_json::from_value(json).unwrap();
    let client_config = cfg.build_client_config();
    assert_eq!(client_config.get("security.protocol"), Some("SASL_SSL"));
    assert_eq!(client_config.get("sasl.mechanism"), Some("SCRAM-SHA-512"));
    assert_eq!(client_config.get("sasl.username"), Some("user"));
    assert_eq!(client_config.get("sasl.password"), Some("pass"));
    assert_eq!(client_config.get("ssl.ca.location"), Some("/certs/ca.pem"));
}

/// Scenario (compatibility): mutual TLS is configured with a CA, client
/// certificate, client key, and an encrypted-key password.
/// Guarantees: `build_client_config` sets `security.protocol` to SSL and
/// wires all four mTLS paths -- including `ssl.key.password` -- so an
/// encrypted client key can be used for mutual TLS.
#[test]
fn build_client_config_mtls_sets_key_password() {
    let tls = TlsConfig::new(
        "/certs/ca.pem".to_string(),
        "/certs/client.pem".to_string(),
        "/certs/client-key.pem".to_string(),
        Some("key-secret".to_string()),
        false,
    );
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_tls(tls);
    let client_config = cfg.build_client_config();
    assert_eq!(client_config.get("security.protocol"), Some("SSL"));
    assert_eq!(client_config.get("ssl.ca.location"), Some("/certs/ca.pem"));
    assert_eq!(
        client_config.get("ssl.certificate.location"),
        Some("/certs/client.pem")
    );
    assert_eq!(
        client_config.get("ssl.key.location"),
        Some("/certs/client-key.pem")
    );
    assert_eq!(client_config.get("ssl.key.password"), Some("key-secret"));
}
