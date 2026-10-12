// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Capacity-one local proxy. The shared receiver owns polling and feedback.

use super::{config::Validated, worker};
use async_trait::async_trait;
use otel_arrow_dfe_engine::capability::auth::BasicAuthCredential;
use otel_arrow_dfe_engine::capability::auth::basic_auth_provider::BASIC_AUTH_CREDENTIAL_USABLE_MARGIN;
use otel_arrow_dfe_engine::local::capability::auth::basic_auth_provider::BasicAuthProvider;
use otel_arrow_dfe_scraper::database::{
    ColumnMetadata, CompiledQuery, CompositeCursor, Cursor, DatabaseSystem, DriverAdapter,
    DriverCancellation, QueryPage,
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::JoinHandle,
    time::Duration,
};
use tokio::sync::{Notify, mpsc, oneshot};

#[derive(Default)]
pub(crate) struct Operation {
    pub cancelled: AtomicBool,
    pub notify: Notify,
}
impl Operation {
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
        self.notify.notify_one();
    }
    pub fn check(&self) -> Result<()> {
        if self.cancelled.load(Ordering::Acquire) {
            Err(Error::Cancelled)
        } else {
            Ok(())
        }
    }
}

#[derive(Clone)]
pub(crate) struct Cancellation(Arc<Operation>);
#[async_trait(?Send)]
impl DriverCancellation for Cancellation {
    type Error = Error;
    async fn cancel(&self) -> Result<()> {
        self.0.cancel();
        Ok(())
    }
}

pub(crate) enum Action {
    Validate,
    Reconnect,
    Execute(CompositeCursor),
    Shutdown,
}
pub(crate) enum Reply {
    Columns(Vec<ColumnMetadata>),
    Page(QueryPage),
    Done,
}
pub(crate) struct Command {
    pub action: Action,
    pub operation: Arc<Operation>,
    pub reply: oneshot::Sender<Result<Reply>>,
    pub credential: Option<BasicAuthCredential>,
}
pub(crate) struct PostgreSqlAdapter {
    credentials: Box<dyn BasicAuthProvider>,
    commands: mpsc::Sender<Command>,
    operation: Arc<Operation>,
    busy: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
    exit: oneshot::Receiver<Result<()>>,
    exit_result: Option<Result<()>>,
    stopped: bool,
    timeout: Duration,
}

impl PostgreSqlAdapter {
    pub fn new(validated: Validated, credentials: Box<dyn BasicAuthProvider>) -> Result<Self> {
        let (commands, rx) = mpsc::channel(1);
        let (exit_tx, exit) = oneshot::channel();
        let busy = Arc::new(AtomicBool::new(false));
        // The local adapter and its one dedicated worker share only admission
        // and cancellation signals; database state stays on the worker.
        let worker_busy = busy.clone();
        let timeout = validated.common.timeout();
        let thread = std::thread::Builder::new()
            .name("postgresql-receiver".into())
            .spawn(move || {
                let outcome = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|_| Error::Cleanup)
                    .and_then(|runtime| {
                        tokio::task::LocalSet::new()
                            .block_on(&runtime, worker::run(validated, rx, worker_busy))
                    });
                let _ = exit_tx.send(outcome);
            })
            .map_err(|_| Error::Cleanup)?;
        Ok(Self {
            credentials,
            commands,
            operation: Arc::new(Operation::default()),
            busy,
            thread: Some(thread),
            exit,
            exit_result: None,
            stopped: false,
            timeout,
        })
    }

    async fn request(&mut self, action: Action) -> Result<Reply> {
        self.operation.check()?;
        // Resolve the local capability here and send only its credential snapshot.
        // The worker consumes it when connecting, not to replace a healthy session.
        let credential = tokio::select! {
            biased;
            _ = self.operation.notify.notified() => return Err(Error::Cancelled),
            result = tokio::time::timeout(self.timeout, self.credentials.get_credential()) => {
                result.map_err(|_| Error::Credential)?.map_err(|_| Error::Credential)?
            }
        };
        self.operation.check()?;
        if credential.expires_on().is_some_and(|expiry| {
            expiry.saturating_duration_since(std::time::Instant::now())
                <= BASIC_AUTH_CREDENTIAL_USABLE_MARGIN
        }) {
            return Err(Error::Credential);
        }
        // A dropped response future does not mean its database work stopped.
        // Only the worker releases this gate after completing work and any cleanup.
        if self.busy.swap(true, Ordering::AcqRel) {
            return Err(Error::Cleanup);
        }
        let (reply, result) = oneshot::channel();
        self.commands
            .send(Command {
                action,
                operation: self.operation.clone(),
                reply,
                credential: Some(credential),
            })
            .await
            .map_err(|_| Error::Cleanup)?;
        result.await.map_err(|_| Error::Cleanup)?
    }
}

#[async_trait(?Send)]
impl DriverAdapter for PostgreSqlAdapter {
    type Error = Error;
    type Cancellation = Cancellation;
    fn system(&self) -> DatabaseSystem {
        DatabaseSystem::PostgreSQL
    }
    fn begin_operation(&mut self) -> Result<Cancellation> {
        if self.stopped || self.exit_result.is_some() || self.busy.load(Ordering::Acquire) {
            return Err(Error::Cleanup);
        }
        self.operation = Arc::new(Operation::default());
        Ok(Cancellation(self.operation.clone()))
    }
    fn is_retryable(error: &Error) -> bool {
        matches!(
            error,
            Error::Unavailable | Error::Cancelled | Error::Timeout
        )
    }
    async fn reconnect(&mut self, _: &CompiledQuery) -> Result<()> {
        match self.request(Action::Reconnect).await? {
            Reply::Done => Ok(()),
            _ => Err(Error::Cleanup),
        }
    }
    async fn validate_query(&mut self, _: &CompiledQuery) -> Result<Vec<ColumnMetadata>> {
        match self.request(Action::Validate).await? {
            Reply::Columns(columns) => Ok(columns),
            _ => Err(Error::Cleanup),
        }
    }
    async fn execute(&mut self, query: &CompiledQuery, cursor: &Cursor) -> Result<QueryPage> {
        query
            .watermark()
            .validate_cursor(cursor)
            .map_err(|_| Error::Value)?;
        let cursor = match cursor {
            Cursor::Composite(cursor) => cursor,
            Cursor::Scalar(_) => return Err(Error::Config),
        };
        match self.request(Action::Execute(cursor.clone())).await? {
            Reply::Page(page) => Ok(page),
            _ => Err(Error::Cleanup),
        }
    }
    async fn shutdown(&mut self) -> Result<()> {
        if self.stopped {
            return Ok(());
        }
        self.operation.cancel();
        let (reply, _result) = oneshot::channel();
        if self.exit_result.is_none() {
            let _ = self
                .commands
                .send(Command {
                    action: Action::Shutdown,
                    operation: self.operation.clone(),
                    reply,
                    credential: None,
                })
                .await;
            self.exit_result = Some((&mut self.exit).await.unwrap_or(Err(Error::Cleanup)));
        }
        self.exit_result.ok_or(Error::Cleanup)??;
        // Exit acknowledgment precedes the final thread return by a few instructions.
        while self.thread.as_ref().is_some_and(|t| !t.is_finished()) {
            tokio::task::yield_now().await;
        }
        if let Some(thread) = self.thread.take()
            && thread.join().is_err()
        {
            self.exit_result = Some(Err(Error::Cleanup));
            return Err(Error::Cleanup);
        }
        self.stopped = true;
        Ok(())
    }

    // Reporting categories do not authorize retries; is_retryable is the narrower
    // policy, and Cleanup always remains terminal.
    fn classify_error(error: &Error) -> otel_arrow_dfe_engine::error::ReceiverErrorKind {
        use otel_arrow_dfe_engine::error::ReceiverErrorKind;
        match error {
            Error::Config | Error::Sql | Error::Metadata | Error::Credential => {
                ReceiverErrorKind::Configuration
            }
            Error::Value | Error::Limit | Error::Database | Error::Unavailable | Error::Timeout => {
                ReceiverErrorKind::Transport
            }
            Error::Cancelled | Error::Cleanup => ReceiverErrorKind::Shutdown,
        }
    }
}

impl Drop for PostgreSqlAdapter {
    fn drop(&mut self) {
        self.operation.cancel();
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub(crate) enum Error {
    #[error("postgresql: invalid configuration")]
    Config,
    #[error("postgresql: unsupported SQL contract")]
    Sql,
    #[error("postgresql: incompatible source metadata")]
    Metadata,
    #[error("postgresql: invalid or unsupported source value")]
    Value,
    #[error("postgresql: protocol or memory limit exceeded")]
    Limit,
    #[error("postgresql: credential provider unavailable, expired, or invalid")]
    Credential,
    #[error("postgresql: database operation rejected")]
    Database,
    #[error("postgresql: transport unavailable after cleanup")]
    Unavailable,
    #[error("postgresql: cancelled operation stopped")]
    Cancelled,
    #[error("postgresql: query timed out after cleanup")]
    Timeout,
    #[error("postgresql: cleanup unconfirmed; source requires process restart")]
    Cleanup,
}

pub(crate) type Result<T> = std::result::Result<T, Error>;

pub(crate) fn database(error: tokio_postgres::Error) -> Error {
    if let Some(code) = error.code() {
        sqlstate(code.code())
    } else if error.is_closed() {
        Error::Unavailable
    } else {
        // Only actual transport I/O is retryable.
        use std::error::Error as _;
        let mut source = error.source();
        while let Some(value) = source {
            if let Some(io) = value.downcast_ref::<std::io::Error>()
                && matches!(
                    io.kind(),
                    std::io::ErrorKind::ConnectionRefused
                        | std::io::ErrorKind::ConnectionReset
                        | std::io::ErrorKind::ConnectionAborted
                        | std::io::ErrorKind::NotConnected
                        | std::io::ErrorKind::TimedOut
                        | std::io::ErrorKind::UnexpectedEof
                        | std::io::ErrorKind::BrokenPipe
                )
            {
                return Error::Unavailable;
            }
            source = value.source();
        }
        Error::Database
    }
}

fn sqlstate(code: &str) -> Error {
    if code == "57014" {
        Error::Timeout
    } else if code.starts_with("08")
        || matches!(
            code,
            "53300" | "57P01" | "57P02" | "57P03" | "40001" | "40P01"
        )
    {
        Error::Unavailable
    } else {
        Error::Database
    }
}

#[cfg(test)]
postgresql_module_tests!();
