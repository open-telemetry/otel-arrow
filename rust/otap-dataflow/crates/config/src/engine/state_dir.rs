// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Lexical state-root validation, without filesystem access or path normalization.

use std::path::{Component, Path};

/// Validates an explicit, native absolute directory path.
///
/// Reject ambiguous spelling instead of normalizing away components before
/// handle-relative traversal. This function does not establish filesystem trust.
pub fn validate_state_dir(path: &Path) -> Result<(), &'static str> {
    if !path.is_absolute() {
        return Err("must be an explicit absolute path");
    }
    let bytes = path.as_os_str().as_encoded_bytes();
    if bytes.contains(&0) || bytes.windows(2).any(|pair| pair == b"${") {
        return Err("must not contain NUL or unresolved placeholders");
    }
    if !path.components().any(|c| matches!(c, Component::Normal(_))) {
        return Err("must name a directory below the filesystem root");
    }
    // Path::components hides interior `.` and repeated separators. Inspect
    // native encoded bytes for these ASCII delimiters without converting identity
    // to a Unicode string. Windows accepts both slash and backslash separators.
    let separator = |byte: &u8| *byte == b'/' || (cfg!(windows) && *byte == b'\\');
    if bytes.last().is_some_and(separator)
        || bytes
            .windows(2)
            .any(|pair| separator(&pair[0]) && separator(&pair[1]))
        || bytes
            .split(separator)
            .any(|part| part == b"." || part == b"..")
    {
        return Err(
            "must not contain dot/parent components, repeated separators, or a trailing separator",
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::OtelDataflowSpec;

    /// Scenario: state_dir is omitted or contains an invalid path.
    /// Guarantees: omission is valid; ambiguous and relative roots are rejected.
    #[test]
    fn configuration_validation() {
        let config = OtelDataflowSpec::from_yaml("version: otel_dataflow/v1").unwrap();
        assert!(config.engine.state_dir.is_none());
        for path in [
            "",
            ".",
            "relative",
            "/",
            "/a/..",
            "/a/./b",
            "/a//b",
            "/a/",
            "/a/${unknown}",
            "/a\0b",
        ] {
            let mut candidate = config.clone();
            candidate.engine.state_dir = Some(path.into());
            assert!(candidate.validate().is_err(), "accepted {path:?}");
        }
        let path = if cfg!(windows) {
            r"C:\otel\state"
        } else {
            "/var/lib/otel/state"
        };
        let json =
            serde_json::json!({"version": "otel_dataflow/v1", "engine": {"state_dir": path}});
        let config = OtelDataflowSpec::from_json(&json.to_string()).unwrap();
        assert_eq!(config.engine.state_dir.as_deref(), Some(Path::new(path)));
    }
}
