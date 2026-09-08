// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared ACK-driven composite watermark receiver core.
//!
//! Delivery is at least once. A page is emitted with a unique batch ID, and the
//! durable cursor advances only after a matching ACK is followed by a
//! successful checkpoint write. A NACK keeps the durable cursor and replays the
//! same page after a fixed backoff, so an unacknowledged row is never skipped.

use super::checkpoint::{CheckpointState, CheckpointStore, SourceLease};
use super::driver::{DriverAdapter, DriverCancellation};
use super::metrics::DatabaseReceiverMetrics;
use super::otlp::{EncodedPage, encode_page, validate_mapping};
use super::page::CompositeCursor;
use super::query::CompiledQuery;
use async_trait::async_trait;
use otel_arrow_dfe_channel::error::SendError;
use otel_arrow_dfe_engine::control::{CallData, Context8u8, NodeControlMsg};
use otel_arrow_dfe_engine::error::{Error, ReceiverErrorKind, TypedError, format_error_sources};
use otel_arrow_dfe_engine::local::receiver as local;
use otel_arrow_dfe_engine::terminal_state::TerminalState;
use otel_arrow_dfe_engine::{Interests, ProducerEffectHandlerExtension};
use otel_arrow_dfe_otap::pdata::OtapPdata;
use otel_arrow_dfe_telemetry::metrics::{MetricSet, MetricSetSnapshot};
use otel_arrow_dfe_telemetry::{otel_debug, otel_info, otel_warn};
use std::future::Future;
use std::pin::Pin;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Executes one compiled query through a database-specific adapter.
pub struct DatabaseReceiver<A> {
    adapter: A,
    query: CompiledQuery,
    checkpoint: CheckpointStore,
    nack_backoff: Duration,
    max_consecutive_failures: u32,
    source_id: String,
    // The lease is released when the receiver is dropped, so no second
    // receiver in this process can advance the same durable checkpoint.
    _lease: SourceLease,
    metrics: Option<MetricSet<DatabaseReceiverMetrics>>,
}

impl<A> DatabaseReceiver<A>
where
    A: DriverAdapter,
{
    /// Creates a receiver bound to one durable checkpoint source.
    #[must_use]
    pub fn new(
        adapter: A,
        query: CompiledQuery,
        checkpoint: CheckpointStore,
        lease: SourceLease,
        nack_backoff: Duration,
        max_consecutive_failures: u32,
        source_id: String,
        metrics: Option<MetricSet<DatabaseReceiverMetrics>>,
    ) -> Self {
        Self {
            adapter,
            query,
            checkpoint,
            nack_backoff,
            max_consecutive_failures,
            source_id,
            _lease: lease,
            metrics,
        }
    }
}

/// One page awaiting downstream ACK or NACK.
#[derive(Clone, Debug)]
struct PendingPage {
    id: u64,
    candidate: CompositeCursor,
}

/// Committed cursor plus in-flight and scheduling state.
#[derive(Clone, Debug)]
struct ReceiverState {
    committed: CompositeCursor,
    revision: u64,
    pending: Option<PendingPage>,
    next_batch_id: u64,
    next_poll: Instant,
    draining: bool,
}

#[derive(Debug, thiserror::Error)]
enum ProgressError {
    #[error("database query returned a page that did not advance the committed cursor")]
    NonAdvancingCursor,
}

fn ensure_cursor_advanced(
    committed: &CompositeCursor,
    candidate: &CompositeCursor,
) -> Result<(), ProgressError> {
    if committed == candidate
        || (committed.timestamp == candidate.timestamp
            && candidate.tie_breaker <= committed.tie_breaker)
    {
        Err(ProgressError::NonAdvancingCursor)
    } else {
        Ok(())
    }
}

impl ReceiverState {
    fn new(checkpoint: CheckpointState, now: Instant) -> Self {
        Self {
            committed: checkpoint.cursor,
            revision: checkpoint.revision,
            pending: None,
            next_batch_id: 1,
            next_poll: now,
            draining: false,
        }
    }

    /// Returns whether a new query may start.
    ///
    /// At most one page is in flight per source, so a pending ACK/NACK blocks
    /// the next poll and prevents overlapping database work.
    const fn can_poll(&self) -> bool {
        !self.draining && self.pending.is_none()
    }

    fn schedule_after(&mut self, delay: Duration, now: Instant) {
        self.next_poll = now.checked_add(delay).unwrap_or(now);
    }

    fn record_sent(&mut self, candidate: CompositeCursor, poll_interval: Duration, now: Instant) {
        debug_assert!(self.pending.is_none());
        let id = self.next_batch_id;
        self.next_batch_id = self.next_batch_id.saturating_add(1);
        self.pending = Some(PendingPage { id, candidate });
        self.schedule_after(poll_interval, now);
    }

    /// Returns the candidate only when the feedback matches the in-flight page.
    fn ack_candidate(&self, batch_id: u64) -> Option<CompositeCursor> {
        self.pending
            .as_ref()
            .filter(|pending| pending.id == batch_id)
            .map(|pending| pending.candidate.clone())
    }

    fn commit(&mut self, checkpoint: CheckpointState) {
        self.committed = checkpoint.cursor;
        self.revision = checkpoint.revision;
        self.pending = None;
    }

    /// Rewinds to the committed cursor when the feedback matches; else no-op.
    fn nack(&mut self, batch_id: u64, replay_at: Instant) -> bool {
        if self
            .pending
            .as_ref()
            .is_none_or(|pending| pending.id != batch_id)
        {
            return false;
        }
        self.pending = None;
        self.next_poll = replay_at;
        true
    }

    const fn begin_drain(&mut self) {
        self.draining = true;
    }
}

fn batch_id_from_call_data(call_data: &CallData) -> Option<u64> {
    call_data.first().copied().map(u64::from)
}

#[async_trait(?Send)]
impl<A> local::Receiver<OtapPdata> for DatabaseReceiver<A>
where
    A: DriverAdapter + 'static,
{
    async fn start(
        self: Box<Self>,
        mut ctrl_msg_recv: local::ControlChannel<OtapPdata>,
        effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        let Self {
            mut adapter,
            query,
            checkpoint,
            nack_backoff,
            max_consecutive_failures,
            source_id,
            _lease,
            mut metrics,
        } = *self;

        // Fail closed on unreadable durable state before any row is fetched.
        let loaded = read_checkpoint(&checkpoint, &effect_handler).await?;
        let mut state = ReceiverState::new(
            loaded.unwrap_or_else(|| CheckpointState {
                revision: 0,
                cursor: query.watermark().initial.clone(),
            }),
            Instant::now(),
        );
        if let Some(metrics) = metrics.as_mut() {
            metrics.starts.add(1);
        }
        otel_info!(
            "database_receiver.start",
            source_id = source_id.as_str(),
            db_system = adapter.system().as_str(),
            checkpoint_revision = state.revision
        );

        // Prepare the live query before entering the ingestion loop. Selecting
        // over control messages keeps slow database startup drainable.
        let cancellation = adapter
            .begin_operation()
            .map_err(|error| receiver_error(&effect_handler, A::classify_error(&error), error))?;
        let columns = match await_database_operation_or_stop(
            adapter.validate_query(&query),
            cancellation,
            &mut ctrl_msg_recv,
            &mut metrics,
        )
        .await?
        {
            OperationOutcome::Completed(result) => result.map_err(|error| {
                receiver_error(&effect_handler, A::classify_error(&error), error)
            })?,
            OperationOutcome::Stopped(stop) => {
                return finish_stop(stop, &effect_handler, &metrics).await;
            }
        };
        validate_mapping(&columns, query.output()).map_err(|error| {
            receiver_error(&effect_handler, ReceiverErrorKind::Configuration, error)
        })?;

        let mut consecutive_checkpoint_failures = 0_u32;
        let mut drain_deadline: Option<Instant> = None;

        loop {
            tokio::select! {
                biased;

                () = deadline_elapsed(drain_deadline), if drain_deadline.is_some() => {
                    let Some(deadline) = drain_deadline else { continue };
                    if state.pending.is_some() {
                        otel_warn!(
                            "database_receiver.drain_deadline_reached",
                            source_id = source_id.as_str(),
                            message = "Drain deadline reached while a page awaited ACK/NACK; the checkpoint was not advanced"
                        );
                    }
                    return finish_stop(StopRequest::Drain(deadline), &effect_handler, &metrics).await;
                }

                control = ctrl_msg_recv.recv() => {
                    let control = control.map_err(Error::ChannelRecvError)?;
                    match control {
                        NodeControlMsg::CollectTelemetry { mut metrics_reporter } => {
                            if let Some(metrics) = metrics.as_mut() {
                                _ = metrics_reporter.report(metrics);
                            }
                        }
                        NodeControlMsg::Ack(ack) => {
                            let Some(batch_id) = batch_id_from_call_data(&ack.unwind.route.calldata)
                            else {
                                continue;
                            };
                            let Some(candidate) = state.ack_candidate(batch_id) else {
                                // Late or duplicate feedback cannot advance state.
                                if let Some(metrics) = metrics.as_mut() {
                                    metrics.stale_feedback.add(1);
                                }
                                continue;
                            };
                            if let Some(metrics) = metrics.as_mut() {
                                metrics.acks.add(1);
                            }
                            let committed = commit_checkpoint(
                                &checkpoint,
                                state.revision,
                                &candidate,
                                max_consecutive_failures,
                                nack_backoff,
                                &mut consecutive_checkpoint_failures,
                                &source_id,
                                batch_id,
                                &effect_handler,
                                &mut metrics,
                                &mut ctrl_msg_recv,
                            )
                            .await?;
                            let committed = match committed {
                                CommitOutcome::Committed(committed) => committed,
                                CommitOutcome::Stopped(stop) => {
                                    return finish_stop(stop, &effect_handler, &metrics).await;
                                }
                            };
                            // In-memory state advances only after the durable write.
                            state.commit(committed);
                            if state.draining {
                                let deadline = drain_deadline.unwrap_or_else(Instant::now);
                                return finish_stop(
                                    StopRequest::Drain(deadline),
                                    &effect_handler,
                                    &metrics,
                                )
                                .await;
                            }
                        }
                        NodeControlMsg::Nack(nack) => {
                            let Some(batch_id) =
                                batch_id_from_call_data(&nack.unwind.route.calldata)
                            else {
                                continue;
                            };
                            let replay_at = Instant::now()
                                .checked_add(nack_backoff)
                                .unwrap_or_else(Instant::now);
                            if state.nack(batch_id, replay_at) {
                                if let Some(metrics) = metrics.as_mut() {
                                    metrics.nacks.add(1);
                                    metrics.replays.add(1);
                                }
                                otel_warn!(
                                    "database_receiver.page_nacked",
                                    source_id = source_id.as_str(),
                                    batch_id = batch_id,
                                    backoff_millis = nack_backoff.as_millis() as u64,
                                    message = "Database receiver retained its checkpoint and will replay the page"
                                );
                                if state.draining {
                                    let deadline = drain_deadline.unwrap_or_else(Instant::now);
                                    return finish_stop(
                                        StopRequest::Drain(deadline),
                                        &effect_handler,
                                        &metrics,
                                    )
                                    .await;
                                }
                            } else if let Some(metrics) = metrics.as_mut() {
                                metrics.stale_feedback.add(1);
                            }
                        }
                        NodeControlMsg::DrainIngress { deadline, .. } => {
                            if let Some(metrics) = metrics.as_mut() {
                                metrics.drains.add(1);
                            }
                            state.begin_drain();
                            // Honor the earliest deadline when drain is requested twice.
                            let deadline = drain_deadline
                                .map_or(deadline, |current| current.min(deadline));
                            drain_deadline = Some(deadline);
                            if state.pending.is_none() {
                                return finish_stop(
                                    StopRequest::Drain(deadline),
                                    &effect_handler,
                                    &metrics,
                                )
                                .await;
                            }
                        }
                        NodeControlMsg::Shutdown { deadline, .. } => {
                            if let Some(metrics) = metrics.as_mut() {
                                metrics.shutdowns.add(1);
                            }
                            return finish_stop(
                                StopRequest::Shutdown(deadline),
                                &effect_handler,
                                &metrics,
                            )
                            .await;
                        }
                        _ => {}
                    }
                }

                () = poll_due(state.next_poll, state.can_poll()), if state.can_poll() => {
                    if let Some(metrics) = metrics.as_mut() {
                        metrics.polls.add(1);
                    }
                    let cursor = state.committed.clone();
                    let cancellation = adapter.begin_operation().map_err(|error| {
                        receiver_error(&effect_handler, A::classify_error(&error), error)
                    })?;
                    let page = match await_database_operation_or_stop(
                        adapter.execute(&query, &cursor),
                        cancellation,
                        &mut ctrl_msg_recv,
                        &mut metrics,
                    )
                    .await?
                    {
                        OperationOutcome::Completed(result) => result,
                        OperationOutcome::Stopped(stop) => {
                            return finish_stop(stop, &effect_handler, &metrics).await;
                        }
                    };
                    let now = Instant::now();
                    let page = match page {
                        Ok(page) => page,
                        Err(error) => {
                            if let Some(metrics) = metrics.as_mut() {
                                metrics.query_failures.add(1);
                            }
                            return Err(receiver_error(
                                &effect_handler,
                                A::classify_error(&error),
                                error,
                            ));
                        }
                    };
                    if page.is_empty() {
                        state.schedule_after(query.interval(), now);
                        continue;
                    }

                    let observed_time = observed_time_unix_nano().map_err(|error| {
                        receiver_error(&effect_handler, ReceiverErrorKind::Other, error)
                    })?;
                    let encoded = match encode_page(
                        page,
                        adapter.system(),
                        &source_id,
                        query.output(),
                        observed_time,
                        query.max_batch_bytes(),
                    ) {
                        Ok(Some(encoded)) => encoded,
                        Ok(None) => {
                            state.schedule_after(query.interval(), now);
                            continue;
                        }
                        Err(error) => {
                            // An oversized first row or an invalid mapping
                            // cannot be skipped without losing data.
                            return Err(receiver_error(
                                &effect_handler,
                                ReceiverErrorKind::Configuration,
                                error,
                            ));
                        }
                    };
                    let EncodedPage {
                        mut pdata,
                        candidate,
                        row_count,
                        encoded_bytes,
                        deferred_rows,
                        event_time_fallbacks,
                    } = encoded;
                    ensure_cursor_advanced(&state.committed, &candidate).map_err(|error| {
                        receiver_error(&effect_handler, ReceiverErrorKind::Configuration, error)
                    })?;

                    let batch_id = state.next_batch_id;
                    let mut call_data = CallData::new();
                    call_data.push(Context8u8::from(batch_id));
                    effect_handler.subscribe_to(Interests::ACKS_OR_NACKS, call_data, &mut pdata);
                    match send_or_stop(
                        pdata,
                        &mut ctrl_msg_recv,
                        &effect_handler,
                        &mut state,
                        &mut metrics,
                        &mut drain_deadline,
                    )
                    .await?
                    {
                        SendOutcome::Sent => {}
                        SendOutcome::Stopped(stop) => {
                            return finish_stop(stop, &effect_handler, &metrics).await;
                        }
                    }
                    // Recording after the successful send keeps the pending
                    // page and the emitted batch ID consistent on every path.
                    state.record_sent(candidate, query.interval(), Instant::now());
                    if let Some(metrics) = metrics.as_mut() {
                        metrics.batches_sent.add(1);
                        metrics.rows_sent.add(row_count as u64);
                        metrics.encoded_bytes_sent.add(encoded_bytes as u64);
                        metrics
                            .event_time_fallbacks
                            .add(event_time_fallbacks as u64);
                    }
                    if event_time_fallbacks > 0 {
                        otel_warn!(
                            "database_receiver.event_time_fallback",
                            source_id = source_id.as_str(),
                            records = event_time_fallbacks as u64,
                            message = "Database records used observation time because source event time was outside the OTLP range"
                        );
                    }
                    otel_debug!(
                        "database_receiver.page_sent",
                        source_id = source_id.as_str(),
                        batch_id = batch_id,
                        rows = row_count as u64,
                        encoded_bytes = encoded_bytes as u64,
                        deferred_rows = deferred_rows as u64
                    );
                }
            }
        }
    }
}

enum OperationOutcome<T> {
    Completed(T),
    Stopped(StopRequest),
}

enum SendOutcome {
    Sent,
    Stopped(StopRequest),
}

#[derive(Clone, Copy)]
enum StopRequest {
    Drain(Instant),
    Shutdown(Instant),
}

/// Waits until the drain deadline, or forever when no drain is pending.
async fn deadline_elapsed(deadline: Option<Instant>) {
    match deadline {
        Some(deadline) => {
            tokio::time::sleep_until(tokio::time::Instant::from_std(deadline)).await;
        }
        None => std::future::pending::<()>().await,
    }
}

/// Waits until the next poll is due, or forever when polling is blocked.
async fn poll_due(next_poll: Instant, can_poll: bool) {
    if can_poll {
        tokio::time::sleep_until(tokio::time::Instant::from_std(next_poll)).await;
    } else {
        std::future::pending::<()>().await;
    }
}

async fn read_checkpoint(
    store: &CheckpointStore,
    effect_handler: &local::EffectHandler<OtapPdata>,
) -> Result<Option<CheckpointState>, Error> {
    // Checkpoint filesystem work blocks, so it must not run on the local
    // async engine core.
    let store = store.clone();
    tokio::task::spawn_blocking(move || store.read())
        .await
        .map_err(|error| receiver_error(effect_handler, ReceiverErrorKind::Other, error))?
        .map_err(|error| receiver_error(effect_handler, ReceiverErrorKind::Configuration, error))
}

#[allow(clippy::too_many_arguments)]
async fn commit_checkpoint(
    store: &CheckpointStore,
    revision: u64,
    candidate: &CompositeCursor,
    max_failures: u32,
    retry_backoff: Duration,
    consecutive_failures: &mut u32,
    source_id: &str,
    batch_id: u64,
    effect_handler: &local::EffectHandler<OtapPdata>,
    metrics: &mut Option<MetricSet<DatabaseReceiverMetrics>>,
    ctrl_msg_recv: &mut local::ControlChannel<OtapPdata>,
) -> Result<CommitOutcome, Error> {
    loop {
        let store = store.clone();
        let cursor = candidate.clone();
        let result = tokio::task::spawn_blocking(move || store.write(revision, &cursor))
            .await
            .map_err(|error| receiver_error(effect_handler, ReceiverErrorKind::Other, error))?;
        match result {
            Ok((checkpoint, outcome)) => {
                *consecutive_failures = 0;
                if let Some(metrics) = metrics.as_mut() {
                    metrics.checkpoint_commits.add(1);
                    metrics
                        .checkpoint_cleanup_failures
                        .add(outcome.cleanup_failures as u64);
                }
                if outcome.cleanup_failures > 0 {
                    otel_warn!(
                        "database_receiver.checkpoint_cleanup_failed",
                        source_id = source_id,
                        failures = outcome.cleanup_failures as u64,
                        message = "Database receiver could not remove stale checkpoint revisions"
                    );
                }
                otel_debug!(
                    "database_receiver.checkpoint_committed",
                    source_id = source_id,
                    batch_id = batch_id,
                    revision = checkpoint.revision
                );
                return Ok(CommitOutcome::Committed(checkpoint));
            }
            Err(error) => {
                // The in-memory cursor is never advanced on a failed write, so
                // a later retry or restart replays the same page.
                *consecutive_failures = consecutive_failures.saturating_add(1);
                if let Some(metrics) = metrics.as_mut() {
                    metrics.checkpoint_failures.add(1);
                }
                otel_warn!(
                    "database_receiver.checkpoint_failed",
                    source_id = source_id,
                    batch_id = batch_id,
                    attempt = u64::from(*consecutive_failures),
                    error = %error
                );
                if *consecutive_failures >= max_failures {
                    return Err(receiver_error(
                        effect_handler,
                        ReceiverErrorKind::Other,
                        error,
                    ));
                }
                let retry = tokio::time::sleep(retry_backoff);
                tokio::pin!(retry);
                loop {
                    tokio::select! {
                        biased;

                        control = ctrl_msg_recv.recv() => {
                            let control = control.map_err(Error::ChannelRecvError)?;
                            match control {
                                NodeControlMsg::CollectTelemetry { mut metrics_reporter } => {
                                    if let Some(metrics) = metrics.as_mut() {
                                        _ = metrics_reporter.report(metrics);
                                    }
                                }
                                control => {
                                    if let Some(stop) = stop_request(&control) {
                                        return Ok(CommitOutcome::Stopped(stop));
                                    }
                                }
                            }
                        }
                        () = &mut retry => break,
                    }
                }
            }
        }
    }
}

enum CommitOutcome {
    Committed(CheckpointState),
    Stopped(StopRequest),
}

async fn await_database_operation_or_stop<F, T, C>(
    operation: F,
    cancellation: C,
    ctrl_msg_recv: &mut local::ControlChannel<OtapPdata>,
    metrics: &mut Option<MetricSet<DatabaseReceiverMetrics>>,
) -> Result<OperationOutcome<T>, Error>
where
    F: Future<Output = T>,
    C: DriverCancellation,
{
    tokio::pin!(operation);
    loop {
        tokio::select! {
            biased;

            control = ctrl_msg_recv.recv() => {
                match control {
                    Ok(NodeControlMsg::CollectTelemetry { mut metrics_reporter }) => {
                        if let Some(metrics) = metrics.as_mut() {
                            _ = metrics_reporter.report(metrics);
                        }
                    }
                    Ok(control) => {
                        if let Some(stop) = stop_request(&control) {
                            if let Some(metrics) = metrics.as_mut() {
                                metrics.cancellations.add(1);
                                match stop {
                                    StopRequest::Drain(_) => metrics.drains.add(1),
                                    StopRequest::Shutdown(_) => metrics.shutdowns.add(1),
                                }
                            }
                            cancel_and_join(operation.as_mut(), &cancellation).await;
                            return Ok(OperationOutcome::Stopped(stop));
                        }
                    }
                    Err(error) => {
                        if let Some(metrics) = metrics.as_mut() {
                            metrics.cancellations.add(1);
                        }
                        cancel_and_join(operation.as_mut(), &cancellation).await;
                        return Err(Error::ChannelRecvError(error));
                    }
                }
            }
            result = &mut operation => return Ok(OperationOutcome::Completed(result)),
        }
    }
}

async fn cancel_and_join<F, C>(mut operation: Pin<&mut F>, cancellation: &C)
where
    F: Future,
    C: DriverCancellation,
{
    if let Err(error) = cancellation.cancel().await {
        otel_warn!(
            "database_receiver.cancellation_failed",
            error = %error,
            message = "Database receiver could not interrupt its active operation"
        );
    }
    // Join blocking work even when cancellation itself fails, so a
    // replacement receiver cannot overlap this operation.
    _ = operation.as_mut().await;
}

async fn send_or_stop(
    pdata: OtapPdata,
    ctrl_msg_recv: &mut local::ControlChannel<OtapPdata>,
    effect_handler: &local::EffectHandler<OtapPdata>,
    state: &mut ReceiverState,
    metrics: &mut Option<MetricSet<DatabaseReceiverMetrics>>,
    drain_deadline: &mut Option<Instant>,
) -> Result<SendOutcome, Error> {
    let pdata = match effect_handler.try_send_message(pdata) {
        Ok(()) => return Ok(SendOutcome::Sent),
        Err(TypedError::ChannelSendError(SendError::Full(pdata))) => pdata,
        Err(error) => return Err(error.into()),
    };
    // Backpressure: block on the downstream send while remaining drainable.
    let send = effect_handler.send_message(pdata);
    tokio::pin!(send);

    loop {
        tokio::select! {
            biased;

            () = deadline_elapsed(*drain_deadline), if drain_deadline.is_some() => {
                let Some(deadline) = *drain_deadline else { continue };
                otel_warn!(
                    "database_receiver.drain_deadline_reached",
                    message = "Database receiver drain deadline reached while sending downstream"
                );
                return Ok(SendOutcome::Stopped(StopRequest::Drain(deadline)));
            }
            control = ctrl_msg_recv.recv() => {
                let control = control.map_err(Error::ChannelRecvError)?;
                match control {
                    NodeControlMsg::CollectTelemetry { mut metrics_reporter } => {
                        if let Some(metrics) = metrics.as_mut() {
                            _ = metrics_reporter.report(metrics);
                        }
                    }
                    NodeControlMsg::DrainIngress { deadline, .. } => {
                        if let Some(metrics) = metrics.as_mut() {
                            metrics.drains.add(1);
                        }
                        state.begin_drain();
                        *drain_deadline =
                            Some(drain_deadline.map_or(deadline, |current| current.min(deadline)));
                    }
                    NodeControlMsg::Shutdown { deadline, .. } => {
                        if let Some(metrics) = metrics.as_mut() {
                            metrics.shutdowns.add(1);
                        }
                        return Ok(SendOutcome::Stopped(StopRequest::Shutdown(deadline)));
                    }
                    _ => {}
                }
            }
            result = &mut send => {
                result.map_err(Error::from)?;
                return Ok(SendOutcome::Sent);
            }
        }
    }
}

fn stop_request(control: &NodeControlMsg<OtapPdata>) -> Option<StopRequest> {
    match control {
        NodeControlMsg::DrainIngress { deadline, .. } => Some(StopRequest::Drain(*deadline)),
        NodeControlMsg::Shutdown { deadline, .. } => Some(StopRequest::Shutdown(*deadline)),
        _ => None,
    }
}

async fn finish_stop(
    stop: StopRequest,
    effect_handler: &local::EffectHandler<OtapPdata>,
    metrics: &Option<MetricSet<DatabaseReceiverMetrics>>,
) -> Result<TerminalState, Error> {
    let deadline = match stop {
        StopRequest::Drain(deadline) => {
            effect_handler.notify_receiver_drained().await?;
            deadline
        }
        StopRequest::Shutdown(deadline) => deadline,
    };
    Ok(match metrics {
        Some(metrics) => TerminalState::new(deadline, [metrics.snapshot()]),
        None => TerminalState::new::<[MetricSetSnapshot; 0]>(deadline, []),
    })
}

fn observed_time_unix_nano() -> Result<u64, ObservationTimeError> {
    let duration = SystemTime::now().duration_since(UNIX_EPOCH)?;
    // OTLP uses u64 nanoseconds. Reject an unrepresentable clock value rather
    // than silently saturating and emitting a misleading timestamp.
    Ok(duration.as_nanos().try_into()?)
}

#[derive(Debug, thiserror::Error)]
enum ObservationTimeError {
    /// The system clock is earlier than the Unix epoch.
    #[error("system clock is earlier than the Unix epoch")]
    BeforeUnixEpoch(#[from] std::time::SystemTimeError),
    /// Nanoseconds since the epoch do not fit OTLP's unsigned 64-bit field.
    #[error("observation time is outside the supported OTLP range")]
    OutOfRange(#[from] std::num::TryFromIntError),
}

fn receiver_error(
    effect_handler: &local::EffectHandler<OtapPdata>,
    kind: ReceiverErrorKind,
    error: impl std::error::Error + 'static,
) -> Error {
    let source_detail = format_error_sources(&error);
    Error::ReceiverError {
        receiver: effect_handler.receiver_id(),
        kind,
        error: error.to_string(),
        source_detail,
    }
}

#[cfg(test)]
#[path = "receiver_tests.rs"]
mod tests;
