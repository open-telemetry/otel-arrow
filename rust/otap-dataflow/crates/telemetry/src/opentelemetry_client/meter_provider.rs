// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configures the OpenTelemetry meter provider based on the provided configuration.

pub mod prometheus_exporter_provider;
pub mod views_provider;

use opentelemetry::global;
use opentelemetry_otlp::{Protocol, WithExportConfig};
use opentelemetry_sdk::{
    Resource,
    metrics::{MeterProviderBuilder, PeriodicReader, SdkMeterProvider},
};
use otap_df_config::pipeline::service::telemetry::metrics::{
    MetricsConfig,
    readers::{
        MetricsReaderConfig, Temporality,
        periodic::{
            MetricsPeriodicExporterConfig,
            otlp::{OtlpExporterConfig, OtlpProtocol},
        },
        pull::MetricsPullExporterConfig,
    },
};

use crate::{
    error::Error,
    opentelemetry_client::meter_provider::prometheus_exporter_provider::PrometheusExporterProvider,
};

/// Wrapper around the OpenTelemetry SDK meter provider and its runtime.
pub struct MeterProvider {
    sdk_meter_provider: SdkMeterProvider,
    runtime: Option<tokio::runtime::Runtime>,
}

impl MeterProvider {
    /// Configures the OpenTelemetry meter provider based on the provided configuration.
    pub fn configure(
        sdk_resource: Resource,
        metrics_config: &MetricsConfig,
        initial_runtime: Option<tokio::runtime::Runtime>,
    ) -> Result<MeterProvider, Error> {
        let mut sdk_meter_builder = SdkMeterProvider::builder();
        sdk_meter_builder = sdk_meter_builder.with_resource(sdk_resource);

        let mut runtime: Option<tokio::runtime::Runtime> = initial_runtime;

        let metric_readers = &metrics_config.readers;

        for reader in metric_readers {
            (sdk_meter_builder, runtime) =
                Self::configure_metric_reader(sdk_meter_builder, reader, runtime)?;
        }

        let views_config = &metrics_config.views;
        sdk_meter_builder =
            views_provider::ViewsProvider::configure(sdk_meter_builder, views_config.clone())?;

        let sdk_meter_provider = sdk_meter_builder.build();

        global::set_meter_provider(sdk_meter_provider.clone());

        Ok(MeterProvider {
            sdk_meter_provider,
            runtime,
        })
    }

    /// Consume the MeterProvider and return its components.
    pub fn into_parts(self) -> (SdkMeterProvider, Option<tokio::runtime::Runtime>) {
        (self.sdk_meter_provider, self.runtime)
    }

    fn configure_metric_reader(
        mut sdk_meter_builder: MeterProviderBuilder,
        reader_config: &MetricsReaderConfig,
        mut runtime: Option<tokio::runtime::Runtime>,
    ) -> Result<(MeterProviderBuilder, Option<tokio::runtime::Runtime>), Error> {
        match reader_config {
            MetricsReaderConfig::Periodic(periodic_config) => {
                let interval = &periodic_config.interval;
                match &periodic_config.exporter {
                    MetricsPeriodicExporterConfig::Console => {
                        sdk_meter_builder =
                            Self::configure_console_metric_exporter(sdk_meter_builder, interval)?;
                    }
                    MetricsPeriodicExporterConfig::Otlp(otlp_config) => {
                        (sdk_meter_builder, runtime) = Self::configure_otlp_metric_exporter(
                            sdk_meter_builder,
                            otlp_config,
                            interval,
                            runtime,
                        )?;
                    }
                }
                Ok((sdk_meter_builder, runtime))
            }
            MetricsReaderConfig::Pull(pull_config) => match &pull_config.exporter {
                MetricsPullExporterConfig::Prometheus(prometheus_config) => {
                    (sdk_meter_builder, runtime) = PrometheusExporterProvider::configure_exporter(
                        sdk_meter_builder,
                        prometheus_config,
                        runtime,
                    )?;
                    Ok((sdk_meter_builder, runtime))
                }
            },
        }
    }

    fn configure_otlp_metric_exporter(
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

        //sdk_meter_builder = sdk_meter_builder.with_periodic_exporter(exporter);
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

        let exporter = tokio_runtime
            .block_on(async {
                opentelemetry_otlp::MetricExporter::builder()
                    .with_tonic()
                    .with_endpoint(&otlp_config.endpoint)
                    .with_temporality(Self::to_sdk_temporality(&otlp_config.temporality))
                    .build()
            })
            .map_err(|e| Error::ConfigurationError(e.to_string()))?;
        Ok((exporter, Some(tokio_runtime)))
    }

    fn to_sdk_temporality(config: &Temporality) -> opentelemetry_sdk::metrics::Temporality {
        match config {
            Temporality::Cumulative => opentelemetry_sdk::metrics::Temporality::Cumulative,
            Temporality::Delta => opentelemetry_sdk::metrics::Temporality::Delta,
        }
    }

    fn configure_http_exporter(
        otlp_config: &OtlpExporterConfig,
        protocol: Protocol,
    ) -> Result<opentelemetry_otlp::MetricExporter, Error> {
        let exporter = opentelemetry_otlp::MetricExporter::builder()
            .with_http()
            .with_protocol(protocol)
            .with_endpoint(&otlp_config.endpoint)
            .with_temporality(Self::to_sdk_temporality(&otlp_config.temporality))
            .build()
            .map_err(|e| Error::ConfigurationError(e.to_string()))?;
        Ok(exporter)
    }

    fn configure_console_metric_exporter(
        mut sdk_meter_builder: MeterProviderBuilder,
        interval: &std::time::Duration,
    ) -> Result<MeterProviderBuilder, Error> {
        let exporter = opentelemetry_stdout::MetricExporter::default();
        let reader = PeriodicReader::builder(exporter)
            .with_interval(*interval)
            .build();
        sdk_meter_builder = sdk_meter_builder.with_reader(reader);
        Ok(sdk_meter_builder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_config::pipeline::service::telemetry::metrics::readers::{MetricsReaderPeriodicConfig, MetricsReaderPullConfig, pull::PrometheusExporterConfig};

    #[test]
    fn test_meter_provider_configure_with_non_runtime_readers() -> Result<(), Error> {
        let resource = Resource::builder().build();
        let metric_readers = vec![
            MetricsReaderConfig::Periodic(MetricsReaderPeriodicConfig {
                interval: std::time::Duration::from_secs(10),
                exporter: MetricsPeriodicExporterConfig::Console,
            }),
            MetricsReaderConfig::Periodic(MetricsReaderPeriodicConfig {
                interval: std::time::Duration::from_secs(15),
                exporter: MetricsPeriodicExporterConfig::Otlp(OtlpExporterConfig {
                    protocol: OtlpProtocol::HttpBinary,
                    endpoint: "http://localhost:4318/v1/metrics".to_string(),
                    temporality: Temporality::Cumulative,
                }),
            }),
            MetricsReaderConfig::Periodic(MetricsReaderPeriodicConfig {
                interval: std::time::Duration::from_secs(15),
                exporter: MetricsPeriodicExporterConfig::Otlp(OtlpExporterConfig {
                    protocol: OtlpProtocol::HttpJson,
                    endpoint: "http://localhost:4318".to_string(),
                    temporality: Temporality::Cumulative,
                }),
            }),
        ];

        let metrics_config = MetricsConfig {
            readers: metric_readers,
            views: Vec::new(),
        };

        let meter_provider = MeterProvider::configure(resource, &metrics_config, None)?;
        let (_sdk_meter_provider, runtime) = meter_provider.into_parts();
        assert!(runtime.is_none());
        Ok(())
    }

    #[test]
    fn test_meter_provider_configure_with_runtime_readers() -> Result<(), Error> {
        let resource = Resource::builder().build();
        let metric_readers = vec![
            MetricsReaderConfig::Periodic(MetricsReaderPeriodicConfig {
                interval: std::time::Duration::from_secs(10),
                exporter: MetricsPeriodicExporterConfig::Console,
            }),
            MetricsReaderConfig::Periodic(MetricsReaderPeriodicConfig {
                interval: std::time::Duration::from_secs(15),
                exporter: MetricsPeriodicExporterConfig::Otlp(OtlpExporterConfig {
                    protocol: OtlpProtocol::Grpc,
                    endpoint: "http://localhost:4318".to_string(),
                    temporality: Temporality::Cumulative,
                }),
            }),
            MetricsReaderConfig::Pull(MetricsReaderPullConfig {
                exporter: MetricsPullExporterConfig::Prometheus(
                    PrometheusExporterConfig {
                        host: "0.0.0.0".to_string(),
                        port: 9090,
                        path: "/metrics".to_string(),
                    }
                ),
            }),
        ];
        let metrics_config = MetricsConfig {
            readers: metric_readers,
            views: Vec::new(),
        };
        let meter_provider = MeterProvider::configure(resource, &metrics_config, None)?;
        let (_sdk_meter_provider, runtime) = meter_provider.into_parts();
        assert!(runtime.is_some());
        Ok(())
    }

    #[test]
    fn test_meter_provider_configure_empty() -> Result<(), Error> {
        let resource = Resource::builder().build();
        let metrics_config = MetricsConfig::default();

        let meter_provider = MeterProvider::configure(resource, &metrics_config, None)?;
        let (_sdk_meter_provider, runtime) = meter_provider.into_parts();
        assert!(runtime.is_none());
        Ok(())
    }

    #[test]
    fn test_configure_console_metric_exporter() {
        let sdk_meter_builder = SdkMeterProvider::builder();
        let interval = std::time::Duration::from_secs(10);
        let result = MeterProvider::configure_console_metric_exporter(sdk_meter_builder, &interval);
        assert!(result.is_ok());
    }

    #[test]
    fn test_configure_http_binary_exporter() {
        let sdk_meter_builder = SdkMeterProvider::builder();
        let otlp_config = OtlpExporterConfig {
            protocol: OtlpProtocol::HttpBinary,
            endpoint: "http://localhost:4318/v1/metrics".to_string(),
            temporality: Temporality::Cumulative,
        };
        let result = MeterProvider::configure_otlp_metric_exporter(
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
        };
        let result = MeterProvider::configure_otlp_metric_exporter(
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
            MeterProvider::to_sdk_temporality(&Temporality::Cumulative),
            opentelemetry_sdk::metrics::Temporality::Cumulative
        );
        assert_eq!(
            MeterProvider::to_sdk_temporality(&Temporality::Delta),
            opentelemetry_sdk::metrics::Temporality::Delta
        );
    }
}
