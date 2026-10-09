// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Protocol-neutral acquisition of username/password material from files.

use std::path::{Path, PathBuf};

use secrecy::SecretString;

use super::secret_file::{ReadSecretFileError, read_secret_file};

/// A failed acquisition, identified without exposing credential contents.
#[derive(Debug, thiserror::Error)]
pub(crate) enum ReadUserPassError {
    #[error("failed to read `{field}` at {}: {source}", .path.display())]
    ReadFile {
        field: &'static str,
        path: PathBuf,
        source: ReadSecretFileError,
    },
    #[error("no `{field}` or `{field}_file` configured")]
    MissingValue { field: &'static str },
}

/// Acquires both values before returning a pair; protocol validation is left
/// to the caller. Files take precedence over inline values without fallback.
/// Separate reads do not guarantee a cross-file snapshot.
pub(crate) async fn read_user_pass(
    username_file: Option<&Path>,
    username: Option<&SecretString>,
    password_file: Option<&Path>,
    password: Option<&SecretString>,
) -> Result<(SecretString, SecretString), ReadUserPassError> {
    let username = read_value(username_file, username, "username", "username_file").await?;
    let password = read_value(
        password_file,
        password,
        "password_secret",
        "password_secret_file",
    )
    .await?;
    Ok((username, password))
}

async fn read_value(
    file: Option<&Path>,
    inline: Option<&SecretString>,
    field: &'static str,
    file_field: &'static str,
) -> Result<SecretString, ReadUserPassError> {
    if let Some(path) = file {
        return read_secret_file(path)
            .await
            .map_err(|source| ReadUserPassError::ReadFile {
                field: file_field,
                path: path.to_owned(),
                source,
            });
    }
    inline
        .cloned()
        .ok_or(ReadUserPassError::MissingValue { field })
}
