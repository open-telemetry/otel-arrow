// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![cfg_attr(not(target_os = "linux"), allow(dead_code, unused_imports))]

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::io;
use std::rc::Rc;
use std::time::{Duration, Instant};

#[cfg(target_os = "linux")]
use nix::libc;
#[cfg(target_os = "linux")]
use one_collect::perf_event::{PerfSession, RingBufBuilder, RingBufSessionBuilder};
#[cfg(target_os = "linux")]
use one_collect::tracefs::TraceFS;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CollectedEvent {
    pub timestamp_unix_nano: u64,
    pub cpu: Option<u32>,
    pub pid: Option<i32>,
    pub tid: Option<i32>,
    pub payload: Vec<u8>,
    pub source: EventSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub(crate) enum EventSource {
    UserEvents(UserEventsSource),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UserEventsSource {
    pub tracepoint: String,
    pub sample_id: u64,
}

#[derive(Debug)]
pub(crate) struct CollectedDrain {
    pub events: Vec<CollectedEvent>,
    pub lost_samples: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct UserEventsSubscription {
    pub tracepoint: String,
}

#[derive(Debug, Clone)]
pub(crate) struct UserEventsSessionConfig {
    pub per_cpu_buffer_size: usize,
    pub cpu_ids: Vec<usize>,
}

#[derive(Debug)]
pub(crate) enum CollectInitError {
    MissingTracepoint(String),
    InvalidTracepoint(String),
    Io(io::Error),
}

impl std::fmt::Display for CollectInitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingTracepoint(name) => write!(f, "tracepoint `{name}` is not registered"),
            Self::InvalidTracepoint(name) => write!(f, "tracepoint `{name}` is invalid"),
            Self::Io(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for CollectInitError {}

#[cfg(target_os = "linux")]
pub(crate) struct OneCollectUserEventsSession {
    session: PerfSession,
    pending: Rc<RefCell<VecDeque<CollectedEvent>>>,
    lost_samples: Rc<Cell<u64>>,
    subscription_count: usize,
}

#[cfg(target_os = "linux")]
impl OneCollectUserEventsSession {
    pub(crate) fn open(
        subscriptions: &[UserEventsSubscription],
        config: &UserEventsSessionConfig,
    ) -> Result<Self, CollectInitError> {
        if subscriptions.is_empty() {
            return Err(CollectInitError::InvalidTracepoint(
                "at least one tracepoint subscription is required".to_owned(),
            ));
        }
        if config.cpu_ids.is_empty() {
            return Err(CollectInitError::Io(io::Error::new(
                io::ErrorKind::InvalidInput,
                "at least one target CPU is required for one_collect user_events collection",
            )));
        }

        let page_count = page_count(config.per_cpu_buffer_size);
        let mut builder = RingBufSessionBuilder::new()
            .with_page_count(page_count)
            .with_tracepoint_events(RingBufBuilder::for_tracepoint());
        for cpu in &config.cpu_ids {
            builder = builder.with_target_cpu(*cpu as u16);
        }

        let mut session = builder.build().map_err(CollectInitError::Io)?;
        session.set_read_timeout(Duration::from_millis(0));

        let pending = Rc::new(RefCell::new(VecDeque::new()));
        let lost_samples = Rc::new(Cell::new(0u64));
        let monotonic_raw_to_realtime_offset_ns = monotonic_raw_to_realtime_offset_ns()?;

        let ancillary = session.ancillary_data();
        let time_field = session.time_data_ref();
        let pid_field = session.pid_field_ref();
        let tid_field = session.tid_data_ref();
        let tracefs = TraceFS::open().map_err(CollectInitError::Io)?;

        for subscription in subscriptions {
            let (_, event_name) = subscription
                .tracepoint
                .split_once(':')
                .ok_or_else(|| CollectInitError::InvalidTracepoint(subscription.tracepoint.clone()))?;

            let mut event = match tracefs.find_event("user_events", event_name) {
                Ok(event) => event,
                Err(error) => match error.kind() {
                    io::ErrorKind::NotFound => {
                        return Err(CollectInitError::MissingTracepoint(subscription.tracepoint.clone()))
                    }
                    io::ErrorKind::PermissionDenied => {
                        return Err(CollectInitError::Io(io::Error::new(
                            io::ErrorKind::PermissionDenied,
                            format!(
                                "tracepoint `{}` is registered but tracefs metadata is not readable; run df_engine with elevated privileges or relax tracefs read permissions",
                                subscription.tracepoint
                            ),
                        )))
                    }
                    _ => return Err(CollectInitError::Io(error)),
                },
            };

            let sample_id = event.id() as u64;
            let tracepoint = subscription.tracepoint.clone();
            let event_pending = Rc::clone(&pending);
            let event_ancillary = ancillary.clone();
            let event_time_field = time_field.clone();
            let event_pid_field = pid_field.clone();
            let event_tid_field = tid_field.clone();

            event.add_callback(move |data| {
                let full_data = data.full_data();
                let timestamp = event_time_field
                    .try_get_u64(full_data)
                    .map(|value| perf_timestamp_to_unix_nano(value, monotonic_raw_to_realtime_offset_ns))
                    .unwrap_or_default();
                let pid = event_pid_field.try_get_u32(full_data).map(|value| value as i32);
                let tid = event_tid_field.try_get_u32(full_data).map(|value| value as i32);
                let mut cpu = None;
                event_ancillary.read(|values| {
                    cpu = Some(values.cpu());
                });

                event_pending.borrow_mut().push_back(CollectedEvent {
                    timestamp_unix_nano: timestamp,
                    cpu,
                    pid,
                    tid,
                    payload: data.event_data().to_vec(),
                    source: EventSource::UserEvents(UserEventsSource {
                        tracepoint: tracepoint.clone(),
                        sample_id,
                    }),
                });
                Ok(())
            });

            session.add_event(event).map_err(CollectInitError::Io)?;
        }

        register_lost_callbacks(&mut session, Rc::clone(&lost_samples));
        session.enable().map_err(CollectInitError::Io)?;

        Ok(Self {
            session,
            pending,
            lost_samples,
            subscription_count: subscriptions.len(),
        })
    }

    pub(crate) fn subscription_count(&self) -> usize {
        self.subscription_count
    }

    pub(crate) fn drain(
        &mut self,
        max_records: usize,
        max_bytes: usize,
        max_drain_ns: Duration,
    ) -> io::Result<CollectedDrain> {
        self.session
            .parse_all()
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;

        let started = Instant::now();
        let mut drained_bytes = 0usize;
        let mut events = Vec::new();
        let mut pending = self.pending.borrow_mut();

        while let Some(front) = pending.front() {
            if events.len() >= max_records
                || drained_bytes >= max_bytes
                || started.elapsed() >= max_drain_ns
            {
                break;
            }

            drained_bytes = drained_bytes.saturating_add(front.payload.len());
            if let Some(event) = pending.pop_front() {
                events.push(event);
            }
        }

        Ok(CollectedDrain {
            events,
            lost_samples: self.lost_samples.replace(0),
        })
    }
}

#[cfg(target_os = "linux")]
fn register_lost_callbacks(session: &mut PerfSession, lost_samples: Rc<Cell<u64>>) {
    let lost_field = session.lost_event().format().get_field_ref("lost");
    let lost_counter = Rc::clone(&lost_samples);
    session.lost_event().add_callback(move |data| {
        let lost = lost_field
            .and_then(|field| data.format().try_get_u64(field, data.event_data()))
            .unwrap_or(1);
        lost_counter.set(lost_counter.get().saturating_add(lost));
        Ok(())
    });

    let lost_samples_field = session.lost_samples_event().format().get_field_ref("lost");
    session.lost_samples_event().add_callback(move |data| {
        let lost = lost_samples_field
            .and_then(|field| data.format().try_get_u64(field, data.event_data()))
            .unwrap_or(1);
        lost_samples.set(lost_samples.get().saturating_add(lost));
        Ok(())
    });
}

#[cfg(target_os = "linux")]
fn page_count(per_cpu_buffer_size: usize) -> usize {
    let page_size = one_collect::os::linux::system_page_size() as usize;
    let rounded = per_cpu_buffer_size.max(page_size).next_power_of_two();
    (rounded / page_size).max(1)
}

#[cfg(target_os = "linux")]
fn monotonic_raw_to_realtime_offset_ns() -> Result<i128, CollectInitError> {
    let before = clock_gettime_ns(libc::CLOCK_MONOTONIC_RAW)?;
    let realtime = clock_gettime_ns(libc::CLOCK_REALTIME)?;
    let after = clock_gettime_ns(libc::CLOCK_MONOTONIC_RAW)?;
    let midpoint = before + (after - before) / 2;
    Ok(realtime as i128 - midpoint as i128)
}

#[cfg(target_os = "linux")]
fn clock_gettime_ns(clock_id: libc::clockid_t) -> Result<u64, CollectInitError> {
    let mut timespec = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    let status = unsafe { libc::clock_gettime(clock_id, &mut timespec) };
    if status != 0 {
        return Err(CollectInitError::Io(io::Error::last_os_error()));
    }
    if timespec.tv_sec < 0 || timespec.tv_nsec < 0 {
        return Err(CollectInitError::Io(io::Error::new(
            io::ErrorKind::InvalidData,
            "clock_gettime returned a negative timestamp",
        )));
    }
    Ok((timespec.tv_sec as u64)
        .saturating_mul(1_000_000_000)
        .saturating_add(timespec.tv_nsec as u64))
}

#[cfg(target_os = "linux")]
fn perf_timestamp_to_unix_nano(timestamp: u64, monotonic_raw_to_realtime_offset_ns: i128) -> u64 {
    let unix_timestamp = (timestamp as i128).saturating_add(monotonic_raw_to_realtime_offset_ns);
    if unix_timestamp <= 0 {
        0
    } else if unix_timestamp >= u64::MAX as i128 {
        u64::MAX
    } else {
        unix_timestamp as u64
    }
}

#[cfg(not(target_os = "linux"))]
pub(crate) struct OneCollectUserEventsSession;

#[cfg(not(target_os = "linux"))]
impl OneCollectUserEventsSession {
    pub(crate) fn open(
        _subscriptions: &[UserEventsSubscription],
        _config: &UserEventsSessionConfig,
    ) -> Result<Self, CollectInitError> {
        Err(CollectInitError::Io(io::Error::new(
            io::ErrorKind::Unsupported,
            "one_collect user_events collection is supported only on Linux",
        )))
    }

    pub(crate) fn subscription_count(&self) -> usize {
        0
    }

    pub(crate) fn drain(
        &mut self,
        _max_records: usize,
        _max_bytes: usize,
        _max_drain_ns: Duration,
    ) -> io::Result<CollectedDrain> {
        Ok(CollectedDrain {
            events: Vec::new(),
            lost_samples: 0,
        })
    }
}
