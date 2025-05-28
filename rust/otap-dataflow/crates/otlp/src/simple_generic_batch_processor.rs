use crate::OTLPData;
use crate::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
use crate::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
use crate::proto::opentelemetry::collector::trace::v1::ExportTraceServiceRequest;
use crate::proto::opentelemetry::logs::v1::{LogRecord, ResourceLogs, ScopeLogs};
use crate::proto::opentelemetry::metrics::v1::{Metric, ResourceMetrics, ScopeMetrics};
use crate::proto::opentelemetry::trace::v1::{ResourceSpans, ScopeSpans, Span};
use async_trait::async_trait;
use otap_df_engine::error::Error;
use otap_df_engine::local::processor::{EffectHandler, Processor};
use otap_df_engine::message::{ControlMsg, Message};
use std::time::{Duration, Instant};

/// Trait for hierarchical batch splitting
///
/// This trait is used to split a batch into a vector of smaller batches, each with at most `max_batch_size`
/// leaf items, preserving all resource/scope/leaf (span/metric/logrecord) structure.
pub trait HierarchicalBatchSplit: Sized {
    /// Splits a batch into a vector of smaller batches, each with at most `max_batch_size`
    /// leaf items, preserving all resource/scope/leaf (span/metric/logrecord) structure.
    fn split_into_batches(&self, max_batch_size: usize) -> Vec<Self>;
}

impl HierarchicalBatchSplit for ExportTraceServiceRequest {
    fn split_into_batches(&self, max_batch_size: usize) -> Vec<Self> {
        let mut batches = Vec::new();
        let mut current_batch = ExportTraceServiceRequest {
            resource_spans: Vec::new(),
        };
        let mut current_span_count = 0;

        // Cursor position if we have to split mid-group
        let mut rs_idx = 0;
        while rs_idx < self.resource_spans.len() {
            let rs = &self.resource_spans[rs_idx];
            let mut res = ResourceSpans {
                resource: rs.resource.clone(),
                scope_spans: Vec::new(),
                schema_url: rs.schema_url.clone(),
            };
            let mut ss_idx = 0;
            while ss_idx < rs.scope_spans.len() {
                let ss = &rs.scope_spans[ss_idx];
                let mut sc = ScopeSpans {
                    scope: ss.scope.clone(),
                    spans: Vec::new(),
                    schema_url: ss.schema_url.clone(),
                };
                let mut span_idx = 0;
                while span_idx < ss.spans.len() {
                    let remaining = max_batch_size - current_span_count;
                    let span_left = ss.spans.len() - span_idx;
                    let take = remaining.min(span_left);

                    sc.spans
                        .extend(ss.spans[span_idx..span_idx + take].iter().cloned());
                    current_span_count += take;
                    span_idx += take;

                    if !sc.spans.is_empty() {
                        res.scope_spans.push(sc.clone());
                        sc.spans.clear();
                    }

                    if current_span_count == max_batch_size {
                        if !res.scope_spans.is_empty() {
                            current_batch.resource_spans.push(res.clone());
                            res.scope_spans.clear();
                        }
                        batches.push(current_batch.clone());
                        current_batch = ExportTraceServiceRequest {
                            resource_spans: Vec::new(),
                        };
                        current_span_count = 0;
                    }
                }
                ss_idx += 1;
            }
            if !res.scope_spans.is_empty() {
                current_batch.resource_spans.push(res.clone());
            }
            rs_idx += 1;
        }
        if current_span_count > 0 && !current_batch.resource_spans.is_empty() {
            batches.push(current_batch);
        }
        batches
    }
}

impl HierarchicalBatchSplit for ExportMetricsServiceRequest {
    fn split_into_batches(&self, max_batch_size: usize) -> Vec<Self> {
        let mut batches = Vec::new();
        let mut current_batch = ExportMetricsServiceRequest {
            resource_metrics: Vec::new(),
        };
        let mut current_metric_count = 0;

        let mut rm_idx = 0;
        while rm_idx < self.resource_metrics.len() {
            let rm = &self.resource_metrics[rm_idx];
            let mut res = ResourceMetrics {
                resource: rm.resource.clone(),
                scope_metrics: Vec::new(),
                schema_url: rm.schema_url.clone(),
            };
            let mut sm_idx = 0;
            while sm_idx < rm.scope_metrics.len() {
                let sm = &rm.scope_metrics[sm_idx];
                let mut sc = ScopeMetrics {
                    scope: sm.scope.clone(),
                    metrics: Vec::new(),
                    schema_url: sm.schema_url.clone(),
                };
                let mut m_idx = 0;
                while m_idx < sm.metrics.len() {
                    let remaining = max_batch_size - current_metric_count;
                    let m_left = sm.metrics.len() - m_idx;
                    let take = remaining.min(m_left);

                    sc.metrics
                        .extend(sm.metrics[m_idx..m_idx + take].iter().cloned());
                    current_metric_count += take;
                    m_idx += take;

                    if !sc.metrics.is_empty() {
                        res.scope_metrics.push(sc.clone());
                        sc.metrics.clear();
                    }

                    if current_metric_count == max_batch_size {
                        if !res.scope_metrics.is_empty() {
                            current_batch.resource_metrics.push(res.clone());
                            res.scope_metrics.clear();
                        }
                        batches.push(current_batch.clone());
                        current_batch = ExportMetricsServiceRequest {
                            resource_metrics: Vec::new(),
                        };
                        current_metric_count = 0;
                    }
                }
                sm_idx += 1;
            }
            if !res.scope_metrics.is_empty() {
                current_batch.resource_metrics.push(res.clone());
            }
            rm_idx += 1;
        }
        if current_metric_count > 0 && !current_batch.resource_metrics.is_empty() {
            batches.push(current_batch);
        }
        batches
    }
}

impl HierarchicalBatchSplit for ExportLogsServiceRequest {
    fn split_into_batches(&self, max_batch_size: usize) -> Vec<Self> {
        let mut batches = Vec::new();
        let mut current_batch = ExportLogsServiceRequest {
            resource_logs: Vec::new(),
        };
        let mut current_log_count = 0;

        let mut rl_idx = 0;
        while rl_idx < self.resource_logs.len() {
            let rl = &self.resource_logs[rl_idx];
            let mut res = ResourceLogs {
                resource: rl.resource.clone(),
                scope_logs: Vec::new(),
                schema_url: rl.schema_url.clone(),
            };
            let mut sl_idx = 0;
            while sl_idx < rl.scope_logs.len() {
                let sl = &rl.scope_logs[sl_idx];
                let mut sc = ScopeLogs {
                    scope: sl.scope.clone(),
                    log_records: Vec::new(),
                    schema_url: sl.schema_url.clone(),
                };
                let mut rec_idx = 0;
                while rec_idx < sl.log_records.len() {
                    let remaining = max_batch_size - current_log_count;
                    let rec_left = sl.log_records.len() - rec_idx;
                    let take = remaining.min(rec_left);

                    sc.log_records
                        .extend(sl.log_records[rec_idx..rec_idx + take].iter().cloned());
                    current_log_count += take;
                    rec_idx += take;

                    if !sc.log_records.is_empty() {
                        res.scope_logs.push(sc.clone());
                        sc.log_records.clear();
                    }

                    if current_log_count == max_batch_size {
                        if !res.scope_logs.is_empty() {
                            current_batch.resource_logs.push(res.clone());
                            res.scope_logs.clear();
                        }
                        batches.push(current_batch.clone());
                        current_batch = ExportLogsServiceRequest {
                            resource_logs: Vec::new(),
                        };
                        current_log_count = 0;
                    }
                }
                sl_idx += 1;
            }
            if !res.scope_logs.is_empty() {
                current_batch.resource_logs.push(res.clone());
            }
            rl_idx += 1;
        }
        if current_log_count > 0 && !current_batch.resource_logs.is_empty() {
            batches.push(current_batch);
        }
        batches
    }
}

// Generic batch config
#[derive(Debug, Clone)]
pub struct BatchConfig {
    pub send_batch_size: usize,
    pub timeout: Duration,
}

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
                        let mut batches = req.split_into_batches(self.config.send_batch_size);
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
                        let mut batches = req.split_into_batches(self.config.send_batch_size);
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
                        let mut batches = req.split_into_batches(self.config.send_batch_size);
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

    fn wrap_local<P>(processor: P) -> ProcessorWrapper<OTLPData>
    where
        P: Processor<OTLPData> + 'static,
    {
        let config = ProcessorConfig::new("simple_generic_batch_processor_test");
        ProcessorWrapper::local(processor, &config)
    }

    #[test]
    fn traces_group_preserving_split() {
        let mut runtime = TestRuntime::<OTLPData>::new();
        let config = BatchConfig {
            send_batch_size: 3,
            timeout: Duration::from_secs(60),
        };
        let wrapper = wrap_local(GenericBatcher::new(config));
        runtime
            .set_processor(wrapper)
            .run_test(|mut ctx| async move {
                // Compose a request with 2 resource groups, each with 2 scope groups, each with 2 spans
                let mut req = ExportTraceServiceRequest {
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
        let mut runtime = TestRuntime::<OTLPData>::new();
        let config = BatchConfig {
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
        let mut runtime = TestRuntime::<OTLPData>::new();
        let config = BatchConfig {
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
        let mut runtime = TestRuntime::<OTLPData>::new();
        let wrapper = wrap_local(GenericBatcher::new(BatchConfig { send_batch_size: 10, timeout: Duration::from_secs(1) }));
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
        let mut runtime = TestRuntime::<OTLPData>::new();
        let wrapper = wrap_local(GenericBatcher::new(BatchConfig { send_batch_size: 3, timeout: Duration::from_secs(1) }));
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
}