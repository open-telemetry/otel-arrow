// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared limits for endpoint compilation and event mapping.

#![allow(dead_code)] // Invoked by the runtime in a later phase.

use serde::Serialize;
use serde_json::Value;
use std::io::{self, Read, Write};
use std::path::Path;

/// A failure is explicit and never returns a partially mapped log.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub(crate) enum MappingError {
    #[error("invalid endpoint configuration: {0}")]
    Config(String),
    #[error("endpoint file error: {0}")]
    File(String),
    #[error("event rejected: {0}")]
    Rejected(String),
    #[error("event rejected by instrumentation key origin policy")]
    Unauthorized,
    #[error("request payload rejected by event override")]
    RejectedPayload,
    #[error("mapping limit exceeded: {0}")]
    Limit(&'static str),
}

/// Internal safeguards, independent of the configurable protocol byte limits.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Limits {
    pub depth: usize,
    pub nodes: usize,
    pub work: usize,
    pub output_bytes: usize,
    pub attributes: usize,
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
    pub fn validate(self) -> Result<(), MappingError> {
        if self.depth == 0
            || self.depth > 64
            || self.nodes == 0
            || self.work == 0
            || self.output_bytes == 0
            || self.attributes == 0
            || self.rules == 0
        {
            return Err(MappingError::Config(
                "mapping safeguards must be positive; depth must not exceed 64".into(),
            ));
        }
        Ok(())
    }
}

pub(crate) struct Budget {
    remaining: usize,
}

impl Budget {
    pub fn new(work: usize) -> Self {
        Self { remaining: work }
    }

    pub fn charge(&mut self, amount: usize) -> Result<(), MappingError> {
        self.remaining = self
            .remaining
            .checked_sub(amount)
            .ok_or(MappingError::Limit("traversal work"))?;
        Ok(())
    }

    pub fn remaining(&self) -> usize {
        self.remaining
    }
}

/// Inspect before recursively cloning or serializing caller-owned JSON.
pub(crate) fn inspect(
    value: &Value,
    limits: Limits,
    budget: &mut Budget,
) -> Result<(), MappingError> {
    fn visit(
        value: &Value,
        depth: usize,
        nodes: &mut usize,
        limits: Limits,
        budget: &mut Budget,
    ) -> Result<(), MappingError> {
        budget.charge(1)?;
        if depth > limits.depth {
            return Err(MappingError::Limit("nesting depth"));
        }
        *nodes += 1;
        if *nodes > limits.nodes {
            return Err(MappingError::Limit("value nodes"));
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
pub(crate) fn json_size(value: &impl Serialize, limit: usize) -> Result<usize, MappingError> {
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
        .map_err(|_| MappingError::Limit("serialized bytes"))?;
    Ok(writer.size)
}

pub(crate) const CONFIG_FILE_BYTES: usize = 1024 * 1024;

pub(crate) fn read_config(path: &Path) -> Result<String, MappingError> {
    let file = std::fs::File::open(path)
        .map_err(|e| MappingError::File(format!("{}: {e}", path.display())))?;
    let mut bytes = Vec::new();
    _ = file
        .take((CONFIG_FILE_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|e| MappingError::File(format!("{}: {e}", path.display())))?;
    if bytes.len() > CONFIG_FILE_BYTES {
        return Err(MappingError::Limit("configuration file bytes"));
    }
    String::from_utf8(bytes).map_err(|e| MappingError::File(format!("{}: {e}", path.display())))
}
