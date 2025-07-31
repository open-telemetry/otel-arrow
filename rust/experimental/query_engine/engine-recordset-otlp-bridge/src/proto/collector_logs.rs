use crate::*;

#[derive(Debug)]
pub struct ExportLogsServiceRequest {
    pub resource_logs: Vec<ResourceLogs>,
}

impl Default for ExportLogsServiceRequest {
    fn default() -> Self {
        Self::new()
    }
}

impl ExportLogsServiceRequest {
    pub fn new() -> ExportLogsServiceRequest {
        Self {
            resource_logs: Vec::new(),
        }
    }

    pub fn with_resource_logs(mut self, value: ResourceLogs) -> ExportLogsServiceRequest {
        self.resource_logs.push(value);
        self
    }
}
