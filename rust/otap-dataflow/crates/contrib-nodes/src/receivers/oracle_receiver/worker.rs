// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bounded native work outside the pipeline's Tokio runtime.

use std::io;
use std::sync::mpsc::{SyncSender, TrySendError, sync_channel};
use tokio::sync::oneshot;

type Job<S> = Box<dyn FnOnce(&mut S) + Send>;

pub(super) struct NativeWorker<S> {
    jobs: Option<SyncSender<Job<S>>>,
    exited: oneshot::Receiver<()>,
    completion: Option<bool>,
}

impl<S: Default + Send + 'static> NativeWorker<S> {
    pub(super) fn new(name: &str) -> io::Result<Self> {
        let (jobs, requests) = sync_channel::<Job<S>>(1);
        let (exit, exited) = oneshot::channel();
        // Dropping this handle detaches the thread from runtime teardown.
        // Completion is reported only after worker-owned resources are dropped.
        let _thread = std::thread::Builder::new()
            .name(name.to_owned())
            .spawn(move || {
                let mut state = S::default();
                while let Ok(job) = requests.recv() {
                    job(&mut state);
                }
                drop(state);
                drop(requests);
                // A panic drops this sender without confirming successful cleanup.
                let _ = exit.send(());
            })?;
        Ok(Self {
            jobs: Some(jobs),
            exited,
            completion: None,
        })
    }

    pub(super) fn run<T: Send + 'static>(
        &self,
        operation: impl FnOnce(&mut S) -> T + Send + 'static,
    ) -> io::Result<oneshot::Receiver<T>> {
        let (response, result) = oneshot::channel();
        self.jobs
            .as_ref()
            .ok_or_else(stopped)?
            .try_send(Box::new(move |state| {
                let value = operation(state);
                let _ = response.send(value);
            }))
            .map_err(|error| match error {
                TrySendError::Full(_) => {
                    io::Error::new(io::ErrorKind::WouldBlock, "Oracle worker queue is full")
                }
                TrySendError::Disconnected(_) => stopped(),
            })?;
        Ok(result)
    }

    pub(super) async fn stop(&mut self) -> io::Result<()> {
        if let Some(success) = self.completion {
            return if success { Ok(()) } else { Err(stopped()) };
        }
        drop(self.jobs.take());
        let result = (&mut self.exited).await.map_err(|_| stopped());
        self.completion = Some(result.is_ok());
        result
    }
}

pub(super) async fn receive<T>(result: oneshot::Receiver<T>) -> io::Result<T> {
    result.await.map_err(|_| stopped())
}

fn stopped() -> io::Error {
    io::Error::new(
        io::ErrorKind::BrokenPipe,
        "Oracle worker stopped without confirming completion",
    )
}

#[cfg(test)]
oracle_module_tests!(worker);
