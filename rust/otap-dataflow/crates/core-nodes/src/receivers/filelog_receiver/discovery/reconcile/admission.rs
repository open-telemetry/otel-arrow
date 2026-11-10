// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! One bounded preflight/durability/publication boundary. No checkpoint ownership.
use super::*;
use std::collections::HashSet;

/// One already-selected identity decision for a current offer.
#[derive(Debug)]
pub struct AdmissionChange {
    /// Current, unacknowledged candidate token.
    pub token: OfferToken,
    /// Resulting reader/identity view; its ID must be lifetime-unique.
    pub file: TrackedFile,
    /// Retire this old ID only for a same-locator logical-identity replacement.
    /// Ordinary move/create retains its predecessor and leaves this field empty.
    pub replace_id: Option<TrackedId>,
}

/// Preflight errors occur before the durable callback. Commit errors preserve
/// tracking but may represent uncertain persistence; the caller must stop on
/// uncertainty, never continue reconciliation as though nothing was written.
#[derive(Debug, thiserror::Error)]
pub enum AdmissionError<E> {
    /// The group was rejected without invoking its durable callback.
    #[error(transparent)]
    Preflight(#[from] ReconcileError),
    /// The durable owner failed. The owner decides whether this is terminal.
    #[error("identity commit failed: {0}")]
    Commit(#[source] E),
}

struct Publication {
    replacing: Option<TrackedId>,
    parent: Option<FileLocator>,
    parent_changed: bool,
}

impl Reconciler {
    /// Normal path for grouped admissions and same-locator identity replacement.
    /// Preflight every member, commit the exact group, then publish all members.
    /// [`Self::admit`] delegates here for one admission without identity retirement.
    /// The exclusive borrow freezes offers and bindings throughout the callback.
    /// None means known-no-write deferral with the durable owner still running.
    /// A stopped durable owner must return an error. A recoverable pending write must be
    /// resolved inside the callback; an uncertain terminal error requires stopping
    /// this reconciler. Callback panic must also terminate its owner.
    ///
    /// Same-locator replacement is net-zero in tracked population. It preserves
    /// the old view on every rejected/deferred/failed commit, and carries directory
    /// baselines forward without carrying the retired identity's successor state.
    pub fn admit_batch<E>(
        &mut self,
        mut changes: Vec<AdmissionChange>,
        commit: impl FnOnce(&[&Candidate]) -> Result<Option<Duration>, E>,
    ) -> Result<bool, AdmissionError<E>> {
        if self.pass.is_some() {
            return Err(ReconcileError::Phase.into());
        }
        if changes.is_empty() || changes.len() > self.limits.offers {
            return Err(ReconcileError::Limits.into());
        }
        let mut tokens = HashSet::with_capacity(changes.len());
        let mut ids = HashSet::with_capacity(changes.len());
        let mut locators = HashSet::with_capacity(changes.len());
        let mut retired = HashSet::with_capacity(changes.len());
        let mut added = 0;
        for change in &mut changes {
            if !tokens.insert(change.token)
                || !ids.insert(change.file.id)
                || !locators.insert(change.file.locator)
            {
                return Err(ReconcileError::Tracked.into());
            }
            let candidate = self.candidates.offer(change.token)?.candidate;
            if candidate.locator != change.file.locator {
                return Err(ReconcileError::Tracked.into());
            }
            let new_id = !self.tracked.contains_key(&change.file.id);
            // A different alias cannot bypass same-locator replacement checks.
            // Reject before capacity admission or any durable work, regardless of
            // the existing record's role or availability of its saved binding.
            if new_id
                && self
                    .locators
                    .records(&change.file.locator, &self.tracked)
                    .any(|old| old.file.active && Some(old.file.id) != change.replace_id)
            {
                return Err(ReconcileError::Feedback.into());
            }
            added += usize::from(new_id);
            if let Some(old_id) = change.replace_id {
                let old = self.tracked.get(&old_id).ok_or(ReconcileError::Tracked)?;
                if !new_id
                    || old_id == change.file.id
                    || !retired.insert(old_id)
                    || !old.file.active
                    || old.file.resident
                    || old.file.locator != candidate.locator
                    || old.binding_blocked()
                    || old.pass.conflict
                {
                    return Err(ReconcileError::Tracked.into());
                }
            }
            // Bound caller-owned spare capacity before the durable callback.
            change.file.binding = change.file.binding.as_ref().map(|p| p.to_path_buf());
        }
        if retired.iter().any(|id| ids.contains(id)) {
            return Err(ReconcileError::Tracked.into());
        }
        let resulting = self
            .tracked
            .len()
            .checked_sub(retired.len())
            .and_then(|n| n.checked_add(added))
            .ok_or(ReconcileError::Capacity)?;
        if resulting > self.limits.tracked {
            return Err(ReconcileError::Capacity.into());
        }

        let mut publications = Vec::with_capacity(changes.len());
        let mut superseded = HashSet::with_capacity(changes.len());
        for change in &changes {
            let candidate = self.candidates.offer(change.token)?.candidate;
            let file = &change.file;
            let mut replacing = None;
            if let Some(old) = self.tracked.get(&file.id) {
                let role_allowed = file.role == old.file.role
                    || (old.file.role == BindingRole::Unverified
                        && file.role == BindingRole::Live
                        && old.pass.binding_evidence.is_full()
                        && !old.binding_blocked());
                if old.file.locator != file.locator
                    || old.file.binding.as_ref().map(|p| p.as_os_str())
                        != file.binding.as_ref().map(|p| p.as_os_str())
                    || !role_allowed
                {
                    return Err(ReconcileError::Tracked.into());
                }
                self.validate_binding(file, None)?;
            } else {
                if self
                    .bindings
                    .records(candidate.path.as_os_str(), &self.tracked)
                    .any(|old| old.file.active && old.binding_blocked())
                {
                    return Err(ReconcileError::Feedback.into());
                }
                if file
                    .binding
                    .as_ref()
                    .is_none_or(|p| p.as_os_str() != candidate.path.as_os_str())
                {
                    return Err(ReconcileError::Tracked.into());
                }
                let owner = self
                    .bindings
                    .records(candidate.path.as_os_str(), &self.tracked)
                    .find(|old| old.file.active && old.file.role == BindingRole::Live)
                    .map(|old| old.file.id);
                if let Some(owner) = owner
                    && Some(owner) != change.replace_id
                {
                    // Cross-dependent retirement/rotation groups must be split
                    // before preflight, never guessed during publication.
                    if change.replace_id.is_some()
                        || retired.contains(&owner)
                        || !self.tracked[&owner].allows_replacement(candidate.locator)
                    {
                        return Err(ReconcileError::Feedback.into());
                    }
                    replacing = Some(owner);
                    if !superseded.insert(owner) {
                        return Err(ReconcileError::Feedback.into());
                    }
                }
                self.validate_binding(file, change.replace_id.or(replacing))?;
            }
            let (parent, parent_changed) = self.parent_baseline(file.binding.as_deref());
            publications.push(Publication {
                replacing,
                parent,
                parent_changed,
            });
        }
        // The group may reconnect a predecessor while admitting its successor.
        // Check the resulting Live claims, not that predecessor's historical path.
        let mut paths = HashSet::with_capacity(changes.len());
        for change in &changes {
            if change.file.active
                && change.file.role == BindingRole::Live
                && !superseded.contains(&change.file.id)
                && let Some(path) = &change.file.binding
                && !paths.insert(path.as_os_str())
            {
                return Err(ReconcileError::Tracked.into());
            }
        }
        let candidates: Vec<_> = changes
            .iter()
            .map(|change| {
                self.candidates
                    .offer(change.token)
                    .map(|offer| offer.candidate)
            })
            .collect::<Result<_, _>>()?;
        let Some(admitted_at) = commit(&candidates).map_err(AdmissionError::Commit)? else {
            return Ok(false);
        };
        drop(candidates);
        drop(paths);

        // No fallible protocol validation remains after durable success.
        for id in &retired {
            let _ = self.untrack(*id).expect("preflighted retirement");
        }
        for (change, publication) in changes.iter_mut().zip(&publications) {
            let file = TrackedFile {
                id: change.file.id,
                locator: change.file.locator,
                binding: change.file.binding.take(),
                role: change.file.role,
                active: change.file.active,
                resident: change.file.resident,
            };
            if let Some(old) = self.tracked.get_mut(&file.id) {
                old.file = file;
            } else {
                let id = file.id;
                self.insert_tracked(file);
                let new = self.tracked.get_mut(&id).expect("inserted admission");
                new.parent = publication.parent;
                new.parent_changed = publication.parent_changed;
            }
        }
        // Existing rotation predecessors memoize the live successor's ID and
        // locator. Identity replacement changes only that ID. Retarget those
        // links after successful publication so later scans keep rebounds resolved.
        // Scan once per replacement group; ordinary admissions need no extra scan.
        if !retired.is_empty() {
            for tracked in self.tracked.values_mut() {
                if let Some((id, locator)) = tracked.successor
                    && retired.contains(&id)
                {
                    let replacement = changes
                        .iter()
                        .find(|change| change.replace_id == Some(id))
                        .expect("preflighted retirement");
                    debug_assert_eq!(locator, replacement.file.locator);
                    tracked.successor = Some((replacement.file.id, locator));
                }
            }
        }
        for (change, publication) in changes.iter().zip(publications) {
            if let Some(old_id) = publication.replacing {
                self.tracked
                    .get_mut(&old_id)
                    .expect("preflighted predecessor")
                    .file
                    .role = BindingRole::Superseded;
                self.link_successor(old_id, change.file.id);
            }
            self.acknowledge(change.token, IdentityFeedback::Admitted, admitted_at)
                .expect("exclusive preflighted token");
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests;
