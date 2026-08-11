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
//! | Feature           | Assertion-signing backend                         |
//! |-------------------|---------------------------------------------------|
//! | `crypto-ring`     | `ring`, in [`ring_backend`]                       |
//! | `crypto-aws-lc`   | `aws-lc-rs`, through `jsonwebtoken`'s own provider |
//! | `crypto-openssl`  | `openssl`, in [`openssl_backend`]                 |
//! | `crypto-symcrypt` | none                                              |
//!
//! SymCrypt is absent because its Rust bindings import an RSA key only as raw
//! (modulus, exponent, prime) components, while `jsonwebtoken` hands a provider
//! a PKCS#1 DER blob. A build with no backend rejects the JWT-bearer grant when
//! the extension is constructed instead of panicking at the first signature;
//! the client-credentials grant is unaffected.
//!
//! The backends cover only the RSA PKCS#1 v1.5 algorithms the JWT-bearer grant
//! accepts (RS256, RS384, RS512), and do not support JWKs. Nothing else in the
//! collector uses `jsonwebtoken`, so the narrower algorithm set is not
//! observable elsewhere.

use std::sync::Once;

// Every enabled backend is compiled, not just the selected one, so that
// `cargo clippy --all-features` type-checks each of them and their tests run.
// That leaves the unselected backend unreferenced in a non-test
// `--all-features` build, hence the `dead_code` allowance.

/// `ring`-backed provider. Also compiled into test builds that select no
/// backend at all, so the crate's suite can sign without committing the library
/// to one; `ring` is a dev-dependency for that case.
#[cfg(any(
    feature = "crypto-ring",
    all(test, not(feature = "crypto-aws-lc"), not(feature = "crypto-openssl"))
))]
#[allow(dead_code, reason = "unselected when another crypto-* feature wins")]
mod ring_backend;

/// `openssl`-backed provider.
#[cfg(feature = "crypto-openssl")]
#[allow(dead_code, reason = "unselected when another crypto-* feature wins")]
mod openssl_backend;

/// Assertions shared by the backend test modules. Compiled only when at least
/// one backend exists to exercise.
#[cfg(all(
    test,
    any(
        feature = "crypto-ring",
        feature = "crypto-openssl",
        not(feature = "crypto-aws-lc")
    )
))]
mod test_support;

/// Whether this build can sign JWT-bearer assertions.
pub(super) const SIGNING_AVAILABLE: bool = cfg!(any(
    feature = "crypto-ring",
    feature = "crypto-aws-lc",
    feature = "crypto-openssl",
    test
));

/// Explanation attached to the error raised when the JWT-bearer grant is
/// configured in a build that has no assertion-signing backend.
pub(super) const NO_BACKEND_MESSAGE: &str = "this build has no JWT signing backend; the `jwt-bearer` grant requires one of the \
     `crypto-ring`, `crypto-aws-lc`, or `crypto-openssl` features";

/// Installs the assertion-signing backend as the process-wide default, at most
/// once per process.
///
/// Installation is a no-op when `jsonwebtoken` already selects a backend from
/// its own crate features, and when this build has no backend at all.
pub(super) fn ensure_provider() {
    static INIT: Once = Once::new();
    INIT.call_once(install);
}

// Selection order mirrors `otap_df_otap::crypto::install_crypto_provider`, so a
// build that enables several `crypto-*` features signs with the same library it
// uses for TLS. The features are documented as mutually exclusive, but CI builds
// with `--all-features`, so an order has to be defined.

/// `ring` is selected when `crypto-ring` is on, and in test builds that pick no
/// backend at all.
#[cfg(any(
    feature = "crypto-ring",
    all(test, not(feature = "crypto-aws-lc"), not(feature = "crypto-openssl"))
))]
fn install() {
    // `install_default` reports an error only when a provider is already
    // installed, which is benign: the first caller wins and both would have
    // installed the same backend.
    let _ = ring_backend::PROVIDER.install_default();
}

/// `jsonwebtoken` resolves its bundled `aws-lc-rs` provider from the
/// `aws_lc_rs` crate feature that `crypto-aws-lc` turns on, so there is nothing
/// to install here.
#[cfg(all(feature = "crypto-aws-lc", not(feature = "crypto-ring")))]
fn install() {}

/// `openssl` is selected when it is the only backend enabled.
#[cfg(all(
    feature = "crypto-openssl",
    not(feature = "crypto-ring"),
    not(feature = "crypto-aws-lc")
))]
fn install() {
    let _ = openssl_backend::PROVIDER.install_default();
}

/// No backend is compiled in; the JWT-bearer grant is rejected before any
/// signature is attempted.
#[cfg(not(any(
    feature = "crypto-ring",
    feature = "crypto-aws-lc",
    feature = "crypto-openssl",
    test
)))]
fn install() {}
