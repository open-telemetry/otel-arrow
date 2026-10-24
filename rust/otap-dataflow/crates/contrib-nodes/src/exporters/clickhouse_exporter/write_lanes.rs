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
//! - With `insert_batching`, `BatchedWritePool` first coalesces compatible batches and then
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
//! group, then runs as one uncoalesced lane job so its original message still has one completion.
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
//! During normal operation, every accepted pdata message produces exactly one `CompletedWrite`. A
//! coalesced insertion keeps the original pdata and row count for each member in `PendingWrite`;
//! all members receive the same insertion outcome. The exceptions are an expired shutdown deadline
//! or a terminal dispatcher failure, both of which stop the exporter with unresolved work reported
//! or surfaced as an exporter error rather than a successful drain.
//!
//! Completions from different lanes can arrive out of input order. Members of one completed group
//! retain their original order, but incremental group fan-out can interleave them with members of
//! other completed groups. A pdata message mapped to multiple ClickHouse tables bypasses
//! coalescing. Those table insertions are sequential but not atomic: a later table can fail after
//! an earlier table has committed, and retrying the failed pdata may duplicate those committed
//! rows.
//!
//! Structured insertion failures are shared with `Rc` because the entire path runs on the
//! exporter's local runtime. Start/write failures are classified as request errors, while `end()`
//! failures are response errors, and the ClickHouse client source chain is retained.
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
//! deadline reports how many delivery outcomes remain unresolved. Writer tasks are supervised:
//! an unexpected exit, cancellation, or panic becomes a terminal exporter error instead of a
//! disconnected completion stream or a busy loop. Dropping the pool aborts its local lane tasks
//! after draining finishes or the deadline expires.

use std::any::Any;
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
use tokio::task::{Id as TaskId, JoinError, JoinSet};
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
    PendingFlushed,
    Completed(CompletedWrite),
    Failed(Rc<ClickhouseExporterError>),
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
    let message_count = pending.len();

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
                messages = message_count,
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

/// Writes every mapped destination for one pdata message without coalescing.
///
/// Each destination uses an independent ClickHouse insertion. A failure after an earlier
/// destination commits fails the original pdata message but cannot roll back the committed rows.
async fn write_uncoalesced<W: LaneWriter>(
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

/// One closed unit of ClickHouse work waiting for a writer lane.
enum LaneJob {
    Coalesced(PendingInsertion),
    Uncoalesced {
        pdata: OtapPdata,
        export_started_at: StdInstant,
        batches: Vec<(ArrowPayloadType, String, RecordBatch)>,
    },
}

impl LaneJob {
    fn fail(self, lane: usize) -> CompletedGroup {
        match self {
            Self::Coalesced(insertion) => CompletedGroup::lane_closed(insertion.pending, lane),
            Self::Uncoalesced {
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
            LaneJob::Uncoalesced {
                pdata,
                export_started_at,
                batches,
            } => write_uncoalesced(&writer, pdata, export_started_at, batches).await,
        };
        if events.send(LaneCompletion { lane, group }).await.is_err() {
            break;
        }
    }
}

/// Coalesces compatible batches and schedules closed groups across local writer tasks.
///
/// `available_lanes`, `waiting_jobs`, and `worker_lanes` jointly describe lane ownership. A lane
/// appears in `available_lanes` only when it owns no job. Normal pdata admission pauses as soon as
/// `waiting_jobs` becomes non-empty, while the bounded exporter inbox constrains additional jobs
/// that can be force-drained during shutdown.
pub(super) struct BatchedWritePool<W: LaneWriter> {
    writer: W,
    config: InsertBatchingConfig,
    pending: Option<PendingInsertion>,
    waiting_jobs: VecDeque<LaneJob>,
    senders: Vec<mpsc::Sender<LaneJob>>,
    events: mpsc::Receiver<LaneCompletion>,
    completed_groups: VecDeque<CompletedGroup>,
    completed_group_capacity: usize,
    completions_since_yield: usize,
    workers: JoinSet<()>,
    worker_lanes: HashMap<TaskId, usize>,
    available_lanes: VecDeque<usize>,
    outstanding: usize,
}

impl<W: LaneWriter> BatchedWritePool<W> {
    fn new(writer: W, lane_count: usize, config: InsertBatchingConfig) -> Self {
        let lane_count = lane_count.max(1);
        // Each lane can have at most one completion waiting to be consumed. Keeping the channel
        // bounded makes acknowledgement delivery part of the writer's backpressure chain.
        let (event_sender, events) = mpsc::channel(lane_count);
        let mut senders = Vec::with_capacity(lane_count);
        let mut workers = JoinSet::new();
        let mut worker_lanes = HashMap::with_capacity(lane_count);
        for lane in 0..lane_count {
            let (job_sender, job_receiver) = mpsc::channel(1);
            senders.push(job_sender);
            let abort_handle = workers.spawn_local(run_lane(
                lane,
                writer.clone(),
                job_receiver,
                event_sender.clone(),
            ));
            _ = worker_lanes.insert(abort_handle.id(), lane);
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
            workers,
            worker_lanes,
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
                self.waiting_jobs.push_back(LaneJob::Uncoalesced {
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

    /// Converts any lane-task termination into a structured terminal dispatcher failure.
    fn worker_failure(
        &mut self,
        result: Option<Result<(TaskId, ()), JoinError>>,
    ) -> Rc<ClickhouseExporterError> {
        let Some(result) = result else {
            return Rc::new(ClickhouseExporterError::WriterLaneSupervisorError {
                message: "no writer tasks remain".to_string(),
            });
        };

        match result {
            Ok((task_id, ())) => match self.worker_lanes.remove(&task_id) {
                Some(lane) => Rc::new(ClickhouseExporterError::WriterLaneExited { lane }),
                None => Rc::new(ClickhouseExporterError::WriterLaneSupervisorError {
                    message: format!("unregistered writer task {task_id:?} exited"),
                }),
            },
            Err(error) => {
                let task_id = error.id();
                let Some(lane) = self.worker_lanes.remove(&task_id) else {
                    return Rc::new(ClickhouseExporterError::WriterLaneSupervisorError {
                        message: format!("unregistered writer task failed: {error}"),
                    });
                };

                if error.is_panic() {
                    Rc::new(ClickhouseExporterError::WriterLanePanicked {
                        lane,
                        message: panic_message(error.into_panic()),
                    })
                } else {
                    Rc::new(ClickhouseExporterError::WriterLaneCancelled { lane })
                }
            }
        }
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
        if let Some(result) = self.workers.try_join_next_with_id() {
            return Some(DispatcherEvent::Failed(self.worker_failure(Some(result))));
        }

        if self
            .pending
            .as_ref()
            .is_some_and(|pending| pending.deadline <= Instant::now())
        {
            self.flush_pending();
            return Some(DispatcherEvent::PendingFlushed);
        }

        if self.completed_groups.len() < self.completed_group_capacity {
            match self.events.try_recv() {
                Ok(event) => self.accept_lane_completion(event),
                Err(mpsc::error::TryRecvError::Empty) => {}
                Err(mpsc::error::TryRecvError::Disconnected)
                    if self.completed_groups.is_empty() =>
                {
                    let result = self.workers.join_next_with_id().await;
                    return Some(DispatcherEvent::Failed(self.worker_failure(result)));
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

                result = self.workers.join_next_with_id() => {
                    return Some(DispatcherEvent::Failed(self.worker_failure(result)));
                }

                _ = tokio::time::sleep_until(deadline) => {
                    self.flush_pending();
                    return Some(DispatcherEvent::PendingFlushed);
                }
                event = self.events.recv() => event,
            }
        } else {
            tokio::select! {
                biased;

                result = self.workers.join_next_with_id() => {
                    return Some(DispatcherEvent::Failed(self.worker_failure(result)));
                }
                event = self.events.recv() => event,
            }
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

pub(super) enum WriteDispatcher {
    PerMessage {
        writer: Rc<ClickHouseWriter>,
        in_flight: InFlightWrites,
    },
    Batched(Box<BatchedWritePool<ClickHouseWriter>>),
}

impl WriteDispatcher {
    pub(super) fn new(
        writer: ClickHouseWriter,
        max_in_flight: NonZeroUsize,
        insert_batching: Option<InsertBatchingConfig>,
    ) -> Self {
        match insert_batching {
            Some(config) => Self::Batched(Box::new(BatchedWritePool::new(
                writer,
                max_in_flight.get(),
                config,
            ))),
            None => Self::PerMessage {
                writer: Rc::new(writer),
                in_flight: InFlightWrites::new(max_in_flight),
            },
        }
    }

    pub(super) fn is_at_capacity(&self) -> bool {
        match self {
            Self::PerMessage { in_flight, .. } => in_flight.is_at_capacity(),
            Self::Batched(pool) => pool.is_at_capacity(),
        }
    }

    pub(super) fn has_pending(&self) -> bool {
        match self {
            Self::PerMessage { in_flight, .. } => !in_flight.is_empty(),
            Self::Batched(pool) => pool.outstanding > 0,
        }
    }

    pub(super) fn submit(
        &mut self,
        pdata: OtapPdata,
        export_started_at: StdInstant,
        batches: HashMap<ArrowPayloadType, RecordBatch>,
    ) -> Option<CompletedWrite> {
        match self {
            Self::PerMessage { writer, in_flight } => {
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
            Self::Batched(pool) => pool.submit(WriteJob::new(pdata, export_started_at, batches)),
        }
    }

    pub(super) async fn next_event(&mut self) -> Option<DispatcherEvent> {
        match self {
            Self::PerMessage { in_flight, .. } => in_flight
                .next_completion()
                .await
                .map(DispatcherEvent::Completed),
            Self::Batched(pool) => pool.next_event().await,
        }
    }

    /// Dispatches a partial insertion group before shutdown draining begins.
    pub(super) fn flush_pending(&mut self) {
        if let Self::Batched(pool) = self {
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
            Self::PerMessage { in_flight, .. } => in_flight
                .next_completion_until(deadline)
                .await
                .map(|completed| completed.map(DispatcherEvent::Completed)),
            Self::Batched(pool) => pool.next_event_until(deadline).await,
        }
    }
}

fn panic_message(payload: Box<dyn Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "non-string panic payload".to_string()
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
        fail_start: bool,
        fail_end: bool,
        fail_end_at: Option<usize>,
        stall_end: bool,
        panic_end: bool,
        delay_rows: Option<usize>,
    }

    #[derive(Clone, Default)]
    struct FakeWriter {
        state: Rc<RefCell<FakeState>>,
        release_delayed: Rc<tokio::sync::Notify>,
    }

    struct FakeInsert {
        state: Rc<RefCell<FakeState>>,
        release_delayed: Rc<tokio::sync::Notify>,
        insertion_index: usize,
        rows: Vec<usize>,
    }

    #[async_trait(?Send)]
    impl LaneInsert for FakeInsert {
        async fn write(&mut self, batch: &RecordBatch) -> Result<(), clickhouse::error::Error> {
            self.rows.push(batch.num_rows());
            Ok(())
        }

        async fn end(self) -> Result<(), clickhouse::error::Error> {
            let (stall_end, panic_end, delay_rows) = {
                let state = self.state.borrow();
                (state.stall_end, state.panic_end, state.delay_rows)
            };
            assert!(!panic_end, "injected writer-lane panic");
            if stall_end {
                futures::future::pending().await
            }
            if delay_rows.is_some_and(|rows| self.rows == [rows]) {
                self.release_delayed.notified().await;
            }
            let mut state = self.state.borrow_mut();
            if state.fail_end || state.fail_end_at == Some(self.insertion_index) {
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
            match payload_type {
                ArrowPayloadType::Logs => Some("logs"),
                ArrowPayloadType::Spans => Some("spans"),
                _ => None,
            }
        }

        fn start_insert(
            &self,
            _table_name: &str,
        ) -> Result<Self::Insert, clickhouse::error::Error> {
            let insertion_index = {
                let mut state = self.state.borrow_mut();
                if state.fail_start {
                    return Err(clickhouse::error::Error::Custom(
                        "injected start failure".to_string(),
                    ));
                }
                let insertion_index = state.insertions_started;
                state.insertions_started += 1;
                insertion_index
            };
            Ok(FakeInsert {
                state: Rc::clone(&self.state),
                release_delayed: Rc::clone(&self.release_delayed),
                insertion_index,
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

    fn multi_table_job(log_rows: usize, span_rows: usize) -> WriteJob {
        WriteJob::new(
            logs_pdata(),
            StdInstant::now(),
            HashMap::from([
                (ArrowPayloadType::Logs, batch(log_rows)),
                (ArrowPayloadType::Spans, batch(span_rows)),
            ]),
        )
    }

    fn submit(pool: &mut BatchedWritePool<FakeWriter>, job: WriteJob) {
        assert!(pool.submit(job).is_none());
    }

    async fn next_completed(pool: &mut BatchedWritePool<FakeWriter>) -> CompletedWrite {
        loop {
            match pool.next_event().await.expect("a write should complete") {
                DispatcherEvent::PendingFlushed => {}
                DispatcherEvent::Completed(completed) => return completed,
                DispatcherEvent::Failed(error) => panic!("writer lane failed: {error}"),
            }
        }
    }

    /// Scenario: a transformed message contains no batch mapped to a configured table.
    /// Guarantees: the message completes successfully without entering a lane or changing outstanding work.
    #[tokio::test(flavor = "current_thread")]
    async fn unmapped_message_completes_without_lane_work() {
        LocalSet::new()
            .run_until(async {
                let writer = FakeWriter::default();
                let mut pool = BatchedWritePool::new(
                    writer.clone(),
                    1,
                    config(usize::MAX, usize::MAX, 60_000),
                );
                let job = WriteJob::new(
                    logs_pdata(),
                    StdInstant::now(),
                    HashMap::from([(ArrowPayloadType::UnivariateMetrics, batch(2))]),
                );

                let completed = pool
                    .submit(job)
                    .expect("an unmapped message should complete immediately");

                assert_eq!(completed.result.expect("completion should succeed"), vec![]);
                assert_eq!(pool.outstanding, 0);
                assert_eq!(writer.state.borrow().insertions_started, 0);
            })
            .await;
    }

    /// Scenario: one message maps to log and span tables and both insertions succeed.
    /// Guarantees: the uncoalesced lane job completes the message once with both written row counts.
    #[tokio::test(flavor = "current_thread")]
    async fn multi_table_message_completes_after_every_insertion() {
        LocalSet::new()
            .run_until(async {
                let writer = FakeWriter::default();
                let mut pool = BatchedWritePool::new(
                    writer.clone(),
                    1,
                    config(usize::MAX, usize::MAX, 60_000),
                );

                submit(&mut pool, multi_table_job(2, 3));
                let written_rows = next_completed(&mut pool)
                    .await
                    .result
                    .expect("both table insertions should succeed");

                assert_eq!(written_rows.len(), 2);
                assert!(written_rows.contains(&(ArrowPayloadType::Logs, 2)));
                assert!(written_rows.contains(&(ArrowPayloadType::Spans, 3)));
                assert_eq!(writer.state.borrow().insertions_started, 2);
            })
            .await;
    }

    /// Scenario: the second insertion for a multi-table message fails after the first commits.
    /// Guarantees: the message fails once while the already committed insertion remains observable.
    #[tokio::test(flavor = "current_thread")]
    async fn multi_table_partial_commit_is_reported_as_message_failure() {
        LocalSet::new()
            .run_until(async {
                let writer = FakeWriter::default();
                writer.state.borrow_mut().fail_end_at = Some(1);
                let mut pool = BatchedWritePool::new(
                    writer.clone(),
                    1,
                    config(usize::MAX, usize::MAX, 60_000),
                );

                submit(&mut pool, multi_table_job(2, 3));
                let error = next_completed(&mut pool)
                    .await
                    .result
                    .expect_err("the second table insertion should fail");

                assert!(matches!(
                    error.as_ref(),
                    ClickhouseExporterError::InsertResponseError {
                        source: clickhouse::error::Error::Custom(message)
                    } if message == "injected end failure"
                ));
                assert_eq!(writer.state.borrow().insertions_started, 2);
                assert_eq!(writer.state.borrow().insertion_rows.len(), 1);
            })
            .await;
    }

    /// Scenario: two writer lanes receive groups whose first insertion is deliberately delayed.
    /// Guarantees: lanes execute concurrently and completions may arrive out of input order.
    #[tokio::test(flavor = "current_thread")]
    async fn concurrent_lanes_can_complete_out_of_input_order() {
        LocalSet::new()
            .run_until(async {
                let writer = FakeWriter::default();
                writer.state.borrow_mut().delay_rows = Some(1);
                let mut pool =
                    BatchedWritePool::new(writer.clone(), 2, config(1, usize::MAX, 60_000));

                submit(&mut pool, logs_job(1));
                submit(&mut pool, logs_job(2));

                let second = next_completed(&mut pool)
                    .await
                    .result
                    .expect("the undelayed insertion should succeed");
                assert_eq!(second, vec![(ArrowPayloadType::Logs, 2)]);
                assert_eq!(writer.state.borrow().insertions_started, 2);

                writer.release_delayed.notify_one();
                let first = next_completed(&mut pool)
                    .await
                    .result
                    .expect("the released insertion should succeed");
                assert_eq!(first, vec![(ArrowPayloadType::Logs, 1)]);
            })
            .await;
    }

    /// Scenario: starting a shared insertion fails for a group containing two messages.
    /// Guarantees: every grouped message receives the same structured request failure.
    #[tokio::test(flavor = "current_thread")]
    async fn insertion_start_failure_is_reported_to_every_grouped_message() {
        LocalSet::new()
            .run_until(async {
                let writer = FakeWriter::default();
                writer.state.borrow_mut().fail_start = true;
                let mut pool = BatchedWritePool::new(writer, 1, config(5, usize::MAX, 60_000));

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
                    ClickhouseExporterError::InsertRequestError {
                        source: clickhouse::error::Error::Custom(message)
                    } if message == "injected start failure"
                ));
            })
            .await;
    }

    /// Scenario: a writer lane panics while ending an insertion with accepted pdata.
    /// Guarantees: task supervision surfaces a structured terminal failure instead of disconnecting silently.
    #[tokio::test(flavor = "current_thread")]
    async fn writer_lane_panic_is_a_terminal_dispatcher_failure() {
        LocalSet::new()
            .run_until(async {
                let writer = FakeWriter::default();
                writer.state.borrow_mut().panic_end = true;
                let mut pool = BatchedWritePool::new(writer, 1, config(1, usize::MAX, 60_000));

                submit(&mut pool, logs_job(1));
                let event = pool
                    .next_event()
                    .await
                    .expect("task supervision should produce a terminal event");

                assert!(matches!(
                    event,
                    DispatcherEvent::Failed(error)
                        if matches!(
                            error.as_ref(),
                            ClickhouseExporterError::WriterLanePanicked { lane: 0, message }
                                if message == "injected writer-lane panic"
                        )
                ));
                assert_eq!(pool.outstanding, 1);
            })
            .await;
    }

    /// Scenario: two compatible log batches reach the row threshold with sixteen writer lanes.
    /// Guarantees: pre-lane coalescing puts both batches in one insertion and completes both inputs.
    #[tokio::test(flavor = "current_thread")]
    async fn row_threshold_groups_compatible_batches() {
        LocalSet::new()
            .run_until(async {
                let writer = FakeWriter::default();
                let mut pool =
                    BatchedWritePool::new(writer.clone(), 16, config(5, usize::MAX, 60_000));

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
                let mut pool = BatchedWritePool::new(
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
                let mut pool =
                    BatchedWritePool::new(writer.clone(), 16, config(usize::MAX, usize::MAX, 50));

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
                let mut pool = BatchedWritePool::new(
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
                let mut pool = BatchedWritePool::new(writer, 1, config(1, usize::MAX, 60_000));

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
                    BatchedWritePool::new(writer.clone(), 1, config(5, usize::MAX, 60_000));

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
                let mut pool = BatchedWritePool::new(
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
                    BatchedWritePool::new(writer.clone(), 1, config(4, usize::MAX, 60_000));

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
                    BatchedWritePool::new(writer.clone(), 1, config(3, usize::MAX, 60_000));
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
