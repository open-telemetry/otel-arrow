// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use bytes::Bytes;
use futures::StreamExt;
use futures::future::BoxFuture;
use futures::stream::FuturesUnordered;
use tokio::time::Duration;

use super::client::LogsIngestionClient;

pub struct CompletedExport {
    pub batch_id: u64,
    pub client: LogsIngestionClient,
    pub result: Result<Duration, String>,
    pub row_count: f64,
}

pub struct InFlightExports {
    futures: FuturesUnordered<BoxFuture<'static, CompletedExport>>,
    limit: usize,
}

impl InFlightExports {
    pub fn new(limit: usize) -> Self {
        Self {
            futures: FuturesUnordered::new(),
            limit,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.futures.len()
    }

    #[inline]
    pub async fn next_completion(&mut self) -> Option<CompletedExport> {
        if self.futures.is_empty() {
            // Stay pending forever when empty - prevents busy loop
            std::future::pending().await
        } else {
            self.futures.next().await
        }
    }

    /// Push a future. If at capacity, waits for one completion and returns it.
    #[inline]
    pub async fn push(
        &mut self,
        fut: BoxFuture<'static, CompletedExport>,
    ) -> Option<CompletedExport> {
        let completed = if self.futures.len() >= self.limit {
            self.futures.next().await
        } else {
            None
        };
        self.futures.push(fut);
        completed
    }

    /// Create and push an export. Returns any completed export due to backpressure.
    pub async fn push_export(
        &mut self,
        client: LogsIngestionClient,
        batch_id: u64,
        row_count: f64,
        body: Bytes,
    ) -> Option<CompletedExport> {
        let fut = Self::make_export_future(client, batch_id, row_count, body);
        self.push(fut).await
    }

    /// Create a boxed export future.
    pub fn make_export_future(
        mut client: LogsIngestionClient,
        batch_id: u64,
        row_count: f64,
        body: Bytes,
    ) -> BoxFuture<'static, CompletedExport> {
        Box::pin(async move {
            let result = client.export(body).await;
            CompletedExport {
                batch_id,
                client,
                result,
                row_count,
            }
        })
    }

    /// Drain all in-flight exports to completion.
    pub async fn drain(&mut self) -> Vec<CompletedExport> {
        let mut out = Vec::with_capacity(self.futures.len());
        while let Some(completed) = self.futures.next().await {
            out.push(completed);
        }
        out
    }
}
