// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OpenTelemetry SDK integration for telemetry collection and reporting as a client.

pub mod logger_provider;
pub mod meter_provider;

use opentelemetry::KeyValue;
use opentelemetry_sdk::{Resource, logs::SdkLoggerProvider, metrics::SdkMeterProvider};
use otap_df_config::pipeline::service::telemetry::{
    AttributeValue, AttributeValueArray, TelemetryConfig,
    logs::{OutputMode, ProviderMode},
};

use crate::{
    LogsReceiver,
    error::Error,
    logs::{LogsCollector, LogsReporter, TelemetrySetup},
    telemetry_runtime::logger_provider::LoggerProvider,
    telemetry_runtime::meter_provider::MeterProvider,
};
use otap_df_config::pipeline::service::telemetry::logs::LogLevel;

/// Client for the OpenTelemetry SDK and internal telemetry settings.
///
/// This struct owns all telemetry infrastructure including:
/// - OpenTelemetry SDK meter and logger providers
/// - Internal logs reporter and receiver channels
/// - Optional logs collector for Direct output mode
pub struct TelemetryRuntime {
    /// The tokio runtime used to run the OpenTelemetry SDK OTLP exporter.
    /// The reference is kept to ensure the runtime lives as long as the client.
    _runtime: Option<tokio::runtime::Runtime>,
    meter_provider: SdkMeterProvider,
    logger_provider: Option<SdkLoggerProvider>,
    /// Reporter for sending logs through the internal channel.
    /// Present when global or engine provider mode needs a channel.
    logs_reporter: Option<LogsReporter>,
    /// Receiver for the internal logs channel (Internal output mode only).
    /// The ITR node consumes this to process internal telemetry.
    logs_receiver: Option<LogsReceiver>,
    /// Collector for Direct output mode. Must be spawned by the controller.
    logs_collector: Option<LogsCollector>,
    /// Deferred global subscriber setup. Must be initialized by controller
    /// AFTER the internal pipeline is started (so the channel is being consumed).
    global_setup: Option<TelemetrySetup>,
    /// Log level for the global subscriber.
    global_log_level: LogLevel,
    // TODO: Add traces providers.
}

impl TelemetryRuntime {
    /// Create a new OpenTelemetry client from the given configuration.
    ///
    /// Logging-specific notes:
    ///
    /// The log level can be controlled via:
    /// 1. The `logs.level` config setting (off, debug, info, warn, error)
    /// 2. The `RUST_LOG` environment variable for fine-grained control
    ///
    /// When `RUST_LOG` is set, it takes precedence and allows filtering by target.
    /// Example: `RUST_LOG=info,h2=warn,hyper=warn` enables info level but silences
    /// noisy HTTP/2 and hyper logs.
    ///
    /// The logs reporter is created internally based on the configuration:
    /// - For `Direct` output: creates reporter + collector (collector must be spawned)
    /// - For `Internal` output: creates reporter + receiver (receiver goes to ITR node)
    /// - For `Noop` output: no reporter is created
    ///
    /// The logger provider is configured when either global or engine providers
    /// are set to `OpenTelemetry`. This allows the engine to use the same SDK
    /// pipeline even when global uses a different logging strategy.
    pub fn new(config: &TelemetryConfig) -> Result<Self, Error> {
        let sdk_resource = Self::configure_resource(&config.resource);

        let runtime = None;

        let (meter_provider, runtime) =
            MeterProvider::configure(sdk_resource.clone(), &config.metrics, runtime)?.into_parts();

        // Determine if we need a logs reporter based on provider modes
        let providers_need_reporter = config.logs.providers.global.needs_reporter()
            || config.logs.providers.engine.needs_reporter();

        // Create the logs reporter, receiver, and collector based on output mode
        let (logs_reporter, logs_receiver, logs_collector) = if providers_need_reporter {
            match config.logs.output {
                OutputMode::Direct => {
                    // Direct mode: logs go to a collector that prints to console
                    let (collector, reporter) = LogsCollector::new(config.reporting_channel_size);
                    eprintln!("DEBUG: TelemetryRuntime::new - Direct mode, no receiver");
                    (Some(reporter), None, Some(collector))
                }
                OutputMode::Internal => {
                    // Internal mode: logs go through channel to ITR node
                    let (receiver, reporter) =
                        LogsCollector::channel(config.reporting_channel_size);
                    eprintln!("DEBUG: TelemetryRuntime::new - Internal mode, receiver created");
                    (Some(reporter), Some(receiver), None)
                }
                OutputMode::Noop => (None, None, None),
            }
        } else {
            (None, None, None)
        };

        // Check if either global or engine needs the OpenTelemetry logger provider
        let global_needs_otel = config.logs.providers.global == ProviderMode::OpenTelemetry;
        let engine_needs_otel = config.logs.providers.engine == ProviderMode::OpenTelemetry;

        // Configure the logger provider if either global or engine needs it
        let (logger_provider, runtime) = if global_needs_otel || engine_needs_otel {
            let (provider, rt) =
                LoggerProvider::configure(sdk_resource.clone(), &config.logs, runtime)?
                    .into_parts();
            (Some(provider), rt)
        } else {
            (None, runtime)
        };

        // Build the global setup but DO NOT initialize it yet.
        // The controller must call init_global_subscriber() after the internal
        // pipeline is started, so the channel receiver is being consumed.
        let global_setup = Self::make_telemetry_setup(
            config.logs.providers.global,
            logs_reporter.as_ref(),
            logger_provider.as_ref(),
        )?;

        Ok(Self {
            _runtime: runtime,
            meter_provider,
            logger_provider,
            logs_reporter,
            logs_receiver,
            logs_collector,
            global_setup: Some(global_setup),
            global_log_level: config.logs.level,
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

    /// Get a reference to the logs reporter.
    ///
    /// Returns `Some` when the configuration requires a channel-based reporter
    /// (global or engine provider is `Immediate`).
    #[must_use]
    pub fn logs_reporter(&self) -> Option<&LogsReporter> {
        self.logs_reporter.as_ref()
    }

    /// Take the logs receiver for the internal telemetry pipeline.
    ///
    /// Returns `Some` only when output mode is `Internal`. The receiver should
    /// be passed to the Internal Telemetry Receiver (ITR) node.
    ///
    /// This method takes ownership of the receiver (can only be called once).
    pub fn take_logs_receiver(&mut self) -> Option<LogsReceiver> {
        self.logs_receiver.take()
    }

    /// Take the logs collector for Direct output mode.
    ///
    /// Returns `Some` only when output mode is `Direct`. The collector should
    /// be spawned on a dedicated thread to process log records.
    ///
    /// This method takes ownership of the collector (can only be called once).
    pub fn take_logs_collector(&mut self) -> Option<LogsCollector> {
        self.logs_collector.take()
    }

    /// Initialize the global tracing subscriber.
    ///
    /// This MUST be called AFTER the internal pipeline is started (when using
    /// Internal output mode), so the channel receiver is being actively consumed.
    /// Otherwise, logs sent before the receiver starts will fill the channel buffer.
    ///
    /// For other output modes (Direct, Noop), this can be called at any time.
    pub fn init_global_subscriber(&mut self) {
        if let Some(setup) = self.global_setup.take() {
            if let Err(err) = setup.try_init_global(self.global_log_level) {
                crate::raw_error!("tracing.subscriber.init", error = err.to_string());
            }
        }
    }

    /// Create a `TelemetrySetup` for the given provider mode.
    ///
    /// This uses the runtime's shared `logs_reporter` and `logger_provider` to configure
    /// the setup for the given provider mode.
    ///
    /// # Panics
    /// Panics if the provider mode requires a resource that wasn't configured:
    /// - `Immediate` requires `logs_reporter` to be present
    /// - `OpenTelemetry` requires `logger_provider` to be present
    #[must_use]
    pub fn telemetry_setup_for(&self, provider_mode: ProviderMode) -> TelemetrySetup {
        Self::make_telemetry_setup(
            provider_mode,
            self.logs_reporter.as_ref(),
            self.logger_provider.as_ref(),
        )
        .expect("validated: provider mode resources should be configured")
    }

    /// Helper to create a TelemetrySetup from a ProviderMode and optional resources.
    ///
    /// Returns an error if the mode requires a resource that isn't provided.
    fn make_telemetry_setup(
        provider_mode: ProviderMode,
        logs_reporter: Option<&LogsReporter>,
        logger_provider: Option<&SdkLoggerProvider>,
    ) -> Result<TelemetrySetup, Error> {

        match provider_mode {
            ProviderMode::Noop => Ok(TelemetrySetup::Noop),
            ProviderMode::Raw => Ok(TelemetrySetup::Raw),
            ProviderMode::Immediate => {
                let reporter = logs_reporter.ok_or_else(|| {
                    Error::ConfigurationError(
                        "Immediate provider mode requires logs_reporter".into(),
                    )
                })?;
                Ok(TelemetrySetup::Immediate {
                    reporter: reporter.clone(),
                })
            }
            ProviderMode::OpenTelemetry => {
                let provider = logger_provider.ok_or_else(|| {
                    Error::ConfigurationError(
                        "OpenTelemetry provider mode requires logger_provider".into(),
                    )
                })?;
                Ok(TelemetrySetup::OpenTelemetry {
                    logger_provider: provider.clone(),
                })
            }
        }
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
    fn test_configure_minimal_telemetry_runtime() -> Result<(), Error> {
        let config = TelemetryConfig::default();
        let client = TelemetryRuntime::new(&config)?;
        let meter = global::meter("test-meter");

        let counter = meter.u64_counter("test-counter").build();
        counter.add(1, &[]);
        //There is nothing to assert here. The test validates that nothing panics/crashes

        client.shutdown()?;
        Ok(())
    }

    #[test]
    fn test_configure_telemetry_runtime() -> Result<(), Error> {
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
        let client = TelemetryRuntime::new(&config)?;
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
            TelemetryRuntime::to_sdk_value(&string_attr),
            opentelemetry::Value::String("example".into())
        );

        let bool_attr = AttributeValue::Bool(true);
        assert_eq!(
            TelemetryRuntime::to_sdk_value(&bool_attr),
            opentelemetry::Value::Bool(true)
        );

        let i64_attr = AttributeValue::I64(42);
        assert_eq!(
            TelemetryRuntime::to_sdk_value(&i64_attr),
            opentelemetry::Value::I64(42)
        );

        let f64_attr = AttributeValue::F64(PI);
        assert_eq!(
            TelemetryRuntime::to_sdk_value(&f64_attr),
            opentelemetry::Value::F64(PI)
        );

        let array_attr = AttributeValue::Array(AttributeValueArray::I64(vec![1, 2, 3]));
        assert_eq!(
            TelemetryRuntime::to_sdk_value(&array_attr),
            opentelemetry::Value::Array(opentelemetry::Array::I64(vec![1, 2, 3]))
        );
    }
}
