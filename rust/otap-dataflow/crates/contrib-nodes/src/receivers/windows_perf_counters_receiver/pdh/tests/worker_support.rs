// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

impl QueryCadence {
    pub(in super::super) fn uniform(cadence: Duration) -> Self {
        Self::new(cadence, cadence)
    }
}

impl Worker {
    pub(in super::super) async fn start_with<A, F>(
        counters: Vec<CounterConfig>,
        query_retry_cap: Duration,
        max_native_counters: usize,
        mut open: F,
    ) -> Result<Self, Error>
    where
        A: PdhApi + 'static,
        F: QueryFactory<A> + Send + 'static,
    {
        Self::start_controlled(
            counters,
            query_retry_cap,
            max_native_counters,
            "test".to_owned(),
            move |configs, limit, seed, _shutdown| open(configs, limit, seed),
        )
        .await
    }
}

impl<A: PdhApi> QuerySlot<A> {
    pub(in super::super) fn open_initial<F>(
        configs: Vec<CounterConfig>,
        retry_cap: Duration,
        max_native_counters: usize,
        open: &mut F,
    ) -> Result<Self, Error>
    where
        F: QueryFactory<A>,
    {
        open_initial_query(
            configs,
            retry_cap,
            max_native_counters,
            open,
            &AtomicBool::new(false),
        )?
        .ok_or(Error::WorkerStopRequested)
    }
}
