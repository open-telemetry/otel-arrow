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

use crate::OTAP_EXPORTER_FACTORIES;
use crate::parquet_exporter::schema::transform_to_known_schema;
use crate::pdata::OtapPdata;
use std::io::ErrorKind;
use std::sync::Arc;
use std::time::{Duration, Instant};

use self::idgen::PartitionSequenceIdGenerator;
use self::partition::{Partition, partition};
use self::writer::WriteBatch;
use crate::metrics::ExporterPDataMetrics;
use crate::parquet_exporter::metrics::ParquetExporterMetrics;
use async_trait::async_trait;
use futures::{FutureExt, pin_mut};
use futures_timer::Delay;
use linkme::distributed_slice;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::ExporterFactory;
use otap_df_engine::config::ExporterConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::error::{Error, ExporterErrorKind, format_error_sources};
use otap_df_engine::exporter::ExporterWrapper;
use otap_df_engine::local::exporter::{EffectHandler, Exporter};
use otap_df_engine::message::{Message, MessageChannel};
use otap_df_engine::node::NodeId;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_pdata::otap::OtapArrowRecords;
use otap_df_telemetry::metrics::{MetricSet, MetricSetHandler};

mod config;
mod error;
mod idgen;
mod metrics;
mod partition;
mod schema;
mod writer;

#[allow(dead_code)]
const PARQUET_EXPORTER_URN: &str = "urn:otel:parquet:exporter";

/// Parquet exporter for OTAP Data
pub struct ParquetExporter {
    config: config::Config,
    pdata_metrics: Option<MetricSet<ExporterPDataMetrics>>,
    io_metrics: Option<MetricSet<ParquetExporterMetrics>>,
}

/// Declares the Parquet exporter as a local exporter factory
///
/// Unsafe code is temporarily used here to allow the use of `distributed_slice` macro
/// This macro is part of the `linkme` crate which is considered safe and well maintained.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
pub static PARQUET_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: PARQUET_EXPORTER_URN,
    create: |pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             exporter_config: &ExporterConfig| {
        Ok(ExporterWrapper::local(
            ParquetExporter::from_config(pipeline, &node_config.config)?,
            node,
            node_config,
            exporter_config,
        ))
    },
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: otap_df_config::validation::validate_typed_config::<config::Config>,
};

impl ParquetExporter {
    /// construct a new instance of the `ParquetExporter`
    #[must_use]
    pub const fn new(config: config::Config) -> Self {
        // NOTE: This constructor does not register metrics because it lacks a PipelineContext.
        // Prefer using from_config in the factory path so metrics are properly wired.
        Self {
            config,
            pdata_metrics: None,
            io_metrics: None,
        }
    }

    /// construct a new instance from the configuration object
    pub fn from_config(
        pipeline_ctx: PipelineContext,
        config: &serde_json::Value,
    ) -> Result<Self, otap_df_config::error::Error> {
        let config: config::Config = serde_json::from_value(config.clone()).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;

        let pdata_metrics = pipeline_ctx.register_metrics::<ExporterPDataMetrics>();
        let io_metrics = pipeline_ctx.register_metrics::<ParquetExporterMetrics>();

        Ok(ParquetExporter {
            config,
            pdata_metrics: Some(pdata_metrics),
            io_metrics: Some(io_metrics),
        })
    }

    fn terminal_state(
        deadline: Instant,
        pdata_metrics: Option<MetricSet<ExporterPDataMetrics>>,
        io_metrics: Option<MetricSet<ParquetExporterMetrics>>,
    ) -> TerminalState {
        let mut snapshots = Vec::new();

        if let Some(metrics) = &pdata_metrics {
            if metrics.needs_flush() {
                snapshots.push(metrics.snapshot());
            }
        }

        if let Some(metrics) = &io_metrics {
            if metrics.needs_flush() {
                snapshots.push(metrics.snapshot());
            }
        }

        TerminalState::new(deadline, snapshots)
    }
}

#[async_trait(?Send)]
impl Exporter<OtapPdata> for ParquetExporter {
    async fn start(
        mut self: Box<Self>,
        mut msg_chan: MessageChannel<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        let exporter_id = effect_handler.exporter_id();
        let object_store =
            crate::object_store::from_storage_type(&self.config.storage).map_err(|e| {
                let source_detail = format_error_sources(&e);
                Error::ExporterError {
                    exporter: exporter_id.clone(),
                    kind: ExporterErrorKind::Configuration,
                    error: format!("error initializing object store {e}"),
                    source_detail,
                }
            })?;

        let writer_options = self.config.writer_options.unwrap_or_default();

        // if threshold set to flush old messages, get the timer interval. If timer interval isn't
        // configured, we use a default period of 5s
        let flush_age_check_interval = writer_options
            .flush_when_older_than
            .map(calculate_flush_timeout_check_period);

        // start timer-tick to periodically flush batches older than threshold
        if let Some(flush_age_check_interval) = flush_age_check_interval {
            // ignoring timer cancel for now..
            // TODO when we have the ability to inject dynamic config we may need to cancel and
            // recreate the interval timer
            // https://github.com/open-telemetry/otel-arrow/issues/500
            let _timer_cancel = effect_handler
                .start_periodic_timer(flush_age_check_interval)
                .await?;
        }

        // Start periodic telemetry collection (internal metrics)
        let telemetry_cancel_handle = effect_handler
            .start_periodic_telemetry(Duration::from_secs(1))
            .await?;

        let mut writer = writer::WriterManager::new(object_store, writer_options);
        let mut batch_id = 0;
        let mut id_generator = PartitionSequenceIdGenerator::new();

        loop {
            match msg_chan.recv().await? {
                Message::Control(NodeControlMsg::TimerTick { .. }) => {
                    match writer.flush_aged_beyond_threshold().await {
                        Ok(stats) => {
                            if let Some(io) = self.io_metrics.as_mut() {
                                if stats.flush_scheduled_max_rows > 0 {
                                    io.flush_scheduled_max_rows
                                        .add(stats.flush_scheduled_max_rows);
                                }
                                if stats.flush_scheduled_max_age > 0 {
                                    io.flush_scheduled_max_age
                                        .add(stats.flush_scheduled_max_age);
                                }
                                if stats.files_closed > 0 {
                                    io.files_closed.add(stats.files_closed);
                                }
                            }
                        }
                        Err(e) => {
                            // TODO - this is not the error handling we want long term.  eventually we
                            // should have the concept of retryable & non-retryable errors and use Nack
                            //  message + a Retry processor to handle this gracefully
                            // https://github.com/open-telemetry/otel-arrow/issues/504
                            let source_detail = format_error_sources(&e);
                            return Err(Error::ExporterError {
                                exporter: effect_handler.exporter_id(),
                                kind: ExporterErrorKind::Transport,
                                error: format!("Parquet write failed: {e}"),
                                source_detail,
                            });
                        }
                    }
                }
                Message::Control(NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                }) => {
                    if let Some(metrics) = self.pdata_metrics.as_mut() {
                        _ = metrics_reporter.report(metrics);
                    }
                    if let Some(metrics) = self.io_metrics.as_mut() {
                        _ = metrics_reporter.report(metrics);
                    }
                }
                Message::Control(NodeControlMsg::Config { .. }) => {
                    // TODO when we have the ability to inject dynamic config into the
                    // pipeline, we'll handle updating the exporter config here.
                    // https://github.com/open-telemetry/otel-arrow/issues/500
                }
                Message::Control(NodeControlMsg::Shutdown {
                    deadline,
                    reason: _,
                }) => {
                    let mut timeout = Delay::new(deadline.duration_since(Instant::now())).fuse();
                    let flush_all = writer.flush_all().fuse();
                    pin_mut!(flush_all);
                    // Stop telemetry loop concurrently with flushing; do not block shutdown on cancel
                    let cancel_fut = async {
                        let _ = telemetry_cancel_handle.cancel().await;
                        futures::future::pending::<()>().await
                    }
                    .fuse();
                    pin_mut!(cancel_fut);

                    return futures::select_biased! {
                        _ = cancel_fut => unreachable!(),
                        _timeout = timeout => Err(Error::IoError {
                                node: exporter_id.clone(),
                                error: std::io::Error::from(ErrorKind::TimedOut)
                            }),
                        _ = flush_all => Ok(Self::terminal_state(deadline, self.pdata_metrics, self.io_metrics)),
                    };
                }

                Message::PData(pdata) => {
                    // Capture signal type before moving pdata into try_from
                    let signal_type = pdata.signal_type();

                    // Note: context is not used
                    let (_context, payload) = pdata.into_parts();

                    // Mark as consumed for this signal
                    if let Some(metrics) = self.pdata_metrics.as_mut() {
                        metrics.inc_consumed(signal_type);
                    }

                    let mut otap_batch: OtapArrowRecords =
                        payload.try_into().inspect_err(|_| {
                            if let Some(metrics) = self.pdata_metrics.as_mut() {
                                metrics.inc_failed(signal_type);
                            }
                        })?;

                    // generate unique IDs
                    let id_gen_result = id_generator.generate_unique_ids(&mut otap_batch);
                    if let Err(e) = id_gen_result {
                        // mark failure before returning
                        if let Some(metrics) = self.pdata_metrics.as_mut() {
                            metrics.inc_failed(signal_type);
                        }
                        // TODO - this is not the error handling we want long term.
                        // eventually we should have the concept of retryable & non-retryable errors and
                        // use Nack message + a Retry processor to handle this gracefully
                        // https://github.com/open-telemetry/otel-arrow/issues/504
                        let source_detail = format_error_sources(&e);
                        return Err(Error::ExporterError {
                            exporter: exporter_id.clone(),
                            kind: ExporterErrorKind::Other,
                            error: format!("ID Generation failed: {e}"),
                            source_detail,
                        });
                    }

                    // ensure the batches has the schema the parquet writer expects
                    transform_to_known_schema(&mut otap_batch).map_err(|e| {
                        // mark failure before returning
                        if let Some(metrics) = self.pdata_metrics.as_mut() {
                            metrics.inc_failed(signal_type);
                        }
                        // TODO - Ack/Nack instead of returning error
                        let source_detail = format_error_sources(&e);
                        Error::ExporterError {
                            exporter: exporter_id.clone(),
                            kind: ExporterErrorKind::Other,
                            error: format!("Schema transformation failed: {e}"),
                            source_detail,
                        }
                    })?;

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
                    batch_id += 1;
                    match writer.write(&writes).await {
                        Ok(stats) => {
                            // successful write
                            if let Some(metrics) = self.pdata_metrics.as_mut() {
                                metrics.inc_exported(signal_type);
                            }
                            if let Some(io) = self.io_metrics.as_mut() {
                                if stats.files_created > 0 {
                                    io.files_created.add(stats.files_created);
                                }
                                if stats.files_closed > 0 {
                                    io.files_closed.add(stats.files_closed);
                                }
                                if stats.rows_written > 0 {
                                    io.rows_written.add(stats.rows_written);
                                }
                                if stats.flush_scheduled_max_rows > 0 {
                                    io.flush_scheduled_max_rows
                                        .add(stats.flush_scheduled_max_rows);
                                }
                                if stats.flush_scheduled_max_age > 0 {
                                    io.flush_scheduled_max_age
                                        .add(stats.flush_scheduled_max_age);
                                }
                            }
                        }
                        Err(e) => {
                            // mark failure before returning
                            if let Some(metrics) = self.pdata_metrics.as_mut() {
                                metrics.inc_failed(signal_type);
                            }
                            // TODO - this is not the error handling we want long term.
                            // eventually we should have the concept of retryable & non-retryable errors and
                            // use Nack message + a Retry processor to handle this gracefully
                            // https://github.com/open-telemetry/otel-arrow/issues/504
                            let source_detail = format_error_sources(&e);
                            return Err(Error::ExporterError {
                                exporter: effect_handler.exporter_id(),
                                kind: ExporterErrorKind::Transport,
                                error: format!("Parquet write failed: {e}"),
                                source_detail,
                            });
                        }
                    }
                }

                _ => {
                    // ignore unexpected messages
                }
            }
        }
    }
}

/// This calculates the period at which we instruct the [`WriterManager`] to flush any writers
/// older than the threshold.
fn calculate_flush_timeout_check_period(configured_threshold: Duration) -> Duration {
    // try to choose a period that is relatively close the the configured threshold.
    // this avoids the check happening long after the file writer is beyond the threshold.
    let period = configured_threshold / 60;

    // we make the minimum period at which we'll check that a flush should happen 1 second.
    // this avoids a too short check interval causing a lot of overhead.
    period.max(Duration::from_secs(1))
}

#[cfg(test)]
mod test {
    use crate::parquet_exporter::config::WriterOptions;
    use std::ops::Add;

    use super::*;

    use std::pin::Pin;
    use std::sync::Arc;
    use std::time::Duration;

    use arrow::array::{DictionaryArray, RecordBatch, StringArray, UInt16Array};
    use arrow::compute::concat_batches;
    use arrow::datatypes::{DataType, Field, Schema, UInt16Type};
    use fixtures::SimpleDataGenOptions;
    use futures::StreamExt;
    use otap_df_config::node::NodeUserConfig;
    use otap_df_engine::control::{
        Controllable, PipelineControlMsg, PipelineCtrlMsgReceiver, PipelineCtrlMsgSender,
        pipeline_ctrl_msg_channel,
    };
    use otap_df_engine::exporter::ExporterWrapper;
    use otap_df_engine::local::message::{LocalReceiver, LocalSender};
    use otap_df_engine::message::{Receiver, Sender};
    use otap_df_engine::node::NodeWithPDataReceiver;
    use otap_df_engine::testing::{create_not_send_channel, setup_test_runtime};
    use otap_df_engine::testing::{
        exporter::{TestContext, TestRuntime},
        test_node,
    };
    use otap_df_pdata::Consumer;
    use otap_df_pdata::otap::from_record_messages;
    use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
    use otap_df_pdata::proto::opentelemetry::common::v1::{AnyValue, KeyValue, any_value::Value};
    use otap_df_pdata::schema::consts;
    use parquet::arrow::async_reader::ParquetRecordBatchStreamBuilder;
    use tokio::fs::File;
    use tokio::time::sleep;

    use crate::fixtures;

    fn logs_scenario(
        num_rows: usize,
        shutdown_timeout: Instant,
    ) -> impl FnOnce(TestContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
        move |ctx| {
            Box::pin(async move {
                let mut consumer = Consumer::default();
                let otap_batch = consumer
                    .consume_bar(&mut fixtures::create_simple_logs_arrow_record_batches(
                        SimpleDataGenOptions {
                            num_rows,
                            ..Default::default()
                        },
                    ))
                    .unwrap();

                ctx.send_pdata(OtapPdata::new_default(
                    OtapArrowRecords::Logs(from_record_messages(otap_batch)).into(),
                ))
                .await
                .expect("Failed to send  logs message");

                ctx.send_shutdown(shutdown_timeout, "test completed")
                    .await
                    .unwrap();
            })
        }
    }

    #[test]
    #[cfg_attr(
        target_os = "windows",
        ignore = "Skipping on Windows due to timing flakiness"
    )]
    fn test_adaptive_schema_dict_upgrade_write() {
        let test_runtime = TestRuntime::<OtapPdata>::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let base_dir: String = temp_dir.path().to_str().unwrap().into();
        let exporter = ParquetExporter::new(config::Config {
            storage: crate::object_store::StorageType::File {
                base_uri: base_dir.clone(),
            },
            partitioning_strategies: None,
            writer_options: None,
        });
        let node_config = Arc::new(NodeUserConfig::new_exporter_config(PARQUET_EXPORTER_URN));
        let exporter = ExporterWrapper::<OtapPdata>::local::<ParquetExporter>(
            exporter,
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        test_runtime
            .set_exporter(exporter)
            .run_test(move |ctx| {
                Box::pin(async move {
                    // this should generate an attributes key & string_val column with type Dict<u16, String>
                    let mut attrs1 = vec![];
                    for i in 0..257 {
                        attrs1.push(KeyValue {
                            key: format!("attr{i}"),
                            value: Some(AnyValue {
                                value: Some(Value::StringValue(format!("val{i}"))),
                            }),
                        })
                    }

                    let pdata1 = fixtures::create_single_logs_pdata_with_attrs(attrs1);
                    ctx.send_pdata(pdata1).await.unwrap();

                    // this should generate an attributes column with type Dict<u8, String>
                    let pdata2 = fixtures::create_single_logs_pdata_with_attrs(vec![KeyValue {
                        key: "attr1".to_string(),
                        value: Some(AnyValue {
                            value: Some(Value::StringValue("val1".to_string())),
                        }),
                    }]);

                    ctx.send_pdata(pdata2).await.unwrap();

                    // this should create a record batch with the string_val column as native array
                    // manually switching the type b/c otherwise would need to create > u16::MAX
                    // attributes, and it could lead to unpredictable write times which could lead
                    // to some test flakiness.
                    let mut attrs3 = vec![];
                    for i in 0..20 {
                        attrs3.push(KeyValue {
                            key: format!("attr{i}"),
                            value: Some(AnyValue {
                                value: Some(Value::StringValue(format!("val{i}"))),
                            }),
                        })
                    }
                    let pdata3 = fixtures::create_single_logs_pdata_with_attrs(attrs3).payload();
                    let mut otap_batch = OtapArrowRecords::try_from(pdata3).unwrap();
                    let mut attrs_batch =
                        otap_batch.get(ArrowPayloadType::LogAttrs).unwrap().clone();
                    let old_column = attrs_batch.remove_column(
                        attrs_batch
                            .schema()
                            .index_of(consts::ATTRIBUTE_STR)
                            .unwrap(),
                    );
                    let tmp = old_column
                        .as_any()
                        .downcast_ref::<DictionaryArray<UInt16Type>>()
                        .unwrap();
                    let new_column =
                        StringArray::from_iter(tmp.downcast_dict::<StringArray>().unwrap());
                    let mut columns = attrs_batch.columns().to_vec();
                    columns.push(Arc::new(new_column));
                    let mut fields = attrs_batch.schema().fields().to_vec();
                    fields.push(Arc::new(Field::new(
                        consts::ATTRIBUTE_STR,
                        DataType::Utf8,
                        true,
                    )));
                    otap_batch.set(
                        ArrowPayloadType::LogAttrs,
                        RecordBatch::try_new(Arc::new(Schema::new(fields)), columns).unwrap(),
                    );
                    ctx.send_pdata(OtapPdata::new_default(otap_batch.into()))
                        .await
                        .unwrap();

                    let deadline = Instant::now().add(Duration::from_millis(200));
                    ctx.send_shutdown(deadline, "test completed").await.unwrap();
                })
            })
            .run_validation(move |_ctx, exporter_result| {
                Box::pin(async move {
                    exporter_result.unwrap();
                    assert_parquet_file_has_rows(&base_dir, ArrowPayloadType::Logs, 3).await;
                    assert_parquet_file_has_rows(&base_dir, ArrowPayloadType::LogAttrs, 278).await;
                })
            });
    }

    #[test]
    fn test_adaptive_schema_optional_columns() {
        let test_runtime = TestRuntime::<OtapPdata>::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let base_dir: String = temp_dir.path().to_str().unwrap().into();
        let exporter = ParquetExporter::new(config::Config {
            storage: crate::object_store::StorageType::File {
                base_uri: base_dir.clone(),
            },
            partitioning_strategies: None,
            writer_options: None,
        });
        let node_config = Arc::new(NodeUserConfig::new_exporter_config(PARQUET_EXPORTER_URN));
        let exporter = ExporterWrapper::<OtapPdata>::local::<ParquetExporter>(
            exporter,
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        test_runtime
            .set_exporter(exporter)
            .run_test(move |ctx| {
                Box::pin(async move {
                    let batch1: OtapArrowRecords =
                        fixtures::create_single_logs_pdata_with_attrs(vec![KeyValue {
                            key: "strkey".to_string(),
                            value: Some(AnyValue::new_string("terry")),
                        }])
                        .payload()
                        .try_into()
                        .unwrap();

                    let batch2: OtapArrowRecords =
                        fixtures::create_single_logs_pdata_with_attrs(vec![KeyValue {
                            key: "intkey".to_string(),
                            value: Some(AnyValue::new_int(418)),
                        }])
                        .payload()
                        .try_into()
                        .unwrap();

                    // double check that these contain schemas that are not the same ...
                    let batch1_attrs = batch1.get(ArrowPayloadType::LogAttrs).unwrap();
                    let batch2_attrs = batch2.get(ArrowPayloadType::LogAttrs).unwrap();
                    assert_ne!(batch1_attrs.schema(), batch2_attrs.schema());

                    ctx.send_pdata(OtapPdata::new_default(batch1.into()))
                        .await
                        .unwrap();
                    ctx.send_pdata(OtapPdata::new_default(batch2.into()))
                        .await
                        .unwrap();

                    ctx.send_shutdown(
                        Instant::now().add(Duration::from_millis(200)),
                        "test completed",
                    )
                    .await
                    .unwrap();
                })
            })
            .run_validation(move |_ctx, exporter_result| {
                Box::pin(async move {
                    // check no error
                    exporter_result.unwrap();
                    assert_parquet_file_has_rows(&base_dir, ArrowPayloadType::Logs, 2).await;
                    assert_parquet_file_has_rows(&base_dir, ArrowPayloadType::LogAttrs, 2).await;
                })
            });
    }

    async fn wait_table_exists(base_dir: &str, payload_type: ArrowPayloadType) {
        let table_name = payload_type.as_str_name().to_lowercase();
        loop {
            _ = sleep(Duration::from_millis(100)).await;

            // ensure the table exists
            let mut dir = match tokio::fs::read_dir(format!("{base_dir}/{table_name}")).await {
                Ok(dir) => dir,
                Err(_) => continue,
            };

            // ensure a parquet file exists
            let table_dir_entry = match dir.next_entry().await.unwrap() {
                Some(table_dir) => table_dir,
                None => continue,
            };

            // open the file and ensure a batch is written in it
            let file = match File::open(table_dir_entry.path()).await {
                Ok(file) => file,
                Err(_) => continue,
            };
            let reader_builder = match ParquetRecordBatchStreamBuilder::new(file).await {
                Ok(rb) => rb,
                Err(_) => continue,
            };
            let mut reader = match reader_builder.build() {
                Ok(r) => r,
                Err(_) => continue,
            };
            match reader.next().await {
                Some(_) => break,
                None => continue,
            }
        }
    }

    async fn try_wait_table_exists(
        base_dir: &str,
        payload_type: ArrowPayloadType,
        max_wait: Duration,
    ) -> Result<(), Error> {
        tokio::select! {
            _ = Delay::new(max_wait) => Err(Error::InternalError {
                message: "timed out waiting for table to exist".into()
            }),
            _ = wait_table_exists(base_dir, payload_type) => Ok(())
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
        let test_runtime = TestRuntime::<OtapPdata>::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let base_dir: String = temp_dir.path().to_str().unwrap().into();
        let exporter = ParquetExporter::new(config::Config {
            storage: crate::object_store::StorageType::File {
                base_uri: base_dir.clone(),
            },
            partitioning_strategies: Some(vec![config::PartitioningStrategy::SchemaMetadata(
                vec![idgen::PARTITION_METADATA_KEY.to_string()],
            )]),
            writer_options: None,
        });
        let node_config = Arc::new(NodeUserConfig::new_exporter_config(PARQUET_EXPORTER_URN));
        let exporter = ExporterWrapper::<OtapPdata>::local::<ParquetExporter>(
            exporter,
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        let num_rows = 100;
        test_runtime
            .set_exporter(exporter)
            .run_test(logs_scenario(
                num_rows,
                Instant::now().add(Duration::from_secs(1)),
            ))
            .run_validation(move |_ctx, exporter_result| {
                Box::pin(async move {
                    exporter_result.unwrap();

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
        let test_runtime = TestRuntime::<OtapPdata>::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let base_dir: String = temp_dir.path().to_str().unwrap().into();
        let exporter = ParquetExporter::new(config::Config {
            storage: crate::object_store::StorageType::File {
                base_uri: base_dir.clone(),
            },
            partitioning_strategies: None,
            writer_options: None,
        });
        let node_config = Arc::new(NodeUserConfig::new_exporter_config(PARQUET_EXPORTER_URN));
        let exporter = ExporterWrapper::<OtapPdata>::local::<ParquetExporter>(
            exporter,
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );
        let num_rows = 100;
        test_runtime
            .set_exporter(exporter)
            .run_test(logs_scenario(
                num_rows,
                Instant::now().add(Duration::from_secs(1)),
            ))
            .run_validation(move |_ctx, exporter_result| {
                Box::pin(async move {
                    exporter_result.unwrap();

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

    // Skipping on Windows and macOS due to flakiness: https://github.com/open-telemetry/otel-arrow/issues/1614
    #[test]
    #[cfg_attr(
        any(target_os = "windows", target_os = "macos"),
        ignore = "Skipping on Windows and macOS due to flakiness"
    )]
    fn test_shutdown_timeout() {
        let test_runtime = TestRuntime::<OtapPdata>::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let base_dir: String = temp_dir.path().to_str().unwrap().into();
        let exporter = ParquetExporter::new(config::Config {
            storage: crate::object_store::StorageType::File {
                base_uri: format!("testdelayed://{base_dir}?delay=500ms"),
            },
            partitioning_strategies: None,
            writer_options: Some(WriterOptions {
                target_rows_per_file: Some(50),
                ..Default::default()
            }),
        });
        let node_config = Arc::new(NodeUserConfig::new_exporter_config(PARQUET_EXPORTER_URN));
        let mut exporter = ExporterWrapper::<OtapPdata>::local::<ParquetExporter>(
            exporter,
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        let exporter_config = ExporterConfig::new("test_parquet_exporter");
        let (rt, _) = setup_test_runtime();
        let control_sender = exporter.control_sender();
        let (pdata_tx, pdata_rx) = create_not_send_channel::<OtapPdata>(1);
        let pdata_tx = Sender::Local(LocalSender::mpsc(pdata_tx));
        let pdata_rx = Receiver::Local(LocalReceiver::mpsc(pdata_rx));

        let (pipeline_ctrl_msg_tx, _pipeline_ctrl_msg_rx) = pipeline_ctrl_msg_channel(10);
        // Keep the receiver alive so EffectHandler can send telemetry/timer requests without error.

        exporter
            .set_pdata_receiver(test_node(exporter_config.name.clone()), pdata_rx)
            .expect("Failed to set PData Receiver");

        async fn start_exporter(
            exporter: ExporterWrapper<OtapPdata>,
            pipeline_ctrl_msg_tx: PipelineCtrlMsgSender<OtapPdata>,
        ) -> Result<(), Error> {
            let (_metrics_rx, metrics_reporter) =
                otap_df_telemetry::reporter::MetricsReporter::create_new_and_receiver(1);
            exporter
                .start(pipeline_ctrl_msg_tx, metrics_reporter)
                .await
                .map(|_| ())
        }

        async fn send_messages(
            base_dir: &str,
            pdata_tx: Sender<OtapPdata>,
            ctrl_sender: Sender<NodeControlMsg<OtapPdata>>,
        ) -> () {
            // have the parquet writer queue a batch to be written. Since it's a bit difficult to
            // know for certain when the batches will actually be queued because we have no direct
            // way to investigate it, we can inspect it indirectly. Below, we'll write just enough
            // data to cause the log attributes table to flush, but not the logs, resource attrs or
            // scope attrs. Since we know the log attrs table has been written, we can guess the
            // writer has buffered files ..

            let logs_data = Consumer::default()
                .consume_bar(&mut fixtures::create_simple_logs_arrow_record_batches(
                    SimpleDataGenOptions {
                        // a pretty big batch
                        num_rows: 48,
                        ..Default::default()
                    },
                ))
                .unwrap();

            let otap_batch = OtapArrowRecords::Logs(from_record_messages(logs_data)).into();
            pdata_tx
                .send(OtapPdata::new_default(otap_batch))
                .await
                .unwrap();

            let logs_data = Consumer::default()
                .consume_bar(&mut fixtures::create_simple_logs_arrow_record_batches(
                    SimpleDataGenOptions {
                        num_rows: 1,
                        ..Default::default()
                    },
                ))
                .unwrap();
            let mut otap_batch2 = OtapArrowRecords::Logs(from_record_messages(logs_data));
            let log_attrs = otap_batch2.get(ArrowPayloadType::LogAttrs).unwrap();
            // adding extra attributes should just put us over the limit where this table will be
            // flushed on write
            otap_batch2.set(
                ArrowPayloadType::LogAttrs,
                concat_batches(log_attrs.schema_ref(), vec![&log_attrs.clone(), log_attrs])
                    .unwrap(),
            );

            pdata_tx
                .send(OtapPdata::new_default(otap_batch2.into()))
                .await
                .unwrap();

            // wait for the log_attrs table to exist
            try_wait_table_exists(base_dir, ArrowPayloadType::LogAttrs, Duration::from_secs(5))
                .await
                .unwrap();

            // double check that the other tables have not been written (this is a sanity check)
            for payload_type in &[
                ArrowPayloadType::Logs,
                ArrowPayloadType::ResourceAttrs,
                ArrowPayloadType::ScopeAttrs,
            ] {
                let table_name = payload_type.as_str_name().to_lowercase();
                assert!(
                    tokio::fs::read_dir(format!("{base_dir}/{table_name}"))
                        .await
                        .is_err()
                );
            }

            // shutdown faster than it could possibly flush
            _ = ctrl_sender
                .send(NodeControlMsg::Shutdown {
                    deadline: Instant::now().add(Duration::from_secs(1)),
                    reason: "shutting down".into(),
                })
                .await;
        }

        // let the exporter's message handling loop and the message sending run concurrently
        let (exporter_result, _) = rt.block_on(async move {
            tokio::join!(
                start_exporter(exporter, pipeline_ctrl_msg_tx),
                send_messages(base_dir.as_str(), pdata_tx, control_sender)
            )
        });

        // expect that we timed out before we could flush messages when the shutdown message was received
        match exporter_result {
            Ok(_) => panic!("expected exporter result to be error, received: Ok(())"),
            Err(Error::IoError { node, error }) => {
                assert_eq!(node.name, "test_exporter");
                assert_eq!(error.kind(), ErrorKind::TimedOut);
            }
            Err(e) => panic!(
                "{}",
                format!("received unexpected error: {e:?}. Expected IoError caused by timeout")
            ),
        }
    }

    #[test]
    fn test_can_flush_on_interval() {
        let temp_dir = tempfile::tempdir().unwrap();
        let base_dir: String = temp_dir.path().to_str().unwrap().into();
        let exporter = ParquetExporter::new(config::Config {
            storage: crate::object_store::StorageType::File {
                base_uri: base_dir.clone(),
            },
            partitioning_strategies: None,
            writer_options: Some(WriterOptions {
                target_rows_per_file: None,
                flush_when_older_than: Some(Duration::from_millis(200)),
            }),
        });

        let test_runtime = TestRuntime::<OtapPdata>::new();
        let node_config = Arc::new(NodeUserConfig::new_exporter_config(PARQUET_EXPORTER_URN));
        let node_id = test_node(test_runtime.config().name.clone());
        let mut exporter = ExporterWrapper::<OtapPdata>::local::<ParquetExporter>(
            exporter,
            node_id.clone(),
            node_config,
            test_runtime.config(),
        );

        let (rt, _) = setup_test_runtime();
        let control_sender = exporter.control_sender();
        let (pdata_tx, pdata_rx) =
            create_not_send_channel::<OtapPdata>(test_runtime.config().control_channel.capacity);
        let pdata_tx = Sender::Local(LocalSender::mpsc(pdata_tx));
        let pdata_rx = Receiver::Local(LocalReceiver::mpsc(pdata_rx));

        let (pipeline_ctrl_msg_tx, pipeline_ctrl_msg_rx) = pipeline_ctrl_msg_channel(10);
        exporter
            .set_pdata_receiver(node_id.clone(), pdata_rx)
            .expect("Failed to set PData Receiver");

        async fn start_exporter(
            exporter: ExporterWrapper<OtapPdata>,
            pipeline_ctrl_msg_tx: PipelineCtrlMsgSender<OtapPdata>,
        ) -> Result<(), Error> {
            let (_metrics_rx, metrics_reporter) =
                otap_df_telemetry::reporter::MetricsReporter::create_new_and_receiver(1);
            exporter
                .start(pipeline_ctrl_msg_tx, metrics_reporter)
                .await
                .map(|_| ())
        }

        async fn run_test(
            base_dir: &str,
            pdata_tx: Sender<OtapPdata>,
            ctrl_tx: Sender<NodeControlMsg<OtapPdata>>,
            mut ctrl_rx: PipelineCtrlMsgReceiver<OtapPdata>,
        ) -> Result<(), Error> {
            // try to receive the first timer start message
            let msg = tokio::select! {
                _ = Delay::new(Duration::from_secs(10)) => return Err(Error::InternalError {
                    message: "Timed out waiting for timer start signal".into(),
                }),
                msg = ctrl_rx.recv() => msg?
            };
            if let PipelineControlMsg::StartTimer {
                node_id: _,
                duration,
            } = msg
            {
                assert_eq!(duration, Duration::from_secs(1));
            } else {
                return Err(Error::InternalError {
                    message: "wrong pipeline control message received. Expected StartTimer".into(),
                });
            }

            // have the parquet writer queue a batch to be written
            let num_rows = 5;

            let mut consumer = Consumer::default();
            let otap_batch = consumer
                .consume_bar(&mut fixtures::create_simple_logs_arrow_record_batches(
                    SimpleDataGenOptions {
                        num_rows,
                        ..Default::default()
                    },
                ))
                .unwrap();
            let otap_batch = OtapArrowRecords::Logs(from_record_messages(otap_batch));
            pdata_tx
                .send(OtapPdata::new_default(otap_batch.into()))
                .await
                .unwrap();

            // wait some time but not as long as the old age threshold, then send a timer tick
            _ = sleep(Duration::from_millis(50)).await;
            ctrl_tx.send(NodeControlMsg::TimerTick {}).await.unwrap();

            // wait a little bit, then ensure we haven't flushed anything
            _ = sleep(Duration::from_millis(200)).await;
            let any_table = tokio::fs::read_dir(base_dir.to_string())
                .await
                .unwrap()
                .next_entry()
                .await
                .unwrap();
            assert!(any_table.is_none());

            // by now we've waited longer than the old age threshold, so if we send the timer tick
            // now, the files should flush
            ctrl_tx.send(NodeControlMsg::TimerTick {}).await.unwrap();

            for payload_type in &[
                ArrowPayloadType::Logs,
                ArrowPayloadType::LogAttrs,
                ArrowPayloadType::ScopeAttrs,
                ArrowPayloadType::ResourceAttrs,
            ] {
                try_wait_table_exists(base_dir, *payload_type, Duration::from_secs(5))
                    .await
                    .unwrap();
                assert_parquet_file_has_rows(base_dir, *payload_type, num_rows).await
            }

            ctrl_tx
                .send(NodeControlMsg::Shutdown {
                    deadline: Instant::now().add(Duration::from_millis(10)),
                    reason: "shutting down".into(),
                })
                .await
                .unwrap();

            Ok(())
        }

        let (exporter_result, test_run_result) = rt.block_on(async move {
            tokio::join!(
                start_exporter(exporter, pipeline_ctrl_msg_tx),
                run_test(
                    base_dir.as_str(),
                    pdata_tx,
                    control_sender,
                    pipeline_ctrl_msg_rx
                )
            )
        });

        // these shouldn't be Err, so unwrap to check
        test_run_result.unwrap();
        exporter_result.unwrap();
    }

    #[test]
    fn test_starts_telemetry_timer() {
        use otap_df_engine::control::pipeline_ctrl_msg_channel;
        use otap_df_engine::testing::test_node;

        let test_runtime = TestRuntime::<OtapPdata>::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let base_dir: String = temp_dir.path().to_str().unwrap().into();
        let exporter = ParquetExporter::new(config::Config {
            storage: crate::object_store::StorageType::File {
                base_uri: base_dir.clone(),
            },
            partitioning_strategies: None,
            writer_options: None,
        });
        let node_config = Arc::new(NodeUserConfig::new_exporter_config(PARQUET_EXPORTER_URN));
        let mut exporter = ExporterWrapper::<OtapPdata>::local::<ParquetExporter>(
            exporter,
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        let (rt, _) = setup_test_runtime();
        let control_sender = exporter.control_sender();
        let (pdata_tx, pdata_rx) = create_not_send_channel::<OtapPdata>(1);
        let _pdata_tx = Sender::Local(LocalSender::mpsc(pdata_tx));
        let pdata_rx = Receiver::Local(LocalReceiver::mpsc(pdata_rx));

        let (pipeline_ctrl_msg_tx, mut pipeline_ctrl_msg_rx) = pipeline_ctrl_msg_channel(10);

        exporter
            .set_pdata_receiver(test_node("exp"), pdata_rx)
            .expect("Failed to set PData Receiver");

        async fn start_exporter(
            exporter: ExporterWrapper<OtapPdata>,
            pipeline_ctrl_msg_tx: PipelineCtrlMsgSender<OtapPdata>,
        ) -> Result<(), Error> {
            let (_metrics_rx, metrics_reporter) =
                otap_df_telemetry::reporter::MetricsReporter::create_new_and_receiver(1);
            exporter
                .start(pipeline_ctrl_msg_tx, metrics_reporter)
                .await
                .map(|_| ())
        }

        let (_exporter_result, _ignored) = rt.block_on(async move {
            tokio::join!(start_exporter(exporter, pipeline_ctrl_msg_tx), async move {
                // Expect StartTelemetryTimer quickly after startup
                let msg = tokio::time::timeout(Duration::from_millis(1500), async {
                    pipeline_ctrl_msg_rx.recv().await
                })
                .await
                .expect("timed out waiting for StartTelemetryTimer")
                .expect("pipeline ctrl channel closed");

                match msg {
                    PipelineControlMsg::StartTelemetryTimer { duration, .. } => {
                        assert_eq!(duration, Duration::from_secs(1));
                    }
                    other => panic!("Expected StartTelemetryTimer, got {other:?}"),
                }

                // Shutdown exporter to end the test
                let _ = control_sender
                    .send(NodeControlMsg::Shutdown {
                        deadline: Instant::now(),
                        reason: "done".into(),
                    })
                    .await;
            })
        });
    }

    #[test]
    fn test_traces() {
        let test_runtime = TestRuntime::<OtapPdata>::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let base_dir: String = temp_dir.path().to_str().unwrap().into();
        let exporter = ParquetExporter::new(config::Config {
            storage: crate::object_store::StorageType::File {
                base_uri: base_dir.clone(),
            },
            partitioning_strategies: None,
            writer_options: None,
        });
        let node_config = Arc::new(NodeUserConfig::new_exporter_config(PARQUET_EXPORTER_URN));
        let exporter = ExporterWrapper::<OtapPdata>::local::<ParquetExporter>(
            exporter,
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        let num_rows = 100;

        let mut consumer = Consumer::default();
        let otap_batch = consumer
            .consume_bar(&mut fixtures::create_simple_trace_arrow_record_batches(
                SimpleDataGenOptions {
                    num_rows,
                    traces_options: Some(Default::default()),
                    ..Default::default()
                },
            ))
            .unwrap();
        let otap_batch = OtapArrowRecords::Traces(from_record_messages(otap_batch));
        test_runtime
            .set_exporter(exporter)
            .run_test(move |ctx| {
                Box::pin(async move {
                    ctx.send_pdata(OtapPdata::new_default(otap_batch.into()))
                        .await
                        .expect("Failed to send  logs message");

                    ctx.send_shutdown(
                        Instant::now().add(Duration::from_millis(200)),
                        "test completed",
                    )
                    .await
                    .unwrap();
                })
            })
            .run_validation(move |_ctx, exporter_result| {
                Box::pin(async move {
                    exporter_result.unwrap();

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
        let test_runtime = TestRuntime::<OtapPdata>::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let base_dir: String = temp_dir.path().to_str().unwrap().into();
        let exporter = ParquetExporter::new(config::Config {
            storage: crate::object_store::StorageType::File {
                base_uri: base_dir.clone(),
            },
            partitioning_strategies: None,
            writer_options: None,
        });
        let node_config = Arc::new(NodeUserConfig::new_exporter_config(PARQUET_EXPORTER_URN));
        let exporter = ExporterWrapper::<OtapPdata>::local::<ParquetExporter>(
            exporter,
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        let num_rows = 100;
        let mut consumer = Consumer::default();
        let otap_batch = consumer
            .consume_bar(&mut fixtures::create_simple_metrics_arrow_record_batches(
                SimpleDataGenOptions {
                    num_rows,
                    metrics_options: Some(Default::default()),
                    ..Default::default()
                },
            ))
            .unwrap();
        let otap_batch = OtapArrowRecords::Metrics(from_record_messages(otap_batch));
        test_runtime
            .set_exporter(exporter)
            .run_test(move |ctx| {
                Box::pin(async move {
                    ctx.send_pdata(OtapPdata::new_default(otap_batch.into()))
                        .await
                        .expect("Failed to send  logs message");

                    ctx.send_shutdown(
                        Instant::now().add(Duration::from_millis(1000)),
                        "test completed",
                    )
                    .await
                    .unwrap();
                })
            })
            .run_validation(move |_ctx, exporter_result| {
                Box::pin(async move {
                    exporter_result.unwrap();

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

    #[test]
    fn test_collect_telemetry_reports_metrics() {
        use otap_df_engine::context::ControllerContext;
        use otap_df_engine::testing::test_node;
        use serde_json::json;

        let test_runtime = TestRuntime::<OtapPdata>::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let base_dir: String = temp_dir.path().to_str().unwrap().into();

        // Telemetry system: registry + reporter + background collector
        let metrics_system = otap_df_telemetry::InternalTelemetrySystem::default();
        let telemetry_registry = metrics_system.registry();
        let reporter = metrics_system.reporter();

        // Build exporter with metrics via from_config
        let controller_ctx = ControllerContext::new(telemetry_registry.clone());
        let pipeline_ctx = controller_ctx
            .pipeline_context_with("grp".into(), "pipe".into(), 0, 1, 0)
            .with_node_context(
                "parquet_exporter".into(),
                PARQUET_EXPORTER_URN.into(),
                otap_df_config::node::NodeKind::Exporter,
                std::collections::HashMap::new(),
            );

        let exporter_impl = ParquetExporter::from_config(
            pipeline_ctx,
            &json!({ "storage": { "file": { "base_uri": base_dir } } }),
        )
        .unwrap();

        let node_config = Arc::new(NodeUserConfig::new_exporter_config(PARQUET_EXPORTER_URN));
        let mut exporter = ExporterWrapper::<OtapPdata>::local::<ParquetExporter>(
            exporter_impl,
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        let (rt, local) = setup_test_runtime();
        let control_sender = exporter.control_sender();

        // pdata channel
        let (pdata_tx_ch, pdata_rx_ch) = create_not_send_channel::<OtapPdata>(1);
        let pdata_tx = Sender::Local(LocalSender::mpsc(pdata_tx_ch));
        let pdata_rx = Receiver::Local(LocalReceiver::mpsc(pdata_rx_ch));
        exporter
            .set_pdata_receiver(test_node("exp"), pdata_rx)
            .expect("Failed to set PData Receiver");

        // Keep pipeline ctrl receiver alive
        let (pipeline_ctrl_msg_tx, _pipeline_ctrl_msg_rx) = pipeline_ctrl_msg_channel(10);

        async fn start_exporter(
            exporter: ExporterWrapper<OtapPdata>,
            pipeline_ctrl_msg_tx: PipelineCtrlMsgSender<OtapPdata>,
            metrics_reporter: otap_df_telemetry::reporter::MetricsReporter,
        ) -> Result<(), Error> {
            exporter
                .start(pipeline_ctrl_msg_tx, metrics_reporter)
                .await
                .map(|_| ())
        }

        async fn drive_test(
            control_sender: Sender<NodeControlMsg<OtapPdata>>,
            pdata_tx: Sender<OtapPdata>,
            reporter: otap_df_telemetry::reporter::MetricsReporter,
        ) {
            // Send minimal pdata to increment pdata metrics (consumed)
            let logs = Consumer::default()
                .consume_bar(&mut fixtures::create_simple_logs_arrow_record_batches(
                    SimpleDataGenOptions {
                        num_rows: 1,
                        ..Default::default()
                    },
                ))
                .unwrap();
            let otap_batch = OtapArrowRecords::Logs(from_record_messages(logs)).into();
            pdata_tx
                .send(OtapPdata::new_default(otap_batch))
                .await
                .unwrap();

            // Allow exporter loop to process pdata before collecting telemetry
            // Control messages are prioritized over pdata; without this short delay, the
            // CollectTelemetry message could be handled first and report zeros.
            sleep(Duration::from_millis(50)).await;

            // Trigger telemetry collection
            control_sender
                .send(NodeControlMsg::CollectTelemetry {
                    metrics_reporter: reporter.clone(),
                })
                .await
                .unwrap();

            // Give collector a little time to process
            sleep(Duration::from_millis(50)).await;

            // Then shutdown
            let _ = control_sender
                .send(NodeControlMsg::Shutdown {
                    deadline: Instant::now(),
                    reason: "done".into(),
                })
                .await;
        }

        // Run everything on the local task set, including the metrics collector
        let _ = rt.block_on(local.run_until(async move {
            // Start collector in background
            let collector = metrics_system.collector();
            let _handle = tokio::task::spawn_local(collector.run_collection_loop());

            tokio::join!(
                start_exporter(exporter, pipeline_ctrl_msg_tx, reporter.clone()),
                drive_test(control_sender, pdata_tx, reporter.clone()),
            )
        }));

        // Inspect registry to ensure exporter.pdata registered counters were reported
        let mut saw_exporter_pdata = false;
        let mut any_positive = false;
        telemetry_registry.visit_current_metrics(|desc, _attrs, iter| {
            if desc.name == "exporter.pdata" {
                saw_exporter_pdata = true;
                for (_field, value) in iter {
                    if value.to_f64() > 0.0 {
                        any_positive = true;
                    }
                }
            }
        });
        assert!(
            saw_exporter_pdata,
            "expected exporter.pdata metric set to be registered"
        );
        assert!(
            any_positive,
            "expected at least one exporter.pdata counter > 0"
        );
    }

    #[test]
    fn test_handles_null_ids() {
        let test_runtime = TestRuntime::<OtapPdata>::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let base_dir: String = temp_dir.path().to_str().unwrap().into();
        let exporter = ParquetExporter::new(config::Config {
            storage: crate::object_store::StorageType::File {
                base_uri: base_dir.clone(),
            },
            partitioning_strategies: None,
            writer_options: None,
        });
        let node_config = Arc::new(NodeUserConfig::new_exporter_config(PARQUET_EXPORTER_URN));
        let exporter = ExporterWrapper::<OtapPdata>::local::<ParquetExporter>(
            exporter,
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        let num_rows = 100;
        let mut consumer = Consumer::default();
        let otap_batch = consumer
            .consume_bar(&mut fixtures::create_simple_logs_arrow_record_batches(
                SimpleDataGenOptions {
                    num_rows,
                    with_main_record_attrs: false,
                    with_resource_attrs: false,
                    with_scope_attrs: false,
                    ..Default::default()
                },
            ))
            .unwrap();
        let mut otap_batch = OtapArrowRecords::Logs(from_record_messages(otap_batch));
        // replace the logs ID column with nulls
        let logs = otap_batch.get(ArrowPayloadType::Logs).unwrap();
        let (schema, mut columns, _) = logs.clone().into_parts();
        columns[schema.index_of(consts::ID).unwrap()] = Arc::new(UInt16Array::new_null(num_rows));
        otap_batch.set(
            ArrowPayloadType::Logs,
            RecordBatch::try_new(schema, columns).unwrap(),
        );

        test_runtime
            .set_exporter(exporter)
            .run_test(move |ctx| {
                Box::pin(async move {
                    ctx.send_pdata(OtapPdata::new_default(otap_batch.into()))
                        .await
                        .unwrap();
                    ctx.send_shutdown(
                        Instant::now().add(Duration::from_millis(1000)),
                        "test complete",
                    )
                    .await
                    .unwrap();
                })
            })
            .run_validation(move |_ctx, exporter_result| {
                Box::pin(async move {
                    exporter_result.unwrap();
                    assert_parquet_file_has_rows(&base_dir, ArrowPayloadType::Logs, num_rows).await;
                })
            });
    }
}
