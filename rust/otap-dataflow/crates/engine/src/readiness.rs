// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared startup-readiness primitives for engine components.
//!
//! Each participating component receives a [`ReadinessSignaller`] and the
//! engine waits on the matching [`ReadinessProbe`]. Components signal only
//! after their fallible runtime initialization has completed.
//!
//! # Examples
//!
//! Component side:
//!
//! ```ignore
//! async fn start(self: Box<Self>, ctrl: ControlChannel, eh: EffectHandler)
//!     -> Result<TerminalState, Error>
//! {
//!     self.fetch_first_token().await?;
//!     eh.signal_ready();
//!     self.run_main_loop(ctrl, eh).await
//! }
//! ```
//!
//! Wiring side:
//!
//! ```ignore
//! let (signaller, probe) = ReadinessSignaller::pair_with_external_timeout();
//! ```

use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::oneshot;

/// Send-side handle for a component's readiness signal. Cloneable;
/// only the first [`ReadinessSignaller::ready`] call delivers.
pub struct ReadinessSignaller {
    inner: Arc<ReadinessInner>,
}

struct ReadinessInner {
    tx_slot: Mutex<Option<oneshot::Sender<()>>>,
}

impl ReadinessSignaller {
    /// Construct a matched ([`ReadinessSignaller`], [`ReadinessProbe`]) pair.
    pub(crate) fn pair(timeout: Duration) -> (Self, ReadinessProbe) {
        let (tx, rx) = oneshot::channel();
        let sig = Self {
            inner: Arc::new(ReadinessInner {
                tx_slot: Mutex::new(Some(tx)),
            }),
        };
        let probe = ReadinessProbe {
            rx,
            timeout: Some(timeout),
        };
        (sig, probe)
    }

    /// Constructs an observable readiness pair for receiver tests in downstream crates.
    #[cfg(any(test, feature = "test-utils"))]
    #[must_use]
    pub fn pair_for_test(timeout: Duration) -> (Self, ReadinessProbe) {
        Self::pair(timeout)
    }

    /// Construct a matched pair whose timeout is owned by the caller.
    ///
    /// Receiver startup uses the controller's rollout or recovery deadline
    /// instead of layering an independent component timeout on top.
    pub(crate) fn pair_with_external_timeout() -> (Self, ReadinessProbe) {
        let (tx, rx) = oneshot::channel();
        let sig = Self {
            inner: Arc::new(ReadinessInner {
                tx_slot: Mutex::new(Some(tx)),
            }),
        };
        let probe = ReadinessProbe { rx, timeout: None };
        (sig, probe)
    }

    /// Signal that this variant is ready. Idempotent across clones.
    pub fn ready(&self) {
        if let Some(tx) = self
            .inner
            .tx_slot
            .lock()
            .expect("ReadinessSignaller tx_slot mutex poisoned")
            .take()
        {
            let _ = tx.send(());
        }
    }

    /// Returns `true` once any clone has fired.
    #[must_use]
    pub fn is_ready(&self) -> bool {
        self.inner
            .tx_slot
            .lock()
            .expect("ReadinessSignaller tx_slot mutex poisoned")
            .is_none()
    }
}

impl Clone for ReadinessSignaller {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl fmt::Debug for ReadinessSignaller {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReadinessSignaller")
            .field("is_ready", &self.is_ready())
            .finish_non_exhaustive()
    }
}

/// Engine-side half of a readiness probe.
pub struct ReadinessProbe {
    rx: oneshot::Receiver<()>,
    timeout: Option<Duration>,
}

impl ReadinessProbe {
    /// Per-variant timeout the engine layers on top of [`wait_ready`](Self::wait_ready).
    #[must_use]
    pub fn timeout(&self) -> Duration {
        self.timeout
            .expect("readiness timeout is owned by the external caller")
    }

    /// Resolves on signal, or [`ReadinessProbeError::SignallerDropped`]
    /// if every clone dropped without firing. Single-use.
    pub async fn wait_ready(self) -> Result<(), ReadinessProbeError> {
        match self.rx.await {
            Ok(()) => Ok(()),
            Err(_) => Err(ReadinessProbeError::SignallerDropped),
        }
    }
}

impl fmt::Debug for ReadinessProbe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReadinessProbe")
            .field("timeout", &self.timeout)
            .finish_non_exhaustive()
    }
}

/// Errors observable while awaiting a [`ReadinessProbe`]. Timeouts are
/// surfaced separately by the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadinessProbeError {
    /// Every [`ReadinessSignaller`] clone dropped without firing.
    SignallerDropped,
}

impl fmt::Display for ReadinessProbeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SignallerDropped => {
                f.write_str("all ReadinessSignaller clones dropped without calling `.ready()`")
            }
        }
    }
}

impl std::error::Error for ReadinessProbeError {}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::timeout;

    #[tokio::test]
    async fn ready_before_wait_resolves_immediately() {
        let (sig, probe) = ReadinessSignaller::pair(Duration::from_secs(1));
        sig.ready();
        timeout(Duration::from_millis(50), probe.wait_ready())
            .await
            .expect("did not time out")
            .expect("ready signal observed");
    }

    #[tokio::test]
    async fn wait_resolves_when_ready_fires_later() {
        let (sig, probe) = ReadinessSignaller::pair(Duration::from_secs(1));
        let sig_for_task = sig.clone();
        drop(sig);

        let waiter = tokio::spawn(async move { probe.wait_ready().await });
        tokio::task::yield_now().await;
        sig_for_task.ready();

        let join = timeout(Duration::from_millis(100), waiter)
            .await
            .expect("did not time out");
        join.expect("task did not panic").expect("ready observed");
    }

    #[tokio::test]
    async fn ready_is_idempotent_across_clones_and_calls() {
        let (sig, probe) = ReadinessSignaller::pair(Duration::from_secs(1));
        let c1 = sig.clone();
        let c2 = sig.clone();

        c1.ready();
        c2.ready();
        sig.ready();
        sig.ready();

        assert!(sig.is_ready());
        assert!(c1.is_ready());
        assert!(c2.is_ready());

        assert!(sig.inner.tx_slot.lock().expect("mutex").is_none());

        timeout(Duration::from_millis(50), probe.wait_ready())
            .await
            .expect("did not time out")
            .expect("ready observed exactly once");
    }

    #[tokio::test]
    async fn concurrent_ready_calls_are_safe() {
        let (sig, probe) = ReadinessSignaller::pair(Duration::from_secs(1));
        let mut handles = Vec::new();
        for _ in 0..64 {
            let c = sig.clone();
            handles.push(tokio::spawn(async move { c.ready() }));
        }
        for h in handles {
            h.await.expect("join");
        }
        assert!(sig.is_ready());

        timeout(Duration::from_millis(50), probe.wait_ready())
            .await
            .expect("did not time out")
            .expect("exactly one ready delivered");
    }

    #[tokio::test]
    async fn all_signallers_dropped_without_fire_yields_error() {
        // Long timeout: drop-detection should resolve the probe
        // immediately, not wait this out.
        let (sig, probe) = ReadinessSignaller::pair(Duration::from_secs(60));
        let c1 = sig.clone();
        drop(sig);
        drop(c1);

        match timeout(Duration::from_millis(50), probe.wait_ready()).await {
            Ok(Err(ReadinessProbeError::SignallerDropped)) => {}
            other => panic!("expected immediate SignallerDropped, got {other:?}"),
        }
    }

    #[test]
    fn timeout_is_carried_on_probe() {
        let (_sig, probe) = ReadinessSignaller::pair(Duration::from_millis(42));
        assert_eq!(probe.timeout(), Duration::from_millis(42));
    }

    #[test]
    fn debug_redacts_internals() {
        let (sig, _probe) = ReadinessSignaller::pair(Duration::from_secs(1));
        let s = format!("{sig:?}");
        assert!(s.contains("ReadinessSignaller"));
        assert!(s.contains("is_ready"));
    }

    #[test]
    fn probe_debug_includes_timeout_only() {
        let (_sig, probe) = ReadinessSignaller::pair(Duration::from_millis(42));
        let s = format!("{probe:?}");
        assert!(s.contains("ReadinessProbe"));
        assert!(s.contains("timeout"));
    }
}
