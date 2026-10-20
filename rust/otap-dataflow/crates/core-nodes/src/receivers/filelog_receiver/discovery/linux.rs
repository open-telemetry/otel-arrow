// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::probe::{ProbeContext, ProbeScratch};
use super::roots::{ExclusionRoot, resolve_exclusion, resolve_include_root};
use super::walk::Walker;
use super::*;
use rustix::fs::{CWD, Mode, OFlags, ResolveFlags, openat2};
use std::mem::MaybeUninit;
use std::os::unix::ffi::OsStrExt;

// Retain allocation capacity across delivery and subsequent passes. The consumer
// borrows only the initialized prefix; no descriptor or unbounded queue lives here.
pub(super) struct Batch {
    slots: Vec<Observation>,
    used: usize,
}

impl Batch {
    pub(super) fn new(limits: ScanLimits) -> Self {
        Self {
            slots: (0..limits.batch)
                .map(|_| Observation {
                    path: PathBuf::with_capacity(limits.path_bytes),
                    locator: None,
                    size: None,
                    modified: None,
                    fingerprint: Vec::with_capacity(limits.fingerprint_bytes),
                    reason: None,
                    errno: None,
                })
                .collect(),
            used: 0,
        }
    }
    pub(super) fn clear(&mut self) {
        self.used = 0;
    }
    pub(super) fn is_full(&self) -> bool {
        self.used == self.slots.len()
    }
    pub(super) fn emit(&mut self, path: &Path, reason: ScanReason, errno: Option<i32>) {
        // Overflow is an internal invariant violation: stop rather than lose
        // evidence and risk reporting incomplete coverage as complete.
        assert!(!self.is_full(), "deliver a full batch before emitting");
        let out = &mut self.slots[self.used];
        self.used += 1;
        out.path.clear();
        out.path.push(path);
        out.locator = None;
        out.size = None;
        out.modified = None;
        out.fingerprint.clear();
        out.reason = Some(reason);
        out.errno = errno;
    }
    pub(super) fn last_mut(&mut self) -> &mut Observation {
        &mut self.slots[self.used - 1]
    }
}
impl std::ops::Deref for Batch {
    type Target = [Observation];
    fn deref(&self) -> &Self::Target {
        &self.slots[..self.used]
    }
}

/// Linux scanner with bounded reusable scratch and root baselines.
///
/// `scan` emits observations, not reconciliation transitions. Matching policy
/// is supplied precompiled; include-root baselines reset when this scanner is recreated. Expected
/// roots returning after an outage may be used again. A changed root does not
/// become the new baseline automatically.
pub struct Scanner<P> {
    config: ScanConfig,
    matcher: P,
    include_baselines: Vec<Option<FileLocator>>,
    exclusions: Vec<ExclusionRoot>,
    capability: Option<Result<(), i32>>,
    scratch: ProbeScratch,
    target: Vec<u8>,
    directory_buffer: Vec<MaybeUninit<u8>>,
    walker: Walker,
    batch: Batch,
}

impl<P: MatchPolicy> Scanner<P> {
    /// Checks all explicit bounds before allocating scratch.
    pub fn new(config: ScanConfig, matcher: P) -> Result<Self, InvalidScanConfig> {
        let limits = config.limits;
        if !(1..=1024).contains(&limits.depth)
            || !(1..=65536).contains(&limits.path_bytes)
            || !(1..=1024).contains(&limits.roots)
            || !(1..=1024).contains(&limits.batch)
            || !(16..=65535).contains(&limits.fingerprint_bytes)
            || limits.work_units == 0
            || config.include_roots.is_empty()
            || config.include_roots.len() > limits.roots
            || config.exclusion_roots.len() > limits.roots
            || limits
                .ignored_header_bytes
                .checked_add(limits.fingerprint_bytes as u64)
                .is_none_or(|end| end > i64::MAX as u64)
            || config
                .include_roots
                .iter()
                .chain(&config.exclusion_roots)
                .any(|root| {
                    !root.is_absolute()
                        || root
                            .components()
                            .any(|c| matches!(c, std::path::Component::ParentDir))
                        || root.as_os_str().as_bytes().len() >= limits.path_bytes
                        || root.as_os_str().as_bytes().contains(&0)
                })
        {
            return Err(InvalidScanConfig);
        }
        Ok(Self {
            include_baselines: vec![None; config.include_roots.len()],
            exclusions: config
                .exclusion_roots
                .iter()
                .enumerate()
                .map(|(index, _)| ExclusionRoot {
                    index,
                    locator: FileLocator {
                        device: 0,
                        inode: 0,
                    },
                    missing_suffix: PathBuf::new(),
                })
                .collect(),
            capability: None,
            scratch: ProbeScratch::new(limits),
            target: vec![0; limits.path_bytes],
            directory_buffer: vec![MaybeUninit::uninit(); 8192],
            walker: Walker::new(limits),
            batch: Batch::new(limits),
            config,
            matcher,
        })
    }

    #[cfg(test)]
    pub(super) fn inject_capability_result(&mut self, result: Result<(), i32>) {
        self.capability = Some(result);
    }

    /// Accounts for scanner-owned storage, including spare capacity.
    ///
    /// Matcher-owned heap belongs to configuration accounting. Consumer copies
    /// belong to the caller. Call only between synchronous scans; traversal
    /// peak adds at most `depth` frames with two bounded native paths each.
    #[must_use]
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.include_baselines.capacity() * size_of::<Option<FileLocator>>()
            + self.exclusions.capacity() * size_of::<ExclusionRoot>()
            + self
                .exclusions
                .iter()
                .map(|r| r.missing_suffix.capacity())
                .sum::<usize>()
            + self.config.include_roots.capacity() * size_of::<PathBuf>()
            + self.config.exclusion_roots.capacity() * size_of::<PathBuf>()
            + self
                .config
                .include_roots
                .iter()
                .chain(&self.config.exclusion_roots)
                .map(|p| p.capacity())
                .sum::<usize>()
            + self.scratch.retained_bytes()
            + self.target.capacity()
            + self.directory_buffer.capacity()
            + self.walker.heap_bytes()
            + self.batch.slots.capacity() * size_of::<Observation>()
            + self
                .batch
                .slots
                .iter()
                .map(|o| o.path.capacity() + o.fingerprint.capacity())
                .sum::<usize>()
    }

    /// Scans roots in configuration order and delivers batches synchronously.
    ///
    /// Returning false from `consume` cancels the pass. Consumers must bound any
    /// copying and check cancellation while blocked. No directory or source
    /// descriptor is held during delivery. Filesystem errors are not retried;
    /// probe-budget contention gets one scheduler yield/retry. Environmental
    /// backoff and scheduling belong to the worker.
    pub fn scan(
        &mut self,
        generation: u64,
        budget: &ProbeBudget,
        cancelled: impl FnMut() -> bool,
        consume: impl FnMut(&[Observation]) -> bool,
    ) -> ScanCompletion {
        self.scan_with_requests(
            generation,
            budget,
            |_| ProbeRequest::Full,
            cancelled,
            consume,
        )
    }

    /// Scans using a caller-owned bounded lookup to request presence-only evidence.
    /// Request full validation whenever admission or lifecycle validation is needed.
    pub fn scan_with_requests(
        &mut self,
        generation: u64,
        budget: &ProbeBudget,
        mut request: impl FnMut(&Path) -> ProbeRequest,
        mut cancelled: impl FnMut() -> bool,
        mut consume: impl FnMut(&[Observation]) -> bool,
    ) -> ScanCompletion {
        let mut completion = ScanCompletion {
            generation,
            complete: true,
            cancelled: false,
            observations: 0,
        };
        self.walker.clear();
        self.batch.clear();
        if cancelled() {
            completion.complete = false;
            completion.cancelled = true;
            return completion;
        }
        let capability = self.capability.unwrap_or_else(|| {
            openat2(
                CWD,
                c"/",
                OFlags::PATH | OFlags::CLOEXEC,
                Mode::empty(),
                ResolveFlags::NO_SYMLINKS,
            )
            .map(drop)
            .map_err(|e| e.raw_os_error())
        });
        self.capability = match capability {
            Ok(()) | Err(libc::ENOSYS | libc::EINVAL | libc::EPERM) => Some(capability),
            Err(_) => None,
        };
        if let Err(errno) = capability {
            self.batch.emit(
                Path::new("/"),
                if matches!(errno, libc::EMFILE | libc::ENFILE) {
                    ScanReason::DescriptorExhausted
                } else {
                    ScanReason::VerificationUnavailable
                },
                Some(errno),
            );
            let _ = deliver(&mut self.batch, &mut completion, &mut consume);
            return completion;
        }
        let mut work_left = self.config.limits.work_units;
        let mut policy_available = true;
        for (index, root) in self.config.exclusion_roots.iter().enumerate() {
            if cancelled() {
                completion.cancelled = true;
                break;
            }
            match resolve_exclusion(
                root,
                &mut self.target,
                &mut self.exclusions[index],
                &mut cancelled,
            ) {
                Ok(()) => (),
                Err((reason, error)) => {
                    policy_available = false;
                    completion.complete = false;
                    self.batch.emit(root, reason, error.raw_os_error());
                    if (reason == ScanReason::Cancelled || self.batch.is_full())
                        && !deliver(&mut self.batch, &mut completion, &mut consume)
                    {
                        break;
                    }
                }
            }
        }
        // Failed mappings never enter policy evaluation, preserving root indexes.
        let context = ProbeContext {
            config: &self.config,
            matcher: &self.matcher,
            roots: &self.exclusions,
            budget,
            policy_available,
        };
        for index in 0..self.config.include_roots.len() {
            if work_left == 0 {
                self.batch
                    .emit(&self.config.include_roots[index], ScanReason::Limit, None);
                completion.complete = false;
                break;
            }
            if completion.cancelled || cancelled() {
                completion.cancelled = true;
                break;
            }
            let root = &self.config.include_roots[index];
            match resolve_include_root(root, &mut self.target, &mut self.include_baselines[index]) {
                Ok((physical, locator, changed)) => {
                    if changed {
                        self.batch
                            .emit(root, ScanReason::RootChanged, Some(libc::ESTALE));
                        self.batch.last_mut().locator = Some(locator);
                        completion.complete = false;
                        if self.batch.is_full()
                            && !deliver(&mut self.batch, &mut completion, &mut consume)
                        {
                            break;
                        }
                    }
                    if locator == self.config.checkpoint_namespace
                        || self.matcher.excludes_subtree(root)
                    {
                        continue;
                    }
                    self.walker
                        .start(root, &physical, locator, self.config.limits);
                }
                Err((reason, error)) => {
                    self.batch.emit(root, reason, error.raw_os_error());
                    completion.complete = false;
                }
            }
            while !self.walker.is_empty() && !completion.cancelled {
                if work_left == 0 {
                    self.batch
                        .emit(self.walker.current_path(), ScanReason::Limit, None);
                    completion.complete = false;
                    break;
                }
                if cancelled() {
                    completion.cancelled = true;
                    break;
                }
                self.walker.step(
                    &context,
                    &mut self.batch,
                    &mut self.scratch,
                    &mut self.directory_buffer,
                    &mut work_left,
                    &mut request,
                    &mut cancelled,
                );
                if !deliver(&mut self.batch, &mut completion, &mut consume) {
                    break;
                }
            }
            if !completion.cancelled && !deliver(&mut self.batch, &mut completion, &mut consume) {
                break;
            }
        }
        if !completion.cancelled {
            let _ = deliver(&mut self.batch, &mut completion, &mut consume);
        }
        if completion.cancelled {
            completion.complete = false;
        }
        self.walker.clear();
        self.batch.clear();
        completion
    }
}

fn deliver(
    batch: &mut Batch,
    completion: &mut ScanCompletion,
    consume: &mut impl FnMut(&[Observation]) -> bool,
) -> bool {
    if batch.is_empty() {
        return !completion.cancelled;
    }
    for observation in batch.iter() {
        if matches!(
            observation.reason,
            Some(
                ScanReason::VerificationUnavailable
                    | ScanReason::RootUnavailable
                    | ScanReason::RootChanged
                    | ScanReason::Coverage
                    | ScanReason::Resumption
                    | ScanReason::UnqualifiedResumption
                    | ScanReason::Cycle
                    | ScanReason::Limit
                    | ScanReason::Policy
                    | ScanReason::Unstable
                    | ScanReason::ProbeBusy
                    | ScanReason::DescriptorExhausted
                    | ScanReason::Cancelled
            )
        ) {
            completion.complete = false;
        }
        if observation.reason == Some(ScanReason::Cancelled) {
            completion.cancelled = true;
        }
    }
    completion.observations = completion.observations.saturating_add(batch.len() as u64);
    if !consume(batch) {
        completion.complete = false;
        completion.cancelled = true;
    }
    batch.clear();
    !completion.cancelled
}
