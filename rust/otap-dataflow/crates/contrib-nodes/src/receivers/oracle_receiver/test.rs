// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// Expand private-module tests at their original paths without exposing production internals.
macro_rules! oracle_module_tests {
    (adapter) => {
        mod tests {
            use super::{
                CellValue, OracleAdapterError, OracleType, bounded_connect_string,
                cursor_bind_timestamp, cursor_bind_type, discovery_cursor_bind_type,
                extract_normalized_cursor, finite_float, parse_cursor_timestamp,
                validate_described_cursor_columns, validate_types,
            };
            use oracle::sql_type::Timestamp;
            use otel_arrow_dfe_scraper::database::{CompositeCursor, CompositeWatermark, Row};
            use std::fs;
            use std::str::FromStr;
            use std::time::Duration;

            fn assert_redacted(error: OracleAdapterError, sentinel: &str) {
                use otel_arrow_dfe_engine::error::{Error, error_summary_from, format_error_sources};
                use otel_arrow_dfe_scraper::database::DriverAdapter;
                let source_detail = format_error_sources(&error);
                assert!(source_detail.is_empty());
                for fragment in sentinel.split_whitespace() {
                    assert!(!format!("{error:?}").contains(fragment));
                }
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
                    for fragment in sentinel.split_whitespace() {
                        assert!(!rendered.contains(fragment), "{rendered}");
                    }
                }
            }

            /// Scenario: A native Oracle failure contains sentinel SQL, row, endpoint and nested-source text.
            /// Guarantees: Adapter formatting and the complete engine diagnostic expose no native text.
            #[test]
            fn native_error_text_is_redacted_from_engine_diagnostics() {
                const SENTINEL: &str = "secret-row SELECT-private endpoint-private checkpoint-private";
                let constructors: [fn(oracle::Error) -> OracleAdapterError; 8] = [
                    |error| OracleAdapterError::Initialize(error.into()),
                    |error| OracleAdapterError::Connect(error.into()),
                    |error| OracleAdapterError::Configure(error.into()),
                    |error| OracleAdapterError::Prepare(error.into()),
                    |error| OracleAdapterError::Query(error.into()),
                    |error| OracleAdapterError::Fetch(error.into()),
                    |error| OracleAdapterError::Convert(error.into()),
                    |error| OracleAdapterError::Cancellation(error.into()),
                ];
                for constructor in constructors {
                    let native = oracle::Error::with_source(
                        oracle::ErrorKind::InvalidOperation,
                        std::io::Error::other(SENTINEL),
                    );
                    assert_redacted(constructor(native), SENTINEL);
                }
            }

            /// Scenario: Worker I/O errors contain custom sensitive messages.
            /// Guarantees: Only OS categories/codes survive, with no raw source chain in engine diagnostics.
            #[test]
            fn io_errors_do_not_expose_messages_or_sources() {
                const SENTINEL: &str = "PRIVATE_IO_PATH_AND_CONTENT";
                for error in [
                    OracleAdapterError::Worker(std::io::Error::other(SENTINEL).into()),
                    OracleAdapterError::CancellationWorker(std::io::Error::other(SENTINEL).into()),
                ] {
                    assert_redacted(error, SENTINEL);
                }
            }

            /// Scenario: A native error owns a sensitive nested cause.
            /// Guarantees: Sanitization drops that cause immediately instead of merely hiding retained text.
            #[test]
            fn native_error_payload_is_discarded() {
                use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
                #[derive(Debug)]
                struct SensitiveCause(Arc<AtomicBool>);
                impl std::fmt::Display for SensitiveCause {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        f.write_str("PRIVATE_NATIVE_CAUSE")
                    }
                }
                impl std::error::Error for SensitiveCause {}
                impl Drop for SensitiveCause {
                    fn drop(&mut self) { self.0.store(true, Ordering::SeqCst); }
                }
                let dropped = Arc::new(AtomicBool::new(false));
                let native = oracle::Error::with_source(
                    oracle::ErrorKind::InvalidOperation, SensitiveCause(Arc::clone(&dropped)),
                );
                let error = OracleAdapterError::Query(native.into());
                assert!(dropped.load(Ordering::SeqCst));
                assert_redacted(error, "PRIVATE_NATIVE_CAUSE");
            }

            /// Scenario: Redacted native diagnostics still identify outages and authentication failures.
            /// Guarantees: Retry classification uses retained numeric codes, not discarded messages.
            #[test]
            fn redaction_preserves_numeric_recovery_classification() {
                use otel_arrow_dfe_scraper::database::DriverAdapter;
                let outage = super::OracleErrorCodes { oci: Some(3113), dpi: None };
                assert!(super::OracleAdapter::is_retryable(&OracleAdapterError::Query(outage)));
                assert!(!super::OracleAdapter::is_retryable(&OracleAdapterError::Convert(outage)));
                assert!(!super::OracleAdapter::is_retryable(&OracleAdapterError::Connect(
                    super::OracleErrorCodes { oci: Some(1017), dpi: None },
                )));
                assert!(OracleAdapterError::Query(outage).to_string().contains("3113"));
            }

            /// Scenario: Cursor configuration and malformed cursor text contain sensitive values.
            /// Guarantees: Metadata and timestamp failures do not retain or echo those values.
            #[test]
            fn cursor_errors_redact_supplied_values() {
                const SENTINEL: &str = "PRIVATE_CURSOR_SENTINEL";
                let mut watermark = watermark();
                watermark.timestamp_column = SENTINEL.to_owned();
                let error = validate_described_cursor_columns(
                    &columns(OracleType::Timestamp(6), OracleType::Number(18, 0)),
                    &watermark,
                ).expect_err("missing configured column");
                assert_redacted(error, SENTINEL);
                assert_redacted(parse_cursor_timestamp(SENTINEL).expect_err("invalid timestamp"), SENTINEL);
            }

            /// Scenario: Result aliases contain case-insensitive duplicate sensitive identifiers.
            /// Guarantees: Oracle rejects them before the shared mapper can echo the duplicate name.
            #[test]
            fn duplicate_column_names_are_rejected_without_echoing_them() {
                let error = super::validate_column_names(
                    ["PRIVATE_ALIAS", "private_alias"].into_iter(),
                ).expect_err("duplicate aliases");
                assert_redacted(error, "PRIVATE_ALIAS private_alias");
                super::validate_column_names(["EVENT_TS", "EVENT_ID", "PAYLOAD"].into_iter())
                    .expect("distinct aliases");
            }

            fn test_adapter() -> super::OracleAdapter {
                super::OracleAdapter::new(super::OracleAdapterConfig {
                    connect_string: String::new(),
                    instant_client_dir: String::new(),
                }, super::super::tests::test_provider())
            }

            struct UnavailableProvider {
                pending: bool,
            }

            #[async_trait::async_trait(?Send)]
            impl otel_arrow_dfe_engine::local::capability::auth::basic_auth_provider::BasicAuthProvider
                for UnavailableProvider
            {
                async fn get_credential(&self) -> Result<super::BasicAuthCredential, otel_arrow_dfe_engine::capability::CapabilityError> {
                    if self.pending {
                        return futures::future::pending().await;
                    }
                    Err(otel_arrow_dfe_engine::capability::CapabilityErrorSource::<
                        otel_arrow_dfe_engine::capability::auth::basic_auth_provider::BasicAuthProvider
                    >::new("PRIVATE_PROVIDER".into()).error("PRIVATE_CREDENTIAL_CONTENT"))
                }

                fn credential_stream(&self) -> otel_arrow_dfe_engine::capability::auth::basic_auth_provider::BasicAuthCredentialStream {
                    Box::pin(futures::stream::pending())
                }
            }

            /// Scenario: A credential provider fails with sensitive diagnostics before a native operation starts.
            /// Guarantees: The adapter reports a terminal redacted error without creating a native worker.
            #[tokio::test]
            async fn provider_failures_are_redacted_before_native_work() {
                use otel_arrow_dfe_scraper::database::DriverAdapter;
                let mut adapter = test_adapter();
                adapter.credentials = Box::new(UnavailableProvider { pending: false });
                let config: super::super::OracleReceiverConfig =
                    serde_json::from_value(super::super::tests::documented_config()).expect("config");
                _ = adapter.begin_operation().expect("operation");
                let error = adapter.validate_query(&config.query()).await.expect_err("provider failure");
                assert!(matches!(error, OracleAdapterError::CredentialUnavailable));
                assert!(!super::OracleAdapter::is_retryable(&error));
                assert_redacted(error, "PRIVATE_PROVIDER PRIVATE_CREDENTIAL_CONTENT");
                assert!(adapter.worker.is_none());
                adapter.shutdown().await.expect("cleanup");
            }

            /// Scenario: A provider never completes acquisition and stop arrives before a connection exists.
            /// Guarantees: Cancellation wakes the wait promptly and no worker or connection is started.
            #[tokio::test]
            async fn credential_acquisition_is_cancellable() {
                use otel_arrow_dfe_scraper::database::{DriverAdapter, DriverCancellation};
                let mut adapter = test_adapter();
                adapter.credentials = Box::new(UnavailableProvider { pending: true });
                let cancellation = adapter.begin_operation().expect("operation");
                {
                    let acquisition = adapter.credential(Duration::from_secs(300));
                    tokio::pin!(acquisition);
                    assert!(futures::poll!(&mut acquisition).is_pending());
                    cancellation.cancel().await.expect("cancel");
                    assert!(matches!(
                        tokio::time::timeout(Duration::from_secs(1), acquisition).await.expect("prompt cancellation"),
                        Err(OracleAdapterError::Cancelled),
                    ));
                }
                assert!(adapter.worker.is_none());
                adapter.shutdown().await.expect("cleanup");
            }

            /// Scenario: Cancellation from an earlier operation arrives while a new credential request waits.
            /// Guarantees: A stale handle cannot wake or cancel the new request; the matching handle still stops it.
            #[tokio::test]
            async fn stale_cancellation_does_not_wake_new_credential_wait() {
                use otel_arrow_dfe_scraper::database::{DriverAdapter, DriverCancellation};
                let mut adapter = test_adapter();
                adapter.credentials = Box::new(UnavailableProvider { pending: true });
                let previous = adapter.begin_operation().expect("previous operation");
                let current = adapter.begin_operation().expect("current operation");
                {
                    let acquisition = adapter.credential(Duration::from_secs(300));
                    tokio::pin!(acquisition);
                    assert!(futures::poll!(&mut acquisition).is_pending());
                    previous.cancel().await.expect("stale cancellation ignored");
                    assert!(futures::poll!(&mut acquisition).is_pending());
                    current.cancel().await.expect("matching cancellation");
                    assert!(matches!(
                        tokio::time::timeout(Duration::from_secs(1), acquisition).await.expect("prompt cancellation"),
                        Err(OracleAdapterError::Cancelled),
                    ));
                }
                assert!(adapter.worker.is_none());
                adapter.shutdown().await.expect("cleanup");
            }

            /// Scenario: A provider never completes acquisition without an external cancellation.
            /// Guarantees: The configured acquisition deadline produces a terminal error rather than an unbounded wait.
            #[tokio::test(start_paused = true)]
            async fn credential_acquisition_has_a_deadline() {
                let mut adapter = test_adapter();
                adapter.credentials = Box::new(UnavailableProvider { pending: true });
                assert!(matches!(
                    adapter.credential(Duration::from_secs(1)).await,
                    Err(OracleAdapterError::CredentialTimeout),
                ));
                assert!(adapter.worker.is_none());
            }

            /// Scenario: A provider returns a credential inside the shared expiry safety margin.
            /// Guarantees: Oracle rejects the snapshot before native work even if the provider fails to enforce expiry.
            #[tokio::test]
            async fn near_expiry_credentials_are_rejected() {
                let mut adapter = test_adapter();
                adapter.credentials = super::super::tests::credential_provider(
                    super::BasicAuthCredential::new("user", "password").expect("credential")
                        .with_expiry(std::time::Instant::now() + Duration::from_secs(20)),
                );
                assert!(matches!(
                    adapter.credential(Duration::from_secs(1)).await,
                    Err(OracleAdapterError::CredentialUnavailable),
                ));
                assert!(adapter.worker.is_none());
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
            #[ignore = "requires ORACLE_SCHEMA_TEST_CONFIG, ORACLE_USERNAME, and ORACLE_PASSWORD_FILE for a disposable OTAP_SCHEMA_DRIFT fixture"]
            async fn live_timestamp_precision_change_is_rejected() -> Result<(), Box<dyn std::error::Error>> {
                use otel_arrow_dfe_scraper::database::DriverAdapter;

                let path = std::env::var("ORACLE_SCHEMA_TEST_CONFIG")
                    .map_err(|_| "ORACLE_SCHEMA_TEST_CONFIG is required and must contain valid Unicode")?;
                let bytes = fs::read(path).map_err(|_| "cannot read schema test configuration")?;
                let value: serde_json::Value = serde_json::from_slice(&bytes)
                    .map_err(|_| "schema test configuration must contain valid JSON")?;
                let credential = super::super::tests::fixture_credential(
                    &std::env::var("ORACLE_USERNAME")
                        .map_err(|_| "ORACLE_USERNAME is required and must contain valid Unicode")?,
                    &std::env::var("ORACLE_PASSWORD_FILE")
                        .map_err(|_| "ORACLE_PASSWORD_FILE is required and must contain valid Unicode")?,
                );
                let config: super::super::OracleReceiverConfig = serde_json::from_value(value.clone())?;
                let query = config.query();
                let mut adapter = config.adapter(super::super::tests::credential_provider(credential.clone()));
                let result = async {
                    _ = adapter.begin_operation()?;
                    _ = adapter.validate_query(&query).await?;
                    _ = adapter.begin_operation()?;
                    let page = adapter.execute(&query, &query.watermark().initial()).await?;
                    let cursor = page.rows.last().ok_or("fixture must contain one row")?.cursor.clone();
                    let writer = oracle::Connection::connect(
                        credential.expose_username(),
                        credential.expose_password(),
                        value["connection"]["connect_string"].as_str().ok_or("connect string")?,
                    ).map_err(|error| OracleAdapterError::Connect(error.into()))?;
                    _ = writer.execute(
                        "ALTER TABLE OTAP_SCHEMA_DRIFT MODIFY EVENT_TS TIMESTAMP(9)",
                        &[],
                    ).map_err(|error| OracleAdapterError::Query(error.into()))?;
                    _ = writer.execute(
                        "INSERT INTO OTAP_SCHEMA_DRIFT VALUES (2, TIMESTAMP '2026-01-01 00:00:00.123456001', 'drift')",
                        &[],
                    ).map_err(|error| OracleAdapterError::Query(error.into()))?;
                    writer.commit().map_err(|error| OracleAdapterError::Configure(error.into()))?;
                    _ = adapter.begin_operation()?;
                    match adapter.execute(&query, &cursor).await {
                        Err(OracleAdapterError::ResultMetadataChanged) => Ok(()),
                        Err(OracleAdapterError::Query(error))
                            if error.oci_code() == Some(1466) => Ok(()),
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
                    cursor: CompositeCursor::new("2026-01-01 00:00:00".to_owned(), id).into(),
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
                            + super::cursor_heap_bytes(&row.cursor)
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
                                                + super::cursor_heap_bytes(&row.cursor)
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
                    Err(OracleAdapterError::UnsupportedCursorTimestamp)
                ));
                assert!(matches!(
                    validate_described_cursor_columns(
                        &columns(OracleType::Timestamp(6), OracleType::Number(38, 2)),
                        &watermark(),
                    ),
                    Err(OracleAdapterError::UnsupportedCursorTieBreaker)
                ));
                assert!(matches!(
                    validate_described_cursor_columns(
                        &columns(OracleType::Timestamp(6), OracleType::BinaryDouble),
                        &watermark(),
                    ),
                    Err(OracleAdapterError::UnsupportedCursorTieBreaker)
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
                    Err(OracleAdapterError::MissingCursorColumn)
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

            /// Scenario: Metadata discovery has not yet revealed the cursor column's Oracle type.
            /// Guarantees: The single discovery execution uses a lossless zoned, nanosecond bind.
            #[test]
            fn metadata_discovery_uses_a_lossless_cursor_bind() {
                assert!(matches!(
                    discovery_cursor_bind_type(),
                    OracleType::TimestampTZ(9)
                ));
            }

            /// Scenario: Oracle metadata reports each supported cursor timestamp family and precision.
            /// Guarantees: Normal polling binds that exact type instead of universally using TimestampTZ.
            #[test]
            fn normal_polling_matches_the_cursor_column_type() {
                for source_type in [
                    OracleType::Date,
                    OracleType::Timestamp(0),
                    OracleType::Timestamp(9),
                    OracleType::TimestampTZ(6),
                    OracleType::TimestampLTZ(3),
                ] {
                    assert_eq!(
                        cursor_bind_type(&source_type).expect("supported cursor"),
                        source_type
                    );
                }
                assert!(matches!(
                    cursor_bind_type(&OracleType::Varchar2(32)),
                    Err(OracleAdapterError::UnsupportedCursorTimestamp)
                ));
                assert!(matches!(
                    cursor_bind_type(&OracleType::Timestamp(10)),
                    Err(OracleAdapterError::UnsupportedCursorTimestamp)
                ));
            }

            /// Scenario: A timezone-aware checkpoint is rebound to a zoned Oracle cursor column.
            /// Guarantees: Matching the source metadata retains the cursor offset and precision.
            #[test]
            fn timezone_aware_cursor_retains_its_offset() {
                let committed = Timestamp::from_str("2026-01-01 12:34:56.123456789 +05:30")
                    .expect("timezone-aware cursor should parse");
                assert!(committed.with_tz());
                assert_eq!(committed.tz_offset(), 19_800);
                assert_eq!(
                    cursor_bind_type(&OracleType::TimestampTZ(9)).expect("zoned cursor"),
                    OracleType::TimestampTZ(9)
                );
            }

            /// Scenario: An initial or checkpoint cursor has a non-UTC offset and crosses a UTC date boundary.
            /// Guarantees: Every supported source type binds the same UTC instant with its own precision and zone semantics.
            #[test]
            fn cursor_bind_values_are_normalized_to_utc_and_source_precision() {
                for (input, target, expected, with_tz) in [
                    (
                        "2026-01-01 00:15:30.000000000 +05:30",
                        OracleType::Date,
                        "2025-12-31 18:45:30",
                        false,
                    ),
                    (
                        "2026-01-01 00:15:30.123456000 +05:30",
                        OracleType::Timestamp(6),
                        "2025-12-31 18:45:30.123456",
                        false,
                    ),
                    (
                        "2024-03-01 01:00:00.987654000 +14:00",
                        OracleType::Timestamp(6),
                        "2024-02-29 11:00:00.987654",
                        false,
                    ),
                    (
                        "2026-01-01 23:30:00.123456789 -12:00",
                        OracleType::TimestampTZ(9),
                        "2026-01-02 11:30:00.123456789 +00:00",
                        true,
                    ),
                    (
                        "2026-01-01 00:15:30.123000000 +05:30",
                        OracleType::TimestampLTZ(3),
                        "2025-12-31 18:45:30.123 +00:00",
                        true,
                    ),
                ] {
                    let source = Timestamp::from_str(input).expect("offset cursor");
                    let bound =
                        cursor_bind_timestamp(source, &target).expect("supported source type");
                    assert_eq!(bound.to_string(), expected);
                    assert_eq!(bound.with_tz(), with_tz);
                }
            }

            /// Scenario: A timezone-naive cursor already represents UTC for a plain source column.
            /// Guarantees: Removing trailing fractional zeros never shifts the cursor's UTC instant.
            #[test]
            fn naive_cursor_is_already_utc() {
                let source = Timestamp::from_str("2026-01-01 00:15:30.123456000")
                    .expect("naive cursor");
                let bound = cursor_bind_timestamp(source, &OracleType::Timestamp(6))
                    .expect("timestamp");
                assert_eq!(bound.to_string(), "2026-01-01 00:15:30.123456");
                assert!(!bound.with_tz());
            }

            /// Scenario: A TIMESTAMP(6) initial cursor falls between two representable microseconds.
            /// Guarantees: Binding rejects the cursor rather than admitting earlier rows with higher IDs.
            #[test]
            fn timestamp6_initial_cursor_rejects_precision_loss() {
                use otel_arrow_dfe_engine::error::ReceiverErrorKind;
                use otel_arrow_dfe_scraper::database::DriverAdapter;

                let initial = CompositeCursor::new(
                    "2026-01-01 10:00:00.123456789".to_owned(), 100,
                );
                let error = cursor_bind_timestamp(
                    parse_cursor_timestamp(&initial.timestamp).expect("valid initial cursor"),
                    &OracleType::Timestamp(6),
                ).expect_err("must not bind the earlier .123456000 boundary");
                assert!(matches!(
                    error, OracleAdapterError::CursorTimestampPrecisionLoss { precision: 6 },
                ));
                assert!(matches!(
                    super::OracleAdapter::classify_error(&error),
                    ReceiverErrorKind::Configuration,
                ));
                assert!(!super::OracleAdapter::is_retryable(&error));
                assert!(error.to_string().contains("6-digit fractional precision"));
                assert_redacted(error, &initial.timestamp);
                assert_eq!(initial.timestamp, "2026-01-01 10:00:00.123456789");
                assert_eq!(initial.tie_breaker, 100);
            }

            /// Scenario: A DATE initial cursor has a fractional second, with or without a timezone offset.
            /// Guarantees: Binding rejects it rather than admitting rows at the preceding whole second.
            #[test]
            fn date_initial_cursor_rejects_fractional_seconds() {
                for input in [
                    "2026-01-01 10:00:00.000000001",
                    "2026-01-01 10:00:00.500000000",
                    "2026-01-01 00:15:30.123456789 +05:30",
                ] {
                    let error = cursor_bind_timestamp(
                        parse_cursor_timestamp(input).expect("valid initial cursor"),
                        &OracleType::Date,
                    ).expect_err("must not bind an earlier whole-second boundary");
                    assert!(matches!(
                        error, OracleAdapterError::CursorTimestampPrecisionLoss { precision: 0 },
                    ));
                    assert_redacted(error, input);
                }
            }

            /// Scenario: Initial or checkpoint cursors use each timestamp family and precision from zero to nine.
            /// Guarantees: Exact fractions bind unchanged; losing even one nanosecond is rejected, including after UTC conversion.
            #[test]
            fn timestamp_family_binds_require_exact_precision() {
                for precision in 0..=9 {
                    let scale = 10_u32.pow(u32::from(9 - precision));
                    for target in [
                        OracleType::Timestamp(precision),
                        OracleType::TimestampTZ(precision),
                        OracleType::TimestampLTZ(precision),
                    ] {
                        for nanos in [0, 1, 1_000, 123_000_000, 123_456_000, 123_456_789, 999_999_999] {
                            for offset in ["", " +05:30", " -12:00"] {
                                let input = format!("2026-01-01 00:15:30.{nanos:09}{offset}");
                                let result = cursor_bind_timestamp(
                                    parse_cursor_timestamp(&input).expect("valid cursor"), &target,
                                );
                                if nanos % scale == 0 {
                                    assert_eq!(
                                        result.expect("exactly representable cursor").nanosecond(), nanos,
                                    );
                                } else {
                                    assert!(matches!(
                                        result,
                                        Err(OracleAdapterError::CursorTimestampPrecisionLoss { precision: actual })
                                            if actual == precision
                                    ), "{target:?}: {input}");
                                }
                            }
                        }
                    }
                }
            }

            /// Scenario: Normalized Oracle output contains plain/zoned timestamps and number/int tie-breakers.
            /// Guarantees: The composite cursor reuses those values without another native row decode.
            #[test]
            fn cursor_is_derived_from_normalized_output() {
                for (values, expected_timestamp, expected_id) in [
                    (
                        vec![
                            CellValue::String("payload".to_owned()),
                            CellValue::Timestamp("2026-01-01T00:00:00.123456000".to_owned()),
                            CellValue::Decimal("42".to_owned()),
                        ],
                        "2026-01-01T00:00:00.123456000",
                        42,
                    ),
                    (
                        vec![
                            CellValue::String("payload".to_owned()),
                            CellValue::TimestampTz(
                                "2025-12-31T18:30:00.123456789+00:00".to_owned(),
                            ),
                            CellValue::Int64(-7),
                        ],
                        "2025-12-31T18:30:00.123456789+00:00",
                        -7,
                    ),
                ] {
                    let row = Row { values };
                    let unchanged = row.clone();
                    let cursor = extract_normalized_cursor(&row, 1, 2)
                        .expect("valid normalized cursor");
                    assert_eq!(cursor.timestamp, expected_timestamp);
                    assert_eq!(cursor.tie_breaker, expected_id);
                    assert_eq!(row, unchanged, "cursor extraction must not alter emitted values");
                }
            }

            /// Scenario: Normalized cursor cells are NULL, malformed, out of range, or the wrong value type.
            /// Guarantees: The receiver fails the page rather than advancing with a fabricated cursor.
            #[test]
            fn invalid_normalized_cursor_fails_closed() {
                for values in [
                    vec![CellValue::Null, CellValue::Int64(1)],
                    vec![CellValue::Timestamp("2026-01-01T00:00:00".to_owned()), CellValue::Null],
                    vec![
                        CellValue::Timestamp("2026-01-01T00:00:00".to_owned()),
                        CellValue::Decimal("9223372036854775808".to_owned()),
                    ],
                    vec![CellValue::String("not a timestamp".to_owned()), CellValue::Int64(1)],
                ] {
                    assert!(extract_normalized_cursor(&Row { values }, 0, 1).is_err());
                }
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
                        Err(OracleAdapterError::InvalidCursorTimestamp)
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
                    Err(OracleAdapterError::UnsupportedType)
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

        }
    };
    (worker) => {
        mod tests {
            use super::*;
            use std::time::{Duration, Instant};

            struct DropProbe(std::sync::mpsc::Sender<std::thread::ThreadId>);

            impl Drop for DropProbe {
                fn drop(&mut self) {
                    let _ = self.0.send(std::thread::current().id());
                }
            }

            /// Scenario: A successful worker is explicitly stopped or dropped between jobs.
            /// Guarantees: State is destroyed off-core, and awaited shutdown acknowledges only completed cleanup.
            #[tokio::test]
            async fn successful_job_and_cleanup_stay_on_worker() {
                for await_cleanup in [true, false] {
                    let (dropped, observed) = std::sync::mpsc::channel();
                    let mut worker =
                        NativeWorker::<Option<DropProbe>>::new("oracle-cleanup-test").expect("worker");
                    let operation = worker
                        .run(move |state| {
                            *state = Some(DropProbe(dropped));
                            std::thread::current().id()
                        })
                        .expect("accepted");
                    let thread = receive(operation).await.expect("successful operation");
                    assert_ne!(thread, std::thread::current().id());
                    assert!(observed.try_recv().is_err());
                    let dropped_on = if await_cleanup {
                        worker.stop().await.expect("confirmed cleanup");
                        observed.try_recv().expect("cleanup before acknowledgement")
                    } else {
                        drop(worker);
                        observed
                            .recv_timeout(Duration::from_secs(2))
                            .expect("cleanup")
                    };
                    assert_eq!(dropped_on, thread);
                }
            }

            /// Scenario: One worker job is active and another occupies the only queue slot.
            /// Guarantees: Further submissions fail immediately and accepted jobs finish after release.
            #[tokio::test]
            async fn worker_queue_is_bounded() {
                let mut worker = NativeWorker::<()>::new("oracle-capacity-test").expect("worker");
                let (release, gate) = sync_channel(1);
                let (started, ready) = oneshot::channel();
                let first = worker
                    .run(move |_| {
                        let _ = started.send(());
                        gate.recv().expect("released");
                    })
                    .expect("first job");
                receive(ready).await.expect("worker started");
                let second = worker.run(|_| ()).expect("queued job");
                assert_eq!(
                    worker.run(|_| ()).expect_err("queue must be full").kind(),
                    io::ErrorKind::WouldBlock
                );
                release.send(()).expect("release worker");
                receive(first).await.expect("first result");
                receive(second).await.expect("second result");
                worker.stop().await.expect("cleanup");
            }

            /// Scenario: Native worker code panics before replying.
            /// Guarantees: Neither the reply nor cleanup channel can report false success.
            #[tokio::test]
            async fn worker_panic_does_not_confirm_cleanup() {
                let mut worker = NativeWorker::<()>::new("oracle-panic-test").expect("worker");
                let result = worker
                    .run::<()>(|_| panic!("synthetic worker failure"))
                    .expect("accepted");
                assert_eq!(
                    receive(result).await.expect_err("no reply").kind(),
                    io::ErrorKind::BrokenPipe
                );
                assert_eq!(
                    worker
                        .stop()
                        .await
                        .expect_err("no cleanup confirmation")
                        .kind(),
                    io::ErrorKind::BrokenPipe
                );
            }

            struct StuckDrop;

            impl Drop for StuckDrop {
                fn drop(&mut self) {
                    loop {
                        std::thread::park();
                    }
                }
            }

            /// Scenario: A real worker operation or its resource destructor never returns.
            /// Guarantees: An async deadline and ordinary Tokio runtime destruction finish without joining that thread.
            #[test]
            fn stuck_work_does_not_hold_tokio_runtime() {
                const CHILD: &str = "OTEL_ORACLE_WORKER_HANG_CASE";
                if let Ok(mode) = std::env::var(CHILD) {
                    assert!(matches!(mode.as_str(), "operation" | "cleanup"));
                    let stuck_operation = mode == "operation";
                    let mut worker =
                        NativeWorker::<Option<StuckDrop>>::new("oracle-stuck-test").expect("worker");
                    let (started, ready) = oneshot::channel();
                    let _operation = worker
                        .run(move |state| {
                            *state = (!stuck_operation).then(|| StuckDrop);
                            let _ = started.send(());
                            if stuck_operation {
                                loop {
                                    std::thread::park();
                                }
                            }
                        })
                        .expect("job");
                    let runtime = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .expect("runtime");
                    let began = Instant::now();
                    runtime.block_on(async {
                        tokio::time::timeout(Duration::from_secs(1), receive(ready))
                            .await
                            .expect("worker started promptly")
                            .expect("start notification");
                        assert!(
                            tokio::time::timeout(Duration::from_millis(30), worker.stop())
                                .await
                                .is_err()
                        );
                    });
                    drop(runtime);
                    assert!(began.elapsed() < Duration::from_secs(2));
                    return;
                }

                for mode in ["operation", "cleanup"] {
                    let mut child = std::process::Command::new(std::env::current_exe().expect("test binary"))
                        .args([
                            "--exact",
                            "receivers::oracle_receiver::worker::tests::stuck_work_does_not_hold_tokio_runtime",
                            "--nocapture",
                        ])
                        .env(CHILD, mode)
                        .spawn()
                        .expect("subprocess");
                    let deadline = Instant::now() + Duration::from_secs(10);
                    loop {
                        if let Some(status) = child.try_wait().expect("child status") {
                            assert!(status.success(), "{mode} subprocess failed");
                            break;
                        }
                        if Instant::now() >= deadline {
                            child.kill().expect("kill hung regression subprocess");
                            let _ = child.wait();
                            panic!("runtime destruction waited for the {mode} worker");
                        }
                        std::thread::sleep(Duration::from_millis(10));
                    }
                }
            }
        }
    };
}

use super::*;
use otel_arrow_dfe_engine::capability::CapabilityError;
use otel_arrow_dfe_engine::capability::auth::BasicAuthCredential;
use otel_arrow_dfe_engine::capability::auth::basic_auth_provider::BasicAuthCredentialStream;
use otel_arrow_dfe_engine::context::ControllerContext;
use otel_arrow_dfe_engine::local::capability::auth::basic_auth_provider::BasicAuthProvider as LocalBasicAuthProvider;
use otel_arrow_dfe_engine::receiver::ReceiverWrapper;
use otel_arrow_dfe_engine::testing::{receiver::TestRuntime, test_node};
use otel_arrow_dfe_scraper::SourceLease;
use otel_arrow_dfe_scraper::database::OnNack;
use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;
use std::time::{Duration, Instant};

const COMPOSITE_STATEMENT: &str = "SELECT AUDIT_ID, LAST_UPDATED, PAYLOAD FROM AUDIT_LOGS \
     WHERE (LAST_UPDATED > :last_timestamp \
     OR (LAST_UPDATED = :last_timestamp AND AUDIT_ID > :last_tie_breaker)) \
     ORDER BY LAST_UPDATED ASC, AUDIT_ID ASC";

struct FixedCredentials(BasicAuthCredential);

#[async_trait::async_trait(?Send)]
impl LocalBasicAuthProvider for FixedCredentials {
    async fn get_credential(&self) -> Result<BasicAuthCredential, CapabilityError> {
        Ok(self.0.clone())
    }

    fn credential_stream(&self) -> BasicAuthCredentialStream {
        Box::pin(futures::stream::iter([self.0.clone()]))
    }
}

pub(super) fn credential_provider(
    credential: BasicAuthCredential,
) -> Box<dyn LocalBasicAuthProvider> {
    Box::new(FixedCredentials(credential))
}

pub(super) fn test_provider() -> Box<dyn LocalBasicAuthProvider> {
    credential_provider(BasicAuthCredential::new("test-user", "test-password").expect("credential"))
}

fn build(pipeline: &PipelineContext, name: &str, config: &Value) -> Result<Receiver, ConfigError> {
    super::build(pipeline, name, config, test_provider())
}

pub(super) fn fixture_credential(username: &str, password_file: &str) -> BasicAuthCredential {
    use secrecy::ExposeSecret;
    let password = secrecy::SecretString::from(
        std::fs::read_to_string(password_file).expect("fixture password"),
    );
    BasicAuthCredential::new(
        username,
        password.expose_secret().trim_end_matches(['\r', '\n']),
    )
    .expect("fixture credentials")
}

/// Scenario: A live-test fixture supplies a username value and a newline-terminated password file.
/// Guarantees: Only the password is read from disk; the username is used verbatim like extension YAML.
#[test]
fn fixture_credential_uses_inline_username() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let password_file = directory.path().join("password");
    std::fs::write(&password_file, "fixture-password\r\n").expect("write fixture password");
    let credential = fixture_credential(
        "fixture-user",
        password_file.to_str().expect("UTF-8 fixture path"),
    );
    assert_eq!(credential.expose_username(), "fixture-user");
    assert_eq!(credential.expose_password(), "fixture-password");
}

pub(super) fn documented_config() -> Value {
    serde_json::json!({
        "source_id": "oracle-audit",
        "connection": {
            "connect_string": "database.contoso.com:1521/ORCL",
            "instant_client_dir": "/opt/oracle/instantclient"
        },
        "query": {
            "statement": COMPOSITE_STATEMENT,
            "interval": "1m",
            "fetch_size_rows": 300,
            "max_rows_per_poll": 10000,
            "max_batch_bytes": "10 MiB",
            "timeout": "30s"
        },
        "watermark": {
            "mode": "composite",
            "timestamp": {
                "column": "LAST_UPDATED",
                "bind": "last_timestamp",
                "initial": "1970-01-01 00:00:00",
                "timezone": "UTC"
            },
            "tie_breaker": {
                "column": "AUDIT_ID",
                "bind": "last_tie_breaker",
                "initial": 0
            }
        },
        "checkpoint": {
            "directory": "${engine.state_dir}/oracle",
            "on_nack": "rewind",
            "nack_backoff": "1s",
            "max_consecutive_failures": 5
        }
    })
}

fn with_statement(statement: &str) -> Value {
    let mut config = documented_config();
    config["query"]["statement"] = serde_json::json!(statement);
    config
}

/// Scenario: SQL contains all watermark comparisons but combines them incorrectly or unions rows.
/// Guarantees: Unwatermarked branches and predicates that skip timestamp ties fail before connecting.
#[test]
fn rejects_composite_predicate_lookalikes_and_set_operations() {
    for statement in [
        "SELECT AUDIT_ID, LAST_UPDATED FROM AUDIT_LOGS WHERE LAST_UPDATED > :last_timestamp AND (LAST_UPDATED = :last_timestamp OR AUDIT_ID > :last_tie_breaker) ORDER BY LAST_UPDATED ASC, AUDIT_ID ASC",
        "SELECT AUDIT_ID, LAST_UPDATED FROM AUDIT_LOGS WHERE LAST_UPDATED > :last_timestamp OR (LAST_UPDATED = :last_timestamp AND AUDIT_ID > :last_tie_breaker) OR 1 = 1 ORDER BY LAST_UPDATED ASC, AUDIT_ID ASC",
    ] {
        assert!(parsed(with_statement(statement)).is_err());
    }
    for operation in ["UNION", "UNION ALL", "INTERSECT", "MINUS", "EXCEPT"] {
        let statement = COMPOSITE_STATEMENT.replacen(
            " ORDER BY",
            &format!(
                " {operation} SELECT AUDIT_ID, LAST_UPDATED, PAYLOAD FROM AUDIT_LOGS ORDER BY"
            ),
            1,
        );
        for statement in [statement.clone(), statement.to_ascii_lowercase()] {
            assert!(parsed(with_statement(&statement)).is_err(), "{statement}");
        }
    }
}

/// Scenario: Native Oracle configuration supplies only the original max_batch_bytes setting.
/// Guarantees: It parses without an additional field and uses that limit for both row storage and encoding.
#[test]
fn uses_one_byte_limit_for_rows_and_encoding() {
    let config = parsed(documented_config()).expect("configuration");
    assert_eq!(config.query().max_batch_bytes(), 10 * 1024 * 1024);
    assert_eq!(
        config.query().max_normalized_bytes(),
        config.query().max_batch_bytes()
    );
}

/// Scenario: Oracle configuration omits or overrides the shared catch-up budgets.
/// Guarantees: Shared defaults are preserved and explicit native settings reach the query plan.
#[test]
fn forwards_catch_up_configuration() {
    let defaults = parsed(documented_config())
        .expect("defaults")
        .query()
        .catch_up();
    assert_eq!(defaults.max_pages, 32);
    assert_eq!(defaults.max_duration, Duration::from_secs(10));
    let mut config = documented_config();
    config["query"]["catch_up"] = serde_json::json!({"max_pages": 1, "max_duration": "2s"});
    let configured = parsed(config).expect("override").query().catch_up();
    assert_eq!(configured.max_pages, 1);
    assert_eq!(configured.max_duration, Duration::from_secs(2));
}

/// Scenario: Oracle configuration supplies an invalid shared catch-up page budget.
/// Guarantees: The vendor wrapper applies shared validation instead of bypassing it.
#[test]
fn rejects_invalid_catch_up_budget() {
    for pages in [0, 1025] {
        let mut config = documented_config();
        config["query"]["catch_up"] = serde_json::json!({"max_pages": pages});
        assert!(parsed(config).is_err());
    }
}

/// Scenario: Configuration includes the removed independent normalized-row byte setting.
/// Guarantees: The obsolete field is rejected rather than silently ignored.
#[test]
fn rejects_obsolete_normalized_byte_limit() {
    let mut config = documented_config();
    config["query"]["max_normalized_bytes"] = serde_json::json!("5 MiB");
    let error = parsed(config).err().expect("obsolete field must fail");
    assert!(
        error
            .to_string()
            .contains("invalid Oracle receiver configuration")
    );
}

/// Scenario: Oracle accepts date-only and timezone-naive ISO initial timestamps.
/// Guarantees: Initial cursors are normalized to the same text representation as fetched cursors.
#[test]
fn normalizes_all_accepted_initial_timestamp_spellings() {
    for initial in ["1970-01-01T00:00:00", "1970-01-01"] {
        let mut config = documented_config();
        config["watermark"]["timestamp"]["initial"] = serde_json::json!(initial);
        let plan = parsed(config).expect("Oracle timestamp").query();
        assert!(
            plan.watermark()
                .as_composite()
                .expect("composite fixture")
                .initial
                .timestamp
                .starts_with("1970-01-01 00:00:00")
        );
    }
}

fn parsed(config: Value) -> Result<OracleReceiverConfig, serde_json::Error> {
    serde_json::from_value(config)
}

fn pipeline_context() -> PipelineContext {
    ControllerContext::new(TelemetryRegistryHandle::new()).pipeline_context_with(
        "group".into(),
        "pipeline".into(),
        0,
        1,
        0,
    )
}

/// Scenario: The engine attempts to create the unpartitioned Oracle receiver on two cores.
/// Guarantees: Factory validation rejects duplicate per-core pollers before acquiring a lease.
#[test]
fn factory_rejects_multi_core_placement() {
    let pipeline = ControllerContext::new(TelemetryRegistryHandle::new()).pipeline_context_with(
        "group".into(),
        "pipeline".into(),
        0,
        2,
        0,
    );
    let runtime = TestRuntime::<OtapPdata>::new();
    let result = (ORACLE_RECEIVER.create)(
        pipeline,
        test_node("oracle-test"),
        Arc::new(NodeUserConfig::new_receiver_config(ORACLE_RECEIVER_URN)),
        runtime.config(),
        &otel_arrow_dfe_engine::capability::registry::Capabilities::empty(),
    );
    assert!(matches!(
        result,
        Err(ConfigError::InvalidUserConfig { error }) if error.contains("single-core")
    ));
}

/// Scenario: A single-core Oracle receiver has no basic_auth_provider binding.
/// Guarantees: Construction fails with an actionable capability error before checkpoint or native work.
#[test]
fn factory_requires_authentication_capability() {
    let runtime = TestRuntime::<OtapPdata>::new();
    let result = (ORACLE_RECEIVER.create)(
        pipeline_context(),
        test_node("oracle-test"),
        Arc::new(NodeUserConfig::new_receiver_config(ORACLE_RECEIVER_URN)),
        runtime.config(),
        &otel_arrow_dfe_engine::capability::registry::Capabilities::empty(),
    );
    assert!(matches!(
        result,
        Err(ConfigError::InvalidUserConfig { error }) if error.contains("basic_auth_provider"),
    ));
}

/// Scenario: A configuration still supplies Oracle's removed credential-file block.
/// Guarantees: Legacy authentication is rejected rather than silently ignored alongside a capability binding.
#[test]
fn rejects_legacy_authentication_configuration() {
    let mut config = documented_config();
    config["authentication"] =
        serde_json::json!({"username_file": "username", "password_file": "password"});
    assert!(parsed(config).is_err());
}

/// Scenario: The receiver loads the complete documented composite configuration.
/// Guarantees: The public schema builds a query plan whose cursor binds, checkpoint policy, and
/// source identity are all present, so the documented example remains runnable.
#[test]
fn accepts_the_documented_composite_configuration() {
    let config = parsed(documented_config()).expect("configuration should deserialize");

    assert_eq!(config.source_id(), "oracle-audit");
    assert_eq!(config.checkpoint().on_nack, OnNack::Rewind);
    assert_eq!(config.checkpoint().nack_backoff, Duration::from_secs(1));
    let query = config.query();
    assert_eq!(
        query
            .watermark()
            .as_composite()
            .expect("composite fixture")
            .timestamp_bind,
        "last_timestamp"
    );
    assert_eq!(
        query
            .watermark()
            .as_composite()
            .expect("composite fixture")
            .tie_breaker_bind,
        "last_tie_breaker"
    );
    assert_eq!(
        query
            .watermark()
            .as_composite()
            .expect("composite fixture")
            .initial
            .tie_breaker,
        0
    );
}

/// Scenario: Permanent-NACK policy is omitted or explicitly configured for shared local retries.
/// Guarantees: Oracle inherits pause by default and changing rejection policy preserves stream identity.
#[test]
fn permanent_rejection_uses_shared_policy() {
    use otel_arrow_dfe_scraper::database::OnPermanentNack;
    let baseline = parsed(documented_config()).expect("default");
    assert_eq!(baseline.query().on_permanent_nack(), OnPermanentNack::Pause);
    let mut raw = documented_config();
    raw["checkpoint"]["on_permanent_nack"] = serde_json::json!("retry");
    let retry = parsed(raw.clone()).expect("retry");
    assert_eq!(retry.query().on_permanent_nack(), OnPermanentNack::Retry);
    assert_eq!(baseline.config_fingerprint(), retry.config_fingerprint());
    raw["checkpoint"]["on_permanent_nack"] = serde_json::json!("skip");
    assert!(parsed(raw).is_err());
}

/// Scenario: Required watermark, checkpoint, or byte-limit sections are omitted.
/// Guarantees: Required page bounds and cursor fields stay explicit, so a receiver can never
/// silently run without a durable checkpoint or a byte ceiling.
#[test]
fn requires_every_operational_and_cursor_field() {
    for section in ["watermark", "checkpoint"] {
        let mut config = documented_config();
        _ = config
            .as_object_mut()
            .expect("config object")
            .remove(section);
        assert!(
            parsed(config).is_err(),
            "required section '{section}' must not be optional"
        );
    }
    for field in ["max_rows_per_poll", "max_batch_bytes"] {
        let mut config = documented_config();
        _ = config["query"]
            .as_object_mut()
            .expect("query object")
            .remove(field);
        assert!(
            parsed(config).is_err(),
            "required query field '{field}' must not be optional"
        );
    }
    let mut config = documented_config();
    _ = config["checkpoint"]
        .as_object_mut()
        .expect("checkpoint object")
        .remove("nack_backoff");
    assert!(parsed(config).is_err());
}

/// Scenario: Collection interval, timeout and fetch size are omitted or explicitly set to their defaults.
/// Guarantees: Both configurations use 60s/30s/300 rows and preserve the same checkpoint compatibility fingerprint.
#[test]
fn collection_defaults_are_applied_without_changing_checkpoint_identity() {
    let mut omitted = documented_config();
    for field in ["interval", "timeout", "fetch_size_rows"] {
        _ = omitted["query"]
            .as_object_mut()
            .expect("query")
            .remove(field);
    }
    let mut explicit = omitted.clone();
    explicit["query"]["interval"] = serde_json::json!("1m");
    explicit["query"]["timeout"] = serde_json::json!("30s");
    explicit["query"]["fetch_size_rows"] = serde_json::json!(300);
    let defaults = parsed(omitted).expect("default collection settings");
    let configured = parsed(explicit).expect("explicit default settings");
    for config in [&defaults, &configured] {
        assert_eq!(config.query().interval(), Duration::from_secs(60));
        assert_eq!(config.query().timeout(), Duration::from_secs(30));
        assert_eq!(config.query().fetch_size_rows(), 300);
    }
    assert_eq!(
        defaults.config_fingerprint(),
        configured.config_fingerprint()
    );
}

/// Scenario: Oracle configuration uses the explicit row-unit name, or the old ambiguous fetch_size field.
/// Guarantees: The new field preserves its configured value and checkpoint identity; the old field fails rather than silently selecting the default.
#[test]
fn fetch_size_rows_preserves_overrides_and_rejects_legacy_name() {
    let baseline = parsed(documented_config()).expect("default configuration");
    let mut value = documented_config();
    value["query"]["fetch_size_rows"] = serde_json::json!(123);
    let configured = parsed(value.clone()).expect("explicit row count");
    assert_eq!(configured.query().fetch_size_rows(), 123);
    assert_eq!(
        baseline.config_fingerprint(),
        configured.config_fingerprint()
    );
    for include_new_name in [false, true] {
        let mut legacy = value.clone();
        legacy["query"]["fetch_size"] = serde_json::json!(50);
        if !include_new_name {
            _ = legacy["query"]
                .as_object_mut()
                .expect("query")
                .remove("fetch_size_rows");
        }
        assert!(
            parsed(legacy).is_err(),
            "legacy setting must not be silently ignored"
        );
    }
}

/// Scenario: The interval uses supported endpoints or fractional minutes equivalent to whole seconds.
/// Guarantees: Values from one minute through 24 hours are preserved exactly without rounding.
#[test]
fn collection_interval_accepts_whole_seconds_and_fractional_minutes() {
    for (value, seconds) in [("1m", 60), ("1.5m", 90), ("61s", 61), ("24h", 86400)] {
        let mut config = documented_config();
        config["query"]["interval"] = serde_json::json!(value);
        assert_eq!(
            parsed(config).expect("valid interval").query().interval(),
            Duration::from_secs(seconds)
        );
    }
}

/// Scenario: Collection settings contain nulls, blanks, non-integral values, or unsupported limits.
/// Guarantees: Invalid values fail configuration instead of receiving defaults or silent rounding.
#[test]
fn collection_settings_reject_invalid_values() {
    for field in ["interval", "timeout", "fetch_size_rows"] {
        for value in [Value::Null, serde_json::json!("")] {
            let mut config = documented_config();
            config["query"][field] = value;
            assert!(parsed(config).is_err(), "{field}");
        }
    }
    for (field, value) in [
        ("interval", "0s"),
        ("interval", "59s"),
        ("interval", "60.5s"),
        ("interval", "24h 1s"),
        ("interval", "-1m"),
        ("timeout", "0s"),
        ("timeout", "-1s"),
        ("timeout", "0.5s"),
        ("timeout", "30.5s"),
        ("timeout", "301s"),
    ] {
        let mut config = documented_config();
        config["query"][field] = serde_json::json!(value);
        assert!(parsed(config).is_err(), "{field}={value}");
    }
    for value in [
        serde_json::json!(0),
        serde_json::json!(-1),
        serde_json::json!(0.5),
        serde_json::json!(10001),
    ] {
        let mut config = documented_config();
        config["query"]["fetch_size_rows"] = value;
        assert!(parsed(config).is_err());
    }
    let mut config = documented_config();
    config["query"]["max_rows_per_poll"] = serde_json::json!(100);
    _ = config["query"]
        .as_object_mut()
        .expect("query")
        .remove("fetch_size_rows");
    assert!(
        parsed(config).is_err(),
        "default fetch size still obeys explicit page bounds"
    );
}

/// Scenario: A statement omits a bind, or references one only inside a literal or as a prefix.
/// Guarantees: Both committed cursor components must appear as real bind markers, so a query
/// cannot silently ignore the checkpoint and re-read the full table every poll.
#[test]
fn requires_both_cursor_binds_as_real_markers() {
    let missing_bind = "SELECT AUDIT_ID, LAST_UPDATED FROM AUDIT_LOGS \
         WHERE LAST_UPDATED > :last_timestamp \
         ORDER BY LAST_UPDATED ASC, AUDIT_ID ASC";
    assert!(parsed(with_statement(missing_bind)).is_err());

    let prefix_only = "SELECT AUDIT_ID, LAST_UPDATED FROM AUDIT_LOGS \
         WHERE LAST_UPDATED > :last_timestamp_extra AND AUDIT_ID > :last_tie_breaker \
         ORDER BY LAST_UPDATED ASC, AUDIT_ID ASC";
    assert!(parsed(with_statement(prefix_only)).is_err());

    let literal_only = "SELECT AUDIT_ID, LAST_UPDATED FROM AUDIT_LOGS \
         WHERE ':last_timestamp' = ':last_timestamp' AND AUDIT_ID > :last_tie_breaker \
         ORDER BY LAST_UPDATED ASC, AUDIT_ID ASC";
    assert!(parsed(with_statement(literal_only)).is_err());
}

/// Scenario: Both bind names exist, but the query does not use the strict composite predicate.
/// Guarantees: A full-table or inclusive-boundary query is rejected before polling so an ACKed
/// page cannot repeatedly commit the same cursor.
#[test]
fn requires_the_strict_composite_predicate() {
    let no_predicate = "SELECT AUDIT_ID, LAST_UPDATED FROM AUDIT_LOGS \
         WHERE :last_timestamp IS NOT NULL AND :last_tie_breaker IS NOT NULL \
         ORDER BY LAST_UPDATED ASC, AUDIT_ID ASC";
    assert!(parsed(with_statement(no_predicate)).is_err());

    let inclusive = "SELECT AUDIT_ID, LAST_UPDATED FROM AUDIT_LOGS \
         WHERE (LAST_UPDATED >= :last_timestamp OR (LAST_UPDATED = :last_timestamp \
         AND AUDIT_ID > :last_tie_breaker)) ORDER BY LAST_UPDATED ASC, AUDIT_ID ASC";
    assert!(parsed(with_statement(inclusive)).is_err());
}

/// Scenario: A cursor column is configured with quoted, qualified, or unsafe identifier syntax.
/// Guarantees: Only plain Oracle identifiers can participate in validated paging SQL, preventing
/// configuration text from becoming executable SQL syntax.
#[test]
fn rejects_unsafe_cursor_identifiers() {
    for column in ["AUDIT.ID", "\"AUDIT_ID\"", "AUDIT ID", "AUDIT_ID;DELETE"] {
        let mut config = documented_config();
        config["watermark"]["tie_breaker"]["column"] = serde_json::json!(column);
        assert!(parsed(config).is_err(), "unsafe identifier '{column}'");
    }
}

/// Scenario: The configured initial timestamp cannot be represented by Oracle's timestamp type.
/// Guarantees: Invalid initial state fails configuration instead of surfacing only after the
/// receiver has acquired its source lease and started database work.
#[test]
fn rejects_invalid_initial_timestamp() {
    let mut config = documented_config();
    config["watermark"]["timestamp"]["initial"] = serde_json::json!("not-a-timestamp");

    assert!(parsed(config).is_err());
}

/// Scenario: Initial timestamp components overflow or narrow in the native parser, or exceed its precision.
/// Guarantees: Configuration returns a field-specific error without panicking or changing the initial position.
#[test]
fn rejects_oversized_or_wrapping_initial_timestamps() {
    for initial in [
        "9".repeat(40),
        "4294969322-01-01 00:00:00".to_owned(),
        "2026-4294967297-01 00:00:00".to_owned(),
        "2026-01-01 00:00:00.1234567890".to_owned(),
        "2026-01-01 00:00:00 +4294967297:00".to_owned(),
    ] {
        let mut config = documented_config();
        config["watermark"]["timestamp"]["initial"] = serde_json::json!(initial);
        let error = parsed(config).err().expect("unsafe timestamp must fail");
        assert!(error.to_string().contains("watermark.timestamp.initial"));
    }
}

/// Scenario: Nested comparisons satisfy token checks but the only top-level WHERE occurs after ORDER BY.
/// Guarantees: Invalid clause ordering returns an error instead of panicking during predicate slicing.
#[test]
fn rejects_where_after_order_without_panicking() {
    let mut config = documented_config();
    config["watermark"]["timestamp"]["column"] = serde_json::json!("WHERE");
    config["query"]["statement"] = serde_json::json!(
        "SELECT (WHERE > :last_timestamp), (WHERE = :last_timestamp), \
         (AUDIT_ID > :last_tie_breaker) FROM AUDIT_LOGS \
         ORDER BY WHERE ASC, AUDIT_ID ASC"
    );
    let error = parsed(config).err().expect("misordered clauses must fail");
    assert!(
        error
            .to_string()
            .contains("WHERE predicate before ORDER BY")
    );
}

/// Scenario: Malformed fields, enum values, and unknown keys contain sensitive configuration text.
/// Guarantees: Neither deserialization errors nor engine diagnostics expose those supplied values.
#[test]
fn configuration_errors_redact_supplied_values() {
    use otel_arrow_dfe_engine::error::{Error, error_summary_from};

    const SENTINEL: &str = "PRIVATE_CONFIGURATION_SENTINEL";
    let mut invalid_cursor = documented_config();
    invalid_cursor["watermark"]["tie_breaker"]["initial"] = serde_json::json!(SENTINEL);
    let mut invalid_mode = documented_config();
    invalid_mode["watermark"]["mode"] = serde_json::json!(SENTINEL);
    let mut invalid_authentication = documented_config();
    invalid_authentication["authentication"] = serde_json::json!(SENTINEL);
    let mut unknown_field = documented_config();
    unknown_field["query"][SENTINEL] = serde_json::json!(SENTINEL);

    for config in [
        invalid_cursor,
        invalid_mode,
        invalid_authentication,
        unknown_field,
    ] {
        let direct_error = parsed(config.clone()).err().expect("invalid schema");
        assert!(!direct_error.to_string().contains(SENTINEL));
        let error = (ORACLE_RECEIVER.validate_config)(&config).expect_err("invalid schema");
        let engine = Error::ConfigError(Box::new(error));
        for rendered in [
            engine.to_string(),
            format!("{engine:?}"),
            serde_json::to_string(&error_summary_from(&engine)).expect("diagnostic JSON"),
        ] {
            assert!(!rendered.contains(SENTINEL), "{rendered}");
        }
    }
}

/// Scenario: Valid identifier-shaped secrets are supplied as cursor binds or column names.
/// Guarantees: SQL-contract errors identify the failed rule without interpolating configured values.
#[test]
fn semantic_configuration_errors_redact_identifiers() {
    const SENTINEL: &str = "PRIVATE_IDENTIFIER_SENTINEL";
    let mut cases = Vec::new();
    for part in ["timestamp", "tie_breaker"] {
        for key in ["bind", "column"] {
            let mut config = documented_config();
            config["watermark"][part][key] = serde_json::json!(SENTINEL);
            cases.push(config);
        }
    }
    let mut order = documented_config();
    order["watermark"]["timestamp"]["column"] = serde_json::json!(SENTINEL);
    order["query"]["statement"] = serde_json::json!(
        COMPOSITE_STATEMENT
            .replace("LAST_UPDATED", SENTINEL)
            .replace(&format!("{SENTINEL} ASC"), &format!("{SENTINEL} DESC"))
    );
    cases.push(order);
    for config in cases {
        let error = validate(&config).expect_err("invalid SQL contract");
        assert!(!error.to_string().contains(SENTINEL));
        assert!(!format!("{error:?}").contains(SENTINEL));
    }
}

/// Scenario: The factory cannot create a lease beneath a sensitive checkpoint path.
/// Guarantees: Its configuration error contains a safe I/O category rather than the supplied path.
#[test]
fn factory_redacts_checkpoint_lease_paths() {
    const SENTINEL: &str = "PRIVATE_CHECKPOINT_PATH";
    let directory = tempfile::tempdir().expect("temporary directory");
    let blocker = directory.path().join(SENTINEL);
    std::fs::write(&blocker, b"not a directory").expect("blocker");
    let mut config = documented_config();
    config["checkpoint"]["directory"] = serde_json::json!(blocker.join("state"));
    let error = build(&pipeline_context(), "oracle-audit", &config)
        .err()
        .expect("lease failure");
    assert!(!error.to_string().contains(SENTINEL));
    assert!(!format!("{error:?}").contains(SENTINEL));
    assert!(error.to_string().contains("lease I/O"));
}

/// Scenario: A statement's final ordering is descending, reordered, missing, or only nested
/// inside a subquery.
/// Guarantees: The outer result must be ascending by timestamp then tie-breaker, so paging by the
/// composite cursor cannot skip rows an unordered or differently ordered result would return.
#[test]
fn requires_the_final_outer_ascending_ordering() {
    let descending = "SELECT AUDIT_ID, LAST_UPDATED FROM AUDIT_LOGS \
         WHERE LAST_UPDATED > :last_timestamp OR (LAST_UPDATED = :last_timestamp \
         AND AUDIT_ID > :last_tie_breaker) ORDER BY LAST_UPDATED DESC, AUDIT_ID ASC";
    assert!(parsed(with_statement(descending)).is_err());

    let reversed = "SELECT AUDIT_ID, LAST_UPDATED FROM AUDIT_LOGS \
         WHERE LAST_UPDATED > :last_timestamp OR (LAST_UPDATED = :last_timestamp \
         AND AUDIT_ID > :last_tie_breaker) ORDER BY AUDIT_ID ASC, LAST_UPDATED ASC";
    assert!(parsed(with_statement(reversed)).is_err());

    let missing = "SELECT AUDIT_ID, LAST_UPDATED FROM AUDIT_LOGS \
         WHERE LAST_UPDATED > :last_timestamp OR (LAST_UPDATED = :last_timestamp \
         AND AUDIT_ID > :last_tie_breaker)";
    assert!(parsed(with_statement(missing)).is_err());

    let nested_only = "SELECT AUDIT_ID, LAST_UPDATED FROM \
         (SELECT AUDIT_ID, LAST_UPDATED FROM AUDIT_LOGS WHERE LAST_UPDATED > :last_timestamp \
         OR (LAST_UPDATED = :last_timestamp AND AUDIT_ID > :last_tie_breaker) \
         ORDER BY LAST_UPDATED ASC, AUDIT_ID ASC)";
    assert!(parsed(with_statement(nested_only)).is_err());

    let nested_then_wrong_outer = "SELECT AUDIT_ID, LAST_UPDATED FROM \
         (SELECT AUDIT_ID, LAST_UPDATED FROM AUDIT_LOGS WHERE LAST_UPDATED > :last_timestamp \
         OR (LAST_UPDATED = :last_timestamp AND AUDIT_ID > :last_tie_breaker) \
         ORDER BY LAST_UPDATED ASC, AUDIT_ID ASC) ORDER BY AUDIT_ID DESC";
    assert!(parsed(with_statement(nested_then_wrong_outer)).is_err());
}

/// Scenario: A statement contains SQL comments or appends a second SELECT or DELETE.
/// Guarantees: Extra statements and comments are rejected before any database execution.
#[test]
fn rejects_comments_and_multiple_statements() {
    let commented = "SELECT AUDIT_ID, LAST_UPDATED FROM AUDIT_LOGS \
         WHERE LAST_UPDATED > :last_timestamp OR (LAST_UPDATED = :last_timestamp \
         AND AUDIT_ID > :last_tie_breaker) ORDER BY LAST_UPDATED ASC, AUDIT_ID ASC -- trailing";
    assert!(parsed(with_statement(commented)).is_err());

    for statement in [
        format!("{COMPOSITE_STATEMENT}; SELECT 1 FROM DUAL"),
        format!("{COMPOSITE_STATEMENT}; DELETE FROM AUDIT_LOGS"),
        "SELECT 1; DELETE FROM audit_logs".to_owned(),
    ] {
        assert!(parsed(with_statement(&statement)).is_err(), "{statement}");
    }
}

/// Scenario: An otherwise valid watermark query adds an Oracle row-locking clause.
/// Guarantees: FOR UPDATE variants are rejected before connecting or acquiring row locks.
#[test]
fn rejects_row_locking_selects() {
    for clause in [
        "FOR UPDATE",
        "FOR UPDATE OF AUDIT_ID",
        "FOR UPDATE NOWAIT",
        "FOR UPDATE SKIP LOCKED",
        "for update wait 1",
    ] {
        let statement = format!("{COMPOSITE_STATEMENT} {clause}");
        assert!(parsed(with_statement(&statement)).is_err(), "{statement}");
    }
}

/// Scenario: A valid composite statement uses a trailing semicolon and extra surrounding spacing.
/// Guarantees: Ordinary operator formatting is accepted, so validation rejects unsafe SQL rather
/// than merely unusual whitespace.
#[test]
fn accepts_ordinary_statement_formatting() {
    let formatted = format!("  {COMPOSITE_STATEMENT} ;  ");

    assert!(parsed(with_statement(&formatted)).is_ok());
}

/// Scenario: An Oracle query contains a field absent from the closed public schema.
/// Guarantees: Undocumented fields are rejected rather than silently accepting behavior the
/// receiver does not implement.
#[test]
fn rejects_undocumented_oracle_query_fields() {
    for (field, value) in [
        ("name", serde_json::json!("audit-query")),
        ("error_policy", serde_json::json!("fail_batch")),
        (
            "output",
            serde_json::json!({"include_columns": ["AUDIT_ID"]}),
        ),
    ] {
        let mut config = documented_config();
        config["query"][field] = value;
        assert!(
            parsed(config).is_err(),
            "undocumented query field '{field}' must be rejected"
        );
    }
}

/// Scenario: Oracle configuration uses an unsupported query timeout.
/// Guarantees: Fractional-second and excessively long timeouts fail before opening a connection.
#[test]
fn rejects_unsupported_oracle_timeouts() {
    for timeout in ["500us", "6m"] {
        let mut config = documented_config();
        config["query"]["timeout"] = serde_json::json!(timeout);
        assert!(
            parsed(config).is_err(),
            "unsupported timeout '{timeout}' must be rejected"
        );
    }
}

/// Scenario: A source identifier is large enough to amplify every emitted row.
/// Guarantees: Repeated OTLP resource identity remains bounded by configuration validation.
#[test]
fn rejects_oversized_source_id() {
    let mut config = documented_config();
    config["source_id"] = serde_json::json!("x".repeat(257));

    assert!(parsed(config).is_err());
}

/// Scenario: Two configurations differ only in the client installation, or only in semantics.
/// Guarantees: Changing the client location preserves the durable checkpoint, while changing the query or a
/// cursor definition invalidates it so an unrelated position is never resumed.
#[test]
fn fingerprint_tracks_semantics_and_ignores_client_location() {
    let baseline = parsed(documented_config()).expect("baseline should parse");

    let mut relocated = documented_config();
    relocated["connection"]["instant_client_dir"] = serde_json::json!("/opt/oracle/ic-23");
    let relocated = parsed(relocated).expect("relocated client should parse");
    assert_eq!(
        baseline.config_fingerprint(),
        relocated.config_fingerprint()
    );

    let mut different_cursor = documented_config();
    different_cursor["watermark"]["tie_breaker"]["initial"] = serde_json::json!(100);
    let different_cursor = parsed(different_cursor).expect("changed cursor should parse");
    assert_ne!(
        baseline.config_fingerprint(),
        different_cursor.config_fingerprint()
    );

    let mut different_source = documented_config();
    different_source["source_id"] = serde_json::json!("oracle-orders");
    let different_source = parsed(different_source).expect("changed source should parse");
    assert_ne!(
        baseline.config_fingerprint(),
        different_source.config_fingerprint()
    );
}

/// Scenario: Equivalent configured starting instants use different offsets and timestamp spellings.
/// Guarantees: Both compile to the same UTC cursor and checkpoint compatibility fingerprint.
#[test]
fn equivalent_initial_offsets_share_checkpoint_identity() {
    let mut utc = documented_config();
    utc["watermark"]["timestamp"]["initial"] = serde_json::json!("2026-01-01T06:30:00Z");
    let utc = parsed(utc).expect("UTC initial");
    let mut offset = documented_config();
    offset["watermark"]["timestamp"]["initial"] = serde_json::json!("2026-01-01 12:00:00 +05:30");
    let offset = parsed(offset).expect("offset initial");
    assert_eq!(
        utc.query().watermark().initial(),
        offset.query().watermark().initial()
    );
    assert_eq!(utc.config_fingerprint(), offset.config_fingerprint());
    assert_eq!(
        adapter::normalize_cursor_timestamp("2026-01-01 12:00:00 +05:30")
            .expect("normalized cursor"),
        "2026-01-01 06:30:00.000000000 +00:00"
    );
}

/// Scenario: A pipeline validates a documented Oracle node before instantiating it.
/// Guarantees: Configuration validation succeeds repeatedly without acquiring a source lease, so
/// validating a pipeline never blocks the receiver that later runs it.
#[test]
fn validation_does_not_acquire_a_source_lease() {
    validate(&documented_config()).expect("documented configuration should validate");
    validate(&documented_config()).expect("validation must be repeatable");
}

/// Scenario: Two receivers in one process are built against the same checkpoint source.
/// Guarantees: The second build fails while the first owner lives, so two receivers can never
/// race to advance one durable checkpoint and duplicate or lose rows.
#[test]
fn duplicate_source_receivers_cannot_be_built() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let mut config = documented_config();
    config["checkpoint"]["directory"] =
        serde_json::json!(directory.path().to_str().expect("UTF-8 path"));
    let context = pipeline_context();

    let first = build(&context, "oracle-audit", &config).expect("first receiver should build");
    assert!(build(&context, "oracle-audit", &config).is_err());

    drop(first);
    _ = build(&context, "oracle-audit", &config)
        .expect("the source becomes available after the owner is dropped");
}

/// Scenario: Oracle builds against a checkpoint already written in the shared store's current layout.
/// Guarantees: The factory leases that exact native path, preserves progress, and releases ownership on drop.
#[test]
fn factory_owns_shared_checkpoint_identity() {
    use otel_arrow_dfe_scraper::database::CompositeCursor;

    let directory = tempfile::tempdir().expect("temporary directory");
    let mut value = documented_config();
    value["checkpoint"]["directory"] = serde_json::json!(directory.path());
    let config = parsed(value.clone()).expect("configuration");
    let store = CheckpointStore::new(
        directory.path(),
        "group",
        "pipeline",
        "oracle-audit",
        config.source_id(),
        config.config_fingerprint().to_owned(),
    );
    let cursor = otel_arrow_dfe_scraper::database::Cursor::Composite(CompositeCursor::new(
        "2026-01-01 00:00:00".to_owned(),
        42,
    ));
    let (committed, _) = store.write(0, &cursor).expect("seed committed progress");
    let before = store
        .read()
        .expect("read seeded progress")
        .expect("checkpoint");
    assert_eq!(before, committed);
    assert_eq!(before.cursor, cursor);

    let receiver = build(&pipeline_context(), "oracle-audit", &value).expect("factory");
    assert!(matches!(
        SourceLease::acquire(store.lease_key()),
        Err(LeaseError::AlreadyOwned)
    ));
    assert_eq!(
        store.read().expect("unchanged checkpoint"),
        Some(before.clone())
    );
    drop(receiver);
    let _lease = SourceLease::acquire(store.lease_key()).expect("factory released exact lease");
    assert_eq!(store.read().expect("restart checkpoint"), Some(before));
}

/// Scenario: Two Oracle source IDs differ only in case and share a Unicode state directory.
/// Guarantees: The new shared namespace isolates progress and leases, including on Windows.
#[test]
fn factory_preserves_case_distinct_source_ownership() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let root = directory.path().join("state-\u{e9}");
    let mut first = documented_config();
    first["checkpoint"]["directory"] = serde_json::json!(root);
    first["source_id"] = serde_json::json!("Audit");
    let mut second = first.clone();
    second["source_id"] = serde_json::json!("audit");
    let context = pipeline_context();
    let upper = build(&context, "oracle-audit", &first).expect("upper-case source");
    let lower = build(&context, "oracle-audit", &second).expect("lower-case source");
    assert!(build(&context, "oracle-audit", &first).is_err());
    assert!(build(&context, "oracle-audit", &second).is_err());
    drop(upper);
    let _reopened = build(&context, "oracle-audit", &first).expect("upper lease released");
    assert!(build(&context, "oracle-audit", &second).is_err());
    drop(lower);
}

/// Scenario: A live Oracle receiver uses a fixed provider with an inline username and a password file.
/// Guarantees: The receiver validates metadata, binds the cursor, and emits logs before shutdown.
#[test]
fn emits_oracle_rows_when_live_test_is_enabled() {
    if std::env::var_os("OTAP_ORACLE_RECEIVER_E2E").is_none() {
        return;
    }
    let state_directory = tempfile::tempdir().expect("temporary directory");
    let config = serde_json::json!({
        "source_id": "otap-oracle-events",
        "connection": {
            "connect_string": std::env::var("ORACLE_CONNECT_STRING")
                .unwrap_or_else(|_| "//localhost:1521/FREEPDB1".to_owned()),
            "instant_client_dir": std::env::var("ORACLE_INSTANT_CLIENT_DIR")
                .unwrap_or_else(|_| "C:\\oracle\\instantclient".to_owned())
        },
        "query": {
            "statement": "SELECT EVENT_ID, EVENT_TS, PAYLOAD FROM OTAP_ORACLE_EVENTS \
                WHERE (EVENT_TS > :last_timestamp OR (EVENT_TS = :last_timestamp \
                AND EVENT_ID > :last_tie_breaker)) ORDER BY EVENT_TS ASC, EVENT_ID ASC",
            "interval": "1m",
            "fetch_size_rows": 10,
            "max_rows_per_poll": 10,
            "max_batch_bytes": "10 MiB",
            "timeout": "10s"
        },
        "watermark": {
            "mode": "composite",
            "timestamp": {
                "column": "EVENT_TS",
                "bind": "last_timestamp",
                "initial": "1970-01-01 00:00:00",
                "timezone": "UTC"
            },
            "tie_breaker": {
                "column": "EVENT_ID",
                "bind": "last_tie_breaker",
                "initial": 0
            }
        },
        "checkpoint": {
            "directory": state_directory.path().to_str().expect("UTF-8 path"),
            "on_nack": "rewind",
            "nack_backoff": "1s",
            "max_consecutive_failures": 5
        }
    });
    let credential = fixture_credential(
        &std::env::var("ORACLE_USERNAME").expect("inline fixture username"),
        &std::env::var("ORACLE_PASSWORD_FILE").expect("password fixture file"),
    );
    let receiver = super::build(
        &pipeline_context(),
        "oracle-e2e",
        &config,
        credential_provider(credential),
    )
    .expect("receiver config should build");
    let test_runtime = TestRuntime::<OtapPdata>::new();
    let node_config = Arc::new(NodeUserConfig::new_receiver_config(ORACLE_RECEIVER_URN));
    let receiver_wrapper = ReceiverWrapper::local(
        receiver,
        test_node(test_runtime.config().name.clone()),
        node_config,
        test_runtime.config(),
    );

    let (emitted, received) = tokio::sync::oneshot::channel();
    test_runtime
        .set_receiver(receiver_wrapper)
        .run_test(|ctx| async move {
            let emission = tokio::time::timeout(Duration::from_secs(20), received).await;
            ctx.send_shutdown(
                Instant::now() + Duration::from_secs(5),
                "Oracle receiver E2E complete",
            )
            .await
            .expect("shutdown should enqueue");
            emission
                .expect("receiver should emit within the smoke-test deadline")
                .expect("validation should signal emission");
        })
        .run_validation_concurrent(|mut ctx| async move {
            let mut pdata = ctx.recv().await.expect("receiver should emit pdata");
            assert!(pdata.num_items() >= 1);
            emitted.send(()).expect("signal emission");
        });
}

#[path = "scalar_config_tests.rs"]
mod scalar_tests;
