// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Parquet exporter for OTAP data
//!
//! This writes the parquet files in a denormalized star-schema, where each OTAP payload
//! type is it's own parquet "table" that can be joined by id -> parent_id relationship.
//!
//! It also handles minor transformations of the data, such as creating a unique ID
//! (normally, IDs are only unique within some OTAP batch) and removing certain encodings
//! such as the delta encoded parent IDs.
//!
//! This exporter is currently experimental and is not yet ready for production use. There
//! are several outstanding issues that need to be addressed including:
//! - support for metrics and traces
//! - proper error handling and retry logic
//! - support for acknowledgements and nack messages
//! - handle periodically flushing batches after some time threshold
//! - dynamic configuration updates
//!   See the [GitHub issue](https://github.com/open-telemetry/otel-arrow/issues/399) for more details.

use std::io::ErrorKind;

use self::idgen::PartitionSequenceIdGenerator;
use self::partition::{Partition, partition};
use self::writer::WriteBatch;
use async_trait::async_trait;
use futures::{FutureExt, pin_mut};
use futures_timer::Delay;
use otap_df_engine::control::ControlMsg;
use otap_df_engine::error::Error;
use otap_df_engine::local::exporter::{EffectHandler, Exporter};
use otap_df_engine::message::{Message, MessageChannel};
use otel_arrow_rust::otap::OtapArrowRecords;

mod config;
mod idgen;
mod object_store;
mod partition;
mod writer;

#[allow(dead_code)]
const OTAP_PARQUET_EXPORTER_URN: &str = "urn:otel:otap:parquet:exporter";

/// Parquet exporter for OTAP Data
pub struct ParquetExporter {
    config: config::Config,
}

impl ParquetExporter {
    /// construct a new instance of the `ParquetExporter`
    #[must_use]
    pub fn new(config: config::Config) -> Self {
        Self { config }
    }
}

#[async_trait(?Send)]
impl<T> Exporter<T> for ParquetExporter
where
    T: TryInto<(i64, OtapArrowRecords), Error = Error<T>> + 'static,
{
    async fn start(
        self: Box<Self>,
        mut msg_chan: MessageChannel<T>,
        effect_handler: EffectHandler<T>,
    ) -> Result<(), Error<T>> {
        let object_store =
            object_store::from_uri(&self.config.base_uri).map_err(|e| Error::ExporterError {
                exporter: effect_handler.exporter_id(),
                error: format!("error initializing object store {e}"),
            })?;

        let mut writer =
            writer::WriterManager::new(object_store, self.config.writer_options.clone());

        let mut id_generator = PartitionSequenceIdGenerator::new();

        loop {
            match msg_chan.recv().await? {
                Message::Control(ControlMsg::TimerTick { .. }) => {
                    // TODO periodically we should tell the writer to flush any data
                    // that has been buffered in memory longer than some time threshold
                    // https://github.com/open-telemetry/otel-arrow/issues/497
                }
                Message::Control(ControlMsg::Config { .. }) => {
                    // TODO when we have the ability to inject dynamic config into the
                    // pipeline, we'll handle updating the exporter config here.
                    // https://github.com/open-telemetry/otel-arrow/issues/500
                }
                Message::Control(ControlMsg::Shutdown {
                    deadline,
                    reason: _,
                }) => {
                    let mut timeout = Delay::new(deadline).fuse();
                    let flush_all = writer.flush_all().fuse();
                    pin_mut!(flush_all);
                    return futures::select! {
                        _timeout = timeout => Err(Error::IoError {
                                node: effect_handler.exporter_id(),
                                error: std::io::Error::from(ErrorKind::TimedOut)
                            }),
                        _ = flush_all => Ok(()),
                    };
                }

                Message::PData(pdata) => {
                    let (batch_id, mut otap_batch): (i64, OtapArrowRecords) = pdata.try_into()?;

                    // generate unique IDs
                    let id_gen_result = id_generator.generate_unique_ids(&mut otap_batch);
                    if let Err(e) = id_gen_result {
                        // TODO - this is not the error handling we want long term.
                        // eventually we should have the concept of retryable & non-retryable errors and
                        // use Nack message + a Retry processor to handle this gracefully
                        // https://github.com/open-telemetry/otel-arrow/issues/504
                        return Err(Error::ExporterError {
                            exporter: effect_handler.exporter_id(),
                            error: format!("ID Generation failed: {e}"),
                        });
                    }

                    // compute any partitions
                    let partitions = match self.config.partitioning_strategies.as_ref() {
                        Some(strategies) => partition(&otap_batch, strategies),
                        None => vec![Partition {
                            otap_batch,
                            attributes: None,
                        }],
                    };

                    // write the data
                    let writes = partitions
                        .iter()
                        .map(|partition| {
                            WriteBatch::new(
                                batch_id,
                                &partition.otap_batch,
                                partition.attributes.as_deref(),
                            )
                        })
                        .collect::<Vec<_>>();
                    let write_result = writer.write(&writes).await;
                    if let Err(e) = write_result {
                        // TODO - this is not the error handling we want long term.
                        // eventually we should have the concept of retryable & non-retryable errors and
                        // use Nack message + a Retry processor to handle this gracefully
                        // https://github.com/open-telemetry/otel-arrow/issues/504
                        return Err(Error::ExporterError {
                            exporter: effect_handler.exporter_id(),
                            error: format!("Parquet write failed: {e}"),
                        });
                    };
                }

                _ => {
                    // ignore unexpected messages
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::pin::Pin;
    use std::rc::Rc;
    use std::time::Duration;

    use datagen::SimpleDataGenOptions;
    use futures::StreamExt;
    use otap_df_config::node::NodeUserConfig;
    use otap_df_engine::exporter::ExporterWrapper;
    use otap_df_engine::testing::exporter::{TestContext, TestRuntime};
    use otel_arrow_rust::Consumer;
    use otel_arrow_rust::otap::from_record_messages;
    use otel_arrow_rust::proto::opentelemetry::arrow::v1::ArrowPayloadType;
    use otel_arrow_rust::{
        otap::OtapArrowRecords, proto::opentelemetry::arrow::v1::BatchArrowRecords,
    };
    use parquet::arrow::async_reader::ParquetRecordBatchStreamBuilder;
    use tokio::fs::File;

    pub mod datagen;

    /// this is input to the test
    #[derive(Debug, Clone)]
    struct TestPDataInput {
        batch: BatchArrowRecords,
    }

    impl TryInto<(i64, OtapArrowRecords)> for TestPDataInput {
        type Error = Error<Self>;

        fn try_into(mut self) -> Result<(i64, OtapArrowRecords), Self::Error> {
            let batch_id = self.batch.batch_id;
            let first_payload_type =
                ArrowPayloadType::try_from(self.batch.arrow_payloads[0].r#type).unwrap();
            let mut consumer = Consumer::default();
            let record_messages = consumer.consume_bar(&mut self.batch).unwrap();

            match first_payload_type {
                ArrowPayloadType::Logs => Ok((
                    batch_id,
                    OtapArrowRecords::Logs(from_record_messages(record_messages)),
                )),
                ArrowPayloadType::Spans => Ok((
                    batch_id,
                    OtapArrowRecords::Traces(from_record_messages(record_messages)),
                )),
                ArrowPayloadType::UnivariateMetrics => Ok((
                    batch_id,
                    OtapArrowRecords::Metrics(from_record_messages(record_messages)),
                )),
                payload_type => {
                    panic!("unexpected payload type in TestPDataInput.try_into: {payload_type:?}")
                }
            }
        }
    }

    fn logs_scenario(
        num_rows: usize,
        shutdown_timeout: Duration,
    ) -> impl FnOnce(TestContext<TestPDataInput>) -> Pin<Box<dyn Future<Output = ()>>> {
        move |ctx| {
            Box::pin(async move {
                ctx.send_pdata(TestPDataInput {
                    batch: datagen::create_simple_logs_arrow_record_batches(SimpleDataGenOptions {
                        num_rows,
                        ..Default::default()
                    }),
                })
                .await
                .expect("Failed to send  logs message");

                ctx.send_shutdown(shutdown_timeout, "test completed")
                    .await
                    .unwrap();
            })
        }
    }

    async fn assert_parquet_file_has_rows(
        base_dir: &str,
        payload_type: ArrowPayloadType,
        num_rows: usize,
    ) {
        let table_name = payload_type.as_str_name().to_lowercase();
        let file_path = tokio::fs::read_dir(format!("{base_dir}/{table_name}"))
            .await
            .unwrap_or_else(|_| panic!("expect to have found table for {payload_type:?}"))
            .next_entry()
            .await
            .unwrap_or_else(|_| {
                panic!("expect at least one parquet file file for type {payload_type:?}")
            })
            .unwrap()
            .path();

        let file = File::open(file_path).await.unwrap();
        let reader_builder = ParquetRecordBatchStreamBuilder::new(file).await.unwrap();
        let mut reader = reader_builder.build().unwrap();
        let batch = reader.next().await.unwrap().unwrap();
        assert_eq!(batch.num_rows(), num_rows);
    }

    #[test]
    fn test_with_partitioning() {
        let test_runtime = TestRuntime::<TestPDataInput>::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let base_dir: String = temp_dir.path().to_str().unwrap().into();
        let exporter = ParquetExporter::new(config::Config {
            base_uri: base_dir.clone(),
            partitioning_strategies: Some(vec![config::PartitioningStrategy::SchemaMetadata(
                vec![idgen::PARTITION_METADATA_KEY.to_string()],
            )]),
            writer_options: None,
        });
        let node_config = Rc::new(NodeUserConfig::new_exporter_config(
            OTAP_PARQUET_EXPORTER_URN,
        ));
        let exporter = ExporterWrapper::<TestPDataInput>::local::<ParquetExporter>(
            exporter,
            node_config,
            test_runtime.config(),
        );

        let num_rows = 100;
        test_runtime
            .set_exporter(exporter)
            .run_test(logs_scenario(num_rows, Duration::from_secs(1)))
            .run_validation(move |_ctx, exporter_result| {
                Box::pin(async move {
                    assert!(exporter_result.is_ok());

                    // simply ensure there is a parquet file for each type we should have
                    // written and that it has the expected number of rows
                    for payload_type in [
                        ArrowPayloadType::Logs,
                        ArrowPayloadType::LogAttrs,
                        ArrowPayloadType::ResourceAttrs,
                        ArrowPayloadType::ScopeAttrs,
                    ] {
                        let table_name = payload_type.as_str_name().to_lowercase();

                        // ensure we have files partitioned and that the partition starts
                        // with the expected key
                        let partition_path = tokio::fs::read_dir(format!(
                            "{base_dir}/{table_name}"
                        ))
                        .await
                        .unwrap_or_else(|_| {
                            panic!("expect to have found table for {payload_type:?}")
                        })
                        .next_entry()
                        .await
                        .unwrap_or_else(|_| {
                            panic!(
                                "expect at least one partition directory for type {payload_type:?}"
                            )
                        })
                        .unwrap()
                        .path();

                        let last_segment = partition_path.iter().next_back().unwrap();
                        assert!(
                            last_segment
                                .to_string_lossy()
                                .starts_with(idgen::PARTITION_METADATA_KEY)
                        );

                        let file_path = tokio::fs::read_dir(partition_path)
                            .await
                            .unwrap()
                            .next_entry()
                            .await
                            .unwrap_or_else(|_| {
                                panic!("expect at least one parquet file for type {payload_type:?}")
                            })
                            .unwrap()
                            .path();

                        let file = File::open(file_path).await.unwrap();
                        let reader_builder =
                            ParquetRecordBatchStreamBuilder::new(file).await.unwrap();
                        let mut reader = reader_builder.build().unwrap();
                        let batch = reader.next().await.unwrap().unwrap();
                        assert_eq!(batch.num_rows(), num_rows);
                    }
                })
            });
    }

    #[test]
    fn test_no_partitioning() {
        let test_runtime = TestRuntime::<TestPDataInput>::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let base_dir: String = temp_dir.path().to_str().unwrap().into();
        let exporter = ParquetExporter::new(config::Config {
            base_uri: base_dir.clone(),
            partitioning_strategies: None,
            writer_options: None,
        });
        let node_config = Rc::new(NodeUserConfig::new_exporter_config(
            OTAP_PARQUET_EXPORTER_URN,
        ));
        let exporter = ExporterWrapper::<TestPDataInput>::local::<ParquetExporter>(
            exporter,
            node_config,
            test_runtime.config(),
        );
        let num_rows = 100;
        test_runtime
            .set_exporter(exporter)
            .run_test(logs_scenario(num_rows, Duration::from_secs(1)))
            .run_validation(move |_ctx, exporter_result| {
                Box::pin(async move {
                    assert!(exporter_result.is_ok());

                    // simply ensure there is a parquet file for each type we should have
                    // written and that it has the expected number of rows
                    for payload_type in [
                        ArrowPayloadType::Logs,
                        ArrowPayloadType::LogAttrs,
                        ArrowPayloadType::ResourceAttrs,
                        ArrowPayloadType::ScopeAttrs,
                    ] {
                        assert_parquet_file_has_rows(&base_dir, payload_type, num_rows).await;
                    }
                })
            });
    }

    #[test]
    fn test_shutdown_timeout() {
        let test_runtime = TestRuntime::<TestPDataInput>::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let base_dir: String = temp_dir.path().to_str().unwrap().into();
        let exporter = ParquetExporter::new(config::Config {
            base_uri: base_dir.clone(),
            partitioning_strategies: None,
            writer_options: None,
        });
        let node_config = Rc::new(NodeUserConfig::new_exporter_config(
            OTAP_PARQUET_EXPORTER_URN,
        ));
        let exporter = ExporterWrapper::<TestPDataInput>::local::<ParquetExporter>(
            exporter,
            node_config,
            test_runtime.config(),
        );
        let num_rows = 1000;

        test_runtime
            .set_exporter(exporter)
            .run_test(logs_scenario(num_rows, Duration::from_nanos(1)))
            .run_validation(move |_ctx, exporter_result| {
                Box::pin(async move {
                    // assert the exporter result is as expected
                    match exporter_result {
                        Ok(_) => panic!("expected exporter result to be error, received: Ok(())"),
                        Err(Error::IoError { node, error }) => {
                            assert_eq!(&node, "test_exporter");
                            assert_eq!(error.kind(), ErrorKind::TimedOut);
                        },
                        Err(e) => panic!("{}", format!("received unexpected error: {e:?}. Expected IoError caused by timeout"))
                    }
                })
            });
    }

    #[test]
    fn test_traces() {
        let test_runtime = TestRuntime::<TestPDataInput>::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let base_dir: String = temp_dir.path().to_str().unwrap().into();
        let exporter = ParquetExporter::new(config::Config {
            base_uri: base_dir.clone(),
            partitioning_strategies: None,
            writer_options: None,
        });
        let node_config = Rc::new(NodeUserConfig::new_exporter_config(
            OTAP_PARQUET_EXPORTER_URN,
        ));
        let exporter = ExporterWrapper::<TestPDataInput>::local::<ParquetExporter>(
            exporter,
            node_config,
            test_runtime.config(),
        );

        let num_rows = 100;
        test_runtime
            .set_exporter(exporter)
            .run_test(move |ctx| {
                Box::pin(async move {
                    ctx.send_pdata(TestPDataInput {
                        batch: datagen::create_simple_trace_arrow_record_batches(
                            SimpleDataGenOptions {
                                num_rows,
                                traces_options: Some(Default::default()),
                                ..Default::default()
                            },
                        ),
                    })
                    .await
                    .expect("Failed to send  logs message");

                    ctx.send_shutdown(Duration::from_millis(200), "test completed")
                        .await
                        .unwrap();
                })
            })
            .run_validation(move |_ctx, exporter_result| {
                Box::pin(async move {
                    assert!(exporter_result.is_ok());

                    // simply ensure there is a parquet file for each type we should have
                    // written and that it has the expected number of rows
                    for payload_type in [
                        ArrowPayloadType::Spans,
                        ArrowPayloadType::SpanAttrs,
                        ArrowPayloadType::ResourceAttrs,
                        ArrowPayloadType::ScopeAttrs,
                        ArrowPayloadType::SpanEvents,
                        ArrowPayloadType::SpanEventAttrs,
                        ArrowPayloadType::SpanLinks,
                        ArrowPayloadType::SpanLinkAttrs,
                    ] {
                        assert_parquet_file_has_rows(&base_dir, payload_type, num_rows).await;
                    }
                })
            });
    }

    #[test]
    fn test_metrics() {
        let test_runtime = TestRuntime::<TestPDataInput>::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let base_dir: String = temp_dir.path().to_str().unwrap().into();
        let exporter = ParquetExporter::new(config::Config {
            base_uri: base_dir.clone(),
            partitioning_strategies: None,
            writer_options: None,
        });
        let node_config = Rc::new(NodeUserConfig::new_exporter_config(
            OTAP_PARQUET_EXPORTER_URN,
        ));
        let exporter = ExporterWrapper::<TestPDataInput>::local::<ParquetExporter>(
            exporter,
            node_config,
            test_runtime.config(),
        );

        let num_rows = 100;
        test_runtime
            .set_exporter(exporter)
            .run_test(move |ctx| {
                Box::pin(async move {
                    ctx.send_pdata(TestPDataInput {
                        batch: datagen::create_simple_metrics_arrow_record_batches(
                            SimpleDataGenOptions {
                                num_rows,
                                metrics_options: Some(Default::default()),
                                ..Default::default()
                            },
                        ),
                    })
                    .await
                    .expect("Failed to send  logs message");

                    ctx.send_shutdown(Duration::from_millis(1000), "test completed")
                        .await
                        .unwrap();
                })
            })
            .run_validation(move |_ctx, exporter_result| {
                Box::pin(async move {
                    assert!(exporter_result.is_ok());

                    // simply ensure there is a parquet file for each type we should have
                    // written and that it has the expected number of rows
                    for payload_type in [
                        ArrowPayloadType::UnivariateMetrics,
                        ArrowPayloadType::ResourceAttrs,
                        ArrowPayloadType::ScopeAttrs,
                        ArrowPayloadType::SummaryDataPoints,
                        ArrowPayloadType::SummaryDpAttrs,
                        ArrowPayloadType::NumberDataPoints,
                        ArrowPayloadType::NumberDpAttrs,
                        ArrowPayloadType::NumberDpExemplars,
                        ArrowPayloadType::NumberDpExemplarAttrs,
                        ArrowPayloadType::HistogramDataPoints,
                        ArrowPayloadType::HistogramDpAttrs,
                        ArrowPayloadType::HistogramDpExemplars,
                        ArrowPayloadType::HistogramDpExemplarAttrs,
                        ArrowPayloadType::ExpHistogramDataPoints,
                        ArrowPayloadType::ExpHistogramDpAttrs,
                        ArrowPayloadType::ExpHistogramDpExemplars,
                        ArrowPayloadType::ExpHistogramDpExemplarAttrs,
                    ] {
                        assert_parquet_file_has_rows(&base_dir, payload_type, num_rows).await;
                    }
                })
            });
    }
}
