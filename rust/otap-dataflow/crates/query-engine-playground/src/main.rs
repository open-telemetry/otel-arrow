// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Interactive web playground for executing OPL programs against OpenTelemetry
//! data.
//!
//! # Running
//!
//! ```bash
//! cargo run -p otap-df-query-engine-playground
//! ```
//!
//! Then open `http://localhost:3000` in a browser.
//!
//! # Architecture
//!
//! A single-binary web server that:
//! - Serves an embedded HTML page with an inline JavaScript application
//! - Serves the OTel `.proto` schema files so the browser can parse them
//! - Exposes `POST /api/execute` which accepts an OPL query and
//!   base64-encoded protobuf data, runs the pipeline, and returns results
//!
//! The browser uses [`protobuf.js`](https://github.com/protobufjs/protobuf.js)
//! (loaded from CDN) to load the `.proto` definitions, encode user-provided
//! JSON into protobuf bytes, and decode the result protobuf back into JSON
//! for display.

#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::collections::HashMap;
use std::net::SocketAddr;

use arrow::util::pretty::pretty_format_batches;
use axum::extract::Path;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{Html, IntoResponse};
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use data_engine_parser_abstractions::Parser;
use otap_df_pdata::otap::OtapArrowRecords;
use otap_df_pdata::proto::OtlpProtoMessage;
use otap_df_pdata::proto::opentelemetry::logs::v1::LogsData;
use otap_df_pdata::proto::opentelemetry::metrics::v1::MetricsData;
use otap_df_pdata::proto::opentelemetry::trace::v1::TracesData;
use otap_df_pdata::testing::round_trip::{otap_to_otlp, otlp_to_otap};
use otap_df_query_engine::parser::default_parser_options;
use otap_df_query_engine::pipeline::Pipeline;
use otap_df_query_engine_languages::opl::parser::OplParser;
use prost::Message;
use serde::{Deserialize, Serialize};

/// The main HTML page (embedded at compile time).
const INDEX_HTML: &str = include_str!("../frontend/index.html");

// OTel `.proto` schema files embedded at compile time.  These are served to
// the browser so that `protobuf.js` can parse them and build message types for
// encoding/decoding OTLP data.  The paths must match the `import` statements
// inside the proto files (e.g. `import "opentelemetry/proto/common/v1/common.proto"`).
const PROTO_COMMON: &str = include_str!(
    "../../../../../proto/opentelemetry-proto/opentelemetry/proto/common/v1/common.proto"
);
const PROTO_RESOURCE: &str = include_str!(
    "../../../../../proto/opentelemetry-proto/opentelemetry/proto/resource/v1/resource.proto"
);
const PROTO_LOGS: &str =
    include_str!("../../../../../proto/opentelemetry-proto/opentelemetry/proto/logs/v1/logs.proto");
const PROTO_TRACE: &str = include_str!(
    "../../../../../proto/opentelemetry-proto/opentelemetry/proto/trace/v1/trace.proto"
);
const PROTO_METRICS: &str = include_str!(
    "../../../../../proto/opentelemetry-proto/opentelemetry/proto/metrics/v1/metrics.proto"
);

/// JSON body for `POST /api/execute`.
#[derive(Deserialize)]
struct ExecuteRequest {
    /// The OPL query string to execute.
    query: String,
    /// The signal type: `"logs"`, `"traces"`, or `"metrics"`.
    signal_type: String,
    /// Base64-encoded protobuf bytes of the OTLP export request.
    data: String,
}

/// JSON response from `POST /api/execute`.
#[derive(Serialize)]
struct ExecuteResponse {
    /// Base64-encoded protobuf bytes of the result, or `null` on error.
    data: Option<String>,
    /// Pretty-printed Arrow tables keyed by payload type name.
    arrow_tables: HashMap<String, String>,
    /// Error message, if any.
    error: Option<String>,
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(serve_index))
        .route("/proto/{*path}", get(serve_proto))
        .route("/api/execute", post(handle_execute));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("OPL Playground listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind");
    axum::serve(listener, app).await.expect("server error");
}

/// Serve the main HTML page.
async fn serve_index() -> Html<&'static str> {
    Html(INDEX_HTML)
}

/// Serve an embedded `.proto` file.
///
/// The path is matched against the import paths used inside the proto files
/// (e.g. `opentelemetry/proto/common/v1/common.proto`).
async fn serve_proto(Path(path): Path<String>) -> impl IntoResponse {
    let path = path.strip_prefix('/').unwrap_or(&path);
    let content = match path {
        "opentelemetry/proto/common/v1/common.proto" => Some(PROTO_COMMON),
        "opentelemetry/proto/resource/v1/resource.proto" => Some(PROTO_RESOURCE),
        "opentelemetry/proto/logs/v1/logs.proto" => Some(PROTO_LOGS),
        "opentelemetry/proto/trace/v1/trace.proto" => Some(PROTO_TRACE),
        "opentelemetry/proto/metrics/v1/metrics.proto" => Some(PROTO_METRICS),
        _ => None,
    };

    match content {
        Some(body) => (
            StatusCode::OK,
            [(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/plain; charset=utf-8"),
            )],
            body,
        )
            .into_response(),
        None => (StatusCode::NOT_FOUND, "not found").into_response(),
    }
}

/// Execute an OPL pipeline against user-provided OTLP data.
///
/// Runs the pipeline on a blocking task because the query engine uses
/// `Rc`-based internals that are not `Send`.
async fn handle_execute(Json(req): Json<ExecuteRequest>) -> Json<ExecuteResponse> {
    let result = tokio::task::spawn_blocking(move || {
        tokio::runtime::Handle::current().block_on(execute_pipeline(req))
    })
    .await;

    let resp = match result {
        Ok(Ok(resp)) => resp,
        Ok(Err(msg)) => ExecuteResponse {
            data: None,
            arrow_tables: HashMap::new(),
            error: Some(msg),
        },
        Err(e) => ExecuteResponse {
            data: None,
            arrow_tables: HashMap::new(),
            error: Some(format!("task panic: {e}")),
        },
    };
    Json(resp)
}

/// Core pipeline execution logic, returning a structured response or an error
/// message.
async fn execute_pipeline(req: ExecuteRequest) -> Result<ExecuteResponse, String> {
    // 1. Decode base64 → protobuf bytes.
    let proto_bytes = BASE64
        .decode(&req.data)
        .map_err(|e| format!("base64 decode error: {e}"))?;

    // 2. Decode protobuf → OTLP message.
    let otlp_msg = match req.signal_type.as_str() {
        "logs" => {
            let data = LogsData::decode(proto_bytes.as_slice())
                .map_err(|e| format!("protobuf decode error: {e}"))?;
            OtlpProtoMessage::Logs(data)
        }
        "traces" => {
            let data = TracesData::decode(proto_bytes.as_slice())
                .map_err(|e| format!("protobuf decode error: {e}"))?;
            OtlpProtoMessage::Traces(data)
        }
        "metrics" => {
            let data = MetricsData::decode(proto_bytes.as_slice())
                .map_err(|e| format!("protobuf decode error: {e}"))?;
            OtlpProtoMessage::Metrics(data)
        }
        other => return Err(format!("unknown signal_type: {other:?}")),
    };

    // 3. Convert OTLP → OTAP columnar format.
    let otap_batch = otlp_to_otap(&otlp_msg);

    // 4. Parse the OPL query.
    let options = default_parser_options();
    let parser_result = OplParser::parse_with_options(&req.query, options).map_err(|errors| {
        let msgs: Vec<String> = errors.iter().map(|e| format!("{e}")).collect();
        format!("parse error: {}", msgs.join("; "))
    })?;

    // 5. Execute the pipeline.
    let mut pipeline = Pipeline::new(parser_result.pipeline);
    let result = pipeline
        .execute(otap_batch)
        .await
        .map_err(|e| format!("execution error: {e}"))?;

    // 6. Build Arrow table views for each payload type.
    let arrow_tables = build_arrow_tables(&result);

    // 7. Convert result back to OTLP protobuf.
    let result_msg = otap_to_otlp(&result);
    let mut buf = Vec::new();
    result_msg
        .encode(&mut buf)
        .map_err(|e| format!("protobuf encode error: {e}"))?;
    let result_b64 = BASE64.encode(&buf);

    Ok(ExecuteResponse {
        data: Some(result_b64),
        arrow_tables,
        error: None,
    })
}

/// Pretty-print each non-empty Arrow `RecordBatch` in the result.
fn build_arrow_tables(result: &OtapArrowRecords) -> HashMap<String, String> {
    let mut tables = HashMap::new();
    for payload_type in result.allowed_payload_types() {
        if let Some(batch) = result.get(*payload_type) {
            if batch.num_rows() > 0 {
                if let Ok(formatted) = pretty_format_batches(std::slice::from_ref(batch)) {
                    let _: Option<String> = tables.insert(
                        payload_type.as_str_name().to_string(),
                        formatted.to_string(),
                    );
                }
            }
        }
    }
    tables
}
