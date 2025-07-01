// SPDX-License-Identifier: Apache-2.0

//! All top-level errors that can occur in the OTAP Pipeline project.

use thiserror::Error;

/// All top-level errors that can occur in the OTAP Pipeline project.
#[derive(Error, Debug)]
pub enum Error {
    /// A wrapper for the config errors.
    #[error("A config error occurred: {0}")]
    ConfigError(#[from] otap_df_config::error::Error),

    /// A generic error for pipeline operations.
    #[error("A pipeline engine error occurred: {0}")]
    PipelineError(String),
}
