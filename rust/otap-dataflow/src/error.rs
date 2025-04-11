// SPDX-License-Identifier: Apache-2.0

//! All top-level errors that can occur in the OTAP Dataflow project.

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("A config error occurred: {0}")]
    ConfigError(#[from] crates::config::Error),
}