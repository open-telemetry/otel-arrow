use std::env;
use std::fmt;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::path::PathBuf;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tokio::time::timeout;

use tokio::sync::mpsc;
use tonic::{transport::Server};
use tokio_stream::wrappers::TcpListenerStream;

use crate::proto::opentelemetry::collector::trace::v1::trace_service_server::TraceServiceServer;
use crate::proto::opentelemetry::collector::metrics::v1::metrics_service_server::MetricsServiceServer;
use crate::proto::opentelemetry::collector::logs::v1::logs_service_server::LogsServiceServer;

use crate::validation::service_type::{ServiceType, TestReceiver};

const READY_TIMEOUT_SECONDS: u64 = 10;
const READY_MESSAGE: &str = "Everything is ready.";

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

// The TestReceiver implementation has been moved to service_type.rs

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
        // Create a temporary config file for the collector
        let config_path = PathBuf::from(env::temp_dir()).join("otel_collector_config.yaml");

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

// Special implementation for starting a traces service test receiver
pub async fn start_traces_receiver(
    timeout_secs: Option<u64>,
) -> Result<
    (
        tokio::task::JoinHandle<Result<(), tonic::transport::Error>>,
        TimeoutReceiver<crate::proto::opentelemetry::collector::trace::v1::ExportTraceServiceRequest>,
        u16, // actual port number that was assigned
    ),
    String,
> {
    use crate::proto::opentelemetry::collector::trace::v1::ExportTraceServiceRequest;
    
    // Bind to a specific address with port 0 for dynamic port allocation
    let addr = "127.0.0.1:0";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| format!("Failed to bind listener: {}", e))?;
    
    // Get the assigned port
    let port = listener
        .local_addr()
        .map_err(|e| format!("Failed to get local address: {}", e))?
        .port();
    
    // Create a channel for receiving trace data
    let (request_tx, request_rx) = mpsc::channel::<ExportTraceServiceRequest>(100);
    
    // Create a test receiver for the trace service
    let receiver = TestReceiver { request_tx };
    let server = TraceServiceServer::new(receiver);
    
    // Convert the listener to a stream of connections
    let incoming = TcpListenerStream::new(listener);
    
    // Create our server
    let handle = tokio::spawn(async move {
        Server::builder()
            .add_service(server)
            .serve_with_incoming(incoming)
            .await
    });
    
    // Create a timeout-wrapped version of the receiver
    let timeout_duration = Duration::from_secs(timeout_secs.unwrap_or(10));
    let request_rx = TimeoutReceiver {
        inner: request_rx,
        timeout: timeout_duration,
    };
    
    Ok((handle, request_rx, port))
}

// Special implementation for starting a metrics service test receiver
pub async fn start_metrics_receiver(
    timeout_secs: Option<u64>,
) -> Result<
    (
        tokio::task::JoinHandle<Result<(), tonic::transport::Error>>,
        TimeoutReceiver<crate::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest>,
        u16, // actual port number that was assigned
    ),
    String,
> {
    use crate::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
    
    // Bind to a specific address with port 0 for dynamic port allocation
    let addr = "127.0.0.1:0";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| format!("Failed to bind listener: {}", e))?;
    
    // Get the assigned port
    let port = listener
        .local_addr()
        .map_err(|e| format!("Failed to get local address: {}", e))?
        .port();
    
    // Create a channel for receiving metrics data
    let (request_tx, request_rx) = mpsc::channel::<ExportMetricsServiceRequest>(100);
    
    // Create a test receiver for the metrics service
    let receiver = TestReceiver { request_tx };
    let server = MetricsServiceServer::new(receiver);
    
    // Convert the listener to a stream of connections
    let incoming = TcpListenerStream::new(listener);
    
    // Create our server
    let handle = tokio::spawn(async move {
        Server::builder()
            .add_service(server)
            .serve_with_incoming(incoming)
            .await
    });
    
    // Create a timeout-wrapped version of the receiver
    let timeout_duration = Duration::from_secs(timeout_secs.unwrap_or(10));
    let request_rx = TimeoutReceiver {
        inner: request_rx,
        timeout: timeout_duration,
    };
    
    Ok((handle, request_rx, port))
}

// Special implementation for starting a logs service test receiver
pub async fn start_logs_receiver(
    timeout_secs: Option<u64>,
) -> Result<
    (
        tokio::task::JoinHandle<Result<(), tonic::transport::Error>>,
        TimeoutReceiver<crate::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest>,
        u16, // actual port number that was assigned
    ),
    String,
> {
    use crate::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
    
    // Bind to a specific address with port 0 for dynamic port allocation
    let addr = "127.0.0.1:0";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| format!("Failed to bind listener: {}", e))?;
    
    // Get the assigned port
    let port = listener
        .local_addr()
        .map_err(|e| format!("Failed to get local address: {}", e))?
        .port();
    
    // Create a channel for receiving logs data
    let (request_tx, request_rx) = mpsc::channel::<ExportLogsServiceRequest>(100);
    
    // Create a test receiver for the logs service
    let receiver = TestReceiver { request_tx };
    let server = LogsServiceServer::new(receiver);
    
    // Convert the listener to a stream of connections
    let incoming = TcpListenerStream::new(listener);
    
    // Create our server
    let handle = tokio::spawn(async move {
        Server::builder()
            .add_service(server)
            .serve_with_incoming(incoming)
            .await
    });
    
    // Create a timeout-wrapped version of the receiver
    let timeout_duration = Duration::from_secs(timeout_secs.unwrap_or(10));
    let request_rx = TimeoutReceiver {
        inner: request_rx,
        timeout: timeout_duration,
    };
    
    Ok((handle, request_rx, port))
}

/// Start a test receiver server on any available port, with a configurable timeout
/// 
/// This is a generic function that can work with any service type that implements the `ServiceType` trait.
pub async fn start_test_receiver<S: ServiceType>(
    timeout_secs: Option<u64>,
) -> Result<
    (
        tokio::task::JoinHandle<Result<(), tonic::transport::Error>>,
        TimeoutReceiver<S::Request>,
        u16, // actual port number that was assigned
    ),
    String,
> {
    // Dispatch to the appropriate specialized receiver based on the service type name
    match S::name() {
        "traces" => {
            let (handle, receiver, port) = start_traces_receiver(timeout_secs).await?;
            // Safe to transmute here because we're ensuring the types match correctly in each function
            Ok(unsafe { std::mem::transmute((handle, receiver, port)) })
        },
        "metrics" => {
            let (handle, receiver, port) = start_metrics_receiver(timeout_secs).await?;
            // Safe to transmute here because we're ensuring the types match correctly in each function
            Ok(unsafe { std::mem::transmute((handle, receiver, port)) })
        },
        "logs" => {
            let (handle, receiver, port) = start_logs_receiver(timeout_secs).await?;
            // Safe to transmute here because we're ensuring the types match correctly in each function
            Ok(unsafe { std::mem::transmute((handle, receiver, port)) })
        },
        _ => Err(format!("Unknown service type: {}", S::name())),
    }
}

/// Configuration generator for OTLP to OTLP test case
pub fn generate_otlp_to_otlp_config(signal: &str, receiver_port: u16, exporter_port: u16) -> String {
    format!(
        r#"
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 127.0.0.1:{receiver_port}

exporters:
  otlp:
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
      receivers: [otlp]
      exporters: [otlp]
  telemetry:
    metrics:
      address: ""
      level: none
    logs:
      level: info
"#
    )
}
