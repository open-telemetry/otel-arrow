// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Tower middleware that rejects requests while hard memory pressure is active.

use futures::future::BoxFuture;
use http::{Request, Response};
use otap_df_engine::memory_limiter::SharedReceiverAdmissionState;
use otap_df_telemetry::metrics::MetricSet;
use parking_lot::Mutex;
use std::sync::Arc;
use std::task::{Context, Poll};
use tonic::{Code, Status, body::Body, metadata::MetadataMap};
use tower::{Layer, Service};

use crate::otlp_metrics::OtlpReceiverMetrics;

/// Records request rejections caused by process-wide hard memory pressure.
pub trait MemoryPressureRejectionMetrics: Send + Sync {
    /// Records one request rejected before entering the pipeline due to hard memory pressure.
    fn record_memory_pressure_rejection(&self);
}

/// Builds a gRPC `resource_exhausted` status with retry pushback metadata.
#[must_use]
pub fn grpc_memory_pressure_status(state: &SharedReceiverAdmissionState) -> Status {
    let mut metadata = MetadataMap::new();
    let retry_pushback_ms = u64::from(state.retry_after_secs().max(1)) * 1_000;
    let _ = metadata.insert(
        "grpc-retry-pushback-ms",
        retry_pushback_ms
            .to_string()
            .parse()
            .expect("retry pushback metadata should be valid ASCII"),
    );
    Status::with_metadata(Code::ResourceExhausted, "memory pressure", metadata)
}

impl MemoryPressureRejectionMetrics for Mutex<MetricSet<OtlpReceiverMetrics>> {
    fn record_memory_pressure_rejection(&self) {
        let mut metrics = self.lock();
        metrics.rejected_requests.inc();
        metrics.refused_memory_pressure.inc();
    }
}

/// Layer that fails fast with `resource_exhausted` before tonic decodes request bodies.
///
/// This is only enforced at `Hard` pressure. `Soft` remains advisory in the
/// process-wide state machine for this Phase 1 implementation.
#[derive(Clone)]
pub struct MemoryPressureLayer {
    state: SharedReceiverAdmissionState,
    metrics: Option<Arc<dyn MemoryPressureRejectionMetrics>>,
}

impl MemoryPressureLayer {
    /// Creates a new layer backed by the shared process-wide memory pressure state.
    #[must_use]
    pub const fn new(state: SharedReceiverAdmissionState) -> Self {
        Self {
            state,
            metrics: None,
        }
    }

    /// Creates a new layer that also records dedicated rejection metrics.
    #[must_use]
    pub fn with_metrics<M>(state: SharedReceiverAdmissionState, metrics: Arc<M>) -> Self
    where
        M: MemoryPressureRejectionMetrics + 'static,
    {
        Self {
            state,
            metrics: Some(metrics),
        }
    }

    /// Creates a new layer that also records dedicated OTLP rejection metrics.
    #[must_use]
    pub fn with_otlp_metrics(
        state: SharedReceiverAdmissionState,
        metrics: Arc<Mutex<MetricSet<OtlpReceiverMetrics>>>,
    ) -> Self {
        Self::with_metrics(state, metrics)
    }
}

impl<S> Layer<S> for MemoryPressureLayer {
    type Service = MemoryPressureService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        MemoryPressureService {
            inner,
            state: self.state.clone(),
            metrics: self.metrics.clone(),
            reject_next_call: false,
        }
    }
}

/// Service implementation for [`MemoryPressureLayer`].
#[derive(Clone)]
pub struct MemoryPressureService<S> {
    inner: S,
    state: SharedReceiverAdmissionState,
    metrics: Option<Arc<dyn MemoryPressureRejectionMetrics>>,
    reject_next_call: bool,
}

impl<S, ReqBody> Service<Request<ReqBody>> for MemoryPressureService<S>
where
    S: Service<Request<ReqBody>, Response = Response<Body>> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response<Body>;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if self.state.should_shed_ingress() {
            self.reject_next_call = true;
            return Poll::Ready(Ok(()));
        }
        self.reject_next_call = false;
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<ReqBody>) -> Self::Future {
        if self.reject_next_call || self.state.should_shed_ingress() {
            self.reject_next_call = false;
            if let Some(metrics) = &self.metrics {
                metrics.record_memory_pressure_rejection();
            }
            let response = grpc_memory_pressure_status(&self.state).into_http();
            return Box::pin(async move { Ok(response) });
        }

        let future = self.inner.call(request);
        Box::pin(future)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::{Request, Response, StatusCode};
    use otap_df_config::policy::MemoryLimiterMode;
    use otap_df_engine::memory_limiter::{MemoryPressureBehaviorConfig, MemoryPressureState};
    use std::convert::Infallible;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::task::{Context, Poll, Waker};

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
        type Future = futures::future::Ready<Result<Self::Response, Self::Error>>;

        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            let _ = self.poll_ready_calls.fetch_add(1, Ordering::Relaxed);
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, _request: Request<Body>) -> Self::Future {
            let _ = self.call_count.fetch_add(1, Ordering::Relaxed);
            futures::future::ready(Ok(Response::new(Body::empty())))
        }
    }

    #[test]
    fn hard_pressure_short_circuits_before_inner_readiness_and_call() {
        let state = MemoryPressureState::default();
        state.set_level_for_tests(otap_df_engine::memory_limiter::MemoryPressureLevel::Hard);
        state.configure(MemoryPressureBehaviorConfig {
            retry_after_secs: 3,
            fail_readiness_on_hard: true,
            mode: MemoryLimiterMode::Enforce,
        });

        let inner = CountingService::new();
        let poll_ready_calls = inner.poll_ready_calls.clone();
        let call_count = inner.call_count.clone();

        let mut service =
            MemoryPressureLayer::new(SharedReceiverAdmissionState::from_process_state(&state))
                .layer(inner);
        let waker = Waker::noop();
        let mut cx = Context::from_waker(waker);

        assert!(matches!(service.poll_ready(&mut cx), Poll::Ready(Ok(()))));

        let response = futures::executor::block_on(service.call(Request::new(Body::empty())))
            .expect("memory pressure rejection should not error");

        assert_eq!(poll_ready_calls.load(Ordering::Relaxed), 0);
        assert_eq!(call_count.load(Ordering::Relaxed), 0);
        assert_eq!(response.status(), StatusCode::OK);
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
            Some("3000")
        );
    }

    #[test]
    fn hard_rejection_decision_from_poll_ready_is_sticky_for_the_following_call() {
        let state = MemoryPressureState::default();
        state.set_level_for_tests(otap_df_engine::memory_limiter::MemoryPressureLevel::Hard);

        let inner = CountingService::new();
        let poll_ready_calls = inner.poll_ready_calls.clone();
        let call_count = inner.call_count.clone();

        let mut service =
            MemoryPressureLayer::new(SharedReceiverAdmissionState::from_process_state(&state))
                .layer(inner);
        let waker = Waker::noop();
        let mut cx = Context::from_waker(waker);

        assert!(matches!(service.poll_ready(&mut cx), Poll::Ready(Ok(()))));
        state.set_level_for_tests(otap_df_engine::memory_limiter::MemoryPressureLevel::Normal);

        let response = futures::executor::block_on(service.call(Request::new(Body::empty())))
            .expect("memory pressure rejection should not error");

        assert_eq!(poll_ready_calls.load(Ordering::Relaxed), 0);
        assert_eq!(call_count.load(Ordering::Relaxed), 0);
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn soft_pressure_remains_advisory() {
        let state = MemoryPressureState::default();
        state.set_level_for_tests(otap_df_engine::memory_limiter::MemoryPressureLevel::Soft);

        let inner = CountingService::new();
        let poll_ready_calls = inner.poll_ready_calls.clone();
        let call_count = inner.call_count.clone();

        let mut service =
            MemoryPressureLayer::new(SharedReceiverAdmissionState::from_process_state(&state))
                .layer(inner);
        let waker = Waker::noop();
        let mut cx = Context::from_waker(waker);

        assert!(matches!(service.poll_ready(&mut cx), Poll::Ready(Ok(()))));

        let response = futures::executor::block_on(service.call(Request::new(Body::empty())))
            .expect("soft pressure should not error");

        assert_eq!(poll_ready_calls.load(Ordering::Relaxed), 1);
        assert_eq!(call_count.load(Ordering::Relaxed), 1);
        assert_eq!(response.status(), StatusCode::OK);
    }
}
