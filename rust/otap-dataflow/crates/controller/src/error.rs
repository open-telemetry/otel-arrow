// SPDX-License-Identifier: Apache-2.0

//! Errors for the controller crate.

use miette::Diagnostic;

/// Errors that can occur in the controller crate.
#[derive(thiserror::Error, Debug, Diagnostic)]
pub enum Error {
    /// A collection of errors that occurred during parsing or validating the configuration.
    #[error("Invalid configuration: {errors:?}")]
    #[diagnostic(code(data_plane::invalid_configuration), url(docsrs))]
    InvalidConfiguration {
        /// A list of errors that occurred during parsing or validating the configuration.
        #[related]
        errors: Vec<otap_df_config::error::Error>,
    }
}
