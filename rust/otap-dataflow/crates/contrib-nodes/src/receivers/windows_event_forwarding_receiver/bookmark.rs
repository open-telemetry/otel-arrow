// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bounded, memory-only progress for one receiver replica and subscription.
//!
//! This is a synchronous state machine, not downstream feedback integration.
//! Sources are never evicted, even after a failed first delivery. Full capacity
//! rejects new admissions instead of forgetting progress. Recreating the store
//! loses all cursors. Delivery URL versions are deliberately not progress keys.
//! At most one committed and one pending bookmark are retained per source;
//! pending entries also obey the global in-flight bound. Each bookmark's UTF-8
//! length is bounded separately; these are not exact allocator-memory ceilings.

use super::{config::Limits, identity::SourceIdentity, wsman::messages::bookmark::Bookmark};
use std::{collections::HashMap, rc::Rc};
use uuid::Uuid;

/// Whole-batch delivery outcome, never merely successful enqueueing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BatchOutcome {
    /// All records accepted downstream, or a validated marker-only local delivery.
    Ack,
    /// Conversion/enqueue failure, downstream rejection, timeout, or cancellation.
    Nack,
}

/// Correlation token for one admission, scoped to its originating store.
///
/// The runtime must resolve every successful admission on all exit paths.
/// Dropping a ticket does not release capacity or commit progress.
#[derive(Debug)]
#[must_use = "resolve admitted batches with Ack or Nack on every exit path"]
pub struct PendingBatch {
    source: SourceIdentity,
    token: Rc<()>,
}

struct Pending {
    token: Rc<()>,
    bookmark: Option<Bookmark>,
}

#[derive(Default)]
struct SourceProgress {
    bookmark: Option<Bookmark>,
    pending: Option<Pending>,
}

/// Single-core progress state for one stable subscription ID.
///
/// No locks, timers, network, or filesystem operations are performed here.
pub struct BookmarkStore {
    subscription_id: Uuid,
    sources: HashMap<SourceIdentity, SourceProgress>,
    max_sources: usize,
    max_in_flight_batches: usize,
    max_bookmark_bytes: usize,
    in_flight: usize,
}

impl BookmarkStore {
    /// Construct an empty store. The caller must recreate it if subscription identity changes.
    ///
    /// # Errors
    ///
    /// Rejects zero source, in-flight, or bookmark-byte limits.
    pub fn new(subscription_id: Uuid, limits: &Limits) -> Result<Self, String> {
        if limits.max_sources == 0
            || limits.max_in_flight_batches == 0
            || limits.max_bookmark_bytes == 0
        {
            return Err("bookmark store limits must be positive".into());
        }
        Ok(Self {
            subscription_id,
            sources: HashMap::new(),
            max_sources: limits.max_sources,
            max_in_flight_batches: limits.max_in_flight_batches,
            max_bookmark_bytes: limits.max_bookmark_bytes,
            in_flight: 0,
        })
    }

    /// Reserve capacity and retain a candidate without advancing committed progress.
    ///
    /// The caller supplies authenticated source and validated subscription identity,
    /// not untrusted event fields. Every returned ticket must eventually be resolved.
    ///
    /// # Errors
    ///
    /// Rejects a different subscription, oversized bookmark, overlapping source
    /// delivery, or exhausted source/in-flight capacity without mutating state.
    pub fn begin(
        &mut self,
        source: SourceIdentity,
        subscription_id: Uuid,
        bookmark: Option<Bookmark>,
    ) -> Result<PendingBatch, String> {
        if subscription_id != self.subscription_id {
            return Err("bookmark store subscription mismatch".into());
        }
        if bookmark
            .as_ref()
            .is_some_and(|b| b.as_xml().len() > self.max_bookmark_bytes)
        {
            return Err("bookmark exceeds max_bookmark_bytes".into());
        }
        if self.in_flight >= self.max_in_flight_batches {
            return Err("maximum in-flight WEF batches reached".into());
        }
        if let Some(progress) = self.sources.get(&source) {
            if progress.pending.is_some() {
                return Err("source already has a pending WEF batch".into());
            }
        } else if self.sources.len() >= self.max_sources {
            return Err("maximum WEF source count reached".into());
        }
        // Local pointer identity avoids sequence reuse across retries or store recreation.
        // Rc keeps stale tickets alive without introducing cross-core shared state.
        let token = Rc::new(());
        self.sources.entry(source.clone()).or_default().pending = Some(Pending {
            token: Rc::clone(&token),
            bookmark,
        });
        self.in_flight += 1;
        Ok(PendingBatch { source, token })
    }

    /// Apply whole-batch feedback before replying to Windows.
    ///
    /// Only Ack with a candidate replaces progress. Nack and Ack without a
    /// candidate preserve the previous cursor. No cursor values are compared.
    ///
    /// # Errors
    ///
    /// Rejects stale, already resolved, or foreign-store tickets without changing
    /// progress or releasing any other batch's capacity.
    pub fn finish(&mut self, batch: &PendingBatch, outcome: BatchOutcome) -> Result<(), String> {
        let progress = self
            .sources
            .get_mut(&batch.source)
            .ok_or("unknown WEF batch source")?;
        if !progress
            .pending
            .as_ref()
            .is_some_and(|p| Rc::ptr_eq(&p.token, &batch.token))
        {
            return Err("WEF batch is no longer pending in this store".into());
        }
        let pending = progress
            .pending
            .take()
            .expect("pending ticket checked above");
        if outcome == BatchOutcome::Ack
            && let Some(bookmark) = pending.bookmark
        {
            progress.bookmark = Some(bookmark);
        }
        self.in_flight -= 1;
        Ok(())
    }

    /// Return only the committed cursor; a pending candidate is never visible.
    #[must_use]
    pub fn bookmark(&self, source: &SourceIdentity) -> Option<&Bookmark> {
        self.sources.get(source)?.bookmark.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::super::{
        config::AuthConfig,
        wsman::{Envelope, Route, messages::Action},
    };
    use super::*;

    const SUBSCRIPTION: Uuid = Uuid::from_u128(42);

    fn source(name: &str) -> SourceIdentity {
        let cert = rcgen::generate_simple_self_signed(vec![name.into()]).unwrap();
        SourceIdentity::from_verified_certificate(
            cert.cert.der(),
            &AuthConfig {
                allowed_sources: vec![name.into()],
            },
        )
        .unwrap()
    }

    fn candidate(value: &str) -> Bookmark {
        let xml = format!(
            r#"<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope" xmlns:a="http://schemas.xmlsoap.org/ws/2004/08/addressing" xmlns:w="http://schemas.dmtf.org/wbem/wsman/1/wsman.xsd" xmlns:e="http://schemas.xmlsoap.org/ws/2004/08/eventing"><s:Header><a:Action>{}</a:Action><a:MessageID>request</a:MessageID><a:To>https://collector/delivery</a:To><e:Identifier>{SUBSCRIPTION}</e:Identifier><w:AckRequested/><w:Bookmark>{value}</w:Bookmark></s:Header><s:Body><w:Events><w:Event Action="http://schemas.dmtf.org/wbem/wsman/1/wsman/Event">&lt;Event/&gt;</w:Event></w:Events></s:Body></s:Envelope>"#,
            Action::Events.uri()
        );
        Envelope::parse(&xml, xml.len(), 128)
            .unwrap()
            .request(&Route::Delivery { version: "opaque" }, 8192)
            .unwrap()
            .event_batch(&Limits::default())
            .unwrap()
            .bookmark
            .unwrap()
    }

    /// Scenario: deliveries are pending, rejected, acknowledged, or omit their next cursor.
    /// Guarantees: only Ack commits, pending cursors stay hidden, missing candidates preserve progress, and restart loses all state.
    #[test]
    fn commits_only_acknowledged_candidates() {
        let identity = source("host.example.com");
        let mut store = BookmarkStore::new(SUBSCRIPTION, &Limits::default()).unwrap();
        let first = candidate("first");
        let expected = first.as_xml().to_owned();
        let ticket = store
            .begin(identity.clone(), SUBSCRIPTION, Some(first))
            .unwrap();
        assert!(store.bookmark(&identity).is_none());
        store.finish(&ticket, BatchOutcome::Ack).unwrap();
        assert_eq!(store.bookmark(&identity).unwrap().as_xml(), expected);
        for outcome in [BatchOutcome::Nack, BatchOutcome::Ack] {
            let ticket = store.begin(identity.clone(), SUBSCRIPTION, None).unwrap();
            store.finish(&ticket, outcome).unwrap();
            assert_eq!(store.bookmark(&identity).unwrap().as_xml(), expected);
        }
        let rejected = store
            .begin(identity.clone(), SUBSCRIPTION, Some(candidate("second")))
            .unwrap();
        store.finish(&rejected, BatchOutcome::Nack).unwrap();
        assert_eq!(store.bookmark(&identity).unwrap().as_xml(), expected);
        let next = candidate("");
        let expected_next = next.as_xml().to_owned();
        let ticket = store
            .begin(identity.clone(), SUBSCRIPTION, Some(next))
            .unwrap();
        assert_eq!(store.bookmark(&identity).unwrap().as_xml(), expected);
        store.finish(&ticket, BatchOutcome::Ack).unwrap();
        assert_eq!(store.bookmark(&identity).unwrap().as_xml(), expected_next);
        assert!(
            BookmarkStore::new(SUBSCRIPTION, &Limits::default())
                .unwrap()
                .bookmark(&identity)
                .is_none()
        );
    }

    /// Scenario: duplicate, late, and foreign-store feedback races with a new admission for the same source.
    /// Guarantees: tickets resolve only their own admission, never commit another candidate, and never release unrelated capacity.
    #[test]
    fn rejects_stale_and_foreign_tickets() {
        let identity = source("host.example.com");
        let mut store = BookmarkStore::new(SUBSCRIPTION, &Limits::default()).unwrap();
        let old = store.begin(identity.clone(), SUBSCRIPTION, None).unwrap();
        assert!(store.begin(identity.clone(), SUBSCRIPTION, None).is_err());
        store.finish(&old, BatchOutcome::Nack).unwrap();
        assert!(store.finish(&old, BatchOutcome::Ack).is_err());
        let current = store
            .begin(identity.clone(), SUBSCRIPTION, Some(candidate("current")))
            .unwrap();
        assert!(store.finish(&old, BatchOutcome::Ack).is_err());
        let mut other = BookmarkStore::new(SUBSCRIPTION, &Limits::default()).unwrap();
        let other_ticket = other
            .begin(identity.clone(), SUBSCRIPTION, Some(candidate("other")))
            .unwrap();
        assert!(other.finish(&current, BatchOutcome::Ack).is_err());
        assert!(store.finish(&other_ticket, BatchOutcome::Ack).is_err());
        assert_eq!(other.in_flight, 1);
        assert_eq!(store.in_flight, 1);
        assert!(other.bookmark(&identity).is_none());
        assert!(store.bookmark(&identity).is_none());
        store.finish(&current, BatchOutcome::Ack).unwrap();
        other.finish(&other_ticket, BatchOutcome::Nack).unwrap();
        assert_eq!(store.in_flight, 0);
        assert_eq!(other.in_flight, 0);
        assert!(
            store
                .bookmark(&identity)
                .unwrap()
                .as_xml()
                .contains(">current<")
        );
        assert!(other.bookmark(&identity).is_none());
    }

    /// Scenario: bookmark bytes, retained sources, and in-flight capacity reach their limits.
    /// Guarantees: rejected admissions do not mutate state, failed first deliveries retain slots, and existing sources keep progressing.
    #[test]
    fn bounds_retained_state_without_eviction() {
        let exact = candidate("1234");
        let limits = Limits {
            max_sources: 2,
            max_in_flight_batches: 1,
            max_bookmark_bytes: exact.as_xml().len(),
            ..Limits::default()
        };
        let mut store = BookmarkStore::new(SUBSCRIPTION, &limits).unwrap();
        let first = source("first.example.com");
        let second = source("second.example.com");
        let third = source("third.example.com");
        assert!(
            store
                .begin(first.clone(), SUBSCRIPTION, Some(candidate("12345")))
                .is_err()
        );
        assert!(
            store
                .begin(first.clone(), Uuid::from_u128(43), None)
                .is_err()
        );
        assert!(store.sources.is_empty());
        assert_eq!(store.in_flight, 0);
        let ticket = store
            .begin(first.clone(), SUBSCRIPTION, Some(exact))
            .unwrap();
        assert!(store.begin(second.clone(), SUBSCRIPTION, None).is_err());
        assert_eq!(store.sources.len(), 1);
        store.finish(&ticket, BatchOutcome::Ack).unwrap();
        let failed = store.begin(second.clone(), SUBSCRIPTION, None).unwrap();
        store.finish(&failed, BatchOutcome::Nack).unwrap();
        assert_eq!(store.sources.len(), 2);
        assert!(store.begin(third, SUBSCRIPTION, None).is_err());
        let retry = store.begin(second.clone(), SUBSCRIPTION, None).unwrap();
        store.finish(&retry, BatchOutcome::Ack).unwrap();
        assert!(store.bookmark(&second).is_none());
        assert!(store.bookmark(&first).unwrap().as_xml().contains(">1234<"));
        let retry = store.begin(first.clone(), SUBSCRIPTION, None).unwrap();
        store.finish(&retry, BatchOutcome::Ack).unwrap();
        assert_eq!(store.in_flight, 0);
    }

    /// Scenario: two sources have independent candidates and feedback completes in reverse order.
    /// Guarantees: each source commits only its own cursor and capacity is returned per resolution.
    #[test]
    fn isolates_sources_and_subscription_stores() {
        let first = source("first.example.com");
        let second = source("second.example.com");
        let mut store = BookmarkStore::new(SUBSCRIPTION, &Limits::default()).unwrap();
        let a = store
            .begin(first.clone(), SUBSCRIPTION, Some(candidate("a")))
            .unwrap();
        let b = store
            .begin(second.clone(), SUBSCRIPTION, Some(candidate("b")))
            .unwrap();
        assert_eq!(store.in_flight, 2);
        store.finish(&b, BatchOutcome::Ack).unwrap();
        assert_eq!(store.in_flight, 1);
        assert!(store.bookmark(&first).is_none());
        assert!(store.bookmark(&second).unwrap().as_xml().contains(">b<"));
        store.finish(&a, BatchOutcome::Nack).unwrap();
        assert!(store.bookmark(&first).is_none());
        let other = BookmarkStore::new(Uuid::from_u128(43), &Limits::default()).unwrap();
        assert!(other.bookmark(&second).is_none());
        assert_eq!(store.in_flight, 0);
    }

    /// Scenario: store construction receives zero resource limits.
    /// Guarantees: invalid limits fail explicitly before any source state can be admitted.
    #[test]
    fn rejects_zero_store_limits() {
        for limits in [
            Limits {
                max_sources: 0,
                ..Limits::default()
            },
            Limits {
                max_in_flight_batches: 0,
                ..Limits::default()
            },
            Limits {
                max_bookmark_bytes: 0,
                ..Limits::default()
            },
        ] {
            assert!(BookmarkStore::new(SUBSCRIPTION, &limits).is_err());
        }
    }
}
