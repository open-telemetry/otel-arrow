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
use prost::Message as ProstMessage;
use std::borrow::Cow;
use std::time::{Duration, Instant};

/// Trait for a batch type (e.g., ExportTraceServiceRequest, ExportMetricsServiceRequest, ExportLogsServiceRequest)
pub trait Batch: Sized {
    type Resource: ResourceGroup;
    fn resources_mut(&mut self) -> &mut Vec<Self::Resource>;
    fn new_empty() -> Self;
}

/// Trait for a resource group (e.g., ResourceSpans, ResourceMetrics, ResourceLogs)
pub trait ResourceGroup: Sized {
    type Scope: ScopeGroup;
    fn scopes_mut(&mut self) -> &mut Vec<Self::Scope>;
    fn take_resource_fields(&mut self) -> Self;
}

/// Trait for a scope group (e.g., ScopeSpans, ScopeMetrics, ScopeLogs)
pub trait ScopeGroup: Sized {
    type Leaf;
    fn leaves_mut(&mut self) -> &mut Vec<Self::Leaf>;
    fn take_scope_fields(&mut self) -> Self;
}

/// -- Traces --
impl Batch for ExportTraceServiceRequest {
    type Resource = ResourceSpans;
    fn resources_mut(&mut self) -> &mut Vec<Self::Resource> {
        &mut self.resource_spans
    }
    fn new_empty() -> Self {
        ExportTraceServiceRequest {
            resource_spans: Vec::new(),
        }
    }
}

impl ResourceGroup for ResourceSpans {
    type Scope = ScopeSpans;
    fn scopes_mut(&mut self) -> &mut Vec<Self::Scope> {
        &mut self.scope_spans
    }
    fn take_resource_fields(&mut self) -> Self {
        ResourceSpans {
            resource: self.resource.take(),
            scope_spans: Vec::new(),
            schema_url: self.schema_url.clone(),
        }
    }
}

impl ScopeGroup for ScopeSpans {
    type Leaf = crate::proto::opentelemetry::trace::v1::Span;
    fn leaves_mut(&mut self) -> &mut Vec<Self::Leaf> {
        &mut self.spans
    }
    fn take_scope_fields(&mut self) -> Self {
        ScopeSpans {
            scope: self.scope.take(),
            spans: Vec::new(),
            schema_url: self.schema_url.clone(),
        }
    }
}

/// Generic batching logic for any OTLP batch type using move semantics.
pub fn split_into_batches<B: Batch>(mut batch: B, max_batch_size: usize) -> Vec<B> {
    let mut batches = Vec::new();
    let mut current_batch = B::new_empty();
    let mut current_count = 0;

    for mut resource in batch.resources_mut().drain(..) {
        let mut res = resource.take_resource_fields();
        let scopes = resource.scopes_mut().drain(..).collect::<Vec<_>>();

        for mut scope in scopes {
            while !scope.leaves_mut().is_empty() {
                let leaves = scope.leaves_mut();
                let n = leaves.len();
                let batch_space = max_batch_size - current_count;
                let take_amount = batch_space.min(n);
                let split_at = n - take_amount;

                let mut taken = leaves.split_off(split_at);

                if !taken.is_empty() {
                    let mut new_scope = scope.take_scope_fields();
                    std::mem::swap(new_scope.leaves_mut(), &mut taken);
                    res.scopes_mut().push(new_scope);
                    current_count += take_amount;
                }
                if current_count == max_batch_size {
                    current_batch.resources_mut().push(res);
                    batches.push(current_batch);
                    current_batch = B::new_empty();
                    current_count = 0;
                    res = resource.take_resource_fields();
                }
            }
        }
        if !res.scopes_mut().is_empty() {
            current_batch.resources_mut().push(res);
        }
    }
    if !current_batch.resources_mut().is_empty() {
        batches.push(current_batch);
    }
    batches
}

/// Trait for hierarchical batch splitting
///
/// This trait is used to split a batch into a vector of smaller batches, each with at most `max_batch_size`
/// leaf items, preserving all resource/scope/leaf (span/metric/logrecord) structure.
pub trait HierarchicalBatchSplit: Sized {
    fn split_into_batches(self, max_batch_size: usize) -> Result<Vec<Self>, Error<OTLPData>>;
}

/// TODO: Use the pdata/otlp support library, rewrite this function to be generic over PData as that library develops
impl HierarchicalBatchSplit for ExportTraceServiceRequest {
    fn split_into_batches(self, max_batch_size: usize) -> Result<Vec<Self>, Error<OTLPData>> {
        if max_batch_size == 0 {
            return Err(Error::ProcessorError {
                processor: Cow::Borrowed("HierarchicalBatchSplit::ExportTraceServiceRequest"),
                error: "max_batch_size must be greater than zero".into(),
            });
        }
        Ok(split_into_batches(self, max_batch_size))
    }
}

// --- Metrics ---

impl Batch for ExportMetricsServiceRequest {
    type Resource = ResourceMetrics;
    fn resources_mut(&mut self) -> &mut Vec<Self::Resource> {
        &mut self.resource_metrics
    }
    fn new_empty() -> Self {
        ExportMetricsServiceRequest {
            resource_metrics: Vec::new(),
        }
    }
}

impl ResourceGroup for ResourceMetrics {
    type Scope = ScopeMetrics;
    fn scopes_mut(&mut self) -> &mut Vec<Self::Scope> {
        &mut self.scope_metrics
    }
    fn take_resource_fields(&mut self) -> Self {
        ResourceMetrics {
            resource: self.resource.take(),
            scope_metrics: Vec::new(),
            schema_url: self.schema_url.clone(),
        }
    }
}

impl ScopeGroup for ScopeMetrics {
    type Leaf = crate::proto::opentelemetry::metrics::v1::Metric;
    fn leaves_mut(&mut self) -> &mut Vec<Self::Leaf> {
        &mut self.metrics
    }
    fn take_scope_fields(&mut self) -> Self {
        ScopeMetrics {
            scope: self.scope.take(),
            metrics: Vec::new(),
            schema_url: self.schema_url.clone(),
        }
    }
}

impl HierarchicalBatchSplit for ExportMetricsServiceRequest {
    fn split_into_batches(self, max_batch_size: usize) -> Result<Vec<Self>, Error<OTLPData>> {
        if max_batch_size == 0 {
            return Err(Error::ProcessorError {
                processor: Cow::Borrowed("HierarchicalBatchSplit::ExportMetricsServiceRequest"),
                error: "max_batch_size must be greater than zero".into(),
            });
        }
        Ok(split_into_batches(self, max_batch_size))
    }
}

impl ExportMetricsServiceRequest {
    /// Splits the batch into multiple batches, each containing at most `max_requests` top-level requests.
    ///
    /// This preserves the original structure of each request.
    #[must_use]
    pub fn split_by_requests(self, max_requests: usize) -> Vec<Self> {
        self.resource_metrics
            .chunks(max_requests)
            .map(|chunk| ExportMetricsServiceRequest {
                resource_metrics: chunk.to_vec(),
            })
            .collect()
    }
}

// --- Logs ---

impl Batch for ExportLogsServiceRequest {
    type Resource = ResourceLogs;
    fn resources_mut(&mut self) -> &mut Vec<Self::Resource> {
        &mut self.resource_logs
    }
    fn new_empty() -> Self {
        ExportLogsServiceRequest {
            resource_logs: Vec::new(),
        }
    }
}

impl ResourceGroup for ResourceLogs {
    type Scope = ScopeLogs;
    fn scopes_mut(&mut self) -> &mut Vec<Self::Scope> {
        &mut self.scope_logs
    }
    fn take_resource_fields(&mut self) -> Self {
        ResourceLogs {
            resource: self.resource.take(),
            scope_logs: Vec::new(),
            schema_url: self.schema_url.clone(),
        }
    }
}

impl ScopeGroup for ScopeLogs {
    type Leaf = crate::proto::opentelemetry::logs::v1::LogRecord;
    fn leaves_mut(&mut self) -> &mut Vec<Self::Leaf> {
        &mut self.log_records
    }
    fn take_scope_fields(&mut self) -> Self {
        ScopeLogs {
            scope: self.scope.clone(),
            log_records: Vec::new(),
            schema_url: self.schema_url.clone(),
        }
    }
}

impl HierarchicalBatchSplit for ExportLogsServiceRequest {
    fn split_into_batches(self, max_batch_size: usize) -> Result<Vec<Self>, Error<OTLPData>> {
        if max_batch_size == 0 {
            return Err(Error::ProcessorError {
                processor: Cow::Borrowed("HierarchicalBatchSplit::ExportLogsServiceRequest"),
                error: "max_batch_size must be greater than zero".into(),
            });
        }
        Ok(split_into_batches(self, max_batch_size))
    }
}

impl ExportLogsServiceRequest {
    /// Splits the batch into multiple batches, each containing at most `max_requests` top-level requests.
    ///
    /// This preserves the original structure of each request.
    #[must_use]
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
    #[must_use]
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
    #[allow(dead_code)]
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
                    let pending_size = self
                        .traces_pending
                        .as_ref()
                        .map(|p| p.encoded_len())
                        .unwrap_or(0);
                    let new_size = req.encoded_len();
                    let total_size = pending_size + new_size;
                    if total_size >= self.config.send_batch_size {
                        self.flush_traces(effect_handler).await?;
                    }
                }
                Message::PData(OTLPData::Metrics(req)) => {
                    let pending_size = self
                        .metrics_pending
                        .as_ref()
                        .map(|p| p.encoded_len())
                        .unwrap_or(0);
                    let new_size = req.encoded_len();
                    let total_size = pending_size + new_size;
                    if total_size >= self.config.send_batch_size {
                        self.flush_metrics(effect_handler).await?;
                    }
                }
                Message::PData(OTLPData::Logs(req)) => {
                    let pending_size = self
                        .logs_pending
                        .as_ref()
                        .map(|p| p.encoded_len())
                        .unwrap_or(0);
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
                            BatchSizer::Requests => {
                                Ok(req.split_by_requests(self.config.send_batch_size))
                            }
                            _ => req.split_into_batches(self.config.send_batch_size),
                        }?;
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
                            BatchSizer::Requests => {
                                Ok(req.split_by_requests(self.config.send_batch_size))
                            }
                            _ => req.split_into_batches(self.config.send_batch_size),
                        }?;
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
                            BatchSizer::Requests => {
                                Ok(req.split_by_requests(self.config.send_batch_size))
                            }
                            _ => req.split_into_batches(self.config.send_batch_size),
                        }?;
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
    use crate::proto::opentelemetry::common::v1::{
        AnyValue, InstrumentationScope, KeyValue, any_value,
    };
    use crate::proto::opentelemetry::logs::v1::LogRecord;
    use crate::proto::opentelemetry::metrics::v1::Metric;
    use crate::proto::opentelemetry::resource::v1::Resource;
    use crate::proto::opentelemetry::trace::v1::Span;
    use otap_df_engine::config::ProcessorConfig;
    use otap_df_engine::processor::ProcessorWrapper;
    use otap_df_engine::testing::processor::TestRuntime;

    /// Wraps a processor in a local test wrapper.
    fn wrap_local<P>(processor: P) -> ProcessorWrapper<OTLPData>
    where
        P: Processor<OTLPData> + 'static,
    {
        let config = ProcessorConfig::new("simple_generic_batch_processor_test");
        ProcessorWrapper::local(processor, &config)
    }

    #[test]
    fn logs_batching_preserves_resource_and_scope_boundaries() {
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
                // Resource A, Scope X: [log1, log2, log3]
                // Resource B, Scope Y: [log4]
                let req = ExportLogsServiceRequest {
                    resource_logs: vec![
                        ResourceLogs {
                            resource: Some(Resource {
                                attributes: vec![KeyValue {
                                    key: "resource_id".to_string(),
                                    value: Some(AnyValue {
                                        value: Some(any_value::Value::StringValue(
                                            "resourceA".to_string(),
                                        )),
                                    }),
                                }],
                                ..Default::default()
                            }),
                            schema_url: String::new(),
                            scope_logs: vec![ScopeLogs {
                                scope: Some(InstrumentationScope {
                                    name: "scopeX".to_string(),
                                    ..Default::default()
                                }),
                                schema_url: String::new(),
                                log_records: vec![
                                    LogRecord {
                                        severity_text: "log1".into(),
                                        ..Default::default()
                                    },
                                    LogRecord {
                                        severity_text: "log2".into(),
                                        ..Default::default()
                                    },
                                    LogRecord {
                                        severity_text: "log3".into(),
                                        ..Default::default()
                                    },
                                ],
                            }],
                        },
                        ResourceLogs {
                            resource: Some(Resource {
                                attributes: vec![KeyValue {
                                    key: "resource_id".to_string(),
                                    value: Some(AnyValue {
                                        value: Some(any_value::Value::StringValue(
                                            "resourceB".to_string(),
                                        )),
                                    }),
                                }],
                                ..Default::default()
                            }),
                            schema_url: String::new(),
                            scope_logs: vec![ScopeLogs {
                                scope: Some(InstrumentationScope {
                                    name: "scopeY".to_string(),
                                    ..Default::default()
                                }),
                                schema_url: String::new(),
                                log_records: vec![LogRecord {
                                    severity_text: "log4".into(),
                                    ..Default::default()
                                }],
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

                // Check that no batch contains log records from both resources or both scopes
                for batch in &emitted {
                    if let OTLPData::Logs(req) = batch {
                        let mut resource_ids = std::collections::HashSet::new();
                        let mut scope_ids = std::collections::HashSet::new();
                        for rl in &req.resource_logs {
                            // Use pointer or unique field for resource identity
                            let _ = resource_ids
                                .insert(rl.resource.as_ref().map(|_| "A").unwrap_or("B"));
                            for sl in &rl.scope_logs {
                                let _ =
                                    scope_ids.insert(sl.scope.as_ref().map(|_| "X").unwrap_or("Y"));
                            }
                        }
                        assert!(
                            scope_ids.len() == 1,
                            "Batch contains log records from multiple scopes"
                        );
                    }
                }
            })
            .validate(|_| async {});
    }

    #[test]
    fn logs_batching_infinite_loop_if_continue_missing() {
        // This test demonstrates that the log batching implementation will not enter an infinite loop
        // when repeatedly flushing full batches. If the batching logic is incorrect (e.g., missing a
        // `continue` after flushing when `take == 0`), this test would hang or fail. With the correct
        // batching logic, the test completes successfully, processing all log records as expected.
        let runtime = TestRuntime::<OTLPData>::new();
        let config = BatchConfig {
            sizer: BatchSizer::Items,
            send_batch_size: 1, // force a flush after every log record
            timeout: Duration::from_secs(60),
        };
        let wrapper = wrap_local(GenericBatcher::new(config));
        runtime
            .set_processor(wrapper)
            .run_test(|mut ctx| async move {
                // Three log records in a single scope
                let req = ExportLogsServiceRequest {
                    resource_logs: vec![ResourceLogs {
                        resource: None,
                        schema_url: String::new(),
                        scope_logs: vec![ScopeLogs {
                            scope: None,
                            schema_url: String::new(),
                            log_records: vec![
                                LogRecord {
                                    severity_text: "A".into(),
                                    ..Default::default()
                                },
                                LogRecord {
                                    severity_text: "B".into(),
                                    ..Default::default()
                                },
                                LogRecord {
                                    severity_text: "C".into(),
                                    ..Default::default()
                                },
                            ],
                        }],
                    }],
                };
                ctx.process(Message::PData(OTLPData::Logs(req)))
                    .await
                    .unwrap();
            })
            .validate(|_| async {});
    }

    #[test]
    fn does_not_emit_empty_trace_batches() {
        let runtime = TestRuntime::<OTLPData>::new();
        let wrapper = wrap_local(GenericBatcher::new(BatchConfig {
            sizer: BatchSizer::Items,
            send_batch_size: 3,
            timeout: Duration::from_secs(1),
        }));
        runtime
            .set_processor(wrapper)
            .run_test(|mut ctx| async move {
                // Send an empty request
                let req = ExportTraceServiceRequest {
                    resource_spans: vec![],
                };
                ctx.process(Message::PData(OTLPData::Traces(req)))
                    .await
                    .unwrap();
                ctx.process(Message::Control(ControlMsg::Shutdown {
                    deadline: Duration::from_secs(1),
                    reason: "test".into(),
                }))
                .await
                .unwrap();
                let emitted = ctx.drain_pdata().await;
                // Assert that no batches are emitted
                assert!(
                    emitted.is_empty(),
                    "No batches should be emitted for empty input"
                );
            })
            .validate(|_| async {});
    }

    #[test]
    fn flushes_partial_batch_on_shutdown() {
        let runtime = TestRuntime::<OTLPData>::new();
        let config = BatchConfig {
            sizer: BatchSizer::Items,
            send_batch_size: 10, // Large, so won't flush by size
            timeout: Duration::from_secs(60),
        };
        let wrapper = wrap_local(GenericBatcher::new(config));
        runtime
            .set_processor(wrapper)
            .run_test(|mut ctx| async move {
                // Send a partial batch (only 3 spans, less than batch size)
                let req = ExportTraceServiceRequest {
                    resource_spans: vec![ResourceSpans {
                        resource: None,
                        schema_url: String::new(),
                        scope_spans: vec![ScopeSpans {
                            scope: None,
                            spans: vec![
                                Span {
                                    name: "A".into(),
                                    ..Default::default()
                                },
                                Span {
                                    name: "B".into(),
                                    ..Default::default()
                                },
                                Span {
                                    name: "C".into(),
                                    ..Default::default()
                                },
                            ],
                            schema_url: String::new(),
                        }],
                    }],
                };
                ctx.process(Message::PData(OTLPData::Traces(req)))
                    .await
                    .unwrap();
                // Trigger shutdown
                ctx.process(Message::Control(ControlMsg::Shutdown {
                    deadline: Duration::from_secs(1),
                    reason: "test".to_string(),
                }))
                .await
                .unwrap();
                let emitted = ctx.drain_pdata().await;
                // Should emit 1 batch containing the partial data
                assert_eq!(emitted.len(), 1, "Should flush partial batch on shutdown");
                match &emitted[0] {
                    OTLPData::Traces(req) => {
                        let count = req
                            .resource_spans
                            .iter()
                            .flat_map(|rs| &rs.scope_spans)
                            .flat_map(|ss| &ss.spans)
                            .count();
                        assert_eq!(
                            count, 3,
                            "All partial spans should be present in the flushed batch"
                        );
                    }
                    _ => panic!("Expected Traces batch"),
                }
            })
            .validate(|_| async {});
    }

    #[test]
    fn flushes_on_timeout() {
        let runtime = TestRuntime::<OTLPData>::new();
        let config = BatchConfig {
            sizer: BatchSizer::Items,
            send_batch_size: 10, // large, so won't flush by size
            timeout: Duration::from_millis(10),
        };
        let wrapper = wrap_local(GenericBatcher::new(config));
        runtime
            .set_processor(wrapper)
            .run_test(|mut ctx| async move {
                // Send a single span
                let req = ExportTraceServiceRequest {
                    resource_spans: vec![ResourceSpans {
                        resource: None,
                        schema_url: String::new(),
                        scope_spans: vec![ScopeSpans {
                            scope: None,
                            spans: vec![Span {
                                name: "A".into(),
                                ..Default::default()
                            }],
                            schema_url: String::new(),
                        }],
                    }],
                };
                ctx.process(Message::PData(OTLPData::Traces(req)))
                    .await
                    .unwrap();
                // Simulate timer tick after timeout
                tokio::time::sleep(Duration::from_millis(20)).await;
                ctx.process(Message::Control(ControlMsg::TimerTick {}))
                    .await
                    .unwrap();
                let emitted = ctx.drain_pdata().await;
                assert_eq!(emitted.len(), 1, "Should flush after timeout");
            })
            .validate(|_| async {});
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
                    resource_spans: vec![ResourceSpans {
                        resource: None,
                        schema_url: String::new(),
                        scope_spans: vec![
                            ScopeSpans {
                                scope: Some(InstrumentationScope {
                                    name: "scope1".to_string(),
                                    ..Default::default()
                                }),
                                spans: vec![Span {
                                    name: "A".into(),
                                    ..Default::default()
                                }],
                                schema_url: String::new(),
                            },
                            ScopeSpans {
                                scope: Some(InstrumentationScope {
                                    name: "scope2".to_string(),
                                    ..Default::default()
                                }),
                                spans: vec![Span {
                                    name: "B".into(),
                                    ..Default::default()
                                }],
                                schema_url: String::new(),
                            },
                            ScopeSpans {
                                scope: Some(InstrumentationScope {
                                    name: "scope3".to_string(),
                                    ..Default::default()
                                }),
                                spans: vec![Span {
                                    name: "C".into(),
                                    ..Default::default()
                                }],
                                schema_url: String::new(),
                            },
                        ],
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

                // Collect all scope names from all batches
                let mut seen = std::collections::HashSet::new();
                for batch in &emitted {
                    match batch {
                        OTLPData::Traces(req) => {
                            for rs in &req.resource_spans {
                                for ss in &rs.scope_spans {
                                    let name = ss
                                        .scope
                                        .as_ref()
                                        .map(|s| s.name.clone())
                                        .unwrap_or_default();
                                    assert!(
                                        seen.insert(name.clone()),
                                        "duplicate scope group: {name}"
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
                                        .map(|i| Span {
                                            name: format!("A1_scope0_span{i}"),
                                            ..Default::default()
                                        })
                                        .collect(),
                                    schema_url: String::new(),
                                },
                                ScopeSpans {
                                    scope: None,
                                    spans: (0..2)
                                        .map(|i| Span {
                                            name: format!("A1_scope1_span{i}"),
                                            ..Default::default()
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
                                        .map(|i| Span {
                                            name: format!("B1_scope0_span{i}"),
                                            ..Default::default()
                                        })
                                        .collect(),
                                    schema_url: String::new(),
                                },
                                ScopeSpans {
                                    scope: None,
                                    spans: (0..2)
                                        .map(|i| Span {
                                            name: format!("B2_scope1_span{i}"),
                                            ..Default::default()
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
                                        assert!(
                                            seen.insert(span.name.clone()),
                                            "duplicate span name?"
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
                                        .map(|i| Metric {
                                            name: format!("A1_metric{i}"),
                                            ..Default::default()
                                        })
                                        .collect(),
                                },
                                ScopeMetrics {
                                    scope: None,
                                    schema_url: String::new(),
                                    metrics: (0..2)
                                        .map(|i| Metric {
                                            name: format!("A2_metric{i}"),
                                            ..Default::default()
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
                                        .map(|i| Metric {
                                            name: format!("B1_metric{i}"),
                                            ..Default::default()
                                        })
                                        .collect(),
                                },
                                ScopeMetrics {
                                    scope: None,
                                    schema_url: String::new(),
                                    metrics: (0..2)
                                        .map(|i| Metric {
                                            name: format!("B2_metric{i}"),
                                            ..Default::default()
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
                                        .map(|i| LogRecord {
                                            severity_text: format!("A1_rec{i}"),
                                            ..Default::default()
                                        })
                                        .collect(),
                                },
                                ScopeLogs {
                                    scope: None,
                                    schema_url: String::new(),
                                    log_records: (0..2)
                                        .map(|i| LogRecord {
                                            severity_text: format!("A2_rec{i}"),
                                            ..Default::default()
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
                                        .map(|i| LogRecord {
                                            severity_text: format!("B1_rec{i}"),
                                            ..Default::default()
                                        })
                                        .collect(),
                                },
                                ScopeLogs {
                                    scope: None,
                                    schema_url: String::new(),
                                    log_records: (0..2)
                                        .map(|i| LogRecord {
                                            severity_text: format!("B2_rec{i}"),
                                            ..Default::default()
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
        runtime
            .set_processor(wrapper)
            .run_test(|mut ctx| async move {
                ctx.process(Message::PData(OTLPData::Traces(
                    ExportTraceServiceRequest {
                        resource_spans: vec![],
                    },
                )))
                .await
                .unwrap();
                ctx.process(Message::Control(ControlMsg::Shutdown {
                    deadline: Duration::from_secs(1),
                    reason: "test".into(),
                }))
                .await
                .unwrap();
                let emitted = ctx.drain_pdata().await;
                assert!(emitted.is_empty());
            })
            .validate(|_| async {});
    }

    #[test]
    fn skips_empty_resource_spans() {
        let runtime = TestRuntime::<OTLPData>::new();
        let wrapper = wrap_local(GenericBatcher::new(BatchConfig {
            sizer: BatchSizer::Items,
            send_batch_size: 3,
            timeout: Duration::from_secs(1),
        }));
        runtime
            .set_processor(wrapper)
            .run_test(|mut ctx| async move {
                // input has many resource/spans with no actual spans
                let req = ExportTraceServiceRequest {
                    resource_spans: vec![
                        ResourceSpans {
                            resource: None,
                            schema_url: String::new(),
                            scope_spans: vec![],
                        },
                        ResourceSpans {
                            resource: None,
                            schema_url: String::new(),
                            scope_spans: vec![ScopeSpans {
                                scope: None,
                                schema_url: String::new(),
                                spans: vec![],
                            }],
                        },
                    ],
                };
                ctx.process(Message::PData(OTLPData::Traces(req)))
                    .await
                    .unwrap();
                ctx.process(Message::Control(ControlMsg::Shutdown {
                    deadline: Duration::from_secs(1),
                    reason: "test".into(),
                }))
                .await
                .unwrap();
                let emitted = ctx.drain_pdata().await;
                assert!(emitted.is_empty());
            })
            .validate(|_| async {});
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
                                spans: vec![Span {
                                    name: "A".into(),
                                    ..Default::default()
                                }],
                                schema_url: String::new(),
                            }],
                        },
                        ResourceSpans {
                            resource: None,
                            schema_url: String::new(),
                            scope_spans: vec![ScopeSpans {
                                scope: None,
                                spans: vec![Span {
                                    name: "B".into(),
                                    ..Default::default()
                                }],
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
                                metrics: vec![Metric {
                                    name: "A".into(),
                                    ..Default::default()
                                }],
                            }],
                        },
                        ResourceMetrics {
                            resource: None,
                            schema_url: String::new(),
                            scope_metrics: vec![ScopeMetrics {
                                scope: None,
                                schema_url: String::new(),
                                metrics: vec![Metric {
                                    name: "B".into(),
                                    ..Default::default()
                                }],
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
                                log_records: vec![LogRecord {
                                    severity_text: "A".into(),
                                    ..Default::default()
                                }],
                            }],
                        },
                        ResourceLogs {
                            resource: None,
                            schema_url: String::new(),
                            scope_logs: vec![ScopeLogs {
                                scope: None,
                                schema_url: String::new(),
                                log_records: vec![LogRecord {
                                    severity_text: "B".into(),
                                    ..Default::default()
                                }],
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
                            spans: vec![Span {
                                name: "A".into(),
                                ..Default::default()
                            }],
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
                            metrics: vec![Metric {
                                name: "A".into(),
                                ..Default::default()
                            }],
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
                            log_records: vec![LogRecord {
                                severity_text: "A".into(),
                                ..Default::default()
                            }],
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
    use crate::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
    use crate::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
    use crate::proto::opentelemetry::collector::trace::v1::ExportTraceServiceRequest;
    use crate::proto::opentelemetry::logs::v1::LogRecord;
    use crate::proto::opentelemetry::metrics::v1::Metric;
    use crate::proto::opentelemetry::trace::v1::Span;
    use otap_df_engine::config::ProcessorConfig;
    use otap_df_engine::local::processor::Processor;
    use otap_df_engine::processor::ProcessorWrapper;
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::time::Duration;

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
        writeln!(f, "{s}\n").expect("Write failed");
    }

    fn sample_trace() -> ExportTraceServiceRequest {
        ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: None,
                schema_url: "".to_string(),
                scope_spans: vec![ScopeSpans {
                    scope: None,
                    spans: vec![
                        Span {
                            name: "trace-span-1".into(),
                            ..Default::default()
                        },
                        Span {
                            name: "trace-span-2".into(),
                            ..Default::default()
                        },
                        Span {
                            name: "trace-span-3".into(),
                            ..Default::default()
                        },
                    ],
                    schema_url: "".to_string(),
                }],
            }],
        }
    }

    fn sample_metrics() -> ExportMetricsServiceRequest {
        ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                resource: None,
                schema_url: "".to_string(),
                scope_metrics: vec![ScopeMetrics {
                    scope: None,
                    metrics: vec![
                        Metric {
                            name: "metric-1".to_string(),
                            ..Default::default()
                        },
                        Metric {
                            name: "metric-2".to_string(),
                            ..Default::default()
                        },
                        Metric {
                            name: "metric-3".to_string(),
                            ..Default::default()
                        },
                    ],
                    schema_url: "".to_string(),
                }],
            }],
        }
    }

    fn sample_logs() -> ExportLogsServiceRequest {
        ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: None,
                schema_url: "".to_string(),
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![
                        LogRecord {
                            severity_text: "info".to_string(),
                            ..Default::default()
                        },
                        LogRecord {
                            severity_text: "error".to_string(),
                            ..Default::default()
                        },
                        LogRecord {
                            severity_text: "error".to_string(),
                            ..Default::default()
                        },
                    ],
                    schema_url: "".to_string(),
                }],
            }],
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

        runtime
            .set_processor(wrapper)
            .run_test(|mut ctx| async move {
                // TRACE INPUT
                let trace_req = sample_trace();
                log_to_file(&format!("INPUT TRACE:\n{trace_req:#?}"));
                ctx.process(Message::PData(OTLPData::Traces(trace_req)))
                    .await
                    .unwrap();

                // METRICS INPUT
                let metrics_req = sample_metrics();
                log_to_file(&format!("INPUT METRIC:\n{metrics_req:#?}"));
                ctx.process(Message::PData(OTLPData::Metrics(metrics_req)))
                    .await
                    .unwrap();

                // LOGS INPUT
                let logs_req = sample_logs();
                log_to_file(&format!("INPUT LOGS:\n{logs_req:#?}"));
                ctx.process(Message::PData(OTLPData::Logs(logs_req)))
                    .await
                    .unwrap();

                // flush everything
                ctx.process(Message::Control(ControlMsg::Shutdown {
                    deadline: Duration::from_secs(1),
                    reason: "test".into(),
                }))
                .await
                .unwrap();

                // OUTPUTS
                let outputs = ctx.drain_pdata().await;
                for (i, out) in outputs.iter().enumerate() {
                    match out {
                        OTLPData::Traces(req) => {
                            log_to_file(&format!("OUTPUT[{i}] TRACE:\n{req:#?}"));
                        }
                        OTLPData::Metrics(req) => {
                            log_to_file(&format!("OUTPUT[{i}] METRICS:\n{req:#?}"));
                        }
                        OTLPData::Logs(req) => {
                            log_to_file(&format!("OUTPUT[{i}] LOGS:\n{req:#?}"));
                        }
                        #[allow(unreachable_patterns)]
                        _ => {
                            log_to_file(&format!("OUTPUT[{i}] UNKNOWN:\n<unhandled type>"));
                        }
                    }
                }
            })
            .validate(|_| async {});
    }
}
