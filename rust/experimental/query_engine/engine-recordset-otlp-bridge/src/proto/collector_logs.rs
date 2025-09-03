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

    pub fn from_protobuf(
        protobuf_data: &[u8],
    ) -> Result<ExportLogsServiceRequest, SerializerError> {
        serializer::otlp_reader::read_export_logs_service_request(protobuf_data)
    }

    pub fn to_protobuf(
        value: &ExportLogsServiceRequest,
        initial_capacity: usize,
    ) -> Result<Vec<u8>, SerializerError> {
        serializer::otlp_writer::write_export_logs_service_request(value, initial_capacity)
    }
}
