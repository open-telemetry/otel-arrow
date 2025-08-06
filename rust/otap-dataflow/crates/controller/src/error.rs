// SPDX-License-Identifier: Apache-2.0

//! Errors for the controller crate.

use miette::Diagnostic;

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

    /// Pipeline runtime error.
    #[error("Pipeline runtime error: {source}")]
    PipelineRuntimeError {
        /// The underlying engine error.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>, // ToDo : Use a more specific error type if possible
    },

    /// Internal controller error (not recoverable).
    #[error("Internal controller error: {message}")]
    InternalError {
        /// Error message describing the thread issue.
        message: String,
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

    /// Thread panic error.
    #[error("Thread {thread_id} panicked: {message}")]
    ThreadPanic {
        /// The thread ID that panicked.
        thread_id: usize,
        /// Panic message.
        message: String,
    },
}
