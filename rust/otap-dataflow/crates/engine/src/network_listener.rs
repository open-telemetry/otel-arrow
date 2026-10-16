// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Network ingress planning and per-pipeline listener assignment.

use futures::Stream;
use otel_arrow_dfe_config::{LogicalPipelineDeploymentKey, NodeId};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::future::Future;
use std::io;
use std::net::{SocketAddr, TcpListener as StdTcpListener};
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc as std_mpsc;
use std::sync::{Arc, Weak};
use std::task::{Context, Poll};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Notify, mpsc};

const LISTEN_BACKLOG: i32 = 8192;
const SHARED_ENDPOINT_CAPACITY: usize = 4096;
const MIN_SHARED_CONSUMER_CAPACITY: usize = 64;
const MAX_SHARED_CONSUMER_CAPACITY: usize = 1024;

/// Transport protocol associated with a declared network endpoint.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NetworkTransport {
    /// Transmission Control Protocol.
    Tcp,
    /// User Datagram Protocol.
    Udp,
}

/// One receiver-owned network endpoint in a logical pipeline.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NetworkEndpoint {
    /// Receiver node that declared the endpoint.
    pub receiver_node_id: NodeId,
    /// Address bound by the receiver.
    pub bind_address: SocketAddr,
    /// Transport protocol.
    pub transport: NetworkTransport,
}

impl NetworkEndpoint {
    /// Creates an endpoint declaration.
    #[must_use]
    pub fn new(
        receiver_node_id: NodeId,
        bind_address: SocketAddr,
        transport: NetworkTransport,
    ) -> Self {
        Self {
            receiver_node_id,
            bind_address,
            transport,
        }
    }
}

/// One pipeline placement member that may own network endpoints.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NetworkListenerMember {
    /// Logical CPU core assigned to this pipeline instance.
    pub core_id: usize,
    /// NUMA node containing the core, when known.
    pub numa_node_id: Option<usize>,
}

/// Complete network requirements for one pipeline placement generation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NetworkListenerPlan {
    /// Logical pipeline placement deployment that owns this plan.
    pub deployment_key: LogicalPipelineDeploymentKey,
    /// Pipeline instances participating in every declared endpoint.
    pub members: Vec<NetworkListenerMember>,
    /// Receiver endpoints declared by the pipeline.
    pub endpoints: Vec<NetworkEndpoint>,
}

impl NetworkListenerPlan {
    /// Creates and validates a pipeline network plan.
    pub fn new(
        deployment_key: LogicalPipelineDeploymentKey,
        mut members: Vec<NetworkListenerMember>,
        mut endpoints: Vec<NetworkEndpoint>,
    ) -> io::Result<Self> {
        members.sort_by_key(|member| member.core_id);
        if members
            .windows(2)
            .any(|pair| pair[0].core_id == pair[1].core_id)
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "network listener plan contains duplicate core members",
            ));
        }

        endpoints.sort();
        let mut physical_endpoints = HashMap::with_capacity(endpoints.len());
        for endpoint in &endpoints {
            let physical_key = (endpoint.bind_address, endpoint.transport);
            if let Some(previous_node) =
                physical_endpoints.insert(physical_key, endpoint.receiver_node_id.clone())
            {
                return Err(io::Error::new(
                    io::ErrorKind::AddrInUse,
                    format!(
                        "receiver nodes `{previous_node}` and `{}` both declare {} {:?}",
                        endpoint.receiver_node_id, endpoint.bind_address, endpoint.transport
                    ),
                ));
            }
        }

        Ok(Self {
            deployment_key,
            members,
            endpoints,
        })
    }

    fn contains_core(&self, core_id: usize) -> bool {
        self.members.iter().any(|member| member.core_id == core_id)
    }

    fn contains_endpoint(&self, expected: &NetworkEndpoint) -> bool {
        self.endpoints.binary_search(expected).is_ok()
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct RegistrationKey {
    core_id: usize,
    endpoint: NetworkEndpoint,
}

struct NetworkListenerDeploymentState {
    plan: Option<Arc<NetworkListenerPlan>>,
    backend: Arc<NetworkListenerBackend>,
    next_registration_id: AtomicU64,
    registrations: Mutex<HashMap<RegistrationKey, u64>>,
}

impl std::fmt::Debug for NetworkListenerDeploymentState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("NetworkListenerDeploymentState")
            .field(
                "deployment_key",
                &self.plan.as_ref().map(|plan| &plan.deployment_key),
            )
            .field("members", &self.plan.as_ref().map(|plan| &plan.members))
            .field("endpoints", &self.plan.as_ref().map(|plan| &plan.endpoints))
            .field("backend", &self.backend.name())
            .finish()
    }
}

enum NetworkListenerBackend {
    SocketPerListener,
    SharedSocket {
        service: Mutex<Option<Arc<SharedListenerService>>>,
    },
}

impl NetworkListenerBackend {
    fn platform_default() -> Self {
        if cfg!(windows) {
            Self::shared_socket()
        } else {
            Self::SocketPerListener
        }
    }

    fn shared_socket() -> Self {
        Self::SharedSocket {
            service: Mutex::new(None),
        }
    }

    const fn name(&self) -> &'static str {
        match self {
            Self::SocketPerListener => "socket-per-listener",
            Self::SharedSocket { .. } => "shared-socket",
        }
    }

    fn shared_listener_service(&self) -> io::Result<Arc<SharedListenerService>> {
        let Self::SharedSocket { service } = self else {
            return Err(io::Error::other(
                "shared listener service requested from socket-per-listener backend",
            ));
        };
        let mut service = service.lock();
        if let Some(existing) = service.as_ref() {
            return Ok(Arc::clone(existing));
        }
        let started = SharedListenerService::start()?;
        *service = Some(Arc::clone(&started));
        Ok(started)
    }
}

/// Controller-scoped registry for logical network listener deployments.
#[derive(Clone)]
pub struct NetworkListenerBroker {
    inner: Arc<NetworkListenerBrokerInner>,
}

struct NetworkListenerBrokerInner {
    deployments: Mutex<HashMap<LogicalPipelineDeploymentKey, Weak<NetworkListenerDeploymentState>>>,
    backend: Arc<NetworkListenerBackend>,
}

impl Default for NetworkListenerBroker {
    fn default() -> Self {
        Self {
            inner: Arc::new(NetworkListenerBrokerInner {
                deployments: Mutex::new(HashMap::new()),
                backend: Arc::new(NetworkListenerBackend::platform_default()),
            }),
        }
    }
}

impl NetworkListenerBroker {
    /// Creates an empty listener broker.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(test)]
    fn with_backend(backend: NetworkListenerBackend) -> Self {
        Self {
            inner: Arc::new(NetworkListenerBrokerInner {
                deployments: Mutex::new(HashMap::new()),
                backend: Arc::new(backend),
            }),
        }
    }

    /// Returns the listener set assigned to one member of a network plan.
    pub fn listener_set_for(
        &self,
        plan: Arc<NetworkListenerPlan>,
        core_id: usize,
    ) -> io::Result<NetworkListenerSet> {
        if !plan.contains_core(core_id) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "core {core_id} is not a member of network listener deployment {:?}",
                    plan.deployment_key
                ),
            ));
        }

        let deployment = {
            let mut deployments = self.inner.deployments.lock();
            deployments.retain(|_, deployment| deployment.strong_count() > 0);
            match deployments
                .get(&plan.deployment_key)
                .and_then(Weak::upgrade)
            {
                Some(existing) => {
                    if existing.plan.as_deref() != Some(plan.as_ref()) {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            format!(
                                "network listener deployment {:?} was registered with a different plan",
                                plan.deployment_key
                            ),
                        ));
                    }
                    existing
                }
                None => {
                    let deployment = Arc::new(NetworkListenerDeploymentState {
                        plan: Some(Arc::clone(&plan)),
                        backend: Arc::clone(&self.inner.backend),
                        next_registration_id: AtomicU64::new(1),
                        registrations: Mutex::new(HashMap::new()),
                    });
                    _ = deployments
                        .insert(plan.deployment_key.clone(), Arc::downgrade(&deployment));
                    deployment
                }
            }
        };

        Ok(NetworkListenerSet {
            deployment,
            core_id,
        })
    }
}

/// Network endpoints available to one runtime pipeline instance.
#[derive(Clone)]
pub struct NetworkListenerSet {
    deployment: Arc<NetworkListenerDeploymentState>,
    core_id: usize,
}

impl std::fmt::Debug for NetworkListenerSet {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("NetworkListenerSet")
            .field("deployment_key", &self.deployment_key())
            .field("core_id", &self.core_id)
            .finish()
    }
}

impl NetworkListenerSet {
    /// Creates unrestricted listener state for test-only standalone construction.
    #[cfg(any(test, feature = "test-utils"))]
    #[must_use]
    pub fn standalone() -> Self {
        Self {
            deployment: Arc::new(NetworkListenerDeploymentState {
                plan: None,
                backend: Arc::new(NetworkListenerBackend::SocketPerListener),
                next_registration_id: AtomicU64::new(1),
                registrations: Mutex::new(HashMap::new()),
            }),
            core_id: 0,
        }
    }

    /// Returns the logical deployment key, when this is a planned listener set.
    #[must_use]
    pub fn deployment_key(&self) -> Option<&LogicalPipelineDeploymentKey> {
        self.deployment
            .plan
            .as_ref()
            .map(|plan| &plan.deployment_key)
    }

    /// Returns the immutable network plan, when this is a planned listener set.
    #[must_use]
    pub fn plan(&self) -> Option<&NetworkListenerPlan> {
        self.deployment.plan.as_deref()
    }

    /// Creates the TCP listener assigned to one receiver instance.
    pub fn tcp_listener(
        &self,
        receiver_node_id: NodeId,
        bind_address: SocketAddr,
    ) -> io::Result<NetworkTcpListener> {
        if let Some(plan) = &self.deployment.plan {
            debug_assert!(plan.contains_core(self.core_id));
            let endpoint =
                NetworkEndpoint::new(receiver_node_id, bind_address, NetworkTransport::Tcp);
            if !plan.contains_endpoint(&endpoint) {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!(
                        "receiver `{}` did not declare TCP endpoint {bind_address}",
                        endpoint.receiver_node_id
                    ),
                ));
            }
            self.create_tcp_listener(endpoint, bind_address)
        } else {
            self.create_tcp_listener(
                NetworkEndpoint::new(receiver_node_id, bind_address, NetworkTransport::Tcp),
                bind_address,
            )
        }
    }

    fn create_tcp_listener(
        &self,
        endpoint: NetworkEndpoint,
        bind_address: SocketAddr,
    ) -> io::Result<NetworkTcpListener> {
        let key = RegistrationKey {
            core_id: self.core_id,
            endpoint,
        };
        let registration_id = self
            .deployment
            .next_registration_id
            .fetch_add(1, Ordering::Relaxed);
        {
            let mut registrations = self.deployment.registrations.lock();
            if registrations.contains_key(&key) {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!(
                        "TCP endpoint {bind_address} was already created for core {}",
                        self.core_id
                    ),
                ));
            }
            let _ = registrations.insert(key.clone(), registration_id);
        }

        let result = match self.deployment.backend.as_ref() {
            NetworkListenerBackend::SocketPerListener => create_listener(bind_address)
                .and_then(TcpListener::from_std)
                .map(NetworkTcpListenerInner::SocketPerListener),
            backend @ NetworkListenerBackend::SharedSocket { .. } => {
                let plan = self
                    .deployment
                    .plan
                    .as_ref()
                    .expect("shared listener backend requires a network plan");
                backend.shared_listener_service().and_then(|service| {
                    service
                        .register(
                            plan.deployment_key.clone(),
                            key.endpoint.clone(),
                            registration_id,
                            shared_consumer_capacity(plan.members.len()),
                        )
                        .map(NetworkTcpListenerInner::SharedSocket)
                })
            }
        };
        match result {
            Ok(inner) => Ok(NetworkTcpListener {
                inner,
                deployment: Arc::clone(&self.deployment),
                key,
                registration_id,
            }),
            Err(error) => {
                self.deployment.release_registration(&key, registration_id);
                Err(error)
            }
        }
    }
}

impl NetworkListenerDeploymentState {
    fn release_registration(&self, key: &RegistrationKey, registration_id: u64) {
        let mut registrations = self.registrations.lock();
        if registrations.get(key) == Some(&registration_id) {
            let _ = registrations.remove(key);
        }
    }
}

fn shared_consumer_capacity(consumer_count: usize) -> usize {
    (SHARED_ENDPOINT_CAPACITY / consumer_count.max(1))
        .clamp(MIN_SHARED_CONSUMER_CAPACITY, MAX_SHARED_CONSUMER_CAPACITY)
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct SharedEndpointKey {
    deployment_key: LogicalPipelineDeploymentKey,
    endpoint: NetworkEndpoint,
}

#[derive(Clone)]
struct SharedAcceptError {
    kind: io::ErrorKind,
    message: Arc<str>,
}

impl SharedAcceptError {
    fn from_io(error: &io::Error) -> Self {
        Self {
            kind: error.kind(),
            message: Arc::from(error.to_string()),
        }
    }

    fn into_io(self) -> io::Error {
        io::Error::new(self.kind, self.message.to_string())
    }
}

struct SharedTcpListener {
    receiver: mpsc::Receiver<(TcpStream, SocketAddr)>,
    local_addr: SocketAddr,
    capacity_available: Arc<Notify>,
    terminal_error: Arc<Mutex<Option<SharedAcceptError>>>,
    _registration: SharedListenerRegistration,
}

impl SharedTcpListener {
    fn poll_accept(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<(TcpStream, SocketAddr)>> {
        let poll = self.receiver.poll_recv(cx);
        match poll {
            Poll::Ready(Some(connection)) => {
                self.capacity_available.notify_one();
                Poll::Ready(Ok(connection))
            }
            Poll::Ready(None) => {
                let error = self.terminal_error.lock().clone().map_or_else(
                    || io::Error::new(io::ErrorKind::BrokenPipe, "shared network listener stopped"),
                    SharedAcceptError::into_io,
                );
                Poll::Ready(Err(error))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

struct SharedListenerRegistration {
    service: Arc<SharedListenerService>,
    endpoint_key: SharedEndpointKey,
    endpoint_id: u64,
    registration_id: u64,
}

impl Drop for SharedListenerRegistration {
    fn drop(&mut self) {
        let _ = self
            .service
            .command_tx
            .send(SharedListenerServiceCommand::Deregister {
                endpoint_key: self.endpoint_key.clone(),
                endpoint_id: self.endpoint_id,
                registration_id: self.registration_id,
            });
    }
}

struct SharedListenerService {
    command_tx: mpsc::UnboundedSender<SharedListenerServiceCommand>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl SharedListenerService {
    fn start() -> io::Result<Arc<Self>> {
        let (command_tx, command_rx) = mpsc::unbounded_channel();
        let (ready_tx, ready_rx) = std_mpsc::sync_channel(1);
        let service_command_tx = command_tx.clone();
        let thread = std::thread::Builder::new()
            .name("network-listener-broker".to_owned())
            .spawn(move || {
                let runtime = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(runtime) => runtime,
                    Err(error) => {
                        let _ = ready_tx.send(Err(error));
                        return;
                    }
                };
                let local_set = tokio::task::LocalSet::new();
                let _ = ready_tx.send(Ok(()));
                runtime.block_on(
                    local_set
                        .run_until(run_shared_listener_service(command_rx, service_command_tx)),
                );
            })
            .map_err(|error| {
                io::Error::new(
                    error.kind(),
                    format!("failed to spawn network listener broker thread: {error}"),
                )
            })?;

        match ready_rx.recv() {
            Ok(Ok(())) => Ok(Arc::new(Self {
                command_tx,
                thread: Some(thread),
            })),
            Ok(Err(error)) => {
                let _ = thread.join();
                Err(error)
            }
            Err(_) => {
                let _ = thread.join();
                Err(io::Error::other(
                    "network listener broker thread exited during startup",
                ))
            }
        }
    }

    fn register(
        self: &Arc<Self>,
        deployment_key: LogicalPipelineDeploymentKey,
        endpoint: NetworkEndpoint,
        registration_id: u64,
        capacity: usize,
    ) -> io::Result<SharedTcpListener> {
        let endpoint_key = SharedEndpointKey {
            deployment_key,
            endpoint,
        };
        let (response_tx, response_rx) = std_mpsc::sync_channel(1);
        self.command_tx
            .send(SharedListenerServiceCommand::Register {
                endpoint_key: endpoint_key.clone(),
                registration_id,
                capacity,
                response_tx,
            })
            .map_err(|_| {
                io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "network listener broker thread stopped",
                )
            })?;
        let parts = response_rx.recv().map_err(|_| {
            io::Error::new(
                io::ErrorKind::BrokenPipe,
                "network listener broker registration was interrupted",
            )
        })??;
        Ok(SharedTcpListener {
            receiver: parts.receiver,
            local_addr: parts.local_addr,
            capacity_available: parts.capacity_available,
            terminal_error: parts.terminal_error,
            _registration: SharedListenerRegistration {
                service: Arc::clone(self),
                endpoint_key,
                endpoint_id: parts.endpoint_id,
                registration_id,
            },
        })
    }
}

impl Drop for SharedListenerService {
    fn drop(&mut self) {
        let _ = self.command_tx.send(SharedListenerServiceCommand::Shutdown);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

enum SharedListenerServiceCommand {
    Register {
        endpoint_key: SharedEndpointKey,
        registration_id: u64,
        capacity: usize,
        response_tx: std_mpsc::SyncSender<io::Result<SharedConsumerParts>>,
    },
    Deregister {
        endpoint_key: SharedEndpointKey,
        endpoint_id: u64,
        registration_id: u64,
    },
    EndpointStopped {
        endpoint_key: SharedEndpointKey,
        endpoint_id: u64,
    },
    Shutdown,
}

struct SharedConsumerParts {
    receiver: mpsc::Receiver<(TcpStream, SocketAddr)>,
    local_addr: SocketAddr,
    capacity_available: Arc<Notify>,
    terminal_error: Arc<Mutex<Option<SharedAcceptError>>>,
    endpoint_id: u64,
}

struct SharedEndpointHandle {
    endpoint_id: u64,
    command_tx: mpsc::UnboundedSender<SharedEndpointCommand>,
    consumer_count: usize,
    local_addr: SocketAddr,
    capacity_available: Arc<Notify>,
    terminal_error: Arc<Mutex<Option<SharedAcceptError>>>,
}

enum SharedEndpointCommand {
    Add {
        registration_id: u64,
        sender: mpsc::Sender<(TcpStream, SocketAddr)>,
    },
    Remove {
        registration_id: u64,
    },
    Shutdown,
}

struct SharedConsumer {
    registration_id: u64,
    sender: mpsc::Sender<(TcpStream, SocketAddr)>,
}

async fn run_shared_listener_service(
    mut command_rx: mpsc::UnboundedReceiver<SharedListenerServiceCommand>,
    command_tx: mpsc::UnboundedSender<SharedListenerServiceCommand>,
) {
    let mut endpoints = HashMap::<SharedEndpointKey, SharedEndpointHandle>::new();
    let mut next_endpoint_id = 1_u64;
    while let Some(command) = command_rx.recv().await {
        match command {
            SharedListenerServiceCommand::Register {
                endpoint_key,
                registration_id,
                capacity,
                response_tx,
            } => {
                register_shared_consumer(
                    &mut endpoints,
                    &mut next_endpoint_id,
                    endpoint_key,
                    registration_id,
                    capacity,
                    response_tx,
                    &command_tx,
                );
            }
            SharedListenerServiceCommand::Deregister {
                endpoint_key,
                endpoint_id,
                registration_id,
            } => {
                if let Some(endpoint) = endpoints.get_mut(&endpoint_key)
                    && endpoint.endpoint_id == endpoint_id
                {
                    endpoint.consumer_count = endpoint.consumer_count.saturating_sub(1);
                    let _ = endpoint
                        .command_tx
                        .send(SharedEndpointCommand::Remove { registration_id });
                    if endpoint.consumer_count == 0 {
                        let endpoint = endpoints
                            .remove(&endpoint_key)
                            .expect("shared endpoint checked above");
                        let _ = endpoint.command_tx.send(SharedEndpointCommand::Shutdown);
                    }
                }
            }
            SharedListenerServiceCommand::EndpointStopped {
                endpoint_key,
                endpoint_id,
            } => {
                if endpoints
                    .get(&endpoint_key)
                    .map(|endpoint| endpoint.endpoint_id)
                    == Some(endpoint_id)
                {
                    let _ = endpoints.remove(&endpoint_key);
                }
            }
            SharedListenerServiceCommand::Shutdown => break,
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn register_shared_consumer(
    endpoints: &mut HashMap<SharedEndpointKey, SharedEndpointHandle>,
    next_endpoint_id: &mut u64,
    endpoint_key: SharedEndpointKey,
    registration_id: u64,
    capacity: usize,
    response_tx: std_mpsc::SyncSender<io::Result<SharedConsumerParts>>,
    service_command_tx: &mpsc::UnboundedSender<SharedListenerServiceCommand>,
) {
    let (sender, receiver) = mpsc::channel(capacity);
    if let Some(endpoint) = endpoints.get_mut(&endpoint_key) {
        if endpoint
            .command_tx
            .send(SharedEndpointCommand::Add {
                registration_id,
                sender: sender.clone(),
            })
            .is_ok()
        {
            endpoint.consumer_count += 1;
            let parts = SharedConsumerParts {
                receiver,
                local_addr: endpoint.local_addr,
                capacity_available: Arc::clone(&endpoint.capacity_available),
                terminal_error: Arc::clone(&endpoint.terminal_error),
                endpoint_id: endpoint.endpoint_id,
            };
            if response_tx.send(Ok(parts)).is_err() {
                endpoint.consumer_count -= 1;
                let _ = endpoint
                    .command_tx
                    .send(SharedEndpointCommand::Remove { registration_id });
            }
            return;
        }
        let _ = endpoints.remove(&endpoint_key);
    }

    let cleanup_key = endpoint_key.clone();
    let result = create_listener(endpoint_key.endpoint.bind_address)
        .and_then(TcpListener::from_std)
        .and_then(|listener| {
            let local_addr = listener.local_addr()?;
            let endpoint_id = *next_endpoint_id;
            *next_endpoint_id = next_endpoint_id.wrapping_add(1);
            let capacity_available = Arc::new(Notify::new());
            let terminal_error = Arc::new(Mutex::new(None));
            let (endpoint_command_tx, endpoint_command_rx) = mpsc::unbounded_channel();
            let consumer = SharedConsumer {
                registration_id,
                sender,
            };
            _ = tokio::task::spawn_local(run_shared_endpoint(
                endpoint_key.clone(),
                endpoint_id,
                listener,
                endpoint_command_rx,
                vec![consumer],
                Arc::clone(&capacity_available),
                Arc::clone(&terminal_error),
                service_command_tx.clone(),
            ));
            let _ = endpoints.insert(
                endpoint_key,
                SharedEndpointHandle {
                    endpoint_id,
                    command_tx: endpoint_command_tx,
                    consumer_count: 1,
                    local_addr,
                    capacity_available: Arc::clone(&capacity_available),
                    terminal_error: Arc::clone(&terminal_error),
                },
            );
            Ok(SharedConsumerParts {
                receiver,
                local_addr,
                capacity_available,
                terminal_error,
                endpoint_id,
            })
        });
    if let Err(std_mpsc::SendError(Ok(parts))) = response_tx.send(result)
        && endpoints
            .get(&cleanup_key)
            .map(|endpoint| endpoint.endpoint_id)
            == Some(parts.endpoint_id)
        && let Some(endpoint) = endpoints.remove(&cleanup_key)
    {
        let _ = endpoint.command_tx.send(SharedEndpointCommand::Shutdown);
    }
}

async fn run_shared_endpoint(
    endpoint_key: SharedEndpointKey,
    endpoint_id: u64,
    listener: TcpListener,
    mut command_rx: mpsc::UnboundedReceiver<SharedEndpointCommand>,
    mut consumers: Vec<SharedConsumer>,
    capacity_available: Arc<Notify>,
    terminal_error: Arc<Mutex<Option<SharedAcceptError>>>,
    service_command_tx: mpsc::UnboundedSender<SharedListenerServiceCommand>,
) {
    let mut next_consumer = 0;
    let mut pending_connection = None;
    loop {
        if let Some(connection) = pending_connection.take() {
            match dispatch_shared_connection(&mut consumers, &mut next_consumer, connection) {
                Ok(()) => continue,
                Err(connection) => pending_connection = Some(connection),
            }
        }

        if consumers.is_empty() {
            break;
        }

        if pending_connection.is_some() {
            tokio::select! {
                command = command_rx.recv() => {
                    if !apply_shared_endpoint_command(command, &mut consumers, &mut next_consumer) {
                        break;
                    }
                }
                () = capacity_available.notified() => {}
            }
        } else {
            tokio::select! {
                command = command_rx.recv() => {
                    if !apply_shared_endpoint_command(command, &mut consumers, &mut next_consumer) {
                        break;
                    }
                }
                accepted = listener.accept() => {
                    match accepted {
                        Ok(connection) => pending_connection = Some(connection),
                        Err(error) => {
                            *terminal_error.lock() = Some(SharedAcceptError::from_io(&error));
                            break;
                        }
                    }
                }
            }
        }
    }
    consumers.clear();
    let _ = service_command_tx.send(SharedListenerServiceCommand::EndpointStopped {
        endpoint_key,
        endpoint_id,
    });
}

fn apply_shared_endpoint_command(
    command: Option<SharedEndpointCommand>,
    consumers: &mut Vec<SharedConsumer>,
    next_consumer: &mut usize,
) -> bool {
    match command {
        Some(SharedEndpointCommand::Add {
            registration_id,
            sender,
        }) => consumers.push(SharedConsumer {
            registration_id,
            sender,
        }),
        Some(SharedEndpointCommand::Remove { registration_id }) => {
            consumers.retain(|consumer| consumer.registration_id != registration_id);
            if *next_consumer >= consumers.len() {
                *next_consumer = 0;
            }
        }
        Some(SharedEndpointCommand::Shutdown) | None => return false,
    }
    true
}

fn dispatch_shared_connection(
    consumers: &mut Vec<SharedConsumer>,
    next_consumer: &mut usize,
    mut connection: (TcpStream, SocketAddr),
) -> Result<(), (TcpStream, SocketAddr)> {
    let mut checked = 0;
    while checked < consumers.len() {
        if *next_consumer >= consumers.len() {
            *next_consumer = 0;
        }
        let index = *next_consumer;
        match consumers[index].sender.try_send(connection) {
            Ok(()) => {
                *next_consumer = (index + 1) % consumers.len();
                return Ok(());
            }
            Err(mpsc::error::TrySendError::Full(returned)) => {
                connection = returned;
                *next_consumer = (index + 1) % consumers.len();
                checked += 1;
            }
            Err(mpsc::error::TrySendError::Closed(returned)) => {
                connection = returned;
                let _ = consumers.remove(index);
                if *next_consumer >= consumers.len() {
                    *next_consumer = 0;
                }
            }
        }
    }
    Err(connection)
}

/// Pipeline-local TCP listener assigned to one receiver instance.
pub struct NetworkTcpListener {
    inner: NetworkTcpListenerInner,
    deployment: Arc<NetworkListenerDeploymentState>,
    key: RegistrationKey,
    registration_id: u64,
}

enum NetworkTcpListenerInner {
    SocketPerListener(TcpListener),
    SharedSocket(SharedTcpListener),
}

impl NetworkTcpListener {
    /// Returns the local address of the underlying listener.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        match &self.inner {
            NetworkTcpListenerInner::SocketPerListener(listener) => listener.local_addr(),
            NetworkTcpListenerInner::SharedSocket(listener) => Ok(listener.local_addr),
        }
    }

    /// Waits for the next connection assigned to this pipeline instance.
    pub async fn accept(&mut self) -> io::Result<(TcpStream, SocketAddr)> {
        futures::future::poll_fn(|cx| self.poll_accept(cx)).await
    }

    fn poll_accept(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<(TcpStream, SocketAddr)>> {
        match &mut self.inner {
            NetworkTcpListenerInner::SocketPerListener(listener) => listener.poll_accept(cx),
            NetworkTcpListenerInner::SharedSocket(listener) => listener.poll_accept(cx),
        }
    }
}

impl Drop for NetworkTcpListener {
    fn drop(&mut self) {
        self.deployment
            .release_registration(&self.key, self.registration_id);
    }
}

/// Incoming TCP stream suitable for servers such as Tonic.
pub struct TcpIncoming {
    listener: Option<NetworkTcpListener>,
    shutdown: Pin<Box<dyn Future<Output = ()> + Send>>,
}

impl Stream for TcpIncoming {
    type Item = io::Result<TcpStream>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        if this.listener.is_none() {
            return Poll::Ready(None);
        }
        if this.shutdown.as_mut().poll(cx).is_ready() {
            this.listener = None;
            return Poll::Ready(None);
        }

        let listener = this.listener.as_mut().expect("listener checked above");
        match listener.poll_accept(cx) {
            Poll::Ready(Ok((stream, _))) => Poll::Ready(Some(Ok(stream))),
            Poll::Ready(Err(error)) => {
                this.listener = None;
                Poll::Ready(Some(Err(error)))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Converts a TCP listener into an incoming stream that closes on cancellation.
pub fn tcp_incoming<F>(listener: NetworkTcpListener, shutdown: F) -> TcpIncoming
where
    F: Future<Output = ()> + Send + 'static,
{
    TcpIncoming {
        listener: Some(listener),
        shutdown: Box::pin(shutdown),
    }
}

fn create_listener(addr: SocketAddr) -> io::Result<StdTcpListener> {
    let socket = socket2::Socket::new(
        match addr {
            SocketAddr::V4(_) => socket2::Domain::IPV4,
            SocketAddr::V6(_) => socket2::Domain::IPV6,
        },
        socket2::Type::STREAM,
        None,
    )?;
    socket.set_reuse_address(true)?;
    #[cfg(unix)]
    socket.set_reuse_port(true)?;
    socket.set_nonblocking(true)?;
    socket.bind(&addr.into())?;
    socket.listen(LISTEN_BACKLOG)?;
    Ok(socket.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_config::{PipelineGroupId, PipelineId, PipelineKey};
    use tokio::time::{Duration, timeout};

    fn deployment_key(placement_generation: u64) -> LogicalPipelineDeploymentKey {
        LogicalPipelineDeploymentKey::new(
            PipelineKey::new(PipelineGroupId::from("group"), PipelineId::from("pipeline")),
            placement_generation,
        )
    }

    fn two_member_plan(placement_generation: u64, addr: SocketAddr) -> NetworkListenerPlan {
        NetworkListenerPlan::new(
            deployment_key(placement_generation),
            vec![
                NetworkListenerMember {
                    core_id: 2,
                    numa_node_id: None,
                },
                NetworkListenerMember {
                    core_id: 3,
                    numa_node_id: None,
                },
            ],
            vec![NetworkEndpoint::new(
                "syslog".into(),
                addr,
                NetworkTransport::Tcp,
            )],
        )
        .unwrap()
    }

    async fn assert_backend_accepts(backend: NetworkListenerBackend) {
        let broker = NetworkListenerBroker::with_backend(backend);
        let listener_set = broker
            .listener_set_for(
                Arc::new(two_member_plan(1, "127.0.0.1:0".parse().unwrap())),
                2,
            )
            .unwrap();
        let mut listener = listener_set
            .tcp_listener("syslog".into(), "127.0.0.1:0".parse().unwrap())
            .unwrap();
        let addr = listener.local_addr().unwrap();
        let client = TcpStream::connect(addr);
        let accepted = timeout(Duration::from_secs(2), listener.accept());
        let (client, accepted) = tokio::join!(client, accepted);
        let _ = client.unwrap();
        let (_, peer_addr) = accepted.unwrap().unwrap();
        assert!(peer_addr.ip().is_loopback());
    }

    /// Scenario: the socket-per-listener backend serves a declared endpoint.
    /// Guarantees: backend extraction preserves directly owned listener behavior.
    #[tokio::test(flavor = "current_thread")]
    async fn socket_per_listener_backend_accepts_connections() {
        assert_backend_accepts(NetworkListenerBackend::SocketPerListener).await;
    }

    /// Scenario: the shared-socket backend serves a declared endpoint.
    /// Guarantees: its broker-owned networking thread hands accepted streams to the consumer.
    #[tokio::test(flavor = "current_thread")]
    async fn shared_socket_backend_accepts_connections() {
        assert_backend_accepts(NetworkListenerBackend::shared_socket()).await;
    }

    /// Scenario: a shared backend is configured before any receiver requests a socket.
    /// Guarantees: the shared listener service starts on the first socket request, not broker setup.
    #[test]
    fn shared_listener_service_starts_lazily() {
        let broker = NetworkListenerBroker::with_backend(NetworkListenerBackend::shared_socket());
        let NetworkListenerBackend::SharedSocket { service } = broker.inner.backend.as_ref() else {
            panic!("broker did not retain the configured shared backend");
        };
        assert!(service.lock().is_none());

        let listener_set = broker
            .listener_set_for(
                Arc::new(two_member_plan(1, "127.0.0.1:0".parse().unwrap())),
                2,
            )
            .unwrap();
        assert!(service.lock().is_none());

        let listener = listener_set
            .tcp_listener("syslog".into(), "127.0.0.1:0".parse().unwrap())
            .unwrap();
        assert!(service.lock().is_some());
        drop(listener);
    }

    /// Scenario: two shared-socket consumers register for one physical endpoint.
    /// Guarantees: accepted streams are distributed across both bounded consumer queues.
    #[tokio::test(flavor = "current_thread")]
    async fn shared_socket_backend_distributes_connections() {
        let probe = StdTcpListener::bind("127.0.0.1:0").unwrap();
        let addr = probe.local_addr().unwrap();
        drop(probe);

        let broker = NetworkListenerBroker::with_backend(NetworkListenerBackend::shared_socket());
        let plan = Arc::new(two_member_plan(1, addr));
        let first_set = broker.listener_set_for(Arc::clone(&plan), 2).unwrap();
        let second_set = broker.listener_set_for(plan, 3).unwrap();
        let mut first = first_set.tcp_listener("syslog".into(), addr).unwrap();
        let mut second = second_set.tcp_listener("syslog".into(), addr).unwrap();
        tokio::time::sleep(Duration::from_millis(10)).await;

        let first_client = TcpStream::connect(addr);
        let second_client = TcpStream::connect(addr);
        let first_accepted = timeout(Duration::from_secs(2), first.accept());
        let second_accepted = timeout(Duration::from_secs(2), second.accept());
        let (first_client, second_client, first_accepted, second_accepted) =
            tokio::join!(first_client, second_client, first_accepted, second_accepted);
        let _ = first_client.unwrap();
        let _ = second_client.unwrap();
        let _ = first_accepted.unwrap().unwrap();
        let _ = second_accepted.unwrap().unwrap();
    }

    /// Scenario: the first shared-socket consumer drops while another remains registered.
    /// Guarantees: the broker-owned accept loop continues independently of that consumer.
    #[tokio::test(flavor = "current_thread")]
    async fn shared_socket_backend_survives_consumer_drop() {
        let probe = StdTcpListener::bind("127.0.0.1:0").unwrap();
        let addr = probe.local_addr().unwrap();
        drop(probe);

        let broker = NetworkListenerBroker::with_backend(NetworkListenerBackend::shared_socket());
        let plan = Arc::new(two_member_plan(1, addr));
        let first_set = broker.listener_set_for(Arc::clone(&plan), 2).unwrap();
        let second_set = broker.listener_set_for(plan, 3).unwrap();
        let first = first_set.tcp_listener("syslog".into(), addr).unwrap();
        let mut second = second_set.tcp_listener("syslog".into(), addr).unwrap();
        drop(first);

        let client = TcpStream::connect(addr);
        let accepted = timeout(Duration::from_secs(2), second.accept());
        let (client, accepted) = tokio::join!(client, accepted);
        let _ = client.unwrap();
        let _ = accepted.unwrap().unwrap();
    }

    /// Scenario: the final shared-socket consumer drops and later registers again.
    /// Guarantees: the physical endpoint can shut down and be recreated for the same deployment.
    #[tokio::test(flavor = "current_thread")]
    async fn shared_socket_backend_recreates_endpoint_after_last_consumer() {
        let probe = StdTcpListener::bind("127.0.0.1:0").unwrap();
        let addr = probe.local_addr().unwrap();
        drop(probe);

        let broker = NetworkListenerBroker::with_backend(NetworkListenerBackend::shared_socket());
        let listener_set = broker
            .listener_set_for(Arc::new(two_member_plan(1, addr)), 2)
            .unwrap();
        let first = listener_set.tcp_listener("syslog".into(), addr).unwrap();
        drop(first);

        let mut replacement = listener_set.tcp_listener("syslog".into(), addr).unwrap();
        let client = TcpStream::connect(addr);
        let accepted = timeout(Duration::from_secs(2), replacement.accept());
        let (client, accepted) = tokio::join!(client, accepted);
        let _ = client.unwrap();
        let _ = accepted.unwrap().unwrap();
    }

    /// Scenario: shared endpoint capacity is divided across varying consumer counts.
    /// Guarantees: per-consumer queues remain within the agreed bounds and aggregate budget.
    #[test]
    fn shared_consumer_capacity_is_bounded() {
        assert_eq!(shared_consumer_capacity(1), 1024);
        assert_eq!(shared_consumer_capacity(4), 1024);
        assert_eq!(shared_consumer_capacity(16), 256);
        assert_eq!(shared_consumer_capacity(64), 64);
        assert_eq!(shared_consumer_capacity(256), 64);
    }

    /// Scenario: a standalone listener accepts a loopback connection.
    /// Guarantees: the receiver-facing listener preserves Tokio TCP accept behavior.
    #[tokio::test(flavor = "current_thread")]
    async fn listener_accept_matches_tokio_listener() {
        let listener_set = NetworkListenerSet::standalone();
        let mut listener = listener_set
            .tcp_listener("receiver".into(), "127.0.0.1:0".parse().unwrap())
            .unwrap();
        let addr = listener.local_addr().unwrap();
        let client = TcpStream::connect(addr);
        let accepted = timeout(Duration::from_secs(2), listener.accept());
        let (client, accepted) = tokio::join!(client, accepted);
        let _ = client.unwrap();
        let (_, peer_addr) = accepted.unwrap().unwrap();
        assert!(peer_addr.ip().is_loopback());
    }

    /// Scenario: two plan members request the same declared endpoint.
    /// Guarantees: each runtime receives a listener handle regardless of the selected backend.
    #[tokio::test(flavor = "current_thread")]
    async fn deployment_members_can_share_an_endpoint() {
        let probe = StdTcpListener::bind("127.0.0.1:0").unwrap();
        let addr = probe.local_addr().unwrap();
        drop(probe);

        let broker = NetworkListenerBroker::with_backend(NetworkListenerBackend::shared_socket());
        let plan = Arc::new(two_member_plan(1, addr));
        let first_set = broker.listener_set_for(Arc::clone(&plan), 2).unwrap();
        let second_set = broker.listener_set_for(plan, 3).unwrap();
        let first = first_set.tcp_listener("syslog".into(), addr).unwrap();
        let second = second_set.tcp_listener("syslog".into(), addr).unwrap();
        assert_eq!(first.local_addr().unwrap(), addr);
        assert_eq!(second.local_addr().unwrap(), addr);
    }

    /// Scenario: two placement generations declare the same physical endpoint.
    /// Guarantees: distinct deployment keys maintain independent listener namespaces.
    #[tokio::test(flavor = "current_thread")]
    async fn separate_deployments_can_overlap() {
        let probe = StdTcpListener::bind("127.0.0.1:0").unwrap();
        let addr = probe.local_addr().unwrap();
        drop(probe);

        let broker = NetworkListenerBroker::with_backend(NetworkListenerBackend::shared_socket());
        let old_set = broker
            .listener_set_for(Arc::new(two_member_plan(1, addr)), 2)
            .unwrap();
        let new_set = broker
            .listener_set_for(Arc::new(two_member_plan(2, addr)), 2)
            .unwrap();
        let old_listener = old_set.tcp_listener("syslog".into(), addr).unwrap();
        let new_listener = new_set.tcp_listener("syslog".into(), addr).unwrap();
        assert_eq!(old_listener.local_addr().unwrap(), addr);
        assert_eq!(new_listener.local_addr().unwrap(), addr);
    }

    /// Scenario: a deployment key is reused with a different immutable plan.
    /// Guarantees: the broker rejects conflicting definitions for stable deployment identity.
    #[test]
    fn broker_rejects_mismatched_plan_for_existing_key() {
        let broker = NetworkListenerBroker::new();
        let first_addr = "127.0.0.1:1514".parse().unwrap();
        let second_addr = "127.0.0.1:1515".parse().unwrap();
        let first_set = broker
            .listener_set_for(Arc::new(two_member_plan(1, first_addr)), 2)
            .unwrap();

        let result = broker.listener_set_for(Arc::new(two_member_plan(1, second_addr)), 2);
        let Err(error) = result else {
            panic!("mismatched plan unexpectedly reused an existing deployment");
        };
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        drop(first_set);
    }

    /// Scenario: an incoming stream observes cancellation while it owns a listener.
    /// Guarantees: polling the cancelled stream closes the socket and releases its registration.
    #[tokio::test(flavor = "current_thread")]
    async fn cancelling_incoming_drops_listener_registration() {
        use futures::StreamExt;
        use tokio_util::sync::CancellationToken;

        let probe = StdTcpListener::bind("127.0.0.1:0").unwrap();
        let addr = probe.local_addr().unwrap();
        drop(probe);

        let broker = NetworkListenerBroker::new();
        let listener_set = broker
            .listener_set_for(Arc::new(two_member_plan(1, addr)), 2)
            .unwrap();
        let listener = listener_set.tcp_listener("syslog".into(), addr).unwrap();
        let shutdown = CancellationToken::new();
        let mut incoming = tcp_incoming(listener, shutdown.clone().cancelled_owned());
        shutdown.cancel();
        assert!(incoming.next().await.is_none());

        let replacement = listener_set.tcp_listener("syslog".into(), addr).unwrap();
        drop(replacement);
    }

    /// Scenario: a listener is dropped and a replacement claims the same endpoint.
    /// Guarantees: stale registration releases cannot remove the replacement's claim.
    #[tokio::test(flavor = "current_thread")]
    async fn registration_claim_is_released_by_listener_drop() {
        let probe = StdTcpListener::bind("127.0.0.1:0").unwrap();
        let addr = probe.local_addr().unwrap();
        drop(probe);

        let broker = NetworkListenerBroker::new();
        let listener_set = broker
            .listener_set_for(Arc::new(two_member_plan(1, addr)), 2)
            .unwrap();
        let first = listener_set.tcp_listener("syslog".into(), addr).unwrap();
        let stale_key = first.key.clone();
        let stale_registration_id = first.registration_id;

        let duplicate = listener_set.tcp_listener("syslog".into(), addr);
        let Err(duplicate) = duplicate else {
            panic!("duplicate registration unexpectedly succeeded");
        };
        assert_eq!(duplicate.kind(), io::ErrorKind::AlreadyExists);

        drop(first);
        let replacement = listener_set.tcp_listener("syslog".into(), addr).unwrap();
        listener_set
            .deployment
            .release_registration(&stale_key, stale_registration_id);

        let duplicate = listener_set.tcp_listener("syslog".into(), addr);
        let Err(duplicate) = duplicate else {
            panic!("stale registration released the replacement claim");
        };
        assert_eq!(duplicate.kind(), io::ErrorKind::AlreadyExists);
        drop(replacement);
    }

    /// Scenario: callers request an unplanned member or receiver endpoint.
    /// Guarantees: listener sets enforce both plan membership and endpoint declarations.
    #[test]
    fn registration_requires_planned_core_and_endpoint() {
        let addr = "127.0.0.1:1514".parse().unwrap();
        let broker = NetworkListenerBroker::new();
        let plan = Arc::new(two_member_plan(1, addr));

        let wrong_core = broker.listener_set_for(Arc::clone(&plan), 4);
        let Err(wrong_core) = wrong_core else {
            panic!("unplanned core listener set unexpectedly succeeded");
        };
        assert_eq!(wrong_core.kind(), io::ErrorKind::InvalidInput);

        let listener_set = broker.listener_set_for(plan, 2).unwrap();
        let wrong_node = listener_set.tcp_listener("otlp".into(), addr);
        let Err(wrong_node) = wrong_node else {
            panic!("unplanned endpoint registration unexpectedly succeeded");
        };
        assert_eq!(wrong_node.kind(), io::ErrorKind::NotFound);
    }

    /// Scenario: two receiver nodes declare one physical address and transport.
    /// Guarantees: plan validation rejects ambiguous ownership before listener creation.
    #[test]
    fn plan_rejects_duplicate_physical_endpoints() {
        let addr = "127.0.0.1:1514".parse().unwrap();
        let error = NetworkListenerPlan::new(
            deployment_key(1),
            vec![NetworkListenerMember {
                core_id: 0,
                numa_node_id: None,
            }],
            vec![
                NetworkEndpoint::new("first".into(), addr, NetworkTransport::Tcp),
                NetworkEndpoint::new("second".into(), addr, NetworkTransport::Tcp),
            ],
        )
        .unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::AddrInUse);
    }
}
