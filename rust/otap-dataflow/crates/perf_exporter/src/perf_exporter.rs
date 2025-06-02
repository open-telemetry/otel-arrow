// SPDX-License-Identifier: Apache-2.0

//! Implementation of the Perf exporter node
//!
//! ToDo: Handle configuration changes
//! ToDo: Implement proper deadline function for Shutdown ctrl msg

use crate::config::Config;
use async_trait::async_trait;
use byte_unit::Byte;
use otap_df_engine::error::Error;
use otap_df_engine::local::exporter as local;
use otap_df_engine::message::{ControlMsg, Message, MessageChannel};
use otap_df_otap::proto::opentelemetry::experimental::arrow::v1::{
    ArrowPayloadType, BatchArrowRecords,
};
use self_meter::{Meter, Report, ThreadReport};
use std::borrow::Cow;
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::time::{Duration, Instant};

/// Perf Exporter that emits performance data
struct PerfExporter {
    config: Config,
    meter_enabled: bool,
    addr: SocketAddr,
    thread_map: Option<HashMap<u32, String>>,
}

impl PerfExporter {
    pub fn new(
        meter_enabled: bool,
        config: Config,
        addr: SocketAddr,
        thread_map: Option<HashMap<u32, String>>,
    ) -> Self {
        PerfExporter {
            meter_enabled,
            config,
            addr,
            thread_map,
        }
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
        let mut stream =
            TcpStream::connect(self.addr)
                .await
                .map_err(|error| Error::ExporterError {
                    exporter: effect_handler.exporter_name(),
                    error: error.to_string(),
                })?;

        // init variables for tracking
        let mut received_msg_count: usize = 0;
        let mut received_evt_count: usize = 0;
        let mut total_received_msg_count: usize = 0;
        let mut total_received_evt_count: usize = 0;
        let mut last_perf_time: Instant = Instant::now();

        let mut meter: Option<Meter> = None;

        // if meter is enabled then init meter
        if self.meter_enabled {
            let mut new_meter =
                Meter::new(Duration::from_millis(self.config.timeout())).map_err(|error| {
                    Error::ExporterError {
                        exporter: effect_handler.exporter_name(),
                        error: error.to_string(),
                    }
                })?;
            // track threads if they are provided
            if let Some(thread_map) = self.thread_map {
                for (pid, thread_name) in thread_map {
                    new_meter.track_thread(pid, thread_name.as_str());
                }
            }
            meter = Some(new_meter);
        } else {
            return Err(Error::ExporterError {
                exporter: effect_handler.exporter_name().to_owned(),
                error: "Meter enabled but no interval duration provided".to_owned(),
            });
        }
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

                    // Formatted output messages for the report
                    // generate msgs
                    let msg_throughput = format!(
                        "message throughput     : {:.2} msg/s",
                        (received_msg_count as f64 / duration.as_secs_f64())
                    );
                    // TODO HANDLE LATENCY need to get message timestamps from batch
                    let msg_latency = "message average latency: TBD".to_string();

                    let msg_received =
                        format!("total message received : {}", total_received_msg_count);
                    let evt_throughput = format!(
                        "event   throughput     : {:.2} evt/s",
                        (received_evt_count as f64 / duration.as_secs_f64())
                    );
                    let evt_received =
                        format!("total event received   : {}", total_received_evt_count);

                    display_report_default(
                        effect_handler.exporter_name(),
                        &msg_throughput,
                        &msg_latency,
                        &msg_received,
                        &evt_throughput,
                        &evt_received,
                        &mut stream,
                    )
                    .await
                    .map_err(|_| Error::ExporterError {
                        exporter: effect_handler.exporter_name(),
                        error: "Failed to provide report".to_owned(),
                    })?;

                    // generate meter reports if enabled
                    if let Some(meter) = meter.as_mut() {
                        // measure resource usage using self_meter package
                        meter.scan().map_err(|_| Error::ExporterError {
                            exporter: effect_handler.exporter_name(),
                            error: "Failed to scan resource usage".to_owned(),
                        })?;
                        // get report of resource usage
                        let meter_report = meter.report();

                        // if meter is enabled report it
                        if let Some(meter_report) = meter_report {
                            display_report_meter(&self.config, meter_report, &mut stream)
                                .await
                                .map_err(|_| Error::ExporterError {
                                    exporter: effect_handler.exporter_name(),
                                    error: "Failed to provide meter report".to_owned(),
                                })?;
                        }
                        // get thread report
                        let thread_report_iter = meter.thread_report();
                        // if threads are being scanned then display the report of each thread
                        if let Some(thread_report_iter) = thread_report_iter {
                            for thread_report in thread_report_iter {
                                display_report_thread(&self.config, thread_report, &mut stream)
                                    .await
                                    .map_err(|_| Error::ExporterError {
                                        exporter: effect_handler.exporter_name(),
                                        error: "Failed to provide thread report".to_owned(),
                                    })?;
                            }
                        }
                    }

                    // reset received counters
                    // update last_perf_time
                    received_msg_count = 0;
                    received_evt_count = 0;
                    last_perf_time = now;
                }
                Message::Control(ControlMsg::Config { .. }) => {}
                Message::Control(ControlMsg::Shutdown { .. }) => {
                    break;
                }
                Message::PData(batch) => {
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
                        exporter: effect_handler.exporter_name().to_owned(),
                        error: "Unknown control message".to_owned(),
                    });
                }
            }
        }
        Ok(())
    }
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

async fn display_report_meter(
    config: &Config,
    report: Report,
    stream: &mut TcpStream,
) -> Result<(), ()> {
    if config.cpu_usage() {
        // print all cpu usage data
        let global_cpu_usage = format!(
            "\t- global cpu usage       : {}% (100% is all cores)",
            report.global_cpu_usage
        );
        let process_cpu_usage = format!(
            "\t- process cpu usage      : {}% (100% is a single core)",
            report.process_cpu_usage
        );
        let gross_cpu_usage = format!(
            "\t- gross cpu usage  : {}% (100% is a single core)",
            report.gross_cpu_usage
        );
        stream
            .write_all(global_cpu_usage.as_bytes())
            .await
            .map_err(|_| ())?;
        stream
            .write_all(process_cpu_usage.as_bytes())
            .await
            .map_err(|_| ())?;
        stream
            .write_all(gross_cpu_usage.as_bytes())
            .await
            .map_err(|_| ())?;
    }
    if config.mem_usage() {
        // print all memory usage data
        let memory_rss = format!(
            "\t- memory rss             : {}",
            Byte::from_bytes(report.memory_rss as u128).get_appropriate_unit(false)
        );
        let memory_virtual = format!(
            "\t- memory virtual         : {}",
            Byte::from_bytes(report.memory_virtual as u128).get_appropriate_unit(false)
        );
        let memory_swap = format!(
            "\t- memory swap            : {}",
            Byte::from_bytes(report.memory_swap as u128).get_appropriate_unit(false)
        );
        let memory_rss_peak = format!(
            "\t- memory rss peak        : {}",
            Byte::from_bytes(report.memory_rss_peak as u128).get_appropriate_unit(false)
        );
        let memory_virtual_peak = format!(
            "\t- memory virtual peak    : {}",
            Byte::from_bytes(report.memory_virtual_peak as u128).get_appropriate_unit(false)
        );
        let memory_swap_peak = format!(
            "\t- memory swap peak       : {}",
            Byte::from_bytes(report.memory_swap_peak as u128).get_appropriate_unit(false)
        );
        stream
            .write_all(memory_rss.as_bytes())
            .await
            .map_err(|_| ())?;
        stream
            .write_all(memory_virtual.as_bytes())
            .await
            .map_err(|_| ())?;
        stream
            .write_all(memory_swap.as_bytes())
            .await
            .map_err(|_| ())?;
        stream
            .write_all(memory_rss_peak.as_bytes())
            .await
            .map_err(|_| ())?;
        stream
            .write_all(memory_virtual_peak.as_bytes())
            .await
            .map_err(|_| ())?;
        stream
            .write_all(memory_swap_peak.as_bytes())
            .await
            .map_err(|_| ())?;
    }
    if config.disk_usage() {
        let disk_read = format!(
            "\t- disk read              : {}/s",
            Byte::from_bytes(report.disk_read as u128).get_appropriate_unit(false)
        );
        let disk_write = format!(
            "\t- disk write_all             : {}/s",
            Byte::from_bytes(report.disk_write as u128).get_appropriate_unit(false)
        );
        let disk_cancelled = format!(
            "\t- disk cancelled         : {}/s",
            Byte::from_bytes(report.disk_cancelled as u128).get_appropriate_unit(false)
        );
        stream
            .write_all(disk_read.as_bytes())
            .await
            .map_err(|_| ())?;
        stream
            .write_all(disk_write.as_bytes())
            .await
            .map_err(|_| ())?;
        stream
            .write_all(disk_cancelled.as_bytes())
            .await
            .map_err(|_| ())?;
    }
    if config.io_usage() {
        let io_read = format!(
            "\t- io read                : {}/s",
            Byte::from_bytes(report.io_read as u128).get_appropriate_unit(false)
        );
        let io_write = format!(
            "\t- io write_all               : {}/s",
            Byte::from_bytes(report.io_write as u128).get_appropriate_unit(false)
        );
        let io_read_ops = format!("\t- io read ops            : {} read/s", report.io_read_ops);
        let io_write_ops = format!(
            "\t- io write_all ops           : {} write_all/s",
            report.io_write_ops
        );
        stream.write_all(io_read.as_bytes()).await.map_err(|_| ())?;
        stream
            .write_all(io_write.as_bytes())
            .await
            .map_err(|_| ())?;
        stream
            .write_all(io_read_ops.as_bytes())
            .await
            .map_err(|_| ())?;
        stream
            .write_all(io_write_ops.as_bytes())
            .await
            .map_err(|_| ())?;
    }
    Ok(())
}

async fn display_report_thread(
    config: &Config,
    thread_report: (&str, ThreadReport),
    stream: &mut TcpStream,
) -> Result<(), ()> {
    let thread_name = thread_report.0;
    let report = thread_report.1;
    if config.cpu_usage() {
        // print all cpu usage data
        let thread_cpu_usage = format!(
            "\t- thread {}'s cpu usage       : {}% (100% is single cores)",
            thread_name, report.cpu_usage
        );
        let thread_system_cpu = format!(
            "\t- thread {}'s cpu usage in kernel space : {}% (100% is a single core)",
            thread_name, report.system_cpu
        );
        let thread_user_cpu = format!(
            "\t- thread {}'s cpu usage in user space  : {}% (100% is a single core)",
            thread_name, report.user_cpu
        );
        stream
            .write_all(thread_cpu_usage.as_bytes())
            .await
            .map_err(|_| ())?;
        stream
            .write_all(thread_system_cpu.as_bytes())
            .await
            .map_err(|_| ())?;
        stream
            .write_all(thread_user_cpu.as_bytes())
            .await
            .map_err(|_| ())?;
    }
    Ok(())
}

async fn display_report_default(
    name: Cow<'static, str>,
    msg_throughput: &str,
    msg_latency: &str,
    msg_received: &str,
    evt_throughput: &str,
    evt_received: &str,
    stream: &mut TcpStream,
) -> Result<(), ()> {
    let report = format!(
        "{}\n\t- {}\n\t- {}\n\t- {}\n\t- {}\n\t- {}",
        name, msg_throughput, msg_latency, msg_received, evt_throughput, evt_received
    );
    stream.write_all(report.as_bytes()).await.map_err(|_| ())?;
    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::config::Config;
    use crate::perf_exporter::PerfExporter;
    use otap_df_engine::exporter::ExporterWrapper;
    use otap_df_engine::testing::exporter::TestContext;
    use otap_df_engine::testing::exporter::TestRuntime;
    use otap_df_otap::proto::opentelemetry::experimental::arrow::v1::{
        ArrowPayload, ArrowPayloadType, BatchArrowRecords,
    };
    use self_meter::Meter;
    use std::net::SocketAddr;
    use tokio::io::AsyncReadExt;
    use tokio::net::TcpListener;
    use tokio::runtime::Runtime;
    use tokio::time::{Duration, timeout};

    const TRACES_BATCH_ID: i64 = 0;
    const LOGS_BATCH_ID: i64 = 1;
    const METRICS_BATCH_ID: i64 = 2;

    pub fn create_batch_arrow_record_helper(
        batch_id: i64,
        payload_type: ArrowPayloadType,
    ) -> BatchArrowRecords {
        let arrow_payload = ArrowPayload {
            schema_id: "0".to_string(),
            r#type: payload_type as i32,
            record: vec![0],
        };
        BatchArrowRecords {
            batch_id: batch_id,
            arrow_payloads: vec![arrow_payload],
            headers: vec![0],
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
                let traces_batch_data =
                    create_batch_arrow_record_helper(TRACES_BATCH_ID, ArrowPayloadType::Spans);
                let logs_batch_data =
                    create_batch_arrow_record_helper(LOGS_BATCH_ID, ArrowPayloadType::Logs);
                let metrics_batch_data = create_batch_arrow_record_helper(
                    METRICS_BATCH_ID,
                    ArrowPayloadType::UnivariateMetrics,
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

                // send timer_tick to trigger report
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
        mut receiver: tokio::sync::mpsc::Receiver<String>,
    ) -> impl FnOnce(TestContext<BatchArrowRecords>) -> std::pin::Pin<Box<dyn Future<Output = ()>>>
    {
        |_| {
            Box::pin(async move {
                let report_received = timeout(Duration::from_secs(3), receiver.recv())
                    .await
                    .expect("Timed out waiting for message")
                    .expect("No message received");

                // Assert that the message received is what the exporter sen
                let expected_message = "test".to_string();
                assert!(matches!(report_received, expected_message));
            })
        }
    }

    #[test]
    fn test_exporter_local() {
        let test_runtime = TestRuntime::new();
        let (sender, receiver) = tokio::sync::mpsc::channel(64);
        let config = Config::default();
        let addr = "127.0.0.1";
        let port = portpicker::pick_unused_port().expect("No free ports");
        let listening_addr: SocketAddr = format!("{addr}:{port}").parse().unwrap();
        let exporter = ExporterWrapper::local(
            PerfExporter::new(true, config, listening_addr, None),
            test_runtime.config(),
        );
        // tokio runtime to run tcp server in the background
        let tokio_rt = Runtime::new().unwrap();

        _ = tokio_rt.spawn(async move {
            let listener = TcpListener::bind(listening_addr).await.unwrap();
            loop {
                let (mut tcp_stream, _) = listener.accept().await.unwrap();
                let mut buf = vec![0u8; 1024];
                tcp_stream.read_exact(&mut buf).await;
                // send received data to channel to verify in the validation stage
                let result = String::from_utf8(buf).unwrap();
                sender.send(result).await.unwrap();
            }
        });

        test_runtime
            .set_exporter(exporter)
            .run_test(scenario())
            .run_validation(validation_procedure(receiver));
    }
}
