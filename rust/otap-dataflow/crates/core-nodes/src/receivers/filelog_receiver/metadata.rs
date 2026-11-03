// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bounded source-file metadata projection, independent of OTAP batch construction.

use super::config::FileMetadata;
use otel_arrow_dfe_filelog_checkpoint::{AdvisoryPath, AdvisoryPathKind};

const FILE_NAME: &str = "log.file.name";
const FILE_PATH: &str = "log.file.path";
const PATH_KIND: &str = "otel.arrow.filelog.path.kind";
const PATH_NATIVE: &str = "otel.arrow.filelog.path.native";
const PATH_TRUNCATED: &str = "otel.arrow.filelog.path.truncated";
const PATH_SHA256: &str = "otel.arrow.filelog.path.sha256";

/// A value that can be written to the future OTAP log-attribute builder.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FileMetadataValue {
    /// Losslessly converted text.
    String(String),
    /// Exact stored native path bytes.
    Bytes(Vec<u8>),
    /// Whether the stored native path is truncated.
    Bool(bool),
}

/// One source-file attribute with a fixed key.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileMetadataAttribute {
    /// Receiver-owned attribute key.
    pub key: &'static str,
    /// Attribute value, preserving its OTLP type.
    pub value: FileMetadataValue,
}

/// Projected source-file attributes for one log record.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FileMetadataAttributes {
    attributes: Vec<FileMetadataAttribute>,
}

impl FileMetadataAttributes {
    /// Returns the attributes in stable order for a future batch builder.
    #[must_use]
    pub fn as_slice(&self) -> &[FileMetadataAttribute] {
        &self.attributes
    }

    /// Transfers the owned attributes to a future batch builder.
    #[must_use]
    pub fn into_vec(self) -> Vec<FileMetadataAttribute> {
        self.attributes
    }
}

/// Projects available Unix path evidence under the configured metadata policy.
///
/// Windows evidence is left unsupported until its platform-specific projection is added.
/// The result uses only stored evidence and never reconstructs a truncated path.
#[must_use]
pub fn project_file_metadata(
    policy: FileMetadata,
    evidence: &AdvisoryPath,
) -> FileMetadataAttributes {
    if policy == FileMetadata::None || evidence.kind() != AdvisoryPathKind::UnixBytes {
        return FileMetadataAttributes::default();
    }

    let bytes = evidence.stored_path_bytes();
    let mut attributes = Vec::new();

    if !evidence.is_truncated() {
        if policy != FileMetadata::Name
            && let Ok(path) = std::str::from_utf8(bytes)
        {
            attributes.push(FileMetadataAttribute {
                key: FILE_PATH,
                value: FileMetadataValue::String(path.to_owned()),
            });
        }

        // Unix uses only '/' as a separator. Validate the basename separately
        // so an invalid parent directory does not hide a valid file name.
        if let Some(name) = bytes.rsplit(|byte| *byte == b'/').next()
            && !name.is_empty()
            && let Ok(name) = std::str::from_utf8(name)
        {
            attributes.push(FileMetadataAttribute {
                key: FILE_NAME,
                value: FileMetadataValue::String(name.to_owned()),
            });
        }
    }

    if policy == FileMetadata::Native {
        attributes.extend([
            FileMetadataAttribute {
                key: PATH_KIND,
                value: FileMetadataValue::String("unix_bytes".to_owned()),
            },
            FileMetadataAttribute {
                key: PATH_NATIVE,
                value: FileMetadataValue::Bytes(bytes.to_vec()),
            },
            FileMetadataAttribute {
                key: PATH_TRUNCATED,
                value: FileMetadataValue::Bool(evidence.is_truncated()),
            },
        ]);
        if evidence.is_truncated() {
            attributes.push(FileMetadataAttribute {
                key: PATH_SHA256,
                value: FileMetadataValue::String(hex::encode(evidence.full_path_digest())),
            });
        }
    }

    FileMetadataAttributes { attributes }
}

#[cfg(test)]
mod tests;
