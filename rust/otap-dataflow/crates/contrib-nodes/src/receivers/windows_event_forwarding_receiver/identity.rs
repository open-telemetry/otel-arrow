// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::config::{AuthConfig, normalize_source};
use x509_parser::{
    extensions::GeneralName,
    prelude::{FromDer, X509Certificate},
};

/// A normalized, authorized source DNS identity.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SourceIdentity(String);

impl SourceIdentity {
    /// Extract and authorize an identity from an already verified client leaf certificate.
    ///
    /// The caller must first complete mutual TLS certificate verification. This method
    /// parses identity only; it does not verify signatures, validity periods, or EKUs.
    /// Certificate renewal preserves identity when the single DNS SAN is unchanged.
    /// Non-DNS SANs and the subject common name are not used for authorization.
    ///
    /// # Errors
    ///
    /// Rejects malformed certificates, missing or ambiguous DNS SANs, invalid DNS
    /// identities, and sources absent from the configured exact lowercase allowlist.
    pub fn from_verified_certificate(
        certificate_der: &[u8],
        auth: &AuthConfig,
    ) -> Result<Self, String> {
        let (remaining, certificate) = X509Certificate::from_der(certificate_der)
            .map_err(|_| "invalid client certificate".to_owned())?;
        if !remaining.is_empty() {
            return Err("trailing bytes in client certificate".into());
        }
        let names = certificate
            .subject_alternative_name()
            .map_err(|_| "invalid subject alternative name extension".to_owned())?
            .ok_or_else(|| "client certificate requires one DNS SAN".to_owned())?;
        let mut dns_names = names.value.general_names.iter().filter_map(|name| {
            if let GeneralName::DNSName(dns) = name {
                Some(*dns)
            } else {
                None
            }
        });
        let dns_name = dns_names
            .next()
            .ok_or_else(|| "client certificate requires one DNS SAN".to_owned())?;
        if dns_names.next().is_some() {
            return Err("client certificate has multiple DNS SANs".into());
        }
        let source = normalize_source(dns_name)?;
        if !auth.allowed_sources.contains(&source) {
            return Err("client certificate source is not authorized".into());
        }
        Ok(Self(source))
    }

    /// Return the authenticated source name, independent of event and SOAP claims.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rcgen::{CertificateParams, CustomExtension, DistinguishedName, DnType, KeyPair};

    fn certificate(names: &[&str], extensions: Vec<CustomExtension>) -> Vec<u8> {
        let mut params = CertificateParams::new(
            names
                .iter()
                .map(|name| (*name).to_owned())
                .collect::<Vec<_>>(),
        )
        .unwrap();
        params.distinguished_name = DistinguishedName::new();
        params
            .distinguished_name
            .push(DnType::CommonName, "host.example.com");
        params.custom_extensions = extensions;
        params
            .self_signed(&KeyPair::generate().unwrap())
            .unwrap()
            .der()
            .to_vec()
    }

    fn auth() -> AuthConfig {
        AuthConfig {
            allowed_sources: vec!["host.example.com".into()],
        }
    }

    /// Scenario: a source renews its certificate with the same DNS SAN in different case.
    /// Guarantees: independently issued certificates map to the same normalized identity.
    #[test]
    fn certificate_renewal_preserves_identity() {
        let first = SourceIdentity::from_verified_certificate(
            &certificate(&["HOST.Example.com"], vec![]),
            &auth(),
        )
        .unwrap();
        let renewed = SourceIdentity::from_verified_certificate(
            &certificate(&["host.example.com"], vec![]),
            &auth(),
        )
        .unwrap();
        assert_eq!(first, renewed);
        assert_eq!(first.as_str(), "host.example.com");
    }

    /// Scenario: a certificate has no DNS SAN or multiple DNS SANs, even identical ones.
    /// Guarantees: CN and IP SANs cannot replace a DNS SAN, and DNS ambiguity is rejected.
    #[test]
    fn reject_missing_or_ambiguous_dns_san() {
        for names in [
            vec![],
            vec!["127.0.0.1"],
            vec!["host.example.com", "other.example.com"],
            vec!["host.example.com", "host.example.com"],
        ] {
            assert!(
                SourceIdentity::from_verified_certificate(&certificate(&names, vec![]), &auth())
                    .is_err(),
                "accepted {names:?}"
            );
        }
    }

    /// Scenario: an authorized DNS SAN coexists with an IP SAN.
    /// Guarantees: non-DNS SANs do not change the single authorized DNS identity.
    #[test]
    fn ignores_non_dns_sans() {
        let identity = SourceIdentity::from_verified_certificate(
            &certificate(&["127.0.0.1", "host.example.com"], vec![]),
            &auth(),
        )
        .unwrap();
        assert_eq!(identity.as_str(), "host.example.com");
    }

    /// Scenario: a DNS SAN is invalid or does not exactly match the configured allowlist.
    /// Guarantees: wildcards, malformed names, other hosts, and subdomains cannot grant access.
    #[test]
    fn reject_invalid_or_unauthorized_identity() {
        for name in [
            "*.example.com",
            "host.example.com.",
            "-host.example.com",
            "host..example.com",
            "host_name.example.com",
            "other.example.com",
            "child.host.example.com",
        ] {
            assert!(
                SourceIdentity::from_verified_certificate(&certificate(&[name], vec![]), &auth())
                    .is_err(),
                "accepted {name}"
            );
        }
        assert!(
            SourceIdentity::from_verified_certificate(
                &certificate(&["host.example.com"], vec![]),
                &AuthConfig {
                    allowed_sources: vec![]
                },
            )
            .is_err()
        );
    }

    /// Scenario: certificate input is malformed or contains bytes after its DER value.
    /// Guarantees: parsing fails closed without panicking or accepting a valid prefix.
    #[test]
    fn reject_invalid_der() {
        for invalid in [&[][..], &[0, 1, 2][..]] {
            assert!(SourceIdentity::from_verified_certificate(invalid, &auth()).is_err());
        }
        let mut der = certificate(&["host.example.com"], vec![]);
        assert!(SourceIdentity::from_verified_certificate(&der[..der.len() - 1], &auth()).is_err());
        der.push(0);
        assert_eq!(
            SourceIdentity::from_verified_certificate(&der, &auth()).unwrap_err(),
            "trailing bytes in client certificate"
        );
    }

    /// Scenario: a certificate contains a malformed SAN extension or duplicate SAN extensions.
    /// Guarantees: invalid extensions cannot hide a missing DNS identity or bypass ambiguity checks.
    #[test]
    fn reject_invalid_san_extensions() {
        let malformed = CustomExtension::from_oid_content(&[2, 5, 29, 17], vec![0x05, 0x00]);
        let duplicate = CustomExtension::from_oid_content(&[2, 5, 29, 17], vec![0x30, 0x00]);
        for der in [
            certificate(&[], vec![malformed]),
            certificate(&["host.example.com"], vec![duplicate]),
        ] {
            assert_eq!(
                SourceIdentity::from_verified_certificate(&der, &auth()).unwrap_err(),
                "invalid subject alternative name extension"
            );
        }
    }
}
