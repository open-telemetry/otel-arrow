// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configures the OpenTelemetry meter provider based on the provided configuration.

pub mod otlp_exporter_provider;
pub mod prometheus_exporter_provider;
pub mod views_provider;

use opentelemetry::global;
use opentelemetry_sdk::{
    Resource,
    metrics::{MeterProviderBuilder, PeriodicReader, SdkMeterProvider},
};
use otap_df_config::pipeline::telemetry::metrics::{
    MetricsConfig,
    readers::{
        MetricsReaderConfig,
        periodic::{MetricsPeriodicExporterType, otlp::OtlpExporterConfig},
        pull::{MetricsPullExporterType, PrometheusExporterConfig},
    },
};

use crate::{
    error::Error,
    otel_sdk::meter_provider::{
        otlp_exporter_provider::OtlpExporterProvider,
        prometheus_exporter_provider::PrometheusExporterProvider,
    },
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
                match &periodic_config.exporter.exporter_type {
                    MetricsPeriodicExporterType::Console => {
                        sdk_meter_builder =
                            Self::configure_console_metric_exporter(sdk_meter_builder, interval)?;
                    }
                    MetricsPeriodicExporterType::Otlp => {
                        match OtlpExporterConfig::from_config(&periodic_config.exporter.config) {
                            Ok(otlp_config) => {
                                (sdk_meter_builder, runtime) =
                                    OtlpExporterProvider::configure_otlp_metric_exporter(
                                        sdk_meter_builder,
                                        &otlp_config,
                                        interval,
                                        runtime,
                                    )?;
                            }
                            Err(e) => return Err(Error::ConfigurationError(e.to_string())),
                        }
                    }
                }
                Ok((sdk_meter_builder, runtime))
            }
            MetricsReaderConfig::Pull(pull_config) => match &pull_config.exporter.exporter_type {
                MetricsPullExporterType::Prometheus => {
                    match PrometheusExporterConfig::from_config(&pull_config.exporter.config) {
                        Ok(prometheus_config) => {
                            (sdk_meter_builder, runtime) =
                                PrometheusExporterProvider::configure_exporter(
                                    sdk_meter_builder,
                                    &prometheus_config,
                                    runtime,
                                )?;
                        }
                        Err(e) => return Err(Error::ConfigurationError(e.to_string())),
                    }
                    Ok((sdk_meter_builder, runtime))
                }
            },
        }
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
    use otap_df_config::pipeline::telemetry::metrics::readers::{
        MetricsReaderPeriodicConfig, MetricsReaderPullConfig,
        periodic::MetricsPeriodicExporterConfig,
        pull::{MetricsPullExporterConfig, MetricsPullExporterType},
    };

    #[test]
    fn test_meter_provider_configure_with_non_runtime_readers() -> Result<(), Error> {
        let resource = Resource::builder().build();
        let metric_readers = vec![
            MetricsReaderConfig::Periodic(MetricsReaderPeriodicConfig {
                interval: std::time::Duration::from_secs(10),
                exporter: MetricsPeriodicExporterConfig {
                    exporter_type: MetricsPeriodicExporterType::Console,
                    config: serde_json::Value::Null,
                },
            }),
            MetricsReaderConfig::Periodic(MetricsReaderPeriodicConfig {
                interval: std::time::Duration::from_secs(15),
                exporter: MetricsPeriodicExporterConfig {
                    exporter_type: MetricsPeriodicExporterType::Otlp,
                    config: serde_json::json!({
                        "protocol": "http/protobuf",
                        "endpoint": "http://localhost:4318/v1/metrics",
                        "temporality": "cumulative",
                        "tls": null
                    }),
                },
            }),
            MetricsReaderConfig::Periodic(MetricsReaderPeriodicConfig {
                interval: std::time::Duration::from_secs(15),
                exporter: MetricsPeriodicExporterConfig {
                    exporter_type: MetricsPeriodicExporterType::Otlp,
                    config: serde_json::json!({
                        "protocol": "http/json",
                        "endpoint": "http://localhost:4318",
                        "temporality": "cumulative",
                        "tls": null
                    }),
                },
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
                exporter: MetricsPeriodicExporterConfig {
                    exporter_type: MetricsPeriodicExporterType::Console,
                    config: serde_json::Value::Null,
                },
            }),
            MetricsReaderConfig::Periodic(MetricsReaderPeriodicConfig {
                interval: std::time::Duration::from_secs(15),
                exporter: MetricsPeriodicExporterConfig {
                    exporter_type: MetricsPeriodicExporterType::Otlp,
                    config: serde_json::json!({
                        "protocol": "grpc/protobuf",
                        "endpoint": "http://localhost:4317",
                        "temporality": "cumulative",
                        "tls": null
                    }),
                },
            }),
            MetricsReaderConfig::Pull(MetricsReaderPullConfig {
                exporter: MetricsPullExporterConfig {
                    exporter_type: MetricsPullExporterType::Prometheus,
                    config: serde_json::json!({
                        "host": "0.0.0.0",
                        "port": 9090,
                        "path": "/metrics"
                    }),
                },
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
}
