// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! `openssl`-backed JWT signing, selected by the `crypto-openssl` feature.

use jsonwebtoken::crypto::{CryptoProvider, JwtSigner, JwtVerifier, KeyUtils};
use jsonwebtoken::errors::{ErrorKind, Result as JwtResult};
use jsonwebtoken::signature::{Error as SignatureError, Signer, Verifier};
use jsonwebtoken::{Algorithm, DecodingKey, DecodingKeyKind, EncodingKey};
use openssl::hash::MessageDigest;
use openssl::pkey::{PKey, Private, Public};
use openssl::rsa::Rsa;
use openssl::sign::{Signer as OpensslSigner, Verifier as OpensslVerifier};

/// Provider covering RS256, RS384 and RS512.
pub(super) static PROVIDER: CryptoProvider = CryptoProvider {
    signer_factory,
    verifier_factory,
    key_utils: KeyUtils::new_unimplemented(),
};

/// Maps a JWT algorithm to the digest it is defined over. `openssl`'s default
/// RSA padding is PKCS#1 v1.5, which is what RS* requires.
fn digest_for(algorithm: &Algorithm) -> JwtResult<MessageDigest> {
    match algorithm {
        Algorithm::RS256 => Ok(MessageDigest::sha256()),
        Algorithm::RS384 => Ok(MessageDigest::sha384()),
        Algorithm::RS512 => Ok(MessageDigest::sha512()),
        _ => Err(ErrorKind::InvalidAlgorithm.into()),
    }
}

/// Signs with an RSA private key using PKCS#1 v1.5 padding.
struct RsaSigner {
    algorithm: Algorithm,
    key: PKey<Private>,
    digest: MessageDigest,
}

impl Signer<Vec<u8>> for RsaSigner {
    fn try_sign(&self, message: &[u8]) -> Result<Vec<u8>, SignatureError> {
        let mut signer =
            OpensslSigner::new(self.digest, &self.key).map_err(|_| SignatureError::new())?;
        signer.update(message).map_err(|_| SignatureError::new())?;
        signer.sign_to_vec().map_err(|_| SignatureError::new())
    }
}

impl JwtSigner for RsaSigner {
    fn algorithm(&self) -> Algorithm {
        self.algorithm
    }
}

/// Verifies an RSA PKCS#1 v1.5 signature against a public key.
struct RsaVerifier {
    algorithm: Algorithm,
    key: PKey<Public>,
    digest: MessageDigest,
}

impl Verifier<Vec<u8>> for RsaVerifier {
    fn verify(&self, message: &[u8], signature: &Vec<u8>) -> Result<(), SignatureError> {
        let mut verifier =
            OpensslVerifier::new(self.digest, &self.key).map_err(|_| SignatureError::new())?;
        verifier
            .update(message)
            .map_err(|_| SignatureError::new())?;
        match verifier.verify(signature) {
            Ok(true) => Ok(()),
            _ => Err(SignatureError::new()),
        }
    }
}

impl JwtVerifier for RsaVerifier {
    fn algorithm(&self) -> Algorithm {
        self.algorithm
    }
}

fn signer_factory(algorithm: &Algorithm, key: &EncodingKey) -> JwtResult<Box<dyn JwtSigner>> {
    let digest = digest_for(algorithm)?;
    // `EncodingKey::from_rsa_pem` normalizes both PKCS#1 and PKCS#8 input to a
    // PKCS#1 `RSAPrivateKey`.
    let rsa = Rsa::private_key_from_der(key.as_bytes())
        .map_err(|e| ErrorKind::InvalidRsaKey(e.to_string()))?;
    let key = PKey::from_rsa(rsa).map_err(|e| ErrorKind::InvalidRsaKey(e.to_string()))?;
    Ok(Box::new(RsaSigner {
        algorithm: *algorithm,
        key,
        digest,
    }))
}

fn verifier_factory(algorithm: &Algorithm, key: &DecodingKey) -> JwtResult<Box<dyn JwtVerifier>> {
    let digest = digest_for(algorithm)?;
    // JWK-sourced keys arrive as raw components. They are unreachable here
    // because `key_utils` is unimplemented, so only DER keys are accepted.
    let DecodingKeyKind::SecretOrDer(der) = key.kind() else {
        return Err(ErrorKind::InvalidKeyFormat.into());
    };
    let rsa =
        Rsa::public_key_from_der_pkcs1(der).map_err(|e| ErrorKind::InvalidRsaKey(e.to_string()))?;
    let key = PKey::from_rsa(rsa).map_err(|e| ErrorKind::InvalidRsaKey(e.to_string()))?;
    Ok(Box::new(RsaVerifier {
        algorithm: *algorithm,
        key,
        digest,
    }))
}

#[cfg(test)]
mod tests {
    use super::PROVIDER;

    // Scenario: the openssl-backed provider signs an assertion payload under
    // RS256, RS384 and RS512, verifies each signature, and re-checks a tampered
    // one.
    // Guarantees: selecting `crypto-openssl` produces signatures that verify
    // against the matching public key and reject modified ones.
    #[test]
    fn signs_and_verifies_every_supported_algorithm() {
        super::super::test_support::assert_round_trips(&PROVIDER);
    }

    // Scenario: the openssl-backed provider is asked for an ECDSA algorithm.
    // Guarantees: unsupported algorithms surface an error instead of producing
    // a signature the token endpoint would reject.
    #[test]
    fn rejects_an_unsupported_algorithm() {
        super::super::test_support::assert_rejects_unsupported_algorithm(&PROVIDER);
    }
}
