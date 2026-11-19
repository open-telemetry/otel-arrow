// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Lossless, bounded source-file metadata projection without filesystem access.

use otel_arrow_dfe_filelog_checkpoint::{AdvisoryPath, AdvisoryPathKind};
use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::any_value::Value;
use smallvec::SmallVec;

use super::config::FileMetadata;

const FILE_PATH: &str = "log.file.path";
const FILE_NAME: &str = "log.file.name";
const PATH_KIND: &str = "otel.arrow.filelog.path.kind";
const PATH_NATIVE: &str = "otel.arrow.filelog.path.native";
const PATH_TRUNCATED: &str = "otel.arrow.filelog.path.truncated";
const PATH_SHA256: &str = "otel.arrow.filelog.path.sha256";

/// Selected source-file attributes, owned independently of the advisory evidence.
///
/// Keys are fixed by the provenance contract. Attribute values remain platform-neutral
/// so additional evidence kinds can be supported without changing this output type.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FileMetadataAttributes {
    attributes: SmallVec<[(&'static str, Value); 5]>,
}

impl FileMetadataAttributes {
    /// Projects only the provided bounded evidence under the selected export policy.
    ///
    /// Unavailable and unsupported Windows evidence produce no attributes. Truncated
    /// evidence never produces text attributes; only native mode exports its stored
    /// suffix and existing complete-path digest.
    #[must_use]
    pub fn project(policy: FileMetadata, path: &AdvisoryPath) -> Self {
        let mut result = Self::default();
        if policy == FileMetadata::None {
            return result;
        }
        match path.kind() {
            AdvisoryPathKind::Unavailable | AdvisoryPathKind::WindowsUtf16Le => return result,
            AdvisoryPathKind::UnixBytes => {}
        }

        let bytes = path.stored_path_bytes();
        if !path.is_truncated() {
            if matches!(policy, FileMetadata::Path | FileMetadata::Native)
                && let Ok(text) = str::from_utf8(bytes)
            {
                result
                    .attributes
                    .push((FILE_PATH, Value::StringValue(text.to_owned())));
            }
            // Split native Unix bytes, not a host-platform Path or decoded full path.
            if let Some(name) = bytes.rsplit(|byte| *byte == b'/').find(|s| !s.is_empty())
                && let Ok(text) = str::from_utf8(name)
            {
                result
                    .attributes
                    .push((FILE_NAME, Value::StringValue(text.to_owned())));
            }
        }
        if policy == FileMetadata::Native {
            result
                .attributes
                .push((PATH_KIND, Value::StringValue("unix_bytes".to_owned())));
            result
                .attributes
                .push((PATH_NATIVE, Value::BytesValue(bytes.to_vec())));
            result
                .attributes
                .push((PATH_TRUNCATED, Value::BoolValue(path.is_truncated())));
            if path.is_truncated() {
                result.attributes.push((
                    PATH_SHA256,
                    Value::StringValue(hex::encode(path.full_path_digest())),
                ));
            }
        }
        result
    }

    /// Borrows the selected fixed keys and typed attribute values for batch construction.
    #[must_use]
    pub fn as_slice(&self) -> &[(&'static str, Value)] {
        &self.attributes
    }
}

#[cfg(test)]
mod tests;
