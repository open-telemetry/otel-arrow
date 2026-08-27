// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use rustls::{
    ClientConfig, RootCertStore,
    pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject},
};

use otel_arrow_dfe_config::{error::Error, tls::TlsClientConfig};
use otel_arrow_dfe_otap::tls_utils::{
    add_system_trust_anchors_to_root_cert_store, read_file_with_limit_async,
};
use otel_arrow_dfe_telemetry::otel_error;

pub async fn create_client_config(config: &TlsClientConfig) -> Result<Option<ClientConfig>, Error> {
    let insecure = config.insecure.unwrap_or(false);
    // TODO - there might be some other stuff that conflicts with insecure flag?
    // see the gRPC implementation where we check if there's also a custom CA configured
    // and if not, we don't return this
    if insecure {
        return Ok(None);
    }

    if let Some(domain) = &config.server_name {
        // TODO
    }

    // TODO
    // - server_name
    // - insecure_skip_verify

    let mut cert_store = RootCertStore::empty();
    if config.include_system_ca_certs_pool.unwrap_or(false) {
        add_system_trust_anchors_to_root_cert_store(&mut cert_store)
            .await
            .map_err(|e| Error::ConfigHttpClientBuildFailed {
                details: format!("Failed to add system rust anchors to cert store: {e}"),
            })?;
    }

    if let Some(ca_pem) = &config.ca_pem {
        let cert = CertificateDer::from_pem_slice(ca_pem.as_bytes()).map_err(|e| {
            Error::ConfigHttpClientBuildFailed {
                details: format!("Failed to create cert from tls.ca_pem config: {e}"),
            }
        })?;
        add_cert(&mut cert_store, cert)?
    }

    if let Some(ca_file) = &config.ca_file {
        let cert = CertificateDer::from_pem_file(ca_file).map_err(|e| {
            Error::ConfigHttpClientBuildFailed {
                details: format!("Failed to create cert from tls.ca_pem config: {e}"),
            }
        })?;
        add_cert(&mut cert_store, cert)?;
    }

    // mTLS client certificate configuration
    let client_cert_configured = config.config.cert_file.is_some()
        || config
            .config
            .cert_pem
            .as_ref()
            .is_some_and(|pem| !pem.trim().is_empty());
    let client_key_configured = config.config.key_file.is_some()
        || config
            .config
            .key_pem
            .as_ref()
            .is_some_and(|pem| !pem.trim().is_empty());

    let builder = ClientConfig::builder().with_root_certificates(cert_store);

    if client_cert_configured || client_key_configured {
        if !(client_cert_configured && client_key_configured) {
            return Err(Error::ConfigHttpClientBuildFailed {
                details: "TLS configuration error: both cert and key must be provided for mTLS. \
                    Provide both cert_file/cert_pem and key_file/key_pem"
                    .into(),
            });
        }

        // Read cert and key
        let cert_pem = if let Some(cert_file) = &config.config.cert_file {
            read_file_with_limit_async(cert_file).await.map_err(|e| {
                otel_error!(
                    "tls.cert_file.read_error",
                    cert_file = ?cert_file,
                    error = ?e, message = "Failed to read client cert file"
                );
                Error::ConfigHttpClientBuildFailed {
                    details: format!("failed to read client mTLS cert file"),
                }
            })?
        } else if let Some(cert_pem) = &config.config.cert_pem {
            cert_pem.as_bytes().to_vec()
        } else {
            unreachable!()
        };

        let key_pem = if let Some(key_file) = &config.config.key_file {
            read_file_with_limit_async(key_file).await.map_err(|e| {
                otel_error!(
                    "tls.key_file.read_error",
                    key_file = ?key_file,
                    error = ?e,
                    message = "Failed to read client key file"
                );
                Error::ConfigHttpClientBuildFailed {
                    details: format!("failed to read client mTLS Key file"),
                }
            })?
        } else if let Some(key_pem) = &config.config.key_pem {
            key_pem.as_bytes().to_vec()
        } else {
            unreachable!()
        };

        let cert_chain = CertificateDer::pem_slice_iter(&cert_pem)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| todo!())?;

        let key_der = PrivateKeyDer::from_pem_slice(&key_pem).map_err(|e| todo!())?;

        // TODO - deal with reload duration

        //
        let client_config = builder.with_client_auth_cert(cert_chain, key_der).unwrap();
        Ok(Some(client_config))
    } else {
        Ok(Some(builder.with_no_client_auth()))
    }
}

fn add_cert(cert_store: &mut RootCertStore, cert: CertificateDer<'_>) -> Result<(), Error> {
    cert_store
        .add(cert)
        .map_err(|e| Error::ConfigHttpClientBuildFailed {
            details: format!("failed to add cert to root trust store {e}"),
        })
}
