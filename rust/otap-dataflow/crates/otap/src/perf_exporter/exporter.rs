// SPDX-License-Identifier: Apache-2.0

//! Implementation of the Perf exporter node
//!
//! ToDo: Handle configuration changes
//! ToDo: Implement proper deadline function for Shutdown ctrl msg
//! ToDo: track cpu usage per thread
//! ToDo: track memory allocation per thread
//! ToDo: calculate average latency of otlp signals

use crate::OTAP_EXPORTER_FACTORIES;
use crate::grpc::OtapArrowBytes;
use crate::pdata::OtapPdata;
use crate::perf_exporter::config::Config;
use async_trait::async_trait;
use byte_unit::Byte;
use fluke_hpack::Decoder;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::config::ExporterConfig;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::error::Error;
use otap_df_engine::exporter::ExporterWrapper;
use otap_df_engine::local::exporter as local;
use otap_df_engine::message::{Message, MessageChannel};
use otap_df_engine::{ExporterFactory, distributed_slice};
use otel_arrow_rust::Consumer;
use otel_arrow_rust::otap::{OtapArrowRecords, from_record_messages};
use otel_arrow_rust::proto::opentelemetry::arrow::v1::{ArrowPayloadType, BatchArrowRecords};
use serde_json::Value;
use std::borrow::Cow;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use sysinfo::{
    CpuRefreshKind, DiskUsage, NetworkData, Networks, Process, ProcessRefreshKind, RefreshKind,
    System, get_current_pid,
};
use tokio::fs::File;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use tokio::time::{Duration, Instant};

/// A wrapper around AsyncWrite that simplifies error handling for debug output
struct OutputWriter {
    writer: Box<dyn AsyncWrite + Unpin>,
    exporter_id: Cow<'static, str>,
}

impl OutputWriter {
    fn new(writer: Box<dyn AsyncWrite + Unpin>, exporter_id: Cow<'static, str>) -> Self {
        Self {
            writer,
            exporter_id,
        }
    }

    async fn write(&mut self, data: &str) -> Result<(), Error<OtapPdata>> {
        self.writer
            .write_all(data.as_bytes())
            .await
            .map_err(|e| Error::ExporterError {
                exporter: self.exporter_id.clone(),
                error: format!("Write error: {e}\n"),
            })
    }
}

/// The URN for the OTAP Perf exporter
pub const OTAP_PERF_EXPORTER_URN: &str = "urn:otel:otap:perf:exporter";

/// Perf Exporter that emits performance data
pub struct PerfExporter {
    config: Config,
    output: Option<String>,
}

/// Declares the OTAP Perf exporter as a local exporter factory
///
/// Unsafe code is temporarily used here to allow the use of `distributed_slice` macro
/// This macro is part of the `linkme` crate which is considered safe and well maintained.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
pub static PERF_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: OTAP_PERF_EXPORTER_URN,
    create: |node_config: Arc<NodeUserConfig>, exporter_config: &ExporterConfig| {
        Ok(ExporterWrapper::local(
            PerfExporter::from_config(&node_config.config)?,
            node_config,
            exporter_config,
        ))
    },
};

impl PerfExporter {
    /// creates a perf exporter with the provided config
    #[must_use]
    pub fn new(config: Config, output: Option<String>) -> Self {
        PerfExporter { config, output }
    }

    /// Creates a new PerfExporter from a configuration object
    pub fn from_config(config: &Value) -> Result<Self, otap_df_config::error::Error> {
        Ok(PerfExporter {
            config: serde_json::from_value(config.clone()).map_err(|e| {
                otap_df_config::error::Error::InvalidUserConfig {
                    error: e.to_string(),
                }
            })?,
            output: None,
        })
    }
}

#[async_trait(?Send)]
impl local::Exporter<OtapPdata> for PerfExporter {
    async fn start(
        self: Box<Self>,
        mut msg_chan: MessageChannel<OtapPdata>,
        effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<(), Error<OtapPdata>> {
        // init variables for tracking
        let mut average_pipeline_latency: f64 = 0.0;
        let mut received_arrow_records_count: u64 = 0;
        let mut received_otlp_signal_count: u64 = 0;
        let mut total_received_arrow_records_count: u128 = 0;
        let mut total_received_otlp_signal_count: u128 = 0;
        let mut received_pdata_batch_count: u64 = 0;
        let mut invalid_pdata_count: u64 = 0;
        let mut total_received_pdata_batch_count: u64 = 0;
        let mut last_perf_time: Instant = Instant::now();
        let process_pid = get_current_pid().map_err(|e| Error::ExporterError {
            exporter: effect_handler.exporter_id(),
            error: format!("Failed to get process pid: {e}\n"),
        })?;
        let mut system = System::new();

        let raw_writer = get_writer(self.output).await;
        let mut writer = OutputWriter::new(raw_writer, effect_handler.exporter_id());

        let sys_refresh_list = sys_refresh_list(&self.config);

        effect_handler.info("Starting Perf Exporter\n").await;
        _ = effect_handler
            .start_periodic_timer(Duration::from_millis(self.config.frequency()))
            .await?;

        // Helper function to generate performance report
        let generate_report = async |received_arrow_records_count: u64,
                                     received_otlp_signal_count: u64,
                                     received_pdata_batch_count: u64,
                                     total_received_arrow_records_count: u128,
                                     total_received_otlp_signal_count: u128,
                                     total_received_pdata_batch_count: u64,
                                     invalid_pdata_count: u64,
                                     average_pipeline_latency: f64,
                                     last_perf_time: Instant,
                                     system: &mut System,
                                     config: &Config,
                                     writer: &mut OutputWriter,
                                     process_pid: sysinfo::Pid|
               -> Result<(), Error<OtapPdata>> {
            let now = Instant::now();
            let duration = now - last_perf_time;

            // display pipeline report
            writer
                .write("====================Pipeline Report====================\n")
                .await?;
            display_report_pipeline(
                received_arrow_records_count,
                received_otlp_signal_count,
                received_pdata_batch_count,
                total_received_arrow_records_count,
                total_received_otlp_signal_count,
                total_received_pdata_batch_count,
                invalid_pdata_count,
                average_pipeline_latency,
                duration,
                writer,
            )
            .await?;

            // update sysinfo data
            system.refresh_specifics(sys_refresh_list);
            let process = system.process(process_pid).ok_or(Error::ExporterError {
                exporter: effect_handler.exporter_id(),
                error: "Failed to get process".to_owned(),
            })?;

            // check configuration and display data accordingly
            if config.mem_usage() {
                writer
                    .write("=====================Memory Usage======================\n")
                    .await?;
                display_mem_usage(process, writer).await?;
            }
            if config.cpu_usage() {
                writer
                    .write("=======================Cpu Usage=======================\n")
                    .await?;
                display_cpu_usage(process, system, writer).await?;
            }
            if config.disk_usage() {
                writer
                    .write("======================Disk Usage=======================\n")
                    .await?;
                let disk_usage = process.disk_usage();
                display_disk_usage(disk_usage, duration, writer).await?;
            }
            if config.io_usage() {
                let networks = Networks::new_with_refreshed_list();
                writer
                    .write("=====================Network Usage=====================\n")
                    .await?;
                for (interface_name, network) in &networks {
                    display_io_usage(interface_name, network, duration, writer).await?;
                }
            }

            Ok(())
        };

        // Loop until a Shutdown event is received.
        loop {
            let msg = msg_chan.recv().await?;
            match msg {
                Message::Control(NodeControlMsg::TimerTick { .. }) => {
                    generate_report(
                        received_arrow_records_count,
                        received_otlp_signal_count,
                        received_pdata_batch_count,
                        total_received_arrow_records_count,
                        total_received_otlp_signal_count,
                        total_received_pdata_batch_count,
                        invalid_pdata_count,
                        average_pipeline_latency,
                        last_perf_time,
                        &mut system,
                        &self.config,
                        &mut writer,
                        process_pid,
                    )
                    .await?;

                    // reset received counters and update last_perf_time
                    received_arrow_records_count = 0;
                    received_otlp_signal_count = 0;
                    received_pdata_batch_count = 0;
                    last_perf_time = Instant::now();
                }
                // ToDo: Handle configuration changes
                Message::Control(NodeControlMsg::Config { .. }) => {}
                Message::Control(NodeControlMsg::Shutdown { .. }) => {
                    // Generate final performance report before shutting down
                    generate_report(
                        received_arrow_records_count,
                        received_otlp_signal_count,
                        received_pdata_batch_count,
                        total_received_arrow_records_count,
                        total_received_otlp_signal_count,
                        total_received_pdata_batch_count,
                        invalid_pdata_count,
                        average_pipeline_latency,
                        last_perf_time,
                        &mut system,
                        &self.config,
                        &mut writer,
                        process_pid,
                    )
                    .await?;
                    break;
                }
                Message::PData(pdata) => {
                    let mut batch = match OtapArrowBytes::try_from(pdata) {
                        Ok(OtapArrowBytes::ArrowLogs(batch)) => batch,
                        Ok(OtapArrowBytes::ArrowMetrics(batch)) => batch,
                        Ok(OtapArrowBytes::ArrowTraces(batch)) => batch,
                        Err(_) => {
                            // if the pdata is not a valid Arrow batch, we skip it.
                            // For example, when it is a not supported signal type.
                            invalid_pdata_count += 1;
                            total_received_pdata_batch_count += 1;
                            continue;
                        }
                    };
                    // keep track of batches received
                    received_pdata_batch_count += 1;
                    total_received_pdata_batch_count += 1;
                    // decode the headers which are hpack encoded
                    // check for timestamp
                    // get time delta between now and timestamp
                    // calculate average
                    let mut decoder = Decoder::new();
                    let header_list =
                        decoder
                            .decode(&batch.headers)
                            .map_err(|_| Error::ExporterError {
                                exporter: effect_handler.exporter_id(),
                                error: "Failed to decode batch headers".to_owned(),
                            })?;
                    // find the timestamp header and parse it
                    // timestamp will be added in the receiver to enable pipeline latency calculation
                    let timestamp_pair = header_list.iter().find(|(name, _)| name == b"timestamp");
                    if let Some((_, value)) = timestamp_pair {
                        let timestamp =
                            decode_timestamp(value).map_err(|error| Error::ExporterError {
                                exporter: effect_handler.exporter_id(),
                                error,
                            })?;
                        let current_unix_time = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .map_err(|error| Error::ExporterError {
                                exporter: effect_handler.exporter_id(),
                                error: error.to_string(),
                            })?;
                        let latency = (current_unix_time - timestamp).as_secs_f64();
                        average_pipeline_latency = update_average(
                            latency,
                            average_pipeline_latency,
                            self.config.smoothing_factor() as f64,
                        );
                    }

                    // increment counters for received arrow payloads
                    received_arrow_records_count += batch.arrow_payloads.len() as u64;
                    total_received_arrow_records_count += batch.arrow_payloads.len() as u128;
                    // increment counters for otlp signals
                    let batch_received_otlp_signal_count =
                        calculate_otlp_signal_count(&mut batch).unwrap_or_default();
                    received_otlp_signal_count += batch_received_otlp_signal_count;
                    total_received_otlp_signal_count += batch_received_otlp_signal_count as u128;
                }
                _ => {
                    return Err(Error::ExporterError {
                        exporter: effect_handler.exporter_id(),
                        error: "Unknown control message".to_owned(),
                    });
                }
            }
        }
        Ok(())
    }
}

/// determine if output goes to console or to a file
async fn get_writer(output_file: Option<String>) -> Box<dyn AsyncWrite + Unpin> {
    match output_file {
        Some(file_name) => {
            let file = File::options()
                .write(true)
                .create(true)
                .truncate(true)
                .open(file_name)
                .await
                .expect("could not open output file\n");
            Box::new(file)
        }
        None => Box::new(tokio::io::stdout()),
    }
}

/// takes in the configuration and returns the appropriate system refresh list to make sure only the necessary data is refreshed
fn sys_refresh_list(config: &Config) -> RefreshKind {
    let mut sys_refresh_list = RefreshKind::nothing();
    let mut process_refresh_list = ProcessRefreshKind::nothing();

    if config.disk_usage() {
        process_refresh_list = process_refresh_list.with_disk_usage();
    }
    if config.mem_usage() {
        process_refresh_list = process_refresh_list.with_memory();
    }
    if config.cpu_usage() {
        process_refresh_list = process_refresh_list.with_cpu();
        sys_refresh_list = sys_refresh_list.with_cpu(CpuRefreshKind::nothing().with_cpu_usage());
    }

    sys_refresh_list.with_processes(process_refresh_list)
}

/// uses the exponential moving average formula to update the average
fn update_average(new_value: f64, old_average: f64, smoothing_factor: f64) -> f64 {
    // update the average using a exponential moving average which allows new data points to have a greater impact depending on the smoothing factor
    smoothing_factor * new_value + (1.0 - smoothing_factor) * old_average
}

/// decodes the byte array from the timestamp header and gets the equivalent duration value
fn decode_timestamp(timestamp: &[u8]) -> Result<Duration, String> {
    let timestamp_string = std::str::from_utf8(timestamp).map_err(|error| error.to_string())?;
    let timestamp_parts: Vec<&str> = timestamp_string.split(":").collect();
    let secs = timestamp_parts[0]
        .parse::<u64>()
        .map_err(|error| error.to_string())?;
    let nanosecs = timestamp_parts[1]
        .parse::<u32>()
        .map_err(|error| error.to_string())?;

    Ok(Duration::new(secs, nanosecs))
}

/// Calculate number of OTLP signals based on Arrow payload type
fn calculate_otlp_signal_count(
    batch: &mut BatchArrowRecords,
) -> Result<u64, otel_arrow_rust::error::Error> {
    let main_record_type = batch.arrow_payloads[0].r#type;

    // Handle the different ArrowPayloadTypes using a match statement
    match ArrowPayloadType::try_from(main_record_type) {
        Ok(payload_type)
            if matches!(
                payload_type,
                ArrowPayloadType::Spans
                    | ArrowPayloadType::Logs
                    | ArrowPayloadType::UnivariateMetrics
            ) =>
        {
            let mut consumer = Consumer::default();
            consumer.consume_bar(batch).map(|record_messages| {
                let otap_batch = match payload_type {
                    ArrowPayloadType::Spans => {
                        OtapArrowRecords::Traces(from_record_messages(record_messages))
                    }
                    ArrowPayloadType::Logs => {
                        OtapArrowRecords::Logs(from_record_messages(record_messages))
                    }
                    ArrowPayloadType::UnivariateMetrics => {
                        OtapArrowRecords::Metrics(from_record_messages(record_messages))
                    }
                    _ => return 0, // Handle all other cases by returning 0 immediately
                };
                otap_batch
                    .get(payload_type)
                    .map_or(0, |record_batch| record_batch.num_rows()) as u64
            })
        }
        // Return 0 for all other types or invalid input
        _ => Ok(0),
    }
}

/// takes in the process and system outputs cpu stats via the writer
async fn display_cpu_usage(
    process: &Process,
    system: &System,
    writer: &mut OutputWriter,
) -> Result<(), Error<OtapPdata>> {
    let process_cpu_usage = process.cpu_usage(); // as %
    let global_cpu_usage = system.global_cpu_usage(); // as %
    writer
        .write(&format!(
            "\t- global cpu usage                  : {process_cpu_usage}% (100% is all cores)\n"
        ))
        .await?;
    writer
        .write(&format!(
            "\t- process cpu usage                 : {global_cpu_usage}% (100% is a single core)\n",
        ))
        .await?;
    Ok(())
}

/// takes in the process outputs memory stats via the writer
async fn display_mem_usage(
    process: &Process,
    writer: &mut OutputWriter,
) -> Result<(), Error<OtapPdata>> {
    let memory_rss = process.memory(); // as bytes
    let memory_virtual = process.virtual_memory(); // as bytes
    writer
        .write(&format!(
            "\t- memory rss                        : {}\n",
            Byte::from_bytes(memory_rss as u128).get_appropriate_unit(false),
        ))
        .await?;
    writer
        .write(&format!(
            "\t- memory virtual                    : {}\n",
            Byte::from_bytes(memory_virtual as u128).get_appropriate_unit(false),
        ))
        .await?;
    Ok(())
}

/// takes in the disk usage data and outputs stats via the writer
async fn display_disk_usage(
    disk_usage: DiskUsage,
    duration: Duration,
    writer: &mut OutputWriter,
) -> Result<(), Error<OtapPdata>> {
    let total_written_bytes = disk_usage.total_written_bytes;
    let total_read_bytes = disk_usage.total_read_bytes;
    let written_bytes = disk_usage.written_bytes;
    let read_bytes = disk_usage.read_bytes;
    writer
        .write(&format!(
            "\t- read bytes                        : {}/s\n",
            Byte::from_bytes((read_bytes as f64 / duration.as_secs_f64()) as u128)
                .get_appropriate_unit(false), // duration.as_secs_f64()
        ))
        .await?;
    writer
        .write(&format!(
            "\t- total read bytes                  : {}\n",
            Byte::from_bytes(total_read_bytes as u128).get_appropriate_unit(false),
        ))
        .await?;
    writer
        .write(&format!(
            "\t- written bytes                     : {}/s\n",
            Byte::from_bytes((written_bytes as f64 / duration.as_secs_f64()) as u128)
                .get_appropriate_unit(false),
        ))
        .await?;
    writer
        .write(&format!(
            "\t- total written bytes               : {}\n",
            Byte::from_bytes(total_written_bytes as u128).get_appropriate_unit(false),
        ))
        .await?;
    Ok(())
}

/// takes in network data and outputs networks stats via the writer
async fn display_io_usage(
    network_name: &str,
    network_data: &NetworkData,
    duration: Duration,
    writer: &mut OutputWriter,
) -> Result<(), Error<OtapPdata>> {
    // per interface, get networkdata
    let bytes_received = network_data.received(); //as bytes u64
    let total_bytes_received = network_data.total_received(); // as bytes u64

    let bytes_transmitted = network_data.transmitted();
    let total_bytes_transmitted = network_data.total_transmitted();

    let packets_received = network_data.packets_received();
    let total_packets_received = network_data.total_packets_received();

    let packets_transmitted = network_data.packets_transmitted();
    let total_packets_transmitted = network_data.total_packets_transmitted();

    let errors_on_received = network_data.errors_on_received();
    let total_errors_on_received = network_data.total_errors_on_received();

    let errors_on_transmitted = network_data.errors_on_transmitted();
    let total_errors_on_transmitted = network_data.total_errors_on_transmitted();
    writer
        .write(&format!("Network Interface: {network_name}\n"))
        .await?;
    writer
        .write(&format!(
            "\t- bytes read                        : {}/s\n",
            Byte::from_bytes((bytes_received as f64 / duration.as_secs_f64()) as u128)
                .get_appropriate_unit(false),
        ))
        .await?;
    writer
        .write(&format!(
            "\t- total bytes recevied              : {}\n",
            Byte::from_bytes(total_bytes_received as u128).get_appropriate_unit(false),
        ))
        .await?;
    writer
        .write(&format!(
            "\t- bytes transmitted                 : {}/s\n",
            Byte::from_bytes((bytes_transmitted as f64 / duration.as_secs_f64()) as u128)
                .get_appropriate_unit(false),
        ))
        .await?;
    writer
        .write(&format!(
            "\t- total bytes transmitted           : {}\n",
            Byte::from_bytes(total_bytes_transmitted as u128).get_appropriate_unit(false),
        ))
        .await?;
    writer
        .write(&format!(
            "\t- packets received                  : {}/s\n",
            Byte::from_bytes((packets_received as f64 / duration.as_secs_f64()) as u128)
                .get_appropriate_unit(false),
        ))
        .await?;
    writer
        .write(&format!(
            "\t- total packets received            : {}\n",
            Byte::from_bytes(total_packets_received as u128).get_appropriate_unit(false),
        ))
        .await?;
    writer
        .write(&format!(
            "\t- packets transmitted               : {}/s\n",
            Byte::from_bytes((packets_transmitted as f64 / duration.as_secs_f64()) as u128)
                .get_appropriate_unit(false),
        ))
        .await?;
    writer
        .write(&format!(
            "\t- total packets transmitted         : {}\n",
            Byte::from_bytes(total_packets_transmitted as u128).get_appropriate_unit(false),
        ))
        .await?;
    writer
        .write(&format!(
            "\t- errors on received                : {}/s\n",
            Byte::from_bytes((errors_on_received as f64 / duration.as_secs_f64()) as u128)
                .get_appropriate_unit(false),
        ))
        .await?;
    writer
        .write(&format!(
            "\t- total errors on received          : {}\n",
            Byte::from_bytes(total_errors_on_received as u128).get_appropriate_unit(false),
        ))
        .await?;
    writer
        .write(&format!(
            "\t- errors on transmitted             : {}/s\n",
            Byte::from_bytes((errors_on_transmitted as f64 / duration.as_secs_f64()) as u128)
                .get_appropriate_unit(false),
        ))
        .await?;
    writer
        .write(&format!(
            "\t- total errors on transmitted       : {}\n",
            Byte::from_bytes(total_errors_on_transmitted as u128).get_appropriate_unit(false),
        ))
        .await?;
    Ok(())
}

/// accepts pipeline statistics and prints a report via the writer
async fn display_report_pipeline(
    received_arrow_records_count: u64,
    received_otlp_signal_count: u64,
    received_pdata_batch_count: u64,
    total_received_arrow_records_count: u128,
    total_received_otlp_signal_count: u128,
    total_received_pdata_batch_count: u64,
    invalid_pdata_count: u64,
    average_pipeline_latency: f64,
    duration: Duration,
    writer: &mut OutputWriter,
) -> Result<(), Error<OtapPdata>> {
    writer
        .write(&format!(
            "\t- arrow records throughput          : {:.2} arrow-records/s\n",
            (received_arrow_records_count as f64 / duration.as_secs_f64()),
        ))
        .await?;
    writer
        .write(&format!(
            "\t- average pipeline latency          : {:.2} s\n",
            average_pipeline_latency,
        ))
        .await?;

    writer
        .write(&format!(
            "\t- total arrow records received      : {total_received_arrow_records_count}\n"
        ))
        .await?;
    writer
        .write(&format!(
            "\t- otlp signal throughput            : {:.2} otlp/s\n",
            (received_otlp_signal_count as f64 / duration.as_secs_f64()),
        ))
        .await?;
    writer
        .write(&format!(
            "\t- total otlp signal received        : {total_received_otlp_signal_count}\n"
        ))
        .await?;
    writer
        .write(&format!(
            "\t- pdata batch throughput            : {:.2} pdata/s\n",
            received_pdata_batch_count as f64 / duration.as_secs_f64(),
        ))
        .await?;

    writer
        .write(&format!(
            "\t- total pdata batch received        : {total_received_pdata_batch_count}\n"
        ))
        .await?;

    writer
        .write(&format!(
            "\t- invalid pdata batches received    : {invalid_pdata_count}\n"
        ))
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::fixtures::{
        SimpleDataGenOptions, create_simple_logs_arrow_record_batches,
        create_simple_metrics_arrow_record_batches, create_simple_trace_arrow_record_batches,
    };
    use crate::grpc::OtapArrowBytes;
    use crate::pdata::OtapPdata;
    use crate::perf_exporter::config::Config;
    use crate::perf_exporter::exporter::{OTAP_PERF_EXPORTER_URN, PerfExporter};
    use otap_df_config::node::NodeUserConfig;
    use otap_df_engine::error::Error;
    use otap_df_engine::exporter::ExporterWrapper;
    use otap_df_engine::testing::exporter::TestContext;
    use otap_df_engine::testing::exporter::TestRuntime;
    use std::collections::HashMap;
    use std::fs::File;
    use std::future::Future;
    use std::io::{BufReader, prelude::*};
    use std::sync::Arc;
    use tempfile::NamedTempFile;
    use tokio::time::{Duration, sleep};

    /// Test closure that simulates a typical test scenario by sending timer ticks, config,
    /// data message, and shutdown control messages.
    ///
    fn scenario()
    -> impl FnOnce(TestContext<OtapPdata>) -> std::pin::Pin<Box<dyn Future<Output = ()>>> {
        |ctx| {
            Box::pin(async move {
                // send some messages to the exporter to calculate pipeline statistics
                for i in 0..3 {
                    let traces_batch_data =
                        create_simple_trace_arrow_record_batches(SimpleDataGenOptions {
                            id_offset: 3 * i,
                            num_rows: 5,
                            ..Default::default()
                        });
                    let logs_batch_data =
                        create_simple_logs_arrow_record_batches(SimpleDataGenOptions {
                            id_offset: 3 * i + 1,
                            num_rows: 5,
                            ..Default::default()
                        });
                    let metrics_batch_data =
                        create_simple_metrics_arrow_record_batches(SimpleDataGenOptions {
                            id_offset: 3 * i + 2,
                            num_rows: 5,
                            ..Default::default()
                        });
                    // // Send a data message
                    ctx.send_pdata(OtapArrowBytes::ArrowTraces(traces_batch_data).into())
                        .await
                        .expect("Failed to send data message");
                    ctx.send_pdata(OtapArrowBytes::ArrowLogs(logs_batch_data).into())
                        .await
                        .expect("Failed to send data message");
                    ctx.send_pdata(OtapArrowBytes::ArrowMetrics(metrics_batch_data).into())
                        .await
                        .expect("Failed to send data message");
                }

                // TODO ADD DELAY BETWEEN HERE
                _ = sleep(Duration::from_millis(5000));

                // send timertick to generate the report
                ctx.send_timer_tick()
                    .await
                    .expect("Failed to send TimerTick");

                // Send shutdown
                ctx.send_shutdown(Duration::from_millis(200), "test complete")
                    .await
                    .expect("Failed to send Shutdown");
            })
        }
    }

    /// Validation closure that checks the expected counter values
    fn validation_procedure(
        output_file: String,
    ) -> impl FnOnce(
        TestContext<OtapPdata>,
        Result<(), Error<OtapPdata>>,
    ) -> std::pin::Pin<Box<dyn Future<Output = ()>>> {
        |_, exporter_result| {
            Box::pin(async move {
                assert!(exporter_result.is_ok());

                // get a file to read and validate the output
                // open file
                // read the output file
                // assert each line accordingly
                let file = File::open(output_file).expect("failed to open file");
                let reader = BufReader::new(file);
                let mut lines = Vec::new();
                for line in reader.lines() {
                    lines.push(line.unwrap());
                }

                // Check lines with batch / signal counts for correct values
                let prefixes = vec![
                    ("total_otlp", "\t- total otlp signal received"),
                    ("total_pdata", "\t- total pdata batch received"),
                ];
                // Map to store the last match per pattern
                let mut last_values: HashMap<&str, u32> = HashMap::new();
                for line in &lines {
                    for (key, prefix) in &prefixes {
                        if line.starts_with(prefix) {
                            if let Some((_prefix, value_str)) = line.split_once(":") {
                                if let Ok(value) = value_str.trim().parse::<u32>() {
                                    _ = last_values.insert(*key, value);
                                }
                            }
                        }
                    }
                }

                // 3 * 3 batches of 5 signals each = 9 batches, 45 signals
                assert_eq!(
                    last_values.get("total_otlp"),
                    Some(&45),
                    "Expected total otlp signal received to be 45"
                );
                assert_eq!(
                    last_values.get("total_pdata"),
                    Some(&9),
                    "Expected total pdata batch received to be 9"
                );

                // report contains message throughput
                let message_throughput_line = &lines[1];
                assert!(message_throughput_line.contains("- arrow records throughput"),);

                // report contains pipeline latency
                let avg_pipeline_latency_line = &lines[2];
                assert!(avg_pipeline_latency_line.contains("- average pipeline latency"));

                let total_msg_line = &lines[3];
                assert!(total_msg_line.contains("- total arrow records received"),);

                let otlp_signal_throughput_line = &lines[4];
                assert!(otlp_signal_throughput_line.contains("- otlp signal throughput"),);

                let total_otlp_signal_line = &lines[5];
                assert!(total_otlp_signal_line.contains("- total otlp signal received"),);

                let pdata_throughput_line = &lines[6];
                assert!(pdata_throughput_line.contains("- pdata batch throughput"),);

                let total_pdata_line = &lines[7];
                assert!(total_pdata_line.contains("- total pdata batch received"),);

                //memory report contains memory rss
                let memory_rss_line = &lines[10];
                assert!(memory_rss_line.contains("- memory rss"),);

                //memory report contains memory virtual
                let memory_virtual_line = &lines[11];
                assert!(memory_virtual_line.contains("- memory virtual"),);

                // cpu report contains cpu usage
                let process_cpu_usage = &lines[13];
                assert!(process_cpu_usage.contains("- global cpu usage"),);

                let global_cpu_usage = &lines[14];
                assert!(global_cpu_usage.contains("- process cpu usage"),);

                // disk report contains read bytes
                let read_bytes_line = &lines[16];
                assert!(read_bytes_line.contains("- read bytes"),);

                // disk report contains total read bytes
                let total_read_bytes_line = &lines[17];
                assert!(total_read_bytes_line.contains("- total read bytes"),);

                // disk report contains written bytes
                let written_bytes_line = &lines[18];
                assert!(written_bytes_line.contains("- written bytes"),);

                // disk report contains total written bytes
                let total_written_bytes_line = &lines[19];
                assert!(total_written_bytes_line.contains("- total written bytes"));
            })
        }
    }

    #[test]
    fn test_exporter_local() {
        let test_runtime = TestRuntime::new();
        let config = Config::new(1000, 0.3, true, true, true, true, true);
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let output_file = temp_file.path().to_str().unwrap().to_string();
        let node_config = Arc::new(NodeUserConfig::new_exporter_config(OTAP_PERF_EXPORTER_URN));
        let exporter = ExporterWrapper::local(
            PerfExporter::new(config, Some(output_file.clone())),
            node_config,
            test_runtime.config(),
        );

        test_runtime
            .set_exporter(exporter)
            .run_test(scenario())
            .run_validation(validation_procedure(output_file.clone()));
    }
}
