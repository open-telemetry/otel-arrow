// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Process-wide memory limiter state and sampling.

use otap_df_config::policy::{MemoryLimiterMode, MemoryLimiterPolicy, MemoryLimiterSource};
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
    /// Above the soft limit; pressure is elevated but ingress still flows in Phase 1.
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
            .fail_readiness_on_hard
            .store(config.fail_readiness_on_hard, Ordering::Relaxed);
    }

    /// Returns the current pressure level.
    #[must_use]
    pub fn level(&self) -> MemoryPressureLevel {
        MemoryPressureLevel::from_u8(self.inner.level.load(Ordering::Relaxed))
    }

    /// Returns whether ingress should be rejected.
    ///
    /// In Phase 1, only `Hard` pressure sheds ingress. `Soft` remains advisory so
    /// operators can observe rising pressure without immediately rejecting traffic.
    #[must_use]
    pub fn should_shed_ingress(&self) -> bool {
        mode_from_u8(self.inner.mode.load(Ordering::Relaxed)) == MemoryLimiterMode::Enforce
            && self.level() == MemoryPressureLevel::Hard
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
        self.inner.mode == MemoryLimiterMode::Enforce
            && MemoryPressureLevel::from_u8(self.inner.level.load(Ordering::Relaxed))
                == MemoryPressureLevel::Hard
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
        self.inner.mode == MemoryLimiterMode::Enforce
            && self.inner.level.get() == MemoryPressureLevel::Hard
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

        let hysteresis_bytes = policy.hysteresis.unwrap_or_else(|| {
            hard_limit_bytes
                .saturating_sub(soft_limit_bytes)
                .min(soft_limit_bytes.saturating_sub(1))
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
        });
        _ = state.update_level(MemoryPressureLevel::Soft, 91);
        assert!(!state.should_fail_readiness());

        _ = state.update_level(MemoryPressureLevel::Hard, 96);
        assert!(state.should_fail_readiness());

        state.configure(MemoryPressureBehaviorConfig {
            retry_after_secs: 3,
            fail_readiness_on_hard: false,
            mode: MemoryLimiterMode::Enforce,
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

        assert_eq!(limiter.hysteresis_bytes, 99);
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
}
