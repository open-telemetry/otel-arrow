// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::super::config::Config;
use super::*;
use otel_arrow_dfe_filelog_checkpoint::ADVISORY_PATH_STORED_MAX_BYTES;
use std::path::Path;

fn string(key: &'static str, value: &str) -> FileMetadataAttribute {
    FileMetadataAttribute {
        key,
        value: FileMetadataValue::String(value.to_owned()),
    }
}

fn bytes(key: &'static str, value: &[u8]) -> FileMetadataAttribute {
    FileMetadataAttribute {
        key,
        value: FileMetadataValue::Bytes(value.to_vec()),
    }
}

fn boolean(key: &'static str, value: bool) -> FileMetadataAttribute {
    FileMetadataAttribute {
        key,
        value: FileMetadataValue::Bool(value),
    }
}

/// Scenario: Every metadata policy receives a complete Unix path.
/// Guarantees: Each policy emits exactly its selected fixed keys and typed values.
#[test]
fn complete_unix_path_respects_all_modes() {
    let path = AdvisoryPath::from_unix_bytes(b"/var/log/app.log").unwrap();

    assert!(
        project_file_metadata(FileMetadata::None, &path)
            .as_slice()
            .is_empty()
    );
    assert_eq!(
        project_file_metadata(FileMetadata::Name, &path).into_vec(),
        vec![string(FILE_NAME, "app.log")]
    );
    assert_eq!(
        project_file_metadata(FileMetadata::Path, &path).into_vec(),
        vec![
            string(FILE_PATH, "/var/log/app.log"),
            string(FILE_NAME, "app.log"),
        ]
    );
    assert_eq!(
        project_file_metadata(FileMetadata::Native, &path).into_vec(),
        vec![
            string(FILE_PATH, "/var/log/app.log"),
            string(FILE_NAME, "app.log"),
            string(PATH_KIND, "unix_bytes"),
            bytes(PATH_NATIVE, b"/var/log/app.log"),
            boolean(PATH_TRUNCATED, false),
        ]
    );
}

/// Scenario: A Unix path is just a filename with no directory.
/// Guarantees: The complete path and basename are both the original filename.
#[test]
fn filename_without_directory() {
    let path = AdvisoryPath::from_unix_bytes(b"app.log").unwrap();

    assert_eq!(
        project_file_metadata(FileMetadata::Path, &path).into_vec(),
        vec![string(FILE_PATH, "app.log"), string(FILE_NAME, "app.log")]
    );
}

/// Scenario: A Unix filename contains a backslash.
/// Guarantees: Only '/' separates components, so the backslash remains in the basename.
#[test]
fn backslash_is_not_a_unix_separator() {
    let path = AdvisoryPath::from_unix_bytes(br"/var/log/app\part.log").unwrap();

    assert_eq!(
        project_file_metadata(FileMetadata::Name, &path).into_vec(),
        vec![string(FILE_NAME, r"app\part.log")]
    );
}

/// Scenario: A parent directory is invalid UTF-8 but the basename is valid.
/// Guarantees: Text path conversion fails independently of basename conversion.
#[test]
fn invalid_parent_preserves_valid_basename() {
    let path = AdvisoryPath::from_unix_bytes(b"/var/\xff/app.log").unwrap();

    assert_eq!(
        project_file_metadata(FileMetadata::Path, &path).into_vec(),
        vec![string(FILE_NAME, "app.log")]
    );
    assert_eq!(
        project_file_metadata(FileMetadata::Native, &path).into_vec(),
        vec![
            string(FILE_NAME, "app.log"),
            string(PATH_KIND, "unix_bytes"),
            bytes(PATH_NATIVE, b"/var/\xff/app.log"),
            boolean(PATH_TRUNCATED, false),
        ]
    );
}

/// Scenario: The Unix basename itself contains invalid UTF-8.
/// Guarantees: Text modes do not use replacement characters; native bytes remain exact.
#[test]
fn invalid_basename_has_no_text_attributes() {
    let path = AdvisoryPath::from_unix_bytes(b"/var/log/app\xff.log").unwrap();

    for policy in [FileMetadata::Name, FileMetadata::Path] {
        assert!(project_file_metadata(policy, &path).as_slice().is_empty());
    }
    assert_eq!(
        project_file_metadata(FileMetadata::Native, &path).into_vec(),
        vec![
            string(PATH_KIND, "unix_bytes"),
            bytes(PATH_NATIVE, b"/var/log/app\xff.log"),
            boolean(PATH_TRUNCATED, false),
        ]
    );
}

/// Scenario: The complete path exceeds the stored-evidence limit.
/// Guarantees: Native mode exports the stored suffix and existing digest, never text.
#[test]
fn truncated_path_has_native_evidence_only() {
    let mut complete = vec![b'/'];
    complete.extend(vec![b'a'; ADVISORY_PATH_STORED_MAX_BYTES]);
    let path = AdvisoryPath::from_unix_bytes(&complete).unwrap();
    assert!(path.is_truncated());
    assert_eq!(path.stored_path_bytes(), &complete[1..]);

    for policy in [FileMetadata::Name, FileMetadata::Path] {
        assert!(project_file_metadata(policy, &path).as_slice().is_empty());
    }
    assert_eq!(
        project_file_metadata(FileMetadata::Native, &path).into_vec(),
        vec![
            string(PATH_KIND, "unix_bytes"),
            bytes(PATH_NATIVE, &complete[1..]),
            boolean(PATH_TRUNCATED, true),
            string(
                PATH_SHA256,
                "2b37dd4b2de90dd9a2aa22505d68749f887b5028f24077bee6eab321d25cbd89",
            ),
        ]
    );
}

fn projected_size(attributes: &[FileMetadataAttribute]) -> u64 {
    attributes
        .iter()
        .map(|attribute| {
            let value_len = match &attribute.value {
                FileMetadataValue::String(value) => value.len(),
                FileMetadataValue::Bytes(value) => value.len(),
                FileMetadataValue::Bool(_) => 1,
            };
            (attribute.key.len() + value_len) as u64
        })
        .sum()
}

fn configured_record_bound(policy: FileMetadata) -> u64 {
    let mut config = Config::default();
    let (include, base) = if cfg!(windows) {
        ("C:/var/log/*.log", "C:/work")
    } else {
        ("/var/log/*.log", "/work")
    };
    config.include.push(include.to_owned());
    config.include_file_metadata = policy;
    config
        .compile("default", "main", "filelog", Path::new(base))
        .unwrap()
        .inputs()
        .resources()
        .record_bytes
}

/// Scenario: Unix evidence fills the 4,096-byte storage limit, with and without truncation.
/// Guarantees: Projected key/value bytes fit the existing policy-specific record allowance.
#[test]
fn maximum_stored_evidence_fits_record_allowance() {
    let complete =
        AdvisoryPath::from_unix_bytes(&vec![b'a'; ADVISORY_PATH_STORED_MAX_BYTES]).unwrap();
    let mut longer = vec![b'/'];
    longer.extend(vec![b'a'; ADVISORY_PATH_STORED_MAX_BYTES]);
    let truncated = AdvisoryPath::from_unix_bytes(&longer).unwrap();
    assert!(!complete.is_truncated());
    assert!(truncated.is_truncated());
    assert_eq!(
        complete.stored_path_bytes().len(),
        ADVISORY_PATH_STORED_MAX_BYTES
    );
    assert_eq!(
        truncated.stored_path_bytes().len(),
        ADVISORY_PATH_STORED_MAX_BYTES
    );

    let base = configured_record_bound(FileMetadata::None);
    for evidence in [&complete, &truncated] {
        for policy in [
            FileMetadata::None,
            FileMetadata::Name,
            FileMetadata::Path,
            FileMetadata::Native,
        ] {
            let attributes = project_file_metadata(policy, evidence);
            let allowed = configured_record_bound(policy) - base;
            assert!(
                projected_size(attributes.as_slice()) <= allowed,
                "{policy:?} exceeded its configured path-attribute allowance"
            );
        }
    }
}

/// Scenario: There is no advisory path evidence.
/// Guarantees: No policy invents file-path attributes or a digest.
#[test]
fn unavailable_path_emits_nothing() {
    let path = AdvisoryPath::unavailable();

    for policy in [
        FileMetadata::None,
        FileMetadata::Name,
        FileMetadata::Path,
        FileMetadata::Native,
    ] {
        assert!(project_file_metadata(policy, &path).as_slice().is_empty());
    }
}

/// Scenario: The evidence contains Windows UTF-16 code units.
/// Guarantees: Unix projection treats Windows as unsupported under every policy.
#[test]
fn windows_path_is_unsupported() {
    let path = AdvisoryPath::from_windows_utf16_units(&[
        b'C' as u16,
        b':' as u16,
        b'\\' as u16,
        b'a' as u16,
    ])
    .unwrap();

    for policy in [
        FileMetadata::None,
        FileMetadata::Name,
        FileMetadata::Path,
        FileMetadata::Native,
    ] {
        assert!(project_file_metadata(policy, &path).as_slice().is_empty());
    }
}
