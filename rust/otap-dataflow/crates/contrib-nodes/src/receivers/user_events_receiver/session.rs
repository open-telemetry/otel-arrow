// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Session wrapper for Linux user_events collection.

use std::io;
use std::time::Duration;

use tokio::time;

use super::one_collect_adapter::{
    CollectInitError, OneCollectUserEventsSession, UserEventsSessionConfig, UserEventsSubscription,
};
use super::{DrainConfig, SessionConfig, SubscriptionConfig};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TracefsFieldLocation {
    Static,
    StaticString,
    DynRelative,
    DynAbsolute,
    StaticLenPrefixArray,
    StaticUtf16String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TracefsField {
    pub name: String,
    pub type_name: String,
    pub location: TracefsFieldLocation,
    pub offset: usize,
    pub size: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawUserEventsRecord {
    pub subscription_index: usize,
    pub timestamp_unix_nano: u64,
    /// Process id from perf `PERF_SAMPLE_TID` metadata, when present.
    pub process_id: Option<u32>,
    /// Thread id from perf `PERF_SAMPLE_TID` metadata, when present.
    pub thread_id: Option<u32>,
    /// Full raw tracepoint sample bytes as delivered by the perf session.
    ///
    /// Conceptually, for normal tracefs decoding this is:
    ///
    /// ```text
    /// +----------------------+-------------------------------+
    /// | common_* fields      | user-declared tracefs fields  |
    /// | fixed trace metadata | producer payload fields       |
    /// +----------------------+-------------------------------+
    /// 0                      user_data_offset
    /// ```
    ///
    /// For EventHeader tracepoints, the user-declared region starts with
    /// the EventHeader bytes:
    ///
    /// ```text
    /// +----------------------+------------------------------------------+
    /// | common_* fields      | EventHeader + extensions + event payload |
    /// +----------------------+------------------------------------------+
    /// 0                      user_data_offset
    /// ```
    ///
    /// The receiver keeps these layers separate: tracefs metadata tells us
    /// where the user region begins, and `FormatConfig` decides whether the
    /// user region is decoded as standard tracefs fields or as EventHeader.
    pub event_data: Vec<u8>,
    /// Byte offset from the start of `event_data` to the first non-common
    /// tracefs field. This is the beginning of the producer-defined payload
    /// region for EventHeader decoding.
    pub user_data_offset: usize,
    /// Tracefs field metadata for the tracepoint. These fields come from
    /// the tracefs `format` file, not from the sample payload itself.
    pub fields: std::sync::Arc<[TracefsField]>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SessionDrainStats {
    pub received_samples: u64,
    pub dropped_no_subscription: u64,
    pub lost_samples: u64,
    pub dropped_pending_overflow: u64,
}

#[derive(Debug)]
pub(crate) enum SessionInitError {
    MissingTracepoint(String),
    InvalidTracepoint(String),
    Io(io::Error),
}

impl std::fmt::Display for SessionInitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingTracepoint(name) => {
                write!(f, "tracepoint `{name}` is not registered")
            }
            Self::InvalidTracepoint(name) => write!(f, "tracepoint `{name}` is invalid"),
            Self::Io(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for SessionInitError {}

impl From<io::Error> for SessionInitError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

pub(crate) struct UserEventsSession {
    inner: OneCollectUserEventsSession,
}

impl UserEventsSession {
    fn drain_once_impl(
        &mut self,
        config: &DrainConfig,
        out: &mut Vec<RawUserEventsRecord>,
    ) -> io::Result<SessionDrainStats> {
        let drained = self.inner.drain(
            config.max_records_per_turn,
            config.max_bytes_per_turn,
            config.max_drain_ns,
        )?;

        out.clear();
        out.reserve(drained.events.len());
        let mut stats = SessionDrainStats {
            received_samples: drained.events.len() as u64,
            dropped_no_subscription: 0,
            lost_samples: drained.lost_samples,
            dropped_pending_overflow: drained.dropped_pending_overflow,
        };
        for event in drained.events {
            // This should be unreachable with today's adapter: subscription
            // indices are captured from the fixed subscription list when the
            // session opens. Keep the guard as defense-in-depth in case future
            // adapter changes ever decouple callbacks from that list.
            if event.subscription_index >= self.inner.subscription_count() {
                stats.dropped_no_subscription += 1;
                continue;
            }
            out.push(RawUserEventsRecord {
                subscription_index: event.subscription_index,
                timestamp_unix_nano: event.timestamp_unix_nano,
                process_id: event.process_id,
                thread_id: event.thread_id,
                event_data: event.event_data,
                user_data_offset: event.user_data_offset,
                fields: event.fields,
            });
        }

        Ok(stats)
    }

    pub(crate) fn open(
        subscriptions: &[SubscriptionConfig],
        config: &SessionConfig,
        cpu_id: usize,
    ) -> Result<Self, SessionInitError> {
        let subscriptions = subscriptions
            .iter()
            .map(|subscription| UserEventsSubscription {
                tracepoint: subscription.tracepoint.clone(),
            })
            .collect::<Vec<_>>();
        let config = UserEventsSessionConfig {
            per_cpu_buffer_size: config.per_cpu_buffer_size,
            max_pending_events: config.max_pending_events,
            max_pending_bytes: config.max_pending_bytes,
            // Open the perf ring for this pipeline's pinned CPU only.
            // Keeping ring reads on the same CPU as the pipeline thread
            // preserves the NUMA-locality design documented in the
            // receiver README; do not widen this to "all CPUs" without
            // revisiting that contract.
            cpu_ids: vec![cpu_id],
        };

        let inner = OneCollectUserEventsSession::open(&subscriptions, &config).map_err(
            |error| match error {
                CollectInitError::MissingTracepoint(tracepoint) => {
                    SessionInitError::MissingTracepoint(tracepoint)
                }
                CollectInitError::InvalidTracepoint(tracepoint) => {
                    SessionInitError::InvalidTracepoint(tracepoint)
                }
                CollectInitError::Io(error) => SessionInitError::Io(error),
            },
        )?;

        Ok(Self { inner })
    }

    pub(crate) async fn drain_ready(
        &mut self,
        config: &DrainConfig,
        out: &mut Vec<RawUserEventsRecord>,
    ) -> io::Result<SessionDrainStats> {
        let poll_interval = Duration::from_millis(1);

        loop {
            let stats = self.drain_once_impl(config, out)?;
            if !out.is_empty()
                || stats.lost_samples > 0
                || stats.dropped_pending_overflow > 0
                || stats.dropped_no_subscription > 0
            {
                return Ok(stats);
            }

            // TODO: Replace this fixed sleep with an event-driven wakeup once
            // one-collect exposes a waitable/readiness API for PerfSession.
            // Tracking issue: https://github.com/microsoft/one-collect/issues/254
            time::sleep(poll_interval).await;
        }
    }

    pub(crate) fn drain_once(
        &mut self,
        config: &DrainConfig,
        out: &mut Vec<RawUserEventsRecord>,
    ) -> io::Result<SessionDrainStats> {
        self.drain_once_impl(config, out)
    }

    pub(crate) fn subscription_count(&self) -> usize {
        self.inner.subscription_count()
    }
}
