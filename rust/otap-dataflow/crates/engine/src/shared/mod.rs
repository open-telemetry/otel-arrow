// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Traits and structs defining the shared (Send) version of receivers, processors, exporters, and extensions.

pub mod exporter;
pub mod extension;
pub mod message;
pub mod processor;
pub mod receiver;
