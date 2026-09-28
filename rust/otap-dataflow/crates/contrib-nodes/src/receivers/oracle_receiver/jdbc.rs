// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! JDBC Oracle adapter accessed through the JVM invocation and JNI APIs.

use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, Utc};
use jni::objects::{Global, JObject, JString, JValue};
use jni::vm::{InitArgsBuilder, JavaVM};
use jni::{Env, JNIVersion, jni_sig, jni_str};
use otel_arrow_dfe_engine::error::ReceiverErrorKind;
use otel_arrow_dfe_scraper::database::{
    CellValue, ColumnMetadata, CompiledQuery, CompositeCursor, CursorRow, DatabaseSystem,
    DriverAdapter, DriverCancellation, QueryPage, Row,
};
use secrecy::{ExposeSecret, SecretString};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

const MAX_CREDENTIAL_BYTES: u64 = 64 * 1024;

#[derive(Clone)]
pub(crate) struct JdbcAdapterConfig {
    pub(crate) url: String,
    pub(crate) driver_jar: String,
    pub(crate) java_home: String,
    pub(crate) username_file: String,
    pub(crate) password_file: String,
}

pub struct JdbcAdapter {
    config: JdbcAdapterConfig,
    session: Option<JdbcSession>,
    cancellation: JdbcCancellation,
}

struct JdbcSession {
    vm: Arc<JavaVM>,
}

#[derive(Clone, Default)]
pub struct JdbcCancellation {
    // Shares the active JDBC statement between the local receiver and its blocking worker.
    state: Arc<Mutex<JdbcCancellationState>>,
}

#[derive(Default)]
struct JdbcCancellationState {
    requested: bool,
    vm: Option<Arc<JavaVM>>,
    statement: Option<Arc<Global<JObject<'static>>>>,
}

struct ActiveStatement {
    cancellation: JdbcCancellation,
}

impl Drop for ActiveStatement {
    fn drop(&mut self) {
        if let Ok(mut state) = self.cancellation.state.lock() {
            state.statement = None;
            state.vm = None;
        }
    }
}

struct JvmState {
    java_home: PathBuf,
    driver_jar: PathBuf,
    vm: Arc<JavaVM>,
}

static JVM: OnceLock<Result<JvmState, String>> = OnceLock::new();

#[derive(Debug, thiserror::Error)]
pub enum JdbcAdapterError {
    #[error("JNI operation failed: {0}")]
    Jni(#[from] jni::errors::Error),
    #[error("JVM initialization failed: {0}")]
    Jvm(String),
    #[error("JDBC operation failed: {0}")]
    Jdbc(String),
    #[error("transient JDBC connection failure")]
    Retryable,
    #[error("JDBC value conversion failed: {0}")]
    Conversion(String),
    #[error("JDBC worker failed")]
    Worker(#[source] tokio::task::JoinError),
    #[error("JDBC credential error")]
    Credential(#[source] std::io::Error),
    #[error("JDBC credential file is not a regular file")]
    CredentialNotRegularFile,
    #[error("JDBC credential file is empty")]
    EmptyCredential,
    #[error("JDBC credential file is not valid UTF-8")]
    InvalidCredentialEncoding,
    #[error("JDBC row exceeds the configured normalized byte limit")]
    NormalizedByteLimit,
    #[error("JDBC operation was cancelled")]
    Cancelled,
    #[error("JDBC query exceeded its configured timeout")]
    Timeout,
    #[error("JDBC cancellation state is unavailable")]
    CancellationState,
}

impl JdbcAdapter {
    pub(crate) fn new(config: JdbcAdapterConfig) -> Self {
        Self {
            config,
            session: None,
            cancellation: JdbcCancellation::default(),
        }
    }

    async fn run<T>(
        &mut self,
        query: &CompiledQuery,
        cursor: &CompositeCursor,
        operation: fn(
            &JdbcSession,
            &JdbcAdapterConfig,
            &CompiledQuery,
            &CompositeCursor,
            &JdbcCancellation,
        ) -> Result<T, JdbcAdapterError>,
    ) -> Result<T, JdbcAdapterError>
    where
        T: Send + 'static,
    {
        let session = self.session.take();
        let config = self.config.clone();
        let query = query.clone();
        let timeout = query.timeout();
        let cursor = cursor.clone();
        let cancellation = self.cancellation.clone();
        let worker = tokio::task::spawn_blocking(move || {
            let session = match session {
                Some(session) => session,
                None => open_jvm(&config)?,
            };
            let result = operation(&session, &config, &query, &cursor, &cancellation);
            Ok::<_, JdbcAdapterError>((session, result))
        });
        tokio::pin!(worker);
        let result = tokio::select! {
            result = &mut worker => result.map_err(JdbcAdapterError::Worker)??,
            () = tokio::time::sleep(timeout) => {
                let cancellation_result = self.cancellation.cancel().await;
                let worker_result = worker.await.map_err(JdbcAdapterError::Worker)?;
                let (session, _) = worker_result?;
                self.session = Some(session);
                cancellation_result?;
                return Err(JdbcAdapterError::Timeout);
            }
        };
        self.session = Some(result.0);
        result.1
    }
}

#[async_trait(?Send)]
impl DriverCancellation for JdbcCancellation {
    type Error = JdbcAdapterError;

    async fn cancel(&self) -> Result<(), Self::Error> {
        let active = {
            let mut state = self
                .state
                .lock()
                .map_err(|_| JdbcAdapterError::CancellationState)?;
            state.requested = true;
            state
                .vm
                .as_ref()
                .zip(state.statement.as_ref())
                .map(|(vm, statement)| (Arc::clone(vm), Arc::clone(statement)))
        };
        let Some((vm, statement)) = active else {
            return Ok(());
        };
        tokio::task::spawn_blocking(move || {
            vm.attach_current_thread(|env| {
                _ = env
                    .call_method(&**statement, jni_str!("cancel"), jni_sig!("()V"), &[])
                    .map_err(jni_cleanup_error)?;
                Ok::<_, JdbcAdapterError>(())
            })
        })
        .await
        .map_err(JdbcAdapterError::Worker)??;
        Ok(())
    }
}

#[async_trait(?Send)]
impl DriverAdapter for JdbcAdapter {
    type Error = JdbcAdapterError;
    type Cancellation = JdbcCancellation;

    fn system(&self) -> DatabaseSystem {
        DatabaseSystem::Oracle
    }

    fn begin_operation(&mut self) -> Result<Self::Cancellation, Self::Error> {
        let mut state = self
            .cancellation
            .state
            .lock()
            .map_err(|_| JdbcAdapterError::CancellationState)?;
        state.requested = false;
        state.vm = None;
        state.statement = None;
        drop(state);
        Ok(self.cancellation.clone())
    }

    async fn validate_query(
        &mut self,
        query: &CompiledQuery,
    ) -> Result<Vec<ColumnMetadata>, Self::Error> {
        self.run(query, &query.watermark().initial, validate_blocking)
            .await
    }

    async fn execute(
        &mut self,
        query: &CompiledQuery,
        cursor: &CompositeCursor,
    ) -> Result<QueryPage, Self::Error> {
        self.run(query, cursor, execute_blocking).await
    }

    fn is_retryable(error: &Self::Error) -> bool {
        matches!(
            error,
            JdbcAdapterError::Retryable | JdbcAdapterError::Timeout
        )
    }

    async fn reconnect(&mut self, _query: &CompiledQuery) -> Result<(), Self::Error> {
        // JDBC connections are opened and closed within each operation. Dropping
        // this handle forces the next validation to reacquire the shared JVM.
        self.session = None;
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), Self::Error> {
        self.session = None;
        Ok(())
    }

    fn classify_error(error: &Self::Error) -> ReceiverErrorKind {
        match error {
            JdbcAdapterError::Jvm(_)
            | JdbcAdapterError::Credential(_)
            | JdbcAdapterError::CredentialNotRegularFile
            | JdbcAdapterError::EmptyCredential
            | JdbcAdapterError::InvalidCredentialEncoding => ReceiverErrorKind::Configuration,
            JdbcAdapterError::Jni(_) => ReceiverErrorKind::Other,
            JdbcAdapterError::Worker(_) => ReceiverErrorKind::Other,
            JdbcAdapterError::Cancelled => ReceiverErrorKind::Shutdown,
            JdbcAdapterError::CancellationState => ReceiverErrorKind::Other,
            JdbcAdapterError::Timeout | JdbcAdapterError::Retryable => ReceiverErrorKind::Transport,
            JdbcAdapterError::Jdbc(_)
            | JdbcAdapterError::Conversion(_)
            | JdbcAdapterError::NormalizedByteLimit => ReceiverErrorKind::Transport,
        }
    }
}

fn open_jvm(config: &JdbcAdapterConfig) -> Result<JdbcSession, JdbcAdapterError> {
    let classpath = normalize_jvm_path(
        Path::new(&config.driver_jar)
            .canonicalize()
            .map_err(|error| JdbcAdapterError::Jvm(error.to_string()))?,
    );
    if !classpath.is_file() {
        return Err(JdbcAdapterError::Jvm(
            "jdbc.driver_jar must reference a regular file".to_owned(),
        ));
    }
    let java_home = normalize_jvm_path(
        Path::new(&config.java_home)
            .canonicalize()
            .map_err(|error| JdbcAdapterError::Jvm(error.to_string()))?,
    );
    if !java_home.is_dir() {
        return Err(JdbcAdapterError::Jvm(
            "jdbc.java_home must reference a directory".to_owned(),
        ));
    }
    let configured = JVM.get_or_init(|| {
        let jvm_library = jvm_library_path(&java_home)?;
        let classpath_option = format!("-Djava.class.path={}", classpath.display());
        let java_home_option = format!("-Djava.home={}", java_home.display());
        let application_home_option = format!("-Dapplication.home={}", java_home.display());
        let args = InitArgsBuilder::new()
            .version(JNIVersion::V1_8)
            .option(&classpath_option)
            .option(&java_home_option)
            .option(&application_home_option)
            .build()
            .map_err(|error| error.to_string())?;
        let vm =
            JavaVM::with_libjvm(args, || Ok(jvm_library)).map_err(|error| error.to_string())?;
        Ok(JvmState {
            java_home: java_home.clone(),
            driver_jar: classpath.clone(),
            vm: Arc::new(vm),
        })
    });
    let state = configured
        .as_ref()
        .map_err(|error| JdbcAdapterError::Jvm(error.clone()))?;
    if state.java_home != java_home || state.driver_jar != classpath {
        return Err(JdbcAdapterError::Jvm(
            "the process JVM is already initialized with a different Java home or JDBC driver JAR"
                .to_owned(),
        ));
    }
    Ok(JdbcSession {
        vm: Arc::clone(&state.vm),
    })
}

#[cfg(target_os = "windows")]
fn jvm_library_path(java_home: &Path) -> Result<PathBuf, String> {
    jvm_library_from_candidates([
        java_home.join("bin").join("server").join("jvm.dll"),
        java_home
            .join("jre")
            .join("bin")
            .join("server")
            .join("jvm.dll"),
    ])
}

#[cfg(target_os = "linux")]
fn jvm_library_path(java_home: &Path) -> Result<PathBuf, String> {
    jvm_library_from_candidates([java_home.join("lib").join("server").join("libjvm.so")])
}

#[cfg(target_os = "macos")]
fn jvm_library_path(java_home: &Path) -> Result<PathBuf, String> {
    jvm_library_from_candidates([java_home.join("lib").join("server").join("libjvm.dylib")])
}

fn jvm_library_from_candidates<const N: usize>(
    candidates: [PathBuf; N],
) -> Result<PathBuf, String> {
    candidates
        .into_iter()
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| "could not find the JVM shared library under jdbc.java_home".to_owned())
}

#[cfg(windows)]
fn normalize_jvm_path(path: PathBuf) -> PathBuf {
    let path_string = path.to_string_lossy();
    if let Some(unc_path) = path_string.strip_prefix(r"\\?\UNC\") {
        PathBuf::from(format!(r"\\{unc_path}"))
    } else if let Some(path) = path_string.strip_prefix(r"\\?\") {
        PathBuf::from(path)
    } else {
        path
    }
}

#[cfg(not(windows))]
fn normalize_jvm_path(path: PathBuf) -> PathBuf {
    path
}

fn validate_blocking(
    session: &JdbcSession,
    config: &JdbcAdapterConfig,
    query: &CompiledQuery,
    cursor: &CompositeCursor,
    cancellation: &JdbcCancellation,
) -> Result<Vec<ColumnMetadata>, JdbcAdapterError> {
    Ok(session
        .vm
        .attach_current_thread(|mut env| -> Result<_, JdbcAdapterError> {
            let (connection, statement, result_set, active) =
                execute_statement(&mut env, session, config, query, cursor, cancellation)?;
            let result = result_metadata(&mut env, &result_set, query).map(|(columns, _)| columns);
            let cleanup = cleanup_jdbc_connection(&mut env, result_set, statement, connection);
            drop(active);
            match result {
                Err(error) => Err(error),
                Ok(metadata) => cleanup.map(|()| metadata),
            }
        })?)
}

fn execute_blocking(
    session: &JdbcSession,
    config: &JdbcAdapterConfig,
    query: &CompiledQuery,
    cursor: &CompositeCursor,
    cancellation: &JdbcCancellation,
) -> Result<QueryPage, JdbcAdapterError> {
    Ok(session
        .vm
        .attach_current_thread(|mut env| -> Result<_, JdbcAdapterError> {
            let (connection, statement, result_set, active) =
                execute_statement(&mut env, session, config, query, cursor, cancellation)?;
            let result = (|| {
                let (columns, scales) = result_metadata(&mut env, &result_set, query)?;
                let timestamp_index = find_column(&columns, &query.watermark().timestamp_column)?;
                let tie_breaker_index =
                    find_column(&columns, &query.watermark().tie_breaker_column)?;
                validate_cursor_types(&columns, &scales, timestamp_index, tie_breaker_index)?;
                let mut rows = Vec::new();
                let mut normalized_bytes = 0_u64;
                while call_bool(&mut env, &result_set, "next", "()Z", &[])? {
                    let timestamp = get_string(&mut env, &result_set, timestamp_index)?
                        .ok_or_else(|| {
                            JdbcAdapterError::Conversion(format!(
                                "cursor column '{}' returned NULL",
                                query.watermark().timestamp_column
                            ))
                        })?;
                    let tie_breaker_text = get_string(&mut env, &result_set, tie_breaker_index)?
                        .ok_or_else(|| {
                            JdbcAdapterError::Conversion(format!(
                                "cursor column '{}' returned NULL",
                                query.watermark().tie_breaker_column
                            ))
                        })?;
                    let tie_breaker = tie_breaker_text.parse::<i64>().map_err(|_| {
                        JdbcAdapterError::Conversion(format!(
                            "cursor column '{}' is not representable as int64",
                            query.watermark().tie_breaker_column
                        ))
                    })?;
                    let mut values = Vec::with_capacity(columns.len());
                    for index in 0..columns.len() {
                        values.push(get_cell(
                            &mut env,
                            &result_set,
                            index,
                            &columns[index].source_type,
                        )?);
                    }
                    let row = Row { values };
                    let row_bytes = row.normalized_size();
                    let next_bytes = normalized_bytes.saturating_add(row_bytes);
                    if next_bytes > query.max_normalized_bytes() {
                        if rows.is_empty() {
                            return Err(JdbcAdapterError::NormalizedByteLimit);
                        }
                        break;
                    }
                    normalized_bytes = next_bytes;
                    rows.push(CursorRow {
                        row,
                        cursor: CompositeCursor::new(timestamp, tie_breaker),
                    });
                    if rows.len() >= query.max_rows() {
                        break;
                    }
                }
                Ok(QueryPage { columns, rows })
            })();
            let cleanup = cleanup_jdbc_connection(&mut env, result_set, statement, connection);
            drop(active);
            match result {
                Err(error) => Err(error),
                Ok(page) => cleanup.map(|()| page),
            }
        })?)
}

fn execute_statement<'a>(
    env: &mut Env<'a>,
    session: &JdbcSession,
    config: &JdbcAdapterConfig,
    query: &CompiledQuery,
    cursor: &CompositeCursor,
    cancellation: &JdbcCancellation,
) -> Result<(JObject<'a>, JObject<'a>, JObject<'a>, ActiveStatement), JdbcAdapterError> {
    let username = read_credential(&config.username_file)?;
    let password = read_credential(&config.password_file)?;
    let driver_manager = env
        .find_class(jni_str!("java/sql/DriverManager"))
        .map_err(jni_error)?;
    let url = env.new_string(&config.url).map_err(jni_error)?;
    let properties_class = env
        .find_class(jni_str!("java/util/Properties"))
        .map_err(jni_error)?;
    let properties = env
        .new_object(&properties_class, jni_sig!("()V"), &[])
        .map_err(jni_error)?;
    let connect_timeout = query
        .timeout()
        .min(Duration::from_secs(10))
        .as_millis()
        .to_string();
    let read_timeout = query.timeout().as_millis().to_string();
    for (key, value) in [
        ("user", username.expose_secret()),
        ("password", password.expose_secret()),
        ("oracle.net.CONNECT_TIMEOUT", connect_timeout.as_str()),
        ("oracle.jdbc.ReadTimeout", read_timeout.as_str()),
    ] {
        let key = env.new_string(key).map_err(jni_error)?;
        let value = env.new_string(value).map_err(jni_error)?;
        _ = env
            .call_method(
                &properties,
                jni_str!("setProperty"),
                jni_sig!("(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/Object;"),
                &[JValue::Object(&key), JValue::Object(&value)],
            )
            .map_err(jni_error)?;
    }
    let connection = env
        .call_static_method(
            driver_manager,
            jni_str!("getConnection"),
            jni_sig!("(Ljava/lang/String;Ljava/util/Properties;)Ljava/sql/Connection;"),
            &[JValue::Object(&url), JValue::Object(&properties)],
        )
        .map_err(jni_error)?
        .l()
        .map_err(jni_error)?;
    match execute_connection_query(env, session, &connection, query, cursor, cancellation) {
        Ok((statement, result_set, active)) => Ok((connection, statement, result_set, active)),
        Err(error) => match close_connection(env, connection) {
            Ok(()) => Err(error),
            Err(cleanup_error) => Err(JdbcAdapterError::Jdbc(format!(
                "{error}; JDBC connection cleanup also failed: {cleanup_error}"
            ))),
        },
    }
}

fn execute_connection_query<'a>(
    env: &mut Env<'a>,
    session: &JdbcSession,
    connection: &JObject<'a>,
    query: &CompiledQuery,
    cursor: &CompositeCursor,
    cancellation: &JdbcCancellation,
) -> Result<(JObject<'a>, JObject<'a>, ActiveStatement), JdbcAdapterError> {
    execute_session_statement(env, connection, "ALTER SESSION SET TIME_ZONE = 'UTC'")?;
    _ = env
        .call_method(
            connection,
            jni_str!("setAutoCommit"),
            jni_sig!("(Z)V"),
            &[JValue::Bool(false)],
        )
        .map_err(jni_error)?;
    _ = env
        .call_method(
            connection,
            jni_str!("setReadOnly"),
            jni_sig!("(Z)V"),
            &[JValue::Bool(true)],
        )
        .map_err(jni_error)?;
    execute_session_statement(env, connection, "SET TRANSACTION READ ONLY")?;

    let (sql_text, binds) = jdbc_sql(query.sql(), query)?;
    let sql = env.new_string(&sql_text).map_err(jni_error)?;
    let statement = env
        .call_method(
            connection,
            jni_str!("prepareStatement"),
            jni_sig!("(Ljava/lang/String;)Ljava/sql/PreparedStatement;"),
            &[JValue::Object(&sql)],
        )
        .map_err(jni_error)?
        .l()
        .map_err(jni_error)?;
    let result = (|| {
        let fetch_size = i32::try_from(query.fetch_size_rows()).map_err(|_| {
            JdbcAdapterError::Conversion("fetch size exceeds JDBC limits".to_owned())
        })?;
        _ = env
            .call_method(
                &statement,
                jni_str!("setFetchSize"),
                jni_sig!("(I)V"),
                &[JValue::Int(fetch_size)],
            )
            .map_err(jni_error)?;
        let timeout_seconds = query
            .timeout()
            .as_secs()
            .saturating_add(u64::from(query.timeout().subsec_nanos() > 0));
        let timeout_seconds = i32::try_from(timeout_seconds).map_err(|_| {
            JdbcAdapterError::Conversion("query timeout exceeds JDBC limits".to_owned())
        })?;
        _ = env
            .call_method(
                &statement,
                jni_str!("setQueryTimeout"),
                jni_sig!("(I)V"),
                &[JValue::Int(timeout_seconds)],
            )
            .map_err(jni_error)?;
        let global_statement = env.new_global_ref(&statement).map_err(jni_error)?;
        {
            let mut state = cancellation
                .state
                .lock()
                .map_err(|_| JdbcAdapterError::CancellationState)?;
            if state.requested {
                return Err(JdbcAdapterError::Cancelled);
            }
            state.vm = Some(Arc::clone(&session.vm));
            state.statement = Some(Arc::new(global_statement));
        }
        let active = ActiveStatement {
            cancellation: cancellation.clone(),
        };
        let mut parameter_index = 1_i32;
        for bind in binds {
            match bind {
                CursorBind::Timestamp => {
                    set_timestamp(env, &statement, parameter_index, &cursor.timestamp)?
                }
                CursorBind::TieBreaker => {
                    set_long(env, &statement, parameter_index, cursor.tie_breaker)?
                }
            }
            parameter_index = parameter_index.saturating_add(1);
        }
        let result_set = env
            .call_method(
                &statement,
                jni_str!("executeQuery"),
                jni_sig!("()Ljava/sql/ResultSet;"),
                &[],
            )
            .map_err(jni_error)?
            .l()
            .map_err(jni_error)?;
        Ok::<_, JdbcAdapterError>((result_set, active))
    })();
    match result {
        Ok((result_set, active)) => Ok((statement, result_set, active)),
        Err(error) => match close_statement(env, statement) {
            Ok(()) => Err(error),
            Err(cleanup_error) => Err(JdbcAdapterError::Jdbc(format!(
                "{error}; JDBC statement cleanup also failed: {cleanup_error}"
            ))),
        },
    }
}

fn execute_session_statement(
    env: &mut Env<'_>,
    connection: &JObject<'_>,
    sql: &str,
) -> Result<(), JdbcAdapterError> {
    let sql = env.new_string(sql).map_err(jni_error)?;
    let statement = env
        .call_method(
            connection,
            jni_str!("createStatement"),
            jni_sig!("()Ljava/sql/Statement;"),
            &[],
        )
        .map_err(jni_error)?
        .l()
        .map_err(jni_error)?;
    let result = env
        .call_method(
            &statement,
            jni_str!("execute"),
            jni_sig!("(Ljava/lang/String;)Z"),
            &[JValue::Object(&sql)],
        )
        .map(|_| ())
        .map_err(jni_error);
    let cleanup = close_statement(env, statement);
    match result {
        Err(error) => Err(error),
        Ok(()) => cleanup,
    }
}

fn result_metadata(
    env: &mut Env<'_>,
    result_set: &JObject<'_>,
    query: &CompiledQuery,
) -> Result<(Vec<ColumnMetadata>, Vec<i32>), JdbcAdapterError> {
    let metadata = env
        .call_method(
            result_set,
            jni_str!("getMetaData"),
            jni_sig!("()Ljava/sql/ResultSetMetaData;"),
            &[],
        )
        .map_err(jni_error)?
        .l()
        .map_err(jni_error)?;
    let count = call_int(env, &metadata, "getColumnCount", "()I", &[])?;
    let mut columns = Vec::with_capacity(count as usize);
    let mut scales = Vec::with_capacity(count as usize);
    for index in 1..=count {
        let name = call_string(
            env,
            &metadata,
            "getColumnLabel",
            "(I)Ljava/lang/String;",
            &[JValue::Int(index)],
        )?;
        let source_type = call_string(
            env,
            &metadata,
            "getColumnTypeName",
            "(I)Ljava/lang/String;",
            &[JValue::Int(index)],
        )?;
        let scale = env
            .call_method(
                &metadata,
                jni_str!("getScale"),
                jni_sig!("(I)I"),
                &[JValue::Int(index)],
            )
            .map_err(jni_error)?
            .i()
            .map_err(jni_error)?;
        let nullable = env
            .call_method(
                &metadata,
                jni_str!("isNullable"),
                jni_sig!("(I)I"),
                &[JValue::Int(index)],
            )
            .map_err(jni_error)?
            .i()
            .map_err(jni_error)?;
        columns.push(ColumnMetadata {
            name,
            source_type,
            nullable: nullable != 0,
        });
        scales.push(scale);
    }
    validate_cursor_types(
        &columns,
        &scales,
        find_column(&columns, &query.watermark().timestamp_column)?,
        find_column(&columns, &query.watermark().tie_breaker_column)?,
    )?;
    validate_result_types(&columns)?;
    Ok((columns, scales))
}

fn validate_cursor_types(
    columns: &[ColumnMetadata],
    scales: &[i32],
    timestamp_index: usize,
    tie_breaker_index: usize,
) -> Result<(), JdbcAdapterError> {
    let timestamp_type = columns[timestamp_index].source_type.to_ascii_uppercase();
    if timestamp_type != "DATE" && !timestamp_type.starts_with("TIMESTAMP") {
        return Err(JdbcAdapterError::Conversion(format!(
            "timestamp cursor has unsupported JDBC type '{}'",
            columns[timestamp_index].source_type
        )));
    }
    let tie_breaker_type = columns[tie_breaker_index].source_type.to_ascii_uppercase();
    if !(tie_breaker_type.contains("NUMBER")
        || matches!(
            tie_breaker_type.as_str(),
            "INTEGER" | "INT" | "BIGINT" | "SMALLINT"
        ))
        || scales[tie_breaker_index] != 0
    {
        return Err(JdbcAdapterError::Conversion(format!(
            "tie-breaker cursor must use an integral JDBC type, got '{}'",
            columns[tie_breaker_index].source_type
        )));
    }
    Ok(())
}

fn validate_result_types(columns: &[ColumnMetadata]) -> Result<(), JdbcAdapterError> {
    for column in columns {
        let source_type = column.source_type.to_ascii_uppercase();
        if !(source_type.contains("NUMBER")
            || source_type.contains("DECIMAL")
            || source_type == "FLOAT"
            || source_type.contains("BINARY_FLOAT")
            || source_type.contains("BINARY_DOUBLE")
            || source_type.starts_with("TIMESTAMP")
            || source_type == "DATE"
            || source_type.starts_with("INTERVAL")
            || source_type.contains("CHAR")
            || source_type.contains("VARCHAR")
            || source_type == "ROWID"
            || source_type == "RAW"
            || matches!(
                source_type.as_str(),
                "BOOLEAN" | "INTEGER" | "INT" | "BIGINT" | "SMALLINT"
            ))
        {
            return Err(JdbcAdapterError::Conversion(format!(
                "unsupported Oracle JDBC result type '{source_type}'"
            )));
        }
    }
    Ok(())
}

fn get_cell(
    env: &mut Env<'_>,
    result_set: &JObject<'_>,
    index: usize,
    source_type: &str,
) -> Result<CellValue, JdbcAdapterError> {
    let Some(value) = get_string(env, result_set, index)? else {
        return Ok(CellValue::Null);
    };
    let upper = source_type.to_ascii_uppercase();
    if upper.contains("NUMBER") || upper.contains("DECIMAL") || upper == "FLOAT" {
        return Ok(CellValue::Decimal(value));
    }
    if matches!(upper.as_str(), "INTEGER" | "INT" | "BIGINT" | "SMALLINT") {
        return value.parse::<i64>().map(CellValue::Int64).map_err(|_| {
            JdbcAdapterError::Conversion(format!("invalid integer in {source_type}"))
        });
    }
    if upper == "BOOLEAN" {
        return match value.to_ascii_lowercase().as_str() {
            "true" | "1" => Ok(CellValue::Bool(true)),
            "false" | "0" => Ok(CellValue::Bool(false)),
            _ => Err(JdbcAdapterError::Conversion(format!(
                "invalid boolean in {source_type}"
            ))),
        };
    }
    if upper.contains("BINARY_FLOAT") || upper.contains("BINARY_DOUBLE") {
        let value = value.parse::<f64>().map_err(|_| {
            JdbcAdapterError::Conversion(format!("invalid floating-point value in {source_type}"))
        })?;
        if !value.is_finite() {
            return Err(JdbcAdapterError::Conversion(
                "Oracle returned a non-finite floating-point value".to_owned(),
            ));
        }
        return Ok(CellValue::Float64(value));
    }
    if upper.starts_with("TIMESTAMP") || upper == "DATE" {
        return Ok(CellValue::Timestamp(value));
    }
    if upper.starts_with("INTERVAL") {
        return Ok(CellValue::Interval(value));
    }
    if upper.contains("CHAR") || upper.contains("VARCHAR") || upper == "ROWID" || upper == "RAW" {
        return Ok(CellValue::String(value));
    }
    Err(JdbcAdapterError::Conversion(format!(
        "unsupported Oracle JDBC result type '{source_type}'"
    )))
}

fn find_column(columns: &[ColumnMetadata], name: &str) -> Result<usize, JdbcAdapterError> {
    columns
        .iter()
        .position(|column| column.name.eq_ignore_ascii_case(name))
        .ok_or_else(|| JdbcAdapterError::Conversion(format!("missing column {name}")))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CursorBind {
    Timestamp,
    TieBreaker,
}

fn jdbc_sql(
    sql: &str,
    query: &CompiledQuery,
) -> Result<(String, Vec<CursorBind>), JdbcAdapterError> {
    let chars = sql.chars().collect::<Vec<_>>();
    let watermark = query.watermark();
    let mut jdbc = String::with_capacity(sql.len());
    let mut binds = Vec::new();
    let mut index = 0;
    while index < chars.len() {
        let quote = chars[index];
        if quote == '\'' || quote == '"' {
            jdbc.push(quote);
            index += 1;
            while index < chars.len() {
                jdbc.push(chars[index]);
                if chars[index] == quote {
                    if chars.get(index + 1) == Some(&quote) {
                        jdbc.push(quote);
                        index += 2;
                        continue;
                    }
                    index += 1;
                    break;
                }
                index += 1;
            }
            continue;
        }
        if quote == ':' {
            let start = index + 1;
            let mut end = start;
            while chars
                .get(end)
                .is_some_and(|character| character.is_ascii_alphanumeric() || *character == '_')
            {
                end += 1;
            }
            let name = chars[start..end].iter().collect::<String>();
            let bind = if name.eq_ignore_ascii_case(&watermark.timestamp_bind) {
                Some(CursorBind::Timestamp)
            } else if name.eq_ignore_ascii_case(&watermark.tie_breaker_bind) {
                Some(CursorBind::TieBreaker)
            } else {
                None
            };
            if let Some(bind) = bind {
                jdbc.push('?');
                binds.push(bind);
                index = end;
                continue;
            }
        }
        jdbc.push(quote);
        index += 1;
    }
    if !binds.contains(&CursorBind::Timestamp) || !binds.contains(&CursorBind::TieBreaker) {
        return Err(JdbcAdapterError::Conversion(
            "validated query is missing a JDBC cursor bind".to_owned(),
        ));
    }
    Ok((jdbc, binds))
}

fn timestamp_epoch(text: &str) -> Result<(i64, u32), JdbcAdapterError> {
    if let Ok(timestamp) = DateTime::parse_from_rfc3339(text) {
        let timestamp = timestamp.with_timezone(&Utc);
        return Ok((timestamp.timestamp(), timestamp.timestamp_subsec_nanos()));
    }
    if let Ok(timestamp) = DateTime::parse_from_str(text, "%Y-%m-%d %H:%M:%S%.f %:z") {
        let timestamp = timestamp.with_timezone(&Utc);
        return Ok((timestamp.timestamp(), timestamp.timestamp_subsec_nanos()));
    }
    let timestamp = NaiveDateTime::parse_from_str(text, "%Y-%m-%dT%H:%M:%S%.f")
        .or_else(|_| NaiveDateTime::parse_from_str(text, "%Y-%m-%d %H:%M:%S%.f"))
        .map_err(|_| {
            JdbcAdapterError::Conversion(
                "watermark timestamp is not a supported ISO-8601 value".to_owned(),
            )
        })?
        .and_utc();
    Ok((timestamp.timestamp(), timestamp.timestamp_subsec_nanos()))
}

fn set_timestamp(
    env: &mut Env<'_>,
    statement: &JObject<'_>,
    index: i32,
    value: &str,
) -> Result<(), JdbcAdapterError> {
    let (seconds, nanos) = timestamp_epoch(value)?;
    let instant_class = env
        .find_class(jni_str!("java/time/Instant"))
        .map_err(jni_error)?;
    let instant = env
        .call_static_method(
            &instant_class,
            jni_str!("ofEpochSecond"),
            jni_sig!("(JJ)Ljava/time/Instant;"),
            &[JValue::Long(seconds), JValue::Long(i64::from(nanos))],
        )
        .map_err(jni_error)?
        .l()
        .map_err(jni_error)?;
    let timestamp_class = env
        .find_class(jni_str!("java/sql/Timestamp"))
        .map_err(jni_error)?;
    let timestamp = env
        .call_static_method(
            &timestamp_class,
            jni_str!("from"),
            jni_sig!("(Ljava/time/Instant;)Ljava/sql/Timestamp;"),
            &[JValue::Object(&instant)],
        )
        .map_err(jni_error)?
        .l()
        .map_err(jni_error)?;
    let timezone_class = env
        .find_class(jni_str!("java/util/TimeZone"))
        .map_err(jni_error)?;
    let timezone_name = env.new_string("UTC").map_err(jni_error)?;
    let timezone = env
        .call_static_method(
            &timezone_class,
            jni_str!("getTimeZone"),
            jni_sig!("(Ljava/lang/String;)Ljava/util/TimeZone;"),
            &[JValue::Object(&timezone_name)],
        )
        .map_err(jni_error)?
        .l()
        .map_err(jni_error)?;
    let calendar_class = env
        .find_class(jni_str!("java/util/Calendar"))
        .map_err(jni_error)?;
    let calendar = env
        .call_static_method(
            &calendar_class,
            jni_str!("getInstance"),
            jni_sig!("(Ljava/util/TimeZone;)Ljava/util/Calendar;"),
            &[JValue::Object(&timezone)],
        )
        .map_err(jni_error)?
        .l()
        .map_err(jni_error)?;
    _ = env
        .call_method(
            statement,
            jni_str!("setTimestamp"),
            jni_sig!("(ILjava/sql/Timestamp;Ljava/util/Calendar;)V"),
            &[
                JValue::Int(index),
                JValue::Object(&timestamp),
                JValue::Object(&calendar),
            ],
        )
        .map_err(jni_error)?;
    Ok(())
}

fn read_credential(path: &str) -> Result<SecretString, JdbcAdapterError> {
    let metadata = std::fs::metadata(path).map_err(JdbcAdapterError::Credential)?;
    if !metadata.is_file() {
        return Err(JdbcAdapterError::CredentialNotRegularFile);
    }
    if metadata.len() > MAX_CREDENTIAL_BYTES {
        return Err(JdbcAdapterError::Credential(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "credential file is too large",
        )));
    }
    let mut bytes = Vec::new();
    _ = std::fs::File::open(path)
        .map_err(JdbcAdapterError::Credential)?
        .take(MAX_CREDENTIAL_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(JdbcAdapterError::Credential)?;
    let mut value =
        String::from_utf8(bytes).map_err(|_| JdbcAdapterError::InvalidCredentialEncoding)?;
    while value.ends_with(['\r', '\n']) {
        _ = value.pop();
    }
    if value.is_empty() {
        return Err(JdbcAdapterError::EmptyCredential);
    }
    Ok(value.into())
}

fn jni_error(error: jni::errors::Error) -> JdbcAdapterError {
    match error {
        jni::errors::Error::CaughtJavaException { name, .. }
            if name == "java.sql.SQLRecoverableException"
                || name.starts_with("java.sql.SQLTransient") =>
        {
            JdbcAdapterError::Retryable
        }
        error => JdbcAdapterError::Jdbc(error.to_string()),
    }
}

fn jni_cleanup_error(error: jni::errors::Error) -> JdbcAdapterError {
    match jni_error(error) {
        JdbcAdapterError::Retryable => {
            JdbcAdapterError::Jdbc("JDBC cancellation or resource cleanup failed".to_owned())
        }
        error => error,
    }
}

fn call_string(
    env: &mut Env<'_>,
    object: &JObject<'_>,
    name: &str,
    signature: &str,
    args: &[JValue<'_>],
) -> Result<String, JdbcAdapterError> {
    let value = match (name, signature) {
        ("getColumnLabel", "(I)Ljava/lang/String;") => env.call_method(
            object,
            jni_str!("getColumnLabel"),
            jni_sig!("(I)Ljava/lang/String;"),
            args,
        ),
        ("getColumnTypeName", "(I)Ljava/lang/String;") => env.call_method(
            object,
            jni_str!("getColumnTypeName"),
            jni_sig!("(I)Ljava/lang/String;"),
            args,
        ),
        _ => {
            return Err(JdbcAdapterError::Jdbc(format!(
                "unsupported JDBC string method {name}"
            )));
        }
    }
    .map_err(jni_error)?
    .l()
    .map_err(jni_error)?;
    if value.is_null() {
        return Err(JdbcAdapterError::Conversion(format!(
            "JDBC method {name} returned NULL"
        )));
    }
    let value = env.cast_local::<JString<'_>>(value).map_err(jni_error)?;
    value.try_to_string(env).map_err(jni_error)
}

fn get_string(
    env: &mut Env<'_>,
    object: &JObject<'_>,
    index: usize,
) -> Result<Option<String>, JdbcAdapterError> {
    let value = env
        .call_method(
            object,
            jni_str!("getString"),
            jni_sig!("(I)Ljava/lang/String;"),
            &[JValue::Int((index + 1) as i32)],
        )
        .map_err(jni_error)?
        .l()
        .map_err(jni_error)?;
    if value.is_null() {
        return Ok(None);
    }
    let value = env.cast_local::<JString<'_>>(value).map_err(jni_error)?;
    value.try_to_string(env).map(Some).map_err(jni_error)
}

fn call_bool(
    env: &mut Env<'_>,
    object: &JObject<'_>,
    name: &str,
    signature: &str,
    args: &[JValue<'_>],
) -> Result<bool, JdbcAdapterError> {
    let value = match (name, signature) {
        ("next", "()Z") => env.call_method(object, jni_str!("next"), jni_sig!("()Z"), args),
        _ => {
            return Err(JdbcAdapterError::Jdbc(format!(
                "unsupported JDBC boolean method {name}"
            )));
        }
    };
    value.map_err(jni_error)?.z().map_err(jni_error)
}

fn call_int(
    env: &mut Env<'_>,
    object: &JObject<'_>,
    name: &str,
    signature: &str,
    args: &[JValue<'_>],
) -> Result<i32, JdbcAdapterError> {
    let value = match (name, signature) {
        ("getColumnCount", "()I") => {
            env.call_method(object, jni_str!("getColumnCount"), jni_sig!("()I"), args)
        }
        _ => {
            return Err(JdbcAdapterError::Jdbc(format!(
                "unsupported JDBC integer method {name}"
            )));
        }
    };
    value.map_err(jni_error)?.i().map_err(jni_error)
}

fn set_long(
    env: &mut Env<'_>,
    statement: &JObject<'_>,
    index: i32,
    value: i64,
) -> Result<(), JdbcAdapterError> {
    _ = env
        .call_method(
            statement,
            jni_str!("setLong"),
            jni_sig!("(IJ)V"),
            &[JValue::Int(index), JValue::Long(value)],
        )
        .map_err(jni_error)?;
    Ok(())
}

fn close_jdbc_objects(
    env: &mut Env<'_>,
    result_set: JObject<'_>,
    statement: JObject<'_>,
) -> Result<(), JdbcAdapterError> {
    let result_set_close = env
        .call_method(&result_set, jni_str!("close"), jni_sig!("()V"), &[])
        .map(|_| ())
        .map_err(jni_cleanup_error);
    let statement_close = close_statement(env, statement);
    match (result_set_close, statement_close) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error),
        (Err(error), Err(cleanup_error)) => Err(JdbcAdapterError::Jdbc(format!(
            "{error}; JDBC statement cleanup also failed: {cleanup_error}"
        ))),
    }
}

fn cleanup_jdbc_connection(
    env: &mut Env<'_>,
    result_set: JObject<'_>,
    statement: JObject<'_>,
    connection: JObject<'_>,
) -> Result<(), JdbcAdapterError> {
    let objects = close_jdbc_objects(env, result_set, statement);
    let connection = close_connection(env, connection);
    match (objects, connection) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error),
        (Err(error), Err(cleanup_error)) => Err(JdbcAdapterError::Jdbc(format!(
            "{error}; JDBC connection cleanup also failed: {cleanup_error}"
        ))),
    }
}

fn close_statement(env: &mut Env<'_>, statement: JObject<'_>) -> Result<(), JdbcAdapterError> {
    _ = env
        .call_method(&statement, jni_str!("close"), jni_sig!("()V"), &[])
        .map_err(jni_cleanup_error)?;
    Ok(())
}

fn close_connection(env: &mut Env<'_>, connection: JObject<'_>) -> Result<(), JdbcAdapterError> {
    _ = env
        .call_method(&connection, jni_str!("close"), jni_sig!("()V"), &[])
        .map_err(jni_cleanup_error)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_scraper::database::{
        CheckpointConfig, OnNack, OnPermanentNack, OutputConfig, PollingConfig,
        TieBreakerCursorConfig, TimestampCursorConfig, WatermarkConfig,
    };

    fn query(sql: &str) -> CompiledQuery {
        let watermark = WatermarkConfig::Composite {
            timestamp: TimestampCursorConfig {
                column: "EVENT_TS".to_owned(),
                bind: "last_timestamp".to_owned(),
                initial: "1970-01-01 00:00:00".to_owned(),
                timezone: "UTC".to_owned(),
            },
            tie_breaker: TieBreakerCursorConfig {
                column: "EVENT_ID".to_owned(),
                bind: "last_tie_breaker".to_owned(),
                initial: 0,
            },
        };
        let checkpoint = CheckpointConfig {
            directory: "state".to_owned(),
            on_nack: OnNack::Rewind,
            on_permanent_nack: OnPermanentNack::default(),
            nack_backoff: Duration::from_secs(1),
            max_consecutive_failures: 3,
        };
        CompiledQuery::compile(
            sql.to_owned(),
            PollingConfig {
                interval: Duration::from_secs(1),
                timeout: Duration::from_secs(5),
                fetch_size_rows: 10,
                max_rows_per_poll: 20,
                max_batch_bytes: 1024,
                catch_up: Default::default(),
            },
            &watermark,
            &checkpoint,
            OutputConfig::default(),
        )
        .expect("query should compile")
    }

    /// Scenario: JDBC reports a recoverable connection failure or query timeout.
    /// Guarantees: Only explicitly transient failures enter the reconnect loop.
    #[test]
    fn retries_only_transient_jdbc_failures() {
        assert!(JdbcAdapter::is_retryable(&JdbcAdapterError::Retryable));
        assert!(JdbcAdapter::is_retryable(&JdbcAdapterError::Timeout));
        assert!(!JdbcAdapter::is_retryable(&JdbcAdapterError::Conversion(
            "invalid cursor".to_owned()
        )));
    }

    /// Scenario: Canonical Windows paths contain the extended-length path prefix.
    /// Guarantees: JVM options use conventional drive and UNC paths that OpenJDK can resolve.
    #[cfg(windows)]
    #[test]
    fn normalizes_windows_jvm_paths() {
        assert_eq!(
            normalize_jvm_path(PathBuf::from(r"\\?\C:\Java\jdk")),
            PathBuf::from(r"C:\Java\jdk")
        );
        assert_eq!(
            normalize_jvm_path(PathBuf::from(r"\\?\UNC\host\share\Java\jdk")),
            PathBuf::from(r"\\host\share\Java\jdk")
        );
    }

    /// Scenario: Cursor markers appear in different orders, repeat, or inside quoted SQL text.
    /// Guarantees: Only real markers become JDBC placeholders, and each value remains paired with
    /// the positional placeholder at which it appears.
    #[test]
    fn rewrites_only_real_cursor_binds_in_occurrence_order() {
        let query = query(
            "SELECT ':last_timestamp', \"T:last_tie_breaker\" FROM T \
             WHERE EVENT_ID > :last_tie_breaker AND EVENT_TS > :last_timestamp \
             AND EVENT_TS = :LAST_TIMESTAMP ORDER BY EVENT_TS ASC, EVENT_ID ASC",
        );

        let (sql, binds) = jdbc_sql(query.sql(), &query).expect("cursor binds should rewrite");

        assert_eq!(
            sql,
            "SELECT ':last_timestamp', \"T:last_tie_breaker\" FROM T \
             WHERE EVENT_ID > ? AND EVENT_TS > ? AND EVENT_TS = ? \
             ORDER BY EVENT_TS ASC, EVENT_ID ASC"
        );
        assert_eq!(
            binds,
            [
                CursorBind::TieBreaker,
                CursorBind::Timestamp,
                CursorBind::Timestamp
            ]
        );
    }

    /// Scenario: A timestamp cursor includes a timezone offset or UTC-naive text.
    /// Guarantees: JDBC timestamp binding maps both forms to the same UTC instant with nanosecond
    /// precision, independent of the host's default timezone.
    #[test]
    fn normalizes_timestamp_cursors_to_utc_without_losing_nanos() {
        let offset = timestamp_epoch("2026-01-01T12:00:00.123456789+05:30")
            .expect("offset timestamp should parse");
        let utc =
            timestamp_epoch("2026-01-01 06:30:00.123456789").expect("UTC timestamp should parse");

        assert_eq!(offset, utc);
        assert_eq!(offset.1, 123_456_789);
    }

    /// Scenario: Live JDBC metadata uses unsupported cursor types or a fractional tie-breaker.
    /// Guarantees: Invalid composite cursors fail before rows can advance the shared checkpoint.
    #[test]
    fn rejects_invalid_jdbc_cursor_metadata() {
        let columns = vec![
            ColumnMetadata {
                name: "EVENT_TS".to_owned(),
                source_type: "VARCHAR2".to_owned(),
                nullable: true,
            },
            ColumnMetadata {
                name: "EVENT_ID".to_owned(),
                source_type: "NUMBER".to_owned(),
                nullable: false,
            },
        ];

        assert!(validate_cursor_types(&columns, &[0, 0], 0, 1).is_err());

        let columns = vec![
            ColumnMetadata {
                name: "EVENT_TS".to_owned(),
                source_type: "TIMESTAMP".to_owned(),
                nullable: false,
            },
            ColumnMetadata {
                name: "EVENT_ID".to_owned(),
                source_type: "NUMBER".to_owned(),
                nullable: false,
            },
        ];
        assert!(validate_cursor_types(&columns, &[0, 2], 0, 1).is_err());
    }
}
