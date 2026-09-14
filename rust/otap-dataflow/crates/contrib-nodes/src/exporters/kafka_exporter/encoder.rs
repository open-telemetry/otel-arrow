// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Encoding logic for converting OTAP payloads to bytes for Kafka.

use super::error::KafkaExporterError;
use otel_arrow_dfe_pdata::{OtapArrowRecords, Producer as PdataProducer};
use otel_arrow_dfe_pdata::{OtapPayload, OtlpProtoBytes, TryIntoWithOptions};

use prost::Message as ProstMessage;

/// Encodes an OTAP payload to OTLP protobuf bytes.
///
/// This function handles both payload types:
/// - `OtlpProtoBytes`: Returns the bytes as-is
/// - `OtapArrowRecords`: Converts to OTLP protobuf using the built-in encoder
///
/// # Arguments
///
/// * `payload` - The OTAP payload to encode
///
/// # Returns
///
/// A vector of bytes containing the OTLP protobuf representation,
/// ready to be sent to Kafka.
pub fn encode_to_otlp_bytes(payload: OtapPayload) -> Result<Vec<u8>, KafkaExporterError> {
    // Convert payload to OTLP protobuf bytes
    // This uses the built-in TryFrom implementation that handles both cases:
    // - OtlpProtoBytes -> return as-is
    // - OtapArrowRecords -> encode using LogsProtoBytesEncoder, MetricsProtoBytesEncoder, etc.
    let otlp_bytes: OtlpProtoBytes = payload
        .try_into_with_default()
        .map_err(|e| KafkaExporterError::OtlpConversion(format!("{}", e)))?;

    // Extract the bytes from the OTLP wrapper
    Ok(otlp_bytes.as_bytes().to_vec())
}

/// Encodes an OTAP payload to BatchArrowRecord bytes.
///
/// # Arguments
///
/// * `payload` - The OTAP payload to encode
/// * `producer` - The OTAP PdataProducer used to encode BatchArrowRecords
///
/// # Returns
///
/// A vector of bytes containing the BatchArrowRecord byte representation,
/// ready to be sent to Kafka.
pub fn encode_to_batch_arrow_record_bytes(
    payload: OtapPayload,
    producer: &mut PdataProducer,
) -> Result<Vec<u8>, KafkaExporterError> {
    let mut otap_records: OtapArrowRecords = payload
        .try_into_with_default()
        .map_err(|e| KafkaExporterError::OtapArrowRecordsConversion(format!("{}", e)))?;
    let bar = producer
        .produce_bar(&mut otap_records)
        .map_err(|e| KafkaExporterError::BatchArrowRecordConversion(format!("{}", e)))?;
    producer
        .reset_streams()
        .map_err(|e| KafkaExporterError::BatchArrowRecordConversion(format!("{}", e)))?;
    Ok(bar.encode_to_vec())
}

/// Builds an OTAP logs `OtapArrowRecords` whose dictionary-encoded attribute
/// value column has a null row carrying a stale physical key equal to the
/// dictionary length (2). Arrow leaves arbitrary values in null key slots
/// (e.g. after `take` reorders keys) and skips bounds validation for those
/// slots, so this batch is valid to construct but exercises the out-of-range
/// stale-key path during transport-optimized encoding.
#[cfg(test)]
pub(crate) fn logs_otap_records_with_stale_dict_key() -> OtapArrowRecords {
    use arrow::array::NullBufferBuilder;
    use arrow::array::{DictionaryArray, RecordBatch, StringArray, UInt8Array, UInt16Array};
    use arrow::buffer::ScalarBuffer;
    use arrow::datatypes::{DataType, Field, Schema};
    use otel_arrow_dfe_pdata::otap::Logs;
    use otel_arrow_dfe_pdata::otlp::attributes::AttributeValueType;
    use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
    use otel_arrow_dfe_pdata::schema::{FieldExt, consts};
    use std::sync::Arc;

    // Root Logs batch with 4 rows so the attributes actually get sorted and
    // transport-optimized (batches with <= 1 row skip the sort path).
    let logs_batch = RecordBatch::try_new(
        Arc::new(Schema::new(vec![Field::new(
            consts::ID,
            DataType::UInt16,
            false,
        )])),
        vec![Arc::new(UInt16Array::from_iter_values([0, 1, 2, 3]))],
    )
    .expect("logs root batch is valid");

    // Dictionary value column with exactly 2 dict entries. Build the keys
    // from a raw physical buffer plus a separate null bitmap so that a NULL
    // slot carries a stale physical key equal to the dictionary length (2).
    // Arrow skips bounds validation for null key slots, so this is a valid
    // array to construct but exercises the out-of-range stale-key path.
    let str_values = Arc::new(StringArray::from_iter_values(["va", "vb"]));
    let raw_keys = ScalarBuffer::<u16>::from(vec![0u16, 1u16, 0u16, 2u16]);
    let mut null_builder = NullBufferBuilder::new(4);
    null_builder.append_non_null();
    null_builder.append_non_null();
    null_builder.append_non_null();
    null_builder.append_null(); // stale physical key 2 lives in this null slot
    let keys = UInt16Array::new(raw_keys, null_builder.finish());
    let str_col = Arc::new(DictionaryArray::new(keys, str_values));

    let attrs_batch = RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            // PARENT_ID must be marked PLAIN (not yet transport-optimized) so
            // apply_transport_optimized_encodings actually runs the attribute
            // encoding path instead of skipping an already-encoded batch.
            Field::new(consts::PARENT_ID, DataType::UInt16, false)
                .with_encoding(consts::metadata::encodings::PLAIN),
            Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
            Field::new(
                consts::ATTRIBUTE_KEY,
                DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                false,
            ),
            Field::new(
                consts::ATTRIBUTE_STR,
                DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                true,
            ),
        ])),
        vec![
            Arc::new(UInt16Array::from_iter_values([0, 1, 2, 3])),
            // All rows are Str-typed so the ATTRIBUTE_STR dictionary value
            // column is actually sorted/encoded (the value sorter is keyed by
            // attribute type). With a non-Str type the str column is skipped.
            Arc::new(UInt8Array::from_iter_values([
                AttributeValueType::Str as u8,
                AttributeValueType::Str as u8,
                AttributeValueType::Str as u8,
                AttributeValueType::Str as u8,
            ])),
            Arc::new(DictionaryArray::new(
                UInt8Array::from_iter_values([0, 0, 0, 0]),
                Arc::new(StringArray::from_iter_values(["ka"])),
            )),
            str_col,
        ],
    )
    .expect("attrs batch is valid");

    let mut records = OtapArrowRecords::Logs(Logs::default());
    records
        .set(ArrowPayloadType::Logs, logs_batch)
        .expect("set logs batch");
    records
        .set(ArrowPayloadType::LogAttrs, attrs_batch)
        .expect("set log attrs batch");

    records
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_pdata::Producer;

    #[test]
    fn test_encode_otlp_bytes_passthrough() {
        // Basic smoke test: OtlpProtoBytes should pass through unchanged
        let bytes = vec![1, 2, 3, 4, 5];
        let otlp_bytes = OtlpProtoBytes::ExportTracesRequest(bytes.clone().into());
        let payload = OtapPayload::from(otlp_bytes);

        let result = encode_to_otlp_bytes(payload).expect("encoding should succeed");
        assert_eq!(result, bytes);
    }

    /// Scenario: the Kafka OTAP serialization path encodes a logs batch whose
    /// dictionary-encoded attribute value column has a null row whose raw
    /// physical key buffer holds a stale index equal to the dictionary length
    /// (as produced by Arrow `take` reordering keys, which leaves arbitrary
    /// values in null key slots).
    /// Guarantees: encode_to_batch_arrow_record_bytes (produce_bar ->
    /// encode_transport_optimized -> transport_optimize_encode_attrs) does not
    /// panic on out-of-range stale physical keys in null slots and returns
    /// encoded BatchArrowRecord bytes.
    #[test]
    fn encode_to_batch_arrow_record_bytes_handles_dict_null_slot_with_stale_key() {
        let payload = OtapPayload::from(logs_otap_records_with_stale_dict_key());
        let mut producer = Producer::new();

        let bytes = encode_to_batch_arrow_record_bytes(payload, &mut producer)
            .expect("OTAP encoding should not panic or fail on stale null dictionary keys");
        assert!(
            !bytes.is_empty(),
            "encoded BatchArrowRecord bytes must be non-empty"
        );
    }
}
