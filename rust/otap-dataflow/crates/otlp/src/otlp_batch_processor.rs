use crate::OTLPData;
use crate::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
use crate::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
use crate::proto::opentelemetry::collector::trace::v1::ExportTraceServiceRequest;
use crate::proto::opentelemetry::logs::v1::{ResourceLogs, ScopeLogs};
use crate::proto::opentelemetry::metrics::v1::{ResourceMetrics, ScopeMetrics};
use crate::proto::opentelemetry::trace::v1::{ResourceSpans, ScopeSpans};
use async_trait::async_trait;
use otap_df_engine::error::Error;
use otap_df_engine::local::processor::{EffectHandler, Processor};
use otap_df_engine::message::{ControlMsg, Message};
use std::time::{Duration, Instant};
use prost::Message as ProstMessage;

/// Trait for hierarchical batch splitting
///
/// This trait is used to split a batch into a vector of smaller batches, each with at most `max_batch_size`
/// leaf items, preserving all resource/scope/leaf (span/metric/logrecord) structure.
pub trait HierarchicalBatchSplit: Sized {
    fn split_into_batches(self, max_batch_size: usize) -> Vec<Self>;
}

/// TODO: Use the pdata/otlp support library, rewrite this function to be generic over PData as that library develops

impl HierarchicalBatchSplit for ExportTraceServiceRequest {
    fn split_into_batches(mut self, max_batch_size: usize) -> Vec<Self> {
        let mut batches = Vec::new();

        // Accumulates resource groups and their contents until the batch reaches max_batch_size leaf items.
        // When full, this batch is pushed to the output and a new one is started.
        let mut current_batch = ExportTraceServiceRequest {
            resource_spans: Vec::new(),
        };
        let mut current_span_count = 0;

        // ToDo: The current implementation is recreating the entire hierarchy of ResourceSpans, ScopeSpans, Spans but we should probably avoid most of those allocations.
        for mut rs in self.resource_spans.drain(..) {
            let mut res = ResourceSpans {
                resource: rs.resource.take(),
                scope_spans: Vec::new(),
                schema_url: rs.schema_url.clone(),
            };
            for mut ss in rs.scope_spans.drain(..) {
                let mut sc = ScopeSpans {
                    scope: ss.scope.take(),
                    spans: Vec::new(),
                    schema_url: ss.schema_url.clone(),
                };
                while !ss.spans.is_empty() {
                    // Number of items that can still be added to the current batch before reaching max_batch_size
                    let remaining = max_batch_size - current_span_count;
                    let take = remaining.min(ss.spans.len());
                    sc.spans.extend(ss.spans.drain(..take));
                    current_span_count += take;

                    if !sc.spans.is_empty() {
                        res.scope_spans.push(sc.clone());
                        sc.spans.clear();
                    }   

                    if current_span_count == max_batch_size {
                        if !res.scope_spans.is_empty() {
                            current_batch.resource_spans.push(res.clone());
                            // Clear res.scope_spans after pushing to prevent duplicating scope groups in subsequent batches.
                            // This ensures each batch only contains the intended scope groups for that batch.
                            res.scope_spans.clear();
                        }
                        // Push the full batch to output and start a new one.
                        batches.push(current_batch.clone());
                        current_batch = ExportTraceServiceRequest {
                            resource_spans: Vec::new(),
                        };
                        current_span_count = 0;
                    }
                }
            }
            if !res.scope_spans.is_empty() {
                current_batch.resource_spans.push(res.clone());
            }
        }
        if current_span_count > 0 && !current_batch.resource_spans.is_empty() {
            batches.push(current_batch);
        }
        batches
    }
}


impl HierarchicalBatchSplit for ExportMetricsServiceRequest {
    fn split_into_batches(mut self, max_batch_size: usize) -> Vec<Self> {
        let mut batches = Vec::new();

        // The batch currently being filled; pushed to output when max_batch_size is reached.
        let mut current_batch = ExportMetricsServiceRequest::default();
        let mut current_metric_count = 0;

        // ToDo: The current implementation is recreating the entire hierarchy of ResourceMetrics, ScopeMetrics, Metrics but we should probably avoid most of those allocations.
        for mut rm in self.resource_metrics.drain(..) {
            let mut res = ResourceMetrics {
                resource: rm.resource.take(),
                scope_metrics: Vec::new(),
                schema_url: rm.schema_url.clone(),
            };
            for mut sm in rm.scope_metrics.drain(..) {
                let mut sc = ScopeMetrics {
                    scope: sm.scope.take(),
                    metrics: Vec::new(),
                    schema_url: sm.schema_url.clone(),
                };
                while !sm.metrics.is_empty() {
                    // Number of items that can still be added to the current batch before reaching max_batch_size
                    let remaining = max_batch_size - current_metric_count;
                    let take = remaining.min(sm.metrics.len());

                    sc.metrics.extend(sm.metrics.drain(..take));
                    current_metric_count += take;

                    if !sc.metrics.is_empty() {
                        res.scope_metrics.push(sc.clone());
                        sc.metrics.clear();
                    }

                    if current_metric_count == max_batch_size {
                        if !res.scope_metrics.is_empty() {
                            current_batch.resource_metrics.push(res.clone());
                            res.scope_metrics.clear();
                        }
                        // Push the full batch to output and start a new one.
                        batches.push(std::mem::take(&mut current_batch));
                        current_batch = ExportMetricsServiceRequest::default();
                        current_metric_count = 0;
                    }
                }
            }
            if !res.scope_metrics.is_empty() {
                current_batch.resource_metrics.push(res.clone());
            }
        }
        if current_metric_count > 0 && !current_batch.resource_metrics.is_empty() {
            batches.push(current_batch);
        }
        batches
    }
}

impl ExportMetricsServiceRequest {
    /// Splits the batch into multiple batches, each containing at most `max_requests` top-level requests.
    ///
    /// This preserves the original structure of each request.
    pub fn split_by_requests(self, max_requests: usize) -> Vec<Self> {
        self.resource_metrics
            .chunks(max_requests)
            .map(|chunk| ExportMetricsServiceRequest {
                resource_metrics: chunk.to_vec(),
            })
            .collect()
    }
}

impl HierarchicalBatchSplit for ExportLogsServiceRequest {
    fn split_into_batches(mut self, max_batch_size: usize) -> Vec<Self> {
        let mut batches = Vec::new();

        // The batch currently being filled; pushed to output when max_batch_size is reached.
        let mut current_batch = ExportLogsServiceRequest {
            resource_logs: Vec::new(),
        };
        let mut current_log_count = 0;
        // ToDo: The current implementation is recreating the entire hierarchy of ResourceLogs, ScopeLogs, LogRecords but we should probably avoid most of those allocations.
        for mut rl in self.resource_logs.drain(..) {
            let mut res = ResourceLogs {
                resource: rl.resource.take(),
                scope_logs: Vec::new(),
                schema_url: rl.schema_url.clone(),
            };
            for mut sl in rl.scope_logs.drain(..) {
                let mut sc = ScopeLogs {
                    scope: sl.scope.take(),
                    log_records: Vec::new(),
                    schema_url: sl.schema_url.clone(),
                };
                while !sl.log_records.is_empty() {
                    // Number of items that can still be added to the current batch before reaching max_batch_size
                    let remaining = max_batch_size - current_log_count;
                    let take = remaining.min(sl.log_records.len());

                    sc.log_records.extend(sl.log_records.drain(..take));
                    current_log_count += take;

                    if !sc.log_records.is_empty() {
                        res.scope_logs.push(sc.clone());
                        sc.log_records.clear();
                    }

                    if current_log_count == max_batch_size {
                        if !res.scope_logs.is_empty() {
                            current_batch.resource_logs.push(res.clone());
                            res.scope_logs.clear();
                        }
                        // Push the full batch to output and start a new one.
                        batches.push(current_batch.clone());
                        current_batch = ExportLogsServiceRequest {
                            resource_logs: Vec::new(),
                        };
                        current_log_count = 0;
                    }
                }
            }
            if !res.scope_logs.is_empty() {
                current_batch.resource_logs.push(res.clone());
            }
        }
        if current_log_count > 0 && !current_batch.resource_logs.is_empty() {
            batches.push(current_batch);
        }
        batches
    }
}

impl ExportLogsServiceRequest {
    /// Splits the batch into multiple batches, each containing at most `max_requests` top-level requests.
    /// 
    /// This preserves the original structure of each request.
    pub fn split_by_requests(self, max_requests: usize) -> Vec<Self> {
        self.resource_logs
            .chunks(max_requests)
            .map(|chunk| ExportLogsServiceRequest {
                resource_logs: chunk.to_vec(),
            })
            .collect()
    }
}

impl ExportTraceServiceRequest {
    /// Splits the batch into multiple batches, each containing at most `max_requests` top-level requests.
    ///
    /// This preserves the original structure of each request.
    pub fn split_by_requests(self, max_requests: usize) -> Vec<Self> {
        self.resource_spans
            .chunks(max_requests)
            .map(|chunk| ExportTraceServiceRequest {
                resource_spans: chunk.to_vec(),
            })
            .collect()
    }
}

/// Defines the strategy for sizing batches in the batch processor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchSizer {
    /// Batch by the number of top-level requests (resource groups).
    Requests,
    /// Batch by the number of leaf items (spans, metrics, or log records).
    Items,
    /// Batch by the total serialized byte size.
    Bytes,
}

/// Configuration for the generic batch processor.
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// The strategy for sizing batches.
    pub sizer: BatchSizer,
    /// The maximum batch size (meaning depends on `sizer`).
    pub send_batch_size: usize,
    /// The maximum time to buffer before flushing a batch.
    pub timeout: Duration,
}
impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            sizer: BatchSizer::Items,
            send_batch_size: 512,
            timeout: Duration::from_secs(5),
        }
    }
}

/// A generic batch processor for OTLP data, supporting traces, metrics, and logs.
pub struct GenericBatcher {
    // These are the "pending" partial batch for each type.
    traces_pending: Option<ExportTraceServiceRequest>,
    metrics_pending: Option<ExportMetricsServiceRequest>,
    logs_pending: Option<ExportLogsServiceRequest>,
    last_update_traces: Instant,
    last_update_metrics: Instant,
    last_update_logs: Instant,
    config: BatchConfig,
}

impl GenericBatcher {
    // Your struct's own helper functions, constructors, etc.
    fn count_spans(req: &ExportTraceServiceRequest) -> usize {
        req.resource_spans
            .iter()
            .flat_map(|rs| &rs.scope_spans)
            .flat_map(|ss| &ss.spans)
            .count()
    }
    fn count_metrics(req: &ExportMetricsServiceRequest) -> usize {
        req.resource_metrics
            .iter()
            .flat_map(|rm| &rm.scope_metrics)
            .flat_map(|sm| &sm.metrics)
            .count()
    }
    fn count_logs(req: &ExportLogsServiceRequest) -> usize {
        req.resource_logs
            .iter()
            .flat_map(|rl| &rl.scope_logs)
            .flat_map(|sl| &sl.log_records)
            .count()
    }
}

impl GenericBatcher {
    /// Creates a new `GenericBatcher` with the given configuration.
    pub fn new(config: BatchConfig) -> Self {
        let now = Instant::now();
        Self {
            traces_pending: None,
            metrics_pending: None,
            logs_pending: None,
            last_update_traces: now,
            last_update_metrics: now,
            last_update_logs: now,
            config,
        }
    }

    fn count_trace_requests(req: &ExportTraceServiceRequest) -> usize {
        req.resource_spans.len()
    }
    fn count_metric_requests(req: &ExportMetricsServiceRequest) -> usize {
        req.resource_metrics.len()
    }
    fn count_log_requests(req: &ExportLogsServiceRequest) -> usize {
        req.resource_logs.len()
    }

    async fn flush_traces(
        &mut self,
        handler: &mut EffectHandler<OTLPData>,
    ) -> Result<(), Error<OTLPData>> {
        if let Some(pending) = self.traces_pending.take() {
            handler.send_message(OTLPData::Traces(pending)).await?;
        }
        self.last_update_traces = Instant::now();
        Ok(())
    }
    async fn flush_metrics(
        &mut self,
        handler: &mut EffectHandler<OTLPData>,
    ) -> Result<(), Error<OTLPData>> {
        if let Some(pending) = self.metrics_pending.take() {
            handler.send_message(OTLPData::Metrics(pending)).await?;
        }
        self.last_update_metrics = Instant::now();
        Ok(())
    }
    async fn flush_logs(
        &mut self,
        handler: &mut EffectHandler<OTLPData>,
    ) -> Result<(), Error<OTLPData>> {
        if let Some(pending) = self.logs_pending.take() {
            handler.send_message(OTLPData::Logs(pending)).await?;
        }
        self.last_update_logs = Instant::now();
        Ok(())
    }
    async fn flush_on_timeout(
        &mut self,
        handler: &mut EffectHandler<OTLPData>,
    ) -> Result<(), Error<OTLPData>> {
        let now = Instant::now();
        let timeout = self.config.timeout;
        if timeout != Duration::from_secs(0) {
            if self.traces_pending.is_some()
                && now.duration_since(self.last_update_traces) >= timeout
            {
                self.flush_traces(handler).await?;
            }
            if self.metrics_pending.is_some()
                && now.duration_since(self.last_update_metrics) >= timeout
            {
                self.flush_metrics(handler).await?;
            }
            if self.logs_pending.is_some() && now.duration_since(self.last_update_logs) >= timeout {
                self.flush_logs(handler).await?;
            }
        }
        Ok(())
    }
    async fn flush_all(
        &mut self,
        handler: &mut EffectHandler<OTLPData>,
    ) -> Result<(), Error<OTLPData>> {
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
        effect_handler: &mut EffectHandler<OTLPData>,
    ) -> Result<(), Error<OTLPData>> {

        // Check if we need to flush on requests size

        // ToDo: Future optimization, we should avoid to traverse multiple times the same request to compute multiple counters. That could be done in one pass.
        if self.config.sizer == BatchSizer::Requests {

            match &msg {
                Message::PData(OTLPData::Traces(req)) => {
                    if Self::count_trace_requests(req) >= self.config.send_batch_size {
                        self.flush_traces(effect_handler).await?;
                    }
                }
                Message::PData(OTLPData::Metrics(req)) => {
                    if Self::count_metric_requests(req) >= self.config.send_batch_size {
                        self.flush_metrics(effect_handler).await?;
                    }
                }
                Message::PData(OTLPData::Logs(req)) => {
                    if Self::count_log_requests(req) >= self.config.send_batch_size {
                        self.flush_logs(effect_handler).await?;
                    }
                }
                _ => {}
            }
        }

        // Check if we need to flush on items size
        if self.config.sizer == BatchSizer::Items {
            match &msg {
                Message::PData(OTLPData::Traces(req)) => {
                    if Self::count_spans(req) >= self.config.send_batch_size {
                        self.flush_traces(effect_handler).await?;
                    }
                }
                Message::PData(OTLPData::Metrics(req)) => {
                    if Self::count_metrics(req) >= self.config.send_batch_size {
                        self.flush_metrics(effect_handler).await?;
                    }
                }
                Message::PData(OTLPData::Logs(req)) => {
                    if Self::count_logs(req) >= self.config.send_batch_size {
                        self.flush_logs(effect_handler).await?;
                    }
                }
                _ => {}
            }
        }
        // Check if we need to flush on bytes size
        if self.config.sizer == BatchSizer::Bytes {
            match &msg {
                Message::PData(OTLPData::Traces(req)) => {
                    // ToDo: There is probably a more optimal way to do that! The current implementation is serializing the traces_pending for every incoming req to maintain the total number of bytes and that is way too much overhead.
                    let pending_size = self.traces_pending.as_ref().map(|p| p.encoded_len()).unwrap_or(0);
                    let new_size = req.encoded_len();
                    let total_size = pending_size + new_size;
                    if total_size >= self.config.send_batch_size {
                        self.flush_traces(effect_handler).await?;
                    }
                }
                Message::PData(OTLPData::Metrics(req)) => {
                    let pending_size = self.metrics_pending.as_ref().map(|p| p.encoded_len()).unwrap_or(0);
                    let new_size = req.encoded_len();
                    let total_size = pending_size + new_size;
                    if total_size >= self.config.send_batch_size {
                        self.flush_metrics(effect_handler).await?;
                    }
                }
                Message::PData(OTLPData::Logs(req)) => {
                    let pending_size = self.logs_pending.as_ref().map(|p| p.encoded_len()).unwrap_or(0);
                    let new_size = req.encoded_len();
                    let total_size = pending_size + new_size;
                    if total_size >= self.config.send_batch_size {
                        self.flush_logs(effect_handler).await?;
                    }
                }
                _ => {}
            }
        }
                   
        match msg {
            Message::PData(data) => {
                match data {
                    OTLPData::Traces(mut req) => {
                        if let Some(mut pending) = self.traces_pending.take() {
                            // Merge previous un-flushed groups into input
                            let _ = req
                                .resource_spans
                                .splice(0..0, pending.resource_spans.drain(..));
                        }
                        let mut batches = match self.config.sizer {
                            BatchSizer::Requests => req.split_by_requests(self.config.send_batch_size),
                            _ => req.split_into_batches(self.config.send_batch_size),
                        };
                        // The last batch may not be full: buffer it, emit the rest
                        if let Some(last) = batches.pop() {
                            for batch in &batches {
                                effect_handler
                                    .send_message(OTLPData::Traces(batch.clone()))
                                    .await?;
                            }
                            if Self::count_spans(&last) < self.config.send_batch_size {
                                self.traces_pending = Some(last);
                            } else {
                                // last is also full
                                effect_handler.send_message(OTLPData::Traces(last)).await?;
                            }
                        }
                        self.last_update_traces = Instant::now();
                    }
                    OTLPData::Metrics(mut req) => {
                        if let Some(mut pending) = self.metrics_pending.take() {
                            let _ = req
                                .resource_metrics
                                .splice(0..0, pending.resource_metrics.drain(..));
                        }
                        let mut batches = match self.config.sizer {
                            BatchSizer::Requests => req.split_by_requests(self.config.send_batch_size),
                            _ => req.split_into_batches(self.config.send_batch_size),
                        };
                        if let Some(last) = batches.pop() {
                            for batch in &batches {
                                effect_handler
                                    .send_message(OTLPData::Metrics(batch.clone()))
                                    .await?;
                            }
                            if Self::count_metrics(&last) < self.config.send_batch_size {
                                self.metrics_pending = Some(last);
                            } else {
                                effect_handler.send_message(OTLPData::Metrics(last)).await?;
                            }
                        }
                        self.last_update_metrics = Instant::now();
                    }
                    OTLPData::Logs(mut req) => {
                        if let Some(mut pending) = self.logs_pending.take() {
                            let _ = req
                                .resource_logs
                                .splice(0..0, pending.resource_logs.drain(..));
                        }
                        let mut batches = match self.config.sizer {
                            BatchSizer::Requests => req.split_by_requests(self.config.send_batch_size),
                            _ => req.split_into_batches(self.config.send_batch_size),
                        };
                        if let Some(last) = batches.pop() {
                            for batch in &batches {
                                effect_handler
                                    .send_message(OTLPData::Logs(batch.clone()))
                                    .await?;
                            }
                            if Self::count_logs(&last) < self.config.send_batch_size {
                                self.logs_pending = Some(last);
                            } else {
                                effect_handler.send_message(OTLPData::Logs(last)).await?;
                            }
                        }
                        self.last_update_logs = Instant::now();
                    }
                    OTLPData::Profiles(_) => {
                        // Not implemented yet
                        todo!("Profiles batching is not implemented yet");
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
    use otap_df_engine::config::ProcessorConfig;
    use otap_df_engine::processor::ProcessorWrapper;
    use otap_df_engine::testing::processor::TestRuntime;
    use crate::proto::opentelemetry::logs::v1::LogRecord;
    use crate::proto::opentelemetry::metrics::v1::Metric;
    use crate::proto::opentelemetry::trace::v1::Span;
    use crate::proto::opentelemetry::common::v1::InstrumentationScope;

    /// Wraps a processor in a local test wrapper.
    fn wrap_local<P>(processor: P) -> ProcessorWrapper<OTLPData>
    where
        P: Processor<OTLPData> + 'static,
    {
        let config = ProcessorConfig::new("simple_generic_batch_processor_test");
        ProcessorWrapper::local(processor, &config)
    }

    #[test]
    fn no_duplicate_scope_groups_across_batches() {
        let runtime = TestRuntime::<OTLPData>::new();
        let config = BatchConfig {
            sizer: BatchSizer::Items,
            send_batch_size: 2, // force batching
            timeout: Duration::from_secs(60),
        };
        let wrapper = wrap_local(GenericBatcher::new(config));
        runtime
            .set_processor(wrapper)
            .run_test(|mut ctx| async move {
                // Compose a request with 1 resource group, 3 scope groups, each with 1 span
                let req = ExportTraceServiceRequest {
                    resource_spans: vec![
                        ResourceSpans {
                            resource: None,
                            schema_url: String::new(),
                            scope_spans: vec![
                                ScopeSpans {
                                    scope: Some(InstrumentationScope {
                                        name: "scope1".to_string(),
                                        ..Default::default()
                                    }),
                                    spans: vec![Span { name: "A".into(), ..Default::default() }],
                                    schema_url: String::new(),
                                },
                                ScopeSpans {
                                    scope: Some(InstrumentationScope {
                                        name: "scope2".to_string(),
                                        ..Default::default()
                                    }),
                                    spans: vec![Span { name: "B".into(), ..Default::default() }],
                                    schema_url: String::new(),
                                },
                                ScopeSpans {
                                    scope: Some(InstrumentationScope {
                                        name: "scope3".to_string(),
                                        ..Default::default()
                                    }),
                                    spans: vec![Span { name: "C".into(), ..Default::default() }],
                                    schema_url: String::new(),
                                },
                            ],
                        }
                    ],
                };
                ctx.process(Message::PData(OTLPData::Traces(req)))
                    .await
                    .unwrap();
                ctx.process(Message::Control(ControlMsg::Shutdown {
                    deadline: Duration::from_secs(1),
                    reason: "test".to_string(),
                }))
                .await
                .unwrap();
                let emitted = ctx.drain_pdata().await;
    
                // Collect all scope names from all batches
                let mut seen = std::collections::HashSet::new();
                for batch in &emitted {
                    match batch {
                        OTLPData::Traces(req) => {
                            for rs in &req.resource_spans {
                                for ss in &rs.scope_spans {
                                    let name = ss.scope.as_ref().map(|s| s.name.clone()).unwrap_or_default();
                                    assert!(
                                        seen.insert(name.clone()),
                                        "duplicate scope group: {}",
                                        name
                                    );
                                }
                            }
                        }
                        _ => panic!("Wrong batch type"),
                    }
                }
                // Should see all 3 unique scopes, none duplicated
                assert_eq!(seen.len(), 3, "All scope groups found, none duplicated");
            })
            .validate(|_ctx| async {});
    }

    #[test]
    fn traces_group_preserving_split() {
        let runtime = TestRuntime::<OTLPData>::new();
        let config = BatchConfig {
            sizer: BatchSizer::Items,
            send_batch_size: 3,
            timeout: Duration::from_secs(60),
        };
        let wrapper = wrap_local(GenericBatcher::new(config));
        runtime
            .set_processor(wrapper)
            .run_test(|mut ctx| async move {
                // Compose a request with 2 resource groups, each with 2 scope groups, each with 2 spans
                let req = ExportTraceServiceRequest {
                    resource_spans: vec![
                        ResourceSpans {
                            resource: None,
                            schema_url: String::new(),
                            scope_spans: vec![
                                ScopeSpans {
                                    scope: None,
                                    spans: (0..2)
                                        .map(|i| {
                                            let mut s = Span::default();
                                            s.name = format!("A1_span{i}");
                                            s
                                        })
                                        .collect(),
                                    schema_url: String::new(),
                                },
                                ScopeSpans {
                                    scope: None,
                                    spans: (0..2)
                                        .map(|i| {
                                            let mut s = Span::default();
                                            s.name = format!("A2_span{i}");
                                            s
                                        })
                                        .collect(),
                                    schema_url: String::new(),
                                },
                            ],
                        },
                        ResourceSpans {
                            resource: None,
                            schema_url: String::new(),
                            scope_spans: vec![
                                ScopeSpans {
                                    scope: None,
                                    spans: (0..2)
                                        .map(|i| {
                                            let mut s = Span::default();
                                            s.name = format!("B1_span{i}");
                                            s
                                        })
                                        .collect(),
                                    schema_url: String::new(),
                                },
                                ScopeSpans {
                                    scope: None,
                                    spans: (0..2)
                                        .map(|i| {
                                            let mut s = Span::default();
                                            s.name = format!("B2_span{i}");
                                            s
                                        })
                                        .collect(),
                                    schema_url: String::new(),
                                },
                            ],
                        },
                    ],
                };
                ctx.process(Message::PData(OTLPData::Traces(req)))
                    .await
                    .unwrap();
                ctx.process(Message::Control(ControlMsg::Shutdown {
                    deadline: Duration::from_secs(1),
                    reason: "test".to_string(),
                }))
                .await
                .unwrap();
                let emitted = ctx.drain_pdata().await;

                // There are 8 spans, batch size 3: expect 3 batches (3+3+2)
                assert_eq!(emitted.len(), 3, "Should emit 3 batches");
                let mut seen = std::collections::HashSet::new();
                for batch in &emitted {
                    match batch {
                        OTLPData::Traces(req) => {
                            // Each batch must have 1 or more resource groups
                            for rs in &req.resource_spans {
                                for ss in &rs.scope_spans {
                                    for span in &ss.spans {
                                        assert!(seen.insert(span.name.clone()), "duplicate span name?");
                                    }
                                }
                            }
                        }
                        _ => panic!("Wrong batch type"),
                    }
                }
                assert_eq!(
                    seen.len(),
                    8,
                    "All 8 unique spans must be present, none dropped or duplicated"
                );
            })
            .validate(|_ctx| async {});
    }

    #[test]
    fn metrics_group_preserving_split() {
        let runtime = TestRuntime::<OTLPData>::new();
        let config = BatchConfig {
            sizer: BatchSizer::Items,
            send_batch_size: 2,
            timeout: Duration::from_secs(60),
        };
        let wrapper = wrap_local(GenericBatcher::new(config));
        runtime
            .set_processor(wrapper)
            .run_test(|mut ctx| async move {
                // 2 resource_metrics × 2 scopes each × 2 metrics = 8 metrics
                let req = ExportMetricsServiceRequest {
                    resource_metrics: vec![
                        ResourceMetrics {
                            resource: None,
                            schema_url: String::new(),
                            scope_metrics: vec![
                                ScopeMetrics {
                                    scope: None,
                                    schema_url: String::new(),
                                    metrics: (0..2)
                                        .map(|i| {
                                            let mut m = Metric::default();
                                            m.name = format!("A1_metric{i}");
                                            m
                                        })
                                        .collect(),
                                },
                                ScopeMetrics {
                                    scope: None,
                                    schema_url: String::new(),
                                    metrics: (0..2)
                                        .map(|i| {
                                            let mut m = Metric::default();
                                            m.name = format!("A2_metric{i}");
                                            m
                                        })
                                        .collect(),
                                },
                            ],
                        },
                        ResourceMetrics {
                            resource: None,
                            schema_url: String::new(),
                            scope_metrics: vec![
                                ScopeMetrics {
                                    scope: None,
                                    schema_url: String::new(),
                                    metrics: (0..2)
                                        .map(|i| {
                                            let mut m = Metric::default();
                                            m.name = format!("B1_metric{i}");
                                            m
                                        })
                                        .collect(),
                                },
                                ScopeMetrics {
                                    scope: None,
                                    schema_url: String::new(),
                                    metrics: (0..2)
                                        .map(|i| {
                                            let mut m = Metric::default();
                                            m.name = format!("B2_metric{i}");
                                            m
                                        })
                                        .collect(),
                                },
                            ],
                        },
                    ],
                };
                ctx.process(Message::PData(OTLPData::Metrics(req)))
                    .await
                    .unwrap();
                ctx.process(Message::Control(ControlMsg::Shutdown {
                    deadline: Duration::from_secs(1),
                    reason: "test".to_string(),
                }))
                .await
                .unwrap();
                let emitted = ctx.drain_pdata().await;

                let mut seen = std::collections::HashSet::new();
                for batch in &emitted {
                    match batch {
                        OTLPData::Metrics(req) => {
                            for rm in &req.resource_metrics {
                                for sm in &rm.scope_metrics {
                                    for metric in &sm.metrics {
                                        assert!(
                                            seen.insert(metric.name.clone()),
                                            "duplicate metric name?"
                                        );
                                    }
                                }
                            }
                        }
                        _ => panic!("Wrong batch type"),
                    }
                }
                assert_eq!(
                    seen.len(),
                    8,
                    "All metrics found, none dropped or duplicated"
                );
            })
            .validate(|_ctx| async {});
    }

    #[test]
    fn logs_group_preserving_split() {
        let runtime = TestRuntime::<OTLPData>::new();
        let config = BatchConfig {
            sizer: BatchSizer::Items,
            send_batch_size: 3,
            timeout: Duration::from_secs(60),
        };
        let wrapper = wrap_local(GenericBatcher::new(config));
        runtime
            .set_processor(wrapper)
            .run_test(|mut ctx| async move {
                // 2 resource_logs × 2 scopes × 2 records = 8 records
                let req = ExportLogsServiceRequest {
                    resource_logs: vec![
                        ResourceLogs {
                            resource: None,
                            schema_url: String::new(),
                            scope_logs: vec![
                                ScopeLogs {
                                    scope: None,
                                    schema_url: String::new(),
                                    log_records: (0..2)
                                        .map(|i| {
                                            let mut rec = LogRecord::default();
                                            rec.severity_text = format!("A1_rec{i}");
                                            rec
                                        })
                                        .collect(),
                                },
                                ScopeLogs {
                                    scope: None,
                                    schema_url: String::new(),
                                    log_records: (0..2)
                                        .map(|i| {
                                            let mut rec = LogRecord::default();
                                            rec.severity_text = format!("A2_rec{i}");
                                            rec
                                        })
                                        .collect(),
                                },
                            ],
                        },
                        ResourceLogs {
                            resource: None,
                            schema_url: String::new(),
                            scope_logs: vec![
                                ScopeLogs {
                                    scope: None,
                                    schema_url: String::new(),
                                    log_records: (0..2)
                                        .map(|i| {
                                            let mut rec = LogRecord::default();
                                            rec.severity_text = format!("B1_rec{i}");
                                            rec
                                        })
                                        .collect(),
                                },
                                ScopeLogs {
                                    scope: None,
                                    schema_url: String::new(),
                                    log_records: (0..2)
                                        .map(|i| {
                                            let mut rec = LogRecord::default();
                                            rec.severity_text = format!("B2_rec{i}");
                                            rec
                                        })
                                        .collect(),
                                },
                            ],
                        },
                    ],
                };
                ctx.process(Message::PData(OTLPData::Logs(req)))
                    .await
                    .unwrap();
                ctx.process(Message::Control(ControlMsg::Shutdown {
                    deadline: Duration::from_secs(1),
                    reason: "test".to_string(),
                }))
                .await
                .unwrap();
                let emitted = ctx.drain_pdata().await;

                let mut seen = std::collections::HashSet::new();
                for batch in &emitted {
                    match batch {
                        OTLPData::Logs(req) => {
                            for rl in &req.resource_logs {
                                for sl in &rl.scope_logs {
                                    for l in &sl.log_records {
                                        assert!(
                                            seen.insert(l.severity_text.clone()),
                                            "duplicate log text?"
                                        );
                                    }
                                }
                            }
                        }
                        _ => panic!("Wrong batch type"),
                    }
                }
                assert_eq!(
                    seen.len(),
                    8,
                    "All log records found, none dropped or duplicated"
                );
            })
            .validate(|_ctx| async {});
    }

    #[test]
    fn handles_empty_input_batch() {
        let runtime = TestRuntime::<OTLPData>::new();
        let wrapper = wrap_local(GenericBatcher::new(BatchConfig {
            sizer: BatchSizer::Items,
            send_batch_size: 10,
            timeout: Duration::from_secs(1),
        }));
        runtime.set_processor(wrapper)
            .run_test(|mut ctx| async move {
                ctx.process(Message::PData(OTLPData::Traces(ExportTraceServiceRequest { resource_spans: vec![] }))).await.unwrap();
            ctx.process(Message::Control(ControlMsg::Shutdown { deadline: Duration::from_secs(1), reason: "test".into() })).await.unwrap();
            let emitted = ctx.drain_pdata().await;
            assert!(emitted.is_empty());
        }).validate(|_| async {});
    }

    #[test]
    fn skips_empty_resource_spans() {
        let runtime = TestRuntime::<OTLPData>::new();
        let wrapper = wrap_local(GenericBatcher::new(BatchConfig {
            sizer: BatchSizer::Items,
            send_batch_size: 3,
            timeout: Duration::from_secs(1),
        }));
        runtime.set_processor(wrapper).run_test(|mut ctx| async move {
            // input has many resource/spans with no actual spans
            let req = ExportTraceServiceRequest {
            resource_spans: vec![
                ResourceSpans { resource: None, schema_url: String::new(), scope_spans: vec![] },
                ResourceSpans { resource: None, schema_url: String::new(), scope_spans: vec![
                    ScopeSpans { scope: None, schema_url: String::new(), spans: vec![] }
                ] },
            ]
        };
        ctx.process(Message::PData(OTLPData::Traces(req))).await.unwrap();
        ctx.process(Message::Control(ControlMsg::Shutdown { deadline: Duration::from_secs(1), reason: "test".into() })).await.unwrap();
        let emitted = ctx.drain_pdata().await;
        assert!(emitted.is_empty());
        }).validate(|_| async {});
    }
    
    #[test]
    fn traces_flushes_on_request_count() {
        let runtime = TestRuntime::<OTLPData>::new();
        let config = BatchConfig {
            sizer: BatchSizer::Requests,
            send_batch_size: 2,
            timeout: Duration::from_secs(60),
        };
        let wrapper = wrap_local(GenericBatcher::new(config));
        runtime
            .set_processor(wrapper)
            .run_test(|mut ctx| async move {
                // Compose a request with 2 resource_spans, each with 1 span
                let req = ExportTraceServiceRequest {
                    resource_spans: vec![
                        ResourceSpans {
                            resource: None,
                            schema_url: String::new(),
                            scope_spans: vec![ScopeSpans {
                                scope: None,
                                spans: vec![Span { name: "A".into(), ..Default::default() }],
                                schema_url: String::new(),
                            }],
                        },
                        ResourceSpans {
                            resource: None,
                            schema_url: String::new(),
                            scope_spans: vec![ScopeSpans {
                                scope: None,
                                spans: vec![Span { name: "B".into(), ..Default::default() }],
                                schema_url: String::new(),
                            }],
                        },
                    ],
                };
                ctx.process(Message::PData(OTLPData::Traces(req)))
                    .await
                    .unwrap();
                ctx.process(Message::Control(ControlMsg::Shutdown {
                    deadline: Duration::from_secs(1),
                    reason: "test".to_string(),
                }))
                .await
                .unwrap();
                let emitted = ctx.drain_pdata().await;
                // Should flush after 2 requests (resource_spans)
                assert_eq!(emitted.len(), 1, "Should emit 1 batch for 2 requests");
            })
            .validate(|_ctx| async {});
    }
    
    #[test]
    fn metrics_flushes_on_request_count() {
        let runtime = TestRuntime::<OTLPData>::new();
        let config = BatchConfig {
            sizer: BatchSizer::Requests,
            send_batch_size: 2,
            timeout: Duration::from_secs(60),
        };
        let wrapper = wrap_local(GenericBatcher::new(config));
        runtime
            .set_processor(wrapper)
            .run_test(|mut ctx| async move {
                let req = ExportMetricsServiceRequest {
                    resource_metrics: vec![
                        ResourceMetrics {
                            resource: None,
                            schema_url: String::new(),
                            scope_metrics: vec![ScopeMetrics {
                                scope: None,
                                schema_url: String::new(),
                                metrics: vec![Metric { name: "A".into(), ..Default::default() }],
                            }],
                        },
                        ResourceMetrics {
                            resource: None,
                            schema_url: String::new(),
                            scope_metrics: vec![ScopeMetrics {
                                scope: None,
                                schema_url: String::new(),
                                metrics: vec![Metric { name: "B".into(), ..Default::default() }],
                            }],
                        },
                    ],
                };
                ctx.process(Message::PData(OTLPData::Metrics(req)))
                    .await
                    .unwrap();
                ctx.process(Message::Control(ControlMsg::Shutdown {
                    deadline: Duration::from_secs(1),
                    reason: "test".to_string(),
                }))
                .await
                .unwrap();
                let emitted = ctx.drain_pdata().await;
                assert_eq!(emitted.len(), 1, "Should emit 1 batch for 2 requests");
            })
            .validate(|_ctx| async {});
    }
    
    #[test]
    fn logs_flushes_on_request_count() {
        let runtime = TestRuntime::<OTLPData>::new();
        let config = BatchConfig {
            sizer: BatchSizer::Requests,
            send_batch_size: 2,
            timeout: Duration::from_secs(60),
        };
        let wrapper = wrap_local(GenericBatcher::new(config));
        runtime
            .set_processor(wrapper)
            .run_test(|mut ctx| async move {
                let req = ExportLogsServiceRequest {
                    resource_logs: vec![
                        ResourceLogs {
                            resource: None,
                            schema_url: String::new(),
                            scope_logs: vec![ScopeLogs {
                                scope: None,
                                schema_url: String::new(),
                                log_records: vec![LogRecord { severity_text: "A".into(), ..Default::default() }],
                            }],
                        },
                        ResourceLogs {
                            resource: None,
                            schema_url: String::new(),
                            scope_logs: vec![ScopeLogs {
                                scope: None,
                                schema_url: String::new(),
                                log_records: vec![LogRecord { severity_text: "B".into(), ..Default::default() }],
                            }],
                        },
                    ],
                };
                ctx.process(Message::PData(OTLPData::Logs(req)))
                    .await
                    .unwrap();
                ctx.process(Message::Control(ControlMsg::Shutdown {
                    deadline: Duration::from_secs(1),
                    reason: "test".to_string(),
                }))
                .await
                .unwrap();
                let emitted = ctx.drain_pdata().await;
                assert_eq!(emitted.len(), 1, "Should emit 1 batch for 2 requests");
            })
            .validate(|_ctx| async {});
    }
    
    #[test]
    fn traces_flushes_on_byte_size() {
        let runtime = TestRuntime::<OTLPData>::new();
        let config = BatchConfig {
            sizer: BatchSizer::Bytes,
            send_batch_size: 1, // very small, should flush immediately
            timeout: Duration::from_secs(60),
        };
        let wrapper = wrap_local(GenericBatcher::new(config));
        runtime
            .set_processor(wrapper)
            .run_test(|mut ctx| async move {
                let req = ExportTraceServiceRequest {
                    resource_spans: vec![ResourceSpans {
                        resource: None,
                        schema_url: String::new(),
                        scope_spans: vec![ScopeSpans {
                            scope: None,
                            spans: vec![Span { name: "A".into(), ..Default::default() }],
                            schema_url: String::new(),
                        }],
                    }],
                };
                ctx.process(Message::PData(OTLPData::Traces(req)))
                    .await
                    .unwrap();
                ctx.process(Message::Control(ControlMsg::Shutdown {
                    deadline: Duration::from_secs(1),
                    reason: "test".to_string(),
                }))
                .await
                .unwrap();
                let emitted = ctx.drain_pdata().await;
                assert_eq!(emitted.len(), 1, "Should emit 1 batch due to byte size");
            })
            .validate(|_ctx| async {});
    }
    
    #[test]
    fn metrics_flushes_on_byte_size() {
        let runtime = TestRuntime::<OTLPData>::new();
        let config = BatchConfig {
            sizer: BatchSizer::Bytes,
            send_batch_size: 1, // very small, should flush immediately
            timeout: Duration::from_secs(60),
        };
        let wrapper = wrap_local(GenericBatcher::new(config));
        runtime
            .set_processor(wrapper)
            .run_test(|mut ctx| async move {
                let req = ExportMetricsServiceRequest {
                    resource_metrics: vec![ResourceMetrics {
                        resource: None,
                        schema_url: String::new(),
                        scope_metrics: vec![ScopeMetrics {
                            scope: None,
                            schema_url: String::new(),
                            metrics: vec![Metric { name: "A".into(), ..Default::default() }],
                        }],
                    }],
                };
                ctx.process(Message::PData(OTLPData::Metrics(req)))
                    .await
                    .unwrap();
                ctx.process(Message::Control(ControlMsg::Shutdown {
                    deadline: Duration::from_secs(1),
                    reason: "test".to_string(),
                }))
                .await
                .unwrap();
                let emitted = ctx.drain_pdata().await;
                assert_eq!(emitted.len(), 1, "Should emit 1 batch due to byte size");
            })
            .validate(|_ctx| async {});
    }
    
    #[test]
    fn logs_flushes_on_byte_size() {
        let runtime = TestRuntime::<OTLPData>::new();
        let config = BatchConfig {
            sizer: BatchSizer::Bytes,
            send_batch_size: 1, // very small, should flush immediately
            timeout: Duration::from_secs(60),
        };
        let wrapper = wrap_local(GenericBatcher::new(config));
        runtime
            .set_processor(wrapper)
            .run_test(|mut ctx| async move {
                let req = ExportLogsServiceRequest {
                    resource_logs: vec![ResourceLogs {
                        resource: None,
                        schema_url: String::new(),
                        scope_logs: vec![ScopeLogs {
                            scope: None,
                            schema_url: String::new(),
                            log_records: vec![LogRecord { severity_text: "A".into(), ..Default::default() }],
                        }],
                    }],
                };
                ctx.process(Message::PData(OTLPData::Logs(req)))
                    .await
                    .unwrap();
                ctx.process(Message::Control(ControlMsg::Shutdown {
                    deadline: Duration::from_secs(1),
                    reason: "test".to_string(),
                }))
                .await
                .unwrap();
                let emitted = ctx.drain_pdata().await;
                assert_eq!(emitted.len(), 1, "Should emit 1 batch due to byte size");
            })
            .validate(|_ctx| async {});
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::time::Duration;
    use crate::proto::opentelemetry::logs::v1::LogRecord;
    use crate::proto::opentelemetry::metrics::v1::Metric;
    use crate::proto::opentelemetry::trace::v1::Span;
    use crate::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
    use crate::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
    use crate::proto::opentelemetry::collector::trace::v1::ExportTraceServiceRequest;
    use otap_df_engine::config::ProcessorConfig;
    use otap_df_engine::processor::ProcessorWrapper;
    use otap_df_engine::local::processor::Processor;

    fn wrap_local<P>(processor: P) -> ProcessorWrapper<OTLPData>
    where
        P: Processor<OTLPData> + 'static,
    {
        let config = ProcessorConfig::new("simple_generic_batch_processor_test");
        ProcessorWrapper::local(processor, &config)
    }

    // Helper: Write string to a file
    fn log_to_file(s: &str) {
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open("/tmp/generic_batch_proc_test.json")
            .expect("could not open /tmp file for writing");
        writeln!(f, "{}\n", s).expect("Write failed");
    }

    fn sample_trace() -> ExportTraceServiceRequest {
        ExportTraceServiceRequest {
            resource_spans: vec![
                ResourceSpans {
                    resource: None,
                    schema_url: "".to_string(),
                    scope_spans: vec![
                        ScopeSpans {
                            scope: None,
                            spans: vec![
                                Span { name: "trace-span-1".into(), ..Default::default() },
                                Span { name: "trace-span-2".into(), ..Default::default() },
                                Span { name: "trace-span-3".into(), ..Default::default() },
                            ],
                            schema_url: "".to_string(),
                        }
                    ],
                }
            ]
        }
    }

    fn sample_metrics() -> ExportMetricsServiceRequest {
        ExportMetricsServiceRequest {
            resource_metrics: vec![
                ResourceMetrics {
                    resource: None,
                    schema_url: "".to_string(),
                    scope_metrics: vec![
                        ScopeMetrics {
                            scope: None,
                            metrics: vec![
                                Metric { name: "metric-1".to_string(), ..Default::default() },
                                Metric { name: "metric-2".to_string(), ..Default::default() },
                                Metric { name: "metric-3".to_string(), ..Default::default() },
                            ],
                            schema_url: "".to_string(),
                        }
                    ],
                }
            ]
        }
    }

    fn sample_logs() -> ExportLogsServiceRequest {
        ExportLogsServiceRequest {
            resource_logs: vec![
                ResourceLogs {
                    resource: None,
                    schema_url: "".to_string(),
                    scope_logs: vec![
                        ScopeLogs {
                            scope: None,
                            log_records: vec![
                                LogRecord { severity_text: "info".to_string(), ..Default::default() },
                                LogRecord { severity_text: "error".to_string(), ..Default::default() },
                                LogRecord { severity_text: "error".to_string(), ..Default::default() },
                            ],
                            schema_url: "".to_string(),
                        }
                    ],
                }
            ]
        }
    }

    #[test]
    fn log_inputs_and_outputs_json() {
        // Clear output file
        std::fs::write("/tmp/generic_batch_proc_test.json", "").unwrap();

        let runtime = otap_df_engine::testing::processor::TestRuntime::<OTLPData>::new();
        let wrapper = wrap_local(GenericBatcher::new(BatchConfig {
            sizer: BatchSizer::Items,
            send_batch_size: 2, // test splitting
            timeout: Duration::from_secs(1),
        }));

        runtime.set_processor(wrapper)
            .run_test(|mut ctx| async move {
                // TRACE INPUT
                let trace_req = sample_trace();
                log_to_file(&format!("INPUT TRACE:\n{}", format!("{:#?}", trace_req)));
                ctx.process(Message::PData(OTLPData::Traces(trace_req))).await.unwrap();

                // METRICS INPUT
                let metrics_req = sample_metrics();
                log_to_file(&format!("INPUT METRIC:\n{}", format!("{:#?}", metrics_req)));
                ctx.process(Message::PData(OTLPData::Metrics(metrics_req))).await.unwrap();

                // LOGS INPUT
                let logs_req = sample_logs();
                log_to_file(&format!("INPUT LOGS:\n{}", format!("{:#?}", logs_req)));
                ctx.process(Message::PData(OTLPData::Logs(logs_req))).await.unwrap();

                // flush everything
                ctx.process(Message::Control(ControlMsg::Shutdown { deadline: Duration::from_secs(1), reason: "test".into() })).await.unwrap();

                // OUTPUTS
                let outputs = ctx.drain_pdata().await;
                for (i, out) in outputs.iter().enumerate() {
                    match out {
                        OTLPData::Traces(req) => log_to_file(&format!("OUTPUT[{}] TRACE:\n{}", i, format!("{:#?}", req))),
                        OTLPData::Metrics(req) => log_to_file(&format!("OUTPUT[{}] METRIC:\n{}", i, format!("{:#?}", req))),
                        OTLPData::Logs(req) => log_to_file(&format!("OUTPUT[{}] LOGS:\n{}", i, format!("{:#?}", req))),
                        #[allow(unreachable_patterns)]
                        _ => log_to_file(&format!("OUTPUT[{}] (unknown type)", i)),
                    }
                }
            })
            .validate(|_| async {});
    }
}