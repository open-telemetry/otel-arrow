// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configures the OpenTelemetry logger provider based on the provided configuration.

pub mod internal_exporter;

use opentelemetry_appender_tracing::layer;
use opentelemetry_otlp::{Protocol, WithExportConfig};
use opentelemetry_sdk::{Resource, logs::SdkLoggerProvider};
use otap_df_config::pipeline::service::telemetry::{
    logs::{
        LogLevel, LogsConfig,
        processors::{
            BatchLogProcessorConfig,
            batch::{LogBatchProcessorExporterConfig, otlp::OtlpExporterConfig},
        },
    },
    metrics::readers::periodic::otlp::OtlpProtocol,
};
use otap_df_pdata::OtapPayload;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt};

use crate::error::Error;

/// Provider for configuring OpenTelemetry Logger.
pub struct LoggerProvider {
    sdk_logger_provider: SdkLoggerProvider,
    runtime: Option<tokio::runtime::Runtime>,
}

impl LoggerProvider {
    /// Initializes internal logging for the OTAP engine.
    ///
    /// The log level can be controlled via:
    /// 1. The `logs.level` config setting (off, debug, info, warn, error)
    /// 2. The `RUST_LOG` environment variable for fine-grained control
    ///
    /// When `RUST_LOG` is set, it takes precedence and allows filtering by target.
    /// Example: `RUST_LOG=info,h2=warn,hyper=warn` enables info level but silences
    /// noisy HTTP/2 and hyper logs.
    ///
    /// TODO: The engine uses a thread-per-core model
    /// and is NUMA aware.
    /// The fmt::init() here is truly global, and hence
    /// this will be a source of contention.
    /// We need to evaluate alternatives:
    ///
    /// 1. Set up per thread subscriber.
    ///    ```ignore
    ///    // start of thread
    ///    let _guard = tracing::subscriber::set_default(subscriber);
    ///    // now, with this thread, all tracing calls will go to this subscriber
    ///    // eliminating contention.
    ///    // end of thread
    ///    ```
    ///
    /// 2. Use custom subscriber that batches logs in thread-local buffer, and
    ///    flushes them periodically.
    ///
    /// The TODO here is to evaluate these options and implement one of them.
    /// As of now, this causes contention, and we just need to accept temporarily.
    ///
    /// TODO: Evaluate also alternatives for the contention caused by the global
    /// OpenTelemetry logger provider added as layer.
    pub fn configure(
        sdk_resource: Resource,
        logger_config: &LogsConfig,
        initial_runtime: Option<tokio::runtime::Runtime>,
        sender: crossbeam_channel::Sender<OtapPayload>,
    ) -> Result<LoggerProvider, Error> {
        let mut sdk_logger_builder = SdkLoggerProvider::builder().with_resource(sdk_resource);

        let mut runtime: Option<tokio::runtime::Runtime> = initial_runtime;

        let log_processors = &logger_config.processors;

        for processor in log_processors {
            (sdk_logger_builder, runtime) =
                Self::configure_log_processor(sdk_logger_builder, processor, runtime)?;
        }

        sdk_logger_builder = Self::configure_internal_logs_exporter(sdk_logger_builder, sender)?;

        let sdk_logger_provider = sdk_logger_builder.build();

        let level = match logger_config.level {
            LogLevel::Off => LevelFilter::OFF,
            LogLevel::Debug => LevelFilter::DEBUG,
            LogLevel::Info => LevelFilter::INFO,
            LogLevel::Warn => LevelFilter::WARN,
            LogLevel::Error => LevelFilter::ERROR,
        };

        // If RUST_LOG is set, use it for fine-grained control.
        // Otherwise, fall back to the config level with some noisy dependencies silenced.
        // Users can override by setting RUST_LOG explicitly.
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            // Default filter: use config level, but silence known noisy HTTP dependencies
            EnvFilter::new(format!("{level},h2=off,hyper=off"))
        });

        // Formatting layer
        let fmt_layer = tracing_subscriber::fmt::layer().with_thread_names(true);

        let sdk_layer = layer::OpenTelemetryTracingBridge::new(&sdk_logger_provider);

        // Try to initialize the global subscriber. In tests, this may fail if already set,
        // which is acceptable as we're only validating the configuration works.
        let _ = tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .with(sdk_layer)
            .try_init();

        Ok(LoggerProvider {
            sdk_logger_provider,
            runtime,
        })
    }

    /// Consume the LoggerProvider and return its components.
    pub fn into_parts(self) -> (SdkLoggerProvider, Option<tokio::runtime::Runtime>) {
        (self.sdk_logger_provider, self.runtime)
    }

    fn configure_log_processor(
        sdk_logger_builder: opentelemetry_sdk::logs::LoggerProviderBuilder,
        processor_config: &otap_df_config::pipeline::service::telemetry::logs::processors::LogProcessorConfig,
        runtime: Option<tokio::runtime::Runtime>,
    ) -> Result<
        (
            opentelemetry_sdk::logs::LoggerProviderBuilder,
            Option<tokio::runtime::Runtime>,
        ),
        Error,
    > {
        match processor_config {
            otap_df_config::pipeline::service::telemetry::logs::processors::LogProcessorConfig::Batch(
                batch_config,
            ) => {
                Self::configure_batch_log_processor(
                    sdk_logger_builder,
                    batch_config,
                    runtime,
                )
            }
        }
    }

    fn configure_batch_log_processor(
        mut sdk_logger_builder: opentelemetry_sdk::logs::LoggerProviderBuilder,
        batch_config: &BatchLogProcessorConfig,
        mut runtime: Option<tokio::runtime::Runtime>,
    ) -> Result<
        (
            opentelemetry_sdk::logs::LoggerProviderBuilder,
            Option<tokio::runtime::Runtime>,
        ),
        Error,
    > {
        match batch_config.exporter {
            LogBatchProcessorExporterConfig::Console => {
                sdk_logger_builder = Self::configure_console_logs_exporter(sdk_logger_builder)?
            }
            LogBatchProcessorExporterConfig::Otlp(ref otlp_config) => {
                let (builder, rt) =
                    Self::configure_otlp_logs_exporter(sdk_logger_builder, otlp_config, runtime)?;
                sdk_logger_builder = builder;
                runtime = rt;
            }
        }
        Ok((sdk_logger_builder, runtime))
    }

    fn configure_console_logs_exporter(
        mut sdk_logger_builder: opentelemetry_sdk::logs::LoggerProviderBuilder,
    ) -> Result<opentelemetry_sdk::logs::LoggerProviderBuilder, Error> {
        let exporter = opentelemetry_stdout::LogExporter::default();
        sdk_logger_builder = sdk_logger_builder.with_batch_exporter(exporter);
        Ok(sdk_logger_builder)
    }

    fn configure_otlp_logs_exporter(
        mut sdk_logger_builder: opentelemetry_sdk::logs::LoggerProviderBuilder,
        otlp_config: &OtlpExporterConfig,
        mut runtime: Option<tokio::runtime::Runtime>,
    ) -> Result<
        (
            opentelemetry_sdk::logs::LoggerProviderBuilder,
            Option<tokio::runtime::Runtime>,
        ),
        Error,
    > {
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
        sdk_logger_builder = sdk_logger_builder.with_batch_exporter(exporter);

        Ok((sdk_logger_builder, runtime))
    }

    fn configure_grpc_otlp_exporter(
        otlp_config: &OtlpExporterConfig,
        runtime: Option<tokio::runtime::Runtime>,
    ) -> Result<
        (
            opentelemetry_otlp::LogExporter,
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
                opentelemetry_otlp::LogExporter::builder()
                    .with_tonic()
                    .with_endpoint(&otlp_config.endpoint)
                    .build()
            })
            .map_err(|e| Error::ConfigurationError(e.to_string()))?;
        Ok((exporter, Some(tokio_runtime)))
    }

    fn configure_http_exporter(
        otlp_config: &OtlpExporterConfig,
        protocol: Protocol,
    ) -> Result<opentelemetry_otlp::LogExporter, Error> {
        let exporter = opentelemetry_otlp::LogExporter::builder()
            .with_http()
            .with_protocol(protocol)
            .with_endpoint(&otlp_config.endpoint)
            .build()
            .map_err(|e| Error::ConfigurationError(e.to_string()))?;
        Ok(exporter)
    }

    fn configure_internal_logs_exporter(
        mut sdk_logger_builder: opentelemetry_sdk::logs::LoggerProviderBuilder,
        sender: crossbeam_channel::Sender<OtapPayload>,
    ) -> Result<opentelemetry_sdk::logs::LoggerProviderBuilder, Error> {
        let exporter = internal_exporter::InternalLogsExporter::new(sender);
        sdk_logger_builder = sdk_logger_builder.with_batch_exporter(exporter);
        Ok(sdk_logger_builder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::error;

    #[test]
    fn test_logger_provider_configure_console_exporter() -> Result<(), Error> {
        let resource = Resource::builder().build();
        let logger_config = LogsConfig {
            level: LogLevel::Info,
            processors: vec![
                otap_df_config::pipeline::service::telemetry::logs::processors::LogProcessorConfig::Batch(
                    BatchLogProcessorConfig {
                        exporter: LogBatchProcessorExporterConfig::Console,
                    },
                ),
            ],
        };
        let sender = crossbeam_channel::unbounded().0; // Dummy sender for test
        let logger_provider = LoggerProvider::configure(resource, &logger_config, None, sender)?;
        let (sdk_logger_provider, _) = logger_provider.into_parts();

        emit_log();

        let result = sdk_logger_provider.shutdown();
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn test_logger_provider_configure_otlp_exporter() -> Result<(), Error> {
        let resource = Resource::builder().build();
        let logger_config = LogsConfig {
            level: LogLevel::Info,
            processors: vec![
                otap_df_config::pipeline::service::telemetry::logs::processors::LogProcessorConfig::Batch(
                    BatchLogProcessorConfig {
                        exporter: LogBatchProcessorExporterConfig::Otlp(
                            OtlpExporterConfig {
                                endpoint: "http://localhost:4317".to_string(),
                                protocol: OtlpProtocol::Grpc,
                            },
                        ),
                    },
                ),
            ],
        };
        let sender = crossbeam_channel::unbounded().0; // Dummy sender for test
        let logger_provider = LoggerProvider::configure(resource, &logger_config, None, sender)?;
        let (sdk_logger_provider, runtime_option) = logger_provider.into_parts();

        assert!(runtime_option.is_some());

        emit_log();

        let result = sdk_logger_provider.shutdown();
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn test_logger_provider_configure_default() -> Result<(), Error> {
        let resource = Resource::builder().build();
        let logger_config = LogsConfig {
            level: LogLevel::default(),
            processors: vec![],
        };
        let sender = crossbeam_channel::unbounded().0; // Dummy sender for test
        let logger_provider = LoggerProvider::configure(resource, &logger_config, None, sender)?;
        let (sdk_logger_provider, _) = logger_provider.into_parts();

        emit_log();

        let result = sdk_logger_provider.shutdown();
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn test_configure_http_binary_exporter() -> Result<(), Error> {
        let otlp_config = OtlpExporterConfig {
            endpoint: "http://localhost:4318".to_string(),
            protocol: OtlpProtocol::HttpBinary,
        };
        let exporter = LoggerProvider::configure_http_exporter(&otlp_config, Protocol::HttpBinary)?;
        drop(exporter); // just ensure it constructs without error
        Ok(())
    }

    #[test]
    fn test_configure_http_json_exporter() -> Result<(), Error> {
        let otlp_config = OtlpExporterConfig {
            endpoint: "http://localhost:4318".to_string(),
            protocol: OtlpProtocol::HttpJson,
        };
        let exporter = LoggerProvider::configure_http_exporter(&otlp_config, Protocol::HttpJson)?;
        drop(exporter); // just ensure it constructs without error
        Ok(())
    }

    fn emit_log() {
        error!(name: "my-event-name", target: "my-system", event_id = 20, user_name = "otel", user_email = "otel@opentelemetry.io");
    }
}
