// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Constants related to OTLP proto messages such as field numbers and other proto constants like
//! wire types

/// Protobuf wire types
pub mod wire_types {
    /// Varint (int32, int64, uint32, uint64, sint32, sint64, bool)
    pub const VARINT: u64 = 0;
    /// 64-bit (fixed64, sfixed64, double)
    pub const FIXED64: u64 = 1;
    /// Length-delimited (string, bytes, embedded messages)
    pub const LEN_DELIMITED: u64 = 2;
    /// 32-bit (fixed32, sfixed32, float)
    pub const FIXED_32: u64 = 5;
}

/// field number for OTLP protobuf messages
pub mod field_num {
    #[allow(missing_docs)]
    pub mod common {
        pub const KEY_VALUE_KEY: u64 = 1;
        pub const KEY_VALUE_VALUE: u64 = 2;

        pub const ANY_VALUE_STRING_VALUE: u64 = 1;
        pub const ANY_VALUE_BOOL_VALUE: u64 = 2;
        pub const ANY_VALUE_INT_VALUE: u64 = 3;
        pub const ANY_VALUE_DOUBLE_VALUE: u64 = 4;
        pub const ANY_VALUE_ARRAY_VALUE: u64 = 5;
        pub const ANY_VALUE_KVLIST_VALUE: u64 = 6;
        pub const ANY_VALUE_BYES_VALUE: u64 = 7;

        pub const KEY_VALUE_LIST_VALUES: u64 = 1;
        pub const ARRAY_VALUE_VALUES: u64 = 1;

        pub const INSTRUMENTATION_SCOPE_NAME: u64 = 1;
        pub const INSTRUMENTATION_SCOPE_VERSION: u64 = 2;
        pub const INSTRUMENTATION_SCOPE_ATTRIBUTES: u64 = 3;
        pub const INSTRUMENTATION_DROPPED_ATTRIBUTES_COUNT: u64 = 4;
    }

    #[allow(missing_docs)]
    pub mod logs {
        pub const LOGS_DATA_RESOURCE: u64 = 1;

        pub const RESOURCE_LOGS_RESOURCE: u64 = 1;
        pub const RESOURCE_LOGS_SCOPE_LOGS: u64 = 2;
        pub const RESOURCE_LOGS_SCHEMA_URL: u64 = 3;

        pub const SCOPE_LOG_SCOPE: u64 = 1;
        pub const SCOPE_LOGS_LOG_RECORDS: u64 = 2;
        pub const SCOPE_LOGS_SCHEMA_URL: u64 = 3;

        pub const LOG_RECORD_TIME_UNIX_NANO: u64 = 1;
        pub const LOG_RECORD_OBSERVED_TIME_UNIX_NANO: u64 = 11;
        pub const LOG_RECORD_SEVERITY_NUMBER: u64 = 2;
        pub const LOG_RECORD_SEVERITY_TEXT: u64 = 3;
        pub const LOG_RECORD_BODY: u64 = 5;
        pub const LOG_RECORD_ATTRIBUTES: u64 = 6;
        pub const LOG_RECORD_DROPPED_ATTRIBUTES_COUNT: u64 = 7;
        pub const LOG_RECORD_FLAGS: u64 = 8;
        pub const LOG_RECORD_TRACE_ID: u64 = 9;
        pub const LOG_RECORD_SPAN_ID: u64 = 10;
        pub const LOG_RECORD_EVENT_NAME: u64 = 12;
    }

    #[allow(missing_docs)]
    pub mod resource {
        pub const RESOURCE_ATTRIBUTES: u64 = 1;
        pub const RESOURCE_DROPPED_ATTRIBUTES_COUNT: u64 = 2;
    }
}
