// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTLP Debug exporter node
//!
//! ToDo: Handle Ack and Nack messages in the pipeline
//! ToDo: Handle configuration changes
//! ToDo: Implement proper deadline function for Shutdown ctrl msg
//! ToDo: Use OTLP Views instead of the OTLP Request structs
//!

use crate::LOCAL_EXPORTERS;
use crate::debug_exporter::{
    config::{Config, Verbosity},
    counter::DebugCounter,
    detailed_otlp_marshaler::DetailedOTLPMarshaler,
    marshaler::OTLPMarshaler,
    normal_otlp_marshaler::NormalOTLPMarshaler,
};
use crate::grpc::OTLPData;
use crate::proto::opentelemetry::{
    collector::{
        logs::v1::ExportLogsServiceRequest, metrics::v1::ExportMetricsServiceRequest,
        profiles::v1development::ExportProfilesServiceRequest,
        trace::v1::ExportTraceServiceRequest,
    },
    metrics::v1::metric::Data,
};
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_engine::error::Error;
use otap_df_engine::local::{LocalExporterFactory, exporter as local};
use otap_df_engine::message::{ControlMsg, Message, MessageChannel};
use serde_json::Value;
use std::fs::OpenOptions;
use std::io::Write;

/// Exporter that outputs all data received to stdout
pub struct DebugExporter {
    config: Config,
    output: Option<String>,
}

/// Declares the Debug exporter as a local exporter factory
///
/// Unsafe code is temporarily used here to allow the use of `distributed_slice` macro
/// This macro is part of the `linkme` crate which is considered safe and well maintained.
#[allow(unsafe_code)]
#[distributed_slice(LOCAL_EXPORTERS)]
pub static DEBUG_EXPORTER: LocalExporterFactory<OTLPData> = LocalExporterFactory {
    name: "urn:otel:debug:exporter",
    create: |config: &Value| Box::new(DebugExporter::from_config(config)),
};

impl DebugExporter {
    /// Creates a new Debug exporter
    #[must_use]
    #[allow(dead_code)]
    pub fn new(config: Config, output: Option<String>) -> Self {
        DebugExporter { config, output }
    }

    /// Creates a new DebugExporter from a configuration object
    #[must_use]
    pub fn from_config(config: &Value) -> Self {
        let config: Config = serde_json::from_value(config.clone())
            .unwrap_or_else(|_| Config::new(Verbosity::Normal));
        DebugExporter {
            config,
            output: None,
        }
    }
}

/// Implement the local exporter trait for a OTAP Exporter
#[async_trait(?Send)]
impl local::Exporter<OTLPData> for DebugExporter {
    async fn start(
        self: Box<Self>,
        mut msg_chan: MessageChannel<OTLPData>,
        effect_handler: local::EffectHandler<OTLPData>,
    ) -> Result<(), Error<OTLPData>> {
        // counter to count number of objects received between timerticks
        let mut counter = DebugCounter::default();

        // create a marshaler to take the otlp objects and extract various data to report
        let marshaler: Box<dyn OTLPMarshaler> = if self.config.verbosity() == Verbosity::Normal {
            Box::new(NormalOTLPMarshaler)
        } else {
            Box::new(DetailedOTLPMarshaler)
        };

        // get a writer to write to stdout or to a file
        let mut writer = get_writer(self.output);
        // Loop until a Shutdown event is received.
        loop {
            match msg_chan.recv().await? {
                // handle control messages
                Message::Control(ControlMsg::TimerTick { .. }) => {
                    _ = writeln!(writer, "Timer tick received");

                    // output count of messages received since last timertick
                    _ = write!(writer, "{report}", report = counter.signals_count_report());

                    // reset counters after timertick
                    counter.reset_signal_count();
                }
                Message::Control(ControlMsg::Config { .. }) => {
                    _ = writeln!(writer, "Config message received");
                }
                // shutdown the exporter
                Message::Control(ControlMsg::Shutdown { .. }) => {
                    // ToDo: add proper deadline function
                    _ = writeln!(writer, "Shutdown message received");
                    _ = write!(writer, "{report}", report = counter.debug_report());
                    break;
                }
                //send data
                Message::PData(message) => {
                    match message {
                        // ToDo: Add Ack/Nack handling, send a signal that data has been exported
                        // ToDo: Use the views instead of OTLPData

                        // match on OTLPData type and use the respective method to collect data about the received object
                        // increment the counters for each respective OTLP Datatype
                        OTLPData::Metrics(req) => {
                            push_metric(
                                &self.config.verbosity(),
                                req,
                                &*marshaler,
                                &mut writer,
                                &mut counter,
                            );
                            counter.increment_metric_signal_count();
                        }
                        OTLPData::Logs(req) => {
                            push_log(
                                &self.config.verbosity(),
                                req,
                                &*marshaler,
                                &mut writer,
                                &mut counter,
                            );
                            counter.increment_log_signal_count();
                        }
                        OTLPData::Traces(req) => {
                            push_trace(
                                &self.config.verbosity(),
                                req,
                                &*marshaler,
                                &mut writer,
                                &mut counter,
                            );
                            counter.increment_span_signal_count();
                        }
                        OTLPData::Profiles(req) => {
                            push_profile(
                                &self.config.verbosity(),
                                req,
                                &*marshaler,
                                &mut writer,
                                &mut counter,
                            );
                            counter.increment_profile_signal_count();
                        }
                    }
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

/// Function to collect and report the data contained in a Metrics object received by the Debug exporter
fn push_metric(
    verbosity: &Verbosity,
    metric_request: ExportMetricsServiceRequest,
    marshaler: &dyn OTLPMarshaler,
    writer: &mut impl Write,
    counter: &mut DebugCounter,
) {
    // collect number of resource metrics
    // collect number of metrics
    // collect number of datapoints
    let resource_metrics = metric_request.resource_metrics.len();
    let mut data_points = 0;
    let mut metrics = 0;
    for resource_metrics in &metric_request.resource_metrics {
        for scope_metrics in &resource_metrics.scope_metrics {
            metrics += scope_metrics.metrics.len();
            for metric in &scope_metrics.metrics {
                if let Some(data) = &metric.data {
                    match data {
                        Data::Gauge(gauge) => {
                            data_points += gauge.data_points.len();
                        }
                        Data::Sum(sum) => {
                            data_points += sum.data_points.len();
                        }
                        Data::Histogram(histogram) => {
                            data_points += histogram.data_points.len();
                        }
                        Data::ExponentialHistogram(exponential_histogram) => {
                            data_points += exponential_histogram.data_points.len();
                        }
                        Data::Summary(summary) => {
                            data_points += summary.data_points.len();
                        }
                    }
                }
            }
        }
    }

    _ = writeln!(writer, "Received {resource_metrics} resource metrics");
    _ = writeln!(writer, "Received {metrics} metrics");
    _ = writeln!(writer, "Received {data_points} data points");
    counter.update_metric_data(resource_metrics as u64, metrics as u64, data_points as u64);
    // if verbosity is basic we don't report anymore information, if a higher verbosity is specified than we call the marshaler
    if *verbosity == Verbosity::Basic {
        return;
    }

    let report = marshaler.marshal_metrics(metric_request);
    _ = writeln!(writer, "{report}");
}

fn push_trace(
    verbosity: &Verbosity,
    trace_request: ExportTraceServiceRequest,
    marshaler: &dyn OTLPMarshaler,
    writer: &mut impl Write,
    counter: &mut DebugCounter,
) {
    // collect number of resource spans
    // collect number of spans
    let resource_spans = trace_request.resource_spans.len();
    let mut spans = 0;
    let mut events = 0;
    let mut links = 0;
    for resource_span in &trace_request.resource_spans {
        for scope_span in &resource_span.scope_spans {
            spans += scope_span.spans.len();
            for span in &scope_span.spans {
                events += span.events.len();
                links += span.links.len();
            }
        }
    }

    _ = writeln!(writer, "Received {resource_spans} resource spans");
    _ = writeln!(writer, "Received {spans} spans");
    _ = writeln!(writer, "Received {events} events");
    _ = writeln!(writer, "Received {links} links");
    counter.update_span_data(
        resource_spans as u64,
        spans as u64,
        events as u64,
        links as u64,
    );
    // if verbosity is basic we don't report anymore information, if a higher verbosity is specified than we call the marshaler
    if *verbosity == Verbosity::Basic {
        return;
    }

    let report = marshaler.marshal_traces(trace_request);
    _ = writeln!(writer, "{report}");
}

fn push_log(
    verbosity: &Verbosity,
    log_request: ExportLogsServiceRequest,
    marshaler: &dyn OTLPMarshaler,
    writer: &mut impl Write,
    counter: &mut DebugCounter,
) {
    let resource_logs = log_request.resource_logs.len();
    let mut log_records = 0;
    let mut events = 0;
    for resource_log in &log_request.resource_logs {
        for scope_log in &resource_log.scope_logs {
            log_records += scope_log.log_records.len();
            for log_record in &scope_log.log_records {
                if !log_record.event_name.is_empty() {
                    events += 1;
                }
            }
        }
    }
    _ = writeln!(writer, "Received {resource_logs} resource logs");
    _ = writeln!(writer, "Received {log_records} log records");
    _ = writeln!(writer, "Received {events} events");
    counter.update_log_data(resource_logs as u64, log_records as u64, events as u64);
    if *verbosity == Verbosity::Basic {
        return;
    }

    let report = marshaler.marshal_logs(log_request);
    _ = writeln!(writer, "{report}");
}

fn push_profile(
    verbosity: &Verbosity,
    profile_request: ExportProfilesServiceRequest,
    marshaler: &dyn OTLPMarshaler,
    writer: &mut impl Write,
    counter: &mut DebugCounter,
) {
    // collect number of resource profiles
    // collect number of sample records
    let resource_profiles = profile_request.resource_profiles.len();
    let mut samples = 0;
    for resource_profile in &profile_request.resource_profiles {
        for scope_profile in &resource_profile.scope_profiles {
            for profile in &scope_profile.profiles {
                samples += profile.sample.len();
            }
        }
    }

    _ = writeln!(writer, "Received {resource_profiles} resource profiles");
    _ = writeln!(writer, "Received {samples} samples");
    counter.update_profile_data(resource_profiles as u64, samples as u64);
    if *verbosity == Verbosity::Basic {
        return;
    }

    let report = marshaler.marshal_profiles(profile_request);
    _ = writeln!(writer, "{report}");
}

#[cfg(test)]
mod tests {

    use crate::debug_exporter::config::{Config, Verbosity};
    use crate::debug_exporter::exporter::DebugExporter;
    use crate::grpc::OTLPData;
    use crate::mock::{
        create_otlp_log, create_otlp_metric, create_otlp_profile, create_otlp_trace,
    };

    use otap_df_engine::exporter::ExporterWrapper;
    use otap_df_engine::testing::exporter::TestContext;
    use otap_df_engine::testing::exporter::TestRuntime;
    use tokio::time::{Duration, sleep};

    use std::fs::{File, remove_file};
    use std::io::{BufReader, read_to_string};

    /// Test closure that simulates a typical test scenario by sending timer ticks, config,
    /// data message, and shutdown control messages.
    ///
    fn scenario()
    -> impl FnOnce(TestContext<OTLPData>) -> std::pin::Pin<Box<dyn Future<Output = ()>>> {
        |ctx| {
            Box::pin(async move {
                // send some messages to the exporter to calculate pipeline statistics
                // // Send a data message
                ctx.send_pdata(OTLPData::Metrics(create_otlp_metric(1, 1, 5, 1)))
                    .await
                    .expect("Failed to send data message");
                ctx.send_pdata(OTLPData::Traces(create_otlp_trace(1, 1, 1, 1, 1)))
                    .await
                    .expect("Failed to send data message");
                ctx.send_pdata(OTLPData::Logs(create_otlp_log(1, 1, 1)))
                    .await
                    .expect("Failed to send data message");
                ctx.send_pdata(OTLPData::Profiles(create_otlp_profile(1, 1, 1)))
                    .await
                    .expect("Failed to send data message");

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
    ) -> impl FnOnce(TestContext<OTLPData>) -> std::pin::Pin<Box<dyn Future<Output = ()>>> {
        |_| {
            Box::pin(async move {
                // get a file to read and validate the output
                // open file
                // read the output file
                // assert each line accordingly
                let file = File::open(output_file).expect("failed to open file");
                let reader = read_to_string(BufReader::new(file)).expect("failed to get string");

                // check the the exporter has received the expected number of messages
                assert!(reader.contains("Timer tick received"));
                assert!(reader.contains("OTLP Metric objects received: 0"));
                assert!(reader.contains("OTLP Trace objects received: 0"));
                assert!(reader.contains("OTLP Profile objects received: 0"));
                assert!(reader.contains("OTLP Log objects received: 0"));
                assert!(reader.contains("Received 1 resource metrics"));
                assert!(reader.contains("Received 5 metrics"));
                assert!(reader.contains("Received 5 data points"));
                assert!(reader.contains("Received 1 resource spans"));
                assert!(reader.contains("Received 1 spans"));
                assert!(reader.contains("Received 1 events"));
                assert!(reader.contains("Received 1 links"));
                assert!(reader.contains("Received 1 resource logs"));
                assert!(reader.contains("Received 1 log records"));
                assert!(reader.contains("Received 1 events"));
                assert!(reader.contains("Received 1 resource profiles"));
                assert!(reader.contains("Received 0 samples"));
                assert!(reader.contains("Shutdown message received"));
            })
        }
    }

    #[test]
    fn test_debug_exporter_basic_verbosity() {
        let test_runtime = TestRuntime::new();
        let output_file = "debug_output_basic.txt".to_string();
        let config = Config::new(Verbosity::Basic);
        let exporter = ExporterWrapper::local(
            DebugExporter::new(config, Some(output_file.clone())),
            test_runtime.config(),
        );

        test_runtime
            .set_exporter(exporter)
            .run_test(scenario())
            .run_validation(validation_procedure(output_file.clone()));

        // remove the created file, prevent accidental check in of report
        remove_file(output_file).expect("Failed to remove file");
    }

    #[test]
    fn test_debug_exporter_normal_verbosity() {
        let test_runtime = TestRuntime::new();
        let output_file = "debug_output_normal.txt".to_string();
        let config = Config::new(Verbosity::Normal);
        let exporter = ExporterWrapper::local(
            DebugExporter::new(config, Some(output_file.clone())),
            test_runtime.config(),
        );

        test_runtime
            .set_exporter(exporter)
            .run_test(scenario())
            .run_validation(validation_procedure(output_file.clone()));

        // remove the created file, prevent accidental check in of report
        remove_file(output_file).expect("Failed to remove file");
    }

    #[test]
    fn test_debug_exporter_detailed_verbosity() {
        let test_runtime = TestRuntime::new();
        let output_file = "debug_output_detailed.txt".to_string();
        let config = Config::new(Verbosity::Detailed);
        let exporter = ExporterWrapper::local(
            DebugExporter::new(config, Some(output_file.clone())),
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
