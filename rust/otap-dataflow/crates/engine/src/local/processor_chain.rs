// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Processor chain: a composite node that runs multiple inline sub-processors
//! sequentially within a single task, eliminating inter-processor channels.
//!
//! Each sub-processor implements [`InlineProcessor`], a narrow trait whose
//! shape structurally prevents operations unsafe for inlined execution:
//! multi-output (memory amplification), fan-out, timers, and wakeups.
//!
//! The chain times the entire sequence with a composite [`ComputeDuration`]
//! so that callers can observe the total compute cost of a record batch
//! passing through all sub-processors.  Individual sub-processor timing
//! is handled internally by each sub-processor via
//! [`ComputeDuration::timed`], matching how standalone processors use
//! `effect_handler.timed()` to exclude deserialization from their
//! duration metrics.
//!
//! # Scheduling fairness
//!
//! The chain inserts `yield_now().await` between stages so the executor can
//! schedule other tasks (receiver accepts, exporter flushes, timer ticks)
//! between each sub-processor.  This bounds uninterrupted CPU time to a
//! single sub-processor's work rather than the sum of the chain.
//!
//! # Non-expanding constraint
//!
//! [`InlineProcessor::process`] returns `Result<InlineOutput<PData>, Error>`,
//! which caps each stage to at most one forward output.  This is a deliberate
//! design constraint: with bounded channels, backpressure is structural; in
//! the inlined model, expanding stages would hold amplified intermediate state
//! in memory while later stages catch up.  Restricting to non-expanding stages
//! avoids that entirely.

use crate::_private::AckNackRouting;
use crate::Interests;
use crate::Unwindable;
use crate::control::{NackMsg, NodeControlMsg};
use crate::error::Error;
use crate::inline_processor::{InlineOutput, InlineProcessor};
use crate::local::processor::{EffectHandler, Processor};
use crate::message::Message;
use crate::process_duration::ComputeDuration;
use async_trait::async_trait;
use otap_df_telemetry::instrument::Timer;
use tokio::task::yield_now;

/// A processor that chains multiple inline sub-processors sequentially.
///
/// When processing data, the chain threads a single `PData` value through
/// each sub-processor in order.  If any stage returns [`InlineOutput::Drop`],
/// processing stops and no output is sent.  The final output (if any) is
/// sent downstream via the real `EffectHandler`.
///
/// The composite [`ComputeDuration`] measures the wall-clock time of the
/// entire chain, giving a single `compute.success.duration` metric that
/// represents the total processing cost per batch.
///
/// When `enable_sub_processor_telemetry` is enabled, the chain also calls
/// [`InlineProcessor::collect_telemetry`] on each sub-processor during
/// `CollectTelemetry` control messages.
pub struct ProcessorChainNode<PData> {
    /// Sub-processors in execution order.
    sub_processors: Vec<Box<dyn InlineProcessor<PData>>>,
    /// Composite compute duration covering the full chain.
    composite_duration: ComputeDuration,
    /// Whether to forward `CollectTelemetry` to each sub-processor.
    enable_sub_processor_telemetry: bool,
}

impl<PData> ProcessorChainNode<PData> {
    /// Create a new processor chain from a list of inline sub-processors.
    ///
    /// `composite_duration` is registered under the chain node's entity
    /// and will report the total compute time across all sub-processors.
    pub fn new(
        sub_processors: Vec<Box<dyn InlineProcessor<PData>>>,
        composite_duration: ComputeDuration,
        enable_sub_processor_telemetry: bool,
    ) -> Self {
        Self {
            sub_processors,
            composite_duration,
            enable_sub_processor_telemetry,
        }
    }
}

#[async_trait(?Send)]
impl<PData: 'static + Clone + Unwindable> Processor<PData> for ProcessorChainNode<PData> {
    async fn process(
        &mut self,
        msg: Message<PData>,
        effect_handler: &mut EffectHandler<PData>,
    ) -> Result<(), Error> {
        match msg {
            Message::Control(control) => {
                match control {
                    NodeControlMsg::CollectTelemetry {
                        mut metrics_reporter,
                    } => {
                        self.composite_duration.report(&mut metrics_reporter);

                        if self.enable_sub_processor_telemetry {
                            for sub in &mut self.sub_processors {
                                sub.collect_telemetry(&mut metrics_reporter);
                            }
                        }
                    }
                    NodeControlMsg::Config { config } => {
                        for sub in &mut self.sub_processors {
                            sub.on_config(config.clone());
                        }
                    }
                    _ => {}
                }
                Ok(())
            }
            Message::PData(pdata) => {
                let num_subs = self.sub_processors.len();
                if num_subs == 0 {
                    effect_handler
                        .send_message(pdata)
                        .await
                        .map_err(Error::from)?;
                    return Ok(());
                }

                // Clone the original data for NACK if any upstream
                // subscriber is listening. Guarded by has_frames()
                // so the clone is skipped when nobody cares.
                let saved_for_nack = if pdata.has_frames() {
                    Some(pdata.clone())
                } else {
                    None
                };

                let interests = effect_handler.node_interests();

                let timer = if interests.contains(Interests::PROCESS_DURATION) {
                    Some(Timer::start())
                } else {
                    None
                };

                let result: Result<(), Error> = async {
                    let mut current = pdata;

                    for (i, sub) in self.sub_processors.iter_mut().enumerate() {
                        // Yield between stages so the executor can schedule
                        // other tasks.  This bounds uninterrupted CPU time
                        // to a single sub-processor's work.
                        if i > 0 {
                            yield_now().await;
                        }

                        let output = sub.process_inline(current);

                        match output? {
                            InlineOutput::Forward(data) => current = data,
                            InlineOutput::Drop => return Ok(()),
                        }
                    }

                    effect_handler
                        .send_message(current)
                        .await
                        .map_err(Error::from)?;
                    Ok(())
                }
                .await;

                // Record composite duration split by outcome.
                if let Some(timer) = timer {
                    self.composite_duration
                        .record(timer.elapsed_nanos(), result.is_ok());
                }

                if let Err(ref e) = result {
                    if let Some(saved) = saved_for_nack {
                        let nack = NackMsg::new(e.to_string(), saved);
                        effect_handler.route_nack(nack).await?;
                    }
                }

                result
            }
        }
    }
}

#[cfg(all(test, feature = "test-utils"))]
mod tests {
    use super::*;
    use crate::control::NodeControlMsg;
    use crate::inline_processor::{InlineOutput, InlineProcessor};
    use crate::local::processor::{EffectHandler, Processor};
    use crate::message::{Message, Sender};
    use crate::node::NodeId;
    use crate::process_duration::ComputeDuration;
    use crate::testing::test_pipeline_ctx;
    use otap_df_channel::mpsc;
    use otap_df_config::PortName;
    use otap_df_telemetry::reporter::MetricsReporter;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    // Helpers
    fn test_node_id(name: &str) -> NodeId {
        NodeId {
            index: 0,
            name: name.to_string().into(),
        }
    }

    fn test_reporter() -> (
        flume::Receiver<otap_df_telemetry::metrics::MetricSetSnapshot>,
        MetricsReporter,
    ) {
        MetricsReporter::create_new_and_receiver(16)
    }

    fn test_effect_handler() -> (EffectHandler<String>, mpsc::Receiver<String>) {
        let (sender, receiver) = mpsc::Channel::new(100);
        let port: PortName = "default".into();
        let mut senders: HashMap<PortName, Sender<String>> = HashMap::new();
        let _ = senders.insert(port.clone(), Sender::new_local_mpsc_sender(sender));
        let (_rx, reporter) = test_reporter();
        let eh = EffectHandler::new(test_node_id("chain"), senders, Some(port), reporter);
        (eh, receiver)
    }

    fn make_chain(subs: Vec<Box<dyn InlineProcessor<String>>>) -> ProcessorChainNode<String> {
        make_chain_with_telemetry(subs, false)
    }

    fn make_chain_with_telemetry(
        subs: Vec<Box<dyn InlineProcessor<String>>>,
        enable_sub_processor_telemetry: bool,
    ) -> ProcessorChainNode<String> {
        let (ctx, _) = test_pipeline_ctx();
        let cd = ComputeDuration::new(&ctx);
        ProcessorChainNode::new(subs, cd, enable_sub_processor_telemetry)
    }

    // Mock inline processors

    /// Appends a suffix to data.  Tracks call count for assertions.
    struct SuffixProcessor {
        suffix: String,
        call_count: Arc<AtomicUsize>,
    }

    impl SuffixProcessor {
        fn new(suffix: &str) -> Self {
            Self {
                suffix: suffix.into(),
                call_count: Arc::new(AtomicUsize::new(0)),
            }
        }
    }

    impl InlineProcessor<String> for SuffixProcessor {
        fn process_inline(&mut self, data: String) -> Result<InlineOutput<String>, Error> {
            let _ = self.call_count.fetch_add(1, Ordering::Relaxed);
            Ok(InlineOutput::Forward(format!("{}{}", data, self.suffix)))
        }
    }

    /// Drops all data (simulates a filter that rejects everything).
    struct DropAllProcessor;

    impl InlineProcessor<String> for DropAllProcessor {
        fn process_inline(&mut self, _data: String) -> Result<InlineOutput<String>, Error> {
            Ok(InlineOutput::Drop)
        }
    }

    /// Always returns an error.
    struct ErrorProcessor;

    impl InlineProcessor<String> for ErrorProcessor {
        fn process_inline(&mut self, _data: String) -> Result<InlineOutput<String>, Error> {
            Err(Error::ProcessorError {
                processor: test_node_id("error"),
                kind: crate::error::ProcessorErrorKind::Other,
                error: "synthetic error".into(),
                source_detail: String::new(),
            })
        }
    }

    /// Tracks `collect_telemetry` calls.
    struct TelemetryTracker {
        telemetry_count: Arc<AtomicUsize>,
    }

    impl TelemetryTracker {
        fn new() -> (Self, Arc<AtomicUsize>) {
            let count = Arc::new(AtomicUsize::new(0));
            (
                Self {
                    telemetry_count: count.clone(),
                },
                count,
            )
        }
    }

    impl InlineProcessor<String> for TelemetryTracker {
        fn process_inline(&mut self, data: String) -> Result<InlineOutput<String>, Error> {
            Ok(InlineOutput::Forward(data))
        }

        fn collect_telemetry(&mut self, _reporter: &mut MetricsReporter) {
            let _ = self.telemetry_count.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Tracks `on_config` calls.
    struct ConfigTracker {
        config_count: Arc<AtomicUsize>,
    }

    impl ConfigTracker {
        fn new() -> (Self, Arc<AtomicUsize>) {
            let count = Arc::new(AtomicUsize::new(0));
            (
                Self {
                    config_count: count.clone(),
                },
                count,
            )
        }
    }

    impl InlineProcessor<String> for ConfigTracker {
        fn process_inline(&mut self, data: String) -> Result<InlineOutput<String>, Error> {
            Ok(InlineOutput::Forward(data))
        }

        fn on_config(&mut self, _config: serde_json::Value) {
            let _ = self.config_count.fetch_add(1, Ordering::Relaxed);
        }
    }

    // Tests

    /// Single sub-processor: data passes through and arrives at the output.
    #[tokio::test]
    async fn single_processor_forwards_data() {
        let proc = SuffixProcessor::new("");
        let count = proc.call_count.clone();
        let mut chain = make_chain(vec![Box::new(proc)]);
        let (mut eh, rx) = test_effect_handler();

        chain
            .process(Message::PData("hello".into()), &mut eh)
            .await
            .unwrap();

        assert_eq!(rx.try_recv().unwrap(), "hello");
        assert_eq!(count.load(Ordering::Relaxed), 1);
    }

    /// Two sub-processors: different suffixes prove sequential ordering.
    #[tokio::test]
    async fn two_processors_compose() {
        let mut chain = make_chain(vec![
            Box::new(SuffixProcessor::new("_A")),
            Box::new(SuffixProcessor::new("_B")),
        ]);
        let (mut eh, rx) = test_effect_handler();

        chain
            .process(Message::PData("hello".into()), &mut eh)
            .await
            .unwrap();

        assert_eq!(rx.try_recv().unwrap(), "hello_A_B");
    }

    /// Three sub-processors compose in order.
    #[tokio::test]
    async fn three_processors_compose() {
        let mut chain = make_chain(vec![
            Box::new(SuffixProcessor::new("_1")),
            Box::new(SuffixProcessor::new("_2")),
            Box::new(SuffixProcessor::new("_3")),
        ]);
        let (mut eh, rx) = test_effect_handler();

        chain
            .process(Message::PData("hello".into()), &mut eh)
            .await
            .unwrap();

        assert_eq!(rx.try_recv().unwrap(), "hello_1_2_3");
    }

    /// A filter processor in the middle drops all data — nothing arrives downstream.
    #[tokio::test]
    async fn filter_drops_data() {
        let tail = SuffixProcessor::new("_tail");
        let count = tail.call_count.clone();
        let mut chain = make_chain(vec![
            Box::new(SuffixProcessor::new("_head")),
            Box::new(DropAllProcessor),
            Box::new(tail),
        ]);
        let (mut eh, rx) = test_effect_handler();

        chain
            .process(Message::PData("hello".into()), &mut eh)
            .await
            .unwrap();

        assert!(rx.try_recv().is_err(), "no data should reach downstream");
        assert_eq!(count.load(Ordering::Relaxed), 0, "tail never called");
    }

    /// An empty chain (zero sub-processors) forwards data unchanged.
    #[tokio::test]
    async fn empty_chain_forwards() {
        let mut chain = make_chain(vec![]);
        let (mut eh, rx) = test_effect_handler();

        chain
            .process(Message::PData("direct".into()), &mut eh)
            .await
            .unwrap();

        assert_eq!(rx.try_recv().unwrap(), "direct");
    }

    /// Composite duration is recorded when PROCESS_DURATION interest is active.
    #[tokio::test]
    async fn composite_duration_recorded() {
        let (ctx, _) = test_pipeline_ctx();
        let cd = ComputeDuration::new(&ctx);
        let mut chain = ProcessorChainNode::new(
            vec![
                Box::new(SuffixProcessor::new("_A")),
                Box::new(SuffixProcessor::new("_B")),
            ],
            cd,
            false,
        );

        // Wire an effect handler with PROCESS_DURATION enabled.
        let (sender, rx) = mpsc::Channel::new(100);
        let port: PortName = "default".into();
        let mut senders: HashMap<PortName, Sender<String>> = HashMap::new();
        let _ = senders.insert(port.clone(), Sender::new_local_mpsc_sender(sender));
        let (_metrics_rx, reporter) = test_reporter();
        let mut eh = EffectHandler::new(test_node_id("chain"), senders, Some(port), reporter);
        eh.core.set_node_interests(Interests::PROCESS_DURATION);

        chain
            .process(Message::PData("hello".into()), &mut eh)
            .await
            .unwrap();

        assert_eq!(rx.try_recv().unwrap(), "hello_A_B");

        // Composite duration should have one observation.
        let snap = chain.composite_duration.snapshot_success().get();
        assert_eq!(snap.count, 1, "one duration observation expected");
        assert!(snap.sum > 0.0, "composite duration should be > 0");
    }

    /// Composite duration is NOT recorded when interest is disabled.
    #[tokio::test]
    async fn composite_duration_not_recorded_when_disabled() {
        let mut chain = make_chain(vec![Box::new(SuffixProcessor::new("_X"))]);
        let (mut eh, _rx) = test_effect_handler();
        // Default interests are empty — no PROCESS_DURATION.

        chain
            .process(Message::PData("hello".into()), &mut eh)
            .await
            .unwrap();

        let snap = chain.composite_duration.snapshot_success().get();
        assert_eq!(snap.count, 0, "no duration should be recorded");
    }

    /// When a sub-processor errors, the composite records into acc_failed.
    #[tokio::test]
    async fn composite_duration_records_failed_on_error() {
        let (ctx, _) = test_pipeline_ctx();
        let cd = ComputeDuration::new(&ctx);
        let mut chain = ProcessorChainNode::new(
            vec![
                Box::new(SuffixProcessor::new("_A")),
                Box::new(ErrorProcessor),
            ],
            cd,
            false,
        );

        let (sender, _rx) = mpsc::Channel::new(100);
        let port: PortName = "default".into();
        let mut senders: HashMap<PortName, Sender<String>> = HashMap::new();
        let _ = senders.insert(port.clone(), Sender::new_local_mpsc_sender(sender));
        let (_metrics_rx, reporter) = test_reporter();
        let mut eh = EffectHandler::new(test_node_id("chain"), senders, Some(port), reporter);
        eh.core.set_node_interests(Interests::PROCESS_DURATION);

        let result = chain.process(Message::PData("hello".into()), &mut eh).await;
        assert!(result.is_err());

        let success_snap = chain.composite_duration.snapshot_success().get();
        assert_eq!(success_snap.count, 0, "success should have no observations");

        let failed_snap = chain.composite_duration.snapshot_failed().get();
        assert_eq!(failed_snap.count, 1, "failed should have one observation");
        assert!(failed_snap.sum > 0.0, "failed duration should be > 0");
    }

    /// Sub-stage telemetry is NOT called when disabled (default).
    #[tokio::test]
    async fn enable_sub_processor_telemetry_disabled_by_default() {
        let (tracker, count) = TelemetryTracker::new();
        let mut chain = make_chain(vec![Box::new(tracker)]);
        let (mut eh, _rx) = test_effect_handler();

        let (_metrics_rx, reporter) = test_reporter();
        chain
            .process(
                Message::Control(NodeControlMsg::CollectTelemetry {
                    metrics_reporter: reporter,
                }),
                &mut eh,
            )
            .await
            .unwrap();

        assert_eq!(
            count.load(Ordering::Relaxed),
            0,
            "collect_telemetry should not be called when disabled"
        );
    }

    /// Sub-stage telemetry IS called when enabled.
    #[tokio::test]
    async fn enable_sub_processor_telemetry_enabled() {
        let (tracker1, count1) = TelemetryTracker::new();
        let (tracker2, count2) = TelemetryTracker::new();
        let mut chain =
            make_chain_with_telemetry(vec![Box::new(tracker1), Box::new(tracker2)], true);
        let (mut eh, _rx) = test_effect_handler();

        let (_metrics_rx, reporter) = test_reporter();
        chain
            .process(
                Message::Control(NodeControlMsg::CollectTelemetry {
                    metrics_reporter: reporter,
                }),
                &mut eh,
            )
            .await
            .unwrap();

        assert_eq!(count1.load(Ordering::Relaxed), 1);
        assert_eq!(count2.load(Ordering::Relaxed), 1);
    }

    /// Config control messages are forwarded to sub-processors via on_config.
    #[tokio::test]
    async fn config_forwarded_to_sub_processors() {
        let (tracker1, count1) = ConfigTracker::new();
        let (tracker2, count2) = ConfigTracker::new();
        let mut chain = make_chain(vec![Box::new(tracker1), Box::new(tracker2)]);
        let (mut eh, _rx) = test_effect_handler();

        chain
            .process(
                Message::Control(NodeControlMsg::Config {
                    config: serde_json::json!({"key": "value"}),
                }),
                &mut eh,
            )
            .await
            .unwrap();

        assert_eq!(count1.load(Ordering::Relaxed), 1);
        assert_eq!(count2.load(Ordering::Relaxed), 1);
    }

    /// Error in the first sub-processor prevents later sub-processors from running.
    #[tokio::test]
    async fn error_in_first_processor_skips_remaining() {
        let tail = SuffixProcessor::new("_tail");
        let tail_count = tail.call_count.clone();
        let mut chain = make_chain(vec![Box::new(ErrorProcessor), Box::new(tail)]);
        let (mut eh, rx) = test_effect_handler();

        let result = chain.process(Message::PData("hello".into()), &mut eh).await;
        assert!(result.is_err());
        assert_eq!(tail_count.load(Ordering::Relaxed), 0, "tail never called");
        assert!(rx.try_recv().is_err(), "nothing sent downstream");
    }

    /// Drop records composite duration as success (drop is intentional, not a failure).
    #[tokio::test]
    async fn drop_records_success_duration() {
        let (ctx, _) = test_pipeline_ctx();
        let cd = ComputeDuration::new(&ctx);
        let mut chain = ProcessorChainNode::new(
            vec![
                Box::new(SuffixProcessor::new("_head")),
                Box::new(DropAllProcessor),
                Box::new(SuffixProcessor::new("_tail")),
            ],
            cd,
            false,
        );

        let (sender, _rx) = mpsc::Channel::new(100);
        let port: PortName = "default".into();
        let mut senders: HashMap<PortName, Sender<String>> = HashMap::new();
        let _ = senders.insert(port.clone(), Sender::new_local_mpsc_sender(sender));
        let (_metrics_rx, reporter) = test_reporter();
        let mut eh = EffectHandler::new(test_node_id("chain"), senders, Some(port), reporter);
        eh.core.set_node_interests(Interests::PROCESS_DURATION);

        chain
            .process(Message::PData("hello".into()), &mut eh)
            .await
            .unwrap();

        let success_snap = chain.composite_duration.snapshot_success().get();
        assert_eq!(success_snap.count, 1, "drop counts as success");

        let failed_snap = chain.composite_duration.snapshot_failed().get();
        assert_eq!(failed_snap.count, 0, "drop is not a failure");
    }

    /// With String PData (has_frames = false), error does not attempt NACK.
    /// This verifies the guard works — no pipeline completion sender is wired,
    /// so an unguarded route_nack call would panic/error.
    #[tokio::test]
    async fn error_without_frames_does_not_nack() {
        let mut chain = make_chain(vec![Box::new(ErrorProcessor)]);
        let (mut eh, _rx) = test_effect_handler();
        // No pipeline completion sender — route_nack would fail if called.

        let result = chain.process(Message::PData("hello".into()), &mut eh).await;
        assert!(result.is_err(), "error propagates");
        // Test passes because saved_for_nack was None (String has no frames),
        // so route_nack was never called.
    }

    /// Chain is reusable across multiple batches — no one-shot state.
    #[tokio::test]
    async fn chain_processes_multiple_batches() {
        let mut chain = make_chain(vec![
            Box::new(SuffixProcessor::new("_A")),
            Box::new(SuffixProcessor::new("_B")),
        ]);
        let (mut eh, rx) = test_effect_handler();

        for i in 0..3 {
            chain
                .process(Message::PData(format!("batch_{i}")), &mut eh)
                .await
                .unwrap();
        }

        assert_eq!(rx.try_recv().unwrap(), "batch_0_A_B");
        assert_eq!(rx.try_recv().unwrap(), "batch_1_A_B");
        assert_eq!(rx.try_recv().unwrap(), "batch_2_A_B");
    }

    /// Drop at the very first sub-processor — nothing downstream, returns Ok.
    #[tokio::test]
    async fn drop_at_first_processor() {
        let tail = SuffixProcessor::new("_tail");
        let count = tail.call_count.clone();
        let mut chain = make_chain(vec![Box::new(DropAllProcessor), Box::new(tail)]);
        let (mut eh, rx) = test_effect_handler();

        chain
            .process(Message::PData("hello".into()), &mut eh)
            .await
            .unwrap();

        assert!(rx.try_recv().is_err(), "nothing sent downstream");
        assert_eq!(count.load(Ordering::Relaxed), 0, "tail never called");
    }

    /// Chain does NOT externally time sub-processors. Each sub-processor
    /// is responsible for calling ComputeDuration::timed() internally on
    /// its core work, matching standalone processors that use
    /// effect_handler.timed(). This avoids inflating timing with
    /// deserialization costs.
    #[tokio::test]
    async fn chain_does_not_externally_time_sub_processors() {
        let (ctx, _) = test_pipeline_ctx();
        let sub_cd = ComputeDuration::new(&ctx);

        struct TimedProcessor {
            cd: ComputeDuration,
        }
        impl InlineProcessor<String> for TimedProcessor {
            fn process_inline(&mut self, data: String) -> Result<InlineOutput<String>, Error> {
                // Does NOT call self.cd.timed() — simulates a processor
                // that hasn't added internal timing yet.
                Ok(InlineOutput::Forward(data))
            }
            fn compute_duration(&self) -> Option<&ComputeDuration> {
                Some(&self.cd)
            }
        }

        let chain_cd = ComputeDuration::new(&ctx);
        let mut chain = ProcessorChainNode::new(
            vec![Box::new(TimedProcessor { cd: sub_cd })],
            chain_cd,
            false,
        );

        let (sender, _rx) = mpsc::Channel::new(100);
        let port: PortName = "default".into();
        let mut senders: HashMap<PortName, Sender<String>> = HashMap::new();
        let _ = senders.insert(port.clone(), Sender::new_local_mpsc_sender(sender));
        let (_metrics_rx, reporter) = test_reporter();
        let mut eh = EffectHandler::new(test_node_id("chain"), senders, Some(port), reporter);
        eh.core.set_node_interests(Interests::PROCESS_DURATION);

        chain
            .process(Message::PData("hello".into()), &mut eh)
            .await
            .unwrap();

        // Chain should NOT have recorded into the sub-processor's ComputeDuration
        // (that's the sub-processor's own responsibility via timed()).
        let sub_cd = chain.sub_processors[0]
            .compute_duration()
            .expect("sub-processor exposes compute_duration");
        let snap = sub_cd.snapshot_success().get();
        assert_eq!(
            snap.count, 0,
            "chain should not externally time sub-processors"
        );

        // Composite duration should still be recorded.
        let composite_snap = chain.composite_duration.snapshot_success().get();
        assert_eq!(
            composite_snap.count, 1,
            "composite duration should be recorded"
        );
    }
}
