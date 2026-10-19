// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::{
    SqlServerAdapter, SqlServerConnectionConfig, SqlServerConnector, SqlServerError as Error,
    SqlServerReceiverConfig,
    data::{ColumnPlan, NativeType, ResultPlan, bind_scalar, convert},
};
use async_trait::async_trait;
use futures::io::{AsyncRead, AsyncWrite};
use otel_arrow_dfe_engine::{
    capability::{
        CapabilityError,
        auth::{
            BasicAuthCredential,
            basic_auth_provider::{BASIC_AUTH_CREDENTIAL_USABLE_MARGIN, BasicAuthCredentialStream},
        },
    },
    error::Error as EngineError,
    local::{capability::auth::basic_auth_provider::BasicAuthProvider, receiver as local},
    terminal_state::TerminalState,
};
use otel_arrow_dfe_otap::pdata::OtapPdata;
use otel_arrow_dfe_scraper::database::{
    CellValue, ColumnMetadata, CompiledQuery, Cursor, DatabaseSystem, DriverAdapter,
    DriverCancellation, ScalarValue,
};
use serde_json::{Value, json};
use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    io,
    pin::Pin,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    task::{Context, Poll},
    time::Duration,
};
use tiberius::{ColumnData, ColumnType, time};

fn config(snapshot: bool) -> SqlServerReceiverConfig {
    config_with_bytes(snapshot, 4096)
}

fn config_with_bytes(snapshot: bool, bytes: u64) -> SqlServerReceiverConfig {
    serde_json::from_value(raw_config(snapshot, bytes)).unwrap()
}

fn raw_config(snapshot: bool, bytes: u64) -> Value {
    json!({
        "source_id":"events",
        "connection":{"host":"localhost","database":"fixture"},
        "query":{
            "statement":if snapshot {"SELECT ID FROM dbo.events"} else {"SELECT ID FROM dbo.events WHERE ID > @key ORDER BY ID"},
            "max_rows_per_poll":2,"fetch_size_rows":1,"max_batch_bytes":bytes,"timeout":"1s"
        },
        "watermark": if snapshot {json!({"mode":"snapshot"})} else {json!({"mode":"scalar","column":"ID","bind":"key","initial":{"type":"int64","value":0}})},
        "checkpoint":{"directory":"state","on_nack":"rewind","nack_backoff":"1s","max_consecutive_failures":5}
    })
}

fn column(kind: NativeType, scale: u8, nullable: bool) -> ColumnPlan {
    ColumnPlan {
        metadata: ColumnMetadata {
            name: "ID".into(),
            source_type: "bigint".into(),
            nullable,
        },
        kind,
        precision: 38,
        scale,
        max_length: 8,
    }
}

/// Scenario: Decimal values span all 38 digits, including a fractional decimal(38,38).
/// Guarantees: Output retains every digit and scale without f64 or a narrower decimal conversion.
#[test]
fn decimal_conversion_is_exact_at_full_precision() {
    for (value, scale, text) in [
        (
            99_999_999_999_999_999_999_999_999_999_999_999_999i128,
            0,
            "99999999999999999999999999999999999999",
        ),
        (-1234500, 4, "-123.4500"),
        (1, 38, "0.00000000000000000000000000000000000001"),
        (0, 38, "0.00000000000000000000000000000000000000"),
    ] {
        let input = ColumnData::Numeric(Some(tiberius::numeric::Numeric::new_with_scale(
            value, scale,
        )));
        assert_eq!(
            convert(input, &column(NativeType::Decimal, scale, false)).unwrap(),
            CellValue::Decimal(text.into())
        );
    }
}

/// Scenario: The server returns unsupported metadata or money decoded through floating point.
/// Guarantees: Money, legacy temporal, XML, variant, and UDT values cannot silently lose semantics.
#[test]
fn unsupported_native_types_fail_closed() {
    for id in [60, 122, 61, 58, 241, 98, 240, 35, 99, 34] {
        assert_eq!(NativeType::from_system_id(id), Err(Error::Metadata));
    }
    assert!(!NativeType::Float.matches(ColumnType::Money));
    assert_eq!(
        convert(
            ColumnData::F64(Some(f64::NAN)),
            &column(NativeType::Float, 0, true)
        ),
        Err(Error::Conversion)
    );
    assert_eq!(
        convert(ColumnData::I64(None), &column(NativeType::BigInt, 0, false)),
        Err(Error::Conversion)
    );
    assert_eq!(
        convert(ColumnData::I64(None), &column(NativeType::BigInt, 0, true)),
        Ok(CellValue::Null)
    );
}

/// Scenario: A cursor needs a narrower native integer or timestamp precision.
/// Guarantees: The exact lower bound is bound or rejected, never rounded or narrowed.
#[test]
fn cursor_bindings_preserve_native_range_and_precision() {
    assert_eq!(
        bind_scalar(
            &column(NativeType::SmallInt, 0, false),
            &ScalarValue::Int64(32768)
        ),
        Err(Error::Cursor)
    );
    assert_eq!(
        bind_scalar(
            &column(NativeType::TinyInt, 0, false),
            &ScalarValue::Int64(1)
        ),
        Err(Error::Cursor)
    );
    assert_eq!(
        bind_scalar(
            &column(NativeType::Int, 0, false),
            &ScalarValue::Int64(i64::MAX)
        ),
        Err(Error::Cursor)
    );
    for scale in 0..=7 {
        let initial = ScalarValue::Timestamp("2026-01-01T00:00:00.0000000Z".into());
        let value = bind_scalar(&column(NativeType::DateTime2, scale, false), &initial).unwrap();
        let ColumnData::DateTime2(Some(value)) = value else {
            panic!("datetime2")
        };
        assert_eq!(value.time().scale(), scale);
    }
    let value = ScalarValue::Timestamp("2026-01-01T00:00:00.1234567Z".into());
    assert_eq!(
        bind_scalar(&column(NativeType::DateTime2, 6, false), &value),
        Err(Error::Cursor)
    );
}

/// Scenario: Wire temporal values carry invalid scales, out-of-day times, or invalid offsets.
/// Guarantees: Malformed temporal input errors rather than wrapping, saturating, or panicking.
#[test]
fn malformed_temporal_values_are_rejected() {
    for value in [
        time::Time::new(0, 8),
        time::Time::new(86400, 0),
        time::Time::new(u64::MAX, 7),
    ] {
        assert_eq!(
            convert(
                ColumnData::Time(Some(value)),
                &column(NativeType::Time, 7, true)
            ),
            Err(Error::Conversion)
        );
    }
    let invalid = time::DateTimeOffset::new(
        time::DateTime2::new(time::Date::new(1), time::Time::new(0, 7)),
        841,
    );
    assert_eq!(
        convert(
            ColumnData::DateTimeOffset(Some(invalid)),
            &column(NativeType::DateTimeOffset, 7, false)
        ),
        Err(Error::Conversion)
    );
}

/// Scenario: Real Tiberius rows have metadata consistent or inconsistent with the result plan.
/// Guarantees: Rows become shared values/cursors only under an unambiguous validated schema.
#[test]
fn row_conversion_checks_metadata_and_cursor() {
    let query = config(false).query().clone();
    let plan = ResultPlan::new(vec![column(NativeType::BigInt, 0, false)], &query).unwrap();
    let row = tiberius::Row::builder()
        .column("ID", ColumnType::Int8, ColumnData::I64(Some(42)))
        .build();
    let row = plan.row(row).unwrap();
    assert_eq!(row.row.values, vec![CellValue::Int64(42)]);
    assert_eq!(row.cursor, Cursor::Scalar(ScalarValue::Int64(42)));
    let wrong = tiberius::Row::builder()
        .column("id", ColumnType::Int8, ColumnData::I64(Some(42)))
        .build();
    assert_eq!(plan.row(wrong), Err(Error::Metadata));
    assert!(ResultPlan::new(vec![column(NativeType::BigInt, 0, true)], &query).is_err());
}

struct Credentials;
#[async_trait(?Send)]
impl BasicAuthProvider for Credentials {
    async fn get_credential(&self) -> Result<BasicAuthCredential, CapabilityError> {
        Ok(BasicAuthCredential::new("fixture-user", "fixture-password").unwrap())
    }
    fn credential_stream(&self) -> BasicAuthCredentialStream {
        Box::pin(futures::stream::empty())
    }
}

struct ScriptedCredentials {
    values: RefCell<VecDeque<BasicAuthCredential>>,
    delay: Option<Duration>,
    calls: Rc<Cell<usize>>,
}

impl ScriptedCredentials {
    fn new(values: impl IntoIterator<Item = BasicAuthCredential>) -> Self {
        Self {
            values: RefCell::new(values.into_iter().collect()),
            delay: Some(Duration::ZERO),
            calls: Rc::default(),
        }
    }
}

#[async_trait(?Send)]
impl BasicAuthProvider for ScriptedCredentials {
    async fn get_credential(&self) -> Result<BasicAuthCredential, CapabilityError> {
        self.calls.set(self.calls.get() + 1);
        let Some(delay) = self.delay else {
            return futures::future::pending().await;
        };
        if !delay.is_zero() {
            tokio::time::sleep(delay).await;
        }
        Ok(self
            .values
            .borrow_mut()
            .pop_front()
            .expect("scripted credential"))
    }

    fn credential_stream(&self) -> BasicAuthCredentialStream {
        Box::pin(futures::stream::empty())
    }
}

#[derive(Default)]
struct PendingConnector {
    calls: Rc<Cell<usize>>,
}
#[async_trait(?Send)]
impl SqlServerConnector for PendingConnector {
    type Stream = ScriptedStream;
    async fn connect(
        &mut self,
        _: &SqlServerConnectionConfig,
        _: tiberius::Config,
    ) -> Result<tiberius::Client<Self::Stream>, Error> {
        self.calls.set(self.calls.get() + 1);
        futures::future::pending().await
    }
}

/// Scenario: Cancellation interrupts a connection, then an old handle fires during a new attempt.
/// Guarantees: Connection waits stop promptly and stale handles cannot cancel a fresh generation.
#[tokio::test]
async fn connection_cancellation_is_local_and_generation_scoped() {
    let config = config(false);
    let query = config.query().clone();
    let mut adapter =
        SqlServerAdapter::new(config, PendingConnector::default(), Box::new(Credentials));
    let stale = adapter.begin_operation().unwrap();
    let cancel = stale.clone();
    let (result, _) = tokio::join!(adapter.validate_query(&query), async {
        tokio::task::yield_now().await;
        cancel.cancel().await.unwrap();
    });
    assert_eq!(result, Err(Error::Cancelled));
    let current = adapter.begin_operation().unwrap();
    stale.cancel().await.unwrap();
    let operation =
        tokio::time::timeout(Duration::from_millis(20), adapter.validate_query(&query)).await;
    assert!(
        operation.is_err(),
        "old generation must not cancel the connector"
    );
    current.cancel().await.unwrap();
    adapter.shutdown().await.unwrap();
}

/// Scenario: A provider supplies expired credentials or credentials at/inside the shared usability margin.
/// Guarantees: Unusable credentials fail permanently before any connector or login work starts.
#[tokio::test]
async fn unusable_credentials_are_rejected_before_connecting() {
    let now = std::time::Instant::now();
    for expiry in [
        now,
        now + BASIC_AUTH_CREDENTIAL_USABLE_MARGIN / 2,
        now + BASIC_AUTH_CREDENTIAL_USABLE_MARGIN,
    ] {
        let config = config(false);
        let query = config.query().clone();
        let credentials = ScriptedCredentials::new([BasicAuthCredential::new(
            "expired-user",
            "expired-password",
        )
        .unwrap()
        .with_expiry(expiry)]);
        let acquisitions = Rc::clone(&credentials.calls);
        let connector = FixtureConnector::new([]);
        let calls = Arc::clone(&connector.calls);
        let mut adapter = SqlServerAdapter::new(config, connector, Box::new(credentials));
        _ = adapter.begin_operation().unwrap();
        assert_eq!(
            adapter.validate_query(&query).await,
            Err(Error::Credentials)
        );
        assert!(!SqlServerAdapter::<FixtureConnector>::is_retryable(
            &Error::Credentials
        ));
        assert_eq!(acquisitions.get(), 1);
        assert_eq!(calls.connections.load(Ordering::Relaxed), 0);
        assert_eq!(calls.requests.load(Ordering::Relaxed), 0);
        adapter.shutdown().await.unwrap();
    }
}

/// Scenario: A provider supplies a credential without expiry or comfortably outside the usability margin.
/// Guarantees: Valid credentials reach the driver's actual login unchanged.
#[tokio::test]
async fn usable_credentials_reach_the_login() {
    for expiring in [false, true] {
        let mut credential = BasicAuthCredential::new("usable-user", "usable-password").unwrap();
        if expiring {
            credential = credential
                .with_expiry(std::time::Instant::now() + BASIC_AUTH_CREDENTIAL_USABLE_MARGIN * 2);
        }
        let config = config(false);
        let query = config.query().clone();
        let mut connector = FixtureConnector::new([vec![metadata_response()]]);
        connector.expected_credentials.push_back(credential.clone());
        let calls = Arc::clone(&connector.calls);
        let credentials = ScriptedCredentials::new([credential]);
        let acquisitions = Rc::clone(&credentials.calls);
        let mut adapter = SqlServerAdapter::new(config, connector, Box::new(credentials));
        _ = adapter.begin_operation().unwrap();
        _ = adapter.validate_query(&query).await.unwrap();
        assert_eq!(acquisitions.get(), 1);
        assert_eq!(calls.connections.load(Ordering::Relaxed), 1);
        assert_eq!(calls.checked_logins.load(Ordering::Relaxed), 1);
        adapter.shutdown().await.unwrap();
    }
}

/// Scenario: Credential acquisition never completes and reaches the configured connect deadline.
/// Guarantees: The adapter returns its own deadline error without invoking the connector.
#[tokio::test(start_paused = true)]
async fn credential_acquisition_obeys_connect_deadline() {
    let mut raw = raw_config(false, 4096);
    raw["connection"]["connect_timeout"] = json!("1s");
    let config: SqlServerReceiverConfig = serde_json::from_value(raw).unwrap();
    let query = config.query().clone();
    let mut credentials = ScriptedCredentials::new([]);
    credentials.delay = None;
    let acquisitions = Rc::clone(&credentials.calls);
    let connector = FixtureConnector::new([]);
    let calls = Arc::clone(&connector.calls);
    let mut adapter = SqlServerAdapter::new(config, connector, Box::new(credentials));
    _ = adapter.begin_operation().unwrap();
    let started = tokio::time::Instant::now();
    let result = tokio::time::timeout(Duration::from_secs(2), adapter.validate_query(&query))
        .await
        .expect("adapter must enforce its own connect deadline");
    assert_eq!(result, Err(Error::Deadline));
    assert_eq!(started.elapsed(), Duration::from_secs(1));
    assert_eq!(acquisitions.get(), 1);
    assert_eq!(calls.connections.load(Ordering::Relaxed), 0);
    assert_eq!(calls.requests.load(Ordering::Relaxed), 0);
    adapter.shutdown().await.unwrap();
}

/// Scenario: Credential acquisition consumes most of the connect budget before the connector stalls.
/// Guarantees: Connection setup receives only the remaining budget, not a fresh connect timeout.
#[tokio::test(start_paused = true)]
async fn credentials_and_connector_share_one_deadline() {
    let mut raw = raw_config(false, 4096);
    raw["connection"]["connect_timeout"] = json!("1s");
    let config: SqlServerReceiverConfig = serde_json::from_value(raw).unwrap();
    let query = config.query().clone();
    let mut credentials =
        ScriptedCredentials::new([
            BasicAuthCredential::new("delayed-user", "delayed-password").unwrap()
        ]);
    credentials.delay = Some(Duration::from_millis(750));
    let acquisitions = Rc::clone(&credentials.calls);
    let connector = PendingConnector::default();
    let connections = Rc::clone(&connector.calls);
    let mut adapter = SqlServerAdapter::new(config, connector, Box::new(credentials));
    _ = adapter.begin_operation().unwrap();
    let started = tokio::time::Instant::now();
    let result = tokio::time::timeout(Duration::from_secs(2), adapter.validate_query(&query))
        .await
        .expect("credential acquisition must not restart the connect deadline");
    assert_eq!(result, Err(Error::Deadline));
    assert_eq!(started.elapsed(), Duration::from_secs(1));
    assert_eq!(acquisitions.get(), 1);
    assert_eq!(connections.get(), 1);
    adapter.shutdown().await.unwrap();
}

/// Scenario: Credentials rotate after the first connection and the shared lifecycle reconnects.
/// Guarantees: Existing-session polling does not reacquire credentials; the next login uses both new secrets.
#[tokio::test]
async fn reconnect_uses_newly_acquired_credentials() {
    let first = BasicAuthCredential::new("first-user", "first-password").unwrap();
    let second = BasicAuthCredential::new("second-user", "second-password").unwrap();
    let credentials = ScriptedCredentials::new([first.clone(), second.clone()]);
    let acquisitions = Rc::clone(&credentials.calls);
    let mut connector = FixtureConnector::new([
        vec![
            metadata_response(),
            metadata_response(),
            rows_response(&[1]),
        ],
        vec![
            metadata_response(),
            metadata_response(),
            rows_response(&[2]),
        ],
    ]);
    connector.expected_credentials = VecDeque::from([first, second]);
    let calls = Arc::clone(&connector.calls);
    let config = config(false);
    let query = config.query().clone();
    let mut adapter = SqlServerAdapter::new(config, connector, Box::new(credentials));
    _ = adapter.begin_operation().unwrap();
    _ = adapter.validate_query(&query).await.unwrap();
    _ = adapter.begin_operation().unwrap();
    let first_page = adapter
        .execute(&query, &query.watermark().initial())
        .await
        .unwrap();
    assert_eq!(acquisitions.get(), 1);
    assert_eq!(calls.checked_logins.load(Ordering::Relaxed), 1);
    _ = adapter.begin_operation().unwrap();
    adapter.reconnect(&query).await.unwrap();
    _ = adapter.begin_operation().unwrap();
    _ = adapter.validate_query(&query).await.unwrap();
    _ = adapter.begin_operation().unwrap();
    let second_page = adapter
        .execute(&query, &first_page.rows[0].cursor)
        .await
        .unwrap();
    assert_eq!(
        second_page.rows[0].cursor,
        Cursor::Scalar(ScalarValue::Int64(2))
    );
    assert_eq!(acquisitions.get(), 2);
    assert_eq!(calls.connections.load(Ordering::Relaxed), 2);
    assert_eq!(calls.checked_logins.load(Ordering::Relaxed), 2);
    adapter.shutdown().await.unwrap();
}

/// Scenario: Driver errors contain sensitive text or represent permanent SQL/authentication failures.
/// Guarantees: Diagnostics retain only safe codes/categories and retries remain transient-only.
#[test]
fn driver_errors_are_redacted_and_retries_are_allowlisted() {
    use std::error::Error as _;
    for error in [
        tiberius::error::Error::Tls("SENSITIVE".into()),
        tiberius::error::Error::Protocol("SENSITIVE".into()),
        tiberius::error::Error::Io {
            kind: io::ErrorKind::ConnectionReset,
            message: "SENSITIVE".into(),
        },
    ] {
        let error = Error::from(error);
        assert!(!format!("{error} {error:?}").contains("SENSITIVE"));
        assert!(error.source().is_none());
    }
    assert!(SqlServerAdapter::<PendingConnector>::is_retryable(
        &Error::Transport(io::ErrorKind::ConnectionReset)
    ));
    assert!(SqlServerAdapter::<PendingConnector>::is_retryable(
        &Error::QueryTimeout
    ));
    for error in [
        Error::Tls,
        Error::Credentials,
        Error::Metadata,
        Error::Deadline,
        Error::UnconfirmedTransport(io::ErrorKind::ConnectionReset),
        Error::Server(102),
        Error::Server(147),
        Error::Server(208),
        Error::Server(229),
        Error::Server(4145),
        Error::Server(18456),
    ] {
        assert!(!SqlServerAdapter::<PendingConnector>::is_retryable(&error));
    }
}

// Test-only shared counters use atomics because Tiberius requires a Send stream.
#[derive(Default)]
struct ProtocolCalls {
    connections: AtomicUsize,
    requests: AtomicUsize,
    checked_logins: AtomicUsize,
    resets: AtomicUsize,
}

#[derive(Clone, Copy, Debug)]
enum ResponseFault {
    SplitAttention,
    MissingAttention,
    LateAttention,
    DroppedCleanup,
    ResetDuringRows,
    ResetOnceDuringRows,
    ResetBeforeWrite,
    ResetAfterWrite,
    ResetDuringCleanup,
}

// A complete request releases one in-memory response, exercising the real driver's
// framing/decoding without opening sockets or transporting customer credentials.
struct ScriptedStream {
    responses: VecDeque<Vec<u8>>,
    readable: VecDeque<u8>,
    writing: Vec<u8>,
    calls: Arc<ProtocolCalls>,
    expected_login: Option<BasicAuthCredential>,
    fault: Option<ResponseFault>,
    delayed: Option<(Pin<Box<tokio::time::Sleep>>, Vec<u8>)>,
    reset: bool,
}

impl AsyncRead for ScriptedStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        output: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        if self.readable.is_empty() {
            if self.reset {
                if matches!(self.fault, Some(ResponseFault::ResetOnceDuringRows)) {
                    self.reset = false;
                }
                _ = self.calls.resets.fetch_add(1, Ordering::Relaxed);
                return Poll::Ready(Err(io::Error::from(io::ErrorKind::ConnectionReset)));
            }
            if let Some((delay, _)) = &mut self.delayed {
                if delay.as_mut().poll(cx).is_pending() {
                    return Poll::Pending;
                }
                let (_, bytes) = self.delayed.take().unwrap();
                self.readable.extend(bytes);
            }
        }
        if self.readable.is_empty() {
            return Poll::Pending;
        }
        let count = output.len().min(self.readable.len());
        for byte in &mut output[..count] {
            *byte = self.readable.pop_front().unwrap();
        }
        Poll::Ready(Ok(count))
    }
}

impl AsyncWrite for ScriptedStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        _: &mut Context<'_>,
        input: &[u8],
    ) -> Poll<io::Result<usize>> {
        if matches!(self.fault, Some(ResponseFault::ResetBeforeWrite))
            && self.calls.requests.load(Ordering::Relaxed) == 4
        {
            _ = self.calls.resets.fetch_add(1, Ordering::Relaxed);
            return Poll::Ready(Err(io::Error::from(io::ErrorKind::ConnectionReset)));
        }
        self.writing.extend_from_slice(input);
        loop {
            if self.writing.len() < 8 {
                break;
            }
            let length = usize::from(u16::from_be_bytes([self.writing[2], self.writing[3]]));
            if self.writing.len() < length {
                break;
            }
            let end = self.writing[1] & 1 != 0;
            if self.writing[0] == 0x10
                && let Some(expected) = self.expected_login.take()
            {
                assert!(end, "fixture LOGIN7 must fit one packet");
                let login = &self.writing[8..length];
                assert!(login_field(login, 40, false) == expected.expose_username());
                assert!(login_field(login, 44, true) == expected.expose_password());
                _ = self.calls.checked_logins.fetch_add(1, Ordering::Relaxed);
            }
            _ = self.writing.drain(..length);
            if end {
                let request = self.calls.requests.fetch_add(1, Ordering::Relaxed) + 1;
                if let Some(response) = self.responses.pop_front() {
                    if request == 6 && matches!(self.fault, Some(ResponseFault::LateAttention)) {
                        self.delayed = Some((
                            Box::pin(tokio::time::sleep(Duration::from_secs(2))),
                            response,
                        ));
                    } else {
                        self.readable.extend(response);
                    }
                    if request == 5
                        && matches!(
                            self.fault,
                            Some(
                                ResponseFault::ResetDuringRows
                                    | ResponseFault::ResetOnceDuringRows
                                    | ResponseFault::ResetAfterWrite
                            )
                        )
                    {
                        self.reset = true;
                    }
                    if request == 6 && matches!(self.fault, Some(ResponseFault::ResetDuringCleanup))
                    {
                        self.reset = true;
                    }
                } else {
                    return Poll::Ready(Err(io::Error::from(io::ErrorKind::UnexpectedEof)));
                }
            }
        }
        Poll::Ready(Ok(input.len()))
    }
    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
    fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

fn login_field(login: &[u8], offset: usize, password: bool) -> String {
    let start = usize::from(u16::from_le_bytes([login[offset], login[offset + 1]]));
    let length = usize::from(u16::from_le_bytes([login[offset + 2], login[offset + 3]]));
    let value = login[start..start + length * 2]
        .as_chunks::<2>()
        .0
        .iter()
        .map(|bytes| {
            let decode = |byte: u8| {
                if password {
                    (byte ^ 0xa5).rotate_left(4)
                } else {
                    byte
                }
            };
            u16::from_le_bytes([decode(bytes[0]), decode(bytes[1])])
        })
        .collect::<Vec<_>>();
    String::from_utf16(&value).unwrap()
}

fn packet(body: Vec<u8>) -> Vec<u8> {
    let length = u16::try_from(body.len() + 8).unwrap().to_be_bytes();
    let mut bytes = vec![4, 1, length[0], length[1], 0, 0, 1, 0];
    bytes.extend(body);
    bytes
}

fn done() -> Vec<u8> {
    vec![0xfd, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
}

fn attention_done() -> Vec<u8> {
    let mut bytes = done();
    bytes[1] = 0x20;
    packet(bytes)
}

fn login() -> Vec<u8> {
    let mut body = vec![0xad, 10, 0, 1, 0x74, 0, 0, 4, 0, 16, 0, 0, 0];
    body.extend(done());
    packet(body)
}

fn nvarchar(bytes: &mut Vec<u8>, value: Option<&str>) {
    match value {
        None => bytes.extend(u16::MAX.to_le_bytes()),
        Some(value) => {
            let value = value.encode_utf16().collect::<Vec<_>>();
            bytes.extend(u16::try_from(value.len() * 2).unwrap().to_le_bytes());
            for c in value {
                bytes.extend(c.to_le_bytes());
            }
        }
    }
}

fn metadata_response() -> Vec<u8> {
    metadata_response_with(false, None)
}

fn metadata_response_with(nullable: bool, error_number: Option<i32>) -> Vec<u8> {
    metadata_response_with_type(nullable, error_number, 127, "bigint")
}

fn metadata_response_with_type(
    nullable: bool,
    error_number: Option<i32>,
    system_type_id: i32,
    system_type_name: &str,
) -> Vec<u8> {
    let columns = [
        ("column_ordinal", 0x26, 4),
        ("name", 0xe7, 0),
        ("is_nullable", 0x68, 1),
        ("system_type_id", 0x26, 4),
        ("system_type_name", 0xe7, 0),
        ("max_length", 0x26, 2),
        ("precision", 0x26, 1),
        ("scale", 0x26, 1),
        ("user_type_name", 0xe7, 0),
        ("error_number", 0x26, 4),
    ];
    let mut body = vec![0x81, 10, 0];
    for (name, kind, len) in columns {
        body.extend([0, 0, 0, 0, 1, 0, kind]);
        if kind == 0xe7 {
            body.extend([0x40, 0x1f, 0x09, 0x04, 0xd0, 0, 0x34]);
        } else {
            body.push(len);
        }
        body.push(u8::try_from(name.len()).unwrap());
        for c in name.encode_utf16() {
            body.extend(c.to_le_bytes());
        }
    }
    body.push(0xd1);
    body.extend([4, 1, 0, 0, 0]);
    nvarchar(&mut body, Some("ID"));
    body.extend([1, u8::from(nullable), 4]);
    body.extend(system_type_id.to_le_bytes());
    nvarchar(&mut body, Some(system_type_name));
    body.extend([2, 8, 0, 1, 19, 1, 0]);
    nvarchar(&mut body, None);
    if let Some(code) = error_number {
        body.push(4);
        body.extend(code.to_le_bytes());
    } else {
        body.push(0);
    }
    body.extend(done());
    packet(body)
}

fn rows_response(values: &[i64]) -> Vec<u8> {
    let mut body = vec![0x81, 1, 0, 0, 0, 0, 0, 0, 0, 0x7f, 2, b'I', 0, b'D', 0];
    for value in values {
        body.push(0xd1);
        body.extend(value.to_le_bytes());
    }
    body.extend(done());
    packet(body)
}

struct FixtureConnector {
    sessions: VecDeque<Vec<Vec<u8>>>,
    calls: Arc<ProtocolCalls>,
    expected_credentials: VecDeque<BasicAuthCredential>,
    fault: Option<ResponseFault>,
    connect_errors: VecDeque<Error>,
}

impl FixtureConnector {
    fn new(sessions: impl IntoIterator<Item = Vec<Vec<u8>>>) -> Self {
        Self {
            sessions: sessions.into_iter().collect(),
            calls: Arc::default(),
            expected_credentials: VecDeque::new(),
            fault: None,
            connect_errors: VecDeque::new(),
        }
    }
}

#[async_trait(?Send)]
impl SqlServerConnector for FixtureConnector {
    type Stream = ScriptedStream;
    async fn connect(
        &mut self,
        _: &SqlServerConnectionConfig,
        mut config: tiberius::Config,
    ) -> Result<tiberius::Client<Self::Stream>, Error> {
        _ = self.calls.connections.fetch_add(1, Ordering::Relaxed);
        if let Some(error) = self.connect_errors.pop_front() {
            return Err(error);
        }
        // Test-only login against in-memory bytes, not a production TLS connector.
        config.encryption(tiberius::EncryptionLevel::NotSupported);
        config.handshake_timeout(Some(Duration::from_secs(1)));
        let mut responses = VecDeque::from([packet(vec![1, 0, 6, 0, 1, 0xff, 2]), login()]);
        responses.extend(self.sessions.pop_front().ok_or(Error::Driver)?);
        tiberius::Client::connect(
            config,
            ScriptedStream {
                responses,
                readable: VecDeque::new(),
                writing: Vec::new(),
                calls: Arc::clone(&self.calls),
                expected_login: self.expected_credentials.pop_front(),
                fault: self.fault,
                delayed: None,
                reset: false,
            },
        )
        .await
        .map_err(Error::from)
    }
}

fn adapter(
    snapshot: bool,
    responses: Vec<Vec<u8>>,
) -> (SqlServerAdapter<FixtureConnector>, CompiledQuery) {
    let config = config(snapshot);
    let query = config.query().clone();
    (
        SqlServerAdapter::new(
            config,
            FixtureConnector::new([responses]),
            Box::new(Credentials),
        ),
        query,
    )
}

/// Scenario: Stock Tiberius logs in, describes the query, and reads a bounded incremental result.
/// Guarantees: Real driver APIs return normalized cursor rows under the SQL Server identity.
#[tokio::test]
async fn stock_driver_reads_a_validated_page() {
    let (mut adapter, query) = adapter(
        false,
        vec![
            metadata_response(),
            metadata_response(),
            rows_response(&[1, 2]),
        ],
    );
    assert_eq!(adapter.system(), DatabaseSystem::SqlServer);
    assert_eq!(adapter.system().as_str(), "microsoft.sql_server");
    _ = adapter.begin_operation().unwrap();
    let columns = adapter.validate_query(&query).await.unwrap();
    assert_eq!(columns[0].name, "ID");
    _ = adapter.begin_operation().unwrap();
    let page = adapter
        .execute(&query, &query.watermark().initial())
        .await
        .unwrap();
    assert_eq!(page.rows.len(), 2);
    assert_eq!(page.rows[1].cursor, Cursor::Scalar(ScalarValue::Int64(2)));
    adapter.shutdown().await.unwrap();
}

/// Scenario: A snapshot fits entirely within the row budget.
/// Guarantees: Real Tiberius rows carry Snapshot cursors rather than invented incremental keys.
#[tokio::test]
async fn stock_driver_reads_snapshot_rows() {
    let (mut adapter, query) = adapter(
        true,
        vec![
            metadata_response(),
            metadata_response(),
            rows_response(&[1, 1]),
        ],
    );
    _ = adapter.begin_operation().unwrap();
    _ = adapter.validate_query(&query).await.unwrap();
    _ = adapter.begin_operation().unwrap();
    let page = adapter.execute(&query, &Cursor::Snapshot).await.unwrap();
    assert_eq!(page.rows.len(), 2);
    assert!(page.rows.iter().all(|row| row.cursor == Cursor::Snapshot));
    adapter.shutdown().await.unwrap();
}

/// Scenario: The snapshot overflow probe returns one row more than the limit.
/// Guarantees: No snapshot prefix is returned and source-local overflow remains explicit.
#[tokio::test]
async fn stock_driver_rejects_snapshot_overflow() {
    let (mut adapter, query) = adapter(
        true,
        vec![
            metadata_response(),
            metadata_response(),
            rows_response(&[1, 2, 3]),
            attention_done(),
        ],
    );
    _ = adapter.begin_operation().unwrap();
    _ = adapter.validate_query(&query).await.unwrap();
    _ = adapter.begin_operation().unwrap();
    assert_eq!(
        adapter.execute(&query, &Cursor::Snapshot).await,
        Err(Error::SnapshotLimit)
    );
    adapter.shutdown().await.unwrap();
}

/// Scenario: A database operation stops responding after successful metadata validation.
/// Guarantees: The absolute deadline ends the operation and unconfirmed cleanup blocks reuse.
#[tokio::test]
async fn query_deadline_and_failed_cleanup_prevent_reconnect() {
    let (mut adapter, query) = adapter(
        false,
        vec![metadata_response(), metadata_response(), vec![], vec![]],
    );
    _ = adapter.begin_operation().unwrap();
    _ = adapter.validate_query(&query).await.unwrap();
    _ = adapter.begin_operation().unwrap();
    let result = tokio::time::timeout(
        Duration::from_secs(4),
        adapter.execute(&query, &query.watermark().initial()),
    )
    .await
    .unwrap();
    assert!(matches!(result, Err(Error::Driver | Error::Deadline)));
    assert!(adapter.begin_operation().is_err());
    assert!(adapter.shutdown().await.is_err());
}

/// Scenario: An outer timeout drops validation or execution while ATTENTION is unacknowledged.
/// Guarantees: Cleanup failure survives future drop and prevents reuse or successful shutdown.
#[tokio::test(start_paused = true)]
async fn stock_driver_cleanup_survives_outer_timeout() {
    for validating in [true, false] {
        let responses = if validating {
            vec![vec![], vec![]]
        } else {
            vec![metadata_response(), metadata_response(), vec![], vec![]]
        };
        let config = config(false);
        let query = config.query().clone();
        let connector = FixtureConnector::new([responses]);
        let calls = Arc::clone(&connector.calls);
        let mut adapter = SqlServerAdapter::new(config, connector, Box::new(Credentials));
        if !validating {
            _ = adapter.begin_operation().unwrap();
            _ = adapter.validate_query(&query).await.unwrap();
        }
        let cancellation = adapter.begin_operation().unwrap();
        let cursor = query.watermark().initial();
        let mut operation = Box::pin(async {
            if validating {
                adapter.validate_query(&query).await.map(|_| ())
            } else {
                adapter.execute(&query, &cursor).await.map(|_| ())
            }
        });
        assert!(futures::poll!(operation.as_mut()).is_pending());
        cancellation.cancel().await.unwrap();
        assert!(futures::poll!(operation.as_mut()).is_pending());
        let requests = if validating { 4 } else { 6 };
        assert_eq!(
            calls.requests.load(Ordering::Relaxed),
            requests,
            "ATTENTION was sent before the outer timeout",
        );
        assert!(
            tokio::time::timeout(Duration::from_millis(100), operation)
                .await
                .is_err()
        );
        assert!(adapter.begin_operation().is_err());
        assert_eq!(adapter.reconnect(&query).await, Err(Error::NotValidated));
        assert_eq!(
            adapter.validate_query(&query).await,
            Err(Error::NotValidated)
        );
        assert_eq!(
            adapter.execute(&query, &cursor).await,
            Err(Error::NotValidated)
        );
        assert_eq!(adapter.shutdown().await, Err(Error::Driver));
        assert_eq!(calls.requests.load(Ordering::Relaxed), requests);
    }
}

/// Scenario: Cancellation of validation or execution receives a confirmed ATTENTION response.
/// Guarantees: Successful cleanup clears the failure latch and permits a fresh validated connection.
#[tokio::test]
async fn stock_driver_confirmed_cleanup_allows_revalidation() {
    for validating in [true, false] {
        let responses = if validating {
            vec![vec![], attention_done()]
        } else {
            vec![
                metadata_response(),
                metadata_response(),
                vec![],
                attention_done(),
            ]
        };
        let config = config(false);
        let query = config.query().clone();
        let connector = FixtureConnector::new([responses, vec![metadata_response()]]);
        let calls = Arc::clone(&connector.calls);
        let mut adapter = SqlServerAdapter::new(config, connector, Box::new(Credentials));
        if !validating {
            _ = adapter.begin_operation().unwrap();
            _ = adapter.validate_query(&query).await.unwrap();
        }
        let cancellation = adapter.begin_operation().unwrap();
        let cursor = query.watermark().initial();
        let mut operation = Box::pin(async {
            if validating {
                adapter.validate_query(&query).await.map(|_| ())
            } else {
                adapter.execute(&query, &cursor).await.map(|_| ())
            }
        });
        assert!(futures::poll!(operation.as_mut()).is_pending());
        cancellation.cancel().await.unwrap();
        assert_eq!(operation.await, Err(Error::Cancelled));
        _ = adapter.begin_operation().unwrap();
        _ = adapter.validate_query(&query).await.unwrap();
        assert_eq!(calls.connections.load(Ordering::Relaxed), 2);
        adapter.shutdown().await.unwrap();
    }
}

fn uncertain_response(fault: ResponseFault) -> (SqlServerReceiverConfig, FixtureConnector) {
    use otel_arrow_dfe_scraper::database::{CursorRow, Row};
    let bytes = Row {
        values: vec![CellValue::Int64(8)],
    }
    .normalized_size()
        + size_of::<CursorRow>() as u64;
    let config = config_with_bytes(false, bytes);
    let rows = if matches!(fault, ResponseFault::ResetAfterWrite) {
        Vec::new()
    } else if matches!(
        fault,
        ResponseFault::ResetDuringRows | ResponseFault::ResetOnceDuringRows
    ) {
        let complete = rows_response(&[8]);
        let mut partial = packet(complete[8..complete.len() - done().len()].to_vec());
        partial[1] = 0; // The next TDS packet never arrives; the transport resets instead.
        partial
    } else {
        rows_response(&[8, 9])
    };
    let acknowledgement = match fault {
        ResponseFault::SplitAttention => {
            let mut error_done = done();
            error_done[1] = 0x02;
            // One response to one ATTENTION, but two separately terminated TDS messages.
            let mut response = packet(error_done);
            response.extend(attention_done());
            response
        }
        ResponseFault::LateAttention | ResponseFault::ResetOnceDuringRows => attention_done(),
        _ => Vec::new(),
    };
    let mut connector = FixtureConnector::new([vec![
        metadata_response(),
        metadata_response(),
        rows,
        acknowledgement,
    ]]);
    connector.fault = Some(fault);
    (config, connector)
}

fn uncertain_error(fault: ResponseFault) -> Error {
    match fault {
        ResponseFault::ResetBeforeWrite => {
            Error::UnconfirmedTransport(io::ErrorKind::ConnectionReset)
        }
        // Tiberius 0.13.0 maps framed-stream read errors to BrokenPipe.
        ResponseFault::ResetDuringRows
        | ResponseFault::ResetAfterWrite
        | ResponseFault::ResetDuringCleanup => {
            Error::UnconfirmedTransport(io::ErrorKind::BrokenPipe)
        }
        _ => Error::Driver,
    }
}

/// Scenario: Metadata or fetch exceeds its absolute deadline and receives a genuine cancellation ACK.
/// Guarantees: Only confirmed query timeouts become retryable; the failed operation returns no page.
#[tokio::test(start_paused = true)]
async fn acknowledged_query_timeouts_are_retryable() {
    for validating in [true, false] {
        let responses = if validating {
            vec![vec![], attention_done()]
        } else {
            vec![
                metadata_response(),
                metadata_response(),
                vec![],
                attention_done(),
            ]
        };
        let (mut adapter, query) = adapter(false, responses);
        if !validating {
            _ = adapter.begin_operation().unwrap();
            _ = adapter.validate_query(&query).await.unwrap();
        }
        _ = adapter.begin_operation().unwrap();
        let started = tokio::time::Instant::now();
        let error = if validating {
            adapter.validate_query(&query).await.unwrap_err()
        } else {
            adapter
                .execute(&query, &query.watermark().initial())
                .await
                .unwrap_err()
        };
        assert_eq!(error, Error::QueryTimeout);
        assert!(SqlServerAdapter::<FixtureConnector>::is_retryable(&error));
        assert!(started.elapsed() >= query.timeout());
        assert!(started.elapsed() < query.timeout() + Duration::from_secs(1));
        _ = adapter.begin_operation().unwrap();
        adapter.shutdown().await.unwrap();
    }
}

/// Scenario: Streaming reports a transport reset but a later cancellation response is acknowledged.
/// Guarantees: Explicit confirmation preserves the retryable original transport cause, never a partial row page.
#[tokio::test]
async fn acknowledged_transport_cleanup_preserves_original_error() {
    let (config, connector) = uncertain_response(ResponseFault::ResetOnceDuringRows);
    let query = config.query().clone();
    let calls = Arc::clone(&connector.calls);
    let mut adapter = SqlServerAdapter::new(config, connector, Box::new(Credentials));
    _ = adapter.begin_operation().unwrap();
    _ = adapter.validate_query(&query).await.unwrap();
    _ = adapter.begin_operation().unwrap();
    let error = adapter
        .execute(&query, &query.watermark().initial())
        .await
        .unwrap_err();
    assert_eq!(error, Error::Transport(io::ErrorKind::BrokenPipe));
    assert!(SqlServerAdapter::<FixtureConnector>::is_retryable(&error));
    assert_eq!(calls.resets.load(Ordering::Relaxed), 1);
    _ = adapter.begin_operation().unwrap();
    adapter.shutdown().await.unwrap();
}

/// Scenario: Streaming resets and an outer timeout drops the ensuing unacknowledged cleanup.
/// Guarantees: Shutdown retains the original redacted transport cause and reuse remains forbidden.
#[tokio::test(start_paused = true)]
async fn interrupted_transport_cleanup_retains_cause() {
    let (config, mut connector) = uncertain_response(ResponseFault::ResetOnceDuringRows);
    *connector.sessions[0].last_mut().unwrap() = Vec::new();
    let query = config.query().clone();
    let mut adapter = SqlServerAdapter::new(config, connector, Box::new(Credentials));
    _ = adapter.begin_operation().unwrap();
    _ = adapter.validate_query(&query).await.unwrap();
    _ = adapter.begin_operation().unwrap();
    assert!(
        tokio::time::timeout(
            Duration::from_millis(100),
            adapter.execute(&query, &query.watermark().initial()),
        )
        .await
        .is_err()
    );
    assert!(adapter.begin_operation().is_err());
    assert_eq!(adapter.reconnect(&query).await, Err(Error::NotValidated));
    assert_eq!(
        adapter.shutdown().await,
        Err(Error::UnconfirmedTransport(io::ErrorKind::BrokenPipe))
    );
}

/// Scenario: Published Tiberius sees split, missing, late, interrupted, or reset cancellation responses.
/// Guarantees: No partial page escapes; uncertain cleanup blocks every reuse path and stays failed at shutdown.
#[tokio::test(start_paused = true)]
async fn published_driver_contains_uncertain_cancellation() {
    for fault in [
        ResponseFault::SplitAttention,
        ResponseFault::MissingAttention,
        ResponseFault::LateAttention,
        ResponseFault::DroppedCleanup,
        ResponseFault::ResetDuringRows,
        ResponseFault::ResetBeforeWrite,
        ResponseFault::ResetAfterWrite,
        ResponseFault::ResetDuringCleanup,
    ] {
        let (config, connector) = uncertain_response(fault);
        let query = config.query().clone();
        let calls = Arc::clone(&connector.calls);
        let mut adapter = SqlServerAdapter::new(config, connector, Box::new(Credentials));
        _ = adapter.begin_operation().unwrap();
        _ = adapter.validate_query(&query).await.unwrap();
        _ = adapter.begin_operation().unwrap();
        let cursor = Cursor::Scalar(ScalarValue::Int64(7));
        let started = tokio::time::Instant::now();
        let operation = adapter.execute(&query, &cursor);
        if matches!(fault, ResponseFault::DroppedCleanup) {
            assert!(
                tokio::time::timeout(Duration::from_millis(100), operation)
                    .await
                    .is_err()
            );
        } else {
            assert_eq!(operation.await, Err(uncertain_error(fault)), "{fault:?}");
        }
        assert!(
            started.elapsed() < Duration::from_secs(2),
            "one-second cleanup cap"
        );
        if matches!(
            fault,
            ResponseFault::MissingAttention | ResponseFault::LateAttention
        ) {
            assert!(started.elapsed() >= Duration::from_secs(1));
        }
        if matches!(
            fault,
            ResponseFault::ResetDuringRows
                | ResponseFault::ResetBeforeWrite
                | ResponseFault::ResetAfterWrite
                | ResponseFault::ResetDuringCleanup
        ) {
            assert!(
                calls.resets.load(Ordering::Relaxed) > 0,
                "streaming actually reset"
            );
        }
        assert!(adapter.begin_operation().is_err());
        assert_eq!(adapter.reconnect(&query).await, Err(Error::NotValidated));
        assert_eq!(
            adapter.validate_query(&query).await,
            Err(Error::NotValidated)
        );
        assert_eq!(
            adapter.execute(&query, &cursor).await,
            Err(Error::NotValidated)
        );
        assert_eq!(adapter.shutdown().await, Err(uncertain_error(fault)));
        assert_eq!(calls.connections.load(Ordering::Relaxed), 1);
        assert_eq!(
            calls.requests.load(Ordering::Relaxed),
            if matches!(fault, ResponseFault::ResetBeforeWrite) {
                4
            } else {
                6
            },
            "no request after ATTENTION"
        );
    }
}

struct CleanupFailureProbe {
    receiver: otel_arrow_dfe_scraper::DatabaseReceiver<SqlServerAdapter<FixtureConnector>>,
    expected: Error,
}

#[async_trait(?Send)]
impl local::Receiver<OtapPdata> for CleanupFailureProbe {
    async fn start(
        self: Box<Self>,
        controls: local::ControlChannel<OtapPdata>,
        effects: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, EngineError> {
        let result = tokio::time::timeout(
            Duration::from_secs(5),
            local::Receiver::start(Box::new(self.receiver), controls, effects),
        )
        .await
        .expect("uncertain cleanup must end boundedly");
        let Err(error) = result else {
            panic!("cleanup cannot report success")
        };
        assert!(error.to_string().contains("cleanup failed"), "{error}");
        assert!(
            error.to_string().contains(&self.expected.to_string()),
            "{error}"
        );
        Ok(TerminalState::default())
    }
}

fn run_uncertain_response(directory: &std::path::Path, fault: ResponseFault) {
    use otel_arrow_dfe_config::node::NodeUserConfig;
    use otel_arrow_dfe_engine::{
        control::NodeControlMsg,
        memory_limiter::{LocalReceiverAdmissionState, MemoryPressureState},
        receiver::ReceiverWrapper,
        testing::{receiver::TestRuntime, test_node},
    };
    use otel_arrow_dfe_scraper::{CheckpointStore, DatabaseReceiver, SourceBinding};
    let (config, connector) = uncertain_response(fault);
    let query = config.query().clone();
    let calls = Arc::clone(&connector.calls);
    let store = CheckpointStore::new(
        directory,
        "group",
        "pipeline",
        "uncertain",
        "events",
        config.config_fingerprint().to_owned(),
    );
    let source = SourceBinding::acquire(store.clone()).unwrap();
    let previous = store
        .write(0, &Cursor::Scalar(ScalarValue::Int64(7)))
        .unwrap()
        .0;
    let receiver = DatabaseReceiver::new(
        SqlServerAdapter::new(config, connector, Box::new(Credentials)),
        query,
        source,
        Duration::from_millis(1),
        5,
        LocalReceiverAdmissionState::from_process_state(&MemoryPressureState::default()),
        None,
    );
    let runtime = TestRuntime::<OtapPdata>::new();
    let wrapper = ReceiverWrapper::local(
        CleanupFailureProbe {
            receiver,
            expected: uncertain_error(fault),
        },
        test_node(runtime.config().name.clone()),
        Arc::new(NodeUserConfig::new_receiver_config(
            "urn:otel:receiver:sql_server_test",
        )),
        runtime.config(),
    );
    let observed = Arc::clone(&calls);
    runtime
        .set_receiver(wrapper)
        .run_test(|_| async {})
        .run_validation_concurrent(move |mut ctx| async move {
            if matches!(fault, ResponseFault::DroppedCleanup) {
                tokio::time::timeout(Duration::from_secs(2), async {
                    while observed.requests.load(Ordering::Relaxed) < 6 {
                        tokio::task::yield_now().await;
                    }
                })
                .await
                .expect("ATTENTION began");
                ctx.send_control_msg(NodeControlMsg::Shutdown {
                    deadline: std::time::Instant::now() + Duration::from_millis(100),
                    reason: "interrupt unacknowledged cleanup".into(),
                })
                .await
                .unwrap();
            }
            assert!(
                tokio::time::timeout(Duration::from_secs(6), ctx.recv())
                    .await
                    .unwrap()
                    .is_err(),
                "uncertain prefixes must never emit"
            );
        });
    assert_eq!(
        store.read().unwrap(),
        Some(previous),
        "{fault:?}: checkpoint unchanged"
    );
    assert_eq!(
        calls.connections.load(Ordering::Relaxed),
        1,
        "no recovery connection"
    );
    assert_eq!(
        calls.requests.load(Ordering::Relaxed),
        if matches!(fault, ResponseFault::ResetBeforeWrite) {
            4
        } else {
            6
        },
        "no request after ATTENTION"
    );
    assert!(
        SourceBinding::acquire(store).is_err(),
        "{fault:?}: source stays quarantined"
    );
}

/// Scenario: Real shared delivery encounters each unconfirmed response with a durable starting cursor.
/// Guarantees: No output/checkpoint advancement or reconnect occurs; ownership survives receiver exit until process exit.
#[test]
fn shared_controller_quarantines_uncertain_driver_responses() {
    const CHILD: &str = "OTAP_SQL_SERVER_CONTAINMENT_CHILD";
    if let Some(root) = std::env::var_os(CHILD) {
        for (index, fault) in [
            ResponseFault::SplitAttention,
            ResponseFault::MissingAttention,
            ResponseFault::LateAttention,
            ResponseFault::DroppedCleanup,
            ResponseFault::ResetDuringRows,
            ResponseFault::ResetBeforeWrite,
            ResponseFault::ResetAfterWrite,
            ResponseFault::ResetDuringCleanup,
        ]
        .into_iter()
        .enumerate()
        {
            let directory = std::path::PathBuf::from(&root).join(index.to_string());
            std::fs::create_dir(&directory).unwrap();
            run_uncertain_response(&directory, fault);
        }
        return;
    }
    // Quarantined real leases intentionally live until process exit; isolate them
    // so the parent can remove every fixture file, including on Windows.
    let directory = tempfile::tempdir().unwrap();
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact",
            "receivers::sql_server_receiver::adapter_tests::shared_controller_quarantines_uncertain_driver_responses",
            "--nocapture"])
        .env(CHILD, directory.path())
        .spawn().unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(30);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success(), "containment subprocess failed");
            break;
        }
        if std::time::Instant::now() >= deadline {
            child.kill().unwrap();
            _ = child.wait();
            panic!("containment subprocess exceeded its watchdog");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    directory
        .close()
        .expect("process exit released fixture handles");
}

/// Scenario: A result keeps its column name but changes its wire type after metadata discovery.
/// Guarantees: Real driver metadata cannot bypass the previously validated type.
#[tokio::test]
async fn executed_result_type_drift_fails_closed() {
    let mut response = rows_response(&[1]);
    response[17] = 0x3e; // Float8 has the same fixed width as the previous BigInt.
    let (mut adapter, query) = adapter(
        false,
        vec![
            metadata_response(),
            metadata_response(),
            response,
            attention_done(),
        ],
    );
    _ = adapter.begin_operation().unwrap();
    _ = adapter.validate_query(&query).await.unwrap();
    _ = adapter.begin_operation().unwrap();
    assert_eq!(
        adapter.execute(&query, &query.watermark().initial()).await,
        Err(Error::Metadata)
    );
    adapter.shutdown().await.unwrap();
}

struct EarlyResponseFailure {
    validating: bool,
    responses: Vec<Vec<u8>>,
    error: Error,
}

fn early_response_failures() -> [EarlyResponseFailure; 5] {
    let unsupported = || metadata_response_with_type(false, None, 241, "xml");
    let float_metadata = || metadata_response_with_type(false, None, 62, "float");
    let mut drift = rows_response(&[1, 2]);
    drift[17] = 0x3e;
    let mut nonfinite = rows_response(&[0, 1]);
    nonfinite[17] = 0x3e;
    nonfinite[24..32].copy_from_slice(&f64::NAN.to_le_bytes());
    [
        EarlyResponseFailure {
            validating: true,
            responses: vec![unsupported()],
            error: Error::Metadata,
        },
        EarlyResponseFailure {
            validating: true,
            responses: vec![metadata_response_with(false, Some(11509))],
            error: Error::Metadata,
        },
        EarlyResponseFailure {
            validating: false,
            responses: vec![metadata_response(), unsupported()],
            error: Error::Metadata,
        },
        EarlyResponseFailure {
            validating: false,
            responses: vec![metadata_response(), metadata_response(), drift],
            error: Error::Metadata,
        },
        EarlyResponseFailure {
            validating: false,
            responses: vec![float_metadata(), float_metadata(), nonfinite],
            error: Error::Conversion,
        },
    ]
}

/// Scenario: Describe or fetch rejects an unread response and ATTENTION is acknowledged, stalled, or dropped.
/// Guarantees: Only confirmed cleanup permits reuse and successful shutdown; failures never yield a page.
#[tokio::test(start_paused = true)]
async fn early_response_failures_require_confirmed_cleanup() {
    #[derive(Clone, Copy)]
    enum Cleanup {
        Acknowledged,
        Stalled,
        Dropped,
    }
    for cleanup in [Cleanup::Acknowledged, Cleanup::Stalled, Cleanup::Dropped] {
        for case in early_response_failures() {
            let mut responses = case.responses;
            responses.push(if matches!(cleanup, Cleanup::Acknowledged) {
                attention_done()
            } else {
                vec![]
            });
            let expected_requests = responses.len() + 2;
            let config = config(true);
            let query = config.query().clone();
            let connector = FixtureConnector::new([responses, vec![metadata_response()]]);
            let calls = Arc::clone(&connector.calls);
            let mut adapter = SqlServerAdapter::new(config, connector, Box::new(Credentials));
            if !case.validating {
                _ = adapter.begin_operation().unwrap();
                _ = adapter.validate_query(&query).await.unwrap();
            }
            _ = adapter.begin_operation().unwrap();
            let operation = async {
                if case.validating {
                    adapter.validate_query(&query).await.map(|_| ())
                } else {
                    adapter.execute(&query, &Cursor::Snapshot).await.map(|_| ())
                }
            };
            if matches!(cleanup, Cleanup::Dropped) {
                assert!(
                    tokio::time::timeout(Duration::from_millis(100), operation)
                        .await
                        .is_err()
                );
            } else {
                assert_eq!(
                    operation.await,
                    Err(if matches!(cleanup, Cleanup::Acknowledged) {
                        case.error
                    } else {
                        Error::Driver
                    })
                );
            }
            assert_eq!(calls.requests.load(Ordering::Relaxed), expected_requests);
            if matches!(cleanup, Cleanup::Acknowledged) {
                _ = adapter.begin_operation().unwrap();
                _ = adapter.validate_query(&query).await.unwrap();
                assert_eq!(calls.connections.load(Ordering::Relaxed), 2);
                adapter.shutdown().await.unwrap();
            } else {
                assert!(adapter.begin_operation().is_err());
                assert_eq!(adapter.reconnect(&query).await, Err(Error::NotValidated));
                assert_eq!(
                    adapter.validate_query(&query).await,
                    Err(Error::NotValidated)
                );
                assert_eq!(
                    adapter.execute(&query, &Cursor::Snapshot).await,
                    Err(Error::NotValidated)
                );
                assert_eq!(adapter.shutdown().await, Err(Error::Driver));
                assert_eq!(calls.connections.load(Ordering::Relaxed), 1);
                assert_eq!(calls.requests.load(Ordering::Relaxed), expected_requests);
            }
        }
    }
}

/// Scenario: An outer timeout drops submission of a describe or fetch before a response arrives.
/// Guarantees: Unconfirmed work stays latched even when the adapter never enters its cancellation branch.
#[tokio::test(start_paused = true)]
async fn unfinished_submission_survives_outer_timeout() {
    for validating in [true, false] {
        let responses = if validating {
            vec![vec![]]
        } else {
            vec![metadata_response(), metadata_response(), vec![]]
        };
        let (mut adapter, query) = adapter(false, responses);
        if !validating {
            _ = adapter.begin_operation().unwrap();
            _ = adapter.validate_query(&query).await.unwrap();
        }
        _ = adapter.begin_operation().unwrap();
        let operation = async {
            if validating {
                adapter.validate_query(&query).await.map(|_| ())
            } else {
                adapter
                    .execute(&query, &query.watermark().initial())
                    .await
                    .map(|_| ())
            }
        };
        assert!(
            tokio::time::timeout(Duration::from_millis(100), operation)
                .await
                .is_err()
        );
        assert!(adapter.begin_operation().is_err());
        assert_eq!(adapter.shutdown().await, Err(Error::Driver));
    }
}

/// Scenario: A completed describe changes the plan, or cursor binding fails before fetch submission.
/// Guarantees: Local validation errors do not send ATTENTION to an idle session or poison shutdown.
#[tokio::test]
async fn completed_response_errors_do_not_cancel_idle_sessions() {
    for snapshot in [true, false] {
        let responses = vec![
            metadata_response(),
            if snapshot {
                metadata_response_with_type(false, None, 62, "float")
            } else {
                metadata_response()
            },
        ];
        let (mut adapter, query) = adapter(snapshot, responses);
        _ = adapter.begin_operation().unwrap();
        _ = adapter.validate_query(&query).await.unwrap();
        _ = adapter.begin_operation().unwrap();
        assert_eq!(
            adapter.execute(&query, &Cursor::Snapshot).await,
            Err(if snapshot {
                Error::Metadata
            } else {
                Error::Cursor
            })
        );
        adapter.shutdown().await.unwrap();
    }
}

/// Scenario: TDS represents an offset timestamp as a UTC date/time plus its offset.
/// Guarantees: Both the instant and original offset survive normalization.
#[test]
fn zoned_timestamp_preserves_instant_and_offset() {
    let utc = time::DateTime2::new(time::Date::new(0), time::Time::new(0, 7));
    let value = ColumnData::DateTimeOffset(Some(time::DateTimeOffset::new(utc, 330)));
    assert_eq!(
        convert(value, &column(NativeType::DateTimeOffset, 7, false)).unwrap(),
        CellValue::TimestampTz("0001-01-01T05:30:00.0000000+05:30".into()),
    );
}

/// Scenario: An incremental row contains a datetime2 key and a signed tie-breaker.
/// Guarantees: Composite extraction preserves timestamp precision and the integer key.
#[test]
fn composite_conversion_and_binds_use_both_keys() {
    let mut raw = raw_config(false, 4096);
    raw["query"]["statement"] = json!(
        "SELECT EVENT_TS, ID FROM dbo.events WHERE EVENT_TS > @ts OR (EVENT_TS = @ts AND ID > @id) ORDER BY EVENT_TS, ID"
    );
    raw["watermark"] = json!({
        "mode":"composite",
        "timestamp":{"column":"EVENT_TS","bind":"ts","initial":"0001-01-01T00:00:00Z","timezone":"UTC"},
        "tie_breaker":{"column":"ID","bind":"id","initial":0}
    });
    let config: SqlServerReceiverConfig = serde_json::from_value(raw).unwrap();
    let mut time_column = column(NativeType::DateTime2, 7, false);
    time_column.metadata.name = "EVENT_TS".into();
    time_column.metadata.source_type = "datetime2(7)".into();
    let plan = ResultPlan::new(
        vec![time_column, column(NativeType::BigInt, 0, false)],
        config.query(),
    )
    .unwrap();
    let row = tiberius::Row::builder()
        .column(
            "EVENT_TS",
            ColumnType::Datetime2,
            ColumnData::DateTime2(Some(time::DateTime2::new(
                time::Date::new(0),
                time::Time::new(1234567, 7),
            ))),
        )
        .column("ID", ColumnType::Int8, ColumnData::I64(Some(9)))
        .build();
    let row = plan.row(row).unwrap();
    assert!(
        plan.request(config.statement(), config.query(), &row.cursor)
            .is_ok()
    );
    let Cursor::Composite(value) = row.cursor else {
        panic!("composite cursor")
    };
    assert_eq!(value.timestamp, "0001-01-01T00:00:00.1234567Z");
    assert_eq!(value.tie_breaker, 9);
}

/// Scenario: The real shared controller delivers stock-driver rows and receives ACK.
/// Guarantees: Rows become OTLP data and only acknowledged progress reaches the file store.
#[test]
fn shared_controller_checkpoints_stock_driver_rows() {
    use otel_arrow_dfe_config::node::NodeUserConfig;
    use otel_arrow_dfe_engine::{
        control::{AckMsg, NodeControlMsg},
        memory_limiter::{LocalReceiverAdmissionState, MemoryPressureState},
        receiver::ReceiverWrapper,
        testing::{receiver::TestRuntime, test_node},
    };
    use otel_arrow_dfe_otap::{pdata::OtapPdata, testing::next_ack};
    use otel_arrow_dfe_scraper::{CheckpointStore, DatabaseReceiver, SourceBinding};

    let directory = tempfile::tempdir().unwrap();
    let config = config(false);
    let query = config.query().clone();
    let store = CheckpointStore::new(
        directory.path(),
        "group",
        "pipeline",
        "sql-server",
        "events",
        config.config_fingerprint().to_owned(),
    );
    let source = SourceBinding::acquire(store.clone()).unwrap();
    let connector = FixtureConnector::new([vec![
        metadata_response(),
        metadata_response(),
        rows_response(&[1, 2]),
        metadata_response(),
        rows_response(&[]),
    ]]);
    let receiver = DatabaseReceiver::new(
        SqlServerAdapter::new(config, connector, Box::new(Credentials)),
        query,
        source,
        Duration::from_millis(1),
        5,
        LocalReceiverAdmissionState::from_process_state(&MemoryPressureState::default()),
        None,
    );
    let runtime = TestRuntime::<OtapPdata>::new();
    let wrapper = ReceiverWrapper::local(
        receiver,
        test_node(runtime.config().name.clone()),
        Arc::new(NodeUserConfig::new_receiver_config(
            "urn:otel:receiver:sql_server_test",
        )),
        runtime.config(),
    );
    let checkpoint = store.clone();
    runtime
        .set_receiver(wrapper)
        .run_test(|_| async {})
        .run_validation_concurrent(move |mut ctx| async move {
            let mut data = ctx.recv().await.expect("database page");
            assert_eq!(data.num_items(), 2);
            assert!(
                checkpoint.read().unwrap().is_none(),
                "no checkpoint before ACK"
            );
            let (_, ack) = next_ack(AckMsg::new(data)).expect("ACK route");
            ctx.send_control_msg(NodeControlMsg::Ack(ack))
                .await
                .unwrap();
            tokio::time::timeout(Duration::from_secs(3), async {
                while checkpoint.read().unwrap().is_none() {
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
            })
            .await
            .unwrap();
            ctx.send_control_msg(NodeControlMsg::Shutdown {
                deadline: std::time::Instant::now() + Duration::from_secs(2),
                reason: "SQL Server fixture complete".into(),
            })
            .await
            .unwrap();
        });
    let committed = store.read().unwrap().unwrap();
    assert_eq!(committed.cursor, Cursor::Scalar(ScalarValue::Int64(2)));
}

/// Scenario: Normalized bytes allow one incremental row but not two, followed by another poll.
/// Guarantees: Acknowledged prefixes reuse one connection, revalidate metadata, and ignore stale cancellation.
#[tokio::test]
async fn byte_limited_prefix_reuses_acknowledged_connection() {
    use otel_arrow_dfe_scraper::database::{CursorRow, Row};
    let row = Row {
        values: vec![CellValue::Int64(1)],
    };
    let one_row = row.normalized_size() + size_of::<CursorRow>() as u64;
    let config = config_with_bytes(false, one_row);
    let query = config.query().clone();
    let connector = FixtureConnector::new([vec![
        metadata_response(),
        metadata_response(),
        rows_response(&[1, 2]),
        attention_done(),
        metadata_response(),
        rows_response(&[2]),
    ]]);
    let calls = Arc::clone(&connector.calls);
    let mut adapter = SqlServerAdapter::new(config, connector, Box::new(Credentials));
    let stale = adapter.begin_operation().unwrap();
    _ = adapter.validate_query(&query).await.unwrap();
    _ = adapter.begin_operation().unwrap();
    let first = adapter
        .execute(&query, &query.watermark().initial())
        .await
        .unwrap();
    assert_eq!(first.rows.len(), 1);
    _ = adapter.begin_operation().unwrap();
    stale.cancel().await.unwrap();
    let next = adapter
        .execute(&query, &first.rows[0].cursor)
        .await
        .unwrap();
    assert_eq!(next.rows[0].cursor, Cursor::Scalar(ScalarValue::Int64(2)));
    assert_eq!(calls.connections.load(Ordering::Relaxed), 1);
    assert_eq!(calls.requests.load(Ordering::Relaxed), 8);
    adapter.shutdown().await.unwrap();
}

#[derive(Clone, Copy, Debug)]
enum SafeRecovery {
    DescribeTimeout,
    FetchTimeout,
    ConnectReset,
    AcknowledgedReset,
    AcknowledgedPrefix,
}

struct SafeRecoveryProbe(
    otel_arrow_dfe_scraper::DatabaseReceiver<SqlServerAdapter<FixtureConnector>>,
);

#[async_trait(?Send)]
impl local::Receiver<OtapPdata> for SafeRecoveryProbe {
    async fn start(
        self: Box<Self>,
        controls: local::ControlChannel<OtapPdata>,
        effects: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, EngineError> {
        tokio::time::timeout(
            Duration::from_secs(15),
            local::Receiver::start(Box::new(self.0), controls, effects),
        )
        .await
        .expect("bounded recovery fixture")
    }
}

fn assert_row_ids(pdata: &OtapPdata, expected: &[i64]) {
    use otel_arrow_dfe_pdata::{
        otlp::OtlpProtoBytes,
        proto::opentelemetry::{common::v1::any_value, logs::v1::LogsData},
    };
    use otel_arrow_dfe_pdata_codec::PayloadData;
    use prost::Message;
    let PayloadData::OtlpBytes(OtlpProtoBytes::ExportLogsRequest(bytes)) =
        pdata.clone().payload().into_data()
    else {
        panic!("expected OTLP log bytes");
    };
    let logs = LogsData::decode(bytes).unwrap();
    let actual = logs
        .resource_logs
        .iter()
        .flat_map(|resource| &resource.scope_logs)
        .flat_map(|scope| &scope.log_records)
        .map(|record| {
            let Some(any_value::Value::KvlistValue(body)) =
                record.body.as_ref().and_then(|body| body.value.as_ref())
            else {
                panic!("expected structured row")
            };
            let value = body.values.iter().find(|field| field.key == "ID").unwrap();
            let Some(any_value::Value::IntValue(id)) =
                value.value.as_ref().and_then(|value| value.value.as_ref())
            else {
                panic!("expected exact integer")
            };
            *id
        })
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
}

fn run_safe_recovery(case: SafeRecovery) {
    use otel_arrow_dfe_config::node::NodeUserConfig;
    use otel_arrow_dfe_engine::{
        control::{AckMsg, NodeControlMsg},
        memory_limiter::{LocalReceiverAdmissionState, MemoryPressureState},
        receiver::ReceiverWrapper,
        testing::{receiver::TestRuntime, test_node},
    };
    use otel_arrow_dfe_otap::testing::next_ack;
    use otel_arrow_dfe_scraper::database::{CursorRow, Row};
    use otel_arrow_dfe_scraper::{CheckpointStore, DatabaseReceiver, SourceBinding};

    let prefix = matches!(case, SafeRecovery::AcknowledgedPrefix);
    let mut raw = raw_config(false, 4096);
    raw["query"]["catch_up"] = json!({"max_pages":if prefix { 3 } else { 1 }});
    if prefix {
        raw["query"]["max_rows_per_poll"] = json!(5);
        raw["query"]["max_batch_bytes"] = json!(
            4 * (Row {
                values: vec![CellValue::Int64(8)]
            }
            .normalized_size()
                + size_of::<CursorRow>() as u64)
        );
    }
    let config: SqlServerReceiverConfig = serde_json::from_value(raw).unwrap();
    let query = config.query().clone();
    let recovered = vec![
        metadata_response(),
        metadata_response(),
        rows_response(&[8, 9]),
    ];
    let (connector, expected_requests) = match case {
        SafeRecovery::DescribeTimeout => (
            FixtureConnector::new([vec![vec![], attention_done()], recovered]),
            9,
        ),
        SafeRecovery::FetchTimeout => (
            FixtureConnector::new([
                vec![
                    metadata_response(),
                    metadata_response(),
                    vec![],
                    attention_done(),
                ],
                recovered,
            ]),
            11,
        ),
        SafeRecovery::ConnectReset => {
            let mut connector = FixtureConnector::new([recovered]);
            connector
                .connect_errors
                .push_back(Error::Transport(io::ErrorKind::ConnectionReset));
            (connector, 5)
        }
        SafeRecovery::AcknowledgedReset => {
            let (_, mut connector) = uncertain_response(ResponseFault::ResetOnceDuringRows);
            connector.sessions.push_back(recovered);
            (connector, 11)
        }
        SafeRecovery::AcknowledgedPrefix => (
            FixtureConnector::new([vec![
                metadata_response(),
                metadata_response(),
                rows_response(&[8, 9, 10, 11, 12]),
                attention_done(),
                metadata_response(),
                rows_response(&[10, 11, 12]),
                metadata_response(),
                rows_response(&[12]),
            ]]),
            10,
        ),
    };
    let calls = Arc::clone(&connector.calls);
    let directory = tempfile::tempdir().unwrap();
    let store = CheckpointStore::new(
        directory.path(),
        "group",
        "pipeline",
        "recovery",
        "events",
        config.config_fingerprint().to_owned(),
    );
    let source = SourceBinding::acquire(store.clone()).unwrap();
    let previous = store
        .write(0, &Cursor::Scalar(ScalarValue::Int64(7)))
        .unwrap()
        .0;
    let receiver = DatabaseReceiver::new(
        SqlServerAdapter::new(config, connector, Box::new(Credentials)),
        query,
        source,
        Duration::from_millis(1),
        5,
        LocalReceiverAdmissionState::from_process_state(&MemoryPressureState::default()),
        None,
    );
    let runtime = TestRuntime::<OtapPdata>::new();
    let wrapper = ReceiverWrapper::local(
        SafeRecoveryProbe(receiver),
        test_node(runtime.config().name.clone()),
        Arc::new(NodeUserConfig::new_receiver_config(
            "urn:otel:receiver:sql_server_test",
        )),
        runtime.config(),
    );
    let checkpoint = store.clone();
    runtime
        .set_receiver(wrapper)
        .run_test(|_| async {})
        .run_validation_concurrent(move |mut ctx| async move {
            let expected_pages = if prefix {
                // Encoded-page accounting narrows the normalized prefix again;
                // subsequent requests must resume from the acknowledged cursor.
                vec![vec![8, 9], vec![10, 11], vec![12]]
            } else {
                vec![vec![8, 9]]
            };
            let mut committed = previous;
            for ids in expected_pages {
                let data = tokio::time::timeout(Duration::from_secs(8), ctx.recv())
                    .await
                    .expect("safe recovery emits")
                    .expect("row page");
                assert_row_ids(&data, &ids);
                assert_eq!(
                    checkpoint.read().unwrap(),
                    Some(committed.clone()),
                    "{case:?}: failed operation and unacknowledged page cannot advance progress"
                );
                let (_, ack) = next_ack(AckMsg::new(data)).unwrap();
                ctx.send_control_msg(NodeControlMsg::Ack(ack))
                    .await
                    .unwrap();
                committed.revision += 1;
                committed.cursor = Cursor::Scalar(ScalarValue::Int64(*ids.last().unwrap()));
                tokio::time::timeout(Duration::from_secs(3), async {
                    while checkpoint.read().unwrap() != Some(committed.clone()) {
                        tokio::time::sleep(Duration::from_millis(5)).await;
                    }
                })
                .await
                .expect("ACK installs exactly the expected revision and cursor");
            }
            ctx.send_control_msg(NodeControlMsg::Shutdown {
                deadline: std::time::Instant::now() + Duration::from_secs(2),
                reason: "safe recovery complete".into(),
            })
            .await
            .unwrap();
        });
    assert_eq!(
        calls.connections.load(Ordering::Relaxed),
        if prefix { 1 } else { 2 }
    );
    assert_eq!(
        calls.requests.load(Ordering::Relaxed),
        expected_requests,
        "{case:?}"
    );
    if matches!(case, SafeRecovery::AcknowledgedReset) {
        assert_eq!(calls.resets.load(Ordering::Relaxed), 1);
    }
    drop(SourceBinding::acquire(store).expect("confirmed shutdown releases ownership"));
}

/// Scenario: Initial metadata or a fetch times out, but the server acknowledges ATTENTION.
/// Guarantees: Shared retry recovers on a fresh validated connection and advances saved progress only after downstream ACK.
#[test]
fn shared_controller_recovers_acknowledged_query_timeouts() {
    for case in [SafeRecovery::DescribeTimeout, SafeRecovery::FetchTimeout] {
        run_safe_recovery(case);
    }
}

/// Scenario: Connection establishment resets, or a streaming reset receives explicit cleanup confirmation.
/// Guarantees: Only safe transport failures reconnect; retried rows start from unchanged durable progress.
#[test]
fn shared_controller_recovers_safe_transport_failures() {
    for case in [SafeRecovery::ConnectReset, SafeRecovery::AcknowledgedReset] {
        run_safe_recovery(case);
    }
}

/// Scenario: A byte-limited page is acknowledged by both server cancellation and downstream delivery.
/// Guarantees: The same logged-in session emits consecutive nonoverlapping cursor rows with freshly validated metadata.
#[test]
fn shared_controller_reuses_acknowledged_prefix_connection() {
    run_safe_recovery(SafeRecovery::AcknowledgedPrefix);
}

/// Scenario: An outer deadline interrupts cleanup of an otherwise valid byte-limited prefix.
/// Guarantees: An unconfirmed partial stream cannot yield a page or make shutdown report success.
#[tokio::test(start_paused = true)]
async fn stock_driver_prefix_cleanup_survives_outer_timeout() {
    use otel_arrow_dfe_scraper::database::{CursorRow, Row};
    let row = Row {
        values: vec![CellValue::Int64(1)],
    };
    let config = config_with_bytes(false, row.normalized_size() + size_of::<CursorRow>() as u64);
    let query = config.query().clone();
    let connector = FixtureConnector::new([vec![
        metadata_response(),
        metadata_response(),
        rows_response(&[1, 2]),
        vec![],
    ]]);
    let calls = Arc::clone(&connector.calls);
    let mut adapter = SqlServerAdapter::new(config, connector, Box::new(Credentials));
    _ = adapter.begin_operation().unwrap();
    _ = adapter.validate_query(&query).await.unwrap();
    _ = adapter.begin_operation().unwrap();
    assert!(
        tokio::time::timeout(
            Duration::from_millis(100),
            adapter.execute(&query, &query.watermark().initial()),
        )
        .await
        .is_err()
    );
    assert_eq!(calls.requests.load(Ordering::Relaxed), 6);
    assert!(adapter.begin_operation().is_err());
    assert_eq!(adapter.shutdown().await, Err(Error::Driver));
}

/// Scenario: Three rows fit the byte budget exactly but doubling row capacity would exceed it.
/// Guarantees: Snapshots and incremental pages retain all fitting rows without unnecessary cancellation.
#[tokio::test]
async fn stock_driver_fits_exact_capacity_budget() {
    use otel_arrow_dfe_scraper::database::{CursorRow, Row};
    let row = Row {
        values: vec![CellValue::Int64(1)],
    };
    let budget = 3 * (row.normalized_size() + size_of::<CursorRow>() as u64);
    for snapshot in [true, false] {
        let mut raw = raw_config(snapshot, budget);
        raw["query"]["max_rows_per_poll"] = json!(4);
        let config: SqlServerReceiverConfig = serde_json::from_value(raw).unwrap();
        let query = config.query().clone();
        let connector = FixtureConnector::new([vec![
            metadata_response(),
            metadata_response(),
            rows_response(&[1, 2, 3]),
        ]]);
        let calls = Arc::clone(&connector.calls);
        let mut adapter = SqlServerAdapter::new(config, connector, Box::new(Credentials));
        _ = adapter.begin_operation().unwrap();
        _ = adapter.validate_query(&query).await.unwrap();
        _ = adapter.begin_operation().unwrap();
        let page = adapter
            .execute(&query, &query.watermark().initial())
            .await
            .unwrap();
        assert_eq!(page.rows.len(), 3);
        assert_eq!(
            calls.requests.load(Ordering::Relaxed),
            5,
            "no ATTENTION needed"
        );
        adapter.shutdown().await.unwrap();
    }
}

/// Scenario: The minimum capacity for three rows exceeds the normalized byte budget by one byte.
/// Guarantees: Snapshots still fail wholly and incremental pages cancel before returning a fitting prefix.
#[tokio::test]
async fn stock_driver_minimum_capacity_preserves_byte_limit() {
    use otel_arrow_dfe_scraper::database::{CursorRow, Row};
    let row = Row {
        values: vec![CellValue::Int64(1)],
    };
    let budget = 3 * (row.normalized_size() + size_of::<CursorRow>() as u64) - 1;
    for snapshot in [true, false] {
        let mut raw = raw_config(snapshot, budget);
        raw["query"]["max_rows_per_poll"] = json!(4);
        let config: SqlServerReceiverConfig = serde_json::from_value(raw).unwrap();
        let query = config.query().clone();
        let connector = FixtureConnector::new([vec![
            metadata_response(),
            metadata_response(),
            rows_response(&[1, 2, 3]),
            attention_done(),
        ]]);
        let calls = Arc::clone(&connector.calls);
        let mut adapter = SqlServerAdapter::new(config, connector, Box::new(Credentials));
        _ = adapter.begin_operation().unwrap();
        _ = adapter.validate_query(&query).await.unwrap();
        _ = adapter.begin_operation().unwrap();
        let result = adapter.execute(&query, &query.watermark().initial()).await;
        if snapshot {
            assert_eq!(result, Err(Error::SnapshotLimit));
        } else {
            let page = result.unwrap();
            assert_eq!(page.rows.len(), 2);
            assert_eq!(page.rows[1].cursor, Cursor::Scalar(ScalarValue::Int64(2)));
        }
        assert_eq!(
            calls.requests.load(Ordering::Relaxed),
            6,
            "ATTENTION confirmed"
        );
        adapter.shutdown().await.unwrap();
    }
}

fn error_response(code: u32, message: &str) -> Vec<u8> {
    let mut token = Vec::new();
    token.extend(code.to_le_bytes());
    token.extend([1, 14]);
    token.extend(
        u16::try_from(message.encode_utf16().count())
            .unwrap()
            .to_le_bytes(),
    );
    for c in message.encode_utf16() {
        token.extend(c.to_le_bytes());
    }
    token.extend([0, 0]); // Empty server and procedure names.
    token.extend(1u32.to_le_bytes());
    let mut body = vec![0xaa];
    body.extend(u16::try_from(token.len()).unwrap().to_le_bytes());
    body.extend(token);
    body.extend(done());
    packet(body)
}

/// Scenario: A server error arrives either with response EOF or before a missing final TDS packet.
/// Guarantees: Published-driver errors clear response_pending only at EOF; dropping an unfinished response retains quarantine state.
#[tokio::test(start_paused = true)]
async fn published_server_errors_are_deferred_until_response_eof() {
    for complete in [true, false] {
        let mut response = error_response(229, "SENSITIVE deferred error");
        if !complete {
            response[1] = 0;
        }
        let config = config(false);
        let query = config.query().clone();
        let connector = FixtureConnector::new([vec![response]]);
        let calls = Arc::clone(&connector.calls);
        let mut adapter = SqlServerAdapter::new(config, connector, Box::new(Credentials));
        _ = adapter.begin_operation().unwrap();
        let result =
            tokio::time::timeout(Duration::from_millis(100), adapter.validate_query(&query)).await;
        if complete {
            assert_eq!(result.unwrap(), Err(Error::Server(229)));
            adapter.shutdown().await.unwrap();
        } else {
            assert!(
                result.is_err(),
                "no Server error is returned before response EOF"
            );
            assert!(adapter.begin_operation().is_err());
            assert_eq!(adapter.shutdown().await, Err(Error::Driver));
        }
        assert_eq!(calls.requests.load(Ordering::Relaxed), 3);
    }
}

/// Scenario: Stock Tiberius emits an ERROR containing server-supplied sensitive text.
/// Guarantees: Driver tracing cannot bypass redaction; the caller receives only a safe code.
#[tokio::test]
async fn stock_driver_tracing_cannot_leak_server_messages() {
    use tracing::instrument::WithSubscriber;
    // The test subscriber requires Send + Sync; production adapter state stays local.
    #[derive(Clone)]
    struct Writer(Arc<std::sync::Mutex<Vec<u8>>>);
    impl io::Write for Writer {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for Writer {
        type Writer = Self;
        fn make_writer(&'a self) -> Self {
            self.clone()
        }
    }
    let output = Writer(Default::default());
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_ansi(false)
        .with_writer(output.clone())
        .finish();
    let (mut adapter, query) = adapter(false, vec![error_response(229, "SENSITIVE-server-data")]);
    _ = adapter.begin_operation().unwrap();
    let result = adapter
        .validate_query(&query)
        .with_subscriber(subscriber)
        .await;
    assert_eq!(result, Err(Error::Server(229)));
    let captured = String::from_utf8(output.0.lock().unwrap().clone()).unwrap();
    assert!(!captured.contains("SENSITIVE-server-data"));
    adapter.shutdown().await.unwrap();
}

/// Scenario: SQL Server TLS is omitted, enabled, or explicitly disabled for testing.
/// Guarantees: Driver policy uses Required or NotSupported, never login-only Off.
#[test]
fn sql_server_tls_policy_selects_exact_driver_modes() {
    use super::adapter::{configure_tls, encryption_level};
    for (tls, enabled, name) in [
        (json!({}), true, "sql.example.test"),
        (
            json!({"server_name_override":"cert.example.test"}),
            true,
            "cert.example.test",
        ),
        (
            json!({"trust_server_certificate":true}),
            true,
            "sql.example.test",
        ),
        (json!({"enabled":false}), false, "sql.example.test"),
    ] {
        let endpoint: SqlServerConnectionConfig = serde_json::from_value(json!({
            "host":"sql.example.test", "database":"fixture", "tls":tls
        }))
        .unwrap();
        configure_tls(&endpoint, &mut tiberius::Config::new()).unwrap();
        assert_eq!(endpoint.host(), "sql.example.test");
        assert_eq!(
            endpoint.server_name_override().unwrap_or(endpoint.host()),
            name
        );
        assert_eq!(
            encryption_level(endpoint.tls_enabled()),
            if enabled {
                tiberius::EncryptionLevel::Required
            } else {
                tiberius::EncryptionLevel::NotSupported
            },
        );
    }
}

/// Scenario: A direct connector caller tries to deserialize contradictory TLS options.
/// Guarantees: Validated construction rejects the options before any driver can receive them.
#[test]
fn programmatic_tls_conflicts_fail_at_configuration_boundary() {
    for tls in [
        json!({"ca_file":"SENSITIVE","trust_server_certificate":true}),
        json!({"server_name_override":"SENSITIVE.example","trust_server_certificate":true}),
        json!({"enabled":false,"ca_file":"SENSITIVE"}),
        json!({"enabled":false,"server_name_override":"SENSITIVE"}),
        json!({"enabled":false,"trust_server_certificate":true}),
    ] {
        let error = serde_json::from_value::<SqlServerConnectionConfig>(json!({
            "host":"localhost", "database":"fixture", "tls":tls
        }))
        .unwrap_err();
        assert!(!error.to_string().contains("SENSITIVE"));
    }
}

fn database_invalid_queries() -> [(&'static str, u32); 4] {
    [
        ("SELECT ID FROM dbo.events WHERE COUNT(*) > 0", 147),
        ("SELECT ID FROM dbo.events WHERE ID", 4145),
        ("SELECT ID FROM dbo.missing", 208),
        ("SELECT ID FROM dbo.denied", 229),
    ]
}

/// Scenario: Locally accepted expressions, missing objects, and denied access fail server validation.
/// Guarantees: Driver validation returns a permanent redacted failure and cannot execute an unvalidated plan.
#[tokio::test]
async fn database_rejection_prevents_execution() {
    for (statement, code) in database_invalid_queries() {
        let mut raw = raw_config(true, 4096);
        raw["query"]["statement"] = json!(statement);
        let config: SqlServerReceiverConfig = serde_json::from_value(raw).unwrap();
        let query = config.query().clone();
        let connector =
            FixtureConnector::new([vec![error_response(code, "SENSITIVE query or object")]]);
        let calls = Arc::clone(&connector.calls);
        let mut adapter = SqlServerAdapter::new(config, connector, Box::new(Credentials));
        _ = adapter.begin_operation().unwrap();
        let error = adapter.validate_query(&query).await.unwrap_err();
        assert_eq!(error, Error::Server(code));
        assert!(!SqlServerAdapter::<FixtureConnector>::is_retryable(&error));
        assert!(!format!("{error} {error:?}").contains("SENSITIVE"));
        _ = adapter.begin_operation().unwrap();
        assert_eq!(
            adapter.execute(&query, &Cursor::Snapshot).await,
            Err(Error::NotValidated)
        );
        assert_eq!(calls.connections.load(Ordering::Relaxed), 1);
        assert_eq!(
            calls.requests.load(Ordering::Relaxed),
            3,
            "only prelogin, login, describe"
        );
        adapter.shutdown().await.unwrap();
    }
}

struct ValidationFailureProbe {
    receiver: otel_arrow_dfe_scraper::DatabaseReceiver<SqlServerAdapter<FixtureConnector>>,
    expected: Error,
}

#[async_trait(?Send)]
impl local::Receiver<OtapPdata> for ValidationFailureProbe {
    async fn start(
        self: Box<Self>,
        controls: local::ControlChannel<OtapPdata>,
        effects: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, EngineError> {
        let result = tokio::time::timeout(
            Duration::from_secs(5),
            local::Receiver::start(Box::new(self.receiver), controls, effects),
        )
        .await
        .expect("permanent validation failure must terminate without a retry loop");
        let Err(error) = result else {
            panic!("database validation must fail");
        };
        assert!(matches!(error, EngineError::ReceiverError { .. }));
        assert!(
            error.to_string().contains(&self.expected.to_string()),
            "{error}"
        );
        assert!(!error.to_string().contains("SENSITIVE"));
        Ok(TerminalState::default())
    }
}

fn run_validation_failure(
    config: SqlServerReceiverConfig,
    rejection: Vec<Vec<u8>>,
    expected: Error,
    after_reconnect: bool,
) {
    use otel_arrow_dfe_config::node::NodeUserConfig;
    use otel_arrow_dfe_engine::{
        memory_limiter::{LocalReceiverAdmissionState, MemoryPressureState},
        receiver::ReceiverWrapper,
        testing::{receiver::TestRuntime, test_node},
    };
    use otel_arrow_dfe_otap::pdata::OtapPdata;
    use otel_arrow_dfe_scraper::{CheckpointStore, DatabaseReceiver, SourceBinding};

    let directory = tempfile::tempdir().unwrap();
    let query = config.query().clone();
    let store = CheckpointStore::new(
        directory.path(),
        "group",
        "pipeline",
        "validation",
        "events",
        config.config_fingerprint().to_owned(),
    );
    let source = SourceBinding::acquire(store.clone()).unwrap();
    let previous = if matches!(query.watermark().initial(), Cursor::Snapshot) {
        None
    } else {
        Some(
            store
                .write(0, &Cursor::Scalar(ScalarValue::Int64(7)))
                .unwrap()
                .0,
        )
    };
    let mut sessions = Vec::new();
    if after_reconnect {
        sessions.push(vec![
            metadata_response(),
            metadata_response(),
            error_response(1205, "transient fixture deadlock"),
        ]);
    }
    let expected_requests = if after_reconnect { 7 } else { 2 } + rejection.len();
    sessions.push(rejection);
    let connector = FixtureConnector::new(sessions);
    let calls = Arc::clone(&connector.calls);
    let receiver = DatabaseReceiver::new(
        SqlServerAdapter::new(config, connector, Box::new(Credentials)),
        query,
        source,
        Duration::from_millis(1),
        5,
        LocalReceiverAdmissionState::from_process_state(&MemoryPressureState::default()),
        None,
    );
    let runtime = TestRuntime::<OtapPdata>::new();
    let wrapper = ReceiverWrapper::local(
        ValidationFailureProbe { receiver, expected },
        test_node(runtime.config().name.clone()),
        Arc::new(NodeUserConfig::new_receiver_config(
            "urn:otel:receiver:sql_server_test",
        )),
        runtime.config(),
    );
    runtime
        .set_receiver(wrapper)
        .run_test(|_| async {})
        .run_validation_concurrent(|mut ctx| async move {
            assert!(
                tokio::time::timeout(Duration::from_secs(6), ctx.recv())
                    .await
                    .unwrap()
                    .is_err(),
                "failed validation must never emit a page",
            );
        });
    assert_eq!(
        store.read().unwrap(),
        previous,
        "saved progress must remain unchanged"
    );
    assert_eq!(
        calls.connections.load(Ordering::Relaxed),
        if after_reconnect { 2 } else { 1 }
    );
    assert_eq!(
        calls.requests.load(Ordering::Relaxed),
        expected_requests,
        "no fetch or extra reconnect after failed validation",
    );
    drop(SourceBinding::acquire(store).expect("confirmed cleanup releases ownership"));
}

/// Scenario: SQL expressions, object lookup, or permissions fail before initial polling.
/// Guarantees: The shared receiver terminates without emitting, checkpointing, or retrying permanent failures.
#[test]
fn shared_controller_stops_on_initial_database_validation_errors() {
    for (statement, code) in database_invalid_queries() {
        let mut raw = raw_config(true, 4096);
        raw["query"]["statement"] = json!(statement);
        run_validation_failure(
            serde_json::from_value(raw).unwrap(),
            vec![error_response(code, "SENSITIVE database validation")],
            Error::Server(code),
            false,
        );
    }
}

/// Scenario: A transient query failure reconnects, then SQL or permission validation fails.
/// Guarantees: Revalidation precedes another fetch; permanent failure emits nothing and ends recovery.
#[test]
fn shared_controller_revalidates_database_after_reconnect() {
    for code in [147, 4145, 208, 229] {
        run_validation_failure(
            config(false),
            vec![error_response(code, "SENSITIVE changed database")],
            Error::Server(code),
            true,
        );
    }
}

/// Scenario: Initial or reconnect metadata reports an error or makes the cursor nullable.
/// Guarantees: Incompatible plans never fetch rows or change an existing checkpoint.
#[test]
fn shared_controller_rejects_invalid_metadata_before_polling() {
    for reconnect in [false, true] {
        for response in [
            vec![metadata_response_with(true, None)],
            vec![metadata_response_with(false, Some(11509)), attention_done()],
        ] {
            run_validation_failure(config(false), response, Error::Metadata, reconnect);
        }
    }
}
