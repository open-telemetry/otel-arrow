// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Internal log tap with non-blocking ingestion and bounded in-memory retention.

use crate::error::Error;
use crate::event::LogEvent;
use otap_df_config::settings::telemetry::logs::InternalLogTapConfig;
use parking_lot::RwLock;
use std::collections::VecDeque;
use std::mem::size_of;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone)]
struct QueuedLogEvent {
    seq: u64,
    event: LogEvent,
    estimated_bytes: usize,
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
}

impl LogRetention {
    fn new(max_entries: usize, max_bytes: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(max_entries),
            retained_bytes: 0,
            max_entries,
            max_bytes,
            dropped_on_retention: 0,
        }
    }

    fn push(&mut self, item: QueuedLogEvent) {
        let slot = RetainedLogSlot {
            event: RetainedLogEvent {
                seq: item.seq,
                event: item.event,
            },
            estimated_bytes: item.estimated_bytes,
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

/// A non-blocking reporter used on the tracing hot path.
#[derive(Clone, Debug)]
pub struct InternalLogTapReporter {
    next_seq: Arc<AtomicU64>,
    dropped_on_ingest: Arc<AtomicU64>,
    tx: flume::Sender<QueuedLogEvent>,
}

impl InternalLogTapReporter {
    /// Queue a structured internal log event without blocking the caller.
    pub fn log(&self, event: LogEvent) {
        let seq = self.next_seq.fetch_add(1, Ordering::Relaxed);
        let item = QueuedLogEvent {
            seq,
            estimated_bytes: estimate_event_size(&event),
            event,
        };
        if self.tx.try_send(item).is_err() {
            let _ = self.dropped_on_ingest.fetch_add(1, Ordering::Relaxed);
        }
    }
}

/// Shared query handle for the retained internal log ring buffer.
#[derive(Clone, Debug)]
pub struct InternalLogTapHandle {
    state: Arc<RwLock<LogRetention>>,
    dropped_on_ingest: Arc<AtomicU64>,
}

impl InternalLogTapHandle {
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
            dropped_on_ingest: self.dropped_on_ingest.load(Ordering::Relaxed),
            dropped_on_retention: state.dropped_on_retention,
            retained_bytes: state.retained_bytes,
            logs,
        }
    }
}

/// Runtime task draining the ingestion queue into the retained ring buffer.
#[derive(Debug)]
pub struct InternalLogTapRuntime {
    rx: flume::Receiver<QueuedLogEvent>,
    state: Arc<RwLock<LogRetention>>,
}

impl InternalLogTapRuntime {
    fn flush_batch(&self, batch: &mut Vec<QueuedLogEvent>) {
        if batch.is_empty() {
            return;
        }

        let mut retention = self.state.write();
        for item in batch.drain(..) {
            retention.push(item);
        }
    }

    fn drain_remaining(&self, batch: &mut Vec<QueuedLogEvent>) {
        while let Ok(item) = self.rx.try_recv() {
            batch.push(item);
            if batch.len() >= 64 {
                self.flush_batch(batch);
            }
        }
        self.flush_batch(batch);
    }

    /// Drain the ingestion queue until cancellation is requested.
    pub async fn run(self, cancel: CancellationToken) -> Result<(), Error> {
        let mut batch = Vec::with_capacity(64);

        loop {
            tokio::select! {
                _ = cancel.cancelled() => {
                    // Best-effort final flush for events already accepted on the hot path.
                    self.drain_remaining(&mut batch);
                    return Ok(());
                },
                received = self.rx.recv_async() => {
                    let Ok(first) = received else {
                        return Ok(());
                    };
                    batch.push(first);
                    while batch.len() < 64 {
                        match self.rx.try_recv() {
                            Ok(item) => batch.push(item),
                            Err(_) => break,
                        }
                    }
                    self.flush_batch(&mut batch);
                }
            }
        }
    }
}

/// Create a new internal log tap from configuration.
#[must_use]
pub fn build(
    config: &InternalLogTapConfig,
) -> (
    InternalLogTapReporter,
    InternalLogTapHandle,
    InternalLogTapRuntime,
) {
    let (tx, rx) = flume::bounded(config.ingest_channel_size);
    let next_seq = Arc::new(AtomicU64::new(1));
    let dropped_on_ingest = Arc::new(AtomicU64::new(0));
    let state = Arc::new(RwLock::new(LogRetention::new(
        config.max_entries,
        config.max_bytes,
    )));

    let reporter = InternalLogTapReporter {
        next_seq: next_seq.clone(),
        dropped_on_ingest: dropped_on_ingest.clone(),
        tx,
    };
    let handle = InternalLogTapHandle {
        state: state.clone(),
        dropped_on_ingest,
    };
    let runtime = InternalLogTapRuntime { rx, state };

    (reporter, handle, runtime)
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
            ProviderSetup::InternalAsync {
                reporter,
                tap: None,
            },
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
            ingest_channel_size: 4,
            max_entries: 2,
            max_bytes: usize::MAX,
        };
        let (_reporter, handle, runtime) = build(&config);

        {
            let mut state = runtime.state.write();
            state.push(QueuedLogEvent {
                seq: 1,
                event: event_with_message("1"),
                estimated_bytes: 1,
            });
            state.push(QueuedLogEvent {
                seq: 2,
                event: event_with_message("2"),
                estimated_bytes: 1,
            });
            state.push(QueuedLogEvent {
                seq: 3,
                event: event_with_message("3"),
                estimated_bytes: 1,
            });
        }

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
            ingest_channel_size: 4,
            max_entries: 10,
            max_bytes: 5,
        };
        let (_reporter, handle, runtime) = build(&config);

        {
            let mut state = runtime.state.write();
            state.push(QueuedLogEvent {
                seq: 1,
                event: event_with_message("1234"),
                estimated_bytes: 4,
            });
            state.push(QueuedLogEvent {
                seq: 2,
                event: event_with_message("56"),
                estimated_bytes: 2,
            });
        }

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
            ingest_channel_size: 4,
            max_entries: 10,
            max_bytes: usize::MAX,
        };
        let (_reporter, handle, runtime) = build(&config);

        {
            let mut state = runtime.state.write();
            for seq in 1..=4 {
                state.push(QueuedLogEvent {
                    seq,
                    event: event_with_message(&seq.to_string()),
                    estimated_bytes: 1,
                });
            }
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
            ingest_channel_size: 4,
            max_entries: 2,
            max_bytes: usize::MAX,
        };
        let (_reporter, handle, runtime) = build(&config);

        {
            let mut state = runtime.state.write();
            state.push(QueuedLogEvent {
                seq: 5,
                event: event_with_message("a"),
                estimated_bytes: 1,
            });
            state.push(QueuedLogEvent {
                seq: 6,
                event: event_with_message("b"),
                estimated_bytes: 1,
            });
        }

        let result = handle.query(LogQuery {
            after: Some(1),
            limit: 10,
        });
        assert_eq!(result.truncated_before_seq, Some(5));
        assert_eq!(result.next_seq, 6);
    }

    #[test]
    fn query_does_not_report_truncation_when_cursor_is_immediately_before_oldest() {
        let config = InternalLogTapConfig {
            enabled: true,
            ingest_channel_size: 4,
            max_entries: 2,
            max_bytes: usize::MAX,
        };
        let (_reporter, handle, runtime) = build(&config);

        {
            let mut state = runtime.state.write();
            state.push(QueuedLogEvent {
                seq: 5,
                event: event_with_message("a"),
                estimated_bytes: 1,
            });
            state.push(QueuedLogEvent {
                seq: 6,
                event: event_with_message("b"),
                estimated_bytes: 1,
            });
        }

        let result = handle.query(LogQuery {
            after: Some(4),
            limit: 10,
        });
        let seqs: Vec<u64> = result.logs.into_iter().map(|log| log.seq).collect();
        assert_eq!(seqs, vec![5, 6]);
        assert!(result.truncated_before_seq.is_none());
    }

    #[tokio::test]
    async fn runtime_flushes_queued_logs_on_cancellation() {
        let config = InternalLogTapConfig {
            enabled: true,
            ingest_channel_size: 4,
            max_entries: 10,
            max_bytes: usize::MAX,
        };
        let (reporter, handle, runtime) = build(&config);

        reporter.log(event_with_message("first"));
        reporter.log(event_with_message("second"));

        let cancel = CancellationToken::new();
        cancel.cancel();
        runtime
            .run(cancel)
            .await
            .expect("runtime should shut down cleanly");

        let result = handle.query(LogQuery {
            after: None,
            limit: 10,
        });
        let seqs: Vec<u64> = result.logs.into_iter().map(|log| log.seq).collect();
        assert_eq!(seqs, vec![1, 2]);
    }

    #[test]
    fn query_keeps_cursor_when_no_new_logs_are_available() {
        let config = InternalLogTapConfig {
            enabled: true,
            ingest_channel_size: 4,
            max_entries: 10,
            max_bytes: usize::MAX,
        };
        let (_reporter, handle, runtime) = build(&config);

        {
            let mut state = runtime.state.write();
            for seq in 1..=4 {
                state.push(QueuedLogEvent {
                    seq,
                    event: event_with_message(&seq.to_string()),
                    estimated_bytes: 1,
                });
            }
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
            ingest_channel_size: 4,
            max_entries: 10,
            max_bytes: usize::MAX,
        };
        let (_reporter, handle, runtime) = build(&config);

        {
            let mut state = runtime.state.write();
            for seq in 1..=4 {
                state.push(QueuedLogEvent {
                    seq,
                    event: event_with_message(&seq.to_string()),
                    estimated_bytes: 1,
                });
            }
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
