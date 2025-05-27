use std::collections::VecDeque;
use std::time::{Duration, Instant};
use async_trait::async_trait;
use otap_df_engine::local::processor::{Processor, EffectHandler};
use otap_df_engine::error::Error;
use otap_df_engine::message::{Message, ControlMsg};
use crate::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
use crate::proto::opentelemetry::logs::v1::{ResourceLogs, ScopeLogs, LogRecord};
use crate::proto::opentelemetry::collector::trace::v1::ExportTraceServiceRequest;
use crate::proto::opentelemetry::trace::v1::{ResourceSpans, ScopeSpans, Span};
use crate::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
use crate::proto::opentelemetry::metrics::v1::{ResourceMetrics, ScopeMetrics, Metric};

// The enum that wraps your OTLP requests
#[derive(Clone, Debug)]
pub enum OTLPData {
    Traces(ExportTraceServiceRequest),
    Metrics(ExportMetricsServiceRequest),
    Logs(ExportLogsServiceRequest),
}

// Generic batch config
#[derive(Debug, Clone)]
pub struct BatchConfig {
    pub send_batch_size: usize,
    pub timeout: Duration,
}

// --- Batchable trait ---
pub trait Batchable: Sized + Clone {
    type Item: Clone;

    fn extract_items(&self) -> Vec<Self::Item>;
    fn build_batch(items: Vec<Self::Item>) -> Self;
    fn is_empty(&self) -> bool;
}

// --- Trait impls for traces ---
impl Batchable for ExportTraceServiceRequest {
    type Item = Span;
    fn extract_items(&self) -> Vec<Span> {
        let mut out = Vec::new();
        for rs in &self.resource_spans {
            for ss in &rs.scope_spans {
                for s in &ss.spans {
                    out.push(s.clone());
                }
            }
        }
        out
    }
    fn build_batch(spans: Vec<Span>) -> Self {
        ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: None,
                scope_spans: vec![ScopeSpans {
                    scope: None,
                    spans,
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }]
        }
    }
    fn is_empty(&self) -> bool {
        self.resource_spans.iter().all(|rs| {
            rs.scope_spans.iter().all(|ss| ss.spans.is_empty())
        })
    }
}

// --- For metrics (flatten Metric) ---
impl Batchable for ExportMetricsServiceRequest {
    type Item = Metric;
    fn extract_items(&self) -> Vec<Metric> {
        let mut out = Vec::new();
        for rm in &self.resource_metrics {
            for sm in &rm.scope_metrics {
                for m in &sm.metrics {
                    out.push(m.clone());
                }
            }
        }
        out
    }
    fn build_batch(metrics: Vec<Metric>) -> Self {
        ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                resource: None,
                scope_metrics: vec![ScopeMetrics {
                    scope: None,
                    metrics,
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }]
        }
    }
    fn is_empty(&self) -> bool {
        self.resource_metrics.iter().all(|rm| {
            rm.scope_metrics.iter().all(|sm| sm.metrics.is_empty())
        })
    }
}

// --- For logs (flatten LogRecord) ---
impl Batchable for ExportLogsServiceRequest {
    type Item = LogRecord;
    fn extract_items(&self) -> Vec<LogRecord> {
        let mut out = Vec::new();
        for rl in &self.resource_logs {
            for sl in &rl.scope_logs {
                for r in &sl.log_records {
                    out.push(r.clone());
                }
            }
        }
        out
    }
    fn build_batch(logs: Vec<LogRecord>) -> Self {
        ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: logs,
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }]
        }
    }
    fn is_empty(&self) -> bool {
        self.resource_logs.iter().all(|rl| {
            rl.scope_logs.iter().all(|sl| sl.log_records.is_empty())
        })
    }
}

// --- Generic buffer + batcher for any OTLPRequest ---
#[derive(Clone)]
struct ChunkBuffer<T: Batchable> {
    pub buffer: VecDeque<T::Item>,
    pub batch_size: usize,
}

impl<T: Batchable> ChunkBuffer<T> {
    pub fn new(batch_size: usize) -> Self {
        Self {
            buffer: VecDeque::new(),
            batch_size,
        }
    }

    /// Add new items; return complete batches
    pub fn push_and_split(&mut self, req: T) -> Vec<T> {
        self.buffer.extend(req.extract_items());
        let mut out = Vec::new();
        while self.buffer.len() >= self.batch_size {
            let items: Vec<T::Item> =
                self.buffer.drain(..self.batch_size).collect();
            out.push(T::build_batch(items));
        }
        out
    }

    /// Drain any remaining items as single batch
    pub fn drain_partial(&mut self) -> Option<T> {
        if self.buffer.is_empty() {
            None
        } else {
            let items: Vec<T::Item> = self.buffer.drain(..).collect();
            Some(T::build_batch(items))
        }
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

#[derive(Clone)]
pub struct GenericBatcher {
    traces: ChunkBuffer<ExportTraceServiceRequest>,
    metrics: ChunkBuffer<ExportMetricsServiceRequest>,
    logs: ChunkBuffer<ExportLogsServiceRequest>,
    last_update_traces: Instant,
    last_update_metrics: Instant,
    last_update_logs: Instant,
    config: BatchConfig,
}

impl GenericBatcher {
    pub fn new(config: BatchConfig) -> Self {
        let now = Instant::now();
        Self {
            traces: ChunkBuffer::new(config.send_batch_size),
            metrics: ChunkBuffer::new(config.send_batch_size),
            logs: ChunkBuffer::new(config.send_batch_size),
            last_update_traces: now,
            last_update_metrics: now,
            last_update_logs: now,
            config,
        }
    }
    async fn flush_traces(&mut self, handler: &mut EffectHandler<OTLPData>) -> Result<(), Error<OTLPData>> {
        if let Some(batch) = self.traces.drain_partial() {
            handler.send_message(OTLPData::Traces(batch)).await?;
        }
        self.last_update_traces = Instant::now();
        Ok(())
    }
    async fn flush_metrics(&mut self, handler: &mut EffectHandler<OTLPData>) -> Result<(), Error<OTLPData>> {
        if let Some(batch) = self.metrics.drain_partial() {
            handler.send_message(OTLPData::Metrics(batch)).await?;
        }
        self.last_update_metrics = Instant::now();
        Ok(())
    }
    async fn flush_logs(&mut self, handler: &mut EffectHandler<OTLPData>) -> Result<(), Error<OTLPData>> {
        if let Some(batch) = self.logs.drain_partial() {
            handler.send_message(OTLPData::Logs(batch)).await?;
        }
        self.last_update_logs = Instant::now();
        Ok(())
    }
    async fn flush_on_timeout(&mut self, handler: &mut EffectHandler<OTLPData>) -> Result<(), Error<OTLPData>> {
        let now = Instant::now();
        let timeout = self.config.timeout;
        if timeout != Duration::from_secs(0) {
            if !self.traces.is_empty() && now.duration_since(self.last_update_traces) >= timeout {
                self.flush_traces(handler).await?;
            }
            if !self.metrics.is_empty() && now.duration_since(self.last_update_metrics) >= timeout {
                self.flush_metrics(handler).await?;
            }
            if !self.logs.is_empty() && now.duration_since(self.last_update_logs) >= timeout {
                self.flush_logs(handler).await?;
            }
        }
        Ok(())
    }
    async fn flush_all(&mut self, handler: &mut EffectHandler<OTLPData>) -> Result<(), Error<OTLPData>> {
        self.flush_traces(handler).await?;
        self.flush_metrics(handler).await?;
        self.flush_logs(handler).await?;
        Ok(())
    }
}

#[async_trait(?Send)]
impl Processor<OTLPData> for GenericBatcher {
    async fn process(
        &mut self,
        msg: Message<OTLPData>,
        effect_handler: &mut EffectHandler<OTLPData>
    ) -> Result<(), Error<OTLPData>> {
        match msg {
            Message::PData(data) => {
                match data {
                    OTLPData::Traces(req) => {
                        for batch in self.traces.push_and_split(req) {
                            effect_handler.send_message(OTLPData::Traces(batch)).await?;
                        }
                        self.last_update_traces = Instant::now();
                    }
                    OTLPData::Metrics(req) => {
                        for batch in self.metrics.push_and_split(req) {
                            effect_handler.send_message(OTLPData::Metrics(batch)).await?;
                        }
                        self.last_update_metrics = Instant::now();
                    }
                    OTLPData::Logs(req) => {
                        for batch in self.logs.push_and_split(req) {
                            effect_handler.send_message(OTLPData::Logs(batch)).await?;
                        }
                        self.last_update_logs = Instant::now();
                    }
                }
                Ok(())
            }
            Message::Control(ctrl_msg) => {
                match ctrl_msg {
                    ControlMsg::TimerTick { .. } => {
                        self.flush_on_timeout(effect_handler).await?;
                    }
                    ControlMsg::Shutdown{..} => {
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
    use otap_df_engine::local::processor::{Processor, EffectHandler};
    use otap_df_engine::message::{Message, ControlMsg};
    use otap_df_engine::config::ProcessorConfig;
    use crate::proto::opentelemetry::collector::trace::v1::ExportTraceServiceRequest;
    use crate::proto::opentelemetry::trace::v1::{ResourceSpans, ScopeSpans, Span};
    use crate::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
    use crate::proto::opentelemetry::metrics::v1::{ResourceMetrics, ScopeMetrics, Metric};
    use crate::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
    use crate::proto::opentelemetry::logs::v1::{ResourceLogs, ScopeLogs, LogRecord};
    use std::time::Duration;

    fn trace_batch_with_n_spans(n: usize) -> ExportTraceServiceRequest {
        ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: None,
                scope_spans: vec![ScopeSpans {
                    scope: None,
                    spans: (0..n).map(|i| Span {
                        name: format!("span{i}"),
                        ..Default::default()
                    }).collect(),
                    schema_url: "".to_string(),
                }],
                schema_url: "".to_string(),
            }]
        }
    }

    fn metrics_batch_with_n_metrics(n: usize) -> ExportMetricsServiceRequest {
        ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                resource: None,
                scope_metrics: vec![ScopeMetrics {
                    scope: None,
                    metrics: (0..n).map(|i| Metric {
                        name: format!("metric{i}"),
                        ..Default::default()
                    }).collect(),
                    schema_url: "".to_string(),
                }],
                schema_url: "".to_string(),
            }]
        }
    }

    fn logs_batch_with_n_logs(n: usize) -> ExportLogsServiceRequest {
        ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: (0..n).map(|i| LogRecord {
                        severity_text: format!("log{i}"),
                        ..Default::default()
                    }).collect(),
                    schema_url: "".to_string(),
                }],
                schema_url: "".to_string(),
            }]
        }
    }

    fn wrap_local<P>(processor: P) -> otap_df_engine::processor::ProcessorWrapper<OTLPData>
    where
        P: otap_df_engine::local::processor::Processor<OTLPData> + 'static {
        let config = ProcessorConfig::new("simple_generic_batch_processor_test");
        otap_df_engine::processor::ProcessorWrapper::local(processor, &config)
    }

    #[test]
    fn traces_single_batch_on_size() {
        let mut runtime = TestRuntime::<OTLPData>::new();
        let config = BatchConfig { send_batch_size: 4, timeout: Duration::from_secs(60) };
        let wrapper = wrap_local(GenericBatcher::new(config));
        runtime.set_processor(wrapper)
            .run_test(|mut ctx| async move {
                // send 4 spans: should emit 1 batch
                ctx.process(Message::PData(OTLPData::Traces(trace_batch_with_n_spans(4)))).await.unwrap();
                let emitted = ctx.drain_pdata().await;
                assert_eq!(emitted.len(), 1, "Should emit 1 batch");
                match &emitted[0] {
                    OTLPData::Traces(req) => {
                        let count = req.resource_spans.iter()
                            .flat_map(|rs| &rs.scope_spans)
                            .flat_map(|ss| &ss.spans)
                            .count();
                        assert_eq!(count, 4, "Batch has 4 spans");
                    }
                    _ => panic!("Wrong batch type"),
                }
            })
            .validate(|_ctx| async {});
    }

    #[test]
    fn traces_split_large_batch() {
        let mut runtime = TestRuntime::<OTLPData>::new();
        let config = BatchConfig { send_batch_size: 5, timeout: Duration::from_secs(60) };
        let wrapper = wrap_local(GenericBatcher::new(config));
        runtime.set_processor(wrapper)
            .run_test(|mut ctx| async move {
                // send 11 spans: should emit 2 batches of 5 and one of 1 (on shutdown/flush)
                ctx.process(Message::PData(OTLPData::Traces(trace_batch_with_n_spans(11)))).await.unwrap();
                ctx.process(Message::Control(ControlMsg::TimerTick {})).await.unwrap();
                // Also trigger shutdown to ensure all flushed
                ctx.process(Message::Control(ControlMsg::Shutdown { 
                    deadline: Duration::from_secs(1), 
                    reason: "test".to_string()
                })).await.unwrap();
                let emitted = ctx.drain_pdata().await;
                assert_eq!(emitted.len(), 3, "Should emit 3 batches");
                let mut total_spans = 0;
                for b in &emitted {
                    match b {
                        OTLPData::Traces(req) => {
                            total_spans += req.resource_spans.iter()
                                .flat_map(|rs| &rs.scope_spans)
                                .flat_map(|ss| &ss.spans)
                                .count()
                        }
                        _ => (),
                    }
                }
                assert_eq!(total_spans, 11, "Total spans == 11");
            })
            .validate(|_ctx| async {});
    }

    #[test]
    fn metrics_and_logs_batching() {
        let mut runtime = TestRuntime::<OTLPData>::new();
        let config = BatchConfig { send_batch_size: 3, timeout: Duration::from_secs(60) };
        let wrapper = wrap_local(GenericBatcher::new(config));
        runtime.set_processor(wrapper)
            .run_test(|mut ctx| async move {
                // metrics: 4, logs: 6
                ctx.process(Message::PData(OTLPData::Metrics(metrics_batch_with_n_metrics(4)))).await.unwrap();
                ctx.process(Message::PData(OTLPData::Logs(logs_batch_with_n_logs(6)))).await.unwrap();
                ctx.process(Message::Control(ControlMsg::Shutdown { deadline: Duration::from_secs(1), reason: "test".into() })).await.unwrap();
                let emitted = ctx.drain_pdata().await;

                // There should be: 2 metric batches (3 + 1) and 2 log batches (3 + 3)
                let metric_batches = emitted.iter().filter(|e| matches!(e, OTLPData::Metrics(_))).count();
                let log_batches = emitted.iter().filter(|e| matches!(e, OTLPData::Logs(_))).count();
                assert_eq!(metric_batches, 2, "Metrics split as 3+1");
                assert_eq!(log_batches, 2, "Logs split as 3+3");
            })
            .validate(|_ctx| async {});
    }

    #[test]
    fn flush_on_timeout() {
        let mut runtime = TestRuntime::<OTLPData>::new();
        let config = BatchConfig { send_batch_size: 4, timeout: Duration::from_millis(10) };
        let wrapper = wrap_local(GenericBatcher::new(config));
        runtime.set_processor(wrapper)
            .run_test(|mut ctx| async move {
                ctx.process(Message::PData(OTLPData::Traces(trace_batch_with_n_spans(2)))).await.unwrap();
                ctx.sleep(Duration::from_millis(15)).await;
                ctx.process(Message::Control(ControlMsg::TimerTick {})).await.unwrap();
                let emitted = ctx.drain_pdata().await;
                assert!(!emitted.is_empty(), "Should emit partial batch on timeout");
            })
            .validate(|_ctx| async {});
    }

    #[test]
    fn batch_multiple_types_flush_on_shutdown() {
        let mut runtime = TestRuntime::<OTLPData>::new();
        let config = BatchConfig { send_batch_size: 10, timeout: Duration::from_millis(50) };
        let wrapper = wrap_local(GenericBatcher::new(config));
        runtime.set_processor(wrapper)
            .run_test(|mut ctx| async move {
                // Send one of each type of data
                ctx.process(Message::PData(OTLPData::Logs(logs_batch_with_n_logs(2)))).await.unwrap();
                ctx.process(Message::PData(OTLPData::Metrics(metrics_batch_with_n_metrics(2)))).await.unwrap();
                ctx.process(Message::PData(OTLPData::Traces(trace_batch_with_n_spans(2)))).await.unwrap();
                // Trigger shutdown to flush all batches
                ctx.process(Message::Control(ControlMsg::Shutdown { 
                    deadline: Duration::from_millis(50), 
                    reason: "test".to_string() 
                })).await.unwrap();

                let emitted = ctx.drain_pdata().await;
                assert_eq!(emitted.len(), 3, "Should emit 3 batches on shutdown");

                let mut has_logs = false;
                let mut has_metrics = false;
                let mut has_traces = false;

                for item in &emitted {
                    match item {
                        OTLPData::Logs(_) => has_logs = true,
                        OTLPData::Metrics(_) => has_metrics = true,
                        OTLPData::Traces(_) => has_traces = true,
                    }
                }
                assert!(has_logs, "Missing logs batch in output");
                assert!(has_metrics, "Missing metrics batch in output");
                assert!(has_traces, "Missing traces batch in output");
            })
            .validate(|_ctx| async {});
    }

    #[test]
    fn metrics_batch_preserves_all_metric_kinds() {
        use crate::proto::opentelemetry::metrics::v1::{
            Metric, Gauge, Sum, Histogram, NumberDataPoint, HistogramDataPoint,
        };
        use crate::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
        use crate::proto::opentelemetry::metrics::v1::{ResourceMetrics, ScopeMetrics};
        use otap_df_engine::message::{Message, ControlMsg};
        use std::time::Duration;

        // Create a Gauge metric
        let gauge_metric = Metric {
            name: "gauge".to_string(),
            data: Some(crate::proto::opentelemetry::metrics::v1::metric::Data::Gauge(Gauge {
                data_points: vec![NumberDataPoint::default()],
            })),
        ..Default::default()
    };

    // Create a Sum metric
    let sum_metric = Metric {
        name: "sum".to_string(),
        data: Some(crate::proto::opentelemetry::metrics::v1::metric::Data::Sum(Sum {
            data_points: vec![NumberDataPoint::default()],
            aggregation_temporality: 1,
            is_monotonic: false,
        })),
        ..Default::default()
    };

    // Create a Histogram metric
    let histogram_metric = Metric {
        name: "histogram".to_string(),
        data: Some(crate::proto::opentelemetry::metrics::v1::metric::Data::Histogram(Histogram {
            data_points: vec![HistogramDataPoint::default()],
            aggregation_temporality: 1,
        })),
        ..Default::default()
    };

    // Build one batch with all three types
    let req = ExportMetricsServiceRequest {
        resource_metrics: vec![ResourceMetrics {
            resource: None,
            scope_metrics: vec![ScopeMetrics {
                scope: None,
                metrics: vec![
                    gauge_metric.clone(),
                    sum_metric.clone(),
                    histogram_metric.clone(),
                ],
                schema_url: String::new(),
            }],
            schema_url: String::new(),
        }],
    };

    // Set up processor; batch size larger than input, so all show up together
    let mut runtime = TestRuntime::<OTLPData>::new();
    let config = BatchConfig { send_batch_size: 10, timeout: Duration::from_secs(60) };
    let wrapper = wrap_local(GenericBatcher::new(config));

    runtime.set_processor(wrapper)
    .run_test(|mut ctx| async move {
        ctx.process(Message::PData(OTLPData::Metrics(req))).await.unwrap();
        ctx.process(Message::Control(ControlMsg::Shutdown { deadline: Duration::from_secs(1), reason: "test".into() })).await.unwrap();
        let emitted = ctx.drain_pdata().await;
        // Should emit one batch with all metrics
        assert_eq!(emitted.len(), 1, "Should emit a single metrics batch");
        match &emitted[0] {
            OTLPData::Metrics(batch) => {
                // Flatten all metrics in output
                let all_metrics: Vec<Metric> = batch.resource_metrics.iter()
                    .flat_map(|rm| &rm.scope_metrics)
                    .flat_map(|sm| &sm.metrics)
                    .cloned()
                    .collect();
                let names: Vec<_> = all_metrics.iter().map(|m| m.name.as_str()).collect();
                assert!(names.contains(&"gauge"),   "Gauge metric found in batch");
                assert!(names.contains(&"sum"),     "Sum metric found in batch");
                assert!(names.contains(&"histogram"), "Histogram metric found in batch");

                // Optionally: Check types preserved
                fn metric_kind(m: &Metric) -> &'static str {
                    match &m.data {
                        Some(crate::proto::opentelemetry::metrics::v1::metric::Data::Gauge(_)) => "gauge",
                        Some(crate::proto::opentelemetry::metrics::v1::metric::Data::Sum(_)) => "sum",
                        Some(crate::proto::opentelemetry::metrics::v1::metric::Data::Histogram(_)) => "histogram",
                        Some(_) => "other",
                        None => "none"
                    }
                }
                assert!(all_metrics.iter().any(|m| metric_kind(m) == "gauge"), "Gauge type found");
                assert!(all_metrics.iter().any(|m| metric_kind(m) == "sum"), "Sum type found");
                assert!(all_metrics.iter().any(|m| metric_kind(m) == "histogram"), "Histogram type found");
            }
            _ => panic!("Expected Metrics batch"),
        }
    })
    .validate(|_ctx| async {});
}
}