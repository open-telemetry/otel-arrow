// SPDX-License-Identifier: Apache-2.0
// Copyright The OpenTelemetry Authors

use chrono::Utc;
use otel_arrow_rust::{
    encode::record::{attributes::AttributesRecordBatchBuilder, logs::LogsRecordBatchBuilder},
    otap::{Logs, OtapArrowRecords},
    proto::opentelemetry::arrow::v1::ArrowPayloadType,
};

use otap_df_otap::encoder::error::Result;

use crate::parser::parsed_message::ParsedSyslogMessage;

pub(crate) struct ArrowRecordsBuilder {
    curr_log_id: u16,
    logs: LogsRecordBatchBuilder,
    log_attrs: AttributesRecordBatchBuilder<u16>,
}

impl ArrowRecordsBuilder {
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            curr_log_id: 0,
            logs: LogsRecordBatchBuilder::new(),
            log_attrs: AttributesRecordBatchBuilder::<u16>::new(),
        }
    }

    pub(crate) fn len(&self) -> u16 {
        // Current log record ID is incremented for each new log appended
        // so it can be used to get the number of logs in the builder.
        self.curr_log_id
    }

    pub(crate) fn append_syslog(&mut self, syslog_message: ParsedSyslogMessage<'_>) {
        self.logs
            .append_time_unix_nano(syslog_message.timestamp().map(|v| v as i64));

        let (severity_number, severity_text) =
            syslog_message.severity().unwrap_or((0, "UNSPECIFIED"));
        self.logs.append_severity_number(Some(severity_number));
        self.logs.append_severity_text(Some(severity_text));

        self.logs.body.append_str(syslog_message.input().as_ref());

        let attributes_added = syslog_message.add_attribues_to_arrow(&mut self.log_attrs);

        for _ in 0..attributes_added {
            self.log_attrs.append_parent_id(&self.curr_log_id);
        }

        self.logs.append_id(Some(self.curr_log_id));

        self.curr_log_id += 1;
    }

    pub(crate) fn build(mut self) -> Result<OtapArrowRecords> {
        let log_record_count = self.curr_log_id.into();

        // All the logs belong to the same resource and scope
        self.logs.resource.append_id_n(0, log_record_count);
        self.logs
            .resource
            .append_schema_url_n(None, log_record_count);
        self.logs
            .resource
            .append_dropped_attributes_count_n(0, log_record_count);

        self.logs.scope.append_id_n(0, log_record_count);
        self.logs.scope.append_name_n(None, log_record_count);
        self.logs.scope.append_version_n(None, log_record_count);
        self.logs
            .scope
            .append_dropped_attributes_count_n(0, log_record_count);

        let observed_time = Utc::now().timestamp_nanos_opt().unwrap_or(0);
        for _ in 0..log_record_count {
            self.logs
                .append_observed_time_unix_nano(Some(observed_time));
        }

        self.logs.append_schema_url_n(None, log_record_count);

        for _ in 0..log_record_count {
            self.logs.append_dropped_attributes_count(0);
        }

        for _ in 0..log_record_count {
            self.logs.append_flags(None);
        }

        for _ in 0..log_record_count {
            _ = self.logs.append_trace_id(None);
        }

        for _logs in 0..log_record_count {
            _ = self.logs.append_trace_id(None);
        }

        for _logs in 0..log_record_count {
            _ = self.logs.append_span_id(None);
        }

        let mut otap_batch = OtapArrowRecords::Logs(Logs::default());

        // append logs record
        otap_batch.set(ArrowPayloadType::Logs, self.logs.finish()?);

        // append log attrs record batch if there is one
        let log_attrs_rb = self.log_attrs.finish()?;
        if log_attrs_rb.num_rows() > 0 {
            otap_batch.set(ArrowPayloadType::LogAttrs, log_attrs_rb);
        }

        Ok(otap_batch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    use otel_arrow_rust::{
        otlp::logs::logs_from, proto::opentelemetry::common::v1::any_value::Value,
    };

    use chrono::{DateTime, Datelike, Local, TimeZone};

    /// Custom enum for attribute values that provides stronger type assertions
    /// than using String for all values
    #[derive(Debug, Clone, PartialEq)]
    enum AttributeValue {
        String(String),
        Integer(i64),
    }

    /// Helper function to validate observed timestamps are within a reasonable time range
    /// The observed timestamp should be between when we started building and when we finished
    fn validate_observed_timestamp_range(
        start_time: i64,
        end_time: i64,
        observed_time_array: &arrow::array::TimestampNanosecondArray,
        record_count: usize,
    ) {
        assert_eq!(observed_time_array.len(), record_count);
        for i in 0..record_count {
            let observed_time = observed_time_array.value(i);
            assert!(
                observed_time >= start_time && observed_time <= end_time,
                "Observed timestamp {observed_time} should be between {start_time} and {end_time}"
            );
        }
    }

    /// Helper function to validate observed timestamp for OTLP log records
    /// The observed timestamp should be between when we started building and when we finished
    fn validate_otlp_observed_timestamp_range(
        start_time: i64,
        end_time: i64,
        observed_timestamp: u64,
    ) {
        let observed_time = observed_timestamp as i64;
        assert!(
            observed_time >= start_time && observed_time <= end_time,
            "OTLP observed timestamp {observed_time} should be between {start_time} and {end_time}"
        );
    }

    #[test]
    fn test_rfc5424_syslog_to_arrow_structure() {
        // This test validates that RFC 5424 syslog messages are correctly converted to Arrow format
        // by directly inspecting the Arrow record batch structure without converting to OTLP.
        // It verifies the schema, column data types, and actual values in each column.

        let mut builder = ArrowRecordsBuilder::new();

        // Capture start time before processing messages
        let start_time = Utc::now().timestamp_nanos_opt().unwrap_or(0);

        // Sample RFC 5424 syslog messages
        let syslog_messages = vec![
            "<34>1 2024-01-15T10:30:45.123Z mymachine.example.com su - ID47 - 'su root' failed for lonvick on /dev/pts/8",
            "<165>1 2024-01-15T10:31:00.456Z host.example.com myapp 1234 ID123 [exampleSDID@32473 iut=\"3\" eventSource=\"Application\" eventID=\"1011\"] Application started successfully",
            "<14>1 2024-01-15T10:32:15.789Z server01.example.com kernel - - - Kernel panic - not syncing: VFS",
        ];

        // Parse and append each message
        for msg in syslog_messages {
            let parsed = parse(msg.as_bytes()).unwrap();
            builder.append_syslog(parsed);
        }

        // Build arrow records and capture end time
        let arrow_records = builder.build().unwrap();
        let end_time = Utc::now().timestamp_nanos_opt().unwrap_or(0);

        // Extract the logs record batch
        if let OtapArrowRecords::Logs(_logs) = &arrow_records {
            let logs_batch = arrow_records
                .get(ArrowPayloadType::Logs)
                .expect("Logs record batch should be present");

            // Verify the number of rows (log records)
            assert_eq!(logs_batch.num_rows(), 3);

            // Verify schema - check that expected columns are present
            let schema = logs_batch.schema();
            let column_names: Vec<&str> =
                schema.fields().iter().map(|f| f.name().as_str()).collect();

            // Core log fields that should be present (based on actual implementation)
            assert!(column_names.contains(&"id"));
            assert!(column_names.contains(&"resource"));
            assert!(column_names.contains(&"scope"));
            assert!(column_names.contains(&"time_unix_nano"));
            assert!(column_names.contains(&"observed_time_unix_nano"));
            assert!(column_names.contains(&"severity_number"));
            assert!(column_names.contains(&"severity_text"));
            assert!(column_names.contains(&"body"));

            // Get specific columns for detailed validation
            use arrow::array::*;
            use arrow::datatypes::{UInt8Type, UInt16Type};

            // Check body column (it's a struct, so we need to access it differently)
            let body_column = logs_batch
                .column_by_name("body")
                .expect("Body column should exist");
            let body_struct = body_column
                .as_any()
                .downcast_ref::<StructArray>()
                .expect("Body should be StructArray");

            // The body struct should contain the syslog message in a string field
            // For now, let's just verify the structure exists
            assert_eq!(body_struct.len(), 3);

            // Check severity_number column (it's a dictionary array)
            let severity_num_column = logs_batch
                .column_by_name("severity_number")
                .expect("Severity number column should exist");

            // Handle dictionary array for severity_number
            if let Some(severity_dict_array) = severity_num_column
                .as_any()
                .downcast_ref::<DictionaryArray<UInt8Type>>()
            {
                assert_eq!(severity_dict_array.len(), 3);

                // Get the dictionary values
                let dict_values = severity_dict_array
                    .values()
                    .as_any()
                    .downcast_ref::<Int32Array>()
                    .expect("Dictionary values should be Int32Array");

                // Get the actual values by looking up the keys
                let val1 = dict_values.value(severity_dict_array.keys().value(0) as usize);
                let val2 = dict_values.value(severity_dict_array.keys().value(1) as usize);
                let val3 = dict_values.value(severity_dict_array.keys().value(2) as usize);

                assert_eq!(val1, 18); // Critical -> ERROR2
                assert_eq!(val2, 10); // Notice -> INFO2
                assert_eq!(val3, 9); // Informational -> INFO
            } else {
                panic!(
                    "Unexpected severity_number data type: {:?}",
                    severity_num_column.data_type()
                );
            }

            // Check severity_text column (likely also a dictionary)
            let severity_text_column = logs_batch
                .column_by_name("severity_text")
                .expect("Severity text column should exist");

            // Handle dictionary array for severity_text
            if let Some(severity_text_dict_array) = severity_text_column
                .as_any()
                .downcast_ref::<DictionaryArray<UInt8Type>>()
            {
                assert_eq!(severity_text_dict_array.len(), 3);

                // Get the dictionary values
                let dict_values = severity_text_dict_array
                    .values()
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .expect("Dictionary values should be StringArray");

                // Get the actual values by looking up the keys
                let val1 = dict_values.value(severity_text_dict_array.keys().value(0) as usize);
                let val2 = dict_values.value(severity_text_dict_array.keys().value(1) as usize);
                let val3 = dict_values.value(severity_text_dict_array.keys().value(2) as usize);

                assert_eq!(val1, "ERROR2");
                assert_eq!(val2, "INFO2");
                assert_eq!(val3, "INFO");
            } else if let Some(severity_text_array) =
                severity_text_column.as_any().downcast_ref::<StringArray>()
            {
                assert_eq!(severity_text_array.len(), 3);
                assert_eq!(severity_text_array.value(0), "ERROR2");
                assert_eq!(severity_text_array.value(1), "INFO2");
                assert_eq!(severity_text_array.value(2), "INFO");
            } else {
                panic!(
                    "Unexpected severity_text data type: {:?}",
                    severity_text_column.data_type()
                );
            }

            // Check time_unix_nano column
            let time_column = logs_batch
                .column_by_name("time_unix_nano")
                .expect("Time column should exist");
            let time_array = time_column
                .as_any()
                .downcast_ref::<TimestampNanosecondArray>()
                .expect("Time should be TimestampNanosecondArray");

            assert_eq!(time_array.len(), 3);

            // Verify timestamps are correctly parsed
            let expected_timestamp_1 = DateTime::parse_from_rfc3339("2024-01-15T10:30:45.123Z")
                .unwrap()
                .timestamp_nanos_opt()
                .unwrap();
            let expected_timestamp_2 = DateTime::parse_from_rfc3339("2024-01-15T10:31:00.456Z")
                .unwrap()
                .timestamp_nanos_opt()
                .unwrap();
            let expected_timestamp_3 = DateTime::parse_from_rfc3339("2024-01-15T10:32:15.789Z")
                .unwrap()
                .timestamp_nanos_opt()
                .unwrap();

            assert_eq!(time_array.value(0), expected_timestamp_1);
            assert_eq!(time_array.value(1), expected_timestamp_2);
            assert_eq!(time_array.value(2), expected_timestamp_3);

            // Check observed_time_unix_nano column (should be between start and end time)
            let observed_time_column = logs_batch
                .column_by_name("observed_time_unix_nano")
                .expect("Observed time column should exist");
            let observed_time_array = observed_time_column
                .as_any()
                .downcast_ref::<TimestampNanosecondArray>()
                .expect("Observed time should be TimestampNanosecondArray");

            validate_observed_timestamp_range(start_time, end_time, observed_time_array, 3);

            // Check attributes record batch using Arrow APIs directly
            if let Some(attrs_batch) = arrow_records.get(ArrowPayloadType::LogAttrs) {
                assert!(
                    attrs_batch.num_rows() > 0,
                    "Should have attributes for RFC5424 messages"
                );

                // Verify attributes schema
                let attrs_schema = attrs_batch.schema();
                let attrs_column_names: Vec<&str> = attrs_schema
                    .fields()
                    .iter()
                    .map(|f| f.name().as_str())
                    .collect();

                assert!(attrs_column_names.contains(&"parent_id"));
                assert!(attrs_column_names.contains(&"key"));
                // Based on data model docs, attributes have separate type-specific columns (str, int, etc.)
                assert!(attrs_column_names.contains(&"type"));
                assert!(attrs_column_names.contains(&"str"));

                // Get the columns for attribute validation
                let parent_id_column = attrs_batch
                    .column_by_name("parent_id")
                    .expect("Parent ID column should exist");
                let key_column = attrs_batch
                    .column_by_name("key")
                    .expect("Key column should exist");
                let type_column = attrs_batch
                    .column_by_name("type")
                    .expect("Type column should exist");
                let str_column = attrs_batch
                    .column_by_name("str")
                    .expect("String column should exist");

                // Parent ID should be UInt16Array based on schema docs
                let parent_ids = parent_id_column
                    .as_any()
                    .downcast_ref::<UInt16Array>()
                    .expect("Parent ID should be UInt16Array");

                // Key column is a Dictionary(UInt8, Utf8) or similar
                let key_dict = key_column
                    .as_any()
                    .downcast_ref::<DictionaryArray<UInt8Type>>()
                    .expect("Key should be DictionaryArray");
                let key_values = key_dict
                    .values()
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .expect("Key dictionary values should be StringArray");

                // Type column indicates the value type
                let type_array = type_column
                    .as_any()
                    .downcast_ref::<UInt8Array>()
                    .expect("Type should be UInt8Array");

                // String value column should be a Dictionary(UInt16, Utf8)
                let str_dict = str_column
                    .as_any()
                    .downcast_ref::<DictionaryArray<UInt16Type>>()
                    .expect("String value should be DictionaryArray");
                let str_values = str_dict
                    .values()
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .expect("String dictionary values should be StringArray");

                // Check if there's an int column for integer values
                let int_column = attrs_batch.column_by_name("int");
                let int_dict = int_column
                    .and_then(|col| col.as_any().downcast_ref::<DictionaryArray<UInt16Type>>());
                let int_values =
                    int_dict.and_then(|dict| dict.values().as_any().downcast_ref::<Int64Array>());

                // Build a map of log_id -> {key -> value} for easier validation
                let mut log_attributes: std::collections::HashMap<
                    u16,
                    std::collections::HashMap<String, AttributeValue>,
                > = std::collections::HashMap::new();

                for i in 0..attrs_batch.num_rows() {
                    let parent_id = parent_ids.value(i);
                    let key_index = key_dict.key(i).unwrap();
                    let key = key_values.value(key_index);
                    let value_type = type_array.value(i);

                    // Extract the appropriate value based on type
                    let value = match value_type {
                        1 => {
                            // String type
                            if !str_dict.is_null(i) {
                                let str_index = str_dict.key(i).unwrap();
                                AttributeValue::String(str_values.value(str_index).to_string())
                            } else {
                                AttributeValue::String(String::new())
                            }
                        }
                        2 => {
                            // Integer type
                            if let (Some(int_dict), Some(int_vals)) = (int_dict, int_values) {
                                if !int_dict.is_null(i) {
                                    let int_index = int_dict.key(i).unwrap();
                                    AttributeValue::Integer(int_vals.value(int_index))
                                } else {
                                    AttributeValue::String(String::new())
                                }
                            } else {
                                AttributeValue::String(String::new())
                            }
                        }
                        _ => {
                            // Handle other types if needed
                            AttributeValue::String(String::new())
                        }
                    };

                    _ = log_attributes
                        .entry(parent_id)
                        .or_default()
                        .insert(key.to_string(), value);
                }

                // Validate attributes for each log record
                // Note: parent_id 0 corresponds to the first log record, 1 to the second, etc.

                // Check first log record attributes (RFC5424 with minimal structured data)
                let Some(log1_attrs) = log_attributes.get(&0) else {
                    panic!("Expected attributes for first log record");
                };

                assert_eq!(
                    log1_attrs.get("syslog.version"),
                    Some(&AttributeValue::Integer(1))
                );
                assert_eq!(
                    log1_attrs.get("syslog.facility"),
                    Some(&AttributeValue::Integer(4))
                );
                assert_eq!(
                    log1_attrs.get("syslog.severity"),
                    Some(&AttributeValue::Integer(2))
                );
                assert_eq!(
                    log1_attrs.get("syslog.host_name"),
                    Some(&AttributeValue::String("mymachine.example.com".to_string()))
                );
                assert_eq!(
                    log1_attrs.get("syslog.app_name"),
                    Some(&AttributeValue::String("su".to_string()))
                );
                assert_eq!(
                    log1_attrs.get("syslog.msg_id"),
                    Some(&AttributeValue::String("ID47".to_string()))
                );
                assert_eq!(
                    log1_attrs.get("syslog.message"),
                    Some(&AttributeValue::String(
                        "'su root' failed for lonvick on /dev/pts/8".to_string()
                    ))
                );

                // Check that we have exactly the expected number of attributes (7 for this message)
                // Note: process_id and structured_data are "-" (nil) so they might not be included
                assert!(
                    log1_attrs.len() >= 7,
                    "Log 1 should have at least 7 attributes, got {}",
                    log1_attrs.len()
                );

                // Check second log record attributes (RFC5424 with structured data)
                let Some(log2_attrs) = log_attributes.get(&1) else {
                    panic!("Expected attributes for second log record");
                };

                assert_eq!(
                    log2_attrs.get("syslog.version"),
                    Some(&AttributeValue::Integer(1))
                );
                assert_eq!(
                    log2_attrs.get("syslog.facility"),
                    Some(&AttributeValue::Integer(20))
                );
                assert_eq!(
                    log2_attrs.get("syslog.severity"),
                    Some(&AttributeValue::Integer(5))
                );
                assert_eq!(
                    log2_attrs.get("syslog.host_name"),
                    Some(&AttributeValue::String("host.example.com".to_string()))
                );
                assert_eq!(
                    log2_attrs.get("syslog.app_name"),
                    Some(&AttributeValue::String("myapp".to_string()))
                );
                assert_eq!(
                    log2_attrs.get("syslog.process_id"),
                    Some(&AttributeValue::Integer(1234))
                );
                assert_eq!(
                    log2_attrs.get("syslog.msg_id"),
                    Some(&AttributeValue::String("ID123".to_string()))
                );
                assert_eq!(log2_attrs.get("syslog.structured_data"), Some(&AttributeValue::String("[exampleSDID@32473 iut=\"3\" eventSource=\"Application\" eventID=\"1011\"]".to_string())));
                assert_eq!(
                    log2_attrs.get("syslog.message"),
                    Some(&AttributeValue::String(
                        "Application started successfully".to_string()
                    ))
                );

                // Check that we have exactly the expected number of attributes (9 for complete RFC5424)
                assert_eq!(
                    log2_attrs.len(),
                    9,
                    "Log 2 should have exactly 9 attributes, got {}",
                    log2_attrs.len()
                );

                // Check third log record attributes (RFC5424 with nil values)
                // Message: "<14>1 2024-01-15T10:32:15.789Z server01.example.com kernel - - - Kernel panic - not syncing: VFS"
                // This should have 6 attributes (version, facility, severity, host_name, app_name, message)
                // Note: process_id, msg_id, and structured_data are "-" (nil values) so they should not be included
                let Some(log3_attrs) = log_attributes.get(&2) else {
                    panic!("Expected attributes for third log record");
                };

                assert_eq!(
                    log3_attrs.get("syslog.version"),
                    Some(&AttributeValue::Integer(1))
                );
                assert_eq!(
                    log3_attrs.get("syslog.facility"),
                    Some(&AttributeValue::Integer(1))
                ); // Priority 14: 1 (user level) * 8 + 6
                assert_eq!(
                    log3_attrs.get("syslog.severity"),
                    Some(&AttributeValue::Integer(6))
                ); // informational
                assert_eq!(
                    log3_attrs.get("syslog.host_name"),
                    Some(&AttributeValue::String("server01.example.com".to_string()))
                );
                assert_eq!(
                    log3_attrs.get("syslog.app_name"),
                    Some(&AttributeValue::String("kernel".to_string()))
                );
                assert_eq!(
                    log3_attrs.get("syslog.message"),
                    Some(&AttributeValue::String(
                        "Kernel panic - not syncing: VFS".to_string()
                    ))
                );

                // Check that we have exactly the expected number of attributes (6 for this message)
                // process_id, msg_id, and structured_data should not be present as they are nil ("-")
                assert_eq!(
                    log3_attrs.len(),
                    6,
                    "Log 3 should have exactly 6 attributes, got {}",
                    log3_attrs.len()
                );

                // Verify that nil fields are not present
                assert!(
                    !log3_attrs.contains_key("syslog.process_id"),
                    "syslog.process_id should not be present (nil value)"
                );
                assert!(
                    !log3_attrs.contains_key("syslog.msg_id"),
                    "syslog.msg_id should not be present (nil value)"
                );
                assert!(
                    !log3_attrs.contains_key("syslog.structured_data"),
                    "syslog.structured_data should not be present (nil value)"
                );
            }
        } else {
            panic!("Expected Logs record batch");
        }
    }

    #[test]
    fn test_rfc3164_syslog_to_arrow_structure() {
        // This test validates that RFC 3164 syslog messages are correctly converted to Arrow format
        // by directly inspecting the Arrow record batch structure without converting to OTLP.
        // RFC 3164 is the traditional syslog format without version numbers or structured data.

        let mut builder = ArrowRecordsBuilder::new();

        // Capture start time before processing messages
        let start_time = Utc::now().timestamp_nanos_opt().unwrap_or(0);

        // Sample RFC 3164 syslog messages
        let syslog_messages = vec![
            "<34>Oct 11 22:14:15 mymachine su: 'su root' failed for lonvick on /dev/pts/8",
            "<165>Feb  5 17:32:18 hostname app[1234]: Application started successfully",
            "<14>Jan 15 10:30:45 server01 kernel: Kernel panic - not syncing: VFS",
        ];

        // Parse and append each message
        for msg in syslog_messages {
            let parsed = parse(msg.as_bytes()).unwrap();
            builder.append_syslog(parsed);
        }

        // Build arrow records and capture end time
        let arrow_records = builder.build().unwrap();
        let end_time = Utc::now().timestamp_nanos_opt().unwrap_or(0);

        // Extract the logs record batch
        if let OtapArrowRecords::Logs(_logs) = &arrow_records {
            let logs_batch = arrow_records
                .get(ArrowPayloadType::Logs)
                .expect("Logs record batch should be present");

            // Verify the number of rows (log records)
            assert_eq!(logs_batch.num_rows(), 3);

            // Get specific columns for detailed validation
            use arrow::array::*;
            use arrow::datatypes::{UInt8Type, UInt16Type};

            // Check body column (it's a struct, so we need to access it differently)
            let body_column = logs_batch
                .column_by_name("body")
                .expect("Body column should exist");
            let body_struct = body_column
                .as_any()
                .downcast_ref::<StructArray>()
                .expect("Body should be StructArray");

            // The body struct should contain the syslog message in a string field
            // For now, let's just verify the structure exists
            assert_eq!(body_struct.len(), 3);

            // Check severity_number column (it's a dictionary array)
            let severity_num_column = logs_batch
                .column_by_name("severity_number")
                .expect("Severity number column should exist");

            // Handle dictionary array for severity_number
            if let Some(severity_dict_array) = severity_num_column
                .as_any()
                .downcast_ref::<DictionaryArray<UInt8Type>>()
            {
                assert_eq!(severity_dict_array.len(), 3);

                // Get the dictionary values
                let dict_values = severity_dict_array
                    .values()
                    .as_any()
                    .downcast_ref::<Int32Array>()
                    .expect("Dictionary values should be Int32Array");

                // Get the actual values by looking up the keys
                let val1 = dict_values.value(severity_dict_array.keys().value(0) as usize);
                let val2 = dict_values.value(severity_dict_array.keys().value(1) as usize);
                let val3 = dict_values.value(severity_dict_array.keys().value(2) as usize);

                assert_eq!(val1, 18); // Critical -> ERROR2
                assert_eq!(val2, 10); // Notice -> INFO2
                assert_eq!(val3, 9); // Informational -> INFO
            } else {
                panic!(
                    "Unexpected severity_number data type: {:?}",
                    severity_num_column.data_type()
                );
            }

            // Check severity_text column (likely also a dictionary)
            let severity_text_column = logs_batch
                .column_by_name("severity_text")
                .expect("Severity text column should exist");

            // Handle dictionary array for severity_text
            if let Some(severity_text_dict_array) = severity_text_column
                .as_any()
                .downcast_ref::<DictionaryArray<UInt8Type>>()
            {
                assert_eq!(severity_text_dict_array.len(), 3);

                // Get the dictionary values
                let dict_values = severity_text_dict_array
                    .values()
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .expect("Dictionary values should be StringArray");

                // Get the actual values by looking up the keys
                let val1 = dict_values.value(severity_text_dict_array.keys().value(0) as usize);
                let val2 = dict_values.value(severity_text_dict_array.keys().value(1) as usize);
                let val3 = dict_values.value(severity_text_dict_array.keys().value(2) as usize);

                assert_eq!(val1, "ERROR2");
                assert_eq!(val2, "INFO2");
                assert_eq!(val3, "INFO");
            } else if let Some(severity_text_array) =
                severity_text_column.as_any().downcast_ref::<StringArray>()
            {
                assert_eq!(severity_text_array.len(), 3);
                assert_eq!(severity_text_array.value(0), "ERROR2");
                assert_eq!(severity_text_array.value(1), "INFO2");
                assert_eq!(severity_text_array.value(2), "INFO");
            } else {
                panic!(
                    "Unexpected severity_text data type: {:?}",
                    severity_text_column.data_type()
                );
            }

            // Check time_unix_nano column for RFC3164 timestamps
            let time_column = logs_batch
                .column_by_name("time_unix_nano")
                .expect("Time column should exist");
            let time_array = time_column
                .as_any()
                .downcast_ref::<TimestampNanosecondArray>()
                .expect("Time should be TimestampNanosecondArray");

            assert_eq!(time_array.len(), 3);

            // Verify timestamps are correctly parsed (RFC3164 assumes current year)
            let current_year = Utc::now().year();
            let expected_timestamp_1 = Local
                .from_local_datetime(
                    &chrono::NaiveDate::from_ymd_opt(current_year, 10, 11)
                        .unwrap()
                        .and_hms_opt(22, 14, 15)
                        .unwrap(),
                )
                .unwrap()
                .with_timezone(&Utc)
                .timestamp_nanos_opt()
                .unwrap();
            let expected_timestamp_2 = Local
                .from_local_datetime(
                    &chrono::NaiveDate::from_ymd_opt(current_year, 2, 5)
                        .unwrap()
                        .and_hms_opt(17, 32, 18)
                        .unwrap(),
                )
                .unwrap()
                .with_timezone(&Utc)
                .timestamp_nanos_opt()
                .unwrap();
            let expected_timestamp_3 = Local
                .from_local_datetime(
                    &chrono::NaiveDate::from_ymd_opt(current_year, 1, 15)
                        .unwrap()
                        .and_hms_opt(10, 30, 45)
                        .unwrap(),
                )
                .unwrap()
                .with_timezone(&Utc)
                .timestamp_nanos_opt()
                .unwrap();

            assert_eq!(time_array.value(0), expected_timestamp_1);
            assert_eq!(time_array.value(1), expected_timestamp_2);
            assert_eq!(time_array.value(2), expected_timestamp_3);

            // Check observed_time_unix_nano column (should be between start and end time)
            let observed_time_column = logs_batch
                .column_by_name("observed_time_unix_nano")
                .expect("Observed time column should exist");
            let observed_time_array = observed_time_column
                .as_any()
                .downcast_ref::<TimestampNanosecondArray>()
                .expect("Observed time should be TimestampNanosecondArray");

            validate_observed_timestamp_range(start_time, end_time, observed_time_array, 3);

            // Check that other columns exist and have the right length
            assert_eq!(
                logs_batch
                    .column_by_name("id")
                    .expect("ID column should exist")
                    .len(),
                3
            );
            assert_eq!(
                logs_batch
                    .column_by_name("resource")
                    .expect("Resource column should exist")
                    .len(),
                3
            );
            assert_eq!(
                logs_batch
                    .column_by_name("scope")
                    .expect("Scope column should exist")
                    .len(),
                3
            );

            // Check attributes record batch using Arrow APIs directly
            if let Some(attrs_batch) = arrow_records.get(ArrowPayloadType::LogAttrs) {
                assert!(
                    attrs_batch.num_rows() > 0,
                    "Should have attributes for RFC3164 messages"
                );

                // Verify attributes schema
                let attrs_schema = attrs_batch.schema();
                let attrs_column_names: Vec<&str> = attrs_schema
                    .fields()
                    .iter()
                    .map(|f| f.name().as_str())
                    .collect();

                assert!(attrs_column_names.contains(&"parent_id"));
                assert!(attrs_column_names.contains(&"key"));
                assert!(attrs_column_names.contains(&"type"));
                assert!(attrs_column_names.contains(&"str"));

                // Get the columns for attribute validation
                let parent_id_column = attrs_batch
                    .column_by_name("parent_id")
                    .expect("Parent ID column should exist");
                let key_column = attrs_batch
                    .column_by_name("key")
                    .expect("Key column should exist");
                let type_column = attrs_batch
                    .column_by_name("type")
                    .expect("Type column should exist");
                let str_column = attrs_batch
                    .column_by_name("str")
                    .expect("String column should exist");

                // Parent ID should be UInt16Array
                let parent_ids = parent_id_column
                    .as_any()
                    .downcast_ref::<UInt16Array>()
                    .expect("Parent ID should be UInt16Array");

                // Key column is a Dictionary(UInt8, Utf8)
                let key_dict = key_column
                    .as_any()
                    .downcast_ref::<DictionaryArray<UInt8Type>>()
                    .expect("Key should be DictionaryArray");
                let key_values = key_dict
                    .values()
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .expect("Key dictionary values should be StringArray");

                // Type column indicates the value type
                let type_array = type_column
                    .as_any()
                    .downcast_ref::<UInt8Array>()
                    .expect("Type should be UInt8Array");

                // String value column should be a Dictionary(UInt16, Utf8)
                let str_dict = str_column
                    .as_any()
                    .downcast_ref::<DictionaryArray<UInt16Type>>()
                    .expect("String value should be DictionaryArray");
                let str_values = str_dict
                    .values()
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .expect("String dictionary values should be StringArray");

                // Check if there's an int column for integer values
                let int_column = attrs_batch.column_by_name("int");
                let int_dict = int_column
                    .and_then(|col| col.as_any().downcast_ref::<DictionaryArray<UInt16Type>>());
                let int_values =
                    int_dict.and_then(|dict| dict.values().as_any().downcast_ref::<Int64Array>());

                // Build a map of log_id -> {key -> value} for easier validation
                let mut log_attributes: std::collections::HashMap<
                    u16,
                    std::collections::HashMap<String, AttributeValue>,
                > = std::collections::HashMap::new();

                for i in 0..attrs_batch.num_rows() {
                    let parent_id = parent_ids.value(i);
                    let key_index = key_dict.key(i).unwrap();
                    let key = key_values.value(key_index);
                    let value_type = type_array.value(i);

                    // Extract the appropriate value based on type (RFC3164 includes integer types for facility/severity)
                    let value = match value_type {
                        1 => {
                            // String type
                            if !str_dict.is_null(i) {
                                let str_index = str_dict.key(i).unwrap();
                                AttributeValue::String(str_values.value(str_index).to_string())
                            } else {
                                AttributeValue::String(String::new())
                            }
                        }
                        2 => {
                            // Integer type
                            if let (Some(int_dict), Some(int_vals)) = (int_dict, int_values) {
                                if !int_dict.is_null(i) {
                                    let int_index = int_dict.key(i).unwrap();
                                    AttributeValue::Integer(int_vals.value(int_index))
                                } else {
                                    AttributeValue::String(String::new())
                                }
                            } else {
                                AttributeValue::String(String::new())
                            }
                        }
                        _ => {
                            // Handle other types if needed
                            AttributeValue::String(String::new())
                        }
                    };

                    _ = log_attributes
                        .entry(parent_id)
                        .or_default()
                        .insert(key.to_string(), value);
                }

                // Validate attributes for each log record (RFC3164 format)
                // Note: parent_id 0 corresponds to the first log record, 1 to the second, etc.

                // Check first log record attributes (RFC3164)
                // Message: "<34>Oct 11 22:14:15 mymachine su: 'su root' failed for lonvick on /dev/pts/8"
                let Some(log1_attrs) = log_attributes.get(&0) else {
                    panic!("Expected attributes for first log record");
                };

                assert_eq!(
                    log1_attrs.get("syslog.facility"),
                    Some(&AttributeValue::Integer(4))
                );
                assert_eq!(
                    log1_attrs.get("syslog.severity"),
                    Some(&AttributeValue::Integer(2))
                );
                assert_eq!(
                    log1_attrs.get("syslog.host_name"),
                    Some(&AttributeValue::String("mymachine".to_string()))
                );
                assert_eq!(
                    log1_attrs.get("syslog.tag"),
                    Some(&AttributeValue::String("su".to_string()))
                );
                assert_eq!(
                    log1_attrs.get("syslog.message"),
                    Some(&AttributeValue::String(
                        "'su root' failed for lonvick on /dev/pts/8".to_string()
                    ))
                );

                // Check that we have exactly the expected number of attributes (5 for RFC3164)
                assert_eq!(
                    log1_attrs.len(),
                    5,
                    "Log 1 should have exactly 5 attributes, got {}",
                    log1_attrs.len()
                );

                // Check second log record attributes (RFC3164 with process ID in tag)
                // Message: "<165>Feb  5 17:32:18 hostname app[1234]: Application started successfully"
                let Some(log2_attrs) = log_attributes.get(&1) else {
                    panic!("Expected attributes for second log record");
                };

                assert_eq!(
                    log2_attrs.get("syslog.facility"),
                    Some(&AttributeValue::Integer(20))
                );
                assert_eq!(
                    log2_attrs.get("syslog.severity"),
                    Some(&AttributeValue::Integer(5))
                );
                assert_eq!(
                    log2_attrs.get("syslog.host_name"),
                    Some(&AttributeValue::String("hostname".to_string()))
                );
                assert_eq!(
                    log2_attrs.get("syslog.tag"),
                    Some(&AttributeValue::String("app[1234]".to_string()))
                );
                assert_eq!(
                    log2_attrs.get("syslog.message"),
                    Some(&AttributeValue::String(
                        "Application started successfully".to_string()
                    ))
                );

                // Check that we have exactly the expected number of attributes (5 for RFC3164)
                assert_eq!(
                    log2_attrs.len(),
                    5,
                    "Log 2 should have exactly 5 attributes, got {}",
                    log2_attrs.len()
                );

                // Check third log record attributes (RFC3164)
                // Message: "<14>Jan 15 10:30:45 server01 kernel: Kernel panic - not syncing: VFS"
                let Some(log3_attrs) = log_attributes.get(&2) else {
                    panic!("Expected attributes for third log record");
                };

                assert_eq!(
                    log3_attrs.get("syslog.facility"),
                    Some(&AttributeValue::Integer(1))
                );
                assert_eq!(
                    log3_attrs.get("syslog.severity"),
                    Some(&AttributeValue::Integer(6))
                );
                assert_eq!(
                    log3_attrs.get("syslog.host_name"),
                    Some(&AttributeValue::String("server01".to_string()))
                );
                assert_eq!(
                    log3_attrs.get("syslog.tag"),
                    Some(&AttributeValue::String("kernel".to_string()))
                );
                assert_eq!(
                    log3_attrs.get("syslog.message"),
                    Some(&AttributeValue::String(
                        "Kernel panic - not syncing: VFS".to_string()
                    ))
                );

                // Check that we have exactly the expected number of attributes (5 for RFC3164)
                assert_eq!(
                    log3_attrs.len(),
                    5,
                    "Log 3 should have exactly 5 attributes, got {}",
                    log3_attrs.len()
                );
            }
        } else {
            panic!("Expected Logs record batch");
        }
    }

    #[test]
    fn test_cef_to_arrow_structure() {
        // This test validates that CEF (Common Event Format) messages are correctly converted to Arrow format
        // by directly inspecting the Arrow record batch structure without converting to OTLP.
        // CEF is a standard format for logging security events.

        let mut builder = ArrowRecordsBuilder::new();

        // Capture start time before processing messages
        let start_time = Utc::now().timestamp_nanos_opt().unwrap_or(0);

        // Sample CEF messages with varying complexity
        let cef_messages = vec![
            "CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232",
            "CEF:1|ArcSight|ArcSight|2.4.1|400|Successful Login|3|",
            "CEF:0|Vendor|Product|1.2.3|SignatureID|Event Name|5|deviceExternalId=12345 sourceAddress=192.168.1.100",
        ];

        // Parse and append each message
        for msg in cef_messages {
            let parsed = parse(msg.as_bytes()).unwrap();
            builder.append_syslog(parsed);
        }

        // Build arrow records and capture end time
        let arrow_records = builder.build().unwrap();
        let end_time = Utc::now().timestamp_nanos_opt().unwrap_or(0);

        // Extract the logs record batch
        if let OtapArrowRecords::Logs(_logs) = &arrow_records {
            let logs_batch = arrow_records
                .get(ArrowPayloadType::Logs)
                .expect("Logs record batch should be present");

            // Verify the number of rows (log records)
            assert_eq!(logs_batch.num_rows(), 3);

            // Get specific columns for detailed validation
            use arrow::array::*;
            use arrow::datatypes::{UInt8Type, UInt16Type};

            // Check body column (should contain the original CEF messages)
            let body_column = logs_batch
                .column_by_name("body")
                .expect("Body column should exist");

            // Body is actually a StructArray, extract the 'str' field
            let body_struct = body_column
                .as_any()
                .downcast_ref::<StructArray>()
                .expect("Body should be StructArray");
            let body_column_str = body_struct
                .column_by_name("str")
                .expect("Body struct should have str field");

            // The str field is a Dictionary(UInt16, Utf8), need to handle accordingly
            if let Some(dict_array) = body_column_str
                .as_any()
                .downcast_ref::<DictionaryArray<UInt16Type>>()
            {
                let values = dict_array
                    .values()
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .expect("Dictionary values should be StringArray");

                assert_eq!(dict_array.len(), 3);

                // Check each CEF message value
                for i in 0..3 {
                    let dict_index = dict_array.key(i).unwrap();
                    let value = values.value(dict_index);
                    match i {
                        0 => assert_eq!(
                            value,
                            "CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232"
                        ),
                        1 => assert_eq!(
                            value,
                            "CEF:1|ArcSight|ArcSight|2.4.1|400|Successful Login|3|"
                        ),
                        2 => assert_eq!(
                            value,
                            "CEF:0|Vendor|Product|1.2.3|SignatureID|Event Name|5|deviceExternalId=12345 sourceAddress=192.168.1.100"
                        ),
                        _ => panic!("Unexpected index"),
                    }
                }
            } else {
                panic!("Expected str field to be a dictionary array");
            }

            // CEF doesn't have severity_number, only severity_text
            // Check severity_text column
            let severity_text_column = logs_batch
                .column_by_name("severity_text")
                .expect("Severity text column should exist");

            // severity_text is a Dictionary(UInt8, Utf8), need to handle accordingly
            if let Some(dict_array) = severity_text_column
                .as_any()
                .downcast_ref::<DictionaryArray<UInt8Type>>()
            {
                let values = dict_array
                    .values()
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .expect("Dictionary values should be StringArray");
                assert_eq!(dict_array.len(), 3);
                for i in 0..3 {
                    let dict_index = dict_array.key(i).unwrap();
                    assert_eq!(values.value(dict_index), "UNSPECIFIED");
                }
            } else {
                // Fallback to regular StringArray
                let severity_text_array = severity_text_column
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .expect("Severity text should be StringArray");
                assert_eq!(severity_text_array.len(), 3);
                assert_eq!(severity_text_array.value(0), "UNSPECIFIED");
                assert_eq!(severity_text_array.value(1), "UNSPECIFIED");
                assert_eq!(severity_text_array.value(2), "UNSPECIFIED");
            }

            // Check time_unix_nano column (CEF doesn't have timestamps)
            let time_column = logs_batch
                .column_by_name("time_unix_nano")
                .expect("Time column should exist");

            // Time column is Timestamp(Nanosecond), need to handle accordingly
            if let Some(timestamp_array) = time_column
                .as_any()
                .downcast_ref::<TimestampNanosecondArray>()
            {
                assert_eq!(timestamp_array.len(), 3);
                // CEF messages have timestamp of 0 - just verify they exist
                for i in 0..3 {
                    // Timestamps should be 0 for CEF messages
                    if !timestamp_array.is_null(i) {
                        assert_eq!(timestamp_array.value(i), 0);
                    }
                }
            } else {
                // Fallback to Int64Array
                let time_array = time_column
                    .as_any()
                    .downcast_ref::<Int64Array>()
                    .expect("Time should be Int64Array or TimestampArray");
                assert_eq!(time_array.len(), 3);
                for i in 0..3 {
                    if !time_array.is_null(i) {
                        assert_eq!(time_array.value(i), 0);
                    }
                }
            }

            // Check observed_time_unix_nano column (should be between start and end time)
            let observed_time_column = logs_batch
                .column_by_name("observed_time_unix_nano")
                .expect("Observed time column should exist");

            // Observed time column is a Timestamp(Nanosecond)
            if let Some(timestamp_array) = observed_time_column
                .as_any()
                .downcast_ref::<TimestampNanosecondArray>()
            {
                validate_observed_timestamp_range(start_time, end_time, timestamp_array, 3);
            } else {
                panic!(
                    "Expected observed_time_unix_nano to be TimestampNanosecondArray, got: {:?}",
                    observed_time_column.data_type()
                );
            }

            // CEF doesn't have flags column either, skip that check

            // Check attributes record batch using Arrow APIs directly
            if let Some(attrs_batch) = arrow_records.get(ArrowPayloadType::LogAttrs) {
                assert!(
                    attrs_batch.num_rows() > 0,
                    "Should have attributes for CEF messages"
                );

                // Verify attributes schema
                let attrs_schema = attrs_batch.schema();
                let attrs_column_names: Vec<&str> = attrs_schema
                    .fields()
                    .iter()
                    .map(|f| f.name().as_str())
                    .collect();

                assert!(attrs_column_names.contains(&"parent_id"));
                assert!(attrs_column_names.contains(&"key"));
                assert!(attrs_column_names.contains(&"type"));
                assert!(attrs_column_names.contains(&"str"));

                // Get the columns for attribute validation
                let parent_id_column = attrs_batch
                    .column_by_name("parent_id")
                    .expect("Parent ID column should exist");
                let key_column = attrs_batch
                    .column_by_name("key")
                    .expect("Key column should exist");
                let type_column = attrs_batch
                    .column_by_name("type")
                    .expect("Type column should exist");
                let str_column = attrs_batch
                    .column_by_name("str")
                    .expect("String column should exist");

                // Parent ID should be UInt16Array
                let parent_ids = parent_id_column
                    .as_any()
                    .downcast_ref::<UInt16Array>()
                    .expect("Parent ID should be UInt16Array");

                // Key column is a Dictionary(UInt8, Utf8)
                let key_dict = key_column
                    .as_any()
                    .downcast_ref::<DictionaryArray<UInt8Type>>()
                    .expect("Key should be DictionaryArray");
                let key_values = key_dict
                    .values()
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .expect("Key dictionary values should be StringArray");

                // Type column indicates the value type
                let type_array = type_column
                    .as_any()
                    .downcast_ref::<UInt8Array>()
                    .expect("Type should be UInt8Array");

                // String value column should be a Dictionary(UInt16, Utf8)
                let str_dict = str_column
                    .as_any()
                    .downcast_ref::<DictionaryArray<UInt16Type>>()
                    .expect("String value should be DictionaryArray");
                let str_values = str_dict
                    .values()
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .expect("String dictionary values should be StringArray");

                // Check if there's an int column for integer values
                let int_column = attrs_batch.column_by_name("int");
                let int_dict = int_column
                    .and_then(|col| col.as_any().downcast_ref::<DictionaryArray<UInt16Type>>());
                let int_values =
                    int_dict.and_then(|dict| dict.values().as_any().downcast_ref::<Int64Array>());

                // Build a map of log_id -> {key -> value} for easier validation
                let mut log_attributes: std::collections::HashMap<
                    u16,
                    std::collections::HashMap<String, AttributeValue>,
                > = std::collections::HashMap::new();

                for i in 0..attrs_batch.num_rows() {
                    let parent_id = parent_ids.value(i);
                    let key_index = key_dict.key(i).unwrap();
                    let key = key_values.value(key_index);
                    let value_type = type_array.value(i);

                    // Extract the appropriate value based on type (CEF primarily uses strings)
                    let value = match value_type {
                        1 => {
                            // String type
                            if !str_dict.is_null(i) {
                                let str_index = str_dict.key(i).unwrap();
                                AttributeValue::String(str_values.value(str_index).to_string())
                            } else {
                                AttributeValue::String(String::new())
                            }
                        }
                        2 => {
                            // Integer type
                            if let (Some(int_dict), Some(int_vals)) = (int_dict, int_values) {
                                if !int_dict.is_null(i) {
                                    let int_index = int_dict.key(i).unwrap();
                                    AttributeValue::Integer(int_vals.value(int_index))
                                } else {
                                    AttributeValue::String(String::new())
                                }
                            } else {
                                AttributeValue::String(String::new())
                            }
                        }
                        _ => {
                            // Handle other types if needed
                            AttributeValue::String(String::new())
                        }
                    };

                    _ = log_attributes
                        .entry(parent_id)
                        .or_default()
                        .insert(key.to_string(), value);
                }

                // Validate attributes for each log record (CEF format)
                // Note: parent_id 0 corresponds to the first log record, 1 to the second, etc.

                // Check first CEF log record attributes (with extensions)
                // Message: "CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232"
                let Some(log1_attrs) = log_attributes.get(&0) else {
                    panic!("Expected attributes for first log record");
                };

                assert_eq!(
                    log1_attrs.get("cef.version"),
                    Some(&AttributeValue::Integer(0))
                );
                assert_eq!(
                    log1_attrs.get("cef.device_vendor"),
                    Some(&AttributeValue::String("Security".to_string()))
                );
                assert_eq!(
                    log1_attrs.get("cef.device_product"),
                    Some(&AttributeValue::String("threatmanager".to_string()))
                );
                assert_eq!(
                    log1_attrs.get("cef.device_version"),
                    Some(&AttributeValue::String("1.0".to_string()))
                );
                assert_eq!(
                    log1_attrs.get("cef.signature_id"),
                    Some(&AttributeValue::String("100".to_string()))
                );
                assert_eq!(
                    log1_attrs.get("cef.name"),
                    Some(&AttributeValue::String(
                        "worm successfully stopped".to_string()
                    ))
                );
                assert_eq!(
                    log1_attrs.get("cef.severity"),
                    Some(&AttributeValue::String("10".to_string()))
                );
                assert_eq!(
                    log1_attrs.get("src"),
                    Some(&AttributeValue::String("10.0.0.1".to_string()))
                );
                assert_eq!(
                    log1_attrs.get("dst"),
                    Some(&AttributeValue::String("2.1.2.2".to_string()))
                );
                assert_eq!(
                    log1_attrs.get("spt"),
                    Some(&AttributeValue::String("1232".to_string()))
                );

                // Check that we have exactly the expected number of attributes (7 core + 3 extensions = 10)
                assert_eq!(
                    log1_attrs.len(),
                    10,
                    "Log 1 should have exactly 10 attributes, got {}",
                    log1_attrs.len()
                );

                // Check second CEF log record attributes (no extensions)
                // Message: "CEF:1|ArcSight|ArcSight|2.4.1|400|Successful Login|3|"
                let Some(log2_attrs) = log_attributes.get(&1) else {
                    panic!("Expected attributes for second log record");
                };

                assert_eq!(
                    log2_attrs.get("cef.version"),
                    Some(&AttributeValue::Integer(1))
                );
                assert_eq!(
                    log2_attrs.get("cef.device_vendor"),
                    Some(&AttributeValue::String("ArcSight".to_string()))
                );
                assert_eq!(
                    log2_attrs.get("cef.device_product"),
                    Some(&AttributeValue::String("ArcSight".to_string()))
                );
                assert_eq!(
                    log2_attrs.get("cef.device_version"),
                    Some(&AttributeValue::String("2.4.1".to_string()))
                );
                assert_eq!(
                    log2_attrs.get("cef.signature_id"),
                    Some(&AttributeValue::String("400".to_string()))
                );
                assert_eq!(
                    log2_attrs.get("cef.name"),
                    Some(&AttributeValue::String("Successful Login".to_string()))
                );
                assert_eq!(
                    log2_attrs.get("cef.severity"),
                    Some(&AttributeValue::String("3".to_string()))
                );

                // Check that we have exactly the expected number of attributes (7 core attributes, no extensions)
                assert_eq!(
                    log2_attrs.len(),
                    7,
                    "Log 2 should have exactly 7 attributes, got {}",
                    log2_attrs.len()
                );

                // Check third CEF log record attributes (with limited extensions)
                // Message: "CEF:0|Vendor|Product|1.2.3|SignatureID|Event Name|5|deviceExternalId=12345 sourceAddress=192.168.1.100"
                let Some(log3_attrs) = log_attributes.get(&2) else {
                    panic!("Expected attributes for third log record");
                };

                assert_eq!(
                    log3_attrs.get("cef.version"),
                    Some(&AttributeValue::Integer(0))
                );
                assert_eq!(
                    log3_attrs.get("cef.device_vendor"),
                    Some(&AttributeValue::String("Vendor".to_string()))
                );
                assert_eq!(
                    log3_attrs.get("cef.device_product"),
                    Some(&AttributeValue::String("Product".to_string()))
                );
                assert_eq!(
                    log3_attrs.get("cef.device_version"),
                    Some(&AttributeValue::String("1.2.3".to_string()))
                );
                assert_eq!(
                    log3_attrs.get("cef.signature_id"),
                    Some(&AttributeValue::String("SignatureID".to_string()))
                );
                assert_eq!(
                    log3_attrs.get("cef.name"),
                    Some(&AttributeValue::String("Event Name".to_string()))
                );
                assert_eq!(
                    log3_attrs.get("cef.severity"),
                    Some(&AttributeValue::String("5".to_string()))
                );
                assert_eq!(
                    log3_attrs.get("deviceExternalId"),
                    Some(&AttributeValue::String("12345".to_string()))
                );
                assert_eq!(
                    log3_attrs.get("sourceAddress"),
                    Some(&AttributeValue::String("192.168.1.100".to_string()))
                );

                // Check that we have exactly the expected number of attributes (7 core + 2 extensions = 9)
                assert_eq!(
                    log3_attrs.len(),
                    9,
                    "Log 3 should have exactly 9 attributes, got {}",
                    log3_attrs.len()
                );
            }
        } else {
            panic!("Expected Logs record batch");
        }
    }

    #[test]
    fn test_mixed_format_messages_to_arrow_structure() {
        // This test validates that mixed message formats (RFC5424, RFC3164, and CEF)
        // can be processed together in a single batch and correctly converted to Arrow format
        // by directly inspecting the Arrow record batch structure without converting to OTLP.

        let mut builder = ArrowRecordsBuilder::new();

        // Capture start time before processing messages
        let start_time = Utc::now().timestamp_nanos_opt().unwrap_or(0);

        // Mixed format messages: RFC5424, RFC3164, and CEF
        let mixed_messages = vec![
            // RFC5424 syslog message
            "<165>1 2024-01-15T10:31:00.456Z host.example.com myapp 1234 ID123 [exampleSDID@32473 iut=\"3\"] Application started successfully",
            // RFC3164 syslog message
            "<34>Oct 11 22:14:15 mymachine su: 'su root' failed for lonvick on /dev/pts/8",
            // CEF message
            "CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232",
        ];

        // Parse and append each message
        for msg in mixed_messages {
            let parsed = parse(msg.as_bytes()).unwrap();
            builder.append_syslog(parsed);
        }

        // Build arrow records and capture end time
        let arrow_records = builder.build().unwrap();
        let end_time = Utc::now().timestamp_nanos_opt().unwrap_or(0);

        // Extract the logs record batch
        if let OtapArrowRecords::Logs(_logs) = &arrow_records {
            let logs_batch = arrow_records
                .get(ArrowPayloadType::Logs)
                .expect("Logs record batch should be present");

            // Verify the number of rows (log records)
            assert_eq!(logs_batch.num_rows(), 3);

            // Get specific columns for detailed validation
            use arrow::array::*;
            use arrow::datatypes::{UInt8Type, UInt16Type};

            // Check body column (should contain the original messages)
            let body_column = logs_batch
                .column_by_name("body")
                .expect("Body column should exist");

            // Body is actually a StructArray, just verify it exists and has correct length
            let body_struct = body_column
                .as_any()
                .downcast_ref::<StructArray>()
                .expect("Body should be StructArray");
            assert_eq!(body_struct.len(), 3);

            // Skip detailed body content validation for mixed format since it's complex

            // Check severity_number column (mixed formats may not all have this)
            if let Some(severity_num_column) = logs_batch.column_by_name("severity_number") {
                // severity_number is a Dictionary(UInt8, Int32), need to handle accordingly
                if let Some(dict_array) = severity_num_column
                    .as_any()
                    .downcast_ref::<DictionaryArray<UInt8Type>>()
                {
                    let values = dict_array
                        .values()
                        .as_any()
                        .downcast_ref::<Int32Array>()
                        .expect("Dictionary values should be Int32Array");
                    assert_eq!(dict_array.len(), 3);
                    // Mixed format - just check that we have valid severity values
                    for i in 0..3 {
                        let dict_index = dict_array.key(i).unwrap();
                        let severity = values.value(dict_index);
                        assert!(
                            (0..=23).contains(&severity),
                            "Severity should be valid OpenTelemetry value"
                        );
                    }
                } else {
                    // Fallback to regular Int32Array
                    let severity_num_array = severity_num_column
                        .as_any()
                        .downcast_ref::<Int32Array>()
                        .expect("Severity number should be Int32Array");
                    assert_eq!(severity_num_array.len(), 3);
                    for i in 0..3 {
                        let severity = severity_num_array.value(i);
                        assert!(
                            (0..=23).contains(&severity),
                            "Severity should be valid OpenTelemetry value"
                        );
                    }
                }
            }

            // Check severity_text column
            let severity_text_column = logs_batch
                .column_by_name("severity_text")
                .expect("Severity text column should exist");

            // severity_text is a Dictionary(UInt8, Utf8), need to handle accordingly
            if let Some(dict_array) = severity_text_column
                .as_any()
                .downcast_ref::<DictionaryArray<UInt8Type>>()
            {
                let values = dict_array
                    .values()
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .expect("Dictionary values should be StringArray");
                assert_eq!(dict_array.len(), 3);
                // Mixed format - just verify we have valid severity text values
                for i in 0..3 {
                    let dict_index = dict_array.key(i).unwrap();
                    let severity_text = values.value(dict_index);
                    assert!(
                        !severity_text.is_empty(),
                        "Severity text should not be empty"
                    );
                }
            } else {
                // Fallback to regular StringArray
                let severity_text_array = severity_text_column
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .expect("Severity text should be StringArray");
                assert_eq!(severity_text_array.len(), 3);
                for i in 0..3 {
                    let severity_text = severity_text_array.value(i);
                    assert!(
                        !severity_text.is_empty(),
                        "Severity text should not be empty"
                    );
                }
            }

            // Check time_unix_nano column (mixed formats have different timestamp handling)
            let time_column = logs_batch
                .column_by_name("time_unix_nano")
                .expect("Time column should exist");

            // Time column is Timestamp(Nanosecond), need to handle accordingly
            if let Some(timestamp_array) = time_column
                .as_any()
                .downcast_ref::<TimestampNanosecondArray>()
            {
                assert_eq!(timestamp_array.len(), 3);

                // Just verify timestamps exist for mixed format - don't check specific values
                // since mixed formats have complex timestamp handling
                for i in 0..3 {
                    // All formats should have some timestamp (even if 0 for CEF)
                    if !timestamp_array.is_null(i) {
                        let timestamp = timestamp_array.value(i);
                        assert!(timestamp >= 0, "Timestamp should be non-negative");
                    }
                }
            } else {
                // Fallback to Int64Array
                let time_array = time_column
                    .as_any()
                    .downcast_ref::<Int64Array>()
                    .expect("Time should be Int64Array or TimestampArray");
                assert_eq!(time_array.len(), 3);
                for i in 0..3 {
                    if !time_array.is_null(i) {
                        let timestamp = time_array.value(i);
                        assert!(timestamp >= 0, "Timestamp should be non-negative");
                    }
                }
            }

            // Check observed_time_unix_nano column (should be between start and end time)
            let observed_time_column = logs_batch
                .column_by_name("observed_time_unix_nano")
                .expect("Observed time column should exist");

            if let Some(timestamp_array) = observed_time_column
                .as_any()
                .downcast_ref::<TimestampNanosecondArray>()
            {
                validate_observed_timestamp_range(start_time, end_time, timestamp_array, 3);
            } else {
                panic!(
                    "Expected observed_time_unix_nano to be TimestampNanosecondArray, got: {:?}",
                    observed_time_column.data_type()
                );
            }

            // Verify that essential columns are present (only check ones that exist in mixed format)
            let schema = logs_batch.schema();
            let column_names: Vec<&str> =
                schema.fields().iter().map(|f| f.name().as_str()).collect();

            assert!(column_names.contains(&"time_unix_nano"));
            assert!(column_names.contains(&"severity_text"));
            assert!(column_names.contains(&"body"));
            // Note: severity_number, flags, trace_id, span_id may not exist in mixed format due to CEF

            // Check attributes record batch using Arrow APIs directly (mixed formats)
            if let Some(attrs_batch) = arrow_records.get(ArrowPayloadType::LogAttrs) {
                assert!(
                    attrs_batch.num_rows() > 0,
                    "Should have attributes for mixed messages"
                );

                // Verify attributes schema
                let attrs_schema = attrs_batch.schema();
                let attrs_column_names: Vec<&str> = attrs_schema
                    .fields()
                    .iter()
                    .map(|f| f.name().as_str())
                    .collect();

                assert!(attrs_column_names.contains(&"parent_id"));
                assert!(attrs_column_names.contains(&"key"));
                assert!(attrs_column_names.contains(&"type"));
                assert!(attrs_column_names.contains(&"str"));

                // Get the columns for attribute validation
                let parent_id_column = attrs_batch
                    .column_by_name("parent_id")
                    .expect("Parent ID column should exist");
                let key_column = attrs_batch
                    .column_by_name("key")
                    .expect("Key column should exist");
                let type_column = attrs_batch
                    .column_by_name("type")
                    .expect("Type column should exist");
                let str_column = attrs_batch
                    .column_by_name("str")
                    .expect("String column should exist");

                // Parent ID should be UInt16Array
                let parent_ids = parent_id_column
                    .as_any()
                    .downcast_ref::<UInt16Array>()
                    .expect("Parent ID should be UInt16Array");

                // Key column is a Dictionary(UInt8, Utf8)
                let key_dict = key_column
                    .as_any()
                    .downcast_ref::<DictionaryArray<UInt8Type>>()
                    .expect("Key should be DictionaryArray");
                let key_values = key_dict
                    .values()
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .expect("Key dictionary values should be StringArray");

                // Type column indicates the value type
                let type_array = type_column
                    .as_any()
                    .downcast_ref::<UInt8Array>()
                    .expect("Type should be UInt8Array");

                // String value column should be a Dictionary(UInt16, Utf8)
                let str_dict = str_column
                    .as_any()
                    .downcast_ref::<DictionaryArray<UInt16Type>>()
                    .expect("String value should be DictionaryArray");
                let str_values = str_dict
                    .values()
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .expect("String dictionary values should be StringArray");

                // Check if there's an int column for integer values
                let int_column = attrs_batch.column_by_name("int");
                let int_dict = int_column
                    .and_then(|col| col.as_any().downcast_ref::<DictionaryArray<UInt16Type>>());
                let int_values =
                    int_dict.and_then(|dict| dict.values().as_any().downcast_ref::<Int64Array>());

                // Build a map of log_id -> {key -> value} for easier validation
                let mut log_attributes: std::collections::HashMap<
                    u16,
                    std::collections::HashMap<String, AttributeValue>,
                > = std::collections::HashMap::new();

                for i in 0..attrs_batch.num_rows() {
                    let parent_id = parent_ids.value(i);
                    let key_index = key_dict.key(i).unwrap();
                    let key = key_values.value(key_index);
                    let value_type = type_array.value(i);

                    // Extract the appropriate value based on type
                    let value = match value_type {
                        1 => {
                            // String type
                            if !str_dict.is_null(i) {
                                let str_index = str_dict.key(i).unwrap();
                                AttributeValue::String(str_values.value(str_index).to_string())
                            } else {
                                AttributeValue::String(String::new())
                            }
                        }
                        2 => {
                            // Integer type
                            if let (Some(int_dict), Some(int_vals)) = (int_dict, int_values) {
                                if !int_dict.is_null(i) {
                                    let int_index = int_dict.key(i).unwrap();
                                    AttributeValue::Integer(int_vals.value(int_index))
                                } else {
                                    AttributeValue::String(String::new())
                                }
                            } else {
                                AttributeValue::String(String::new())
                            }
                        }
                        _ => {
                            // Handle other types if needed
                            AttributeValue::String(String::new())
                        }
                    };

                    _ = log_attributes
                        .entry(parent_id)
                        .or_default()
                        .insert(key.to_string(), value);
                }

                // Validate attributes for each log record (mixed formats)
                // Note: parent_id 0 corresponds to the first log record, 1 to the second, etc.

                // Check first log record attributes (RFC5424)
                // Message: "<165>1 2024-01-15T10:31:00.456Z host.example.com myapp 1234 ID123 [exampleSDID@32473 iut=\"3\"] Application started successfully"
                let Some(log1_attrs) = log_attributes.get(&0) else {
                    panic!("Expected attributes for first log record");
                };

                assert_eq!(
                    log1_attrs.get("syslog.version"),
                    Some(&AttributeValue::Integer(1))
                );
                assert_eq!(
                    log1_attrs.get("syslog.facility"),
                    Some(&AttributeValue::Integer(20))
                );
                assert_eq!(
                    log1_attrs.get("syslog.severity"),
                    Some(&AttributeValue::Integer(5))
                );
                assert_eq!(
                    log1_attrs.get("syslog.host_name"),
                    Some(&AttributeValue::String("host.example.com".to_string()))
                );
                assert_eq!(
                    log1_attrs.get("syslog.app_name"),
                    Some(&AttributeValue::String("myapp".to_string()))
                );
                assert_eq!(
                    log1_attrs.get("syslog.process_id"),
                    Some(&AttributeValue::Integer(1234))
                );
                assert_eq!(
                    log1_attrs.get("syslog.msg_id"),
                    Some(&AttributeValue::String("ID123".to_string()))
                );
                assert_eq!(
                    log1_attrs.get("syslog.structured_data"),
                    Some(&AttributeValue::String(
                        "[exampleSDID@32473 iut=\"3\"]".to_string()
                    ))
                );
                assert_eq!(
                    log1_attrs.get("syslog.message"),
                    Some(&AttributeValue::String(
                        "Application started successfully".to_string()
                    ))
                );

                // Check that we have exactly the expected number of attributes for RFC5424 (9)
                assert_eq!(
                    log1_attrs.len(),
                    9,
                    "Log 1 should have exactly 9 attributes, got {}",
                    log1_attrs.len()
                );

                // Check second log record attributes (RFC3164)
                // Message: "<34>Oct 11 22:14:15 mymachine su: 'su root' failed for lonvick on /dev/pts/8"
                let Some(log2_attrs) = log_attributes.get(&1) else {
                    panic!("Expected attributes for second log record");
                };

                assert_eq!(
                    log2_attrs.get("syslog.facility"),
                    Some(&AttributeValue::Integer(4))
                );
                assert_eq!(
                    log2_attrs.get("syslog.severity"),
                    Some(&AttributeValue::Integer(2))
                );
                assert_eq!(
                    log2_attrs.get("syslog.host_name"),
                    Some(&AttributeValue::String("mymachine".to_string()))
                );
                assert_eq!(
                    log2_attrs.get("syslog.tag"),
                    Some(&AttributeValue::String("su".to_string()))
                );
                assert_eq!(
                    log2_attrs.get("syslog.message"),
                    Some(&AttributeValue::String(
                        "'su root' failed for lonvick on /dev/pts/8".to_string()
                    ))
                );

                // Check that we have exactly the expected number of attributes for RFC3164 (5)
                assert_eq!(
                    log2_attrs.len(),
                    5,
                    "Log 2 should have exactly 5 attributes, got {}",
                    log2_attrs.len()
                );

                // Check third log record attributes (CEF)
                // Message: "CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232"
                let Some(log3_attrs) = log_attributes.get(&2) else {
                    panic!("Expected attributes for third log record");
                };

                assert_eq!(
                    log3_attrs.get("cef.version"),
                    Some(&AttributeValue::Integer(0))
                );
                assert_eq!(
                    log3_attrs.get("cef.device_vendor"),
                    Some(&AttributeValue::String("Security".to_string()))
                );
                assert_eq!(
                    log3_attrs.get("cef.device_product"),
                    Some(&AttributeValue::String("threatmanager".to_string()))
                );
                assert_eq!(
                    log3_attrs.get("cef.device_version"),
                    Some(&AttributeValue::String("1.0".to_string()))
                );
                assert_eq!(
                    log3_attrs.get("cef.signature_id"),
                    Some(&AttributeValue::String("100".to_string()))
                );
                assert_eq!(
                    log3_attrs.get("cef.name"),
                    Some(&AttributeValue::String(
                        "worm successfully stopped".to_string()
                    ))
                );
                assert_eq!(
                    log3_attrs.get("cef.severity"),
                    Some(&AttributeValue::String("10".to_string()))
                );
                assert_eq!(
                    log3_attrs.get("src"),
                    Some(&AttributeValue::String("10.0.0.1".to_string()))
                );
                assert_eq!(
                    log3_attrs.get("dst"),
                    Some(&AttributeValue::String("2.1.2.2".to_string()))
                );
                assert_eq!(
                    log3_attrs.get("spt"),
                    Some(&AttributeValue::String("1232".to_string()))
                );

                // Check that we have exactly the expected number of attributes for CEF (7 core + 3 extensions = 10)
                assert_eq!(
                    log3_attrs.len(),
                    10,
                    "Log 3 should have exactly 10 attributes, got {}",
                    log3_attrs.len()
                );

                // Verify that we have both syslog and CEF attributes in the batch
                let has_syslog_attrs = log_attributes
                    .values()
                    .any(|attrs| attrs.keys().any(|k| k.starts_with("syslog.")));
                let has_cef_attrs = log_attributes
                    .values()
                    .any(|attrs| attrs.keys().any(|k| k.starts_with("cef.")));
                assert!(has_syslog_attrs, "Should have syslog attributes");
                assert!(has_cef_attrs, "Should have CEF attributes");
            }
        } else {
            panic!("Expected Logs record batch");
        }
    }

    #[test]
    fn test_rfc5424_syslog_to_arrow_to_otlp() {
        // Update the test comment to reflect what we're now testing comprehensively
        // This test validates that RFC 5424 syslog messages are correctly parsed,
        // converted to Arrow format, and then converted back to OTLP format.
        // It includes comprehensive assertions checking:
        // 1. Which attributes SHOULD be present for each message type
        // 2. That the total number of attributes matches expectations (ensuring no unexpected attributes are present)
        // 3. All log record fields including body, timestamps, severity, and optional fields
        // 4. Proper handling of None/empty values for trace_id, span_id, flags, etc.

        let mut builder = ArrowRecordsBuilder::new();

        // Capture start time before processing messages
        let start_time = Utc::now().timestamp_nanos_opt().unwrap_or(0);

        // Sample RFC 5424 syslog messages
        let syslog_messages = vec![
            "<34>1 2024-01-15T10:30:45.123Z mymachine.example.com su - ID47 - 'su root' failed for lonvick on /dev/pts/8",
            "<165>1 2024-01-15T10:31:00.456Z host.example.com myapp 1234 ID123 [exampleSDID@32473 iut=\"3\" eventSource=\"Application\" eventID=\"1011\"] Application started successfully",
            "<14>1 2024-01-15T10:32:15.789Z server01.example.com kernel - - - Kernel panic - not syncing: VFS",
        ];

        // Parse and append each message
        for msg in syslog_messages {
            let parsed = parse(msg.as_bytes()).unwrap();
            builder.append_syslog(parsed);
        }

        // Build arrow records and capture end time
        let arrow_records = builder.build().unwrap();
        let end_time = Utc::now().timestamp_nanos_opt().unwrap_or(0);

        // Convert to ExportLogsServiceRequest
        let export_request = logs_from(arrow_records).unwrap();

        // Verify we have the expected number of resource logs
        assert_eq!(export_request.resource_logs.len(), 1);

        let resource_logs = &export_request.resource_logs[0];
        assert_eq!(resource_logs.scope_logs.len(), 1);

        let scope_logs = &resource_logs.scope_logs[0];
        assert_eq!(scope_logs.log_records.len(), 3);

        // Verify first log record
        let log1 = &scope_logs.log_records[0];
        assert_eq!(log1.body.as_ref().unwrap().value.as_ref().unwrap(),
                   &Value::StringValue(
                       "<34>1 2024-01-15T10:30:45.123Z mymachine.example.com su - ID47 - 'su root' failed for lonvick on /dev/pts/8".to_string()
                   ));

        // Priority = Facility * 8 + Severity
        // Priority 34: 4 (mail system) * 8 + 2 (critical)
        // RFC5424 severity 2 (critical) maps to OpenTelemetry severity 18 (ERROR2)
        assert_eq!(log1.severity_number, 18);
        assert_eq!(log1.severity_text, "ERROR2");
        // Parse timestamp from message: 2024-01-15T10:30:45.123Z
        let expected_timestamp = DateTime::parse_from_rfc3339("2024-01-15T10:30:45.123Z")
            .unwrap()
            .timestamp_nanos_opt()
            .unwrap() as u64;
        assert_eq!(log1.time_unix_nano, expected_timestamp);

        // Verify fields that should be None/0 for log1
        validate_otlp_observed_timestamp_range(start_time, end_time, log1.observed_time_unix_nano);
        assert_eq!(log1.dropped_attributes_count, 0);
        assert_eq!(log1.flags, 0);
        assert!(log1.trace_id.is_empty());
        assert!(log1.span_id.is_empty());

        // Verify second log record with structured data
        let log2 = &scope_logs.log_records[1];
        assert_eq!(log2.body.as_ref().unwrap().value.as_ref().unwrap(),
                   &Value::StringValue(
                       "<165>1 2024-01-15T10:31:00.456Z host.example.com myapp 1234 ID123 [exampleSDID@32473 iut=\"3\" eventSource=\"Application\" eventID=\"1011\"] Application started successfully".to_string()
                   ));

        // Priority = Facility * 8 + Severity
        // Priority 165: 20 (local use 4) * 8 + 5 (notice)
        // RFC5424 severity 5 (notice) maps to OpenTelemetry severity 10 (INFO2)
        assert_eq!(log2.severity_number, 10); // Notice (RFC5424 severity=5) -> INFO2 (OTel severity=10)
        assert_eq!(log2.severity_text, "INFO2");
        // Parse timestamp from message: 2024-01-15T10:31:00.456Z
        let expected_timestamp_2 = DateTime::parse_from_rfc3339("2024-01-15T10:31:00.456Z")
            .unwrap()
            .timestamp_nanos_opt()
            .unwrap() as u64;
        assert_eq!(log2.time_unix_nano, expected_timestamp_2);

        // Verify fields that should be None/0 for log2
        validate_otlp_observed_timestamp_range(start_time, end_time, log2.observed_time_unix_nano);
        assert_eq!(log2.dropped_attributes_count, 0);
        assert_eq!(log2.flags, 0);
        assert!(log2.trace_id.is_empty());
        assert!(log2.span_id.is_empty());

        // Verify first log record attributes
        let log1_attrs: std::collections::HashMap<String, String> = log1
            .attributes
            .iter()
            .map(|kv| {
                (
                    kv.key.clone(),
                    match &kv.value.as_ref().unwrap().value.as_ref().unwrap() {
                        Value::StringValue(s) => s.clone(),
                        Value::IntValue(i) => i.to_string(),
                        _ => String::new(),
                    },
                )
            })
            .collect();

        // Check attributes that SHOULD be present for log1
        assert_eq!(log1_attrs.get("syslog.version"), Some(&"1".to_string()));
        assert_eq!(log1_attrs.get("syslog.facility"), Some(&"4".to_string()));
        assert_eq!(log1_attrs.get("syslog.severity"), Some(&"2".to_string()));
        assert_eq!(
            log1_attrs.get("syslog.host_name"),
            Some(&"mymachine.example.com".to_string())
        );
        assert_eq!(log1_attrs.get("syslog.app_name"), Some(&"su".to_string()));
        assert_eq!(log1_attrs.get("syslog.msg_id"), Some(&"ID47".to_string()));
        assert_eq!(
            log1_attrs.get("syslog.message"),
            Some(&"'su root' failed for lonvick on /dev/pts/8".to_string())
        );

        // Ensure no unexpected attributes are present (exactly 7 attributes expected)
        assert_eq!(log1.attributes.len(), 7);

        // Verify second log record with structured data
        let log2_attrs: std::collections::HashMap<String, String> = log2
            .attributes
            .iter()
            .map(|kv| {
                (
                    kv.key.clone(),
                    match &kv.value.as_ref().unwrap().value.as_ref().unwrap() {
                        Value::StringValue(s) => s.clone(),
                        Value::IntValue(i) => i.to_string(),
                        _ => String::new(),
                    },
                )
            })
            .collect();

        // Check attributes that SHOULD be present for log2
        assert_eq!(log2_attrs.get("syslog.version"), Some(&"1".to_string()));
        assert_eq!(log2_attrs.get("syslog.facility"), Some(&"20".to_string()));
        assert_eq!(log2_attrs.get("syslog.severity"), Some(&"5".to_string()));
        assert_eq!(
            log2_attrs.get("syslog.host_name"),
            Some(&"host.example.com".to_string())
        );
        assert_eq!(
            log2_attrs.get("syslog.app_name"),
            Some(&"myapp".to_string())
        );
        assert_eq!(
            log2_attrs.get("syslog.process_id"),
            Some(&"1234".to_string())
        );
        assert_eq!(log2_attrs.get("syslog.msg_id"), Some(&"ID123".to_string()));
        assert_eq!(
            log2_attrs.get("syslog.structured_data"),
            Some(
                &"[exampleSDID@32473 iut=\"3\" eventSource=\"Application\" eventID=\"1011\"]"
                    .to_string()
            )
        );
        assert_eq!(
            log2_attrs.get("syslog.message"),
            Some(&"Application started successfully".to_string())
        );

        // Ensure no unexpected attributes are present (exactly 9 attributes expected)
        assert_eq!(log2.attributes.len(), 9);

        // Verify third log record
        let log3 = &scope_logs.log_records[2];
        assert_eq!(log3.body.as_ref().unwrap().value.as_ref().unwrap(),
                   &Value::StringValue(
                       "<14>1 2024-01-15T10:32:15.789Z server01.example.com kernel - - - Kernel panic - not syncing: VFS".to_string()
                   ));

        let log3_attrs: std::collections::HashMap<String, String> = log3
            .attributes
            .iter()
            .map(|kv| {
                (
                    kv.key.clone(),
                    match &kv.value.as_ref().unwrap().value.as_ref().unwrap() {
                        Value::StringValue(s) => s.clone(),
                        Value::IntValue(i) => i.to_string(),
                        _ => String::new(),
                    },
                )
            })
            .collect();

        // Check attributes that SHOULD be present for log3
        assert_eq!(log3_attrs.get("syslog.version"), Some(&"1".to_string()));
        assert_eq!(log3_attrs.get("syslog.facility"), Some(&"1".to_string()));
        assert_eq!(log3_attrs.get("syslog.severity"), Some(&"6".to_string()));
        assert_eq!(
            log3_attrs.get("syslog.host_name"),
            Some(&"server01.example.com".to_string())
        );
        assert_eq!(
            log3_attrs.get("syslog.app_name"),
            Some(&"kernel".to_string())
        );
        assert_eq!(
            log3_attrs.get("syslog.message"),
            Some(&"Kernel panic - not syncing: VFS".to_string())
        );

        // // Ensure no unexpected attributes are present (exactly 6 attributes expected)
        assert_eq!(log3.attributes.len(), 6);

        // Priority = Facility * 8 + Severity
        // Priority 14: 1 (user level) * 8 + 6 (informational)
        // RFC5424 severity 6 (informational) maps to OpenTelemetry severity 9 (INFO)
        assert_eq!(log3.severity_number, 9); // Informational (RFC5424 severity=6) -> INFO (OTel severity=9)
        assert_eq!(log3.severity_text, "INFO");
        // Parse timestamp from message: 2024-01-15T10:32:15.789Z
        let expected_timestamp_3 = DateTime::parse_from_rfc3339("2024-01-15T10:32:15.789Z")
            .unwrap()
            .timestamp_nanos_opt()
            .unwrap() as u64;
        assert_eq!(log3.time_unix_nano, expected_timestamp_3);

        // Verify fields that should be None/0 for log3
        validate_otlp_observed_timestamp_range(start_time, end_time, log3.observed_time_unix_nano);
        assert_eq!(log3.dropped_attributes_count, 0);
        assert_eq!(log3.flags, 0);
        assert!(log3.trace_id.is_empty());
        assert!(log3.span_id.is_empty());
    }

    #[test]
    fn test_rfc3164_syslog_to_arrow_to_otlp() {
        // This test validates that RFC 3164 syslog messages are correctly parsed,
        // converted to Arrow format, and then converted back to OTLP format.
        // RFC 3164 is the traditional syslog format without version numbers or structured data.
        // It includes comprehensive assertions checking:
        // 1. Which attributes SHOULD be present for each message type
        // 2. That the total number of attributes matches expectations (ensuring no unexpected attributes are present)
        // 3. All log record fields including body, timestamps, severity, and optional fields
        // 4. Proper handling of None/empty values for trace_id, span_id, flags, etc.

        let mut builder = ArrowRecordsBuilder::new();

        // Capture start time before processing messages
        let start_time = Utc::now().timestamp_nanos_opt().unwrap_or(0);

        // Sample RFC 3164 syslog messages (timestamp must be exactly 15 chars: "MMM dd HH:MM:SS")
        let syslog_messages = vec![
            "<34>Oct 11 22:14:15 mymachine su: 'su root' failed for lonvick on /dev/pts/8",
            "<165>Feb  5 17:32:18 hostname app[1234]: Application started successfully",
            "<14>Jan 15 10:30:45 server01 kernel: Kernel panic - not syncing: VFS",
        ];

        // Parse and append each message
        for msg in syslog_messages {
            let parsed = parse(msg.as_bytes()).unwrap();
            builder.append_syslog(parsed);
        }

        // Build arrow records and capture end time
        let arrow_records = builder.build().unwrap();
        let end_time = Utc::now().timestamp_nanos_opt().unwrap_or(0);

        // Convert to ExportLogsServiceRequest
        let export_request = logs_from(arrow_records).unwrap();

        // Verify we have the expected number of resource logs
        assert_eq!(export_request.resource_logs.len(), 1);

        let resource_logs = &export_request.resource_logs[0];
        assert_eq!(resource_logs.scope_logs.len(), 1);

        let scope_logs = &resource_logs.scope_logs[0];
        assert_eq!(scope_logs.log_records.len(), 3);

        // Verify first log record
        let log1 = &scope_logs.log_records[0];
        assert_eq!(
            log1.body.as_ref().unwrap().value.as_ref().unwrap(),
            &Value::StringValue(
                "<34>Oct 11 22:14:15 mymachine su: 'su root' failed for lonvick on /dev/pts/8"
                    .to_string()
            )
        );

        // Priority = Facility * 8 + Severity
        // Priority 34: 4 (auth) * 8 + 2 (critical)
        // RFC3164 severity 2 (critical) maps to OpenTelemetry severity 18 (ERROR2)
        assert_eq!(log1.severity_number, 18);
        assert_eq!(log1.severity_text, "ERROR2");

        // Parse timestamp from message: Oct 11 22:14:15 UTC (assumes current year)
        let current_year = Utc::now().year();
        let expected_timestamp_1 = Local
            .from_local_datetime(
                &chrono::NaiveDate::from_ymd_opt(current_year, 10, 11)
                    .unwrap()
                    .and_hms_opt(22, 14, 15)
                    .unwrap(),
            )
            .unwrap()
            .with_timezone(&Utc)
            .timestamp_nanos_opt()
            .unwrap() as u64;
        assert_eq!(log1.time_unix_nano, expected_timestamp_1);

        // Verify fields that should be None/0 for log1
        validate_otlp_observed_timestamp_range(start_time, end_time, log1.observed_time_unix_nano);
        assert_eq!(log1.dropped_attributes_count, 0);
        assert_eq!(log1.flags, 0);
        assert!(log1.trace_id.is_empty());
        assert!(log1.span_id.is_empty());

        // Verify second log record
        let log2 = &scope_logs.log_records[1];
        assert_eq!(
            log2.body.as_ref().unwrap().value.as_ref().unwrap(),
            &Value::StringValue(
                "<165>Feb  5 17:32:18 hostname app[1234]: Application started successfully"
                    .to_string()
            )
        );

        // Priority = Facility * 8 + Severity
        // Priority 165: 20 (local use 4) * 8 + 5 (notice)
        // RFC3164 severity 5 (notice) maps to OpenTelemetry severity 10 (INFO2)
        assert_eq!(log2.severity_number, 10);
        assert_eq!(log2.severity_text, "INFO2");

        // Parse timestamp from message: Feb  5 17:32:18 (assumes current year)
        let expected_timestamp_2 = Local
            .from_local_datetime(
                &chrono::NaiveDate::from_ymd_opt(current_year, 2, 5)
                    .unwrap()
                    .and_hms_opt(17, 32, 18)
                    .unwrap(),
            )
            .unwrap()
            .with_timezone(&Utc)
            .timestamp_nanos_opt()
            .unwrap() as u64;
        assert_eq!(log2.time_unix_nano, expected_timestamp_2);

        // Verify fields that should be None/0 for log2
        validate_otlp_observed_timestamp_range(start_time, end_time, log2.observed_time_unix_nano);
        assert_eq!(log2.dropped_attributes_count, 0);
        assert_eq!(log2.flags, 0);
        assert!(log2.trace_id.is_empty());
        assert!(log2.span_id.is_empty());

        // Verify first log record attributes (RFC3164 format)
        let log1_attrs: std::collections::HashMap<String, String> = log1
            .attributes
            .iter()
            .map(|kv| {
                (
                    kv.key.clone(),
                    match &kv.value.as_ref().unwrap().value.as_ref().unwrap() {
                        Value::StringValue(s) => s.clone(),
                        Value::IntValue(i) => i.to_string(),
                        _ => String::new(),
                    },
                )
            })
            .collect();

        // Check attributes that SHOULD be present for log1 (RFC3164)
        assert_eq!(log1_attrs.get("syslog.facility"), Some(&"4".to_string()));
        assert_eq!(log1_attrs.get("syslog.severity"), Some(&"2".to_string()));
        assert_eq!(
            log1_attrs.get("syslog.host_name"),
            Some(&"mymachine".to_string())
        );
        assert_eq!(log1_attrs.get("syslog.tag"), Some(&"su".to_string()));
        assert_eq!(
            log1_attrs.get("syslog.message"),
            Some(&"'su root' failed for lonvick on /dev/pts/8".to_string())
        );

        // Ensure no unexpected attributes are present (exactly 5 attributes expected)
        assert_eq!(log1.attributes.len(), 5);

        // Verify second log record attributes (RFC3164 with process ID in tag)
        let log2_attrs: std::collections::HashMap<String, String> = log2
            .attributes
            .iter()
            .map(|kv| {
                (
                    kv.key.clone(),
                    match &kv.value.as_ref().unwrap().value.as_ref().unwrap() {
                        Value::StringValue(s) => s.clone(),
                        Value::IntValue(i) => i.to_string(),
                        _ => String::new(),
                    },
                )
            })
            .collect();

        // Check attributes that SHOULD be present for log2 (RFC3164 with process ID)
        assert_eq!(log2_attrs.get("syslog.facility"), Some(&"20".to_string()));
        assert_eq!(log2_attrs.get("syslog.severity"), Some(&"5".to_string()));
        assert_eq!(
            log2_attrs.get("syslog.host_name"),
            Some(&"hostname".to_string())
        );
        assert_eq!(log2_attrs.get("syslog.tag"), Some(&"app[1234]".to_string()));
        assert_eq!(
            log2_attrs.get("syslog.message"),
            Some(&"Application started successfully".to_string())
        );

        // Ensure no unexpected attributes are present (exactly 5 attributes expected)
        assert_eq!(log2.attributes.len(), 5);

        // Verify third log record
        let log3 = &scope_logs.log_records[2];
        assert_eq!(
            log3.body.as_ref().unwrap().value.as_ref().unwrap(),
            &Value::StringValue(
                "<14>Jan 15 10:30:45 server01 kernel: Kernel panic - not syncing: VFS".to_string()
            )
        );

        // Priority = Facility * 8 + Severity
        // Priority 14: 1 (user level) * 8 + 6 (informational)
        // RFC3164 severity 6 (informational) maps to OpenTelemetry severity 9 (INFO)
        assert_eq!(log3.severity_number, 9);
        assert_eq!(log3.severity_text, "INFO");

        // Parse timestamp from message: Jan 15 10:30:45 (assumes current year)
        let expected_timestamp_3 = Local
            .from_local_datetime(
                &chrono::NaiveDate::from_ymd_opt(current_year, 1, 15)
                    .unwrap()
                    .and_hms_opt(10, 30, 45)
                    .unwrap(),
            )
            .unwrap()
            .with_timezone(&Utc)
            .timestamp_nanos_opt()
            .unwrap() as u64;
        assert_eq!(log3.time_unix_nano, expected_timestamp_3);

        // Verify fields that should be None/0 for log3
        validate_otlp_observed_timestamp_range(start_time, end_time, log3.observed_time_unix_nano);
        assert_eq!(log3.dropped_attributes_count, 0);
        assert_eq!(log3.flags, 0);
        assert!(log3.trace_id.is_empty());
        assert!(log3.span_id.is_empty());

        // Verify third log record attributes

        // Check attributes that SHOULD be present for log3 (RFC3164)
        let log3_attrs: std::collections::HashMap<String, String> = log3
            .attributes
            .iter()
            .map(|kv| {
                (
                    kv.key.clone(),
                    match &kv.value.as_ref().unwrap().value.as_ref().unwrap() {
                        Value::StringValue(s) => s.clone(),
                        Value::IntValue(i) => i.to_string(),
                        _ => String::new(),
                    },
                )
            })
            .collect();
        assert_eq!(log3_attrs.get("syslog.facility"), Some(&"1".to_string()));
        assert_eq!(log3_attrs.get("syslog.severity"), Some(&"6".to_string()));
        assert_eq!(
            log3_attrs.get("syslog.host_name"),
            Some(&"server01".to_string())
        );
        assert_eq!(log3_attrs.get("syslog.tag"), Some(&"kernel".to_string()));
        assert_eq!(
            log3_attrs.get("syslog.message"),
            Some(&"Kernel panic - not syncing: VFS".to_string())
        );

        // Ensure no unexpected attributes are present (exactly 5 attributes expected)
        assert_eq!(log3.attributes.len(), 5);
    }

    #[test]
    fn test_cef_to_arrow_to_otlp() {
        // This test validates that CEF (Common Event Format) messages are correctly parsed,
        // converted to Arrow format, and then converted back to OTLP format.
        // CEF is a standard format for logging security events.
        // It includes comprehensive assertions checking:
        // 1. Which attributes SHOULD be present for each message type
        // 2. That the total number of attributes matches expectations (ensuring no unexpected attributes are present)
        // 3. All log record fields including body, severity, and optional fields
        // 4. Proper handling of None/empty values for trace_id, span_id, flags, timestamps, etc.

        let mut builder = ArrowRecordsBuilder::new();

        // Capture start time before processing messages
        let start_time = Utc::now().timestamp_nanos_opt().unwrap_or(0);

        // Sample CEF messages with varying complexity
        let cef_messages = vec![
            // Basic CEF message with extensions
            "CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232",
            // CEF message without extensions
            "CEF:1|ArcSight|ArcSight|2.4.1|400|Successful Login|3|",
            // CEF message with complex extensions
            "CEF:0|Vendor|Product|1.2.3|SignatureID|Event Name|5|deviceExternalId=12345 sourceAddress=192.168.1.100 destinationAddress=10.0.0.50 sourcePort=12345 destinationPort=80 protocol=TCP requestURL=http://example.com/path requestMethod=GET cs1=value1 cs2=value2",
        ];

        // Parse and append each message
        for msg in cef_messages {
            let parsed = parse(msg.as_bytes()).unwrap();
            builder.append_syslog(parsed);
        }

        // Build arrow records and capture end time
        let arrow_records = builder.build().unwrap();
        let end_time = Utc::now().timestamp_nanos_opt().unwrap_or(0);

        // Convert to ExportLogsServiceRequest
        let export_request = logs_from(arrow_records).unwrap();

        // Verify we have the expected number of resource logs
        assert_eq!(export_request.resource_logs.len(), 1);

        let resource_logs = &export_request.resource_logs[0];
        assert_eq!(resource_logs.scope_logs.len(), 1);

        let scope_logs = &resource_logs.scope_logs[0];
        assert_eq!(scope_logs.log_records.len(), 3);

        // Verify first CEF log record (with extensions)
        let log1 = &scope_logs.log_records[0];
        assert_eq!(log1.body.as_ref().unwrap().value.as_ref().unwrap(),
                   &Value::StringValue(
                       "CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232".to_string()
                   ));

        // CEF does not map to OpenTelemetry severity - should be default values
        assert_eq!(log1.severity_number, 0);
        assert_eq!(log1.severity_text, "UNSPECIFIED");

        // CEF does not have timestamp - should be 0
        assert_eq!(log1.time_unix_nano, 0);

        // Verify fields that should be None/0 for CEF logs
        validate_otlp_observed_timestamp_range(start_time, end_time, log1.observed_time_unix_nano);
        assert_eq!(log1.dropped_attributes_count, 0);
        assert_eq!(log1.flags, 0);
        assert!(log1.trace_id.is_empty());
        assert!(log1.span_id.is_empty());

        // Verify first CEF log record attributes
        let log1_attrs: std::collections::HashMap<String, String> = log1
            .attributes
            .iter()
            .map(|kv| {
                (
                    kv.key.clone(),
                    match &kv.value.as_ref().unwrap().value.as_ref().unwrap() {
                        Value::StringValue(s) => s.clone(),
                        Value::IntValue(i) => i.to_string(),
                        _ => String::new(),
                    },
                )
            })
            .collect();

        // Check CEF core attributes that SHOULD be present for log1
        assert_eq!(log1_attrs.get("cef.version"), Some(&"0".to_string()));
        assert_eq!(
            log1_attrs.get("cef.device_vendor"),
            Some(&"Security".to_string())
        );
        assert_eq!(
            log1_attrs.get("cef.device_product"),
            Some(&"threatmanager".to_string())
        );
        assert_eq!(
            log1_attrs.get("cef.device_version"),
            Some(&"1.0".to_string())
        );
        assert_eq!(log1_attrs.get("cef.signature_id"), Some(&"100".to_string()));
        assert_eq!(
            log1_attrs.get("cef.name"),
            Some(&"worm successfully stopped".to_string())
        );
        assert_eq!(log1_attrs.get("cef.severity"), Some(&"10".to_string()));

        // Check CEF extensions that SHOULD be present for log1
        assert_eq!(log1_attrs.get("src"), Some(&"10.0.0.1".to_string()));
        assert_eq!(log1_attrs.get("dst"), Some(&"2.1.2.2".to_string()));
        assert_eq!(log1_attrs.get("spt"), Some(&"1232".to_string()));

        // Ensure no unexpected attributes are present (7 core + 3 extensions = 10 attributes expected)
        assert_eq!(log1.attributes.len(), 10);

        // Verify second CEF log record (no extensions)
        let log2 = &scope_logs.log_records[1];
        assert_eq!(
            log2.body.as_ref().unwrap().value.as_ref().unwrap(),
            &Value::StringValue(
                "CEF:1|ArcSight|ArcSight|2.4.1|400|Successful Login|3|".to_string()
            )
        );

        // CEF does not map to OpenTelemetry severity - should be default values
        assert_eq!(log2.severity_number, 0);
        assert_eq!(log2.severity_text, "UNSPECIFIED");

        // CEF does not have timestamp - should be 0
        assert_eq!(log2.time_unix_nano, 0);

        // Verify fields that should be None/0 for CEF logs
        validate_otlp_observed_timestamp_range(start_time, end_time, log2.observed_time_unix_nano);
        assert_eq!(log2.dropped_attributes_count, 0);
        assert_eq!(log2.flags, 0);
        assert!(log2.trace_id.is_empty());
        assert!(log2.span_id.is_empty());

        // Verify second CEF log record attributes (no extensions)
        let log2_attrs: std::collections::HashMap<String, String> = log2
            .attributes
            .iter()
            .map(|kv| {
                (
                    kv.key.clone(),
                    match &kv.value.as_ref().unwrap().value.as_ref().unwrap() {
                        Value::StringValue(s) => s.clone(),
                        Value::IntValue(i) => i.to_string(),
                        _ => String::new(),
                    },
                )
            })
            .collect();

        // Check CEF core attributes that SHOULD be present for log2
        assert_eq!(log2_attrs.get("cef.version"), Some(&"1".to_string()));
        assert_eq!(
            log2_attrs.get("cef.device_vendor"),
            Some(&"ArcSight".to_string())
        );
        assert_eq!(
            log2_attrs.get("cef.device_product"),
            Some(&"ArcSight".to_string())
        );
        assert_eq!(
            log2_attrs.get("cef.device_version"),
            Some(&"2.4.1".to_string())
        );
        assert_eq!(log2_attrs.get("cef.signature_id"), Some(&"400".to_string()));
        assert_eq!(
            log2_attrs.get("cef.name"),
            Some(&"Successful Login".to_string())
        );
        assert_eq!(log2_attrs.get("cef.severity"), Some(&"3".to_string()));

        // Ensure no unexpected attributes are present (only 7 core attributes expected, no extensions)
        assert_eq!(log2.attributes.len(), 7);

        // Verify third CEF log record (complex extensions)
        let log3 = &scope_logs.log_records[2];
        assert_eq!(log3.body.as_ref().unwrap().value.as_ref().unwrap(),
                   &Value::StringValue(
                       "CEF:0|Vendor|Product|1.2.3|SignatureID|Event Name|5|deviceExternalId=12345 sourceAddress=192.168.1.100 destinationAddress=10.0.0.50 sourcePort=12345 destinationPort=80 protocol=TCP requestURL=http://example.com/path requestMethod=GET cs1=value1 cs2=value2".to_string()
                   ));

        // CEF does not map to OpenTelemetry severity - should be default values
        assert_eq!(log3.severity_number, 0);
        assert_eq!(log3.severity_text, "UNSPECIFIED");

        // CEF does not have timestamp - should be 0
        assert_eq!(log3.time_unix_nano, 0);

        // Verify fields that should be None/0 for CEF logs
        validate_otlp_observed_timestamp_range(start_time, end_time, log3.observed_time_unix_nano);
        assert_eq!(log3.dropped_attributes_count, 0);
        assert_eq!(log3.flags, 0);
        assert!(log3.trace_id.is_empty());
        assert!(log3.span_id.is_empty());

        let log3_attrs: std::collections::HashMap<String, String> = log3
            .attributes
            .iter()
            .map(|kv| {
                (
                    kv.key.clone(),
                    match &kv.value.as_ref().unwrap().value.as_ref().unwrap() {
                        Value::StringValue(s) => s.clone(),
                        Value::IntValue(i) => i.to_string(),
                        _ => String::new(),
                    },
                )
            })
            .collect();

        // Check CEF core attributes that SHOULD be present for log3
        assert_eq!(log3_attrs.get("cef.version"), Some(&"0".to_string()));
        assert_eq!(
            log3_attrs.get("cef.device_vendor"),
            Some(&"Vendor".to_string())
        );
        assert_eq!(
            log3_attrs.get("cef.device_product"),
            Some(&"Product".to_string())
        );
        assert_eq!(
            log3_attrs.get("cef.device_version"),
            Some(&"1.2.3".to_string())
        );
        assert_eq!(
            log3_attrs.get("cef.signature_id"),
            Some(&"SignatureID".to_string())
        );
        assert_eq!(log3_attrs.get("cef.name"), Some(&"Event Name".to_string()));
        assert_eq!(log3_attrs.get("cef.severity"), Some(&"5".to_string()));

        // Check some of the CEF extensions that SHOULD be present for log3
        assert_eq!(
            log3_attrs.get("deviceExternalId"),
            Some(&"12345".to_string())
        );
        assert_eq!(
            log3_attrs.get("sourceAddress"),
            Some(&"192.168.1.100".to_string())
        );
        assert_eq!(
            log3_attrs.get("destinationAddress"),
            Some(&"10.0.0.50".to_string())
        );
        assert_eq!(log3_attrs.get("sourcePort"), Some(&"12345".to_string()));
        assert_eq!(log3_attrs.get("destinationPort"), Some(&"80".to_string()));
        assert_eq!(log3_attrs.get("protocol"), Some(&"TCP".to_string()));
        assert_eq!(
            log3_attrs.get("requestURL"),
            Some(&"http://example.com/path".to_string())
        );
        assert_eq!(log3_attrs.get("requestMethod"), Some(&"GET".to_string()));
        assert_eq!(log3_attrs.get("cs1"), Some(&"value1".to_string()));
        assert_eq!(log3_attrs.get("cs2"), Some(&"value2".to_string()));

        // Ensure no unexpected attributes are present (7 core + 10 extensions = 17 attributes expected)
        assert_eq!(log3.attributes.len(), 17);
    }

    #[test]
    fn test_mixed_format_messages_to_arrow_to_otlp() {
        // This test validates that mixed message formats (RFC5424, RFC3164, and CEF)
        // can be processed together in a single batch and correctly converted to Arrow
        // format and then back to OTLP format.
        // It includes comprehensive assertions checking:
        // 1. Which attributes SHOULD be present for each message type
        // 2. That the total number of attributes matches expectations (ensuring no unexpected attributes are present)
        // 3. All log record fields including body, timestamps, severity, and optional fields
        // 4. Proper handling of different timestamp formats and severity mappings
        // 5. Verification that different message formats can coexist in the same batch

        let mut builder = ArrowRecordsBuilder::new();

        // Capture start time before processing messages
        let start_time = Utc::now().timestamp_nanos_opt().unwrap_or(0);

        // Mixed format messages: RFC5424, RFC3164, and CEF
        let mixed_messages = vec![
            // RFC5424 syslog message
            "<165>1 2024-01-15T10:31:00.456Z host.example.com myapp 1234 ID123 [exampleSDID@32473 iut=\"3\" eventSource=\"Application\" eventID=\"1011\"] Application started successfully",
            // RFC3164 syslog message
            "<34>Oct 11 22:14:15 mymachine su: 'su root' failed for lonvick on /dev/pts/8",
            // CEF message
            "CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232",
        ];

        // Parse and append each message
        for msg in mixed_messages {
            let parsed = parse(msg.as_bytes()).unwrap();
            builder.append_syslog(parsed);
        }

        // Build arrow records and capture end time
        let arrow_records = builder.build().unwrap();
        let end_time = Utc::now().timestamp_nanos_opt().unwrap_or(0);

        // Convert to ExportLogsServiceRequest
        let export_request = logs_from(arrow_records).unwrap();

        // Verify we have the expected number of resource logs
        assert_eq!(export_request.resource_logs.len(), 1);

        let resource_logs = &export_request.resource_logs[0];
        assert_eq!(resource_logs.scope_logs.len(), 1);

        let scope_logs = &resource_logs.scope_logs[0];
        assert_eq!(scope_logs.log_records.len(), 3);

        // Verify first log record (RFC5424)
        let log1 = &scope_logs.log_records[0];
        assert_eq!(log1.body.as_ref().unwrap().value.as_ref().unwrap(),
                   &Value::StringValue(
                       "<165>1 2024-01-15T10:31:00.456Z host.example.com myapp 1234 ID123 [exampleSDID@32473 iut=\"3\" eventSource=\"Application\" eventID=\"1011\"] Application started successfully".to_string()
                   ));

        // Priority = Facility * 8 + Severity
        // Priority 165: 20 (local use 4) * 8 + 5 (notice)
        // RFC5424 severity 5 (notice) maps to OpenTelemetry severity 10 (INFO2)
        assert_eq!(log1.severity_number, 10);
        assert_eq!(log1.severity_text, "INFO2");

        // Parse timestamp from RFC5424 message: 2024-01-15T10:31:00.456Z
        let expected_timestamp_1 = DateTime::parse_from_rfc3339("2024-01-15T10:31:00.456Z")
            .unwrap()
            .timestamp_nanos_opt()
            .unwrap() as u64;
        assert_eq!(log1.time_unix_nano, expected_timestamp_1);

        // Verify fields that should be None/0 for RFC5424 log
        validate_otlp_observed_timestamp_range(start_time, end_time, log1.observed_time_unix_nano);
        assert_eq!(log1.dropped_attributes_count, 0);
        assert_eq!(log1.flags, 0);
        assert!(log1.trace_id.is_empty());
        assert!(log1.span_id.is_empty());

        // Verify RFC5424 log record attributes
        let log1_attrs: std::collections::HashMap<String, String> = log1
            .attributes
            .iter()
            .map(|kv| {
                (
                    kv.key.clone(),
                    match &kv.value.as_ref().unwrap().value.as_ref().unwrap() {
                        Value::StringValue(s) => s.clone(),
                        Value::IntValue(i) => i.to_string(),
                        _ => String::new(),
                    },
                )
            })
            .collect();

        // Check RFC5424 attributes that SHOULD be present
        assert_eq!(log1_attrs.get("syslog.version"), Some(&"1".to_string()));
        assert_eq!(log1_attrs.get("syslog.facility"), Some(&"20".to_string()));
        assert_eq!(log1_attrs.get("syslog.severity"), Some(&"5".to_string()));
        assert_eq!(
            log1_attrs.get("syslog.host_name"),
            Some(&"host.example.com".to_string())
        );
        assert_eq!(
            log1_attrs.get("syslog.app_name"),
            Some(&"myapp".to_string())
        );
        assert_eq!(
            log1_attrs.get("syslog.process_id"),
            Some(&"1234".to_string())
        );
        assert_eq!(log1_attrs.get("syslog.msg_id"), Some(&"ID123".to_string()));
        assert_eq!(
            log1_attrs.get("syslog.structured_data"),
            Some(
                &"[exampleSDID@32473 iut=\"3\" eventSource=\"Application\" eventID=\"1011\"]"
                    .to_string()
            )
        );
        assert_eq!(
            log1_attrs.get("syslog.message"),
            Some(&"Application started successfully".to_string())
        );

        // Ensure no unexpected attributes are present for RFC5424 (exactly 9 attributes expected)
        assert_eq!(log1.attributes.len(), 9);

        // Verify second log record (RFC3164)
        let log2 = &scope_logs.log_records[1];
        assert_eq!(
            log2.body.as_ref().unwrap().value.as_ref().unwrap(),
            &Value::StringValue(
                "<34>Oct 11 22:14:15 mymachine su: 'su root' failed for lonvick on /dev/pts/8"
                    .to_string()
            )
        );

        // Priority = Facility * 8 + Severity
        // Priority 34: 4 (auth) * 8 + 2 (critical)
        // RFC3164 severity 2 (critical) maps to OpenTelemetry severity 18 (ERROR2)
        assert_eq!(log2.severity_number, 18);
        assert_eq!(log2.severity_text, "ERROR2");

        // Parse timestamp from RFC3164 message: Oct 11 22:14:15 (assumes current year)
        let current_year = Utc::now().year();
        let expected_timestamp_2 = Local
            .from_local_datetime(
                &chrono::NaiveDate::from_ymd_opt(current_year, 10, 11)
                    .unwrap()
                    .and_hms_opt(22, 14, 15)
                    .unwrap(),
            )
            .unwrap()
            .with_timezone(&Utc)
            .timestamp_nanos_opt()
            .unwrap() as u64;
        assert_eq!(log2.time_unix_nano, expected_timestamp_2);

        // Verify fields that should be None/0 for RFC3164 log
        validate_otlp_observed_timestamp_range(start_time, end_time, log2.observed_time_unix_nano);
        assert_eq!(log2.dropped_attributes_count, 0);
        assert_eq!(log2.flags, 0);
        assert!(log2.trace_id.is_empty());
        assert!(log2.span_id.is_empty());

        // Verify RFC3164 log record attributes
        let log2_attrs: std::collections::HashMap<String, String> = log2
            .attributes
            .iter()
            .map(|kv| {
                (
                    kv.key.clone(),
                    match &kv.value.as_ref().unwrap().value.as_ref().unwrap() {
                        Value::StringValue(s) => s.clone(),
                        Value::IntValue(i) => i.to_string(),
                        _ => String::new(),
                    },
                )
            })
            .collect();

        // Check RFC3164 attributes that SHOULD be present
        assert_eq!(log2_attrs.get("syslog.facility"), Some(&"4".to_string()));
        assert_eq!(log2_attrs.get("syslog.severity"), Some(&"2".to_string()));
        assert_eq!(
            log2_attrs.get("syslog.host_name"),
            Some(&"mymachine".to_string())
        );
        assert_eq!(log2_attrs.get("syslog.tag"), Some(&"su".to_string()));
        assert_eq!(
            log2_attrs.get("syslog.message"),
            Some(&"'su root' failed for lonvick on /dev/pts/8".to_string())
        );

        // Ensure no unexpected attributes are present for RFC3164 (exactly 5 attributes expected)
        assert_eq!(log2.attributes.len(), 5);

        // Verify third log record (CEF)
        let log3 = &scope_logs.log_records[2];
        assert_eq!(log3.body.as_ref().unwrap().value.as_ref().unwrap(),
                   &Value::StringValue(
                       "CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232".to_string()
                   ));

        // CEF does not map to OpenTelemetry severity - should be default values
        assert_eq!(log3.severity_number, 0);
        assert_eq!(log3.severity_text, "UNSPECIFIED");

        // CEF does not have timestamp - should be 0
        assert_eq!(log3.time_unix_nano, 0);

        // Verify fields that should be None/0 for CEF log
        validate_otlp_observed_timestamp_range(start_time, end_time, log3.observed_time_unix_nano);
        assert_eq!(log3.dropped_attributes_count, 0);
        assert_eq!(log3.flags, 0);
        assert!(log3.trace_id.is_empty());
        assert!(log3.span_id.is_empty());

        let log3_attrs: std::collections::HashMap<String, String> = log3
            .attributes
            .iter()
            .map(|kv| {
                (
                    kv.key.clone(),
                    match &kv.value.as_ref().unwrap().value.as_ref().unwrap() {
                        Value::StringValue(s) => s.clone(),
                        Value::IntValue(i) => i.to_string(),
                        _ => String::new(),
                    },
                )
            })
            .collect();

        // Check CEF core attributes that SHOULD be present for log3
        assert_eq!(log3_attrs.get("cef.version"), Some(&"0".to_string()));
        assert_eq!(
            log3_attrs.get("cef.device_vendor"),
            Some(&"Security".to_string())
        );
        assert_eq!(
            log3_attrs.get("cef.device_product"),
            Some(&"threatmanager".to_string())
        );
        assert_eq!(
            log3_attrs.get("cef.device_version"),
            Some(&"1.0".to_string())
        );
        assert_eq!(log3_attrs.get("cef.signature_id"), Some(&"100".to_string()));
        assert_eq!(
            log3_attrs.get("cef.name"),
            Some(&"worm successfully stopped".to_string())
        );
        assert_eq!(log3_attrs.get("cef.severity"), Some(&"10".to_string()));

        // Check CEF extensions that SHOULD be present for log3
        assert_eq!(log3_attrs.get("src"), Some(&"10.0.0.1".to_string()));
        assert_eq!(log3_attrs.get("dst"), Some(&"2.1.2.2".to_string()));
        assert_eq!(log3_attrs.get("spt"), Some(&"1232".to_string()));

        // Ensure no unexpected attributes are present for CEF (7 core + 3 extensions = 10 attributes expected)
        assert_eq!(log3.attributes.len(), 10);
    }
}
