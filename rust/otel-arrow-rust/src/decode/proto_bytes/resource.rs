// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Code and utilities for decoding OTAP to OTLP bytes for Resource

use arrow::array::StructArray;

pub struct ResourceProtoBytesEncoder {}

impl ResourceProtoBytesEncoder {
    pub fn new() -> Self {
        Self {}
    }

    pub fn encode(&mut self, resource_col: &StructArray, index: usize, result_buf: &mut [u8]) {
        todo!()
    }
}
