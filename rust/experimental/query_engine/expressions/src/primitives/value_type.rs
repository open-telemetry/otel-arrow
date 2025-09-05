// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    Array,
    Boolean,
    DateTime,
    Double,
    Integer,
    Map,
    Null,
    Regex,
    String,
    TimeSpan,
}
