// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Single connection owner with read-only portal pages and observed cleanup.

use super::{
    adapter::{Action, Command, Error, Operation, Reply, Result, database},
    config::Validated,
    query::{CatalogStatements, CursorColumns, ResolvedSchema, ScalarKind, Signature},
    value as convert,
};
use futures::{StreamExt, future::poll_fn};
use otel_arrow_dfe_engine::capability::auth::BasicAuthCredential;
use otel_arrow_dfe_scraper::database::{
    ColumnMetadata, Cursor, CursorRow, QueryPage, Row, ScalarValue,
};
use secrecy::ExposeSecret;
use std::{
    cell::{Cell, RefCell},
    io,
    net::SocketAddr,
    pin::Pin,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Poll, ready},
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    sync::mpsc,
    task::JoinHandle,
};
use tokio_postgres::{
    CancelToken, Client, IsolationLevel, NoTls, Statement, config::SslMode, types::ToSql,
};

type CancelSlot = Rc<RefCell<Option<(CancelToken, SocketAddr)>>>;

const CONNECTION_TIMEOUT: Duration = Duration::from_secs(10);
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
                        if this.header[0] == b'D' {
                            Error::DataRowLimit
                        } else {
                            Error::Limit
                        },
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

struct Session {
    client: Option<Client>,
    driver: JoinHandle<Result<()>>,
    statement: Option<Statement>,
    catalog: Option<CatalogStatements>,
    receive_failure: ReceiveFailure,
}
struct Worker {
    validated: Validated,
    session: Option<Session>,
    schema: Option<CachedSchema>,
    cancel: CancelSlot,
    poisoned: bool,
    principal: Option<secrecy::SecretString>,
    fetch_size: FetchSize,
    timestamp_cache: convert::CursorTimestampCache,
    snapshot_active: bool,
}

struct CachedSchema {
    signature: Signature,
    resolved: ResolvedSchema,
}

fn resolve_schema<'a>(
    cache: &'a mut Option<CachedSchema>,
    validated: &Validated,
    signature: Signature,
) -> Result<&'a CachedSchema> {
    if let Some(cached) = cache {
        if cached.signature != signature {
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
struct FetchSize {
    largest_row_bytes: Option<u64>,
}

impl FetchSize {
    fn target(&self, row_limit: usize, bytes_left: u64) -> usize {
        self.largest_row_bytes
            .map_or(1, |size| bytes_left / size)
            .min(row_limit as u64) as usize
    }

    fn observe(&mut self, row_bytes: u64) {
        self.largest_row_bytes = Some(self.largest_row_bytes.unwrap_or(1).max(row_bytes));
    }
}

pub(crate) async fn run(
    validated: Validated,
    mut commands: mpsc::Receiver<Command>,
    busy: Arc<AtomicBool>,
) -> Result<()> {
    let mut worker = Worker {
        validated,
        session: None,
        schema: None,
        cancel: Rc::new(RefCell::new(None)),
        poisoned: false,
        principal: None,
        fetch_size: FetchSize::default(),
        timestamp_cache: convert::CursorTimestampCache::default(),
        snapshot_active: false,
    };
    while let Some(command) = commands.recv().await {
        if matches!(command.action, Action::Shutdown) {
            return worker.close().await.and(if worker.poisoned {
                Err(Error::Cleanup)
            } else {
                Ok(())
            });
        }
        let cancel = worker.cancel.clone();
        let operation = command.operation.clone();
        let timeout = worker.validated.common.timeout();
        let mut result = {
            let credential = command.credential.ok_or(Error::Credential)?;
            let future = worker.perform(command.action, &operation, &credential);
            tokio::pin!(future);
            tokio::select! {
                biased;
                result = &mut future => result,
                _ = operation.notify.notified() => {
                    operation.cancel();
                    cancel_native(&cancel).await;
                    match tokio::time::timeout(CANCEL_COMPLETION_TIMEOUT, &mut future).await {
                        Ok(_) => Err(Error::Cancelled),
                        Err(_) => Err(Error::Cleanup),
                    }
                }
                _ = tokio::time::sleep(timeout) => {
                    operation.cancel();
                    cancel_native(&cancel).await;
                    match tokio::time::timeout(CANCEL_COMPLETION_TIMEOUT, &mut future).await {
                        Ok(_) => Err(Error::Timeout),
                        Err(_) => Err(Error::Cleanup),
                    }
                }
            }
        };
        // Requests see only a closed driver channel after a framing failure.
        // Preserve the actual receive limit instead of retrying an unavailable DB.
        if result.as_ref().err() != Some(&Error::Cleanup)
            && let Some(error) = worker
                .session
                .as_ref()
                .and_then(|session| session.receive_failure.get())
        {
            result = Err(error);
        }
        if result.as_ref().err() == Some(&Error::Cleanup) {
            // A later successful close cannot prove an earlier abandoned operation
            // stopped. Keep shutdown terminal so shared source ownership is quarantined.
            worker.poisoned = true;
        }
        if result.is_err() {
            operation.cancel();
            if worker.close().await.is_err() {
                worker.poisoned = true;
                result = Err(Error::Cleanup);
            }
        }
        // Only data-query frames are snapshot budget failures. Catalog and
        // control-frame failures retain their terminal classification. Pause
        // becomes safe only after the worker has confirmed connection cleanup.
        result = result.map_err(|error| after_cleanup(error, worker.snapshot_active));
        busy.store(false, Ordering::Release);
        let _ = command.reply.send(result);
    }
    worker.close().await?;
    if worker.poisoned {
        Err(Error::Cleanup)
    } else {
        Ok(())
    }
}

fn after_cleanup(error: Error, snapshot_active: bool) -> Error {
    if snapshot_active && error == Error::DataRowLimit {
        Error::SnapshotByteLimit
    } else {
        error
    }
}

fn validate_server_encoding(cursor: CursorColumns, encoding: Option<&str>) -> Result<()> {
    if matches!(
        cursor,
        CursorColumns::Scalar {
            kind: ScalarKind::String,
            ..
        }
    ) && encoding != Some("UTF8")
    {
        return Err(Error::Metadata);
    }
    Ok(())
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

impl Worker {
    async fn close(&mut self) -> Result<()> {
        let _ = self.cancel.borrow_mut().take();
        if let Some(session) = self.session.as_mut() {
            let _ = session.statement.take();
            let _ = session.catalog.take();
            let _ = session.client.take();
            let stopped = match tokio::time::timeout(
                CONNECTION_SHUTDOWN_TIMEOUT,
                &mut session.driver,
            )
            .await
            {
                Ok(joined) => joined.is_ok(),
                Err(_) => {
                    // Aborting is not proof that the server operation stopped.
                    session.driver.abort();
                    let _ = (&mut session.driver).await;
                    false
                }
            };
            let _ = self.session.take();
            if !stopped {
                self.poisoned = true;
                return Err(Error::Cleanup);
            }
        }
        Ok(())
    }

    async fn connect(&mut self, op: &Operation, credential: &BasicAuthCredential) -> Result<()> {
        if self.poisoned {
            return Err(Error::Cleanup);
        }
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
            .options(format!(
                "-c search_path=pg_catalog -c TimeZone=UTC -c default_transaction_read_only=on \
                -c statement_timeout={} -c idle_in_transaction_session_timeout={} \
                -c application_name=otel_arrow_postgresql_receiver",
                self.validated.common.timeout().as_millis(),
                self.validated.common.timeout().as_millis(),
            ));
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
        validate_server_encoding(
            self.validated.plan.cursor,
            connection.parameter("server_encoding"),
        )?;
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
        self.snapshot_active = false;
        op.check()?;
        if self.poisoned {
            return Err(Error::Cleanup);
        }
        if matches!(action, Action::Reconnect) {
            self.close().await?;
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
                let parameter_types: Vec<_> = plan
                    .cursor
                    .indices()
                    .iter()
                    .map(|&index| schema.native_types[index].clone())
                    .collect();
                let statement = tx
                    .prepare_typed(&plan.sql, &parameter_types)
                    .await
                    .map_err(database)?;
                op.check()?;
                if statement.params() != parameter_types
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
            self.validated
                .common
                .watermark()
                .validate_cursor(&cursor)
                .map_err(|_| Error::Value)?;
            let parameters = convert::parameters(&cursor, plan, schema)?;
            let parameters: Vec<&(dyn ToSql + Sync)> =
                parameters.iter().map(|p| p.as_ref()).collect();
            let mut previous = convert::cursor_position(&cursor)?;
            let snapshot = matches!(plan.cursor, CursorColumns::Snapshot);
            op.check()?;
            let statement = session.statement.as_ref().ok_or(Error::Metadata)?;
            let portal = tx.bind(statement, &parameters).await.map_err(database)?;
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
                return Err(if snapshot {
                    Error::SnapshotByteLimit
                } else {
                    Error::Limit
                });
            }
            self.snapshot_active = snapshot;
            let probe_rows = max_rows + usize::from(snapshot);
            'page: while page.rows.len() < probe_rows {
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
                        (probe_rows - page.rows.len()).min(self.validated.common.fetch_size_rows()),
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
                    if snapshot && page.rows.len() == max_rows {
                        return Err(Error::SnapshotRowLimit);
                    }
                    let decoded =
                        convert::row(&native, plan, schema, &mut self.timestamp_cache, raw_limit)
                            .map_err(|error| {
                            if snapshot && error == Error::Limit {
                                Error::SnapshotByteLimit
                            } else {
                                error
                            }
                        })?;
                    op.check()?;
                    if !snapshot {
                        ensure_advance(&previous, &decoded.position)?;
                    }
                    let row = decoded.row;
                    let size = row_payload_bytes(&row)? + size_of::<CursorRow>() as u64;
                    self.fetch_size.observe(size);
                    // Fetch sizing is only an estimate. Enforce the byte ceiling
                    // without advancing the returned cursor past an unaccepted row.
                    if !admit_row(&mut page.rows, &mut payload, row, max_rows, limit, snapshot)? {
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
    let cursor_bytes = match &row.cursor {
        Cursor::Snapshot => 0,
        Cursor::Composite(cursor) => cursor.timestamp.capacity(),
        Cursor::Scalar(ScalarValue::String(text) | ScalarValue::Timestamp(text)) => text.capacity(),
        Cursor::Scalar(_) => 0,
    };
    // Row and cursor headers are already included in the CursorRow slots.
    Ok(row
        .row
        .normalized_size()
        .saturating_sub(size_of::<Row>() as u64)
        + cursor_bytes as u64)
}

fn admit_row(
    rows: &mut Vec<CursorRow>,
    payload: &mut u64,
    row: CursorRow,
    max_rows: usize,
    limit: u64,
    snapshot: bool,
) -> Result<bool> {
    if snapshot && rows.len() >= max_rows {
        return Err(Error::SnapshotRowLimit);
    }
    match push_bounded_row(rows, payload, row, max_rows, limit) {
        Ok(false) | Err(Error::Limit) if snapshot => Err(Error::SnapshotByteLimit),
        result => result,
    }
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

fn ensure_advance<T: PartialOrd>(previous: &T, next: &T) -> Result<()> {
    if previous.partial_cmp(next) != Some(std::cmp::Ordering::Less) {
        return Err(Error::Value);
    }

    Ok(())
}

#[cfg(test)]
postgresql_module_tests!(worker);

#[cfg(test)]
postgresql_module_tests!(framing);
