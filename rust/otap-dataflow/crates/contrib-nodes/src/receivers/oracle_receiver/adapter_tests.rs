// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::{
    CellValue, OracleAdapterError, OracleType, bounded_connect_string, cursor_bind_type,
    finite_float, parse_cursor_timestamp, read_credential, validate_described_cursor_columns,
    validate_types,
};
use oracle::sql_type::Timestamp;
use otel_arrow_dfe_scraper::database::{CompositeCursor, CompositeWatermark};
use secrecy::ExposeSecret;
use std::fs;
use std::str::FromStr;
use std::time::Duration;

/// Scenario: A native Oracle failure contains sentinel SQL, row, endpoint and nested-source text.
/// Guarantees: Adapter formatting and the complete engine diagnostic expose no native text.
#[test]
fn native_error_text_is_redacted_from_engine_diagnostics() {
    use otel_arrow_dfe_engine::error::{Error, error_summary_from, format_error_sources};
    use otel_arrow_dfe_scraper::database::DriverAdapter;
    const SENTINEL: &str = "secret-row SELECT-private endpoint-private checkpoint-private";
    let constructors: [fn(oracle::Error) -> OracleAdapterError; 8] = [
        OracleAdapterError::Initialize,
        OracleAdapterError::Connect,
        OracleAdapterError::Configure,
        OracleAdapterError::Prepare,
        OracleAdapterError::Query,
        OracleAdapterError::Fetch,
        OracleAdapterError::Convert,
        OracleAdapterError::Cancellation,
    ];
    for constructor in constructors {
        let native = oracle::Error::with_source(
            oracle::ErrorKind::InvalidOperation,
            std::io::Error::other(SENTINEL),
        );
        let error = constructor(native);
        let source_detail = format_error_sources(&error);
        assert!(source_detail.is_empty());
        assert!(!format!("{error:?}").contains(SENTINEL));
        let engine = Error::ReceiverError {
            receiver: otel_arrow_dfe_engine::testing::test_node("oracle-test"),
            kind: super::OracleAdapter::classify_error(&error),
            error: error.to_string(),
            source_detail,
        };
        for rendered in [
            engine.to_string(),
            format!("{engine:?}"),
            serde_json::to_string(&error_summary_from(&engine)).expect("diagnostic JSON"),
        ] {
            assert!(!rendered.contains(SENTINEL), "{rendered}");
            assert!(!rendered.contains("endpoint-private"), "{rendered}");
        }
    }
}

fn test_adapter() -> super::OracleAdapter {
    super::OracleAdapter::new(super::OracleAdapterConfig {
        connect_string: String::new(),
        instant_client_dir: String::new(),
        username_file: String::new(),
        password_file: String::new(),
    })
}

/// Scenario: Oracle errors include known outages alongside authorization, SQL and schema failures.
/// Guarantees: Only the explicit availability allowlist is retryable; unrelated native failures stay terminal.
#[test]
fn recovery_codes_are_allowlisted() {
    for code in [
        28, 1012, 1033, 1034, 1089, 1090, 3113, 3114, 3135, 12170, 12514, 12516, 12519, 12520,
        12528, 12537, 12541, 12543, 12547, 12571,
    ] {
        assert!(super::transient_native_error(Some(code), None), "{code}");
    }
    for code in [
        1, 600, 900, 942, 1017, 1031, 1466, 1722, 12154, 12545, 28000, 28001, 29024,
    ] {
        assert!(!super::transient_native_error(Some(code), None), "{code}");
    }
    for code in [1010, 1067, 1080] {
        assert!(super::transient_native_error(None, Some(code)));
    }
    assert!(!super::transient_native_error(None, Some(1047)));
    assert!(!super::transient_native_error(None, None));
}

/// Scenario: Cancellation from a finished attempt arrives after recovery starts another attempt.
/// Guarantees: Old handles neither cancel nor clear the new operation, while the current handle still cancels.
#[tokio::test]
async fn stale_cancellation_cannot_interrupt_reconnect() {
    use otel_arrow_dfe_scraper::database::{DriverAdapter, DriverCancellation};
    let mut adapter = test_adapter();
    let previous = adapter.begin_operation().expect("first");
    let current = adapter.begin_operation().expect("next");
    previous.cancel().await.expect("stale cancel ignored");
    current
        .ensure_not_requested()
        .expect("new operation remains active");
    assert!(matches!(
        previous.ensure_not_requested(),
        Err(OracleAdapterError::Cancelled)
    ));
    current.cancel().await.expect("current cancellation");
    assert!(matches!(
        current.ensure_not_requested(),
        Err(OracleAdapterError::Cancelled)
    ));
    adapter.shutdown().await.expect("cleanup");
}

/// Scenario: A previous operation's cancellation is queued behind a blocked native cancellation worker.
/// Guarantees: Another operation cannot start until that cancellation job has finished.
#[tokio::test]
async fn pending_native_cancel_blocks_new_operations() {
    use otel_arrow_dfe_scraper::database::{DriverAdapter, DriverCancellation};
    let mut adapter = test_adapter();
    let cancellation = adapter.begin_operation().expect("operation");
    let worker = super::NativeWorker::<()>::new("oracle-cancel-race-test").expect("worker");
    let (release, gate) = std::sync::mpsc::sync_channel(1);
    let (started, ready) = tokio::sync::oneshot::channel();
    let _blocked = worker
        .run(move |_| {
            let _ = started.send(());
            gate.recv().expect("release");
        })
        .expect("blocked worker");
    super::receive(ready).await.expect("started");
    {
        let mut state = cancellation.state.lock().expect("state");
        state.worker = Some(worker);
        state.connection = Some(std::sync::Weak::new());
    }
    let cancel = cancellation.cancel();
    tokio::pin!(cancel);
    assert!(futures::poll!(&mut cancel).is_pending());
    assert!(matches!(
        adapter.begin_operation(),
        Err(OracleAdapterError::CancellationState)
    ));
    release.send(()).expect("release");
    cancel.await.expect("cancel completed");
    _ = adapter.begin_operation().expect("next operation can start");
    adapter.shutdown().await.expect("cleanup");
}

/// Scenario: Reconnect is cancelled while disconnected, before credentials or native initialization run.
/// Guarantees: Stop is honored without opening a session or retrying inside the adapter.
#[tokio::test]
async fn cancelled_reconnect_does_not_open_a_session() {
    use otel_arrow_dfe_scraper::database::{DriverAdapter, DriverCancellation};
    let mut adapter = test_adapter();
    let cancellation = adapter.begin_operation().expect("operation");
    cancellation.cancel().await.expect("cancel");
    let config: super::super::OracleReceiverConfig =
        serde_json::from_value(super::super::tests::documented_config()).expect("config");
    let query = config.query();
    assert!(matches!(
        adapter.reconnect(&query).await,
        Err(OracleAdapterError::Cancelled)
    ));
    assert!(adapter.worker.is_none());
    adapter.shutdown().await.expect("cleanup");
}

/// Scenario: Shutdown's future is interrupted while a worker still owns an unfinished native operation.
/// Guarantees: Repeated shutdown cannot release ownership before the same retained worker has exited.
#[tokio::test]
async fn interrupted_shutdown_retains_worker_completion() {
    use otel_arrow_dfe_scraper::database::DriverAdapter;
    let mut adapter = test_adapter();
    adapter.worker = Some(super::NativeWorker::new("oracle-stop-resume-test").expect("worker"));
    let (release, gate) = std::sync::mpsc::sync_channel(1);
    let (started, ready) = tokio::sync::oneshot::channel();
    let _work = adapter
        .worker
        .as_ref()
        .expect("worker")
        .run(move |_| {
            let _ = started.send(());
            gate.recv().expect("release");
        })
        .expect("operation");
    super::receive(ready).await.expect("started");
    assert!(
        tokio::time::timeout(Duration::from_millis(20), adapter.shutdown())
            .await
            .is_err()
    );
    assert!(adapter.worker.is_some());
    assert!(adapter.begin_operation().is_err());
    release.send(()).expect("release");
    tokio::time::timeout(Duration::from_secs(2), adapter.shutdown())
        .await
        .expect("bounded")
        .expect("joined");
    adapter.shutdown().await.expect("idempotent");
}

/// Scenario: A disposable Oracle source widens its cursor timestamp between two adapter executions.
/// Guarantees: The database or adapter rejects incompatible DDL before returning another page.
#[tokio::test]
#[ignore = "requires ORACLE_SCHEMA_TEST_CONFIG pointing to a disposable OTAP_SCHEMA_DRIFT fixture"]
async fn live_timestamp_precision_change_is_rejected() -> Result<(), Box<dyn std::error::Error>> {
    use otel_arrow_dfe_scraper::database::DriverAdapter;

    let path = std::env::var("ORACLE_SCHEMA_TEST_CONFIG")?;
    let value: serde_json::Value = serde_json::from_slice(&fs::read(path)?)?;
    let config: super::super::OracleReceiverConfig = serde_json::from_value(value.clone())?;
    let query = config.query();
    let mut adapter = config.adapter();
    let result = async {
        _ = adapter.begin_operation()?;
        _ = adapter.validate_query(&query).await?;
        _ = adapter.begin_operation()?;
        let page = adapter.execute(&query, &query.watermark().initial).await?;
        let cursor = page.rows.last().ok_or("fixture must contain one row")?.cursor.clone();
        let username = read_credential(
            value["authentication"]["username_file"].as_str().ok_or("username file")?,
            "username",
        )?;
        let password = read_credential(
            value["authentication"]["password_file"].as_str().ok_or("password file")?,
            "password",
        )?;
        let writer = oracle::Connection::connect(
            username.expose_secret(),
            password.expose_secret(),
            value["connection"]["connect_string"].as_str().ok_or("connect string")?,
        )?;
        _ = writer.execute(
            "ALTER TABLE OTAP_SCHEMA_DRIFT MODIFY EVENT_TS TIMESTAMP(9)",
            &[],
        )?;
        _ = writer.execute(
            "INSERT INTO OTAP_SCHEMA_DRIFT VALUES (2, TIMESTAMP '2026-01-01 00:00:00.123456001', 'drift')",
            &[],
        )?;
        writer.commit()?;
        _ = adapter.begin_operation()?;
        match adapter.execute(&query, &cursor).await {
            Err(OracleAdapterError::ResultMetadataChanged) => Ok(()),
            Err(OracleAdapterError::Query(error))
                if error.db_error().is_some_and(|error| error.code() == 1466) => Ok(()),
            Err(error) => Err(Box::new(error) as Box<dyn std::error::Error>),
            Ok(_) => Err("schema change was not rejected before returning a page".into()),
        }

    }.await;
    adapter.shutdown().await?;
    result
}

fn page_row(id: i64, bytes: usize) -> otel_arrow_dfe_scraper::database::CursorRow {
    otel_arrow_dfe_scraper::database::CursorRow {
        row: otel_arrow_dfe_scraper::database::Row {
            values: vec![CellValue::String("x".repeat(bytes))],
        },
        cursor: CompositeCursor::new("2026-01-01 00:00:00".to_owned(), id),
    }
}

/// Scenario: A large page appends many small rows within a generous byte budget.
/// Guarantees: Growth is logarithmic rather than one allocation request per row, with spare slots accounted.
#[test]
fn page_capacity_grows_geometrically_within_limits() {
    let mut rows = Vec::new();
    let mut payload = 0;
    let mut growths = 0;
    for id in 1..=1000 {
        let previous = rows.capacity();
        assert!(
            super::push_bounded_row(&mut rows, &mut payload, page_row(id, 16), 1000, 1024 * 1024)
                .expect("fits")
        );
        growths += usize::from(rows.capacity() != previous);
    }
    assert_eq!(rows.len(), 1000);
    assert!(rows.capacity() <= 1000);
    assert!(growths <= 10, "{growths} growth requests for 1000 rows");
    let measured = rows
        .iter()
        .map(|row| {
            row.row.normalized_size() - size_of::<otel_arrow_dfe_scraper::database::Row>() as u64
                + row.cursor.timestamp.capacity() as u64
        })
        .sum::<u64>();
    assert_eq!(payload, measured);
}

/// Scenario: Tight byte limits admit a prefix but reject an oversized first row.
/// Guarantees: Header, spare slots and payload stay bounded and deferred rows do not alter the candidate cursor.
#[test]
fn page_growth_preserves_byte_bound_and_prefix_cursor() {
    use otel_arrow_dfe_scraper::database::{CursorRow, Row};
    for limit in [1u64, 64, 256, 512, 1024, 4096] {
        let mut rows = Vec::new();
        let mut payload = 0;
        for id in 1..=100 {
            let row = page_row(id, 37);
            let before = rows.last().map(|row: &CursorRow| row.cursor.clone());
            match super::push_bounded_row(&mut rows, &mut payload, row, 100, limit) {
                Ok(true) => {
                    let actual = size_of::<Vec<CursorRow>>() as u64
                        + (rows.capacity() * size_of::<CursorRow>()) as u64
                        + rows
                            .iter()
                            .map(|row| {
                                row.row.normalized_size() - size_of::<Row>() as u64
                                    + row.cursor.timestamp.capacity() as u64
                            })
                            .sum::<u64>();
                    assert!(actual <= limit, "{actual} exceeds {limit}");
                }
                Ok(false) => {
                    assert_eq!(rows.last().map(|row| row.cursor.clone()), before);
                    break;
                }
                Err(OracleAdapterError::NormalizedByteLimit { .. }) => {
                    assert!(rows.is_empty());
                    break;
                }
                Err(error) => panic!("unexpected error: {error}"),
            }
        }
    }
}
/// Scenario: An Oracle worker is idle after successfully completing an operation.
/// Guarantees: Adapter shutdown closes its work channel and confirms worker cleanup before returning.
#[tokio::test]
async fn shutdown_joins_worker_after_successful_operation() {
    let mut adapter = test_adapter();
    adapter.worker = Some(super::NativeWorker::new("oracle-adapter-test").expect("worker"));
    let operation = adapter
        .worker
        .as_ref()
        .expect("worker")
        .run(|_| std::thread::current().id())
        .expect("accepted operation");
    let worker_thread = super::receive(operation)
        .await
        .expect("successful operation");
    assert_ne!(worker_thread, std::thread::current().id());
    otel_arrow_dfe_scraper::database::DriverAdapter::shutdown(&mut adapter)
        .await
        .expect("shutdown");
    assert!(adapter.worker.is_none());
    assert!(adapter.cancellation.state.lock().expect("state").stopped);
}

/// Scenario: Native cancellation is still running after the query worker becomes idle.
/// Guarantees: Adapter shutdown does not confirm cleanup before the cancellation worker finishes.
#[tokio::test]
async fn shutdown_waits_for_cancellation_worker() {
    let mut adapter = test_adapter();
    adapter.worker = Some(super::NativeWorker::new("oracle-query-test").expect("worker"));
    let cancellation = super::NativeWorker::new("oracle-cancel-test").expect("worker");
    let (release, gate) = std::sync::mpsc::sync_channel(1);
    let (started, ready) = tokio::sync::oneshot::channel();
    let _operation = cancellation
        .run(move |_| {
            let _ = started.send(());
            gate.recv().expect("released");
        })
        .expect("cancel job");
    adapter.cancellation.state.lock().expect("state").worker = Some(cancellation);
    super::receive(ready).await.expect("cancellation started");
    let shutdown = otel_arrow_dfe_scraper::database::DriverAdapter::shutdown(&mut adapter);
    tokio::pin!(shutdown);
    assert!(
        tokio::time::timeout(Duration::from_millis(20), &mut shutdown)
            .await
            .is_err()
    );
    release.send(()).expect("release cancellation");
    tokio::time::timeout(Duration::from_secs(2), shutdown)
        .await
        .expect("cleanup completes")
        .expect("shutdown");
}

/// Scenario: Cancellation arrives before any connection exists, then shutdown closes the adapter.
/// Guarantees: The operation is cancelled without spawning native work and cannot restart after shutdown.
#[tokio::test]
async fn cancellation_before_connect_stops_admission() {
    use otel_arrow_dfe_scraper::database::{DriverAdapter, DriverCancellation};
    let mut adapter = test_adapter();
    let cancellation = adapter.begin_operation().expect("operation");
    cancellation.cancel().await.expect("no active connection");
    assert!(matches!(
        cancellation.ensure_not_requested(),
        Err(OracleAdapterError::Cancelled)
    ));
    adapter.shutdown().await.expect("shutdown");
    assert!(matches!(
        adapter.begin_operation(),
        Err(OracleAdapterError::Cancelled)
    ));
}

/// Scenario: Cancellation is requested during a native call that otherwise succeeds.
/// Guarantees: Its result is rejected and no subsequent call begins for that operation.
#[test]
fn cancellation_is_checked_between_native_calls() {
    let cancellation = super::OracleCancellation::default();
    let result = cancellation.native_call(|| {
        cancellation.state.lock().expect("state").requested = true;
        Ok(())
    });
    assert!(matches!(result, Err(OracleAdapterError::Cancelled)));
    assert!(matches!(
        cancellation.native_call::<()>(|| panic!("cancelled operation must not run")),
        Err(OracleAdapterError::Cancelled)
    ));
}

fn watermark() -> CompositeWatermark {
    CompositeWatermark {
        timestamp_column: "EVENT_TS".to_owned(),
        timestamp_bind: "last_timestamp".to_owned(),
        tie_breaker_column: "EVENT_ID".to_owned(),
        tie_breaker_bind: "last_tie_breaker".to_owned(),
        initial: CompositeCursor::new("1970-01-01 00:00:00".to_owned(), 0),
    }
}

fn columns(timestamp: OracleType, tie_breaker: OracleType) -> Vec<(String, OracleType)> {
    vec![
        ("PAYLOAD".to_owned(), OracleType::Varchar2(64)),
        ("EVENT_TS".to_owned(), timestamp),
        ("EVENT_ID".to_owned(), tie_breaker),
    ]
}

/// Scenario: live metadata reports supported cursor types under differing identifier case.
/// Guarantees: both cursor columns are resolved to their result positions, so the receiver reads
/// each row's cursor from the correct columns regardless of driver quoting behavior.
#[test]
fn resolves_supported_cursor_columns_case_insensitively() {
    let mut described = columns(OracleType::Timestamp(6), OracleType::Number(18, 0));
    described[1].0 = "event_ts".to_owned();

    let (timestamp_index, tie_breaker_index) =
        validate_described_cursor_columns(&described, &watermark()).expect("cursor columns");

    assert_eq!(timestamp_index, 1);
    assert_eq!(tie_breaker_index, 2);
}

/// Scenario: every supported Oracle DATE and TIMESTAMP family type is used as the cursor.
/// Guarantees: the documented supported timestamp types are all accepted, so a valid deployment
/// is not rejected because of a timestamp precision or timezone variant.
#[test]
fn accepts_the_supported_oracle_timestamp_family() {
    for timestamp in [
        OracleType::Date,
        OracleType::Timestamp(0),
        OracleType::Timestamp(9),
        OracleType::TimestampTZ(6),
        OracleType::TimestampLTZ(6),
    ] {
        assert!(
            validate_described_cursor_columns(
                &columns(timestamp.clone(), OracleType::Int64),
                &watermark(),
            )
            .is_ok(),
            "timestamp type '{timestamp}' must be supported"
        );
    }
}

/// Scenario: a cursor column has a non-temporal, fractional, or otherwise non-deterministic type.
/// Guarantees: unsupported cursor metadata fails before polling, so the receiver never paginates
/// on a column whose ordering or checkpoint round trip is not exact.
#[test]
fn rejects_unsupported_cursor_column_types() {
    for kind in [
        OracleType::UInt64,
        OracleType::Number(38, 0),
        OracleType::Number(19, 0),
    ] {
        assert!(
            validate_described_cursor_columns(
                &columns(OracleType::Timestamp(6), kind),
                &watermark(),
            )
            .is_err()
        );
    }
    assert!(matches!(
        validate_described_cursor_columns(
            &columns(OracleType::Varchar2(32), OracleType::Int64),
            &watermark(),
        ),
        Err(OracleAdapterError::UnsupportedCursorTimestamp { .. })
    ));
    assert!(matches!(
        validate_described_cursor_columns(
            &columns(OracleType::Timestamp(6), OracleType::Number(38, 2)),
            &watermark(),
        ),
        Err(OracleAdapterError::UnsupportedCursorTieBreaker { .. })
    ));
    assert!(matches!(
        validate_described_cursor_columns(
            &columns(OracleType::Timestamp(6), OracleType::BinaryDouble),
            &watermark(),
        ),
        Err(OracleAdapterError::UnsupportedCursorTieBreaker { .. })
    ));
}

/// Scenario: a configured cursor column is absent from the query's result metadata.
/// Guarantees: a mismatch between the statement and the cursor configuration fails at startup
/// rather than at the first row fetch.
#[test]
fn rejects_missing_cursor_columns() {
    let described = vec![("PAYLOAD".to_owned(), OracleType::Varchar2(64))];

    assert!(matches!(
        validate_described_cursor_columns(&described, &watermark()),
        Err(OracleAdapterError::MissingCursorColumn(column)) if column == "EVENT_TS"
    ));
}

/// Scenario: a committed cursor timestamp is bound back into Oracle after a restart.
/// Guarantees: the checkpointed text round-trips through the Oracle timestamp type without
/// losing sub-second precision, so replay resumes at the exact committed boundary.
#[test]
fn cursor_timestamp_round_trips_through_oracle() {
    let committed = Timestamp::from_str("2026-01-01 12:34:56.123456789")
        .expect("committed timestamp should parse")
        .to_string();

    let rebound = Timestamp::from_str(&committed).expect("committed text should rebind");

    assert_eq!(rebound.to_string(), committed);
    assert_eq!(rebound.nanosecond(), 123_456_789);
}

/// Scenario: a committed timezone-aware cursor is rebound after restart.
/// Guarantees: the bind type retains the cursor's UTC offset instead of coercing it to a
/// timezone-naive timestamp and moving the polling boundary.
#[test]
fn timezone_aware_cursor_uses_a_timezone_aware_bind() {
    let committed = Timestamp::from_str("2026-01-01 12:34:56.123456789 +05:30")
        .expect("timezone-aware cursor should parse");

    assert!(committed.with_tz());
    assert_eq!(committed.tz_offset(), 19_800);
    assert!(matches!(cursor_bind_type(), OracleType::TimestampTZ(9)));
}

/// Scenario: a checkpoint file holds a cursor timestamp Oracle cannot parse.
/// Guarantees: an invalid committed timestamp is reported explicitly instead of being
/// interpolated into SQL or silently reset to the initial cursor.
#[test]
fn rejects_uninterpretable_cursor_timestamps() {
    assert!(parse_cursor_timestamp("not-a-timestamp").is_err());
}

/// Scenario: Configured or checkpointed cursor text contains oversized numeric components.
/// Guarantees: The shared Oracle parsing boundary rejects overflow, narrowing, and precision loss.
#[test]
fn cursor_parser_rejects_oversized_numeric_components() {
    for text in [
        "9".repeat(40),
        "4294969322-01-01 00:00:00".to_owned(),
        "2026-01-01 00:00:00.1234567890".to_owned(),
    ] {
        assert!(matches!(
            parse_cursor_timestamp(&text),
            Err(OracleAdapterError::InvalidCursorTimestamp(_))
        ));
    }
}

/// Scenario: Oracle returns a non-finite binary floating-point value.
/// Guarantees: Driver normalization fails the batch instead of emitting invalid OTLP data.
#[test]
fn rejects_non_finite_float() {
    assert!(matches!(
        finite_float(CellValue::Float64(f64::NAN)),
        Err(OracleAdapterError::NonFiniteFloat)
    ));
}
/// Scenario: Oracle result metadata contains a vendor type without a CellValue mapping.
/// Guarantees: Metadata validation fails explicitly instead of using a lossy fallback.
#[test]
fn rejects_unsupported_vendor_type() {
    assert!(matches!(
        validate_types(&[OracleType::BLOB]),
        Err(OracleAdapterError::UnsupportedType(_))
    ));
}

/// Scenario: An Easy Connect string uses a query timeout longer than connection establishment.
/// Guarantees: Connection and transport attempts remain bounded while query calls retain their
/// independently configured timeout.
#[test]
fn adds_bounded_network_timeouts() {
    let connect_string =
        bounded_connect_string("database.contoso.com:1521/ORCL", Duration::from_secs(120))
            .expect("Easy Connect string should be supported");

    assert_eq!(
        connect_string,
        "database.contoso.com:1521/ORCL?connect_timeout=10&transport_connect_timeout=10"
    );
}

/// Scenario: An Easy Connect string adds retries or multiple database addresses.
/// Guarantees: Connection establishment cannot multiply the fixed per-attempt startup bound.
#[test]
fn rejects_unbounded_connection_attempts() {
    for connect_string in [
        "database.contoso.com:1521/ORCL?retry_count=10",
        "database.contoso.com:1521/ORCL?retry_delay=5",
        "db1.contoso.com,db2.contoso.com:1521/ORCL",
    ] {
        assert!(bounded_connect_string(connect_string, Duration::from_secs(120)).is_err());
    }
}

/// Scenario: A mounted credential contains a trailing newline.
/// Guarantees: Kubernetes-style secret files load without adding the line ending to the credential.
#[test]
fn trims_credential_line_endings() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("password");
    fs::write(&path, b"secret\r\n").expect("write credential");

    assert_eq!(
        read_credential(path.to_str().expect("UTF-8 path"), "password")
            .expect("credential should load")
            .expose_secret(),
        "secret"
    );
}

/// Scenario: A mounted credential exceeds the receiver's fixed secret-file ceiling.
/// Guarantees: The adapter rejects the file before allocating or retaining unbounded secret data.
#[test]
fn rejects_oversized_credential() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("password");
    fs::write(&path, vec![b'x'; 64 * 1024 + 1]).expect("write credential");

    assert!(matches!(
        read_credential(path.to_str().expect("UTF-8 path"), "password"),
        Err(OracleAdapterError::CredentialTooLarge("password"))
    ));
}

/// Scenario: A mounted credential is not valid UTF-8.
/// Guarantees: Invalid text is rejected without including credential bytes in diagnostics.
#[test]
fn rejects_non_utf8_credential() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("password");
    fs::write(&path, [0xff]).expect("write credential");

    assert!(matches!(
        read_credential(path.to_str().expect("UTF-8 path"), "password"),
        Err(OracleAdapterError::InvalidCredentialEncoding("password"))
    ));
}
