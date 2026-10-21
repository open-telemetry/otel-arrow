// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Lexical matching only. Discovery supplies handle-verified canonical mappings.

use super::compile::ConfigError;
use globset::{Glob, GlobBuilder};
#[path = "glob_program.rs"]
mod glob_program;
use glob_program::{GlobProgram, IndexedGlobProgram};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

/// Immutable, precompiled native-path policy. Root indices retain configuration order.
#[derive(Debug)]
pub struct PathPolicy {
    includes: Option<GlobProgram>,
    excludes: Option<GlobProgram>,
    excluded_subtrees: Option<GlobProgram>,
    include_roots: Vec<PathBuf>,
    exclude_roots: Vec<PathBuf>,
    mapped_excludes: Option<IndexedGlobProgram>,
    directory_components: Option<IndexedGlobProgram>,
    include_depths: Vec<Option<usize>>,
    directory_rules: Vec<DirectoryRule>,
}

#[derive(Debug, Default)]
struct DirectoryRule {
    components: Vec<usize>,
    recursive: bool,
    prefix_pattern: Option<usize>,
}

impl PathPolicy {
    pub(super) fn compile(
        include: &[String],
        exclude: &[String],
        base: &Path,
    ) -> Result<Self, ConfigError> {
        if include.is_empty() {
            return Err(ConfigError::new(
                "include",
                "requires at least one path glob",
            ));
        }
        if include
            .len()
            .checked_add(exclude.len())
            .is_none_or(|n| n > 1024)
        {
            return Err(ConfigError::new(
                "include/exclude",
                "at most 1024 patterns are permitted",
            ));
        }
        let mut bytes = 0usize;
        for (setting, patterns) in [("include", include), ("exclude", exclude)] {
            for (index, pattern) in patterns.iter().enumerate() {
                if pattern.is_empty() || pattern.len() > 4096 || pattern.contains('\0') {
                    return Err(ConfigError::new(
                        format!("{setting}[{index}]"),
                        "must contain 1..=4096 UTF-8 bytes and no NUL",
                    ));
                }
                bytes = bytes
                    .checked_add(pattern.len())
                    .ok_or_else(|| ConfigError::new(setting, "pattern byte sum overflow"))?;
            }
        }
        if bytes > 1024 * 1024 {
            return Err(ConfigError::new(
                "include/exclude",
                "aggregate pattern text exceeds 1 MiB",
            ));
        }
        let mut include_set = Vec::new();
        let mut include_depths = Vec::new();
        let mut exclude_set = Vec::new();
        let mut excluded_subtrees = Vec::new();
        let mut include_roots = Vec::with_capacity(include.len());
        let mut exclude_roots = Vec::with_capacity(exclude.len());
        let mut mapped_patterns = Vec::with_capacity(exclude.len());
        let mut directory_rules = Vec::with_capacity(include.len());
        let mut component_patterns = HashMap::new();
        for (setting, patterns, set, roots) in [
            ("include", include, &mut include_set, &mut include_roots),
            ("exclude", exclude, &mut exclude_set, &mut exclude_roots),
        ] {
            for (index, pattern) in patterns.iter().enumerate() {
                let field = format!("{setting}[{index}]");
                let absolute = if Path::new(pattern).is_absolute() {
                    pattern.clone()
                } else {
                    if !base.is_absolute() {
                        return Err(ConfigError::new(
                            &field,
                            "relative patterns require an explicit absolute base",
                        ));
                    }
                    if cfg!(windows)
                        && (Path::new(pattern).has_root()
                            || matches!(
                                Path::new(pattern).components().next(),
                                Some(std::path::Component::Prefix(_))
                            ))
                    {
                        return Err(ConfigError::new(
                            &field,
                            "Windows rooted patterns require a drive or UNC prefix",
                        ));
                    }
                    let base = base.to_str().ok_or_else(|| ConfigError::new(&field, "relative patterns require a UTF-8 base; absolute patterns preserve native candidate bytes"))?;
                    format!(
                        "{}/{}",
                        escape_base(
                            base.trim_end_matches(|c| c == '/' || (cfg!(windows) && c == '\\'))
                        ),
                        pattern
                    )
                };
                let glob = build(&absolute, &field)?;
                validate_path_syntax(pattern, &field)?;
                let (root, suffix) = prefix(pattern);
                let root = if Path::new(pattern).is_absolute() {
                    root
                } else {
                    base.join(root)
                };
                if root
                    .components()
                    .any(|c| matches!(c, std::path::Component::ParentDir))
                    || root.as_os_str().as_encoded_bytes().contains(&0)
                {
                    return Err(ConfigError::new(
                        &field,
                        "literal traversal root must not contain parent components or NUL",
                    ));
                }
                roots.push(root);
                if setting == "include" {
                    include_depths.push(directory_depth(build(suffix, &field)?.regex(), &field)?);
                    directory_rules.push(directory_rule(suffix, &field, &mut component_patterns)?);
                }
                if setting == "exclude" {
                    mapped_patterns.push(build(suffix, &field)?.regex().to_owned());
                    // Only an explicit recursive suffix proves subtree exclusion.
                    // A match of a directory name alone says nothing about children.
                    if let Some(parent) = absolute.strip_suffix("/**").or_else(|| {
                        if cfg!(windows) {
                            absolute.strip_suffix(r"\**")
                        } else {
                            None
                        }
                    }) {
                        excluded_subtrees.push(glob.regex().to_owned());
                        // globset 0.4 treats the final slash as required. The
                        // directory itself still has every descendant excluded.
                        excluded_subtrees.push(
                            build(if parent.is_empty() { "/" } else { parent }, &field)?
                                .regex()
                                .to_owned(),
                        );
                    }
                }
                set.push(glob.regex().to_owned());
            }
        }
        let mapped: Vec<_> = mapped_patterns.iter().map(String::as_str).collect();
        let mut components = vec![""; component_patterns.len()];
        for (pattern, index) in &component_patterns {
            components[*index] = pattern.as_str();
        }
        Ok(Self {
            includes: GlobProgram::compile(&include_set, "include")?,
            excludes: GlobProgram::compile(&exclude_set, "exclude")?,
            excluded_subtrees: GlobProgram::compile(&excluded_subtrees, "exclude")?,
            include_depths,
            include_roots,
            exclude_roots,
            mapped_excludes: IndexedGlobProgram::compile(&mapped, "exclude")?,
            directory_components: IndexedGlobProgram::compile(&components, "include traversal")?,
            directory_rules,
        })
    }

    /// Conservative retained configuration and worker-scratch charge.
    /// Includes capacity growth allowances from the shared audited regex model.
    pub fn retained_bytes(&self) -> Result<u64, ConfigError> {
        self.program_bytes()?
            .checked_add(self.programs().try_fold(0u64, |n, (memory, _)| {
                n.checked_add(memory.worker_heap_bound as u64)
                    .ok_or_else(|| {
                        ConfigError::new("include/exclude", "scratch memory sum overflow")
                    })
            })?)
            .ok_or_else(|| ConfigError::new("include/exclude", "matcher memory sum overflow"))
    }

    /// Retained program and owned-container allowance, excluding search scratch.
    pub fn program_bytes(&self) -> Result<u64, ConfigError> {
        let mut total = size_of::<Self>() as u64;
        let mut add = |n: usize| -> Result<(), ConfigError> {
            total = total.checked_add(n as u64).ok_or_else(|| {
                ConfigError::new("include/exclude", "program memory sum overflow")
            })?;
            Ok(())
        };
        add(self.include_roots.capacity() * size_of::<PathBuf>())?;
        add(self.exclude_roots.capacity() * size_of::<PathBuf>())?;
        add(self.include_depths.capacity() * size_of::<Option<usize>>())?;
        add(self.directory_rules.capacity() * size_of::<DirectoryRule>())?;
        for rule in &self.directory_rules {
            add(rule.components.capacity() * size_of::<usize>())?;
        }
        for root in self.include_roots.iter().chain(&self.exclude_roots) {
            add(root.capacity())?;
        }
        for (memory, _) in self.programs() {
            add(memory.program_heap_bound)?;
        }
        Ok(total)
    }

    /// Current engine-reported scratch bytes, for diagnostics rather than admission.
    /// The reported value does not include every allocator/capacity allowance.
    pub fn observed_scratch_bytes(&self) -> Result<u64, ConfigError> {
        self.programs().try_fold(0u64, |n, (_, observed)| {
            n.checked_add(observed as u64)
                .ok_or_else(|| ConfigError::new("include/exclude", "observed scratch sum overflow"))
        })
    }

    fn programs(
        &self,
    ) -> impl Iterator<Item = (super::super::multiline_pattern::MemoryEstimate, usize)> {
        let any =
            |p: &Option<GlobProgram>| p.as_ref().map(|p| (p.memory, p.observed_scratch_bytes()));
        let indexed = |p: &Option<IndexedGlobProgram>| {
            p.as_ref().map(|p| (p.memory, p.observed_scratch_bytes()))
        };
        [
            any(&self.includes),
            any(&self.excludes),
            any(&self.excluded_subtrees),
            indexed(&self.mapped_excludes),
            indexed(&self.directory_components),
        ]
        .into_iter()
        .flatten()
    }

    /// Check native root lengths against the selected scanner's buffer capacity.
    /// Startup calls this before constructing the scanner; no default capacity is assumed.
    pub fn validate_root_lengths(&self, path_bytes: usize) -> Result<(), ConfigError> {
        for (setting, roots) in [
            ("include", &self.include_roots),
            ("exclude", &self.exclude_roots),
        ] {
            for (index, root) in roots.iter().enumerate() {
                #[cfg(windows)]
                let bytes = {
                    use std::os::windows::ffi::OsStrExt;
                    root.as_os_str()
                        .encode_wide()
                        .count()
                        .checked_mul(2)
                        .ok_or_else(|| {
                            ConfigError::new(
                                format!("{setting}[{index}]"),
                                "native root length overflow",
                            )
                        })?
                };
                #[cfg(not(windows))]
                let bytes = root.as_os_str().len();
                if bytes >= path_bytes {
                    return Err(ConfigError::new(
                        format!("{setting}[{index}]"),
                        format!(
                            "native traversal root has {bytes} bytes; scanner path capacity {path_bytes} must leave room for a terminator"
                        ),
                    ));
                }
            }
        }
        Ok(())
    }

    /// Lexical traversal roots; discovery must resolve and verify their objects.
    #[must_use]
    pub fn include_roots(&self) -> &[PathBuf] {
        &self.include_roots
    }
    /// Exclusion roots in the same order as `excludes_mapped` indices.
    #[must_use]
    pub fn exclude_roots(&self) -> &[PathBuf] {
        &self.exclude_roots
    }
    /// Whether the lexical native path matches an include. Exclusions are separate.
    #[must_use]
    pub fn includes(&self, path: &Path) -> bool {
        self.includes
            .as_ref()
            .is_some_and(|p| p.matches(path, false))
    }
    /// Whether directory names and depth can lead to a configured include.
    /// Separator-bearing syntax uses a compiled language of directory prefixes.
    #[must_use]
    pub fn may_descend(&self, path: &Path) -> bool {
        self.include_roots
            .iter()
            .zip(&self.include_depths)
            .zip(&self.directory_rules)
            .any(|((root, depth), rule)| {
                if root.starts_with(path) {
                    return true;
                }
                let Ok(suffix) = path.strip_prefix(root) else {
                    return false;
                };
                if depth.is_some_and(|max| suffix.components().count() > max) {
                    return false;
                }
                if let Some(pattern) = rule.prefix_pattern {
                    return self
                        .directory_components
                        .as_ref()
                        .is_some_and(|program| program.matches_pattern(pattern, suffix, true));
                }
                for (index, component) in suffix.components().enumerate() {
                    let Some(pattern) = rule.components.get(index) else {
                        return rule.recursive;
                    };
                    if !self.directory_components.as_ref().is_some_and(|program| {
                        program.matches_pattern(*pattern, Path::new(component.as_os_str()), true)
                    }) {
                        return false;
                    }
                }
                true
            })
    }
    /// Apply to both lexical and verified target paths before reading content.
    #[must_use]
    pub fn excludes(&self, path: &Path) -> bool {
        self.excludes
            .as_ref()
            .is_some_and(|p| p.matches(path, true))
    }
    /// Prune only when a matching explicit `/**` suffix covers all descendants.
    #[must_use]
    pub fn excludes_subtree(&self, path: &Path) -> bool {
        self.excluded_subtrees
            .as_ref()
            .is_some_and(|p| p.matches(path, false))
    }
    /// Match the suffix of a verified canonical exclusion root without joining paths.
    /// Invalid indices and non-relative suffixes fail closed.
    #[must_use]
    pub fn excludes_mapped(&self, root_index: usize, suffix: &Path) -> bool {
        if suffix
            .components()
            .any(|c| !matches!(c, std::path::Component::Normal(_)))
        {
            return true;
        }
        if root_index >= self.exclude_roots.len() {
            return true;
        }
        self.mapped_excludes
            .as_ref()
            .is_none_or(|p| p.matches_pattern(root_index, suffix, true))
    }
}

fn build(pattern: &str, setting: &str) -> Result<Glob, ConfigError> {
    GlobBuilder::new(pattern)
        .literal_separator(true)
        .backslash_escape(!cfg!(windows))
        .build()
        .map_err(|e| ConfigError::new(setting, e.to_string()))
}

// Find the last separator before any metacharacter. Escaped Unix metacharacters
// are literal root bytes; the suffix retains its original glob syntax.
fn prefix(pattern: &str) -> (PathBuf, &str) {
    let mut literal = String::new();
    let mut root = String::new();
    let mut suffix = 0;
    let mut chars = pattern.char_indices();
    while let Some((i, ch)) = chars.next() {
        if ch == '\\' && !cfg!(windows) {
            if let Some((_, escaped)) = chars.next() {
                literal.push(escaped);
            }
        } else if ch == '/' || (cfg!(windows) && ch == '\\') {
            literal.push(ch);
            root.clone_from(&literal);
            suffix = i + ch.len_utf8();
        } else if matches!(ch, '*' | '?' | '[' | '{') {
            break;
        } else {
            literal.push(ch);
        }
    }
    (PathBuf::from(root), &pattern[suffix..])
}

fn escape_base(base: &str) -> String {
    let literal = globset::escape(base);
    if cfg!(windows) {
        literal
    } else {
        literal.replace('\\', "\\\\")
    }
}

fn validate_path_syntax(pattern: &str, field: &str) -> Result<(), ConfigError> {
    if pattern.contains('\0') {
        return Err(ConfigError::new(
            field,
            "resolved lexical pattern must not contain NUL",
        ));
    }
    if component_tokens(pattern)
        .split(|c| c == '/' || (cfg!(windows) && c == '\\'))
        .any(|c| c == "..")
    {
        return Err(ConfigError::new(
            field,
            "parent-directory components (..) are not supported",
        ));
    }
    // Literal prefixes are preserved by discovery. In a wildcard suffix, the
    // walker cannot manufacture extra separators or dot components.
    let (_, suffix) = prefix(pattern);
    if suffix.is_empty() {
        return Err(ConfigError::new(
            field,
            "pattern must name files, not end in a separator",
        ));
    }
    if component_tokens(suffix)
        .split(|c| c == '/' || (cfg!(windows) && c == '\\'))
        .any(|c| c.is_empty() || c == ".")
    {
        return Err(ConfigError::new(
            field,
            "empty or dot components after a wildcard cannot match discovered names",
        ));
    }
    Ok(())
}

// Only classify literal path components. A slash inside a character class is
// part of glob syntax, not an operator-written directory component. Globset has
// already validated classes; its first `]` (after optional negation) is literal.
fn component_tokens(pattern: &str) -> String {
    let mut out = String::with_capacity(pattern.len());
    let mut chars = pattern.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' && !cfg!(windows) {
            if let Some(literal) = chars.next() {
                out.push(literal);
            }
        } else if c == '[' {
            out.push('x');
            if matches!(chars.peek(), Some('!' | '^')) {
                let _ = chars.next();
            }
            let mut first = true;
            for item in chars.by_ref() {
                if item == ']' && !first {
                    break;
                }
                first = false;
            }
        } else {
            out.push(c);
        }
    }
    out
}

// Derive a conservative directory depth from translated glob syntax. Counting
// slash-capable HIR nodes handles classes, escaped separators and alternatives.
fn directory_depth(regex: &str, field: &str) -> Result<Option<usize>, ConfigError> {
    use regex_syntax::hir::{Class, Hir, HirKind};
    fn count(hir: &Hir) -> Option<usize> {
        match hir.kind() {
            HirKind::Empty | HirKind::Look(_) => Some(0),
            HirKind::Literal(lit) => Some(lit.0.iter().filter(|b| **b == b'/').count()),
            HirKind::Class(class) => Some(usize::from(match class {
                Class::Bytes(c) => c
                    .ranges()
                    .iter()
                    .any(|r| r.start() <= b'/' && b'/' <= r.end()),
                Class::Unicode(c) => c
                    .ranges()
                    .iter()
                    .any(|r| r.start() <= '/' && '/' <= r.end()),
            })),
            HirKind::Capture(c) => count(&c.sub),
            HirKind::Concat(parts) => parts
                .iter()
                .try_fold(0usize, |sum, part| sum.checked_add(count(part)?)),
            HirKind::Alternation(parts) => parts
                .iter()
                .try_fold(0usize, |max, part| Some(max.max(count(part)?))),
            HirKind::Repetition(rep) => {
                let n = count(&rep.sub)?;
                if n == 0 {
                    Some(0)
                } else {
                    n.checked_mul(rep.max? as usize)
                }
            }
        }
    }
    let hir = regex_syntax::ParserBuilder::new()
        .utf8(false)
        .unicode(false)
        .dot_matches_new_line(true)
        .build()
        .parse(regex)
        .map_err(|e| ConfigError::new(field, e.to_string()))?;
    // None means unbounded traversal; arithmetic uncertainty also conservatively
    // disables pruning instead of excluding a directory that might match.
    Ok(count(&hir))
}

fn directory_rule(
    suffix: &str,
    field: &str,
    names: &mut HashMap<String, usize>,
) -> Result<DirectoryRule, ConfigError> {
    let Some(parts) = split_components(suffix) else {
        return directory_prefix_rule(suffix, field, names);
    };
    let mut rule = DirectoryRule::default();
    for (index, part) in parts.iter().enumerate() {
        if *part == "**" {
            rule.recursive = true;
            break;
        }
        let glob = build(part, field)?;
        if directory_depth(glob.regex(), field)? != Some(0) {
            return directory_prefix_rule(suffix, field, names);
        }
        if index + 1 == parts.len() {
            break;
        } // The last component names files.
        let id = if let Some(id) = names.get(glob.regex()) {
            *id
        } else {
            let id = names.len();
            let _ = names.insert(glob.regex().to_owned(), id);
            id
        };
        rule.components.push(id);
    }
    Ok(rule)
}

// Split only unambiguous component boundaries. Alternatives/classes containing
// separators use a compiled directory-prefix language instead.
fn split_components(pattern: &str) -> Option<Vec<&str>> {
    let mut parts = Vec::new();
    let mut chars = pattern.char_indices().peekable();
    let mut braces = 0usize;
    let mut start = 0;
    while let Some((offset, c)) = chars.next() {
        if c == '\\' && !cfg!(windows) {
            if chars.next().is_some_and(|(_, escaped)| escaped == '/') {
                return None;
            }
        } else if c == '[' {
            if matches!(chars.peek(), Some((_, '!' | '^'))) {
                let _ = chars.next();
            }
            let mut first = true;
            for (_, item) in chars.by_ref() {
                if item == '/' || (cfg!(windows) && item == '\\') {
                    return None;
                }
                if item == ']' && !first {
                    break;
                }
                first = false;
            }
        } else if c == '{' {
            braces += 1;
        } else if c == '}' {
            braces = braces.saturating_sub(1);
        } else if c == '/' || (cfg!(windows) && c == '\\') {
            if braces != 0 {
                return None;
            }
            parts.push(&pattern[start..offset]);
            start = offset + c.len_utf8();
        }
    }
    parts.push(&pattern[start..]);
    Some(parts)
}

// Compile exactly the prefixes that precede a separator in the glob language.
// Keeping alternatives in HIR avoids expanding their Cartesian product.
fn directory_prefix_rule(
    suffix: &str,
    field: &str,
    names: &mut HashMap<String, usize>,
) -> Result<DirectoryRule, ConfigError> {
    use regex_syntax::hir::{Class, Hir, HirKind, Repetition};
    fn prefixes(hir: &Hir) -> Hir {
        match hir.kind() {
            HirKind::Literal(literal) => {
                // Share the logical suffix structure instead of materializing
                // every full prefix of a separator-heavy literal.
                let mut parts = literal.0.rsplit(|byte| *byte == b'/');
                let _ = parts.next(); // The final component does not precede a slash.
                let mut tail = Hir::fail();
                for part in parts {
                    tail = Hir::concat(vec![
                        Hir::literal(part),
                        Hir::alternation(vec![
                            Hir::empty(),
                            Hir::concat(vec![Hir::literal(*b"/"), tail]),
                        ]),
                    ]);
                }
                tail
            }
            HirKind::Class(class) => {
                let slash = match class {
                    Class::Bytes(c) => c
                        .ranges()
                        .iter()
                        .any(|r| r.start() <= b'/' && b'/' <= r.end()),
                    Class::Unicode(c) => c
                        .ranges()
                        .iter()
                        .any(|r| r.start() <= '/' && '/' <= r.end()),
                };
                if slash { Hir::empty() } else { Hir::fail() }
            }
            HirKind::Capture(capture) => prefixes(&capture.sub),
            HirKind::Alternation(parts) => Hir::alternation(parts.iter().map(prefixes).collect()),
            HirKind::Concat(parts) => {
                let mut tail = Hir::fail();
                for part in parts.iter().rev() {
                    tail = Hir::alternation(vec![
                        prefixes(part),
                        Hir::concat(vec![part.clone(), tail]),
                    ]);
                }
                tail
            }
            HirKind::Repetition(rep) => {
                if rep.max == Some(0) {
                    return Hir::fail();
                }
                Hir::concat(vec![
                    Hir::repetition(Repetition {
                        min: 0,
                        max: rep.max.map(|n| n - 1),
                        greedy: rep.greedy,
                        sub: rep.sub.clone(),
                    }),
                    prefixes(&rep.sub),
                ])
            }
            HirKind::Empty | HirKind::Look(_) => Hir::fail(),
        }
    }
    let glob = build(suffix, field)?;
    let hir = regex_syntax::ParserBuilder::new()
        .utf8(false)
        .unicode(false)
        .dot_matches_new_line(true)
        .build()
        .parse(glob.regex())
        .map_err(|e| ConfigError::new(field, e.to_string()))?;
    let pattern = format!("(?-u)^(?:{})$", prefixes(&hir));
    let next = names.len();
    let id = *names.entry(pattern).or_insert(next);
    Ok(DirectoryRule {
        prefix_pattern: Some(id),
        ..DirectoryRule::default()
    })
}
