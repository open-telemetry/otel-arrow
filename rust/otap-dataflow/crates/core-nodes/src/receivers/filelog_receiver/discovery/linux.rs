// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::probe::{ExclusionRoot, ProbeContext, ProbeScratch, pinned_path, probe, verify_path};
use super::*;
use rustix::fs::{
    AtFlags, CWD, FileType, Mode, OFlags, RawDir, ResolveFlags, SeekFrom, openat, openat2, seek,
    statat,
};
use std::fs::{File, Metadata};
use std::io;
use std::mem::MaybeUninit;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DirectoryEvidence {
    locator: FileLocator,
    modified: (i64, i64),
    changed: (i64, i64),
}

impl DirectoryEvidence {
    fn new(metadata: &Metadata) -> Self {
        Self {
            locator: FileLocator::from_metadata(metadata),
            modified: (metadata.mtime(), metadata.mtime_nsec()),
            changed: (metadata.ctime(), metadata.ctime_nsec()),
        }
    }
}

struct Frame {
    lexical: PathBuf,
    physical: PathBuf,
    evidence: Option<DirectoryEvidence>,
    cookie: u64,
    expected: Option<FileLocator>,
    replay: bool,
    replay_reported: bool,
    uncertain: bool,
    visited: u64,
    prefix: [u8; 32],
}

// Retain allocation capacity across delivery and subsequent passes. The consumer
// borrows only the initialized prefix; no descriptor or unbounded queue lives here.
struct Batch {
    slots: Vec<Observation>,
    used: usize,
}

impl Batch {
    fn new(limits: ScanLimits) -> Self {
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
    fn clear(&mut self) {
        self.used = 0;
    }
    fn emit(&mut self, path: &Path, reason: ScanReason, errno: Option<i32>) {
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
    fn last_mut(&mut self) -> &mut Observation {
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
    entry_path: PathBuf,
    scratch: ProbeScratch,
    target: Vec<u8>,
    directory_buffer: Vec<MaybeUninit<u8>>,
    stack: Vec<Frame>,
    free_frames: Vec<Frame>,
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
            entry_path: PathBuf::with_capacity(limits.path_bytes),
            scratch: ProbeScratch::new(limits),
            target: vec![0; limits.path_bytes],
            directory_buffer: vec![MaybeUninit::uninit(); 8192],
            stack: Vec::with_capacity(limits.depth),
            free_frames: Vec::with_capacity(limits.depth),
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
            + self.entry_path.capacity()
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
            + (self.stack.capacity() + self.free_frames.capacity()) * size_of::<Frame>()
            + self
                .stack
                .iter()
                .chain(&self.free_frames)
                .map(|f| f.lexical.capacity() + f.physical.capacity())
                .sum::<usize>()
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
        while !self.stack.is_empty() {
            recycle_frame(&mut self.stack, &mut self.free_frames);
        }
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
                    self.batch.emit(
                        root,
                        if reason == ScanReason::RootChanged {
                            reason
                        } else {
                            ScanReason::Policy
                        },
                        error.raw_os_error(),
                    );
                    if self.batch.len() == self.config.limits.batch
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
                        if self.batch.len() == self.config.limits.batch
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
                    self.stack.push(take_frame(
                        &mut self.free_frames,
                        root,
                        &physical,
                        Some(locator),
                        self.config.limits.path_bytes,
                    ));
                }
                Err((reason, error)) => {
                    self.batch.emit(root, reason, error.raw_os_error());
                    completion.complete = false;
                }
            }
            while !self.stack.is_empty() && !completion.cancelled {
                if work_left == 0 {
                    self.batch.emit(
                        &self.stack.last().expect("active frame").lexical,
                        ScanReason::Limit,
                        None,
                    );
                    completion.complete = false;
                    break;
                }
                if cancelled() {
                    completion.cancelled = true;
                    break;
                }
                let result = chunk(
                    &context,
                    &mut self.stack,
                    &mut self.free_frames,
                    &mut self.batch,
                    &mut self.scratch,
                    &mut self.directory_buffer,
                    &mut self.entry_path,
                    &mut work_left,
                    &mut request,
                    &mut cancelled,
                );
                if let Some(child) = result {
                    self.stack.push(child);
                }
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
        while !self.stack.is_empty() {
            recycle_frame(&mut self.stack, &mut self.free_frames);
        }
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

fn resolve_include_root(
    root: &Path,
    scratch: &mut [u8],
    baseline: &mut Option<FileLocator>,
) -> Result<(PathBuf, FileLocator, bool), (ScanReason, io::Error)> {
    let mut resolve = || -> io::Result<(PathBuf, FileLocator)> {
        let file = File::from(openat(
            CWD,
            root,
            OFlags::PATH | OFlags::DIRECTORY | OFlags::CLOEXEC,
            Mode::empty(),
        )?);
        let locator = FileLocator::from_metadata(&file.metadata()?);
        let path = pinned_path(&file, scratch)?.to_owned();
        drop(file); // One directory handle, including root verification.
        verify_path(&path, locator)?;
        Ok((path, locator))
    };
    let (path, locator) = resolve().map_err(|error| (ScanReason::RootUnavailable, error))?;
    let changed = baseline.is_some_and(|old| old != locator);
    if baseline.is_none() {
        *baseline = Some(locator);
    }
    // Positive observations may use the current directory without adopting it
    // as the expected environment or authorizing negative conclusions.
    Ok((path, locator, changed))
}

fn chunk<P: MatchPolicy>(
    context: &ProbeContext<'_, P>,
    stack: &mut Vec<Frame>,
    free_frames: &mut Vec<Frame>,
    batch: &mut Batch,
    scratch: &mut ProbeScratch,
    buffer: &mut [MaybeUninit<u8>],
    path: &mut PathBuf,
    work_left: &mut u64,
    request: &mut impl FnMut(&Path) -> ProbeRequest,
    cancelled: &mut impl FnMut() -> bool,
) -> Option<Frame> {
    *work_left -= 1; // The caller checks capacity before opening a directory.
    let depth = stack.len();
    let (ancestors, current) = stack.split_at_mut(depth - 1);
    let frame = &mut current[0];
    let open_directory = || -> io::Result<File> {
        let flags = OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC | OFlags::NONBLOCK;
        Ok(File::from(
            if context.config.symlinks == SymlinkPolicy::Reject {
                openat2(
                    CWD,
                    &frame.physical,
                    flags,
                    Mode::empty(),
                    ResolveFlags::NO_SYMLINKS,
                )?
            } else {
                openat(CWD, &frame.physical, flags, Mode::empty())?
            },
        ))
    };
    let file = match open_directory() {
        Ok(file) => file,
        Err(error) => {
            batch.emit(&frame.lexical, ScanReason::Coverage, error.raw_os_error());
            recycle_frame(stack, free_frames);
            return None;
        }
    };
    let evidence = match file.metadata() {
        Ok(metadata) => DirectoryEvidence::new(&metadata),
        Err(error) => {
            batch.emit(&frame.lexical, ScanReason::Coverage, error.raw_os_error());
            recycle_frame(stack, free_frames);
            return None;
        }
    };
    if evidence.locator == context.config.checkpoint_namespace {
        recycle_frame(stack, free_frames);
        return None;
    }
    if frame
        .expected
        .is_some_and(|expected| expected != evidence.locator)
        || (frame.evidence.is_none()
            && ancestors.iter().any(|ancestor| {
                ancestor
                    .evidence
                    .is_some_and(|e| e.locator == evidence.locator)
            }))
        || frame
            .evidence
            .is_some_and(|prior| prior.locator != evidence.locator)
    {
        batch.emit(
            &frame.lexical,
            if frame.evidence.is_none() && frame.expected.is_none() {
                ScanReason::Cycle
            } else {
                ScanReason::Coverage
            },
            None,
        );
        recycle_frame(stack, free_frames);
        return None;
    }
    if frame.evidence.is_some_and(|prior| prior != evidence) && !frame.uncertain {
        // Keep positive progress where resumption is possible, but never turn
        // detected mutation into complete coverage. Report once per directory.
        frame.uncertain = true;
        batch.emit(&frame.lexical, ScanReason::Resumption, None);
    }
    if frame.evidence.is_none() {
        let filesystem = match nix::sys::statfs::fstatfs(&file) {
            Ok(stat) => stat.filesystem_type(),
            Err(error) => {
                batch.emit(&frame.lexical, ScanReason::Resumption, Some(error as i32));
                recycle_frame(stack, free_frames);
                return None;
            }
        };
        frame.replay = !matches!(
            filesystem,
            nix::sys::statfs::EXT4_SUPER_MAGIC
                | nix::sys::statfs::TMPFS_MAGIC
                | nix::sys::statfs::XFS_SUPER_MAGIC
                | nix::sys::statfs::BTRFS_SUPER_MAGIC
        );
        batch.emit(&frame.lexical, ScanReason::Directory, None);
        batch.last_mut().locator = Some(evidence.locator);
    }
    frame.evidence = Some(evidence);
    // Deliver diagnostics before attempting another operation that can emit one.
    // In particular, mutation plus a failed seek must fit a one-slot batch.
    if batch.len() == context.config.limits.batch {
        return None;
    }
    if frame.replay && frame.visited != 0 && !frame.replay_reported {
        batch.emit(&frame.lexical, ScanReason::UnqualifiedResumption, None);
        frame.replay_reported = true;
        if batch.len() == context.config.limits.batch {
            return None;
        }
    }
    if !frame.replay
        && frame.cookie != 0
        && let Err(error) = seek(&file, SeekFrom::Start(frame.cookie))
    {
        batch.emit(
            &frame.lexical,
            ScanReason::Resumption,
            Some(error.raw_os_error()),
        );
        recycle_frame(stack, free_frames);
        return None;
    }
    let mut entries = RawDir::new(&file, buffer);
    if frame.replay && frame.visited != 0 {
        let mut seen = 0;
        let mut prefix = [0; 32];
        while seen < frame.visited {
            if cancelled() || *work_left == 0 {
                batch.emit(
                    &frame.lexical,
                    if *work_left == 0 {
                        ScanReason::Limit
                    } else {
                        ScanReason::Cancelled
                    },
                    None,
                );
                recycle_frame(stack, free_frames);
                return None;
            }
            *work_left -= 1;
            match entries.next() {
                Some(Ok(entry)) => {
                    if entry.file_name() == c"." || entry.file_name() == c".." {
                        continue;
                    }
                    prefix = prefix_hash(prefix, &entry);
                    seen += 1;
                }
                outcome => {
                    let errno = outcome.and_then(Result::err).map(|e| e.raw_os_error());
                    batch.emit(&frame.lexical, ScanReason::Resumption, errno);
                    recycle_frame(stack, free_frames);
                    return None;
                }
            }
        }
        if prefix != frame.prefix {
            batch.emit(&frame.lexical, ScanReason::Resumption, None);
            recycle_frame(stack, free_frames);
            return None;
        }
    }
    while batch.len() < context.config.limits.batch {
        if cancelled() {
            batch.emit(&frame.lexical, ScanReason::Cancelled, None);
            recycle_frame(stack, free_frames);
            return None;
        }
        if *work_left == 0 {
            batch.emit(&frame.lexical, ScanReason::Limit, None);
            recycle_frame(stack, free_frames);
            return None;
        }
        *work_left -= 1;
        let entry = match entries.next() {
            Some(Ok(entry)) => entry,
            Some(Err(error)) => {
                batch.emit(
                    &frame.lexical,
                    ScanReason::Coverage,
                    Some(error.raw_os_error()),
                );
                recycle_frame(stack, free_frames);
                return None;
            }
            None => {
                match file.metadata() {
                    Err(error) => {
                        batch.emit(&frame.lexical, ScanReason::Coverage, error.raw_os_error())
                    }
                    Ok(metadata) if DirectoryEvidence::new(&metadata) != evidence => {
                        batch.emit(&frame.lexical, ScanReason::Resumption, None)
                    }
                    Ok(_) => (),
                }
                recycle_frame(stack, free_frames);
                return None;
            }
        };
        let name = entry.file_name();
        if name == c"." || name == c".." {
            continue;
        }
        frame.cookie = entry.next_entry_cookie();
        if frame.replay {
            frame.visited += 1;
            frame.prefix = prefix_hash(frame.prefix, &entry);
        }
        if frame.lexical.as_os_str().len() + name.to_bytes().len() + 1
            >= context.config.limits.path_bytes
            || frame.physical.as_os_str().len() + name.to_bytes().len() + 1
                >= context.config.limits.path_bytes
        {
            batch.emit(&frame.lexical, ScanReason::Limit, None);
            continue;
        }
        let name_path = Path::new(std::ffi::OsStr::from_bytes(name.to_bytes()));
        path.clear();
        path.push(&frame.lexical);
        path.push(name_path);
        let should_descend =
            context.matcher.may_descend(path) && !context.matcher.excludes_subtree(path);
        let file_type = entry.file_type();
        let mut is_directory = file_type == FileType::Directory;
        let mut observed_regular = file_type == FileType::RegularFile;
        if should_descend
            && (file_type == FileType::Unknown
                || (file_type == FileType::Symlink
                    && context.config.symlinks == SymlinkPolicy::Follow))
        {
            match statat(
                &file,
                name,
                if context.config.symlinks == SymlinkPolicy::Reject {
                    AtFlags::SYMLINK_NOFOLLOW
                } else {
                    AtFlags::empty()
                },
            ) {
                Ok(stat) => {
                    let observed_type = FileType::from_raw_mode(stat.st_mode);
                    is_directory = observed_type == FileType::Directory;
                    observed_regular = observed_type == FileType::RegularFile;
                }
                Err(error) => {
                    // A required subtree's kind is unknown, including ENOENT.
                    let reason = ScanReason::Coverage;
                    batch.emit(path, reason, Some(error.raw_os_error()));
                    continue;
                }
            }
        }
        if is_directory {
            if should_descend {
                if depth == context.config.limits.depth {
                    batch.emit(path, ScanReason::Limit, None);
                } else {
                    let mut child = take_frame(
                        free_frames,
                        path,
                        &frame.physical,
                        None,
                        context.config.limits.path_bytes,
                    );
                    child.physical.push(name_path);
                    return Some(child);
                }
            }
            continue;
        }
        if !context.matcher.includes(path) {
            continue;
        }
        if context.matcher.excludes(path) {
            batch.emit(path, ScanReason::Excluded, None);
            let out = batch.last_mut();
            match statat(
                &file,
                name,
                if context.config.symlinks == SymlinkPolicy::Follow {
                    AtFlags::empty()
                } else {
                    AtFlags::SYMLINK_NOFOLLOW
                },
            ) {
                Ok(stat) if FileType::from_raw_mode(stat.st_mode) == FileType::RegularFile => {
                    out.locator = Some(probe::locator(&stat));
                }
                Ok(stat) => {
                    if should_descend
                        && FileType::from_raw_mode(stat.st_mode) == FileType::Directory
                    {
                        out.reason = Some(ScanReason::Unstable);
                    }
                }
                Err(error) => {
                    out.errno = Some(error.raw_os_error());
                    out.reason = Some(match error {
                        _ if should_descend => ScanReason::Coverage,
                        rustix::io::Errno::NOENT => ScanReason::Missing,
                        rustix::io::Errno::LOOP => ScanReason::LeafLoop,
                        _ => ScanReason::Coverage,
                    });
                }
            }
        } else {
            batch.emit(path, ScanReason::ProbeBusy, None);
            probe(
                context,
                &file,
                name,
                observed_regular,
                request(path),
                batch.last_mut(),
                scratch,
                cancelled,
            );
        }
    }
    None
}

fn prefix_hash(prefix: [u8; 32], entry: &rustix::fs::RawDirEntry<'_>) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    let _ = hasher.update(&prefix);
    let _ = hasher.update(&entry.ino().to_le_bytes());
    let _ = hasher.update(&(entry.file_name().to_bytes().len() as u64).to_le_bytes());
    let _ = hasher.update(entry.file_name().to_bytes());
    *hasher.finalize().as_bytes()
}

// Only an initial path lookup returning ENOENT can shorten a literal prefix.
// A dangling prefix alias or a failure after opening a root remains a failure.
fn resolve_exclusion(
    root: &Path,
    scratch: &mut [u8],
    mapping: &mut ExclusionRoot,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<(), (ScanReason, io::Error)> {
    let mut ancestor = root;
    loop {
        if cancelled() {
            return Err((
                ScanReason::Cancelled,
                io::Error::from_raw_os_error(libc::ECANCELED),
            ));
        }
        match openat(
            CWD,
            ancestor,
            OFlags::PATH | OFlags::DIRECTORY | OFlags::CLOEXEC,
            Mode::empty(),
        ) {
            Ok(fd) => {
                let file = File::from(fd);
                let identity = file
                    .metadata()
                    .map(|m| FileLocator::from_metadata(&m))
                    .map_err(|e| (ScanReason::Policy, e))?;
                let canonical = pinned_path(&file, scratch).map_err(|e| (ScanReason::Policy, e))?;
                drop(file);
                verify_path(canonical, identity).map_err(|e| (ScanReason::Policy, e))?;
                mapping.locator = identity;
                mapping.missing_suffix.clear();
                mapping
                    .missing_suffix
                    .push(root.strip_prefix(ancestor).expect("literal ancestor"));
                return Ok(());
            }
            Err(error) if error == rustix::io::Errno::NOENT => {
                // Unlike metadata(), this exposes a dangling final symlink.
                match std::fs::symlink_metadata(ancestor) {
                    Err(e) if e.raw_os_error() == Some(libc::ENOENT) => (),
                    Err(e) => return Err((ScanReason::Policy, e)),
                    Ok(_) => return Err((ScanReason::Policy, error.into())),
                }
                ancestor = ancestor
                    .parent()
                    .ok_or_else(|| (ScanReason::Policy, io::Error::from(error)))?;
            }
            Err(error) => return Err((ScanReason::Policy, error.into())),
        }
    }
}

fn take_frame(
    pool: &mut Vec<Frame>,
    lexical: &Path,
    physical: &Path,
    expected: Option<FileLocator>,
    path_bytes: usize,
) -> Frame {
    let mut frame = pool.pop().unwrap_or_else(|| Frame {
        lexical: PathBuf::with_capacity(path_bytes),
        physical: PathBuf::with_capacity(path_bytes),
        evidence: None,
        expected: None,
        cookie: 0,
        replay: false,
        replay_reported: false,
        uncertain: false,
        visited: 0,
        prefix: [0; 32],
    });
    frame.lexical.clear();
    frame.lexical.push(lexical);
    frame.physical.clear();
    frame.physical.push(physical);
    frame.evidence = None;
    frame.expected = expected;
    frame.cookie = 0;
    frame.replay = false;
    frame.replay_reported = false;
    frame.uncertain = false;
    frame.visited = 0;
    frame.prefix = [0; 32];
    frame
}
fn recycle_frame(stack: &mut Vec<Frame>, pool: &mut Vec<Frame>) {
    if let Some(frame) = stack.pop() {
        pool.push(frame);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NoMatches;
    impl MatchPolicy for NoMatches {
        fn includes(&self, _: &Path) -> bool {
            false
        }
        fn may_descend(&self, _: &Path) -> bool {
            false
        }
        fn excludes(&self, _: &Path) -> bool {
            false
        }
        fn excludes_subtree(&self, _: &Path) -> bool {
            false
        }
        fn excludes_mapped(&self, _: usize, _: &Path) -> bool {
            false
        }
    }

    /// Scenario: A changed directory has an invalid saved cookie and only one diagnostic slot.
    /// Guarantees: Mutation and seek failure are delivered separately without overflowing the batch.
    #[test]
    fn one_slot_mutation_then_failed_seek() {
        let dir = tempfile::tempdir().unwrap();
        let config = ScanConfig {
            include_roots: vec![dir.path().to_owned()],
            exclusion_roots: vec![],
            symlinks: SymlinkPolicy::Reject,
            checkpoint_namespace: FileLocator {
                device: u64::MAX,
                inode: u64::MAX,
            },
            limits: ScanLimits {
                batch: 1,
                ..ScanLimits::default()
            },
        };
        let mut scanner = Scanner::new(config, NoMatches).unwrap();
        let mut frame = take_frame(
            &mut scanner.free_frames,
            dir.path(),
            dir.path(),
            None,
            scanner.config.limits.path_bytes,
        );
        let mut previous = DirectoryEvidence::new(&std::fs::metadata(dir.path()).unwrap());
        previous.modified.0 = previous.modified.0.wrapping_sub(1);
        frame.evidence = Some(previous);
        // Native directory seeking rejects a negative offset on the qualified backends.
        frame.cookie = u64::MAX;
        scanner.stack.push(frame);
        let budget = ProbeBudget::default();
        let context = ProbeContext {
            config: &scanner.config,
            matcher: &scanner.matcher,
            roots: &[],
            budget: &budget,
            policy_available: true,
        };
        let mut work = 10;
        for attempt in 0..2 {
            assert!(
                chunk(
                    &context,
                    &mut scanner.stack,
                    &mut scanner.free_frames,
                    &mut scanner.batch,
                    &mut scanner.scratch,
                    &mut scanner.directory_buffer,
                    &mut scanner.entry_path,
                    &mut work,
                    &mut |_| ProbeRequest::Full,
                    &mut || false
                )
                .is_none()
            );
            assert_eq!(scanner.batch.len(), 1);
            assert_eq!(scanner.batch[0].reason, Some(ScanReason::Resumption));
            if attempt == 0 {
                assert_eq!(scanner.batch[0].errno, None);
                assert_eq!(scanner.stack.len(), 1);
            } else {
                assert!(scanner.batch[0].errno.is_some());
                assert!(scanner.stack.is_empty());
            }
            scanner.batch.clear();
        }
    }
}
