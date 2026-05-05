// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::decode::record_message::RecordMessage;
use crate::error::{Error, Result};
use crate::otap::{OtapArrowRecords, from_record_messages};
use crate::otlp::logs::LogsProtoBytesEncoder;
use crate::otlp::metrics::MetricsProtoBytesEncoder;
use crate::otlp::traces::TracesProtoBytesEncoder;
use crate::otlp::{ProtoBuffer, ProtoBytesEncoder};
use crate::proto::opentelemetry::arrow::v1::{ArrowPayload, ArrowPayloadType, BatchArrowRecords};
use crate::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
use crate::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
use crate::proto::opentelemetry::collector::trace::v1::ExportTraceServiceRequest;
use arrow::array::RecordBatch;
use arrow::error::ArrowError;
use arrow::ipc::reader::StreamReader;
use prost::Message;
use std::collections::HashMap;
use std::io::Cursor;

pub struct StreamConsumer {
    payload_type: ArrowPayloadType,
    stream_reader: StreamReader<Cursor<Vec<u8>>>,
}

impl StreamConsumer {
    fn try_new(payload: ArrowPayloadType, initial_bytes: Vec<u8>) -> Result<Self> {
        let data = Cursor::new(initial_bytes);
        let stream_reader = StreamReader::try_new(data.clone(), None)
            .map_err(|e| Error::BuildStreamReader { source: e })?;
        Ok(Self {
            payload_type: payload,
            stream_reader,
        })
    }

    fn replace_bytes(&mut self, bytes: Vec<u8>) {
        *self.stream_reader.get_mut() = Cursor::new(bytes);
    }

    fn next(&mut self) -> Option<std::result::Result<RecordBatch, ArrowError>> {
        self.stream_reader.next()
    }
}

/// Consumer consumes OTAP `BatchArrowRecords` and can convert them into OTLP messages.
#[derive(Default)]
pub struct Consumer {
    stream_consumers: HashMap<String, StreamConsumer>,
    logs_proto_encoder: LogsProtoBytesEncoder,
    metrics_proto_encoder: MetricsProtoBytesEncoder,
    traces_proto_encoder: TracesProtoBytesEncoder,
    proto_buffer: ProtoBuffer,
}

impl Consumer {
    /// consume and deserialize record batches
    pub fn consume_bar(&mut self, bar: &mut BatchArrowRecords) -> Result<Vec<RecordMessage>> {
        let mut records = Vec::with_capacity(bar.arrow_payloads.len());

        for payload in std::mem::take(&mut bar.arrow_payloads) {
            let ArrowPayload {
                schema_id,
                r#type,
                record,
            } = payload;
            let payload_type = ArrowPayloadType::try_from(r#type)
                .map_err(|_| Error::UnsupportedPayloadType { actual: r#type })?;

            let stream_consumer = match self.stream_consumers.get_mut(&schema_id) {
                None => {
                    // stream consumer does not exist, remove all stream consumer with
                    // the same payload_type since schema already changed for that payload.
                    let new_stream_consumer: HashMap<String, StreamConsumer> =
                        (std::mem::take(&mut self.stream_consumers))
                            .into_iter()
                            .filter(|(_, v)| v.payload_type != payload_type)
                            .collect::<HashMap<_, _>>();
                    self.stream_consumers = new_stream_consumer;
                    self.stream_consumers
                        .entry(schema_id.clone())
                        .or_insert(StreamConsumer::try_new(payload_type, record)?)
                }
                Some(s) => {
                    // stream consumer exists for given schema id, just reset the bytes.
                    s.replace_bytes(record);
                    s
                }
            };

            if let Some(rs) = stream_consumer.next() {
                // the encoder side ensures there should be only one record here.
                let record = rs.map_err(|e| Error::ReadRecordBatch { source: e })?;
                records.push(RecordMessage {
                    batch_id: bar.batch_id,
                    schema_id,
                    payload_type,
                    record,
                });
            } else {
                //todo: handle stream reader finished
            }
        }
        Ok(records)
    }

    /// Consumes all the arrow payloads in the passed OTAP `BatchArrayRecords` and decodes them
    /// into OTLP messages, then constructs the `ExportMetricsServiceRequest` containing the
    /// metrics messages
    pub fn consume_metrics_batches(
        &mut self,
        records: &mut BatchArrowRecords,
    ) -> Result<ExportMetricsServiceRequest> {
        let record_messages = self.consume_bar(records)?;
        let mut otap_batch = OtapArrowRecords::Metrics(from_record_messages(record_messages)?);

        if otap_batch
            .get(ArrowPayloadType::UnivariateMetrics)
            .is_none()
        {
            return Err(Error::RecordBatchNotFound {
                payload_type: ArrowPayloadType::UnivariateMetrics,
            });
        }

        self.proto_buffer.clear();
        self.metrics_proto_encoder
            .encode(&mut otap_batch, &mut self.proto_buffer)?;

        ExportMetricsServiceRequest::decode(self.proto_buffer.as_ref()).map_err(|e| {
            Error::UnexpectedRecordBatchState {
                reason: format!("error decoding proto serialization: {e:?}"),
            }
        })
    }

    /// Consumes all the arrow payloads in the passed OTAP `BatchArrayRecords` and decodes them
    /// into OTLP messages, then constructs the `ExportLogsServiceRequest` containing the
    /// logs messages
    pub fn consume_logs_batches(
        &mut self,
        records: &mut BatchArrowRecords,
    ) -> Result<ExportLogsServiceRequest> {
        let record_messages = self.consume_bar(records)?;
        let mut otap_batch = OtapArrowRecords::Logs(from_record_messages(record_messages)?);

        if otap_batch.get(ArrowPayloadType::Logs).is_none() {
            return Err(Error::RecordBatchNotFound {
                payload_type: ArrowPayloadType::Logs,
            });
        }

        self.proto_buffer.clear();
        self.logs_proto_encoder
            .encode(&mut otap_batch, &mut self.proto_buffer)?;

        ExportLogsServiceRequest::decode(self.proto_buffer.as_ref()).map_err(|e| {
            Error::UnexpectedRecordBatchState {
                reason: format!("error decoding proto serialization: {e:?}"),
            }
        })
    }

    /// Consumes record batches in [BatchArrowRecords] to [ExportTraceServiceRequest].
    pub fn consume_traces_batches(
        &mut self,
        records: &mut BatchArrowRecords,
    ) -> Result<ExportTraceServiceRequest> {
        let record_messages = self.consume_bar(records)?;
        let mut otap_batch = OtapArrowRecords::Traces(from_record_messages(record_messages)?);

        if otap_batch.get(ArrowPayloadType::Spans).is_none() {
            return Err(Error::RecordBatchNotFound {
                payload_type: ArrowPayloadType::Spans,
            });
        }

        self.proto_buffer.clear();
        self.traces_proto_encoder
            .encode(&mut otap_batch, &mut self.proto_buffer)?;

        ExportTraceServiceRequest::decode(self.proto_buffer.as_ref()).map_err(|e| {
            Error::UnexpectedRecordBatchState {
                reason: format!("error decoding proto serialization {e:?}"),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use arrow::array::{
        ArrayRef, BinaryArray, BooleanArray, Float32Array, Float64Array, Int8Array, Int16Array,
        Int32Array, Int64Array, RecordBatch, StringArray, TimestampMicrosecondArray,
        TimestampMillisecondArray, TimestampNanosecondArray, TimestampSecondArray, UInt8Array,
        UInt16Array, UInt32Array, UInt64Array,
    };
    use arrow::datatypes::{DataType, Field, Schema, SchemaRef, TimeUnit};
    use rand::RngExt;
    use rand::distr::{Alphanumeric, SampleString};
    use std::io::Cursor;
    use std::sync::Arc;

    fn create_test_schema() -> Schema {
        Schema::new(vec![
            Field::new("a", DataType::UInt16, true),
            Field::new("b", DataType::Utf8, true),
            Field::new("c", DataType::Float64, true),
        ])
    }

    fn create_record_batch(schema: SchemaRef, num_rows: usize) -> RecordBatch {
        let columns = schema
            .fields
            .iter()
            .map(|f| create_array(f.data_type(), num_rows))
            .collect::<Vec<_>>();
        RecordBatch::try_new(schema, columns).unwrap()
    }

    fn create_array(dt: &DataType, num_rows: usize) -> ArrayRef {
        let mut r = rand::rng();
        match dt {
            DataType::Boolean => Arc::new(
                (0..num_rows)
                    .map(|_| Some(r.random_bool(1.0 / 2.0)))
                    .collect::<BooleanArray>(),
            ) as Arc<_>,
            DataType::Int8 => Arc::new(Int8Array::from_iter(
                (0..num_rows).map(|_| r.random::<i8>()),
            )) as Arc<_>,
            DataType::Int16 => Arc::new(Int16Array::from_iter(
                (0..num_rows).map(|_| r.random::<i16>()),
            )) as Arc<_>,
            DataType::Int32 => Arc::new(Int32Array::from_iter(
                (0..num_rows).map(|_| r.random::<i32>()),
            )) as Arc<_>,
            DataType::Int64 => Arc::new(Int64Array::from_iter(
                (0..num_rows).map(|_| r.random::<i64>()),
            )) as Arc<_>,
            DataType::UInt8 => Arc::new(UInt8Array::from_iter(
                (0..num_rows).map(|_| r.random::<u8>()),
            )) as Arc<_>,
            DataType::UInt16 => Arc::new(UInt16Array::from_iter(
                (0..num_rows).map(|_| r.random::<u16>()),
            )) as Arc<_>,
            DataType::UInt32 => Arc::new(UInt32Array::from_iter(
                (0..num_rows).map(|_| r.random::<u32>()),
            )) as Arc<_>,
            DataType::UInt64 => Arc::new(UInt64Array::from_iter(
                (0..num_rows).map(|_| r.random::<u64>()),
            )) as Arc<_>,
            DataType::Float32 => Arc::new(Float32Array::from_iter(
                (0..num_rows).map(|_| r.random::<f32>()),
            )) as Arc<_>,
            DataType::Float64 => Arc::new(Float64Array::from_iter(
                (0..num_rows).map(|_| r.random::<f64>()),
            )) as Arc<_>,
            DataType::Timestamp(unit, _) => match unit {
                TimeUnit::Second => Arc::new(TimestampSecondArray::from_iter(
                    &Int64Array::from_iter((0..num_rows).map(|_| r.random::<i64>())),
                )) as Arc<_>,
                TimeUnit::Millisecond => Arc::new(TimestampMillisecondArray::from_iter(
                    &Int64Array::from_iter((0..num_rows).map(|_| r.random::<i64>())),
                )) as Arc<_>,
                TimeUnit::Microsecond => Arc::new(TimestampMicrosecondArray::from_iter(
                    &Int64Array::from_iter((0..num_rows).map(|_| r.random::<i64>())),
                )) as Arc<_>,

                TimeUnit::Nanosecond => Arc::new(TimestampNanosecondArray::from_iter(
                    &Int64Array::from_iter((0..num_rows).map(|_| r.random::<i64>())),
                )) as Arc<_>,
            },
            DataType::Binary | DataType::LargeBinary => Arc::new(BinaryArray::from_iter(
                (0..num_rows).map(|_| Some(Alphanumeric.sample_string(&mut r, 10))),
            )) as Arc<_>,
            DataType::Utf8 => Arc::new(StringArray::from_iter(
                (0..num_rows).map(|_| Some(Alphanumeric.sample_string(&mut r, 10))),
            )) as Arc<_>,
            _ => {
                unimplemented!()
            }
        }
    }

    use super::Consumer;
    use crate::Producer;
    use crate::error::Error;
    use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
    use crate::testing::fixtures::{
        logs_with_full_resource_and_scope, metrics_sum_with_full_resource_and_scope,
        traces_with_full_resource_and_scope,
    };
    use crate::testing::round_trip::{encode_logs, encode_metrics, encode_traces};

    /// Helper: produce BatchArrowRecords from OTAP data using the Producer.
    fn produce_bar(
        otap: &mut crate::otap::OtapArrowRecords,
    ) -> crate::proto::opentelemetry::arrow::v1::BatchArrowRecords {
        let mut producer = Producer::new();
        producer
            .produce_bar(otap)
            .expect("produce_bar should succeed")
    }

    #[test]
    fn test_consume_logs_root_not_at_position_zero() {
        let logs_data = logs_with_full_resource_and_scope();
        let mut otap = encode_logs(&logs_data);
        let mut bar = produce_bar(&mut otap);

        // Ensure there are multiple payloads and move the root (Logs) to the end
        assert!(
            bar.arrow_payloads.len() > 1,
            "need multiple payloads to test reordering"
        );
        let logs_idx = bar
            .arrow_payloads
            .iter()
            .position(|p| p.r#type == ArrowPayloadType::Logs as i32)
            .expect("Logs payload should exist");
        let logs_payload = bar.arrow_payloads.remove(logs_idx);
        bar.arrow_payloads.push(logs_payload);
        // Root payload is now at the last position, not position 0
        assert_ne!(
            bar.arrow_payloads[0].r#type,
            ArrowPayloadType::Logs as i32,
            "root payload should not be at position 0 for this test"
        );

        let mut consumer = Consumer::default();
        let result = consumer.consume_logs_batches(&mut bar);
        assert!(
            result.is_ok(),
            "should decode logs even when root is not at position 0: {result:?}"
        );
    }

    #[test]
    fn test_consume_metrics_root_not_at_position_zero() {
        let metrics_data = metrics_sum_with_full_resource_and_scope();
        let mut otap = encode_metrics(&metrics_data);
        let mut bar = produce_bar(&mut otap);

        assert!(
            bar.arrow_payloads.len() > 1,
            "need multiple payloads to test reordering"
        );
        let metrics_idx = bar
            .arrow_payloads
            .iter()
            .position(|p| p.r#type == ArrowPayloadType::UnivariateMetrics as i32)
            .expect("UnivariateMetrics payload should exist");
        let metrics_payload = bar.arrow_payloads.remove(metrics_idx);
        bar.arrow_payloads.push(metrics_payload);
        assert_ne!(
            bar.arrow_payloads[0].r#type,
            ArrowPayloadType::UnivariateMetrics as i32,
            "root payload should not be at position 0 for this test"
        );

        let mut consumer = Consumer::default();
        let result = consumer.consume_metrics_batches(&mut bar);
        assert!(
            result.is_ok(),
            "should decode metrics even when root is not at position 0: {result:?}"
        );
    }

    #[test]
    fn test_consume_traces_root_not_at_position_zero() {
        let traces_data = traces_with_full_resource_and_scope();
        let mut otap = encode_traces(&traces_data);
        let mut bar = produce_bar(&mut otap);

        assert!(
            bar.arrow_payloads.len() > 1,
            "need multiple payloads to test reordering"
        );
        let spans_idx = bar
            .arrow_payloads
            .iter()
            .position(|p| p.r#type == ArrowPayloadType::Spans as i32)
            .expect("Spans payload should exist");
        let spans_payload = bar.arrow_payloads.remove(spans_idx);
        bar.arrow_payloads.push(spans_payload);
        assert_ne!(
            bar.arrow_payloads[0].r#type,
            ArrowPayloadType::Spans as i32,
            "root payload should not be at position 0 for this test"
        );

        let mut consumer = Consumer::default();
        let result = consumer.consume_traces_batches(&mut bar);
        assert!(
            result.is_ok(),
            "should decode traces even when root is not at position 0: {result:?}"
        );
    }

    #[test]
    fn test_consume_logs_missing_root_payload() {
        let logs_data = logs_with_full_resource_and_scope();
        let mut otap = encode_logs(&logs_data);
        let mut bar = produce_bar(&mut otap);

        // Remove the Logs root payload entirely
        bar.arrow_payloads
            .retain(|p| p.r#type != ArrowPayloadType::Logs as i32);

        let mut consumer = Consumer::default();
        let result = consumer.consume_logs_batches(&mut bar);
        assert!(result.is_err());
        assert!(
            matches!(result.unwrap_err(), Error::RecordBatchNotFound { payload_type } if payload_type == ArrowPayloadType::Logs),
            "should return RecordBatchNotFound for Logs"
        );
    }

    #[test]
    fn test_consume_metrics_missing_root_payload() {
        let metrics_data = metrics_sum_with_full_resource_and_scope();
        let mut otap = encode_metrics(&metrics_data);
        let mut bar = produce_bar(&mut otap);

        // Remove the UnivariateMetrics root payload entirely
        bar.arrow_payloads
            .retain(|p| p.r#type != ArrowPayloadType::UnivariateMetrics as i32);

        let mut consumer = Consumer::default();
        let result = consumer.consume_metrics_batches(&mut bar);
        assert!(result.is_err());
        assert!(
            matches!(result.unwrap_err(), Error::RecordBatchNotFound { payload_type } if payload_type == ArrowPayloadType::UnivariateMetrics),
            "should return RecordBatchNotFound for UnivariateMetrics"
        );
    }

    #[test]
    fn test_consume_traces_missing_root_payload() {
        let traces_data = traces_with_full_resource_and_scope();
        let mut otap = encode_traces(&traces_data);
        let mut bar = produce_bar(&mut otap);

        // Remove the Spans root payload entirely
        bar.arrow_payloads
            .retain(|p| p.r#type != ArrowPayloadType::Spans as i32);

        let mut consumer = Consumer::default();
        let result = consumer.consume_traces_batches(&mut bar);
        assert!(result.is_err());
        assert!(
            matches!(result.unwrap_err(), Error::RecordBatchNotFound { payload_type } if payload_type == ArrowPayloadType::Spans),
            "should return RecordBatchNotFound for Spans"
        );
    }

    #[test]
    fn test_replace_bytes() {
        let schema = Arc::new(create_test_schema());
        let mut writer = arrow::ipc::writer::StreamWriter::try_new(vec![], &schema).unwrap();

        // write and check batch1
        let batch1 = create_record_batch(schema.clone(), 10);
        writer.write(&batch1).unwrap();
        writer.flush().unwrap();
        let mut reader = arrow::ipc::reader::StreamReader::try_new(
            Cursor::new(std::mem::take(writer.get_mut())),
            None,
        )
        .unwrap();
        assert_eq!(batch1, reader.next().unwrap().unwrap());

        // write and check batch2
        *writer.get_mut() = vec![];
        let batch2 = create_record_batch(schema.clone(), 11);
        writer.write(&batch2).unwrap();
        writer.flush().unwrap();
        *reader.get_mut() = Cursor::new(std::mem::take(writer.get_mut()));
        assert_eq!(batch2, reader.next().unwrap().unwrap());
    }
}
