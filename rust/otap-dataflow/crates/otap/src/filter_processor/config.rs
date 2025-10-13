// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the configuration of the filter processor
//! 

use super::filter_logs::LogFilter;

pub struct Config {
    // ToDo: add metrics and spans
    logs: LogFilter,
}


