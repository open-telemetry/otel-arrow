//! A simple batching processor for OTLP (OpenTelemetry Protocol) data.
//!
//! This module provides a basic batching implementation that collects OTLP data (traces, metrics, and logs)
//! and flushes them based on either:
//! - Batch size: When the number of items reaches a configured threshold
//! - Timeout: When a specified duration has passed since the last flush
//!
//! # Configuration
//! The processor is configured using [`BatchConfig`] which allows setting:
//! - `send_batch_size`: Maximum number of items to batch before flushing
//! - `timeout` : Maximum duration to wait before flushing an incomplete batch
//!
//! # Features
//! - Handles all three OTLP data types (traces, metrics, logs) independently
//! - Thread-safe operation through message passing
//! - Configurable batching behavior
//! - Simple and efficient implementation suitable for basic use cases

use std::time::{Duration, Instant};
use async_trait::async_trait;

// Your protobuf types
use crate::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
use crate::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
use crate::proto::opentelemetry::collector::trace::v1::ExportTraceServiceRequest;

use otap_df_engine::local::processor::Processor;
use otap_df_engine::message::{Message, ControlMsg};
use otap_df_engine::local::processor::EffectHandler;
use otap_df_engine::error::Error;

// The data wrapper you use for message passing
#[derive(Clone, Debug)]
enum OTLPData {
    Traces(ExportTraceServiceRequest),
    Metrics(ExportMetricsServiceRequest),
    Logs(ExportLogsServiceRequest),
}

// Simple configuration
#[derive(Debug, Clone)]
pub struct BatchConfig {
    pub send_batch_size: usize,
    pub timeout: Duration,
}

// The processor implementation
pub struct SimpleBatchProcessor {
    traces: Vec<ExportTraceServiceRequest>,
    metrics: Vec<ExportMetricsServiceRequest>,
    logs: Vec<ExportLogsServiceRequest>,
    last_update_traces: Instant,
    last_update_metrics: Instant,
    last_update_logs: Instant,
    config: BatchConfig,
}

impl SimpleBatchProcessor {
    pub fn new(config: BatchConfig) -> Self {
        let now = Instant::now();
        Self {
            traces: Vec::new(),
            metrics: Vec::new(),
            logs: Vec::new(),
            last_update_traces: now,
            last_update_metrics: now,
            last_update_logs: now,
            config,
        }
    }

    fn combine_traces(traces: &[ExportTraceServiceRequest]) -> Option<ExportTraceServiceRequest> {
        if traces.is_empty() { return None; }
        let mut combined = ExportTraceServiceRequest::default();
        for req in traces {
            combined.resource_spans.extend(req.resource_spans.clone());
        }
        Some(combined)
    }
    fn combine_metrics(metrics: &[ExportMetricsServiceRequest]) -> Option<ExportMetricsServiceRequest> {
        if metrics.is_empty() { return None; }
        let mut combined = ExportMetricsServiceRequest::default();
        for req in metrics {
            combined.resource_metrics.extend(req.resource_metrics.clone());
        }
        Some(combined)
    }
    fn combine_logs(logs: &[ExportLogsServiceRequest]) -> Option<ExportLogsServiceRequest> {
        if logs.is_empty() { return None; }
        let mut combined = ExportLogsServiceRequest::default();
        for req in logs {
            combined.resource_logs.extend(req.resource_logs.clone());
        }
        Some(combined)
    }
    async fn flush_traces(&mut self, effect_handler: &mut EffectHandler<OTLPData>) -> Result<(), Error<OTLPData>> {
        if let Some(batch) = Self::combine_traces(&self.traces) {
            effect_handler.send_message(OTLPData::Traces(batch)).await?;
            self.traces.clear();
        }
        self.last_update_traces = Instant::now();
        Ok(())
    }
    async fn flush_metrics(&mut self, effect_handler: &mut EffectHandler<OTLPData>) -> Result<(), Error<OTLPData>> {
        if let Some(batch) = Self::combine_metrics(&self.metrics) {
            effect_handler.send_message(OTLPData::Metrics(batch)).await?;
            self.metrics.clear();
        }
        self.last_update_metrics = Instant::now();
        Ok(())
    }
    async fn flush_logs(&mut self, effect_handler: &mut EffectHandler<OTLPData>) -> Result<(), Error<OTLPData>> {
        if let Some(batch) = Self::combine_logs(&self.logs) {
            effect_handler.send_message(OTLPData::Logs(batch)).await?;
            self.logs.clear();
        }
        self.last_update_logs = Instant::now();
        Ok(())
    }
    async fn flush_on_timeout(&mut self, effect_handler: &mut EffectHandler<OTLPData>) -> Result<(), Error<OTLPData>> {
        let now = Instant::now();
        let timeout = self.config.timeout;
        if timeout != Duration::from_secs(0) {
            if !self.traces.is_empty() && now.duration_since(self.last_update_traces) >= timeout {
                self.flush_traces(effect_handler).await?;
            }
            if !self.metrics.is_empty() && now.duration_since(self.last_update_metrics) >= timeout {
                self.flush_metrics(effect_handler).await?;
            }
            if !self.logs.is_empty() && now.duration_since(self.last_update_logs) >= timeout {
                self.flush_logs(effect_handler).await?;
            }
        }
        Ok(())
    }
    async fn flush_all(&mut self, effect_handler: &mut EffectHandler<OTLPData>) -> Result<(), Error<OTLPData>> {
        self.flush_traces(effect_handler).await?;
        self.flush_metrics(effect_handler).await?;
        self.flush_logs(effect_handler).await?;
        Ok(())
    }
}

#[async_trait(?Send)]
impl Processor<OTLPData> for SimpleBatchProcessor {
    async fn process(
        &mut self,
        msg: Message<OTLPData>,
        effect_handler: &mut EffectHandler<OTLPData>,
    ) -> Result<(), Error<OTLPData>> {
        match msg {
            Message::PData(data) => {
                match data {
                    OTLPData::Traces(req) => {
                        self.traces.push(req);
                        self.last_update_traces = Instant::now();
                        if self.traces.len() >= self.config.send_batch_size {
                            self.flush_traces(effect_handler).await?;
                        }
                    }
                    OTLPData::Metrics(req) => {
                        self.metrics.push(req);
                        self.last_update_metrics = Instant::now();
                        if self.metrics.len() >= self.config.send_batch_size {
                            self.flush_metrics(effect_handler).await?;
                        }
                    }
                    OTLPData::Logs(req) => {
                        self.logs.push(req);
                        self.last_update_logs = Instant::now();
                        if self.logs.len() >= self.config.send_batch_size {
                            self.flush_logs(effect_handler).await?;
                        }
                    }
                }
                Ok(())
            }
            Message::Control(ctrl_msg) => {
                match ctrl_msg {
                    ControlMsg::TimerTick { .. } => {
                        self.flush_on_timeout(effect_handler).await?;
                    }
                    ControlMsg::Shutdown { .. } => {
                        self.flush_all(effect_handler).await?;
                    }
                    _ => {}
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_engine::testing::processor::TestRuntime;
    use otap_df_engine::local::processor::Processor;
    use otap_df_engine::processor::ProcessorWrapper;
    use otap_df_engine::local::processor::EffectHandler;
    use crate::simple_batch_processor::{SimpleBatchProcessor, OTLPData, BatchConfig};
    use crate::proto::opentelemetry::collector::trace::v1::ExportTraceServiceRequest;
    use crate::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
    use crate::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
    use std::time::Duration;
    use otap_df_channel::mpsc;
    use otap_df_engine::message::{Message, ControlMsg,Sender, Receiver};

    // -- Helper constructors for test data --
    fn trace_req(_span_name: &str) -> ExportTraceServiceRequest {
        ExportTraceServiceRequest::default()
    }
    fn metric_req(_metric_name: &str) -> ExportMetricsServiceRequest {
        ExportMetricsServiceRequest::default()
    }
    fn logs_req(_log_name: &str) -> ExportLogsServiceRequest {
        ExportLogsServiceRequest::default()
    }

    // -- Helper to wrap the processor using tokio mpsc --
    fn wrap_local<P>(processor: P) -> ProcessorWrapper<OTLPData>
    where
        P: otap_df_engine::local::processor::Processor<OTLPData> + 'static
    {   
        // Update the channel creation to use the correct API
        let (control_sender, control_receiver) = mpsc::Channel::new(32);
        let (pdata_sender, pdata_receiver) = mpsc::Channel::new(32);
    
        ProcessorWrapper::Local {
            processor: Box::new(processor),
            effect_handler: EffectHandler::new(
                "simple_batch_processor".into(),
                Sender::Local(pdata_sender),
            ),
            control_sender: Sender::Local(control_sender),
            control_receiver: Receiver::Local(control_receiver),
            pdata_receiver: Some(Receiver::Local(pdata_receiver)),
        }
    }

    #[test]
    fn batch_traces_flush_on_size() {
        let mut runtime = TestRuntime::<OTLPData>::new();
        let config = BatchConfig { send_batch_size: 2, timeout: Duration::from_secs(60) };
        let wrapper = wrap_local(SimpleBatchProcessor::new(config));

        runtime.set_processor(wrapper)
            .run_test(|mut ctx| async move {
                ctx.process(Message::PData(OTLPData::Traces(trace_req("foo1")))).await.unwrap();
                ctx.process(Message::PData(OTLPData::Traces(trace_req("foo2")))).await.unwrap();
                let emitted = ctx.drain_pdata().await;
                assert_eq!(emitted.len(), 1, "Should emit 1 trace batch on size");
                match &emitted[0] {
                    OTLPData::Traces(_) => {},
                    _ => panic!("Expected traces batch"),
                }
            })
            .validate(|_vctx| async {});
    }

    #[test]
    fn batch_traces_flush_on_timeout() {
        let mut runtime = TestRuntime::<OTLPData>::new();
        let config = BatchConfig { send_batch_size: 10, timeout: Duration::from_millis(50) };
        let wrapper = wrap_local(SimpleBatchProcessor::new(config));

        runtime.set_processor(wrapper)
            .run_test(|mut ctx| async move {
                ctx.process(Message::PData(OTLPData::Traces(trace_req("foo1")))).await.unwrap();
                ctx.sleep(Duration::from_millis(55)).await;
                ctx.process(Message::Control(ControlMsg::TimerTick {})).await.unwrap();
                let emitted = ctx.drain_pdata().await;
                assert_eq!(emitted.len(), 1, "Should emit 1 trace batch on timeout");
                match &emitted[0] {
                    OTLPData::Traces(_) => {},
                    _ => panic!("Expected traces batch"),
                }
            })
            .validate(|_vctx| async {});
    }

    #[test]
    fn batch_metrics_flush_on_size() {
        let mut runtime = TestRuntime::<OTLPData>::new();
        let config = BatchConfig { send_batch_size: 3, timeout: Duration::from_secs(60) };
        let wrapper = wrap_local(SimpleBatchProcessor::new(config));

        runtime.set_processor(wrapper)
            .run_test(|mut ctx| async move {
                ctx.process(Message::PData(OTLPData::Metrics(metric_req("bar1")))).await.unwrap();
                ctx.process(Message::PData(OTLPData::Metrics(metric_req("bar2")))).await.unwrap();
                ctx.process(Message::PData(OTLPData::Metrics(metric_req("bar3")))).await.unwrap();
                let emitted = ctx.drain_pdata().await;
                assert_eq!(emitted.len(), 1, "Should emit 1 metrics batch on size");
                match &emitted[0] {
                    OTLPData::Metrics(_) => {},
                    _ => panic!("Expected metrics batch"),
                }
            })
            .validate(|_vctx| async {});
    }

    #[test]
    fn batch_metrics_flush_on_timeout() {
        let mut runtime = TestRuntime::<OTLPData>::new();
        let config = BatchConfig { send_batch_size: 10, timeout: Duration::from_millis(50) };
        let wrapper = wrap_local(SimpleBatchProcessor::new(config));

        runtime.set_processor(wrapper)
            .run_test(|mut ctx| async move {
                ctx.process(Message::PData(OTLPData::Metrics(metric_req("bar1")))).await.unwrap();
                ctx.sleep(Duration::from_millis(55)).await;
                ctx.process(Message::Control(ControlMsg::TimerTick {})).await.unwrap();
                let emitted = ctx.drain_pdata().await;
                assert_eq!(emitted.len(), 1, "Should emit 1 metrics batch on timeout");
                match &emitted[0] {
                    OTLPData::Metrics(_) => {},
                    _ => panic!("Expected metrics batch"),
                }
            })
            .validate(|_vctx| async {});
    }

    #[test]
    fn batch_logs_flush_on_size() {
        let mut runtime = TestRuntime::<OTLPData>::new();
        let config = BatchConfig { send_batch_size: 2, timeout: Duration::from_secs(60) };
        let wrapper = wrap_local(SimpleBatchProcessor::new(config));

        runtime.set_processor(wrapper)
            .run_test(|mut ctx| async move {
                ctx.process(Message::PData(OTLPData::Logs(logs_req("baz1")))).await.unwrap();
                ctx.process(Message::PData(OTLPData::Logs(logs_req("baz2")))).await.unwrap();
                let emitted = ctx.drain_pdata().await;
                assert_eq!(emitted.len(), 1, "Should emit 1 logs batch on size");
                match &emitted[0] {
                    OTLPData::Logs(_) => {},
                    _ => panic!("Expected logs batch"),
                }
            })
            .validate(|_vctx| async {});
    }

    #[test]
    fn batch_logs_flush_on_timeout() {
        let mut runtime = TestRuntime::<OTLPData>::new();
        let config = BatchConfig { send_batch_size: 10, timeout: Duration::from_millis(50) };
        let wrapper = wrap_local(SimpleBatchProcessor::new(config));

        runtime.set_processor(wrapper)
            .run_test(|mut ctx| async move {
                ctx.process(Message::PData(OTLPData::Logs(logs_req("baz1")))).await.unwrap();
                ctx.sleep(Duration::from_millis(55)).await;
                ctx.process(Message::Control(ControlMsg::TimerTick {})).await.unwrap();
                let emitted = ctx.drain_pdata().await;
                assert_eq!(emitted.len(), 1, "Should emit 1 logs batch on timeout");
                match &emitted[0] {
                    OTLPData::Logs(_) => {},
                    _ => panic!("Expected logs batch"),
                }
            })
            .validate(|_vctx| async {});
    }
}
