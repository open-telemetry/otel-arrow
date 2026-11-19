// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Explicit single-owner scratch for glob-generated regular expressions.

use super::super::super::multiline_pattern::{
    MemoryEstimate, compile_glob_regex, compile_indexed_globs,
};
use super::ConfigError;
use regex_automata::{Anchored, Input, PatternID, meta, nfa::thompson::pikevm};
use std::{borrow::Cow, cell::RefCell, path::Path};

#[derive(Debug)]
pub(super) struct GlobProgram {
    regex: meta::Regex,
    cache: RefCell<meta::Cache>,
    pub memory: MemoryEstimate,
}
impl GlobProgram {
    pub fn compile(patterns: &[String], field: &str) -> Result<Option<Self>, ConfigError> {
        if patterns.is_empty() {
            return Ok(None);
        }
        let mut unique: Vec<&str> = patterns.iter().map(String::as_str).collect();
        unique.sort_unstable();
        unique.dedup();
        let (regex, memory) =
            compile_glob_regex(&unique).map_err(|e| ConfigError::new(field, e.to_string()))?;
        let cache = RefCell::new(regex.create_cache());
        Ok(Some(Self {
            regex,
            cache,
            memory,
        }))
    }
    pub fn observed_scratch_bytes(&self) -> usize {
        self.cache.borrow().memory_usage()
    }
    pub fn matches(&self, path: &Path, unrepresentable: bool) -> bool {
        let Some(bytes) = native_bytes(path) else {
            return unrepresentable;
        };
        // No pooled cache, locking, retained input, or per-search program creation.
        self.regex
            .search_half_with(
                &mut self.cache.borrow_mut(),
                &Input::new(&bytes).earliest(true),
            )
            .is_some()
    }
}

#[derive(Debug)]
pub(super) struct IndexedGlobProgram {
    regex: pikevm::PikeVM,
    cache: RefCell<pikevm::Cache>,
    pub memory: MemoryEstimate,
}
impl IndexedGlobProgram {
    pub fn compile(patterns: &[&str], field: &str) -> Result<Option<Self>, ConfigError> {
        if patterns.is_empty() {
            return Ok(None);
        }
        let (regex, memory) =
            compile_indexed_globs(patterns).map_err(|e| ConfigError::new(field, e.to_string()))?;
        let cache = RefCell::new(regex.create_cache());
        Ok(Some(Self {
            regex,
            cache,
            memory,
        }))
    }
    pub fn matches_pattern(&self, index: usize, path: &Path, unrepresentable: bool) -> bool {
        if index >= self.regex.get_nfa().pattern_len() {
            return unrepresentable;
        }
        let Ok(id) = PatternID::new(index) else {
            return unrepresentable;
        };
        let Some(bytes) = native_bytes(path) else {
            return unrepresentable;
        };
        self.regex.is_match(
            &mut self.cache.borrow_mut(),
            Input::new(&bytes).anchored(Anchored::Pattern(id)),
        )
    }
    pub fn observed_scratch_bytes(&self) -> usize {
        self.cache.borrow().memory_usage()
    }
}

#[cfg(unix)]
fn native_bytes(path: &Path) -> Option<Cow<'_, [u8]>> {
    use std::os::unix::ffi::OsStrExt;
    Some(Cow::Borrowed(path.as_os_str().as_bytes()))
}
#[cfg(not(unix))]
fn native_bytes(path: &Path) -> Option<Cow<'_, [u8]>> {
    let bytes = path.to_str()?.as_bytes();
    if cfg!(windows) && bytes.contains(&b'\\') {
        Some(Cow::Owned(
            bytes
                .iter()
                .map(|b| if *b == b'\\' { b'/' } else { *b })
                .collect(),
        ))
    } else {
        Some(Cow::Borrowed(bytes))
    }
}
