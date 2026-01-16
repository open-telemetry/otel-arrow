// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Tokio tracing subscriber initialization.
//!
//! This module handles the setup of the global tokio tracing subscriber,
//! which is separate from OpenTelemetry SDK configuration. The tracing
//! subscriber determines how log events are captured and routed.

use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use otap_df_config::pipeline::service::telemetry::logs::LogLevel;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt};

/// Creates an `EnvFilter` for the given log level.
///
/// If `RUST_LOG` is set in the environment, it takes precedence for fine-grained control.
/// Otherwise, falls back to the config level with known noisy dependencies (h2, hyper) silenced.
#[must_use]
pub fn create_env_filter(level: LogLevel) -> EnvFilter {
    let level_filter = match level {
        LogLevel::Off => LevelFilter::OFF,
        LogLevel::Debug => LevelFilter::DEBUG,
        LogLevel::Info => LevelFilter::INFO,
        LogLevel::Warn => LevelFilter::WARN,
        LogLevel::Error => LevelFilter::ERROR,
    };

    EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        // Default filter: use config level, but silence known noisy HTTP dependencies
        EnvFilter::new(format!("{level_filter},h2=off,hyper=off"))
    })
}

/// Initializes the global tracing subscriber with OTel and fmt layers.
///
/// This sets up tracing to output to both console (fmt) and OpenTelemetry.
/// The log level can be controlled via:
/// 1. The `logs.level` config setting (off, debug, info, warn, error)
/// 2. The `RUST_LOG` environment variable for fine-grained control
///
/// When `RUST_LOG` is set, it takes precedence and allows filtering by target.
/// Example: `RUST_LOG=info,h2=warn,hyper=warn` enables info level but silences
/// noisy HTTP/2 and hyper logs.
///
/// # Notes on Contention
///
/// TODO: The engine uses a thread-per-core model and is NUMA aware.
/// The global subscriber here is truly global, and hence this will be a source
/// of contention. We need to evaluate alternatives:
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
pub fn init_global_subscriber(
    log_level: LogLevel,
    logger_provider: &SdkLoggerProvider,
) {
    let filter = create_env_filter(log_level);

    // Formatting layer
    let fmt_layer = tracing_subscriber::fmt::layer().with_thread_names(true);

    let sdk_layer = OpenTelemetryTracingBridge::new(logger_provider);

    // Try to initialize the global subscriber. In tests, this may fail if already set,
    // which is acceptable as we're only validating the configuration works.
    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .with(sdk_layer)
        .try_init();
}
