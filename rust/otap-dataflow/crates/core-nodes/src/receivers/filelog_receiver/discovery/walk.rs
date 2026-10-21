// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::batch::Batch;
use super::native_path::NativePath;
use super::probe::{ProbeContext, ProbeScratch, probe};
use super::*;
use rustix::fs::{
    AtFlags, CWD, FileType, Mode, OFlags, RawDir, ResolveFlags, SeekFrom, openat, openat2, seek,
    statat,
};
use std::fs::{File, Metadata};
use std::os::unix::{ffi::OsStrExt, fs::MetadataExt};
use std::{io, mem::MaybeUninit, ops::ControlFlow};

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

/// Owns traversal state; the directory buffer stays with the scanner so RawDir
/// can borrow it without blocking mutation of the frame stack and pool.
pub(super) struct Walker {
    stack: Vec<Frame>,
    free_frames: Vec<Frame>,
    path: PathBuf,
}

enum WalkStep {
    Descend(Frame),
    Deliver,
    Finished,
    Failed,
}

impl Walker {
    pub fn new(limits: ScanLimits) -> Self {
        Self {
            stack: Vec::with_capacity(limits.depth),
            free_frames: Vec::with_capacity(limits.depth),
            path: PathBuf::with_capacity(limits.path_bytes),
        }
    }
    pub fn clear(&mut self) {
        while !self.stack.is_empty() {
            recycle_frame(&mut self.stack, &mut self.free_frames);
        }
    }
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }
    pub fn current_path(&self) -> &Path {
        &self.stack.last().expect("active traversal").lexical
    }
    pub fn start(
        &mut self,
        lexical: &Path,
        physical: &Path,
        locator: FileLocator,
        limits: ScanLimits,
    ) {
        self.stack.push(take_frame(
            &mut self.free_frames,
            lexical,
            physical,
            Some(locator),
            limits.path_bytes,
        ));
    }
    pub fn heap_bytes(&self) -> usize {
        self.path.capacity()
            + (self.stack.capacity() + self.free_frames.capacity()) * size_of::<Frame>()
            + self
                .stack
                .iter()
                .chain(&self.free_frames)
                .map(|f| f.lexical.capacity() + f.physical.capacity())
                .sum::<usize>()
    }
    pub fn step<P: MatchPolicy>(
        &mut self,
        context: &ProbeContext<'_, P>,
        batch: &mut Batch,
        scratch: &mut ProbeScratch,
        buffer: &mut [MaybeUninit<u8>],
        work_left: &mut u64,
        request: &mut impl FnMut(&Path) -> ProbeRequest,
        cancelled: &mut impl FnMut() -> bool,
    ) {
        // Never perform work that can emit an observation without a free slot.
        if batch.is_full() {
            return;
        }
        match self.visit(
            context, batch, scratch, buffer, work_left, request, cancelled,
        ) {
            WalkStep::Descend(child) => self.stack.push(child),
            WalkStep::Deliver => (),
            WalkStep::Finished | WalkStep::Failed => {
                recycle_frame(&mut self.stack, &mut self.free_frames)
            }
        }
    }
    fn visit<P: MatchPolicy>(
        &mut self,
        context: &ProbeContext<'_, P>,
        batch: &mut Batch,
        scratch: &mut ProbeScratch,
        buffer: &mut [MaybeUninit<u8>],
        work_left: &mut u64,
        request: &mut impl FnMut(&Path) -> ProbeRequest,
        cancelled: &mut impl FnMut() -> bool,
    ) -> WalkStep {
        let Self {
            stack,
            free_frames,
            path,
        } = self;
        *work_left -= 1; // The caller checks the work limit before opening a directory.
        let depth = stack.len();
        let (ancestors, current) = stack.split_at_mut(depth - 1);
        let frame = &mut current[0];
        let (file, evidence) =
            match prepare_directory(frame, ancestors, context, batch, &mut scratch.native) {
                ControlFlow::Continue(opened) => opened,
                ControlFlow::Break(step) => return step,
            };
        let mut entries = RawDir::new(&file, buffer);
        if let ControlFlow::Break(step) =
            replay_prefix(&mut entries, frame, batch, work_left, cancelled)
        {
            return step;
        }
        while !batch.is_full() {
            if cancelled() {
                batch.emit(&frame.lexical, ScanReason::Cancelled, None);
                return WalkStep::Failed;
            }
            if *work_left == 0 {
                batch.emit(&frame.lexical, ScanReason::Limit, None);
                return WalkStep::Failed;
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
                    return WalkStep::Failed;
                }
                None => {
                    match file.metadata() {
                        Err(error) => {
                            batch.emit(&frame.lexical, ScanReason::Coverage, error.raw_os_error());
                            return WalkStep::Failed;
                        }
                        Ok(metadata) if DirectoryEvidence::new(&metadata) != evidence => {
                            batch.emit(&frame.lexical, ScanReason::Resumption, None);
                            return WalkStep::Failed;
                        }
                        Ok(_) => (),
                    }
                    return WalkStep::Finished;
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
            let (observed_regular, should_descend) =
                match classify_entry(context, &file, &entry, path) {
                    Ok(EntryKind::Skip) => continue,
                    Ok(EntryKind::Directory(expected)) => {
                        if depth == context.config.limits.depth {
                            batch.emit(path, ScanReason::Limit, None);
                            continue;
                        }
                        let mut child = take_frame(
                            free_frames,
                            path,
                            &frame.physical,
                            expected,
                            context.config.limits.path_bytes,
                        );
                        child.physical.push(name_path);
                        return WalkStep::Descend(child);
                    }
                    Ok(EntryKind::Candidate {
                        observed_regular,
                        should_descend,
                    }) => (observed_regular, should_descend),
                    Err((reason, errno)) => {
                        batch.emit(path, reason, errno);
                        continue;
                    }
                };
            if context.matcher.excludes(path) {
                batch.emit(path, ScanReason::Excluded, None);
                observe_excluded(
                    &file,
                    name,
                    context.config.symlinks,
                    should_descend,
                    batch.last_mut(),
                );
            } else {
                batch.emit(path, ScanReason::ProbeBusy, None);
                probe(
                    context,
                    &file,
                    name,
                    observed_regular,
                    should_descend,
                    request(path),
                    batch.last_mut(),
                    scratch,
                    cancelled,
                );
                if batch.last_mut().reason == Some(ScanReason::Cancelled) {
                    return WalkStep::Failed;
                }
            }
        }
        WalkStep::Deliver
    }
}
fn prepare_directory<P: MatchPolicy>(
    frame: &mut Frame,
    ancestors: &[Frame],
    context: &ProbeContext<'_, P>,
    batch: &mut Batch,
    native: &mut NativePath,
) -> ControlFlow<WalkStep, (File, DirectoryEvidence)> {
    let mut open_directory = || -> io::Result<File> {
        native.load(&frame.physical)?;
        let flags = OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC | OFlags::NONBLOCK;
        Ok(File::from(
            if context.config.symlinks == SymlinkPolicy::Reject {
                openat2(
                    CWD,
                    native.as_c_str(),
                    flags,
                    Mode::empty(),
                    ResolveFlags::NO_SYMLINKS,
                )?
            } else {
                openat(CWD, native.as_c_str(), flags, Mode::empty())?
            },
        ))
    };
    let file = match open_directory() {
        Ok(file) => file,
        Err(error) => {
            batch.emit(&frame.lexical, ScanReason::Coverage, error.raw_os_error());
            return ControlFlow::Break(WalkStep::Failed);
        }
    };
    let evidence = match file.metadata() {
        Ok(metadata) => DirectoryEvidence::new(&metadata),
        Err(error) => {
            batch.emit(&frame.lexical, ScanReason::Coverage, error.raw_os_error());
            return ControlFlow::Break(WalkStep::Failed);
        }
    };
    if frame
        .expected
        .is_some_and(|expected| expected != evidence.locator)
        || frame
            .evidence
            .is_some_and(|prior| prior.locator != evidence.locator)
    {
        batch.emit(&frame.lexical, ScanReason::Coverage, None);
        return ControlFlow::Break(WalkStep::Failed);
    }
    if frame.evidence.is_none()
        && ancestors.iter().any(|ancestor| {
            ancestor
                .evidence
                .is_some_and(|e| e.locator == evidence.locator)
        })
    {
        batch.emit(&frame.lexical, ScanReason::Cycle, None);
        return ControlFlow::Break(WalkStep::Failed);
    }
    // Prune a known namespace, not a replacement for a directory whose
    // identity this pass already relied on.
    if evidence.locator == context.config.checkpoint_namespace {
        return ControlFlow::Break(WalkStep::Finished);
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
                return ControlFlow::Break(WalkStep::Failed);
            }
        };
        // tmpfs offset implementations vary by kernel. A matching replay
        // prefix alone does not qualify complete coverage on those backends.
        frame.replay = !matches!(
            filesystem,
            nix::sys::statfs::EXT4_SUPER_MAGIC
                | nix::sys::statfs::XFS_SUPER_MAGIC
                | nix::sys::statfs::BTRFS_SUPER_MAGIC
        );
        batch.emit(&frame.lexical, ScanReason::Directory, None);
        batch.last_mut().locator = Some(evidence.locator);
    }
    frame.evidence = Some(evidence);
    // Deliver diagnostics before attempting another operation that can emit one.
    // In particular, mutation plus a failed seek must fit a one-slot batch.
    if batch.is_full() {
        return ControlFlow::Break(WalkStep::Deliver);
    }
    if frame.replay && frame.visited != 0 && !frame.replay_reported {
        batch.emit(&frame.lexical, ScanReason::UnqualifiedResumption, None);
        frame.replay_reported = true;
        if batch.is_full() {
            return ControlFlow::Break(WalkStep::Deliver);
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
        return ControlFlow::Break(WalkStep::Failed);
    }
    ControlFlow::Continue((file, evidence))
}

fn replay_prefix(
    entries: &mut RawDir<'_, &File>,
    frame: &Frame,
    batch: &mut Batch,
    work_left: &mut u64,
    cancelled: &mut impl FnMut() -> bool,
) -> ControlFlow<WalkStep> {
    if frame.replay && frame.visited != 0 {
        let mut seen = 0;
        let mut prefix = [0; 32];
        while seen < frame.visited {
            if cancelled() {
                batch.emit(&frame.lexical, ScanReason::Cancelled, None);
                return ControlFlow::Break(WalkStep::Failed);
            }
            if *work_left == 0 {
                batch.emit(&frame.lexical, ScanReason::Limit, None);
                return ControlFlow::Break(WalkStep::Failed);
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
                    return ControlFlow::Break(WalkStep::Failed);
                }
            }
        }
        if prefix != frame.prefix {
            batch.emit(&frame.lexical, ScanReason::Resumption, None);
            return ControlFlow::Break(WalkStep::Failed);
        }
    }
    ControlFlow::Continue(())
}

enum EntryKind {
    Skip,
    Directory(Option<FileLocator>),
    Candidate {
        observed_regular: bool,
        should_descend: bool,
    },
}

fn classify_entry<P: MatchPolicy>(
    context: &ProbeContext<'_, P>,
    file: &File,
    entry: &rustix::fs::RawDirEntry<'_>,
    path: &Path,
) -> Result<EntryKind, (ScanReason, Option<i32>)> {
    let name = entry.file_name();
    let should_descend =
        context.matcher.may_descend(path) && !context.matcher.excludes_subtree(path);
    let file_type = entry.file_type();
    let mut is_directory = file_type == FileType::Directory;
    let mut observed_regular = file_type == FileType::RegularFile;
    let mut directory_locator = None;
    if should_descend
        && (file_type == FileType::Unknown
            || (file_type == FileType::Symlink && context.config.symlinks == SymlinkPolicy::Follow))
    {
        match statat(
            file,
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
                if is_directory {
                    directory_locator = Some(probe::locator(&stat));
                }
            }
            Err(error) => {
                // A required subtree's kind is unknown, including ENOENT.
                return Err((ScanReason::Coverage, Some(error.raw_os_error())));
            }
        }
    }
    Ok(if is_directory {
        if should_descend {
            EntryKind::Directory(directory_locator)
        } else {
            EntryKind::Skip
        }
    } else if context.matcher.includes(path) {
        EntryKind::Candidate {
            observed_regular,
            should_descend,
        }
    } else {
        EntryKind::Skip
    })
}

fn observe_excluded(
    file: &File,
    name: &std::ffi::CStr,
    symlinks: SymlinkPolicy,
    should_descend: bool,
    out: &mut Observation,
) {
    match statat(
        file,
        name,
        if symlinks == SymlinkPolicy::Follow {
            AtFlags::empty()
        } else {
            AtFlags::SYMLINK_NOFOLLOW
        },
    ) {
        Ok(stat) if FileType::from_raw_mode(stat.st_mode) == FileType::RegularFile => {
            out.locator = Some(probe::locator(&stat));
        }
        Ok(stat) => {
            if should_descend && FileType::from_raw_mode(stat.st_mode) == FileType::Directory {
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
}

fn prefix_hash(prefix: [u8; 32], entry: &rustix::fs::RawDirEntry<'_>) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    let _ = hasher.update(&prefix);
    let _ = hasher.update(&entry.ino().to_le_bytes());
    let _ = hasher.update(&(entry.file_name().to_bytes().len() as u64).to_le_bytes());
    let _ = hasher.update(entry.file_name().to_bytes());
    *hasher.finalize().as_bytes()
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
        let mut walker = Walker::new(config.limits);
        let mut batch = Batch::new(config.limits);
        let mut scratch = ProbeScratch::new(config.limits);
        let mut buffer = vec![MaybeUninit::uninit(); 8192];
        let mut frame = take_frame(
            &mut walker.free_frames,
            dir.path(),
            dir.path(),
            None,
            config.limits.path_bytes,
        );
        let mut previous = DirectoryEvidence::new(&std::fs::metadata(dir.path()).unwrap());
        previous.modified.0 = previous.modified.0.wrapping_sub(1);
        frame.evidence = Some(previous);
        // Native directory seeking rejects a negative offset on the qualified backends.
        frame.cookie = u64::MAX;
        walker.stack.push(frame);
        let budget = ProbeBudget::default();
        let context = ProbeContext {
            config: &config,
            matcher: &NoMatches,
            roots: &[],
            budget: &budget,
            policy_available: true,
        };
        let mut work = 10;
        for attempt in 0..2 {
            walker.step(
                &context,
                &mut batch,
                &mut scratch,
                &mut buffer,
                &mut work,
                &mut |_| ProbeRequest::Full,
                &mut || false,
            );
            assert_eq!(batch.len(), 1);
            assert_eq!(batch[0].reason, Some(ScanReason::Resumption));
            if attempt == 0 {
                assert_eq!(batch[0].errno, None);
                assert_eq!(walker.stack.len(), 1);
            } else {
                assert!(batch[0].errno.is_some());
                assert!(walker.stack.is_empty());
            }
            batch.clear();
        }
    }

    /// Scenario: One-shot cancellation coincides with an exhausted replay work budget.
    /// Guarantees: Cancellation takes precedence and is preserved in the delivered completion marker.
    #[test]
    fn replay_cancellation_precedes_work_limit() {
        let dir = tempfile::tempdir().unwrap();
        let file = File::open(dir.path()).unwrap();
        let mut buffer = vec![MaybeUninit::uninit(); 8192];
        let mut frame = take_frame(&mut Vec::new(), dir.path(), dir.path(), None, 4096);
        frame.replay = true;
        frame.visited = 1;
        let mut batch = Batch::new(ScanLimits {
            batch: 1,
            ..ScanLimits::default()
        });
        let mut calls = 0;
        {
            let mut entries = RawDir::new(&file, &mut buffer);
            assert!(matches!(
                replay_prefix(&mut entries, &frame, &mut batch, &mut 0, &mut || {
                    calls += 1;
                    calls == 1
                }),
                ControlFlow::Break(WalkStep::Failed)
            ));
        }
        assert_eq!(calls, 1);
        assert_eq!(batch[0].reason, Some(ScanReason::Cancelled));
        drop(file);
        let mut end = ScanCompletion {
            generation: 1,
            complete: true,
            cancelled: false,
            observations: 0,
        };
        assert!(!batch::deliver(&mut batch, &mut end, &mut |_| true));
        assert!(end.cancelled && !end.complete);
    }
}
