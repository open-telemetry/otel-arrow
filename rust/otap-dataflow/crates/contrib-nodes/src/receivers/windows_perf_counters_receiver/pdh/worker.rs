// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The thread that owns the PDH queries, its scrape requests, and shutdown.

use super::super::config::CounterConfig;
use super::query::Collector;
use super::{Pdh, Report};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use tokio::sync::oneshot;

/// How the worker ended.
#[derive(Debug, PartialEq, Eq)]
pub(in super::super) enum Stopped {
    /// The worker closed its queries and exited.
    Cleanly,
    /// The deadline passed first; the thread is left to finish on its own.
    TimedOut,
    /// The worker panicked.
    Failed,
}

/// Client for the worker thread. PDH calls block, so they never run on the async runtime.
pub(in super::super) struct Worker {
    /// Each message requests one scrape; closing the channel ends the worker's loop.
    scrapes: mpsc::Sender<oneshot::Sender<Report>>,
    /// Set on shutdown; the worker checks it before each counter's work.
    stop: Arc<AtomicBool>,
    /// Sent once the worker thread has fully exited, thread-local destructors included: `true`
    /// for a clean exit and `false` after a panic.
    exited: oneshot::Receiver<bool>,
}

impl Worker {
    /// Spawns the worker, which opens, adds, and primes the queries, and a thread that waits for
    /// it to exit. Returns at once with a receiver for the start report, so the caller can keep
    /// serving control messages.
    pub(in super::super) fn start<P, F>(
        counters: Vec<CounterConfig>,
        open: F,
    ) -> std::io::Result<(Self, oneshot::Receiver<Report>)>
    where
        P: Pdh,
        F: FnMut() -> Result<P, u32> + Send + 'static,
    {
        let (scrapes, requests) = mpsc::channel::<oneshot::Sender<Report>>();
        let (started_tx, started) = oneshot::channel();
        let (exited_tx, exited) = oneshot::channel();
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop);
        let thread = std::thread::Builder::new()
            .name("windows-perf-counters-pdh".to_owned())
            .spawn(move || {
                let (mut collector, report) = Collector::start(counters, open, worker_stop);
                let _ = started_tx.send(report);
                while let Ok(reply) = requests.recv() {
                    let _ = reply.send(collector.scrape());
                }
            })?;
        // Joining blocks, so it runs on its own thread rather than on the async runtime.
        let _ = std::thread::Builder::new()
            .name("windows-perf-counters-pdh-join".to_owned())
            .spawn(move || {
                let _ = exited_tx.send(thread.join().is_ok());
            })?;
        let worker = Self {
            scrapes,
            stop,
            exited,
        };
        Ok((worker, started))
    }

    /// Requests one scrape. The receiver errors if the worker has stopped.
    pub(in super::super) fn scrape(&self) -> oneshot::Receiver<Report> {
        let (reply, report) = oneshot::channel();
        let _ = self.scrapes.send(reply);
        report
    }

    /// Stops the worker, waiting until the deadline. A start or scrape in progress finishes the
    /// counter operation it is in and skips the remaining counters.
    pub(in super::super) async fn shutdown(self, deadline: tokio::time::Instant) -> Stopped {
        let Self {
            scrapes,
            stop,
            exited,
        } = self;
        stop.store(true, Ordering::Relaxed);
        drop(scrapes);
        match tokio::time::timeout_at(deadline, exited).await {
            Ok(Ok(true)) => Stopped::Cleanly,
            Ok(_) => Stopped::Failed,
            Err(_) => Stopped::TimedOut,
        }
    }
}
