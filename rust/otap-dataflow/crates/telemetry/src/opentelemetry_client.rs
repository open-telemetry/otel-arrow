// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Opentelemetry SDK integration for telemetry collection and reporting as a client.

pub mod meter_provider;

use opentelemetry::KeyValue;
use opentelemetry_sdk::{Resource, metrics::SdkMeterProvider};
use otap_df_config::pipeline::TelemetryConfig;

use crate::{error::Error, opentelemetry_client::meter_provider::MeterProvider};

/// Client for the OpenTelemetry SDK.
pub struct OpentelemetryClient {
    /// The tokio runtime used to run the OpenTelemetry SDK.
    /// The reference is kept to ensure the runtime lives as long as the client.
    _runtime: Option<tokio::runtime::Runtime>,
    meter_provider: SdkMeterProvider,
    // TODO: Add traces and logs providers.
}

impl OpentelemetryClient {
    /// Create a new OpenTelemetry client from the given configuration.
    pub fn new(config: &TelemetryConfig) -> Result<Self, Error> {
        let sdk_resource = Self::configure_resource(&config.resource);

        let runtime = None;

        let meter_provider =
            MeterProvider::configure(sdk_resource, &config.metrics.readers, runtime)?;

        // Extract the meter provider and runtime by consuming the MeterProvider
        let (meter_provider, runtime) = meter_provider.deconstruct();

        //TODO: Configure traces and logs providers.

        Ok(Self {
            _runtime: runtime,
            meter_provider,
        })
    }

    fn configure_resource(
        resource_attributes: &std::collections::HashMap<String, String>,
    ) -> Resource {
        let mut sdk_resource_builder = Resource::builder();
        for (k, v) in resource_attributes.iter() {
            sdk_resource_builder =
                sdk_resource_builder.with_attribute(KeyValue::new(k.clone(), v.clone()));
        }
        sdk_resource_builder.build()
    }

    /// Get a reference to the meter provider.
    #[must_use]
    pub fn meter_provider(&self) -> &SdkMeterProvider {
        &self.meter_provider
    }

    /// Shutdown the OpenTelemetry SDK.
    pub fn shutdown(&self) -> Result<(), Error> {
        let meter_shutdown_result = self.meter_provider().shutdown();
        meter_shutdown_result.map_err(|e| Error::ShutdownError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use opentelemetry::global;
    use otap_df_config::pipeline::{
        MetricsConfig, MetricsPeriodicExporterConfig, MetricsReaderConfig,
        MetricsReaderPeriodicConfig,
    };

    use super::*;
    use std::time::Duration;

    #[test]
    fn test_configure_minimal_opentelemetry_client() -> Result<(), Error> {
        let config = TelemetryConfig::default();
        let client = OpentelemetryClient::new(&config)?;
        let meter = global::meter("test-meter");

        let counter = meter.u64_counter("test-counter").build();
        counter.add(1, &[]);
        //There is nothing to assert here. The test validates that nothing panics/crashes

        client.shutdown()?;
        Ok(())
    }

    #[test]
    fn test_configure_opentelemetry_client() -> Result<(), Error> {
        let mut resource = std::collections::HashMap::new();
        _ = resource.insert("service.name".to_string(), "test-service".to_string());

        let metrics_config = MetricsConfig {
            readers: vec![MetricsReaderConfig::Periodic(MetricsReaderPeriodicConfig {
                exporter: MetricsPeriodicExporterConfig::Console,
                interval: Duration::from_millis(10),
            })],
        };

        let config = TelemetryConfig {
            reporting_channel_size: 10,
            reporting_interval: Duration::from_millis(10),
            metrics: metrics_config,
            resource,
        };
        let client = OpentelemetryClient::new(&config)?;
        let meter = global::meter("test-meter");

        let counter = meter.u64_counter("test-counter").build();
        counter.add(1, &[]);
        //There is nothing to assert here. The test validates that nothing panics/crashes

        client.shutdown()?;
        Ok(())
    }
}
