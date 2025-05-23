// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::decode::record_message::RecordMessage;
use crate::error;
use crate::otap::{OtapBatch, from_record_messages};
use crate::otlp::logs::logs_from;
use crate::otlp::metrics::metrics_from;
use crate::proto::opentelemetry::arrow::v1::{ArrowPayload, ArrowPayloadType, BatchArrowRecords};
use crate::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
use crate::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
use arrow::array::RecordBatch;
use arrow::error::ArrowError;
use arrow::ipc::reader::StreamReader;
use snafu::{ResultExt, ensure};
use std::collections::HashMap;
use std::io::Cursor;

pub struct StreamConsumer {
    payload_type: ArrowPayloadType,
    stream_reader: StreamReader<Cursor<Vec<u8>>>,
}

impl StreamConsumer {
    fn new(payload: ArrowPayloadType, initial_bytes: Vec<u8>) -> error::Result<Self> {
        let data = Cursor::new(initial_bytes);
        let stream_reader =
            StreamReader::try_new(data.clone(), None).context(error::BuildStreamReaderSnafu)?;
        Ok(Self {
            payload_type: payload,
            stream_reader,
        })
    }

    fn replace_bytes(&mut self, bytes: Vec<u8>) {
        *self.stream_reader.get_mut() = Cursor::new(bytes);
    }

    fn next(&mut self) -> Option<Result<RecordBatch, ArrowError>> {
        self.stream_reader.next()
    }
}

/// Consumer consumes OTAP `BatchArrowRecords` and converts them into OTLP messages.
#[derive(Default)]
pub struct Consumer {
    stream_consumers: HashMap<String, StreamConsumer>,
}

impl Consumer {
    /// consume and deserialize record batches
    pub fn consume_bar(
        &mut self,
        bar: &mut BatchArrowRecords,
    ) -> error::Result<Vec<RecordMessage>> {
        let mut records = Vec::with_capacity(bar.arrow_payloads.len());

        for payload in std::mem::take(&mut bar.arrow_payloads) {
            let ArrowPayload {
                schema_id,
                r#type,
                record,
            } = payload;
            let payload_type = ArrowPayloadType::try_from(r#type)
                .map_err(|_| error::UnsupportedPayloadTypeSnafu { actual: r#type }.build())?;

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
                        .or_insert(StreamConsumer::new(payload_type, record)?)
                }
                Some(s) => {
                    // stream consumer exists for given schema id, just reset the bytes.
                    s.replace_bytes(record);
                    s
                }
            };

            if let Some(rs) = stream_consumer.next() {
                // the encoder side ensures there should be only one record here.
                let record = rs.context(error::ReadRecordBatchSnafu)?;
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
    ) -> error::Result<ExportMetricsServiceRequest> {
        match get_main_payload_type(records)? {
            ArrowPayloadType::UnivariateMetrics => {
                let record_messages = self.consume_bar(records)?;
                let otap_batch = OtapBatch::Metrics(from_record_messages(record_messages));
                metrics_from(otap_batch)
            }
            main_record_type => error::UnsupportedPayloadTypeSnafu {
                actual: main_record_type,
            }
            .fail(),
        }
    }

    /// Consumes all the arrow payloads in the passed OTAP `BatchArrayRecords` and decodes them
    /// into OTLP messages, then constructs the `ExportLogsServiceRequest` containing the
    /// logs messages
    pub fn consume_logs_batches(
        &mut self,
        records: &mut BatchArrowRecords,
    ) -> error::Result<ExportLogsServiceRequest> {
        match get_main_payload_type(records)? {
            ArrowPayloadType::Logs => {
                let record_messages = self.consume_bar(records)?;
                let otap_batch = OtapBatch::Logs(from_record_messages(record_messages));
                logs_from(otap_batch)
            }
            main_record_type => error::UnsupportedPayloadTypeSnafu {
                actual: main_record_type,
            }
            .fail(),
        }
    }
}

/// Get the main logs, metrics, or traces from a received BatchArrowRecords message.
fn get_main_payload_type(records: &BatchArrowRecords) -> error::Result<ArrowPayloadType> {
    ensure!(!records.arrow_payloads.is_empty(), error::EmptyBatchSnafu);

    // Per the specification, the main record type is the first payload
    let main_record_type = records.arrow_payloads[0].r#type;
    ArrowPayloadType::try_from(main_record_type).map_err(|_| {
        error::UnsupportedPayloadTypeSnafu {
            actual: main_record_type,
        }
        .build()
    })
}

#[cfg(test)]
mod tests {
    use crate::test_util::{create_record_batch, create_test_schema};
    use std::io::Cursor;
    use std::sync::Arc;

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
