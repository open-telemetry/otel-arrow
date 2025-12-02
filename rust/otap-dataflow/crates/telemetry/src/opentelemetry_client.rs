// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Opentelemetry SDK integration for telemetry collection and reporting as a client.

use std::time::Duration;

use opentelemetry::{KeyValue, global};
use opentelemetry_sdk::{
    Resource,
    metrics::{PeriodicReader, SdkMeterProvider},
};
use otap_df_config::pipeline::{
    MetricsPeriodicExporterConfig, MetricsReaderConfig, TelemetryConfig,
};

/// Configure the global OpenTelemetry SDK according to the provided telemetry config.
pub fn configure_otel_sdk(config: &TelemetryConfig) {
    let mut sdk_meter_builder = SdkMeterProvider::builder();

    // TODO: Load from config
    // In the meantime, only configure it if there is one console metric reader..

    let metric_readers = &config.metrics.readers;
    for reader in metric_readers {
        match reader {
            MetricsReaderConfig::Periodic(periodic_config) => {
                let interval = &periodic_config.interval;
                match &periodic_config.exporter {
                    MetricsPeriodicExporterConfig::Console(_console_config) => {
                        let exporter = opentelemetry_stdout::MetricExporter::default();
                        let reader = PeriodicReader::builder(exporter)
                            .with_interval(Duration::from_millis(*interval))
                            .build();
                        sdk_meter_builder = sdk_meter_builder.with_reader(reader);
                    }
                    _ => {
                        // Ignore other exporters
                    }
                }
            }
            _ => {
                // Ignore other readers
            }
        }
    }

    let resource_attributes = &config.resource;
    if !resource_attributes.is_empty() {
        let mut sdk_resource_builder = Resource::builder();
        for (k, v) in resource_attributes.iter() {
            sdk_resource_builder =
                sdk_resource_builder.with_attribute(KeyValue::new(k.clone(), v.clone()));
        }
        let sdk_resource = sdk_resource_builder.build();
        sdk_meter_builder = sdk_meter_builder.with_resource(sdk_resource);
    }

    let sdk_meter_provider = sdk_meter_builder.build();

    global::set_meter_provider(sdk_meter_provider);
}

#[cfg(test)]
mod tests {
    use otap_df_config::pipeline::{
        ConsoleExporterConfig, MetricsConfig, MetricsReaderPeriodicConfig,
    };

    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_configure_minimal_otel_sdk() {
        let config = TelemetryConfig::default();
        configure_otel_sdk(&config);
        let meter = global::meter("test-meter");

        let counter = meter.u64_counter("test-counter").build();
        counter.add(1, &[]);
    }

    #[test]
    fn test_configure_otel_sdk() {
        let mut resource = std::collections::HashMap::new();
        _ = resource.insert("service.name".to_string(), "test-service".to_string());

        let metrics_config = MetricsConfig {
            readers: vec![MetricsReaderConfig::Periodic(MetricsReaderPeriodicConfig {
                exporter: MetricsPeriodicExporterConfig::Console(ConsoleExporterConfig {
                    temporality: None,
                }),
                interval: 10,
            })],
        };

        let config = TelemetryConfig {
            reporting_channel_size: 10,
            flush_interval: Duration::from_millis(10),
            metrics: metrics_config,
            resource,
        };
        configure_otel_sdk(&config);
        let meter = global::meter("test-meter");

        let counter = meter.u64_counter("test-counter").build();
        counter.add(1, &[]);

        // Sleep to allow the periodic reader to export metrics
        thread::sleep(Duration::from_millis(20));
    }
}
