// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Live tests for the existing disposable TLS fixture, explicitly run by ci/run.py.

use super::super::config::{PostgreSqlReceiverConfig, Validated};
use super::*;
use otel_arrow_dfe_engine::capability::{
    CapabilityError, auth::basic_auth_provider::BasicAuthCredentialStream,
};
use otel_arrow_dfe_engine::local::capability::auth::basic_auth_provider::BasicAuthProvider;
use otel_arrow_dfe_scraper::database::{CellValue, DriverAdapter};
use serde_json::{Value, json};
use std::path::Path;

struct Credentials;

#[async_trait::async_trait(?Send)]
impl BasicAuthProvider for Credentials {
    async fn get_credential(&self) -> std::result::Result<BasicAuthCredential, CapabilityError> {
        Ok(BasicAuthCredential::new("otel_reader", "otel-reader-password").expect("fixture"))
    }

    fn credential_stream(&self) -> BasicAuthCredentialStream {
        Box::pin(futures::stream::empty())
    }
}

fn config(mode: &str, state: &Path) -> Value {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/receivers/postgresql_receiver/fixtures/tls/out/ca.pem");
    let ca_file = std::env::var_os("OTEL_ARROW_POSTGRESQL_TEST_CA_FILE")
        .map(std::path::PathBuf::from)
        .unwrap_or(fixture);
    assert!(
        ca_file.is_file(),
        "provision the PostgreSQL TLS fixture first"
    );
    let port = std::env::var("OTEL_ARROW_POSTGRESQL_TEST_PORT")
        .map(|value| value.parse::<u16>().expect("valid live fixture port"))
        .unwrap_or(5432);
    let initial = "2026-10-05T00:00:00.000000Z";
    let (predicate, watermark) = match mode {
        "int64" => (
            "WHERE e.event_id > :key ORDER BY e.event_id",
            json!({"mode":"scalar","column":"event_id","bind":"key","initial":{"type":"int64","value":0}}),
        ),
        "timestamp" => (
            "WHERE e.scalar_ts > :key ORDER BY e.scalar_ts",
            json!({"mode":"scalar","column":"scalar_ts","bind":"key","initial":{"type":"timestamp","value":initial}}),
        ),
        "composite" => (
            "WHERE e.event_ts > :ts OR (e.event_ts = :ts AND e.event_id > :id) ORDER BY e.event_ts, e.event_id",
            json!({"mode":"composite","timestamp":{"column":"event_ts","bind":"ts","initial":initial,"timezone":"UTC"},"tie_breaker":{"column":"event_id","bind":"id","initial":0}}),
        ),
        "snapshot" => ("ORDER BY e.event_id", json!({"mode":"snapshot"})),
        _ => panic!("unsupported test mode"),
    };
    json!({
        "source_id":"postgresql-live",
        "connection":{"host":"localhost","port":port,"database":"receiver_fixture","tls":{"ca_file":ca_file}},
        "query":{
            "statement":format!("SELECT e.event_id, e.event_ts, e.actor, e.scalar_ts FROM public.receiver_events AS e {predicate}"),
            "interval":"1m","fetch_size_rows":1,"max_rows_per_poll":match mode {"snapshot" => 3, "composite" => 1, _ => 2},
            "max_batch_bytes":"10 MiB","catch_up":{"max_pages":1}
        },
        "watermark":watermark,
        "checkpoint":{"directory":state,"nack_backoff":"1ms"}
    })
}

fn validated(raw: &Value) -> Validated {
    PostgreSqlReceiverConfig::parse(raw)
        .and_then(PostgreSqlReceiverConfig::validate)
        .expect("live receiver config")
}

fn adapter(raw: &Value) -> PostgreSqlAdapter {
    PostgreSqlAdapter::new(validated(raw), Box::new(Credentials)).expect("adapter")
}

async fn bounded(future: impl Future<Output = ()>) {
    otel_arrow_dfe_otap::crypto::ensure_crypto_provider();
    tokio::task::LocalSet::new()
        .run_until(async {
            // Real socket and server state cannot use a paused Tokio clock.
            tokio::time::timeout(Duration::from_secs(60), future)
                .await
                .expect("live PostgreSQL scenario completed within 60s");
        })
        .await;
}

async fn shutdown(adapter: &mut PostgreSqlAdapter) {
    tokio::time::timeout(Duration::from_secs(5), adapter.shutdown())
        .await
        .expect("bounded driver shutdown")
        .expect("confirmed session cleanup");
}

/// Scenario: Verified TLS polls integer/timestamp scalar, duplicate-timestamp composite, and snapshot modes.
/// Guarantees: Native binds preserve exact values and microseconds across pages; snapshot overflow returns no partial page.
#[tokio::test]
#[ignore = "requires the PostgreSQL TLS fixture; run ci/run.py test"]
async fn live_verified_tls_polling_modes() {
    bounded(async {
        let state = tempfile::tempdir().expect("checkpoint directory");
        for mode in ["int64", "timestamp", "composite", "snapshot"] {
            let raw = config(mode, state.path());
            let query = validated(&raw).common;
            let mut adapter = adapter(&raw);
            _ = adapter.begin_operation().expect("metadata operation");
            assert_eq!(
                adapter
                    .validate_query(&query)
                    .await
                    .expect("metadata")
                    .len(),
                4
            );
            let mut cursor = query.watermark().initial();
            let mut rows = Vec::new();
            let mut page_sizes = Vec::new();
            loop {
                _ = adapter.begin_operation().expect("page operation");
                let page = adapter.execute(&query, &cursor).await.expect("native page");
                if page.rows.is_empty() {
                    break;
                }
                page_sizes.push(page.rows.len());
                cursor = page.rows.last().expect("page cursor").cursor.clone();
                rows.extend(page.rows);
                if mode == "snapshot" {
                    break;
                }
                assert!(rows.len() <= 3, "keyset advances without replay");
            }
            assert_eq!(rows.len(), 3);
            assert_eq!(
                page_sizes,
                match mode {
                    "snapshot" => vec![3],
                    "composite" => vec![1, 1, 1],
                    _ => vec![2, 1],
                }
            );
            if mode == "composite" {
                assert_eq!(
                    rows[0].cursor,
                    Cursor::composite("2026-10-05T00:00:01.000000Z".into(), 1)
                );
                assert_eq!(
                    rows[1].cursor,
                    Cursor::composite("2026-10-05T00:00:01.000000Z".into(), 2)
                );
            }
            for (index, row) in rows.iter().enumerate() {
                let timestamp = if index < 2 {
                    "2026-10-05T00:00:01.000000Z"
                } else {
                    "2026-10-05T00:00:02.123456Z"
                };
                let scalar_ts = [
                    "2026-10-05T00:00:01.000001Z",
                    "2026-10-05T00:00:01.000002Z",
                    "2026-10-05T00:00:02.123456Z",
                ][index];
                let id = i64::try_from(index + 1).expect("fixture id");
                assert_eq!(
                    row.row.values,
                    vec![
                        CellValue::Int64(id),
                        CellValue::TimestampTz(timestamp.into()),
                        CellValue::String(["alice", "bob", "carol"][index].into()),
                        CellValue::TimestampTz(scalar_ts.into()),
                    ]
                );
                assert_eq!(
                    row.cursor,
                    match mode {
                        "int64" => Cursor::Scalar(ScalarValue::Int64(id)),
                        "timestamp" => Cursor::Scalar(ScalarValue::Timestamp(scalar_ts.into())),
                        "composite" => Cursor::composite(timestamp.into(), id),
                        _ => Cursor::Snapshot,
                    }
                );
            }
            shutdown(&mut adapter).await;
        }
        let mut raw = config("snapshot", state.path());
        raw["query"]["max_rows_per_poll"] = json!(2);
        let query = validated(&raw).common;
        let mut adapter = adapter(&raw);
        _ = adapter.begin_operation().expect("snapshot operation");
        assert_eq!(
            adapter.execute(&query, &Cursor::Snapshot).await.err(),
            Some(Error::SnapshotRowLimit { limit: 2 }),
        );
        shutdown(&mut adapter).await;
    })
    .await;
}

/// Scenario: The registered receiver emits rows, receives NACK/replay/ACK, shuts down, and restarts.
/// Guarantees: Emitted OTLP matches the fixture; only acknowledged cursors/revisions persist and restart skips committed rows.
#[test]
#[ignore = "requires the PostgreSQL TLS fixture; run ci/run.py test"]
fn live_factory_replay_and_restart() {
    use otel_arrow_dfe_config::node::NodeUserConfig;
    use otel_arrow_dfe_engine::{
        capability::{
            auth::basic_auth_provider::BasicAuthProvider as Registration,
            registry::{CapabilityRegistry, LocalCapabilityEntry},
        },
        context::ControllerContext,
        control::{AckMsg, NackMsg, NodeControlMsg},
        testing::{capability::resolve_bindings_for_test, receiver::TestRuntime, test_node},
    };
    use otel_arrow_dfe_otap::{
        pdata::OtapPdata,
        testing::{next_ack, next_nack},
    };
    use otel_arrow_dfe_pdata::proto::opentelemetry::{common::v1::any_value, logs::v1::LogsData};
    use otel_arrow_dfe_scraper::CheckpointStore;
    use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;
    use prost::Message;
    use std::sync::Arc;

    fn bodies(data: &OtapPdata) -> Vec<Vec<(String, any_value::Value)>> {
        let logs = LogsData::decode(data.payload_ref().encoded_bytes().expect("OTLP").as_ref())
            .expect("valid logs");
        logs.resource_logs
            .into_iter()
            .flat_map(|resource| resource.scope_logs)
            .flat_map(|scope| scope.log_records)
            .map(|record| {
                let Some(any_value::Value::KvlistValue(body)) =
                    record.body.and_then(|body| body.value)
                else {
                    panic!("structured row body");
                };
                body.values
                    .into_iter()
                    .map(|field| {
                        (
                            field.key,
                            field
                                .value
                                .and_then(|value| value.value)
                                .expect("field value"),
                        )
                    })
                    .collect()
            })
            .collect()
    }

    otel_arrow_dfe_otap::crypto::ensure_crypto_provider();
    let directory = tempfile::tempdir().expect("checkpoint directory");
    let raw = config("int64", directory.path());
    let fingerprint = validated(&raw).fingerprint;
    for (restart, expected_id, expected_revision) in [(false, 2, 1), (true, 3, 2)] {
        let context = ControllerContext::new(TelemetryRegistryHandle::new()).pipeline_context_with(
            "group".into(),
            "pipeline".into(),
            0,
            1,
            0,
        );
        let runtime = TestRuntime::<OtapPdata>::new();
        let store = CheckpointStore::new(
            directory.path(),
            "group",
            "pipeline",
            runtime.config().name.as_ref(),
            "postgresql-live",
            fingerprint.clone(),
        );
        let mut registry = CapabilityRegistry::new();
        registry
            .register_local(
                std::any::TypeId::of::<Registration>(),
                LocalCapabilityEntry::new("pg_fixture".into(), || -> Box<dyn std::any::Any> {
                    Box::new(Box::new(Credentials) as Box<dyn BasicAuthProvider>)
                }),
            )
            .expect("fixture credentials");
        let capabilities = resolve_bindings_for_test(
            &std::collections::HashMap::from([("basic_auth_provider".into(), "pg_fixture".into())]),
            &registry,
            &std::collections::HashSet::from(["pg_fixture".into()]),
        )
        .expect("credential wiring");
        let mut node = NodeUserConfig::new_receiver_config(super::super::POSTGRESQL_RECEIVER_URN);
        node.config = raw.clone();
        let wrapper = (super::super::POSTGRESQL_RECEIVER.create)(
            context,
            test_node(runtime.config().name.clone()),
            Arc::new(node),
            runtime.config(),
            &capabilities,
        )
        .expect("registered PostgreSQL receiver");
        let check = store.clone();
        let (finished, completed) = tokio::sync::oneshot::channel();
        runtime
            .set_receiver(wrapper)
            .run_test(|ctx| async move {
                let completion = tokio::time::timeout(Duration::from_secs(45), completed).await;
                ctx.send_shutdown(
                    std::time::Instant::now() + Duration::from_secs(5),
                    "live PostgreSQL restart",
                )
                .await
                .expect("shutdown enqueued even if validation fails");
                completion
                    .expect("bounded validation")
                    .expect("validation signalled completion");
            })
            .run_validation_concurrent(move |mut ctx| async move {
                tokio::time::timeout(Duration::from_secs(30), async {
                    let mut data = ctx.recv().await.expect("receiver emits");
                    let emitted = bodies(&data);
                    let ids: Vec<_> = emitted
                        .iter()
                        .map(|body| {
                            body.iter()
                                .find(|(name, _)| name == "event_id")
                                .expect("event id")
                                .1
                                .clone()
                        })
                        .collect();
                    assert_eq!(
                        ids,
                        if restart {
                            vec![any_value::Value::IntValue(3)]
                        } else {
                            vec![any_value::Value::IntValue(1), any_value::Value::IntValue(2)]
                        }
                    );
                    for body in &emitted {
                        let value = |name| {
                            &body
                                .iter()
                                .find(|(key, _)| key == name)
                                .expect("row field")
                                .1
                        };
                        let any_value::Value::IntValue(id) = value("event_id") else {
                            panic!("native integer field");
                        };
                        let index = usize::try_from(id - 1).expect("fixture row");
                        assert_eq!(body.len(), 4);
                        assert_eq!(
                            value("actor"),
                            &any_value::Value::StringValue(["alice", "bob", "carol"][index].into()),
                        );
                        assert_eq!(
                            value("event_ts"),
                            &any_value::Value::StringValue(if index < 2 {
                                "2026-10-05T00:00:01.000000Z".into()
                            } else {
                                "2026-10-05T00:00:02.123456Z".into()
                            }),
                        );
                        assert_eq!(
                            value("scalar_ts"),
                            &any_value::Value::StringValue(
                                [
                                    "2026-10-05T00:00:01.000001Z",
                                    "2026-10-05T00:00:01.000002Z",
                                    "2026-10-05T00:00:02.123456Z",
                                ][index]
                                    .into()
                            ),
                        );
                    }
                    if !restart {
                        assert!(check.read().expect("unacknowledged checkpoint").is_none());
                        let (_, nack) =
                            next_nack(NackMsg::new("live replay", data)).expect("NACK route");
                        ctx.send_control_msg(NodeControlMsg::Nack(nack))
                            .await
                            .expect("NACK");
                        data = ctx.recv().await.expect("replayed page");
                        assert_eq!(bodies(&data), emitted);
                        assert!(check.read().expect("NACK cannot commit").is_none());
                    } else {
                        assert_eq!(
                            check
                                .read()
                                .expect("restart checkpoint")
                                .expect("state")
                                .revision,
                            1
                        );
                    }
                    let (_, ack) = next_ack(AckMsg::new(data)).expect("ACK route");
                    ctx.send_control_msg(NodeControlMsg::Ack(ack))
                        .await
                        .expect("ACK");
                    loop {
                        if check.read().expect("checkpoint read").is_some_and(|state| {
                            state.cursor == Cursor::Scalar(ScalarValue::Int64(expected_id))
                                && state.revision == expected_revision
                        }) {
                            break;
                        }
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    }
                })
                .await
                .expect("emission, replay and durable ACK within deadline");
                finished.send(()).expect("validation finished");
            });
        let checkpoint = store
            .read()
            .expect("durable checkpoint")
            .expect("committed state");
        assert_eq!(
            checkpoint.cursor,
            Cursor::Scalar(ScalarValue::Int64(expected_id))
        );
        assert_eq!(checkpoint.revision, expected_revision);
    }
}

/// Scenario: The owned TLS database cancels a running query, then disconnects an idle reader.
/// Guarantees: Native cancellation is observed; failed polling recovers through the current reconnect fence and revalidation.
#[tokio::test]
#[ignore = "requires the PostgreSQL TLS fixture; run ci/run.py test"]
async fn live_transport_cancellation_and_reconnect() {
    bounded(async {
        let state = tempfile::tempdir().expect("checkpoint directory");
        let raw = config("int64", state.path());
        let query = validated(&raw).common;
        let mut reader = adapter(&raw);
        let mut observer = adapter(&raw);
        for adapter in [&mut reader, &mut observer] {
            _ = adapter.begin_operation().expect("connect operation");
            _ = adapter.validate_query(&query).await.expect("verified TLS");
        }
        {
            let client = reader.session.as_ref().expect("reader").client.as_ref().expect("client");
            let observer = observer.session.as_ref().expect("observer").client.as_ref().expect("client");
            // A statement timeout has the same SQLSTATE as native cancellation.
            // Disable it only for this probe; reconnect restores session defaults.
            client.batch_execute("SET statement_timeout = 0").await.expect("disable probe timeout");
            let pid: i32 = client.query_one("SELECT pg_backend_pid()", &[]).await.expect("reader PID").get(0);
            let (cancelled, ()) = tokio::join!(
                client.simple_query("SELECT pg_sleep(30)"),
                async {
                    loop {
                        let row = observer.query_one(
                            "SELECT EXISTS (SELECT 1 FROM pg_stat_activity WHERE pid = $1 AND wait_event = 'PgSleep')",
                            &[&pid],
                        ).await.expect("owned query state");
                        if row.get::<_, bool>(0) {
                            break;
                        }
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    }
                    cancel_native(&reader.cancel).await;
                },
            );
            assert_eq!(cancelled.expect_err("server observed cancellation").code(), Some(&tokio_postgres::error::SqlState::QUERY_CANCELED));
            let row = observer.query_one("SELECT pg_terminate_backend($1)", &[&pid]).await.expect("disconnect owned reader");
            assert!(row.get::<_, bool>(0));
        }
        while !reader.session.as_ref().expect("reader").driver.is_finished() {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        _ = reader.begin_operation().expect("disconnected poll operation");
        let error = reader.execute(&query, &query.watermark().initial()).await.expect_err("disconnected session cannot emit");
        assert!(PostgreSqlAdapter::is_retryable(&error), "production controller retries this transport error");
        // Existing behavior conservatively fences even this already-dead idle
        // session. Do not bypass the fence; shutdown during it reports Cleanup.
        assert!(reader.reconnect_not_before.is_some_and(|deadline| deadline > Instant::now()));
        // Match the controller's real retry policy (1s, doubling up to 30s).
        // Waiting on the fence consumes the reconnect operation's timeout too,
        // so a retryable Timeout can require another reconnect attempt.
        let mut backoff = Duration::from_secs(1);
        loop {
            tokio::time::sleep(backoff).await;
            _ = reader.begin_operation().expect("reconnect operation");
            match reader.reconnect(&query).await {
                Ok(()) => break,
                Err(error) => {
                    assert_eq!(error, Error::Timeout, "only the known reconnect-fence timeout is allowed");
                    assert!(PostgreSqlAdapter::is_retryable(&error));
                    backoff = backoff.saturating_mul(2).min(Duration::from_secs(30));
                }
            }
        }
        assert!(reader.reconnect_not_before.is_none(), "reconnect awaited the production fence");
        _ = reader.begin_operation().expect("revalidation operation");
        assert_eq!(reader.validate_query(&query).await.expect("metadata after reconnect").len(), 4);
        _ = reader.begin_operation().expect("poll operation");
        let page = reader.execute(&query, &query.watermark().initial()).await.expect("resumed polling");
        assert_eq!(page.rows.len(), 2);
        assert_eq!(page.rows[0].row.values[0], CellValue::Int64(1));
        shutdown(&mut reader).await;
        shutdown(&mut observer).await;
        let mut untrusted = raw;
        _ = untrusted["connection"].as_object_mut().expect("connection").remove("tls");
        let mut untrusted = adapter(&untrusted);
        _ = untrusted.begin_operation().expect("untrusted connection");
        assert!(untrusted.validate_query(&query).await.is_err(), "private CA must be required");
        shutdown(&mut untrusted).await;
    }).await;
}
