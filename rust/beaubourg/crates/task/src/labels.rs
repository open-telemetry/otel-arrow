//! List of labels for a process or a task.
use std::collections::HashMap;

/// List of labels for a process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessLabels {
    /// The process ID.
    pub process_id: String,
}

/// List of labels for a task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskLabels {
    /// The task category.
    pub task_cat: String,
    /// The task id.
    pub task_id: String,
    /// The task source id.
    pub task_source: String,
}

impl Default for ProcessLabels {
    fn default() -> Self {
        Self {
            process_id: "undefined".to_string(),
        }
    }
}

impl ProcessLabels {
    /// Create a new `ProcessLabels` instance.
    pub fn new(process_id: &str) -> Self {
        Self {
            process_id: process_id.into(),
        }
    }
}

impl TaskLabels {
    /// Create a new `TaskLabels` instance.
    pub fn new(task_cat: &str, task_id: &str, task_source: &str) -> Self {
        Self {
            task_cat: task_cat.into(),
            task_id: task_id.into(),
            task_source: task_source.into(),
        }
    }

    /// Create a unique task id.
    pub fn unique_id(&self) -> String {
        format!("{}:{}:{}", self.task_cat, self.task_id, self.task_source)
    }

    /// Convert this TaskLabels to a HashMap.
    pub fn as_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        let _ = map.insert("task_cat".into(), self.task_cat.clone());
        let _ = map.insert("task_id".into(), self.task_id.clone());
        let _ = map.insert("task_source".into(), self.task_source.clone());

        map
    }
}
