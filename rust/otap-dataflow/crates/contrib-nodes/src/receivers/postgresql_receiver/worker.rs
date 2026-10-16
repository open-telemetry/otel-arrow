// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Single connection owner with read-only portal pages and observed cleanup.

use super::{
    adapter::{CachedSchema, Error, Operation, PostgreSqlAdapter, Result, database},
    query::{CatalogStatements, ResolvedSchema, Signature},
    value as convert,
};
use futures::{StreamExt, future::poll_fn};
use otel_arrow_dfe_engine::capability::auth::BasicAuthCredential;
use otel_arrow_dfe_scraper::database::{ColumnMetadata, CursorRow, QueryPage, Row};
use secrecy::ExposeSecret;
use std::{
    cell::{Cell, RefCell},
    future::Future,
    io,
    net::SocketAddr,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, ready},
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    task::JoinHandle,
    time::Instant,
};
use tokio_postgres::{
    CancelToken, Client, IsolationLevel, NoTls, Statement, config::SslMode, types::ToSql,
};

pub(crate) type CancelSlot = Rc<RefCell<Option<(CancelToken, SocketAddr)>>>;

const CONNECTION_TIMEOUT: Duration = Duration::from_secs(10);
const FETCH_SIZE_WINDOW: usize = 32;
const CANCEL_REQUEST_TIMEOUT: Duration = Duration::from_secs(1);
const CANCEL_COMPLETION_TIMEOUT: Duration = Duration::from_secs(2);
const CONNECTION_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(1);

/// Non-row startup, authentication, and catalog descriptions need room even
/// when the customer selects a tiny page budget. DataRows never use this floor.
const CONTROL_FRAME_BYTES: usize = 64 * 1024;
/// Fixed socket read-ahead; independent of any advertised backend frame size.
const READ_BUFFER_BYTES: usize = 8 * 1024;

#[derive(Clone, Copy)]
struct FrameLimits {
    data_row: usize,
    control: usize,
}

impl FrameLimits {
    fn new(normalized_bytes: u64) -> Result<Self> {
        // Wire and normalized sizes are not interchangeable: a DataRow has a
        // tag, length, field count, and one length per selected column (<=128).
        // This is only a receive bound; decoded allocations and encoded OTLP
        // still undergo their own page-budget checks, including expansion.
        let data_row = usize::try_from(normalized_bytes)
            .ok()
            .and_then(|bytes| bytes.checked_add(5 + 2 + 4 * 128))
            .ok_or(Error::Limit)?;
        Ok(Self {
            data_row,
            control: data_row.max(CONTROL_FRAME_BYTES),
        })
    }

    fn for_tag(self, tag: u8) -> usize {
        if tag == b'D' {
            self.data_row
        } else {
            self.control
        }
    }

    fn raw_row(self) -> usize {
        self.data_row - 5
    }
}

/// Driver I/O errors otherwise reach waiting requests only as Error::closed().
/// This latch is shared solely by tasks on the worker's local runtime.
type ReceiveFailure = Rc<Cell<Option<Error>>>;

/// Bounds plaintext before decoding: tokio-postgres 0.7.18 batches all buffered
/// frames, so its capacity-one channel alone cannot bound bytes. One frame per
/// read bounds its receive buffer and queued/current BackendMessages values,
/// even for 300-row fetches. Recheck this invariant when updating the driver.
struct BoundedBackend<S> {
    inner: S,
    failure: ReceiveFailure,
    limits: FrameLimits,
    header: [u8; 5],
    header_read: usize,
    header_sent: usize,
    body_left: usize,
}

impl<S> BoundedBackend<S> {
    fn new(inner: S, failure: ReceiveFailure, limits: FrameLimits) -> Self {
        Self {
            inner,
            failure,
            limits,
            header: [0; 5],
            header_read: 0,
            header_sent: 0,
            body_left: 0,
        }
    }

    fn fail(&self, error: Error, kind: io::ErrorKind, message: &'static str) -> io::Error {
        if self.failure.get().is_none() {
            self.failure.set(Some(error));
        }
        io::Error::new(kind, message)
    }

    fn finish_frame(&mut self) {
        if self.header_sent == self.header.len() && self.body_left == 0 {
            self.header_read = 0;
            self.header_sent = 0;
        }
    }

    fn read_error(&self, error: io::Error) -> io::Error {
        if error.kind() == io::ErrorKind::UnexpectedEof {
            self.fail(
                Error::Unavailable,
                io::ErrorKind::UnexpectedEof,
                "postgresql backend frame truncated",
            )
        } else {
            error
        }
    }
}

impl<S: AsyncRead + Unpin> AsyncRead for BoundedBackend<S> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        output: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        if output.remaining() == 0 {
            return Poll::Ready(Ok(()));
        }
        if this.failure.get().is_some() {
            return Poll::Ready(Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "postgresql backend receive stopped",
            )));
        }
        while this.header_read < this.header.len() {
            let mut header = ReadBuf::new(&mut this.header[this.header_read..]);
            let read = match ready!(Pin::new(&mut this.inner).poll_read(cx, &mut header)) {
                Ok(()) => header.filled().len(),
                Err(error) => return Poll::Ready(Err(this.read_error(error))),
            };
            if read == 0 {
                return Poll::Ready(if this.header_read == 0 {
                    Ok(())
                } else {
                    Err(this.fail(
                        Error::Unavailable,
                        io::ErrorKind::UnexpectedEof,
                        "postgresql backend header truncated",
                    ))
                });
            }
            this.header_read += read;
            if this.header_read == this.header.len() {
                let length = i32::from_be_bytes([
                    this.header[1],
                    this.header[2],
                    this.header[3],
                    this.header[4],
                ]);
                if length < 4 {
                    return Poll::Ready(Err(this.fail(
                        Error::Database,
                        io::ErrorKind::InvalidData,
                        "postgresql backend frame length invalid",
                    )));
                }
                if length as usize >= this.limits.for_tag(this.header[0]) {
                    return Poll::Ready(Err(this.fail(
                        Error::Limit,
                        io::ErrorKind::InvalidData,
                        "postgresql backend frame exceeds receive limit",
                    )));
                }
                this.body_left = length as usize - 4;
            }
        }
        if this.header_sent < this.header.len() {
            let count = output.remaining().min(this.header.len() - this.header_sent);
            output.put_slice(&this.header[this.header_sent..this.header_sent + count]);
            this.header_sent += count;
            this.finish_frame();
            return Poll::Ready(Ok(()));
        }

        let count = output.remaining().min(this.body_left);
        let mut body = ReadBuf::new(output.initialize_unfilled_to(count));
        let read = match ready!(Pin::new(&mut this.inner).poll_read(cx, &mut body)) {
            Ok(()) => body.filled().len(),
            Err(error) => return Poll::Ready(Err(this.read_error(error))),
        };
        if read == 0 {
            return Poll::Ready(Err(this.fail(
                Error::Unavailable,
                io::ErrorKind::UnexpectedEof,
                "postgresql backend body truncated",
            )));
        }
        output.advance(read);
        this.body_left -= read;
        this.finish_frame();
        // Return at this frame's boundary even if the destination has room.
        // Reading the next header here would let the driver coalesce more frames.
        Poll::Ready(Ok(()))
    }
}

impl<S: AsyncWrite + Unpin> AsyncWrite for BoundedBackend<S> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bytes: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.get_mut().inner).poll_write(cx, bytes)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().inner).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().inner).poll_shutdown(cx)
    }
}

pub(crate) struct Session {
    client: Option<Client>,
    driver: JoinHandle<Result<()>>,
    statement: Option<Statement>,
    catalog: Option<CatalogStatements>,
    receive_failure: ReceiveFailure,
}
fn resolve_schema<'a>(
    cache: &'a mut Option<CachedSchema>,
    validated: &super::config::Validated,
    signature: Signature,
) -> Result<&'a CachedSchema> {
    if let Some(cached) = cache {
        // Fresh catalog queries already recheck every required uniqueness
        // property. Compare stable column semantics, not physical index OIDs:
        // REINDEX CONCURRENTLY can replace an equivalent index.
        if cached.signature.columns != signature.columns {
            return Err(Error::Metadata);
        }
    } else {
        let resolved = ResolvedSchema::discover(&validated.plan, &validated.common, &signature)?;
        *cache = Some(CachedSchema {
            signature,
            resolved,
        });
    }
    cache.as_ref().ok_or(Error::Metadata)
}

#[derive(Default)]
pub(crate) struct FetchSize {
    recent_row_bytes: [u64; FETCH_SIZE_WINDOW],
    observed: usize,
    next: usize,
}

impl FetchSize {
    fn target(&self, row_limit: usize, bytes_left: u64) -> usize {
        self.recent_row_bytes[..self.observed]
            .iter()
            .copied()
            .max()
            .map_or(1, |size| bytes_left / size)
            .min(row_limit as u64) as usize
    }

    fn observe(&mut self, row_bytes: u64) {
        self.recent_row_bytes[self.next] = row_bytes.max(1);
        self.next = (self.next + 1) % FETCH_SIZE_WINDOW;
        self.observed = self.observed.saturating_add(1).min(FETCH_SIZE_WINDOW);
    }
}

pub(crate) enum Action {
    Validate,
    Reconnect,
    Execute(otel_arrow_dfe_scraper::database::CompositeCursor),
}

pub(crate) enum Reply {
    Columns(Vec<ColumnMetadata>),
    Page(QueryPage),
    Done,
}

enum OperationCompletion<T> {
    Completed(Result<T>),
    Incomplete(Error),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SessionClose {
    Graceful,
    Aborted,
}

/// Whether a statement could still be executing on the server when a session
/// closes. A joined driver only proves the local task stopped, so this decides
/// if an unexplained driver failure needs a server-query fence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CloseReason {
    /// An operation was in flight. A transport failure may have interrupted the
    /// client without stopping the backend statement.
    OperationInFlight,
    /// No statement was running, so nothing can outlive the local driver.
    Idle,
}

/// Classifies a joined driver result. A driver error proves local completion
/// only; during an active statement it cannot prove the server stopped.
fn driver_outcome(result: Result<()>, reason: CloseReason) -> SessionClose {
    match (result, reason) {
        (Ok(()), _) | (Err(_), CloseReason::Idle) => SessionClose::Graceful,
        (Err(_), CloseReason::OperationInFlight) => SessionClose::Aborted,
    }
}

async fn cancel_native(slot: &CancelSlot) {
    let token = slot.borrow().clone();
    if let Some((token, address)) = token {
        // This is only a request. The operation and driver are still awaited.
        let _ = tokio::time::timeout(CANCEL_REQUEST_TIMEOUT, async {
            let stream = tokio::net::TcpStream::connect(address).await?;
            token
                .cancel_query_raw(stream, NoTls)
                .await
                .map_err(io::Error::other)
        })
        .await;
    }
}

async fn await_operation<T>(
    future: impl Future<Output = Result<T>>,
    operation: &Operation,
    cancel: &CancelSlot,
    timeout: Duration,
) -> OperationCompletion<T> {
    tokio::pin!(future);
    tokio::select! {
        biased;
        result = &mut future => OperationCompletion::Completed(result),
        _ = operation.notify.notified() => {
            operation.cancel();
            cancel_native(cancel).await;
            match tokio::time::timeout(CANCEL_COMPLETION_TIMEOUT, &mut future).await {
                Ok(_) => OperationCompletion::Completed(Err(Error::Cancelled)),
                Err(_) => OperationCompletion::Incomplete(Error::Cancelled),
            }
        }
        _ = tokio::time::sleep(timeout) => {
            operation.cancel();
            cancel_native(cancel).await;
            match tokio::time::timeout(CANCEL_COMPLETION_TIMEOUT, &mut future).await {
                Ok(_) => OperationCompletion::Completed(Err(Error::Timeout)),
                Err(_) => OperationCompletion::Incomplete(Error::Timeout),
            }
        }
    }
}

fn session_options(timeout: Duration) -> String {
    format!(
        "-c search_path=pg_catalog -c TimeZone=UTC -c row_security=off \
         -c default_transaction_read_only=on -c statement_timeout={} \
         -c idle_in_transaction_session_timeout={} \
         -c application_name=otel_arrow_postgresql_receiver",
        timeout.as_millis(),
        timeout.as_millis(),
    )
}

impl PostgreSqlAdapter {
    pub(crate) async fn run_worker(
        &mut self,
        action: Action,
        operation: &Operation,
        credential: &BasicAuthCredential,
    ) -> Result<Reply> {
        let cancel = self.cancel.clone();
        let timeout = self.validated.common.timeout();
        let completion = await_operation(
            self.perform(action, operation, credential),
            operation,
            &cancel,
            timeout,
        )
        .await;
        let mut result = match completion {
            OperationCompletion::Completed(result) => result,
            OperationCompletion::Incomplete(error) => {
                return Err(self.recover_incomplete_operation(error).await);
            }
        };
        // Requests see only a closed driver after a framing failure. Preserve the
        // actual receive limit instead of retrying an unavailable database.
        if result.as_ref().err() != Some(&Error::Cleanup)
            && let Some(error) = self
                .session
                .as_ref()
                .and_then(|session| session.receive_failure.get())
        {
            result = Err(error);
        }
        if result.as_ref().err() == Some(&Error::Cleanup) {
            // A later successful close cannot prove an earlier abandoned operation
            // stopped. Keep shutdown terminal so source ownership is quarantined.
            self.poisoned = true;
        }
        if result.is_err() {
            operation.cancel();
            // The failed operation may have left the backend executing, so a
            // driver error here must fence rather than look graceful.
            if self
                .close_session(CloseReason::OperationInFlight)
                .await
                .is_err()
            {
                self.poisoned = true;
                result = Err(Error::Cleanup);
            }
        }
        result
    }

    async fn recover_incomplete_operation(&mut self, error: Error) -> Error {
        let had_session = self.session.is_some();
        let close = match self.close_session(CloseReason::OperationInFlight).await {
            Ok(close) => close,
            Err(_) => {
                self.poisoned = true;
                return Error::Cleanup;
            }
        };
        if had_session && close == SessionClose::Graceful {
            // The joined local driver proves that it cannot send more work, but
            // a partitioned server may still be executing the last statement.
            // Every session uses this same statement timeout, so waiting one
            // full interval after the join safely fences that old backend work.
            self.fence_server_query();
        }
        error
    }

    /// Delays the next connection until an orphaned backend statement must have
    /// ended.
    ///
    /// What this protects: server resources, not receiver correctness. Pages are
    /// read-only and a failed page is discarded without advancing the
    /// checkpoint, so a stale backend query overlapping a new one cannot corrupt
    /// the cursor. The fence keeps the receiver from stacking duplicate
    /// expensive queries and from pinning extra snapshots against VACUUM.
    ///
    /// One rule decides when it applies: fence whenever a backend statement may
    /// still be running. A joined driver only proves the local task stopped, so
    /// an interrupted statement still needs the wait, while an idle teardown
    /// needs none. See `CloseReason`.
    ///
    /// The wait is one operation timeout, which is also the `statement_timeout`
    /// set by `session_options`, so the server has terminated the orphan by the
    /// time the fence expires.
    fn fence_server_query(&mut self) {
        let deadline = Instant::now() + self.validated.common.timeout();
        self.reconnect_not_before = Some(
            self.reconnect_not_before
                .map_or(deadline, |existing| existing.max(deadline)),
        );
    }

    async fn await_reconnect_fence(&mut self, op: &Operation) -> Result<()> {
        let Some(deadline) = self.reconnect_not_before else {
            return Ok(());
        };
        if deadline > Instant::now() {
            tokio::select! {
                biased;
                _ = op.notify.notified() => return Err(Error::Cancelled),
                _ = tokio::time::sleep_until(deadline) => {}
            }
        }
        op.check()?;
        self.reconnect_not_before = None;
        Ok(())
    }

    pub(crate) async fn close_session(&mut self, reason: CloseReason) -> Result<SessionClose> {
        let _ = self.cancel.borrow_mut().take();
        if let Some(session) = self.session.as_mut() {
            let _ = session.statement.take();
            let _ = session.catalog.take();
            let _ = session.client.take();
            let outcome = match tokio::time::timeout(
                CONNECTION_SHUTDOWN_TIMEOUT,
                &mut session.driver,
            )
            .await
            {
                // A joined task confirms the driver stopped, but only a driver
                // that ended without error rules out interrupted server work.
                // A panic-style JoinError means cleanup itself was not observed.
                Ok(Ok(result)) => Ok(driver_outcome(result, reason)),
                Ok(Err(_)) => Err(Error::Cleanup),
                Err(_) => {
                    // Aborting and then observing a cancellation-style JoinError
                    // proves the local task stopped. It does not prove the server
                    // query stopped, so the outcome installs a reconnect fence.
                    session.driver.abort();
                    match (&mut session.driver).await {
                        Err(error) if error.is_cancelled() => Ok(SessionClose::Aborted),
                        Ok(result) => Ok(driver_outcome(result, reason)),
                        Err(_) => Err(Error::Cleanup),
                    }
                }
            };
            let _ = self.session.take();
            match outcome {
                Ok(SessionClose::Aborted) => self.fence_server_query(),
                Err(_) => self.poisoned = true,
                Ok(SessionClose::Graceful) => {}
            }
            return outcome;
        }
        Ok(SessionClose::Graceful)
    }

    async fn connect(&mut self, op: &Operation, credential: &BasicAuthCredential) -> Result<()> {
        if self.poisoned {
            return Err(Error::Cleanup);
        }
        self.await_reconnect_fence(op).await?;
        op.check()?;
        let config = &self.validated.config.connection;
        // Password refresh may reconnect the same principal, but switching users
        // could change visible rows/grants underneath an existing source cursor.
        if self
            .principal
            .as_ref()
            .is_some_and(|principal| principal.expose_secret() != credential.expose_username())
        {
            return Err(Error::Credential);
        }
        op.check()?;
        let mut pg = tokio_postgres::Config::new();
        let _ = pg
            .host(&config.host)
            .port(config.port)
            .dbname(&config.database)
            .user(credential.expose_username())
            .password(credential.expose_password())
            .ssl_mode(SslMode::Disable)
            .connect_timeout(CONNECTION_TIMEOUT)
            .options(session_options(self.validated.common.timeout()));
        let receive_failure = ReceiveFailure::default();
        let limits = FrameLimits::new(self.validated.common.max_normalized_bytes())?;
        let connected = tokio::time::timeout(CONNECTION_TIMEOUT, async {
            let stream = tokio::net::TcpStream::connect((config.host.as_str(), config.port))
                .await
                .map_err(|_| Error::Unavailable)?;
            let address = stream.peer_addr().map_err(|_| Error::Unavailable)?;
            // Match the native driver's socket setting when supplying our own stream.
            stream.set_nodelay(true).map_err(|_| Error::Unavailable)?;
            let stream = tokio::io::BufReader::with_capacity(READ_BUFFER_BYTES, stream);
            let stream = BoundedBackend::new(stream, receive_failure.clone(), limits);
            let (client, connection) = pg
                .connect_raw(stream, NoTls)
                .await
                .map_err(|error| receive_failure.get().unwrap_or_else(|| database(error)))?;
            Ok::<_, Error>((client, connection, address))
        })
        .await;
        drop(pg);
        let (client, mut connection, address) =
            connected.map_err(|_| receive_failure.get().unwrap_or(Error::Unavailable))??;
        let driver_failure = receive_failure.clone();
        let driver = tokio::task::spawn_local(async move {
            loop {
                match poll_fn(|cx| connection.poll_message(cx)).await {
                    None => return Ok(()),
                    Some(Err(e)) => {
                        let error = driver_failure.get().unwrap_or_else(|| database(e));
                        driver_failure.set(Some(error));
                        return Err(error);
                    }
                    Some(Ok(tokio_postgres::AsyncMessage::Notice(_))) => {}
                    Some(Ok(_)) => {
                        driver_failure.set(Some(Error::Limit));
                        return Err(Error::Limit);
                    }
                }
            }
        });
        *self.cancel.borrow_mut() = Some((client.cancel_token(), address));
        if self.principal.is_none() {
            self.principal = Some(credential.expose_username().to_owned().into());
        }
        self.session = Some(Session {
            client: Some(client),
            driver,
            statement: None,
            catalog: None,
            receive_failure,
        });
        op.check()
    }

    async fn perform(
        &mut self,
        action: Action,
        op: &Operation,
        credential: &BasicAuthCredential,
    ) -> Result<Reply> {
        op.check()?;
        if self.poisoned {
            return Err(Error::Cleanup);
        }
        if matches!(action, Action::Reconnect) {
            // Any session reaching reconnect is idle: a failed operation already
            // closed its own session before the controller retried.
            _ = self.close_session(CloseReason::Idle).await?;
            self.connect(op, credential).await?;
            return Ok(Reply::Done);
        }
        if self.session.is_none() {
            self.connect(op, credential).await?;
        }
        let session = self.session.as_mut().ok_or(Error::Cleanup)?;
        let plan = &self.validated.plan;
        let tx = session
            .client
            .as_mut()
            .ok_or(Error::Cleanup)?
            .build_transaction()
            .read_only(true)
            .isolation_level(IsolationLevel::ReadCommitted)
            .start()
            .await
            .map_err(database)?;
        let result = async {
            op.check()?;
            if session.catalog.is_none() {
                session.catalog = Some(CatalogStatements::prepare(&tx, op).await?);
            }
            // Cache prepared catalog statements, never their results: each page
            // must re-establish the source contract before using cached decoders.
            let signature = plan
                .catalog(&tx, session.catalog.as_ref().ok_or(Error::Metadata)?, op)
                .await?;
            op.check()?;
            let cached = resolve_schema(&mut self.schema, &self.validated, signature)?;
            let schema = &cached.resolved;
            if session.statement.is_none() {
                let timestamp_type = schema.native_types[plan.timestamp].clone();
                let tie_type = schema.native_types[plan.tie].clone();
                let statement = tx
                    .prepare_typed(&plan.sql, &[timestamp_type.clone(), tie_type.clone()])
                    .await
                    .map_err(database)?;
                op.check()?;
                if statement.params() != [timestamp_type, tie_type]
                    || statement.columns().len() != schema.columns.len()
                {
                    return Err(Error::Metadata);
                }
                for ((column, reference), expected) in statement
                    .columns()
                    .iter()
                    .zip(&plan.projections)
                    .zip(&schema.columns)
                {
                    let source = cached
                        .signature
                        .columns
                        .get(reference)
                        .ok_or(Error::Metadata)?;
                    // Matching names/types alone cannot prove a projection came
                    // from the expected table and column; check native provenance.
                    if column.name() != expected.name
                        || column.table_oid() != Some(source.oid)
                        || column.column_id() != Some(source.attribute)
                        || column.type_().oid() != source.type_oid
                        || column.type_modifier() != source.modifier
                    {
                        return Err(Error::Metadata);
                    }
                }
                session.statement = Some(statement);
            }
            let columns = metadata(schema);
            let Action::Execute(cursor) = action else {
                return Ok(Reply::Columns(columns));
            };
            let timestamp = convert::cursor_time(
                &cursor.timestamp,
                schema.columns[plan.timestamp].type_modifier,
            )?;
            let tie_type = &schema.columns[plan.tie].source_type;
            convert::check_tie(cursor.tie_breaker, tie_type)?;
            let mut previous = (timestamp.naive_utc(), cursor.tie_breaker);
            let timestamp: Box<dyn ToSql + Sync> =
                if schema.columns[plan.timestamp].source_type == "timestamp" {
                    Box::new(timestamp.naive_utc())
                } else {
                    Box::new(timestamp)
                };
            let tie: Box<dyn ToSql + Sync> = match tie_type.as_str() {
                "int2" => Box::new(i16::try_from(cursor.tie_breaker).map_err(|_| Error::Value)?),
                "int4" => Box::new(i32::try_from(cursor.tie_breaker).map_err(|_| Error::Value)?),
                "int8" => Box::new(cursor.tie_breaker),
                _ => return Err(Error::Metadata),
            };
            op.check()?;
            let statement = session.statement.as_ref().ok_or(Error::Metadata)?;
            let portal = tx
                .bind(statement, &[timestamp.as_ref(), tie.as_ref()])
                .await
                .map_err(database)?;
            op.check()?;
            let mut page = QueryPage {
                columns,
                rows: Vec::new(),
            };
            let max_rows = self.validated.common.max_rows();
            let limit = self.validated.common.max_normalized_bytes();
            let raw_limit = FrameLimits::new(limit)?.raw_row();
            let mut payload = page_metadata_bytes(&page);
            if payload > limit {
                return Err(Error::Limit);
            }
            'page: while page.rows.len() < max_rows {
                op.check()?;
                // Probe one row initially; retain the largest observed size across
                // pages so wide rows do not repeatedly drain oversized fetch groups.
                // A historical wide row or spare capacity must not prevent a
                // smaller next row from fitting: probe one when the estimate is
                // exhausted, and let actual admission decide.
                let used = payload + (page.rows.capacity() * size_of::<CursorRow>()) as u64;
                let target = self
                    .fetch_size
                    .target(
                        (max_rows - page.rows.len()).min(self.validated.common.fetch_size_rows()),
                        limit.saturating_sub(used),
                    )
                    .max(1);
                let stream = tx
                    .query_portal_raw(&portal, target as i32)
                    .await
                    .map_err(database)?;
                op.check()?;
                tokio::pin!(stream);
                let mut fetched = 0;
                while let Some(native) = stream.next().await {
                    op.check()?;
                    let native = native.map_err(database)?;
                    let decoded =
                        convert::row(&native, plan, schema, &mut self.timestamp_cache, raw_limit)?;
                    op.check()?;
                    ensure_advance(&previous, &decoded.position)?;
                    let row = decoded.row;
                    let size = row_payload_bytes(&row)? + size_of::<CursorRow>() as u64;
                    self.fetch_size.observe(size);
                    // Fetch sizing is only an estimate. Enforce the byte ceiling
                    // without advancing the returned cursor past an unaccepted row.
                    if !push_bounded_row(&mut page.rows, &mut payload, row, max_rows, limit)? {
                        break 'page;
                    }
                    previous = decoded.position;
                    fetched += 1;
                }
                op.check()?;
                if fetched < target {
                    break;
                }
            }
            Ok(Reply::Page(page))
        }
        .await;
        // Rollback also closes discarded portal rows. Never hold a snapshot over ACK.
        // An error discards the entire partial page; rollback failure takes precedence
        // so no otherwise-successful page escapes without transaction cleanup.
        let rollback = tx.rollback().await.map_err(database);
        rollback?;
        op.check()?;
        result
    }
}

fn page_metadata_bytes(page: &QueryPage) -> u64 {
    (size_of::<QueryPage>() + page.columns.capacity() * size_of::<ColumnMetadata>()) as u64
        + page
            .columns
            .iter()
            .map(|column| (column.name.capacity() + column.source_type.capacity()) as u64)
            .sum::<u64>()
}

fn row_payload_bytes(row: &CursorRow) -> Result<u64> {
    let cursor = row.cursor.as_composite().ok_or(Error::Value)?;
    // Row and cursor headers are already included in the CursorRow slots.
    Ok(row
        .row
        .normalized_size()
        .saturating_sub(size_of::<Row>() as u64)
        + cursor.timestamp.capacity() as u64)
}

fn push_bounded_row(
    rows: &mut Vec<CursorRow>,
    payload: &mut u64,
    row: CursorRow,
    max_rows: usize,
    limit: u64,
) -> Result<bool> {
    let next_payload = payload.saturating_add(row_payload_bytes(&row)?);
    let slot = size_of::<CursorRow>() as u64;
    let minimum_slots = rows.len() + 1;
    let minimum = next_payload.saturating_add(minimum_slots as u64 * slot);
    if rows.len() >= max_rows || minimum > limit {
        return if rows.is_empty() {
            Err(Error::Limit)
        } else {
            Ok(false)
        };
    }
    let available_slots = ((limit - next_payload) / slot).min(max_rows as u64) as usize;
    if rows.capacity() > available_slots {
        // Release speculative spare slots before rejecting a row that really
        // fits. Recheck actual capacity; allocator behavior is not an exemption.
        rows.shrink_to_fit();
    }
    if rows.len() == rows.capacity() {
        let target = rows
            .capacity()
            .saturating_mul(2)
            .max(4)
            .min(available_slots);
        rows.try_reserve_exact(target - rows.len())
            .map_err(|_| Error::Limit)?;
    }
    if next_payload.saturating_add(rows.capacity() as u64 * slot) > limit {
        return Err(Error::Limit);
    }
    *payload = next_payload;
    rows.push(row);
    Ok(true)
}

fn metadata(schema: &ResolvedSchema) -> Vec<ColumnMetadata> {
    schema
        .columns
        .iter()
        .map(|c| ColumnMetadata {
            name: c.name.clone(),
            source_type: c.source_type.clone(),
            nullable: c.nullable,
        })
        .collect()
}

fn ensure_advance(
    previous: &convert::CursorPosition,
    next: &convert::CursorPosition,
) -> Result<()> {
    if previous >= next {
        return Err(Error::Value);
    }

    Ok(())
}

#[cfg(test)]
postgresql_module_tests!(worker);

#[cfg(test)]
postgresql_module_tests!(framing);
