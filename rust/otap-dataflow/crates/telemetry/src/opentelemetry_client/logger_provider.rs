// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configures the OpenTelemetry logger provider based on the provided configuration.

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
use tracing::level_filters::LevelFilter;
use tracing::Level;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt};
use std::future::Future;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use tokio_util::bytes::Bytes;

use crate::error::Error;
use crate::tracing_integration::{
    OtlpTracingLayer, OtlpBytesFormattingLayer, OtlpBytesChannel
};
use otap_df_pdata::otlp::stateful_encoder::StatefulOtlpEncoder;

/// Writer that routes to stdout or stderr based on log level.
///
/// This implements the MakeWriter trait to provide different writers
/// for different severity levels:
/// - TRACE, DEBUG, INFO → stdout
/// - WARN, ERROR → stderr
#[derive(Clone)]
struct LevelBasedWriter;

impl LevelBasedWriter {
    fn new() -> Self {
        LevelBasedWriter
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for LevelBasedWriter {
    type Writer = Box<dyn Write + 'a>;

    fn make_writer(&'a self) -> Self::Writer {
        Box::new(io::stdout())
    }

    fn make_writer_for(&'a self, meta: &tracing::Metadata<'_>) -> Self::Writer {
        match *meta.level() {
            Level::ERROR | Level::WARN => Box::new(io::stderr()),
            _ => Box::new(io::stdout()),
        }
    }
}

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
    ) -> Result<LoggerProvider, Error> {
        let mut sdk_logger_builder = SdkLoggerProvider::builder().with_resource(sdk_resource);

        let mut runtime: Option<tokio::runtime::Runtime> = initial_runtime;

        let log_processors = &logger_config.processors;

        for processor in log_processors {
            (sdk_logger_builder, runtime) =
                Self::configure_log_processor(sdk_logger_builder, processor, runtime)?;
        }

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

        // Set up our custom OTLP-based tracing (replaces OpenTelemetryTracingBridge)
        // This uses synchronous OTLP encoding → formatting for console output
        Self::init_console_tracing_with_filter(filter);

        // Note: We keep the sdk_logger_provider for applications that want to use
        // the OpenTelemetry SDK directly (for OTLP/stdout exporters, not for tracing)

        Ok(LoggerProvider {
            sdk_logger_provider,
            runtime,
        })
    }

    /// Consume the LoggerProvider and return its components.
    pub fn into_parts(self) -> (SdkLoggerProvider, Option<tokio::runtime::Runtime>) {
        (self.sdk_logger_provider, self.runtime)
    }

    /// Initialize console tracing with our custom OTLP-based formatter.
    ///
    /// This sets up synchronous OTLP encoding → decoding → formatting:
    /// - All tracing events are encoded to OTLP bytes
    /// - Immediately decoded and formatted to console
    /// - INFO/DEBUG/TRACE → stdout, WARN/ERROR → stderr
    /// - Colorized output with ISO8601 timestamps and thread names
    ///
    /// This replaces the OpenTelemetryTracingBridge which would send events
    /// through the OpenTelemetry SDK's batching/exporting pipeline.
    fn init_console_tracing_with_filter(filter: EnvFilter) {
        // Create a stateful encoder (shared across all events)
        let encoder = Arc::new(Mutex::new(StatefulOtlpEncoder::new(4096)));
        
        // Create the formatter that will decode OTLP bytes and write to stdout/stderr
        let formatter = Arc::new(
            OtlpBytesFormattingLayer::new(LevelBasedWriter::new())
                .with_ansi(true)
                .with_timestamp(true)
                .with_level(true)
                .with_target(true)
                .with_event_name(false) // Don't show event_name by default (less noise)
                .with_thread_names(true) // Show thread names like the original fmt layer
        );
        
        // Create the OTLP layer that captures events, encodes them, and formats synchronously
        let encoder_clone = encoder.clone();
        let formatter_clone = formatter.clone();
        
        let otlp_layer = OtlpTracingLayer::new(move |log_record| {
            // Encode to OTLP bytes (synchronous)
            let mut enc = encoder_clone.lock().unwrap();
            let resource_bytes = Vec::new();
            let target = log_record.target();
            
            if let Ok(_) = enc.encode_log_record(&log_record, &resource_bytes, target) {
                // Flush and get the bytes
                let otlp_bytes = enc.flush();
                
                // Format immediately (synchronous - no channel)
                let _ = formatter_clone.format_otlp_bytes(otlp_bytes.as_ref());
            }
        });
        
        // Try to initialize the global subscriber. In tests, this may fail if already set,
        // which is acceptable as we're only validating the configuration works.
        let _ = tracing_subscriber::registry()
            .with(otlp_layer)
            .with(filter)
            .try_init();
    }

    /// Initialize default console tracing for applications.
    ///
    /// This is a convenience method for applications that want simple console
    /// logging without configuring the full OpenTelemetry pipeline.
    ///
    /// Uses the same OTLP-based architecture but with a default filter:
    /// - INFO level for OTAP components
    /// - WARN level for third-party libraries
    /// - Respects RUST_LOG environment variable if set
    pub fn init_default_console_tracing() {
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| {
                // Default: INFO level for df_engine components, WARN for third-party
                EnvFilter::new("otap_df=info,warn")
            });
        
        Self::init_console_tracing_with_filter(filter);
    }

    /// Initialize channel-based tracing for normal operation.
    ///
    /// This transitions from synchronous console logging to multi-threaded channel-based logging.
    /// Call this after initial startup when the admin runtime is ready.
    ///
    /// This sets up:
    /// - Global tracing subscriber that encodes to OTLP bytes and sends to channel
    /// - Returns OtlpBytesChannel for spawning consumer task in admin runtime
    ///
    /// The global subscriber can block the caller when channel is full, providing backpressure.
    /// This is acceptable for 3rd party instrumentation where threaded logging is the norm.
    ///
    /// # Arguments
    /// * `channel_capacity` - Bounded channel capacity (e.g., 1000)
    /// * `filter` - Environment filter for log levels
    ///
    /// # Returns
    /// Returns OtlpBytesChannel - split it and spawn consumer task in admin runtime
    ///
    /// # Example
    /// ```ignore
    /// // IMPORTANT: Log the transition BEFORE init (goes to old synchronous subscriber)
    /// tracing::info!(
    ///     mode = "channel-based",
    ///     capacity = 1000,
    ///     "Transitioning to multi-threaded channel-based logging for 3rd party instrumentation"
    /// );
    /// 
    /// let channel = LoggerProvider::init_channel_based_tracing(1000, filter);
    /// let receiver = channel.into_receiver();
    /// 
    /// // Spawn consumer in admin runtime (all subsequent events go here)
    /// runtime.spawn(async move {
    ///     LoggerProvider::run_console_formatter_task(receiver).await;
    /// });
    /// ```
    pub fn init_channel_based_tracing(
        channel_capacity: usize,
        filter: EnvFilter,
    ) -> OtlpBytesChannel {
        let channel = OtlpBytesChannel::new(channel_capacity);
        let sender = channel.sender().clone();
        
        // Create encoder (one per global subscriber)
        let encoder = Arc::new(Mutex::new(StatefulOtlpEncoder::new(4096)));
        
        // Create the OTLP layer that captures events, encodes them, and sends to channel
        let otlp_layer = OtlpTracingLayer::new(move |log_record| {
            // Encode to OTLP bytes (synchronous)
            let mut enc = encoder.lock().unwrap();
            let resource_bytes = Vec::new();
            let target = log_record.target();
            
            if let Ok(_) = enc.encode_log_record(&log_record, &resource_bytes, target) {
                // Flush and get the bytes
                let otlp_bytes = enc.flush();
                
                // Send to channel (can block caller if channel is full - provides backpressure)
                let _ = sender.send(otlp_bytes);
            }
        });
        
        // Initialize the global subscriber
        let _ = tracing_subscriber::registry()
            .with(otlp_layer)
            .with(filter)
            .try_init();
        
        channel
    }

    /// Run the console formatter task that consumes OTLP bytes from channel.
    ///
    /// This should be spawned as a task in the admin runtime:
    /// ```ignore
    /// runtime.spawn(async move {
    ///     LoggerProvider::run_console_formatter_task(receiver).await;
    /// });
    /// ```
    ///
    /// The task will run until the channel is closed (all senders dropped).
    /// In production, this runs for the lifetime of the application.
    ///
    /// This provides single-threaded formatting in the admin runtime.
    /// Use this for human-readable console output.
    pub async fn run_console_formatter_task(receiver: mpsc::Receiver<Bytes>) {
        let formatter = OtlpBytesFormattingLayer::new(LevelBasedWriter::new())
            .with_ansi(true)
            .with_timestamp(true)
            .with_level(true)
            .with_target(true)
            .with_event_name(false)
            .with_thread_names(true);
        
        // Run in a blocking task since mpsc::Receiver::recv() is blocking
        let _ = tokio::task::spawn_blocking(move || {
            while let Ok(otlp_bytes) = receiver.recv() {
                let _ = formatter.format_otlp_bytes(&otlp_bytes);
            }
        }).await;
    }

    /// Run the OTLP bytes forwarder task that consumes from channel.
    ///
    /// This should be spawned as a task in the admin runtime:
    /// ```ignore
    /// runtime.spawn(async move {
    ///     LoggerProvider::run_otlp_forwarder_task(receiver, sender).await;
    /// });
    /// ```
    ///
    /// This forwards OTLP bytes to the internal telemetry receiver which bridges
    /// to the OTAP pipeline. The internal receiver can then export via the
    /// built-in OTLP exporter or any other OTAP exporter.
    ///
    /// # Arguments
    /// * `receiver` - Channel receiver from global tracing subscriber
    /// * `forward_fn` - Async function to forward OTLP bytes (e.g., to internal telemetry receiver)
    pub async fn run_otlp_forwarder_task<F, Fut>(
        receiver: mpsc::Receiver<Bytes>,
        mut forward_fn: F,
    )
    where
        F: FnMut(Bytes) -> Fut + Send + 'static,
        Fut: Future<Output = Result<(), Error>> + Send,
    {
        // Run in a blocking task since mpsc::Receiver::recv() is blocking
        let _ = tokio::task::spawn_blocking(move || {
            while let Ok(otlp_bytes) = receiver.recv() {
                // We need to block on the async forward_fn from within spawn_blocking
                // This is acceptable since this is the dedicated logging task
                let rt = tokio::runtime::Handle::current();
                let result = rt.block_on(forward_fn(otlp_bytes));
                if result.is_err() {
                    // If forwarding fails, we could fall back to stderr or drop
                    // For now, just continue (dropping the event)
                    eprintln!("Failed to forward OTLP bytes to telemetry receiver");
                }
            }
        }).await;
    }

    // Note: Support for OpenTelemetry SDK exporters (run_otel_sdk_exporter_task) has been
    // removed temporarily. It required decoding OTLP bytes back to SdkLogRecord, which had
    // type compatibility issues. This can be added back in the future if needed.

    /// Initialize channel-based tracing with default filter.
    ///
    /// Convenience method that uses default filter (INFO for OTAP, WARN for third-party).
    ///
    /// # Arguments
    /// * `channel_capacity` - Bounded channel capacity (e.g., 1000)
    ///
    /// # Returns
    /// Returns OtlpBytesChannel - call `.into_receiver()` and spawn consumer task
    pub fn init_default_channel_based_tracing(
        channel_capacity: usize,
    ) -> OtlpBytesChannel {
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| {
                EnvFilter::new("otap_df=info,warn")
            });
        
        Self::init_channel_based_tracing(channel_capacity, filter)
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
            internal_collection: Default::default(),
        };
        let logger_provider = LoggerProvider::configure(resource, &logger_config, None)?;
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
            internal_collection: Default::default(),
        };
        let logger_provider = LoggerProvider::configure(resource, &logger_config, None)?;
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
            internal_collection: Default::default(),
        };
        let logger_provider = LoggerProvider::configure(resource, &logger_config, None)?;
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
