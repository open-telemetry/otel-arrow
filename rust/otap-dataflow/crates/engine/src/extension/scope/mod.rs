// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Extension declaration scopes, controller-owned hosts, and inherited providers.
//!
//! Extensions declared at engine or pipeline-group scope are hosted once by the
//! controller. Pipelines receive immutable registrations for the shared
//! providers visible from their declaration scope.

mod host;
mod registry;
mod supervisor;

pub use host::ExtensionHostRuntimePolicy;
pub use registry::{ExtensionScopeRegistry, InheritedExtensionRegistrations};
pub use supervisor::{PreparedExtensionScopeSupervisor, RunningExtensionScopeSupervisor};
