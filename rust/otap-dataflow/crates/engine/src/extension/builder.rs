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
//! // Passive extension, constructed-per-consumer policy.
//! builder
//!     .passive()
//!     .constructed()
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
//! | Instance policy | *(implicit Cloned for Active)* `.cloned()` / `.constructed()` *(Passive only)* |
//! | Execution model | `.shared(...)` / `.local(...)` *(each registerable at most once)*           |
//!
//! **Per-axis typestate enforcement.** After selecting lifecycle (and, for
//! Passive, instance policy), the typestate transitions through four
//! per-axis stages: an empty stage that requires `.shared(...)` or
//! `.local(...)`, a `*SharedStage` / `*LocalStage` that exposes only the
//! complementary registration plus `.build()`, and a `*CompleteStage` that
//! exposes only `.build()`. As a result, calling `.shared(...)` or
//! `.local(...)` twice is a compile error, and calling `.build()` without
//! registering at least one variant is a compile error too.
//!
//! **Why the lifecycle and policy are sealed per bundle.** "This
//! extension has an event loop" or "this capability hands out a new
//! instance per consumer" is a property of the extension, not of which
//! trait-object shape (shared vs local) the consumer happens to
//! request. Letting the two execution models diverge would mean
//! consumers observe different instance-sharing semantics depending on
//! whether they call `require_local` vs `require_shared` on the same
//! extension — a debugging hazard with no known legitimate use case.
//! Forcing a single strategy across both execution models makes the
//! extension's behavior predictable and eliminates a combinatorial
//! footgun.
//!
//! **Active + Constructed is unrepresentable.** Active extensions have
//! a single engine-driven event loop; constructing a new instance per
//! consumer doesn't compose with that. The `.active()` stage provides
//! no `.constructed()` method — the invalid combination is a
//! compile-time error.

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
//
// Each lifecycle/instance-policy axis exposes four stages so the typestate
// statically prevents duplicate `.shared(...)` / `.local(...)` registration
// and statically requires at least one variant before `.build()`:
//
//   <Axis>Stage          (empty)         → .shared() / .local()
//   <Axis>SharedStage    (shared set)    → .local() / .build()
//   <Axis>LocalStage     (local set)     → .shared() / .build()
//   <Axis>CompleteStage  (both set)      → .build()
//
// Shared field-mutation logic lives in private helpers on
// `ExtensionBundleBuilder` to keep the per-stage methods thin.

// ── Active lifecycle (shared/local stages) ───────────────────────────────────

/// Lifecycle-selected: Active (engine drives an event loop).
///
/// Instance policy is implicitly Cloned — constructed-per-consumer is
/// Passive-only. Call [`shared()`](Self::shared) or [`local()`](Self::local)
/// to register the first variant; the typestate then forbids registering
/// the same variant twice.
#[doc(hidden)]
pub struct ActiveStage {
    parent: ExtensionBundleBuilder,
}

impl ActiveStage {
    /// Register the shared (Send) variant.
    #[must_use]
    pub fn shared<E>(mut self, extension: E) -> ActiveSharedStage
    where
        E: shared_ext::Extension + Clone + Send + 'static,
    {
        self.parent.set_shared_active(extension);
        ActiveSharedStage {
            parent: self.parent,
        }
    }

    /// Register the local (!Send) variant.
    #[must_use]
    pub fn local<E>(mut self, extension: std::rc::Rc<E>) -> ActiveLocalStage
    where
        E: local_ext::Extension + 'static,
    {
        self.parent.set_local_active(extension);
        ActiveLocalStage {
            parent: self.parent,
        }
    }
}

/// Active + shared registered. Awaiting optional `.local(...)` or `.build()`.
#[doc(hidden)]
pub struct ActiveSharedStage {
    parent: ExtensionBundleBuilder,
}

impl ActiveSharedStage {
    /// Register the local (!Send) variant alongside the previously-registered
    /// shared variant.
    #[must_use]
    pub fn local<E>(mut self, extension: std::rc::Rc<E>) -> ActiveCompleteStage
    where
        E: local_ext::Extension + 'static,
    {
        self.parent.set_local_active(extension);
        ActiveCompleteStage {
            parent: self.parent,
        }
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

/// Active + local registered. Awaiting optional `.shared(...)` or `.build()`.
#[doc(hidden)]
pub struct ActiveLocalStage {
    parent: ExtensionBundleBuilder,
}

impl ActiveLocalStage {
    /// Register the shared (Send) variant alongside the previously-registered
    /// local variant.
    #[must_use]
    pub fn shared<E>(mut self, extension: E) -> ActiveCompleteStage
    where
        E: shared_ext::Extension + Clone + Send + 'static,
    {
        self.parent.set_shared_active(extension);
        ActiveCompleteStage {
            parent: self.parent,
        }
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

/// Active + both shared and local registered. Only `.build()` remains.
#[doc(hidden)]
pub struct ActiveCompleteStage {
    parent: ExtensionBundleBuilder,
}

impl ActiveCompleteStage {
    /// Finalize the bundle.
    ///
    /// # Errors
    ///
    /// See [`ExtensionBundleBuilder::build`].
    pub fn build(self) -> Result<ExtensionBundle, Error> {
        self.parent.build()
    }
}

// ── Passive lifecycle (instance-policy selection) ────────────────────────────

/// Lifecycle-selected: Passive (no event loop, capabilities only).
///
/// Select the instance policy with [`cloned()`](Self::cloned) or
/// [`constructed()`](Self::constructed) before registering variants.
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

    /// Select the **constructed-per-consumer** instance policy: each
    /// consumer receives a newly-constructed instance from the stored
    /// closure.
    #[must_use]
    pub fn constructed(self) -> PassiveConstructedStage {
        PassiveConstructedStage {
            parent: self.parent,
        }
    }
}

// ── Passive + Cloned (shared/local stages) ───────────────────────────────────

/// Passive + Cloned (clone-per-consumer) stage. Awaiting first
/// `.shared(...)` or `.local(...)` registration.
#[doc(hidden)]
pub struct PassiveClonedStage {
    parent: ExtensionBundleBuilder,
}

impl PassiveClonedStage {
    /// Register the shared (Send) variant. Consumers receive
    /// independent clones of the prototype.
    #[must_use]
    pub fn shared<E>(mut self, extension: E) -> PassiveClonedSharedStage
    where
        E: Clone + Send + 'static,
    {
        self.parent.set_shared_cloned(extension);
        PassiveClonedSharedStage {
            parent: self.parent,
        }
    }

    /// Register the local (!Send) variant. Consumers receive
    /// `Rc::clone`s of the stored instance.
    #[must_use]
    pub fn local<E>(mut self, extension: std::rc::Rc<E>) -> PassiveClonedLocalStage
    where
        E: 'static,
    {
        self.parent.set_local_cloned(extension);
        PassiveClonedLocalStage {
            parent: self.parent,
        }
    }
}

/// Passive + Cloned + shared registered. Awaiting optional `.local(...)` or
/// `.build()`.
#[doc(hidden)]
pub struct PassiveClonedSharedStage {
    parent: ExtensionBundleBuilder,
}

impl PassiveClonedSharedStage {
    /// Register the local (!Send) variant alongside the previously-registered
    /// shared variant.
    #[must_use]
    pub fn local<E>(mut self, extension: std::rc::Rc<E>) -> PassiveClonedCompleteStage
    where
        E: 'static,
    {
        self.parent.set_local_cloned(extension);
        PassiveClonedCompleteStage {
            parent: self.parent,
        }
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

/// Passive + Cloned + local registered. Awaiting optional `.shared(...)` or
/// `.build()`.
#[doc(hidden)]
pub struct PassiveClonedLocalStage {
    parent: ExtensionBundleBuilder,
}

impl PassiveClonedLocalStage {
    /// Register the shared (Send) variant alongside the previously-registered
    /// local variant.
    #[must_use]
    pub fn shared<E>(mut self, extension: E) -> PassiveClonedCompleteStage
    where
        E: Clone + Send + 'static,
    {
        self.parent.set_shared_cloned(extension);
        PassiveClonedCompleteStage {
            parent: self.parent,
        }
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

/// Passive + Cloned + both variants registered. Only `.build()` remains.
#[doc(hidden)]
pub struct PassiveClonedCompleteStage {
    parent: ExtensionBundleBuilder,
}

impl PassiveClonedCompleteStage {
    /// Finalize the bundle.
    ///
    /// # Errors
    ///
    /// See [`ExtensionBundleBuilder::build`].
    pub fn build(self) -> Result<ExtensionBundle, Error> {
        self.parent.build()
    }
}

// ── Passive + Constructed (shared/local stages) ──────────────────────────────

/// Passive + Constructed (constructed-per-consumer) stage. Awaiting first
/// `.shared(...)` or `.local(...)` registration.
#[doc(hidden)]
pub struct PassiveConstructedStage {
    parent: ExtensionBundleBuilder,
}

impl PassiveConstructedStage {
    /// Register the shared (Send) variant via a factory closure. Each
    /// consumer receives a newly-constructed instance.
    ///
    /// `F: Clone` is required so per-node factories can duplicate the
    /// closure; closures capturing `Clone` configuration (e.g.
    /// `Arc<Config>`, `String`) satisfy this automatically.
    #[must_use]
    pub fn shared<E, F>(mut self, produce: F) -> PassiveConstructedSharedStage
    where
        E: Send + 'static,
        F: Fn() -> E + Clone + Send + 'static,
    {
        self.parent.set_shared_constructed::<E, F>(produce);
        PassiveConstructedSharedStage {
            parent: self.parent,
        }
    }

    /// Register the local (!Send) variant via a factory closure. Each
    /// consumer receives a freshly-constructed instance.
    #[must_use]
    pub fn local<E, F>(mut self, produce: F) -> PassiveConstructedLocalStage
    where
        E: 'static,
        F: Fn() -> std::rc::Rc<E> + Clone + 'static,
    {
        self.parent.set_local_constructed::<E, F>(produce);
        PassiveConstructedLocalStage {
            parent: self.parent,
        }
    }
}

/// Passive + Constructed + shared registered. Awaiting optional `.local(...)`
/// or `.build()`.
#[doc(hidden)]
pub struct PassiveConstructedSharedStage {
    parent: ExtensionBundleBuilder,
}

impl PassiveConstructedSharedStage {
    /// Register the local (!Send) variant alongside the previously-registered
    /// shared variant.
    #[must_use]
    pub fn local<E, F>(mut self, produce: F) -> PassiveConstructedCompleteStage
    where
        E: 'static,
        F: Fn() -> std::rc::Rc<E> + Clone + 'static,
    {
        self.parent.set_local_constructed::<E, F>(produce);
        PassiveConstructedCompleteStage {
            parent: self.parent,
        }
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

/// Passive + Constructed + local registered. Awaiting optional `.shared(...)`
/// or `.build()`.
#[doc(hidden)]
pub struct PassiveConstructedLocalStage {
    parent: ExtensionBundleBuilder,
}

impl PassiveConstructedLocalStage {
    /// Register the shared (Send) variant alongside the previously-registered
    /// local variant.
    #[must_use]
    pub fn shared<E, F>(mut self, produce: F) -> PassiveConstructedCompleteStage
    where
        E: Send + 'static,
        F: Fn() -> E + Clone + Send + 'static,
    {
        self.parent.set_shared_constructed::<E, F>(produce);
        PassiveConstructedCompleteStage {
            parent: self.parent,
        }
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

/// Passive + Constructed + both variants registered. Only `.build()` remains.
#[doc(hidden)]
pub struct PassiveConstructedCompleteStage {
    parent: ExtensionBundleBuilder,
}

impl PassiveConstructedCompleteStage {
    /// Finalize the bundle.
    ///
    /// # Errors
    ///
    /// See [`ExtensionBundleBuilder::build`].
    pub fn build(self) -> Result<ExtensionBundle, Error> {
        self.parent.build()
    }
}

// ── Background lifecycle (engine-driven, no capabilities) ───────────────────

/// Lifecycle-selected: Background (engine drives an event loop, no
/// capabilities exposed to nodes).
///
/// Pick exactly **one** of [`shared()`](Self::shared) or
/// [`local()`](Self::local) — the bg task instance — then `.build()`. The
/// flavor (shared vs local) chooses how the engine hosts the instance
/// (Send+Clone vs !Send), but only one registration is allowed because
/// a Background extension does not expose capabilities and there is no
/// reason to maintain two parallel instances of the same task.
///
/// Authors writing a Background extension should set
/// [`ExtensionFactory::capabilities`](crate::ExtensionFactory::capabilities)
/// to `None`; that `None` is the engine's runtime signal that this is a
/// Background extension (`register_into` skips capability registration).
#[doc(hidden)]
pub struct BackgroundEmptyStage {
    parent: ExtensionBundleBuilder,
}

impl BackgroundEmptyStage {
    /// Register the shared (Send + Clone) bg task instance.
    ///
    /// Same parameter shape as [`ActiveStage::shared`] — the difference
    /// is purely typestate: Background allows exactly one registration.
    #[must_use]
    pub fn shared<E>(mut self, extension: E) -> BackgroundCompleteStage
    where
        E: shared_ext::Extension + Clone + Send + 'static,
    {
        self.parent.set_shared_active(extension);
        BackgroundCompleteStage {
            parent: self.parent,
        }
    }

    /// Register the local (!Send) bg task instance.
    ///
    /// Same parameter shape as [`ActiveStage::local`] — the difference
    /// is purely typestate: Background allows exactly one registration.
    #[must_use]
    pub fn local<E>(mut self, extension: std::rc::Rc<E>) -> BackgroundCompleteStage
    where
        E: local_ext::Extension + 'static,
    {
        self.parent.set_local_active(extension);
        BackgroundCompleteStage {
            parent: self.parent,
        }
    }
}

/// Background + the single bg task instance registered. Only `.build()`
/// remains — neither a second `.shared(...)` nor `.local(...)` is reachable.
#[doc(hidden)]
pub struct BackgroundCompleteStage {
    parent: ExtensionBundleBuilder,
}

impl BackgroundCompleteStage {
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
/// The typestate stages reachable from this builder statically guarantee
/// that at least one variant (local or shared) is registered before
/// `.build()` becomes callable, and that neither `.shared(...)` nor
/// `.local(...)` can be called twice on the same bundle.
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

    // ── Stage helpers (called by typestate stages) ──────────────────────────

    fn set_shared_active<E>(&mut self, extension: E)
    where
        E: shared_ext::Extension + Clone + Send + 'static,
    {
        let for_factory = extension.clone();
        self.shared = Some(SharedDecomposed {
            extension: Some(Box::new(extension)),
            instance_factory: SharedInstanceFactory::new(move || {
                Box::new(for_factory.clone()) as Box<dyn Any + Send>
            }),
            type_id: TypeId::of::<E>(),
        });
    }

    fn set_local_active<E>(&mut self, extension: std::rc::Rc<E>)
    where
        E: local_ext::Extension + 'static,
    {
        let for_factory = std::rc::Rc::clone(&extension);
        self.local = Some(LocalDecomposed {
            extension: Some(extension),
            instance_factory: LocalInstanceFactory::new(move || {
                std::rc::Rc::clone(&for_factory) as std::rc::Rc<dyn Any>
            }),
            type_id: TypeId::of::<E>(),
        });
    }

    fn set_shared_cloned<E>(&mut self, extension: E)
    where
        E: Clone + Send + 'static,
    {
        self.shared = Some(SharedDecomposed {
            extension: None,
            instance_factory: SharedInstanceFactory::new(move || {
                Box::new(extension.clone()) as Box<dyn Any + Send>
            }),
            type_id: TypeId::of::<E>(),
        });
    }

    fn set_local_cloned<E>(&mut self, extension: std::rc::Rc<E>)
    where
        E: 'static,
    {
        self.local = Some(LocalDecomposed {
            extension: None,
            instance_factory: LocalInstanceFactory::new(move || {
                std::rc::Rc::clone(&extension) as std::rc::Rc<dyn Any>
            }),
            type_id: TypeId::of::<E>(),
        });
    }

    fn set_shared_constructed<E, F>(&mut self, produce: F)
    where
        E: Send + 'static,
        F: Fn() -> E + Clone + Send + 'static,
    {
        self.shared = Some(SharedDecomposed {
            extension: None,
            instance_factory: SharedInstanceFactory::new(move || {
                Box::new(produce()) as Box<dyn Any + Send>
            }),
            type_id: TypeId::of::<E>(),
        });
    }

    fn set_local_constructed<E, F>(&mut self, produce: F)
    where
        E: 'static,
        F: Fn() -> std::rc::Rc<E> + Clone + 'static,
    {
        self.local = Some(LocalDecomposed {
            extension: None,
            instance_factory: LocalInstanceFactory::new(move || produce() as std::rc::Rc<dyn Any>),
            type_id: TypeId::of::<E>(),
        });
    }

    /// Select the **Active** lifecycle for this extension bundle.
    /// The engine will drive an event loop for whichever sides are
    /// registered.
    ///
    /// Instance policy is implicitly clone-per-consumer —
    /// constructed-per-consumer (factory closure) is Passive-only.
    #[must_use]
    pub fn active(self) -> ActiveStage {
        ActiveStage { parent: self }
    }

    /// Select the **Passive** lifecycle for this extension bundle. No
    /// event loop is spawned; the extension exposes capabilities only.
    ///
    /// Continue with [`PassiveStage::cloned`] (clone-per-consumer)
    /// or [`PassiveStage::constructed`] (constructed-per-consumer).
    #[must_use]
    pub fn passive(self) -> PassiveStage {
        PassiveStage { parent: self }
    }

    /// Select the **Background** lifecycle for this extension bundle.
    ///
    /// Background extensions are engine-driven services that run a
    /// `start()` event loop but expose **no** capabilities to nodes —
    /// periodic reporters, schedulers, health monitors, global
    /// coordinators. The engine hosts them exactly like Active
    /// extensions (same control channel, same shutdown sequencing); the
    /// only difference is that capability registration is a no-op.
    ///
    /// Pick exactly one of `.shared(...)` / `.local(...)` on the
    /// returned stage, then `.build()`. The factory built around this
    /// bundle should set
    /// [`ExtensionFactory::capabilities`](crate::ExtensionFactory::capabilities)
    /// to `None`.
    #[must_use]
    pub fn background(self) -> BackgroundEmptyStage {
        BackgroundEmptyStage { parent: self }
    }

    /// Build the [`ExtensionBundle`].
    ///
    /// At least one variant (local or shared) is statically guaranteed
    /// to be registered by the typestate stages, and duplicate
    /// registration is statically prevented. The remaining error
    /// condition is a same-type dual registration.
    ///
    /// # Errors
    ///
    /// Returns an error if both variants use the same concrete type
    /// (dual registration requires distinct local and shared
    /// implementations).
    pub(super) fn build(self) -> Result<ExtensionBundle, Error> {
        if let (Some(local), Some(shared)) = (&self.local, &self.shared) {
            if local.type_id == shared.type_id {
                return Err(Error::InternalError {
                    message: "local and shared variants must use different concrete types; \
                              register only one execution model for single-variant extensions"
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
            // Unreachable: typestate stages guarantee at least one variant is
            // registered before `.build()` is callable. Kept as a defensive
            // guard against future refactors.
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
