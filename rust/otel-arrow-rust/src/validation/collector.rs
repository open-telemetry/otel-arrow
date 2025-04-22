// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::env;
use std::fmt;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::path::PathBuf;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::LazyLock;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tokio::time::timeout;

use tokio::sync::mpsc;

use super::service_type::{start_test_receiver, ServiceInputType, ServiceOutputType};

const READY_TIMEOUT_SECONDS: u64 = 10;
const READY_MESSAGE: &str = "Everything is ready.";
pub const SHUTDOWN_TIMEOUT_SECONDS: u64 = 15;
pub const RECEIVER_TIMEOUT_SECONDS: u64 = 10;
pub const TEST_TIMEOUT_SECONDS: u64 = 20;

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

/// TimeoutError represents an error when a receiver operation times out
#[derive(Debug)]
pub struct TimeoutError {
    pub duration: Duration,
}

impl fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Receiver operation timed out after {:?}", self.duration)
    }
}

impl std::error::Error for TimeoutError {}

/// A wrapper around mpsc::Receiver that adds timeout functionality
pub struct TimeoutReceiver<T> {
    pub inner: mpsc::Receiver<T>,
    pub timeout: Duration,
}

impl<T> TimeoutReceiver<T> {
    /// Receive a value with timeout
    pub async fn recv(&mut self) -> Result<T, Box<dyn std::error::Error + Send + Sync>> {
        match timeout(self.timeout, self.inner.recv()).await {
            Ok(Some(value)) => Ok(value),
            Ok(None) => Err("Channel closed".into()),
            Err(_) => Err(Box::new(TimeoutError {
                duration: self.timeout,
            })),
        }
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
    pub async fn shutdown(&mut self) -> Result<Option<ExitStatus>, std::io::Error> {
        #[cfg(unix)]
        {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;
            let pid = self.process.id();
            eprintln!("Sending SIGTERM to collector process {}", pid);

            if let Err(e) = kill(Pid::from_raw(pid as i32), Signal::SIGTERM) {
                panic!("Failed to send SIGTERM: {}", e);
            }
        }

        #[cfg(not(unix))]
        {
            panic!("SIGTERM not supported on this platform");
        }

        // Wait for the collector to exit
        Ok(Some(self.process.wait()?))
    }

    /// Start a collector with the given configuration
    pub async fn start<T: AsRef<Path>>(
        collector_path: T,
        config_content: &str,
    ) -> Result<Self, String> {
        // Create a unique temporary config file for the collector with a random identifier
        // to prevent collision with other tests
        let random_id = format!("{:016x}", rand::random::<u64>());
        let config_path = PathBuf::from(env::temp_dir())
            .join(format!("otel_collector_config_{}.yaml", random_id));

        // Write the config to the file
        let mut file = fs::File::create(&config_path)
            .map_err(|e| format!("Failed to create config file: {}", e))?;

        file.write_all(config_content.as_bytes())
            .map_err(|e| format!("Failed to write config content: {}", e))?;

        // Start the collector process with piped stdout and stderr
        let mut process = Command::new(collector_path.as_ref())
            .arg("--config")
            .arg(&config_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start collector process: {}", e))?;

        // Get handles to stdout and stderr
        let stdout = process
            .stdout
            .take()
            .ok_or_else(|| "Failed to capture process stdout".to_string())?;

        let stderr = process
            .stderr
            .take()
            .ok_or_else(|| "Failed to capture process stderr".to_string())?;

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
        match tokio::time::timeout(timeout_duration, tokio_rx).await {
            Ok(Ok(())) => Ok(Self {
                process,
                config_path,
                stdout_handle: Some(stdout_handle),
                stderr_handle: Some(stderr_handle),
            }),
            Ok(Err(_)) => Err("Channel closed before receiving ready message".to_string()),
            Err(_) => Err(format!(
                "Timed out after waiting {:?} for collector to be ready",
                timeout_duration
            )),
        }
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
pub struct TestContext<I: ServiceInputType, O: ServiceOutputType> {
    pub client: I::Client,
    pub collector: CollectorProcess,
    pub request_rx: TimeoutReceiver<O::Request>,
    pub server_handle: tokio::task::JoinHandle<Result<(), tonic::transport::Error>>,
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
pub async fn run_test<I, O, T, F>(test_logic: F) -> Result<(), Box<dyn std::error::Error>>
where
    I: ServiceInputType,
    O: ServiceOutputType,
    I::Request: std::fmt::Debug + PartialEq,
    O::Request: std::fmt::Debug + PartialEq,
    F: FnOnce(TestContext<I, O>) -> T,
    T: std::future::Future<Output = (TestContext<I, O>, Result<(), Box<dyn std::error::Error>>)>,
{
    // Generate random ports in the high u16 range to avoid conflicts
    let random_value = rand::random::<u16>();
    let receiver_port = 40000 + (random_value % 25000);

    // Start the test receiver server and wrap it with a timeout to avoid tests getting stuck
    let (server_handle, request_rx_raw, exporter_port, server_shutdown_tx) = start_test_receiver::<O>()
        .await
        .map_err(|e| format!("Failed to start test receiver: {}", e))?;

    // Create a timeout-wrapped version of the receiver
    let timeout_duration = std::time::Duration::from_secs(RECEIVER_TIMEOUT_SECONDS);
    let request_rx = TimeoutReceiver {
        inner: request_rx_raw,
        timeout: timeout_duration,
    };

    // Generate and start the collector with the input and output protocols using the dynamic ports
    let collector_config = generate_config(
        I::protocol(), I::signal(), receiver_port, 
        O::protocol(), O::signal(), exporter_port,
    );

    let collector = CollectorProcess::start(COLLECTOR_PATH.clone(), &collector_config)
        .await
        .map_err(|e| format!("Failed to start collector: {}", e))?;

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
    match context.collector.shutdown().await {
        Ok(status) => {
            if let Some(s) = status {
                eprintln!("Collector exited with status: {}", s);
            } else {
                eprintln!("Collector shut down");
            }
        }
        Err(e) => eprintln!("Error shutting down collector: {}", e),
    }

    drop(context.request_rx);

    // Gracefully shut down the server by sending a signal through the shutdown channel
    let _ = context.server_shutdown_tx.send(());
    
    // Wait for the server to shut down with timeout
    match tokio::time::timeout(
        std::time::Duration::from_secs(SHUTDOWN_TIMEOUT_SECONDS),
        context.server_handle,
    )
    .await
    {
        Ok(Ok(_)) => eprintln!("{} server shut down successfully", O::signal()),
        Ok(Err(e)) => eprintln!("Error shutting down {} server: {}", O::signal(), e),
        Err(e) => {
            eprintln!("Timed out waiting for {} server to shut down: {}", O::signal(), e);
        }
    }

    // Return the result from the test logic
    result
}
