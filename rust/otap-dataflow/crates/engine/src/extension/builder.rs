// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Typestate builder for [`ExtensionBundle`].
//!
//! The extension-registration API uses a typestate chain on
//! [`ExtensionBundleBuilder`] that **seals lifecycle and instance
//! policy per bundle**:
//!
//! ```ignore
//! // Active extension (implicitly instance-based; factory is Passive-only).
//! builder
//!     .active()
//!     .shared(MyShared::new(cfg))
//!     .local(Rc::new(MyLocal::new(cfg)))
//!     .build()?;
//!
//! // Passive extension, clone-per-consumer policy.
//! builder
//!     .passive()
//!     .cloned()
//!     .shared(MyShared::new(cfg))
//!     .build()?;
//!
//! // Passive extension, fresh-per-consumer policy.
//! builder
//!     .passive()
//!     .fresh()
//!     .shared(|| MyShared::new(cfg.clone()))
//!     .local(|| Rc::new(MyLocal::new(cfg.clone())))
//!     .build()?;
//! ```
//!
//! **Axes** (chosen once per bundle, in order):
//!
//! | Axis            | Methods                                                                    |
//! |-----------------|----------------------------------------------------------------------------|
//! | Lifecycle       | `.active()` / `.passive()`                                                 |
//! | Instance policy | *(implicit Cloned for Active)* `.cloned()` / `.fresh()` *(Passive only)* |
//! | Execution model | `.shared(...)` / `.local(...)` *(repeatable, register once each)*          |
//!
//! **Why the lifecycle and policy are sealed per bundle.** "This
//! extension has an event loop" or "this capability hands out fresh
//! instances" is a property of the extension, not of which trait-object
//! shape (shared vs local) the consumer happens to request. Letting the
//! two execution models diverge would mean consumers observe different
//! instance-sharing semantics depending on whether they call
//! `require_local` vs `require_shared` on the same extension — a
//! debugging hazard with no known legitimate use case. Forcing a single
//! strategy across both execution models makes the extension's behavior
//! predictable and eliminates a combinatorial footgun.
//!
//! **Active + Fresh is unrepresentable.** Active extensions have a
//! single engine-driven event loop; minting fresh instances per
//! consumer doesn't compose with that. The `.active()` stage provides
//! no `.fresh()` method — the invalid combination is a compile-time
//! error.

use super::{ExtensionBundle, ExtensionLifecycle, ExtensionWrapper};
use crate::capability::factory::{LocalInstanceFactory, SharedInstanceFactory};
use crate::config::ExtensionConfig;
use crate::error::Error;
use crate::local::extension as local_ext;
use crate::local::message::{LocalReceiver, LocalSender};
use crate::message::Sender;
use crate::shared::extension as shared_ext;
use crate::shared::message::{SharedReceiver, SharedSender};
use otap_df_channel::mpsc;
use otap_df_config::ExtensionId;
use otap_df_config::extension::ExtensionUserConfig;
use otap_df_telemetry::otel_debug;
use std::any::{Any, TypeId};
use std::sync::Arc;

// ── Decomposed (type-erased) provider outputs ────────────────────────────────

/// Decomposed result of a shared extension provider.
#[doc(hidden)]
pub struct SharedDecomposed {
    pub(crate) extension: Option<Box<dyn shared_ext::Extension>>,
    /// Factory that mints instances of the extension's concrete type
    /// for capability consumers. The engine downcasts back to `E` via
    /// generated registration glue.
    pub(crate) instance_factory: SharedInstanceFactory,
    /// Used by the same-type guard and the capability system.
    pub(crate) type_id: TypeId,
}

/// Decomposed result of a local extension provider.
#[doc(hidden)]
pub struct LocalDecomposed {
    pub(crate) extension: Option<std::rc::Rc<dyn local_ext::Extension>>,
    /// Factory that mints instances of the extension's concrete type
    /// for capability consumers.
    pub(crate) instance_factory: LocalInstanceFactory,
    /// Used by the same-type guard and the capability system.
    pub(crate) type_id: TypeId,
}

// ── Typestate builder stages ─────────────────────────────────────────────────

/// Lifecycle-selected: Active (engine drives an event loop).
///
/// Instance policy is implicitly Cloned — factory is Passive-only.
/// Call [`shared()`](Self::shared) and/or [`local()`](Self::local)
/// (each at most once), then [`build()`](Self::build).
#[doc(hidden)]
pub struct ActiveStage {
    parent: ExtensionBundleBuilder,
}

impl ActiveStage {
    /// Register the shared (Send) variant.
    #[must_use]
    pub fn shared<E>(mut self, extension: E) -> Self
    where
        E: shared_ext::Extension + Clone + Send + 'static,
    {
        debug_assert!(
            self.parent.shared.is_none(),
            "ExtensionBundleBuilder: .shared(...) called more than once",
        );
        let for_factory = extension.clone();
        self.parent.shared = Some(SharedDecomposed {
            extension: Some(Box::new(extension)),
            instance_factory: SharedInstanceFactory::new(move || {
                Box::new(for_factory.clone()) as Box<dyn Any + Send>
            }),
            type_id: TypeId::of::<E>(),
        });
        self
    }

    /// Register the local (!Send) variant.
    #[must_use]
    pub fn local<E>(mut self, extension: std::rc::Rc<E>) -> Self
    where
        E: local_ext::Extension + 'static,
    {
        debug_assert!(
            self.parent.local.is_none(),
            "ExtensionBundleBuilder: .local(...) called more than once",
        );
        let for_factory = std::rc::Rc::clone(&extension);
        self.parent.local = Some(LocalDecomposed {
            extension: Some(extension),
            instance_factory: LocalInstanceFactory::new(move || {
                std::rc::Rc::clone(&for_factory) as std::rc::Rc<dyn Any>
            }),
            type_id: TypeId::of::<E>(),
        });
        self
    }

    /// Finalize the bundle.
    ///
    /// # Errors
    ///
    /// See [`ExtensionBundleBuilder::build`].
    pub fn build(self) -> Result<ExtensionBundle, Error> {
        self.parent.build()
    }
}

/// Lifecycle-selected: Passive (no event loop, capabilities only).
///
/// Select the instance policy with [`cloned()`](Self::cloned) or
/// [`factory()`](Self::factory) before registering sides.
#[doc(hidden)]
pub struct PassiveStage {
    parent: ExtensionBundleBuilder,
}

impl PassiveStage {
    /// Select the **clone-per-consumer** instance policy: each consumer
    /// receives an independent clone of the stored prototype.
    #[must_use]
    pub fn cloned(self) -> PassiveClonedStage {
        PassiveClonedStage {
            parent: self.parent,
        }
    }

    /// Select the **fresh-per-consumer** instance policy: each consumer
    /// receives a freshly-constructed instance from the stored closure.
    #[must_use]
    pub fn fresh(self) -> PassiveFreshStage {
        PassiveFreshStage {
            parent: self.parent,
        }
    }
}

/// Passive + Cloned (clone-per-consumer) stage.
#[doc(hidden)]
pub struct PassiveClonedStage {
    parent: ExtensionBundleBuilder,
}

impl PassiveClonedStage {
    /// Register the shared (Send) variant. The extension will be
    /// cloned per consumer via the clone-per-consumer factory.
    #[must_use]
    pub fn shared<E>(mut self, extension: E) -> Self
    where
        E: Clone + Send + 'static,
    {
        debug_assert!(
            self.parent.shared.is_none(),
            "ExtensionBundleBuilder: .shared(...) called more than once",
        );
        self.parent.shared = Some(SharedDecomposed {
            extension: None,
            instance_factory: SharedInstanceFactory::new(move || {
                Box::new(extension.clone()) as Box<dyn Any + Send>
            }),
            type_id: TypeId::of::<E>(),
        });
        self
    }

    /// Register the local (!Send) variant. Consumers receive
    /// `Rc::clone`s of the stored instance.
    #[must_use]
    pub fn local<E>(mut self, extension: std::rc::Rc<E>) -> Self
    where
        E: 'static,
    {
        debug_assert!(
            self.parent.local.is_none(),
            "ExtensionBundleBuilder: .local(...) called more than once",
        );
        self.parent.local = Some(LocalDecomposed {
            extension: None,
            instance_factory: LocalInstanceFactory::new(move || {
                std::rc::Rc::clone(&extension) as std::rc::Rc<dyn Any>
            }),
            type_id: TypeId::of::<E>(),
        });
        self
    }

    /// Finalize the bundle.
    ///
    /// # Errors
    ///
    /// See [`ExtensionBundleBuilder::build`].
    pub fn build(self) -> Result<ExtensionBundle, Error> {
        self.parent.build()
    }
}

/// Passive + Fresh (fresh-per-consumer) stage.
#[doc(hidden)]
pub struct PassiveFreshStage {
    parent: ExtensionBundleBuilder,
}

impl PassiveFreshStage {
    /// Register the shared (Send) variant via a factory closure.
    /// Each consumer receives a freshly-constructed instance.
    ///
    /// `F: Clone` is required so per-node factories can duplicate the
    /// closure; closures capturing `Clone` configuration (e.g.
    /// `Arc<Config>`, `String`) satisfy this automatically.
    #[must_use]
    pub fn shared<E, F>(mut self, produce: F) -> Self
    where
        E: Send + 'static,
        F: Fn() -> E + Clone + Send + 'static,
    {
        debug_assert!(
            self.parent.shared.is_none(),
            "ExtensionBundleBuilder: .shared(...) called more than once",
        );
        self.parent.shared = Some(SharedDecomposed {
            extension: None,
            instance_factory: SharedInstanceFactory::new(move || {
                Box::new(produce()) as Box<dyn Any + Send>
            }),
            type_id: TypeId::of::<E>(),
        });
        self
    }

    /// Register the local (!Send) variant via a factory closure.
    /// Each consumer receives a freshly-constructed instance.
    #[must_use]
    pub fn local<E, F>(mut self, produce: F) -> Self
    where
        E: 'static,
        F: Fn() -> std::rc::Rc<E> + Clone + 'static,
    {
        debug_assert!(
            self.parent.local.is_none(),
            "ExtensionBundleBuilder: .local(...) called more than once",
        );
        self.parent.local = Some(LocalDecomposed {
            extension: None,
            instance_factory: LocalInstanceFactory::new(move || produce() as std::rc::Rc<dyn Any>),
            type_id: TypeId::of::<E>(),
        });
        self
    }

    /// Finalize the bundle.
    ///
    /// # Errors
    ///
    /// See [`ExtensionBundleBuilder::build`].
    pub fn build(self) -> Result<ExtensionBundle, Error> {
        self.parent.build()
    }
}

// ── Builder ──────────────────────────────────────────────────────────────────

/// Builder for [`ExtensionBundle`].
///
/// At least one variant (local or shared) must be added before calling `build()`.
/// Both variants can be provided for dual-mode extensions.
pub struct ExtensionBundleBuilder {
    pub(super) name: ExtensionId,
    pub(super) user_config: Arc<ExtensionUserConfig>,
    pub(super) runtime_config: ExtensionConfig,
    shared: Option<SharedDecomposed>,
    local: Option<LocalDecomposed>,
}

impl ExtensionBundleBuilder {
    /// Construct a new [`ExtensionBundleBuilder`]. Called by
    /// [`ExtensionWrapper::builder`].
    pub(super) fn new(
        name: ExtensionId,
        user_config: Arc<ExtensionUserConfig>,
        runtime_config: ExtensionConfig,
    ) -> Self {
        Self {
            name,
            user_config,
            runtime_config,
            shared: None,
            local: None,
        }
    }

    /// Select the **Active** lifecycle for this extension bundle.
    /// The engine will drive an event loop for whichever sides are
    /// registered.
    ///
    /// Instance policy is implicitly clone-per-consumer — fresh-per-
    /// consumer (factory) is Passive-only.
    #[must_use]
    pub fn active(self) -> ActiveStage {
        ActiveStage { parent: self }
    }

    /// Select the **Passive** lifecycle for this extension bundle. No
    /// event loop is spawned; the extension exposes capabilities only.
    ///
    /// Continue with [`PassiveStage::cloned`] (clone-per-consumer)
    /// or [`PassiveStage::fresh`] (fresh-per-consumer).
    #[must_use]
    pub fn passive(self) -> PassiveStage {
        PassiveStage { parent: self }
    }

    /// Build the [`ExtensionBundle`].
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Neither `.shared(...)` nor `.local(...)` was registered via
    ///   the lifecycle stages.
    /// - Both variants use the same concrete type (dual registration
    ///   requires distinct local and shared implementations).
    pub fn build(self) -> Result<ExtensionBundle, Error> {
        if let (Some(local), Some(shared)) = (&self.local, &self.shared) {
            if local.type_id == shared.type_id {
                return Err(Error::InternalError {
                    message: "local and shared variants must use different concrete types; \
                              register only one side for single-variant extensions"
                        .into(),
                });
            }
        }

        let cap = self.runtime_config.control_channel.capacity;

        let local = self.local.map(|l| {
            let lifecycle = match l.extension {
                Some(ext) => {
                    let (tx, rx) = mpsc::Channel::new(cap);
                    ExtensionLifecycle::Active {
                        extension: ext,
                        control_sender: Sender::Local(LocalSender::mpsc(tx)),
                        control_receiver: LocalReceiver::mpsc(rx),
                    }
                }
                None => ExtensionLifecycle::Passive,
            };
            ExtensionWrapper::Local {
                name: self.name.clone(),
                user_config: self.user_config.clone(),
                runtime_config: self.runtime_config.clone(),
                telemetry: None,
                lifecycle,
                instance_factory: l.instance_factory,
            }
        });

        let shared = self.shared.map(|s| {
            let lifecycle = match s.extension {
                Some(ext) => {
                    let (tx, rx) = tokio::sync::mpsc::channel(cap);
                    ExtensionLifecycle::Active {
                        extension: ext,
                        control_sender: Sender::Shared(SharedSender::mpsc(tx)),
                        control_receiver: SharedReceiver::mpsc(rx),
                    }
                }
                None => ExtensionLifecycle::Passive,
            };
            ExtensionWrapper::Shared {
                name: self.name.clone(),
                user_config: self.user_config.clone(),
                runtime_config: self.runtime_config.clone(),
                telemetry: None,
                lifecycle,
                instance_factory: s.instance_factory,
            }
        });

        if local.is_none() && shared.is_none() {
            return Err(Error::InternalError {
                message: "ExtensionBundle must have at least one variant (local or shared)".into(),
            });
        }

        for w in local.iter().chain(shared.iter()) {
            let name = w.name();
            otel_debug!(
                "extension.builder.build",
                name = name.as_ref(),
                variant = match w {
                    ExtensionWrapper::Local { .. } => "local",
                    ExtensionWrapper::Shared { .. } => "shared",
                },
                lifecycle = if w.is_passive() { "passive" } else { "active" },
            );
        }

        Ok(ExtensionBundle::from_parts(local, shared))
    }
}

impl ExtensionWrapper {
    /// Start building an [`ExtensionBundle`].
    #[must_use]
    pub fn builder(
        name: ExtensionId,
        user_config: Arc<ExtensionUserConfig>,
        config: &ExtensionConfig,
    ) -> ExtensionBundleBuilder {
        ExtensionBundleBuilder::new(name, user_config, config.clone())
    }
}
