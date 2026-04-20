// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Internal log tap with bounded in-memory retention.

use crate::event::LogEvent;
use otap_df_config::settings::telemetry::logs::InternalLogTapConfig;
use parking_lot::RwLock;
use std::collections::VecDeque;
use std::mem::size_of;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::broadcast;

/// Capacity of the per-handle subscriber broadcast channel.
///
/// When a subscriber falls more than this many events behind the producer,
/// the oldest unread events are dropped for that subscriber and a `Lagged`
/// error is returned on the next receive.  The sender (i.e., the `record()`
/// hot-path) is never blocked regardless of subscriber speed.
const SUBSCRIBER_CHANNEL_CAPACITY: usize = 512;

/// Shared counter for logs dropped before they reached the retained log sink.
#[derive(Clone, Debug, Default)]
pub struct InternalLogTapDropCounter {
    count: Arc<AtomicU64>,
}

impl InternalLogTapDropCounter {
    /// Record a dropped log event.
    pub fn increment(&self) {
        let _ = self.count.fetch_add(1, Ordering::Relaxed);
    }

    fn load(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }
}

/// A retained log event stored in the in-memory ring.
#[derive(Debug, Clone)]
pub struct RetainedLogEvent {
    /// Monotonic sequence number assigned at ingest time.
    pub seq: u64,
    /// Structured log event.
    pub event: LogEvent,
}

#[derive(Debug)]
struct RetainedLogSlot {
    event: RetainedLogEvent,
    estimated_bytes: usize,
}

#[derive(Debug)]
struct LogRetention {
    entries: VecDeque<RetainedLogSlot>,
    retained_bytes: usize,
    max_entries: usize,
    max_bytes: usize,
    dropped_on_retention: u64,
    next_seq: u64,
}

impl LogRetention {
    fn new(max_entries: usize, max_bytes: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(max_entries),
            retained_bytes: 0,
            max_entries,
            max_bytes,
            dropped_on_retention: 0,
            next_seq: 1,
        }
    }

    fn next_seq(&mut self) -> u64 {
        let seq = self.next_seq;
        self.next_seq = self.next_seq.saturating_add(1);
        seq
    }

    /// Push a new retained log event into the ring buffer.
    fn push(&mut self, retained: RetainedLogEvent, estimated_bytes: usize) {
        let slot = RetainedLogSlot {
            event: retained,
            estimated_bytes,
        };
        self.retained_bytes = self.retained_bytes.saturating_add(slot.estimated_bytes);
        self.entries.push_back(slot);
        self.enforce_limits();
    }

    fn enforce_limits(&mut self) {
        while self.entries.len() > self.max_entries || self.retained_bytes > self.max_bytes {
            let Some(oldest) = self.entries.pop_front() else {
                break;
            };
            self.retained_bytes = self.retained_bytes.saturating_sub(oldest.estimated_bytes);
            self.dropped_on_retention = self.dropped_on_retention.saturating_add(1);
        }
    }
}

/// Query parameters for retained internal logs.
#[derive(Debug, Clone, Copy)]
pub struct LogQuery {
    /// Return entries strictly newer than this sequence number.
    pub after: Option<u64>,
    /// Maximum number of entries to return.
    pub limit: usize,
}

/// Snapshot of retained internal logs for admin consumers.
#[derive(Debug, Clone)]
pub struct LogQueryResult {
    /// Oldest retained sequence number, if any.
    pub oldest_seq: Option<u64>,
    /// Newest retained sequence number, if any.
    pub newest_seq: Option<u64>,
    /// Cursor to use as the next `after` value for incremental fetches.
    pub next_seq: u64,
    /// Indicates that `after` predates the retained window.
    pub truncated_before_seq: Option<u64>,
    /// Number of logs dropped before they entered retention.
    pub dropped_on_ingest: u64,
    /// Number of retained logs evicted due to retention limits.
    pub dropped_on_retention: u64,
    /// Current retained memory estimate.
    pub retained_bytes: usize,
    /// Retained entries matching the query.
    pub logs: Vec<RetainedLogEvent>,
}

/// Shared query/update handle for the retained internal log ring buffer.
///
/// Cloning the handle is cheap: all clones share the same ring buffer,
/// drop counter, and subscriber broadcast channel.
#[derive(Clone, Debug)]
pub struct InternalLogTapHandle {
    state: Arc<RwLock<LogRetention>>,
    dropped_on_ingest: InternalLogTapDropCounter,
    /// Broadcast sender for live log events.  Each call to `subscribe()`
    /// returns a new receiver.  Sending never blocks; lagging receivers
    /// automatically drop the oldest buffered events.
    subscribers: broadcast::Sender<Arc<RetainedLogEvent>>,
}

impl InternalLogTapHandle {
    /// Record a structured internal log event in retention and fan it out
    /// non-blockingly to any active subscribers.
    ///
    /// The broadcast send is performed **while the write lock is still held**
    /// so that concurrent `record()` callers cannot interleave their sends and
    /// invert the monotonic seq order visible to subscribers.
    /// `broadcast::send()` is non-blocking (it never waits for receivers), so
    /// holding the lock is safe.
    pub fn record(&self, event: LogEvent) {
        let mut state = self.state.write();
        let seq = state.next_seq();
        let estimated_bytes = estimate_event_size(&event);
        // Skip the send when nobody is listening (benign race: a subscriber
        // that connects between the check and the send picks up the event
        // from the snapshot query instead).
        if self.subscribers.receiver_count() > 0 {
            let retained = RetainedLogEvent { seq, event };
            state.push(retained.clone(), estimated_bytes);
            let _ = self.subscribers.send(Arc::new(retained));
        } else {
            state.push(RetainedLogEvent { seq, event }, estimated_bytes);
        }
        // Write lock released here — after the send, preserving seq order.
    }

    /// Returns a counter that tracks logs dropped before retention.
    #[must_use]
    pub fn ingest_drop_counter(&self) -> InternalLogTapDropCounter {
        self.dropped_on_ingest.clone()
    }

    /// Subscribe to the live stream of newly recorded log events.
    ///
    /// Returns a broadcast receiver that delivers each `RetainedLogEvent` as
    /// an `Arc` immediately after `record()` appends it to the ring buffer.
    /// The receiver is bounded: if a subscriber falls more than
    /// `SUBSCRIBER_CHANNEL_CAPACITY` events behind, the next `recv()` returns
    /// `Err(broadcast::error::RecvError::Lagged(n))` indicating how many
    /// events were skipped.  The producer is never slowed by slow subscribers.
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<Arc<RetainedLogEvent>> {
        self.subscribers.subscribe()
    }

    /// Query retained logs using cursor-based semantics.
    #[must_use]
    pub fn query(&self, query: LogQuery) -> LogQueryResult {
        let state = self.state.read();
        let oldest_seq = state.entries.front().map(|entry| entry.event.seq);
        let newest_seq = state.entries.back().map(|entry| entry.event.seq);
        let truncated_before_seq = match (query.after, oldest_seq) {
            (Some(after), Some(oldest)) if after < oldest.saturating_sub(1) => Some(oldest),
            _ => None,
        };

        let logs: Vec<_> = if let Some(after) = query.after {
            state
                .entries
                .iter()
                .filter(|entry| entry.event.seq > after)
                .take(query.limit)
                .map(|entry| entry.event.clone())
                .collect()
        } else {
            let start = state.entries.len().saturating_sub(query.limit);
            state
                .entries
                .iter()
                .skip(start)
                .map(|entry| entry.event.clone())
                .collect()
        };
        let next_seq = logs
            .last()
            .map(|entry| entry.seq)
            .or(query.after)
            .unwrap_or(0);

        LogQueryResult {
            oldest_seq,
            newest_seq,
            next_seq,
            truncated_before_seq,
            dropped_on_ingest: self.dropped_on_ingest.load(),
            dropped_on_retention: state.dropped_on_retention,
            retained_bytes: state.retained_bytes,
            logs,
        }
    }
}

/// Create a new internal log tap from configuration.
#[must_use]
pub fn build(config: &InternalLogTapConfig) -> InternalLogTapHandle {
    let (subscribers, _) = broadcast::channel(SUBSCRIBER_CHANNEL_CAPACITY);
    InternalLogTapHandle {
        state: Arc::new(RwLock::new(LogRetention::new(
            config.max_entries,
            config.max_bytes,
        ))),
        dropped_on_ingest: InternalLogTapDropCounter::default(),
        subscribers,
    }
}

fn estimate_event_size(event: &LogEvent) -> usize {
    size_of::<RetainedLogEvent>()
        + event.record.body_attrs_bytes.len()
        + event.record.context.len() * size_of::<crate::registry::EntityKey>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{ObservedEvent, ObservedEventReporter};
    use crate::self_tracing::LogContext;
    use crate::tracing_init::{ProviderSetup, TracingSetup};
    use otap_df_config::observed_state::SendPolicy;
    use otap_df_config::settings::telemetry::logs::LogLevel;

    fn level(s: &str) -> LogLevel {
        serde_yaml::from_str(&format!("\"{s}\"")).unwrap()
    }

    fn event_with_message(message: &str) -> LogEvent {
        let (tx, rx) = flume::bounded(1);
        let reporter = ObservedEventReporter::new(SendPolicy::default(), tx);
        let setup = TracingSetup::new(
            ProviderSetup::InternalAsync { reporter },
            level("info"),
            LogContext::new,
        );

        setup.with_subscriber_ignoring_env(|| {
            tracing::event!(tracing::Level::INFO, message = message);
        });

        match rx.recv().expect("log event should be captured") {
            ObservedEvent::Log(log) => log,
            ObservedEvent::Engine(_) => unreachable!("expected log event"),
        }
    }

    #[test]
    fn retention_evicts_oldest_by_entry_limit() {
        let config = InternalLogTapConfig {
            enabled: true,
            max_entries: 2,
            max_bytes: usize::MAX,
        };
        let handle = build(&config);

        handle.record(event_with_message("1"));
        handle.record(event_with_message("2"));
        handle.record(event_with_message("3"));

        let result = handle.query(LogQuery {
            after: None,
            limit: 10,
        });
        let seqs: Vec<u64> = result.logs.into_iter().map(|log| log.seq).collect();
        assert_eq!(seqs, vec![2, 3]);
        assert_eq!(result.dropped_on_retention, 1);
    }

    #[test]
    fn retention_evicts_oldest_by_byte_limit() {
        let config = InternalLogTapConfig {
            enabled: true,
            max_entries: 10,
            max_bytes: estimate_event_size(&event_with_message("1234")) + 1,
        };
        let handle = build(&config);

        handle.record(event_with_message("1234"));
        handle.record(event_with_message("56"));

        let result = handle.query(LogQuery {
            after: None,
            limit: 10,
        });
        let seqs: Vec<u64> = result.logs.into_iter().map(|log| log.seq).collect();
        assert_eq!(seqs, vec![2]);
        assert_eq!(result.dropped_on_retention, 1);
    }

    #[test]
    fn query_returns_incremental_logs_after_cursor() {
        let config = InternalLogTapConfig {
            enabled: true,
            max_entries: 10,
            max_bytes: usize::MAX,
        };
        let handle = build(&config);

        for seq in 1..=4 {
            handle.record(event_with_message(&seq.to_string()));
        }

        let result = handle.query(LogQuery {
            after: Some(2),
            limit: 10,
        });
        let seqs: Vec<u64> = result.logs.into_iter().map(|log| log.seq).collect();
        assert_eq!(seqs, vec![3, 4]);
        assert_eq!(result.next_seq, 4);
        assert!(result.truncated_before_seq.is_none());
    }

    #[test]
    fn query_reports_truncation_when_cursor_falls_behind() {
        let config = InternalLogTapConfig {
            enabled: true,
            max_entries: 2,
            max_bytes: usize::MAX,
        };
        let handle = build(&config);

        handle.record(event_with_message("1"));
        handle.record(event_with_message("2"));
        handle.record(event_with_message("3"));

        let result = handle.query(LogQuery {
            after: Some(0),
            limit: 10,
        });
        let seqs: Vec<u64> = result.logs.into_iter().map(|log| log.seq).collect();
        assert_eq!(seqs, vec![2, 3]);
        assert_eq!(result.truncated_before_seq, Some(2));
        assert_eq!(result.next_seq, 3);
    }

    #[test]
    fn query_does_not_report_truncation_when_cursor_is_current() {
        let config = InternalLogTapConfig {
            enabled: true,
            max_entries: 2,
            max_bytes: usize::MAX,
        };
        let handle = build(&config);

        handle.record(event_with_message("1"));
        handle.record(event_with_message("2"));

        let result = handle.query(LogQuery {
            after: Some(1),
            limit: 10,
        });
        assert!(result.truncated_before_seq.is_none());
    }

    #[test]
    fn drop_counter_is_reported() {
        let config = InternalLogTapConfig {
            enabled: true,
            max_entries: 10,
            max_bytes: usize::MAX,
        };
        let handle = build(&config);

        handle.ingest_drop_counter().increment();
        handle.record(event_with_message("accepted"));

        let result = handle.query(LogQuery {
            after: None,
            limit: 10,
        });
        let seqs: Vec<u64> = result.logs.into_iter().map(|log| log.seq).collect();
        assert_eq!(seqs, vec![1]);
        assert_eq!(result.dropped_on_ingest, 1);
    }

    #[test]
    fn query_keeps_cursor_when_no_new_logs_are_available() {
        let config = InternalLogTapConfig {
            enabled: true,
            max_entries: 10,
            max_bytes: usize::MAX,
        };
        let handle = build(&config);

        for seq in 1..=4 {
            handle.record(event_with_message(&seq.to_string()));
        }

        let result = handle.query(LogQuery {
            after: Some(4),
            limit: 10,
        });
        assert!(result.logs.is_empty());
        assert_eq!(result.next_seq, 4);
    }

    #[tokio::test]
    async fn subscriber_receives_recorded_events() {
        let config = InternalLogTapConfig {
            enabled: true,
            max_entries: 10,
            max_bytes: usize::MAX,
        };
        let handle = build(&config);
        let mut rx = handle.subscribe();

        handle.record(event_with_message("hello"));
        handle.record(event_with_message("world"));

        let e1 = rx.recv().await.expect("first event");
        let e2 = rx.recv().await.expect("second event");
        assert_eq!(e1.seq, 1);
        assert_eq!(e2.seq, 2);
    }

    #[tokio::test]
    async fn subscriber_receives_lagged_error_when_too_slow() {
        let config = InternalLogTapConfig {
            enabled: true,
            max_entries: 1024,
            max_bytes: usize::MAX,
        };
        let handle = build(&config);
        // Subscribe but don't read.
        let mut rx = handle.subscribe();

        // Overflow the broadcast channel capacity (SUBSCRIBER_CHANNEL_CAPACITY = 512).
        for i in 0..=512u32 {
            handle.record(event_with_message(&i.to_string()));
        }

        // The receiver should get a Lagged error rather than blocking the producer.
        let result = rx.recv().await;
        assert!(
            matches!(result, Err(broadcast::error::RecvError::Lagged(_))),
            "expected Lagged error, got {result:?}"
        );
    }

    #[tokio::test]
    async fn subscriber_does_not_receive_events_recorded_before_subscribe() {
        let config = InternalLogTapConfig {
            enabled: true,
            max_entries: 10,
            max_bytes: usize::MAX,
        };
        let handle = build(&config);

        handle.record(event_with_message("before"));
        // Subscribe after the first event.
        let mut rx = handle.subscribe();
        handle.record(event_with_message("after"));

        let entry = rx.recv().await.expect("second event");
        assert_eq!(entry.seq, 2);
    }

    #[tokio::test]
    async fn subscriber_receives_events_in_seq_order_under_concurrency() {
        // Regression test: seq is assigned under the write lock AND the
        // broadcast send happens while the lock is still held, so concurrent
        // record() callers cannot invert the order visible to subscribers.
        let config = InternalLogTapConfig {
            enabled: true,
            max_entries: 200,
            max_bytes: usize::MAX,
        };
        let handle = build(&config);
        let mut rx = handle.subscribe();

        const N: usize = 50;
        // Spawn many concurrent tasks that each record one event.
        let tasks: Vec<_> = (0..N)
            .map(|i| {
                let h = handle.clone();
                tokio::spawn(async move {
                    h.record(event_with_message(&i.to_string()));
                })
            })
            .collect();
        for t in tasks {
            t.await.expect("task panicked");
        }

        let mut seqs = Vec::new();
        while let Ok(e) = rx.try_recv() {
            seqs.push(e.seq);
        }

        assert_eq!(seqs.len(), N, "expected {N} events, got {}", seqs.len());
        // Seqs must be strictly increasing — no inversions allowed.
        for w in seqs.windows(2) {
            assert!(
                w[0] < w[1],
                "out-of-order delivery: seq {} before {}",
                w[0],
                w[1]
            );
        }
    }

    #[test]
    fn initial_query_uses_latest_returned_seq_as_next_cursor() {
        let config = InternalLogTapConfig {
            enabled: true,
            max_entries: 10,
            max_bytes: usize::MAX,
        };
        let handle = build(&config);

        for seq in 1..=4 {
            handle.record(event_with_message(&seq.to_string()));
        }

        let result = handle.query(LogQuery {
            after: None,
            limit: 2,
        });
        let seqs: Vec<u64> = result.logs.into_iter().map(|log| log.seq).collect();
        assert_eq!(seqs, vec![3, 4]);
        assert_eq!(result.next_seq, 4);
    }
}
