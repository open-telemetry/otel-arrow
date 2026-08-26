// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Debug endpoints.
//!
//! - /api/v1/debug/pprof/heap -- dump profile of unfreed heap allocations.
//!   Returns pprof-format data when jemalloc profiling is available, or
//!   HTTP 500 when the feature is not compiled in or profiling is not enabled.
//!   Concurrent dumps are rejected with HTTP 429 to keep health checks and
//!   shutdown responsive.

use axum::Router;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::Response;
use axum::routing::get;

#[cfg(all(feature = "jemalloc-pprof", not(windows)))]
use std::panic::AssertUnwindSafe;

#[cfg(not(windows))]
use axum::{body::Body, http::header};

use crate::AppState;

pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route("/debug/pprof/heap", get(get_heap_profile))
        .route("/debug/pprof/profile", get(get_cpu_profile))
}

async fn get_heap_profile(State(state): State<AppState>) -> Result<Response, (StatusCode, String)> {
    #[cfg(all(feature = "jemalloc-pprof", not(windows)))]
    {
        // Only one heap dump at a time -- reject excess requests immediately
        // so health checks and shutdown remain responsive.
        let permit = state
            .heap_profile_permits
            .clone()
            .try_acquire_owned()
            .map_err(|_| {
                (
                    StatusCode::TOO_MANY_REQUESTS,
                    "A heap profile dump is already in progress".into(),
                )
            })?;

        // Accessing `PROF_CTL` can panic if jemalloc is not the active
        // allocator or was compiled without the profiling feature. Catch
        // this panic so we return a proper HTTP error instead.
        let prof_ctl =
            std::panic::catch_unwind(AssertUnwindSafe(|| jemalloc_pprof::PROF_CTL.as_ref()));
        let prof_ctl = match prof_ctl {
            Ok(Some(ctl)) => ctl.clone(),
            Ok(None) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Memory Profiling not activated".into(),
                ));
            }
            Err(_) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Heap profiling not available \
                     (jemalloc with profiling support is not the active allocator)."
                        .into(),
                ));
            }
        };

        // Offload the blocking native dump to a dedicated thread so the
        // admin server's async runtime stays responsive.
        let pprof = tokio::task::spawn_blocking(move || {
            let _permit = permit;
            let mut guard = prof_ctl.blocking_lock();
            if !guard.activated() {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Memory Profiling not activated".into(),
                ));
            }
            guard.dump_pprof().map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Could not dump heap pprof: {e}"),
                )
            })
        })
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Heap profile task failed to join: {e}"),
            )
        })??;

        let body = Body::from(pprof);
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/x-protobuf")
            .body(body)
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Could not build heap pprof response: {e}"),
                )
            })
    }

    #[cfg(not(all(feature = "jemalloc-pprof", not(windows))))]
    {
        // Suppress dead-code warning for the semaphore field when the
        // jemalloc-pprof feature is not compiled in.
        let _ = &state.heap_profile_permits;
        Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Heap profiling is not available in this build".into(),
        ))
    }
}

/// Querystring parameters for /profile endpoint
#[derive(serde::Deserialize)]
struct CpuProfileParams {
    /// How long (in seconds) to sample CPU. If the parameter is not specified, the default will
    /// be 30 seconds
    seconds: Option<u16>,

    /// profile sampling frequency. Default = 100 (sample each 10ms)
    frequency: Option<u16>,
}

async fn get_cpu_profile(
    State(state): State<AppState>,
    Query(params): Query<CpuProfileParams>,
) -> Result<Response, (StatusCode, String)> {
    #[cfg(not(windows))]
    {
        use pprof::protos::Message;
        use std::time::Duration;

        let seconds = params.seconds.unwrap_or(30);
        if seconds == 0 {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                "seconds must be greater than 0".into(),
            ));
        }
        let frequency = params.frequency.unwrap_or(100);
        if frequency == 0 {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                "frequency must be greater than 0".into(),
            ));
        }

        let permit = state
            .cpu_profile_permits
            .clone()
            .try_acquire_owned()
            .map_err(|_| {
                (
                    StatusCode::TOO_MANY_REQUESTS,
                    "A cpu profile dump is already in progress".into(),
                )
            })?;

        // start profile
        let profile_builder = pprof::ProfilerGuardBuilder::default().frequency(frequency as i32);
        let guard = profile_builder.build().map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Could not build profiler: {e}"),
            )
        })?;

        // sleep for profile duration
        let profile_time = Duration::from_secs(seconds as u64);
        tokio::time::sleep(profile_time).await;

        // offload the blocking generation of the profile to a dedicated thread so that the
        // admin server's async runtime stays responsive.
        let pprof = tokio::task::spawn_blocking(move || {
            let _permit = permit;
            // finish profiling
            let report = guard.report().build().map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Could not build profile report: {e}"),
                )
            })?;

            // encode profile as proto-encoded pprof
            let pprof = match report.pprof() {
                Ok(pprof) => pprof.encode_to_vec(),
                Err(e) => {
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Could not proto-encode profile report: {e}"),
                    ));
                }
            };

            Ok(pprof)
        })
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Heap profile task failed to join: {e}"),
            )
        })??;

        // return response
        let body = Body::from(pprof);
        let resp = Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/x-protobuf")
            .body(body)
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Could not dump profile pprof: {e}"),
                )
            })?;

        Ok(resp)
    }

    #[cfg(not(not(windows)))]
    {
        // Suppress dead-code warning for the unused fields when the feature is not compiled in.
        let _ = &state.cpu_profile_permits;
        let _ = params.seconds;
        let _ = params.frequency;
        Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "CPU profiling is not available in this build".into(),
        ))
    }
}
