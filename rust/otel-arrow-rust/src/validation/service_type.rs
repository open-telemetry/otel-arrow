// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// Service type abstractions for validation testing
use std::fmt::Debug;
use tokio::sync::mpsc;
use tokio_stream::wrappers::TcpListenerStream;

/// A trait that abstracts over the different OTLP service types
pub trait ServiceType: Debug + Send + Sync + 'static {
    /// The request type for this service
    type Request: Clone + PartialEq + Send + Sync + 'static;

    /// The response type for this service
    type Response: Default + Send + 'static;

    /// The client type for this service
    type Client;

    /// Server type to add to the tonic server
    type Server;

    /// The name of this service type (for logging and identification)
    fn name() -> &'static str;

    /// Create a new client for this service
    async fn connect_client(endpoint: String) -> Result<Self::Client, tonic::transport::Error>;

    /// Send data through the client
    async fn send_data(
        client: &mut Self::Client,
        request: Self::Request,
    ) -> Result<Self::Response, tonic::Status>;

    /// Create a server with the given receiver and listener stream
    fn create_server(
        receiver: TestReceiver<Self::Request>,
        incoming: TcpListenerStream,
    ) -> tokio::task::JoinHandle<Result<(), tonic::transport::Error>>;

    /// Start a service-specific receiver
    async fn start_receiver(
        listener: tokio::net::TcpListener,
    ) -> Result<
        (
            tokio::task::JoinHandle<Result<(), tonic::transport::Error>>,
            mpsc::Receiver<Self::Request>,
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
#[derive(Debug)]
pub struct TestReceiver<T> {
    pub request_tx: mpsc::Sender<T>,
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

/// Helper function to start a test receiver for any service type
pub async fn start_test_receiver<T: ServiceType>() -> Result<
    (
        tokio::task::JoinHandle<Result<(), tonic::transport::Error>>,
        mpsc::Receiver<T::Request>,
        u16, // actual port number that was assigned
    ),
    String,
> {
    // Create listener with dynamically allocated port
    let (listener, port) = create_listener_with_port().await?;

    // Start the service-specific receiver
    let (handle, request_rx) = T::start_receiver(listener).await?;

    Ok((handle, request_rx, port))
}

/// Generic helper function to create a TCP server for any OTLP service type
async fn create_service_server<T: ServiceType + ?Sized>(
    listener: tokio::net::TcpListener,
) -> Result<
    (
        tokio::task::JoinHandle<Result<(), tonic::transport::Error>>,
        mpsc::Receiver<T::Request>,
    ),
    String,
> {
    // Create a channel for receiving data
    let (request_tx, request_rx) = mpsc::channel::<T::Request>(100);

    // Create a test receiver
    let receiver = TestReceiver { request_tx };

    // Convert the listener to a stream of connections
    let incoming = TcpListenerStream::new(listener);

    // Create our server - we need to delegate to the service-specific functions
    // since we can't construct the server generically
    let handle = T::create_server(receiver, incoming);

    Ok((handle, request_rx))
}
