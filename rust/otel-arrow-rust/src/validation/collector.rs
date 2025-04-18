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

const READY_TIMEOUT_SECONDS: u64 = 10;
const READY_MESSAGE: &str = "Everything is ready.";

pub static COLLECTOR_PATH: LazyLock<String> =
    LazyLock::new(|| {
        let default_path = "../../bin/otelarrowcol";
        let path = std::env::var("OTEL_COLLECTOR_PATH").unwrap_or(default_path.to_string());
        
        // Check if the collector exists at the specified path
        if !std::path::Path::new(&path).exists() {
            eprintln!("Warning: OpenTelemetry collector not found at '{}'. Tests may fail.", path);
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
        self.process.try_wait()
    }

    /// Start a collector with the given configuration
    pub async fn start<T: AsRef<Path>>(
        collector_path: T,
        config_content: &str,
    ) -> Result<Self, String> {
        // Create a unique temporary config file for the collector with a random identifier
        // to prevent collision with other tests
        let random_id = format!("{:016x}", rand::random::<u64>());
        let config_path = PathBuf::from(env::temp_dir()).join(format!("otel_collector_config_{}.yaml", random_id));

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
            Err(_) => Err(format!("Timed out after waiting {:?} for collector to be ready", timeout_duration))
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
pub fn generate_config(exporter_name: &str, receiver_name: &str, signal: &str, receiver_port: u16, exporter_port: u16) -> String {
    format!(
        r#"
receivers:
  {receiver_name}:
    protocols:
      grpc:
        endpoint: 127.0.0.1:{receiver_port}

exporters:
  {exporter_name}:
    endpoint: 127.0.0.1:{exporter_port}
    compression: none
    tls:
      insecure: true
    wait_for_ready: true
    timeout: 2s
    retry_on_failure:
      enabled: false

service:
  pipelines:
    {signal}:
      receivers: [{receiver_name}]
      exporters: [{exporter_name}]
  telemetry:
    metrics:
      level: none
    logs:
      level: info
"#
    )
}
