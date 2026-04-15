// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![cfg_attr(not(target_os = "linux"), allow(dead_code, unused_imports))]

//! Session wrapper for Linux userevents collection.

#[cfg(target_os = "linux")]
mod imp {
    use std::io;
    use std::time::Duration;

    use tokio::time;

    use crate::collection::one_collect_adapter::{
        CollectInitError, EventSource, OneCollectUserEventsSession, UserEventsSessionConfig,
        UserEventsSubscription,
    };

    use super::super::{DrainConfig, SessionConfig, SubscriptionConfig};

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(crate) struct RawUsereventsRecord {
        pub tracepoint: String,
        pub timestamp_unix_nano: u64,
        pub cpu: u32,
        pub pid: i32,
        pub tid: i32,
        pub sample_id: u64,
        pub payload: Vec<u8>,
        pub payload_size: usize,
    }

    #[derive(Debug)]
    pub(crate) struct SessionDrain {
        pub records: Vec<RawUsereventsRecord>,
        pub lost_samples: u64,
    }

    #[derive(Debug)]
    pub(crate) enum SessionInitError {
        MissingTracepoint(String),
        InvalidTracepoint(String),
        Io(io::Error),
    }

    impl std::fmt::Display for SessionInitError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::MissingTracepoint(name) => {
                    write!(f, "tracepoint `{name}` is not registered")
                }
                Self::InvalidTracepoint(name) => write!(f, "tracepoint `{name}` is invalid"),
                Self::Io(err) => err.fmt(f),
            }
        }
    }

    impl std::error::Error for SessionInitError {}

    impl From<io::Error> for SessionInitError {
        fn from(value: io::Error) -> Self {
            Self::Io(value)
        }
    }

    pub(crate) struct UsereventsSession {
        inner: OneCollectUserEventsSession,
    }

    impl UsereventsSession {
        pub(crate) fn open(
            subscriptions: &[SubscriptionConfig],
            config: &SessionConfig,
            cpu_id: usize,
        ) -> Result<Self, SessionInitError> {
            let subscriptions = subscriptions
                .iter()
                .map(|subscription| UserEventsSubscription {
                    tracepoint: subscription.tracepoint.clone(),
                })
                .collect::<Vec<_>>();
            let config = UserEventsSessionConfig {
                per_cpu_buffer_size: config.per_cpu_buffer_size,
                cpu_ids: vec![cpu_id],
            };

            let inner = OneCollectUserEventsSession::open(&subscriptions, &config).map_err(
                |error| match error {
                    CollectInitError::MissingTracepoint(tracepoint) => {
                        SessionInitError::MissingTracepoint(tracepoint)
                    }
                    CollectInitError::InvalidTracepoint(tracepoint) => {
                        SessionInitError::InvalidTracepoint(tracepoint)
                    }
                    CollectInitError::Io(error) => SessionInitError::Io(error),
                },
            )?;

            Ok(Self { inner })
        }

        pub(crate) async fn drain_ready(
            &mut self,
            config: &DrainConfig,
        ) -> io::Result<SessionDrain> {
            let poll_interval = Duration::from_millis(1);

            loop {
                let drained = self.inner.drain(
                    config.max_records_per_turn,
                    config.max_bytes_per_turn,
                    config.max_drain_ns,
                )?;
                if !drained.events.is_empty() || drained.lost_samples > 0 {
                    let records = drained
                        .events
                        .into_iter()
                        .filter_map(|event| match event.source {
                            EventSource::UserEvents(source) => Some(RawUsereventsRecord {
                                tracepoint: source.tracepoint,
                                timestamp_unix_nano: event.timestamp_unix_nano,
                                cpu: event.cpu.unwrap_or_default(),
                                pid: event.pid.unwrap_or_default(),
                                tid: event.tid.unwrap_or_default(),
                                sample_id: source.sample_id,
                                payload_size: event.payload.len(),
                                payload: event.payload,
                            }),
                        })
                        .collect();
                    return Ok(SessionDrain {
                        records,
                        lost_samples: drained.lost_samples,
                    });
                }

                time::sleep(poll_interval).await;
            }
        }

        pub(crate) fn subscription_count(&self) -> usize {
            self.inner.subscription_count()
        }
    }
}

#[cfg(not(target_os = "linux"))]
mod imp {
    use std::io;

    use super::super::{DrainConfig, SessionConfig, SubscriptionConfig};

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(crate) struct RawUsereventsRecord {
        pub tracepoint: String,
        pub timestamp_unix_nano: u64,
        pub cpu: u32,
        pub pid: i32,
        pub tid: i32,
        pub sample_id: u64,
        pub payload: Vec<u8>,
        pub payload_size: usize,
    }

    #[derive(Debug)]
    pub(crate) struct SessionDrain {
        pub records: Vec<RawUsereventsRecord>,
        pub lost_samples: u64,
    }

    #[derive(Debug)]
    pub(crate) enum SessionInitError {
        Unsupported,
    }

    impl std::fmt::Display for SessionInitError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Unsupported => write!(f, "userevents sessions are supported only on Linux"),
            }
        }
    }

    impl std::error::Error for SessionInitError {}

    pub(crate) struct UsereventsSession;

    impl UsereventsSession {
        pub(crate) fn open(
            _subscriptions: &[SubscriptionConfig],
            _config: &SessionConfig,
            _cpu_id: usize,
        ) -> Result<Self, SessionInitError> {
            Err(SessionInitError::Unsupported)
        }

        pub(crate) async fn drain_ready(
            &mut self,
            _config: &DrainConfig,
        ) -> io::Result<SessionDrain> {
            Ok(SessionDrain {
                records: Vec::new(),
                lost_samples: 0,
            })
        }

        pub(crate) fn subscription_count(&self) -> usize {
            0
        }
    }
}

pub(super) use imp::{RawUsereventsRecord, SessionInitError, UsereventsSession};
