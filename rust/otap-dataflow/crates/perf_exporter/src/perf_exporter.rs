// SPDX-License-Identifier: Apache-2.0

//! Implementation of the Perf exporter node
//!
//! ToDo: Handle configuration changes
//! ToDo: Implement proper deadline function for Shutdown ctrl msg

use crate::config::Config;
use async_trait::async_trait;
use byte_unit::Byte;
use fluke_hpack::Decoder;
use otap_df_engine::error::Error;
use otap_df_engine::local::exporter as local;
use otap_df_engine::message::{ControlMsg, Message, MessageChannel};
use otap_df_otap::proto::opentelemetry::experimental::arrow::v1::{
    ArrowPayloadType, BatchArrowRecords,
};
use std::borrow::Cow;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use sysinfo::{DiskUsage, NetworkData, Networks, Process, System, get_current_pid};
use tokio::time::{Duration, Instant};

/// Perf Exporter that emits performance data
struct PerfExporter {
    config: Config,
    output: Option<String>,
}

impl PerfExporter {
    pub fn new(config: Config, output: Option<String>) -> Self {
        PerfExporter { config, output }
    }
}

#[async_trait(?Send)]
impl local::Exporter<BatchArrowRecords> for PerfExporter {
    async fn start(
        self: Box<Self>,
        mut msg_chan: MessageChannel<BatchArrowRecords>,
        effect_handler: local::EffectHandler<BatchArrowRecords>,
    ) -> Result<(), Error<BatchArrowRecords>> {
        // create a tcp stream for sending data

        // init variables for tracking
        //TODO: Add msg latency tracking
        let mut average_pipeline_latency: f64 = 0.0;
        let mut received_msg_count: usize = 0;
        let mut received_evt_count: usize = 0;
        let mut total_received_msg_count: usize = 0;
        let mut total_received_evt_count: usize = 0;
        let mut received_pdata_count: usize = 0;
        let mut total_received_pdata_count: usize = 0;
        let mut last_perf_time: Instant = Instant::now();
        let process_pid = get_current_pid().map_err(|e| Error::ExporterError {
            exporter: effect_handler.exporter_name(),
            error: format!("Failed to get process pid: {}", e),
        })?;
        let mut system = System::new();
        let mut networks = Networks::new();

        let mut writer = get_writer(self.output);

        // Loop until a Shutdown event is received.
        loop {
            match msg_chan.recv().await? {
                Message::Control(ControlMsg::TimerTick { .. }) => {
                    /*
                    Calculates the time elapsed since the last report
                    Computes message and event throughput (items per second)
                    Formats performance statistics
                    If the meter is enabled, collects system resource usage
                    Prints all metrics to the console
                    Resets counters for the next interval
                    */
                    // get delta time
                    let now = Instant::now();
                    let duration = now - last_perf_time;

                    // display pipeline report
                    _ = writeln!(writer, "---Pipeline Report--");
                    display_report_pipeline(
                        effect_handler.exporter_name(),
                        received_msg_count,
                        received_evt_count,
                        received_pdata_count,
                        total_received_msg_count,
                        total_received_evt_count,
                        total_received_pdata_count,
                        average_pipeline_latency,
                        duration,
                        &mut writer,
                    );

                    system = System::new_all();
                    networks = Networks::new_with_refreshed_list();
                    let process = system.process(process_pid).ok_or(Error::ExporterError {
                        exporter: effect_handler.exporter_name(),
                        error: "Failed to get process".to_owned(),
                    })?;

                    // check configuration and display data accordingly
                    if self.config.mem_usage() {
                        // get process
                        _ = writeln!(writer, "---Memory Usage---");
                        display_mem_usage(process, &mut writer);
                    }
                    if self.config.cpu_usage() {
                        _ = writeln!(writer, "---Cpu Usage---");
                        display_cpu_usage(process, &system, &mut writer);
                    }
                    if self.config.disk_usage() {
                        _ = writeln!(writer, "---Disk Usage---");
                        let disk_usage = process.disk_usage();
                        display_disk_usage(disk_usage, duration, &mut writer);
                    }
                    if self.config.io_usage() {
                        _ = writeln!(writer, "---Network Usage---");
                        for (interface_name, network) in &networks {
                            display_io_usage(interface_name, network, duration, &mut writer);
                        }
                    }

                    // reset received counters
                    // update last_perf_time
                    received_msg_count = 0;
                    received_evt_count = 0;
                    received_pdata_count = 0;
                    last_perf_time = now;
                }
                Message::Control(ControlMsg::Config { .. }) => {}
                Message::Control(ControlMsg::Shutdown { .. }) => {
                    break;
                }
                Message::PData(batch) => {
                    //TODO: check headers for timestamp
                    //get time delta between now and timestamp
                    // calculate average
                    // decode the headers which are hpack encoded
                    received_pdata_count += 1;
                    total_received_pdata_count += 1;
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
                            decode_timestamp(value).map_err(|_| Error::ExporterError {
                                exporter: effect_handler.exporter_name(),
                                error: "Failed to decode timestamp".to_owned(),
                            })?;
                        let unix_time =
                            SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .map_err(|error| Error::ExporterError {
                                    exporter: effect_handler.exporter_name(),
                                    error: error.to_string(),
                                })?;
                        let latency = (unix_time - timestamp).as_secs_f64();
                        average_pipeline_latency = update_average(
                            latency,
                            average_pipeline_latency,
                            total_received_pdata_count as f64,
                        );
                    }

                    // increment counters for received messages
                    // increment by number of arrowpayloads
                    received_msg_count += batch.arrow_payloads.len();
                    total_received_msg_count += received_msg_count;
                    // increment counters for received events
                    received_evt_count += calculate_event_count(batch);
                    total_received_evt_count += received_evt_count;
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

fn update_average(new_value: f64, old_average: f64, total: f64) -> f64 {
    (old_average * (total - 1.0) + new_value) / total
}

fn decode_timestamp(timestamp: &[u8]) -> Result<Duration, ()> {
    let timestamp_string = std::str::from_utf8(timestamp).map_err(|_| ())?;
    let timestamp_parts: Vec<&str> = timestamp_string.split(":").collect();
    let secs = timestamp_parts[0].parse::<u64>().map_err(|_| ())?;
    let nanosecs = timestamp_parts[1].parse::<u32>().map_err(|_| ())?;

    Ok(Duration::new(secs, nanosecs))
}

fn calculate_event_count(batch: BatchArrowRecords) -> usize {
    batch.arrow_payloads.iter().fold(0, |acc, arrow_payload| {
        let table_size = if arrow_payload.r#type == ArrowPayloadType::Spans as i32
            || arrow_payload.r#type == ArrowPayloadType::Logs as i32
            || arrow_payload.r#type == ArrowPayloadType::UnivariateMetrics as i32
        {
            arrow_payload.record.len()
        } else {
            0
        };
        acc + table_size
    })
}

fn display_cpu_usage(process: &Process, system: &System, writer: &mut impl Write) {
    let process_cpu_usage = process.cpu_usage(); // as %
    let global_cpu_usage = system.global_cpu_usage(); // as %
    _ = writeln!(
        writer,
        "\t- global cpu usage            : {}% (100% is all cores)",
        process_cpu_usage
    );
    _ = writeln!(
        writer,
        "\t- process cpu usage           : {}% (100% is a single core)",
        global_cpu_usage
    );
}

fn display_mem_usage(process: &Process, writer: &mut impl Write) {
    let memory_rss = process.memory(); // as bytes
    let memory_virtual = process.virtual_memory(); // as bytes
    _ = writeln!(
        writer,
        "\t- memory rss                  : {}",
        Byte::from_bytes(memory_rss as u128).get_appropriate_unit(false)
    );
    _ = writeln!(
        writer,
        "\t- memory virtual              : {}",
        Byte::from_bytes(memory_virtual as u128).get_appropriate_unit(false)
    );
}

fn display_disk_usage(disk_usage: DiskUsage, duration: Duration, writer: &mut impl Write) {
    let total_written_bytes = disk_usage.total_written_bytes;
    let total_read_bytes = disk_usage.total_read_bytes;
    let written_bytes = disk_usage.written_bytes;
    let read_bytes = disk_usage.read_bytes;
    _ = writeln!(
        writer,
        "\t- read bytes                  : {}/s",
        Byte::from_bytes((read_bytes as f64 / duration.as_secs_f64()) as u128)
            .get_appropriate_unit(false) // duration.as_secs_f64()
    );
    _ = writeln!(
        writer,
        "\t- total read bytes            : {}",
        Byte::from_bytes(total_read_bytes as u128).get_appropriate_unit(false)
    );
    _ = writeln!(
        writer,
        "\t- written bytes               : {}/s",
        Byte::from_bytes((written_bytes as f64 / duration.as_secs_f64()) as u128)
            .get_appropriate_unit(false)
    );
    _ = writeln!(
        writer,
        "\t- total written bytes         : {}",
        Byte::from_bytes(total_written_bytes as u128).get_appropriate_unit(false)
    );
}

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
    _ = writeln!(writer, "Network {}", network_name);
    _ = writeln!(
        writer,
        "\t- bytes read                  : {}/s",
        Byte::from_bytes((bytes_received as f64 / duration.as_secs_f64()) as u128)
            .get_appropriate_unit(false)
    );
    _ = writeln!(
        writer,
        "\t- total bytes recevied        : {}",
        Byte::from_bytes(total_bytes_received as u128).get_appropriate_unit(false)
    );
    _ = writeln!(
        writer,
        "\t- bytes transmitted           : {}/s",
        Byte::from_bytes((bytes_transmitted as f64 / duration.as_secs_f64()) as u128)
            .get_appropriate_unit(false)
    );
    _ = writeln!(
        writer,
        "\t- total bytes transmitted     : {}",
        Byte::from_bytes(total_bytes_transmitted as u128).get_appropriate_unit(false)
    );
    _ = writeln!(
        writer,
        "\t- packets received            : {}/s",
        Byte::from_bytes((packets_received as f64 / duration.as_secs_f64()) as u128)
            .get_appropriate_unit(false)
    );
    _ = writeln!(
        writer,
        "\t- total packets received      : {}",
        Byte::from_bytes(total_packets_received as u128).get_appropriate_unit(false)
    );
    _ = writeln!(
        writer,
        "\t- packets transmitted         : {}/s",
        Byte::from_bytes((packets_transmitted as f64 / duration.as_secs_f64()) as u128)
            .get_appropriate_unit(false)
    );
    _ = writeln!(
        writer,
        "\t- total packets transmitted   : {}",
        Byte::from_bytes(total_packets_transmitted as u128).get_appropriate_unit(false)
    );
    _ = writeln!(
        writer,
        "\t- errors on received          : {}/s",
        Byte::from_bytes((errors_on_received as f64 / duration.as_secs_f64()) as u128)
            .get_appropriate_unit(false)
    );
    _ = writeln!(
        writer,
        "\t- total errors on received    : {}",
        Byte::from_bytes(total_errors_on_received as u128).get_appropriate_unit(false)
    );
    _ = writeln!(
        writer,
        "\t- errors on transmitted       : {}/s",
        Byte::from_bytes((errors_on_transmitted as f64 / duration.as_secs_f64()) as u128)
            .get_appropriate_unit(false)
    );
    _ = writeln!(
        writer,
        "\t- total errors on transmitted : {}",
        Byte::from_bytes(total_errors_on_transmitted as u128).get_appropriate_unit(false)
    );
}

fn display_report_pipeline(
    name: Cow<'static, str>,
    received_msg_count: usize,
    received_evt_count: usize,
    received_pdata_count: usize,
    total_received_msg_count: usize,
    total_received_evt_count: usize,
    total_received_pdata_count: usize,
    average_pipeline_latency: f64,
    duration: Duration,
    writer: &mut impl Write,
) {
    _ = writeln!(writer, "Performance report for {}:", name);
    _ = writeln!(
        writer,
        "\t- message throughput          : {:.2} msg/s",
        (received_msg_count as f64 / duration.as_secs_f64())
    );
    _ = writeln!(
        writer,
        "\t- average pipeline latency    : {:.2} s",
        average_pipeline_latency * 1000.0
    );

    _ = writeln!(
        writer,
        "\t- total message received      : {}",
        total_received_msg_count
    );
    _ = writeln!(
        writer,
        "\t- event throughput            : {:.2} evt/s",
        (received_evt_count as f64 / duration.as_secs_f64())
    );
    _ = writeln!(
        writer,
        "\t- total event received        : {}",
        total_received_evt_count
    );
    _ = writeln!(
        writer,
        "\t- pdata throughput            : {:.2} pdata/s",
        received_pdata_count as f64 / duration.as_secs_f64()
    );

    _ = writeln!(
        writer,
        "\t- total pdata received        : {}",
        total_received_pdata_count
    );
}

#[cfg(test)]
mod tests {

    use crate::config::Config;
    use crate::perf_exporter::PerfExporter;
    use fluke_hpack::Encoder;
    use otap_df_engine::exporter::ExporterWrapper;
    use otap_df_engine::testing::exporter::TestContext;
    use otap_df_engine::testing::exporter::TestRuntime;
    use otap_df_otap::proto::opentelemetry::experimental::arrow::v1::{
        ArrowPayload, ArrowPayloadType, BatchArrowRecords,
    };
    use tokio::time::{Duration, sleep, timeout};

    use std::fs::{File, remove_file};
    use std::io::{self, BufReader, prelude::*};
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
        // unix timestamp -> number -> byte array &[u8]
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

        let secs = timestamp.as_secs().to_string();
        let nanos = timestamp.subsec_nanos().to_string();

        // string formatted with timestamp secs : timestamp subsec_nanos
        let timestamp_string = format!("{}:{}", secs, nanos);
        let timestamp_bytes = timestamp_string.as_bytes();

        // convert time to
        let headers = vec![(b"timestamp" as &[u8], timestamp_bytes)];
        let mut encoder = Encoder::new();
        let encoded_headers = encoder.encode(headers);
        BatchArrowRecords {
            batch_id: batch_id,
            arrow_payloads: arrow_payloads,
            headers: encoded_headers,
        }
    }

    /// Test closure that simulates a typical test scenario by sending timer ticks, config,
    /// data message, and shutdown control messages.
    ///
    fn scenario()
    -> impl FnOnce(TestContext<BatchArrowRecords>) -> std::pin::Pin<Box<dyn Future<Output = ()>>>
    {
        |ctx| {
            Box::pin(async move {
                // Send 3 TimerTick events. with a couple messages between them

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
                    // Send a data message
                    ctx.send_pdata(traces_batch_data)
                        .await
                        .expect("Failed to send data message");
                    ctx.send_pdata(logs_batch_data)
                        .await
                        .expect("Failed to send data message");
                    ctx.send_pdata(metrics_batch_data)
                        .await
                        .expect("Failed to send data message");
                }

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
    ) -> impl FnOnce(TestContext<BatchArrowRecords>) -> std::pin::Pin<Box<dyn Future<Output = ()>>>
    {
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
                let message_throughput_line = &lines[2];
                assert!(message_throughput_line.contains("- message throughput"),);

                // report contains pipeline latency
                let avg_pipeline_latency_line = &lines[3];
                assert!(avg_pipeline_latency_line.contains("- average pipeline latency"));

                let total_msg_line = &lines[4];
                assert!(total_msg_line.contains("- total message received"),);

                let event_throughput_line = &lines[5];
                assert!(event_throughput_line.contains("- event throughput"),);

                let total_evt_line = &lines[6];
                assert!(total_evt_line.contains("- total event received"),);

                let pdata_throughput_line = &lines[7];
                assert!(pdata_throughput_line.contains("- pdata throughput"),);

                let total_pdata_line = &lines[8];
                assert!(total_pdata_line.contains("- total pdata received"),);

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
        let config = Config::new(1000, true, true, true, true, false);
        let output_file = "perf_output.txt".to_string();
        let exporter = ExporterWrapper::local(
            PerfExporter::new(config, Some(output_file.clone())),
            test_runtime.config(),
        );
        // tokio runtime to run tcp server in the background

        test_runtime
            .set_exporter(exporter)
            .run_test(scenario())
            .run_validation(validation_procedure(output_file.clone()));

        remove_file(output_file).expect("Failed to remove file");
    }
}
