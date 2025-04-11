// SPDX-License-Identifier: Apache-2.0

//! All top-level errors that can occur in the OTAP Dataflow project.

/// All top-level errors that can occur in the OTAP Dataflow project.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// A wrapper for the config errors.
    #[error("A config error occurred: {0}")]
    ConfigError(#[from] crates::config::Error),

    /// A wrapper for the dataflow engine errors.
    #[error("A dataflow engine error occurred: {0}")]
    DataflowEngineError(#[from] crates::engine::Error),
}