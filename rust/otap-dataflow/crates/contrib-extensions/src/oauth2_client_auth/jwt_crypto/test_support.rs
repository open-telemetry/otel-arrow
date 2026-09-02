// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Assertions shared by the JWT backend test modules, so each backend is held
//! to the same contract.

use jsonwebtoken::crypto::CryptoProvider;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey};

/// Stands in for the `base64(header).base64(claims)` string that `jsonwebtoken`
/// actually signs.
const MESSAGE: &[u8] = b"eyJhbGciOiJSUzI1NiJ9.eyJpc3MiOiJ0ZXN0In0";

/// Every RSA algorithm the JWT-bearer grant accepts.
const SUPPORTED: [Algorithm; 3] = [Algorithm::RS256, Algorithm::RS384, Algorithm::RS512];

/// Builds a throwaway RSA keypair as `jsonwebtoken` would load it.
fn test_keys() -> (EncodingKey, DecodingKey) {
    let (private_pem, public_pem) = super::super::tests::generate_test_rsa_keypair();
    (
        EncodingKey::from_rsa_pem(private_pem.as_bytes()).expect("encoding key"),
        DecodingKey::from_rsa_pem(public_pem.as_bytes()).expect("decoding key"),
    )
}

/// Signs and verifies `MESSAGE` with `provider` under every supported
/// algorithm, and confirms a tampered signature is rejected.
pub(super) fn assert_round_trips(provider: &CryptoProvider) {
    let (encoding, decoding) = test_keys();

    for algorithm in SUPPORTED {
        let signer = (provider.signer_factory)(&algorithm, &encoding).expect("signer builds");
        assert_eq!(signer.algorithm(), algorithm);
        let signature = signer.try_sign(MESSAGE).expect("signing succeeds");

        let verifier = (provider.verifier_factory)(&algorithm, &decoding).expect("verifier builds");
        verifier
            .verify(MESSAGE, &signature)
            .unwrap_or_else(|_| panic!("{algorithm:?} signature verifies"));

        let mut tampered = signature.clone();
        tampered[0] ^= 0xff;
        assert!(
            verifier.verify(MESSAGE, &tampered).is_err(),
            "{algorithm:?} tampered signature must not verify"
        );
    }
}

/// Asserts `provider` refuses an algorithm outside the RSA PKCS#1 v1.5 set.
pub(super) fn assert_rejects_unsupported_algorithm(provider: &CryptoProvider) {
    let (encoding, decoding) = test_keys();
    assert!((provider.signer_factory)(&Algorithm::ES256, &encoding).is_err());
    assert!((provider.verifier_factory)(&Algorithm::ES256, &decoding).is_err());
}
