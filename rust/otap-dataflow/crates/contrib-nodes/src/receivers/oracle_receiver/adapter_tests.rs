// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::{
    CellValue, OracleAdapterError, OracleType, bounded_connect_string, finite_float, validate_types,
};
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

/// Scenario: An Easy Connect string omits network timeout properties.
/// Guarantees: Connection and transport timeouts are derived from the documented query timeout.
#[test]
fn adds_bounded_network_timeouts() {
    let connect_string =
        bounded_connect_string("database.contoso.com:1521/ORCL", Duration::from_secs(120))
            .expect("Easy Connect string should be supported");

    assert_eq!(
        connect_string,
        "database.contoso.com:1521/ORCL?connect_timeout=120&transport_connect_timeout=120"
    );
}
