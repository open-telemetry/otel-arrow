use super::config::{AuthConfig, normalize_source};
use x509_parser::{extensions::GeneralName, prelude::*};

/// A normalized, authorized source DNS identity.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SourceIdentity(String);

impl SourceIdentity {
    /// Extract and authorize an identity from an already verified client leaf certificate.
    ///
    /// The caller must first complete mutual TLS certificate verification. This method
    /// parses identity only; it does not verify signatures, validity periods, or EKUs.
    /// Certificate renewal preserves identity when the single DNS SAN is unchanged.
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
    use rcgen::{CertificateParams, DistinguishedName, DnType, KeyPair};

    fn certificate(names: &[&str]) -> Vec<u8> {
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
        let first =
            SourceIdentity::from_verified_certificate(&certificate(&["HOST.Example.com"]), &auth())
                .unwrap();
        let renewed =
            SourceIdentity::from_verified_certificate(&certificate(&["host.example.com"]), &auth())
                .unwrap();
        assert_eq!(first, renewed);
        assert_eq!(first.as_str(), "host.example.com");
    }

    /// Scenario: a certificate has no DNS SAN, ambiguous DNS SANs, or an unauthorized name.
    /// Guarantees: subject CN, IP SANs, wildcard SANs, and other hosts cannot grant access.
    #[test]
    fn reject_ambiguous_or_unauthorized_identity() {
        for names in [
            vec![],
            vec!["127.0.0.1"],
            vec!["host.example.com", "other.example.com"],
            vec!["host.example.com", "host.example.com"],
            vec!["*.example.com"],
            vec!["other.example.com"],
        ] {
            assert!(
                SourceIdentity::from_verified_certificate(&certificate(&names), &auth()).is_err()
            );
        }
    }

    /// Scenario: certificate input is malformed or contains bytes after its DER value.
    /// Guarantees: parsing fails closed without panicking or accepting a valid prefix.
    #[test]
    fn reject_invalid_der() {
        assert!(SourceIdentity::from_verified_certificate(&[0, 1, 2], &auth()).is_err());
        let mut trailing = certificate(&["host.example.com"]);
        trailing.push(0);
        assert!(SourceIdentity::from_verified_certificate(&trailing, &auth()).is_err());
    }
}
