# Syslog CEF Views Implementation

This implementation provides OpenTelemetry views for syslog and CEF (Common Event Format) messages, allowing you to access structured log data through the standard OTLP views interface.

## Overview

The implementation provides the following view traits for syslog messages:

- `LogsDataView` - Top-level logs data container
- `ResourceLogsView` - Resource-level logs container 
- `ScopeLogsView` - Scope-level logs container
- `LogRecordView` - Individual log record
- `ResourceView` - Minimal resource (no attributes for syslog)
- `InstrumentationScopeView` - Minimal scope (named "syslog")

## Design Decisions

Since syslog messages don't have a natural mapping to OpenTelemetry's resource and scope concepts, this implementation uses:

- **Single Resource**: Each syslog message is wrapped in one resource with no attributes
- **Single Scope**: Each resource contains one scope named "syslog" with no attributes  
- **Single Log Record**: Each scope contains one log record representing the syslog message
- **Message as Body**: The syslog message content becomes the log record body
- **Fields as Attributes**: Syslog fields (facility, severity, hostname, etc.) become log record attributes

## Supported Formats

- **RFC 5424**: Modern syslog format with structured data
- **RFC 3164**: Traditional syslog format  
- **CEF**: Common Event Format messages

## Attribute Mapping

### RFC 5424/3164 Messages
- `syslog.facility` (int64) - Syslog facility code
- `syslog.severity` (int64) - Syslog severity code  
- `syslog.version` (int64) - Syslog version (RFC 5424 only)
- `syslog.hostname` (string) - Hostname field
- `syslog.app_name` (string) - Application name (RFC 5424 only)
- `syslog.proc_id` (string) - Process ID (RFC 5424 only)
- `syslog.msg_id` (string) - Message ID (RFC 5424 only)
- `syslog.structured_data` (string) - Structured data (RFC 5424 only)
- `syslog.tag` (string) - Tag field (RFC 3164 only)

### CEF Messages
- `cef.version` (int64) - CEF version
- `cef.device_vendor` (string) - Device vendor
- `cef.device_product` (string) - Device product
- `cef.device_version` (string) - Device version
- `cef.signature_id` (string) - Signature ID
- `cef.severity` (string) - CEF severity
- `cef.extension` (string) - Extension values (one attribute per extension)

## Usage Example

```rust
use otap_df_syslog_cef::{parser, views::logs::SyslogLogsData};
use otap_df_pdata_views::views::logs::{LogsDataView, ResourceLogsView, ScopeLogsView, LogRecordView};

// Parse a syslog message
let message = b"<165>1 2003-10-11T22:14:15.003Z mymachine.example.com evntslog - ID47 - An application event log entry...";
let parsed = parser::parse(message)?;

// Create views
let logs_data = SyslogLogsData::new(&parsed);

// Access structured data
for resource_logs in logs_data.resources() {
    for scope_logs in resource_logs.scopes() {
        for log_record in scope_logs.log_records() {
            // Access severity, body, attributes, etc.
            if let Some(severity) = log_record.severity_number() {
                println!("Severity: {}", severity);
            }
            
            for attribute in log_record.attributes() {
                println!("{}: {:?}", attribute.key(), attribute.value());
            }
        }
    }
}
```

## Limitations

- Timestamps are not parsed - `time_unix_nano()` returns `None`
- No trace/span correlation - `trace_id()` and `span_id()` return `None`
- CEF extensions use generic `cef.extension` key rather than parsing extension names
- No schema URLs provided
- All attributes are at log record level (no resource or scope attributes)

## Testing

The implementation includes comprehensive tests for both RFC 5424 and CEF message formats:

```bash
cargo test -p otap-df-syslog-cef views::logs::tests
```

## Example

See `examples/views_example.rs` for a complete working example:

```bash
cargo run --package otap-df-syslog-cef --example views_example
```
