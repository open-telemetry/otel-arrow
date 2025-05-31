// SPDX-License-Identifier: Apache-2.0

//! OTAP data plane configuration.
//!
//! Data Model:
//! - data plane
//!   - tenants
//!     - pipelines
//!       - nodes
//!
//! A data plane is a collection of tenants, where each tenant can have multiple pipelines.
//! A pipeline is a collection of nodes interconnected in a directed acyclic graph (DAG).

use std::borrow::Cow;
pub mod data_plane;
pub mod error;
mod experimental;
pub mod node;
pub mod pipeline;
pub mod tenant;

/// The id of a tenant.
pub type TenantId = Cow<'static, str>;

/// The id of a pipeline.
pub type PipelineId = Cow<'static, str>;

/// The id of a node in the pipeline.
pub type NodeId = Cow<'static, str>;

/// The name of a node out port in the pipeline.
pub type PortName = Cow<'static, str>;

/// The description of a pipeline or a node.
pub type Description = Cow<'static, str>;
