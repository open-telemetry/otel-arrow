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

    fn push(&mut self, event: LogEvent) {
        let seq = self.next_seq;
        self.next_seq = self.next_seq.saturating_add(1);
        let estimated_bytes = estimate_event_size(&event);
        let slot = RetainedLogSlot {
            event: RetainedLogEvent { seq, event },
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
#[derive(Clone, Debug)]
pub struct InternalLogTapHandle {
    state: Arc<RwLock<LogRetention>>,
    dropped_on_ingest: InternalLogTapDropCounter,
}

impl InternalLogTapHandle {
    /// Record a structured internal log event in retention.
    pub fn record(&self, event: LogEvent) {
        self.state.write().push(event);
    }

    /// Returns a counter that tracks logs dropped before retention.
    #[must_use]
    pub fn ingest_drop_counter(&self) -> InternalLogTapDropCounter {
        self.dropped_on_ingest.clone()
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
    InternalLogTapHandle {
        state: Arc::new(RwLock::new(LogRetention::new(
            config.max_entries,
            config.max_bytes,
        ))),
        dropped_on_ingest: InternalLogTapDropCounter::default(),
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
