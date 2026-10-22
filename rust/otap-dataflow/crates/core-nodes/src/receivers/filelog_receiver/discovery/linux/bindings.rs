// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bounded targeted checks sharing one refreshed root context per batch.

use super::*;
use std::fs::File;
use std::os::unix::ffi::OsStrExt;

#[derive(Clone, Copy)]
struct Failure {
    reason: ScanReason,
    errno: Option<i32>,
    scope: BindingScope,
    parent: Option<FileLocator>,
    root: Option<(bool, usize)>,
}
impl Failure {
    fn local(reason: ScanReason, errno: Option<i32>) -> Self {
        Self {
            reason,
            errno,
            scope: BindingScope::Binding,
            parent: None,
            root: None,
        }
    }
    fn coverage(reason: ScanReason, errno: Option<i32>) -> Self {
        Self {
            scope: BindingScope::Coverage,
            ..Self::local(reason, errno)
        }
    }
    fn parent_changed(parent: FileLocator) -> Self {
        Self {
            scope: BindingScope::ParentChanged,
            parent: Some(parent),
            ..Self::local(ScanReason::RootChanged, None)
        }
    }
}

/// One root-context refresh followed by borrowed, descriptor-free binding results.
///
/// Results borrow scanner storage, not the request path. The caller can therefore
/// borrow a tracked path for `check`, then mutate reconciliation using the result
/// without cloning that path. No directory descriptor survives a method call.
pub struct BindingBatch<'a, P, C> {
    scanner: &'a mut Scanner<P>,
    budget: &'a ProbeBudget,
    cancelled: C,
    failure: Option<Failure>,
}

impl<P: MatchPolicy> Scanner<P> {
    /// Refreshes include and exclusion roots once for a bounded sequence of checks.
    /// Detected environment changes remain receiver-wide failures. Like a scan,
    /// this context is not a filesystem snapshot; each parent is checked twice.
    pub fn binding_batch<'a, C: FnMut() -> bool>(
        &'a mut self,
        budget: &'a ProbeBudget,
        mut cancelled: C,
    ) -> BindingBatch<'a, P, C> {
        let failure = self.refresh_binding_roots(&mut cancelled).err();
        BindingBatch {
            scanner: self,
            budget,
            cancelled,
            failure,
        }
    }

    /// Convenience for one binding. Use `binding_batch` to share root work across
    /// requests. The callback runs after all descriptors have closed.
    pub fn check_binding(
        &mut self,
        path: &Path,
        expected_parent: Option<FileLocator>,
        budget: &ProbeBudget,
        cancelled: impl FnMut() -> bool,
        consume: impl FnOnce(BindingCheck<'_>),
    ) -> Result<(), InvalidBindingPath> {
        let mut batch = self.binding_batch(budget, cancelled);
        consume(batch.check(path, expected_parent)?);
        Ok(())
    }

    fn refresh_binding_roots(
        &mut self,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<(), Failure> {
        for (index, root) in self.config.include_roots.iter().enumerate() {
            if cancelled() {
                return Err(Failure::coverage(ScanReason::Cancelled, None));
            }
            let (physical, _, changed) = resolve_include_root(
                root,
                &mut self.target,
                &mut self.scratch.native,
                &mut self.include_baselines[index],
                self.config.checkpoint_namespace,
            )
            .map_err(|(reason, error)| Failure {
                root: Some((false, index)),
                ..Failure::coverage(reason, error.raw_os_error())
            })?;
            if changed {
                return Err(Failure {
                    root: Some((false, index)),
                    ..Failure::coverage(ScanReason::RootChanged, None)
                });
            }
            self.binding_roots[index].clear();
            self.binding_roots[index].push(physical);
        }
        for (index, root) in self.config.exclusion_roots.iter().enumerate() {
            resolve_exclusion(
                root,
                &mut self.target,
                &mut self.scratch.native,
                &mut self.exclusions[index],
                cancelled,
            )
            .map_err(|(reason, error)| Failure {
                root: Some((true, index)),
                ..Failure::coverage(reason, error.raw_os_error())
            })?;
        }
        Ok(())
    }

    fn probe_binding_path(
        &mut self,
        path: &Path,
        expected_parent: Option<FileLocator>,
        budget: &ProbeBudget,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<FileLocator, Failure> {
        if cancelled() {
            return Err(Failure::coverage(ScanReason::Cancelled, None));
        }
        let parent_path = path.parent().expect("validated absolute filename");
        let mut selected = None;
        let mut prefix_len = 0;
        for (index, root) in self.config.include_roots.iter().enumerate() {
            if root.as_os_str().len() >= prefix_len && parent_path.starts_with(root) {
                // Check every lexical ancestor that traversal would visit. Do not
                // reinterpret the traversal stack-depth limit as an exclusion.
                let allowed = parent_path
                    .ancestors()
                    .take_while(|p| p.starts_with(root))
                    .all(|p| walk::may_visit_directory(&self.matcher, p));
                if allowed {
                    selected = Some(index);
                    prefix_len = root.as_os_str().len();
                }
            }
        }
        let Some(index) = selected else {
            // No eligible route to this name; do not open its contents.
            return Err(Failure::local(ScanReason::Excluded, None));
        };
        let suffix = parent_path
            .strip_prefix(&self.config.include_roots[index])
            .expect("selected root");
        let physical = &self.binding_roots[index];
        if physical
            .as_os_str()
            .len()
            .checked_add(suffix.as_os_str().len())
            .and_then(|len| len.checked_add(1))
            .is_none_or(|len| len >= self.config.limits.path_bytes)
        {
            return Err(Failure::local(ScanReason::Limit, None));
        }
        self.binding_parent.clear();
        self.binding_parent.push(physical);
        self.binding_parent.push(suffix);
        let open_parent = |native: &mut native_path::NativePath| {
            native
                .load(&self.binding_parent)
                .map_err(|e| Failure::local(ScanReason::Coverage, e.raw_os_error()))?;
            let resolve = if self.config.symlinks == SymlinkPolicy::Reject {
                ResolveFlags::NO_SYMLINKS
            } else {
                ResolveFlags::empty()
            };
            openat2(
                CWD,
                native.as_c_str(),
                OFlags::PATH | OFlags::DIRECTORY | OFlags::CLOEXEC,
                Mode::empty(),
                resolve,
            )
            .map(File::from)
            .map_err(|e| {
                let reason = if e == rustix::io::Errno::NOENT {
                    ScanReason::Missing
                } else {
                    ScanReason::Coverage
                };
                Failure::local(reason, Some(e.raw_os_error()))
            })
        };
        let directory = open_parent(&mut self.scratch.native)?;
        let parent = FileLocator::from_metadata(
            &directory
                .metadata()
                .map_err(|e| Failure::local(ScanReason::Coverage, e.raw_os_error()))?,
        );
        if expected_parent.is_some_and(|expected| expected != parent) {
            return Err(Failure::parent_changed(parent));
        }
        let context = ProbeContext {
            config: &self.config,
            matcher: &self.matcher,
            roots: &self.exclusions,
            budget,
            policy_available: true,
        };
        let name = path.file_name().expect("validated filename").as_bytes();
        if name.len() > 255 {
            return Err(Failure::local(ScanReason::Limit, None));
        }
        let mut terminated = [0_u8; 256];
        terminated[..name.len()].copy_from_slice(name);
        let name = std::ffi::CStr::from_bytes_with_nul(&terminated[..=name.len()])
            .expect("native filename");
        if cancelled() {
            return Err(Failure::coverage(ScanReason::Cancelled, None));
        }
        if self.matcher.excludes(path) || !self.matcher.includes(path) {
            self.batch.last_mut().reason = Some(ScanReason::Excluded);
            walk::observe_excluded(
                &directory,
                name,
                self.config.symlinks,
                false,
                self.batch.last_mut(),
            );
        } else {
            self.batch.last_mut().reason = Some(ScanReason::ProbeBusy);
            probe::probe(
                &context,
                &directory,
                name,
                false,
                false,
                ProbeRequest::Full,
                self.batch.last_mut(),
                &mut self.scratch,
                cancelled,
            );
        }
        drop(directory);
        if cancelled() {
            return Err(Failure::coverage(ScanReason::Cancelled, None));
        }
        let recheck = open_parent(&mut self.scratch.native)?;
        let current_parent = FileLocator::from_metadata(
            &recheck
                .metadata()
                .map_err(|e| Failure::local(ScanReason::Coverage, e.raw_os_error()))?,
        );
        if current_parent != parent {
            return Err(if expected_parent.is_some() {
                Failure::parent_changed(current_parent)
            } else {
                Failure::local(ScanReason::Unstable, None)
            });
        }
        Ok(parent)
    }
}

impl<P: MatchPolicy, C: FnMut() -> bool> BindingBatch<'_, P, C> {
    /// Checks a complete native filename. The result's lifetime is independent of
    /// `path`; consume it before the next check reuses scanner storage.
    pub fn check(
        &mut self,
        path: &Path,
        expected_parent: Option<FileLocator>,
    ) -> Result<BindingCheck<'_>, InvalidBindingPath> {
        let scanner = &mut *self.scanner;
        if !native_path::valid_binding_path(path, scanner.config.limits.path_bytes) {
            return Err(InvalidBindingPath);
        }
        scanner.batch.clear();
        scanner.batch.emit(path, ScanReason::Coverage, None);
        let result = if (self.cancelled)() {
            Err(Failure::coverage(ScanReason::Cancelled, None))
        } else if let Some(failure) = self.failure {
            Err(failure)
        } else {
            scanner.probe_binding_path(path, expected_parent, self.budget, &mut self.cancelled)
        };
        let (parent, scope, root) = match result {
            Ok(parent) => {
                let scope = match scanner.batch[0].reason {
                    Some(
                        ScanReason::Policy
                        | ScanReason::ExclusionUnavailable
                        | ScanReason::VerificationUnavailable
                        | ScanReason::Cancelled,
                    ) => BindingScope::Coverage,
                    _ => BindingScope::Binding,
                };
                (Some(parent), scope, None)
            }
            Err(failure) => {
                let out = scanner.batch.last_mut();
                out.reason = Some(failure.reason);
                out.errno = failure.errno;
                out.locator = None;
                out.size = None;
                out.modified = None;
                out.fingerprint.clear();
                (failure.parent, failure.scope, failure.root)
            }
        };
        let diagnostic_path = match root {
            Some((true, index)) => scanner.config.exclusion_roots[index].as_path(),
            Some((false, index)) => scanner.config.include_roots[index].as_path(),
            None if scope == BindingScope::ParentChanged => {
                scanner.batch[0].path.parent().expect("binding parent")
            }
            None => &scanner.batch[0].path,
        };
        Ok(BindingCheck {
            observation: &scanner.batch[0],
            parent,
            context_valid: scope == BindingScope::Binding
                && parent.is_some()
                && parent == expected_parent,
            scope,
            diagnostic_path,
        })
    }
}
