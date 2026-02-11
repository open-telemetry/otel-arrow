// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Attribute validation helpers.
//!
//! Given `&[OtlpProtoMessage]`, verify that certain attribute keys (or key/value
//! pairs) appear for each attribute list or do **not** appear within configured domains (resource,
//! scope, or the signal itself).
