// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! ClickHouse write path and database bootstrap for the OTAP ClickHouse exporter.
//!
//! This module owns the runtime ClickHouse client, bootstraps the target database/schema (tables),
//! and writes OTAP Arrow `RecordBatch`es into the appropriate ClickHouse tables.
//!
//! Major components:
//!
//! - **Writer abstraction**
//!   - [`ClickHouseWriter`] maintains a ClickHouse client, the target database name, and a
//!     precomputed map from `ArrowPayloadType` to destination table name.
//!   - `write_batches` accepts a set of per-payload `RecordBatch`es, dispatches inserts, and
//!     returns the successfully written row counts for metrics accounting by the caller.
//!   - `write_batch` performs a single `INSERT ... FORMAT ArrowStream` over HTTP using the official
//!     client's Arrow extension, surfacing any server-side errors on `end()`.
//!
//! - **Client initialization**
//!   - `build_client` builds a ClickHouse HTTP client with endpoint/auth configuration and applies
//!     optional async inserts.
//!
//! - **Database/schema initialization**
//!   - `ensure_db` creates the configured database if missing.
//!   - `init_db` creates tables for all configured entries.
use std::collections::HashMap;

use arrow_array::RecordBatch;
use clickhouse::Client;
use clickhouse_ext_arrow::ArrowClientExt;
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use secrecy::ExposeSecret;

use crate::exporters::clickhouse_exporter::{
    config::Config,
    error::ClickhouseExporterError,
    tables::{build_payload_destination_table_map, init_table, validate_identifier},
};

pub struct ClickHouseWriter {
    client: Client,
    payload_destination_tables: HashMap<ArrowPayloadType, String>,
}

impl ClickHouseWriter {
    pub async fn new(config: &Config) -> Result<Self, ClickhouseExporterError> {
        let payload_destination_tables = build_payload_destination_table_map(config);
        otel_info!(
            "clickhouse.exporter.tables.bound",
            message = "Destination tables bound",
            logs = payload_destination_tables.get(&ArrowPayloadType::Logs),
            spans = payload_destination_tables.get(&ArrowPayloadType::Spans),
        );
        // Bootstrap DB/schema with a client bound to the `default` database, since the configured
        // database may not exist yet. DDL is fully qualified (`db.table`) so the bound database is
        // irrelevant for it.
        let bootstrap_client = build_client(config, "default").await?;
        init_db(&bootstrap_client, config).await?;
        // Inserts use `insert_arrow(table)`, which binds the table's database from the client (the
        // generated `INSERT INTO \`table\` FORMAT ArrowStream` is unqualified), so the runtime
        // client must point at the configured database.
        //
        // Reuse the bootstrap client's transport and connection pool rather than
        // calling build_client a second time.
        let client = bootstrap_client.with_database(&config.database);
        Ok(Self {
            client,
            payload_destination_tables,
        })
    }

    async fn write_batch(
        &self,
        table_name: &str,
        batch: &RecordBatch,
    ) -> Result<(), ClickhouseExporterError> {
        let mut insert = self.client.insert_arrow(table_name).map_err(|e| {
            ClickhouseExporterError::InsertRequestError {
                error: format!("{e}"),
            }
        })?;
        insert
            .write(batch)
            .await
            .map_err(|e| ClickhouseExporterError::InsertRequestError {
                error: format!("{e}"),
            })?;
        insert
            .end()
            .await
            .map_err(|e| ClickhouseExporterError::InsertResponseError {
                error: format!("{e}"),
            })?;
        otel_debug!(
            "clickhouse.exporter.batch.written",
            message = "Record batch successfully written.",
            table = table_name,
            rows = batch.num_rows(),
            columns = batch.num_columns(),
        );

        Ok(())
    }
    pub async fn write_batches(
        &self,
        write_batches: &HashMap<ArrowPayloadType, RecordBatch>,
    ) -> Result<Vec<(ArrowPayloadType, u64)>, ClickhouseExporterError> {
        let mut written_rows = Vec::with_capacity(write_batches.len());
        for (payload_type, batch) in write_batches {
            let Some(table_name) = self.payload_destination_tables.get(payload_type) else {
                continue;
            };
            self.write_batch(table_name, batch).await?;
            written_rows.push((*payload_type, batch.num_rows() as u64));
        }
        Ok(written_rows)
    }
}

/// Pool idle timeout applied to every hyper client we construct.
///
/// The ClickHouse server default `keep_alive_timeout` is 3 s. We use a
/// slightly shorter idle timeout so that hyper reclaims connections before
/// the server closes them, avoiding "connection reset" errors on reuse.
const POOL_IDLE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(2);

/// TCP keepalive interval for ClickHouse HTTP connections.
const TCP_KEEPALIVE: std::time::Duration = std::time::Duration::from_secs(60);

/// Build an `HttpConnector` pre-configured for use as the TCP layer
/// beneath an HTTPS connector.
///
/// Sets `enforce_http(false)` so hyper allows `https://` URLs through
/// the raw connector (TLS is handled by the wrapping
/// [`hyper_rustls::HttpsConnector`]), and enables TCP keepalive so
/// that silently-dead connections are detected during long inserts.
fn build_https_base_connector() -> hyper_util::client::legacy::connect::HttpConnector {
    let mut http = hyper_util::client::legacy::connect::HttpConnector::new();
    http.enforce_http(false);
    http.set_keepalive(Some(TCP_KEEPALIVE));
    http
}

/// Build a plaintext-only hyper client (no TLS) wrapped in a ClickHouse
/// [`Client`].
fn build_plaintext_client() -> Client {
    let mut connector = hyper_util::client::legacy::connect::HttpConnector::new();
    connector.set_keepalive(Some(TCP_KEEPALIVE));

    let hyper_client =
        hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
            .pool_idle_timeout(POOL_IDLE_TIMEOUT)
            .build(connector);

    Client::with_http_client(hyper_client)
}

/// Build a ClickHouse HTTP client bound to `database` from the exporter config.
///
/// All branches construct a provider-neutral hyper client rather than relying
/// on `clickhouse::Client::default()`, which hard-codes a specific rustls
/// crypto provider.  This keeps the exporter aligned with the workspace's
/// `crypto-*` feature selection (ring, aws-lc, openssl, or symcrypt).
///
/// # TLS paths
///
/// When a `tls` block is present, the shared TLS loader decides whether
/// certificate material is available.  If the loader returns material, the
/// endpoint **must** be `https://` -- an `http://` endpoint is rejected at
/// startup because hyper-rustls would silently ignore the configured
/// certificates over plaintext.  A custom hyper HTTPS client is built with
/// the loaded CA trust anchors and optional client certificate (mTLS).  The
/// connector uses `https_only()` so that configured certificates are always
/// applied.
///
/// When a `tls` block is present with `insecure: true` and no custom CA, the
/// shared loader returns `None` and a plaintext-only hyper client is used,
/// consistent with the shared loader's contract and the behavior of other
/// components (OTLP/gRPC, OAuth2).
///
/// Without a `tls` block the endpoint scheme decides: `https://` endpoints
/// get a provider-neutral HTTPS connector with system CA trust anchors, while
/// `http://` endpoints use a plaintext connector.
///
/// `Client` is cheap to clone (it is `Arc`-backed internally), so callers can
/// build one per target database without concern.
pub(crate) async fn build_client(
    config: &Config,
    database: &str,
) -> Result<Client, ClickhouseExporterError> {
    let mut client = if let Some(tls_config) = &config.tls {
        // Reject contradictory config: insecure=true makes no sense on an
        // https:// endpoint. Other components (OpAMP, OAuth2, admin-api, proxy)
        // follow the same pattern.
        if tls_config.insecure == Some(true) && config.endpoint.starts_with("https://") {
            return Err(ClickhouseExporterError::TlsConfig(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "tls.insecure=true is not supported on https:// endpoints; \
                 use an http:// endpoint for plaintext connections or remove \
                 the insecure setting",
            )));
        }

        // Let the shared TLS loader decide whether material is present.
        // Checking the loader result (rather than inspecting individual
        // fields) catches every combination the loader recognises.
        match otel_arrow_dfe_otap::tls_utils::build_rustls_client_config(
            Some(tls_config),
            &config.endpoint,
        )
        .await
        .map_err(ClickhouseExporterError::TlsConfig)?
        {
            Some((rustls_config, server_name_override)) => {
                // The loader produced TLS material. Reject http://
                // endpoints: hyper-rustls enforces TLS only when the
                // request URL uses https://, so the configured
                // certificates would be silently ignored on plaintext
                // connections.
                if !config.endpoint.starts_with("https://") {
                    return Err(ClickhouseExporterError::TlsConfig(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "TLS certificates were loaded from the tls block but the \
                         endpoint uses http://; the configured certificates cannot \
                         be applied over plaintext HTTP.",
                    )));
                }

                // TLS material was loaded -- enforce HTTPS so the configured
                // certificates are actually used. With https_only() the
                // connector rejects plaintext requests, preventing credentials
                // and telemetry from being sent unencrypted when TLS was
                // explicitly configured.
                let mut connector_builder = hyper_rustls::HttpsConnectorBuilder::new()
                    .with_tls_config(rustls_config)
                    .https_only();

                // Apply server_name_override for SNI and certificate
                // verification when the user connects by IP or needs a
                // hostname that differs from the URL.
                if let Some(ref server_name) = server_name_override {
                    let name = rustls::pki_types::ServerName::try_from(server_name.clone())
                        .map_err(|e| {
                            ClickhouseExporterError::TlsConfig(std::io::Error::new(
                                std::io::ErrorKind::InvalidInput,
                                format!("invalid server_name_override '{server_name}': {e}"),
                            ))
                        })?;
                    connector_builder = connector_builder.with_server_name_resolver(
                        hyper_rustls::FixedServerNameResolver::new(name),
                    );
                }

                let https_connector = connector_builder
                    .enable_http1()
                    .wrap_connector(build_https_base_connector());

                let hyper_client = hyper_util::client::legacy::Client::builder(
                    hyper_util::rt::TokioExecutor::new(),
                )
                .pool_idle_timeout(POOL_IDLE_TIMEOUT)
                .build(https_connector);

                Client::with_http_client(hyper_client)
            }
            None => {
                // The shared loader returned None -- insecure=true with no
                // custom CA on a non-https endpoint. Plaintext is explicitly
                // requested.
                build_plaintext_client()
            }
        }
    } else {
        // No tls block: let the shared loader decide based on endpoint scheme.
        // - https:// => provider-neutral TLS with system CA trust anchors
        // - http://  => plaintext
        match otel_arrow_dfe_otap::tls_utils::build_rustls_client_config(None, &config.endpoint)
            .await
            .map_err(ClickhouseExporterError::TlsConfig)?
        {
            Some((rustls_config, _server_name)) => {
                let https_connector = hyper_rustls::HttpsConnectorBuilder::new()
                    .with_tls_config(rustls_config)
                    .https_only()
                    .enable_http1()
                    .wrap_connector(build_https_base_connector());

                let hyper_client = hyper_util::client::legacy::Client::builder(
                    hyper_util::rt::TokioExecutor::new(),
                )
                .pool_idle_timeout(POOL_IDLE_TIMEOUT)
                .build(https_connector);

                Client::with_http_client(hyper_client)
            }
            None => build_plaintext_client(),
        }
    };

    client = client
        .with_url(config.endpoint.clone())
        .with_database(database)
        .with_user(config.username.clone())
        .with_password(config.password.expose_secret());

    if config.async_insert {
        client = client
            .with_setting("async_insert", "1")
            .with_setting("wait_for_async_insert", "1");
    }
    Ok(client)
}

/// Ensure db and all tables are initialized if required.
async fn ensure_db(client: &Client, config: &Config) -> Result<(), ClickhouseExporterError> {
    validate_identifier(&config.database, "database")?;
    client
        .query(&format!(
            "CREATE DATABASE IF NOT EXISTS {};",
            config.database
        ))
        .execute()
        .await
        .map_err(|e| ClickhouseExporterError::TableCreationError {
            error: format!("{e}"),
        })?;
    Ok(())
}

/// Initialize tables.
async fn init_db(client: &Client, config: &Config) -> Result<(), ClickhouseExporterError> {
    ensure_db(client, config).await?;
    for entry in config.tables.iter_tables() {
        init_table(client, &entry, config).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(unused_results)]
    use super::*;
    use arrow_array::{ArrayRef, Int32Array, RecordBatch, StringArray};
    use arrow_schema::{DataType, Field, Schema};
    use otel_arrow_dfe_config::tls::{TlsClientConfig, TlsConfig};
    use std::{
        collections::HashMap,
        num::NonZeroUsize,
        sync::{Arc, Mutex},
    };

    /// Build a minimal Config for tests that only exercise `build_client`.
    fn test_config_with_tls(endpoint: &str, tls: Option<TlsClientConfig>) -> Config {
        Config {
            endpoint: endpoint.to_string(),
            database: "default".to_string(),
            username: "default".to_string(),
            password: secrecy::SecretString::from(""),
            async_insert: false,
            max_in_flight: NonZeroUsize::new(1).unwrap(),
            tls,
            table_defaults:
                crate::exporters::clickhouse_exporter::config::DefaultTableConfig::default(),
            tables: crate::exporters::clickhouse_exporter::config::TablesConfig::default(),
        }
    }

    /// Scenario: build_client with a valid CA-only TLS config on an https endpoint.
    /// Guarantees: the TLS construction path (rustls ClientConfig, hyper-rustls
    /// HttpsConnector, hyper Client) succeeds and returns an Ok(Client).
    #[tokio::test]
    async fn build_client_with_tls_ca_only_succeeds() {
        otel_arrow_dfe_otap::crypto::ensure_crypto_provider();

        let ca = otel_arrow_dfe_test_tls_certs::generate_ca("Test CA");

        let tls = TlsClientConfig {
            ca_pem: Some(ca.cert_pem.clone()),
            include_system_ca_certs_pool: Some(false),
            ..Default::default()
        };

        let config = test_config_with_tls("https://clickhouse.example.com:8443", Some(tls));
        let result = build_client(&config, "default").await;
        assert!(
            result.is_ok(),
            "build_client with valid CA should succeed: {result:?}"
        );
    }

    /// Scenario: build_client with mTLS (CA + client cert + client key).
    /// Guarantees: the mTLS construction path with client identity succeeds.
    #[tokio::test]
    async fn build_client_with_mtls_succeeds() {
        otel_arrow_dfe_otap::crypto::ensure_crypto_provider();

        let ca = otel_arrow_dfe_test_tls_certs::generate_ca("Test CA");
        let client_cert = ca.issue_leaf(
            "client",
            Some("client"),
            Some(otel_arrow_dfe_test_tls_certs::ExtendedKeyUsage::ClientAuth),
        );

        let tls = TlsClientConfig {
            ca_pem: Some(ca.cert_pem.clone()),
            include_system_ca_certs_pool: Some(false),
            config: TlsConfig {
                cert_pem: Some(client_cert.cert_pem),
                key_pem: Some(client_cert.key_pem),
                ..Default::default()
            },
            ..Default::default()
        };

        let config = test_config_with_tls("https://clickhouse.example.com:8443", Some(tls));
        let result = build_client(&config, "default").await;
        assert!(
            result.is_ok(),
            "build_client with mTLS should succeed: {result:?}"
        );
    }

    /// Scenario: build_client with a server_name_override on an https endpoint.
    /// Guarantees: the server name override is accepted and does not cause an
    /// error during client construction.
    #[tokio::test]
    async fn build_client_with_server_name_override_succeeds() {
        otel_arrow_dfe_otap::crypto::ensure_crypto_provider();

        let ca = otel_arrow_dfe_test_tls_certs::generate_ca("Test CA");

        let tls = TlsClientConfig {
            ca_pem: Some(ca.cert_pem.clone()),
            include_system_ca_certs_pool: Some(false),
            server_name: Some("override.example.com".to_string()),
            ..Default::default()
        };

        let config = test_config_with_tls("https://10.0.0.1:8443", Some(tls));
        let result = build_client(&config, "default").await;
        assert!(
            result.is_ok(),
            "build_client with server_name_override should succeed: {result:?}"
        );
    }

    /// Scenario: build_client with insecure=true and no custom CA on a
    /// non-https endpoint inside a tls block.
    /// Guarantees: the configuration is accepted and a default plaintext
    /// client is returned, consistent with the shared TLS loader's contract
    /// and the behavior of other components (OTLP/gRPC, OAuth2).
    #[tokio::test]
    async fn build_client_tls_block_with_insecure_succeeds() {
        otel_arrow_dfe_otap::crypto::ensure_crypto_provider();

        let tls = TlsClientConfig {
            insecure: Some(true),
            ..Default::default()
        };

        let config = test_config_with_tls("http://localhost:8123", Some(tls));
        let result = build_client(&config, "default").await;
        assert!(
            result.is_ok(),
            "insecure=true with tls block on http should succeed: {result:?}"
        );
    }

    /// Scenario: build_client with insecure=true on an https:// endpoint.
    /// Guarantees: the contradictory configuration is rejected with a clear
    /// error before any TLS material is loaded.
    #[tokio::test]
    async fn build_client_tls_insecure_on_https_errors() {
        otel_arrow_dfe_otap::crypto::ensure_crypto_provider();

        let tls = TlsClientConfig {
            insecure: Some(true),
            ..Default::default()
        };

        let config = test_config_with_tls("https://clickhouse.example.com:8443", Some(tls));
        let result = build_client(&config, "default").await;
        assert!(result.is_err(), "insecure=true on https:// should error");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("insecure=true"),
            "error should mention insecure=true: {err_msg}",
        );
        assert!(
            err_msg.contains("https://"),
            "error should mention https://: {err_msg}",
        );
    }

    /// Scenario: build_client with a custom CA on an http:// endpoint without
    /// insecure=true.
    /// Guarantees: the misconfiguration is rejected at startup with a message
    /// explaining that TLS material requires an https:// endpoint, preventing
    /// certificates from being silently ignored over plaintext.
    #[tokio::test]
    async fn build_client_http_with_tls_material_errors() {
        otel_arrow_dfe_otap::crypto::ensure_crypto_provider();

        let ca = otel_arrow_dfe_test_tls_certs::generate_ca("Test CA");

        let tls = TlsClientConfig {
            ca_pem: Some(ca.cert_pem.clone()),
            include_system_ca_certs_pool: Some(false),
            ..Default::default()
        };

        let config = test_config_with_tls("http://localhost:8123", Some(tls));
        let result = build_client(&config, "default").await;
        assert!(
            result.is_err(),
            "http:// with TLS material should error: {result:?}"
        );
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("https://"),
            "error should mention https://: {err_msg}",
        );
    }

    /// Scenario: build_client with insecure=true and a custom CA on an http://
    /// endpoint. The shared TLS loader recognises the CA and returns TLS
    /// material, but the endpoint is plaintext.
    /// Guarantees: the misconfiguration is rejected at startup instead of
    /// building an HTTPS-only connector that would fail on the first request.
    #[tokio::test]
    async fn build_client_insecure_with_ca_on_http_errors() {
        otel_arrow_dfe_otap::crypto::ensure_crypto_provider();

        let ca = otel_arrow_dfe_test_tls_certs::generate_ca("Test CA");

        let tls = TlsClientConfig {
            insecure: Some(true),
            ca_pem: Some(ca.cert_pem.clone()),
            include_system_ca_certs_pool: Some(false),
            ..Default::default()
        };

        let config = test_config_with_tls("http://localhost:8123", Some(tls));
        let result = build_client(&config, "default").await;
        assert!(
            result.is_err(),
            "insecure=true with CA on http:// should error: {result:?}"
        );
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("http://"),
            "error should mention http://: {err_msg}",
        );
        assert!(
            err_msg.contains("https://"),
            "error should mention https://: {err_msg}",
        );
    }

    /// Scenario: build_client with insecure=true and client identity (mTLS
    /// material) on an http:// endpoint. The shared TLS loader builds
    /// material for the client cert/key, but the endpoint is plaintext.
    /// Guarantees: the misconfiguration is rejected at startup, preventing
    /// client credentials from being silently unused.
    #[tokio::test]
    async fn build_client_insecure_with_client_cert_on_http_errors() {
        otel_arrow_dfe_otap::crypto::ensure_crypto_provider();

        let ca = otel_arrow_dfe_test_tls_certs::generate_ca("Test CA");
        let client_cert = ca.issue_leaf(
            "client",
            Some("client"),
            Some(otel_arrow_dfe_test_tls_certs::ExtendedKeyUsage::ClientAuth),
        );

        let tls = TlsClientConfig {
            insecure: Some(true),
            ca_pem: Some(ca.cert_pem.clone()),
            include_system_ca_certs_pool: Some(false),
            config: TlsConfig {
                cert_pem: Some(client_cert.cert_pem),
                key_pem: Some(client_cert.key_pem),
                ..Default::default()
            },
            ..Default::default()
        };

        let config = test_config_with_tls("http://localhost:8123", Some(tls));
        let result = build_client(&config, "default").await;
        assert!(
            result.is_err(),
            "insecure=true with client cert on http:// should error: {result:?}"
        );
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("https://"),
            "error should mention https://: {err_msg}",
        );
    }

    /// Scenario: build_client with an empty tls block (all defaults) on an
    /// http:// endpoint. The shared TLS loader builds material using
    /// system CA trust anchors (include_system_ca_certs_pool defaults to
    /// true), so the loader returns Some(...).
    /// Guarantees: the misconfiguration is rejected because TLS material
    /// cannot be applied over plaintext HTTP. Users who want plaintext
    /// should either remove the tls block or set tls.insecure=true.
    #[tokio::test]
    async fn build_client_empty_tls_block_on_http_errors() {
        otel_arrow_dfe_otap::crypto::ensure_crypto_provider();

        let tls = TlsClientConfig::default();

        let config = test_config_with_tls("http://localhost:8123", Some(tls));
        let result = build_client(&config, "default").await;
        assert!(
            result.is_err(),
            "empty tls block on http:// should error: {result:?}"
        );
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("https://"),
            "error should mention https://: {err_msg}",
        );
    }

    /// Scenario: build_client with an invalid server_name_override value.
    /// Guarantees: the ServerName::try_from error branch is exercised and
    /// returns a TlsConfig error mentioning the invalid value.
    #[tokio::test]
    async fn build_client_with_invalid_server_name_override_errors() {
        otel_arrow_dfe_otap::crypto::ensure_crypto_provider();

        let ca = otel_arrow_dfe_test_tls_certs::generate_ca("Test CA");

        let tls = TlsClientConfig {
            ca_pem: Some(ca.cert_pem.clone()),
            include_system_ca_certs_pool: Some(false),
            server_name: Some("not a valid name!@#".to_string()),
            ..Default::default()
        };

        let config = test_config_with_tls("https://10.0.0.1:8443", Some(tls));
        let result = build_client(&config, "default").await;
        assert!(result.is_err(), "invalid server_name_override should error");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("server_name_override"),
            "error should mention server_name_override: {err_msg}",
        );
    }

    /// Scenario: build_client without a tls block on an http:// endpoint.
    /// Guarantees: the plaintext path builds a provider-neutral hyper client
    /// and returns Ok without requiring a crypto provider.
    #[tokio::test]
    async fn build_client_without_tls_http_succeeds() {
        let config = test_config_with_tls("http://localhost:8123", None);
        let result = build_client(&config, "default").await;
        assert!(
            result.is_ok(),
            "build_client without TLS on http:// should succeed: {result:?}"
        );
    }

    /// Scenario: build_client without a tls block on an https:// endpoint.
    /// Guarantees: the provider-neutral default-TLS path loads system CA
    /// trust anchors via the installed crypto provider and returns Ok.
    #[tokio::test]
    async fn build_client_without_tls_https_succeeds() {
        otel_arrow_dfe_otap::crypto::ensure_crypto_provider();

        let config = test_config_with_tls("https://clickhouse.example.com:8443", None);
        let result = build_client(&config, "default").await;
        assert!(
            result.is_ok(),
            "build_client without TLS on https:// should succeed: {result:?}"
        );
    }

    /// Make a tiny RecordBatch with a known number of rows.
    fn mk_batch(rows: usize) -> RecordBatch {
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("msg", DataType::Utf8, false),
        ]));

        let ids: ArrayRef = Arc::new(Int32Array::from_iter_values(0..rows as i32));
        let msgs: ArrayRef = Arc::new(StringArray::from_iter_values(
            (0..rows).map(|i| format!("row-{i}")),
        ));

        RecordBatch::try_new(schema, vec![ids, msgs]).unwrap()
    }

    /// A lightweight harness that mirrors ClickHouseWriter::write_batches logic,
    /// but replaces the actual ClickHouse insert with an in-memory "call log".
    ///
    /// This keeps tests as pure unit tests.
    #[derive(Clone, Default)]
    struct TestWriter {
        payload_destination_tables: HashMap<ArrowPayloadType, String>,
        calls: Arc<Mutex<Vec<(ArrowPayloadType, String, usize)>>>, // (payload, table, rows)
    }

    impl TestWriter {
        async fn write_batch(
            &self,
            payload_type: ArrowPayloadType,
            table_name: &str,
            batch: &RecordBatch,
        ) -> Result<(), ClickhouseExporterError> {
            self.calls.lock().unwrap().push((
                payload_type,
                table_name.to_string(),
                batch.num_rows(),
            ));
            Ok(())
        }

        async fn write_batches(
            &self,
            write_batches: &HashMap<&ArrowPayloadType, RecordBatch>,
        ) -> Result<Vec<(ArrowPayloadType, u64)>, ClickhouseExporterError> {
            let mut stats = Vec::new();
            for (payload_type, batch) in write_batches {
                let Some(table_name) = self.payload_destination_tables.get(payload_type) else {
                    continue;
                };
                self.write_batch(**payload_type, table_name, batch).await?;
                stats.push((**payload_type, batch.num_rows() as u64));
            }
            Ok(stats)
        }
    }

    /// Scenario: a batch set contains mapped signal payloads and an unmapped attribute payload.
    /// Guarantees: mapped batches are written once and their exact row counts are returned.
    #[tokio::test]
    async fn writes_all_mapped_payloads_and_reports_rows() {
        // Arrange: only signal tables have mappings (no attribute table mappings)
        let mut tables = HashMap::new();
        tables.insert(ArrowPayloadType::Logs, "logs".to_string());
        tables.insert(ArrowPayloadType::Spans, "spans".to_string());

        let writer = TestWriter {
            payload_destination_tables: tables,
            ..Default::default()
        };

        let spans = mk_batch(2);
        let logs = mk_batch(5);
        // ResourceAttrs has no table mapping, so it will be skipped
        let resource = mk_batch(3);

        let mut batches: HashMap<&ArrowPayloadType, RecordBatch> = HashMap::new();
        batches.insert(&ArrowPayloadType::Spans, spans);
        batches.insert(&ArrowPayloadType::ResourceAttrs, resource);
        batches.insert(&ArrowPayloadType::Logs, logs);

        // Act
        let stats = writer.write_batches(&batches).await.unwrap();
        let calls = writer.calls.lock().unwrap().clone();

        // Assert: only mapped payloads are written (ResourceAttrs skipped - no table mapping)
        assert_eq!(calls.len(), 2);

        // Build maps for order-independent assertions
        let mut call_map: HashMap<ArrowPayloadType, (String, usize)> = HashMap::new();
        for (pt, table, rows) in &calls {
            call_map.insert(*pt, (table.clone(), *rows));
        }
        assert_eq!(
            call_map.get(&ArrowPayloadType::Logs),
            Some(&("logs".to_string(), 5))
        );
        assert_eq!(
            call_map.get(&ArrowPayloadType::Spans),
            Some(&("spans".to_string(), 2))
        );

        // Assert: row counts reported correctly
        let mut stat_map: HashMap<ArrowPayloadType, u64> = HashMap::new();
        for (pt, rows) in stats {
            stat_map.insert(pt, rows);
        }
        assert_eq!(stat_map.get(&ArrowPayloadType::Logs), Some(&5));
        assert_eq!(stat_map.get(&ArrowPayloadType::Spans), Some(&2));
        assert!(!stat_map.contains_key(&ArrowPayloadType::ResourceAttrs));
    }

    /// Scenario: a batch is supplied for a payload without a destination table mapping.
    /// Guarantees: the unmapped batch is skipped and only mapped rows are reported as written.
    #[tokio::test]
    async fn skips_when_batch_present_but_no_table_mapping() {
        // Arrange: only Logs has a table mapping
        let mut tables = HashMap::new();
        tables.insert(ArrowPayloadType::Logs, "logs".to_string());

        let writer = TestWriter {
            payload_destination_tables: tables,
            ..Default::default()
        };

        let mut batches: HashMap<&ArrowPayloadType, RecordBatch> = HashMap::new();
        batches.insert(&ArrowPayloadType::Logs, mk_batch(1));
        batches.insert(&ArrowPayloadType::Spans, mk_batch(10)); // should be skipped (no table mapping)

        // Act
        let stats = writer.write_batches(&batches).await.unwrap();
        let calls = writer.calls.lock().unwrap().clone();

        // Assert: only Logs written
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, ArrowPayloadType::Logs);
        assert_eq!(calls[0].1, "logs".to_string());
        assert_eq!(calls[0].2, 1);

        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].0, ArrowPayloadType::Logs);
        assert_eq!(stats[0].1, 1);
    }

    /// Scenario: the writer receives an empty collection of transformed batches.
    /// Guarantees: no inserts occur and no written-row metrics are returned.
    #[tokio::test]
    async fn empty_input_writes_nothing() {
        let writer = TestWriter::default();
        let batches: HashMap<&ArrowPayloadType, RecordBatch> = HashMap::new();

        let result = writer.write_batches(&batches).await.unwrap();
        let calls = writer.calls.lock().unwrap().clone();

        assert!(result.is_empty());
        assert!(calls.is_empty());
    }
}
