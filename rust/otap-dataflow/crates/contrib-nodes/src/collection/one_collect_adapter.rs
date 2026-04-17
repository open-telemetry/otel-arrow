// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![cfg_attr(not(target_os = "linux"), allow(dead_code, unused_imports))]

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::io;
use std::rc::Rc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[cfg(target_os = "linux")]
use one_collect::helpers::exporting::ExportMachine;
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
            let cpu = u16::try_from(*cpu).map_err(|_| {
                CollectInitError::Io(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(
                        "target CPU id `{cpu}` exceeds supported u16 range for one_collect user_events collection"
                    ),
                ))
            })?;
            builder = builder.with_target_cpu(cpu);
        }

        let mut session = builder.build().map_err(CollectInitError::Io)?;
        session.set_read_timeout(Duration::from_millis(0));

        let pending = Rc::new(RefCell::new(VecDeque::new()));
        let lost_samples = Rc::new(Cell::new(0u64));
        let ancillary = session.ancillary_data();
        let time_field = session.time_data_ref();
        let pid_field = session.pid_field_ref();
        let tid_field = session.tid_data_ref();
        let tracefs = TraceFS::open().map_err(CollectInitError::Io)?;

        for subscription in subscriptions {
            let (_, event_name) = subscription.tracepoint.split_once(':').ok_or_else(|| {
                CollectInitError::InvalidTracepoint(subscription.tracepoint.clone())
            })?;

            let mut event = match tracefs.find_event("user_events", event_name) {
                Ok(event) => event,
                Err(error) => match error.kind() {
                    io::ErrorKind::NotFound => {
                        return Err(CollectInitError::MissingTracepoint(
                            subscription.tracepoint.clone(),
                        ));
                    }
                    io::ErrorKind::PermissionDenied => {
                        return Err(CollectInitError::Io(io::Error::new(
                            io::ErrorKind::PermissionDenied,
                            format!(
                                "tracepoint `{}` is registered but tracefs metadata is not readable; run df_engine with elevated privileges or relax tracefs read permissions",
                                subscription.tracepoint
                            ),
                        )));
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
                    .map(sample_qpc_to_unix_nano)
                    .unwrap_or_else(current_time_unix_nano);
                let pid = event_pid_field
                    .try_get_u32(full_data)
                    .map(|value| value as i32);
                let tid = event_tid_field
                    .try_get_u32(full_data)
                    .map(|value| value as i32);
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
        let started = Instant::now();
        // Use bounded parsing so the drain returns promptly even under continuous
        // writer load. `parse_all()` only stops when the ring buffers drain fully,
        // which can starve the pop loop below when producers emit faster than we
        // can empty the rings. `max_drain_ns` is the total work budget for this
        // drain call, so any time spent parsing reduces the time left to pop the
        // callback queue below.
        self.session
            .parse_for_duration(max_drain_ns)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;

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
fn current_time_unix_nano() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos().min(u128::from(u64::MAX)) as u64)
        .unwrap_or_default()
}

#[cfg(target_os = "linux")]
fn sample_qpc_to_unix_nano(sample_qpc: u64) -> u64 {
    let now_unix = current_time_unix_nano();
    let now_qpc = ExportMachine::qpc_time();
    if sample_qpc <= now_qpc {
        now_unix.saturating_sub(now_qpc - sample_qpc)
    } else {
        now_unix.saturating_add(sample_qpc - now_qpc)
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
