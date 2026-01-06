// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Provider for OTLP exporter configuration.

use opentelemetry_otlp::{
    Protocol, WithExportConfig, WithTonicConfig,
    tonic_types::transport::{Certificate, ClientTlsConfig},
};
use opentelemetry_sdk::metrics::{MeterProviderBuilder, PeriodicReader};
use otap_df_config::pipeline::service::telemetry::metrics::readers::{
    Temporality,
    periodic::otlp::{OtlpExporterConfig, OtlpProtocol},
};

use crate::error::Error;

/// Provider for OTLP exporter configuration.
pub(crate) struct OtlpExporterProvider {}

impl OtlpExporterProvider {
    /// Configure the OTLP exporter for the given MeterProviderBuilder.
    pub(crate) fn configure_otlp_metric_exporter(
        mut sdk_meter_builder: MeterProviderBuilder,
        otlp_config: &OtlpExporterConfig,
        interval: &std::time::Duration,
        mut runtime: Option<tokio::runtime::Runtime>,
    ) -> Result<(MeterProviderBuilder, Option<tokio::runtime::Runtime>), Error> {
        let exporter;
        match &otlp_config.protocol {
            OtlpProtocol::Grpc => {
                (exporter, runtime) = Self::configure_grpc_otlp_exporter(otlp_config, runtime)?
            }
            OtlpProtocol::HttpBinary => {
                exporter = Self::configure_http_exporter(otlp_config, Protocol::HttpBinary)?
            }
            OtlpProtocol::HttpJson => {
                exporter = Self::configure_http_exporter(otlp_config, Protocol::HttpJson)?
            }
        };
        let reader = PeriodicReader::builder(exporter)
            .with_interval(*interval)
            .build();
        sdk_meter_builder = sdk_meter_builder.with_reader(reader);

        Ok((sdk_meter_builder, runtime))
    }

    fn configure_grpc_otlp_exporter(
        otlp_config: &OtlpExporterConfig,
        runtime: Option<tokio::runtime::Runtime>,
    ) -> Result<
        (
            opentelemetry_otlp::MetricExporter,
            Option<tokio::runtime::Runtime>,
        ),
        Error,
    > {
        // If there is a tokio runtime already, use it. Otherwise, create a new one.
        let tokio_runtime = match runtime {
            Some(rt) => rt,
            None => tokio::runtime::Runtime::new()
                .map_err(|e| Error::ConfigurationError(e.to_string()))?,
        };

        let tls_config_option = if let Some(tls) = &otlp_config.tls {
            Some(Self::get_tls_config(&tls.ca_file)?)
        } else {
            None
        };

        let exporter = tokio_runtime
            .block_on(async {
                let mut builder = opentelemetry_otlp::MetricExporter::builder()
                    .with_tonic()
                    .with_endpoint(&otlp_config.endpoint)
                    .with_temporality(Self::to_sdk_temporality(&otlp_config.temporality));

                if let Some(tls_config) = tls_config_option {
                    builder = builder.with_tls_config(tls_config)
                }

                builder.build()
            })
            .map_err(|e| Error::ConfigurationError(e.to_string()))?;
        Ok((exporter, Some(tokio_runtime)))
    }

    fn get_tls_config(ca_path: &str) -> Result<ClientTlsConfig, Error> {
        // Read the CA certificate from a file
        let ca_cert = std::fs::read_to_string(ca_path).map_err(|e| {
            Error::ConfigurationError(format!(
                "Failed to read CA certificate file from {}: {}",
                ca_path, e
            ))
        })?;
        let ca_cert = Certificate::from_pem(ca_cert);

        // Create TLS configuration with the CA certificate
        let tls_config = ClientTlsConfig::new().ca_certificate(ca_cert);
        Ok(tls_config)
    }

    fn configure_http_exporter(
        otlp_config: &OtlpExporterConfig,
        protocol: Protocol,
    ) -> Result<opentelemetry_otlp::MetricExporter, Error> {
        let mut builder = opentelemetry_otlp::MetricExporter::builder().with_http();
        builder = builder
            .with_protocol(protocol)
            .with_endpoint(&otlp_config.endpoint)
            .with_temporality(Self::to_sdk_temporality(&otlp_config.temporality));

        if let Some(_tls_config) = &otlp_config.tls {
            // TODO: Add TLS configuration for HTTP exporter.
        }

        let exporter = builder
            .build()
            .map_err(|e| Error::ConfigurationError(e.to_string()))?;
        Ok(exporter)
    }

    fn to_sdk_temporality(config: &Temporality) -> opentelemetry_sdk::metrics::Temporality {
        match config {
            Temporality::Cumulative => opentelemetry_sdk::metrics::Temporality::Cumulative,
            Temporality::Delta => opentelemetry_sdk::metrics::Temporality::Delta,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry_sdk::metrics::SdkMeterProvider;
    use std::io::Write;

    #[test]
    fn test_configure_grpc_exporter() {
        let sdk_meter_builder = SdkMeterProvider::builder();
        let otlp_config = OtlpExporterConfig {
            protocol: OtlpProtocol::Grpc,
            endpoint: "http://localhost:4317".to_string(),
            temporality: Temporality::Cumulative,
            tls: None,
        };
        let result = OtlpExporterProvider::configure_otlp_metric_exporter(
            sdk_meter_builder,
            &otlp_config,
            &std::time::Duration::from_secs(10),
            None,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_configure_http_binary_exporter() {
        let sdk_meter_builder = SdkMeterProvider::builder();
        let otlp_config = OtlpExporterConfig {
            protocol: OtlpProtocol::HttpBinary,
            endpoint: "http://localhost:4318/v1/metrics".to_string(),
            temporality: Temporality::Cumulative,
            tls: None,
        };
        let result = OtlpExporterProvider::configure_otlp_metric_exporter(
            sdk_meter_builder,
            &otlp_config,
            &std::time::Duration::from_secs(10),
            None,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_configure_http_json_exporter() {
        let sdk_meter_builder = SdkMeterProvider::builder();
        let otlp_config = OtlpExporterConfig {
            protocol: OtlpProtocol::HttpJson,
            endpoint: "http://localhost:4318/v1/metrics".to_string(),
            temporality: Temporality::Cumulative,
            tls: None,
        };
        let result = OtlpExporterProvider::configure_otlp_metric_exporter(
            sdk_meter_builder,
            &otlp_config,
            &std::time::Duration::from_secs(10),
            None,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_to_sdk_temporality() {
        assert_eq!(
            OtlpExporterProvider::to_sdk_temporality(&Temporality::Cumulative),
            opentelemetry_sdk::metrics::Temporality::Cumulative
        );
        assert_eq!(
            OtlpExporterProvider::to_sdk_temporality(&Temporality::Delta),
            opentelemetry_sdk::metrics::Temporality::Delta
        );
    }

    #[test]
    fn test_get_tls_config_invalid_path() {
        let result = OtlpExporterProvider::get_tls_config("invalid/path/to/ca.pem");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_tls_config_with_ca() {
        let temp_ca_file = create_test_ca_file();
        let ca_path = temp_ca_file.path().to_str().unwrap();

        let result = OtlpExporterProvider::get_tls_config(ca_path);
        assert!(result.is_ok());
    }

    fn create_test_ca_file() -> tempfile::NamedTempFile {
        let ca_content = r#"-----BEGIN CERTIFICATE-----
fake-ca-certificate-content
-----END CERTIFICATE-----"#;

        let mut temp_file =
            tempfile::NamedTempFile::new().expect("Failed to create temporary file");

        temp_file
            .write_all(ca_content.as_bytes())
            .expect("Failed to write CA certificate to temp file");

        temp_file.flush().expect("Failed to flush temp file");

        temp_file
    }
}
