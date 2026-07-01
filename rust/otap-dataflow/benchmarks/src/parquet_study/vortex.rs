// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Vortex file-format contender (enabled by the `vortex` cargo feature).
//!
//! Vortex is a next-generation columnar file format that interoperates with
//! Arrow. This contender reuses the same flattening as [`super::nested`] (one
//! flat Arrow `RecordBatch` per logs batch) but serializes it to an in-memory
//! Vortex file instead of Parquet, then reads it back and unflattens to OTAP.
//!
//! Vortex applies its own cascading compression (BtrBlocks), so the study's
//! [`Compressor`](super::Compressor) axis does not apply; the single setting is
//! reported as `none` (meaning "Vortex default encodings").

use std::sync::{Arc, OnceLock};

use arrow::array::{Array, ArrayRef as ArrowArrayRef, RecordBatch, StructArray};
use arrow::compute::cast;
use arrow::datatypes::{DataType, Field, Fields, Schema};
use otap_df_pdata::otap::OtapArrowRecords;
use otap_df_pdata::schema::consts;

use vortex::VortexSessionDefault;
use vortex::array::ArrayRef;
use vortex::array::VortexSessionExecute;
use vortex::array::arrow::ArrowSessionExt;
use vortex::array::arrow::FromArrowArray;
use vortex::array::stream::ArrayStreamExt;
use vortex::buffer::ByteBuffer;
use vortex::compressor::BtrBlocksCompressorBuilder;
use vortex::file::{OpenOptionsSessionExt, WriteOptionsSessionExt, WriteStrategyBuilder};
use vortex::io::session::RuntimeSessionExt;
use vortex::session::VortexSession;

use super::{Codec, StudyResult, nested};

/// Contender that flattens OTAP logs (nested layout) and stores them in an
/// in-memory Vortex file. `fast` selects an uncompressed, near-zero-copy write
/// strategy instead of the default BtrBlocks cascading compressor.
pub struct VortexCodec {
    /// When true, write with no compression to prioritize write throughput.
    pub fast: bool,
}

fn runtime() -> &'static tokio::runtime::Runtime {
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("build tokio runtime")
    })
}

/// True for `FixedSizeBinary` or a dictionary whose values are `FixedSizeBinary`.
/// Vortex 0.75 has no `FixedSizeBinary` encoding, so such columns (the logs
/// `trace_id`/`span_id`, which are dictionary-encoded in the OTAP form) must be
/// rewritten before writing.
fn is_fixed_size_binary(dt: &DataType) -> bool {
    match dt {
        DataType::FixedSizeBinary(_) => true,
        DataType::Dictionary(_, value) => matches!(value.as_ref(), DataType::FixedSizeBinary(_)),
        _ => false,
    }
}

/// Cast any `FixedSizeBinary`-valued column to plain `Binary` before writing.
fn to_vortex_compatible(flat: &RecordBatch) -> StudyResult<RecordBatch> {
    let mut fields: Vec<Field> = Vec::with_capacity(flat.num_columns());
    let mut columns: Vec<ArrowArrayRef> = Vec::with_capacity(flat.num_columns());
    for (field, column) in flat.schema().fields().iter().zip(flat.columns()) {
        if is_fixed_size_binary(field.data_type()) {
            fields.push(Field::new(
                field.name(),
                DataType::Binary,
                field.is_nullable(),
            ));
            columns.push(cast(column, &DataType::Binary)?);
        } else {
            fields.push(field.as_ref().clone());
            columns.push(column.clone());
        }
    }
    Ok(RecordBatch::try_new(
        Arc::new(Schema::new(fields)),
        columns,
    )?)
}

/// Reduce a data type to a "plain" Arrow type: decode dictionaries to their
/// value type and turn `FixedSizeBinary` into `Binary`, recursing through struct
/// and list children. Used as the target for Vortex's `execute_arrow` so the
/// decoded batch has deterministic, non-view, non-dictionary types that the OTAP
/// schema check accepts. (`trace_id`/`span_id` are later restored to
/// `FixedSizeBinary`.)
fn plainify(dt: &DataType) -> DataType {
    match dt {
        DataType::Dictionary(_, value) => plainify(value),
        DataType::FixedSizeBinary(_) => DataType::Binary,
        DataType::Struct(fields) => DataType::Struct(plainify_fields(fields)),
        DataType::List(field) => DataType::List(Arc::new(plainify_field(field))),
        DataType::LargeList(field) => DataType::List(Arc::new(plainify_field(field))),
        other => other.clone(),
    }
}

fn plainify_field(field: &Field) -> Field {
    Field::new(
        field.name(),
        plainify(field.data_type()),
        field.is_nullable(),
    )
}

fn plainify_fields(fields: &Fields) -> Fields {
    Fields::from(fields.iter().map(|f| plainify_field(f)).collect::<Vec<_>>())
}

/// The Arrow struct field describing the plain type Vortex should decode into.
/// Derived once from a sample flattened batch (the flat schema is deterministic).
fn vortex_target_field() -> &'static Field {
    static TARGET: OnceLock<Field> = OnceLock::new();
    TARGET.get_or_init(|| {
        use crate::parquet_study::datagen::{LogsGenParams, gen_logs_otap};
        let (otap, _) = gen_logs_otap(&LogsGenParams {
            num_resources: 1,
            num_scopes: 1,
            num_logs: 1,
        });
        let flat = nested::flatten(&otap).expect("sample flatten");
        let compat = to_vortex_compatible(&flat).expect("sample compat");
        Field::new(
            "",
            DataType::Struct(plainify_fields(compat.schema().fields())),
            false,
        )
    })
}

/// Restore the `FixedSizeBinary` columns that [`to_vortex_compatible`] cast to
/// `Binary`, so the reconstructed Logs batch matches the OTAP schema.
fn restore_fixed_size_binary(batch: RecordBatch) -> StudyResult<RecordBatch> {
    let restores = [(consts::TRACE_ID, 16i32), (consts::SPAN_ID, 8i32)];
    let mut fields: Vec<Field> = batch
        .schema()
        .fields()
        .iter()
        .map(|f| f.as_ref().clone())
        .collect();
    let mut columns: Vec<ArrowArrayRef> = batch.columns().to_vec();
    for (name, size) in restores {
        if let Ok(idx) = batch.schema().index_of(name) {
            if matches!(
                columns[idx].data_type(),
                DataType::Binary | DataType::LargeBinary
            ) {
                columns[idx] = cast(&columns[idx], &DataType::FixedSizeBinary(size))?;
                fields[idx] = Field::new(
                    name,
                    DataType::FixedSizeBinary(size),
                    fields[idx].is_nullable(),
                );
            }
        }
    }
    Ok(RecordBatch::try_new(
        Arc::new(Schema::new(fields)),
        columns,
    )?)
}

/// Encode a flat Arrow record batch as an in-memory Vortex file. When `fast` is
/// set, a no-op compressor is used so the write is essentially a canonical
/// (uncompressed) layout, skipping the BtrBlocks encoding search.
fn encode(flat: RecordBatch, fast: bool) -> StudyResult<Vec<u8>> {
    let flat = to_vortex_compatible(&flat)?;
    runtime()
        .block_on(async move {
            let session = VortexSession::default().with_tokio();
            let array = ArrayRef::from_arrow(flat, false)?;
            let mut sink: Vec<u8> = Vec::with_capacity(4096);
            let write_options = session.write_options();
            let stream = array.to_array_stream();
            let _summary = if fast {
                let strategy = WriteStrategyBuilder::default()
                    .with_btrblocks_builder(BtrBlocksCompressorBuilder::empty())
                    .build();
                write_options
                    .with_strategy(strategy)
                    .write(&mut sink, stream)
                    .await?
            } else {
                write_options.write(&mut sink, stream).await?
            };
            Ok::<Vec<u8>, vortex::error::VortexError>(sink)
        })
        .map_err(Into::into)
}

/// Decode an in-memory Vortex file back into a flat Arrow record batch.
fn decode(bytes: &[u8]) -> StudyResult<RecordBatch> {
    let buffer = ByteBuffer::from(bytes.to_vec());
    let batch = runtime()
        .block_on(async move {
            let session = VortexSession::default().with_tokio();
            let file = session.open_options().open_buffer(buffer)?;
            let array = file.scan()?.into_array_stream()?.read_all().await?;
            let mut ctx = session.create_execution_ctx();
            let arrow_session = session.arrow();
            let arrow =
                arrow_session.execute_arrow(array, Some(vortex_target_field()), &mut ctx)?;
            let struct_array = arrow
                .as_any()
                .downcast_ref::<StructArray>()
                .expect("vortex top-level array is a struct");
            Ok::<RecordBatch, vortex::error::VortexError>(RecordBatch::from(struct_array.clone()))
        })
        .map_err(Box::<dyn std::error::Error + Send + Sync>::from)?;
    restore_fixed_size_binary(batch)
}

impl Codec for VortexCodec {
    fn name(&self) -> &'static str {
        if self.fast { "vortex-fast" } else { "vortex" }
    }

    fn write(&self, logs: OtapArrowRecords) -> StudyResult<Vec<u8>> {
        let flat = nested::flatten(&logs)?;
        encode(flat, self.fast)
    }

    fn read(&self, bytes: &[u8]) -> StudyResult<OtapArrowRecords> {
        let flat = decode(bytes)?;
        nested::unflatten(&flat)
    }
}

/// Decode a Vortex file back into the flat Arrow record batch, without
/// unflattening to OTAP. Used by the server-cost model's "reparse" measurement.
pub fn reparse_to_arrow(bytes: &[u8]) -> StudyResult<RecordBatch> {
    decode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parquet_study::attrs::assert_logs_equivalent;
    use crate::parquet_study::datagen::{LogsGenParams, gen_logs_otap};

    #[test]
    fn vortex_round_trip_preserves_structure() {
        let params = LogsGenParams {
            num_resources: 3,
            num_scopes: 2,
            num_logs: 5,
        };
        let (otap, _) = gen_logs_otap(&params);

        for fast in [false, true] {
            let codec = VortexCodec { fast };
            let bytes = codec.write(otap.clone()).expect("write");
            assert!(!bytes.is_empty());
            let decoded = codec.read(&bytes).expect("read");
            assert_logs_equivalent(&otap, &decoded, codec.name(), "none");
        }
    }
}
