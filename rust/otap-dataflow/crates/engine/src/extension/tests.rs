// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Unit tests for the extension module (wrapper + builder + control channel).

use super::wrapper::{EffectHandler, ExtensionWrapper};
use crate::channel_metrics::ChannelMetricsRegistry;
use crate::config::ExtensionConfig;
use crate::control::ExtensionControlMsg;
use crate::error::Error;
use crate::local::extension as local_ext;
use crate::message::Sender;
use crate::shared::extension as shared_ext;
use crate::shared::message::{SharedReceiver, SharedSender};
use crate::terminal_state::TerminalState;
use crate::testing::CtrlMsgCounters;
use async_trait::async_trait;
use otap_df_config::ExtensionId;
use otap_df_config::extension::ExtensionUserConfig;
use otap_df_telemetry::reporter::MetricsReporter;
use serde_json::Value;
use shared_ext::Extension as SharedExtension;
use std::sync::Arc;
use std::time::Instant;

fn test_metrics_reporter() -> MetricsReporter {
    let (tx, _rx) = flume::bounded(1);
    MetricsReporter::new(tx)
}

fn ext_config(name: &'static str) -> (ExtensionId, Arc<ExtensionUserConfig>, ExtensionConfig) {
    (
        name.into(),
        Arc::new(ExtensionUserConfig::new(
            "urn:otap:test".into(),
            Value::Null,
        )),
        ExtensionConfig::new(name),
    )
}

#[derive(Clone)]
struct TestSharedExt {
    counter: CtrlMsgCounters,
}
impl TestSharedExt {
    fn new(c: CtrlMsgCounters) -> Self {
        Self { counter: c }
    }
}

#[async_trait]
impl SharedExtension for TestSharedExt {
    async fn start(
        self: Box<Self>,
        mut ctrl: shared_ext::ControlChannel,
        _eh: EffectHandler,
    ) -> Result<TerminalState, Error> {
        loop {
            match ctrl.recv().await? {
                ExtensionControlMsg::Config { .. } => self.counter.increment_config(),
                ExtensionControlMsg::Shutdown { .. } => {
                    self.counter.increment_shutdown();
                    break;
                }
                ExtensionControlMsg::CollectTelemetry { .. } => {}
            }
        }
        Ok(TerminalState::default())
    }
}

#[derive(Clone)]
struct TestLocalExt {
    _counter: CtrlMsgCounters,
}
impl TestLocalExt {
    fn new(c: CtrlMsgCounters) -> Self {
        Self { _counter: c }
    }
}

#[async_trait(?Send)]
impl crate::local::extension::Extension for TestLocalExt {
    async fn start(
        self: std::rc::Rc<Self>,
        mut ctrl: local_ext::ControlChannel,
        _eh: EffectHandler,
    ) -> Result<TerminalState, Error> {
        loop {
            if let ExtensionControlMsg::Shutdown { .. } = ctrl.recv().await? {
                break;
            }
        }
        Ok(TerminalState::default())
    }
}

#[test]
fn test_shared_active() {
    let (n, u, c) = ext_config("sa");
    let mut set = ExtensionWrapper::builder(n, u, &c)
        .active()
        .shared(TestSharedExt::new(CtrlMsgCounters::new()))
        .build()
        .unwrap();
    assert!(set.local().is_none());
    let w = set.take_shared().unwrap();
    assert!(!w.is_passive());
    assert!(w.extension_control_sender().is_some());
}

#[test]
fn test_shared_passive() {
    let (n, u, c) = ext_config("sp");
    let mut set = ExtensionWrapper::builder(n, u, &c)
        .passive()
        .cloned()
        .shared("data".to_string())
        .build()
        .unwrap();
    assert!(set.local().is_none());
    let w = set.take_shared().unwrap();
    assert!(w.is_passive());
    assert!(w.extension_control_sender().is_none());
}

#[test]
fn test_local_active() {
    let (n, u, c) = ext_config("la");
    let mut set = ExtensionWrapper::builder(n, u, &c)
        .active()
        .local(std::rc::Rc::new(TestLocalExt::new(CtrlMsgCounters::new())))
        .build()
        .unwrap();
    assert!(set.shared().is_none());
    let w = set.take_local().unwrap();
    assert!(!w.is_passive());
    assert!(w.extension_control_sender().is_some());
}

#[test]
fn test_local_passive() {
    let (n, u, c) = ext_config("lp");
    let mut set = ExtensionWrapper::builder(n, u, &c)
        .passive()
        .cloned()
        .local(std::rc::Rc::new(42u32))
        .build()
        .unwrap();
    assert!(set.shared().is_none());
    let w = set.take_local().unwrap();
    assert!(w.is_passive());
}

#[test]
fn test_empty_returns_error() {
    let (n, u, c) = ext_config("e");
    let result = ExtensionWrapper::builder(n, u, &c).build();
    assert!(result.is_err());
}

#[test]
fn test_same_type_dual_returns_error() {
    // TestSharedExt implements only shared::Extension, so we need
    // a type that implements both to test the same-type guard.
    // Since TestLocalExt and TestSharedExt are different types,
    // we can't easily trigger this. Instead, create a dual-impl type.
    #[derive(Clone)]
    struct DualExt;

    #[async_trait]
    impl SharedExtension for DualExt {
        async fn start(
            self: Box<Self>,
            _ctrl: shared_ext::ControlChannel,
            _eh: EffectHandler,
        ) -> Result<TerminalState, Error> {
            Ok(TerminalState::default())
        }
    }

    #[async_trait(?Send)]
    impl crate::local::extension::Extension for DualExt {
        async fn start(
            self: std::rc::Rc<Self>,
            _ctrl: local_ext::ControlChannel,
            _eh: EffectHandler,
        ) -> Result<TerminalState, Error> {
            Ok(TerminalState::default())
        }
    }

    let (n, u, c) = ext_config("st");
    let result = ExtensionWrapper::builder(n, u, &c)
        .active()
        .local(std::rc::Rc::new(DualExt))
        .shared(DualExt)
        .build();
    assert!(result.is_err());
}

#[test]
fn test_dual_creates_local_and_shared() {
    let (n, u, c) = ext_config("d");
    let mut set = ExtensionWrapper::builder(n, u, &c)
        .active()
        .local(std::rc::Rc::new(TestLocalExt::new(CtrlMsgCounters::new())))
        .shared(TestSharedExt::new(CtrlMsgCounters::new()))
        .build()
        .unwrap();
    let local = set.take_local().unwrap();
    let shared = set.take_shared().unwrap();
    assert!(!local.is_passive());
    assert!(!shared.is_passive());
    assert!(local.extension_control_sender().is_some());
    assert!(shared.extension_control_sender().is_some());
    assert!(matches!(local, ExtensionWrapper::Local { .. }));
    assert!(matches!(shared, ExtensionWrapper::Shared { .. }));
}

#[test]
fn test_name_and_config_accessors() {
    let (n, u, c) = ext_config("acc");
    let mut set = ExtensionWrapper::builder(n, u, &c)
        .active()
        .shared(TestSharedExt::new(CtrlMsgCounters::new()))
        .build()
        .unwrap();
    let w = set.take_shared().unwrap();
    assert_eq!(w.name().as_ref(), "acc");
    assert_eq!(w.user_config().r#type.as_ref(), "urn:otap:test");
}

#[test]
fn test_control_msg_is_shutdown() {
    assert!(
        ExtensionControlMsg::Shutdown {
            deadline: Instant::now(),
            reason: "t".into()
        }
        .is_shutdown()
    );
    assert!(
        !ExtensionControlMsg::Config {
            config: Value::Null
        }
        .is_shutdown()
    );
}

#[test]
fn test_shared_start_shutdown() {
    let (rt, ls) = crate::testing::setup_test_runtime();
    rt.block_on(ls.run_until(async {
        let ctr = CtrlMsgCounters::new();
        let (n, u, c) = ext_config("ss");
        let w = ExtensionWrapper::builder(n, u, &c)
            .active()
            .shared(TestSharedExt::new(ctr.clone()))
            .build()
            .unwrap()
            .take_shared()
            .unwrap();
        let s = w.extension_control_sender().unwrap();
        let h = tokio::task::spawn_local(async move { w.start(test_metrics_reporter()).await });
        s.send(ExtensionControlMsg::Config {
            config: Value::Null,
        })
        .await
        .unwrap();
        s.send(ExtensionControlMsg::Shutdown {
            deadline: Instant::now(),
            reason: "d".into(),
        })
        .await
        .unwrap();
        assert!(h.await.unwrap().is_ok());
        ctr.assert(0, 0, 1, 1);
    }));
}

#[test]
fn test_local_start_shutdown() {
    let (rt, ls) = crate::testing::setup_test_runtime();
    rt.block_on(ls.run_until(async {
        let ctr = CtrlMsgCounters::new();
        let (n, u, c) = ext_config("ls");
        let w = ExtensionWrapper::builder(n, u, &c)
            .active()
            .local(std::rc::Rc::new(TestLocalExt::new(ctr.clone())))
            .build()
            .unwrap()
            .take_local()
            .unwrap();
        let s = w.extension_control_sender().unwrap();
        let h = tokio::task::spawn_local(async move { w.start(test_metrics_reporter()).await });
        s.send(ExtensionControlMsg::Shutdown {
            deadline: Instant::now(),
            reason: "d".into(),
        })
        .await
        .unwrap();
        assert!(h.await.unwrap().is_ok());
        ctr.assert(0, 0, 0, 0);
    }));
}

#[tokio::test]
async fn test_ctrl_immediate_shutdown() {
    let (tx, rx) = tokio::sync::mpsc::channel(8);
    let mut ch = shared_ext::ControlChannel::new(SharedReceiver::mpsc(rx));
    SharedSender::mpsc(tx)
        .send(ExtensionControlMsg::Shutdown {
            deadline: Instant::now(),
            reason: "i".into(),
        })
        .await
        .unwrap();
    assert!(ch.recv().await.unwrap().is_shutdown());
    assert!(ch.recv().await.is_err());
}

#[tokio::test]
async fn test_ctrl_config_then_shutdown() {
    let (tx, rx) = tokio::sync::mpsc::channel(8);
    let s = SharedSender::mpsc(tx);
    let mut ch = shared_ext::ControlChannel::new(SharedReceiver::mpsc(rx));
    s.send(ExtensionControlMsg::Config {
        config: Value::String("h".into()),
    })
    .await
    .unwrap();
    s.send(ExtensionControlMsg::Shutdown {
        deadline: Instant::now(),
        reason: "d".into(),
    })
    .await
    .unwrap();
    assert!(!ch.recv().await.unwrap().is_shutdown());
    assert!(ch.recv().await.unwrap().is_shutdown());
    assert!(ch.recv().await.is_err());
}

#[tokio::test]
async fn test_ctrl_delayed_shutdown() {
    let (tx, rx) = tokio::sync::mpsc::channel(8);
    let mut ch = shared_ext::ControlChannel::new(SharedReceiver::mpsc(rx));
    let dl = Instant::now() + std::time::Duration::from_millis(50);
    let sender = SharedSender::mpsc(tx);
    sender
        .send(ExtensionControlMsg::Shutdown {
            deadline: dl,
            reason: "dl".into(),
        })
        .await
        .unwrap();
    // Keep sender alive so control_rx doesn't close before the deadline.
    assert!(ch.recv().await.unwrap().is_shutdown());
    assert!(Instant::now() >= dl);
    drop(sender);
}

#[tokio::test]
async fn test_ctrl_delayed_shutdown_sender_dropped() {
    // Edge case: sender drops during grace period. The pending shutdown
    // should still be delivered instead of returning RecvError::Closed.
    let (tx, rx) = tokio::sync::mpsc::channel(8);
    let mut ch = shared_ext::ControlChannel::new(SharedReceiver::mpsc(rx));
    let dl = Instant::now() + std::time::Duration::from_millis(100);
    SharedSender::mpsc(tx)
        .send(ExtensionControlMsg::Shutdown {
            deadline: dl,
            reason: "dl".into(),
        })
        .await
        .unwrap();
    // Sender is dropped here — channel closes during grace period.
    // recv() should still return the pending shutdown, not Err(Closed).
    assert!(ch.recv().await.unwrap().is_shutdown());
}

#[tokio::test]
async fn test_ctrl_collect_telemetry() {
    let (tx, rx) = tokio::sync::mpsc::channel(8);
    let mut ch = shared_ext::ControlChannel::new(SharedReceiver::mpsc(rx));
    SharedSender::mpsc(tx)
        .send(ExtensionControlMsg::CollectTelemetry {
            metrics_reporter: test_metrics_reporter(),
        })
        .await
        .unwrap();
    assert!(matches!(
        ch.recv().await.unwrap(),
        ExtensionControlMsg::CollectTelemetry { .. }
    ));
}

#[tokio::test]
async fn test_ctrl_closed() {
    let (tx, rx) = tokio::sync::mpsc::channel::<ExtensionControlMsg>(8);
    let mut ch = shared_ext::ControlChannel::new(SharedReceiver::mpsc(rx));
    drop(tx);
    assert!(ch.recv().await.is_err());
}

#[tokio::test]
async fn test_ctrl_sender_send() {
    let (tx, mut rx) = tokio::sync::mpsc::channel(8);
    let sender = crate::control::ExtensionControlSender {
        name: "st".into(),
        sender: Sender::Shared(SharedSender::mpsc(tx)),
    };
    sender
        .send(ExtensionControlMsg::Config {
            config: Value::Null,
        })
        .await
        .unwrap();
    assert!(matches!(
        rx.recv().await.unwrap(),
        ExtensionControlMsg::Config { .. }
    ));
}

#[test]
fn test_effect_handler() {
    let h = EffectHandler::new("eh".into(), test_metrics_reporter());
    assert_eq!(h.extension_id().as_ref(), "eh");
    let c = h.clone();
    assert_eq!(c.extension_id().as_ref(), "eh");
}

#[test]
fn test_passive_ctrl_metrics_noop() {
    let (n, u, c) = ext_config("pm");
    let w = ExtensionWrapper::builder(n, u, &c)
        .passive()
        .cloned()
        .shared("d".to_string())
        .build()
        .unwrap()
        .take_shared()
        .unwrap();
    let (ctx, _) = crate::testing::test_pipeline_ctx();
    let mut cm = ChannelMetricsRegistry::default();
    let w = w.with_control_channel_metrics(&ctx, &mut cm, true);
    assert!(w.is_passive());
}

#[test]
fn test_start_with_telemetry() {
    let (rt, ls) = crate::testing::setup_test_runtime();
    rt.block_on(ls.run_until(async {
        let ctr = CtrlMsgCounters::new();
        let (n, u, c) = ext_config("st");
        let w = ExtensionWrapper::builder(n, u, &c)
            .active()
            .shared(TestSharedExt::new(ctr.clone()))
            .build()
            .unwrap()
            .take_shared()
            .unwrap();
        let s = w.extension_control_sender().unwrap();
        let h = tokio::task::spawn_local(async move { w.start(test_metrics_reporter()).await });
        s.send(ExtensionControlMsg::CollectTelemetry {
            metrics_reporter: test_metrics_reporter(),
        })
        .await
        .unwrap();
        s.send(ExtensionControlMsg::Config {
            config: Value::Null,
        })
        .await
        .unwrap();
        s.send(ExtensionControlMsg::Shutdown {
            deadline: Instant::now(),
            reason: "d".into(),
        })
        .await
        .unwrap();
        assert!(h.await.unwrap().is_ok());
        ctr.assert(0, 0, 1, 1);
    }));
}

#[test]
fn test_active_with_control_channel_metrics() {
    let (n, u, c) = ext_config("acm");
    let w = ExtensionWrapper::builder(n, u, &c)
        .active()
        .shared(TestSharedExt::new(CtrlMsgCounters::new()))
        .build()
        .unwrap()
        .take_shared()
        .unwrap();

    let (ctx, _) = crate::testing::test_pipeline_ctx();
    let mut cm = ChannelMetricsRegistry::default();
    let w = w.with_control_channel_metrics(&ctx, &mut cm, true);
    assert!(!w.is_passive());
    assert!(w.extension_control_sender().is_some());
}

#[test]
fn test_dual_passive() {
    let (n, u, c) = ext_config("dp");
    let mut set = ExtensionWrapper::builder(n, u, &c)
        .passive()
        .cloned()
        .local(std::rc::Rc::new(42u32))
        .shared("data".to_string())
        .build()
        .unwrap();
    assert!(set.take_local().unwrap().is_passive());
    assert!(set.take_shared().unwrap().is_passive());
}

#[test]
fn test_dual_passive_factory() {
    let (n, u, c) = ext_config("dpf");
    let mut set = ExtensionWrapper::builder(n, u, &c)
        .passive()
        .factory()
        .local(|| std::rc::Rc::new(42u32))
        .shared(|| "data".to_string())
        .build()
        .unwrap();
    assert!(set.take_local().unwrap().is_passive());
    assert!(set.take_shared().unwrap().is_passive());
}

#[test]
fn test_extension_set_iter() {
    let (n, u, c) = ext_config("it");
    let set = ExtensionWrapper::builder(n, u, &c)
        .active()
        .local(std::rc::Rc::new(TestLocalExt::new(CtrlMsgCounters::new())))
        .shared(TestSharedExt::new(CtrlMsgCounters::new()))
        .build()
        .unwrap();
    assert_eq!(set.iter().count(), 2);
}
