// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Encoding logic for converting OTAP payloads to bytes for Kafka.

use super::error::KafkaExporterError;
use otap_df_pdata::{OtapArrowRecords, Producer as PdataProducer};
use otap_df_pdata::{OtapPayload, OtlpProtoBytes, TryIntoWithOptions};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_otlp_bytes_passthrough() {
        // Basic smoke test: OtlpProtoBytes should pass through unchanged
        let bytes = vec![1, 2, 3, 4, 5];
        let otlp_bytes = OtlpProtoBytes::ExportTracesRequest(bytes.clone().into());
        let payload = OtapPayload::OtlpBytes(otlp_bytes);

        let result = encode_to_otlp_bytes(payload).expect("encoding should succeed");
        assert_eq!(result, bytes);
    }
}
