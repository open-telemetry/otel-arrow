// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Durable cursor checkpoint helpers for the journald receiver.

use serde::{Deserialize, Serialize};
use std::io;
use std::path::{Path, PathBuf};

const ENVELOPE_VERSION: u8 = 1;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CursorEnvelope {
    version: u8,
    cursor: String,
    checksum: String,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum CheckpointError {
    #[error("failed to read checkpoint {path}: {source}")]
    Read { path: PathBuf, source: io::Error },
    #[error("failed to parse checkpoint {path}: {source}")]
    Parse {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("unsupported checkpoint version {version} in {path}")]
    UnsupportedVersion { path: PathBuf, version: u8 },
    #[error("checkpoint checksum mismatch in {path}")]
    ChecksumMismatch { path: PathBuf },
    #[error("failed to encode checkpoint envelope: {source}")]
    Encode { source: serde_json::Error },
    #[error("checkpoint path has no parent: {path}")]
    NoParent { path: PathBuf },
    #[error("failed to create checkpoint directory {path}: {source}")]
    CreateDirectory { path: PathBuf, source: io::Error },
    #[error("failed to write checkpoint {path}: {source}")]
    Write { path: PathBuf, source: io::Error },
    #[error("failed to reopen checkpoint {path}: {source}")]
    Reopen { path: PathBuf, source: io::Error },
    #[error("failed to fsync checkpoint {path}: {source}")]
    FsyncFile { path: PathBuf, source: io::Error },
    #[error("failed to install checkpoint {tmp} -> {path}: {source}")]
    Rename {
        tmp: PathBuf,
        path: PathBuf,
        source: io::Error,
    },
    #[error("failed to open checkpoint directory {path} for fsync: {source}")]
    OpenDirectory { path: PathBuf, source: io::Error },
    #[error("failed to fsync checkpoint directory {path}: {source}")]
    FsyncDirectory { path: PathBuf, source: io::Error },
}

pub(crate) fn checkpoint_path(
    root: &Path,
    pipeline_group_id: &str,
    pipeline_id: &str,
    receiver_name: &str,
    source_id: &str,
) -> PathBuf {
    let mut path = expand_state_dir(root);
    path.push(sanitize_segment(pipeline_group_id));
    path.push(sanitize_segment(pipeline_id));
    path.push(sanitize_segment(receiver_name));
    path.push(format!("{}.cursor", sanitize_segment(source_id)));
    path
}

pub(crate) fn read_cursor(path: &Path) -> Result<Option<String>, CheckpointError> {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(CheckpointError::Read {
                path: path.to_path_buf(),
                source,
            });
        }
    };
    let envelope: CursorEnvelope =
        serde_json::from_slice(&bytes).map_err(|source| CheckpointError::Parse {
            path: path.to_path_buf(),
            source,
        })?;
    if envelope.version != ENVELOPE_VERSION {
        return Err(CheckpointError::UnsupportedVersion {
            path: path.to_path_buf(),
            version: envelope.version,
        });
    }
    let expected = checksum(&envelope.cursor);
    if envelope.checksum != expected {
        return Err(CheckpointError::ChecksumMismatch {
            path: path.to_path_buf(),
        });
    }
    Ok(Some(envelope.cursor))
}

pub(crate) fn write_cursor(path: &Path, cursor: &str) -> Result<(), CheckpointError> {
    let envelope = CursorEnvelope {
        version: ENVELOPE_VERSION,
        cursor: cursor.to_owned(),
        checksum: checksum(cursor),
    };
    let bytes =
        serde_json::to_vec(&envelope).map_err(|source| CheckpointError::Encode { source })?;
    let parent = path.parent().ok_or_else(|| CheckpointError::NoParent {
        path: path.to_path_buf(),
    })?;
    std::fs::create_dir_all(parent).map_err(|source| CheckpointError::CreateDirectory {
        path: parent.to_path_buf(),
        source,
    })?;
    // Keep the temporary file beside the final checkpoint so rename is atomic.
    let tmp = path.with_extension("cursor.tmp");
    std::fs::write(&tmp, bytes).map_err(|source| CheckpointError::Write {
        path: tmp.clone(),
        source,
    })?;
    let file = std::fs::OpenOptions::new()
        .read(true)
        .open(&tmp)
        .map_err(|source| CheckpointError::Reopen {
            path: tmp.clone(),
            source,
        })?;
    file.sync_all()
        .map_err(|source| CheckpointError::FsyncFile {
            path: tmp.clone(),
            source,
        })?;
    std::fs::rename(&tmp, path).map_err(|source| CheckpointError::Rename {
        tmp: tmp.clone(),
        path: path.to_path_buf(),
        source,
    })?;
    let dir = std::fs::OpenOptions::new()
        .read(true)
        .open(parent)
        .map_err(|source| CheckpointError::OpenDirectory {
            path: parent.to_path_buf(),
            source,
        })?;
    dir.sync_all()
        .map_err(|source| CheckpointError::FsyncDirectory {
            path: parent.to_path_buf(),
            source,
        })?;
    Ok(())
}

fn checksum(cursor: &str) -> String {
    blake3::hash(cursor.as_bytes()).to_hex().to_string()
}

fn expand_state_dir(root: &Path) -> PathBuf {
    let text = root.to_string_lossy();
    if let Some(rest) = text.strip_prefix("${engine.state_dir}") {
        let base = std::env::var_os("OTAP_DF_STATE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(".otap-state"));
        return base.join(rest.trim_start_matches('/'));
    }
    root.to_path_buf()
}

fn sanitize_segment(value: &str) -> String {
    value
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.') {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_and_reads_cursor_envelope() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cursor");
        write_cursor(&path, "abc").unwrap();
        assert_eq!(read_cursor(&path).unwrap(), Some("abc".to_owned()));
    }

    #[test]
    fn corrupt_checkpoint_fails_closed() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cursor");
        std::fs::write(&path, br#"{"version":1,"cursor":"abc","checksum":"bad"}"#).unwrap();
        assert!(read_cursor(&path).is_err());
    }

    #[test]
    fn checkpoint_path_is_stable() {
        let path = checkpoint_path(
            Path::new("${engine.state_dir}/journald"),
            "g",
            "p",
            "recv",
            "system",
        );
        assert!(path.ends_with("journald/g/p/recv/system.cursor"));
    }
}
