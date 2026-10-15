// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// Private adapter tests remain colocated logically without exposing implementation internals.
macro_rules! postgresql_module_tests {
    (framing) => {
        mod framing_tests {
            use super::*;
            use tokio::io::AsyncReadExt;

            fn frame_limits() -> FrameLimits {
                FrameLimits::new(10 * 1024 * 1024).expect("configured receive budget")
            }

            struct Fragmented {
                bytes: Vec<u8>,
                consumed: Rc<Cell<usize>>,
                fragment: usize,
                pending: bool,
            }

            impl AsyncRead for Fragmented {
                fn poll_read(
                    self: Pin<&mut Self>,
                    cx: &mut Context<'_>,
                    output: &mut ReadBuf<'_>,
                ) -> Poll<io::Result<()>> {
                    let this = self.get_mut();
                    if this.pending {
                        this.pending = false;
                        cx.waker().wake_by_ref();
                        return Poll::Pending;
                    }
                    this.pending = true;
                    let start = this.consumed.get();
                    let count = output
                        .remaining()
                        .min(this.fragment)
                        .min(this.bytes.len() - start);
                    output.put_slice(&this.bytes[start..start + count]);
                    this.consumed.set(start + count);
                    Poll::Ready(Ok(()))
                }
            }

            fn fragmented_limited(
                bytes: Vec<u8>,
                fragment: usize,
                limits: FrameLimits,
            ) -> BoundedBackend<Fragmented> {
                BoundedBackend::new(
                    Fragmented {
                        bytes,
                        consumed: Rc::default(),
                        fragment,
                        pending: true,
                    },
                    ReceiveFailure::default(),
                    limits,
                )
            }

            fn fragmented(bytes: Vec<u8>, fragment: usize) -> BoundedBackend<Fragmented> {
                fragmented_limited(bytes, fragment, frame_limits())
            }

            /// Builds one backend frame: tag, big-endian length including the
            /// length field itself, then the body.
            fn frame(tag: u8, body: &[u8]) -> Vec<u8> {
                let mut bytes = vec![tag];
                bytes.extend_from_slice(&((body.len() + 4) as i32).to_be_bytes());
                bytes.extend_from_slice(body);
                bytes
            }

            /// Drains the stream, recording the bytes returned by each
            /// individual read so frame boundaries stay observable.
            async fn read_chunks(
                stream: &mut BoundedBackend<Fragmented>,
                capacity: usize,
            ) -> io::Result<Vec<Vec<u8>>> {
                let mut chunks = Vec::new();
                loop {
                    let mut buffer = vec![0; capacity];
                    let read = stream.read(&mut buffer).await?;
                    if read == 0 {
                        return Ok(chunks);
                    }
                    buffer.truncate(read);
                    chunks.push(buffer);
                }
            }

            /// Scenario: An oversized backend length arrives one byte at a time.
            /// Guarantees: Only five header bytes are read; no body or header reaches the driver.
            #[tokio::test]
            async fn oversized_header_rejected_before_body() {
                let mut bytes = vec![b'D'];
                bytes.extend_from_slice(&(frame_limits().data_row as i32).to_be_bytes());
                bytes.extend_from_slice(b"unread private row");
                let mut stream = fragmented(bytes, 1);
                let mut output = [0; 64];
                let error = stream.read(&mut output).await.expect_err("oversized frame");
                assert_eq!(error.kind(), io::ErrorKind::InvalidData);
                assert_eq!(stream.failure.get(), Some(Error::Limit));
                assert_eq!(stream.inner.consumed.get(), 5);
                assert_eq!(output, [0; 64]);
                assert!(!error.to_string().contains("private"));
                assert!(stream.read(&mut output).await.is_err());
                assert_eq!(stream.inner.consumed.get(), 5);
            }

            /// Scenario: Two complete backend frames reach a destination with room for both, delivered one byte at a time, in uneven pieces, and all at once.
            /// Guarantees: Every byte is forwarded verbatim and no single read ever spans a frame boundary, whatever the socket hands over.
            #[tokio::test]
            async fn frames_never_coalesce_across_boundaries() {
                let first = frame(b'D', b"first-row-payload");
                let second = frame(b'D', b"second-row-payload-longer");
                let mut bytes = first.clone();
                bytes.extend_from_slice(&second);
                let boundaries = [0, first.len(), bytes.len()];

                // A large fragment lets the socket offer both frames in one
                // read; only the frame bound can stop them from coalescing.
                for fragment in [1, 7, bytes.len()] {
                    let mut stream = fragmented(bytes.clone(), fragment);
                    let chunks = read_chunks(&mut stream, 4096)
                        .await
                        .expect("both frames delivered");

                    assert_eq!(chunks.concat(), bytes, "fragment {fragment}");
                    let mut offset = 0;
                    for chunk in &chunks {
                        assert!(!chunk.is_empty());
                        let end = offset + chunk.len();
                        let frame_index = boundaries
                            .iter()
                            .rposition(|start| *start <= offset)
                            .expect("enclosing frame");
                        assert!(
                            end <= boundaries[frame_index + 1],
                            "fragment {fragment}: read {offset}..{end} crossed a frame boundary"
                        );
                        offset = end;
                    }
                    assert_eq!(offset, bytes.len(), "fragment {fragment}");
                    assert_eq!(stream.failure.get(), None);
                }
            }

            /// Scenario: A zero-body control frame precedes a normal data frame.
            /// Guarantees: The empty frame completes on its own, resets framing state, and the next frame still arrives intact.
            #[tokio::test]
            async fn zero_body_frame_completes_and_resets_state() {
                let empty = frame(b'C', b"");
                let next = frame(b'D', b"payload");
                let mut bytes = empty.clone();
                bytes.extend_from_slice(&next);

                for fragment in [1, bytes.len()] {
                    let mut stream = fragmented(bytes.clone(), fragment);
                    let chunks = read_chunks(&mut stream, 4096)
                        .await
                        .expect("both frames delivered");

                    assert_eq!(chunks.concat(), bytes, "fragment {fragment}");
                    // The empty frame is delivered alone, so the next frame's
                    // header is never merged into the same read.
                    assert_eq!(chunks[0], empty, "fragment {fragment}");
                    assert_eq!(stream.failure.get(), None);
                }
            }

            /// Scenario: The backend stops sending at a frame boundary, part-way through a header, and part-way through a body.
            /// Guarantees: Only a boundary EOF is clean; both truncations fail closed as unavailable transport.
            #[tokio::test]
            async fn truncated_frames_fail_closed_but_boundary_eof_is_clean() {
                let complete = frame(b'D', b"payload");

                let mut stream = fragmented(complete.clone(), 1);
                let chunks = read_chunks(&mut stream, 4096)
                    .await
                    .expect("complete frame");
                assert_eq!(chunks.concat(), complete);
                assert_eq!(stream.failure.get(), None);

                let mut stream = fragmented(complete[..3].to_vec(), 1);
                let mut output = [0; 64];
                let error = stream
                    .read(&mut output)
                    .await
                    .expect_err("truncated header");
                assert_eq!(error.kind(), io::ErrorKind::UnexpectedEof);
                assert_eq!(stream.failure.get(), Some(Error::Unavailable));

                let mut stream = fragmented(complete[..complete.len() - 1].to_vec(), 1);
                let error = read_chunks(&mut stream, 4096)
                    .await
                    .expect_err("truncated body");
                assert_eq!(error.kind(), io::ErrorKind::UnexpectedEof);
                assert_eq!(stream.failure.get(), Some(Error::Unavailable));
            }

            /// Scenario: Declared frame lengths sit exactly at the receive ceiling, one byte below it, and below the minimum legal length.
            /// Guarantees: The ceiling itself is rejected, the largest legal frame is admitted verbatim, and malformed lengths fail closed.
            #[tokio::test]
            async fn frame_length_bounds_are_enforced_exactly() {
                let limits = FrameLimits::new(1024).expect("configured receive budget");
                let ceiling = limits.for_tag(b'D');
                let mut output = [0; 64];

                let mut at_ceiling = vec![b'D'];
                at_ceiling.extend_from_slice(&(ceiling as i32).to_be_bytes());
                let mut stream = fragmented_limited(at_ceiling, 1, limits);
                assert!(stream.read(&mut output).await.is_err());
                assert_eq!(stream.failure.get(), Some(Error::Limit));

                let largest = frame(b'D', &vec![7; ceiling - 1 - 4]);
                let mut stream = fragmented_limited(largest.clone(), 4096, limits);
                let chunks = read_chunks(&mut stream, 8192)
                    .await
                    .expect("largest legal frame");
                assert_eq!(chunks.concat(), largest);
                assert_eq!(stream.failure.get(), None);

                let mut malformed = vec![b'D'];
                malformed.extend_from_slice(&3i32.to_be_bytes());
                let mut stream = fragmented_limited(malformed, 1, limits);
                assert!(stream.read(&mut output).await.is_err());
                assert_eq!(stream.failure.get(), Some(Error::Database));
            }
        }
    };
    (worker) => {
        mod tests {
            use super::*;

            /// Scenario: One large row is followed by ten thousand small rows.
            /// Guarantees: The outlier ages out after the bounded window, target returns to 300, and estimated native requests stay bounded.
            #[test]
            fn fetch_size_recovers_after_recent_small_rows() {
                let mut size = FetchSize::default();
                size.observe(1024 * 1024);
                assert_eq!(size.target(300, 300 * 1024), 0);
                for _ in 0..FETCH_SIZE_WINDOW - 1 {
                    size.observe(1024);
                }
                assert_eq!(size.target(300, 300 * 1024), 0);
                size.observe(1024);
                assert_eq!(size.target(300, 300 * 1024), 300);

                let mut size = FetchSize::default();
                size.observe(1024 * 1024);
                let mut remaining = 10_000;
                let mut adaptive_requests = 0;
                while remaining > 0 {
                    let fetched = size.target(remaining.min(300), 300 * 1024).max(1);
                    for _ in 0..fetched {
                        size.observe(1024);
                    }
                    remaining -= fetched;
                    adaptive_requests += 1;
                }
                let lifetime_requests = 10_000;
                assert_eq!(adaptive_requests, 66);
                assert_eq!(lifetime_requests / adaptive_requests, 151);
            }

            /// Scenario: PostgreSQL rebuilds an equivalent unique index with a new internal OID.
            /// Guarantees: Physical index identity changes are accepted, while column-contract changes still fail closed.
            #[test]
            fn equivalent_rebuilt_index_preserves_cached_schema() {
                use tokio_postgres::types::Type;

                let validated = super::super::tests::test_validated();
                let mut signature = super::super::tests::mock_signature(
                    &validated.plan,
                    &[(Type::TIMESTAMPTZ, 6, true), (Type::INT8, -1, true)],
                );
                let mut cache = None;
                _ = resolve_schema(&mut cache, &validated, signature.clone())
                    .expect("initial schema");
                signature.keys[0].1[0].0 = 999;
                _ = resolve_schema(&mut cache, &validated, signature.clone())
                    .expect("equivalent rebuilt index");
                signature
                    .columns
                    .get_mut(&validated.plan.projections[0])
                    .expect("timestamp")
                    .type_oid = Type::TEXT.oid();
                assert!(resolve_schema(&mut cache, &validated, signature).is_err());
            }

            /// Scenario: RLS is enabled after catalog validation but before query execution.
            /// Guarantees: Every session requests fail-closed PostgreSQL RLS behavior rather than filtered rows.
            #[test]
            fn session_options_disable_filtered_rls_results() {
                let options = session_options(Duration::from_secs(30));
                assert!(options.contains("-c row_security=off"));
                assert!(options.contains("-c statement_timeout=30000"));
                assert!(options.contains("-c idle_in_transaction_session_timeout=30000"));
            }

            /// Scenario: A fetch and its local connection driver both remain silent after timeout and cancellation.
            /// Guarantees: The pending driver is aborted and observably joined, then retry waits for the full server statement-timeout fence.
            #[tokio::test(start_paused = true)]
            async fn silent_fetch_recovers_after_server_query_fence() {
                use futures::future;

                tokio::task::LocalSet::new()
                    .run_until(async {
                        let mut adapter = PostgreSqlAdapter::new(
                            super::super::tests::test_validated(),
                            super::super::tests::test_provider(),
                        )
                        .expect("adapter");
                        adapter.session = Some(Session {
                            client: None,
                            driver: tokio::task::spawn_local(future::pending()),
                            statement: None,
                            catalog: None,
                            receive_failure: ReceiveFailure::default(),
                        });
                        let operation = Operation::default();
                        let cancel = adapter.cancel.clone();
                        let completion = {
                            let completion = await_operation(
                                future::pending::<Result<Reply>>(),
                                &operation,
                                &cancel,
                                Duration::from_secs(30),
                            );
                            tokio::pin!(completion);
                            assert!(futures::poll!(&mut completion).is_pending());
                            tokio::time::advance(Duration::from_secs(30)).await;
                            assert!(futures::poll!(&mut completion).is_pending());
                            tokio::time::advance(CANCEL_COMPLETION_TIMEOUT).await;
                            completion.await
                        };
                        assert!(matches!(
                            completion,
                            OperationCompletion::Incomplete(Error::Timeout)
                        ));

                        let error = {
                            let recovery = adapter.recover_incomplete_operation(Error::Timeout);
                            tokio::pin!(recovery);
                            assert!(futures::poll!(&mut recovery).is_pending());
                            tokio::time::advance(CONNECTION_SHUTDOWN_TIMEOUT).await;
                            recovery.await
                        };
                        assert_eq!(error, Error::Timeout);
                        assert!(!adapter.poisoned);
                        assert!(adapter.session.is_none());

                        let reconnect_operation = Operation::default();
                        {
                            let reconnect = adapter.await_reconnect_fence(&reconnect_operation);
                            tokio::pin!(reconnect);
                            assert!(futures::poll!(&mut reconnect).is_pending());
                            tokio::time::advance(Duration::from_secs(29)).await;
                            assert!(futures::poll!(&mut reconnect).is_pending());
                            tokio::time::advance(Duration::from_secs(1)).await;
                            reconnect.await.expect("server query fenced");
                        }
                        assert!(adapter.reconnect_not_before.is_none());
                    })
                    .await;
            }

            fn test_adapter() -> PostgreSqlAdapter {
                PostgreSqlAdapter::new(
                    super::super::tests::test_validated(),
                    super::super::tests::test_provider(),
                )
                .expect("adapter")
            }

            /// Installs a session whose driver task has already finished with `result`.
            async fn finished_session(adapter: &mut PostgreSqlAdapter, result: Result<()>) {
                adapter.session = Some(Session {
                    client: None,
                    driver: tokio::task::spawn_local(async move { result }),
                    statement: None,
                    catalog: None,
                    receive_failure: ReceiveFailure::default(),
                });
                // Let the driver task reach completion so the close path joins a
                // finished task rather than racing its shutdown timeout.
                tokio::task::yield_now().await;
            }

            /// Scenario: An in-flight operation is dropped without completing, then shutdown joins a driver that finished cleanly.
            /// Guarantees: Shutdown reports unconfirmed cleanup, so the controller cannot clear abandonment and release source ownership.
            #[tokio::test(start_paused = true)]
            async fn shutdown_after_abandoned_operation_reports_unconfirmed_cleanup() {
                use otel_arrow_dfe_scraper::database::DriverAdapter;

                tokio::task::LocalSet::new()
                    .run_until(async {
                        let mut adapter = test_adapter();
                        let guard =
                            super::super::adapter::OperationGuard::start(adapter.operation.clone())
                                .expect("operation started");
                        // Dropping an active guard is exactly what happens when the
                        // controller abandons a request whose join deadline expired.
                        drop(guard);
                        finished_session(&mut adapter, Ok(())).await;

                        assert_eq!(adapter.shutdown().await, Err(Error::Cleanup));
                        assert!(adapter.poisoned);
                        // The terminal result is cached and ownership stays quarantined.
                        assert_eq!(adapter.shutdown().await, Err(Error::Cleanup));
                        assert!(adapter.begin_operation().is_err());
                    })
                    .await;
            }

            /// Scenario: A receiver with no abandoned operation shuts down while its driver finishes cleanly.
            /// Guarantees: Confirmed cleanup still reports success, so ordinary stops release source ownership instead of quarantining it.
            #[tokio::test(start_paused = true)]
            async fn shutdown_without_abandonment_confirms_cleanup() {
                use otel_arrow_dfe_scraper::database::DriverAdapter;

                tokio::task::LocalSet::new()
                    .run_until(async {
                        let mut adapter = test_adapter();
                        finished_session(&mut adapter, Ok(())).await;

                        assert_eq!(adapter.shutdown().await, Ok(()));
                        assert!(!adapter.poisoned);
                        assert!(adapter.session.is_none());
                    })
                    .await;
            }

            /// Scenario: A connection driver fails with a transport error while a statement is in flight.
            /// Guarantees: The failed driver closes as aborted and fences reconnect for the full statement timeout instead of looking graceful.
            #[tokio::test(start_paused = true)]
            async fn failed_driver_during_operation_installs_server_query_fence() {
                tokio::task::LocalSet::new()
                    .run_until(async {
                        let mut adapter = test_adapter();
                        finished_session(&mut adapter, Err(Error::Unavailable)).await;

                        let close = adapter
                            .close_session(CloseReason::OperationInFlight)
                            .await
                            .expect("session closed");
                        assert_eq!(close, SessionClose::Aborted);
                        assert!(!adapter.poisoned);
                        assert!(adapter.reconnect_not_before.is_some());

                        // A replacement session waits out the server statement timeout.
                        let operation = Operation::default();
                        {
                            let fence = adapter.await_reconnect_fence(&operation);
                            tokio::pin!(fence);
                            assert!(futures::poll!(&mut fence).is_pending());
                            tokio::time::advance(Duration::from_secs(29)).await;
                            assert!(futures::poll!(&mut fence).is_pending());
                            tokio::time::advance(Duration::from_secs(1)).await;
                            fence.await.expect("server query fenced");
                        }
                        assert!(adapter.reconnect_not_before.is_none());
                    })
                    .await;
            }

            /// Scenario: A stop arrives while a server-query fence from an interrupted statement is still outstanding.
            /// Guarantees: Shutdown reports unconfirmed cleanup, so source ownership is quarantined rather than released while an orphaned backend statement may still run.
            #[tokio::test(start_paused = true)]
            async fn shutdown_during_server_query_fence_reports_unconfirmed_cleanup() {
                use otel_arrow_dfe_scraper::database::DriverAdapter;

                tokio::task::LocalSet::new()
                    .run_until(async {
                        let mut adapter = test_adapter();
                        finished_session(&mut adapter, Err(Error::Unavailable)).await;
                        let close = adapter
                            .close_session(CloseReason::OperationInFlight)
                            .await
                            .expect("session closed");
                        assert_eq!(close, SessionClose::Aborted);
                        assert!(adapter.reconnect_not_before.is_some());

                        // Stopping inside the fence window cannot prove the server
                        // statement ended, so cleanup stays unconfirmed.
                        assert_eq!(adapter.shutdown().await, Err(Error::Cleanup));
                        assert!(adapter.poisoned);
                    })
                    .await;
            }

            /// Scenario: A stop arrives after an earlier server-query fence has already elapsed.
            /// Guarantees: An expired fence does not quarantine source ownership, so a later stop still reports confirmed cleanup.
            #[tokio::test(start_paused = true)]
            async fn shutdown_after_server_query_fence_elapses_confirms_cleanup() {
                use otel_arrow_dfe_scraper::database::DriverAdapter;

                tokio::task::LocalSet::new()
                    .run_until(async {
                        let mut adapter = test_adapter();
                        finished_session(&mut adapter, Err(Error::Unavailable)).await;
                        _ = adapter
                            .close_session(CloseReason::OperationInFlight)
                            .await
                            .expect("session closed");
                        assert!(adapter.reconnect_not_before.is_some());

                        tokio::time::advance(Duration::from_secs(30)).await;
                        assert_eq!(adapter.shutdown().await, Ok(()));
                        assert!(!adapter.poisoned);
                    })
                    .await;
            }

            /// Scenario: An idle session's driver reports a transport failure during teardown.
            /// Guarantees: No statement can outlive an idle driver, so reconnect is not delayed by an unnecessary fence.
            #[tokio::test(start_paused = true)]
            async fn failed_driver_while_idle_does_not_fence() {
                tokio::task::LocalSet::new()
                    .run_until(async {
                        let mut adapter = test_adapter();
                        finished_session(&mut adapter, Err(Error::Unavailable)).await;

                        let close = adapter
                            .close_session(CloseReason::Idle)
                            .await
                            .expect("session closed");
                        assert_eq!(close, SessionClose::Graceful);
                        assert!(!adapter.poisoned);
                        assert!(adapter.reconnect_not_before.is_none());
                    })
                    .await;
            }

            /// Scenario: A driver ends without error while closing an in-flight operation's session.
            /// Guarantees: A clean driver exit stays graceful and installs no fence, keeping routine retries prompt.
            #[tokio::test(start_paused = true)]
            async fn clean_driver_exit_stays_graceful_during_operation() {
                tokio::task::LocalSet::new()
                    .run_until(async {
                        let mut adapter = test_adapter();
                        finished_session(&mut adapter, Ok(())).await;

                        let close = adapter
                            .close_session(CloseReason::OperationInFlight)
                            .await
                            .expect("session closed");
                        assert_eq!(close, SessionClose::Graceful);
                        assert!(adapter.reconnect_not_before.is_none());
                    })
                    .await;
            }
        }
    };
    () => {
        mod tests {
            use super::*;

            /// Scenario: PostgreSQL reports overload, transaction conflicts, timeout, or permanent failures.
            /// Guarantees: Transient SQLSTATEs and timeouts retry through the shared policy; auth/schema errors stay terminal.
            #[test]
            fn sqlstate_retry_policy() {
                for code in [
                    "53300", "08006", "55P03", "57P01", "57P02", "57P03", "57P05", "40001", "40P01",
                ] {
                    let error = sqlstate(code);
                    assert_eq!(error, Error::Unavailable);
                    assert!(PostgreSqlAdapter::is_retryable(&error));
                }
                let timeout = sqlstate("57014");
                assert_eq!(timeout, Error::Timeout);
                assert!(PostgreSqlAdapter::is_retryable(&timeout));
                for code in ["28P01", "28000"] {
                    let error = sqlstate(code);
                    assert_eq!(error, Error::Credential);
                    assert!(!PostgreSqlAdapter::is_retryable(&error));
                }
                for code in ["42501", "42P01", "23505", "53200", "XX000"] {
                    let error = sqlstate(code);
                    assert_eq!(error, Error::Database);
                    assert!(!PostgreSqlAdapter::is_retryable(&error));
                }
                for error in [
                    Error::Limit,
                    Error::Value,
                    Error::Credential,
                    Error::Cleanup,
                ] {
                    assert!(!PostgreSqlAdapter::is_retryable(&error));
                }
            }
        }
    };
}

use super::{
    config::PostgreSqlReceiverConfig as Config,
    query::{NativeColumn, Plan, Signature, compatible_literal},
    value as convert,
};
use otel_arrow_dfe_engine::capability::{
    CapabilityError,
    auth::{BasicAuthCredential, basic_auth_provider::BasicAuthCredentialStream},
};
use otel_arrow_dfe_engine::local::capability::auth::basic_auth_provider::BasicAuthProvider;
use serde_json::{Value, json};
use sqlparser::ast::Value as SqlValue;
use tokio_postgres::types::Type;

struct TestCredentials;

fn fixed_credential() -> BasicAuthCredential {
    BasicAuthCredential::new("reader", "password").expect("fixture")
}

#[async_trait::async_trait(?Send)]
impl BasicAuthProvider for TestCredentials {
    async fn get_credential(&self) -> Result<BasicAuthCredential, CapabilityError> {
        Ok(fixed_credential())
    }

    fn credential_stream(&self) -> BasicAuthCredentialStream {
        Box::pin(futures::stream::iter([fixed_credential()]))
    }
}

pub(super) fn test_provider() -> Box<dyn BasicAuthProvider> {
    Box::new(TestCredentials)
}

const KEY: &str =
    "((e.ts > :last_timestamp) OR (e.ts = :last_timestamp AND e.id > :last_tie_breaker))";

fn config(sql: &str) -> Value {
    let root = std::env::current_dir().expect("working directory");
    json!({
        "source_id": "test",
        "connection": {"host": "LOCALHOST", "database": "fixture"},
        "watermark": {
            "mode":"composite",
            "timestamp":{"column":"ts","initial":"2026-10-05T00:00:00.000000Z","timezone":"UTC"},
            "tie_breaker":{"column":"id","initial":0}
        },
        "query": {
            "statement": sql, "interval": "60s",
            "max_rows_per_poll": 1000, "max_batch_bytes": "8 MiB"
        },
        "checkpoint":{"directory":root.join("checkpoint"),"on_permanent_nack":"pause"}
    })
}
fn sql(predicate: &str) -> String {
    format!("SELECT e.ts, e.id FROM public.events AS e WHERE {predicate} ORDER BY e.ts, e.id")
}
fn validate(value: &Value) -> super::adapter::Result<super::config::Validated> {
    Config::parse(value)?.validate()
}

pub(super) fn test_validated() -> super::config::Validated {
    validate(&config(&sql(KEY))).expect("worker fixture")
}

pub(super) fn mock_signature(plan: &Plan, columns: &[(Type, i32, bool)]) -> Signature {
    assert_eq!(plan.projections.len(), columns.len());
    Signature {
        columns: plan
            .projections
            .iter()
            .zip(columns)
            .enumerate()
            .map(|(index, (reference, (native, modifier, not_null)))| {
                (
                    reference.clone(),
                    NativeColumn {
                        oid: 42,
                        attribute: index as i16 + 1,
                        type_oid: native.oid(),
                        modifier: *modifier,
                        not_null: *not_null,
                        collation: 0,
                    },
                )
            })
            .collect(),
        keys: vec![(42, vec![(43, vec![1, 2])])],
    }
}

/// Scenario: Legal SQL outside the experiment or an OR-bypass weakens the keyset contract.
/// Guarantees: Rejected statements never reach native prepare, including all extra query/select clauses.
#[test]
fn unsafe_and_unsupported_sql_rejected() {
    let base = sql(KEY);
    for text in [
        sql(&format!("{KEY} OR TRUE")),
        sql(&format!("{KEY} AND ({KEY})")),
        sql(&format!(
            "{KEY} AND (e.ts, e.id) > (:last_timestamp, :last_tie_breaker)"
        )),
        sql("e.ts >= :last_timestamp OR (e.ts = :last_timestamp AND e.id > :last_tie_breaker)"),
        sql("e.ts > :last_timestamp OR (e.ts = :last_timestamp AND e.id >= :last_tie_breaker)"),
        sql("e.ts > :last_timestamp OR (e.id > :last_tie_breaker AND e.ts = :last_timestamp)"),
        sql("e.ts > :last_timestamp AND e.id > :last_tie_breaker"),
        base.replace("e.ts, e.id FROM", "e.ts, abs(e.id) AS id FROM"),
        base.replace("SELECT", "SELECT DISTINCT"),
        base.replace(" FROM", " INTO public.copy FROM"),
        base.replace("e.ts, e.id FROM", "* FROM"),
        base.replace("public.events AS e", "(SELECT * FROM public.events) AS e"),
        base.replace(
            "public.events AS e",
            "public.events AS e LEFT JOIN public.x AS x ON e.id=x.id",
        ),
        base.replace("ORDER BY e.ts, e.id", "ORDER BY e.id, e.ts"),
        base.replace("ORDER BY e.ts, e.id", "ORDER BY e.ts DESC, e.id"),
        base.replace("ORDER BY e.ts, e.id", "ORDER BY e.ts NULLS FIRST, e.id"),
        format!("{base} LIMIT 1"),
        format!(
            "{base} LIMIT {}",
            validate(&config(&base)).expect("valid").common.max_rows()
        ),
        format!("{base} OFFSET 1"),
        format!("{base} FETCH FIRST 1 ROW ONLY"),
        format!("{base} FOR UPDATE"),
        format!("{base}; SELECT 1"),
        format!("{base} UNION {base}"),
        base.replace("ORDER BY", "GROUP BY e.ts, e.id ORDER BY"),
        format!("WITH x AS (DELETE FROM public.events) {base}"),
        sql(&format!(
            "{KEY} AND e.id = pg_catalog.set_config('a', 'b', false)"
        )),
    ] {
        assert!(validate(&config(&text)).is_err(), "{text}");
    }
}

/// Scenario: Static date and timestamp filters use text literals across reconnects and restarts.
/// Guarantees: Relative temporal literals are rejected so replay cannot silently skip unacknowledged rows.
#[test]
fn relative_temporal_literals_rejected() {
    for (oid, literal, valid) in [
        (Type::DATE.oid(), "2026-10-05", true),
        (Type::DATE.oid(), "today", false),
        (Type::TIMESTAMP.oid(), "2026-10-05 00:00:00", true),
        (Type::TIMESTAMP.oid(), "2026-10-05T00:00:00.123456", true),
        (Type::TIMESTAMP.oid(), "2026-10-05T00:00:00Z", true),
        (Type::TIMESTAMP.oid(), "now", false),
        (Type::TIMESTAMPTZ.oid(), "2026-10-05T00:00:00Z", true),
        (Type::TIMESTAMPTZ.oid(), "2026-10-05T00:00:00-07:00", true),
        (Type::TIMESTAMPTZ.oid(), "today", false),
        (Type::TIMESTAMPTZ.oid(), "epoch", false),
    ] {
        let literal = SqlValue::SingleQuotedString(literal.into());
        assert_eq!(compatible_literal(oid, &literal).is_ok(), valid);
    }
}

/// Scenario: Temporal cursors and native integer ties approach their precision/range boundaries.
/// Guarantees: Native binding cannot silently truncate submicroseconds, typmod digits, leap seconds, or integers.
#[test]
fn cursor_precision_and_range() {
    for (text, precision, valid) in [
        ("2026-10-05T00:00:00.123000Z", 3, true),
        ("2026-10-05T00:00:00.123001Z", 3, false),
        ("2026-10-05T00:00:00.123456789Z", 6, false),
        ("2026-10-05T00:00:00.123456000Z", 6, false),
        ("2026-02-30T00:00:00Z", 6, false),
        ("2026-10-05T00:00:60Z", 6, false),
        ("infinity", 6, false),
    ] {
        assert_eq!(convert::cursor_time(text, precision).is_ok(), valid);
    }
    assert!(convert::check_tie(32767, "int2").is_ok());
    assert!(convert::check_tie(32768, "int2").is_err());
    assert!(convert::check_tie(i32::MIN as i64, "int4").is_ok());
    assert!(convert::check_tie(i32::MIN as i64 - 1, "int4").is_err());
    assert!(convert::check_tie(i64::MAX, "int8").is_ok());
}

/// Scenario: Equivalent UTC instants use RFC 3339 offsets, canonical Z text, or timezone-less UTC text.
/// Guarantees: Configuration, compiled cursors, and fingerprints use one exact six-digit UTC representation.
#[test]
fn canonicalizes_exact_postgresql_timestamp_values() {
    let mut baseline = None;
    for value in [
        "2026-01-01T00:00:00.123456Z",
        "2026-01-01T00:00:00.123456000Z",
        "2026-01-01T02:00:00.123456000+02:00",
        "2026-01-01 00:00:00.123456",
        "2026-01-01 00:00:00.123456000",
    ] {
        let mut raw = config(&sql(KEY));
        raw["watermark"]["timestamp"]["initial"] = json!(value);
        raw["watermark"]["timestamp"]["timezone"] = json!("utc");
        let validated = validate(&raw).expect("exact PostgreSQL timestamp");
        let watermark = validated
            .common
            .watermark()
            .as_composite()
            .expect("composite watermark");
        assert_eq!(watermark.initial.timestamp, "2026-01-01T00:00:00.123456Z");
        let super::config::Watermark::Composite { timestamp, .. } = &validated.config.watermark;
        assert_eq!(timestamp.initial, "2026-01-01T00:00:00.123456Z");
        assert_eq!(timestamp.timezone, "UTC");
        if let Some(fingerprint) = &baseline {
            assert_eq!(&validated.fingerprint, fingerprint);
        } else {
            baseline = Some(validated.fingerprint);
        }
    }
}

/// Scenario: PostgreSQL initial timestamps would round, overflow supported years, or encode invalid time.
/// Guarantees: Customer timestamp input is rejected rather than rounded or interpreted in a host timezone.
#[test]
fn rejects_lossy_or_invalid_postgresql_timestamp_values() {
    for value in [
        "2026-01-01T00:00:00.1234567Z",
        "2026-01-01T00:00:00.123456789Z",
        "2026-01-01T00:00:00.1234560001Z",
        "2026-01-01T02:00:00.123456700+02:00",
        "2026-01-01 00:00:00.1234567",
        "0000-01-01T00:00:00Z",
        "+10000-01-01T00:00:00Z",
        "0001-01-01T00:00:00+01:00",
        "2016-12-31T23:59:60Z",
        "2026-01-01T00:00:00+99:99",
        "not-a-timestamp",
    ] {
        let mut raw = config(&sql(KEY));
        raw["watermark"]["timestamp"]["initial"] = json!(value);
        assert!(validate(&raw).is_err(), "{value}");
    }
}

fn number(weight: i16, sign: u16, scale: u16, digits: &[u16]) -> Vec<u8> {
    let mut bytes = (digits.len() as i16).to_be_bytes().to_vec();
    bytes.extend_from_slice(&weight.to_be_bytes());
    bytes.extend_from_slice(&sign.to_be_bytes());
    bytes.extend_from_slice(&scale.to_be_bytes());
    for digit in digits {
        bytes.extend_from_slice(&digit.to_be_bytes());
    }
    bytes
}

/// Scenario: NUMERIC contains signed integers, fractions, scale, malformed groups, or extreme weights.
/// Guarantees: Base-10000 decoding stays exact and bounded without float conversion or scale loss.
#[test]
fn exact_numeric() {
    for (bytes, expected) in [
        (
            number(3, 0, 0, &[9007, 1992, 5474, 993]),
            "9007199254740993",
        ),
        (number(0, 0x4000, 4, &[12, 3400]), "-12.3400"),
        (number(-2, 0, 8, &[1]), "0.00000001"),
        (number(0, 0, 3, &[]), "0.000"),
        (number(1, 0, 0, &[1]), "10000"),
    ] {
        assert_eq!(convert::numeric(&bytes).expect("numeric"), expected);
    }
    for bytes in [
        number(0, 0xC000, 0, &[]),
        number(0, 0, 0, &[10000]),
        number(-1, 0, 2, &[1234]),
        number(32767, 0, 0, &[1]),
        vec![0; 7],
    ] {
        assert!(convert::numeric(&bytes).is_err());
    }
    assert_eq!(
        convert::numeric(&number(4095, 0, 0, &[1000]))
            .expect("boundary")
            .len(),
        16384
    );
    assert!(convert::numeric(&number(4096, 0, 0, &[1])).is_err());
}
