// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OpenTelemetry SDK integration for telemetry collection and reporting as a client.

pub mod logger_provider;
pub mod meter_provider;

use opentelemetry::KeyValue;
use opentelemetry_sdk::{Resource, logs::SdkLoggerProvider, metrics::SdkMeterProvider};
use otap_df_config::pipeline::service::telemetry::{
    AttributeValue, AttributeValueArray, TelemetryConfig, logs::LogLevel,
};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    error::Error,
    opentelemetry_client::{logger_provider::LoggerProvider, meter_provider::MeterProvider},
    self_tracing::{ConsoleWriter, RawLoggingLayer},
};

/// Client for the OpenTelemetry SDK.
pub struct OpentelemetryClient {
    /// The tokio runtime used to run the OpenTelemetry SDK OTLP exporter.
    /// The reference is kept to ensure the runtime lives as long as the client.
    _runtime: Option<tokio::runtime::Runtime>,
    meter_provider: SdkMeterProvider,
    logger_provider: Option<SdkLoggerProvider>,
    // TODO: Add traces providers.
}

// If RUST_LOG is set, use it for fine-grained control.
// Otherwise, fall back to the config level with some noisy dependencies silenced.
// Users can override by setting RUST_LOG explicitly.
fn get_env_filter(level: LogLevel) -> EnvFilter {
    let level = match level {
        LogLevel::Off => LevelFilter::OFF,
        LogLevel::Debug => LevelFilter::DEBUG,
        LogLevel::Info => LevelFilter::INFO,
        LogLevel::Warn => LevelFilter::WARN,
        LogLevel::Error => LevelFilter::ERROR,
    };

    EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        // Default filter: use config level, but silence known noisy HTTP dependencies
        EnvFilter::new(format!("{level},h2=off,hyper=off"))
    })
}

impl OpentelemetryClient {
    /// Create a new OpenTelemetry client from the given configuration.
    pub fn new(config: &TelemetryConfig) -> Result<Self, Error> {
        let sdk_resource = Self::configure_resource(&config.resource);

        let runtime = None;

        let (meter_provider, runtime) =
            MeterProvider::configure(sdk_resource.clone(), &config.metrics, runtime)?.into_parts();

        let tracing_setup = tracing_subscriber::registry().with(get_env_filter(config.logs.level));

        let (logger_provider, runtime) = if !config.logs.internal.enabled {
            let (logger_provider, runtime) =
                LoggerProvider::configure(sdk_resource, &config.logs, runtime)?.into_parts();

            // Tokio provides a console formatting layer, OTel
            // provides other behaviors.
            let fmt_layer = tracing_subscriber::fmt::layer().with_thread_names(true);
            let sdk_layer = opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge::new(
                &logger_provider,
            );

            // Try to initialize the global subscriber. In tests, this may fail if already set,
            // which is acceptable as we're only validating the configuration works.
            let _ = tracing_setup.with(fmt_layer).with(sdk_layer).try_init();
            (Some(logger_provider), runtime)
        } else {
            let writer = if std::env::var("NO_COLOR").is_ok() {
                ConsoleWriter::no_color()
            } else {
                ConsoleWriter::color()
            };
            // See comment above.
            let _ = tracing_setup.with(RawLoggingLayer::new(writer)).try_init();

            (None, runtime)
        };

        //TODO: Configure traces provider.

        Ok(Self {
            _runtime: runtime,
            meter_provider,
            logger_provider,
        })
    }

    fn configure_resource(
        resource_attributes: &std::collections::HashMap<String, AttributeValue>,
    ) -> Resource {
        let mut sdk_resource_builder = Resource::builder_empty();
        for (k, v) in resource_attributes.iter() {
            sdk_resource_builder = sdk_resource_builder
                .with_attribute(KeyValue::new(k.clone(), Self::to_sdk_value(v)));
        }
        sdk_resource_builder.build()
    }

    fn to_sdk_value(attr_value: &AttributeValue) -> opentelemetry::Value {
        match attr_value {
            AttributeValue::String(s) => opentelemetry::Value::String(s.clone().into()),
            AttributeValue::Bool(b) => opentelemetry::Value::Bool(*b),
            AttributeValue::I64(i) => opentelemetry::Value::I64(*i),
            AttributeValue::F64(f) => opentelemetry::Value::F64(*f),
            AttributeValue::Array(arr) => match arr {
                AttributeValueArray::String(array_s) => {
                    let sdk_values = array_s.iter().map(|s| s.clone().into()).collect();
                    opentelemetry::Value::Array(opentelemetry::Array::String(sdk_values))
                }
                AttributeValueArray::Bool(array_b) => {
                    let sdk_values = array_b.to_vec();
                    opentelemetry::Value::Array(opentelemetry::Array::Bool(sdk_values))
                }
                AttributeValueArray::I64(array_i) => {
                    let sdk_values = array_i.to_vec();
                    opentelemetry::Value::Array(opentelemetry::Array::I64(sdk_values))
                }
                AttributeValueArray::F64(array_f) => {
                    let sdk_values = array_f.to_vec();
                    opentelemetry::Value::Array(opentelemetry::Array::F64(sdk_values))
                }
            },
        }
    }

    /// Get a reference to the meter provider.
    #[must_use]
    pub fn meter_provider(&self) -> &SdkMeterProvider {
        &self.meter_provider
    }

    /// Get a reference to the logger provider.
    #[must_use]
    pub fn logger_provider(&self) -> &Option<SdkLoggerProvider> {
        &self.logger_provider
    }

    /// Shutdown the OpenTelemetry SDK.
    pub fn shutdown(&self) -> Result<(), Error> {
        let meter_shutdown_result = self.meter_provider().shutdown();
        let logger_provider_shutdown_result = self
            .logger_provider()
            .as_ref()
            .map(|x| x.shutdown())
            .transpose();

        if let Err(e) = meter_shutdown_result {
            return Err(Error::ShutdownError(e.to_string()));
        }

        if let Err(e) = logger_provider_shutdown_result {
            return Err(Error::ShutdownError(e.to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use opentelemetry::global;
    use otap_df_config::pipeline::service::telemetry::{
        AttributeValue,
        logs::LogsConfig,
        metrics::{
            MetricsConfig,
            readers::{
                MetricsReaderConfig, MetricsReaderPeriodicConfig,
                periodic::MetricsPeriodicExporterConfig,
            },
        },
    };

    use super::*;
    use std::{f64::consts::PI, time::Duration};

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
        _ = resource.insert(
            "service.name".to_string(),
            AttributeValue::String("test-service".to_string()),
        );

        let metrics_config = MetricsConfig {
            readers: vec![MetricsReaderConfig::Periodic(MetricsReaderPeriodicConfig {
                exporter: MetricsPeriodicExporterConfig::Console,
                interval: Duration::from_millis(10),
            })],
            views: Vec::new(),
        };

        let config = TelemetryConfig {
            reporting_channel_size: 10,
            reporting_interval: Duration::from_millis(10),
            metrics: metrics_config,
            logs: LogsConfig::default(),
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

    #[test]
    fn test_to_sdk_value() {
        let string_attr = AttributeValue::String("example".to_string());
        assert_eq!(
            OpentelemetryClient::to_sdk_value(&string_attr),
            opentelemetry::Value::String("example".into())
        );

        let bool_attr = AttributeValue::Bool(true);
        assert_eq!(
            OpentelemetryClient::to_sdk_value(&bool_attr),
            opentelemetry::Value::Bool(true)
        );

        let i64_attr = AttributeValue::I64(42);
        assert_eq!(
            OpentelemetryClient::to_sdk_value(&i64_attr),
            opentelemetry::Value::I64(42)
        );

        let f64_attr = AttributeValue::F64(PI);
        assert_eq!(
            OpentelemetryClient::to_sdk_value(&f64_attr),
            opentelemetry::Value::F64(PI)
        );

        let array_attr = AttributeValue::Array(AttributeValueArray::I64(vec![1, 2, 3]));
        assert_eq!(
            OpentelemetryClient::to_sdk_value(&array_attr),
            opentelemetry::Value::Array(opentelemetry::Array::I64(vec![1, 2, 3]))
        );
    }
}
