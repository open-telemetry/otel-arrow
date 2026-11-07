// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration and validation for HTTP clients and servers.

mod client;
mod server;

pub use client::{HttpClientError, HttpClientSettings, validate_http_header_name};
pub use server::{HttpServerSettings, tune_max_concurrent_requests};
