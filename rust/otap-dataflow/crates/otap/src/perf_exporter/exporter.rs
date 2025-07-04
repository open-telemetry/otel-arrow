// SPDX-License-Identifier: Apache-2.0

//! Implementation of the Perf exporter node
//!
//! ToDo: Handle configuration changes
//! ToDo: Implement proper deadline function for Shutdown ctrl msg
//! ToDo: track cpu usage per thread
//! ToDo: track memory allocation per thread
//! ToDo: calculate average latency of otlp signals

use crate::OTAP_EXPORTER_FACTORIES;
use crate::grpc::OTAPData;
use crate::perf_exporter::config::Config;
use crate::proto::opentelemetry::experimental::arrow::v1::{ArrowPayloadType, BatchArrowRecords};
use async_trait::async_trait;
use byte_unit::Byte;
use fluke_hpack::Decoder;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::config::ExporterConfig;
use otap_df_engine::control::ControlMsg;
use otap_df_engine::error::Error;
use otap_df_engine::exporter::ExporterWrapper;
use otap_df_engine::local::exporter as local;
use otap_df_engine::message::{Message, MessageChannel};
use otap_df_engine::{ExporterFactory, distributed_slice};
use serde_json::Value;
use std::fs::OpenOptions;
use std::io::Write;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};
use sysinfo::{
    CpuRefreshKind, DiskUsage, NetworkData, Networks, Process, ProcessRefreshKind, RefreshKind,
    System, get_current_pid,
};
use tokio::time::{Duration, Instant};

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
pub static PERF_EXPORTER: ExporterFactory<OTAPData> = ExporterFactory {
    name: OTAP_PERF_EXPORTER_URN,
    create: |node_config: Rc<NodeUserConfig>, exporter_config: &ExporterConfig| {
        ExporterWrapper::local(
            PerfExporter::from_config(&node_config.config),
            node_config,
            exporter_config,
        )
    },
};

impl PerfExporter {
    /// creates a perf exporter with the provided config
    #[must_use]
    pub fn new(config: Config, output: Option<String>) -> Self {
        PerfExporter { config, output }
    }

    /// Creates a new PerfExporter from a configuration object
    #[must_use]
    pub fn from_config(config: &Value) -> Self {
        PerfExporter {
            // ToDo (LQ) : Handle config errors
            config: serde_json::from_value(config.clone()).unwrap(),
            output: None,
        }
    }
}

#[async_trait(?Send)]
impl local::Exporter<OTAPData> for PerfExporter {
    async fn start(
        self: Box<Self>,
        mut msg_chan: MessageChannel<OTAPData>,
        effect_handler: local::EffectHandler<OTAPData>,
    ) -> Result<(), Error<OTAPData>> {
        // init variables for tracking
        let mut average_pipeline_latency: f64 = 0.0;
        let mut received_arrow_records_count: u64 = 0;
        let mut received_otlp_signal_count: u64 = 0;
        let mut total_received_arrow_records_count: u128 = 0;
        let mut total_received_otlp_signal_count: u128 = 0;
        let mut received_pdata_batch_count: u64 = 0;
        let mut total_received_pdata_batch_count: u64 = 0;
        let mut last_perf_time: Instant = Instant::now();
        let process_pid = get_current_pid().map_err(|e| Error::ExporterError {
            exporter: effect_handler.exporter_name(),
            error: format!("Failed to get process pid: {e}"),
        })?;
        let mut system = System::new();

        let mut writer = get_writer(self.output);

        let sys_refresh_list = sys_refresh_list(&self.config);

        // Helper function to generate performance report
        let generate_report = |received_arrow_records_count: u64,
                               received_otlp_signal_count: u64,
                               received_pdata_batch_count: u64,
                               total_received_arrow_records_count: u128,
                               total_received_otlp_signal_count: u128,
                               total_received_pdata_batch_count: u64,
                               average_pipeline_latency: f64,
                               last_perf_time: Instant,
                               system: &mut System,
                               config: &Config,
                               writer: &mut Box<dyn Write>,
                               process_pid: sysinfo::Pid|
         -> Result<(), Error<OTAPData>> {
            let now = Instant::now();
            let duration = now - last_perf_time;

            // display pipeline report
            _ = writeln!(
                writer,
                "====================Pipeline Report===================="
            );
            display_report_pipeline(
                received_arrow_records_count,
                received_otlp_signal_count,
                received_pdata_batch_count,
                total_received_arrow_records_count,
                total_received_otlp_signal_count,
                total_received_pdata_batch_count,
                average_pipeline_latency,
                duration,
                writer,
            );

            // update sysinfo data
            system.refresh_specifics(sys_refresh_list);
            let process = system.process(process_pid).ok_or(Error::ExporterError {
                exporter: effect_handler.exporter_name(),
                error: "Failed to get process".to_owned(),
            })?;

            // check configuration and display data accordingly
            if config.mem_usage() {
                _ = writeln!(
                    writer,
                    "=====================Memory Usage======================"
                );
                display_mem_usage(process, writer);
            }
            if config.cpu_usage() {
                _ = writeln!(
                    writer,
                    "=======================Cpu Usage======================="
                );
                display_cpu_usage(process, system, writer);
            }
            if config.disk_usage() {
                _ = writeln!(
                    writer,
                    "======================Disk Usage======================="
                );
                let disk_usage = process.disk_usage();
                display_disk_usage(disk_usage, duration, writer);
            }
            if config.io_usage() {
                let networks = Networks::new_with_refreshed_list();
                _ = writeln!(
                    writer,
                    "=====================Network Usage====================="
                );
                for (interface_name, network) in &networks {
                    display_io_usage(interface_name, network, duration, writer);
                }
            }

            Ok(())
        };

        // Loop until a Shutdown event is received.
        loop {
            let msg = msg_chan.recv().await?;
            match msg {
                Message::Control(ControlMsg::TimerTick { .. }) => {
                    generate_report(
                        received_arrow_records_count,
                        received_otlp_signal_count,
                        received_pdata_batch_count,
                        total_received_arrow_records_count,
                        total_received_otlp_signal_count,
                        total_received_pdata_batch_count,
                        average_pipeline_latency,
                        last_perf_time,
                        &mut system,
                        &self.config,
                        &mut writer,
                        process_pid,
                    )?;

                    // reset received counters and update last_perf_time
                    received_arrow_records_count = 0;
                    received_otlp_signal_count = 0;
                    received_pdata_batch_count = 0;
                    last_perf_time = Instant::now();
                }
                // ToDo: Handle configuration changes
                Message::Control(ControlMsg::Config { .. }) => {}
                Message::Control(ControlMsg::Shutdown { .. }) => {
                    // Generate final performance report before shutting down
                    generate_report(
                        received_arrow_records_count,
                        received_otlp_signal_count,
                        received_pdata_batch_count,
                        total_received_arrow_records_count,
                        total_received_otlp_signal_count,
                        total_received_pdata_batch_count,
                        average_pipeline_latency,
                        last_perf_time,
                        &mut system,
                        &self.config,
                        &mut writer,
                        process_pid,
                    )?;
                    break;
                }
                Message::PData(OTAPData::ArrowMetrics(batch))
                | Message::PData(OTAPData::ArrowLogs(batch))
                | Message::PData(OTAPData::ArrowTraces(batch)) => {
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
                                exporter: effect_handler.exporter_name(),
                                error: "Failed to decode batch headers".to_owned(),
                            })?;
                    // find the timestamp header and parse it
                    // timestamp will be added in the receiver to enable pipeline latency calculation
                    let timestamp_pair = header_list.iter().find(|(name, _)| name == b"timestamp");
                    if let Some((_, value)) = timestamp_pair {
                        let timestamp =
                            decode_timestamp(value).map_err(|error| Error::ExporterError {
                                exporter: effect_handler.exporter_name(),
                                error,
                            })?;
                        let current_unix_time = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .map_err(|error| Error::ExporterError {
                                exporter: effect_handler.exporter_name(),
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
                    total_received_arrow_records_count += received_arrow_records_count as u128;
                    // increment counters for otlp signals
                    received_otlp_signal_count += calculate_otlp_signal_count(batch);
                    total_received_otlp_signal_count += received_otlp_signal_count as u128;

                    // ToDo: Remove these lines, used to debugging
                    // _ = writeln!(writer, "Arrow Records {}", received_arrow_records_count);
                    // _ = writeln!(writer, "Otlp Data Count {}", received_otlp_signal_count);
                    // _ = writeln!(writer, "Pdata Count {}", received_pdata_batch_count);
                    // _ = writeln!(
                    //     writer,
                    //     "Total Records Count {}",
                    //     total_received_arrow_records_count
                    // );
                    // _ = writeln!(
                    //     writer,
                    //     "Total Otlp Count {}",
                    //     total_received_otlp_signal_count
                    // );
                    // _ = writeln!(
                    //     writer,
                    //     "Total Pdata Count {}",
                    //     total_received_pdata_batch_count
                    // );
                    // _ = writeln!(
                    //     writer,
                    //     "Average Pipeline Latency {}",
                    //     average_pipeline_latency
                    // );
                }
                _ => {
                    return Err(Error::ExporterError {
                        exporter: effect_handler.exporter_name(),
                        error: "Unknown control message".to_owned(),
                    });
                }
            }
        }
        Ok(())
    }
}

/// determine if output goes to console or to a file
fn get_writer(output_file: Option<String>) -> Box<dyn Write> {
    match output_file {
        Some(file_name) => Box::new(
            OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(file_name)
                .expect("could not open output file"),
        ),
        None => Box::new(std::io::stdout()),
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

/// calculate number of otlp signals based on arrow payload type
fn calculate_otlp_signal_count(batch: BatchArrowRecords) -> u64 {
    batch.arrow_payloads.iter().fold(0, |acc, arrow_payload| {
        let table_size = if arrow_payload.r#type == ArrowPayloadType::Spans as i32
            || arrow_payload.r#type == ArrowPayloadType::Logs as i32
            || arrow_payload.r#type == ArrowPayloadType::UnivariateMetrics as i32
        {
            arrow_payload.record.len() as u64
        } else {
            0
        };
        acc + table_size
    })
}

/// takes in the process and system outputs cpu stats via the writer
fn display_cpu_usage(process: &Process, system: &System, writer: &mut impl Write) {
    let process_cpu_usage = process.cpu_usage(); // as %
    let global_cpu_usage = system.global_cpu_usage(); // as %
    _ = writeln!(
        writer,
        "\t- global cpu usage                  : {process_cpu_usage}% (100% is all cores)"
    );
    _ = writeln!(
        writer,
        "\t- process cpu usage                 : {global_cpu_usage}% (100% is a single core)"
    );
}

/// takes in the process outputs memory stats via the writer
fn display_mem_usage(process: &Process, writer: &mut impl Write) {
    let memory_rss = process.memory(); // as bytes
    let memory_virtual = process.virtual_memory(); // as bytes
    _ = writeln!(
        writer,
        "\t- memory rss                        : {}",
        Byte::from_bytes(memory_rss as u128).get_appropriate_unit(false)
    );
    _ = writeln!(
        writer,
        "\t- memory virtual                    : {}",
        Byte::from_bytes(memory_virtual as u128).get_appropriate_unit(false)
    );
}

/// takes in the disk usage data and outputs stats via the writer
fn display_disk_usage(disk_usage: DiskUsage, duration: Duration, writer: &mut impl Write) {
    let total_written_bytes = disk_usage.total_written_bytes;
    let total_read_bytes = disk_usage.total_read_bytes;
    let written_bytes = disk_usage.written_bytes;
    let read_bytes = disk_usage.read_bytes;
    _ = writeln!(
        writer,
        "\t- read bytes                        : {}/s",
        Byte::from_bytes((read_bytes as f64 / duration.as_secs_f64()) as u128)
            .get_appropriate_unit(false) // duration.as_secs_f64()
    );
    _ = writeln!(
        writer,
        "\t- total read bytes                  : {}",
        Byte::from_bytes(total_read_bytes as u128).get_appropriate_unit(false)
    );
    _ = writeln!(
        writer,
        "\t- written bytes                     : {}/s",
        Byte::from_bytes((written_bytes as f64 / duration.as_secs_f64()) as u128)
            .get_appropriate_unit(false)
    );
    _ = writeln!(
        writer,
        "\t- total written bytes               : {}",
        Byte::from_bytes(total_written_bytes as u128).get_appropriate_unit(false)
    );
}

/// takes in network data and outputs networks stats via the writer
fn display_io_usage(
    network_name: &str,
    network_data: &NetworkData,
    duration: Duration,
    writer: &mut impl Write,
) {
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
    _ = writeln!(writer, "Network Interface: {network_name}");
    _ = writeln!(
        writer,
        "\t- bytes read                        : {}/s",
        Byte::from_bytes((bytes_received as f64 / duration.as_secs_f64()) as u128)
            .get_appropriate_unit(false)
    );
    _ = writeln!(
        writer,
        "\t- total bytes recevied              : {}",
        Byte::from_bytes(total_bytes_received as u128).get_appropriate_unit(false)
    );
    _ = writeln!(
        writer,
        "\t- bytes transmitted                 : {}/s",
        Byte::from_bytes((bytes_transmitted as f64 / duration.as_secs_f64()) as u128)
            .get_appropriate_unit(false)
    );
    _ = writeln!(
        writer,
        "\t- total bytes transmitted           : {}",
        Byte::from_bytes(total_bytes_transmitted as u128).get_appropriate_unit(false)
    );
    _ = writeln!(
        writer,
        "\t- packets received                  : {}/s",
        Byte::from_bytes((packets_received as f64 / duration.as_secs_f64()) as u128)
            .get_appropriate_unit(false)
    );
    _ = writeln!(
        writer,
        "\t- total packets received            : {}",
        Byte::from_bytes(total_packets_received as u128).get_appropriate_unit(false)
    );
    _ = writeln!(
        writer,
        "\t- packets transmitted               : {}/s",
        Byte::from_bytes((packets_transmitted as f64 / duration.as_secs_f64()) as u128)
            .get_appropriate_unit(false)
    );
    _ = writeln!(
        writer,
        "\t- total packets transmitted         : {}",
        Byte::from_bytes(total_packets_transmitted as u128).get_appropriate_unit(false)
    );
    _ = writeln!(
        writer,
        "\t- errors on received                : {}/s",
        Byte::from_bytes((errors_on_received as f64 / duration.as_secs_f64()) as u128)
            .get_appropriate_unit(false)
    );
    _ = writeln!(
        writer,
        "\t- total errors on received          : {}",
        Byte::from_bytes(total_errors_on_received as u128).get_appropriate_unit(false)
    );
    _ = writeln!(
        writer,
        "\t- errors on transmitted             : {}/s",
        Byte::from_bytes((errors_on_transmitted as f64 / duration.as_secs_f64()) as u128)
            .get_appropriate_unit(false)
    );
    _ = writeln!(
        writer,
        "\t- total errors on transmitted       : {}",
        Byte::from_bytes(total_errors_on_transmitted as u128).get_appropriate_unit(false)
    );
}

/// accepts pipeline statistics and prints a report via the writer
fn display_report_pipeline(
    received_arrow_records_count: u64,
    received_otlp_signal_count: u64,
    received_pdata_batch_count: u64,
    total_received_arrow_records_count: u128,
    total_received_otlp_signal_count: u128,
    total_received_pdata_batch_count: u64,
    average_pipeline_latency: f64,
    duration: Duration,
    writer: &mut impl Write,
) {
    _ = writeln!(
        writer,
        "\t- arrow records throughput          : {:.2} arrow-records/s",
        (received_arrow_records_count as f64 / duration.as_secs_f64())
    );
    _ = writeln!(
        writer,
        "\t- average pipeline latency          : {:.2} s",
        average_pipeline_latency * 1000.0
    );

    _ = writeln!(
        writer,
        "\t- total arrow records received      : {total_received_arrow_records_count}"
    );
    _ = writeln!(
        writer,
        "\t- otlp signal throughput            : {:.2} otlp/s",
        (received_otlp_signal_count as f64 / duration.as_secs_f64())
    );
    _ = writeln!(
        writer,
        "\t- total otlp signal received        : {total_received_otlp_signal_count}"
    );
    _ = writeln!(
        writer,
        "\t- pdata batch throughput            : {:.2} pdata/s",
        received_pdata_batch_count as f64 / duration.as_secs_f64()
    );

    _ = writeln!(
        writer,
        "\t- total pdata batch received        : {total_received_pdata_batch_count}"
    );
}

#[cfg(test)]
mod tests {

    use crate::perf_exporter::config::Config;
    use crate::perf_exporter::exporter::{OTAP_PERF_EXPORTER_URN, PerfExporter};
    use crate::proto::opentelemetry::experimental::arrow::v1::{
        ArrowPayload, ArrowPayloadType, BatchArrowRecords,
    };
    use fluke_hpack::Encoder;
    use otap_df_engine::exporter::ExporterWrapper;
    use otap_df_engine::testing::exporter::TestContext;
    use otap_df_engine::testing::exporter::TestRuntime;
    use tokio::time::{Duration, sleep};

    use crate::grpc::OTAPData;
    use otap_df_config::node::NodeUserConfig;
    use std::fs::{File, remove_file};
    use std::io::{BufReader, prelude::*};
    use std::rc::Rc;
    use std::time::{SystemTime, UNIX_EPOCH};

    const TRACES_BATCH_ID: i64 = 0;
    const LOGS_BATCH_ID: i64 = 1;
    const METRICS_BATCH_ID: i64 = 2;
    const ROW_SIZE: usize = 10;
    const MESSAGE_LEN: usize = 5;

    pub fn create_batch_arrow_record_helper(
        batch_id: i64,
        payload_type: ArrowPayloadType,
        message_len: usize,
        row_size: usize,
    ) -> BatchArrowRecords {
        // init arrow payload vec
        let mut arrow_payloads = Vec::new();
        // create ArrowPayload objects and add to the vector
        for _ in 0..message_len {
            let arrow_payload_object = ArrowPayload {
                schema_id: "0".to_string(),
                r#type: payload_type as i32,
                record: vec![1; row_size],
            };
            arrow_payloads.push(arrow_payload_object);
        }

        // create timestamp
        // unix timestamp -> get secs and nano secs -> format string as secs:nanos -> create byte array &[u8]
        // decoding will do the opposite to get the latency
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

        let secs = timestamp.as_secs().to_string();
        let nanos = timestamp.subsec_nanos().to_string();

        // string formatted with timestamp secs : timestamp subsec_nanos
        let timestamp_string = format!("{secs}:{nanos}");
        let timestamp_bytes = timestamp_string.as_bytes();

        // convert time to
        let headers = vec![(b"timestamp" as &[u8], timestamp_bytes)];
        let mut encoder = Encoder::new();
        let encoded_headers = encoder.encode(headers);
        BatchArrowRecords {
            batch_id,
            arrow_payloads,
            headers: encoded_headers,
        }
    }

    /// Test closure that simulates a typical test scenario by sending timer ticks, config,
    /// data message, and shutdown control messages.
    ///
    fn scenario()
    -> impl FnOnce(TestContext<OTAPData>) -> std::pin::Pin<Box<dyn Future<Output = ()>>> {
        |ctx| {
            Box::pin(async move {
                // send some messages to the exporter to calculate pipeline statistics
                for _ in 0..3 {
                    let traces_batch_data = create_batch_arrow_record_helper(
                        TRACES_BATCH_ID,
                        ArrowPayloadType::Spans,
                        MESSAGE_LEN,
                        ROW_SIZE,
                    );
                    let logs_batch_data = create_batch_arrow_record_helper(
                        LOGS_BATCH_ID,
                        ArrowPayloadType::Logs,
                        MESSAGE_LEN,
                        ROW_SIZE,
                    );
                    let metrics_batch_data = create_batch_arrow_record_helper(
                        METRICS_BATCH_ID,
                        ArrowPayloadType::UnivariateMetrics,
                        MESSAGE_LEN,
                        ROW_SIZE,
                    );
                    // // Send a data message
                    ctx.send_pdata(OTAPData::ArrowTraces(traces_batch_data))
                        .await
                        .expect("Failed to send data message");
                    ctx.send_pdata(OTAPData::ArrowLogs(logs_batch_data))
                        .await
                        .expect("Failed to send data message");
                    ctx.send_pdata(OTAPData::ArrowMetrics(metrics_batch_data))
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
    ) -> impl FnOnce(TestContext<OTAPData>) -> std::pin::Pin<Box<dyn Future<Output = ()>>> {
        |_| {
            Box::pin(async move {
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
                let memory_rss_line = &lines[9];
                assert!(memory_rss_line.contains("- memory rss"),);

                //memory report contains memory virtual
                let memory_virtual_line = &lines[10];
                assert!(memory_virtual_line.contains("- memory virtual"),);

                // cpu report contains cpu usage
                let process_cpu_usage = &lines[12];
                assert!(process_cpu_usage.contains("- global cpu usage"),);

                let global_cpu_usage = &lines[13];
                assert!(global_cpu_usage.contains("- process cpu usage"),);

                // disk report contains read bytes
                let read_bytes_line = &lines[15];
                assert!(read_bytes_line.contains("- read bytes"),);

                // disk report contains total read bytes
                let total_read_bytes_line = &lines[16];
                assert!(total_read_bytes_line.contains("- total read bytes"),);

                // disk report contains written bytes
                let written_bytes_line = &lines[17];
                assert!(written_bytes_line.contains("- written bytes"),);

                // disk report contains total written bytes
                let total_written_bytes_line = &lines[18];
                assert!(total_written_bytes_line.contains("- total written bytes"));
            })
        }
    }

    #[test]
    fn test_exporter_local() {
        let test_runtime = TestRuntime::new();
        let config = Config::new(1000, 0.3, true, true, true, true, true);
        let output_file = "perf_output.txt".to_string();
        let node_config = Rc::new(NodeUserConfig::new_exporter_config(OTAP_PERF_EXPORTER_URN));
        let exporter = ExporterWrapper::local(
            PerfExporter::new(config, Some(output_file.clone())),
            node_config,
            test_runtime.config(),
        );

        test_runtime
            .set_exporter(exporter)
            .run_test(scenario())
            .run_validation(validation_procedure(output_file.clone()));

        // remove the created file, prevent accidental check in of report
        remove_file(output_file).expect("Failed to remove file");
    }
}
