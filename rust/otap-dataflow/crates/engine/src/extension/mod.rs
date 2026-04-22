// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Extension wrapper and typestate builder.
//!
//! Extensions are PData-free — they never process pipeline data, only control
//! messages. This module is split into:
//!
//! | Submodule   | Contents                                                                 |
//! |-------------|--------------------------------------------------------------------------|
//! | [`wrapper`] | [`ControlChannel`], [`EffectHandler`], [`ExtensionLifecycle`], [`ExtensionWrapper`], [`ExtensionBundle`] |
//! | [`builder`] | Typestate chain: [`ExtensionBundleBuilder`], [`ActiveStage`], [`PassiveStage`], [`PassiveClonedStage`], [`PassiveFreshStage`], plus [`SharedDecomposed`] / [`LocalDecomposed`] |
//!
//! For the local (!Send) and shared (Send) Extension traits, see
//! [`local::extension`](crate::local::extension) and
//! [`shared::extension`](crate::shared::extension).

pub mod builder;
pub mod wrapper;

#[cfg(test)]
mod tests;

pub use builder::{
    ActiveStage, ExtensionBundleBuilder, LocalDecomposed, PassiveClonedStage, PassiveFreshStage,
    PassiveStage, SharedDecomposed,
};
pub use wrapper::{
    ControlChannel, ControlReceiver, EffectHandler, ExtensionBundle, ExtensionLifecycle,
    ExtensionWrapper,
};
