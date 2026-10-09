// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bounded `re2-v1` boundary patterns. See `docs/multiline.md` for caller contracts.

use regex_automata::{
    Input, meta,
    nfa::thompson::{self, WhichCaptures},
};
use regex_syntax::{
    ast::{
        self, AssertionKind, Ast, ClassPerl, ClassPerlKind, ClassSet, ClassSetItem, Flag,
        FlagsItemKind, HexLiteralKind, LiteralKind, RepetitionKind, RepetitionRange,
    },
    hir::Hir,
};
use std::sync::Arc;
use thiserror::Error;

#[path = "multiline_pattern/memory.rs"]
mod memory;
pub use memory::MemoryEstimate;

/// Maximum configured pattern length in UTF-8 bytes.
pub const MAX_PATTERN_BYTES: usize = 4096;
/// Maximum parsed syntax-tree depth in the executable profile.
pub const MAX_PATTERN_NESTING: u32 = 64;
/// Maximum compiled program payload permitted by this primitive.
pub const MAX_PROGRAM_BYTES: usize = 10 * 1024 * 1024;
/// Maximum capacity of one lazy-DFA cache.
pub const MAX_LAZY_CACHE_BYTES: usize = 2 * 1024 * 1024;

/// Selects validated decoded text or exact source-byte matching.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PatternMode {
    /// UTF-8 text with ASCII Perl classes and boundaries.
    Text,
    /// Raw source bytes; braced and fixed-width hex escapes both denote bytes.
    Raw,
}

/// Execution limits, not proof of funding from a receiver memory ledger.
#[derive(Clone, Copy, Debug)]
pub struct PatternLimits {
    /// Forward-NFA compiler and aggregate program-payload ceiling, at most 10 MiB.
    pub program_bytes: usize,
    /// Per lazy-cache ceiling, at most 2 MiB; zero disables acceleration.
    pub lazy_cache_bytes: usize,
}

impl Default for PatternLimits {
    fn default() -> Self {
        Self {
            program_bytes: MAX_PROGRAM_BYTES,
            lazy_cache_bytes: MAX_LAZY_CACHE_BYTES,
        }
    }
}

/// A complete physical-line body, already bounded by the caller, without its LF.
#[derive(Clone, Copy, Debug)]
pub enum PatternInput<'a> {
    /// Validated decoded text; matching performs no UTF-8 validation pass.
    Text(&'a str),
    /// Uninterpreted source bytes.
    Raw(&'a [u8]),
}

/// Why compilation selected a policy without lazy-DFA acceleration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccelerationDisabled {
    /// The caller configured zero lazy-cache capacity.
    Requested,
    /// The forward accelerator cannot fit the requested lazy-cache capacity.
    CacheCapacity,
    /// Forward/reverse accelerated programs exceeded the aggregate ceiling.
    ProgramLimit,
    /// Building the accelerated representation failed after forward-NFA validation.
    BuildFailure,
    /// Meta selected another strategy or could not build an accelerator in its cache limit.
    EngineSelection,
}

/// Observable compile-time policy; meta may still choose its engine per search.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecutionPolicy {
    /// A lazy accelerator is available, with equivalent NFA fallback when needed.
    LazyDfa {
        /// Configured ceiling of each lazy cache.
        cache_bytes: usize,
    },
    /// Literal/prefilter or NFA execution, without a lazy accelerator.
    NoLazyDfa {
        /// Requested cache ceiling, retained for diagnostics even when unused.
        requested_cache_bytes: usize,
        /// Configuration/admission reason, suitable for caller-owned startup telemetry.
        reason: AccelerationDisabled,
    },
}

/// Compilation, caller-contract, or accounting-diagnostic failure.
#[derive(Clone, Debug, Error)]
pub enum PatternError {
    /// Pattern source exceeds the profile limit.
    #[error("multiline pattern exceeds 4096 bytes")]
    TooLong,
    /// Malformed or unsupported executable syntax.
    #[error("invalid multiline pattern: {0}")]
    Syntax(String),
    /// Syntax outside the supported profile.
    #[error("unsupported re2-v1 construct: {0}")]
    Unsupported(&'static str),
    /// Program construction failed within the configured limits.
    #[error("multiline pattern compilation failed: {0}")]
    Compile(String),
    /// A configured or compiled resource exceeds its ceiling.
    #[error("multiline {resource} limit exceeded: required {required}, available {available}")]
    Limit {
        /// Resource whose limit was exceeded.
        resource: &'static str,
        /// Requested or reported size.
        required: usize,
        /// Allowed size.
        available: usize,
    },
    /// Source representation differs from the pattern's compiled mode.
    #[error("multiline pattern input does not match its text/raw mode")]
    InputMode,
    /// Post-search diagnostic detected a model violation; discard this matcher.
    #[error("multiline memory model exceeded: observed {observed}, bound {bound}")]
    MemoryModel {
        /// Engine-reported scratch size.
        observed: usize,
        /// Derived peak requested-heap bound.
        bound: usize,
    },
    /// Memory-model arithmetic is not representable.
    #[error("multiline memory estimate overflow")]
    MemoryOverflow,
}

#[derive(Debug)]
struct Program {
    mode: PatternMode,
    regex: meta::Regex,
    policy: ExecutionPolicy,
    memory: MemoryEstimate,
}

/// Shareable immutable program. Cloning shares the program without copying it.
#[derive(Clone, Debug)]
pub struct BoundaryPattern {
    program: Arc<Program>,
}

impl BoundaryPattern {
    /// Compile using the same bounded path as explicit limits.
    /// Receiver-wide construction and activation funding remain caller responsibilities.
    pub fn compile(pattern: &str, mode: PatternMode) -> Result<Self, PatternError> {
        Self::compile_with_limits(pattern, mode, PatternLimits::default())
    }

    /// Compile with explicit limits. No worker-budget estimate silently changes the policy.
    pub fn compile_with_limits(
        pattern: &str,
        mode: PatternMode,
        limits: PatternLimits,
    ) -> Result<Self, PatternError> {
        validate_limits(limits)?;
        let hir = parse(pattern, mode)?;
        build(&hir, mode, limits, true, false)
    }

    /// Compiler policy and any reason for disabling acceleration.
    #[must_use]
    pub fn execution_policy(&self) -> ExecutionPolicy {
        self.program.policy
    }

    /// Estimates for a caller's admission model; this does not reserve memory.
    #[must_use]
    pub fn memory_estimate(&self) -> MemoryEstimate {
        self.program.memory
    }

    /// Engine-reported program payload; excludes allocator and wrapper bookkeeping.
    #[must_use]
    pub fn program_memory_usage(&self) -> usize {
        self.program.memory.program_payload_bytes
    }

    /// Create owned worker state. The caller must separately fund its lifetime.
    #[must_use]
    pub fn matcher(&self) -> BoundaryMatcher {
        BoundaryMatcher {
            program: Arc::clone(&self.program),
            cache: self.program.regex.create_cache(),
            terminal_error: None,
        }
    }
}

fn validate_limits(limits: PatternLimits) -> Result<(), PatternError> {
    for (resource, requested, maximum) in [
        ("program", limits.program_bytes, MAX_PROGRAM_BYTES),
        ("lazy cache", limits.lazy_cache_bytes, MAX_LAZY_CACHE_BYTES),
    ] {
        if requested > maximum {
            return Err(PatternError::Limit {
                resource,
                required: requested,
                available: maximum,
            });
        }
    }
    Ok(())
}

fn parse(pattern: &str, mode: PatternMode) -> Result<Hir, PatternError> {
    if pattern.len() > MAX_PATTERN_BYTES {
        return Err(PatternError::TooLong);
    }
    let mut tree = ast::parse::ParserBuilder::new()
        .nest_limit(MAX_PATTERN_NESTING)
        .build()
        .parse(pattern)
        .map_err(|e| PatternError::Syntax(e.to_string()))?;
    normalize(&mut tree, 1000, mode)?;
    regex_syntax::hir::translate::TranslatorBuilder::new()
        .unicode(mode == PatternMode::Text)
        .utf8(mode == PatternMode::Text)
        .build()
        .translate(pattern, &tree)
        .map_err(|e| PatternError::Syntax(e.kind().to_string()))
}

fn forward_nfa(hir: &Hir, mode: PatternMode, limit: usize) -> Result<thompson::NFA, PatternError> {
    thompson::NFA::compiler()
        .configure(
            thompson::Config::new()
                .utf8(mode == PatternMode::Text)
                .shrink(false)
                .which_captures(WhichCaptures::Implicit)
                .nfa_size_limit(Some(limit)),
        )
        .build_from_hir(hir)
        .map_err(|e| PatternError::Compile(e.to_string()))
}

fn build(
    hir: &Hir,
    mode: PatternMode,
    limits: PatternLimits,
    prefilter: bool,
    backtrack: bool,
) -> Result<BoundaryPattern, PatternError> {
    build_with_payload_limit(
        hir,
        mode,
        limits,
        prefilter,
        backtrack,
        limits.program_bytes,
    )
}

fn build_with_payload_limit(
    hir: &Hir,
    mode: PatternMode,
    limits: PatternLimits,
    prefilter: bool,
    backtrack: bool,
    payload_limit: usize,
) -> Result<BoundaryPattern, PatternError> {
    // Validate the forward program once, before meta can attempt a reverse program.
    // This also supplies exact state/epsilon counts for scratch sizing. Drop it
    // before meta construction so the sizing copy is never retained by workers.
    let nfa = forward_nfa(hir, mode, limits.program_bytes)?;
    let scratch = memory::NfaScratch::from_nfa(&nfa, backtrack)?;
    let minimum_cache = regex_automata::hybrid::dfa::Config::new()
        .starts_for_each_pattern(true)
        .unicode_word_boundary(true)
        .get_minimum_cache_capacity(&nfa)
        .map_err(|e| PatternError::Compile(e.to_string()))?;
    let requested_acceleration = limits.lazy_cache_bytes > 0;
    let accelerated = requested_acceleration && limits.lazy_cache_bytes >= minimum_cache;
    let initial_reason = if requested_acceleration && !accelerated {
        Some(AccelerationDisabled::CacheCapacity)
    } else {
        None
    };
    drop(nfa);
    let config = meta::Config::new()
        .utf8_empty(mode == PatternMode::Text)
        .which_captures(WhichCaptures::Implicit)
        .nfa_size_limit(Some(limits.program_bytes))
        .hybrid(accelerated)
        .hybrid_cache_capacity(limits.lazy_cache_bytes)
        .dfa(false)
        .onepass(false)
        .backtrack(backtrack)
        .auto_prefilter(prefilter)
        .pool_capacity(0);
    let first = meta::Regex::builder()
        .configure(config.clone())
        .build_from_hir(hir);
    let retry_reason = match &first {
        Ok(regex) if regex.memory_usage() > payload_limit => {
            Some(AccelerationDisabled::ProgramLimit)
        }
        Err(_) => Some(AccelerationDisabled::BuildFailure),
        _ => None,
    };
    let (regex, disabled) = if accelerated && retry_reason.is_some() {
        drop(first);
        // The forward NFA already passed. A reverse-NFA size failure is
        // recoverable; size_limit() alone does not identify its direction.
        let regex = meta::Regex::builder()
            .configure(config.hybrid(false))
            .build_from_hir(hir)
            .map_err(|e| PatternError::Compile(e.to_string()))?;
        (regex, retry_reason)
    } else {
        (
            first.map_err(|e| PatternError::Compile(e.to_string()))?,
            initial_reason,
        )
    };
    let payload = regex.memory_usage();
    if payload > payload_limit {
        return Err(PatternError::Limit {
            resource: "aggregate program",
            required: payload,
            available: payload_limit,
        });
    }
    // Unlike get_config().get_hybrid(), a nonempty initial meta cache proves
    // that a lazy engine was actually built (other eager engines are disabled).
    let initial = regex.create_cache().memory_usage();
    let policy = if initial > 0 {
        ExecutionPolicy::LazyDfa {
            cache_bytes: limits.lazy_cache_bytes,
        }
    } else {
        ExecutionPolicy::NoLazyDfa {
            requested_cache_bytes: limits.lazy_cache_bytes,
            reason: disabled.unwrap_or(if requested_acceleration {
                AccelerationDisabled::EngineSelection
            } else {
                AccelerationDisabled::Requested
            }),
        }
    };
    let growing_caches = if initial == 0 {
        0
    } else if !prefilter
        || hir
            .properties()
            .look_set_prefix()
            .contains(regex_syntax::hir::Look::Start)
        || hir
            .properties()
            .look_set_suffix()
            .contains(regex_syntax::hir::Look::End)
    {
        1
    } else {
        3
    };
    let memory = scratch.estimate(payload, initial, growing_caches, limits.lazy_cache_bytes)?;
    Ok(BoundaryPattern {
        program: Arc::new(Program {
            mode,
            regex,
            policy,
            memory,
        }),
    })
}

/// Owned worker state; it can outlive the handle from which it was created.
#[derive(Debug)]
pub struct BoundaryMatcher {
    program: Arc<Program>,
    cache: meta::Cache,
    terminal_error: Option<PatternError>,
}

impl BoundaryMatcher {
    /// Search without retaining input. A memory-model violation latches a terminal error.
    /// The diagnostic runs after allocation and is not an allocator-level limit.
    pub fn is_match(&mut self, body: PatternInput<'_>) -> Result<bool, PatternError> {
        if let Some(error) = &self.terminal_error {
            return Err(error.clone());
        }
        let bytes = match (self.program.mode, body) {
            (PatternMode::Text, PatternInput::Text(text)) => text.as_bytes(),
            (PatternMode::Raw, PatternInput::Raw(bytes)) => bytes,
            _ => return Err(PatternError::InputMode),
        };
        let matched = self
            .program
            .regex
            .search_half_with(&mut self.cache, &Input::new(bytes).earliest(true))
            .is_some();
        let observed = self.cache.memory_usage();
        let bound = self.program.memory.worker_heap_bound;
        if observed > bound {
            let error = PatternError::MemoryModel { observed, bound };
            self.terminal_error = Some(error.clone());
            return Err(error);
        }
        Ok(matched)
    }

    /// Diagnostic engine report, not allocator capacity or proof of funded memory.
    #[must_use]
    pub fn cache_memory_usage(&self) -> usize {
        self.cache.memory_usage()
    }
}

/// Internal qualification controls; absent from normal library builds.
#[cfg(any(test, feature = "bench"))]
pub(crate) mod qualification {
    use super::*;
    // Harnesses include this module directly; the linked library's bench build
    // does not call this entry point. Normal library builds omit the module.
    #[allow(dead_code)]
    pub(crate) fn compile(
        pattern: &str,
        mode: PatternMode,
        limits: PatternLimits,
        prefilter: bool,
        backtrack: bool,
    ) -> Result<BoundaryPattern, PatternError> {
        validate_limits(limits)?;
        build(&parse(pattern, mode)?, mode, limits, prefilter, backtrack)
    }
}

fn flags(flags: &ast::Flags) -> Result<(), PatternError> {
    for item in &flags.items {
        if matches!(
            item.kind,
            FlagsItemKind::Flag(Flag::Unicode | Flag::CRLF | Flag::IgnoreWhitespace)
        ) {
            return Err(PatternError::Unsupported("u, R, or x flag"));
        }
    }
    Ok(())
}

fn literal(literal: &mut ast::Literal, mode: PatternMode) -> Result<(), PatternError> {
    if matches!(
        literal.kind,
        LiteralKind::HexFixed(HexLiteralKind::UnicodeShort | HexLiteralKind::UnicodeLong)
            | LiteralKind::HexBrace(HexLiteralKind::UnicodeShort | HexLiteralKind::UnicodeLong)
    ) {
        return Err(PatternError::Unsupported(
            "use RE2 hex escapes instead of u/U escapes",
        ));
    }
    if mode == PatternMode::Raw && matches!(literal.kind, LiteralKind::HexBrace(HexLiteralKind::X))
    {
        if u32::from(literal.c) > 0xff {
            return Err(PatternError::Unsupported("raw hex escape exceeds one byte"));
        }
        literal.kind = LiteralKind::HexFixed(HexLiteralKind::X);
    }
    Ok(())
}

fn perl_class(class: &ClassPerl) -> ast::ClassBracketed {
    // RE2's Perl space class excludes vertical tab; POSIX [:space:] includes it.
    let source = match (&class.kind, class.negated) {
        (ClassPerlKind::Digit, false) => "[0-9]",
        (ClassPerlKind::Digit, true) => "[^0-9]",
        (ClassPerlKind::Space, false) => r"[\t\n\f\r ]",
        (ClassPerlKind::Space, true) => r"[^\t\n\f\r ]",
        (ClassPerlKind::Word, false) => "[0-9A-Za-z_]",
        (ClassPerlKind::Word, true) => "[^0-9A-Za-z_]",
    };
    match ast::parse::Parser::new()
        .parse(source)
        .expect("static ASCII class")
    {
        Ast::ClassBracketed(ref class) => class.as_ref().clone(),
        _ => unreachable!("static bracketed class"),
    }
}

fn class_item(item: &mut ClassSetItem, mode: PatternMode) -> Result<(), PatternError> {
    match item {
        ClassSetItem::Unicode(_) => return Err(PatternError::Unsupported("Unicode property")),
        ClassSetItem::Bracketed(_) => return Err(PatternError::Unsupported("nested class")),
        ClassSetItem::Perl(class) => {
            *item = ClassSetItem::Bracketed(Box::new(perl_class(class)));
        }
        ClassSetItem::Union(union) => {
            for item in &mut union.items {
                class_item(item, mode)?;
            }
        }
        ClassSetItem::Literal(value) => literal(value, mode)?,
        ClassSetItem::Range(range) => {
            literal(&mut range.start, mode)?;
            literal(&mut range.end, mode)?;
        }
        ClassSetItem::Empty(_) | ClassSetItem::Ascii(_) => {}
    }
    Ok(())
}

// The parser bounds recursion before this walk. Generated nodes are not revisited.
fn normalize(ast: &mut Ast, repeat_budget: u32, mode: PatternMode) -> Result<(), PatternError> {
    match ast {
        Ast::Flags(set) => flags(&set.flags)?,
        Ast::Literal(value) => literal(value, mode)?,
        Ast::ClassUnicode(_) => return Err(PatternError::Unsupported("Unicode property")),
        Ast::ClassPerl(class) => *ast = Ast::ClassBracketed(Box::new(perl_class(class))),
        Ast::ClassBracketed(class) => match &mut class.kind {
            ClassSet::BinaryOp(_) => return Err(PatternError::Unsupported("class set operation")),
            ClassSet::Item(item) => class_item(item, mode)?,
        },
        Ast::Assertion(assertion) => match assertion.kind {
            AssertionKind::WordBoundary | AssertionKind::NotWordBoundary => {
                let source = if assertion.kind == AssertionKind::WordBoundary {
                    r"(?-u:\b)"
                } else {
                    r"(?-u:\B)"
                };
                *ast = ast::parse::Parser::new()
                    .parse(source)
                    .expect("static ASCII boundary");
            }
            AssertionKind::StartLine
            | AssertionKind::EndLine
            | AssertionKind::StartText
            | AssertionKind::EndText => {}
            _ => return Err(PatternError::Unsupported("extended word boundary")),
        },
        Ast::Repetition(repetition) => {
            let max = match repetition.op.kind {
                RepetitionKind::Range(
                    RepetitionRange::Exactly(n) | RepetitionRange::AtLeast(n),
                ) => n,
                RepetitionKind::Range(RepetitionRange::Bounded(_, n)) => n,
                _ => 0,
            };
            if max > repeat_budget {
                return Err(PatternError::Unsupported("counted repetition exceeds 1000"));
            }
            if matches!(*repetition.ast, Ast::Repetition(_)) {
                return Err(PatternError::Unsupported("stacked repetition operators"));
            }
            normalize(&mut repetition.ast, repeat_budget / max.max(1), mode)?;
        }
        Ast::Group(group) => {
            if let Some(group_flags) = group.flags() {
                flags(group_flags)?;
            }
            normalize(&mut group.ast, repeat_budget, mode)?;
        }
        Ast::Alternation(branches) => {
            for branch in &mut branches.asts {
                normalize(branch, repeat_budget, mode)?;
            }
        }
        Ast::Concat(parts) => {
            for part in &mut parts.asts {
                normalize(part, repeat_budget, mode)?;
            }
        }
        Ast::Empty(_) | Ast::Dot(_) => {}
    }
    Ok(())
}

#[cfg(test)]
#[path = "multiline_pattern/tests.rs"]
mod tests;
