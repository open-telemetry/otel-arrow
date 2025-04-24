// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// This file provides facilities for starting and stopping a child
// process that runs an OpenTelemetry Collector (Golang) with either
// OTLP or OTAP, receiver or exporter.  See the run_test::<> entry
// point.

use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::LazyLock;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use snafu::{OptionExt, ResultExt};
use tokio::sync::mpsc;
use tokio::time::timeout;

use super::error;
use super::service_type;

const READY_MESSAGE: &str = "Everything is ready.";

pub(crate) const READY_TIMEOUT_SECONDS: u64 = 10;
pub(crate) const SHUTDOWN_TIMEOUT_SECONDS: u64 = 15;
pub(crate) const RECEIVER_TIMEOUT_SECONDS: u64 = 10;
pub(crate) const TEST_TIMEOUT_SECONDS: u64 = 20;

pub static COLLECTOR_PATH: LazyLock<String> = LazyLock::new(|| {
    let default_path = "../../bin/otelarrowcol";
    let path = std::env::var("OTEL_COLLECTOR_PATH").unwrap_or(default_path.to_string());

    // Check if the collector exists at the specified path
    if !std::path::Path::new(&path).exists() {
        eprintln!(
            "Warning: OpenTelemetry collector not found at '{}'. Tests may fail.",
            path
        );
        eprintln!("Set OTEL_COLLECTOR_PATH environment variable to the correct path or ensure the collector is built.");
    }

    path
});

/// A wrapper around mpsc::Receiver that adds timeout functionality
pub struct TimeoutReceiver<T> {
    pub inner: mpsc::Receiver<T>,
    pub timeout: Duration,
}

impl<T> TimeoutReceiver<T> {
    /// Receive a value with timeout
    pub async fn recv(&mut self) -> error::Result<T> {
        timeout(self.timeout, self.inner.recv())
            .await
            .context(error::ReceiverTimeoutSnafu)?
            .context(error::NoResponseSnafu)
    }
}

/// Helper function to spawn a thread that reads lines from a buffer and logs them with a prefix.
/// Optionally checks for a message substring and sends a signal when it matches.
fn spawn_line_reader<R>(
    reader: R,
    prefix: &'static str,
    mut probe: Option<(std::sync::mpsc::Sender<()>, &'static str)>,
) -> JoinHandle<()>
where
    R: std::io::Read + Send + 'static,
{
    thread::spawn(move || {
        let buf_reader = BufReader::new(reader);
        for line in buf_reader.lines() {
            if let Ok(line) = line {
                eprintln!("[{}] {}", prefix, line);

                // If we need to check for a ready message
                if let Some((ref tx, message)) = probe {
                    if line.contains(message) {
                        // Send using standard sync channel
                        let _ = tx.send(());
                        probe = None;
                    }
                }
            }
        }
    })
}

/// A helper struct to manage the collector process
pub struct CollectorProcess {
    process: Child,
    config_path: PathBuf,
    stdout_handle: Option<JoinHandle<()>>,
    stderr_handle: Option<JoinHandle<()>>,
}

impl CollectorProcess {
    /// Sends a SIGTERM signal to initiate graceful shutdown.
    pub async fn shutdown(&mut self) -> error::Result<()> {
        #[cfg(unix)]
        {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;
            let pid = self.process.id();
            eprintln!("Sending SIGTERM to collector process {}", pid);

            kill(Pid::from_raw(pid as i32), Signal::SIGTERM).context(error::SignalNotDeliveredSnafu)?;
        }

        #[cfg(not(unix))]
        {
            panic!("SIGTERM not supported on this platform");
        }

        // Wait for the collector to exit
        let status = self
            .process
            .wait()
            .context(error::InputOutputSnafu { desc: "wait" })?;

        status.success().then(|| ()).context(error::BadExitStatusSnafu {
            code: status.code(),
        })
    }

    /// Start a collector with the given configuration
    pub async fn start<T: AsRef<Path>>(collector_path: T, config_content: &str) -> error::Result<Self> {
        // Create a unique temporary config file for the collector with a random identifier
        // to prevent collision with other tests
        let random_id = format!("{:016x}", rand::random::<u64>());
        let config_path = PathBuf::from(env::temp_dir())
            .join(format!("otel_collector_config_{}.yaml", random_id));

        // Write the config to the file
        let mut file =
            fs::File::create(&config_path).context(error::InputOutputSnafu { desc: "create" })?;

        file.write_all(config_content.as_bytes())
            .context(error::InputOutputSnafu { desc: "write" })?;

        // Start the collector process with piped stdout and stderr
        let mut process = Command::new(collector_path.as_ref())
            .arg("--config")
            .arg(&config_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context(error::InputOutputSnafu { desc: "command" })?;

        // Get handles to stdout and stderr
        let stdout = process
            .stdout
            .take()
            .context(error::FileNotAvailableSnafu { desc: "stderr" })?;

        let stderr = process
            .stderr
            .take()
            .context(error::FileNotAvailableSnafu { desc: "stderr" })?;

        // Create a standard sync channel to signal when the collector is ready
        let (ready_tx, ready_rx) = std::sync::mpsc::channel();

        // Create threads to read from stdout and stderr
        let (stdout_handle, stderr_handle) = (
            spawn_line_reader(stdout, "Collector stdout", None),
            spawn_line_reader(stderr, "Collector stderr", Some((ready_tx, READY_MESSAGE))),
        );

        // Now create a oneshot channel for the async side
        let (tokio_tx, tokio_rx) = tokio::sync::oneshot::channel();

        // Spawn a thread to bridge between the sync and async worlds
        thread::spawn(move || {
            // Wait for the ready signal from the sync channel
            if ready_rx.recv().is_ok() {
                // Forward it to the async world
                let _ = tokio_tx.send(());
            }
        });

        // Create timeout for the async receiver
        let timeout_duration = Duration::from_secs(READY_TIMEOUT_SECONDS);

        // Wait for the ready message with timeout and return the collector process when ready
        _ = tokio::time::timeout(timeout_duration, tokio_rx)
            .await
            .context(error::ReadyTimeoutSnafu)?
            .context(error::ChannelClosedSnafu)?;

        Ok(Self {
            process,
            config_path,
            stdout_handle: Some(stdout_handle),
            stderr_handle: Some(stderr_handle),
        })
    }
}

impl Drop for CollectorProcess {
    fn drop(&mut self) {
        // Clean up the collector process when done
        let _ = self.process.kill();

        // Wait for the stdout and stderr threads to complete
        if let Some(handle) = self.stdout_handle.take() {
            let _ = handle.join();
        }

        if let Some(handle) = self.stderr_handle.take() {
            let _ = handle.join();
        }

        // Clean up temp config file
        let _ = fs::remove_file(&self.config_path);
    }
}

/// Configuration generator
pub fn generate_config(
    receiver_protocol: &str,
    receiver_signal: &str,
    receiver_port: u16,
    exporter_protocol: &str,
    _exporter_signal: &str,
    exporter_port: u16,
) -> String {
    format!(
        r#"
receivers:
  {receiver_protocol}:
    protocols:
      grpc:
        endpoint: 127.0.0.1:{receiver_port}

exporters:
  {exporter_protocol}:
    endpoint: 127.0.0.1:{exporter_port}
    compression: none
    tls:
      insecure: true
    wait_for_ready: true
    timeout: 2s
    sending_queue:
      enabled: false
    retry_on_failure:
      enabled: false

service:
  pipelines:
    {receiver_signal}:
      receivers: [{receiver_protocol}]
      exporters: [{exporter_protocol}]
  telemetry:
    metrics:
      level: none
    logs:
      level: info
"#
    )
}

/// TestContext contains all the necessary components for running a test
pub struct TestContext<I: service_type::ServiceInputType, O: service_type::ServiceOutputType> {
    pub client: I::Client,
    pub collector: CollectorProcess,
    pub request_rx: TimeoutReceiver<O::Request>,
    pub server_handle: tokio::task::JoinHandle<error::Result<()>>,
    pub server_shutdown_tx: tokio::sync::oneshot::Sender<()>,
}

/// Generic test runner for telemetry signal tests
///
/// This function will:
/// 1. Start a generic test receiver server
/// 2. Start the OTel collector
/// 3. Create a test context with client and receiver
/// 4. Run the supplied test logic
/// 5. Perform cleanup
///
/// The service type parameters I and O determine the input and output signal types to test
pub async fn run_test<I, O, T, F>(test_logic: F) -> error::Result<()>
where
    I: service_type::ServiceInputType,
    O: service_type::ServiceOutputType,
    I::Request: std::fmt::Debug + PartialEq,
    O::Request: std::fmt::Debug + PartialEq,
    F: FnOnce(TestContext<I, O>) -> T,
    T: std::future::Future<Output = (TestContext<I, O>, error::Result<()>)>,
{
    // Generate random ports in the high u16 range to avoid conflicts
    let random_value = rand::random::<u16>();
    let receiver_port = 40000 + (random_value % 25000);

    // Start the test receiver server and wrap it with a timeout to avoid tests getting stuck
    let (server_handle, request_rx_raw, exporter_port, server_shutdown_tx) =
        service_type::start_test_receiver::<O>().await?;

    // Create a timeout-wrapped version of the receiver
    let timeout_duration = std::time::Duration::from_secs(RECEIVER_TIMEOUT_SECONDS);
    let request_rx = TimeoutReceiver {
        inner: request_rx_raw,
        timeout: timeout_duration,
    };

    // Generate and start the collector with the input and output protocols using the dynamic ports
    let collector_config = generate_config(
        I::protocol(),
        I::signal(),
        receiver_port,
        O::protocol(),
        O::signal(),
        exporter_port,
    );

    let collector = CollectorProcess::start(COLLECTOR_PATH.clone(), &collector_config).await?;

    // Create client to send test data
    let client_endpoint = format!("http://127.0.0.1:{}", receiver_port);
    let client = I::connect_client(client_endpoint).await?;

    // Create the test context
    let context = TestContext {
        client,
        collector,
        request_rx,
        server_handle,
        server_shutdown_tx,
    };

    // Run the provided test logic, transferring ownership of the context
    // The test_logic now returns the context back along with the result
    let (mut context, result) = test_logic(context).await;

    // Cleanup: drop the client connection first
    drop(context.client);

    // Send a shutdown signal to the collector process.
    context.collector.shutdown().await?;

    drop(context.request_rx);

    // Gracefully shut down the server by sending a signal through the shutdown channel
    let _ = context.server_shutdown_tx.send(());

    // Wait for the server to shut down with timeout
    tokio::time::timeout(
        std::time::Duration::from_secs(SHUTDOWN_TIMEOUT_SECONDS),
        context.server_handle,
    ).await
	.context(error::TestTimeoutSnafu)?
	.context(error::JoinSnafu)?
    ?;

    // Return the result from the test logic
    result
}
