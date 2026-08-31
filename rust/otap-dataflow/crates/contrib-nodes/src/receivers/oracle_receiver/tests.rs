// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use otel_arrow_dfe_engine::receiver::ReceiverWrapper;
use otel_arrow_dfe_engine::testing::{receiver::TestRuntime, test_node};
use std::time::{Duration, Instant};

fn documented_config() -> Value {
    serde_json::json!({
        "source_id": "oracle-audit",
        "connection": {
            "connect_string": "database.contoso.com:1521/ORCL",
            "instant_client_dir": "/opt/oracle/instantclient"
        },
        "authentication": {
            "username_file": "/var/run/secrets/oracle/oracle-audit/username",
            "password_file": "/var/run/secrets/oracle/oracle-audit/password"
        },
        "query": {
            "statement": "SELECT AUDIT_ID, LAST_UPDATED FROM AUDIT_LOGS",
            "interval": "5m",
            "fetch_size": 1000,
            "max_rows_per_poll": 10000,
            "timeout": "2m"
        },
        "watermark": {
            "timestamp_column": "LAST_UPDATED",
            "tie_breaker_column": "AUDIT_ID",
            "timezone": "UTC",
            "start_at": "beginning"
        },
        "checkpoint": {
            "directory": "${engine.state_dir}/oracle",
            "on_nack": "rewind",
            "max_consecutive_failures": 5
        }
    })
}

/// Scenario: The receiver loads the complete documented Oracle configuration.
/// Guarantees: The public design shape builds while unimplemented state sections remain
/// compatibility-only.
#[test]
fn accepts_the_documented_oracle_configuration() {
    let config: OracleReceiverConfig =
        serde_json::from_value(documented_config()).expect("configuration should deserialize");
    assert_eq!(config.source_id(), "oracle-audit");
    _ = config.build().expect("configuration should build");

    let mut snapshot = documented_config();
    _ = snapshot
        .as_object_mut()
        .expect("config object")
        .remove("watermark");
    _ = snapshot
        .as_object_mut()
        .expect("config object")
        .remove("checkpoint");
    let config: OracleReceiverConfig =
        serde_json::from_value(snapshot).expect("state sections should be optional");
    _ = config.build().expect("snapshot configuration should build");
}

/// Scenario: An Oracle query contains an output or policy field absent from the design.
/// Guarantees: The closed public schema rejects undocumented fields rather than silently
/// accepting behavior the foundation does not implement.
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
            serde_json::from_value::<OracleReceiverConfig>(config).is_err(),
            "undocumented query field '{field}' must be rejected"
        );
    }
}

/// Scenario: A live Oracle database is explicitly configured for an end-to-end smoke test.
/// Guarantees: The registered receiver validates metadata, polls Oracle, and emits one OTLP log
/// for the selected row.
#[test]
fn emits_oracle_rows_when_live_test_is_enabled() {
    if std::env::var_os("OTAP_ORACLE_RECEIVER_E2E").is_none() {
        return;
    }
    let config = serde_json::json!({
        "source_id": "current-time",
        "connection": {
            "connect_string": std::env::var("ORACLE_CONNECT_STRING")
                .unwrap_or_else(|_| "//localhost:1521/FREEPDB1".to_owned()),
            "instant_client_dir": std::env::var("ORACLE_INSTANT_CLIENT_DIR")
                .unwrap_or_else(|_| "C:\\oracle\\instantclient".to_owned())
        },
        "authentication": {
            "username_file": std::env::var("ORACLE_USERNAME_FILE")
                .unwrap_or_else(|_| "C:\\secrets\\oracle-username".to_owned()),
            "password_file": std::env::var("ORACLE_PASSWORD_FILE")
                .unwrap_or_else(|_| "C:\\secrets\\oracle-password".to_owned())
        },
        "query": {
            "statement": "SELECT 1 AS ID, SYSTIMESTAMP AS CURRENT_TIME FROM DUAL",
            "interval": "100ms",
            "fetch_size": 10,
            "max_rows_per_poll": 10,
            "timeout": "10s"
        },
        "watermark": {
            "timestamp_column": "CURRENT_TIME",
            "tie_breaker_column": "ID",
            "timezone": "UTC",
            "start_at": "beginning"
        },
        "checkpoint": {
            "directory": "${engine.state_dir}/oracle",
            "on_nack": "rewind",
            "max_consecutive_failures": 5
        }
    });
    let receiver = build(&config).expect("receiver config should build");
    let test_runtime = TestRuntime::<OtapPdata>::new();
    let node_config = Arc::new(NodeUserConfig::new_receiver_config(ORACLE_RECEIVER_URN));
    let receiver_wrapper = ReceiverWrapper::local(
        receiver,
        test_node(test_runtime.config().name.clone()),
        node_config,
        test_runtime.config(),
    );

    test_runtime
        .set_receiver(receiver_wrapper)
        .run_test(|ctx| async move {
            ctx.sleep(Duration::from_millis(250)).await;
            ctx.send_shutdown(Instant::now(), "Oracle receiver E2E complete")
                .await
                .expect("shutdown should enqueue");
        })
        .run_validation(|mut ctx| async move {
            let mut pdata = ctx.recv().await.expect("receiver should emit pdata");
            assert_eq!(pdata.num_items(), 1);
        });
}
