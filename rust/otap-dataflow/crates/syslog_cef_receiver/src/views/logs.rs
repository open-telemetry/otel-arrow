use crate::parser::ParsedMessage;
use otap_df_pdata_views::views::logs::{LogRecordView, LogsDataView, ResourceLogsView, ScopeLogsView};
use otap_df_pdata_views::views::common::{AnyValueView, AttributeView, InstrumentationScopeView, Str, ValueType};
use otap_df_pdata_views::views::resource::ResourceView;
use std::str;

/// Wrapper for a single ParsedMessage to implement LogsDataView
pub struct SyslogLogsData<'a> {
    /// The parsed syslog message
    pub message: &'a ParsedMessage<'a>,
}

impl<'a> SyslogLogsData<'a> {
    /// Create a new SyslogLogsData from a ParsedMessage
    pub fn new(message: &'a ParsedMessage<'a>) -> Self {
        Self { message }
    }
}

impl<'a> LogsDataView for SyslogLogsData<'a> {
    type ResourceLogs<'res> = SyslogResourceLogs<'res> where Self: 'res;
    type ResourcesIter<'res> = std::iter::Once<Self::ResourceLogs<'res>> where Self: 'res;

    fn resources(&self) -> Self::ResourcesIter<'_> {
        std::iter::once(SyslogResourceLogs { message: self.message })
    }
}

/// Resource logs for syslog - contains a single resource with a single scope
pub struct SyslogResourceLogs<'a> {
    /// The parsed syslog message
    pub message: &'a ParsedMessage<'a>,
}

impl<'a> ResourceLogsView for SyslogResourceLogs<'a> {
    type Resource<'res> = SyslogResource where Self: 'res;
    type ScopeLogs<'scp> = SyslogScopeLogs<'scp> where Self: 'scp;
    type ScopesIter<'scp> = std::iter::Once<Self::ScopeLogs<'scp>> where Self: 'scp;

    fn resource(&self) -> Option<Self::Resource<'_>> {
        Some(SyslogResource)
    }

    fn scopes(&self) -> Self::ScopesIter<'_> {
        std::iter::once(SyslogScopeLogs { message: self.message })
    }

    fn schema_url(&self) -> Option<Str<'_>> {
        None // No schema URL for syslog messages
    }
}

/// Minimal resource implementation for syslog
pub struct SyslogResource;

impl ResourceView for SyslogResource {
    type Attribute<'att> = SyslogAttribute<'att> where Self: 'att;
    type AttributesIter<'att> = std::iter::Empty<Self::Attribute<'att>> where Self: 'att;

    fn attributes(&self) -> Self::AttributesIter<'_> {
        std::iter::empty() // No resource-level attributes for syslog
    }

    fn dropped_attributes_count(&self) -> u32 {
        0
    }
}

/// Scope logs for syslog - contains a single log record
pub struct SyslogScopeLogs<'a> {
    /// The parsed syslog message
    pub message: &'a ParsedMessage<'a>,
}

impl<'a> ScopeLogsView for SyslogScopeLogs<'a> {
    type Scope<'scp> = SyslogScope where Self: 'scp;
    type LogRecord<'rec> = SyslogLogRecord<'rec> where Self: 'rec;
    type LogRecordsIter<'rec> = std::iter::Once<Self::LogRecord<'rec>> where Self: 'rec;

    fn scope(&self) -> Option<Self::Scope<'_>> {
        Some(SyslogScope)
    }

    fn log_records(&self) -> Self::LogRecordsIter<'_> {
        std::iter::once(SyslogLogRecord { message: self.message })
    }

    fn schema_url(&self) -> Option<Str<'_>> {
        None // No schema URL for syslog messages
    }
}

/// Minimal instrumentation scope for syslog
pub struct SyslogScope;

impl InstrumentationScopeView for SyslogScope {
    type Attribute<'att> = SyslogAttribute<'att> where Self: 'att;
    type AttributeIter<'att> = std::iter::Empty<Self::Attribute<'att>> where Self: 'att;

    fn name(&self) -> Option<Str<'_>> {
        Some("syslog") // Generic scope name for syslog messages
    }

    fn version(&self) -> Option<Str<'_>> {
        None // No version info for syslog scope
    }

    fn attributes(&self) -> Self::AttributeIter<'_> {
        std::iter::empty() // No scope-level attributes
    }

    fn dropped_attributes_count(&self) -> u32 {
        0
    }
}

/// Main log record implementation for syslog messages
pub struct SyslogLogRecord<'a> {
    /// The parsed syslog message
    pub message: &'a ParsedMessage<'a>,
}

impl<'a> LogRecordView for SyslogLogRecord<'a> {
    type Attribute<'att> = SyslogAttribute<'att> where Self: 'att;
    type AttributeIter<'att> = SyslogAttributeIterator<'att> where Self: 'att;
    type Body<'bod> = SyslogBody<'bod> where Self: 'bod;

    fn time_unix_nano(&self) -> Option<u64> {
        None // Syslog timestamps would need parsing to convert to unix nano
    }

    fn observed_time_unix_nano(&self) -> Option<u64> {
        None // Not available in syslog format
    }

    fn severity_number(&self) -> Option<i32> {
        match self.message {
            ParsedMessage::Rfc5424(msg) => Some(msg.priority.severity as i32),
            ParsedMessage::Rfc3164(msg) => Some(msg.priority.severity as i32),
            ParsedMessage::Cef(_) => None, // CEF has its own severity format
        }
    }

    fn severity_text(&self) -> Option<Str<'_>> {
        match self.message {
            ParsedMessage::Cef(msg) => msg.severity_str().ok(),
            _ => None, // RFC messages use numeric severity
        }
    }

    fn body(&self) -> Option<Self::Body<'_>> {
        Some(SyslogBody { message: self.message })
    }

    fn attributes(&self) -> Self::AttributeIter<'_> {
        SyslogAttributeIterator::new(self.message)
    }

    fn dropped_attributes_count(&self) -> u32 {
        0 // We don't drop any attributes
    }

    fn flags(&self) -> Option<u32> {
        None // No flags in syslog format
    }

    fn trace_id(&self) -> Option<&[u8]> {
        None // No trace ID in syslog format
    }

    fn span_id(&self) -> Option<&[u8]> {
        None // No span ID in syslog format
    }
}

/// Body implementation for syslog messages
pub struct SyslogBody<'a> {
    /// The parsed syslog message
    pub message: &'a ParsedMessage<'a>,
}

impl<'a> AnyValueView<'a> for SyslogBody<'a> {
    type KeyValue = SyslogAttribute<'a>;
    type ArrayIter<'arr> = std::iter::Empty<Self> where Self: 'arr;
    type KeyValueIter<'kv> = std::iter::Empty<Self::KeyValue> where Self: 'kv;

    fn value_type(&self) -> ValueType {
        ValueType::String
    }

    fn as_string(&self) -> Option<Str<'_>> {
        match self.message {
            ParsedMessage::Rfc5424(msg) => {
                msg.message.and_then(|bytes| str::from_utf8(bytes).ok())
            }
            ParsedMessage::Rfc3164(msg) => {
                msg.message.and_then(|bytes| str::from_utf8(bytes).ok())
            }
            ParsedMessage::Cef(msg) => {
                // For CEF, we could combine name and extensions into a formatted string
                str::from_utf8(msg.name).ok()
            }
        }
    }

    fn as_bool(&self) -> Option<bool> {
        None
    }

    fn as_int64(&self) -> Option<i64> {
        None
    }

    fn as_double(&self) -> Option<f64> {
        None
    }

    fn as_bytes(&self) -> Option<&[u8]> {
        None
    }

    fn as_array(&self) -> Option<Self::ArrayIter<'_>> {
        None
    }

    fn as_kvlist(&self) -> Option<Self::KeyValueIter<'_>> {
        None
    }
}

/// Attribute implementation for syslog field attributes
#[derive(Debug)]
pub struct SyslogAttribute<'a> {
    /// The attribute key
    pub key: &'static str,
    /// The attribute value data
    pub value: SyslogAttributeData<'a>,
}

/// Syslog attribute data, either bytes or a number
#[derive(Debug)]
pub enum SyslogAttributeData<'a> {
    /// Byte data from the syslog message
    Bytes(&'a [u8]),
    /// Numeric value (facility, severity, version)
    Number(u8),
}

impl<'a> AttributeView for SyslogAttribute<'a> {
    type Val<'val> = SyslogAttributeValue<'val> where Self: 'val;

    fn key(&self) -> Str<'_> {
        self.key
    }

    fn value(&self) -> Option<Self::Val<'_>> {
        Some(SyslogAttributeValue { data: &self.value })
    }
}

/// Value implementation for syslog attributes
pub struct SyslogAttributeValue<'a> {
    /// Reference to the attribute data
    pub data: &'a SyslogAttributeData<'a>,
}

impl<'a> AnyValueView<'a> for SyslogAttributeValue<'a> {
    type KeyValue = SyslogAttribute<'a>;
    type ArrayIter<'arr> = std::iter::Empty<Self> where Self: 'arr;
    type KeyValueIter<'kv> = std::iter::Empty<Self::KeyValue> where Self: 'kv;

    fn value_type(&self) -> ValueType {
        match self.data {
            SyslogAttributeData::Bytes(_) => ValueType::String,
            SyslogAttributeData::Number(_) => ValueType::Int64,
        }
    }

    fn as_string(&self) -> Option<Str<'_>> {
        match self.data {
            SyslogAttributeData::Bytes(bytes) => str::from_utf8(bytes).ok(),
            SyslogAttributeData::Number(_) => None,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        None
    }

    fn as_int64(&self) -> Option<i64> {
        match self.data {
            SyslogAttributeData::Number(n) => Some(*n as i64),
            SyslogAttributeData::Bytes(_) => None,
        }
    }

    fn as_double(&self) -> Option<f64> {
        None
    }

    fn as_bytes(&self) -> Option<&[u8]> {
        None
    }

    fn as_array(&self) -> Option<Self::ArrayIter<'_>> {
        None
    }

    fn as_kvlist(&self) -> Option<Self::KeyValueIter<'_>> {
        None
    }
}

/// Iterator for syslog message attributes
pub struct SyslogAttributeIterator<'a> {
    attributes: std::vec::IntoIter<SyslogAttribute<'a>>,
}

impl<'a> SyslogAttributeIterator<'a> {
    fn new(message: &'a ParsedMessage<'a>) -> Self {
        let mut attributes = Vec::new();
        
        match message {
            ParsedMessage::Rfc5424(msg) => {
                attributes.push(SyslogAttribute {
                    key: "syslog.facility",
                    value: SyslogAttributeData::Number(msg.priority.facility),
                });
                attributes.push(SyslogAttribute {
                    key: "syslog.severity",
                    value: SyslogAttributeData::Number(msg.priority.severity),
                });
                attributes.push(SyslogAttribute {
                    key: "syslog.version",
                    value: SyslogAttributeData::Number(msg.version),
                });
                if let Some(hostname) = msg.hostname {
                    attributes.push(SyslogAttribute {
                        key: "syslog.hostname",
                        value: SyslogAttributeData::Bytes(hostname),
                    });
                }
                if let Some(app_name) = msg.app_name {
                    attributes.push(SyslogAttribute {
                        key: "syslog.app_name",
                        value: SyslogAttributeData::Bytes(app_name),
                    });
                }
                if let Some(proc_id) = msg.proc_id {
                    attributes.push(SyslogAttribute {
                        key: "syslog.proc_id",
                        value: SyslogAttributeData::Bytes(proc_id),
                    });
                }
                if let Some(msg_id) = msg.msg_id {
                    attributes.push(SyslogAttribute {
                        key: "syslog.msg_id",
                        value: SyslogAttributeData::Bytes(msg_id),
                    });
                }
                if let Some(structured_data) = msg.structured_data {
                    attributes.push(SyslogAttribute {
                        key: "syslog.structured_data",
                        value: SyslogAttributeData::Bytes(structured_data),
                    });
                }
            }
            ParsedMessage::Rfc3164(msg) => {
                attributes.push(SyslogAttribute {
                    key: "syslog.facility",
                    value: SyslogAttributeData::Number(msg.priority.facility),
                });
                attributes.push(SyslogAttribute {
                    key: "syslog.severity",
                    value: SyslogAttributeData::Number(msg.priority.severity),
                });
                if let Some(hostname) = msg.hostname {
                    attributes.push(SyslogAttribute {
                        key: "syslog.hostname",
                        value: SyslogAttributeData::Bytes(hostname),
                    });
                }
                if let Some(tag) = msg.tag {
                    attributes.push(SyslogAttribute {
                        key: "syslog.tag",
                        value: SyslogAttributeData::Bytes(tag),
                    });
                }
            }
            ParsedMessage::Cef(msg) => {
                attributes.push(SyslogAttribute {
                    key: "cef.version",
                    value: SyslogAttributeData::Number(msg.version),
                });
                attributes.push(SyslogAttribute {
                    key: "cef.device_vendor",
                    value: SyslogAttributeData::Bytes(msg.device_vendor),
                });
                attributes.push(SyslogAttribute {
                    key: "cef.device_product",
                    value: SyslogAttributeData::Bytes(msg.device_product),
                });
                attributes.push(SyslogAttribute {
                    key: "cef.device_version",
                    value: SyslogAttributeData::Bytes(msg.device_version),
                });
                attributes.push(SyslogAttribute {
                    key: "cef.signature_id",
                    value: SyslogAttributeData::Bytes(msg.signature_id),
                });
                attributes.push(SyslogAttribute {
                    key: "cef.severity",
                    value: SyslogAttributeData::Bytes(msg.severity),
                });
                for (_key, value) in &msg.extensions {
                    // For now, we'll just use "cef.extension" as the key
                    // In a more sophisticated implementation, you might want to convert the key bytes to string
                    attributes.push(SyslogAttribute {
                        key: "cef.extension",
                        value: SyslogAttributeData::Bytes(value),
                    });
                }
            }
        }
        
        Self {
            attributes: attributes.into_iter(),
        }
    }
}

impl<'a> Iterator for SyslogAttributeIterator<'a> {
    type Item = SyslogAttribute<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.attributes.next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;

    #[test]
    fn test_syslog_views_rfc5424() {
        let message = b"<165>1 2003-10-11T22:14:15.003Z mymachine.example.com evntslog - ID47 [exampleSDID@32473 iut=\"3\" eventSource=\"Application\" eventID=\"1011\"] BOMAn application event log entry...";
        let parsed = parser::parse(message).expect("Failed to parse message");
        
        let logs_data = SyslogLogsData::new(&parsed);
        let mut resources = logs_data.resources();
        let resource_logs = resources.next().expect("Should have one resource");
        
        assert!(resource_logs.resource().is_some());
        assert!(resource_logs.schema_url().is_none());
        
        let mut scopes = resource_logs.scopes();
        let scope_logs = scopes.next().expect("Should have one scope");
        
        assert!(scope_logs.scope().is_some());
        assert!(scope_logs.schema_url().is_none());
        
        let mut log_records = scope_logs.log_records();
        let log_record = log_records.next().expect("Should have one log record");
        
        assert_eq!(log_record.severity_number(), Some(5)); // RFC5424 severity
        assert!(log_record.body().is_some());
        
        // Test attributes
        let attributes: Vec<_> = log_record.attributes().collect();
        assert!(!attributes.is_empty());
        
        // Check that we have facility and severity attributes
        let facility_attr = attributes.iter().find(|attr| attr.key() == "syslog.facility");
        assert!(facility_attr.is_some());
        
        let severity_attr = attributes.iter().find(|attr| attr.key() == "syslog.severity");
        assert!(severity_attr.is_some());
    }

    #[test] 
    fn test_syslog_views_cef() {
        let message = b"CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232";
        let parsed = parser::parse(message).expect("Failed to parse CEF message");
        
        let logs_data = SyslogLogsData::new(&parsed);
        let mut resources = logs_data.resources();
        let resource_logs = resources.next().expect("Should have one resource");
        
        let mut scopes = resource_logs.scopes();
        let scope_logs = scopes.next().expect("Should have one scope");
        
        let mut log_records = scope_logs.log_records();
        let log_record = log_records.next().expect("Should have one log record");
        
        // CEF doesn't have traditional syslog severity
        assert!(log_record.severity_number().is_none());
        assert!(log_record.severity_text().is_some());
        
        // Test attributes
        let attributes: Vec<_> = log_record.attributes().collect();
        assert!(!attributes.is_empty());
        
        // Check that we have CEF-specific attributes
        let version_attr = attributes.iter().find(|attr| attr.key() == "cef.version");
        assert!(version_attr.is_some());
        
        let vendor_attr = attributes.iter().find(|attr| attr.key() == "cef.device_vendor");
        assert!(vendor_attr.is_some());
    }
}

