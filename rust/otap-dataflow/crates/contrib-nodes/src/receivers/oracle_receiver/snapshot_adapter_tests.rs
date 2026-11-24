// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

/// Scenario: Snapshot metadata contains only nullable, non-cursor columns.
/// Guarantees: No key column, timestamp type, or value extraction is required.
#[test]
fn snapshot_metadata_and_rows_have_no_cursor_columns() {
    let columns = [("PAYLOAD".into(), OracleType::Varchar2(100))];
    let plan =
        validate_described_watermark(&columns, &CompiledWatermark::Snapshot).expect("metadata");
    assert_eq!(plan, CursorColumns::Snapshot);
    assert_eq!(
        timestamp_type_for_cursor(plan, &[]).expect("no timestamp bind"),
        None
    );
    for values in [
        vec![],
        vec![CellValue::Null],
        vec![CellValue::String("duplicate".into())],
    ] {
        let cursor =
            extract_normalized_watermark(&Row { values }, plan).expect("no key extraction");
        assert_eq!(cursor, Cursor::Snapshot);
        assert_eq!(cursor_heap_bytes(&cursor), 0);
    }
}

/// Scenario: Snapshot metadata describes an explicit event-time alias.
/// Guarantees: Native temporal aliases are accepted case-insensitively; missing or textual aliases fail without exposing their names.
#[test]
fn snapshot_event_time_column_requires_native_oracle_metadata() {
    let columns = [
        ("PRIVATE_DATE".into(), OracleType::Date),
        ("PRIVATE_TIMESTAMP".into(), OracleType::TimestampTZ(9)),
        ("PRIVATE_TEXT".into(), OracleType::Varchar2(100)),
    ];
    assert!(validate_event_time_metadata(&columns, Some("private_date")).is_ok());
    assert!(validate_event_time_metadata(&columns, Some("private_timestamp")).is_ok());
    for name in ["PRIVATE_TEXT", "PRIVATE_MISSING"] {
        let error = validate_event_time_metadata(&columns, Some(name)).expect_err("invalid alias");
        assert!(matches!(error, OracleAdapterError::InvalidEventTimeColumn));
        assert!(!error.to_string().contains(name));
    }
}

/// Scenario: An oversized snapshot is reported by the native fetch loop.
/// Guarantees: Its configuration feedback pauses only this source rather than retrying or terminating the pipeline.
#[test]
fn snapshot_limit_is_not_retryable() {
    for error in [
        OracleAdapterError::SnapshotRowLimit { limit: 100 },
        OracleAdapterError::SnapshotNormalizedByteLimit {
            rows: 10,
            limit: 4096,
        },
    ] {
        assert!(!OracleAdapter::is_retryable(&error));
        assert!(OracleAdapter::should_pause_source(&error));
        assert_eq!(
            OracleAdapter::classify_error(&error),
            ReceiverErrorKind::Configuration
        );
    }
    assert!(
        OracleAdapterError::SnapshotRowLimit { limit: 100 }
            .to_string()
            .contains("100 rows")
    );
    let normalized = OracleAdapterError::SnapshotNormalizedByteLimit {
        rows: 10,
        limit: 4096,
    }
    .to_string();
    assert!(normalized.contains("4096 bytes"));
    assert!(normalized.contains("10 complete rows"));
}

/// Scenario: A native snapshot failure is known while statement/session teardown remains blocked.
/// Guarantees: Local collection failure is recorded once before cleanup, and execution cannot finish before cleanup.
#[tokio::test]
async fn snapshot_failure_metrics_precede_native_cleanup() {
    use otel_arrow_dfe_engine::Interests;
    use otel_arrow_dfe_engine::testing::test_pipeline_ctx_with_interests;
    use otel_arrow_dfe_otap::metrics::ReceiverPullMetrics;
    use otel_arrow_dfe_telemetry::metrics::MetricValue;
    use std::cell::Cell;
    use std::time::Duration;

    struct NativeCleanup(std::sync::mpsc::Receiver<()>);
    impl Drop for NativeCleanup {
        fn drop(&mut self) {
            self.0
                .recv_timeout(Duration::from_secs(10))
                .expect("native cleanup released");
        }
    }

    let config: super::super::OracleReceiverConfig =
        serde_json::from_value(super::super::tests::documented_config()).expect("config");
    let mut adapter = config.adapter(super::super::tests::credential_provider(
        BasicAuthCredential::new("user", "password").expect("credential"),
    ));
    _ = adapter.begin_operation().expect("query operation");
    let (pipeline, _registry) = test_pipeline_ctx_with_interests(
        Interests::NODE_OUTPUT_METRICS | Interests::NODE_LOCAL_DURATION,
    );
    let mut metrics = ReceiverPullMetrics::register(&pipeline);
    let attempt = metrics.processing();
    let cancellation = attempt.cancellation_handle();
    let observed = tokio::sync::Notify::new();
    let failures = Cell::new(0);
    let on_failure = || {
        failures.set(failures.get() + 1);
        cancellation.cancel();
        observed.notify_one();
    };
    let (release, gate) = std::sync::mpsc::sync_channel(1);
    let query = config.query();
    let cursor = query.watermark().initial();
    {
        let execution = adapter.run_blocking::<(), _>(
            &query,
            &cursor,
            &on_failure,
            move |_, _, _, _, _, _, on_failure| {
                let _cleanup = NativeCleanup(gate);
                on_failure();
                on_failure();
                Err(OracleAdapterError::SnapshotRowLimit { limit: 1 })
            },
        );
        tokio::pin!(execution);
        tokio::select! {
            biased;
            result = &mut execution => panic!("execution returned before cleanup: {result:?}"),
            () = observed.notified() => {}
            () = tokio::time::sleep(Duration::from_secs(10)) => panic!("native failure was not observed"),
        }
        let snapshots = metrics.terminal_snapshots();
        assert_eq!(snapshots.len(), 2);
        for snapshot in snapshots {
            if let MetricValue::Distribution(duration) = &snapshot.get_metrics()[0] {
                assert_eq!(duration.summary().0, 1);
            } else {
                assert_eq!(
                    snapshot.measurement_attribute_value("outcome"),
                    Some("failure")
                );
                assert_eq!(snapshot.get_metrics()[0].to_u64_lossy(), 1);
            }
        }
        assert!(futures::poll!(&mut execution).is_pending());
        tokio::time::pause();
        tokio::time::advance(Duration::from_secs(60)).await;
        tokio::time::resume();
        release.send(()).expect("release native teardown");
        assert!(matches!(
            execution.await,
            Err(OracleAdapterError::SnapshotRowLimit { limit: 1 })
        ));
    }
    attempt.success();
    adapter.shutdown().await.expect("native worker joined");
    assert_eq!(failures.get(), 1);
    assert!(
        metrics.terminal_snapshots().is_empty(),
        "cleanup cannot record another observation"
    );
}

/// Scenario: Live Oracle returns an exact-fit result, one extra row, or too many normalized bytes.
/// Guarantees: Snapshot fetching accepts complete results and fails before returning a truncated row or byte prefix.
#[tokio::test]
#[ignore = "requires ORACLE_SNAPSHOT_TEST_CONFIG, ORACLE_USERNAME, and ORACLE_PASSWORD_FILE"]
async fn live_snapshot_bounds_are_all_or_nothing() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::var("ORACLE_SNAPSHOT_TEST_CONFIG")?;
    let raw: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    let credential = super::super::tests::fixture_credential(
        &std::env::var("ORACLE_USERNAME")?,
        &std::env::var("ORACLE_PASSWORD_FILE")?,
    );
    for (sql, max_rows, bytes, expected_rows) in [
        (
            "SELECT CAST(NULL AS VARCHAR2(10)) AS PAYLOAD FROM DUAL",
            1,
            4096,
            Some(1),
        ),
        (
            "SELECT LEVEL AS VALUE FROM DUAL CONNECT BY LEVEL <= 2",
            2,
            4096,
            Some(2),
        ),
        (
            "SELECT LEVEL AS VALUE FROM DUAL CONNECT BY LEVEL <= 2",
            1,
            4096,
            None,
        ),
        (
            "SELECT RPAD('x', 2000, 'x') AS PAYLOAD FROM DUAL CONNECT BY LEVEL <= 2",
            10,
            3000,
            None,
        ),
    ] {
        let mut value = raw.clone();
        value["watermark"] = serde_json::json!({"mode":"snapshot"});
        value["query"]["statement"] = sql.into();
        value["query"]["max_rows_per_poll"] = max_rows.into();
        value["query"]["fetch_size_rows"] = 1.into();
        value["query"]["max_batch_bytes"] = bytes.into();
        let config: super::super::OracleReceiverConfig = serde_json::from_value(value)?;
        let query = config.query();
        let mut adapter =
            config.adapter(super::super::tests::credential_provider(credential.clone()));
        let result = async {
            _ = adapter.begin_operation()?;
            _ = adapter.validate_query(&query).await?;
            _ = adapter.begin_operation()?;
            adapter.execute(&query, &Cursor::Snapshot, &|| {}).await
        }
        .await;
        adapter.shutdown().await?;
        if let Some(expected_rows) = expected_rows {
            let page = result?;
            assert_eq!(page.rows.len(), expected_rows);
            assert!(page.rows.iter().all(|row| row.cursor == Cursor::Snapshot));
        } else {
            assert!(matches!(
                result,
                Err(OracleAdapterError::SnapshotRowLimit { .. }
                    | OracleAdapterError::SnapshotNormalizedByteLimit { .. })
            ));
        }
    }
    Ok(())
}
