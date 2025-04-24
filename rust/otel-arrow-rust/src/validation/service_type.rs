// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// Service type abstractions for validation testing.  This abstraction
// enables Logs, Traces, and Metrics to look generically similar.

use snafu::ResultExt;
use std::fmt::Debug;
use tokio::sync::mpsc;

use super::error;
use crate::validation::tcp_stream;

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
    async fn connect_client(endpoint: String) -> error::Result<Self::Client>;

    /// Send data through the client
    async fn send_data(
        client: &mut Self::Client,
        request: Self::Request,
    ) -> error::Result<Self::Response>;
}

/// A trait that abstracts over the output side of service types (server operations)
pub trait ServiceOutputType: Debug + Send + Sync + 'static {
    /// The request type for this service
    type Request: Clone + PartialEq + Send + Sync + 'static;

    /// Server type to add to the tonic server
    type Server;

    /// The name of this service type (for logging and identification)
    fn signal() -> &'static str;

    /// The protocol used by this service type (e.g., "otlp").  This
    /// is expected to match the receiver and exporter name used in
    /// the test, hence OTAP uses "otelarrow".
    fn protocol() -> &'static str;

    /// Create a server with the given receiver and listener stream
    fn create_server(
        receiver: TestReceiver<Self::Request>,
        incoming: crate::validation::tcp_stream::ShutdownableTcpListenerStream,
    ) -> tokio::task::JoinHandle<error::Result<()>>;

    /// Start a service-specific receiver, wrapping create_service_server.
    async fn start_receiver(
        listener: tokio::net::TcpListener,
    ) -> error::Result<(
        tokio::task::JoinHandle<error::Result<()>>,
        mpsc::Receiver<Self::Request>,
        tokio::sync::oneshot::Sender<()>,
    )>
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
async fn create_listener_with_port() -> error::Result<(tokio::net::TcpListener, u16)> {
    // Bind to a specific address with port 0 for dynamic port allocation
    let addr = "127.0.0.1:0";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .context(error::InputOutputSnafu { desc: "bind" })?;

    // Get the assigned port
    let port = listener
        .local_addr()
        .context(error::InputOutputSnafu {
            desc: "local_address",
        })?
        .port();

    Ok((listener, port))
}

/// Helper function to start a test receiver for any service output type
pub async fn start_test_receiver<T: ServiceOutputType>() -> error::Result<(
    tokio::task::JoinHandle<error::Result<()>>,
    mpsc::Receiver<T::Request>,
    u16,                              // actual port number that was assigned
    tokio::sync::oneshot::Sender<()>, // shutdown channel
)> {
    // Create listener with dynamically allocated port
    let (listener, port) = create_listener_with_port().await?;

    // Start the service-specific receiver
    let (handle, request_rx, shutdown_tx) = T::start_receiver(listener).await?;

    Ok((handle, request_rx, port, shutdown_tx))
}

/// Generic helper function to create a TCP server for any service output type
async fn create_service_server<T: ServiceOutputType + ?Sized>(
    listener: tokio::net::TcpListener,
) -> error::Result<(
    tokio::task::JoinHandle<error::Result<()>>,
    mpsc::Receiver<T::Request>,
    tokio::sync::oneshot::Sender<()>,
)> {
    // Create a channel for receiving data
    let (request_tx, request_rx) = mpsc::channel::<T::Request>(100);

    // Create a test receiver
    let receiver = TestReceiver { request_tx };

    // Convert the listener to a stream of connections with a shutdown channel
    let (incoming, shutdown_tx) = tcp_stream::create_shutdownable_tcp_listener(listener);

    // Create our server - we need to delegate to the service-specific functions
    // since we can't construct the server generically
    let handle = T::create_server(receiver, incoming);

    Ok((handle, request_rx, shutdown_tx))
}
