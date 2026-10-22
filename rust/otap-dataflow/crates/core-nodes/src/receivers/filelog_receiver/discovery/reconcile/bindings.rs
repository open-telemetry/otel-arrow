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
        if tracked.pass.conflict || tracked.pass.rebound != Some(replacement) {
            return Err(ReconcileError::Feedback);
        }
        if !tracked.pass.rebound_resolved {
            tracked.pass.rebound_resolved = true;
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
        self.bindings.remove(id, &self.tracked);
        self.parents.remove(id, &self.tracked);
        let tracked = self.tracked.get_mut(&id).expect("validated identity");
        tracked.file.binding = Some(path);
        tracked.file.role = BindingRole::Live;
        tracked.parent = None;
        tracked.parent_changed = false;
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
        if !tracked.file.active || !tracked.pass.eligible {
            return Err(ReconcileError::Feedback);
        }
        let path = if tracked.pass.binding_eligible {
            tracked
                .file
                .binding
                .as_deref()
                .ok_or(ReconcileError::Feedback)?
        } else {
            &tracked.minimum
        };
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
        if tracked.pass.conflict
            || tracked.pass.rebound != Some(replacement)
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
        if !tracked.pass.rebound_resolved {
            tracked.pass.rebound_resolved = true;
            self.unresolved_rebounds -= 1;
        }
        Ok(())
    }

    /// Enumerates unresolved identity work, including rebounds to resident files
    /// that produce no candidate offer. The global reselection barrier remains.
    pub fn rebounds(&self) -> impl Iterator<Item = Rebound> + '_ {
        self.tracked
            .values()
            .filter(|t| !t.pass.rebound_resolved)
            .filter_map(|t| {
                t.pass.rebound.map(|replacement| Rebound {
                    predecessor: t.file.id,
                    replacement,
                })
            })
    }

    pub(super) fn detect_binding_cycles(&mut self) {
        self.binding_keys.clear();
        self.binding_keys.extend(self.tracked.keys().copied());
        for index in 0..self.binding_keys.len() {
            let start = self.binding_keys[index];
            if self.tracked[&start].pass.graph != Visit::Unseen {
                continue;
            }
            self.binding_walk.clear();
            let mut current = Some(start);
            let mut blocked = false;
            while let Some(id) = current {
                match self.tracked[&id].pass.graph {
                    Visit::Visiting | Visit::Blocked => {
                        blocked = true;
                        break;
                    }
                    Visit::Done => break,
                    Visit::Unseen => (),
                }
                self.tracked.get_mut(&id).expect("graph vertex").pass.graph = Visit::Visiting;
                self.binding_walk.push(id);
                let tracked = &self.tracked[&id];
                if !tracked.file.active || tracked.file.role != BindingRole::Live {
                    current = None;
                    continue;
                }
                let mut next = None;
                if let Some(locator) = tracked.pass.rebound {
                    for target in self.locators.get(&locator, &self.tracked) {
                        let other = &self.tracked[&target];
                        if other.file.active && other.file.role == BindingRole::Live {
                            if next.is_some() {
                                blocked = true;
                                break;
                            }
                            next = Some(target);
                        }
                    }
                }
                if blocked {
                    break;
                }
                current = next.filter(|target| !self.tracked[target].pass.binding_eligible);
            }
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
                if tracked.pass.rebound == Some(new.1)
                    && !tracked.pass.conflict
                    && !tracked.pass.rebound_resolved
                {
                    tracked.pass.rebound_resolved = true;
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
        mut file: TrackedFile,
        commit: impl FnOnce(&Candidate) -> Option<Duration>,
    ) -> Result<bool, ReconcileError> {
        if self.pass.is_some() {
            return Err(ReconcileError::Phase);
        }
        let offer = self
            .candidates
            .offers()
            .find(|offer| offer.token == token)
            .ok_or(ReconcileError::Feedback)?;
        let candidate = offer.candidate;
        let new_identity = !self.tracked.contains_key(&file.id);
        let replacing = match (new_identity, candidate.replacement) {
            (true, Replacement::Possible(id)) => Some(id),
            _ => None,
        };
        if file.locator != candidate.locator {
            return Err(ReconcileError::Tracked);
        }
        if let Some(old) = self.tracked.get(&file.id) {
            if old.file.locator != file.locator
                || old.file.binding.as_ref().map(|p| p.as_os_str())
                    != file.binding.as_ref().map(|p| p.as_os_str())
                || (file.role != old.file.role
                    && !(file.role == BindingRole::Live && old.pass.binding_eligible))
            {
                return Err(ReconcileError::Tracked);
            }
            self.validate_binding(&file, None)?;
        } else {
            self.validate_track(&file, replacing)?;
            if file
                .binding
                .as_ref()
                .is_none_or(|p| p.as_os_str() != candidate.path.as_os_str())
            {
                return Err(ReconcileError::Tracked);
            }
        }
        if let Some(id) = replacing {
            let old = self.tracked.get(&id).ok_or(ReconcileError::Tracked)?;
            if old.pass.conflict || old.pass.rebound != Some(candidate.locator) {
                return Err(ReconcileError::Feedback);
            }
        }
        // Do not retain caller-provided unbounded spare path capacity.
        file.binding = file.binding.as_ref().map(|path| path.to_path_buf());
        // Everything that can reject the transition is checked before the callback.
        let Some(admitted_at) = commit(candidate) else {
            return Ok(false);
        };
        if let Some(old) = self.tracked.get_mut(&file.id) {
            old.file = file;
        } else {
            let id = file.id;
            self.insert_tracked(file);
            if let Some(predecessor) = replacing {
                self.tracked
                    .get_mut(&predecessor)
                    .expect("preflighted predecessor")
                    .file
                    .role = BindingRole::Superseded;
                self.link_successor(predecessor, id);
            }
        }
        self.acknowledge(token, IdentityFeedback::Admitted, admitted_at)
            .expect("exclusive current offer");
        Ok(true)
    }
}
