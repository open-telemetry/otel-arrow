// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![allow(missing_docs)]

use otap_df_config::tls::TlsConfig;

#[test]
fn test_tls_config_defaults() {
    let yaml = r#"
    cert_file: /tmp/cert.pem
    key_file: /tmp/key.pem
    "#;

    let config: TlsConfig = serde_yaml::from_str(yaml).expect("Failed to parse YAML");

    assert_eq!(config.reload_interval, Some("5m".to_string()));
}

#[test]
fn test_tls_config_explicit_value() {
    let yaml = r#"
    cert_file: /tmp/cert.pem
    key_file: /tmp/key.pem
    reload_interval: 10s
    "#;

    let config: TlsConfig = serde_yaml::from_str(yaml).expect("Failed to parse YAML");

    assert_eq!(config.reload_interval, Some("10s".to_string()));
}

#[test]
fn test_tls_config_explicit_null() {
    let yaml = r#"
    cert_file: /tmp/cert.pem
    key_file: /tmp/key.pem
    reload_interval: null
    "#;

    let config: TlsConfig = serde_yaml::from_str(yaml).expect("Failed to parse YAML");

    assert_eq!(config.reload_interval, None);
}
