// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared secret-file handling for contrib extensions.

use std::io;
use std::path::Path;

#[cfg(any(
    feature = "flat-file-api-key-auth",
    feature = "flat-file-user-pass-auth",
    test
))]
use otel_arrow_dfe_otap::tls_utils::read_file_with_limit_async;
#[cfg(feature = "flat-file-sasl-auth")]
use otel_arrow_dfe_otap::tls_utils::read_file_with_limit_sync;
use secrecy::SecretString;
use secrecy::zeroize::Zeroize;

/// Errors raised while reading a secret from a file.
#[derive(Debug, thiserror::Error)]
pub(crate) enum ReadSecretFileError {
    /// Reading the file failed.
    #[error(transparent)]
    Read(#[from] io::Error),

    /// The file contents were not valid UTF-8.
    #[error("secret file does not contain valid UTF-8")]
    InvalidUtf8,
}

/// Reads a size-limited UTF-8 secret and strips trailing line endings.
#[cfg(any(
    feature = "flat-file-api-key-auth",
    feature = "flat-file-user-pass-auth",
    test
))]
pub(crate) async fn read_secret_file(path: &Path) -> Result<SecretString, ReadSecretFileError> {
    let contents = read_file_with_limit_async(path).await?;
    decode_secret(contents)
}

/// Blocking counterpart for synchronous startup factories, not runtime calls.
#[cfg(feature = "flat-file-sasl-auth")]
pub(crate) fn read_secret_file_sync(path: &Path) -> Result<SecretString, ReadSecretFileError> {
    let contents = read_file_with_limit_sync(path)?;
    decode_secret(contents)
}

fn decode_secret(contents: Vec<u8>) -> Result<SecretString, ReadSecretFileError> {
    let mut contents = String::from_utf8(contents).map_err(|error| {
        error.into_bytes().zeroize();
        ReadSecretFileError::InvalidUtf8
    })?;
    let secret = contents
        .trim_end_matches(&['\r', '\n'][..])
        .to_string()
        .into();
    contents.zeroize();
    Ok(secret)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use secrecy::ExposeSecret;
    use tempfile::NamedTempFile;

    use super::*;

    /// Scenario: A UTF-8 secret file ends with line endings and contains other trailing whitespace.
    /// Guarantees: Only trailing CR and LF characters are removed from the returned secret.
    #[tokio::test]
    async fn reads_secret_and_trims_line_endings() {
        let mut file = NamedTempFile::new().expect("file created");
        file.write_all(b"secret  \r\n").expect("content written");

        let secret = read_secret_file(file.path())
            .await
            .expect("secret file read");

        assert_eq!(secret.expose_secret(), "secret  ");
    }

    /// Scenario: A secret file contains bytes that are not valid UTF-8.
    /// Guarantees: The helper rejects the content with the encoding-specific error.
    #[tokio::test]
    async fn rejects_invalid_utf8() {
        let mut file = NamedTempFile::new().expect("file created");
        file.write_all(&[0xff, 0xfe, 0xfd])
            .expect("content written");

        let error = read_secret_file(file.path())
            .await
            .expect_err("invalid UTF-8 rejected");

        assert!(matches!(error, ReadSecretFileError::InvalidUtf8));
    }

    /// Scenario: A secret file exceeds the shared four-megabyte file limit.
    /// Guarantees: The helper rejects the file through its bounded-read error path.
    #[tokio::test]
    async fn rejects_oversized_file() {
        let mut file = NamedTempFile::new().expect("file created");
        file.write_all(&vec![b'x'; 5 * 1024 * 1024])
            .expect("content written");

        let error = read_secret_file(file.path())
            .await
            .expect_err("oversized file rejected");

        assert!(matches!(error, ReadSecretFileError::Read(_)));
        assert!(error.to_string().contains("too large"));
    }
}
