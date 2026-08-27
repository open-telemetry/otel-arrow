// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Dispatches transformed Arrow batches to ClickHouse with bounded concurrency and optional
//! exporter-local batching.
//!
//! # Role in the exporter
//!
//! Transformation happens before this module. Each submitted message therefore carries the
//! original `OtapPdata`, used later for ACK/NACK delivery, plus zero or more ClickHouse-compatible
//! `RecordBatch`es. This module owns insertion scheduling and completion tracking; the caller owns
//! transformation, telemetry accounting, and delivery of the final ACK or NACK.
//!
//! Exporter-local batching is deliberately specialized. It combines final ClickHouse
//! `RecordBatch`es so several small upstream messages can share one ArrowStream insertion without
//! decoding, rebuilding, or changing those messages. It complements the generic batch processor,
//! which operates earlier in the pipeline and cannot reason about destination tables, Arrow
//! schemas, or ClickHouse insertion overhead.
//!
//! # Dispatch modes
//!
//! `WriteDispatcher` selects one of two modes:
//!
//! - Without `insert_batching`, `InFlightWrites` runs one ordinary `write_batches` future per
//!   message. `max_in_flight` bounds the number of concurrently polled message writes. This is the
//!   legacy behavior and opens a separate insertion for each mapped batch.
//! - With `insert_batching`, `PersistentWritePool` first coalesces compatible batches and then
//!   dispatches closed insertion groups across `max_in_flight` long-lived lane tasks.
//!
//! The lane tasks are persistent, but ClickHouse insertions are not kept open while batches
//! accumulate. Coalescing happens in memory before lane assignment. Each lane job opens an
//! `ArrowInsert`, writes one or more complete `RecordBatch`es into that ArrowStream, awaits
//! `end()`, and only then accepts another job. This keeps threshold waiting out of the lanes and
//! leaves every available lane ready for ClickHouse I/O.
//!
//! ```text
//! insert_batching disabled:
//!
//!   transformed message -> bounded write future -> ClickHouse -> one completion
//!
//! insert_batching enabled:
//!
//!   transformed messages
//!           |
//!           v
//!   PendingInsertion (one table and schema)
//!           | row, byte, time, or compatibility boundary
//!           v
//!      waiting LaneJob queue
//!           |
//!       +---+---+------------+
//!       v       v            v
//!     lane 0  lane 1  ...  lane N
//!       +-------+------------+
//!               |
//!               v
//!      one aggregate LaneCompletion
//!               |
//!               v
//!      one CompletedWrite per original message
//! ```
//!
//! # Coalescing rules
//!
//! A `PendingInsertion` can contain batches only when their destination table and Arrow schema are
//! identical. It is dispatched when its configured row, estimated in-memory byte, or elapsed-time
//! threshold is reached. Thresholds are checked after appending a complete batch: they never split
//! a batch, reject rows, or limit the accepted payload. A table or schema change closes the current
//! group before the incompatible batch is considered.
//!
//! Only messages with exactly one mapped batch participate in coalescing. A message with no mapped
//! batch completes immediately. A message with batches for multiple tables first closes the pending
//! group, then runs as one immediate lane job so its original message still has one completion.
//!
//! # Lane scheduling and backpressure
//!
//! Each lane is a local task with a single-slot job channel. `available_lanes` records which tasks
//! may receive work, while `waiting_jobs` holds closed groups until a lane is free. Once that queue
//! is non-empty, the pool reports itself at capacity so normal pdata admission pauses while control
//! messages and write completions continue to make progress.
//!
//! A lane sends exactly one `LaneCompletion` per job through a channel bounded by the lane count.
//! When the dispatcher accepts it, the lane is made available and queued ClickHouse work is
//! scheduled before individual message completions are expanded. This prevents ACK/NACK fan-out
//! from leaving an otherwise idle lane unused.
//!
//! # Completion and error semantics
//!
//! Every accepted pdata message produces exactly one `CompletedWrite`. A coalesced insertion keeps
//! the original pdata and row count for each member in `PendingWrite`; all members receive the same
//! insertion outcome. Structured insertion failures are shared with `Rc` because the entire path
//! runs on the exporter's local runtime. Start/write failures are classified as request errors,
//! while `end()` failures are response errors, and the ClickHouse client source chain is retained.
//!
//! `CompletedGroup` expands an aggregate lane result incrementally and rotates partially drained
//! groups through the completion queue. The dispatcher yields after a bounded number of individual
//! completions. Together with the bounded lane-completion channel, this prevents a large coalesced
//! insertion from monopolizing the local runtime or creating an unbounded completion burst.
//!
//! # Shutdown
//!
//! Shutdown first flushes any partial `PendingInsertion`, then drains accepted work until the
//! caller's deadline. `outstanding` counts original pdata messages rather than lane jobs, so a
//! deadline reports how many delivery outcomes remain unresolved. Dropping the pool aborts its
//! local lane tasks after draining finishes or the deadline expires.

use std::collections::{HashMap, VecDeque};
use std::num::NonZeroUsize;
use std::rc::Rc;
use std::time::{Duration, Instant as StdInstant};

use arrow_array::RecordBatch;
use arrow_schema::SchemaRef;
use async_trait::async_trait;
use clickhouse_ext_arrow::ArrowInsert;
use futures::future::LocalBoxFuture;
use otel_arrow_dfe_otap::pdata::OtapPdata;
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::Instant;

use super::config::InsertBatchingConfig;
use super::error::ClickhouseExporterError;
use super::in_flight::{CompletedWrite, InFlightWrites, WrittenRows};
use super::writer::ClickHouseWriter;

/// Maximum number of individual completions emitted without yielding to other local tasks.
const COMPLETIONS_BEFORE_YIELD: usize = 64;

pub(super) struct WriteJob {
    pdata: OtapPdata,
    export_started_at: StdInstant,
    batches: HashMap<ArrowPayloadType, RecordBatch>,
}

impl WriteJob {
    fn new(
        pdata: OtapPdata,
        export_started_at: StdInstant,
        batches: HashMap<ArrowPayloadType, RecordBatch>,
    ) -> Self {
        Self {
            pdata,
            export_started_at,
            batches,
        }
    }
}

/// Shared result for every original message represented by one ClickHouse insertion.
///
/// Writer lanes run on the exporter's local runtime, so `Rc` preserves one structured error and
/// its source chain without rebuilding it for every message in a coalesced insertion.
enum CompletionOutcome {
    Success,
    Failure(Rc<ClickhouseExporterError>),
}

impl CompletionOutcome {
    fn request_error(source: clickhouse::error::Error) -> Self {
        Self::Failure(Rc::new(ClickhouseExporterError::InsertRequestError {
            source,
        }))
    }

    fn response_error(source: clickhouse::error::Error) -> Self {
        Self::Failure(Rc::new(ClickhouseExporterError::InsertResponseError {
            source,
        }))
    }

    fn lane_closed(lane: usize) -> Self {
        Self::Failure(Rc::new(ClickhouseExporterError::WriterLaneClosed { lane }))
    }
}

struct CompletedGroup {
    pending: std::vec::IntoIter<PendingWrite>,
    outcome: CompletionOutcome,
}

impl CompletedGroup {
    fn with_outcome(pending: Vec<PendingWrite>, outcome: CompletionOutcome) -> Self {
        Self {
            pending: pending.into_iter(),
            outcome,
        }
    }

    fn success(pending: Vec<PendingWrite>) -> Self {
        Self::with_outcome(pending, CompletionOutcome::Success)
    }

    fn request_error(pending: Vec<PendingWrite>, source: clickhouse::error::Error) -> Self {
        Self::with_outcome(pending, CompletionOutcome::request_error(source))
    }

    fn response_error(pending: Vec<PendingWrite>, source: clickhouse::error::Error) -> Self {
        Self::with_outcome(pending, CompletionOutcome::response_error(source))
    }

    fn lane_closed(pending: Vec<PendingWrite>, lane: usize) -> Self {
        Self::with_outcome(pending, CompletionOutcome::lane_closed(lane))
    }

    fn next_completion(&mut self) -> Option<CompletedWrite> {
        let pending = self.pending.next()?;
        let result = match &self.outcome {
            CompletionOutcome::Success => Ok(pending.written_rows),
            CompletionOutcome::Failure(error) => Err(Rc::clone(error)),
        };
        Some(CompletedWrite {
            pdata: pending.pdata,
            export_started_at: pending.export_started_at,
            result,
        })
    }

    fn is_empty(&self) -> bool {
        self.pending.len() == 0
    }
}

/// One bounded channel event for all messages completed by a writer-lane job.
struct LaneCompletion {
    lane: usize,
    group: CompletedGroup,
}

#[allow(
    clippy::large_enum_variant,
    reason = "boxing CompletedWrite would allocate on every completed insertion"
)]
pub(super) enum DispatcherEvent {
    CapacityAvailable,
    Completed(CompletedWrite),
}

/// Narrow test seam over an ArrowStream insertion.
///
/// Concrete client errors remain structured here; request/response classification happens where
/// the operation is known instead of discarding the source chain at the trait boundary.
#[async_trait(?Send)]
pub(super) trait LaneInsert: Sized {
    async fn write(&mut self, batch: &RecordBatch) -> Result<(), clickhouse::error::Error>;
    async fn end(self) -> Result<(), clickhouse::error::Error>;
}

#[async_trait(?Send)]
impl LaneInsert for ArrowInsert {
    async fn write(&mut self, batch: &RecordBatch) -> Result<(), clickhouse::error::Error> {
        self.write(batch).await
    }

    async fn end(self) -> Result<(), clickhouse::error::Error> {
        self.end().await
    }
}

pub(super) trait LaneWriter: Clone + 'static {
    type Insert: LaneInsert + 'static;

    fn destination_table(&self, payload_type: ArrowPayloadType) -> Option<&str>;
    fn start_insert(&self, table_name: &str) -> Result<Self::Insert, clickhouse::error::Error>;
}

impl LaneWriter for ClickHouseWriter {
    type Insert = ArrowInsert;

    fn destination_table(&self, payload_type: ArrowPayloadType) -> Option<&str> {
        self.destination_table(payload_type)
    }

    fn start_insert(&self, table_name: &str) -> Result<Self::Insert, clickhouse::error::Error> {
        self.start_insert(table_name)
    }
}

struct PendingWrite {
    pdata: OtapPdata,
    export_started_at: StdInstant,
    written_rows: WrittenRows,
}

struct PendingInsertion {
    table_name: String,
    schema: SchemaRef,
    rows: usize,
    bytes: usize,
    batches: Vec<RecordBatch>,
    deadline: Instant,
    pending: Vec<PendingWrite>,
}

impl PendingInsertion {
    fn new(
        config: InsertBatchingConfig,
        pdata: OtapPdata,
        export_started_at: StdInstant,
        payload_type: ArrowPayloadType,
        table_name: String,
        batch: RecordBatch,
    ) -> Self {
        let rows = batch.num_rows();
        let bytes = batch.get_array_memory_size();
        let schema = batch.schema();
        let now = Instant::now();
        // Configuration parsing caps this duration. Falling back to an immediate deadline keeps
        // programmatically constructed configs safe on platforms with narrower Instant ranges.
        let deadline = now
            .checked_add(Duration::from_millis(config.max_delay_ms.get()))
            .unwrap_or(now);
        Self {
            table_name,
            schema,
            rows,
            bytes,
            batches: vec![batch],
            deadline,
            pending: vec![PendingWrite {
                pdata,
                export_started_at,
                written_rows: vec![(payload_type, rows as u64)],
            }],
        }
    }

    fn is_compatible(&self, table_name: &str, batch: &RecordBatch) -> bool {
        self.table_name == table_name && self.schema.as_ref() == batch.schema_ref().as_ref()
    }

    fn threshold_reached(&self, config: InsertBatchingConfig) -> bool {
        self.rows >= config.max_rows.get() || self.bytes >= config.max_bytes.get()
    }

    fn append(
        &mut self,
        pdata: OtapPdata,
        export_started_at: StdInstant,
        payload_type: ArrowPayloadType,
        batch: RecordBatch,
    ) {
        let rows = batch.num_rows();
        self.rows = self.rows.saturating_add(rows);
        self.bytes = self.bytes.saturating_add(batch.get_array_memory_size());
        self.batches.push(batch);
        self.pending.push(PendingWrite {
            pdata,
            export_started_at,
            written_rows: vec![(payload_type, rows as u64)],
        });
    }
}

async fn write_insertion<I: LaneInsert>(
    mut insert: I,
    insertion: PendingInsertion,
) -> CompletedGroup {
    let PendingInsertion {
        table_name,
        rows,
        batches,
        pending,
        ..
    } = insertion;
    let batch_count = batches.len();

    for batch in batches {
        if let Err(error) = insert.write(&batch).await {
            return CompletedGroup::request_error(pending, error);
        }
    }

    match insert.end().await {
        Ok(()) => {
            otel_debug!(
                "clickhouse.exporter.insertion.written",
                message = "Coalesced insertion successfully written.",
                table = table_name,
                rows = rows,
                batches = batch_count,
            );
            CompletedGroup::success(pending)
        }
        Err(error) => CompletedGroup::response_error(pending, error),
    }
}

async fn write_coalesced<W: LaneWriter>(writer: &W, insertion: PendingInsertion) -> CompletedGroup {
    let insert = match writer.start_insert(&insertion.table_name) {
        Ok(insert) => insert,
        Err(error) => {
            return CompletedGroup::request_error(insertion.pending, error);
        }
    };
    write_insertion(insert, insertion).await
}

async fn write_immediately<W: LaneWriter>(
    writer: &W,
    pdata: OtapPdata,
    export_started_at: StdInstant,
    batches: Vec<(ArrowPayloadType, String, RecordBatch)>,
) -> CompletedGroup {
    let mut written_rows = Vec::with_capacity(batches.len());
    for (payload_type, table_name, batch) in batches {
        let result = async {
            let mut insert = writer
                .start_insert(&table_name)
                .map_err(CompletionOutcome::request_error)?;
            insert
                .write(&batch)
                .await
                .map_err(CompletionOutcome::request_error)?;
            insert
                .end()
                .await
                .map_err(CompletionOutcome::response_error)
        }
        .await;

        if let Err(outcome) = result {
            return CompletedGroup::with_outcome(
                vec![PendingWrite {
                    pdata,
                    export_started_at,
                    written_rows,
                }],
                outcome,
            );
        }
        written_rows.push((payload_type, batch.num_rows() as u64));
    }

    CompletedGroup::success(vec![PendingWrite {
        pdata,
        export_started_at,
        written_rows,
    }])
}

enum LaneJob {
    Coalesced(PendingInsertion),
    Immediate {
        pdata: OtapPdata,
        export_started_at: StdInstant,
        batches: Vec<(ArrowPayloadType, String, RecordBatch)>,
    },
}

impl LaneJob {
    fn fail(self, lane: usize) -> CompletedGroup {
        match self {
            Self::Coalesced(insertion) => CompletedGroup::lane_closed(insertion.pending, lane),
            Self::Immediate {
                pdata,
                export_started_at,
                ..
            } => CompletedGroup::lane_closed(
                vec![PendingWrite {
                    pdata,
                    export_started_at,
                    written_rows: Vec::new(),
                }],
                lane,
            ),
        }
    }
}

async fn run_lane<W: LaneWriter>(
    lane: usize,
    writer: W,
    mut jobs: mpsc::Receiver<LaneJob>,
    events: mpsc::Sender<LaneCompletion>,
) {
    while let Some(job) = jobs.recv().await {
        let group = match job {
            LaneJob::Coalesced(insertion) => write_coalesced(&writer, insertion).await,
            LaneJob::Immediate {
                pdata,
                export_started_at,
                batches,
            } => write_immediately(&writer, pdata, export_started_at, batches).await,
        };
        if events.send(LaneCompletion { lane, group }).await.is_err() {
            break;
        }
    }
}

pub(super) struct PersistentWritePool<W: LaneWriter> {
    writer: W,
    config: InsertBatchingConfig,
    pending: Option<PendingInsertion>,
    waiting_jobs: VecDeque<LaneJob>,
    senders: Vec<mpsc::Sender<LaneJob>>,
    events: mpsc::Receiver<LaneCompletion>,
    completed_groups: VecDeque<CompletedGroup>,
    completed_group_capacity: usize,
    completions_since_yield: usize,
    handles: Vec<JoinHandle<()>>,
    available_lanes: VecDeque<usize>,
    outstanding: usize,
}

impl<W: LaneWriter> PersistentWritePool<W> {
    fn new(writer: W, lane_count: usize, config: InsertBatchingConfig) -> Self {
        let lane_count = lane_count.max(1);
        // Each lane can have at most one completion waiting to be consumed. Keeping the channel
        // bounded makes acknowledgement delivery part of the writer's backpressure chain.
        let (event_sender, events) = mpsc::channel(lane_count);
        let mut senders = Vec::with_capacity(lane_count);
        let mut handles = Vec::with_capacity(lane_count);
        for lane in 0..lane_count {
            let (job_sender, job_receiver) = mpsc::channel(1);
            senders.push(job_sender);
            handles.push(tokio::task::spawn_local(run_lane(
                lane,
                writer.clone(),
                job_receiver,
                event_sender.clone(),
            )));
        }
        Self {
            writer,
            config,
            pending: None,
            waiting_jobs: VecDeque::new(),
            senders,
            events,
            completed_groups: VecDeque::with_capacity(lane_count),
            completed_group_capacity: lane_count,
            completions_since_yield: 0,
            handles,
            available_lanes: (0..lane_count).collect(),
            outstanding: 0,
        }
    }

    fn is_at_capacity(&self) -> bool {
        !self.waiting_jobs.is_empty()
    }

    /// Assigns queued insertion groups to idle lanes without awaiting capacity.
    ///
    /// Keeping this operation non-blocking lets the exporter continue receiving
    /// control messages while every ClickHouse lane is busy.
    fn schedule_waiting_jobs(&mut self) {
        while !self.waiting_jobs.is_empty() && !self.available_lanes.is_empty() {
            let lane = self
                .available_lanes
                .pop_front()
                .expect("an available writer lane is present");
            let job = self
                .waiting_jobs
                .pop_front()
                .expect("a queued writer job is present");
            match self.senders[lane].try_send(job) {
                Ok(()) => {}
                Err(mpsc::error::TrySendError::Full(returned_job)) => {
                    self.waiting_jobs.push_front(returned_job);
                    self.available_lanes.push_front(lane);
                    break;
                }
                Err(mpsc::error::TrySendError::Closed(returned_job)) => {
                    self.completed_groups.push_back(returned_job.fail(lane));
                }
            }
        }
    }

    fn flush_pending(&mut self) {
        if let Some(insertion) = self.pending.take() {
            self.waiting_jobs.push_back(LaneJob::Coalesced(insertion));
            self.schedule_waiting_jobs();
        }
    }

    fn submit(&mut self, job: WriteJob) -> Option<CompletedWrite> {
        let WriteJob {
            pdata,
            export_started_at,
            batches,
        } = job;
        let mut mapped_batches = Vec::new();
        for (payload_type, batch) in batches {
            if let Some(table_name) = self.writer.destination_table(payload_type) {
                mapped_batches.push((payload_type, table_name.to_string(), batch));
            }
        }

        match mapped_batches.len() {
            0 => Some(CompletedWrite {
                pdata,
                export_started_at,
                result: Ok(Vec::new()),
            }),
            1 => {
                self.outstanding = self.outstanding.saturating_add(1);
                let (payload_type, table_name, batch) =
                    mapped_batches.pop().expect("one mapped batch is present");

                if self
                    .pending
                    .as_ref()
                    .is_some_and(|pending| !pending.is_compatible(&table_name, &batch))
                {
                    self.flush_pending();
                }

                if let Some(pending) = self.pending.as_mut() {
                    pending.append(pdata, export_started_at, payload_type, batch);
                } else {
                    self.pending = Some(PendingInsertion::new(
                        self.config,
                        pdata,
                        export_started_at,
                        payload_type,
                        table_name,
                        batch,
                    ));
                }

                if self
                    .pending
                    .as_ref()
                    .is_some_and(|pending| pending.threshold_reached(self.config))
                {
                    self.flush_pending();
                }
                None
            }
            _ => {
                self.outstanding = self.outstanding.saturating_add(1);
                self.flush_pending();
                self.waiting_jobs.push_back(LaneJob::Immediate {
                    pdata,
                    export_started_at,
                    batches: mapped_batches,
                });
                self.schedule_waiting_jobs();
                None
            }
        }
    }

    fn accept_lane_completion(&mut self, event: LaneCompletion) {
        self.completed_groups.push_back(event.group);
        self.available_lanes.push_back(event.lane);
        self.schedule_waiting_jobs();
    }

    fn pop_completion(&mut self) -> Option<CompletedWrite> {
        while let Some(mut group) = self.completed_groups.pop_front() {
            let completed = group.next_completion();
            if !group.is_empty() {
                self.completed_groups.push_back(group);
            }
            if completed.is_some() {
                self.outstanding = self.outstanding.saturating_sub(1);
                self.completions_since_yield = self.completions_since_yield.saturating_add(1);
                return completed;
            }
        }
        None
    }

    async fn next_event(&mut self) -> Option<DispatcherEvent> {
        if self
            .pending
            .as_ref()
            .is_some_and(|pending| pending.deadline <= Instant::now())
        {
            self.flush_pending();
            return Some(DispatcherEvent::CapacityAvailable);
        }

        if self.completed_groups.len() < self.completed_group_capacity {
            match self.events.try_recv() {
                Ok(event) => self.accept_lane_completion(event),
                Err(mpsc::error::TryRecvError::Empty) => {}
                Err(mpsc::error::TryRecvError::Disconnected)
                    if self.completed_groups.is_empty() =>
                {
                    return None;
                }
                Err(mpsc::error::TryRecvError::Disconnected) => {}
            }
        }

        if !self.completed_groups.is_empty() {
            if self.completions_since_yield >= COMPLETIONS_BEFORE_YIELD {
                tokio::task::yield_now().await;
                self.completions_since_yield = 0;
            }
            return self.pop_completion().map(DispatcherEvent::Completed);
        }

        // Waiting on the bounded lane channel already gives the local runtime a scheduling point.
        self.completions_since_yield = 0;
        let event = if let Some(deadline) = self.pending.as_ref().map(|pending| pending.deadline) {
            tokio::select! {
                biased;

                _ = tokio::time::sleep_until(deadline) => {
                    self.flush_pending();
                    return Some(DispatcherEvent::CapacityAvailable);
                }
                event = self.events.recv() => event,
            }
        } else {
            self.events.recv().await
        };

        let event = event?;
        self.accept_lane_completion(event);
        self.pop_completion().map(DispatcherEvent::Completed)
    }

    async fn next_event_until(
        &mut self,
        deadline: Instant,
    ) -> Result<Option<DispatcherEvent>, usize> {
        match tokio::time::timeout_at(deadline, self.next_event()).await {
            Ok(event) => Ok(event),
            Err(_) => Err(self.outstanding),
        }
    }
}

impl<W: LaneWriter> Drop for PersistentWritePool<W> {
    fn drop(&mut self) {
        for handle in &self.handles {
            handle.abort();
        }
    }
}

pub(super) enum WriteDispatcher {
    Immediate {
        writer: Rc<ClickHouseWriter>,
        in_flight: InFlightWrites,
    },
    Persistent(Box<PersistentWritePool<ClickHouseWriter>>),
}

impl WriteDispatcher {
    pub(super) fn new(
        writer: ClickHouseWriter,
        max_in_flight: NonZeroUsize,
        insert_batching: Option<InsertBatchingConfig>,
    ) -> Self {
        match insert_batching {
            Some(config) => Self::Persistent(Box::new(PersistentWritePool::new(
                writer,
                max_in_flight.get(),
                config,
            ))),
            None => Self::Immediate {
                writer: Rc::new(writer),
                in_flight: InFlightWrites::new(max_in_flight),
            },
        }
    }

    pub(super) fn is_at_capacity(&self) -> bool {
        match self {
            Self::Immediate { in_flight, .. } => in_flight.is_at_capacity(),
            Self::Persistent(pool) => pool.is_at_capacity(),
        }
    }

    pub(super) fn has_pending(&self) -> bool {
        match self {
            Self::Immediate { in_flight, .. } => !in_flight.is_empty(),
            Self::Persistent(pool) => pool.outstanding > 0,
        }
    }

    pub(super) fn submit(
        &mut self,
        pdata: OtapPdata,
        export_started_at: StdInstant,
        batches: HashMap<ArrowPayloadType, RecordBatch>,
    ) -> Option<CompletedWrite> {
        match self {
            Self::Immediate { writer, in_flight } => {
                let writer = Rc::clone(writer);
                let write_future: LocalBoxFuture<'static, CompletedWrite> = Box::pin(async move {
                    CompletedWrite {
                        pdata,
                        export_started_at,
                        result: writer.write_batches(&batches).await.map_err(Rc::new),
                    }
                });
                in_flight.push(write_future);
                None
            }
            Self::Persistent(pool) => pool.submit(WriteJob::new(pdata, export_started_at, batches)),
        }
    }

    pub(super) async fn next_event(&mut self) -> Option<DispatcherEvent> {
        match self {
            Self::Immediate { in_flight, .. } => in_flight
                .next_completion()
                .await
                .map(DispatcherEvent::Completed),
            Self::Persistent(pool) => pool.next_event().await,
        }
    }

    /// Dispatches a partial insertion group before shutdown draining begins.
    pub(super) fn flush_pending(&mut self) {
        if let Self::Persistent(pool) = self {
            pool.flush_pending();
        }
    }

    /// Waits for the next accepted write without exceeding the shutdown deadline.
    ///
    /// The error contains the number of original pdata messages that remain
    /// unresolved when the deadline expires.
    pub(super) async fn next_event_until(
        &mut self,
        deadline: Instant,
    ) -> Result<Option<DispatcherEvent>, usize> {
        match self {
            Self::Immediate { in_flight, .. } => in_flight
                .next_completion_until(deadline)
                .await
                .map(|completed| completed.map(DispatcherEvent::Completed)),
            Self::Persistent(pool) => pool.next_event_until(deadline).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::num::{NonZeroU64, NonZeroUsize};
    use std::rc::Rc;
    use std::sync::Arc;

    use arrow_array::{ArrayRef, Int64Array};
    use arrow_schema::{DataType, Field, Schema};
    use bytes::Bytes;
    use otel_arrow_dfe_config::SignalType;
    use otel_arrow_dfe_pdata::{OtapPayload, OtlpProtoBytes};
    use tokio::task::LocalSet;

    use super::*;

    #[derive(Default)]
    struct FakeState {
        insertions_started: usize,
        insertion_rows: Vec<Vec<usize>>,
        fail_end: bool,
        stall_end: bool,
    }

    #[derive(Clone, Default)]
    struct FakeWriter {
        state: Rc<RefCell<FakeState>>,
    }

    struct FakeInsert {
        state: Rc<RefCell<FakeState>>,
        rows: Vec<usize>,
    }

    #[async_trait(?Send)]
    impl LaneInsert for FakeInsert {
        async fn write(&mut self, batch: &RecordBatch) -> Result<(), clickhouse::error::Error> {
            self.rows.push(batch.num_rows());
            Ok(())
        }

        async fn end(self) -> Result<(), clickhouse::error::Error> {
            if self.state.borrow().stall_end {
                futures::future::pending().await
            }
            let mut state = self.state.borrow_mut();
            if state.fail_end {
                return Err(clickhouse::error::Error::Custom(
                    "injected end failure".to_string(),
                ));
            }
            state.insertion_rows.push(self.rows);
            Ok(())
        }
    }

    impl LaneWriter for FakeWriter {
        type Insert = FakeInsert;

        fn destination_table(&self, payload_type: ArrowPayloadType) -> Option<&str> {
            (payload_type == ArrowPayloadType::Logs).then_some("logs")
        }

        fn start_insert(
            &self,
            _table_name: &str,
        ) -> Result<Self::Insert, clickhouse::error::Error> {
            self.state.borrow_mut().insertions_started += 1;
            Ok(FakeInsert {
                state: Rc::clone(&self.state),
                rows: Vec::new(),
            })
        }
    }

    fn config(max_rows: usize, max_bytes: usize, max_delay_ms: u64) -> InsertBatchingConfig {
        InsertBatchingConfig {
            max_rows: NonZeroUsize::new(max_rows).unwrap(),
            max_bytes: NonZeroUsize::new(max_bytes).unwrap(),
            max_delay_ms: NonZeroU64::new(max_delay_ms).unwrap(),
        }
    }

    fn batch(rows: usize) -> RecordBatch {
        batch_with_field(rows, "value")
    }

    fn batch_with_field(rows: usize, field_name: &str) -> RecordBatch {
        let schema = Arc::new(Schema::new(vec![Field::new(
            field_name,
            DataType::Int64,
            false,
        )]));
        let values: ArrayRef = Arc::new(Int64Array::from_iter_values(0..rows as i64));
        RecordBatch::try_new(schema, vec![values]).unwrap()
    }

    fn logs_pdata() -> OtapPdata {
        OtapPdata::new_todo_context(OtapPayload::from(OtlpProtoBytes::ExportLogsRequest(
            Bytes::new(),
        )))
    }

    fn logs_job(rows: usize) -> WriteJob {
        WriteJob::new(
            logs_pdata(),
            StdInstant::now(),
            HashMap::from([(ArrowPayloadType::Logs, batch(rows))]),
        )
    }

    fn submit(pool: &mut PersistentWritePool<FakeWriter>, job: WriteJob) {
        assert!(pool.submit(job).is_none());
    }

    async fn next_completed(pool: &mut PersistentWritePool<FakeWriter>) -> CompletedWrite {
        loop {
            match pool.next_event().await.expect("a write should complete") {
                DispatcherEvent::CapacityAvailable => {}
                DispatcherEvent::Completed(completed) => return completed,
            }
        }
    }

    /// Scenario: two compatible log batches reach the row threshold with sixteen writer lanes.
    /// Guarantees: pre-lane coalescing puts both batches in one insertion and completes both inputs.
    #[tokio::test(flavor = "current_thread")]
    async fn row_threshold_groups_compatible_batches() {
        LocalSet::new()
            .run_until(async {
                let writer = FakeWriter::default();
                let mut pool =
                    PersistentWritePool::new(writer.clone(), 16, config(5, usize::MAX, 60_000));

                submit(&mut pool, logs_job(2));
                submit(&mut pool, logs_job(3));

                assert!(next_completed(&mut pool).await.result.is_ok());
                assert!(next_completed(&mut pool).await.result.is_ok());
                assert_eq!(writer.state.borrow().insertion_rows, vec![vec![2, 3]]);
            })
            .await;
    }

    /// Scenario: compatible batches jointly reach the configured estimated Arrow byte threshold.
    /// Guarantees: the coalescer dispatches their shared insertion without another threshold.
    #[tokio::test(flavor = "current_thread")]
    async fn byte_threshold_closes_the_insertion() {
        LocalSet::new()
            .run_until(async {
                let first = batch(2);
                let second = batch(3);
                let byte_limit = first
                    .get_array_memory_size()
                    .saturating_add(second.get_array_memory_size());
                let writer = FakeWriter::default();
                let mut pool = PersistentWritePool::new(
                    writer.clone(),
                    1,
                    config(usize::MAX, byte_limit, 60_000),
                );

                submit(
                    &mut pool,
                    WriteJob::new(
                        logs_pdata(),
                        StdInstant::now(),
                        HashMap::from([(ArrowPayloadType::Logs, first)]),
                    ),
                );
                submit(
                    &mut pool,
                    WriteJob::new(
                        logs_pdata(),
                        StdInstant::now(),
                        HashMap::from([(ArrowPayloadType::Logs, second)]),
                    ),
                );

                assert!(next_completed(&mut pool).await.result.is_ok());
                assert!(next_completed(&mut pool).await.result.is_ok());
                assert_eq!(writer.state.borrow().insertion_rows, vec![vec![2, 3]]);
            })
            .await;
    }

    /// Scenario: a global insertion remains below row and byte thresholds with sixteen lanes.
    /// Guarantees: its elapsed-time threshold dispatches it and completes the pending message.
    #[tokio::test(flavor = "current_thread", start_paused = true)]
    async fn time_threshold_closes_a_partial_insertion() {
        LocalSet::new()
            .run_until(async {
                let writer = FakeWriter::default();
                let mut pool = PersistentWritePool::new(
                    writer.clone(),
                    16,
                    config(usize::MAX, usize::MAX, 50),
                );

                submit(&mut pool, logs_job(2));
                while pool.is_at_capacity() {
                    let _ = pool.next_event().await;
                }
                tokio::task::yield_now().await;
                tokio::time::advance(Duration::from_millis(50)).await;

                assert!(next_completed(&mut pool).await.result.is_ok());
                assert_eq!(writer.state.borrow().insertion_rows, vec![vec![2]]);
            })
            .await;
    }

    /// Scenario: an internal caller bypasses parsing with the largest possible insertion delay.
    /// Guarantees: platform-specific Instant limits cannot make deadline construction panic.
    #[test]
    fn oversized_programmatic_delay_is_panic_free() {
        let insertion = PendingInsertion::new(
            config(usize::MAX, usize::MAX, u64::MAX),
            logs_pdata(),
            StdInstant::now(),
            ArrowPayloadType::Logs,
            "logs".to_string(),
            batch(2),
        );

        assert_eq!(insertion.rows, 2);
    }

    /// Scenario: shutdown arrives while the coalescer holds a group below every threshold.
    /// Guarantees: shutdown dispatches that insertion and retains the input message's completion.
    #[tokio::test(flavor = "current_thread")]
    async fn shutdown_flushes_a_partial_insertion() {
        LocalSet::new()
            .run_until(async {
                let writer = FakeWriter::default();
                let mut pool = PersistentWritePool::new(
                    writer.clone(),
                    1,
                    config(usize::MAX, usize::MAX, 60_000),
                );

                submit(&mut pool, logs_job(2));
                pool.flush_pending();

                assert!(next_completed(&mut pool).await.result.is_ok());
                assert_eq!(writer.state.borrow().insertion_rows, vec![vec![2]]);
            })
            .await;
    }

    /// Scenario: shutdown drains a dispatched insertion whose ClickHouse response never arrives.
    /// Guarantees: persistent-lane draining stops at the deadline and reports the unresolved input.
    #[tokio::test(flavor = "current_thread", start_paused = true)]
    async fn shutdown_drain_stops_at_deadline() {
        LocalSet::new()
            .run_until(async {
                let writer = FakeWriter::default();
                writer.state.borrow_mut().stall_end = true;
                let mut pool = PersistentWritePool::new(writer, 1, config(1, usize::MAX, 60_000));

                submit(&mut pool, logs_job(1));
                let deadline = Instant::now() + Duration::from_secs(5);
                let abandoned = match pool.next_event_until(deadline).await {
                    Err(abandoned) => abandoned,
                    Ok(_) => panic!("the stalled insertion must reach the shutdown deadline"),
                };

                assert_eq!(abandoned, 1);
            })
            .await;
    }

    /// Scenario: ClickHouse rejects the final response for an insertion containing two messages.
    /// Guarantees: every grouped message shares the same structured ClickHouse failure.
    #[tokio::test(flavor = "current_thread")]
    async fn insertion_response_failure_is_reported_to_every_grouped_message() {
        LocalSet::new()
            .run_until(async {
                let writer = FakeWriter::default();
                writer.state.borrow_mut().fail_end = true;
                let mut pool =
                    PersistentWritePool::new(writer.clone(), 1, config(5, usize::MAX, 60_000));

                submit(&mut pool, logs_job(2));
                submit(&mut pool, logs_job(3));

                let first_error = next_completed(&mut pool)
                    .await
                    .result
                    .expect_err("the first message should fail");
                let second_error = next_completed(&mut pool)
                    .await
                    .result
                    .expect_err("the second message should fail");
                assert!(Rc::ptr_eq(&first_error, &second_error));
                assert!(matches!(
                    first_error.as_ref(),
                    ClickhouseExporterError::InsertResponseError {
                        source: clickhouse::error::Error::Custom(message)
                    } if message == "injected end failure"
                ));
                assert_eq!(writer.state.borrow().insertions_started, 1);
            })
            .await;
    }

    /// Scenario: a batch arrives whose Arrow schema differs from the pending insertion group.
    /// Guarantees: the incompatible batch starts a new group instead of sharing the insertion.
    #[tokio::test(flavor = "current_thread")]
    async fn schema_change_closes_the_current_insertion() {
        LocalSet::new()
            .run_until(async {
                let writer = FakeWriter::default();
                let mut pool = PersistentWritePool::new(
                    writer.clone(),
                    1,
                    config(usize::MAX, usize::MAX, 60_000),
                );

                submit(&mut pool, logs_job(2));
                submit(
                    &mut pool,
                    WriteJob::new(
                        logs_pdata(),
                        StdInstant::now(),
                        HashMap::from([(
                            ArrowPayloadType::Logs,
                            batch_with_field(3, "other_value"),
                        )]),
                    ),
                );

                assert!(next_completed(&mut pool).await.result.is_ok());
                pool.flush_pending();
                assert!(next_completed(&mut pool).await.result.is_ok());
                assert_eq!(writer.state.borrow().insertion_rows, vec![vec![2], vec![3]]);
            })
            .await;
    }

    /// Scenario: a second complete insertion group is ready while the only writer lane is busy.
    /// Guarantees: submission remains non-blocking, admission reports backpressure, and all inputs complete.
    #[tokio::test(flavor = "current_thread")]
    async fn full_lane_applies_backpressure_without_rejecting_the_batch() {
        LocalSet::new()
            .run_until(async {
                let writer = FakeWriter::default();
                let mut pool =
                    PersistentWritePool::new(writer.clone(), 1, config(4, usize::MAX, 60_000));

                assert!(pool.submit(logs_job(2)).is_none());
                assert!(pool.submit(logs_job(2)).is_none());
                assert!(pool.submit(logs_job(2)).is_none());
                assert!(pool.submit(logs_job(2)).is_none());
                assert!(pool.is_at_capacity());

                for _ in 0..4 {
                    let completed = next_completed(&mut pool).await;
                    assert!(completed.result.is_ok());
                    assert_eq!(completed.pdata.signal_type(), SignalType::Logs);
                }
                assert_eq!(
                    writer.state.borrow().insertion_rows,
                    vec![vec![2, 2], vec![2, 2]]
                );
            })
            .await;
    }

    /// Scenario: one lane completes a three-message insertion while another group is queued.
    /// Guarantees: one bounded event releases the lane before individual completions finish draining.
    #[tokio::test(flavor = "current_thread")]
    async fn aggregate_completion_releases_lane_before_ack_fanout_finishes() {
        LocalSet::new()
            .run_until(async {
                let writer = FakeWriter::default();
                let mut pool =
                    PersistentWritePool::new(writer.clone(), 1, config(3, usize::MAX, 60_000));
                assert_eq!(pool.events.max_capacity(), 1);

                for _ in 0..6 {
                    submit(&mut pool, logs_job(1));
                }
                assert!(pool.is_at_capacity());

                assert!(next_completed(&mut pool).await.result.is_ok());
                tokio::task::yield_now().await;

                assert_eq!(writer.state.borrow().insertions_started, 2);
                assert_eq!(pool.outstanding, 5);

                for _ in 0..5 {
                    assert!(next_completed(&mut pool).await.result.is_ok());
                }
                assert_eq!(writer.state.borrow().insertion_rows, vec![vec![1; 3]; 2]);
            })
            .await;
    }
}
