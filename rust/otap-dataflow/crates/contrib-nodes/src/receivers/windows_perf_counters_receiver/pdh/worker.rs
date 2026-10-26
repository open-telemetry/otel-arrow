// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The thread that owns the PDH query, its command channel, and shutdown.

use super::super::config::CounterConfig;
use super::query::Collector;
use super::{Pdh, Report};
use std::sync::mpsc;
use std::thread::JoinHandle;
use tokio::sync::oneshot;

enum Command {
    Scrape(oneshot::Sender<Report>),
    Stop,
}

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
    commands: mpsc::Sender<Command>,
    done: oneshot::Receiver<()>,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    /// Spawns the worker, which opens, adds, and primes the queries. Returns at once with a
    /// receiver for the start report, so the caller can keep serving control messages.
    pub(in super::super) fn start<P, F>(
        counters: Vec<CounterConfig>,
        open: F,
    ) -> std::io::Result<(Self, oneshot::Receiver<Report>)>
    where
        P: Pdh,
        F: FnMut() -> Result<P, u32> + Send + 'static,
    {
        let (commands, receiver) = mpsc::channel();
        let (started_tx, started) = oneshot::channel();
        let (done_tx, done) = oneshot::channel();
        let thread = std::thread::Builder::new()
            .name("windows-perf-counters-pdh".to_owned())
            .spawn(move || {
                let (mut collector, report) = Collector::start(counters, open);
                let _ = started_tx.send(report);
                while let Ok(Command::Scrape(reply)) = receiver.recv() {
                    let _ = reply.send(collector.scrape());
                }
                // Close the queries before reporting completion.
                drop(collector);
                let _ = done_tx.send(());
            })?;
        let worker = Self {
            commands,
            done,
            thread: Some(thread),
        };
        Ok((worker, started))
    }

    /// Requests one scrape. The receiver errors if the worker has stopped.
    pub(in super::super) fn scrape(&self) -> oneshot::Receiver<Report> {
        let (reply, report) = oneshot::channel();
        let _ = self.commands.send(Command::Scrape(reply));
        report
    }

    /// Stops the worker after any start or scrape in progress, waiting until the deadline.
    pub(in super::super) async fn shutdown(mut self, deadline: tokio::time::Instant) -> Stopped {
        let _ = self.commands.send(Command::Stop);
        let completed = match tokio::time::timeout_at(deadline, &mut self.done).await {
            Err(_) => return Stopped::TimedOut,
            Ok(result) => result.is_ok(),
        };
        // The thread has finished or panicked, so this join returns promptly.
        let joined = self
            .thread
            .take()
            .is_none_or(|thread| thread.join().is_ok());
        if completed && joined {
            Stopped::Cleanly
        } else {
            Stopped::Failed
        }
    }
}
