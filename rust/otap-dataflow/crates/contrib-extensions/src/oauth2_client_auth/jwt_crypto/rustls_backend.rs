// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Provider-neutral JWT signing through the selected Rustls crypto provider.

use jsonwebtoken::crypto::{CryptoProvider, JwtSigner, JwtVerifier, KeyUtils};
use jsonwebtoken::errors::{ErrorKind, Result as JwtResult};
use jsonwebtoken::signature::{Error as SignatureError, Signer, Verifier};
use jsonwebtoken::{Algorithm, DecodingKey, DecodingKeyKind, EncodingKey};
use rustls::SignatureScheme;
use rustls::pki_types::{PrivateKeyDer, PrivatePkcs1KeyDer, SignatureVerificationAlgorithm};

/// Provider covering RS256, RS384 and RS512 through the selected Rustls backend.
pub(super) static PROVIDER: CryptoProvider = CryptoProvider {
    signer_factory,
    verifier_factory,
    key_utils: KeyUtils::new_unimplemented(),
};

fn signature_scheme(algorithm: &Algorithm) -> JwtResult<SignatureScheme> {
    match algorithm {
        Algorithm::RS256 => Ok(SignatureScheme::RSA_PKCS1_SHA256),
        Algorithm::RS384 => Ok(SignatureScheme::RSA_PKCS1_SHA384),
        Algorithm::RS512 => Ok(SignatureScheme::RSA_PKCS1_SHA512),
        _ => Err(ErrorKind::InvalidAlgorithm.into()),
    }
}

struct RsaSigner {
    algorithm: Algorithm,
    signer: Box<dyn rustls::sign::Signer>,
}

impl Signer<Vec<u8>> for RsaSigner {
    fn try_sign(&self, message: &[u8]) -> Result<Vec<u8>, SignatureError> {
        self.signer.sign(message).map_err(|_| SignatureError::new())
    }
}

impl JwtSigner for RsaSigner {
    fn algorithm(&self) -> Algorithm {
        self.algorithm
    }
}

struct RsaVerifier {
    algorithm: Algorithm,
    verification_algorithms: &'static [&'static dyn SignatureVerificationAlgorithm],
    public_key: Vec<u8>,
}

impl Verifier<Vec<u8>> for RsaVerifier {
    fn verify(&self, message: &[u8], signature: &Vec<u8>) -> Result<(), SignatureError> {
        if self.verification_algorithms.iter().any(|algorithm| {
            algorithm
                .verify_signature(&self.public_key, message, signature)
                .is_ok()
        }) {
            Ok(())
        } else {
            Err(SignatureError::new())
        }
    }
}

impl JwtVerifier for RsaVerifier {
    fn algorithm(&self) -> Algorithm {
        self.algorithm
    }
}

fn signer_factory(algorithm: &Algorithm, key: &EncodingKey) -> JwtResult<Box<dyn JwtSigner>> {
    let scheme = signature_scheme(algorithm)?;
    let provider = otel_arrow_dfe_otap::crypto::selected_crypto_provider();
    let private_key = PrivateKeyDer::from(PrivatePkcs1KeyDer::from(key.as_bytes().to_vec()));
    let signing_key = provider
        .key_provider
        .load_private_key(private_key)
        .map_err(|error| ErrorKind::InvalidRsaKey(error.to_string()))?;
    let signer = signing_key
        .choose_scheme(&[scheme])
        .ok_or(ErrorKind::InvalidAlgorithm)?;

    Ok(Box::new(RsaSigner {
        algorithm: *algorithm,
        signer,
    }))
}

fn verifier_factory(algorithm: &Algorithm, key: &DecodingKey) -> JwtResult<Box<dyn JwtVerifier>> {
    let scheme = signature_scheme(algorithm)?;
    let DecodingKeyKind::SecretOrDer(public_key) = key.kind() else {
        return Err(ErrorKind::InvalidKeyFormat.into());
    };
    let provider = otel_arrow_dfe_otap::crypto::selected_crypto_provider();
    let verification_algorithms = provider
        .signature_verification_algorithms
        .mapping
        .iter()
        .find_map(|(candidate, algorithms)| (*candidate == scheme).then_some(*algorithms))
        .ok_or(ErrorKind::InvalidAlgorithm)?;

    Ok(Box::new(RsaVerifier {
        algorithm: *algorithm,
        verification_algorithms,
        public_key: public_key.clone(),
    }))
}

#[cfg(test)]
mod tests {
    use jsonwebtoken::errors::ErrorKind;
    use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey};
    use ring::rand::SystemRandom;
    use ring::signature::{
        RSA_PKCS1_2048_8192_SHA256, RSA_PKCS1_2048_8192_SHA384, RSA_PKCS1_2048_8192_SHA512,
        RSA_PKCS1_SHA256, RSA_PKCS1_SHA384, RSA_PKCS1_SHA512, RsaEncoding, RsaKeyPair,
        RsaParameters, UnparsedPublicKey,
    };

    use super::{PROVIDER, signer_factory, verifier_factory};

    const MESSAGE: &[u8] = b"independently verified JWT signing input";

    fn ensure_test_crypto_provider() {
        otel_arrow_dfe_otap::crypto::ensure_crypto_provider();
    }

    fn test_keys() -> (EncodingKey, DecodingKey) {
        let (private_pem, public_pem) = super::super::super::tests::generate_test_rsa_keypair();
        (
            EncodingKey::from_rsa_pem(private_pem.as_bytes()).expect("encoding key"),
            DecodingKey::from_rsa_pem(public_pem.as_bytes()).expect("decoding key"),
        )
    }

    /// Scenario: The Rustls-backed provider signs an assertion payload under
    /// RS256, RS384 and RS512, verifies each signature, and rejects tampering.
    /// Guarantees: Every selectable Rustls provider supports each JWT RSA
    /// algorithm exposed by the extension through the same adapter.
    #[test]
    fn signs_and_verifies_every_supported_algorithm() {
        ensure_test_crypto_provider();
        super::super::test_support::assert_round_trips(&PROVIDER);
    }

    /// Scenario: The Rustls-backed provider is asked for an ECDSA algorithm.
    /// Guarantees: Unsupported algorithms surface an error instead of producing
    /// a signature the token endpoint would reject.
    #[test]
    fn rejects_an_unsupported_algorithm() {
        super::super::test_support::assert_rejects_unsupported_algorithm(&PROVIDER);
    }

    /// Scenario: The Rustls key provider receives malformed RSA private-key DER.
    /// Guarantees: Provider key-loading failures retain jsonwebtoken's
    /// InvalidRsaKey error surface rather than becoming a signing failure.
    #[test]
    fn maps_private_key_loading_failures_to_invalid_rsa_key() {
        ensure_test_crypto_provider();
        let key = EncodingKey::from_rsa_der(b"not a DER private key");
        let error = match signer_factory(&Algorithm::RS256, &key) {
            Ok(_) => panic!("malformed private key must be rejected"),
            Err(error) => error,
        };
        assert!(matches!(error.kind(), ErrorKind::InvalidRsaKey(_)));
    }

    /// Scenario: The Rustls verification algorithm receives malformed RSA
    /// public-key DER.
    /// Guarantees: Invalid public-key encodings cannot verify a signature.
    #[test]
    fn rejects_malformed_public_key_der() {
        ensure_test_crypto_provider();
        let key = DecodingKey::from_rsa_der(b"not a DER public key");
        let verifier =
            verifier_factory(&Algorithm::RS256, &key).expect("verifier construction succeeds");
        assert!(verifier.verify(MESSAGE, &vec![0; 256]).is_err());
    }

    /// Scenario: Signatures flow in both directions between the Rustls adapter
    /// and Ring for RS256, RS384 and RS512.
    /// Guarantees: The adapter interoperates with an independent implementation
    /// instead of passing only same-provider round trips.
    #[test]
    fn interoperates_with_independent_ring_vectors() {
        ensure_test_crypto_provider();
        let (encoding_key, decoding_key) = test_keys();
        let ring_key = RsaKeyPair::from_der(encoding_key.as_bytes()).expect("Ring private key");
        let random = SystemRandom::new();

        let cases: [(Algorithm, &'static dyn RsaEncoding, &'static RsaParameters); 3] = [
            (
                Algorithm::RS256,
                &RSA_PKCS1_SHA256,
                &RSA_PKCS1_2048_8192_SHA256,
            ),
            (
                Algorithm::RS384,
                &RSA_PKCS1_SHA384,
                &RSA_PKCS1_2048_8192_SHA384,
            ),
            (
                Algorithm::RS512,
                &RSA_PKCS1_SHA512,
                &RSA_PKCS1_2048_8192_SHA512,
            ),
        ];

        for (algorithm, encoding, parameters) in cases {
            let mut ring_signature = vec![0; ring_key.public().modulus_len()];
            ring_key
                .sign(encoding, &random, MESSAGE, &mut ring_signature)
                .expect("Ring signs vector");
            let rustls_verifier =
                verifier_factory(&algorithm, &decoding_key).expect("Rustls verifier");
            rustls_verifier
                .verify(MESSAGE, &ring_signature)
                .expect("Rustls verifies Ring vector");

            let rustls_signer = signer_factory(&algorithm, &encoding_key).expect("Rustls signer");
            let rustls_signature = rustls_signer
                .try_sign(MESSAGE)
                .expect("Rustls signs vector");
            UnparsedPublicKey::new(
                parameters,
                match decoding_key.kind() {
                    jsonwebtoken::DecodingKeyKind::SecretOrDer(der) => der,
                    _ => panic!("test key must contain DER"),
                },
            )
            .verify(MESSAGE, &rustls_signature)
            .expect("Ring verifies Rustls vector");
        }
    }
}
