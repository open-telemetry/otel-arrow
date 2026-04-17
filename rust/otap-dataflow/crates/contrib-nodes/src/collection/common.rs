// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::io;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CollectedEvent {
    pub timestamp_unix_nano: u64,
    pub cpu: Option<u32>,
    pub pid: Option<i32>,
    pub tid: Option<i32>,
    pub payload: Vec<u8>,
    pub source: EventSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub(crate) enum EventSource {
    UserEvents(UserEventsSource),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UserEventsSource {
    pub tracepoint: String,
    pub sample_id: u64,
}

#[derive(Debug)]
pub(crate) struct CollectedDrain {
    pub events: Vec<CollectedEvent>,
    pub lost_samples: u64,
}

#[derive(Debug)]
pub(crate) enum CollectInitError {
    MissingTracepoint(String),
    InvalidTracepoint(String),
    Io(io::Error),
}

impl std::fmt::Display for CollectInitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingTracepoint(name) => write!(f, "tracepoint `{name}` is not registered"),
            Self::InvalidTracepoint(name) => write!(f, "tracepoint `{name}` is invalid"),
            Self::Io(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for CollectInitError {}
