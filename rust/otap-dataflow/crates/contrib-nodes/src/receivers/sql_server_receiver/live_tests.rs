// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Explicitly requested live tests against the disposable SQL Server fixture.
//! Missing fixture inputs fail these tests; they never silently skip.

use super::{SqlServerAdapter, SqlServerError, SqlServerReceiverConfig, TcpConnector};
use async_trait::async_trait;
use otel_arrow_dfe_engine::{
    capability::{
        CapabilityError,
        auth::{BasicAuthCredential, basic_auth_provider::BasicAuthCredentialStream},
    },
    local::capability::auth::basic_auth_provider::BasicAuthProvider,
};
use otel_arrow_dfe_scraper::database::{
    CellValue, Cursor, DriverAdapter, DriverCancellation, ScalarValue,
};
use serde_json::{Value, json};

fn required(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("live SQL Server fixture requires {name}"))
}

struct Credentials(BasicAuthCredential);

#[async_trait(?Send)]
impl BasicAuthProvider for Credentials {
    async fn get_credential(&self) -> Result<BasicAuthCredential, CapabilityError> {
        Ok(self.0.clone())
    }
    fn credential_stream(&self) -> BasicAuthCredentialStream {
        Box::pin(futures::stream::empty())
    }
}

fn credentials() -> Box<dyn BasicAuthProvider> {
    let username = required("OTAP_SQL_SERVER_USERNAME");
    let password = std::fs::read_to_string(required("OTAP_SQL_SERVER_PASSWORD_FILE"))
        .expect("read fixture password");
    Box::new(Credentials(
        BasicAuthCredential::new(username, password.trim_end_matches(['\r', '\n'])).unwrap(),
    ))
}

fn config(mode: &str) -> Value {
    let predicate = match mode {
        "scalar" => " WHERE ID > @key ORDER BY ID ASC",
        "timestamp" => " WHERE EVENT_TS > @key ORDER BY EVENT_TS ASC",
        "composite" => {
            " WHERE EVENT_TS > @ts OR (EVENT_TS = @ts AND ID > @id) ORDER BY EVENT_TS ASC, ID ASC"
        }
        "snapshot" => "",
        _ => panic!("test mode"),
    };
    let watermark = match mode {
        "snapshot" => json!({"mode":"snapshot"}),
        "scalar" => {
            json!({"mode":"scalar","column":"ID","bind":"key","initial":{"type":"int64","value":0}})
        }
        "timestamp" => {
            json!({"mode":"scalar","column":"EVENT_TS","bind":"key","initial":{"type":"timestamp","value":"2026-01-01T00:00:00Z"}})
        }
        _ => {
            json!({"mode":"composite","timestamp":{"column":"EVENT_TS","bind":"ts","initial":"2026-01-01T00:00:00Z","timezone":"UTC"},"tie_breaker":{"column":"ID","bind":"id","initial":0}})
        }
    };
    json!({
        "source_id":"sql-server-live",
        "connection":{
            "host":required("OTAP_SQL_SERVER_HOST"),
            "port":required("OTAP_SQL_SERVER_PORT").parse::<u16>().expect("fixture port"),
            "database":required("OTAP_SQL_SERVER_DATABASE"),
            "connect_timeout":"20s",
            "tls":{"ca_file":required("OTAP_SQL_SERVER_CA_FILE")}
        },
        "query":{
            "statement":format!("SELECT ID, EVENT_TS, PAYLOAD, AMOUNT, RAW, AT_TIME FROM dbo.OTAP_SQL_EVENTS{predicate}"),
            "max_rows_per_poll":if mode=="snapshot" {10} else {2},
            "fetch_size_rows":1,"max_batch_bytes":"1MiB","timeout":"10s"
        },
        "watermark":watermark,
        "checkpoint":{"directory":"state/sql-server-live","on_nack":"rewind","nack_backoff":"1s","max_consecutive_failures":5}
    })
}

/// Scenario: A restricted SQL login connects over verified TLS and polls all three supported modes.
/// Guarantees: Real-server metadata, bound keysets, exact body values, and complete snapshots work.
#[tokio::test]
#[ignore = "requires the explicitly provisioned OTAP_SQL_SERVER_* fixture"]
async fn live_verified_tls_polling_modes() {
    for mode in ["scalar", "timestamp", "composite", "snapshot"] {
        let config: SqlServerReceiverConfig = serde_json::from_value(config(mode)).unwrap();
        let query = config.query().clone();
        let mut adapter = SqlServerAdapter::new(config, TcpConnector::default(), credentials());
        _ = adapter.begin_operation().unwrap();
        let metadata = adapter
            .validate_query(&query)
            .await
            .expect("verified TLS and metadata");
        assert_eq!(metadata.len(), 6);
        _ = adapter.begin_operation().unwrap();
        let first = adapter
            .execute(&query, &query.watermark().initial())
            .await
            .expect("live page");
        assert_eq!(first.rows.len(), if mode == "snapshot" { 3 } else { 2 });
        assert_eq!(first.rows[0].row.values[0], CellValue::Int64(1));
        assert_eq!(
            first.rows[0].row.values[1],
            CellValue::Timestamp("2026-01-01T00:00:01.1234567Z".into())
        );
        assert_eq!(
            first.rows[0].row.values[2],
            CellValue::String("hello \u{03c0}".into())
        );
        assert_eq!(
            first.rows[0].row.values[3],
            CellValue::Decimal("123456789012345678901234567890.12345678".into())
        );
        assert_eq!(
            first.rows[0].row.values[4],
            CellValue::Bytes(vec![0, 1, 255])
        );
        assert_eq!(
            first.rows[0].row.values[5],
            CellValue::TimestampTz("2026-01-01T05:30:00.1234567+05:30".into())
        );
        if mode != "snapshot" {
            let cursor = &first.rows.last().unwrap().cursor;
            _ = adapter.begin_operation().unwrap();
            let second = adapter
                .execute(&query, cursor)
                .await
                .expect("next live keyset");
            assert_eq!(second.rows.len(), 1);
            assert_eq!(second.rows[0].row.values[0], CellValue::Int64(3));
        }
        adapter.shutdown().await.expect("session cleanup");
    }
}

/// Scenario: The fixture's CA is omitted or its certificate name does not match the endpoint.
/// Guarantees: The connector cannot silently bypass server certificate or hostname verification.
#[tokio::test]
#[ignore = "requires the explicitly provisioned OTAP_SQL_SERVER_* fixture"]
async fn live_tls_rejects_unknown_ca_and_wrong_hostname() {
    for mismatch in [false, true] {
        let mut raw = config("scalar");
        if mismatch {
            raw["connection"]["host"] = json!("127.0.0.1");
        } else {
            _ = raw["connection"].as_object_mut().unwrap().remove("tls");
        }
        let config: SqlServerReceiverConfig = serde_json::from_value(raw).unwrap();
        let query = config.query().clone();
        let mut adapter = SqlServerAdapter::new(config, TcpConnector::default(), credentials());
        _ = adapter.begin_operation().unwrap();
        assert!(
            adapter.validate_query(&query).await.is_err(),
            "invalid TLS must fail"
        );
        adapter.shutdown().await.expect("failed connection cleanup");
    }
}

/// Scenario: A live TCP/TLS connection attempt is cancelled before it can deliver a session.
/// Guarantees: Cancellation remains generation-local and connection worker shutdown is confirmed.
#[tokio::test]
#[ignore = "requires the explicitly provisioned OTAP_SQL_SERVER_* fixture"]
async fn live_connect_cancellation_cleans_up_worker() {
    let config: SqlServerReceiverConfig = serde_json::from_value(config("scalar")).unwrap();
    let query = config.query().clone();
    let mut adapter = SqlServerAdapter::new(config, TcpConnector::default(), credentials());
    let cancellation = adapter.begin_operation().unwrap();
    let (result, ()) = tokio::join!(adapter.validate_query(&query), async {
        tokio::task::yield_now().await;
        cancellation.cancel().await.unwrap();
    });
    assert_eq!(result, Err(SqlServerError::Cancelled));
    tokio::time::timeout(std::time::Duration::from_secs(5), adapter.shutdown())
        .await
        .expect("bounded worker shutdown")
        .expect("worker stopped");
}

/// Scenario: The registered receiver receives NACK then ACK, stops, and restarts from its file checkpoint.
/// Guarantees: Real SQL Server polling replays rejected rows and resumes only from acknowledged progress.
#[test]
#[ignore = "requires the explicitly provisioned OTAP_SQL_SERVER_* fixture"]
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
    use otel_arrow_dfe_scraper::CheckpointStore;
    use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;
    use std::sync::Arc;
    let directory = tempfile::tempdir().unwrap();
    let mut raw = config("scalar");
    raw["checkpoint"]["directory"] = json!(directory.path());
    raw["query"]["catch_up"] = json!({"max_pages":1});
    raw["checkpoint"]["nack_backoff"] = json!("1ms");
    let parsed: SqlServerReceiverConfig = serde_json::from_value(raw.clone()).unwrap();
    for (restart, expected) in [(false, 2), (true, 3)] {
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
            "sql-server-live",
            parsed.config_fingerprint().to_owned(),
        );
        let mut node = NodeUserConfig::new_receiver_config(super::SQL_SERVER_RECEIVER_URN);
        node.config = raw.clone();
        let mut registry = CapabilityRegistry::new();
        registry
            .register_local(
                std::any::TypeId::of::<Registration>(),
                LocalCapabilityEntry::new("sql_fixture".into(), || -> Box<dyn std::any::Any> {
                    Box::new(credentials())
                }),
            )
            .unwrap();
        let capabilities = resolve_bindings_for_test(
            &std::collections::HashMap::from([(
                "basic_auth_provider".into(),
                "sql_fixture".into(),
            )]),
            &registry,
            &std::collections::HashSet::from(["sql_fixture".into()]),
        )
        .unwrap();
        let wrapper = (super::SQL_SERVER_RECEIVER.create)(
            context,
            test_node("sql-server-live"),
            Arc::new(node),
            runtime.config(),
            &capabilities,
        )
        .expect("registered receiver factory");
        let check = store.clone();
        runtime
            .set_receiver(wrapper)
            .run_test(|_| async {})
            .run_validation_concurrent(move |mut ctx| async move {
                let mut data = tokio::time::timeout(std::time::Duration::from_secs(30), ctx.recv())
                    .await
                    .expect("receiver emits")
                    .expect("live data");
                assert_eq!(data.num_items(), if restart { 1 } else { 2 });
                if !restart {
                    let (_, nack) = next_nack(NackMsg::new("fixture replay", data)).unwrap();
                    ctx.send_control_msg(NodeControlMsg::Nack(nack))
                        .await
                        .unwrap();
                    data = ctx.recv().await.expect("NACK replay");
                    assert_eq!(data.num_items(), 2);
                    assert!(check.read().unwrap().is_none());
                }
                let (_, ack) = next_ack(AckMsg::new(data)).unwrap();
                ctx.send_control_msg(NodeControlMsg::Ack(ack))
                    .await
                    .unwrap();
                tokio::time::timeout(std::time::Duration::from_secs(5), async {
                    loop {
                        if check.read().unwrap().is_some_and(|state| {
                            state.cursor == Cursor::Scalar(ScalarValue::Int64(expected))
                        }) {
                            break;
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
                    }
                })
                .await
                .expect("acknowledged checkpoint");
                ctx.send_control_msg(NodeControlMsg::Shutdown {
                    deadline: std::time::Instant::now() + std::time::Duration::from_secs(3),
                    reason: "live restart test".into(),
                })
                .await
                .unwrap();
            });
        assert_eq!(
            store.read().unwrap().unwrap().cursor,
            Cursor::Scalar(ScalarValue::Int64(expected))
        );
    }
}

/// Scenario: SQL Server permits test connections with explicit TLS policy and server-name overrides.
/// Guarantees: Defaults validate certificates, overrides preserve verification, bypass is explicit,
/// and disabled TLS can poll an isolated fixture without requiring a CA file.
#[tokio::test]
#[ignore = "requires the OTAP_SQL_SERVER_* fixture started with --allow-unencrypted"]
async fn live_sql_server_tls_test_options() {
    assert_eq!(
        required("OTAP_SQL_SERVER_ALLOW_UNENCRYPTED"),
        "1",
        "start the disposable fixture with --allow-unencrypted for this test"
    );
    let ca_file = required("OTAP_SQL_SERVER_CA_FILE");
    let cases = [
        (
            json!({"enabled":true,"ca_file":ca_file,"server_name_override":"localhost"}),
            true,
        ),
        (
            json!({"enabled":true,"ca_file":ca_file,"server_name_override":"wrong.example.invalid"}),
            false,
        ),
        (
            json!({"enabled":true,"trust_server_certificate":true}),
            true,
        ),
        (json!({"enabled":false}), true),
    ];
    for (tls, should_connect) in cases {
        let mut raw = config("scalar");
        // The certificate identifies localhost, not this IP. A positive verified
        // connection must therefore use the explicit verification-name override.
        raw["connection"]["host"] = json!("127.0.0.1");
        raw["connection"]["tls"] = tls;
        let config: SqlServerReceiverConfig = serde_json::from_value(raw).unwrap();
        let query = config.query().clone();
        let mut adapter = SqlServerAdapter::new(config, TcpConnector::default(), credentials());
        _ = adapter.begin_operation().unwrap();
        let result = adapter.validate_query(&query).await;
        if should_connect {
            _ = result.expect("configured SQL Server test mode");
            _ = adapter.begin_operation().unwrap();
            let page = adapter
                .execute(&query, &query.watermark().initial())
                .await
                .unwrap();
            assert_eq!(page.rows[0].row.values[0], CellValue::Int64(1));
        } else {
            assert!(result.is_err(), "wrong verification name must fail");
        }
        adapter.shutdown().await.expect("connection cleanup");
    }
}

/// Scenario: A separate receiver process uses native TLS and explicit non-TLS with no Rustls provider.
/// Guarantees: Native verified connections are independent of engine crypto-* features; non-TLS is explicit.
#[test]
#[ignore = "requires the OTAP_SQL_SERVER_* fixture started with --allow-unencrypted"]
fn live_native_tls_without_rustls_provider() {
    const CHILD: &str = "OTAP_SQL_SERVER_NO_CRYPTO_CHILD";
    if std::env::var_os(CHILD).is_none() {
        let result = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--ignored", "--exact",
                "receivers::sql_server_receiver::live_tests::live_native_tls_without_rustls_provider",
                "--nocapture",
            ])
            .env(CHILD, "1")
            .output()
            .expect("isolated test process");
        assert!(
            result.status.success(),
            "child failed:\n{}\n{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr),
        );
        return;
    }
    assert_eq!(required("OTAP_SQL_SERVER_ALLOW_UNENCRYPTED"), "1");
    assert!(!otel_arrow_dfe_otap::crypto::is_crypto_provider_installed());
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            for enabled in [true, false] {
                let mut raw = config("scalar");
                if !enabled {
                    raw["connection"]["tls"] = json!({"enabled":false});
                }
                let config: SqlServerReceiverConfig = serde_json::from_value(raw).unwrap();
                let query = config.query().clone();
                let mut adapter =
                    SqlServerAdapter::new(config, TcpConnector::default(), credentials());
                _ = adapter.begin_operation().unwrap();
                _ = adapter
                    .validate_query(&query)
                    .await
                    .expect("native TLS or explicit non-TLS metadata");
                _ = adapter.begin_operation().unwrap();
                let page = adapter
                    .execute(&query, &query.watermark().initial())
                    .await
                    .expect("non-TLS rows");
                assert_eq!(page.rows[0].row.values[0], CellValue::Int64(1));
                adapter.shutdown().await.unwrap();
            }
        });
    assert!(!otel_arrow_dfe_otap::crypto::is_crypto_provider_installed());
}

/// Scenario: The fixture serves an original, replacement, or deliberately expired certificate.
/// Guarantees: A configured private CA accepts new valid leaves without global trust changes, but expiry fails closed.
#[tokio::test]
#[ignore = "requires fixture certificate rotation; run the fixture CI runner or documented commands"]
async fn live_certificate_rotation_state() {
    let state = required("OTAP_SQL_SERVER_CERTIFICATE");
    assert!(matches!(
        state.as_str(),
        "valid" | "replacement" | "expired"
    ));
    let config: SqlServerReceiverConfig = serde_json::from_value(config("scalar")).unwrap();
    let query = config.query().clone();
    let mut adapter = SqlServerAdapter::new(config, TcpConnector::default(), credentials());
    _ = adapter.begin_operation().unwrap();
    let result = adapter.validate_query(&query).await;
    if state == "expired" {
        assert_eq!(result, Err(SqlServerError::Tls));
        assert!(!SqlServerAdapter::<TcpConnector>::is_retryable(
            &SqlServerError::Tls
        ));
    } else {
        _ = result.expect("same CA validates replacement leaf");
        _ = adapter.begin_operation().unwrap();
        assert_eq!(
            adapter
                .execute(&query, &query.watermark().initial())
                .await
                .unwrap()
                .rows[0]
                .row
                .values[0],
            CellValue::Int64(1),
        );
    }
    adapter.shutdown().await.unwrap();
}

/// Scenario: The owned database disconnects a healthy receiver session between completed polls.
/// Guarantees: The next submitted operation fails closed without a partial page, retry, or successful cleanup claim.
#[tokio::test]
#[ignore = "requires exclusive fixture use; run live tests with --test-threads=1"]
async fn live_idle_disconnect_is_contained() {
    let config: SqlServerReceiverConfig = serde_json::from_value(config("scalar")).unwrap();
    let query = config.query().clone();
    let mut adapter = SqlServerAdapter::new(config, TcpConnector::default(), credentials());
    _ = adapter.begin_operation().unwrap();
    _ = adapter.validate_query(&query).await.unwrap();
    _ = adapter.begin_operation().unwrap();
    let first = adapter
        .execute(&query, &query.watermark().initial())
        .await
        .unwrap();
    let cursor = first.rows.last().unwrap().cursor.clone();
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("receivers")
        .join("sql_server_receiver")
        .join("ci")
        .join("sql-server-fixture.py");
    let status = std::process::Command::new(required("OTAP_SQL_SERVER_FIXTURE_PYTHON"))
        .arg(script)
        .args(["disconnect-readers", "--directory"])
        .arg(required("OTAP_SQL_SERVER_FIXTURE_DIRECTORY"))
        .status()
        .expect("owned fixture fault injection");
    assert!(status.success());
    _ = adapter.begin_operation().unwrap();
    let error = adapter
        .execute(&query, &cursor)
        .await
        .expect_err("disconnected session cannot emit");
    assert!(!SqlServerAdapter::<TcpConnector>::is_retryable(&error));
    assert!(adapter.begin_operation().is_err());
    assert!(adapter.shutdown().await.is_err());
}
