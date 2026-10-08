// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Assertion-signing backend for the JWT-bearer grant.
//!
//! `jsonwebtoken` 11 decouples its cryptography behind a
//! [`CryptoProvider`](jsonwebtoken::crypto::CryptoProvider) rather than linking
//! `ring` unconditionally the way 9.x did. This module supplies a provider
//! backed by the same library the process already uses for TLS, selected by the
//! workspace `crypto-*` features. A deployment that mandates a particular
//! cryptographic library therefore does not get a second one linked in purely
//! to sign assertions.
//!
//! | Feature           | Selected Rustls provider |
//! |-------------------|--------------------------|
//! | `crypto-ring`     | `ring`                   |
//! | `crypto-aws-lc`   | `aws-lc-rs`              |
//! | `crypto-openssl`  | `rustls-openssl`         |
//! | `crypto-symcrypt` | `rustls-symcrypt`        |
//!
//! Every path reuses the process's selected Rustls provider, whose public
//! extension points load the key, choose the RSA signature scheme, and sign or
//! verify the unhashed JWT input. A build with no backend rejects the JWT-bearer
//! grant when the extension is constructed instead of panicking at the first
//! signature; the client-credentials grant is unaffected.
//!
//! The backends cover only the RSA PKCS#1 v1.5 algorithms the JWT-bearer grant
//! accepts (RS256, RS384, RS512), and do not support JWKs. Nothing else in the
//! collector uses `jsonwebtoken`, so the narrower algorithm set is not
//! observable elsewhere.

use std::sync::Once;

/// Provider-neutral adapter over the selected Rustls implementation. It is also
/// compiled in tests with no explicit backend, where OTAP's test fallback
/// installs Ring.
#[cfg(any(
    feature = "crypto-ring",
    feature = "crypto-aws-lc",
    feature = "crypto-openssl",
    feature = "crypto-symcrypt",
    test
))]
mod rustls_backend;

/// Assertions shared by the adapter tests.
#[cfg(test)]
mod test_support;

/// Whether this build can sign JWT-bearer assertions.
pub(super) const SIGNING_AVAILABLE: bool = cfg!(any(
    feature = "crypto-ring",
    feature = "crypto-aws-lc",
    feature = "crypto-openssl",
    feature = "crypto-symcrypt",
    test
));

/// Explanation attached to the error raised when the JWT-bearer grant is
/// configured in a build that has no assertion-signing backend.
pub(super) const NO_BACKEND_MESSAGE: &str = "this build has no JWT signing backend; the `jwt-bearer` grant requires one of the \
     `crypto-ring`, `crypto-aws-lc`, `crypto-openssl`, or `crypto-symcrypt` features";

/// Installs the assertion-signing backend as the process-wide default, at most
/// once per process.
///
/// Installation is a no-op when this build has no backend at all.
pub(super) fn ensure_provider() {
    static INIT: Once = Once::new();
    INIT.call_once(install);
}

/// Installs the provider-neutral adapter whenever a Rustls provider is
/// available. Provider selection remains centralized in OTAP.
#[cfg(any(
    feature = "crypto-ring",
    feature = "crypto-aws-lc",
    feature = "crypto-openssl",
    feature = "crypto-symcrypt",
    test
))]
fn install() {
    let _ = rustls_backend::PROVIDER.install_default();
}

/// No backend is compiled in; the JWT-bearer grant is rejected before any
/// signature is attempted.
#[cfg(not(any(
    feature = "crypto-ring",
    feature = "crypto-aws-lc",
    feature = "crypto-openssl",
    feature = "crypto-symcrypt",
    test
)))]
fn install() {}
