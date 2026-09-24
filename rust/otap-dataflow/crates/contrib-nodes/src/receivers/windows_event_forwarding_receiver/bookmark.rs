use std::collections::HashMap;

use super::{config::Limits, identity::SourceIdentity};

/// Feedback for the entire WEF batch, not merely successful enqueueing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BatchOutcome {
    /// Every event in the batch was accepted downstream.
    Ack,
    /// Delivery failed or timed out.
    Nack,
}

/// Correlates downstream feedback with a pending source batch.
#[derive(Debug)]
pub struct PendingBatch {
    source: SourceIdentity,
    sequence: u64,
}

#[derive(Default)]
struct SourceProgress {
    bookmark: Option<String>,
    pending: Option<(u64, Option<String>)>,
}

/// Progress owned by one receiver replica for its single configured subscription.
///
/// Create a new store when the subscription identity changes. Delivery URL versions
/// do not participate in progress tracking. No state survives a collector restart.
/// Retained sources are never evicted, including after failed first deliveries.
pub struct BookmarkStore {
    sources: HashMap<SourceIdentity, SourceProgress>,
    max_sources: usize,
    max_in_flight_batches: usize,
    max_bookmark_bytes: usize,
    in_flight: usize,
    next_sequence: u64,
}

impl BookmarkStore {
    /// Construct an empty store using previously validated receiver limits.
    #[must_use]
    pub fn new(limits: &Limits) -> Self {
        Self {
            sources: HashMap::new(),
            max_sources: limits.max_sources,
            max_in_flight_batches: limits.max_in_flight_batches,
            max_bookmark_bytes: limits.max_bookmark_bytes,
            in_flight: 0,
            next_sequence: 0,
        }
    }

    /// Reserve a batch before enqueueing it and retain its candidate bookmark.
    ///
    /// Only authenticated, authorized identities may be supplied. The caller must
    /// resolve every returned ticket, including enqueue failures and timeouts.
    pub fn begin(
        &mut self,
        source: SourceIdentity,
        bookmark: Option<String>,
    ) -> Result<PendingBatch, String> {
        if bookmark
            .as_ref()
            .is_some_and(|value| value.len() > self.max_bookmark_bytes)
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
        let sequence = self.next_sequence;
        let next_sequence = sequence
            .checked_add(1)
            .ok_or_else(|| "WEF batch sequence exhausted".to_owned())?;
        self.sources.entry(source.clone()).or_default().pending = Some((sequence, bookmark));
        self.next_sequence = next_sequence;
        self.in_flight += 1;
        Ok(PendingBatch { source, sequence })
    }

    /// Resolve feedback before replying to Windows; only Ack advances the bookmark.
    ///
    /// Nack covers enqueue failures, downstream rejection, and feedback timeout.
    /// An Ack without a bookmark preserves the previous cursor.
    pub fn finish(&mut self, batch: PendingBatch, outcome: BatchOutcome) -> Result<(), String> {
        let progress = self
            .sources
            .get_mut(&batch.source)
            .ok_or_else(|| "unknown WEF batch source".to_owned())?;
        if progress.pending.as_ref().map(|(sequence, _)| *sequence) != Some(batch.sequence) {
            return Err("WEF batch is no longer pending".into());
        }
        let (_, bookmark) = progress
            .pending
            .take()
            .expect("pending batch checked above");
        if outcome == BatchOutcome::Ack
            && let Some(bookmark) = bookmark
        {
            progress.bookmark = Some(bookmark);
        }
        self.in_flight -= 1;
        Ok(())
    }

    /// Return only the last acknowledged bookmark for subscription enumeration.
    #[must_use]
    pub fn bookmark(&self, source: &SourceIdentity) -> Option<&str> {
        self.sources.get(source)?.bookmark.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::receivers::windows_event_forwarding_receiver::config::AuthConfig;
    use rcgen::{CertificateParams, KeyPair};

    fn source(name: &str) -> SourceIdentity {
        let certificate = CertificateParams::new(vec![name.to_owned()])
            .unwrap()
            .self_signed(&KeyPair::generate().unwrap())
            .unwrap();
        SourceIdentity::from_verified_certificate(
            certificate.der(),
            &AuthConfig {
                allowed_sources: vec![name.to_owned()],
            },
        )
        .unwrap()
    }

    /// Scenario: batches are pending, acknowledged, rejected, or lack a new bookmark.
    /// Guarantees: only Ack commits a cursor; failed delivery and missing cursors preserve it.
    #[test]
    fn bookmark_advances_only_on_ack() {
        let mut store = BookmarkStore::new(&Limits::default());
        let identity = source("host.example.com");
        let first = store.begin(identity.clone(), Some("first".into())).unwrap();
        assert_eq!(store.bookmark(&identity), None);
        store.finish(first, BatchOutcome::Ack).unwrap();
        assert_eq!(store.bookmark(&identity), Some("first"));
        let second = store
            .begin(identity.clone(), Some("second".into()))
            .unwrap();
        assert_eq!(store.bookmark(&identity), Some("first"));
        store.finish(second, BatchOutcome::Nack).unwrap();
        assert_eq!(store.bookmark(&identity), Some("first"));
        let without_bookmark = store.begin(identity.clone(), None).unwrap();
        store.finish(without_bookmark, BatchOutcome::Ack).unwrap();
        assert_eq!(store.bookmark(&identity), Some("first"));
        let retry = store
            .begin(identity.clone(), Some("second".into()))
            .unwrap();
        store.finish(retry, BatchOutcome::Ack).unwrap();
        assert_eq!(store.bookmark(&identity), Some("second"));
        assert_eq!(
            BookmarkStore::new(&Limits::default()).bookmark(&identity),
            None
        );
    }

    /// Scenario: a source sends overlapping batches and late feedback arrives after timeout.
    /// Guarantees: only one batch is pending per source and stale feedback cannot commit it.
    #[test]
    fn serializes_source_and_rejects_stale_feedback() {
        let mut store = BookmarkStore::new(&Limits::default());
        let identity = source("host.example.com");
        let first = store.begin(identity.clone(), Some("first".into())).unwrap();
        let stale = PendingBatch {
            source: identity.clone(),
            sequence: first.sequence,
        };
        assert!(store.begin(identity.clone(), None).is_err());
        store.finish(first, BatchOutcome::Nack).unwrap();
        let retry = store.begin(identity.clone(), Some("retry".into())).unwrap();
        assert!(store.finish(stale, BatchOutcome::Ack).is_err());
        assert_eq!(store.bookmark(&identity), None);
        store.finish(retry, BatchOutcome::Ack).unwrap();
        assert_eq!(store.bookmark(&identity), Some("retry"));
        assert_eq!(store.in_flight, 0);
    }

    /// Scenario: deliveries reach the configured bookmark, source, and in-flight limits.
    /// Guarantees: oversized or excess work is rejected without losing retained progress.
    #[test]
    fn bounds_state_and_releases_batch_capacity() {
        let limits = Limits {
            max_sources: 2,
            max_in_flight_batches: 1,
            max_bookmark_bytes: 4,
            ..Limits::default()
        };
        let mut store = BookmarkStore::new(&limits);
        let first_source = source("first.example.com");
        let second_source = source("second.example.com");
        let third_source = source("third.example.com");
        assert!(
            store
                .begin(first_source.clone(), Some("12345".into()))
                .is_err()
        );
        assert!(store.sources.is_empty());
        let first = store
            .begin(first_source.clone(), Some("1234".into()))
            .unwrap();
        assert!(store.begin(second_source.clone(), None).is_err());
        assert_eq!(store.sources.len(), 1);
        store.finish(first, BatchOutcome::Ack).unwrap();
        let second = store
            .begin(second_source.clone(), Some("next".into()))
            .unwrap();
        store.finish(second, BatchOutcome::Nack).unwrap();
        assert!(store.begin(third_source, None).is_err());
        assert_eq!(store.bookmark(&first_source), Some("1234"));
        assert_eq!(store.bookmark(&second_source), None);
        let retry = store.begin(first_source, None).unwrap();
        store.finish(retry, BatchOutcome::Ack).unwrap();
    }
}
