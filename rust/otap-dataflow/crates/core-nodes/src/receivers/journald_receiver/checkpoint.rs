// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Durable cursor checkpoint helpers for the journald receiver.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const ENVELOPE_VERSION: u8 = 1;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CursorEnvelope {
    version: u8,
    cursor: String,
    checksum: String,
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

pub(crate) fn read_cursor(path: &Path) -> Result<Option<String>, String> {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => {
            return Err(format!(
                "failed to read checkpoint {}: {err}",
                path.display()
            ));
        }
    };
    let envelope: CursorEnvelope = serde_json::from_slice(&bytes)
        .map_err(|err| format!("failed to parse checkpoint {}: {err}", path.display()))?;
    if envelope.version != ENVELOPE_VERSION {
        return Err(format!(
            "unsupported checkpoint version {} in {}",
            envelope.version,
            path.display()
        ));
    }
    let expected = checksum(&envelope.cursor);
    if envelope.checksum != expected {
        return Err(format!(
            "checkpoint checksum mismatch in {}",
            path.display()
        ));
    }
    Ok(Some(envelope.cursor))
}

pub(crate) fn write_cursor(path: &Path, cursor: &str) -> Result<(), String> {
    let envelope = CursorEnvelope {
        version: ENVELOPE_VERSION,
        cursor: cursor.to_owned(),
        checksum: checksum(cursor),
    };
    let bytes = serde_json::to_vec(&envelope)
        .map_err(|err| format!("failed to encode checkpoint envelope: {err}"))?;
    let parent = path
        .parent()
        .ok_or_else(|| format!("checkpoint path has no parent: {}", path.display()))?;
    std::fs::create_dir_all(parent).map_err(|err| {
        format!(
            "failed to create checkpoint directory {}: {err}",
            parent.display()
        )
    })?;
    // Keep the temporary file beside the final checkpoint so rename is atomic.
    let tmp = path.with_extension("cursor.tmp");
    std::fs::write(&tmp, bytes)
        .map_err(|err| format!("failed to write checkpoint {}: {err}", tmp.display()))?;
    let file = std::fs::OpenOptions::new()
        .read(true)
        .open(&tmp)
        .map_err(|err| format!("failed to reopen checkpoint {}: {err}", tmp.display()))?;
    file.sync_all()
        .map_err(|err| format!("failed to fsync checkpoint {}: {err}", tmp.display()))?;
    std::fs::rename(&tmp, path).map_err(|err| {
        format!(
            "failed to install checkpoint {} -> {}: {err}",
            tmp.display(),
            path.display()
        )
    })?;
    let dir = std::fs::OpenOptions::new()
        .read(true)
        .open(parent)
        .map_err(|err| {
            format!(
                "failed to open checkpoint directory {} for fsync: {err}",
                parent.display()
            )
        })?;
    dir.sync_all().map_err(|err| {
        format!(
            "failed to fsync checkpoint directory {}: {err}",
            parent.display()
        )
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
