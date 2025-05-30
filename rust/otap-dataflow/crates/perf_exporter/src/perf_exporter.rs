// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTLP exporter node
//!
//! ToDo: Handle Ack and Nack messages in the pipeline
//! ToDo: Handle configuratin changes
//! ToDo: Implement proper deadline function for Shutdown ctrl msg

use async_trait::async_trait;
use byte_unit::Byte;
use otap_df_engine::error::Error;
use otap_df_engine::local::exporter as local;
use otap_df_engine::message::{ControlMsg, Message, MessageChannel};
use otap_df_otap::proto::opentelemetry::experimental::arrow::v1::{
    ArrowPayloadType, BatchArrowRecords,
};
use self_meter::{Meter, Report, ThreadReport};
use serde::Deserialize;
use std::borrow::Cow;
use std::net::SocketAddr;
use std::time::Instant;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

#[derive(Debug, Deserialize)]
pub struct Config {
    /// Time duration after which a perf trace is displayed (default = 1000ms).
    #[serde(default = "default_timeout")]
    timeout: u64,

    #[serde(default = "default_self_usage")]
    self_usage: bool,

    #[serde(default = "default_cpu_usage")]
    cpu_usage: bool,

    #[serde(default = "default_mem_usage")]
    mem_usage: bool,

    #[serde(default = "default_disk_usage")]
    disk_usage: bool,

    #[serde(default = "default_io_usage")]
    io_usage: bool,
}

fn default_timeout() -> u64 {
    1000
}

fn default_self_usage() -> bool {
    true
}

fn default_cpu_usage() -> bool {
    true
}

fn default_mem_usage() -> bool {
    true
}
fn default_disk_usage() -> bool {
    true
}
fn default_io_usage() -> bool {
    true
}

struct PerfExporter {
    config: Config,
    received_msg_count: usize,
    received_evt_count: usize,
    total_received_msg_count: usize,
    total_received_evt_count: usize,

    last_perf_time: Instant,

    meter: Option<Meter>,
    addr: SocketAddr,
}

impl PerfExporter {
    pub fn new(meter: Option<Meter>, config: Config, addr: SocketAddr) -> Self {
        PerfExporter {
            received_msg_count: 0,
            received_evt_count: 0,
            total_received_msg_count: 0,
            total_received_evt_count: 0,
            last_perf_time: Instant::now(),
            meter: meter,
            config: config,
            addr: addr,
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
        // Loop until a Shutdown event is received.
        let mut stream =
            TcpStream::connect(self.addr)
                .await
                .map_err(|error| Error::ExporterError {
                    exporter: effect_handler.exporter_name(),
                    error: error.to_string(),
                })?;
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
                    let duration = now - self.last_perf_time;

                    // Formatted output messages for the report
                    // generate msgs
                    let msg_throughput = format!(
                        "message throughput     : {:.2} msg/s",
                        (self.received_msg_count as f64 / duration.as_secs_f64())
                    );
                    // TODO HANDLE LATENCY need to get message timestamps from batch
                    let msg_latency = "message average latency: TBD".to_string();

                    let msg_received =
                        format!("total message received : {}", self.total_received_msg_count);
                    let evt_throughput = format!(
                        "event   throughput     : {:.2} evt/s",
                        (self.received_evt_count as f64 / duration.as_secs_f64())
                    );
                    let evt_received =
                        format!("total event received   : {}", self.total_received_evt_count);

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
                    if let Some(meter) = self.meter.as_mut() {
                        // measure resource usage using self_meter package
                        meter.scan().map_err(|error| Error::ExporterError {
                            exporter: effect_handler.exporter_name(),
                            error: error.to_string(),
                        });
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
                            thread_report_iter.map(|thread_report| async move {
                                display_report_thread(&self.config, thread_report, &mut stream)
                                    .await
                                    .map_err(|_| Error::ExporterError {
                                        exporter: effect_handler.exporter_name(),
                                        error: "Failed to provide thread report".to_owned(),
                                    })?;
                            });
                        }
                    }

                    // reset received counters
                    // update last_perf_time
                    self.received_msg_count = 0;
                    self.received_evt_count = 0;
                    self.last_perf_time = now;
                }
                Message::Control(ControlMsg::Config { .. }) => {}
                Message::Control(ControlMsg::Shutdown { .. }) => {
                    break;
                }
                Message::PData(batch) => {
                    // increment counters for received messages
                    // increment by number of arrowpayloads
                    self.received_msg_count += batch.arrow_payloads.len();
                    self.total_received_msg_count += self.received_msg_count;
                    // increment counters for received events
                    self.received_evt_count += calculate_event_count(batch);
                    self.total_received_evt_count += self.received_evt_count;
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
    if config.cpu_usage {
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
    if config.mem_usage {
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
    if config.disk_usage {
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
    if config.io_usage {
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
    if config.cpu_usage {
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

    use crate::perf_exporter::PerfExporter;
    use otap_df_engine::exporter::ExporterWrapper;
    use otap_df_engine::testing::exporter::TestContext;
    use otap_df_engine::testing::exporter::TestRuntime;
    use self_meter::Meter;
    use std::net::SocketAddr;
    use tokio::runtime::Runtime;
    use tokio::time::{Duration, timeout};

    /// Test closure that simulates a typical test scenario by sending timer ticks, config,
    /// data message, and shutdown control messages.
    fn scenario(
        addr: SocketAddr,
    ) -> impl FnOnce(TestContext<BatchArrowRecords>) -> std::pin::Pin<Box<dyn Future<Output = ()>>>
    {
        |ctx| {
            Box::pin(async move {
                // Send 3 TimerTick events. with a couple messages between them
                // let stream = TcpStream::connect(addr).await.unwrap()?;
                for _ in 0..3 {
                    ctx.send_timer_tick()
                        .await
                        .expect("Failed to send TimerTick");
                    let batch_data = create_batch_data();
                    // Send a data message
                    ctx.send_pdata(batch_data)
                        .await
                        .expect("Failed to send data message");

                    // Send a data message
                    ctx.sleep(Duration::from_millis(50)).await;
                }

                // Send shutdown
                ctx.send_shutdown(Duration::from_millis(200), "test complete")
                    .await
                    .expect("Failed to send Shutdown");
            })
        }
    }

    /// Validation closure that checks the expected counter values
    fn validation_procedure()
    -> impl FnOnce(TestContext<BatchArrowRecords>) -> std::pin::Pin<Box<dyn Future<Output = ()>>>
    {
        |ctx| {
            Box::pin(async move {
                // ctx.counters().assert(
                //     3, // timer tick
                //     1, // message
                //     1, // config
                //     1, // shutdown
                // );
            })
        }
    }

    #[test]
    fn test_exporter_local() {
        let test_runtime = TestRuntime::new();
        let meter = Meter::new().unwrap();
        meter.track_current_thread("main");
        let config = Config::default();
        let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
        let exporter = ExporterWrapper::local(
            PerfExporter::new(meter, config, addr),
            test_runtime.config(),
        );

        test_runtime
            .set_exporter(exporter)
            .run_test(scenario(addr))
            .run_validation(validation_procedure());
    }
}
