// SPDX-License-Identifier: Apache-2.0

//! All top-level errors that can occur in the OTAP Pipeline project.

/// All top-level errors that can occur in the OTAP Pipeline project.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// A wrapper for the config errors.
    #[error("A config error occurred: {0}")]
    ConfigError(#[from] crates::config::Error),

    /// A wrapper for the pipeline engine errors.
    #[error("A pipeline engine error occurred: {0}")]
    EngineError(#[from] crates::engine::Error),
}