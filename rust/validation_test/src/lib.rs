// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Validation test module to validate the encoding/decoding process for otlp messages

// ToDo: Support transformative processors in a pipeline,
// we should be able to know when the assert equivalent will fail


pub mod encode_decode;
pub mod pipeline;