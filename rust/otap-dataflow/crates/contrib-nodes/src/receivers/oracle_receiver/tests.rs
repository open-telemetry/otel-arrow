// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::receivers::database::OnNack;
use otel_arrow_dfe_engine::context::ControllerContext;
use otel_arrow_dfe_engine::receiver::ReceiverWrapper;
use otel_arrow_dfe_engine::testing::{receiver::TestRuntime, test_node};
use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;
use std::time::{Duration, Instant};

const COMPOSITE_STATEMENT: &str = "SELECT AUDIT_ID, LAST_UPDATED, PAYLOAD FROM AUDIT_LOGS \
     WHERE (LAST_UPDATED > :last_timestamp \
     OR (LAST_UPDATED = :last_timestamp AND AUDIT_ID > :last_tie_breaker)) \
     ORDER BY LAST_UPDATED ASC, AUDIT_ID ASC";

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
            "statement": COMPOSITE_STATEMENT,
            "interval": "5m",
            "fetch_size": 1000,
            "max_rows_per_poll": 10000,
            "max_batch_bytes": "10 MiB",
            "max_normalized_bytes": "10 MiB",
            "timeout": "2m"
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

/// Scenario: The receiver loads the complete documented composite configuration.
/// Guarantees: The public schema builds a query plan whose cursor binds, checkpoint policy, and
/// source identity are all present, so the documented example remains runnable.
#[test]
fn accepts_the_documented_composite_configuration() {
    let config = parsed(documented_config()).expect("configuration should deserialize");

    assert_eq!(config.source_id(), "oracle-audit");
    assert_eq!(config.checkpoint().on_nack, OnNack::Rewind);
    assert_eq!(config.checkpoint().nack_backoff, Duration::from_secs(1));
    let query = config.compile().expect("query plan should compile");
    assert_eq!(query.watermark().timestamp_bind, "last_timestamp");
    assert_eq!(query.watermark().tie_breaker_bind, "last_tie_breaker");
    assert_eq!(query.watermark().initial.tie_breaker, 0);
}

/// Scenario: Required watermark, checkpoint, or byte-limit sections are omitted.
/// Guarantees: Every operational bound and cursor field stays explicit, so a receiver can never
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
    for field in ["max_batch_bytes", "max_normalized_bytes", "interval"] {
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

/// Scenario: An operator selects the unimplemented scalar or snapshot watermark mode.
/// Guarantees: Unsupported modes are rejected outright rather than silently behaving like
/// composite mode, which would checkpoint positions the query never actually ordered by.
#[test]
fn rejects_unsupported_watermark_modes() {
    for mode in ["scalar", "snapshot"] {
        let mut config = documented_config();
        config["watermark"]["mode"] = serde_json::json!(mode);
        assert!(
            parsed(config).is_err(),
            "watermark mode '{mode}' must be rejected"
        );
    }
}

/// Scenario: An operator selects a NACK policy the receiver does not implement.
/// Guarantees: Only the implemented rewind policy is accepted, so a configured failure policy is
/// never silently downgraded to a replay.
#[test]
fn rejects_unsupported_nack_policy() {
    let mut config = documented_config();
    config["checkpoint"]["on_nack"] = serde_json::json!("fail");

    assert!(parsed(config).is_err());
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

/// Scenario: A statement contains SQL comments or more than one statement.
/// Guarantees: Comment and statement-separator syntax cannot hide a second statement or comment
/// out the validated ordering clause.
#[test]
fn rejects_comments_and_multiple_statements() {
    let commented = "SELECT AUDIT_ID, LAST_UPDATED FROM AUDIT_LOGS \
         WHERE LAST_UPDATED > :last_timestamp OR (LAST_UPDATED = :last_timestamp \
         AND AUDIT_ID > :last_tie_breaker) ORDER BY LAST_UPDATED ASC, AUDIT_ID ASC -- trailing";
    assert!(parsed(with_statement(commented)).is_err());

    let multiple = format!("{COMPOSITE_STATEMENT}; SELECT 1 FROM DUAL");
    assert!(parsed(with_statement(&multiple)).is_err());
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

/// Scenario: Oracle configuration uses an unsupported call timeout.
/// Guarantees: Sub-millisecond and excessively long timeouts fail before opening a connection.
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

/// Scenario: Two configurations differ only in mounted credential paths, or only in semantics.
/// Guarantees: Rotating a secret preserves the durable checkpoint, while changing the query or a
/// cursor definition invalidates it so an unrelated position is never resumed.
#[test]
fn fingerprint_tracks_semantics_and_ignores_credential_paths() {
    let baseline = parsed(documented_config()).expect("baseline should parse");

    let mut rotated = documented_config();
    rotated["authentication"]["password_file"] = serde_json::json!("/var/run/secrets/rotated");
    rotated["connection"]["instant_client_dir"] = serde_json::json!("/opt/oracle/ic-23");
    let rotated = parsed(rotated).expect("rotated credentials should parse");
    assert_eq!(baseline.config_fingerprint(), rotated.config_fingerprint());

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

/// Scenario: The same configuration is parsed twice across separate restarts.
/// Guarantees: The fingerprint is stable across parses, so a restart with unchanged configuration
/// can always adopt its previous checkpoint instead of failing closed.
#[test]
fn fingerprint_is_stable_across_parses() {
    let first = parsed(documented_config()).expect("first parse");
    let second = parsed(documented_config()).expect("second parse");

    assert_eq!(first.config_fingerprint(), second.config_fingerprint());
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

/// Scenario: A live Oracle database is explicitly configured for an end-to-end smoke test.
/// Guarantees: The registered receiver validates cursor metadata, binds the committed composite
/// cursor, and emits OTLP logs for the selected rows.
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
        "authentication": {
            "username_file": std::env::var("ORACLE_USERNAME_FILE")
                .unwrap_or_else(|_| "C:\\secrets\\oracle-username".to_owned()),
            "password_file": std::env::var("ORACLE_PASSWORD_FILE")
                .unwrap_or_else(|_| "C:\\secrets\\oracle-password".to_owned())
        },
        "query": {
            "statement": "SELECT EVENT_ID, EVENT_TS, PAYLOAD FROM OTAP_ORACLE_EVENTS \
                WHERE (EVENT_TS > :last_timestamp OR (EVENT_TS = :last_timestamp \
                AND EVENT_ID > :last_tie_breaker)) ORDER BY EVENT_TS ASC, EVENT_ID ASC",
            "interval": "100ms",
            "fetch_size": 10,
            "max_rows_per_poll": 10,
            "max_batch_bytes": "10 MiB",
            "max_normalized_bytes": "10 MiB",
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
    let receiver =
        build(&pipeline_context(), "oracle-e2e", &config).expect("receiver config should build");
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
            ctx.sleep(Duration::from_millis(500)).await;
            ctx.send_shutdown(Instant::now(), "Oracle receiver E2E complete")
                .await
                .expect("shutdown should enqueue");
        })
        .run_validation(|mut ctx| async move {
            let mut pdata = ctx.recv().await.expect("receiver should emit pdata");
            assert!(pdata.num_items() >= 1);
        });
}
