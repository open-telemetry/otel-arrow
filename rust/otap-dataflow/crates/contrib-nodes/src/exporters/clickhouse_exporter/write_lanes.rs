// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Optional pre-lane coalescing and concurrent ClickHouse insertion writers.

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

#[allow(
    clippy::large_enum_variant,
    reason = "boxing CompletedWrite would allocate on every completed insertion"
)]
enum WriteEvent {
    LaneAvailable(usize),
    Completed(CompletedWrite),
}

#[allow(
    clippy::large_enum_variant,
    reason = "boxing CompletedWrite would allocate on every completed insertion"
)]
pub(super) enum DispatcherEvent {
    CapacityAvailable,
    Completed(CompletedWrite),
}

#[async_trait(?Send)]
pub(super) trait LaneInsert: Sized {
    async fn write(&mut self, batch: &RecordBatch) -> Result<(), String>;
    async fn end(self) -> Result<(), String>;
}

#[async_trait(?Send)]
impl LaneInsert for ArrowInsert {
    async fn write(&mut self, batch: &RecordBatch) -> Result<(), String> {
        self.write(batch).await.map_err(|error| error.to_string())
    }

    async fn end(self) -> Result<(), String> {
        self.end().await.map_err(|error| error.to_string())
    }
}

pub(super) trait LaneWriter: Clone + 'static {
    type Insert: LaneInsert + 'static;

    fn destination_table(&self, payload_type: ArrowPayloadType) -> Option<&str>;
    fn start_insert(&self, table_name: &str) -> Result<Self::Insert, String>;
}

impl LaneWriter for ClickHouseWriter {
    type Insert = ArrowInsert;

    fn destination_table(&self, payload_type: ArrowPayloadType) -> Option<&str> {
        self.destination_table(payload_type)
    }

    fn start_insert(&self, table_name: &str) -> Result<Self::Insert, String> {
        self.start_insert(table_name)
            .map_err(|error| error.to_string())
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
        Self {
            table_name,
            schema,
            rows,
            bytes,
            batches: vec![batch],
            deadline: Instant::now() + Duration::from_millis(config.max_delay_ms.get()),
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

fn emit_completion(events: &mpsc::UnboundedSender<WriteEvent>, completed: CompletedWrite) {
    let _ = events.send(WriteEvent::Completed(completed));
}

fn emit_failure(
    events: &mpsc::UnboundedSender<WriteEvent>,
    pending: Vec<PendingWrite>,
    response_error: bool,
    error: String,
) {
    for pending_write in pending {
        let error = if response_error {
            ClickhouseExporterError::InsertResponseError {
                error: error.clone(),
            }
        } else {
            ClickhouseExporterError::InsertRequestError {
                error: error.clone(),
            }
        };
        emit_completion(
            events,
            CompletedWrite {
                pdata: pending_write.pdata,
                export_started_at: pending_write.export_started_at,
                result: Err(error),
            },
        );
    }
}

async fn write_insertion<I: LaneInsert>(
    mut insert: I,
    insertion: PendingInsertion,
    events: &mpsc::UnboundedSender<WriteEvent>,
) {
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
            emit_failure(events, pending, false, error);
            return;
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
            for pending_write in pending {
                emit_completion(
                    events,
                    CompletedWrite {
                        pdata: pending_write.pdata,
                        export_started_at: pending_write.export_started_at,
                        result: Ok(pending_write.written_rows),
                    },
                );
            }
        }
        Err(error) => emit_failure(events, pending, true, error),
    }
}

async fn write_coalesced<W: LaneWriter>(
    writer: &W,
    insertion: PendingInsertion,
    events: &mpsc::UnboundedSender<WriteEvent>,
) {
    let insert = match writer.start_insert(&insertion.table_name) {
        Ok(insert) => insert,
        Err(error) => {
            emit_failure(events, insertion.pending, false, error);
            return;
        }
    };
    write_insertion(insert, insertion, events).await;
}

async fn write_immediately<W: LaneWriter>(
    writer: &W,
    pdata: OtapPdata,
    export_started_at: StdInstant,
    batches: Vec<(ArrowPayloadType, String, RecordBatch)>,
    events: &mpsc::UnboundedSender<WriteEvent>,
) {
    let mut written_rows = Vec::with_capacity(batches.len());
    for (payload_type, table_name, batch) in batches {
        let result = async {
            let mut insert = writer
                .start_insert(&table_name)
                .map_err(|error| ClickhouseExporterError::InsertRequestError { error })?;
            insert
                .write(&batch)
                .await
                .map_err(|error| ClickhouseExporterError::InsertRequestError { error })?;
            insert
                .end()
                .await
                .map_err(|error| ClickhouseExporterError::InsertResponseError { error })
        }
        .await;

        if let Err(error) = result {
            emit_completion(
                events,
                CompletedWrite {
                    pdata,
                    export_started_at,
                    result: Err(error),
                },
            );
            return;
        }
        written_rows.push((payload_type, batch.num_rows() as u64));
    }

    emit_completion(
        events,
        CompletedWrite {
            pdata,
            export_started_at,
            result: Ok(written_rows),
        },
    );
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
    fn fail(self, events: &mpsc::UnboundedSender<WriteEvent>, error: String) {
        match self {
            Self::Coalesced(insertion) => {
                emit_failure(events, insertion.pending, false, error);
            }
            Self::Immediate {
                pdata,
                export_started_at,
                ..
            } => emit_completion(
                events,
                CompletedWrite {
                    pdata,
                    export_started_at,
                    result: Err(ClickhouseExporterError::InsertRequestError { error }),
                },
            ),
        }
    }
}

async fn run_lane<W: LaneWriter>(
    lane: usize,
    writer: W,
    mut jobs: mpsc::Receiver<LaneJob>,
    events: mpsc::UnboundedSender<WriteEvent>,
) {
    while let Some(job) = jobs.recv().await {
        match job {
            LaneJob::Coalesced(insertion) => {
                write_coalesced(&writer, insertion, &events).await;
            }
            LaneJob::Immediate {
                pdata,
                export_started_at,
                batches,
            } => {
                write_immediately(&writer, pdata, export_started_at, batches, &events).await;
            }
        }
        let _ = events.send(WriteEvent::LaneAvailable(lane));
    }
}

pub(super) struct PersistentWritePool<W: LaneWriter> {
    writer: W,
    config: InsertBatchingConfig,
    pending: Option<PendingInsertion>,
    waiting_jobs: VecDeque<LaneJob>,
    senders: Vec<mpsc::Sender<LaneJob>>,
    event_sender: mpsc::UnboundedSender<WriteEvent>,
    events: mpsc::UnboundedReceiver<WriteEvent>,
    handles: Vec<JoinHandle<()>>,
    available_lanes: VecDeque<usize>,
    outstanding: usize,
}

impl<W: LaneWriter> PersistentWritePool<W> {
    fn new(writer: W, lane_count: usize, config: InsertBatchingConfig) -> Self {
        let lane_count = lane_count.max(1);
        let (event_sender, events) = mpsc::unbounded_channel();
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
            event_sender,
            events,
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
                    returned_job.fail(
                        &self.event_sender,
                        format!("ClickHouse writer lane {lane} is closed"),
                    );
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

    async fn next_event(&mut self) -> Option<DispatcherEvent> {
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
        match event {
            WriteEvent::LaneAvailable(lane) => {
                self.available_lanes.push_back(lane);
                self.schedule_waiting_jobs();
                Some(DispatcherEvent::CapacityAvailable)
            }
            WriteEvent::Completed(completed) => {
                self.outstanding = self.outstanding.saturating_sub(1);
                Some(DispatcherEvent::Completed(completed))
            }
        }
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
                        result: writer.write_batches(&batches).await,
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
        async fn write(&mut self, batch: &RecordBatch) -> Result<(), String> {
            self.rows.push(batch.num_rows());
            Ok(())
        }

        async fn end(self) -> Result<(), String> {
            if self.state.borrow().stall_end {
                futures::future::pending().await
            }
            let mut state = self.state.borrow_mut();
            if state.fail_end {
                return Err("injected end failure".to_string());
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

        fn start_insert(&self, _table_name: &str) -> Result<Self::Insert, String> {
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
    /// Guarantees: every message grouped into that insertion is reported as failed.
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

                assert!(next_completed(&mut pool).await.result.is_err());
                assert!(next_completed(&mut pool).await.result.is_err());
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
}
