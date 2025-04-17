use std::env;
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use std::path::Path;
use std::path::PathBuf;
use tokio::time::sleep;
use std::fs;
use std::io::{Write, BufRead, BufReader};
use std::thread::{self, JoinHandle};

use tonic::{transport::Server, Request, Response, Status};
use tokio::sync::mpsc;

use crate::proto::opentelemetry::collector::trace::v1::{
    trace_service_server::{TraceService, TraceServiceServer},
    ExportTraceServiceRequest, ExportTraceServiceResponse,
};

use crate::proto::opentelemetry::collector::metrics::v1::{
    metrics_service_server::MetricsService,
    ExportMetricsServiceRequest, ExportMetricsServiceResponse,
};

use crate::proto::opentelemetry::collector::logs::v1::{
    logs_service_server::LogsService,
    ExportLogsServiceRequest, ExportLogsServiceResponse,
};

/// A test receiver that implements the OTLP trace service
#[derive(Debug)]
pub struct TestTraceReceiver {
    pub request_rx: mpsc::Sender<ExportTraceServiceRequest>,
}

#[tonic::async_trait]
impl TraceService for TestTraceReceiver {
    async fn export(
        &self,
        request: Request<ExportTraceServiceRequest>,
    ) -> Result<Response<ExportTraceServiceResponse>, Status> {
        let request_inner = request.into_inner();
        
        // Forward the received request to the test channel
        if let Err(err) = self.request_rx.send(request_inner).await {
            return Err(Status::internal(format!("Failed to send trace data to test channel: {}", err)));
        }
        
        // Return success response
        Ok(Response::new(ExportTraceServiceResponse::default()))
    }
}

/// A test receiver that implements the OTLP metrics service
#[derive(Debug)]
pub struct TestMetricsReceiver {
    pub request_rx: mpsc::Sender<ExportMetricsServiceRequest>,
}

#[tonic::async_trait]
impl MetricsService for TestMetricsReceiver {
    async fn export(
        &self,
        request: Request<ExportMetricsServiceRequest>,
    ) -> Result<Response<ExportMetricsServiceResponse>, Status> {
        let request_inner = request.into_inner();
        
        // Forward the received request to the test channel
        if let Err(err) = self.request_rx.send(request_inner).await {
            return Err(Status::internal(format!("Failed to send metrics data to test channel: {}", err)));
        }
        
        // Return success response
        Ok(Response::new(ExportMetricsServiceResponse::default()))
    }
}

/// A test receiver that implements the OTLP logs service
#[derive(Debug)]
pub struct TestLogsReceiver {
    pub request_rx: mpsc::Sender<ExportLogsServiceRequest>,
}

#[tonic::async_trait]
impl LogsService for TestLogsReceiver {
    async fn export(
        &self,
        request: Request<ExportLogsServiceRequest>,
    ) -> Result<Response<ExportLogsServiceResponse>, Status> {
        let request_inner = request.into_inner();
        
        // Forward the received request to the test channel
        if let Err(err) = self.request_rx.send(request_inner).await {
            return Err(Status::internal(format!("Failed to send logs data to test channel: {}", err)));
        }
        
        // Return success response
        Ok(Response::new(ExportLogsServiceResponse::default()))
    }
}

/// A helper struct to manage the collector process
pub struct CollectorProcess {
    process: Child,
    config_path: PathBuf,
    // Add fields to store stdout and stderr thread handles
    stdout_handle: Option<JoinHandle<()>>,
    stderr_handle: Option<JoinHandle<()>>,
}

impl CollectorProcess {
    /// Start a collector with the given configuration
    pub async fn start<T: AsRef<Path>>(collector_path: T, config_content: &str) -> Result<Self, String> {
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
        let stdout = process.stdout.take()
            .ok_or_else(|| "Failed to capture process stdout".to_string())?;
        
        let stderr = process.stderr.take()
            .ok_or_else(|| "Failed to capture process stderr".to_string())?;
        
        // Create threads to read from stdout and stderr and write to test stderr
        let stdout_handle = thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(line) = line {
                    eprintln!("[Collector stdout] {}", line);
                }
            }
        });
        
        let stderr_handle = thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                if let Ok(line) = line {
                    eprintln!("[Collector stderr] {}", line);
                }
            }
        });
        
        // Wait for collector to start up
        sleep(Duration::from_secs(2)).await;
        
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

/// Start a test receiver server on any available port
pub async fn start_test_receiver() -> Result<
    (
        tokio::task::JoinHandle<Result<(), tonic::transport::Error>>, 
        mpsc::Receiver<ExportTraceServiceRequest>,
        u16 // Return the actual port number that was assigned
    ), 
    String
> {
    // Bind to a specific address with port 0 for dynamic port allocation
    // We use the address string directly, no need to parse it
    let addr = "127.0.0.1:0";
    
    // Create a TCP listener with port 0 to get an available port
    let listener = tokio::net::TcpListener::bind(addr).await
        .map_err(|e| format!("Failed to bind listener: {}", e))?;
    
    // Get the assigned port
    let port = listener.local_addr()
        .map_err(|e| format!("Failed to get local address: {}", e))?
        .port();
    
    // Create a channel for receiving the exported data in tests
    let (request_tx, request_rx) = mpsc::channel(100);
    
    let server = TraceServiceServer::new(TestTraceReceiver { 
        request_rx: request_tx,
    });
    
    // Convert the listener to a stream of connections
    let incoming = tokio_stream::wrappers::TcpListenerStream::new(listener);
    
    // Create our server
    let handle = tokio::spawn(async move {
        Server::builder()
            .add_service(server)
            .serve_with_incoming(incoming)
            .await
    });
    
    // Allow a little time for the server to be fully ready
    sleep(Duration::from_millis(100)).await;
    
    Ok((handle, request_rx, port))
}

/// Configuration generator for OTLP to OTLP test case
pub fn generate_otlp_to_otlp_config(receiver_port: u16, exporter_port: u16) -> String {
    format!(r#"
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

service:
  pipelines:
    traces:
      receivers: [otlp]
      exporters: [otlp]
    metrics:
      receivers: [otlp]
      exporters: [otlp]
    logs:
      receivers: [otlp]
      exporters: [otlp]
"#)
}
