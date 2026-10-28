// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared data traversal, serialization and configuration-text limits.

use serde::Serialize;
use serde_json::Value;
use std::io::{self, Read, Write};
use std::path::Path;

/// A configuration or data-processing limit failure.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum DataLimitError {
    /// Invalid safeguard settings.
    #[error("invalid data limits: {0}")]
    Config(String),
    /// Failed to read UTF-8 configuration text.
    #[error("configuration file error: {0}")]
    File(String),
    /// Processing exceeded an explicit resource limit.
    #[error("data limit exceeded: {0}")]
    Limit(&'static str),
}

/// Internal safeguards, independent of the configurable protocol byte limits.
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    /// Maximum JSON and field-path nesting depth.
    pub depth: usize,
    /// Maximum inspected JSON nodes.
    pub nodes: usize,
    /// Maximum cumulative traversal work.
    pub work: usize,
    /// Maximum cumulative output bytes.
    pub output_bytes: usize,
    /// Maximum emitted attributes.
    pub attributes: usize,
    /// Maximum compiled rules, including inherited copies.
    pub rules: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            depth: 64,
            nodes: 65_536,
            work: 16 * 1024 * 1024,
            output_bytes: 3 * 1024 * 1024,
            attributes: 8_192,
            rules: 4_096,
        }
    }
}

impl Limits {
    /// Reject disabled safeguards and unsupported nesting depths.
    pub fn validate(self) -> Result<(), DataLimitError> {
        if self.depth == 0
            || self.depth > 64
            || self.nodes == 0
            || self.work == 0
            || self.output_bytes == 0
            || self.attributes == 0
            || self.rules == 0
        {
            return Err(DataLimitError::Config(
                "mapping safeguards must be positive; depth must not exceed 64".into(),
            ));
        }
        Ok(())
    }
}

/// A monotonically consumed processing-work allowance.
pub struct Budget {
    remaining: usize,
}

impl Budget {
    /// Create an allowance of `work` units.
    #[must_use]
    pub fn new(work: usize) -> Self {
        Self { remaining: work }
    }

    /// Charge work before performing the corresponding operation.
    pub fn charge(&mut self, amount: usize) -> Result<(), DataLimitError> {
        self.remaining = self
            .remaining
            .checked_sub(amount)
            .ok_or(DataLimitError::Limit("traversal work"))?;
        Ok(())
    }

    /// Return the unconsumed allowance.
    #[must_use]
    pub fn remaining(&self) -> usize {
        self.remaining
    }
}

/// Inspect JSON before recursively cloning or serializing caller-owned values.
pub fn inspect_json(
    value: &Value,
    limits: Limits,
    budget: &mut Budget,
) -> Result<(), DataLimitError> {
    fn visit(
        value: &Value,
        depth: usize,
        nodes: &mut usize,
        limits: Limits,
        budget: &mut Budget,
    ) -> Result<(), DataLimitError> {
        budget.charge(1)?;
        if depth > limits.depth {
            return Err(DataLimitError::Limit("nesting depth"));
        }
        *nodes += 1;
        if *nodes > limits.nodes {
            return Err(DataLimitError::Limit("value nodes"));
        }
        match value {
            Value::Object(map) => {
                for (key, value) in map {
                    budget.charge(key.len())?;
                    visit(value, depth + 1, nodes, limits, budget)?;
                }
            }
            Value::Array(array) => {
                for value in array {
                    visit(value, depth + 1, nodes, limits, budget)?;
                }
            }
            Value::String(s) => budget.charge(s.len())?,
            _ => {}
        }
        Ok(())
    }
    visit(value, 1, &mut 0, limits, budget)
}

/// Counts serialized bytes without constructing an unbounded temporary string.
pub fn json_size(value: &impl Serialize, limit: usize) -> Result<usize, DataLimitError> {
    struct Counter {
        size: usize,
        limit: usize,
    }
    impl Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if bytes.len() > self.limit.saturating_sub(self.size) {
                return Err(io::Error::other("JSON byte limit"));
            }
            self.size += bytes.len();
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let mut writer = Counter { size: 0, limit };
    serde_json::to_writer(&mut writer, value)
        .map_err(|_| DataLimitError::Limit("serialized bytes"))?;
    Ok(writer.size)
}

/// Maximum configuration-text file size.
pub const CONFIG_FILE_BYTES: usize = 1024 * 1024;

/// Read limited UTF-8 text without parsing node-specific configuration.
pub fn read_config(path: &Path) -> Result<String, DataLimitError> {
    let file = std::fs::File::open(path)
        .map_err(|e| DataLimitError::File(format!("{}: {e}", path.display())))?;
    let mut bytes = Vec::new();
    _ = file
        .take((CONFIG_FILE_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|e| DataLimitError::File(format!("{}: {e}", path.display())))?;
    if bytes.len() > CONFIG_FILE_BYTES {
        return Err(DataLimitError::Limit("configuration file bytes"));
    }
    String::from_utf8(bytes).map_err(|e| DataLimitError::File(format!("{}: {e}", path.display())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Scenario: consumers inspect and serialize JSON at and beyond shared data limits.
    /// Guarantees: work, node, depth and serialized-byte failures are explicit and exact limits are accepted.
    #[test]
    fn shared_data_limits_enforce_boundaries() {
        let value = json!({"a": "x"});
        assert_eq!(json_size(&value, 9), Ok(9));
        assert_eq!(
            json_size(&value, 8),
            Err(DataLimitError::Limit("serialized bytes"))
        );
        assert!(inspect_json(&value, Limits::default(), &mut Budget::new(4)).is_ok());
        assert_eq!(
            inspect_json(&value, Limits::default(), &mut Budget::new(3)),
            Err(DataLimitError::Limit("traversal work"))
        );
        assert_eq!(
            inspect_json(
                &value,
                Limits {
                    nodes: 1,
                    ..Limits::default()
                },
                &mut Budget::new(100)
            ),
            Err(DataLimitError::Limit("value nodes"))
        );
        assert_eq!(
            inspect_json(
                &value,
                Limits {
                    depth: 1,
                    ..Limits::default()
                },
                &mut Budget::new(100)
            ),
            Err(DataLimitError::Limit("nesting depth"))
        );
        assert!(Limits::default().validate().is_ok());
        assert!(matches!(
            Limits {
                depth: 65,
                ..Limits::default()
            }
            .validate(),
            Err(DataLimitError::Config(_))
        ));
    }

    /// Scenario: generic configuration-text reads encounter valid UTF-8, oversize text, invalid UTF-8 and a missing path.
    /// Guarantees: reads enforce their byte cap and report file/encoding failures without parsing component configuration.
    #[test]
    fn configuration_text_reads_are_limited() {
        let dir = tempfile::tempdir().expect("directory");
        let path = dir.path().join("config.yaml");
        std::fs::write(&path, "x".repeat(CONFIG_FILE_BYTES)).expect("write");
        assert_eq!(
            read_config(&path).expect("exact limit").len(),
            CONFIG_FILE_BYTES
        );
        std::fs::write(&path, "x".repeat(CONFIG_FILE_BYTES + 1)).expect("write");
        assert_eq!(
            read_config(&path),
            Err(DataLimitError::Limit("configuration file bytes"))
        );
        std::fs::write(&path, [0xff]).expect("write");
        assert!(matches!(read_config(&path), Err(DataLimitError::File(_))));
        assert!(matches!(
            read_config(&dir.path().join("missing")),
            Err(DataLimitError::File(_))
        ));
    }
}
