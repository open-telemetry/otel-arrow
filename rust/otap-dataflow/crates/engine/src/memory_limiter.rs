// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Process-wide memory limiter state and sampling.

use otap_df_config::policy::{
    MemoryLimiterMode, MemoryLimiterPolicy, MemoryLimiterSource, SoftAction,
};
use std::cell::Cell;
#[cfg(all(not(windows), feature = "jemalloc"))]
use std::ffi::c_char;
use std::fs;
use std::path::{Path, PathBuf};
#[cfg(all(not(windows), feature = "jemalloc"))]
use std::ptr;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, AtomicU64, Ordering};
use std::time::{Duration, Instant};

#[cfg(test)]
use std::collections::VecDeque;

#[cfg(all(not(windows), feature = "jemalloc"))]
use tikv_jemalloc_ctl::{epoch, stats};

/// Values at or above this threshold are treated as "no limit set" by the
/// cgroup memory controller (e.g. `memory.max = max` parses to `u64::MAX`).
const CGROUP_UNLIMITED_THRESHOLD_BYTES: u64 = 1 << 60;

/// When `source = auto` and no explicit limits are configured, soft and hard
/// limits are derived as percentages of the detected cgroup memory cap:
///   soft = 90 %,  hard = 95 %.
const AUTO_DERIVED_SOFT_NUMERATOR: u64 = 90;
const AUTO_DERIVED_HARD_NUMERATOR: u64 = 95;
const AUTO_DERIVED_DENOMINATOR: u64 = 100;

/// Process-wide memory pressure level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MemoryPressureLevel {
    /// Below the configured soft limit.
    Normal = 0,
    /// Above the soft limit; pressure is elevated. Ingress continues unless
    /// `soft_action: shed` is configured (default `observe` keeps `Soft` advisory).
    Soft = 1,
    /// Above the hard limit; ingress should be shed and readiness can fail.
    Hard = 2,
}

impl MemoryPressureLevel {
    const fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::Soft,
            2 => Self::Hard,
            _ => Self::Normal,
        }
    }
}

/// Transition payload emitted when the process-wide limiter changes pressure level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryPressureChanged {
    /// Monotonic update number assigned by the global sampler.
    pub generation: u64,
    /// Newly classified pressure level.
    pub level: MemoryPressureLevel,
    /// Receiver-facing retry hint to use while shedding ingress.
    pub retry_after_secs: u32,
    /// Most recent sampled process memory usage in bytes.
    pub usage_bytes: u64,
}

impl MemoryPressureChanged {
    /// Initial watch-channel value before the first real transition.
    #[must_use]
    pub const fn initial() -> Self {
        Self {
            generation: 0,
            level: MemoryPressureLevel::Normal,
            retry_after_secs: 1,
            usage_bytes: 0,
        }
    }
}

const fn mode_to_u8(mode: MemoryLimiterMode) -> u8 {
    match mode {
        MemoryLimiterMode::Enforce => 0,
        MemoryLimiterMode::ObserveOnly => 1,
    }
}

const fn mode_from_u8(val: u8) -> MemoryLimiterMode {
    match val {
        1 => MemoryLimiterMode::ObserveOnly,
        _ => MemoryLimiterMode::Enforce,
    }
}

const fn soft_action_to_u8(action: SoftAction) -> u8 {
    match action {
        SoftAction::Observe => 0,
        SoftAction::Shed => 1,
    }
}

const fn soft_action_from_u8(val: u8) -> SoftAction {
    match val {
        1 => SoftAction::Shed,
        _ => SoftAction::Observe,
    }
}

/// Single source of truth for the ingress-shed predicate, shared by the
/// process-wide state and both receiver-local states to prevent drift.
///
/// `ObserveOnly` mode never sheds (the safe dry-run). In `Enforce` mode, `Hard`
/// always sheds; `Soft` sheds only when `soft_action == Shed` (default
/// `Observe` keeps `Soft` advisory — byte-identical to the pre-`soft_action`
/// behavior). `Normal` never sheds.
#[inline]
fn shed_decision(
    mode: MemoryLimiterMode,
    level: MemoryPressureLevel,
    soft_action: SoftAction,
) -> bool {
    if !matches!(mode, MemoryLimiterMode::Enforce) {
        return false;
    }
    match level {
        MemoryPressureLevel::Hard => true,
        MemoryPressureLevel::Soft => matches!(soft_action, SoftAction::Shed),
        MemoryPressureLevel::Normal => false,
    }
}

/// Shared process-wide memory pressure state.
#[derive(Clone, Debug)]
pub struct MemoryPressureState {
    inner: Arc<MemoryPressureStateInner>,
}

#[derive(Debug)]
struct MemoryPressureStateInner {
    level: AtomicU8,
    usage_bytes: AtomicU64,
    soft_limit_bytes: AtomicU64,
    hard_limit_bytes: AtomicU64,
    retry_after_secs: AtomicU32,
    mode: AtomicU8,
    soft_action: AtomicU8,
    fail_readiness_on_hard: AtomicBool,
}

impl Default for MemoryPressureState {
    fn default() -> Self {
        Self {
            inner: Arc::new(MemoryPressureStateInner {
                level: AtomicU8::new(MemoryPressureLevel::Normal as u8),
                usage_bytes: AtomicU64::new(0),
                soft_limit_bytes: AtomicU64::new(0),
                hard_limit_bytes: AtomicU64::new(0),
                retry_after_secs: AtomicU32::new(1),
                mode: AtomicU8::new(mode_to_u8(MemoryLimiterMode::Enforce)),
                soft_action: AtomicU8::new(soft_action_to_u8(SoftAction::Observe)),
                fail_readiness_on_hard: AtomicBool::new(true),
            }),
        }
    }
}

/// Runtime behavior applied by the shared memory pressure state.
///
/// This is configured once at engine startup. Live mode switching is not
/// supported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryPressureBehaviorConfig {
    /// Retry-After seconds advertised by receivers while shedding ingress.
    pub retry_after_secs: u32,
    /// Whether readiness should fail while in `Hard` pressure in `enforce` mode.
    pub fail_readiness_on_hard: bool,
    /// Whether `Hard` pressure is enforced or only observed.
    pub mode: MemoryLimiterMode,
    /// Graduated response applied at `Soft` pressure.
    pub soft_action: SoftAction,
}

impl MemoryPressureState {
    /// Configures shared limiter metadata.
    ///
    /// This is expected to be called once during engine startup. Reconfiguring
    /// the limiter mode while receivers are active is not supported.
    pub fn configure(&self, config: MemoryPressureBehaviorConfig) {
        self.inner
            .retry_after_secs
            .store(config.retry_after_secs.max(1), Ordering::Relaxed);
        self.inner
            .mode
            .store(mode_to_u8(config.mode), Ordering::Relaxed);
        self.inner
            .soft_action
            .store(soft_action_to_u8(config.soft_action), Ordering::Relaxed);
        self.inner
            .fail_readiness_on_hard
            .store(config.fail_readiness_on_hard, Ordering::Relaxed);
    }

    /// Returns the current pressure level.
    #[must_use]
    pub fn level(&self) -> MemoryPressureLevel {
        MemoryPressureLevel::from_u8(self.inner.level.load(Ordering::Relaxed))
    }

    /// Returns whether ingress should be shed under the current pressure level.
    ///
    /// `Hard` pressure always sheds when the limiter is enforcing. `Soft`
    /// pressure sheds only when `soft_action == Shed`; with the default
    /// `Observe`, `Soft` stays advisory so operators can watch rising pressure
    /// without rejecting traffic. `Normal` never sheds.
    #[must_use]
    pub fn should_shed_ingress(&self) -> bool {
        shed_decision(
            mode_from_u8(self.inner.mode.load(Ordering::Relaxed)),
            self.level(),
            self.soft_action(),
        )
    }

    /// Returns whether the admin readiness endpoint should fail.
    #[must_use]
    pub fn should_fail_readiness(&self) -> bool {
        mode_from_u8(self.inner.mode.load(Ordering::Relaxed)) == MemoryLimiterMode::Enforce
            && self.level() == MemoryPressureLevel::Hard
            && self.inner.fail_readiness_on_hard.load(Ordering::Relaxed)
    }

    /// Returns the configured limiter mode.
    #[must_use]
    pub fn mode(&self) -> MemoryLimiterMode {
        mode_from_u8(self.inner.mode.load(Ordering::Relaxed))
    }

    /// Returns the Retry-After value for HTTP shedding responses.
    #[must_use]
    pub fn retry_after_secs(&self) -> u32 {
        self.inner.retry_after_secs.load(Ordering::Relaxed)
    }

    /// Returns the most recently sampled memory usage in bytes.
    #[must_use]
    pub fn usage_bytes(&self) -> u64 {
        self.inner.usage_bytes.load(Ordering::Relaxed)
    }

    /// Returns the configured graduated `Soft` response.
    #[must_use]
    pub fn soft_action(&self) -> SoftAction {
        soft_action_from_u8(self.inner.soft_action.load(Ordering::Relaxed))
    }

    /// Returns the configured soft limit in bytes.
    #[must_use]
    pub fn soft_limit_bytes(&self) -> u64 {
        self.inner.soft_limit_bytes.load(Ordering::Relaxed)
    }

    /// Returns the configured hard limit in bytes.
    #[must_use]
    pub fn hard_limit_bytes(&self) -> u64 {
        self.inner.hard_limit_bytes.load(Ordering::Relaxed)
    }

    /// Returns the current process-wide state as a receiver-facing transition payload.
    #[must_use]
    pub fn current_update(&self, generation: u64) -> MemoryPressureChanged {
        MemoryPressureChanged {
            generation,
            level: self.level(),
            retry_after_secs: self.retry_after_secs(),
            usage_bytes: self.usage_bytes(),
        }
    }

    fn update_limits(&self, soft_limit_bytes: u64, hard_limit_bytes: u64) {
        self.inner
            .soft_limit_bytes
            .store(soft_limit_bytes, Ordering::Relaxed);
        self.inner
            .hard_limit_bytes
            .store(hard_limit_bytes, Ordering::Relaxed);
    }

    fn update_level(&self, level: MemoryPressureLevel, usage_bytes: u64) -> MemoryPressureLevel {
        let previous = self.level();
        self.inner.level.store(level as u8, Ordering::Relaxed);
        self.inner.usage_bytes.store(usage_bytes, Ordering::Relaxed);
        previous
    }

    /// Sets the current pressure level for tests without sampling memory.
    #[cfg(any(test, feature = "test-utils"))]
    pub fn set_level_for_tests(&self, level: MemoryPressureLevel) {
        self.inner.level.store(level as u8, Ordering::Relaxed);
    }

    /// Sets the sampled usage and effective limits for tests.
    #[cfg(any(test, feature = "test-utils"))]
    pub fn set_sample_for_tests(
        &self,
        level: MemoryPressureLevel,
        usage_bytes: u64,
        soft_limit_bytes: u64,
        hard_limit_bytes: u64,
    ) {
        self.update_limits(soft_limit_bytes, hard_limit_bytes);
        _ = self.update_level(level, usage_bytes);
    }
}

#[derive(Debug)]
struct SharedReceiverAdmissionStateInner {
    generation: AtomicU64,
    level: AtomicU8,
    retry_after_secs: AtomicU32,
    usage_bytes: AtomicU64,
    mode: MemoryLimiterMode,
    soft_action: SoftAction,
}

/// Receiver-local admission state shared across task/service clones inside a receiver.
#[derive(Clone, Debug)]
pub struct SharedReceiverAdmissionState {
    inner: Arc<SharedReceiverAdmissionStateInner>,
}

impl Default for SharedReceiverAdmissionState {
    fn default() -> Self {
        Self::from_process_state(&MemoryPressureState::default())
    }
}

impl SharedReceiverAdmissionState {
    /// Bootstraps receiver-local admission state from the current process-wide snapshot.
    #[must_use]
    pub fn from_process_state(state: &MemoryPressureState) -> Self {
        Self {
            inner: Arc::new(SharedReceiverAdmissionStateInner {
                generation: AtomicU64::new(0),
                level: AtomicU8::new(state.level() as u8),
                retry_after_secs: AtomicU32::new(state.retry_after_secs()),
                usage_bytes: AtomicU64::new(state.usage_bytes()),
                mode: state.mode(),
                soft_action: state.soft_action(),
            }),
        }
    }

    /// Applies a transition update, ignoring stale generations.
    pub fn apply(&self, update: MemoryPressureChanged) {
        let current = self.inner.generation.load(Ordering::Relaxed);
        if update.generation <= current {
            return;
        }

        self.inner
            .level
            .store(update.level as u8, Ordering::Relaxed);
        self.inner
            .retry_after_secs
            .store(update.retry_after_secs.max(1), Ordering::Relaxed);
        self.inner
            .usage_bytes
            .store(update.usage_bytes, Ordering::Relaxed);
        self.inner
            .generation
            .store(update.generation, Ordering::Relaxed);
    }

    /// Returns whether ingress should be shed for this receiver.
    #[must_use]
    pub fn should_shed_ingress(&self) -> bool {
        shed_decision(
            self.inner.mode,
            MemoryPressureLevel::from_u8(self.inner.level.load(Ordering::Relaxed)),
            self.inner.soft_action,
        )
    }

    /// Returns the Retry-After value advertised while shedding ingress.
    #[must_use]
    pub fn retry_after_secs(&self) -> u32 {
        self.inner.retry_after_secs.load(Ordering::Relaxed)
    }

    /// Returns the current local pressure level.
    #[must_use]
    pub fn level(&self) -> MemoryPressureLevel {
        MemoryPressureLevel::from_u8(self.inner.level.load(Ordering::Relaxed))
    }
}

#[derive(Debug)]
struct LocalReceiverAdmissionStateInner {
    generation: Cell<u64>,
    level: Cell<MemoryPressureLevel>,
    retry_after_secs: Cell<u32>,
    usage_bytes: Cell<u64>,
    mode: MemoryLimiterMode,
    soft_action: SoftAction,
}

/// Receiver-local admission state for LocalSet-only receivers that do not cross task boundaries.
#[derive(Clone, Debug)]
pub struct LocalReceiverAdmissionState {
    inner: Rc<LocalReceiverAdmissionStateInner>,
}

impl LocalReceiverAdmissionState {
    /// Bootstraps receiver-local admission state from the current process-wide snapshot.
    #[must_use]
    pub fn from_process_state(state: &MemoryPressureState) -> Self {
        Self {
            inner: Rc::new(LocalReceiverAdmissionStateInner {
                generation: Cell::new(0),
                level: Cell::new(state.level()),
                retry_after_secs: Cell::new(state.retry_after_secs()),
                usage_bytes: Cell::new(state.usage_bytes()),
                mode: state.mode(),
                soft_action: state.soft_action(),
            }),
        }
    }

    /// Applies a transition update, ignoring stale generations.
    pub fn apply(&self, update: MemoryPressureChanged) {
        if update.generation <= self.inner.generation.get() {
            return;
        }

        self.inner.level.set(update.level);
        self.inner
            .retry_after_secs
            .set(update.retry_after_secs.max(1));
        self.inner.usage_bytes.set(update.usage_bytes);
        self.inner.generation.set(update.generation);
    }

    /// Returns whether ingress should be shed for this receiver.
    #[must_use]
    pub fn should_shed_ingress(&self) -> bool {
        shed_decision(
            self.inner.mode,
            self.inner.level.get(),
            self.inner.soft_action,
        )
    }

    /// Returns the Retry-After value advertised while shedding ingress.
    #[must_use]
    pub fn retry_after_secs(&self) -> u32 {
        self.inner.retry_after_secs.get()
    }

    /// Returns the current local pressure level.
    #[must_use]
    pub fn level(&self) -> MemoryPressureLevel {
        self.inner.level.get()
    }
}

/// Runtime source used for memory sampling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemorySampleSource {
    /// Linux cgroup accounting working set.
    Cgroup,
    /// Process RSS.
    Rss,
    /// jemalloc resident bytes.
    JemallocResident,
}

impl MemorySampleSource {
    /// Returns a stable string form for logs.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cgroup => "cgroup",
            Self::Rss => "rss",
            Self::JemallocResident => "jemalloc_resident",
        }
    }
}

/// One memory sample from the selected source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemorySample {
    /// Sampled usage in bytes.
    pub usage_bytes: u64,
    /// Source used for the sample.
    pub source: MemorySampleSource,
}

/// Effective process-wide memory limiter configuration.
#[derive(Debug)]
pub struct EffectiveMemoryLimiter {
    mode: MemoryLimiterMode,
    check_interval: Duration,
    soft_limit_bytes: u64,
    hard_limit_bytes: u64,
    hysteresis_bytes: u64,
    purge_on_hard: bool,
    purge_min_interval: Duration,
    last_purge_at: Option<Instant>,
    sampler: MemoryUsageSampler,
}

impl EffectiveMemoryLimiter {
    /// Builds a limiter from user policy.
    pub fn from_policy(policy: &MemoryLimiterPolicy) -> Result<Self, String> {
        let sampler = MemoryUsageSampler::new(policy.source)?;
        let detected_limit = sampler.detected_limit_bytes();

        let (soft_limit_bytes, hard_limit_bytes) = match (policy.soft_limit, policy.hard_limit) {
            (Some(soft), Some(hard)) => (soft, hard),
            (None, None) if policy.source == MemoryLimiterSource::Auto => {
                let limit = detected_limit.ok_or_else(|| {
                    "memory_limiter.soft_limit and hard_limit must be set when no cgroup memory.max limit is available".to_string()
                })?;
                let soft =
                    limit.saturating_mul(AUTO_DERIVED_SOFT_NUMERATOR) / AUTO_DERIVED_DENOMINATOR;
                let hard =
                    limit.saturating_mul(AUTO_DERIVED_HARD_NUMERATOR) / AUTO_DERIVED_DENOMINATOR;
                (soft, hard)
            }
            (None, None) => {
                return Err(
                    "memory_limiter.soft_limit and hard_limit must be set when memory_limiter.source is not auto"
                        .to_string(),
                );
            }
            _ => {
                return Err(
                    "memory_limiter.soft_limit and hard_limit must either both be set or both be omitted"
                        .to_string(),
                );
            }
        };

        if hard_limit_bytes <= soft_limit_bytes {
            return Err(
                "memory_limiter.hard_limit must be greater than memory_limiter.soft_limit"
                    .to_string(),
            );
        }

        // When hysteresis is omitted, derive a small recovery band below the
        // soft limit: the smaller of the soft->hard gap and 10% of the soft
        // limit. Capping at `soft_limit / 10` (rather than `soft_limit - 1`)
        // keeps the band narrow even when the soft->hard gap is wide, so a
        // `soft_action: shed` limiter recovers to `Normal` once usage falls
        // modestly below `soft_limit` instead of only after usage collapses
        // toward zero.
        let hysteresis_bytes = policy.hysteresis.unwrap_or_else(|| {
            hard_limit_bytes
                .saturating_sub(soft_limit_bytes)
                .min(soft_limit_bytes / 10)
        });

        if hysteresis_bytes >= soft_limit_bytes {
            return Err(
                "memory_limiter.hysteresis must be smaller than memory_limiter.soft_limit"
                    .to_string(),
            );
        }

        Ok(Self {
            mode: policy.mode,
            check_interval: policy.check_interval,
            soft_limit_bytes,
            hard_limit_bytes,
            hysteresis_bytes,
            purge_on_hard: policy.purge_on_hard,
            purge_min_interval: policy.purge_min_interval,
            last_purge_at: None,
            sampler,
        })
    }

    /// Returns the sampling interval.
    #[must_use]
    pub const fn check_interval(&self) -> Duration {
        self.check_interval
    }

    fn classify(&self, current: MemoryPressureLevel, usage_bytes: u64) -> MemoryPressureLevel {
        match current {
            MemoryPressureLevel::Normal => {
                if usage_bytes >= self.hard_limit_bytes {
                    MemoryPressureLevel::Hard
                } else if usage_bytes >= self.soft_limit_bytes {
                    MemoryPressureLevel::Soft
                } else {
                    MemoryPressureLevel::Normal
                }
            }
            MemoryPressureLevel::Soft => {
                if usage_bytes >= self.hard_limit_bytes {
                    MemoryPressureLevel::Hard
                } else if usage_bytes < self.soft_limit_bytes.saturating_sub(self.hysteresis_bytes)
                {
                    MemoryPressureLevel::Normal
                } else {
                    MemoryPressureLevel::Soft
                }
            }
            MemoryPressureLevel::Hard => {
                if usage_bytes < self.soft_limit_bytes {
                    MemoryPressureLevel::Soft
                } else {
                    MemoryPressureLevel::Hard
                }
            }
        }
    }

    fn should_attempt_purge(&self, level: MemoryPressureLevel, now: Instant) -> bool {
        self.mode == MemoryLimiterMode::Enforce
            && self.purge_on_hard
            && level == MemoryPressureLevel::Hard
            && self.sampler.supports_purge()
            && self.last_purge_at.is_none_or(|last_purge_at| {
                now.duration_since(last_purge_at) >= self.purge_min_interval
            })
    }

    /// Returns whether purge support is available for this limiter build.
    #[must_use]
    pub fn purge_supported(&self) -> bool {
        self.sampler.supports_purge()
    }

    /// Returns whether forced purge is enabled in policy.
    #[must_use]
    pub const fn purge_on_hard(&self) -> bool {
        self.purge_on_hard
    }

    /// Samples memory and updates the shared state.
    pub fn tick(&mut self, state: &MemoryPressureState) -> Result<MemoryLimiterTick, String> {
        let current = state.level();
        let mut sample = self.sampler.sample()?;
        state.update_limits(self.soft_limit_bytes, self.hard_limit_bytes);
        let mut level = self.classify(current, sample.usage_bytes);
        let mut pre_purge_usage_bytes = None;
        let mut purge_duration = None;
        let mut purge_error = None;

        let started_at = Instant::now();
        if self.should_attempt_purge(level, started_at) {
            pre_purge_usage_bytes = Some(sample.usage_bytes);
            self.last_purge_at = Some(started_at);
            match self.sampler.purge() {
                Ok(()) => {
                    let elapsed = started_at.elapsed();
                    purge_duration = Some(elapsed);
                    match self.sampler.sample() {
                        Ok(post_purge_sample) => {
                            sample = post_purge_sample;
                            level = self.classify(current, sample.usage_bytes);
                        }
                        Err(err) => {
                            purge_error = Some(format!("post-purge sample failed: {err}"));
                        }
                    }
                }
                Err(err) => {
                    purge_error = Some(err);
                }
            }
        }

        let previous = state.update_level(level, sample.usage_bytes);
        Ok(MemoryLimiterTick {
            previous_level: previous,
            current_level: level,
            sample,
            soft_limit_bytes: self.soft_limit_bytes,
            hard_limit_bytes: self.hard_limit_bytes,
            pre_purge_usage_bytes,
            purge_duration,
            purge_error,
        })
    }
}

/// Result of one limiter iteration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryLimiterTick {
    /// Level before the sample was applied.
    pub previous_level: MemoryPressureLevel,
    /// Level after the sample was applied.
    pub current_level: MemoryPressureLevel,
    /// Memory sample used to drive the transition.
    pub sample: MemorySample,
    /// Configured soft limit.
    pub soft_limit_bytes: u64,
    /// Configured hard limit.
    pub hard_limit_bytes: u64,
    /// Usage before a forced purge, when one was attempted during this tick.
    pub pre_purge_usage_bytes: Option<u64>,
    /// Duration of a forced purge, when one was attempted during this tick.
    pub purge_duration: Option<Duration>,
    /// Error from a forced purge attempt, when one failed during this tick.
    pub purge_error: Option<String>,
}

impl MemoryLimiterTick {
    /// Returns whether the level changed.
    #[must_use]
    pub fn transitioned(&self) -> bool {
        self.previous_level != self.current_level
    }
}

trait MemoryUsageProbe: Send {
    fn sample_usage(&mut self) -> Result<MemorySample, String>;
}

trait MemoryLimitProbe: Send + Sync {
    fn detect_limit(&self) -> Option<u64>;
}

trait MemoryPurgeHook: Send {
    fn purge(&mut self) -> Result<(), String>;
}

struct MemoryUsageSampler {
    usage_probe: Box<dyn MemoryUsageProbe>,
    limit_probe: Option<Box<dyn MemoryLimitProbe>>,
    purge_hook: Option<Box<dyn MemoryPurgeHook>>,
}

impl std::fmt::Debug for MemoryUsageSampler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryUsageSampler")
            .field("has_limit_probe", &self.limit_probe.is_some())
            .field("has_purge_hook", &self.purge_hook.is_some())
            .finish_non_exhaustive()
    }
}

impl MemoryUsageSampler {
    fn new(source: MemoryLimiterSource) -> Result<Self, String> {
        let cgroup = CgroupMemorySampler::discover();
        let rss_available = rss_bytes().is_some();
        #[cfg(all(not(windows), feature = "jemalloc"))]
        let jemalloc_resident = JemallocResidentProbe::new();

        let (usage_probe, limit_probe): (
            Box<dyn MemoryUsageProbe>,
            Option<Box<dyn MemoryLimitProbe>>,
        ) = match source {
            MemoryLimiterSource::Auto => {
                if let Some(cgroup_probe) = cgroup {
                    (Box::new(cgroup_probe.clone()), Some(Box::new(cgroup_probe)))
                } else if rss_available {
                    (Box::new(RssProbe), None)
                } else {
                    #[cfg(all(not(windows), feature = "jemalloc"))]
                    {
                        if let Some(jemalloc_probe) = jemalloc_resident {
                            (Box::new(jemalloc_probe), None)
                        } else {
                            return Err("no supported memory source is available".to_string());
                        }
                    }
                    #[cfg(any(windows, not(feature = "jemalloc")))]
                    {
                        return Err("no supported memory source is available".to_string());
                    }
                }
            }
            MemoryLimiterSource::Cgroup => {
                let cgroup_probe = cgroup.ok_or_else(|| {
                    "memory limiter source `cgroup` requested, but no cgroup memory controller was detected".to_string()
                })?;
                (Box::new(cgroup_probe.clone()), Some(Box::new(cgroup_probe)))
            }
            MemoryLimiterSource::Rss => {
                if !rss_available {
                    return Err(
                        "memory limiter source `rss` requested, but process RSS sampling is unavailable"
                            .to_string(),
                    );
                }
                (Box::new(RssProbe), None)
            }
            MemoryLimiterSource::JemallocResident => {
                #[cfg(all(not(windows), feature = "jemalloc"))]
                {
                    let jemalloc_probe = jemalloc_resident.ok_or_else(|| {
                        "memory limiter source `jemalloc_resident` requested, but jemalloc resident metrics are unavailable".to_string()
                    })?;
                    (Box::new(jemalloc_probe), None)
                }
                #[cfg(any(windows, not(feature = "jemalloc")))]
                {
                    return Err("memory limiter source `jemalloc_resident` requested, but this build does not expose jemalloc resident metrics".to_string());
                }
            }
        };

        Ok(Self {
            usage_probe,
            limit_probe,
            purge_hook: Self::build_purge_hook(),
        })
    }

    fn detected_limit_bytes(&self) -> Option<u64> {
        self.limit_probe
            .as_ref()
            .and_then(|probe| probe.detect_limit())
    }

    fn sample(&mut self) -> Result<MemorySample, String> {
        self.usage_probe.sample_usage()
    }

    fn supports_purge(&self) -> bool {
        self.purge_hook.is_some()
    }

    fn purge(&mut self) -> Result<(), String> {
        self.purge_hook
            .as_mut()
            .ok_or_else(|| "memory purge is unavailable for this build".to_string())?
            .purge()
    }

    /// Returns the best available allocator purge hook for this build.
    ///
    /// Backends are evaluated in priority order. To add a new backend,
    /// insert a `#[cfg]`-gated `return` above the final `None`.
    #[allow(unreachable_code)]
    fn build_purge_hook() -> Option<Box<dyn MemoryPurgeHook>> {
        // Priority 1: jemalloc (non-Windows builds with the jemalloc feature).
        #[cfg(all(not(windows), feature = "jemalloc"))]
        return Some(Box::new(JemallocPurgeHook));

        None
    }

    #[cfg(test)]
    fn for_tests(source: MemorySampleSource) -> Self {
        let usage_probe: Box<dyn MemoryUsageProbe> = match source {
            MemorySampleSource::Rss => Box::new(RssProbe),
            MemorySampleSource::Cgroup => {
                panic!("cgroup test probe must be constructed explicitly")
            }
            MemorySampleSource::JemallocResident => {
                #[cfg(all(not(windows), feature = "jemalloc"))]
                {
                    Box::new(JemallocResidentProbe)
                }
                #[cfg(any(windows, not(feature = "jemalloc")))]
                {
                    panic!("jemalloc resident test probe unavailable on this platform")
                }
            }
        };
        Self {
            usage_probe,
            limit_probe: None,
            purge_hook: None,
        }
    }

    #[cfg(test)]
    fn from_test_probes(
        usage_probe: Box<dyn MemoryUsageProbe>,
        purge_hook: Option<Box<dyn MemoryPurgeHook>>,
    ) -> Self {
        Self {
            usage_probe,
            limit_probe: None,
            purge_hook,
        }
    }
}

#[derive(Debug, Clone)]
struct CgroupMemorySampler {
    current_path: PathBuf,
    stat_path: PathBuf,
    limit_path: PathBuf,
    stat_key: &'static str,
}

impl CgroupMemorySampler {
    fn discover() -> Option<Self> {
        let cgroup_file = fs::read_to_string("/proc/self/cgroup").ok()?;

        // cgroup v2: `0::/path`
        if let Some(path) = cgroup_file
            .lines()
            .find_map(|line| line.strip_prefix("0::"))
            .map(str::trim)
        {
            let base = cgroup_path(Path::new("/sys/fs/cgroup"), path);
            let current_path = base.join("memory.current");
            let stat_path = base.join("memory.stat");
            let limit_path = base.join("memory.max");
            if current_path.exists() && stat_path.exists() && limit_path.exists() {
                return Some(Self {
                    current_path,
                    stat_path,
                    limit_path,
                    stat_key: "inactive_file",
                });
            }
        }

        // cgroup v1: `hierarchy:controllers:/path`
        for line in cgroup_file.lines() {
            let mut parts = line.splitn(3, ':');
            let _ = parts.next();
            let controllers = parts.next().unwrap_or_default();
            let path = parts.next().unwrap_or_default().trim();
            if !controllers
                .split(',')
                .any(|controller| controller == "memory")
            {
                continue;
            }
            let base = cgroup_path(Path::new("/sys/fs/cgroup/memory"), path);
            let current_path = base.join("memory.usage_in_bytes");
            let stat_path = base.join("memory.stat");
            let limit_path = base.join("memory.limit_in_bytes");
            if current_path.exists() && stat_path.exists() && limit_path.exists() {
                return Some(Self {
                    current_path,
                    stat_path,
                    limit_path,
                    stat_key: "total_inactive_file",
                });
            }
        }

        None
    }

    fn limit_bytes(&self) -> Option<u64> {
        let raw = read_u64_from_file(&self.limit_path).ok()?;
        (!is_unlimited_limit(raw)).then_some(raw)
    }

    fn sample(&self) -> Result<MemorySample, String> {
        let usage = read_u64_from_file(&self.current_path)
            .map_err(|err| format!("failed to read {}: {err}", self.current_path.display()))?;
        let inactive_file = read_memory_stat_value(&self.stat_path, self.stat_key).unwrap_or(0);

        Ok(MemorySample {
            usage_bytes: usage.saturating_sub(inactive_file),
            source: MemorySampleSource::Cgroup,
        })
    }
}

impl MemoryUsageProbe for CgroupMemorySampler {
    fn sample_usage(&mut self) -> Result<MemorySample, String> {
        self.sample()
    }
}

impl MemoryLimitProbe for CgroupMemorySampler {
    fn detect_limit(&self) -> Option<u64> {
        self.limit_bytes()
    }
}

fn cgroup_path(base: &Path, relative: &str) -> PathBuf {
    if relative == "/" || relative.is_empty() {
        return base.to_path_buf();
    }
    base.join(relative.trim_start_matches('/'))
}

fn read_u64_from_file(path: &Path) -> Result<u64, String> {
    let raw = fs::read_to_string(path).map_err(|err| err.to_string())?;
    let trimmed = raw.trim();
    if trimmed.eq_ignore_ascii_case("max") {
        return Ok(u64::MAX);
    }
    trimmed.parse::<u64>().map_err(|err| err.to_string())
}

fn read_memory_stat_value(path: &Path, key: &str) -> Option<u64> {
    let stats = fs::read_to_string(path).ok()?;
    for line in stats.lines() {
        let mut parts = line.split_whitespace();
        let stat_key = parts.next()?;
        let stat_value = parts.next()?;
        if stat_key == key {
            return stat_value.parse::<u64>().ok();
        }
    }
    None
}

fn is_unlimited_limit(limit_bytes: u64) -> bool {
    limit_bytes == u64::MAX || limit_bytes >= CGROUP_UNLIMITED_THRESHOLD_BYTES
}

fn rss_bytes() -> Option<u64> {
    memory_stats::memory_stats().map(|stats| stats.physical_mem as u64)
}

#[derive(Debug, Clone, Copy)]
struct RssProbe;

impl MemoryUsageProbe for RssProbe {
    fn sample_usage(&mut self) -> Result<MemorySample, String> {
        rss_bytes()
            .map(|usage_bytes| MemorySample {
                usage_bytes,
                source: MemorySampleSource::Rss,
            })
            .ok_or_else(|| "failed to sample process RSS".to_string())
    }
}

#[cfg(all(not(windows), feature = "jemalloc"))]
#[derive(Debug, Clone, Copy)]
struct JemallocResidentProbe;

#[cfg(all(not(windows), feature = "jemalloc"))]
impl JemallocResidentProbe {
    fn new() -> Option<Self> {
        Some(Self)
    }
}

#[cfg(all(not(windows), feature = "jemalloc"))]
impl MemoryUsageProbe for JemallocResidentProbe {
    fn sample_usage(&mut self) -> Result<MemorySample, String> {
        _ = epoch::advance().map_err(|err| format!("failed to advance jemalloc epoch: {err}"))?;
        let usage_bytes = stats::resident::read()
            .map_err(|err| format!("failed to read jemalloc resident bytes: {err}"))?;
        Ok(MemorySample {
            usage_bytes: usage_bytes as u64,
            source: MemorySampleSource::JemallocResident,
        })
    }
}

#[cfg(all(not(windows), feature = "jemalloc"))]
#[derive(Debug, Clone, Copy)]
struct JemallocPurgeHook;

#[cfg(all(not(windows), feature = "jemalloc"))]
impl MemoryPurgeHook for JemallocPurgeHook {
    #[allow(unsafe_code)]
    fn purge(&mut self) -> Result<(), String> {
        const PURGE_MALLCTL: &[u8] = b"arena.4096.purge\0";

        // Safety: `PURGE_MALLCTL` is a static NUL-terminated mallctl name, and
        // `arena.<all>.purge` is a void control, so both read and write
        // pointers must be null with a zero write length.
        let rc = unsafe {
            tikv_jemalloc_sys::mallctl(
                PURGE_MALLCTL.as_ptr().cast::<c_char>(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                0,
            )
        };

        if rc == 0 {
            Ok(())
        } else {
            Err(format!(
                "failed to purge jemalloc arenas: {}",
                std::io::Error::from_raw_os_error(rc)
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    struct SequenceProbe {
        samples: VecDeque<MemorySample>,
    }

    impl MemoryUsageProbe for SequenceProbe {
        fn sample_usage(&mut self) -> Result<MemorySample, String> {
            self.samples
                .pop_front()
                .ok_or_else(|| "no test sample available".to_string())
        }
    }

    struct CountingPurgeHook {
        calls: Arc<AtomicUsize>,
    }

    impl MemoryPurgeHook for CountingPurgeHook {
        fn purge(&mut self) -> Result<(), String> {
            _ = self.calls.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }
    }

    struct FailingPurgeHook;

    impl MemoryPurgeHook for FailingPurgeHook {
        fn purge(&mut self) -> Result<(), String> {
            Err("purge failed".to_string())
        }
    }

    #[cfg(all(not(windows), feature = "jemalloc-testing"))]
    #[test]
    fn jemalloc_purge_hook_succeeds() {
        let mut hook = JemallocPurgeHook;
        hook.purge().expect("jemalloc purge should succeed");
    }

    #[test]
    fn limiter_escalates_and_recovers_with_hysteresis() {
        let limiter = EffectiveMemoryLimiter {
            mode: MemoryLimiterMode::Enforce,
            check_interval: Duration::from_secs(1),
            soft_limit_bytes: 90,
            hard_limit_bytes: 95,
            hysteresis_bytes: 5,
            purge_on_hard: false,
            purge_min_interval: Duration::from_secs(5),
            last_purge_at: None,
            sampler: MemoryUsageSampler::for_tests(MemorySampleSource::Rss),
        };

        assert_eq!(
            limiter.classify(MemoryPressureLevel::Normal, 89),
            MemoryPressureLevel::Normal
        );
        assert_eq!(
            limiter.classify(MemoryPressureLevel::Normal, 90),
            MemoryPressureLevel::Soft
        );
        assert_eq!(
            limiter.classify(MemoryPressureLevel::Normal, 95),
            MemoryPressureLevel::Hard
        );
        assert_eq!(
            limiter.classify(MemoryPressureLevel::Soft, 90),
            MemoryPressureLevel::Soft
        );
        assert_eq!(
            limiter.classify(MemoryPressureLevel::Soft, 84),
            MemoryPressureLevel::Normal
        );
        assert_eq!(
            limiter.classify(MemoryPressureLevel::Hard, 89),
            MemoryPressureLevel::Soft
        );
    }

    #[test]
    fn readiness_only_fails_on_hard_pressure_when_enabled() {
        let state = MemoryPressureState::default();
        state.configure(MemoryPressureBehaviorConfig {
            retry_after_secs: 3,
            fail_readiness_on_hard: true,
            mode: MemoryLimiterMode::Enforce,
            soft_action: SoftAction::Observe,
        });
        _ = state.update_level(MemoryPressureLevel::Soft, 91);
        assert!(!state.should_fail_readiness());

        _ = state.update_level(MemoryPressureLevel::Hard, 96);
        assert!(state.should_fail_readiness());

        state.configure(MemoryPressureBehaviorConfig {
            retry_after_secs: 3,
            fail_readiness_on_hard: false,
            mode: MemoryLimiterMode::Enforce,
            soft_action: SoftAction::Observe,
        });
        assert!(!state.should_fail_readiness());
    }

    #[test]
    fn observe_only_never_sheds_or_fails_readiness() {
        let state = MemoryPressureState::default();
        state.configure(MemoryPressureBehaviorConfig {
            retry_after_secs: 3,
            fail_readiness_on_hard: true,
            mode: MemoryLimiterMode::ObserveOnly,
            soft_action: SoftAction::Observe,
        });
        _ = state.update_level(MemoryPressureLevel::Hard, 96);

        assert_eq!(state.mode(), MemoryLimiterMode::ObserveOnly);
        assert!(!state.should_shed_ingress());
        assert!(!state.should_fail_readiness());
    }

    #[test]
    fn omitted_hysteresis_defaults_to_a_value_below_soft_limit() {
        let limiter = EffectiveMemoryLimiter::from_policy(&MemoryLimiterPolicy {
            mode: MemoryLimiterMode::Enforce,
            soft_action: SoftAction::Observe,
            source: MemoryLimiterSource::Rss,
            check_interval: Duration::from_secs(1),
            soft_limit: Some(100),
            hard_limit: Some(250),
            hysteresis: None,
            retry_after_secs: 1,
            fail_readiness_on_hard: true,
            purge_on_hard: false,
            purge_min_interval: Duration::from_secs(5),
        })
        .expect("limiter should accept omitted hysteresis");

        // Derived band = min(hard - soft = 150, soft / 10 = 10) = 10 bytes, a
        // narrow recovery margin below the soft limit (not soft_limit - 1).
        assert_eq!(limiter.hysteresis_bytes, 10);
    }

    #[test]
    fn soft_shed_recovers_to_normal_with_default_hysteresis() {
        // Regression: with `soft_action: shed` and an omitted hysteresis, a wide
        // soft->hard gap must still yield a recovery band small enough to return
        // to `Normal` once usage falls modestly below `soft_limit` -- not stay
        // stuck shedding until usage collapses toward zero. Samples are driven
        // through `tick()` so the policy-derived hysteresis governs the
        // classification, exercising the defaulting and state machine together.
        let mut limiter = EffectiveMemoryLimiter::from_policy(&MemoryLimiterPolicy {
            mode: MemoryLimiterMode::Enforce,
            soft_action: SoftAction::Shed,
            source: MemoryLimiterSource::Rss,
            check_interval: Duration::from_secs(1),
            soft_limit: Some(100),
            hard_limit: Some(250),
            hysteresis: None,
            retry_after_secs: 1,
            fail_readiness_on_hard: true,
            purge_on_hard: false,
            purge_min_interval: Duration::from_secs(5),
        })
        .expect("limiter should accept omitted hysteresis");

        // min(hard - soft = 150, soft / 10 = 10) = 10.
        assert_eq!(limiter.hysteresis_bytes, 10);

        // Feed usage 100 (== soft_limit) then 50 (well below soft_limit).
        limiter.sampler = MemoryUsageSampler::from_test_probes(
            Box::new(SequenceProbe {
                samples: VecDeque::from([
                    MemorySample {
                        usage_bytes: 100,
                        source: MemorySampleSource::Rss,
                    },
                    MemorySample {
                        usage_bytes: 50,
                        source: MemorySampleSource::Rss,
                    },
                ]),
            }),
            None,
        );

        let state = MemoryPressureState::default();
        state.configure(MemoryPressureBehaviorConfig {
            retry_after_secs: 1,
            fail_readiness_on_hard: true,
            mode: MemoryLimiterMode::Enforce,
            soft_action: SoftAction::Shed,
        });

        // A sample at the soft limit enters Soft and sheds ingress.
        let entered = limiter.tick(&state).expect("tick should succeed");
        assert_eq!(entered.current_level, MemoryPressureLevel::Soft);
        assert!(
            state.should_shed_ingress(),
            "Soft pressure with soft_action: shed must shed ingress"
        );

        // Usage of 50 is below soft_limit - hysteresis (90), so pressure recovers
        // to Normal and ingress reopens.
        let recovered = limiter.tick(&state).expect("tick should succeed");
        assert_eq!(recovered.current_level, MemoryPressureLevel::Normal);
        assert!(
            !state.should_shed_ingress(),
            "recovery to Normal must stop shedding ingress"
        );
    }

    #[test]
    fn soft_shed_recovery_boundary_uses_strict_below_soft_minus_hysteresis() {
        // Pins the exact Soft->Normal recovery threshold. With soft=100 and the
        // derived default hysteresis of 10, recovery is a strict `<` at 90:
        // usage == 90 stays Soft, usage == 89 recovers to Normal.
        fn shed_limiter_with(second_sample: u64) -> (EffectiveMemoryLimiter, MemoryPressureState) {
            let mut limiter = EffectiveMemoryLimiter::from_policy(&MemoryLimiterPolicy {
                mode: MemoryLimiterMode::Enforce,
                soft_action: SoftAction::Shed,
                source: MemoryLimiterSource::Rss,
                check_interval: Duration::from_secs(1),
                soft_limit: Some(100),
                hard_limit: Some(250),
                hysteresis: None,
                retry_after_secs: 1,
                fail_readiness_on_hard: true,
                purge_on_hard: false,
                purge_min_interval: Duration::from_secs(5),
            })
            .expect("limiter should accept omitted hysteresis");
            limiter.sampler = MemoryUsageSampler::from_test_probes(
                Box::new(SequenceProbe {
                    samples: VecDeque::from([
                        MemorySample {
                            usage_bytes: 100,
                            source: MemorySampleSource::Rss,
                        },
                        MemorySample {
                            usage_bytes: second_sample,
                            source: MemorySampleSource::Rss,
                        },
                    ]),
                }),
                None,
            );
            let state = MemoryPressureState::default();
            state.configure(MemoryPressureBehaviorConfig {
                retry_after_secs: 1,
                fail_readiness_on_hard: true,
                mode: MemoryLimiterMode::Enforce,
                soft_action: SoftAction::Shed,
            });
            (limiter, state)
        }

        // usage == soft_limit - hysteresis (90) stays Soft (recovery is strict `<`).
        let (mut at_boundary, state) = shed_limiter_with(90);
        assert_eq!(
            at_boundary.tick(&state).expect("tick").current_level,
            MemoryPressureLevel::Soft
        );
        assert_eq!(
            at_boundary.tick(&state).expect("tick").current_level,
            MemoryPressureLevel::Soft
        );

        // usage == soft_limit - hysteresis - 1 (89) recovers to Normal.
        let (mut below_boundary, state2) = shed_limiter_with(89);
        assert_eq!(
            below_boundary.tick(&state2).expect("tick").current_level,
            MemoryPressureLevel::Soft
        );
        assert_eq!(
            below_boundary.tick(&state2).expect("tick").current_level,
            MemoryPressureLevel::Normal
        );
    }

    #[test]
    fn state_exposes_latest_usage_and_limits() {
        let state = MemoryPressureState::default();
        state.update_limits(90, 95);
        _ = state.update_level(MemoryPressureLevel::Hard, 96);

        assert_eq!(state.usage_bytes(), 96);
        assert_eq!(state.soft_limit_bytes(), 90);
        assert_eq!(state.hard_limit_bytes(), 95);
    }

    #[test]
    fn shared_receiver_admission_state_bootstraps_from_process_state() {
        let state = MemoryPressureState::default();
        state.configure(MemoryPressureBehaviorConfig {
            retry_after_secs: 7,
            fail_readiness_on_hard: true,
            mode: MemoryLimiterMode::Enforce,
            soft_action: SoftAction::Observe,
        });
        _ = state.update_level(MemoryPressureLevel::Hard, 96);

        let local = SharedReceiverAdmissionState::from_process_state(&state);
        assert!(local.should_shed_ingress());
        assert_eq!(local.retry_after_secs(), 7);
        assert_eq!(local.level(), MemoryPressureLevel::Hard);
    }

    #[test]
    fn shared_receiver_admission_state_ignores_stale_generations() {
        let state = MemoryPressureState::default();
        let local = SharedReceiverAdmissionState::from_process_state(&state);

        local.apply(MemoryPressureChanged {
            generation: 2,
            level: MemoryPressureLevel::Hard,
            retry_after_secs: 9,
            usage_bytes: 123,
        });
        local.apply(MemoryPressureChanged {
            generation: 1,
            level: MemoryPressureLevel::Normal,
            retry_after_secs: 1,
            usage_bytes: 0,
        });

        assert!(local.should_shed_ingress());
        assert_eq!(local.retry_after_secs(), 9);
        assert_eq!(local.level(), MemoryPressureLevel::Hard);
    }

    #[test]
    fn shared_receiver_admission_state_clones_observe_updates() {
        let state = MemoryPressureState::default();
        let local = SharedReceiverAdmissionState::from_process_state(&state);
        let clone = local.clone();

        local.apply(MemoryPressureChanged {
            generation: 1,
            level: MemoryPressureLevel::Hard,
            retry_after_secs: 5,
            usage_bytes: 88,
        });

        assert!(clone.should_shed_ingress());
        assert_eq!(clone.retry_after_secs(), 5);
        assert_eq!(clone.level(), MemoryPressureLevel::Hard);
    }

    #[test]
    fn local_receiver_admission_state_ignores_stale_generations() {
        let state = MemoryPressureState::default();
        let local = LocalReceiverAdmissionState::from_process_state(&state);

        local.apply(MemoryPressureChanged {
            generation: 3,
            level: MemoryPressureLevel::Soft,
            retry_after_secs: 4,
            usage_bytes: 22,
        });
        local.apply(MemoryPressureChanged {
            generation: 2,
            level: MemoryPressureLevel::Normal,
            retry_after_secs: 1,
            usage_bytes: 0,
        });

        assert!(!local.should_shed_ingress());
        assert_eq!(local.retry_after_secs(), 4);
        assert_eq!(local.level(), MemoryPressureLevel::Soft);
    }

    #[test]
    fn local_receiver_admission_state_clones_observe_updates() {
        let state = MemoryPressureState::default();
        let local = LocalReceiverAdmissionState::from_process_state(&state);
        let clone = local.clone();

        local.apply(MemoryPressureChanged {
            generation: 1,
            level: MemoryPressureLevel::Hard,
            retry_after_secs: 6,
            usage_bytes: 77,
        });

        assert!(clone.should_shed_ingress());
        assert_eq!(clone.retry_after_secs(), 6);
        assert_eq!(clone.level(), MemoryPressureLevel::Hard);
    }

    #[test]
    fn non_auto_sources_require_explicit_limits() {
        let err = EffectiveMemoryLimiter::from_policy(&MemoryLimiterPolicy {
            mode: MemoryLimiterMode::Enforce,
            soft_action: SoftAction::Observe,
            source: MemoryLimiterSource::Rss,
            check_interval: Duration::from_secs(1),
            soft_limit: None,
            hard_limit: None,
            hysteresis: None,
            retry_after_secs: 1,
            fail_readiness_on_hard: true,
            purge_on_hard: false,
            purge_min_interval: Duration::from_secs(5),
        })
        .expect_err("non-auto source should require explicit limits");

        assert!(err.contains("source is not auto"));
    }

    #[test]
    fn tick_can_purge_before_entering_hard() {
        let purge_calls = Arc::new(AtomicUsize::new(0));
        let sampler = MemoryUsageSampler::from_test_probes(
            Box::new(SequenceProbe {
                samples: VecDeque::from([
                    MemorySample {
                        usage_bytes: 96,
                        source: MemorySampleSource::Rss,
                    },
                    MemorySample {
                        usage_bytes: 80,
                        source: MemorySampleSource::Rss,
                    },
                ]),
            }),
            Some(Box::new(CountingPurgeHook {
                calls: purge_calls.clone(),
            })),
        );
        let mut limiter = EffectiveMemoryLimiter {
            mode: MemoryLimiterMode::Enforce,
            check_interval: Duration::from_secs(1),
            soft_limit_bytes: 90,
            hard_limit_bytes: 95,
            hysteresis_bytes: 5,
            purge_on_hard: true,
            purge_min_interval: Duration::from_secs(5),
            last_purge_at: None,
            sampler,
        };
        let state = MemoryPressureState::default();

        let tick = limiter.tick(&state).expect("tick should succeed");

        assert_eq!(purge_calls.load(Ordering::Relaxed), 1);
        assert_eq!(tick.pre_purge_usage_bytes, Some(96));
        assert!(tick.purge_duration.is_some());
        assert_eq!(tick.current_level, MemoryPressureLevel::Normal);
        assert_eq!(state.level(), MemoryPressureLevel::Normal);
        assert_eq!(state.usage_bytes(), 80);
    }

    #[test]
    fn tick_rate_limits_purge_attempts() {
        let purge_calls = Arc::new(AtomicUsize::new(0));
        let sampler = MemoryUsageSampler::from_test_probes(
            Box::new(SequenceProbe {
                samples: VecDeque::from([MemorySample {
                    usage_bytes: 96,
                    source: MemorySampleSource::Rss,
                }]),
            }),
            Some(Box::new(CountingPurgeHook {
                calls: purge_calls.clone(),
            })),
        );
        let mut limiter = EffectiveMemoryLimiter {
            mode: MemoryLimiterMode::Enforce,
            check_interval: Duration::from_secs(1),
            soft_limit_bytes: 90,
            hard_limit_bytes: 95,
            hysteresis_bytes: 5,
            purge_on_hard: true,
            purge_min_interval: Duration::from_secs(5),
            last_purge_at: Some(Instant::now()),
            sampler,
        };
        let state = MemoryPressureState::default();

        let tick = limiter.tick(&state).expect("tick should succeed");

        assert_eq!(purge_calls.load(Ordering::Relaxed), 0);
        assert_eq!(tick.pre_purge_usage_bytes, None);
        assert!(tick.purge_duration.is_none());
        assert_eq!(tick.current_level, MemoryPressureLevel::Hard);
    }

    #[test]
    fn tick_purge_failure_is_non_fatal() {
        let sampler = MemoryUsageSampler::from_test_probes(
            Box::new(SequenceProbe {
                samples: VecDeque::from([MemorySample {
                    usage_bytes: 96,
                    source: MemorySampleSource::Rss,
                }]),
            }),
            Some(Box::new(FailingPurgeHook)),
        );
        let mut limiter = EffectiveMemoryLimiter {
            mode: MemoryLimiterMode::Enforce,
            check_interval: Duration::from_secs(1),
            soft_limit_bytes: 90,
            hard_limit_bytes: 95,
            hysteresis_bytes: 5,
            purge_on_hard: true,
            purge_min_interval: Duration::from_secs(5),
            last_purge_at: None,
            sampler,
        };
        let state = MemoryPressureState::default();

        let tick = limiter.tick(&state).expect("tick should succeed");

        assert_eq!(tick.pre_purge_usage_bytes, Some(96));
        assert_eq!(tick.current_level, MemoryPressureLevel::Hard);
        assert!(tick.purge_duration.is_none());
        assert_eq!(tick.purge_error.as_deref(), Some("purge failed"));
        assert!(limiter.last_purge_at.is_some());
        assert_eq!(state.level(), MemoryPressureLevel::Hard);
        assert_eq!(state.usage_bytes(), 96);
    }

    #[test]
    fn tick_post_purge_sample_failure_is_non_fatal() {
        let purge_calls = Arc::new(AtomicUsize::new(0));
        let sampler = MemoryUsageSampler::from_test_probes(
            Box::new(SequenceProbe {
                samples: VecDeque::from([MemorySample {
                    usage_bytes: 96,
                    source: MemorySampleSource::Rss,
                }]),
            }),
            Some(Box::new(CountingPurgeHook {
                calls: purge_calls.clone(),
            })),
        );
        let mut limiter = EffectiveMemoryLimiter {
            mode: MemoryLimiterMode::Enforce,
            check_interval: Duration::from_secs(1),
            soft_limit_bytes: 90,
            hard_limit_bytes: 95,
            hysteresis_bytes: 5,
            purge_on_hard: true,
            purge_min_interval: Duration::from_secs(5),
            last_purge_at: None,
            sampler,
        };
        let state = MemoryPressureState::default();

        let tick = limiter.tick(&state).expect("tick should succeed");

        assert_eq!(purge_calls.load(Ordering::Relaxed), 1);
        assert_eq!(tick.pre_purge_usage_bytes, Some(96));
        assert!(tick.purge_duration.is_some());
        assert_eq!(
            tick.purge_error.as_deref(),
            Some("post-purge sample failed: no test sample available")
        );
        assert_eq!(tick.current_level, MemoryPressureLevel::Hard);
        assert_eq!(state.level(), MemoryPressureLevel::Hard);
        assert_eq!(state.usage_bytes(), 96);
    }

    #[test]
    fn observe_only_suppresses_purge_attempts() {
        let purge_calls = Arc::new(AtomicUsize::new(0));
        let sampler = MemoryUsageSampler::from_test_probes(
            Box::new(SequenceProbe {
                samples: VecDeque::from([MemorySample {
                    usage_bytes: 96,
                    source: MemorySampleSource::Rss,
                }]),
            }),
            Some(Box::new(CountingPurgeHook {
                calls: purge_calls.clone(),
            })),
        );
        let mut limiter = EffectiveMemoryLimiter {
            mode: MemoryLimiterMode::ObserveOnly,
            check_interval: Duration::from_secs(1),
            soft_limit_bytes: 90,
            hard_limit_bytes: 95,
            hysteresis_bytes: 5,
            purge_on_hard: true,
            purge_min_interval: Duration::from_secs(5),
            last_purge_at: None,
            sampler,
        };
        let state = MemoryPressureState::default();

        let tick = limiter.tick(&state).expect("tick should succeed");

        assert_eq!(purge_calls.load(Ordering::Relaxed), 0);
        assert_eq!(tick.pre_purge_usage_bytes, None);
        assert!(tick.purge_duration.is_none());
        assert_eq!(tick.current_level, MemoryPressureLevel::Hard);
    }

    // --- PR1: graduated Soft-pressure shed (soft_action) ---

    fn shed_cfg(mode: MemoryLimiterMode, soft_action: SoftAction) -> MemoryPressureBehaviorConfig {
        MemoryPressureBehaviorConfig {
            retry_after_secs: 1,
            fail_readiness_on_hard: true,
            mode,
            soft_action,
        }
    }

    #[test]
    fn shed_decision_covers_full_mode_level_action_matrix() {
        use MemoryLimiterMode::{Enforce, ObserveOnly};
        use MemoryPressureLevel::{Hard, Normal, Soft};
        use SoftAction::{Observe, Shed};
        // Independent oracle table covering the FULL input space
        // (2 modes x 3 levels x 2 actions = 12 cases). `shed_decision` is a pure
        // total function, so an exhaustive table is a complete oracle.
        let cases = [
            (ObserveOnly, Normal, Observe, false),
            (ObserveOnly, Normal, Shed, false),
            (ObserveOnly, Soft, Observe, false),
            (ObserveOnly, Soft, Shed, false),
            (ObserveOnly, Hard, Observe, false),
            (ObserveOnly, Hard, Shed, false),
            (Enforce, Normal, Observe, false),
            (Enforce, Normal, Shed, false),
            (Enforce, Soft, Observe, false),
            (Enforce, Soft, Shed, true),
            (Enforce, Hard, Observe, true),
            (Enforce, Hard, Shed, true),
        ];
        for (mode, level, action, expected) in cases {
            assert_eq!(
                shed_decision(mode, level, action),
                expected,
                "mode={mode:?} level={level:?} action={action:?}"
            );
        }
    }

    #[test]
    fn default_soft_action_keeps_soft_advisory() {
        // Default soft_action (Observe) must be byte-identical to pre-PR1: Hard
        // sheds, Soft/Normal do not.
        let state = MemoryPressureState::default();
        state.set_level_for_tests(MemoryPressureLevel::Soft);
        assert!(!state.should_shed_ingress());
        state.set_level_for_tests(MemoryPressureLevel::Hard);
        assert!(state.should_shed_ingress());
        state.set_level_for_tests(MemoryPressureLevel::Normal);
        assert!(!state.should_shed_ingress());
    }

    #[test]
    fn soft_action_shed_sheds_soft_in_process_and_receiver_states() {
        let state = MemoryPressureState::default();
        state.configure(shed_cfg(MemoryLimiterMode::Enforce, SoftAction::Shed));
        state.set_level_for_tests(MemoryPressureLevel::Soft);
        assert!(state.should_shed_ingress(), "process sheds Soft when Shed");

        let shared = SharedReceiverAdmissionState::from_process_state(&state);
        let local = LocalReceiverAdmissionState::from_process_state(&state);
        let update = MemoryPressureChanged {
            generation: 1,
            level: MemoryPressureLevel::Soft,
            retry_after_secs: 1,
            usage_bytes: 0,
        };
        shared.apply(update);
        local.apply(update);
        assert!(
            shared.should_shed_ingress(),
            "shared receiver sheds Soft when Shed"
        );
        assert!(
            local.should_shed_ingress(),
            "local receiver sheds Soft when Shed"
        );
    }

    #[test]
    fn all_three_sites_agree_with_oracle_across_full_matrix() {
        use MemoryLimiterMode::{Enforce, ObserveOnly};
        use MemoryPressureLevel::{Hard, Normal, Soft};
        use SoftAction::{Observe, Shed};
        // Same exhaustive oracle as the shed_decision matrix test, but asserted
        // through all THREE public should_shed_ingress() sites (process-wide,
        // shared-receiver, local-receiver). Pins site agreement so a future edit
        // that changes only one site is caught for every (mode, level, action).
        let cases = [
            (ObserveOnly, Normal, Observe, false),
            (ObserveOnly, Normal, Shed, false),
            (ObserveOnly, Soft, Observe, false),
            (ObserveOnly, Soft, Shed, false),
            (ObserveOnly, Hard, Observe, false),
            (ObserveOnly, Hard, Shed, false),
            (Enforce, Normal, Observe, false),
            (Enforce, Normal, Shed, false),
            (Enforce, Soft, Observe, false),
            (Enforce, Soft, Shed, true),
            (Enforce, Hard, Observe, true),
            (Enforce, Hard, Shed, true),
        ];
        for (mode, level, action, expected) in cases {
            let state = MemoryPressureState::default();
            state.configure(shed_cfg(mode, action));
            state.set_level_for_tests(level);
            assert_eq!(
                state.should_shed_ingress(),
                expected,
                "process: mode={mode:?} level={level:?} action={action:?}"
            );

            let shared = SharedReceiverAdmissionState::from_process_state(&state);
            let local = LocalReceiverAdmissionState::from_process_state(&state);
            let update = MemoryPressureChanged {
                generation: 1,
                level,
                retry_after_secs: 1,
                usage_bytes: 0,
            };
            shared.apply(update);
            local.apply(update);
            assert_eq!(
                shared.should_shed_ingress(),
                expected,
                "shared: mode={mode:?} level={level:?} action={action:?}"
            );
            assert_eq!(
                local.should_shed_ingress(),
                expected,
                "local: mode={mode:?} level={level:?} action={action:?}"
            );
        }
    }

    #[test]
    fn observe_only_overrides_soft_action_shed() {
        let state = MemoryPressureState::default();
        state.configure(shed_cfg(MemoryLimiterMode::ObserveOnly, SoftAction::Shed));
        for level in [MemoryPressureLevel::Soft, MemoryPressureLevel::Hard] {
            state.set_level_for_tests(level);
            assert!(
                !state.should_shed_ingress(),
                "ObserveOnly never sheds at {level:?}"
            );
            let shared = SharedReceiverAdmissionState::from_process_state(&state);
            shared.apply(MemoryPressureChanged {
                generation: 1,
                level,
                retry_after_secs: 1,
                usage_bytes: 0,
            });
            assert!(!shared.should_shed_ingress());
        }
    }

    #[test]
    fn stale_generation_does_not_change_shed_decision() {
        let state = MemoryPressureState::default();
        state.configure(shed_cfg(MemoryLimiterMode::Enforce, SoftAction::Shed));
        let shared = SharedReceiverAdmissionState::from_process_state(&state);
        shared.apply(MemoryPressureChanged {
            generation: 2,
            level: MemoryPressureLevel::Soft,
            retry_after_secs: 1,
            usage_bytes: 0,
        });
        assert!(shared.should_shed_ingress());
        // A stale (older-generation) Normal update must be ignored.
        shared.apply(MemoryPressureChanged {
            generation: 1,
            level: MemoryPressureLevel::Normal,
            retry_after_secs: 1,
            usage_bytes: 0,
        });
        assert!(
            shared.should_shed_ingress(),
            "stale update must not lift shedding"
        );
    }

    #[test]
    fn retry_after_zero_is_clamped_to_one() {
        let shared =
            SharedReceiverAdmissionState::from_process_state(&MemoryPressureState::default());
        shared.apply(MemoryPressureChanged {
            generation: 1,
            level: MemoryPressureLevel::Hard,
            retry_after_secs: 0,
            usage_bytes: 0,
        });
        assert_eq!(shared.retry_after_secs(), 1);
        let local =
            LocalReceiverAdmissionState::from_process_state(&MemoryPressureState::default());
        local.apply(MemoryPressureChanged {
            generation: 1,
            level: MemoryPressureLevel::Hard,
            retry_after_secs: 0,
            usage_bytes: 0,
        });
        assert_eq!(local.retry_after_secs(), 1);
    }
}
