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
            adapter.execute(&query, &Cursor::Snapshot).await
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
