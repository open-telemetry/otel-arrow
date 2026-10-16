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
use std::sync::{Arc, Weak};
use std::task::{Context, Poll};
use tokio::net::{TcpListener, TcpStream};

const LISTEN_BACKLOG: i32 = 8192;

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
            .finish()
    }
}

/// Controller-scoped registry for logical network listener deployments.
#[derive(Clone, Default)]
pub struct NetworkListenerBroker {
    inner: Arc<NetworkListenerBrokerInner>,
}

#[derive(Default)]
struct NetworkListenerBrokerInner {
    deployments: Mutex<HashMap<LogicalPipelineDeploymentKey, Weak<NetworkListenerDeploymentState>>>,
}

impl NetworkListenerBroker {
    /// Creates an empty listener broker.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
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

        let result = create_listener(bind_address).and_then(TcpListener::from_std);
        match result {
            Ok(listener) => Ok(NetworkTcpListener {
                listener,
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

/// Pipeline-local TCP listener assigned to one receiver instance.
pub struct NetworkTcpListener {
    listener: TcpListener,
    deployment: Arc<NetworkListenerDeploymentState>,
    key: RegistrationKey,
    registration_id: u64,
}

impl NetworkTcpListener {
    /// Returns the local address of the underlying listener.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.listener.local_addr()
    }

    /// Waits for the next connection assigned to this pipeline instance.
    pub async fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
        self.listener.accept().await
    }

    fn poll_accept(&self, cx: &mut Context<'_>) -> Poll<io::Result<(TcpStream, SocketAddr)>> {
        self.listener.poll_accept(cx)
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

        let listener = this.listener.as_ref().expect("listener checked above");
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

    /// Scenario: a standalone listener accepts a loopback connection.
    /// Guarantees: the receiver-facing listener preserves Tokio TCP accept behavior.
    #[tokio::test(flavor = "current_thread")]
    async fn listener_accept_matches_tokio_listener() {
        let listener_set = NetworkListenerSet::standalone();
        let listener = listener_set
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
    /// Guarantees: each runtime member can own its current per-core physical listener.
    #[tokio::test(flavor = "current_thread")]
    async fn deployment_members_can_share_an_endpoint() {
        let probe = StdTcpListener::bind("127.0.0.1:0").unwrap();
        let addr = probe.local_addr().unwrap();
        drop(probe);

        let broker = NetworkListenerBroker::new();
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

        let broker = NetworkListenerBroker::new();
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
