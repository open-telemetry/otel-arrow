// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTLP exporter node
//!
//! ToDo: Handle Ack and Nack messages in the pipeline
//! ToDo: Handle configuratin changes
//! ToDo: Implement proper deadline function for Shutdown ctrl msg

use async_trait::async_trait;
use otap_df_engine::error::Error;
use otap_df_engine::local::exporter as local;
use otap_df_engine::message::{ControlMsg, Message, MessageChannel};
use otap_df_otlp::grpc::OTLPData;
use byte_unit::Byte;
use std::borrow::Cow;
use std::time::Instant;
use serde::Deserialize;



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

    meter: Option<self_meter::Meter>,
}

impl PerfExporter {

    pub fn new(meter: Option<self_meter::Meter>, config: Config) -> Self {
        PerfExporter {
            received_msg_count: 0,
            received_evt_count: 0,
            total_received_msg_count: 0,
            total_received_evt_count: 0,
            last_perf_time: Instant::now(),
            meter: meter,
            config: config
        }
    }
}

#[async_trait(?Send)]
impl local::Exporter<OTLPData> for PerfExporter {
    async fn start(
        self: Box<Self>,
        mut msg_chan: MessageChannel<OTLPData>,
        effect_handler: local::EffectHandler<OTLPData>,
    ) -> Result<(), Error<OTLPData>> {
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
                    let duration = now - self.last_perf_time;

                    // Formatted output messages for the report
                    // generate msgs
                    let msg_throughput = format!(
                        "message throughput     : {:.2} msg/s",
                        (self.received_msg_count as f64 / duration.as_secs_f64())
                    );
                    // TODO HANDLE LATENCY
                    let msg_latency = "message average latency: TBD".to_string();

                    let msg_received = format!("total message received : {}", self.total_received_msg_count);
                    let evt_throughput = format!(
                        "event   throughput     : {:.2} evt/s",
                        (self.received_evt_count as f64 / duration.as_secs_f64())
                    );
                    let evt_received = format!("total event received   : {}", self.total_received_evt_count);


                    if let Some(meter) = self.meter.as_mut() {
                        // measure resource usage using self_meter package
                        meter.scan().map_err(|error| Error::ExporterError {
                            exporter: effect_handler.exporter_name(),
                            error: format!("Failed to get meter report: {:?}", error),
                        });
                        // get report of resource usage
                        let report = meter.report();
                        let thread_reports = meter.thread_report();
                        
                        // if meter is enabled report it
                        match report {
                            Some(report) => {
                                display_report_default(effect_handler.exporter_name(), &msg_throughput, &msg_latency, &msg_received, &evt_throughput, &evt_received);
                                display_report_meter(&self.config.cpu_usage, &self.config.mem_usage, &self.config.disk_usage, &self.config.io_usage, &report)
                            }
                            // failed to get report
                            None => {
                                display_report_default(effect_handler.exporter_name(), &msg_throughput, &msg_latency, &msg_received, &evt_throughput, &evt_received);
                            }
                        }
                    // no meter enabled    
                    } else {
                        display_report_default(effect_handler.exporter_name(), &msg_throughput, &msg_latency, &msg_received, &evt_throughput, &evt_received);
                    }


                    // reset received counters
                    // update last_perf_time
                    self.received_msg_count = 0;
                    self.received_evt_count = 0;
                    self.last_perf_time = now;
                }
                Message::Control(ControlMsg::Config { .. }) => {

                }
                Message::Control(ControlMsg::Shutdown { .. }) => {

                    break;
                }
                Message::PData(message) => {
                    // increment counters for received messages
                    match message {
                        
                    }

                    // increment counters for received events
                    self.received_evt_count += event_count;
                    self.total_received_evt_count += event_count;
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

fn count_events_metrics (req: Resource) -> usize {
    req.iter().fold
}

fn count_events_logs() -> usize {

}

fn count_events_trace() -> usize {

}

fn count_events_profiles() -> usize {

}

fn display_report_meter(cpu_usage: &bool, mem_usage: &bool, disk_usage: &bool, io_usage: &bool, report: &self_meter::Report) {
     if *cpu_usage {
        // print all cpu usage data 
        println!(
            "\t- global cpu usage       : {}% (100% is all cores)",
            report.global_cpu_usage
        );
        println!(
            "\t- process cpu usage      : {}% (100% is a single core)",
            report.process_cpu_usage
        );
        println!(
            "\t- gross threads usage  : {}% (100% is a single core)",
            report.gross_cpu_usage
        );
    }
    if *mem_usage {
        // print all memory usage data
        println!(
            "\t- memory rss             : {}",
            Byte::from_bytes(report.memory_rss as u128).get_appropriate_unit(false)
        );
        println!(
            "\t- memory virtual         : {}",
            Byte::from_bytes(report.memory_virtual as u128).get_appropriate_unit(false)
        );
        println!(
            "\t- memory swap            : {}",
            Byte::from_bytes(report.memory_swap as u128).get_appropriate_unit(false)
        );
        println!(
            "\t- memory rss peak        : {}",
            Byte::from_bytes(report.memory_rss_peak as u128).get_appropriate_unit(false)
        );
        println!(
            "\t- memory virtual peak    : {}",
            Byte::from_bytes(report.memory_virtual_peak as u128)
                .get_appropriate_unit(false)
        );
        println!(
            "\t- memory swap peak       : {}",
            Byte::from_bytes(report.memory_swap_peak as u128).get_appropriate_unit(false)
        );
    }
    if *disk_usage {
        println!(
            "\t- disk read              : {}/s",
            Byte::from_bytes(report.disk_read as u128).get_appropriate_unit(false)
        );
        println!(
            "\t- disk write             : {}/s",
            Byte::from_bytes(report.disk_write as u128).get_appropriate_unit(false)
        );
        println!(
            "\t- disk cancelled         : {}/s",
            Byte::from_bytes(report.disk_cancelled as u128).get_appropriate_unit(false)
        );
    }
    if *io_usage {
        println!(
            "\t- io read                : {}/s",
            Byte::from_bytes(report.io_read as u128).get_appropriate_unit(false)
        );
        println!(
            "\t- io write               : {}/s",
            Byte::from_bytes(report.io_write as u128).get_appropriate_unit(false)
        );
        println!("\t- io read ops            : {} read/s", report.io_read_ops);
        println!("\t- io write ops           : {} write/s", report.io_write_ops);
    }
}

fn display_report_default(name: Cow<'static, str>, msg_throughput: &str, msg_latency: &str, msg_received: &str, evt_throughput: &str, evt_received: &str) {
    println!(
        "{}\n\t- {}\n\t- {}\n\t- {}\n\t- {}\n\t- {}",
        name, msg_throughput, msg_latency, msg_received, evt_throughput, evt_received
    );
}