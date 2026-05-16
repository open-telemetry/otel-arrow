//! ClickHouse write path and database bootstrap for the OTAP ClickHouse exporter.
//!
//! This module owns the runtime ClickHouse client, bootstraps the target database/schema (tables),
//! and writes OTAP Arrow `RecordBatch`es into the appropriate ClickHouse tables.
//!
//! Major components:
//!
//! - **Writer abstraction**
//!   - [`ClickHouseWriter`] maintains a ClickHouse client, the target database name, and a
//!     precomputed map from `ArrowPayloadType` to destination table name.
//!   - `write_batches` accepts a set of per-payload `RecordBatch`es and dispatches inserts,
//!     updating metrics inline.
//!   - `write_batch` performs a single `INSERT ... FORMAT Native` using the Arrow-capable client
//!     and drains the response stream to surface any server-side errors.
//!
//! - **Client initialization**
//!   - `init_client_for_db` builds a ClickHouse client with endpoint/auth configuration and applies
//!     required settings for JSON column support and optional async inserts.
//!
//! - **Database/schema initialization**
//!   - `ensure_db` creates the configured database if missing.
//!   - `init_db` creates tables for all configured entries.
use std::collections::HashMap;

use arrow_array::RecordBatch;
use clickhouse_arrow::{ArrowClient, ArrowFormat, Client, Qid, Result};
use futures_util::StreamExt;
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otap_df_telemetry::metrics::MetricSet;

use crate::clickhouse_exporter::{
    config::{AttributeRepresentation, Config},
    error::ClickhouseExporterError,
    metrics::ClickhouseExporterMetrics,
    tables::{build_payload_destination_table_map, init_table, validate_identifier},
};

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
            spans = payload_destination_tables.get(&ArrowPayloadType::Spans),
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
        for (payload_type, batch) in write_batches {
            let Some(table_name) = self.payload_destination_tables.get(payload_type) else {
                continue;
            };
            self.write_batch(table_name, batch).await?;
            ch_metrics.add(batch.num_rows() as u64, *payload_type);
        }
        Ok(())
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

fn has_json_column(config: &Config) -> bool {
    config.attributes.log.representation == AttributeRepresentation::Json
        || config.attributes.scope.representation == AttributeRepresentation::Json
        || config.attributes.resource.representation == AttributeRepresentation::Json
        || config.attributes.metric.representation == AttributeRepresentation::Json
        || config.attributes.trace.representation == AttributeRepresentation::Json
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

/// Initialize tables.
async fn init_db(client: &ArrowClient, config: &Config) -> Result<(), ClickhouseExporterError> {
    ensure_db(client, config).await?;
    for entry in config.tables.iter_tables() {
        init_table(client, &entry, config).await?;
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

        async fn write_batches(
            &self,
            write_batches: &HashMap<&ArrowPayloadType, RecordBatch>,
        ) -> Result<Vec<(ArrowPayloadType, u64)>, ClickhouseExporterError> {
            let mut stats = Vec::new();
            for (payload_type, batch) in write_batches {
                let Some(table_name) = self.payload_destination_tables.get(payload_type) else {
                    continue;
                };
                self.write_batch(**payload_type, table_name, batch).await?;
                stats.push((**payload_type, batch.num_rows() as u64));
            }
            Ok(stats)
        }
    }

    #[tokio::test]
    async fn writes_all_mapped_payloads_and_reports_rows() {
        // Arrange: only signal tables have mappings (no attribute table mappings)
        let mut tables = HashMap::new();
        tables.insert(ArrowPayloadType::Logs, "logs".to_string());
        tables.insert(ArrowPayloadType::Spans, "spans".to_string());

        let writer = TestWriter {
            payload_destination_tables: tables,
            ..Default::default()
        };

        let spans = mk_batch(2);
        let logs = mk_batch(5);
        // ResourceAttrs has no table mapping, so it will be skipped
        let resource = mk_batch(3);

        let mut batches: HashMap<&ArrowPayloadType, RecordBatch> = HashMap::new();
        batches.insert(&ArrowPayloadType::Spans, spans);
        batches.insert(&ArrowPayloadType::ResourceAttrs, resource);
        batches.insert(&ArrowPayloadType::Logs, logs);

        // Act
        let stats = writer.write_batches(&batches).await.unwrap();
        let calls = writer.calls.lock().unwrap().clone();

        // Assert: only mapped payloads are written (ResourceAttrs skipped - no table mapping)
        assert_eq!(calls.len(), 2);

        // Build maps for order-independent assertions
        let mut call_map: HashMap<ArrowPayloadType, (String, usize)> = HashMap::new();
        for (pt, table, rows) in &calls {
            call_map.insert(*pt, (table.clone(), *rows));
        }
        assert_eq!(
            call_map.get(&ArrowPayloadType::Logs),
            Some(&("logs".to_string(), 5))
        );
        assert_eq!(
            call_map.get(&ArrowPayloadType::Spans),
            Some(&("spans".to_string(), 2))
        );

        // Assert: row counts reported correctly
        let mut stat_map: HashMap<ArrowPayloadType, u64> = HashMap::new();
        for (pt, rows) in stats {
            stat_map.insert(pt, rows);
        }
        assert_eq!(stat_map.get(&ArrowPayloadType::Logs), Some(&5));
        assert_eq!(stat_map.get(&ArrowPayloadType::Spans), Some(&2));
        assert!(stat_map.get(&ArrowPayloadType::ResourceAttrs).is_none());
    }

    #[tokio::test]
    async fn skips_when_batch_present_but_no_table_mapping() {
        // Arrange: only Logs has a table mapping
        let mut tables = HashMap::new();
        tables.insert(ArrowPayloadType::Logs, "logs".to_string());

        let writer = TestWriter {
            payload_destination_tables: tables,
            ..Default::default()
        };

        let mut batches: HashMap<&ArrowPayloadType, RecordBatch> = HashMap::new();
        batches.insert(&ArrowPayloadType::Logs, mk_batch(1));
        batches.insert(&ArrowPayloadType::Spans, mk_batch(10)); // should be skipped (no table mapping)

        // Act
        let stats = writer.write_batches(&batches).await.unwrap();
        let calls = writer.calls.lock().unwrap().clone();

        // Assert: only Logs written
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, ArrowPayloadType::Logs);
        assert_eq!(calls[0].1, "logs".to_string());
        assert_eq!(calls[0].2, 1);

        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].0, ArrowPayloadType::Logs);
        assert_eq!(stats[0].1, 1);
    }

    #[tokio::test]
    async fn empty_input_writes_nothing() {
        let writer = TestWriter::default();
        let batches: HashMap<&ArrowPayloadType, RecordBatch> = HashMap::new();

        let result = writer.write_batches(&batches).await.unwrap();
        let calls = writer.calls.lock().unwrap().clone();

        assert!(result.is_empty());
        assert!(calls.is_empty());
    }
}
