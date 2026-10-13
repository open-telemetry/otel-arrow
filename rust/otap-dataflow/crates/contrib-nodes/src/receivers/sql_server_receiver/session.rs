// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Parameterized metadata discovery and incremental consumption of Tiberius streams.

use super::{
    SqlServerError as Error, SqlServerParameter, SqlServerStatement,
    data::{ColumnPlan, MAX_COLUMNS, MAX_METADATA_BYTES, NativeType, ResultPlan},
};
use futures::{
    TryStreamExt,
    io::{AsyncRead, AsyncWrite},
};
use otel_arrow_dfe_scraper::database::{
    ColumnMetadata, CompiledQuery, Cursor, CursorRow, QueryPage,
};
use std::mem::size_of;
use tiberius::{Client, QueryItem};

const DESCRIBE: &str = "SELECT column_ordinal, name, is_nullable, system_type_id, \
    system_type_name, max_length, precision, scale, user_type_name, error_number \
    FROM sys.dm_exec_describe_first_result_set(@P1, @P2, 0) ORDER BY column_ordinal";

pub(super) async fn describe<S>(
    client: &mut Client<S>,
    statement: &SqlServerStatement,
    query: &CompiledQuery,
) -> Result<ResultPlan, Error>
where
    S: AsyncRead + AsyncWrite + Unpin + Send,
{
    let declarations = statement
        .parameters()
        .iter()
        .enumerate()
        .map(|(index, role)| {
            let kind = match role {
                SqlServerParameter::Timestamp => "datetime2(7)",
                _ => "bigint",
            };
            format!("@P{} {kind}", index + 1)
        })
        .collect::<Vec<_>>()
        .join(", ");
    let mut request = tiberius::Query::new(DESCRIBE);
    request.bind(statement.driver_sql());
    request.bind(declarations);
    let mut stream = request.query(client).await.map_err(Error::from)?;
    let mut columns = Vec::new();
    let mut bytes = 0usize;
    while let Some(item) = stream.try_next().await.map_err(Error::from)? {
        match item {
            QueryItem::Metadata(metadata) if metadata.result_index() == 0 => {}
            QueryItem::Row(row) if row.result_index() == 0 => {
                if columns.len() == MAX_COLUMNS
                    || row
                        .try_get::<i32, _>("error_number")
                        .map_err(Error::from)?
                        .is_some()
                    || row
                        .try_get::<&str, _>("user_type_name")
                        .map_err(Error::from)?
                        .is_some()
                {
                    return Err(Error::Metadata);
                }
                let ordinal = row
                    .try_get::<i32, _>("column_ordinal")
                    .map_err(Error::from)?
                    .ok_or(Error::Metadata)?;
                if usize::try_from(ordinal).ok() != Some(columns.len() + 1) {
                    return Err(Error::Metadata);
                }
                let name = row
                    .try_get::<&str, _>("name")
                    .map_err(Error::from)?
                    .ok_or(Error::Metadata)?;
                let source_type = row
                    .try_get::<&str, _>("system_type_name")
                    .map_err(Error::from)?
                    .ok_or(Error::Metadata)?;
                bytes = bytes
                    .checked_add(name.len() + source_type.len() + size_of::<ColumnPlan>())
                    .ok_or(Error::Metadata)?;
                if bytes > MAX_METADATA_BYTES {
                    return Err(Error::Metadata);
                }
                let kind = NativeType::from_system_id(
                    row.try_get::<i32, _>("system_type_id")
                        .map_err(Error::from)?
                        .ok_or(Error::Metadata)?,
                )?;
                let nullable = row
                    .try_get::<bool, _>("is_nullable")
                    .map_err(Error::from)?
                    .ok_or(Error::Metadata)?;
                let precision = row
                    .try_get::<u8, _>("precision")
                    .map_err(Error::from)?
                    .ok_or(Error::Metadata)?;
                let scale = row
                    .try_get::<u8, _>("scale")
                    .map_err(Error::from)?
                    .ok_or(Error::Metadata)?;
                let max_length = row
                    .try_get::<i16, _>("max_length")
                    .map_err(Error::from)?
                    .ok_or(Error::Metadata)?;
                columns.push(ColumnPlan {
                    metadata: ColumnMetadata {
                        name: name.to_owned(),
                        source_type: if kind == NativeType::Binary {
                            "varbinary".to_owned()
                        } else {
                            source_type.to_owned()
                        },
                        nullable,
                    },
                    kind,
                    precision,
                    scale,
                    max_length,
                });
            }
            _ => return Err(Error::Metadata),
        }
    }
    ResultPlan::new(columns, query)
}

pub(super) struct FetchedPage {
    pub page: QueryPage,
    /// A byte-limited prefix left unread results; cancel before returning or reusing a connection.
    pub unfinished: bool,
}

pub(super) async fn fetch<S>(
    client: &mut Client<S>,
    statement: &SqlServerStatement,
    plan: &ResultPlan,
    query: &CompiledQuery,
    cursor: &Cursor,
) -> Result<FetchedPage, Error>
where
    S: AsyncRead + AsyncWrite + Unpin + Send,
{
    let request = plan.request(statement, query, cursor)?;
    let mut stream = request.query(client).await.map_err(Error::from)?;
    let mut rows = Vec::new();
    let mut bytes = 0u64;
    let snapshot = matches!(cursor, Cursor::Snapshot);
    let mut unfinished = false;
    while let Some(item) = stream.try_next().await.map_err(Error::from)? {
        match item {
            QueryItem::Metadata(metadata) if metadata.result_index() == 0 => {
                plan.check_columns(metadata.columns())?
            }
            QueryItem::Row(row) if row.result_index() == 0 => {
                if rows.len() == query.max_rows() {
                    return Err(if snapshot {
                        Error::SnapshotLimit
                    } else {
                        Error::Driver
                    });
                }
                // Tiberius already allocated the native row. This bounds the normalized
                // page, not driver-internal buffers or process RSS.
                let row = plan.row(row)?;
                let row_bytes = row
                    .row
                    .normalized_size()
                    .saturating_add(cursor_bytes(&row.cursor));
                let required_capacity = rows.len() + 1;
                let future_capacity = if required_capacity > rows.capacity() {
                    required_capacity
                        .max(rows.capacity().saturating_mul(2))
                        .min(query.max_rows())
                } else {
                    rows.capacity()
                };
                let projected = bytes
                    .saturating_add(row_bytes)
                    .saturating_add((future_capacity * size_of::<CursorRow>()) as u64);
                if projected > query.max_normalized_bytes() {
                    if snapshot {
                        return Err(Error::SnapshotLimit);
                    }
                    if rows.is_empty() {
                        return Err(Error::OversizeRow);
                    }
                    unfinished = true;
                    break;
                }
                if future_capacity > rows.capacity() {
                    rows.try_reserve_exact(future_capacity - rows.len())
                        .map_err(|_| Error::OversizeRow)?;
                }
                let retained = bytes
                    .saturating_add(row_bytes)
                    .saturating_add((rows.capacity() * size_of::<CursorRow>()) as u64);
                if retained > query.max_normalized_bytes() {
                    return Err(if snapshot {
                        Error::SnapshotLimit
                    } else {
                        Error::OversizeRow
                    });
                }
                bytes += row_bytes;
                rows.push(row);
                if rows.len() % query.fetch_size_rows() == 0 {
                    tokio::task::yield_now().await;
                }
            }
            _ => return Err(Error::Metadata),
        }
    }
    Ok(FetchedPage {
        page: QueryPage {
            columns: plan.metadata(),
            rows,
        },
        unfinished,
    })
}

fn cursor_bytes(cursor: &Cursor) -> u64 {
    match cursor {
        Cursor::Composite(value) => value.timestamp.capacity() as u64,
        Cursor::Scalar(otel_arrow_dfe_scraper::database::ScalarValue::Timestamp(value)) => {
            value.capacity() as u64
        }
        _ => 0,
    }
}
