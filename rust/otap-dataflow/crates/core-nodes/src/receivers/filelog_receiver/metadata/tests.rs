// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use otel_arrow_dfe_filelog_checkpoint::ADVISORY_PATH_STORED_MAX_BYTES;

const POLICIES: [FileMetadata; 4] = [
    FileMetadata::None,
    FileMetadata::Name,
    FileMetadata::Path,
    FileMetadata::Native,
];

fn value<'a>(metadata: &'a FileMetadataAttributes, key: &str) -> Option<&'a Value> {
    metadata
        .as_slice()
        .iter()
        .find_map(|(name, value)| (*name == key).then_some(value))
}

fn assert_text(metadata: &FileMetadataAttributes, key: &str, expected: Option<&str>) {
    assert_eq!(
        value(metadata, key),
        expected
            .map(|text| Value::StringValue(text.to_owned()))
            .as_ref(),
        "{key}"
    );
}

/// Scenario: A complete ordinary Unix path is projected under all four policies.
/// Guarantees: Each policy emits exactly its permitted keys and correctly typed values.
#[test]
fn all_policies() {
    let path = AdvisoryPath::from_unix_bytes(b"/var/log/app.log").unwrap();
    for (policy, keys) in [
        (FileMetadata::None, vec![]),
        (FileMetadata::Name, vec![FILE_NAME]),
        (FileMetadata::Path, vec![FILE_PATH, FILE_NAME]),
        (
            FileMetadata::Native,
            vec![FILE_PATH, FILE_NAME, PATH_KIND, PATH_NATIVE, PATH_TRUNCATED],
        ),
    ] {
        let metadata = FileMetadataAttributes::project(policy, &path);
        let actual: Vec<_> = metadata.as_slice().iter().map(|(key, _)| *key).collect();
        assert_eq!(actual, keys);
        if policy != FileMetadata::None {
            assert_text(&metadata, FILE_NAME, Some("app.log"));
        }
        if matches!(policy, FileMetadata::Path | FileMetadata::Native) {
            assert_text(&metadata, FILE_PATH, Some("/var/log/app.log"));
        }
        if policy == FileMetadata::Native {
            assert_text(&metadata, PATH_KIND, Some("unix_bytes"));
            assert_eq!(
                value(&metadata, PATH_NATIVE),
                Some(&Value::BytesValue(b"/var/log/app.log".to_vec()))
            );
            assert_eq!(
                value(&metadata, PATH_TRUNCATED),
                Some(&Value::BoolValue(false))
            );
        }
        assert!(value(&metadata, PATH_SHA256).is_none());
    }
}

/// Scenario: Complete paths have no directory, backslashes, Unicode, or trailing separators.
/// Guarantees: Basenames use Unix separators on every host and preserve valid UTF-8 exactly.
#[test]
fn unix_basenames() {
    for (path, name) in [
        ("app.log", Some("app.log")),
        (r"/var\log/app\name.log", Some(r"app\name.log")),
        (r"directory\app.log", Some(r"directory\app.log")),
        ("/var/log/\u{00e9}.log", Some("\u{00e9}.log")),
        ("/var//log/app.log///", Some("app.log")),
        ("/", None),
    ] {
        let evidence = AdvisoryPath::from_unix_bytes(path.as_bytes()).unwrap();
        for policy in [FileMetadata::Name, FileMetadata::Path, FileMetadata::Native] {
            let metadata = FileMetadataAttributes::project(policy, &evidence);
            assert_text(&metadata, FILE_NAME, name);
            assert_text(
                &metadata,
                FILE_PATH,
                (policy != FileMetadata::Name).then_some(path),
            );
        }
    }
}

/// Scenario: Invalid UTF-8 occurs in the basename or only in a parent directory.
/// Guarantees: Text eligibility is independent per attribute and native bytes are never lossy.
#[test]
fn invalid_utf8() {
    for (bytes, name) in [
        (&b"/var/log/app\xff.log"[..], None),
        (&b"/var/\xff/app.log"[..], Some("app.log")),
    ] {
        let evidence = AdvisoryPath::from_unix_bytes(bytes).unwrap();
        for policy in [FileMetadata::Name, FileMetadata::Path, FileMetadata::Native] {
            let metadata = FileMetadataAttributes::project(policy, &evidence);
            assert_text(&metadata, FILE_PATH, None);
            assert_text(&metadata, FILE_NAME, name);
            if policy == FileMetadata::Native {
                assert_eq!(
                    value(&metadata, PATH_NATIVE),
                    Some(&Value::BytesValue(bytes.to_vec()))
                );
                assert!(value(&metadata, PATH_SHA256).is_none());
            } else {
                assert_eq!(metadata.as_slice().len(), usize::from(name.is_some()));
            }
        }
    }
}

/// Scenario: Evidence is unavailable or is native Windows UTF-16, complete or truncated.
/// Guarantees: Every policy omits unsupported evidence rather than interpreting it as Unix bytes.
#[test]
fn unavailable_and_windows_evidence() {
    let paths = [
        AdvisoryPath::unavailable(),
        AdvisoryPath::from_windows_utf16_units(&[0x432f, 0x005c, 0x0061]).unwrap(),
        AdvisoryPath::from_windows_utf16_units(&vec![
            0x0061;
            ADVISORY_PATH_STORED_MAX_BYTES / 2 + 1
        ])
        .unwrap(),
    ];
    for path in &paths {
        for policy in POLICIES {
            assert!(
                FileMetadataAttributes::project(policy, path)
                    .as_slice()
                    .is_empty()
            );
        }
    }
}

/// Scenario: Truncated evidence ends in a valid filename or contains invalid UTF-8.
/// Guarantees: Only native attributes survive, with exact suffix bytes and the existing digest.
#[test]
fn truncated_native_evidence() {
    for suffix in [&b"/app.log"[..], &b"/app\xff.log"[..]] {
        let mut bytes = vec![b'x'; ADVISORY_PATH_STORED_MAX_BYTES + 1];
        bytes.extend_from_slice(suffix);
        let evidence = AdvisoryPath::from_unix_bytes(&bytes).unwrap();
        let expected_digest = hex::encode(evidence.full_path_digest());
        assert_eq!(expected_digest.len(), 64);
        assert!(
            expected_digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        );
        for policy in POLICIES {
            let metadata = FileMetadataAttributes::project(policy, &evidence);
            assert_text(&metadata, FILE_PATH, None);
            assert_text(&metadata, FILE_NAME, None);
            if policy == FileMetadata::Native {
                assert_eq!(metadata.as_slice().len(), 4);
                assert_text(&metadata, PATH_KIND, Some("unix_bytes"));
                assert_eq!(
                    value(&metadata, PATH_NATIVE),
                    Some(&Value::BytesValue(
                        bytes[bytes.len() - ADVISORY_PATH_STORED_MAX_BYTES..].to_vec()
                    ))
                );
                assert_eq!(
                    value(&metadata, PATH_TRUNCATED),
                    Some(&Value::BoolValue(true))
                );
                assert_text(&metadata, PATH_SHA256, Some(&expected_digest));
            } else {
                assert!(metadata.as_slice().is_empty());
            }
        }
    }
}

fn metadata_allowance(policy: FileMetadata) -> u64 {
    use crate::receivers::filelog_receiver::config::Config;
    use std::path::Path;

    let config = Config {
        include: vec![if cfg!(windows) {
            r"C:\logs\*.log".to_owned()
        } else {
            "/logs/*.log".to_owned()
        }],
        include_file_metadata: policy,
        ..Config::default()
    };
    let working_dir = if cfg!(windows) {
        Path::new(r"C:\work")
    } else {
        Path::new("/work")
    };
    config
        .compile("default", "main", "filelog", working_dir)
        .unwrap()
        .inputs()
        .resources()
        .record_bytes
}

/// Scenario: Complete and truncated paths fill the maximum stored evidence capacity.
/// Guarantees: Projection respects the 4096-byte boundary and existing configured size allowance.
#[test]
fn maximum_evidence_fits_attribute_allowance() {
    let no_metadata = metadata_allowance(FileMetadata::None);
    for bytes in [
        vec![b'x'; ADVISORY_PATH_STORED_MAX_BYTES],
        "\u{00e9}"
            .repeat(ADVISORY_PATH_STORED_MAX_BYTES / 2)
            .into_bytes(),
        vec![b'x'; ADVISORY_PATH_STORED_MAX_BYTES + 1],
    ] {
        let evidence = AdvisoryPath::from_unix_bytes(&bytes).unwrap();
        for policy in POLICIES {
            let metadata = FileMetadataAttributes::project(policy, &evidence);
            let actual_bytes: u64 = metadata
                .as_slice()
                .iter()
                .map(|(key, value)| {
                    let size = match value {
                        Value::StringValue(text) => text.len(),
                        Value::BytesValue(bytes) => bytes.len(),
                        Value::BoolValue(_) => 1,
                        _ => panic!("unexpected metadata value type"),
                    };
                    (key.len() + size) as u64
                })
                .sum();
            assert!(actual_bytes <= metadata_allowance(policy) - no_metadata);
            if !evidence.is_truncated() && policy != FileMetadata::None {
                assert_text(&metadata, FILE_NAME, Some(str::from_utf8(&bytes).unwrap()));
                assert_eq!(
                    value(&metadata, FILE_PATH).is_some(),
                    policy != FileMetadata::Name
                );
            }
            if policy == FileMetadata::Native {
                assert_eq!(
                    value(&metadata, PATH_NATIVE),
                    Some(&Value::BytesValue(evidence.stored_path_bytes().to_vec()))
                );
            }
        }
    }
}
