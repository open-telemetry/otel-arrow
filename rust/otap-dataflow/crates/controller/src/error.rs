// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Errors for the controller crate.

use miette::Diagnostic;
use otap_df_config::TopicName;
use otap_df_config::policy::CoreAllocation;
use otap_df_config::topic::TopicBackendKind;

/// Errors that can occur in the controller crate.
#[derive(thiserror::Error, Debug, Diagnostic)]
pub enum Error {
    /// A collection of errors that occurred during parsing or validating the configuration.
    #[error("Invalid configuration: {errors:?}")]
    InvalidConfiguration {
        /// A list of errors that occurred during parsing or validating the configuration.
        #[related]
        errors: Vec<otap_df_config::error::Error>,
    },

    /// An error originating from the admin module.
    #[error("Admin module error: {0}")]
    AdminError(#[from] otap_df_admin::error::Error),

    /// Observed state module error.
    #[error("Observed state error: {0}")]
    ObservedStateError(#[from] otap_df_state::error::Error),

    /// Telemetry system error.
    #[error("Telemetry error: {0}")]
    TelemetryError(#[from] otap_df_telemetry::error::Error),

    /// Memory limiter configuration or runtime initialization error.
    #[error("Memory limiter error: {message}")]
    MemoryLimiterError {
        /// Human-readable error message.
        message: String,
    },

    /// Pipeline runtime error.
    #[error("Pipeline runtime error: {source}")]
    PipelineRuntimeError {
        /// The underlying engine error.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>, // ToDo : Use a more specific error type if possible
    },

    /// A topic declaration requests a backend this runtime does not provide.
    #[error("Unsupported topic backend `{backend}` for topic `{topic}`")]
    #[diagnostic(code(data_plane::unsupported_topic_backend), url(docsrs))]
    UnsupportedTopicBackend {
        /// The declared topic name.
        topic: TopicName,
        /// The configured backend.
        backend: TopicBackendKind,
    },

    /// A topic declaration selects a mode unsupported by the chosen backend.
    #[error("Unsupported topic mode `{mode}` for topic `{topic}` on backend `{backend}`")]
    #[diagnostic(code(data_plane::unsupported_topic_mode), url(docsrs))]
    UnsupportedTopicMode {
        /// The declared topic name.
        topic: TopicName,
        /// The configured backend.
        backend: TopicBackendKind,
        /// The selected runtime mode.
        mode: String,
    },

    /// A topic declaration requests a policy not supported by the chosen backend.
    #[error("Unsupported topic policy for `{topic}` on backend `{backend}`: `{policy}={value}`")]
    #[diagnostic(code(data_plane::unsupported_topic_policy), url(docsrs))]
    UnsupportedTopicPolicy {
        /// The declared topic name.
        topic: TopicName,
        /// The configured backend.
        backend: TopicBackendKind,
        /// The unsupported policy key.
        policy: &'static str,
        /// The configured policy value.
        value: String,
    },

    /// A cycle was detected in the global topic wiring graph.
    #[error("Topic wiring cycle detected: {cycle:?}")]
    #[diagnostic(code(data_plane::topic_wiring_cycle_detected), url(docsrs))]
    TopicWiringCycleDetected {
        /// The cycle path, with the starting vertex repeated at the end.
        cycle: Vec<String>,
    },

    /// Failed to spawn an OS thread.
    #[error("Failed to spawn thread '{thread_name}': {source}")]
    ThreadSpawnError {
        /// Name of the thread we attempted to spawn.
        thread_name: String,
        /// Underlying OS error.
        #[source]
        source: std::io::Error,
    },

    /// Failed to enumerate available CPU cores on this platform.
    #[error("Failed to get available CPU cores (core detection unavailable on this platform)")]
    CoreDetectionUnavailable,

    /// Invalid or out-of-bounds requested CPU core ID allocation.
    #[error("Invalid core ID allocation [{alloc}]: {message}. Available core IDs: {available:?}")]
    InvalidCoreAllocation {
        /// Invalid of out-of-bounds CPU core ID allocation.
        alloc: CoreAllocation,
        /// Error message.
        message: String,
        /// The available CPU core IDs detected on this system.
        available: Vec<usize>,
    },

    /// Core affinity error.
    #[error("Failed to set core affinity for thread {thread_id} to core {core_id}: {message}")]
    CoreAffinityError {
        /// The thread ID that failed to bind.
        thread_id: usize,
        /// The core ID that we tried to bind to.
        core_id: usize,
        /// Error message.
        message: String,
    },

    /// Thread panic error with numeric thread identifier.
    #[error(
        "Thread {thread_name}(thread_id: {thread_id}, core_id: {core_id}) panicked: {panic_message}"
    )]
    ThreadPanic {
        /// The thread name that panicked.
        thread_name: String,
        /// The thread ID that panicked.
        thread_id: usize,
        /// The core ID where the thread was running.
        core_id: usize,
        /// Panic message.
        panic_message: String,
    },

    /// Thread panic error identified by thread name.
    #[error("Thread '{thread_name}' panicked: {panic_message}")]
    ThreadJoinPanic {
        /// The thread name that panicked.
        thread_name: String,
        /// Panic message.
        panic_message: String,
    },
}
