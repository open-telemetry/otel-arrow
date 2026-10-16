// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Pipeline-local PostgreSQL adapter.

use super::{
    config::Validated,
    query::{ResolvedSchema, Signature},
    value,
    worker::{CancelSlot, FetchSize, Session, SessionClose},
};
use async_trait::async_trait;
use otel_arrow_dfe_engine::capability::auth::BasicAuthCredential;
use otel_arrow_dfe_engine::capability::auth::basic_auth_provider::BASIC_AUTH_CREDENTIAL_USABLE_MARGIN;
use otel_arrow_dfe_engine::local::capability::auth::basic_auth_provider::BasicAuthProvider;
use otel_arrow_dfe_scraper::database::{
    ColumnMetadata, CompiledQuery, Cursor, DatabaseSystem, DriverAdapter, DriverCancellation,
    QueryPage,
};
use secrecy::SecretString;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::Duration,
};
use tokio::time::Instant;

#[derive(Default)]
pub(crate) struct Operation {
    cancelled: Cell<bool>,
    active: Cell<bool>,
    abandoned: Cell<bool>,
    pub(crate) notify: tokio::sync::Notify,
}

impl Operation {
    pub(crate) fn cancel(&self) {
        self.cancelled.set(true);
        self.notify.notify_one();
    }

    pub(crate) fn check(&self) -> Result<()> {
        if self.cancelled.get() {
            Err(Error::Cancelled)
        } else {
            Ok(())
        }
    }
}

/// Tracks one in-flight adapter operation so a dropped request is observable.
pub(crate) struct OperationGuard {
    operation: Rc<Operation>,
    completed: bool,
}

impl OperationGuard {
    pub(crate) fn start(operation: Rc<Operation>) -> Result<Self> {
        operation.check()?;
        if operation.active.replace(true) {
            return Err(Error::Cleanup);
        }
        Ok(Self {
            operation,
            completed: false,
        })
    }

    fn complete(mut self) {
        self.operation.active.set(false);
        self.completed = true;
    }
}

impl Drop for OperationGuard {
    fn drop(&mut self) {
        if !self.completed {
            // A dropped operation cannot prove that PostgreSQL stopped its work.
            // Quarantine this adapter until shutdown confirms session cleanup.
            self.operation.abandoned.set(true);
            self.operation.cancel();
        }
    }
}

#[derive(Clone)]
pub(crate) struct Cancellation(Rc<Operation>);

#[async_trait(?Send)]
impl DriverCancellation for Cancellation {
    type Error = Error;

    async fn cancel(&self) -> Result<()> {
        self.0.cancel();
        Ok(())
    }
}

pub(crate) struct CachedSchema {
    pub(crate) signature: Signature,
    pub(crate) resolved: ResolvedSchema,
}

pub(crate) struct PostgreSqlAdapter {
    pub(crate) credentials: Box<dyn BasicAuthProvider>,
    pub(crate) validated: Validated,
    pub(crate) session: Option<Session>,
    pub(crate) schema: Option<CachedSchema>,
    pub(crate) cancel: CancelSlot,
    pub(crate) poisoned: bool,
    pub(crate) principal: Option<SecretString>,
    pub(crate) fetch_size: FetchSize,
    pub(crate) timestamp_cache: value::CursorTimestampCache,
    pub(crate) operation: Rc<Operation>,
    pub(crate) reconnect_not_before: Option<Instant>,
    stopped: bool,
    shutdown_result: Option<Result<()>>,
    timeout: Duration,
}

impl PostgreSqlAdapter {
    pub(crate) fn new(
        validated: Validated,
        credentials: Box<dyn BasicAuthProvider>,
    ) -> Result<Self> {
        let timeout = validated.common.timeout();
        Ok(Self {
            credentials,
            validated,
            session: None,
            schema: None,
            cancel: Rc::new(RefCell::new(None)),
            poisoned: false,
            principal: None,
            fetch_size: FetchSize::default(),
            timestamp_cache: value::CursorTimestampCache::default(),
            operation: Rc::new(Operation::default()),
            reconnect_not_before: None,
            stopped: false,
            shutdown_result: None,
            timeout,
        })
    }

    async fn credential(&self, operation: &Operation) -> Result<BasicAuthCredential> {
        operation.check()?;
        let credential = tokio::select! {
            biased;
            _ = operation.notify.notified() => return Err(Error::Cancelled),
            result = tokio::time::timeout(self.timeout, self.credentials.get_credential()) => {
                result.map_err(|_| Error::Credential)?.map_err(|_| Error::Credential)?
            }
        };
        operation.check()?;
        if credential.expires_on().is_some_and(|expiry| {
            expiry.saturating_duration_since(std::time::Instant::now())
                <= BASIC_AUTH_CREDENTIAL_USABLE_MARGIN
        }) {
            return Err(Error::Credential);
        }
        Ok(credential)
    }

    async fn run(
        &mut self,
        action: worker::Action,
        operation: Rc<Operation>,
    ) -> Result<worker::Reply> {
        let guard = OperationGuard::start(operation.clone())?;
        let result = match self.credential(&operation).await {
            Ok(credential) => self.run_worker(action, &operation, &credential).await,
            Err(error) => Err(error),
        };
        guard.complete();
        result
    }
}

use super::worker;

#[async_trait(?Send)]
impl DriverAdapter for PostgreSqlAdapter {
    type Error = Error;
    type Cancellation = Cancellation;

    fn system(&self) -> DatabaseSystem {
        DatabaseSystem::PostgreSQL
    }

    fn begin_operation(&mut self) -> Result<Cancellation> {
        if self.stopped
            || self.poisoned
            || self.operation.active.get()
            || self.operation.abandoned.get()
        {
            return Err(Error::Cleanup);
        }
        self.operation = Rc::new(Operation::default());
        Ok(Cancellation(self.operation.clone()))
    }

    fn is_retryable(error: &Error) -> bool {
        matches!(
            error,
            Error::Unavailable | Error::Cancelled | Error::Timeout
        )
    }

    async fn reconnect(&mut self, _: &CompiledQuery) -> Result<()> {
        let operation = self.operation.clone();
        match self.run(worker::Action::Reconnect, operation).await? {
            worker::Reply::Done => Ok(()),
            _ => Err(Error::Cleanup),
        }
    }

    async fn validate_query(&mut self, _: &CompiledQuery) -> Result<Vec<ColumnMetadata>> {
        let operation = self.operation.clone();
        match self.run(worker::Action::Validate, operation).await? {
            worker::Reply::Columns(columns) => Ok(columns),
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
            Cursor::Snapshot | Cursor::Scalar(_) => return Err(Error::Config),
        };
        let operation = self.operation.clone();
        match self
            .run(worker::Action::Execute(cursor.clone()), operation)
            .await?
        {
            worker::Reply::Page(page) => Ok(page),
            _ => Err(Error::Cleanup),
        }
    }

    async fn shutdown(&mut self) -> Result<()> {
        if let Some(result) = self.shutdown_result {
            return result;
        }
        self.stopped = true;
        self.operation.cancel();
        let abandoned = self.operation.abandoned.get();
        if abandoned {
            // A dropped operation never observed its own completion, so no later
            // local join can prove PostgreSQL stopped the abandoned statement.
            // Report unconfirmed cleanup instead of letting the controller clear
            // provisional abandonment and release source ownership.
            self.poisoned = true;
        }
        if self
            .reconnect_not_before
            .is_some_and(|deadline| deadline > Instant::now())
        {
            // Do not release source ownership while an abandoned server query
            // may still be running. Process shutdown remains the escape hatch.
            self.poisoned = true;
        }
        let reason = if abandoned {
            worker::CloseReason::OperationInFlight
        } else {
            worker::CloseReason::Idle
        };
        let result = match self.close_session(reason).await {
            Ok(SessionClose::Graceful) if !self.poisoned => Ok(()),
            Ok(SessionClose::Graceful | SessionClose::Aborted) | Err(_) => Err(Error::Cleanup),
        };
        self.shutdown_result = Some(result);
        result
    }

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
    } else if code.starts_with("28") {
        Error::Credential
    } else if code.starts_with("08")
        || matches!(
            code,
            "53300" | "55P03" | "57P01" | "57P02" | "57P03" | "57P05" | "40001" | "40P01"
        )
    {
        Error::Unavailable
    } else {
        Error::Database
    }
}

#[cfg(test)]
postgresql_module_tests!();
