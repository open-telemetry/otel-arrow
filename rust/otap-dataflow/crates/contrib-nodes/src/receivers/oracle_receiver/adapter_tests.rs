// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::{
    CellValue, MAX_CREDENTIAL_BYTES, OracleAdapterError, OracleType, bounded_connect_string,
    finite_float, read_credential, validate_types,
};
use std::fs;
use std::time::Duration;

/// Scenario: Oracle returns a non-finite binary floating-point value.
/// Guarantees: Driver normalization fails the batch instead of emitting invalid OTLP data.
#[test]
fn rejects_non_finite_float() {
    assert!(matches!(
        finite_float(CellValue::Float64(f64::NAN)),
        Err(OracleAdapterError::NonFiniteFloat)
    ));
}

/// Scenario: Oracle result metadata contains a vendor type without a CellValue mapping.
/// Guarantees: Metadata validation fails explicitly instead of using a lossy fallback.
#[test]
fn rejects_unsupported_vendor_type() {
    assert!(matches!(
        validate_types(&[OracleType::BLOB]),
        Err(OracleAdapterError::UnsupportedType(_))
    ));
}

/// Scenario: An Easy Connect string uses a query timeout longer than connection establishment.
/// Guarantees: Connection and transport attempts remain bounded while query calls retain their
/// independently configured timeout.
#[test]
fn adds_bounded_network_timeouts() {
    let connect_string =
        bounded_connect_string("database.contoso.com:1521/ORCL", Duration::from_secs(120))
            .expect("Easy Connect string should be supported");

    assert_eq!(
        connect_string,
        "database.contoso.com:1521/ORCL?connect_timeout=10&transport_connect_timeout=10"
    );
}

/// Scenario: An Easy Connect string adds retries or multiple database addresses.
/// Guarantees: Connection establishment cannot multiply the fixed per-attempt startup bound.
#[test]
fn rejects_unbounded_connection_attempts() {
    for connect_string in [
        "database.contoso.com:1521/ORCL?retry_count=10",
        "database.contoso.com:1521/ORCL?retry_delay=5",
        "db1.contoso.com,db2.contoso.com:1521/ORCL",
    ] {
        assert!(bounded_connect_string(connect_string, Duration::from_secs(120)).is_err());
    }
}

/// Scenario: A mounted credential contains a trailing newline.
/// Guarantees: Kubernetes-style secret files load without adding the line ending to the credential.
#[test]
fn trims_credential_line_endings() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("password");
    fs::write(&path, b"secret\r\n").expect("write credential");

    assert_eq!(
        read_credential(path.to_str().expect("UTF-8 path"), "password")
            .expect("credential should load"),
        "secret"
    );
}

/// Scenario: A mounted credential is not valid UTF-8.
/// Guarantees: Invalid text is rejected without including credential bytes in diagnostics.
#[test]
fn rejects_non_utf8_credential() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("password");
    fs::write(&path, [0xff]).expect("write credential");

    assert!(matches!(
        read_credential(path.to_str().expect("UTF-8 path"), "password"),
        Err(OracleAdapterError::InvalidCredentialEncoding("password"))
    ));
}

/// Scenario: A mounted credential exceeds the shared credential-file size limit.
/// Guarantees: Credential loading is bounded and its error does not disclose the configured path.
#[test]
fn rejects_oversized_credential_without_disclosing_path() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("sensitive-password-path");
    fs::write(&path, vec![b'x'; MAX_CREDENTIAL_BYTES as usize + 1]).expect("write credential");

    let error = read_credential(path.to_str().expect("UTF-8 path"), "password")
        .expect_err("oversized credential should fail");
    assert!(matches!(
        error,
        OracleAdapterError::CredentialTooLarge("password")
    ));
    assert!(!error.to_string().contains("sensitive-password-path"));
}
