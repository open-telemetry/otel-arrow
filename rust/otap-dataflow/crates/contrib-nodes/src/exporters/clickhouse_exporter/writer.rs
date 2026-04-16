//! ClickHouse write path and database bootstrap for the OTAP ClickHouse exporter.
//!
//! This module owns the runtime ClickHouse client, bootstraps the target database/schema (tables
//! and views), and writes OTAP Arrow `RecordBatch`es into the appropriate ClickHouse tables.
//!
//! Major components:
//!
//! - **Payload ordering and grouping**
//!   - Defines the payload groups written in sequence: attribute payloads first, then signal
//!     payloads. This ensures lookup/attribute tables are populated before any signal rows that may
//!     reference them.
//!
//! - **Write instrumentation**
//!   - [`WriteStat`] captures per-payload row counts for internal telemetry and testing.
//!
//! - **Writer abstraction**
//!   - [`ClickHouseWriter`] maintains a ClickHouse client, the target database name, and a
//!     precomputed map from `ArrowPayloadType` to destination table name.
//!   - `write_batches` accepts a set of per-payload `RecordBatch`es and dispatches inserts in the
//!     configured order, producing `WriteStat`s.
//!   - `write_batch` performs a single `INSERT ... FORMAT Native` using the Arrow-capable client
//!     and drains the response stream to surface any server-side errors.
//!
//! - **Client initialization**
//!   - `init_client_for_db` builds a ClickHouse client with endpoint/auth configuration and applies
//!     required settings for JSON column support and optional async inserts.
//!
//! - **Database/schema initialization**
//!   - `ensure_db` creates the configured database if missing.
//!   - `init_db` initializes tables and views using the schema helpers:
//!     - creates tables for all configured entries,
//!     - creates attribute views before signal views (so signal views can reference them),
//!     - creates signal views last for normalized/lookup-based schemas.
use std::collections::HashMap;

use arrow_array::RecordBatch;
use clickhouse_arrow::{ArrowClient, ArrowFormat, Client, Qid, Result};
use futures_util::StreamExt;
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otap_df_telemetry::metrics::MetricSet;

use crate::clickhouse_exporter::{
    config::Config,
    error::ClickhouseExporterError,
    metrics::ClickhouseExporterMetrics,
    tables::{
        build_payload_destination_table_map, has_json_column, init_attribute_views,
        init_signal_views, init_table, validate_identifier,
    },
};

const ATTRIBUTE_PAYLOADS: &[ArrowPayloadType] = &[
    ArrowPayloadType::ResourceAttrs,
    ArrowPayloadType::ScopeAttrs,
    ArrowPayloadType::LogAttrs,
    ArrowPayloadType::SpanAttrs,
    // TODO: [support_new_signal] ... add the rest here
];

const SIGNAL_PAYLOADS: &[ArrowPayloadType] = &[
    ArrowPayloadType::Logs,
    ArrowPayloadType::Spans, // TODO: [support_new_signal]... add the rest here
];

/// Per-table client write stats. Useful for internal telemetry and tests.
#[derive(Debug, Default, Clone)]
pub struct WriteStat {
    /// The payload type associated with the write batch
    pub payload_type: ArrowPayloadType,
    /// Total number of rows written to clickhouse
    pub rows_written: u64,
}

pub struct ClickHouseWriter {
    client: Client<ArrowFormat>,
    target_database: String,
    payload_destination_tables: HashMap<ArrowPayloadType, String>,
}

impl ClickHouseWriter {
    pub async fn new(config: &Config) -> Result<Self, ClickhouseExporterError> {
        let payload_destination_tables = build_payload_destination_table_map(config);
        otap_df_telemetry::otel_info!(
            "clickhouse.exporter.tables.bound",
            message = "Destination tables bound",
            logs = payload_destination_tables.get(&ArrowPayloadType::Logs),
            log_attributes = payload_destination_tables.get(&ArrowPayloadType::LogAttrs),
            resource_attributes = payload_destination_tables.get(&ArrowPayloadType::ResourceAttrs),
            scope_attributes = payload_destination_tables.get(&ArrowPayloadType::ScopeAttrs),
        );
        // Connect to default since the configured db may not exist yet.
        let client = init_client_for_db(config, "default").await?;
        init_db(&client, config).await?;

        Ok(Self {
            client,
            payload_destination_tables,
            target_database: config.database.clone(),
        })
    }

    async fn write_batch(
        &self,
        table_name: &str,
        batch: &RecordBatch,
    ) -> Result<(), ClickhouseExporterError> {
        let mut response_stream = self
            .client
            .insert(
                format!(
                    "INSERT INTO {}.{} FORMAT Native",
                    self.target_database, table_name
                ),
                batch.clone(),
                Some(Qid::new()),
            )
            .await
            .map_err(|e| ClickhouseExporterError::InsertRequestError {
                error: format!("{e}"),
            })?;

        while let Some(result) = response_stream.next().await {
            result.map_err(|e| ClickhouseExporterError::InsertResponseError {
                error: format!("{e}"),
            })?;
        }
        otap_df_telemetry::otel_debug!(
            "clickhouse.exporter.batch.written",
            message = "Record batch successfully written.",
            table = table_name,
            rows = batch.num_rows(),
            columns = batch.num_columns(),
        );

        Ok(())
    }
    pub async fn write_batches(
        &self,
        write_batches: &HashMap<ArrowPayloadType, RecordBatch>,
        ch_metrics: &mut MetricSet<ClickhouseExporterMetrics>,
    ) -> Result<(), ClickhouseExporterError> {
        // 1. Attributes first
        let mut write_stats = self
            .write_payload_group(ATTRIBUTE_PAYLOADS, write_batches)
            .await?;

        // 2. Signals second
        write_stats.extend_from_slice(
            &self
                .write_payload_group(SIGNAL_PAYLOADS, write_batches)
                .await?,
        );
        for stat in write_stats {
            ch_metrics.add(stat.rows_written, stat.payload_type);
        }

        Ok(())
    }

    async fn write_payload_group(
        &self,
        payloads: &[ArrowPayloadType],
        write_batches: &HashMap<ArrowPayloadType, RecordBatch>,
    ) -> Result<Vec<WriteStat>, ClickhouseExporterError> {
        let mut write_stats = Vec::new();
        for payload_type in payloads {
            let Some(batch) = write_batches.get(payload_type) else {
                continue;
            };
            let Some(table_name) = self.payload_destination_tables.get(payload_type) else {
                continue;
            };

            self.write_batch(table_name, batch).await?;
            write_stats.push(WriteStat {
                payload_type: *payload_type,
                rows_written: batch.num_rows() as u64,
            });
        }

        Ok(write_stats)
    }
}

pub async fn init_client_for_db(
    config: &Config,
    database: &str,
) -> Result<Client<ArrowFormat>, ClickhouseExporterError> {
    let mut ch_builder = Client::<ArrowFormat>::builder()
        .with_endpoint(config.endpoint.clone())
        .with_database(database)
        .with_username(config.username.clone())
        .with_password(config.password.clone());
    if has_json_column(config) {
        ch_builder = ch_builder
            .with_setting("input_format_binary_read_json_as_string", 1)
            .with_setting("allow_experimental_json_type", 1);
    }
    if config.async_insert {
        ch_builder = ch_builder.with_setting("async_insert", 1);
    }
    ch_builder
        .build()
        .await
        .map_err(|e| ClickhouseExporterError::ClientConnectionError {
            error: format!("{e}"),
        })
}

/// Ensure db and all tables are initialized if required.
async fn ensure_db(client: &ArrowClient, config: &Config) -> Result<(), ClickhouseExporterError> {
    validate_identifier(&config.database, "database")?;
    client
        .execute(
            format!("CREATE DATABASE IF NOT EXISTS {};", config.database.clone()),
            Some(Qid::new()),
        )
        .await
        .map_err(|e| ClickhouseExporterError::TableCreationError {
            error: format!("{e}"),
        })?;
    Ok(())
}

/// Initialize tables and views.
async fn init_db(client: &ArrowClient, config: &Config) -> Result<(), ClickhouseExporterError> {
    // Make sure the DB is there first
    ensure_db(client, config).await?;

    // Add in any missing tables, and create views for any Attribute tables.
    for entry in config.tables.iter_tables() {
        init_table(client, &entry, config).await?;
        // These must be created before the signal views since they may be referenced.
        if entry.kind.is_attr_kind() {
            init_attribute_views(client, &entry, config).await?;
        }
    }

    // Create any missing primary signal table views.
    for entry in config.tables.iter_tables() {
        if entry.kind.is_signal_kind() {
            init_signal_views(client, &entry, config).await?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(unused_results)]
    use super::*;
    use arrow_array::{ArrayRef, Int32Array, RecordBatch, StringArray};
    use arrow_schema::{DataType, Field, Schema};
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    /// Make a tiny RecordBatch with a known number of rows.
    fn mk_batch(rows: usize) -> RecordBatch {
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("msg", DataType::Utf8, false),
        ]));

        let ids: ArrayRef = Arc::new(Int32Array::from_iter_values(0..rows as i32));
        let msgs: ArrayRef = Arc::new(StringArray::from_iter_values(
            (0..rows).map(|i| format!("row-{i}")),
        ));

        RecordBatch::try_new(schema, vec![ids, msgs]).unwrap()
    }

    /// A lightweight harness that mirrors ClickHouseWriter::write_batches logic,
    /// but replaces the actual ClickHouse insert with an in-memory "call log".
    ///
    /// This keeps tests as pure unit tests.
    #[derive(Clone, Default)]
    struct TestWriter {
        payload_destination_tables: HashMap<ArrowPayloadType, String>,
        calls: Arc<Mutex<Vec<(ArrowPayloadType, String, usize)>>>, // (payload, table, rows)
    }

    impl TestWriter {
        async fn write_batch(
            &self,
            payload_type: ArrowPayloadType,
            table_name: &str,
            batch: &RecordBatch,
        ) -> Result<(), ClickhouseExporterError> {
            self.calls.lock().unwrap().push((
                payload_type,
                table_name.to_string(),
                batch.num_rows(),
            ));
            Ok(())
        }

        async fn write_payload_group(
            &self,
            payloads: &[ArrowPayloadType],
            write_batches: &HashMap<&ArrowPayloadType, RecordBatch>,
        ) -> Result<Vec<WriteStat>, ClickhouseExporterError> {
            let mut write_stats = Vec::new();
            for payload_type in payloads {
                let Some(batch) = write_batches.get(payload_type) else {
                    continue;
                };
                let Some(table_name) = self.payload_destination_tables.get(payload_type) else {
                    continue;
                };

                self.write_batch(*payload_type, table_name, batch).await?;
                write_stats.push(WriteStat {
                    payload_type: *payload_type,
                    rows_written: batch.num_rows() as u64,
                });
            }
            Ok(write_stats)
        }

        async fn write_batches(
            &self,
            write_batches: &HashMap<&ArrowPayloadType, RecordBatch>,
        ) -> Result<Vec<WriteStat>, ClickhouseExporterError> {
            let mut write_stats = self
                .write_payload_group(ATTRIBUTE_PAYLOADS, write_batches)
                .await?;

            write_stats.extend_from_slice(
                &self
                    .write_payload_group(SIGNAL_PAYLOADS, write_batches)
                    .await?,
            );

            Ok(write_stats)
        }
    }

    #[tokio::test]
    async fn writes_attributes_before_signals_and_reports_rows() {
        // Arrange
        let mut tables = HashMap::new();
        tables.insert(
            ArrowPayloadType::ResourceAttrs,
            "resource_attrs".to_string(),
        );
        tables.insert(ArrowPayloadType::ScopeAttrs, "scope_attrs".to_string());
        tables.insert(ArrowPayloadType::Logs, "logs".to_string());
        tables.insert(ArrowPayloadType::Spans, "spans".to_string());

        let writer = TestWriter {
            payload_destination_tables: tables,
            ..Default::default()
        };

        let resource = mk_batch(3);
        let spans = mk_batch(2);
        let logs = mk_batch(5);

        let mut batches: HashMap<&ArrowPayloadType, RecordBatch> = HashMap::new();
        batches.insert(&ArrowPayloadType::Spans, spans);
        batches.insert(&ArrowPayloadType::ResourceAttrs, resource);
        batches.insert(&ArrowPayloadType::Logs, logs);

        // Act
        let stats = writer.write_batches(&batches).await.unwrap();
        let calls = writer.calls.lock().unwrap().clone();

        // Assert: attributes first, then signals (based on ATTRIBUTE_PAYLOADS then SIGNAL_PAYLOADS)
        let payload_call_order: Vec<ArrowPayloadType> = calls.iter().map(|c| c.0).collect();

        // Expected order:
        // - ResourceAttrs (present)
        // - ScopeAttrs (absent) => skipped
        // - LogAttrs (absent) => skipped
        // - SpanAttrs (absent) => skipped
        // then signals:
        // - Logs (present)
        // - Spans (present)
        assert_eq!(
            payload_call_order,
            vec![
                ArrowPayloadType::ResourceAttrs,
                ArrowPayloadType::Logs,
                ArrowPayloadType::Spans
            ]
        );

        // Assert: row counts reported correctly
        let mut stat_map: HashMap<ArrowPayloadType, u64> = HashMap::new();
        for s in stats {
            stat_map.insert(s.payload_type, s.rows_written);
        }
        assert_eq!(stat_map.get(&ArrowPayloadType::ResourceAttrs), Some(&3));
        assert_eq!(stat_map.get(&ArrowPayloadType::Logs), Some(&5));
        assert_eq!(stat_map.get(&ArrowPayloadType::Spans), Some(&2));
    }

    #[tokio::test]
    async fn skips_when_batch_present_but_no_table_mapping() {
        // Arrange: omit table mapping for Logs
        let mut tables = HashMap::new();
        tables.insert(
            ArrowPayloadType::ResourceAttrs,
            "resource_attrs".to_string(),
        );

        let writer = TestWriter {
            payload_destination_tables: tables,
            ..Default::default()
        };

        let mut batches: HashMap<&ArrowPayloadType, RecordBatch> = HashMap::new();
        batches.insert(&ArrowPayloadType::ResourceAttrs, mk_batch(1));
        batches.insert(&ArrowPayloadType::Logs, mk_batch(10)); // should be skipped (no table name)

        // Act
        let stats = writer.write_batches(&batches).await.unwrap();
        let calls = writer.calls.lock().unwrap().clone();

        // Assert: only ResourceAttrs written
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, ArrowPayloadType::ResourceAttrs);
        assert_eq!(calls[0].1, "resource_attrs".to_string());
        assert_eq!(calls[0].2, 1);

        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].payload_type, ArrowPayloadType::ResourceAttrs);
        assert_eq!(stats[0].rows_written, 1);
    }

    #[tokio::test]
    async fn empty_input_writes_nothing() {
        let writer = TestWriter::default();
        let batches: HashMap<&ArrowPayloadType, RecordBatch> = HashMap::new();

        let stats = writer.write_batches(&batches).await.unwrap();
        let calls = writer.calls.lock().unwrap().clone();

        assert!(stats.is_empty());
        assert!(calls.is_empty());
    }
}
