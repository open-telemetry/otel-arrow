// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Traits and structs defining the local (!Send) version of receivers, processors, processor_chains and exporters.

pub mod exporter;
pub mod message;
pub mod processor;
pub mod processor_chain;
pub mod receiver;
