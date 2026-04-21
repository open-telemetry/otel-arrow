// The ID fields in arrow records are relative to the current batch and not globally unique.
// We introduce a new partation id field that ensures global uniqueness when used in conjunction
// with the other ID fields.
pub const PART_ID: &str = "part_id";

// These fields are flattened from the original schema due to clichouse limitation
// in representing nullable structs with nullable values. This prevents reconstruction
// of the original arrow payload from the stored data, but is sufficient for generation of
// valid otel data models.
pub const RESOURCE_ID: &str = "resource_id";
pub const SCOPE_ID: &str = "scope_id";
pub const INSERT_TIME: &str = "insert_time";

pub const CH_TIMESTAMP: &str = "Timestamp";
pub const CH_TIMESTAMP_TIME: &str = "TimestampTime";
pub const CH_RESOURCE_SCHEMA_URL: &str = "ResourceSchemaUrl";
pub const CH_RESOURCE_ATTRIBUTES: &str = "ResourceAttributes";
pub const CH_SCOPE_SCHEMA_URL: &str = "ScopeSchemaUrl";
pub const CH_SCOPE_NAME: &str = "ScopeName";
pub const CH_SCOPE_VERSION: &str = "ScopeVersion";
pub const CH_SCOPE_ATTRIBUTES: &str = "ScopeAttributes";
pub const CH_LOG_ATTRIBUTES: &str = "LogAttributes";
pub const CH_SPAN_ATTRIBUTES: &str = "SpanAttributes";
pub const CH_TRACE_ID: &str = "TraceId";
pub const CH_SPAN_ID: &str = "SpanId";
pub const CH_SEVERITY_TEXT: &str = "SeverityText";
pub const CH_SEVERITY_NUMBER: &str = "SeverityNumber";
pub const CH_SERVICE_NAME: &str = "ServiceName";
pub const CH_BODY: &str = "Body";
pub const CH_EVENT_NAME: &str = "EventName";
pub const CH_PARENT_SPAN_ID: &str = "ParentSpanId";
pub const CH_TRACE_STATE: &str = "TraceState";
pub const CH_SPAN_NAME: &str = "SpanName";
pub const CH_SPAN_KIND: &str = "SpanKind";

pub const CH_DURATION: &str = "Duration";
pub const CH_STATUS_CODE: &str = "StatusCode";
pub const CH_STATUS_MESSAGE: &str = "StatusMessage";
pub const CH_EVENTS_TIMESTAMP: &str = "Events.Timestamp";
pub const CH_EVENTS_NAME: &str = "Events.Name";
pub const CH_EVENTS_ATTRIBUTES: &str = "Events.Attributes";

pub const CH_LINKS_TRACE_ID: &str = "Links.TraceId";
pub const CH_LINKS_SPAN_ID: &str = "Links.SpanId";
pub const CH_LINKS_TRACE_STATE: &str = "Links.TraceState";
pub const CH_LINKS_ATTRIBUTES: &str = "Links.Attributes";
