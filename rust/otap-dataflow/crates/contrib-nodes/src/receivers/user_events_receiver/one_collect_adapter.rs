// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::io;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use one_collect::event::LocationType;
use one_collect::helpers::exporting::ExportMachine;
use one_collect::perf_event::{PerfSession, RingBufBuilder, RingBufSessionBuilder};
use one_collect::tracefs::TraceFS;

use super::session::{TracefsField, TracefsFieldLocation};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CollectedEvent {
    pub timestamp_unix_nano: u64,
    pub process_id: Option<u32>,
    pub thread_id: Option<u32>,
    pub event_data: Vec<u8>,
    pub user_data_offset: usize,
    pub fields: Arc<[TracefsField]>,
    pub subscription_index: usize,
}

#[derive(Debug)]
pub(crate) struct CollectedDrain {
    pub events: Vec<CollectedEvent>,
    pub lost_samples: u64,
    pub dropped_pending_overflow: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct UserEventsSubscription {
    pub tracepoint: String,
    pub limits: Option<UserEventsSubscriptionLimits>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct UserEventsSubscriptionLimits {
    pub max_pending_events: Option<usize>,
    pub max_pending_bytes: Option<usize>,
}

#[derive(Debug, Clone)]
pub(crate) struct UserEventsSessionConfig {
    pub per_cpu_buffer_size: usize,
    pub cpu_ids: Vec<usize>,
    pub max_pending_events: usize,
    pub max_pending_bytes: usize,
}

#[derive(Debug, Clone, Copy)]
struct PerfTimeAnchor {
    unix_nano: u64,
    perf_nano: u64,
}

impl PerfTimeAnchor {
    fn capture() -> Self {
        let before = ExportMachine::qpc_time();
        let unix_nano = current_time_unix_nano();
        let after = ExportMachine::qpc_time();

        Self {
            unix_nano,
            perf_nano: before.saturating_add(after.saturating_sub(before) / 2),
        }
    }

    fn sample_perf_time_to_unix_nano(self, sample_perf_time: u64) -> u64 {
        if sample_perf_time <= self.perf_nano {
            self.unix_nano
                .saturating_sub(self.perf_nano - sample_perf_time)
        } else {
            self.unix_nano
                .saturating_add(sample_perf_time - self.perf_nano)
        }
    }
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

pub(crate) struct OneCollectUserEventsSession {
    session: PerfSession,
    pending: PendingQueues,
    lost_samples: Rc<Cell<u64>>,
    subscription_count: usize,
    round_robin_cursor: usize,
}

#[derive(Clone)]
enum PendingQueues {
    Shared(SharedPending),
    Governed(Rc<RefCell<GovernedPending>>),
}

#[derive(Clone)]
struct SharedPending {
    events: Rc<RefCell<VecDeque<CollectedEvent>>>,
    bytes: Rc<Cell<usize>>,
    dropped_pending_overflow: Rc<Cell<u64>>,
    max_pending_events: usize,
    max_pending_bytes: usize,
}

#[derive(Debug)]
struct GovernedPending {
    subscriptions: Vec<SubscriptionPending>,
    total_pending_events: usize,
    total_pending_bytes: usize,
    max_pending_events: usize,
    max_pending_bytes: usize,
}

#[derive(Debug)]
struct SubscriptionPending {
    events: VecDeque<CollectedEvent>,
    pending_bytes: usize,
    dropped_subscription_cap: u64,
    dropped_global_cap: u64,
    limits: UserEventsSubscriptionLimits,
}

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

        let pending = PendingQueues::new(
            subscriptions,
            config.max_pending_events,
            config.max_pending_bytes,
        );
        let lost_samples = Rc::new(Cell::new(0u64));
        let time_field = session.time_data_ref();
        let pid_field = session.pid_field_ref();
        let tid_field = session.tid_data_ref();
        // TODO: Prefer a one_collect-owned sample-time to realtime conversion API
        // once the session exposes one. Revisit periodic re-anchoring only with
        // an explicit policy for wall-clock steps, since refreshing this anchor
        // can make emitted timestamps jump forward or backward.
        let tracefs = TraceFS::open().map_err(CollectInitError::Io)?;
        let time_anchor = PerfTimeAnchor::capture();

        // Current late-registration behavior is all-or-nothing: every
        // configured tracepoint must exist before the perf session is enabled.
        // If any subscription is missing, the caller retries the entire open
        // later and no already-registered subscriptions are collected yet.
        // TODO: Support partial session startup plus later registration/reopen
        // so present tracepoints can be collected while waiting for missing
        // subscriptions.
        for (subscription_index, subscription) in subscriptions.iter().enumerate() {
            let (group, event_name) = subscription.tracepoint.split_once(':').ok_or_else(|| {
                CollectInitError::InvalidTracepoint(subscription.tracepoint.clone())
            })?;
            if group != "user_events" {
                return Err(CollectInitError::InvalidTracepoint(
                    subscription.tracepoint.clone(),
                ));
            }

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

            let fields = Arc::<[TracefsField]>::from(
                event
                    .format()
                    .fields()
                    .iter()
                    .map(|field| TracefsField {
                        name: field.name.clone(),
                        type_name: field.type_name.clone(),
                        location: match field.location {
                            LocationType::Static => TracefsFieldLocation::Static,
                            LocationType::StaticString => TracefsFieldLocation::StaticString,
                            LocationType::DynRelative => TracefsFieldLocation::DynRelative,
                            LocationType::DynAbsolute => TracefsFieldLocation::DynAbsolute,
                            LocationType::StaticLenPrefixArray => {
                                TracefsFieldLocation::StaticLenPrefixArray
                            }
                            LocationType::StaticUTF16String => {
                                TracefsFieldLocation::StaticUtf16String
                            }
                        },
                        offset: field.offset,
                        size: field.size,
                    })
                    .collect::<Vec<_>>(),
            );
            let user_data_offset = fields
                .iter()
                .find(|field| !field.name.starts_with("common_"))
                .map_or(0, |field| field.offset);

            let event_pending = pending.clone();
            let event_time_field = time_field.clone();
            let event_pid_field = pid_field.clone();
            let event_tid_field = tid_field.clone();
            let event_fields = Arc::clone(&fields);

            event.add_callback(move |data| {
                let payload_len = data.event_data().len();
                if !event_pending.accepts_event(subscription_index, payload_len) {
                    return Ok(());
                }

                let event_data = data.event_data().to_vec();
                let full_data = data.full_data();
                let timestamp = event_time_field
                    .try_get_u64(full_data)
                    .map(|sample_time| time_anchor.sample_perf_time_to_unix_nano(sample_time))
                    .unwrap_or_else(current_time_unix_nano);
                let process_id = event_pid_field.try_get_u32(full_data);
                let thread_id = event_tid_field.try_get_u32(full_data);

                event_pending.push_event(CollectedEvent {
                    timestamp_unix_nano: timestamp,
                    process_id,
                    thread_id,
                    event_data,
                    user_data_offset,
                    fields: Arc::clone(&event_fields),
                    subscription_index,
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
            round_robin_cursor: 0,
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
        // `max_drain_ns` is the total intended wall-time budget for this drain
        // turn. We split it into two reserved phases so neither parsing nor
        // popping can starve the other:
        //
        //   * parse phase: at most `max_drain_ns / 2`. Bounding parse avoids
        //     the `parse_all()` failure mode where a saturated ring keeps
        //     returning records indefinitely and starves the pop loop.
        //   * pop phase:  whatever time remains until `started + max_drain_ns`.
        //     Because parse is capped at half the budget, pop is guaranteed
        //     at least `max_drain_ns / 2` of wall-time before the deadline.
        //
        // Forward-progress rule: if the parse phase somehow exhausts the
        // entire budget (e.g. the OS scheduler delays us) and we would
        // otherwise return zero events while `pending` is non-empty, we pop
        // exactly one record (subject to record/byte caps) so the queue
        // drains monotonically across turns under continuous load.
        let started = Instant::now();
        let parse_budget = max_drain_ns / 2;
        self.session
            .parse_for_duration(parse_budget)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;

        let deadline = started + max_drain_ns;
        let mut drained_bytes = 0usize;
        let pending_len = self.pending.len();
        // TODO(perf): Avoid allocating this temporary events Vec on every drain
        // turn. The session wrapper immediately moves these records into its
        // caller-owned RawUserEventsRecord buffer, so the adapter should
        // eventually drain directly into a reusable caller-provided output
        // buffer once the adapter/session ownership boundary is revisited.
        let mut events = Vec::with_capacity(max_records.min(pending_len));
        self.pending.drain(
            &mut events,
            &mut drained_bytes,
            max_records,
            max_bytes,
            deadline,
            &mut self.round_robin_cursor,
        );

        Ok(CollectedDrain {
            events,
            lost_samples: self.lost_samples.replace(0),
            dropped_pending_overflow: self.pending.take_dropped_pending_overflow(),
        })
    }
}

impl UserEventsSubscriptionLimits {
    fn has_effective_pending_limit(self) -> bool {
        self.max_pending_events.is_some() || self.max_pending_bytes.is_some()
    }
}

impl PendingQueues {
    fn new(
        subscriptions: &[UserEventsSubscription],
        max_pending_events: usize,
        max_pending_bytes: usize,
    ) -> Self {
        if !subscriptions.iter().any(|subscription| {
            subscription
                .limits
                .is_some_and(UserEventsSubscriptionLimits::has_effective_pending_limit)
        }) {
            return Self::Shared(SharedPending {
                events: Rc::new(RefCell::new(VecDeque::new())),
                bytes: Rc::new(Cell::new(0)),
                dropped_pending_overflow: Rc::new(Cell::new(0)),
                max_pending_events,
                max_pending_bytes,
            });
        }

        Self::Governed(Rc::new(RefCell::new(GovernedPending {
            subscriptions: subscriptions
                .iter()
                .map(|subscription| SubscriptionPending {
                    events: VecDeque::new(),
                    pending_bytes: 0,
                    dropped_subscription_cap: 0,
                    dropped_global_cap: 0,
                    limits: subscription.limits.unwrap_or(UserEventsSubscriptionLimits {
                        max_pending_events: None,
                        max_pending_bytes: None,
                    }),
                })
                .collect(),
            total_pending_events: 0,
            total_pending_bytes: 0,
            max_pending_events,
            max_pending_bytes,
        })))
    }

    fn len(&self) -> usize {
        match self {
            Self::Shared(pending) => pending.events.borrow().len(),
            Self::Governed(pending) => pending.borrow().total_pending_events,
        }
    }

    fn accepts_event(&self, subscription_index: usize, payload_len: usize) -> bool {
        match self {
            Self::Shared(pending) => {
                let current_pending_bytes = pending.bytes.get();
                let pending_len = pending.events.borrow().len();
                if pending_accepts_event(
                    pending_len,
                    current_pending_bytes,
                    payload_len,
                    pending.max_pending_events,
                    pending.max_pending_bytes,
                ) {
                    true
                } else {
                    pending
                        .dropped_pending_overflow
                        .set(pending.dropped_pending_overflow.get().saturating_add(1));
                    false
                }
            }
            Self::Governed(pending) => {
                let mut pending = pending.borrow_mut();
                let max_pending_events = pending.max_pending_events;
                let max_pending_bytes = pending.max_pending_bytes;
                let total_pending_events = pending.total_pending_events;
                let total_pending_bytes = pending.total_pending_bytes;

                let Some(subscription) = pending.subscriptions.get_mut(subscription_index) else {
                    return false;
                };

                if let Some(max_pending_events) = subscription.limits.max_pending_events {
                    if subscription.events.len() >= max_pending_events {
                        subscription.dropped_subscription_cap =
                            subscription.dropped_subscription_cap.saturating_add(1);
                        return false;
                    }
                }
                if let Some(max_pending_bytes) = subscription.limits.max_pending_bytes {
                    if subscription.pending_bytes.saturating_add(payload_len) > max_pending_bytes {
                        subscription.dropped_subscription_cap =
                            subscription.dropped_subscription_cap.saturating_add(1);
                        return false;
                    }
                }

                if !pending_accepts_event(
                    total_pending_events,
                    total_pending_bytes,
                    payload_len,
                    max_pending_events,
                    max_pending_bytes,
                ) {
                    subscription.dropped_global_cap =
                        subscription.dropped_global_cap.saturating_add(1);
                    return false;
                }

                true
            }
        }
    }

    fn push_event(&self, event: CollectedEvent) {
        let event_len = event.event_data.len();
        match self {
            Self::Shared(pending) => {
                pending.events.borrow_mut().push_back(event);
                pending
                    .bytes
                    .set(pending.bytes.get().saturating_add(event_len));
            }
            Self::Governed(pending) => {
                let mut pending = pending.borrow_mut();
                let subscription_index = event.subscription_index;
                let Some(subscription) = pending.subscriptions.get_mut(subscription_index) else {
                    return;
                };
                subscription.pending_bytes = subscription.pending_bytes.saturating_add(event_len);
                subscription.events.push_back(event);
                pending.total_pending_events = pending.total_pending_events.saturating_add(1);
                pending.total_pending_bytes = pending.total_pending_bytes.saturating_add(event_len);
            }
        }
    }

    fn drain(
        &self,
        events: &mut Vec<CollectedEvent>,
        drained_bytes: &mut usize,
        max_records: usize,
        max_bytes: usize,
        deadline: Instant,
        round_robin_cursor: &mut usize,
    ) {
        match self {
            Self::Shared(pending) => {
                drain_shared_pending(
                    pending,
                    events,
                    drained_bytes,
                    max_records,
                    max_bytes,
                    deadline,
                );
            }
            Self::Governed(pending) => {
                drain_governed_pending(
                    &mut pending.borrow_mut(),
                    events,
                    drained_bytes,
                    max_records,
                    max_bytes,
                    deadline,
                    round_robin_cursor,
                );
            }
        }
    }

    fn take_dropped_pending_overflow(&self) -> u64 {
        match self {
            Self::Shared(pending) => pending.dropped_pending_overflow.replace(0),
            Self::Governed(pending) => {
                let mut pending = pending.borrow_mut();
                let mut dropped = 0u64;
                for subscription in &mut pending.subscriptions {
                    dropped = dropped
                        .saturating_add(subscription.dropped_subscription_cap)
                        .saturating_add(subscription.dropped_global_cap);
                    subscription.dropped_subscription_cap = 0;
                    subscription.dropped_global_cap = 0;
                }
                dropped
            }
        }
    }
}

fn drain_shared_pending(
    pending: &SharedPending,
    events: &mut Vec<CollectedEvent>,
    drained_bytes: &mut usize,
    max_records: usize,
    max_bytes: usize,
    deadline: Instant,
) {
    let mut pending_events = pending.events.borrow_mut();

    while let Some(front) = pending_events.front() {
        if events.len() >= max_records || Instant::now() >= deadline {
            break;
        }

        let front_len = front.event_data.len();
        let next_bytes = drained_bytes.saturating_add(front_len);
        if next_bytes > max_bytes && !events.is_empty() {
            break;
        }

        *drained_bytes = next_bytes;
        if let Some(event) = pop_pending_event(&mut pending_events, &pending.bytes) {
            events.push(event);
        }
    }

    // Forward-progress guarantee: ensure at least one event drains per turn
    // when the queue is non-empty, even if the parse phase consumed the
    // deadline before the pop loop could run.
    if events.is_empty() && !pending_events.is_empty() && max_records > 0 {
        if let Some(front) = pending_events.front() {
            let front_len = front.event_data.len();
            if front_len <= max_bytes {
                if let Some(event) = pop_pending_event(&mut pending_events, &pending.bytes) {
                    events.push(event);
                }
            }
        }
    }
}

fn drain_governed_pending(
    pending: &mut GovernedPending,
    events: &mut Vec<CollectedEvent>,
    drained_bytes: &mut usize,
    max_records: usize,
    max_bytes: usize,
    deadline: Instant,
    round_robin_cursor: &mut usize,
) {
    let subscription_count = pending.subscriptions.len();
    if subscription_count == 0 {
        return;
    }

    while events.len() < max_records
        && pending.total_pending_events > 0
        && Instant::now() < deadline
    {
        if !pop_next_governed_event(
            pending,
            events,
            drained_bytes,
            max_bytes,
            round_robin_cursor,
            false,
        ) {
            break;
        }
    }

    // Preserve the shared-queue forward-progress behavior for governed queues:
    // if the timed loop returned no records and a queued record fits within the
    // byte cap, pop one from the round-robin cursor even if the deadline passed.
    if events.is_empty()
        && max_records > 0
        && pending.total_pending_events > 0
        && *drained_bytes == 0
    {
        let _ = pop_next_governed_event(
            pending,
            events,
            drained_bytes,
            max_bytes,
            round_robin_cursor,
            true,
        );
    }
}

fn pop_next_governed_event(
    pending: &mut GovernedPending,
    events: &mut Vec<CollectedEvent>,
    drained_bytes: &mut usize,
    max_bytes: usize,
    round_robin_cursor: &mut usize,
    require_byte_fit: bool,
) -> bool {
    let subscription_count = pending.subscriptions.len();
    for _ in 0..subscription_count {
        let subscription_index = *round_robin_cursor % subscription_count;
        *round_robin_cursor = (*round_robin_cursor + 1) % subscription_count;

        let Some(front) = pending.subscriptions[subscription_index].events.front() else {
            continue;
        };

        let front_len = front.event_data.len();
        let next_bytes = drained_bytes.saturating_add(front_len);
        if next_bytes > max_bytes && (!events.is_empty() || require_byte_fit) {
            continue;
        }

        let event = {
            let subscription = &mut pending.subscriptions[subscription_index];
            let event = subscription
                .events
                .pop_front()
                .expect("front() returned Some");
            subscription.pending_bytes = subscription.pending_bytes.saturating_sub(front_len);
            event
        };
        pending.total_pending_events = pending.total_pending_events.saturating_sub(1);
        pending.total_pending_bytes = pending.total_pending_bytes.saturating_sub(front_len);
        *drained_bytes = next_bytes;
        events.push(event);
        return true;
    }

    false
}

fn pending_accepts_event(
    pending_events: usize,
    pending_bytes: usize,
    payload_len: usize,
    max_pending_events: usize,
    max_pending_bytes: usize,
) -> bool {
    pending_events < max_pending_events
        && pending_bytes.saturating_add(payload_len) <= max_pending_bytes
}

fn pop_pending_event(
    pending: &mut VecDeque<CollectedEvent>,
    pending_bytes: &Cell<usize>,
) -> Option<CollectedEvent> {
    let event = pending.pop_front()?;
    pending_bytes.set(pending_bytes.get().saturating_sub(event.event_data.len()));
    Some(event)
}

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

fn page_count(per_cpu_buffer_size: usize) -> usize {
    let page_size = one_collect::os::linux::system_page_size() as usize;
    let rounded = per_cpu_buffer_size.max(page_size).next_power_of_two();
    (rounded / page_size).max(1)
}

fn current_time_unix_nano() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos().min(u128::from(u64::MAX)) as u64)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::super::session::TracefsField;
    use super::{
        CollectedEvent, PendingQueues, UserEventsSubscription, UserEventsSubscriptionLimits,
        pending_accepts_event,
    };
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    fn test_subscription(
        tracepoint: &str,
        limits: Option<UserEventsSubscriptionLimits>,
    ) -> UserEventsSubscription {
        UserEventsSubscription {
            tracepoint: tracepoint.to_owned(),
            limits,
        }
    }

    fn test_event(subscription_index: usize, len: usize) -> CollectedEvent {
        CollectedEvent {
            timestamp_unix_nano: 0,
            process_id: None,
            thread_id: None,
            event_data: vec![0; len],
            user_data_offset: 0,
            fields: Arc::<[TracefsField]>::from(Vec::new()),
            subscription_index,
        }
    }

    #[test]
    fn pending_accepts_event_below_caps() {
        assert!(pending_accepts_event(3, 100, 20, 4, 128));
    }

    #[test]
    fn pending_accepts_event_at_exact_byte_cap() {
        assert!(pending_accepts_event(3, 100, 28, 4, 128));
    }

    #[test]
    fn pending_rejects_event_count_cap() {
        assert!(!pending_accepts_event(4, 100, 1, 4, 128));
    }

    #[test]
    fn pending_rejects_byte_cap() {
        assert!(!pending_accepts_event(3, 100, 29, 4, 128));
    }

    #[test]
    fn pending_uses_shared_mode_without_effective_limits() {
        let subscriptions = vec![
            test_subscription("user_events:first", None),
            test_subscription(
                "user_events:second",
                Some(UserEventsSubscriptionLimits {
                    max_pending_events: None,
                    max_pending_bytes: None,
                }),
            ),
        ];

        assert!(matches!(
            PendingQueues::new(&subscriptions, 10, 1024),
            PendingQueues::Shared(_)
        ));
    }

    #[test]
    fn pending_uses_governed_mode_with_any_effective_limit() {
        let subscriptions = vec![
            test_subscription("user_events:first", None),
            test_subscription(
                "user_events:second",
                Some(UserEventsSubscriptionLimits {
                    max_pending_events: Some(1),
                    max_pending_bytes: None,
                }),
            ),
        ];

        assert!(matches!(
            PendingQueues::new(&subscriptions, 10, 1024),
            PendingQueues::Governed(_)
        ));
    }

    #[test]
    fn governed_pending_rejects_subscription_cap_without_global_scan() {
        let subscriptions = vec![
            test_subscription(
                "user_events:noisy",
                Some(UserEventsSubscriptionLimits {
                    max_pending_events: Some(1),
                    max_pending_bytes: None,
                }),
            ),
            test_subscription("user_events:quiet", None),
        ];
        let pending = PendingQueues::new(&subscriptions, 10, 1024);

        assert!(pending.accepts_event(0, 8));
        pending.push_event(test_event(0, 8));
        assert!(!pending.accepts_event(0, 8));
        assert!(pending.accepts_event(1, 8));
        pending.push_event(test_event(1, 8));

        assert_eq!(pending.len(), 2);
        assert_eq!(pending.take_dropped_pending_overflow(), 1);
    }

    #[test]
    fn governed_pending_rejects_global_cap_attributing_to_rejected_subscription() {
        let subscriptions = vec![
            test_subscription("user_events:first", None),
            test_subscription(
                "user_events:second",
                Some(UserEventsSubscriptionLimits {
                    max_pending_events: Some(10),
                    max_pending_bytes: None,
                }),
            ),
        ];
        let pending = PendingQueues::new(&subscriptions, 1, 1024);

        assert!(pending.accepts_event(0, 8));
        pending.push_event(test_event(0, 8));
        assert!(!pending.accepts_event(1, 8));

        assert_eq!(pending.len(), 1);
        assert_eq!(pending.take_dropped_pending_overflow(), 1);
    }

    #[test]
    fn governed_drain_round_robins_and_persists_cursor() {
        let subscriptions = vec![
            test_subscription(
                "user_events:first",
                Some(UserEventsSubscriptionLimits {
                    max_pending_events: Some(10),
                    max_pending_bytes: None,
                }),
            ),
            test_subscription("user_events:second", None),
        ];
        let pending = PendingQueues::new(&subscriptions, 10, 1024);
        for _ in 0..2 {
            assert!(pending.accepts_event(0, 8));
            pending.push_event(test_event(0, 8));
        }
        assert!(pending.accepts_event(1, 8));
        pending.push_event(test_event(1, 8));

        let mut cursor = 0usize;
        let mut first_turn = Vec::new();
        let mut first_turn_bytes = 0usize;
        pending.drain(
            &mut first_turn,
            &mut first_turn_bytes,
            2,
            1024,
            Instant::now() + Duration::from_secs(1),
            &mut cursor,
        );
        assert_eq!(
            first_turn
                .iter()
                .map(|event| event.subscription_index)
                .collect::<Vec<_>>(),
            vec![0, 1]
        );
        assert_eq!(cursor, 0);

        let mut second_turn = Vec::new();
        let mut second_turn_bytes = 0usize;
        pending.drain(
            &mut second_turn,
            &mut second_turn_bytes,
            1,
            1024,
            Instant::now() + Duration::from_secs(1),
            &mut cursor,
        );
        assert_eq!(second_turn[0].subscription_index, 0);
    }
}
