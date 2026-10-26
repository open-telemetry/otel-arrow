// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Binding graph and identity-owner commit boundary.

use super::*;

impl Reconciler {
    /// Marks a prior rebound resolved only after the identity owner has handled its identity.
    /// This authorizes no path publication by itself. Call before `rebind`.
    pub fn resolve_rebound(
        &mut self,
        generation: u64,
        id: TrackedId,
        replacement: FileLocator,
    ) -> Result<(), ReconcileError> {
        self.check_finished(generation)?;
        let tracked = self.tracked.get_mut(&id).ok_or(ReconcileError::Tracked)?;
        if tracked.pass.conflict || tracked.pass.binding_target.rebound() != Some(replacement) {
            return Err(ReconcileError::Feedback);
        }
        if tracked.pass.binding_target.resolve() {
            self.unresolved_rebounds -= 1;
        }
        Ok(())
    }

    /// Publishes a complete-pass deterministic binding after all prior rebounds
    /// are resolved. The owner must perform its durable update before calling this
    /// method's callback. The exclusive borrow prevents a new pass or feedback
    /// from invalidating the proposal during the commit. False defers publication.
    pub fn rebind(
        &mut self,
        generation: u64,
        id: TrackedId,
        commit: impl FnOnce(&Path) -> bool,
    ) -> Result<bool, ReconcileError> {
        let path = self.binding_proposal(generation, id)?.to_owned();
        if !commit(&path) {
            return Ok(false);
        }
        let tracked = self.tracked.get_mut(&id).expect("validated identity");
        if tracked
            .file
            .binding
            .as_ref()
            .is_some_and(|old| old.as_os_str() == path.as_os_str())
        {
            tracked.file.role = BindingRole::Live;
            tracked.successor = None;
            return Ok(true);
        }
        let (parent, parent_changed) = self.parent_baseline(Some(&path));
        self.aliases.remove(id, &self.tracked);
        self.bindings.remove(id, &self.tracked);
        self.parents.remove(id, &self.tracked);
        let tracked = self.tracked.get_mut(&id).expect("validated identity");
        tracked.file.binding = Some(path);
        tracked.file.role = BindingRole::Live;
        tracked.parent = parent;
        tracked.parent_changed = parent_changed;
        // The published alias is now the binding, not a context-free hint.
        tracked.minimum.clear();
        tracked.pass.minimum_evidence = NameEvidence::None;
        tracked.pass.binding_evidence = NameEvidence::Full;
        tracked.successor = None;
        if tracked.previous_alias.as_os_str()
            == tracked
                .file
                .binding
                .as_ref()
                .expect("published binding")
                .as_os_str()
        {
            tracked.previous_alias.clear();
        }
        self.bindings.insert(id, &self.tracked);
        self.parents.insert(id, &self.tracked);
        Ok(true)
    }

    /// Reads a binding proposal without changing it. No proposal exists for an
    /// incomplete pass, unresolved rebound, conflict, or competing live binding.
    pub fn binding_proposal(
        &self,
        generation: u64,
        id: TrackedId,
    ) -> Result<&Path, ReconcileError> {
        self.check_finished(generation)?;
        if !self.last.is_some_and(|last| last.complete) || self.unresolved_rebounds != 0 {
            return Err(ReconcileError::Feedback);
        }
        let tracked = self.tracked.get(&id).ok_or(ReconcileError::Tracked)?;
        if !tracked.file.active || !tracked.pass.name_evidence().is_full() {
            return Err(ReconcileError::Feedback);
        }
        // Keep replacement context at the old path until its observed target
        // can enter normal identity reconciliation. Observation alone grants
        // no rebound authority and does not block unrelated binding proposals.
        if matches!(tracked.pass.binding_target, BindingTarget::Observed(_)) {
            return Err(ReconcileError::Feedback);
        }
        // Weak evidence preserves an established Live binding, but cannot be
        // returned as publication authority or displaced by a full alias.
        if tracked.file.role == BindingRole::Live
            && tracked.pass.binding_evidence == NameEvidence::PolicyQualified
        {
            return Err(ReconcileError::Feedback);
        }
        let path = if tracked.pass.binding_evidence.is_full() {
            tracked
                .file
                .binding
                .as_deref()
                .ok_or(ReconcileError::Feedback)?
        } else {
            if !tracked.pass.minimum_evidence.is_full() {
                return Err(ReconcileError::Feedback);
            }
            &tracked.minimum
        };
        if !super::super::native_path::valid_binding_path(path, self.limits.path_bytes) {
            return Err(ReconcileError::Feedback);
        }
        if self
            .bindings
            .records(path.as_os_str(), &self.tracked)
            .any(|other| {
                other.file.id != id && other.file.active && other.file.role == BindingRole::Live
            })
        {
            return Err(ReconcileError::Feedback);
        }
        Ok(path)
    }

    /// Preserves the predecessor's historical path after successful uncontested
    /// replacement admission, including on an incomplete pass. No offset changes.
    pub fn supersede(
        &mut self,
        generation: u64,
        predecessor: TrackedId,
        replacement: FileLocator,
    ) -> Result<(), ReconcileError> {
        self.check_finished(generation)?;
        let tracked = self
            .tracked
            .get(&predecessor)
            .ok_or(ReconcileError::Tracked)?;
        if !tracked.allows_replacement(replacement)
            || self.locators.get(&replacement, &self.tracked).any(|id| {
                self.tracked[&id].file.active && self.tracked[&id].file.role == BindingRole::Live
            })
        {
            return Err(ReconcileError::Feedback);
        }
        let tracked = self
            .tracked
            .get_mut(&predecessor)
            .expect("validated predecessor");
        tracked.file.role = BindingRole::Superseded;
        tracked.successor = None;
        if tracked.pass.binding_target.resolve() {
            self.unresolved_rebounds -= 1;
        }
        Ok(())
    }

    /// Enumerates unresolved identity work, including rebounds to resident files
    /// that produce no candidate offer. The global reselection barrier remains.
    pub fn rebounds(&self) -> impl Iterator<Item = Rebound> + '_ {
        self.tracked.values().filter_map(|t| {
            t.pass.binding_target.pending().map(|replacement| Rebound {
                predecessor: t.file.id,
                replacement,
            })
        })
    }

    // A live record has at most one current rebound target. Multiple live owners
    // of that locator are ambiguous; a validated target binding terminates a chain.
    fn rebound_target(
        tracked: &Records,
        locators: &RecordIndex<FileLocator>,
        id: TrackedId,
    ) -> Result<Option<TrackedId>, ()> {
        let record = &tracked[&id];
        if !record.file.active || record.file.role != BindingRole::Live {
            return Ok(None);
        }
        let mut next = None;
        if let Some(locator) = record.pass.binding_target.rebound() {
            for target in locators.get(&locator, tracked) {
                let other = &tracked[&target];
                if other.file.active && other.file.role == BindingRole::Live {
                    if next.is_some() {
                        return Err(());
                    }
                    next = Some(target);
                }
            }
        }
        Ok(next.filter(|target| !tracked[target].pass.binding_evidence.is_named()))
    }

    // Shared traversal for immediate edge checks and the completion-time graph
    // sweep. A path has at most tracked.len() vertices and reuses bounded scratch.
    fn trace_binding_chain(
        tracked: &mut Records,
        locators: &RecordIndex<FileLocator>,
        walk: &mut Vec<TrackedId>,
        start: TrackedId,
    ) -> bool {
        walk.clear();
        let mut current = Some(start);
        let mut blocked = false;
        while let Some(id) = current {
            match tracked[&id].pass.graph {
                Visit::Visiting | Visit::Blocked => {
                    blocked = true;
                    break;
                }
                Visit::Done => break,
                Visit::Unseen => (),
            }
            tracked.get_mut(&id).expect("graph vertex").pass.graph = Visit::Visiting;
            walk.push(id);
            match Self::rebound_target(tracked, locators, id) {
                Ok(next) => current = next,
                Err(()) => {
                    blocked = true;
                    break;
                }
            }
        }
        blocked
    }

    // Union by rank with path halving, stored in already-bounded tracked records.
    // begin initializes every parent to its own ID; records cannot move in/out
    // of the logical graph during a pass. Rank is bounded by log2(population).
    fn rebound_component(tracked: &mut Records, mut id: TrackedId) -> TrackedId {
        loop {
            let parent = TrackedId(tracked[&id].pass.rebound_parent);
            if parent == id {
                return id;
            }
            let grandparent = tracked[&parent].pass.rebound_parent;
            tracked
                .get_mut(&id)
                .expect("component member")
                .pass
                .rebound_parent = grandparent;
            id = parent;
        }
    }

    // Every uncontested vertex adds at most one outgoing edge. A directed cycle
    // can therefore form only when the new endpoints are already connected.
    // This avoids repeatedly walking growing acyclic chains. Changed conflicting
    // edges already reported failure; the final sweep still checks the full graph.
    pub(super) fn detect_new_rebound(
        tracked: &mut Records,
        locators: &RecordIndex<FileLocator>,
        walk: &mut Vec<TrackedId>,
        start: TrackedId,
    ) -> bool {
        if tracked[&start].pass.conflict {
            return false;
        }
        let target = match Self::rebound_target(tracked, locators, start) {
            Ok(Some(target)) => target,
            Ok(None) => return false,
            Err(()) => {
                tracked.get_mut(&start).expect("graph vertex").pass.conflict = true;
                return true;
            }
        };
        if tracked[&target].pass.conflict {
            return false;
        }
        let mut left = Self::rebound_component(tracked, start);
        let mut right = Self::rebound_component(tracked, target);
        if left != right {
            if tracked[&left].pass.rebound_rank > tracked[&right].pass.rebound_rank {
                std::mem::swap(&mut left, &mut right);
            }
            let equal_rank = tracked[&left].pass.rebound_rank == tracked[&right].pass.rebound_rank;
            tracked
                .get_mut(&left)
                .expect("component root")
                .pass
                .rebound_parent = right.0;
            if equal_rank {
                tracked
                    .get_mut(&right)
                    .expect("component root")
                    .pass
                    .rebound_rank += 1;
            }
            return false;
        }
        // Confirm against current directed bindings and mark affected records.
        // Reset scratch marks because later observations can add more edges.
        let blocked = Self::trace_binding_chain(tracked, locators, walk, start);
        for id in walk.drain(..) {
            let record = tracked.get_mut(&id).expect("visited graph vertex");
            record.pass.graph = Visit::Unseen;
            record.pass.conflict |= blocked;
        }
        blocked
    }

    pub(super) fn detect_binding_cycles(&mut self) {
        self.binding_keys.clear();
        self.binding_keys.extend(self.tracked.keys().copied());
        for index in 0..self.binding_keys.len() {
            let start = self.binding_keys[index];
            if self.tracked[&start].pass.graph != Visit::Unseen {
                continue;
            }
            let blocked = Self::trace_binding_chain(
                &mut self.tracked,
                &self.locators,
                &mut self.binding_walk,
                start,
            );
            for id in self.binding_walk.drain(..) {
                let tracked = self.tracked.get_mut(&id).expect("visited graph vertex");
                tracked.pass.graph = if blocked { Visit::Blocked } else { Visit::Done };
                tracked.pass.conflict |= blocked;
            }
        }
    }

    // The only successor-creation path. Existing links are already flattened;
    // redirecting direct ancestors therefore maintains the transitive memo.
    pub(super) fn link_successor(&mut self, predecessor: TrackedId, successor: TrackedId) {
        let old = (predecessor, self.tracked[&predecessor].file.locator);
        let new = (successor, self.tracked[&successor].file.locator);
        for tracked in self.tracked.values_mut() {
            if tracked.file.id == predecessor || tracked.successor == Some(old) {
                tracked.successor = Some(new);
                if tracked.pass.binding_target.rebound() == Some(new.1)
                    && !tracked.pass.conflict
                    && tracked.pass.binding_target.resolve()
                {
                    self.unresolved_rebounds -= 1;
                }
            }
        }
    }

    /// Preflights one admission before invoking external identity/durability work.
    /// On success, successor publication, tracked state and acknowledgement change
    /// together. An existing ID reconnects its own state without a new tracked slot.
    /// The callback returns its monotonic completion time, or None to defer with no
    /// state changes. Ambiguous durable writes must stop the owner, never return None.
    pub fn admit(
        &mut self,
        token: OfferToken,
        file: TrackedFile,
        commit: impl FnOnce(&Candidate) -> Option<Duration>,
    ) -> Result<bool, ReconcileError> {
        let changes = vec![AdmissionChange {
            token,
            file,
            replace_id: None,
        }];
        match self.admit_batch(changes, |candidates| {
            Ok::<_, std::convert::Infallible>(commit(candidates[0]))
        }) {
            Ok(done) => Ok(done),
            Err(AdmissionError::Preflight(error)) => Err(error),
            Err(AdmissionError::Commit(never)) => match never {},
        }
    }
}
