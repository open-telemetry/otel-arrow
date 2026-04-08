// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Processor chain: a composite node that runs multiple sub-processors
//! sequentially within a single task, eliminating inter-processor channels.
//!
//! The chain times the entire sequence with a composite [`ComputeDuration`]
//! so that callers can observe the total compute cost of a record batch
//! passing through all sub-processors.  Each sub-processor can still report
//! its own individual `compute.success.duration` if it uses `ComputeDuration`
//! internally.

use crate::Interests;
use crate::control::NodeControlMsg;
use crate::effect_handler::EffectHandlerCore;
use crate::error::Error;
use crate::local::processor::{EffectHandler, Processor};
use crate::message::{Message, Sender};
use crate::node::NodeId;
use crate::output_router::OutputRouter;
use crate::process_duration::ComputeDuration;
use async_trait::async_trait;
use otap_df_config::PortName;
use otap_df_telemetry::reporter::MetricsReporter;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// A processor that chains multiple sub-processors sequentially.
///
/// When processing data, the chain:
/// 1. Feeds the input to the first sub-processor via a buffer effect handler
/// 2. Collects the first sub-processor's outputs
/// 3. Feeds each output to the next sub-processor, collecting its outputs
/// 4. Repeats until the last sub-processor, which uses the real effect handler
///
/// The composite [`ComputeDuration`] measures the wall-clock time of the
/// entire chain, giving a single `compute.success.duration` metric that
/// represents the total processing cost per batch.
///
/// # Buffer strategy
///
/// Three techniques minimize per-batch overhead:
///
/// **Single-item fast path**: For chains of length 1, the chain delegates
/// directly to the sole sub-processor with the real `EffectHandler` —
/// zero staging overhead. For chains of length ≥ 2, an optimistic fast
/// path threads a single `PData` value through each intermediate stage
/// without any `Vec` operations, falling back to staging vecs only when
/// a stage produces 0 or 2+ outputs.
///
/// **Vec-backed buffer effect handlers** (`buffer_handlers`): Each
/// intermediate sub-processor needs somewhere to "send" its output.
/// Rather than using mpsc channels, we wire each one to a shared
/// `Rc<RefCell<Vec<PData>>>` via a `VecSender`. When the sub-processor
/// calls `send_message()`, the output is pushed directly into the `Vec`
/// with no channel send/recv, waker management, or async polling.
/// These are created once at construction and reused for every batch.
///
/// **Staging vecs** (`stage_a`, `stage_b`): Two `Vec<PData>` serve as
/// fallback buffers when a stage produces multiple outputs. Rather than
/// allocating a new vec per stage, `current` holds this stage's inputs
/// and `next` collects outputs; after each stage they swap roles via
/// `std::mem::swap`. These vecs retain their heap capacity across calls.
pub struct ProcessorChainNode<PData> {
    /// Sub-processors in execution order.
    sub_processors: Vec<SubProcessor<PData>>,
    /// Pre-allocated buffer effect handlers for intermediate (non-last)
    /// sub-processors. Length is `max(0, sub_processors.len() - 1)`.
    buffer_handlers: Vec<BufferSlot<PData>>,
    /// Composite compute duration covering the full chain.
    composite_duration: ComputeDuration,
    /// Reusable staging buffers to avoid per-batch Vec allocations.
    /// Swapped between `stage_a` and `stage_b` on each stage.
    stage_a: Vec<PData>,
    stage_b: Vec<PData>,
}

/// A sub-processor with its own processor implementation.
struct SubProcessor<PData> {
    processor: Box<dyn Processor<PData>>,
}

/// Pre-allocated buffer for an intermediate sub-processor.
///
/// Each intermediate sub-processor (all except the last) gets one of these.
/// The `effect_handler` is wired to a shared `Vec` via a `VecSender`.
/// After each `process()` call, outputs are collected directly from the
/// `buffer` — no channel overhead involved. The last sub-processor doesn't
/// need a buffer — it uses the chain's real `EffectHandler` to send directly
/// to the downstream channel.
struct BufferSlot<PData> {
    effect_handler: EffectHandler<PData>,
    buffer: Rc<RefCell<Vec<PData>>>,
}

impl<PData> ProcessorChainNode<PData> {
    /// Create a new processor chain from a list of sub-processors.
    ///
    /// `composite_duration` is registerd under the chain node's entity
    /// and will report the total compute time across all sub-processors.
    pub fn new(
        sub_processors: Vec<(Box<dyn Processor<PData>>, NodeId, MetricsReporter)>,
        composite_duration: ComputeDuration,
        buffer_capacity: usize,
    ) -> Self {
        // Pre-allocate buffer effect handlers for all intermediate
        // sub-processors (all except the last, which uses the real
        // effect handler).
        let intermediate_count = sub_processors.len().saturating_sub(1);
        let buffer_handlers: Vec<BufferSlot<PData>> = (0..intermediate_count)
            .map(|_| make_buffer_slot(buffer_capacity))
            .collect();

        Self {
            sub_processors: sub_processors
                .into_iter()
                .map(|(processor, _, _)| SubProcessor { processor })
                .collect(),
            buffer_handlers,
            composite_duration,
            stage_a: Vec::new(),
            stage_b: Vec::new(),
        }
    }
}

/// Create a buffer slot with a Vec-backed effect handler.
///
/// The `EffectHandler` is wired to an `Rc<RefCell<Vec<PData>>>` via a
/// `VecSender`. Outputs are pushed directly into the `Vec` — no channel
/// send/recv overhead.
fn make_buffer_slot<PData>(buffer_capacity: usize) -> BufferSlot<PData> {
    let buffer: Rc<RefCell<Vec<PData>>> =
        Rc::new(RefCell::new(Vec::with_capacity(buffer_capacity)));
    let default_port: PortName = "default".into();

    let mut pdata_senders: HashMap<PortName, Sender<PData>> = HashMap::new();
    let _ = pdata_senders.insert(
        default_port.clone(),
        Sender::new_local_vec_sender(buffer.clone()),
    );

    let node_id = NodeId {
        index: 0,
        name: "".into(),
    };
    let (_rx, reporter) = MetricsReporter::create_new_and_receiver(1);
    let effect_handler = EffectHandler {
        core: EffectHandlerCore::new(node_id.clone(), reporter),
        router: OutputRouter::new(node_id, pdata_senders, Some(default_port)),
    };

    BufferSlot {
        effect_handler,
        buffer,
    }
}

#[async_trait(?Send)]
impl<PData: Clone + 'static> Processor<PData> for ProcessorChainNode<PData> {
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

                        for (i, sub) in self.sub_processors.iter_mut().enumerate() {
                            // Forward the real metrics reporter so sub-processor
                            // snapshots reach the telemetry registry.
                            let sub_msg = Message::Control(NodeControlMsg::CollectTelemetry {
                                metrics_reporter: metrics_reporter.clone(),
                            });
                            // Use pre-allocated buffer for intermediate, real
                            // handler is irrelevant for control messages.
                            if let Some(slot) = self.buffer_handlers.get_mut(i) {
                                sub.processor
                                    .process(sub_msg, &mut slot.effect_handler)
                                    .await?;
                            } else {
                                // Last sub-processor — use the real handler.
                                sub.processor.process(sub_msg, effect_handler).await?;
                            }
                        }
                    }
                    other => {
                        for (i, sub) in self.sub_processors.iter_mut().enumerate() {
                            if let Some(slot) = self.buffer_handlers.get_mut(i) {
                                sub.processor
                                    .process(
                                        Message::Control(other.clone()),
                                        &mut slot.effect_handler,
                                    )
                                    .await?;
                            } else {
                                sub.processor
                                    .process(Message::Control(other.clone()), effect_handler)
                                    .await?;
                            }
                        }
                    }
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

                let interests = effect_handler.node_interests();

                let timer = if interests.contains(Interests::PROCESS_DURATION) {
                    Some(otap_df_telemetry::instrument::Timer::start())
                } else {
                    None
                };

                // Fast-path: single sub-processor — delegate directly, skip
                // all staging-buffer bookkeeping.
                if num_subs == 1 {
                    let last = self.sub_processors.first_mut().expect("checked non-empty");
                    let result = last
                        .processor
                        .process(Message::PData(pdata), effect_handler)
                        .await;

                    if let Some(timer) = timer {
                        self.composite_duration
                            .record(timer.elapsed_nanos(), result.is_ok());
                    }
                    return result;
                }

                // Multi-sub path: optimistic single-item fast path.
                //
                // Most processors are 1:1 (one output per input). When that
                // holds for every intermediate stage we can thread a single
                // `PData` value through without touching Vec/staging buffers
                // at all. If any stage produces 0 or 2+ outputs, we fall
                // back to the general staging-vec path for the remaining
                // stages.
                let result: Result<(), Error> = async {
                    let intermediates = &mut self.sub_processors[..num_subs - 1];
                    let mut single = Some(pdata);
                    let mut fallback_stage = None;

                    for (i, (sub, slot)) in intermediates
                        .iter_mut()
                        .zip(self.buffer_handlers.iter_mut())
                        .enumerate()
                    {
                        slot.effect_handler.core.set_node_interests(interests);
                        let data = single
                            .take()
                            .expect("single must be Some on each iteration");
                        sub.processor
                            .process(Message::PData(data), &mut slot.effect_handler)
                            .await?;

                        let mut buf = slot.buffer.borrow_mut();
                        match buf.len() {
                            1 => {
                                // Common 1:1 case — take the single output directly.
                                single = Some(buf.pop().expect("len checked"));
                            }
                            0 => return Ok(()),
                            _ => {
                                // Multi-output: fall back to staging vecs for
                                // the remaining stages.
                                fallback_stage = Some(i + 1);
                                let current = &mut self.stage_a;
                                current.clear();
                                current.append(&mut buf);
                                break;
                            }
                        }
                    }

                    if let Some(start) = fallback_stage {
                        // Continue with staging vecs from where the fast path
                        // left off.
                        let next = &mut self.stage_b;
                        let current = &mut self.stage_a;

                        for (sub, slot) in self.sub_processors[start..num_subs - 1]
                            .iter_mut()
                            .zip(self.buffer_handlers[start..].iter_mut())
                        {
                            slot.effect_handler.core.set_node_interests(interests);
                            next.clear();

                            for data in current.drain(..) {
                                sub.processor
                                    .process(Message::PData(data), &mut slot.effect_handler)
                                    .await?;
                            }
                            next.append(&mut slot.buffer.borrow_mut());

                            std::mem::swap(current, next);
                            if current.is_empty() {
                                return Ok(());
                            }
                        }

                        // Last sub-processor with staging vecs.
                        let last = self.sub_processors.last_mut().expect("checked non-empty");
                        for data in current.drain(..) {
                            last.processor
                                .process(Message::PData(data), effect_handler)
                                .await?;
                        }
                    } else {
                        // Fast path completed — process last sub-processor
                        // directly with the single item.
                        let last = self.sub_processors.last_mut().expect("checked non-empty");
                        let data = single.take().expect("single must be Some after fast path");
                        last.processor
                            .process(Message::PData(data), effect_handler)
                            .await?;
                    }

                    Ok(())
                }
                .await;

                // Record composite duration split by outcome.
                if let Some(timer) = timer {
                    self.composite_duration
                        .record(timer.elapsed_nanos(), result.is_ok());
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
    use crate::local::processor::{EffectHandler, Processor};
    use crate::message::{Message, Sender};
    use crate::node::NodeId;
    use crate::process_duration::ComputeDuration;
    use crate::testing::test_pipeline_ctx;
    use async_trait::async_trait;
    use otap_df_channel::mpsc;
    use otap_df_telemetry::reporter::MetricsReporter;
    use std::collections::HashMap;
    use std::rc::Rc;
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

    fn make_sub(
        processor: Box<dyn Processor<String>>,
        name: &str,
    ) -> (Box<dyn Processor<String>>, NodeId, MetricsReporter) {
        let (_rx, reporter) = test_reporter();
        (processor, test_node_id(name), reporter)
    }

    fn make_chain(
        subs: Vec<(Box<dyn Processor<String>>, NodeId, MetricsReporter)>,
    ) -> ProcessorChainNode<String> {
        let (ctx, _) = test_pipeline_ctx();
        let cd = ComputeDuration::new(&ctx);
        ProcessorChainNode::new(subs, cd, 128)
    }

    // Mock Processors

    /// A configurable processor that appends a suffix to data.
    /// Also tracks call count and elapsed compute time for assertions.
    /// With an empty suffix this acts as a passthrough.
    struct SuffixProcessor {
        suffix: String,
        call_count: Arc<AtomicUsize>,
        elapsed_ns: Rc<std::cell::Cell<f64>>,
    }

    impl SuffixProcessor {
        fn new(suffix: &str) -> Self {
            Self {
                suffix: suffix.into(),
                call_count: Arc::new(AtomicUsize::new(0)),
                elapsed_ns: Rc::new(std::cell::Cell::new(0.0)),
            }
        }
    }

    #[async_trait(?Send)]
    impl Processor<String> for SuffixProcessor {
        async fn process(
            &mut self,
            msg: Message<String>,
            effect_handler: &mut EffectHandler<String>,
        ) -> Result<(), Error> {
            if let Message::PData(data) = msg {
                let timer = otap_df_telemetry::instrument::Timer::start();
                let result = format!("{}{}", data, self.suffix);
                let elapsed = timer.elapsed_nanos();
                self.elapsed_ns.set(self.elapsed_ns.get() + elapsed);
                let _ = self.call_count.fetch_add(1, Ordering::Relaxed);
                effect_handler
                    .send_message(result)
                    .await
                    .map_err(Error::from)?;
            }
            Ok(())
        }
    }

    /// A processor that drops all data (simulates a filter that rejects everything).
    struct DropAllProcessor;

    #[async_trait(?Send)]
    impl Processor<String> for DropAllProcessor {
        async fn process(
            &mut self,
            _msg: Message<String>,
            _effect_handler: &mut EffectHandler<String>,
        ) -> Result<(), Error> {
            Ok(())
        }
    }

    /// A processor that emits two outputs per input.
    struct DuplicateProcessor;

    #[async_trait(?Send)]
    impl Processor<String> for DuplicateProcessor {
        async fn process(
            &mut self,
            msg: Message<String>,
            effect_handler: &mut EffectHandler<String>,
        ) -> Result<(), Error> {
            if let Message::PData(data) = msg {
                effect_handler
                    .send_message(format!("{}_a", data))
                    .await
                    .map_err(Error::from)?;
                effect_handler
                    .send_message(format!("{}_b", data))
                    .await
                    .map_err(Error::from)?;
            }
            Ok(())
        }
    }

    /// A processor that tracks control messages it receives.
    struct ControlTracker {
        shutdown_count: Arc<AtomicUsize>,
        timer_tick_count: Arc<AtomicUsize>,
    }

    impl ControlTracker {
        fn new() -> (Self, Arc<AtomicUsize>, Arc<AtomicUsize>) {
            let shutdown = Arc::new(AtomicUsize::new(0));
            let timer = Arc::new(AtomicUsize::new(0));
            (
                Self {
                    shutdown_count: shutdown.clone(),
                    timer_tick_count: timer.clone(),
                },
                shutdown,
                timer,
            )
        }
    }

    #[async_trait(?Send)]
    impl Processor<String> for ControlTracker {
        async fn process(
            &mut self,
            msg: Message<String>,
            _effect_handler: &mut EffectHandler<String>,
        ) -> Result<(), Error> {
            if let Message::Control(ctrl) = msg {
                match ctrl {
                    NodeControlMsg::Shutdown { .. } => {
                        let _ = self.shutdown_count.fetch_add(1, Ordering::Relaxed);
                    }
                    NodeControlMsg::TimerTick { .. } => {
                        let _ = self.timer_tick_count.fetch_add(1, Ordering::Relaxed);
                    }
                    _ => {}
                }
            }
            Ok(())
        }
    }

    /// A processor that always returns an error.
    struct ErrorProcessor;

    #[async_trait(?Send)]
    impl Processor<String> for ErrorProcessor {
        async fn process(
            &mut self,
            _msg: Message<String>,
            _effect_handler: &mut EffectHandler<String>,
        ) -> Result<(), Error> {
            Err(Error::ProcessorError {
                processor: test_node_id("error"),
                kind: crate::error::ProcessorErrorKind::Other,
                error: "synthetic error".into(),
                source_detail: String::new(),
            })
        }
    }

    // Tests

    /// Single sub-processor: data passes through and arrives at the output.
    #[tokio::test]
    async fn single_processor_forwards_data() {
        let proc = SuffixProcessor::new("");
        let count = proc.call_count.clone();
        let mut chain = make_chain(vec![make_sub(Box::new(proc), "p0")]);
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
            make_sub(Box::new(SuffixProcessor::new("_A")), "s1"),
            make_sub(Box::new(SuffixProcessor::new("_B")), "s2"),
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
            make_sub(Box::new(SuffixProcessor::new("_1")), "s1"),
            make_sub(Box::new(SuffixProcessor::new("_2")), "s2"),
            make_sub(Box::new(SuffixProcessor::new("_3")), "s3"),
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
            make_sub(Box::new(SuffixProcessor::new("_head")), "head"),
            make_sub(Box::new(DropAllProcessor), "drop"),
            make_sub(Box::new(tail), "tail"),
        ]);
        let (mut eh, rx) = test_effect_handler();

        chain
            .process(Message::PData("hello".into()), &mut eh)
            .await
            .unwrap();

        assert!(rx.try_recv().is_err(), "no data should reach downstream");
        assert_eq!(count.load(Ordering::Relaxed), 0, "tail never called");
    }

    /// A sub-processor that emits multiple outputs per input propagates all of them downstream.
    #[tokio::test]
    async fn multi_emit_inside_chain() {
        let mut chain = make_chain(vec![
            make_sub(Box::new(DuplicateProcessor), "dup"),
            make_sub(Box::new(SuffixProcessor::new("_X")), "sfx"),
        ]);
        let (mut eh, rx) = test_effect_handler();

        chain
            .process(Message::PData("hi".into()), &mut eh)
            .await
            .unwrap();

        let mut results = Vec::new();
        while let Ok(item) = rx.try_recv() {
            results.push(item);
        }
        results.sort();
        assert_eq!(results, vec!["hi_a_X", "hi_b_X"]);
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

    /// Control messages are forwarded to all sub-processors.
    #[tokio::test]
    async fn control_messages_forwarded_to_all() {
        let (tracker1, shutdown1, timer1) = ControlTracker::new();
        let (tracker2, shutdown2, timer2) = ControlTracker::new();
        let mut chain = make_chain(vec![
            make_sub(Box::new(tracker1), "t1"),
            make_sub(Box::new(tracker2), "t2"),
        ]);
        let (mut eh, _rx) = test_effect_handler();

        chain
            .process(Message::Control(NodeControlMsg::TimerTick {}), &mut eh)
            .await
            .unwrap();

        assert_eq!(timer1.load(Ordering::Relaxed), 1);
        assert_eq!(timer2.load(Ordering::Relaxed), 1);

        chain
            .process(
                Message::Control(NodeControlMsg::Shutdown {
                    deadline: std::time::Instant::now(),
                    reason: "test".into(),
                }),
                &mut eh,
            )
            .await
            .unwrap();

        assert_eq!(shutdown1.load(Ordering::Relaxed), 1);
        assert_eq!(shutdown2.load(Ordering::Relaxed), 1);
    }

    /// Composite duration is recorded when PROCESS_DURATION interest is active.
    /// Individual sub-processor durations are also tracked, and the composite
    /// must be >= the sum of the individual durations.
    #[tokio::test]
    async fn composite_duration_recorded() {
        let sub1 = SuffixProcessor::new("_A");
        let sub1_elapsed = sub1.elapsed_ns.clone();
        let sub2 = SuffixProcessor::new("_B");
        let sub2_elapsed = sub2.elapsed_ns.clone();

        let (ctx, _) = test_pipeline_ctx();
        let cd = ComputeDuration::new(&ctx);
        let mut chain = ProcessorChainNode::new(
            vec![
                make_sub(Box::new(sub1), "s1"),
                make_sub(Box::new(sub2), "s2"),
            ],
            cd,
            128,
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

        // Each sub-processor should have recorded its own duration.
        let s1 = sub1_elapsed.get();
        let s2 = sub2_elapsed.get();
        assert!(s1 > 0.0, "sub-processor 1 duration should be > 0");
        assert!(s2 > 0.0, "sub-processor 2 duration should be > 0");

        // Composite duration should cover the full chain.
        let snap = chain.composite_duration.snapshot_success().get();
        assert_eq!(snap.count, 1, "one duration observation expected");
        assert!(
            snap.sum >= s1 + s2,
            "composite ({}) should be >= sum of sub-processor durations ({} + {} = {})",
            snap.sum,
            s1,
            s2,
            s1 + s2,
        );
    }

    /// Composite duration is NOT recorded when interest is disabled.
    #[tokio::test]
    async fn composite_duration_not_recorded_when_disabled() {
        let mut chain = make_chain(vec![make_sub(Box::new(SuffixProcessor::new("_X")), "s1")]);
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
                make_sub(Box::new(SuffixProcessor::new("_A")), "s1"),
                make_sub(Box::new(ErrorProcessor), "err"),
            ],
            cd,
            128,
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
}
