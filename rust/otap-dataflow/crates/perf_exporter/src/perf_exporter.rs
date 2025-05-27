use std::time::{Duration, Instant};

use async_trait::async_trait;
use beaubourg::{
    exporter::{effect::EffectHandler, AsyncExporter, ConcurrencyModel, EngineHandler, Error, ExporterBuilder},
    signal::{Signal, SignalReceiver},
};
use byte_unit::Byte;
use message::Message;
use serde::Deserialize;
use serde_yaml::Value;

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

pub struct PerfExporter {
    name: String,
    config: Config,

    received_msg_count: usize,
    received_evt_count: usize,
    total_received_msg_count: usize,
    total_received_evt_count: usize,

    last_perf_time: Instant,

    meter: Option<self_meter::Meter>,
}

#[async_trait]
impl AsyncExporter<Message> for PerfExporter {
    async fn init(&mut self, engine_handler: &mut EngineHandler) -> Result<(), Error> {
        self.last_perf_time = Instant::now();
        engine_handler.timer(Duration::from_millis(self.config.timeout));
        Ok(())
    }

    async fn export(
        &mut self,
        mut signal_receiver: SignalReceiver<Message>,
        _effect_handler: EffectHandler<Message>,
    ) -> Result<(), Error> {
        loop {
            match signal_receiver.recv().await {
                Signal::TimerTick { .. } => {
                    let now = Instant::now();
                    let duration = now - self.last_perf_time;
                    let msg_throughput = format!(
                        "message throughput     : {:.2} msg/s",
                        (self.received_msg_count as f64 / duration.as_secs_f64())
                    );
                    let msg_latency = "message average latency: TBD".to_string();
                    let msg_received = format!("total message received : {}", self.total_received_msg_count);
                    let evt_throughput = format!(
                        "event   throughput     : {:.2} evt/s",
                        (self.received_evt_count as f64 / duration.as_secs_f64())
                    );
                    let evt_received = format!("total event received   : {}", self.total_received_evt_count);

                    if let Some(meter) = self.meter.as_mut() {
                        meter.scan().map_err(|e| eprintln!("self-meter scan error: {}", e)).ok();
                        let report = meter.report();

                        match report {
                            Some(report) => {
                                println!(
                                    "{}\n\t- {}\n\t- {}\n\t- {}\n\t- {}\n\t- {}",
                                    self.name, evt_throughput, evt_received, msg_throughput, msg_latency, msg_received
                                );
                                if self.config.cpu_usage {
                                    println!(
                                        "\t- global cpu usage       : {}% (100% is all cores)",
                                        report.global_cpu_usage
                                    );
                                    println!(
                                        "\t- process cpu usage      : {}% (100% is a single core)",
                                        report.process_cpu_usage
                                    );
                                }
                                if self.config.mem_usage {
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
                                if self.config.disk_usage {
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
                                if self.config.io_usage {
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
                            None => {
                                println!(
                                    "{}\n\t- {}\n\t- {}\n\t- {}\n\t- {}\n\t- {}",
                                    self.name, msg_throughput, msg_latency, msg_received, evt_throughput, evt_received
                                );
                            }
                        }
                    } else {
                        println!(
                            "{}\n\t- {}\n\t- {}\n\t- {}\n\t- {}\n\t- {}",
                            self.name, msg_throughput, msg_latency, msg_received, evt_throughput, evt_received
                        );
                    }

                    // println!("Threads: {:#?}",
                    //          self.meter.thread_report().map(|x| x.collect::<BTreeMap<_,_>>()));

                    self.received_msg_count = 0;
                    self.received_evt_count = 0;
                    self.last_perf_time = now;
                }
                Signal::Messages { messages } => {
                    self.received_msg_count += messages.len();
                    self.total_received_msg_count += messages.len();

                    let event_count = messages.iter().fold(0, |acc, msg| {
                        // Compute the size of the main table for the message
                        let main_table_size = msg
                            .columnar_tables
                            .iter()
                            .find(|table| table.r#type.is_main_table())
                            .map(|table| table.record_batch.num_rows())
                            .unwrap_or(0);

                        acc + main_table_size
                    });

                    self.received_evt_count += event_count;
                    self.total_received_evt_count += event_count;
                }
                Signal::Stop => return Ok(()),
                signal => {
                    return Err(Error::UnsupportedEvent {
                        exporter: self.name.clone(),
                        signal: signal.to_string(),
                    })
                }
            }
        }
    }
}

pub struct PerfExporterBuilder {
    name: String,
    concurrency_model: ConcurrencyModel,
    config: Value,
}

impl PerfExporterBuilder {
    pub fn new(name: String, config: Value) -> Self {
        let concurrency_model = match config.get("concurrency_model") {
            None => ConcurrencyModel::TaskPerCore(1),
            Some(value) => serde_yaml::from_value(value.clone()).expect("failed to deserialize concurrency model"),
        };

        PerfExporterBuilder {
            name,
            concurrency_model,
            config,
        }
    }
}

impl ExporterBuilder<Message> for PerfExporterBuilder {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn r#type(&self) -> String {
        "perf".into()
    }

    fn concurrency_model(&self) -> ConcurrencyModel {
        ConcurrencyModel::Singleton
    }

    fn build(&self) -> Result<Box<dyn AsyncExporter<Message> + Send + Sync>, Error> {
        tracing::info!(exporter_name=%self.name, concurrency_model=?self.concurrency_model, "Building perf exporter");
        let mut config: Config = serde_yaml::from_value(self.config.clone()).map_err(|e| Error::InvalidConfig {
            message: e.to_string(),
            exporter: self.name.clone(),
            line: e.location().map(|loc| loc.line()),
            column: e.location().map(|loc| loc.column()),
        })?;
        if !config.cpu_usage && !config.mem_usage && !config.disk_usage && !config.io_usage {
            config.self_usage = false;
        }
        let scan_interval_ms = config.timeout;
        let self_usage = config.self_usage;

        let exporter = Box::new(PerfExporter {
            name: self.name.clone(),
            config,
            received_msg_count: 0,
            received_evt_count: 0,
            total_received_msg_count: 0,
            total_received_evt_count: 0,
            last_perf_time: Instant::now(),
            meter: if self_usage {
                match self_meter::Meter::new(Duration::from_millis(scan_interval_ms)) {
                    Ok(meter) => Some(meter),
                    Err(err) => {
                        eprintln!("self-meter not created (reason: {:?})", err);
                        None
                    }
                }
            } else {
                None
            },
        });

        Ok(exporter as Box<dyn AsyncExporter<Message> + Send + Sync>)
    }
}