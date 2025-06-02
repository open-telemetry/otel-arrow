#![deny(
    trivial_numeric_casts,
    missing_docs,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications,
    unused_extern_crates,
    unused_results
)]
#![warn(rust_2021_compatibility, unreachable_pub)]

//! A library for working with tasks.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use futures::stream::FuturesUnordered;
use tokio::task::JoinHandle;
use tokio_stream::StreamExt;
use tracing::{error, warn};

use crate::labels::{ProcessLabels, TaskLabels};

pub mod labels;

/// A join handle for a task.
pub type JoinHandleTask = JoinHandle<Box<dyn TaskCleaner>>;

/// All the errors of this crate.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {}

/// A trait for tasks that can be cleaned up.
#[async_trait]
pub trait TaskCleaner: std::fmt::Debug + Send {
    /// Method called when the task is cleaned up.
    async fn cleanup(&self);
}

/// A task manager.
#[derive(Clone, Default)]
#[must_use]
pub struct TaskManager {
    /// The process labels.
    process_labels: ProcessLabels,
    /// The default channel size.
    default_channel_size: usize,
    /// All the tasks registered with this manager.
    registered_tasks: Arc<Mutex<FuturesUnordered<JoinHandleTask>>>,
    /// All the pending tasks.
    pending_registrations: Arc<Mutex<Vec<JoinHandleTask>>>,
}

impl TaskManager {
    /// Creates a new task manager.
    pub fn new() -> Self {
        Self {
            process_labels: Default::default(),
            default_channel_size: 1000,
            registered_tasks: Arc::new(Mutex::new(FuturesUnordered::new())),
            pending_registrations: Arc::new(Mutex::new(vec![])),
        }
    }

    /// Creates a new task manager.
    pub fn with_process_labels(process_labels: ProcessLabels) -> Self {
        Self {
            process_labels,
            default_channel_size: 1000,
            registered_tasks: Arc::new(Mutex::new(FuturesUnordered::new())),
            pending_registrations: Arc::new(Mutex::new(vec![])),
        }
    }

    /// Returns the process labels.
    pub fn process_labels(&self) -> ProcessLabels {
        self.process_labels.clone()
    }

    /// Sets the process labels.
    pub fn set_process_labels(&mut self, process_labels: ProcessLabels) {
        self.process_labels = process_labels;
    }

    /// Returns the default channel size.
    pub fn default_channel_size(mut self, channel_size: usize) -> Self {
        self.default_channel_size = channel_size;
        self
    }

    /// Registers a task.
    pub fn register(&mut self, join_handle: JoinHandle<Box<dyn TaskCleaner>>, task_labels: &TaskLabels) {
        // To authorize registration of new tasks after the call to the join method.
        if let Ok(mutex) = self.registered_tasks.try_lock() {
            mutex.push(join_handle);
        } else {
            self.pending_registrations
                .lock()
                .expect("unexpected pending_registrations lock issue")
                .push(join_handle);
        }

        tracing::info!(
            task_id=%task_labels.task_id,
            task_cat=%task_labels.task_cat,
            task_source=%task_labels.task_source,
            process_id=%self.process_labels.process_id,
            "task registered and started");
    }

    /// Joins all the tasks registered with this manager.
    pub async fn join(&self) {
        let mut task_cleaned_up = 0;

        tracing::info!(
            process_id=%self.process_labels.process_id,
            task_count=%self.registered_tasks.lock().expect("registered_tasks lock failed").len(),
            "joining registered tasks");
        
        #[allow(clippy::await_holding_lock)]
        while let Some(item) = self
            .registered_tasks
            .lock()
            .expect("unexpected tasks lock issue")
            .next()
            .await
        {
            match item {
                Err(err) => error!(error=?err, "join error"),
                Ok(cleaner) => {
                    task_cleaned_up += 1;
                    cleaner.cleanup().await;
                    tracing::trace!(%task_cleaned_up, "cleaned up task");
                }
            }

            tracing::trace!(
                process_id=%self.process_labels.process_id,
                task_count=%self.pending_registrations.lock().expect("pending_registrations lock failed").len(),
                "joining pending registration tasks"
            );

            for join_handle in self
                .pending_registrations
                .lock()
                .expect("unexpected pending_registration lock issue")
                .drain(..)
            {
                self.registered_tasks
                    .lock()
                    .expect("unexpected tasks lock issue")
                    .push(join_handle);
            }
        }
    }

    /// Returns an empty task cleaner.
    pub fn no_task_cleaner(process_labels: ProcessLabels, task_labels: TaskLabels) -> Box<dyn TaskCleaner> {
        Box::new(NoOpTaskCleaner {
            process_labels,
            task_labels,
        })
    }
}

/// A no-op task cleaner.
#[derive(Debug)]
pub struct NoOpTaskCleaner {
    /// The process labels.
    process_labels: ProcessLabels,
    /// The task labels.
    task_labels: TaskLabels,
}

#[async_trait]
impl TaskCleaner for NoOpTaskCleaner {
    /// Method called when the task is cleaned up.
    async fn cleanup(&self) {
        warn!(
            process_id=%self.process_labels.process_id,
            task_cat=%self.task_labels.task_cat,
            task_id=%self.task_labels.task_id,
            task_source=%self.task_labels.task_source,
            "cleaning task"
        )
    }
}
