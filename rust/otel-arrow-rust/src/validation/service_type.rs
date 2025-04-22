// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// Service type abstractions for validation testing
use std::fmt::Debug;
use tokio::sync::mpsc;

use crate::validation::tcp_stream::create_shutdownable_tcp_listener;

/// A trait that abstracts over the input side of service types (client operations)
pub trait ServiceInputType: Debug + Send + Sync + 'static {
    /// The request type for this service
    type Request: Clone + PartialEq + Send + Sync + 'static;

    /// The response type for this service
    type Response: Default + Send + 'static;

    /// The client type for this service
    type Client;

    /// The name of this service type (for logging and identification)
    fn signal() -> &'static str;

    /// The protocol used by this service type (e.g., "otlp")
    fn protocol() -> &'static str;

    /// Create a new client for this service
    async fn connect_client(endpoint: String) -> Result<Self::Client, tonic::transport::Error>;

    /// Send data through the client
    async fn send_data(
        client: &mut Self::Client,
        request: Self::Request,
    ) -> Result<Self::Response, tonic::Status>;
}

/// A trait that abstracts over the output side of service types (server operations)
pub trait ServiceOutputType: Debug + Send + Sync + 'static {
    /// The request type for this service
    type Request: Clone + PartialEq + Send + Sync + 'static;

    /// Server type to add to the tonic server
    type Server;

    /// The name of this service type (for logging and identification)
    fn signal() -> &'static str;

    /// The protocol used by this service type (e.g., "otlp")
    fn protocol() -> &'static str;

    /// Create a server with the given receiver and listener stream
    /// Returns a JoinHandle for the server task and a shutdown channel
    fn create_server(
        receiver: TestReceiver<Self::Request>,
        incoming: crate::validation::tcp_stream::ShutdownableTcpListenerStream,
    ) -> tokio::task::JoinHandle<Result<(), tonic::transport::Error>>;

    /// Start a service-specific receiver
    async fn start_receiver(
        listener: tokio::net::TcpListener,
    ) -> Result<
        (
            tokio::task::JoinHandle<Result<(), tonic::transport::Error>>,
            mpsc::Receiver<Self::Request>,
            tokio::sync::oneshot::Sender<()>,
        ),
        String,
    >
    where
        Self: Sized,
    {
        create_service_server::<Self>(listener).await
    }
}

/// Generic test receiver that can be used for any service
#[derive(Debug, Clone)]
pub struct TestReceiver<T> {
    pub request_tx: mpsc::Sender<T>,
}

impl<T: Send + 'static> TestReceiver<T> {
    /// Generic method to process export requests for any service type
    pub async fn process_export_request<R>(
        &self,
        request: tonic::Request<T>,
        service_name: &str,
    ) -> Result<tonic::Response<R>, tonic::Status>
    where
        R: Default,
    {
        let request_inner = request.into_inner();

        // Forward the received request to the test channel
        if let Err(err) = self.request_tx.send(request_inner).await {
            return Err(tonic::Status::internal(format!(
                "Failed to send {} data to test channel: {}",
                service_name, err
            )));
        }

        // Return success response
        Ok(tonic::Response::new(R::default()))
    }
}

/// Helper function to create a TCP listener with a dynamically allocated port
async fn create_listener_with_port() -> Result<(tokio::net::TcpListener, u16), String> {
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

    Ok((listener, port))
}

/// Helper function to start a test receiver for any service output type
pub async fn start_test_receiver<T: ServiceOutputType>() -> Result<
    (
        tokio::task::JoinHandle<Result<(), tonic::transport::Error>>,
        mpsc::Receiver<T::Request>,
        u16,                              // actual port number that was assigned
        tokio::sync::oneshot::Sender<()>, // shutdown channel
    ),
    String,
> {
    // Create listener with dynamically allocated port
    let (listener, port) = create_listener_with_port().await?;

    // Start the service-specific receiver
    let (handle, request_rx, shutdown_tx) = T::start_receiver(listener).await?;

    Ok((handle, request_rx, port, shutdown_tx))
}

/// Generic helper function to create a TCP server for any service output type
async fn create_service_server<T: ServiceOutputType + ?Sized>(
    listener: tokio::net::TcpListener,
) -> Result<
    (
        tokio::task::JoinHandle<Result<(), tonic::transport::Error>>,
        mpsc::Receiver<T::Request>,
        tokio::sync::oneshot::Sender<()>,
    ),
    String,
> {
    // Create a channel for receiving data
    let (request_tx, request_rx) = mpsc::channel::<T::Request>(100);

    // Create a test receiver
    let receiver = TestReceiver { request_tx };

    // Convert the listener to a stream of connections with a shutdown channel
    let (incoming, shutdown_tx) = create_shutdownable_tcp_listener(listener);

    // Create our server - we need to delegate to the service-specific functions
    // since we can't construct the server generically
    let handle = T::create_server(receiver, incoming);

    Ok((handle, request_rx, shutdown_tx))
}
