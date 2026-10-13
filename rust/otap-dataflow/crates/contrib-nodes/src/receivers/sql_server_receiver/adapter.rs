// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Local async adapter using stock Tiberius sessions and the shared delivery controller.

use super::{
    SqlServerConnectionConfig, SqlServerError as Error, SqlServerReceiverConfig, data::ResultPlan,
    session,
};
use async_trait::async_trait;
use futures::io::{AsyncRead, AsyncWrite};
use otel_arrow_dfe_engine::{
    capability::auth::basic_auth_provider::BASIC_AUTH_CREDENTIAL_USABLE_MARGIN,
    error::ReceiverErrorKind, local::capability::auth::basic_auth_provider::BasicAuthProvider,
};
use otel_arrow_dfe_scraper::database::{
    ColumnMetadata, CompiledQuery, CompiledWatermark, Cursor, DatabaseSystem, DriverAdapter,
    DriverCancellation, QueryPage,
};
use std::{cell::Cell, rc::Rc, time::Duration};
use tiberius::{AuthMethod, Client, Config, EncryptionLevel};
use tokio::{sync::Notify, time::Instant};
use tracing::instrument::WithSubscriber;

/// Establishes a Tiberius session without coupling polling to a TLS implementation.
///
/// Implementations must honor the SQL Server-specific endpoint TLS policy,
/// use the supplied SQL-login credentials, and cancel work when their future drops.
/// Any off-core work must be confirmed stopped by `shutdown` before source ownership
/// can be released. Never log the driver's Config or error text.
#[async_trait(?Send)]
pub trait SqlServerConnector {
    /// Tiberius requires a Send transport; adapter futures remain local and need not be Send.
    type Stream: AsyncRead + AsyncWrite + Unpin + Send;

    /// Opens one session; retry scheduling and the total deadline belong to the adapter.
    async fn connect(
        &mut self,
        endpoint: &SqlServerConnectionConfig,
        config: Config,
    ) -> Result<Client<Self::Stream>, Error>;

    /// Confirms that connection work has stopped, including after a dropped connect future.
    async fn shutdown(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

/// SQL Server DriverAdapter with pluggable connection setup for deterministic tests.
pub struct SqlServerAdapter<C: SqlServerConnector> {
    config: SqlServerReceiverConfig,
    connector: C,
    credentials: Box<dyn BasicAuthProvider>,
    client: Option<Client<C::Stream>>,
    plan: Option<ResultPlan>,
    cancellation: SqlServerCancellation,
    stopping: bool,
    cleanup_failed: bool,
}

#[derive(Default)]
struct CancellationState {
    requested: Cell<bool>,
    wake: Notify,
}

/// Local, operation-scoped cancellation. Old handles cannot affect later operations.
#[derive(Clone, Default)]
pub struct SqlServerCancellation(Rc<CancellationState>);

impl SqlServerCancellation {
    async fn cancelled(&self) {
        let wake = self.0.wake.notified();
        if !self.0.requested.get() {
            wake.await;
        }
    }
}

#[async_trait(?Send)]
impl DriverCancellation for SqlServerCancellation {
    type Error = Error;
    async fn cancel(&self) -> Result<(), Self::Error> {
        self.0.requested.set(true);
        self.0.wake.notify_waiters();
        Ok(())
    }
}

impl<C: SqlServerConnector> SqlServerAdapter<C> {
    /// Creates an adapter without connecting or accessing credentials.
    #[must_use]
    pub fn new(
        config: SqlServerReceiverConfig,
        connector: C,
        credentials: Box<dyn BasicAuthProvider>,
    ) -> Self {
        Self {
            config,
            connector,
            credentials,
            client: None,
            plan: None,
            cancellation: SqlServerCancellation::default(),
            stopping: false,
            cleanup_failed: false,
        }
    }

    fn check_query(&self, query: &CompiledQuery) -> Result<(), Error> {
        // Callers cannot substitute SQL or relax the adapter's validated budgets.
        let own = self.config.query();
        if query.sql() != own.sql()
            || query.max_rows() != own.max_rows()
            || query.max_batch_bytes() != own.max_batch_bytes()
            || query.fetch_size_rows() != own.fetch_size_rows()
            || query.timeout() != own.timeout()
            || !same_watermark(query.watermark(), own.watermark())
            || query.output().timestamp_column != own.output().timestamp_column
            || query.output().validation_columns != own.output().validation_columns
        {
            return Err(Error::NotValidated);
        }

        fn same_watermark(left: &CompiledWatermark, right: &CompiledWatermark) -> bool {
            match (left, right) {
                (CompiledWatermark::Snapshot, CompiledWatermark::Snapshot) => true,
                (CompiledWatermark::Scalar(left), CompiledWatermark::Scalar(right)) => {
                    left.column == right.column
                        && left.bind == right.bind
                        && left.initial == right.initial
                }
                (CompiledWatermark::Composite(left), CompiledWatermark::Composite(right)) => {
                    left.timestamp_column == right.timestamp_column
                        && left.timestamp_bind == right.timestamp_bind
                        && left.tie_breaker_column == right.tie_breaker_column
                        && left.tie_breaker_bind == right.tie_breaker_bind
                        && left.initial == right.initial
                }
                _ => false,
            }
        }
        if self.stopping || self.cleanup_failed {
            return Err(Error::NotValidated);
        }
        Ok(())
    }

    async fn connect(&mut self) -> Result<(), Error> {
        let cancel = self.cancellation.clone();
        let deadline = Instant::now() + self.config.connection().connect_timeout();
        let operation = async {
            let credential = self
                .credentials
                .get_credential()
                .await
                .map_err(|_| Error::Credentials)?;
            if credential.expires_on().is_some_and(|expires| {
                expires.saturating_duration_since(std::time::Instant::now())
                    <= BASIC_AUTH_CREDENTIAL_USABLE_MARGIN
            }) {
                return Err(Error::Credentials);
            }
            let endpoint = self.config.connection();
            let mut config = Config::new();
            config.host(endpoint.host());
            config.port(endpoint.port());
            config.database(endpoint.database());
            configure_tls(endpoint, &mut config)?;
            config.authentication(AuthMethod::sql_server(
                credential.expose_username(),
                credential.expose_password(),
            ));
            config.application_name("otel-arrow-sql-server");
            config.handshake_timeout(None);
            config.command_timeout(None);
            config.lossy_utf16_decoding(false);
            driver_call(self.connector.connect(endpoint, config)).await
        };
        self.client = Some(tokio::select! {
            biased;
            () = cancel.cancelled() => return Err(Error::Cancelled),
            () = tokio::time::sleep_until(deadline) => return Err(Error::Deadline),
            result = operation => result?,
        });
        Ok(())
    }

    async fn cancel_client(&mut self, client: &mut Client<C::Stream>) -> Result<(), Error> {
        // A partial stream is not assumed reusable. Cap ATTENTION and discard the client.
        match tokio::time::timeout(Duration::from_secs(1), driver_call(client.cancel_query())).await
        {
            Ok(Ok(())) => Ok(()),
            _ => {
                self.cleanup_failed = true;
                Err(Error::Driver)
            }
        }
    }

    async fn validate(&mut self, query: &CompiledQuery) -> Result<Vec<ColumnMetadata>, Error> {
        if self.client.is_none() {
            self.connect().await?;
        }
        let mut client = self.client.take().ok_or(Error::NotValidated)?;
        self.plan = None;
        let cancel = self.cancellation.clone();
        let deadline = Instant::now() + query.timeout();
        let result = tokio::select! {
            biased;
            () = cancel.cancelled() => Err(Error::Cancelled),
            () = tokio::time::sleep_until(deadline) => Err(Error::Deadline),
            result = driver_call(session::describe(&mut client, self.config.statement(), query)) => result,
        };
        match result {
            Ok(plan) => {
                let columns = plan.metadata();
                self.plan = Some(plan);
                self.client = Some(client);
                Ok(columns)
            }
            Err(error) => {
                if matches!(error, Error::Cancelled | Error::Deadline) {
                    self.cancel_client(&mut client).await?;
                }
                Err(error)
            }
        }
    }
}

#[async_trait(?Send)]
impl<C: SqlServerConnector> DriverAdapter for SqlServerAdapter<C> {
    type Error = Error;
    type Cancellation = SqlServerCancellation;

    fn system(&self) -> DatabaseSystem {
        DatabaseSystem::SqlServer
    }

    fn begin_operation(&mut self) -> Result<Self::Cancellation, Error> {
        if self.stopping || self.cleanup_failed {
            return Err(Error::NotValidated);
        }
        self.cancellation = SqlServerCancellation::default();
        Ok(self.cancellation.clone())
    }

    fn is_retryable(error: &Error) -> bool {
        matches!(
            error,
            Error::Transport(
                std::io::ErrorKind::ConnectionRefused
                    | std::io::ErrorKind::ConnectionReset
                    | std::io::ErrorKind::ConnectionAborted
                    | std::io::ErrorKind::BrokenPipe
                    | std::io::ErrorKind::UnexpectedEof
            ) | Error::Server(1205 | 40197 | 40501 | 40613)
        )
    }

    fn should_pause_source(error: &Error) -> bool {
        *error == Error::SnapshotLimit
    }

    async fn reconnect(&mut self, query: &CompiledQuery) -> Result<(), Error> {
        self.check_query(query)?;
        self.client = None;
        self.plan = None;
        self.connect().await
    }

    async fn validate_query(
        &mut self,
        query: &CompiledQuery,
    ) -> Result<Vec<ColumnMetadata>, Error> {
        self.check_query(query)?;
        self.validate(query).await
    }

    async fn execute(
        &mut self,
        query: &CompiledQuery,
        cursor: &Cursor,
    ) -> Result<QueryPage, Error> {
        self.check_query(query)?;
        if self.plan.is_none() {
            return Err(Error::NotValidated);
        }
        if self.client.is_none() {
            self.connect().await?;
        }
        let plan = self.plan.take().ok_or(Error::NotValidated)?;
        let mut client = self.client.take().ok_or(Error::NotValidated)?;
        let cancel = self.cancellation.clone();
        let deadline = Instant::now() + query.timeout();
        let work = async {
            let current = session::describe(&mut client, self.config.statement(), query).await?;
            if current.columns != plan.columns {
                return Err(Error::Metadata);
            }
            session::fetch(&mut client, self.config.statement(), &plan, query, cursor).await
        };
        let result = tokio::select! {
            biased;
            () = cancel.cancelled() => Err(Error::Cancelled),
            () = tokio::time::sleep_until(deadline) => Err(Error::Deadline),
            result = driver_call(work) => result,
        };
        match result {
            Ok(result) => {
                if result.unfinished {
                    self.cancel_client(&mut client).await?;
                    // Acknowledged prefixes continue on a fresh, revalidated session.
                } else {
                    self.client = Some(client);
                }
                self.plan = Some(plan);
                Ok(result.page)
            }
            Err(error) => {
                if matches!(
                    error,
                    Error::Cancelled | Error::Deadline | Error::SnapshotLimit | Error::OversizeRow
                ) {
                    self.cancel_client(&mut client).await?;
                }
                Err(error)
            }
        }
    }

    async fn shutdown(&mut self) -> Result<(), Error> {
        self.stopping = true;
        self.cancellation.cancel().await?;
        self.plan = None;
        let close = if let Some(client) = self.client.take() {
            tokio::time::timeout(Duration::from_secs(1), driver_call(client.close()))
                .await
                .map_err(|_| Error::Deadline)
                .and_then(|result| result.map_err(Error::from))
        } else {
            Ok(())
        };
        let connector = self.connector.shutdown().await;
        close?;
        connector?;
        if self.cleanup_failed {
            Err(Error::Driver)
        } else {
            Ok(())
        }
    }

    fn classify_error(error: &Error) -> ReceiverErrorKind {
        match error {
            Error::Transport(_) => ReceiverErrorKind::Connect,
            Error::Credentials
            | Error::Metadata
            | Error::Cursor
            | Error::NotValidated
            | Error::Tls
            | Error::TlsConfiguration => ReceiverErrorKind::Configuration,
            _ => ReceiverErrorKind::Transport,
        }
    }
}

async fn driver_call<F: Future>(future: F) -> F::Output {
    // Tiberius emits server text at ERROR and rows at TRACE. Suppress its subscriber
    // only; the shared controller retains normal redacted diagnostics and accounting.
    future
        .with_subscriber(tracing::subscriber::NoSubscriber::default())
        .await
}

pub(super) fn encryption_level(enabled: bool) -> EncryptionLevel {
    if enabled {
        EncryptionLevel::Required
    } else {
        // Off encrypts only the login. Server policy may refuse NotSupported.
        EncryptionLevel::NotSupported
    }
}

pub(super) fn configure_tls(
    endpoint: &SqlServerConnectionConfig,
    config: &mut Config,
) -> Result<(), Error> {
    // Defense in depth: Tiberius panics if trust_cert and a CA bundle are combined.
    if (endpoint.ca_file().is_some() && endpoint.trust_server_certificate())
        || (!endpoint.tls_enabled()
            && (endpoint.ca_file().is_some()
                || endpoint.server_name_override().is_some()
                || endpoint.trust_server_certificate()))
    {
        return Err(Error::TlsConfiguration);
    }
    config.encryption(encryption_level(endpoint.tls_enabled()));
    config.hostname_in_certificate(endpoint.server_name_override().unwrap_or(endpoint.host()));
    if endpoint.trust_server_certificate() {
        config.trust_cert();
    }
    Ok(())
}
