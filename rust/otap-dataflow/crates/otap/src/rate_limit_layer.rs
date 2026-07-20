// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Tower middleware that rejects gRPC requests when a receiver rate bucket is exhausted.

use crate::otlp_metrics::OtlpReceiverMetrics;
use crate::rate_limiter::RateLimiter;
use futures::future::BoxFuture;
use http::{Request, Response};
use otap_df_telemetry::metrics::MetricSet;
use parking_lot::Mutex;
use std::sync::Arc;
use std::task::{Context, Poll};
use tonic::{Code, Status, body::Body, metadata::MetadataMap};
use tower::{Layer, Service};

/// Builds a gRPC `resource_exhausted` status with retry pushback metadata.
#[must_use]
pub fn grpc_rate_limit_status(retry_after_secs: u32) -> Status {
    let mut metadata = MetadataMap::new();
    let retry_pushback_ms = u64::from(retry_after_secs.max(1)) * 1_000;
    if let Ok(value) = retry_pushback_ms.to_string().parse() {
        let _ = metadata.insert("grpc-retry-pushback-ms", value);
    }
    Status::with_metadata(Code::ResourceExhausted, "rate limit", metadata)
}

/// Layer that fails fast before concurrency limits and tonic request decoding.
#[derive(Clone)]
pub struct RateLimitLayer {
    rate_limiter: Option<RateLimiter>,
    metrics: Arc<Mutex<MetricSet<OtlpReceiverMetrics>>>,
}

impl RateLimitLayer {
    /// Creates a new layer backed by the receiver-local rate limiter.
    #[must_use]
    pub fn new(
        rate_limiter: Option<RateLimiter>,
        metrics: Arc<Mutex<MetricSet<OtlpReceiverMetrics>>>,
    ) -> Self {
        Self {
            rate_limiter,
            metrics,
        }
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimitService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitService {
            inner,
            rate_limiter: self.rate_limiter.clone(),
            metrics: self.metrics.clone(),
            reject_next_call: false,
        }
    }
}

/// Service implementation for [`RateLimitLayer`].
#[derive(Clone)]
pub struct RateLimitService<S> {
    inner: S,
    rate_limiter: Option<RateLimiter>,
    metrics: Arc<Mutex<MetricSet<OtlpReceiverMetrics>>>,
    reject_next_call: bool,
}

impl<S, ReqBody> Service<Request<ReqBody>> for RateLimitService<S>
where
    S: Service<Request<ReqBody>, Response = Response<Body>> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response<Body>;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if self
            .rate_limiter
            .as_ref()
            .is_some_and(RateLimiter::is_exhausted)
        {
            self.reject_next_call = true;
            return Poll::Ready(Ok(()));
        }
        self.reject_next_call = false;
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<ReqBody>) -> Self::Future {
        let exhausted = self.reject_next_call;
        self.reject_next_call = false;

        if exhausted {
            let retry_after_secs = self
                .rate_limiter
                .as_ref()
                .map_or(1, RateLimiter::retry_after_secs);
            let mut metrics = self.metrics.lock();
            metrics.rejected_requests.inc();
            metrics.refused_rate_limit.inc();
            let response = grpc_rate_limit_status(retry_after_secs).into_http();
            return Box::pin(async move { Ok(response) });
        }

        let future = self.inner.call(request);
        Box::pin(future)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rate_limiter::RateAdmissionDecision;
    use futures::future;
    use otap_df_config::policy::{
        RateLimitAggregation, RateLimitMode, RateLimitPolicy, RateLimitPressure, RateLimitUnit,
    };
    use otap_df_engine::memory_limiter::{
        MemoryPressureLevel, MemoryPressureState, SharedReceiverAdmissionState,
    };
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
    use std::convert::Infallible;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::task::Waker;
    use std::time::Duration;

    #[derive(Clone)]
    struct CountingService {
        poll_ready_calls: Arc<AtomicUsize>,
        call_count: Arc<AtomicUsize>,
    }

    impl CountingService {
        fn new() -> Self {
            Self {
                poll_ready_calls: Arc::new(AtomicUsize::new(0)),
                call_count: Arc::new(AtomicUsize::new(0)),
            }
        }
    }

    impl Service<Request<Body>> for CountingService {
        type Response = Response<Body>;
        type Error = Infallible;
        type Future = future::Ready<Result<Self::Response, Self::Error>>;

        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            let _ = self.poll_ready_calls.fetch_add(1, Ordering::Relaxed);
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, _request: Request<Body>) -> Self::Future {
            let _ = self.call_count.fetch_add(1, Ordering::Relaxed);
            future::ready(Ok(Response::new(Body::empty())))
        }
    }

    fn policy() -> RateLimitPolicy {
        RateLimitPolicy {
            mode: RateLimitMode::Enforce,
            aggregation: RateLimitAggregation::ReceiverInstance,
            unit: RateLimitUnit::RequestBytesPerSecond,
            allow: 10,
            interval: Duration::from_secs(1),
            burst: Some(10),
            pressure: RateLimitPressure::Soft,
        }
    }

    /// Scenario: the gRPC rate bucket is exhausted while soft pressure is active.
    /// Guarantees: rate fast-fail rejects before polling the inner concurrency-limited service.
    #[test]
    fn exhausted_rate_limit_short_circuits_before_inner_readiness_and_call() {
        let state = MemoryPressureState::default();
        let admission = SharedReceiverAdmissionState::from_process_state(&state);
        let limiter = RateLimiter::new(policy(), admission.clone());
        assert_eq!(limiter.check_units(10), RateAdmissionDecision::Admit);

        state.set_level_for_tests(MemoryPressureLevel::Soft);
        admission.apply(state.current_update(1));

        let metrics_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx =
            otap_df_engine::context::ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let metrics = Arc::new(Mutex::new(
            pipeline_ctx.register_metrics::<OtlpReceiverMetrics>(),
        ));
        let inner = CountingService::new();
        let poll_ready_calls = inner.poll_ready_calls.clone();
        let call_count = inner.call_count.clone();

        let mut service = RateLimitLayer::new(Some(limiter), metrics.clone()).layer(inner);
        let waker = Waker::noop();
        let mut cx = Context::from_waker(waker);

        assert!(matches!(service.poll_ready(&mut cx), Poll::Ready(Ok(()))));

        let response = futures::executor::block_on(service.call(Request::new(Body::empty())))
            .expect("rate limit rejection should not error");

        assert_eq!(poll_ready_calls.load(Ordering::Relaxed), 0);
        assert_eq!(call_count.load(Ordering::Relaxed), 0);
        assert_eq!(
            response
                .headers()
                .get("grpc-status")
                .and_then(|v| v.to_str().ok()),
            Some("8")
        );
        assert_eq!(
            response
                .headers()
                .get("grpc-retry-pushback-ms")
                .and_then(|v| v.to_str().ok()),
            Some("1000")
        );

        let metrics = metrics.lock();
        assert_eq!(metrics.rejected_requests.get(), 1);
        assert_eq!(metrics.refused_rate_limit.get(), 1);
    }

    /// Scenario: rate exhaustion appears after the layer has polled the inner service ready.
    /// Guarantees: the layer still calls the inner service so reserved readiness is consumed.
    #[test]
    fn exhaustion_after_inner_readiness_does_not_skip_inner_call() {
        let state = MemoryPressureState::default();
        let admission = SharedReceiverAdmissionState::from_process_state(&state);
        let limiter = RateLimiter::new(policy(), admission.clone());
        assert_eq!(limiter.check_units(10), RateAdmissionDecision::Admit);

        let metrics_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx =
            otap_df_engine::context::ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let metrics = Arc::new(Mutex::new(
            pipeline_ctx.register_metrics::<OtlpReceiverMetrics>(),
        ));
        let inner = CountingService::new();
        let poll_ready_calls = inner.poll_ready_calls.clone();
        let call_count = inner.call_count.clone();

        let mut service = RateLimitLayer::new(Some(limiter), metrics.clone()).layer(inner);
        let waker = Waker::noop();
        let mut cx = Context::from_waker(waker);

        assert!(matches!(service.poll_ready(&mut cx), Poll::Ready(Ok(()))));
        state.set_level_for_tests(MemoryPressureLevel::Soft);
        admission.apply(state.current_update(1));

        let response = futures::executor::block_on(service.call(Request::new(Body::empty())))
            .expect("ready inner service should still receive the call");

        assert_eq!(poll_ready_calls.load(Ordering::Relaxed), 1);
        assert_eq!(call_count.load(Ordering::Relaxed), 1);
        assert_eq!(
            response
                .headers()
                .get("grpc-status")
                .and_then(|v| v.to_str().ok()),
            None
        );

        let metrics = metrics.lock();
        assert_eq!(metrics.rejected_requests.get(), 0);
        assert_eq!(metrics.refused_rate_limit.get(), 0);
    }
}
