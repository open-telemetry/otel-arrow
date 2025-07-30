//! Example demonstrating how to use the syslog views to access structured log data.

use otap_df_syslog_cef::{parser, views::logs::SyslogLogsData};
use otap_df_pdata_views::views::logs::{LogsDataView, ResourceLogsView, ScopeLogsView, LogRecordView};
use otap_df_pdata_views::views::common::{AttributeView, AnyValueView, InstrumentationScopeView};
use otap_df_pdata_views::views::resource::ResourceView;

fn main() {
    // Example RFC 5424 syslog message
    let rfc5424_message = b"<165>1 2003-10-11T22:14:15.003Z mymachine.example.com evntslog - ID47 [exampleSDID@32473 iut=\"3\" eventSource=\"Application\" eventID=\"1011\"] BOMAn application event log entry...";
    
    // Parse the message
    let parsed_message = parser::parse(rfc5424_message).expect("Failed to parse syslog message");
    
    // Create logs data view
    let logs_data = SyslogLogsData::new(&parsed_message);
    
    // Iterate through resources (there will be one)
    for resource_logs in logs_data.resources() {
        println!("Processing resource logs...");
        
        if let Some(resource) = resource_logs.resource() {
            println!("  Resource has {} attributes", resource.attributes().count());
        }
        
        // Iterate through scopes (there will be one)
        for scope_logs in resource_logs.scopes() {
            println!("  Processing scope logs...");
            
            if let Some(scope) = scope_logs.scope() {
                if let Some(name) = scope.name() {
                    println!("    Scope name: {}", name);
                }
            }
            
            // Iterate through log records (there will be one)
            for log_record in scope_logs.log_records() {
                println!("    Processing log record...");
                
                if let Some(severity) = log_record.severity_number() {
                    println!("      Severity number: {}", severity);
                }
                
                if let Some(body) = log_record.body() {
                    if let Some(body_str) = body.as_string() {
                        println!("      Body: {}", body_str);
                    }
                }
                
                println!("      Attributes:");
                for attribute in log_record.attributes() {
                    let key = attribute.key();
                    if let Some(value) = attribute.value() {
                        match value.value_type() {
                            otap_df_pdata_views::views::common::ValueType::String => {
                                if let Some(str_val) = value.as_string() {
                                    println!("        {}: {}", key, str_val);
                                }
                            }
                            otap_df_pdata_views::views::common::ValueType::Int64 => {
                                if let Some(int_val) = value.as_int64() {
                                    println!("        {}: {}", key, int_val);
                                }
                            }
                            _ => {
                                println!("        {}: <other type>", key);
                            }
                        }
                    }
                }
            }
        }
    }
    
    println!("\n--- CEF Example ---");
    
    // Example CEF message
    let cef_message = b"CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232";
    
    // Parse the CEF message
    let parsed_cef = parser::parse(cef_message).expect("Failed to parse CEF message");
    
    // Create logs data view
    let cef_logs_data = SyslogLogsData::new(&parsed_cef);
    
    // Process the CEF message similarly
    for resource_logs in cef_logs_data.resources() {
        for scope_logs in resource_logs.scopes() {
            for log_record in scope_logs.log_records() {
                println!("CEF Log Record:");
                
                if let Some(severity_text) = log_record.severity_text() {
                    println!("  Severity text: {}", severity_text);
                }
                
                if let Some(body) = log_record.body() {
                    if let Some(body_str) = body.as_string() {
                        println!("  Body: {}", body_str);
                    }
                }
                
                println!("  CEF Attributes:");
                for attribute in log_record.attributes() {
                    let key = attribute.key();
                    if let Some(value) = attribute.value() {
                        if let Some(str_val) = value.as_string() {
                            println!("    {}: {}", key, str_val);
                        } else if let Some(int_val) = value.as_int64() {
                            println!("    {}: {}", key, int_val);
                        }
                    }
                }
            }
        }
    }
}
