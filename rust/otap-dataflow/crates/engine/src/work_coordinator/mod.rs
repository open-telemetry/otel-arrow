// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Node-declared work coordination for a fixed fleet of pipeline instances.
//!
//! A controller hosts discovery once, outside the data-plane runtimes. Nodes
//! bind a typed participant during construction, report readiness without
//! waiting for assignments, and execute only activated grants. Basic consumers
//! must stop and restart the logical pipeline to change its configuration or
//! fleet. See `docs/work-coordinator.md` for the receiver integration contract.

mod authority;
mod runtime;

pub use authority::{
    AdmissionTicket, AssignmentBackend, AssignmentGrant, BackendOutcome, GrantIdentity,
    InMemoryBackend, OperationId,
};
pub use runtime::{
    AssignmentPhase, AssignmentSnapshot, CoordinatorBinding, CoordinatorBindings,
    CoordinatorRuntime, CoordinatorTask, ParticipantEvent, WorkAssignment, WorkParticipant,
};

use async_trait::async_trait;
use otel_arrow_dfe_config::{NodeId, PipelineGroupId, PipelineId, node::NodeUserConfig};
use std::{fmt::Debug, sync::Arc};

/// A protocol's metadata and custom message types. They never contain PData.
pub trait WorkProtocol: Send + Sync + 'static {
    /// Immutable execution metadata for a work ID. Change the ID when this changes.
    type Work: RetainedBytes + Clone + PartialEq + Debug + Send + Sync + 'static;
    /// Participant-to-coordinator protocol messages.
    type ParticipantRequest: RetainedBytes + Send + 'static;
    /// Coordinator-to-participant protocol messages.
    type CoordinatorNotice: RetainedBytes + Send + 'static;
}

/// Conservative retained size, including owned allocations and inline storage.
///
/// Implementations must include allocation capacities, not just populated lengths.
/// This is a protocol contract: the framework cannot inspect opaque payloads.
pub trait RetainedBytes {
    /// Upper bound on retained bytes for this value.
    fn retained_bytes(&self) -> usize;
}

impl RetainedBytes for String {
    fn retained_bytes(&self) -> usize {
        size_of::<Self>().saturating_add(self.capacity())
    }
}

impl RetainedBytes for () {
    fn retained_bytes(&self) -> usize {
        0
    }
}

/// Stable identity independent of placement or deployment generation.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CoordinatorKey {
    /// Configuration group.
    pub group_id: PipelineGroupId,
    /// Logical pipeline configuration.
    pub pipeline_id: PipelineId,
    /// Stable configured node name.
    pub node_id: NodeId,
    /// Node-local coordinator role.
    pub role: String,
}

/// Exact registration identity; a core ID alone is never an acknowledgement key.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ParticipantSession {
    /// Physical core hosting this instance.
    pub core_id: usize,
    /// Deployment generation.
    pub generation: u64,
    /// Fresh nonce within this coordinator incarnation.
    pub nonce: u64,
}

/// Participant lifecycle observed by the coordinator, including idle instances.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParticipantState {
    /// Bound, but the node has not reported readiness.
    Initializing,
    /// Ready to receive offers; this does not imply ownership.
    ReadyForAssignments,
}

/// Bounded, complete view of the Basic fleet. No administrative polling is used.
#[derive(Clone, Debug)]
pub struct ParticipantRegistry {
    /// Sessions and their readiness state, ordered by core.
    pub participants: Vec<(ParticipantSession, ParticipantState)>,
}

/// Bounded diagnostics without source IDs, paths or URLs.
#[derive(Clone, Copy, Debug, Default)]
pub struct CoordinatorStatus {
    /// Initial fleet size.
    pub participants: usize,
    /// Ready participants, including idle instances.
    pub ready_participants: usize,
    /// Retained work entries, including entries waiting for release.
    pub catalog_items: usize,
    /// Entries with backend-confirmed active authority.
    pub active_assignments: usize,
    /// Entries waiting for worker and completion cleanup.
    pub revoking_assignments: usize,
    /// Current custom-message queue byte charge.
    pub queued_message_bytes: usize,
    /// Final shutdown or fatal authority loss has occurred.
    pub stopped: bool,
}

/// Stable work identity. Terms prevent removed/re-added IDs reviving old grants.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WorkId(pub String);

/// One discovered responsibility, such as a scrape target.
#[derive(Debug)]
pub struct WorkSpec<P: WorkProtocol> {
    /// Stable ID, unique within the coordinator.
    pub id: WorkId,
    /// Immutable metadata interpreted by the node implementation.
    pub work: P::Work,
}

/// Capabilities requested by a node. Unsupported modes fail before host startup.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CapabilityProfile {
    /// Fixed fleet, in-memory authority and cooperative catalog removal.
    #[default]
    Basic,
    /// Live resize and replacement; reserved for the next implementation layer.
    Cooperative,
    /// Durable, conditional progress; reserved for qualified backends/adapters.
    FencedProgress,
}

/// Progress guarantees required by the consumer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ProgressRequirement {
    /// No source checkpoint store, as for ordinary Prometheus scrapes.
    #[default]
    None,
    /// Excluded singleton writer; requires a source-specific adapter.
    ExclusiveWriter,
    /// Authority checks and local progress writes share one serial writer.
    SerializedLocal,
    /// Grants and progress share a crash-consistent transaction boundary.
    DurableFenced,
}

/// Resource limits for a single coordinator. All fields must be nonzero.
#[derive(Clone, Copy, Debug)]
pub struct CoordinatorLimits {
    /// Maximum retained catalog entries, including entries awaiting release.
    pub catalog_items: usize,
    /// Maximum retained catalog bytes, including IDs and execution metadata.
    pub catalog_bytes: usize,
    /// Maximum initial participants.
    pub participants: usize,
    /// Slots in the custom request queue and each custom notice queue.
    pub message_items: usize,
    /// Shared byte budget for all queued custom messages.
    pub message_bytes: usize,
}

impl Default for CoordinatorLimits {
    fn default() -> Self {
        Self {
            catalog_items: 1024,
            catalog_bytes: 8 * 1024 * 1024,
            participants: 64,
            message_items: 32,
            message_bytes: 1024 * 1024,
        }
    }
}

impl CoordinatorLimits {
    /// Validate bounds before allocating queues or starting threads.
    pub fn validate(self) -> Result<(), CoordinatorError> {
        if self.catalog_items == 0
            || self.catalog_bytes == 0
            || self.participants == 0
            || self.message_items == 0
            || self.message_bytes == 0
            || self.catalog_items > 65_536
            || self.participants > 128
            || self.message_items > 1024
            || self.catalog_bytes > 256 * 1024 * 1024
            || self.message_bytes > 64 * 1024 * 1024
        {
            return Err(CoordinatorError::Invalid(
                "invalid coordinator resource limits",
            ));
        }
        Ok(())
    }
}

/// Failure from binding, authority, capacity or a provider.
#[derive(Debug, thiserror::Error)]
pub enum CoordinatorError {
    /// Invalid declaration or protocol operation.
    #[error("{0}")]
    Invalid(&'static str),
    /// A bounded resource is exhausted; the caller can apply backpressure.
    #[error("work coordinator capacity exhausted")]
    Capacity,
    /// Authority has been invalidated or the host has stopped.
    #[error("work coordinator authority is unavailable")]
    Unavailable,
    /// An acknowledgement does not match the current session/term/state.
    #[error("stale or out-of-order work acknowledgement")]
    Stale,
    /// A provider or authority backend failed.
    #[error("work coordinator failed: {0}")]
    Failed(String),
}

/// Node-specific discovery and protocol handling, constructed on the host thread.
///
/// `run` must call `ready` after initialization and continue servicing its runtime.
/// The future and coordinator state may be `!Send`. The host owns its lifetime;
/// any background workers must be accounted for by the provider's shutdown logic.
#[async_trait(?Send)]
pub trait WorkCoordinator<P: WorkProtocol>: 'static {
    /// Discover work and handle custom messages until shutdown is requested.
    async fn run(self: Box<Self>, runtime: CoordinatorRuntime<P>) -> Result<(), CoordinatorError>;
}

/// Side-effect-free, node-supplied declarations; never a pipeline YAML extension.
pub type CoordinatorDeclarationsFn =
    fn(&NodeUserConfig) -> Result<Vec<CoordinatorDeclaration>, otel_arrow_dfe_config::error::Error>;

/// Erased declaration. Type erasure is confined to setup and participant binding.
pub struct CoordinatorDeclaration {
    /// Node-local role, unique for this node.
    pub role: String,
    /// Required lifecycle profile.
    pub profile: CapabilityProfile,
    /// Required progress profile; Basic currently supports `None` only.
    pub progress: ProgressRequirement,
    /// Bounded resources for this host.
    pub limits: CoordinatorLimits,
    factory: runtime::PrepareFn,
}

impl CoordinatorDeclaration {
    /// Capture validated configuration without constructing the provider or doing I/O.
    pub fn new<P, F>(role: impl Into<String>, factory: F) -> Self
    where
        P: WorkProtocol,
        F: FnOnce() -> Box<dyn WorkCoordinator<P>> + Send + 'static,
    {
        Self {
            role: role.into(),
            profile: CapabilityProfile::Basic,
            progress: ProgressRequirement::None,
            limits: CoordinatorLimits::default(),
            factory: Box::new(move |key, sessions, limits, failure| {
                runtime::prepare::<P, F>(key, sessions, limits, factory, failure)
            }),
        }
    }

    /// Check admission requirements without starting a host or opening resources.
    pub fn validate(&self) -> Result<(), CoordinatorError> {
        self.limits.validate()?;
        if self.role.is_empty() || self.role.len() > 128 {
            return Err(CoordinatorError::Invalid("invalid coordinator role"));
        }
        if self.profile != CapabilityProfile::Basic || self.progress != ProgressRequirement::None {
            return Err(CoordinatorError::Invalid(
                "unsupported coordinator capabilities",
            ));
        }
        Ok(())
    }

    /// Prepare endpoints and a host task for the controller to run on its own thread.
    pub fn prepare(
        self,
        key: CoordinatorKey,
        sessions: Vec<ParticipantSession>,
        failure: Arc<dyn Fn(String) + Send + Sync>,
    ) -> Result<(CoordinatorBinding, CoordinatorTask), CoordinatorError> {
        self.validate()?;
        if key.role != self.role {
            return Err(CoordinatorError::Invalid("coordinator role mismatch"));
        }
        (self.factory)(key, sessions, self.limits, failure)
    }
}

#[cfg(test)]
mod tests;
