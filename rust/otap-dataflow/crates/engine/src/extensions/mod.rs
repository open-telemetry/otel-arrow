// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Extension registry and built-in handle types.
//!
//! Extensions are non-pipeline components that provide cross-cutting capabilities
//! (e.g., authentication, health checks) to pipeline nodes. An extension produces
//! one or more **service handles** — lightweight, cloneable values that pipeline
//! components use to interact with the extension at runtime.
//!
//! # Design Principles
//!
//! * **No `Sync` bounds** — handles are `Clone + Send` so each component owns its
//!   own copy. There is no shared mutable state between threads.
//! * **No `unsafe` code** — the registry stores handles as `Box<dyn Any + Send>`
//!   and retrieves them via standard `Any::downcast_ref` + `Clone`.
//! * **Channel-based communication** — handles typically wrap a `tokio::sync::watch`
//!   receiver or similar primitive. The extension task owns the sender end.
//! * **Lifecycle ordering** — extension tasks start before pipeline components and
//!   shut down after them, guaranteeing that handles remain valid for the duration
//!   of the pipeline.
//!
//! # Adding a new handle type
//!
//! 1. Define a concrete struct implementing `Clone + Send + 'static`.
//! 2. In the extension factory, create both the extension task and the handle,
//!    then register the handle via [`ExtensionHandles::register`].
//! 3. Pipeline components retrieve the handle at start-up via
//!    `effect_handler.get_extension_handle::<MyHandle>("extension_name")`.

pub mod auth;
pub mod registry;

pub use auth::{
    AuthError, ClientAuthenticator, ClientAuthenticatorHandle, ServerAuthenticator,
    ServerAuthenticatorHandle,
};
pub use registry::{ExtensionHandles, ExtensionRegistry, ExtensionRegistryBuilder};
