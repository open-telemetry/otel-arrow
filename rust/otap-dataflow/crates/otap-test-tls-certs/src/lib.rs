// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared TLS test certificate generation utilities (rcgen-only).

use rcgen::{
    BasicConstraints, CertificateParams, DnType, ExtendedKeyUsagePurpose, IsCa, Issuer, KeyPair,
    KeyUsagePurpose,
};
use std::fs;
use std::path::Path;

/// Re-exported for tests to avoid importing `rcgen` directly.
pub use rcgen::ExtendedKeyUsagePurpose as ExtendedKeyUsage;

/// Certificate and private key pair in PEM format.
pub struct GeneratedCert {
    /// PEM-encoded certificate.
    pub cert_pem: String,
    /// PEM-encoded private key.
    pub key_pem: String,
}

impl GeneratedCert {
    /// Write certificate and private key to `{name}.crt` and `{name}.key`.
    pub fn write_to_dir(&self, dir: &Path, name: &str) {
        fs::write(dir.join(format!("{name}.crt")), &self.cert_pem).expect("Write cert");
        fs::write(dir.join(format!("{name}.key")), &self.key_pem).expect("Write key");
    }
}

/// CA certificate plus signing issuer for leaf issuance.
pub struct GeneratedCa {
    /// PEM-encoded CA certificate.
    pub cert_pem: String,
    issuer: Issuer<'static, KeyPair>,
}

impl GeneratedCa {
    /// Write CA certificate to `{name}.crt`.
    pub fn write_cert_to_dir(&self, dir: &Path, name: &str) {
        fs::write(dir.join(format!("{name}.crt")), &self.cert_pem).expect("Write CA cert");
    }

    /// Issue a non-CA leaf certificate signed by this CA.
    #[must_use]
    pub fn issue_leaf(
        &self,
        cn: &str,
        san: Option<&str>,
        eku: Option<ExtendedKeyUsagePurpose>,
    ) -> GeneratedCert {
        let mut params = match san {
            Some(san_name) => CertificateParams::new(vec![san_name.to_string()]).expect("SAN"),
            None => CertificateParams::new(Vec::new()).expect("empty SAN"),
        };
        params.distinguished_name.push(DnType::CommonName, cn);
        params.is_ca = IsCa::ExplicitNoCa;
        params.use_authority_key_identifier_extension = true;
        params.key_usages.push(KeyUsagePurpose::DigitalSignature);
        if let Some(eku) = eku {
            params.extended_key_usages.push(eku);
        }

        let key_pair = KeyPair::generate().expect("leaf key");
        let cert = params
            .signed_by(&key_pair, &self.issuer)
            .expect("leaf cert");

        GeneratedCert {
            cert_pem: cert.pem(),
            key_pem: key_pair.serialize_pem(),
        }
    }
}

/// Generate a CA certificate that can issue leaf certificates.
#[must_use]
pub fn generate_ca(cn: &str) -> GeneratedCa {
    let mut params = CertificateParams::new(Vec::new()).expect("empty SAN");
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    params.distinguished_name.push(DnType::CommonName, cn);
    params.key_usages.push(KeyUsagePurpose::DigitalSignature);
    params.key_usages.push(KeyUsagePurpose::KeyCertSign);
    params.key_usages.push(KeyUsagePurpose::CrlSign);

    let key_pair = KeyPair::generate().expect("ca key");
    let cert = params.self_signed(&key_pair).expect("ca cert");
    let issuer = Issuer::new(params, key_pair);

    GeneratedCa {
        cert_pem: cert.pem(),
        issuer,
    }
}

/// Generate a self-signed certificate.
#[must_use]
pub fn generate_self_signed_cert(cn: &str, san: Option<&str>, is_ca: bool) -> GeneratedCert {
    let mut params = match san {
        Some(san_name) => CertificateParams::new(vec![san_name.to_string()]).expect("SAN"),
        None => CertificateParams::new(Vec::new()).expect("empty SAN"),
    };

    params.distinguished_name.push(DnType::CommonName, cn);
    params.is_ca = if is_ca {
        IsCa::Ca(BasicConstraints::Unconstrained)
    } else {
        IsCa::ExplicitNoCa
    };

    let key_pair = KeyPair::generate().expect("self-signed key");
    let cert = params.self_signed(&key_pair).expect("self-signed cert");

    GeneratedCert {
        cert_pem: cert.pem(),
        key_pem: key_pair.serialize_pem(),
    }
}

/// Generate CA + signed leaf cert and write them to disk.
#[must_use]
pub fn write_ca_and_leaf_to_dir(
    dir: &Path,
    ca_name: &str,
    ca_cn: &str,
    cert_name: &str,
    cert_cn: &str,
    cert_san: Option<&str>,
    cert_eku: Option<ExtendedKeyUsage>,
) -> (GeneratedCa, GeneratedCert) {
    let ca = generate_ca(ca_cn);
    ca.write_cert_to_dir(dir, ca_name);

    let leaf = ca.issue_leaf(cert_cn, cert_san, cert_eku);
    leaf.write_to_dir(dir, cert_name);

    (ca, leaf)
}
