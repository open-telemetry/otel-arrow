// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! `ring`-backed JWT signing, selected by the `crypto-ring` feature.

use jsonwebtoken::crypto::{CryptoProvider, JwtSigner, JwtVerifier, KeyUtils};
use jsonwebtoken::errors::{ErrorKind, Result as JwtResult};
use jsonwebtoken::signature::{Error as SignatureError, Signer, Verifier};
use jsonwebtoken::{Algorithm, DecodingKey, DecodingKeyKind, EncodingKey};
use ring::rand::SystemRandom;
use ring::signature::{
    RSA_PKCS1_2048_8192_SHA256, RSA_PKCS1_2048_8192_SHA384, RSA_PKCS1_2048_8192_SHA512,
    RSA_PKCS1_SHA256, RSA_PKCS1_SHA384, RSA_PKCS1_SHA512, RsaEncoding, RsaKeyPair, RsaParameters,
    UnparsedPublicKey,
};

/// Provider covering RS256, RS384 and RS512.
pub(super) static PROVIDER: CryptoProvider = CryptoProvider {
    signer_factory,
    verifier_factory,
    key_utils: KeyUtils::new_unimplemented(),
};

/// Signs with an RSA private key using PKCS#1 v1.5 padding.
struct RsaSigner {
    algorithm: Algorithm,
    key_pair: RsaKeyPair,
    encoding: &'static dyn RsaEncoding,
}

impl Signer<Vec<u8>> for RsaSigner {
    fn try_sign(&self, message: &[u8]) -> Result<Vec<u8>, SignatureError> {
        let mut signature = vec![0u8; self.key_pair.public().modulus_len()];
        self.key_pair
            .sign(self.encoding, &SystemRandom::new(), message, &mut signature)
            .map_err(|_| SignatureError::new())?;
        Ok(signature)
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
    parameters: &'static RsaParameters,
    public_key: Vec<u8>,
}

impl Verifier<Vec<u8>> for RsaVerifier {
    fn verify(&self, message: &[u8], signature: &Vec<u8>) -> Result<(), SignatureError> {
        UnparsedPublicKey::new(self.parameters, &self.public_key)
            .verify(message, signature)
            .map_err(|_| SignatureError::new())
    }
}

impl JwtVerifier for RsaVerifier {
    fn algorithm(&self) -> Algorithm {
        self.algorithm
    }
}

fn signer_factory(algorithm: &Algorithm, key: &EncodingKey) -> JwtResult<Box<dyn JwtSigner>> {
    let encoding: &'static dyn RsaEncoding = match algorithm {
        Algorithm::RS256 => &RSA_PKCS1_SHA256,
        Algorithm::RS384 => &RSA_PKCS1_SHA384,
        Algorithm::RS512 => &RSA_PKCS1_SHA512,
        _ => return Err(ErrorKind::InvalidAlgorithm.into()),
    };
    // `EncodingKey::from_rsa_pem` normalizes both PKCS#1 and PKCS#8 input to a
    // PKCS#1 `RSAPrivateKey`, which is what `from_der` expects.
    let key_pair = RsaKeyPair::from_der(key.as_bytes())
        .map_err(|e| ErrorKind::InvalidRsaKey(e.to_string()))?;
    Ok(Box::new(RsaSigner {
        algorithm: *algorithm,
        key_pair,
        encoding,
    }))
}

fn verifier_factory(algorithm: &Algorithm, key: &DecodingKey) -> JwtResult<Box<dyn JwtVerifier>> {
    let parameters: &'static RsaParameters = match algorithm {
        Algorithm::RS256 => &RSA_PKCS1_2048_8192_SHA256,
        Algorithm::RS384 => &RSA_PKCS1_2048_8192_SHA384,
        Algorithm::RS512 => &RSA_PKCS1_2048_8192_SHA512,
        _ => return Err(ErrorKind::InvalidAlgorithm.into()),
    };
    // JWK-sourced keys arrive as raw components. They are unreachable here
    // because `key_utils` is unimplemented, so only DER keys are accepted.
    let DecodingKeyKind::SecretOrDer(der) = key.kind() else {
        return Err(ErrorKind::InvalidKeyFormat.into());
    };
    Ok(Box::new(RsaVerifier {
        algorithm: *algorithm,
        parameters,
        public_key: der.clone(),
    }))
}

#[cfg(test)]
mod tests {
    use super::PROVIDER;

    // Scenario: the ring-backed provider signs an assertion payload under RS256,
    // RS384 and RS512, verifies each signature, and re-checks a tampered one.
    // Guarantees: selecting `crypto-ring` produces signatures that verify
    // against the matching public key and reject modified ones.
    #[test]
    fn signs_and_verifies_every_supported_algorithm() {
        super::super::test_support::assert_round_trips(&PROVIDER);
    }

    // Scenario: the ring-backed provider is asked for an ECDSA algorithm.
    // Guarantees: unsupported algorithms surface an error instead of producing
    // a signature the token endpoint would reject.
    #[test]
    fn rejects_an_unsupported_algorithm() {
        super::super::test_support::assert_rejects_unsupported_algorithm(&PROVIDER);
    }
}
