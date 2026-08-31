/// Exporter implementations for core nodes.
pub mod exporters {
    /// Console exporter.
    pub mod console_exporter {
        //! Console exporter that prints OTLP data in human-readable or structured formats.
        mod record_json {
            //! Newline-delimited JSON formatting with one compact object per log record.
            use super::{
                RecordJsonBodyField, RecordJsonConfig, RecordJsonInt64Format,
                RecordJsonTimestampFormat,
            };
            use base64::Engine as _;
            use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
            use chrono::{DateTime, SecondsFormat, Utc};
            use otap_df_pdata_views::views::common::{
                AnyValueView, AttributeView, InstrumentationScopeView, ValueType,
            };
            use otap_df_pdata_views::views::logs::{
                LogRecordView, LogsDataView, ResourceLogsView, ScopeLogsView,
            };
            use otap_df_pdata_views::views::resource::ResourceView;
            use serde::Serialize;
            use serde::ser::{SerializeMap, Serializer};
            use serde_json::{Map as JsonMap, Number as JsonNumber, Value as JsonValue};
            use std::borrow::Cow;
            /// Formatter for one-record-per-line compact JSON output.
            pub(super) struct RecordJsonFormatter {
                config: RecordJsonConfig,
            }
            impl RecordJsonFormatter {
                /// Create a record JSON formatter.
                pub(super) const fn new(config: RecordJsonConfig) -> Self {
                    Self { config }
                }
                /// Format every log record as a compact JSON object followed by a newline.
                pub(super) fn format_logs_data_to<L: LogsDataView>(
                    &self,
                    logs_data: &L,
                    output: &mut Vec<u8>,
                ) -> serde_json::Result<()> {
                    for resource_logs in logs_data.resources() {
                        let resource = resource_logs.resource();
                        let resource_schema_url = resource_logs.schema_url();
                        for scope_logs in resource_logs.scopes() {
                            let scope = scope_logs.scope();
                            let scope_schema_url = scope_logs.schema_url();
                            for log_record in scope_logs.log_records() {
                                serde_json::to_writer(
                                    &mut *output,
                                    &RecordJson {
                                        resource: resource.as_ref(),
                                        scope: scope.as_ref(),
                                        log_record: &log_record,
                                        resource_schema_url,
                                        scope_schema_url,
                                        config: self.config,
                                    },
                                )?;
                                output.push(b'\n');
                            }
                        }
                    }
                    Ok(())
                }
            }
            /// Serializable compact record envelope with optional inherited context.
            struct RecordJson<'a, R, S, L> {
                resource: Option<&'a R>,
                scope: Option<&'a S>,
                log_record: &'a L,
                resource_schema_url: Option<&'a [u8]>,
                scope_schema_url: Option<&'a [u8]>,
                config: RecordJsonConfig,
            }
            impl<R, S, L> Serialize for RecordJson<'_, R, S, L>
            where
                R: ResourceView,
                S: InstrumentationScopeView,
                L: LogRecordView,
            {
                fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
                where
                    Ser: Serializer,
                {
                    let record = self.log_record;
                    let mut map = serializer.serialize_map(None)?;
                    if let Some(value) = record.time_unix_nano() {
                        map.serialize_entry(
                            "timestamp",
                            &format_timestamp(value, self.config.timestamp_format),
                        )?;
                    }
                    if let Some(value) = record.observed_time_unix_nano() {
                        map.serialize_entry(
                            "observed_timestamp",
                            &format_timestamp(value, self.config.timestamp_format),
                        )?;
                    }
                    if let Some(value) = record.severity_number() {
                        map.serialize_entry("severity_number", &value)?;
                    }
                    if let Some(value) = record.severity_text() {
                        map.serialize_entry("severity_text", &lossy_utf8(value))?;
                    }
                    if let Some(value) = record.body() {
                        let field_name = match self.config.body_field {
                            RecordJsonBodyField::Body => "body",
                            RecordJsonBodyField::Message => "message",
                        };
                        map.serialize_entry(
                            field_name,
                            &CompactValueJson {
                                value,
                                int64_format: self.config.int64_format,
                            },
                        )?;
                    }
                    if let Some(value) = record.event_name() {
                        map.serialize_entry("event_name", &lossy_utf8(value))?;
                    }
                    map.serialize_entry(
                        "attributes",
                        &LogAttributesJson {
                            record,
                            int64_format: self.config.int64_format,
                        },
                    )?;
                    if self.config.resource {
                        map.serialize_entry(
                            "resource",
                            &ResourceJson {
                                resource: self.resource,
                                int64_format: self.config.int64_format,
                            },
                        )?;
                    }
                    if self.config.scope {
                        map.serialize_entry(
                            "scope",
                            &ScopeJson {
                                scope: self.scope,
                                int64_format: self.config.int64_format,
                            },
                        )?;
                    }
                    if let Some(value) = record.trace_id() {
                        map.serialize_entry("trace_id", &hex::encode(value))?;
                    }
                    if let Some(value) = record.span_id() {
                        map.serialize_entry("span_id", &hex::encode(value))?;
                    }
                    if let Some(value) = record.flags() {
                        map.serialize_entry("trace_flags", &value)?;
                    }
                    if self.config.otel {
                        map.serialize_entry(
                            "otel",
                            &OtelJson {
                                dropped_attributes_count: record.dropped_attributes_count(),
                                resource_schema_url: self.resource_schema_url,
                                scope_schema_url: self.scope_schema_url,
                            },
                        )?;
                    }
                    map.end()
                }
            }
            /// Serializable flattened resource attributes.
            struct ResourceJson<'a, R> {
                resource: Option<&'a R>,
                int64_format: RecordJsonInt64Format,
            }
            impl<R: ResourceView> Serialize for ResourceJson<'_, R> {
                fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
                where
                    Ser: Serializer,
                {
                    let attributes = self
                        .resource
                        .map(|resource| compact_attributes(
                            resource.attributes(),
                            self.int64_format,
                        ))
                        .unwrap_or_default();
                    attributes.serialize(serializer)
                }
            }
            /// Serializable instrumentation scope context.
            struct ScopeJson<'a, S> {
                scope: Option<&'a S>,
                int64_format: RecordJsonInt64Format,
            }
            impl<S: InstrumentationScopeView> Serialize for ScopeJson<'_, S> {
                fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
                where
                    Ser: Serializer,
                {
                    let mut map = serializer.serialize_map(None)?;
                    if let Some(scope) = self.scope {
                        if let Some(value) = scope.name() {
                            map.serialize_entry("name", &lossy_utf8(value))?;
                        }
                        if let Some(value) = scope.version() {
                            map.serialize_entry("version", &lossy_utf8(value))?;
                        }
                        map.serialize_entry(
                            "attributes",
                            &ScopeAttributesJson {
                                scope,
                                int64_format: self.int64_format,
                            },
                        )?;
                    } else {
                        map.serialize_entry(
                            "attributes",
                            &JsonMap::<String, JsonValue>::new(),
                        )?;
                    }
                    map.end()
                }
            }
            /// Serializable OpenTelemetry bookkeeping fields.
            struct OtelJson<'a> {
                dropped_attributes_count: u32,
                resource_schema_url: Option<&'a [u8]>,
                scope_schema_url: Option<&'a [u8]>,
            }
            impl Serialize for OtelJson<'_> {
                fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
                where
                    Ser: Serializer,
                {
                    let mut map = serializer.serialize_map(None)?;
                    map.serialize_entry(
                        "dropped_attributes_count",
                        &self.dropped_attributes_count,
                    )?;
                    if let Some(value) = self.resource_schema_url {
                        map.serialize_entry("resource_schema_url", &lossy_utf8(value))?;
                    }
                    if let Some(value) = self.scope_schema_url {
                        map.serialize_entry("scope_schema_url", &lossy_utf8(value))?;
                    }
                    map.end()
                }
            }
            /// Serializable log-record attributes.
            struct LogAttributesJson<'a, L> {
                record: &'a L,
                int64_format: RecordJsonInt64Format,
            }
            impl<L: LogRecordView> Serialize for LogAttributesJson<'_, L> {
                fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
                where
                    Ser: Serializer,
                {
                    compact_attributes(self.record.attributes(), self.int64_format)
                        .serialize(serializer)
                }
            }
            /// Serializable instrumentation-scope attributes.
            struct ScopeAttributesJson<'a, S> {
                scope: &'a S,
                int64_format: RecordJsonInt64Format,
            }
            impl<S: InstrumentationScopeView> Serialize for ScopeAttributesJson<'_, S> {
                fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
                where
                    Ser: Serializer,
                {
                    compact_attributes(self.scope.attributes(), self.int64_format)
                        .serialize(serializer)
                }
            }
            /// Serializable compact AnyValue.
            struct CompactValueJson<T> {
                value: T,
                int64_format: RecordJsonInt64Format,
            }
            impl<'value, T> Serialize for CompactValueJson<T>
            where
                T: AnyValueView<'value> + 'value,
            {
                fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
                where
                    Ser: Serializer,
                {
                    compact_value(&self.value, self.int64_format).serialize(serializer)
                }
            }
            /// Convert an attribute iterator to a compact JSON object.
            fn compact_attributes<I, A>(
                attributes: I,
                int64_format: RecordJsonInt64Format,
            ) -> JsonMap<String, JsonValue>
            where
                I: Iterator<Item = A>,
                A: AttributeView,
            {
                let mut map = JsonMap::new();
                for attribute in attributes {
                    let key = lossy_utf8(attribute.key()).into_owned();
                    if let Some(value) = attribute.value() {
                        _ = map.insert(key, compact_value(&value, int64_format));
                    } else {
                        _ = map.remove(&key);
                    }
                }
                map
            }
            /// Convert an AnyValue to its compact JSON representation.
            fn compact_value<'value, T>(
                value: &T,
                int64_format: RecordJsonInt64Format,
            ) -> JsonValue
            where
                T: AnyValueView<'value> + 'value,
            {
                match value.value_type() {
                    ValueType::Empty => JsonValue::Null,
                    ValueType::String => {
                        JsonValue::String(
                            lossy_utf8(value.as_string().expect("value type is string"))
                                .into_owned(),
                        )
                    }
                    ValueType::Bool => {
                        JsonValue::Bool(value.as_bool().expect("value type is bool"))
                    }
                    ValueType::Int64 => {
                        let integer = value.as_int64().expect("value type is int64");
                        match int64_format {
                            RecordJsonInt64Format::Number => {
                                JsonValue::Number(integer.into())
                            }
                            RecordJsonInt64Format::String => {
                                JsonValue::String(integer.to_string())
                            }
                        }
                    }
                    ValueType::Double => {
                        JsonNumber::from_f64(
                                value.as_double().expect("value type is double"),
                            )
                            .map(JsonValue::Number)
                            .unwrap_or(JsonValue::Null)
                    }
                    ValueType::Array => {
                        JsonValue::Array(
                            value
                                .as_array()
                                .expect("value type is array")
                                .map(|child| compact_value(&child, int64_format))
                                .collect(),
                        )
                    }
                    ValueType::KeyValueList => {
                        JsonValue::Object(
                            compact_attributes(
                                value.as_kvlist().expect("value type is key-value list"),
                                int64_format,
                            ),
                        )
                    }
                    ValueType::Bytes => {
                        JsonValue::String(
                            BASE64_STANDARD
                                .encode(value.as_bytes().expect("value type is bytes")),
                        )
                    }
                }
            }
            /// Format a nanosecond Unix timestamp according to the configured representation.
            fn format_timestamp(
                value: u64,
                format: RecordJsonTimestampFormat,
            ) -> String {
                match format {
                    RecordJsonTimestampFormat::Rfc3339 => {
                        let seconds = (value / 1_000_000_000) as i64;
                        let nanoseconds = (value % 1_000_000_000) as u32;
                        DateTime::<Utc>::from_timestamp(seconds, nanoseconds)
                            .expect(
                                "u64 nanosecond timestamp is within the chrono range",
                            )
                            .to_rfc3339_opts(SecondsFormat::Nanos, true)
                    }
                    RecordJsonTimestampFormat::UnixNano => value.to_string(),
                }
            }
            /// Convert a potentially invalid protobuf string to valid JSON text.
            fn lossy_utf8(value: &[u8]) -> Cow<'_, str> {
                String::from_utf8_lossy(value)
            }
        }
        use async_trait::async_trait;
        use linkme::distributed_slice;
        use otap_df_config::SignalType;
        use otap_df_config::error::Error as ConfigError;
        use otap_df_config::node::NodeUserConfig;
        use otap_df_engine::config::ExporterConfig;
        use otap_df_engine::context::PipelineContext;
        use otap_df_engine::control::{AckMsg, NodeControlMsg};
        use otap_df_engine::error::Error;
        use otap_df_engine::exporter::ExporterWrapper;
        use otap_df_engine::local::exporter::{EffectHandler, Exporter};
        use otap_df_engine::message::{ExporterInbox, Message};
        use otap_df_engine::node::NodeId;
        use otap_df_engine::terminal_state::TerminalState;
        use otap_df_engine::{ConsumerEffectHandlerExtension, ExporterFactory};
        use otap_df_otap::OTAP_EXPORTER_FACTORIES;
        use otap_df_otap::pdata::OtapPdata;
        use otap_df_pdata::OtapPayload;
        use otap_df_pdata::views::otap::OtapLogsView;
        use otap_df_pdata::views::otlp::bytes::logs::RawLogsData;
        use otap_df_pdata_views::views::common::InstrumentationScopeView;
        use otap_df_pdata_views::views::logs::{
            LogRecordView, LogsDataView, ResourceLogsView, ScopeLogsView,
        };
        use otap_df_pdata_views::views::resource::ResourceView;
        use otap_df_telemetry::otel_error;
        use otap_df_telemetry::self_tracing::{
            AnsiCode, ColorMode, LOG_BUFFER_SIZE, StyledBufWriter,
        };
        use std::io::Write;
        use std::sync::Arc;
        use std::time::{Duration, SystemTime};
        use self::record_json::RecordJsonFormatter;
        /// The URN for the console exporter
        pub const CONSOLE_EXPORTER_URN: &str = "urn:otel:exporter:console";
        /// Output formats supported by the console exporter.
        #[serde(rename_all = "snake_case")]
        pub enum ConsoleOutputFormat {
            /// Human-readable hierarchical output intended for interactive inspection.
            #[default]
            Pretty,
            /// One compact log record JSON object per line.
            RecordJson,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for ConsoleOutputFormat {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(
                    f,
                    match self {
                        ConsoleOutputFormat::Pretty => "Pretty",
                        ConsoleOutputFormat::RecordJson => "RecordJson",
                    },
                )
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        unsafe impl ::core::clone::TrivialClone for ConsoleOutputFormat {}
        #[automatically_derived]
        impl ::core::clone::Clone for ConsoleOutputFormat {
            #[inline]
            fn clone(&self) -> ConsoleOutputFormat {
                *self
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for ConsoleOutputFormat {}
        #[automatically_derived]
        impl ::core::default::Default for ConsoleOutputFormat {
            #[inline]
            fn default() -> ConsoleOutputFormat {
                Self::Pretty
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for ConsoleOutputFormat {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for ConsoleOutputFormat {
            #[inline]
            fn eq(&self, other: &ConsoleOutputFormat) -> bool {
                let __self_discr = ::core::intrinsics::discriminant_value(self);
                let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                __self_discr == __arg1_discr
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Eq for ConsoleOutputFormat {
            #[inline]
            #[doc(hidden)]
            #[coverage(off)]
            fn assert_fields_are_eq(&self) {}
        }
        #[doc(hidden)]
        #[allow(
            non_upper_case_globals,
            unused_attributes,
            unused_qualifications,
            clippy::absolute_paths,
        )]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for ConsoleOutputFormat {
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private229::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    #[allow(non_camel_case_types)]
                    #[doc(hidden)]
                    enum __Field {
                        __field0,
                        __field1,
                    }
                    #[doc(hidden)]
                    struct __FieldVisitor;
                    #[automatically_derived]
                    impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                        type Value = __Field;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private229::Formatter,
                        ) -> _serde::__private229::fmt::Result {
                            _serde::__private229::Formatter::write_str(
                                __formatter,
                                "variant identifier",
                            )
                        }
                        fn visit_u64<__E>(
                            self,
                            __value: u64,
                        ) -> _serde::__private229::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                0u64 => _serde::__private229::Ok(__Field::__field0),
                                1u64 => _serde::__private229::Ok(__Field::__field1),
                                _ => {
                                    _serde::__private229::Err(
                                        _serde::de::Error::invalid_value(
                                            _serde::de::Unexpected::Unsigned(__value),
                                            &"variant index 0 <= i < 2",
                                        ),
                                    )
                                }
                            }
                        }
                        fn visit_str<__E>(
                            self,
                            __value: &str,
                        ) -> _serde::__private229::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                "pretty" => _serde::__private229::Ok(__Field::__field0),
                                "record_json" => _serde::__private229::Ok(__Field::__field1),
                                _ => {
                                    _serde::__private229::Err(
                                        _serde::de::Error::unknown_variant(__value, VARIANTS),
                                    )
                                }
                            }
                        }
                        fn visit_bytes<__E>(
                            self,
                            __value: &[u8],
                        ) -> _serde::__private229::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                b"pretty" => _serde::__private229::Ok(__Field::__field0),
                                b"record_json" => {
                                    _serde::__private229::Ok(__Field::__field1)
                                }
                                _ => {
                                    let __value = &_serde::__private229::from_utf8_lossy(
                                        __value,
                                    );
                                    _serde::__private229::Err(
                                        _serde::de::Error::unknown_variant(__value, VARIANTS),
                                    )
                                }
                            }
                        }
                    }
                    #[automatically_derived]
                    impl<'de> _serde::Deserialize<'de> for __Field {
                        #[inline]
                        fn deserialize<__D>(
                            __deserializer: __D,
                        ) -> _serde::__private229::Result<Self, __D::Error>
                        where
                            __D: _serde::Deserializer<'de>,
                        {
                            _serde::Deserializer::deserialize_identifier(
                                __deserializer,
                                __FieldVisitor,
                            )
                        }
                    }
                    #[doc(hidden)]
                    struct __Visitor<'de> {
                        marker: _serde::__private229::PhantomData<ConsoleOutputFormat>,
                        lifetime: _serde::__private229::PhantomData<&'de ()>,
                    }
                    #[automatically_derived]
                    impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                        type Value = ConsoleOutputFormat;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private229::Formatter,
                        ) -> _serde::__private229::fmt::Result {
                            _serde::__private229::Formatter::write_str(
                                __formatter,
                                "enum ConsoleOutputFormat",
                            )
                        }
                        fn visit_enum<__A>(
                            self,
                            __data: __A,
                        ) -> _serde::__private229::Result<Self::Value, __A::Error>
                        where
                            __A: _serde::de::EnumAccess<'de>,
                        {
                            match _serde::de::EnumAccess::variant(__data) {
                                _serde::__private229::Ok(
                                    (__Field::__field0, __variant),
                                ) => {
                                    _serde::de::VariantAccess::unit_variant(__variant)?;
                                    _serde::__private229::Ok(ConsoleOutputFormat::Pretty)
                                }
                                _serde::__private229::Ok(
                                    (__Field::__field1, __variant),
                                ) => {
                                    _serde::de::VariantAccess::unit_variant(__variant)?;
                                    _serde::__private229::Ok(ConsoleOutputFormat::RecordJson)
                                }
                                _serde::__private229::Err(__err) => {
                                    _serde::__private229::Err(__err)
                                }
                            }
                        }
                    }
                    #[doc(hidden)]
                    const VARIANTS: &'static [&'static str] = &["pretty", "record_json"];
                    _serde::Deserializer::deserialize_enum(
                        __deserializer,
                        "ConsoleOutputFormat",
                        VARIANTS,
                        __Visitor {
                            marker: _serde::__private229::PhantomData::<
                                ConsoleOutputFormat,
                            >,
                            lifetime: _serde::__private229::PhantomData,
                        },
                    )
                }
            }
        };
        /// Timestamp encodings supported by `record_json`.
        #[serde(rename_all = "snake_case")]
        pub enum RecordJsonTimestampFormat {
            /// UTC RFC 3339 with nanosecond precision.
            #[default]
            Rfc3339,
            /// Nanoseconds since the Unix epoch as a decimal string.
            UnixNano,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for RecordJsonTimestampFormat {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(
                    f,
                    match self {
                        RecordJsonTimestampFormat::Rfc3339 => "Rfc3339",
                        RecordJsonTimestampFormat::UnixNano => "UnixNano",
                    },
                )
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        unsafe impl ::core::clone::TrivialClone for RecordJsonTimestampFormat {}
        #[automatically_derived]
        impl ::core::clone::Clone for RecordJsonTimestampFormat {
            #[inline]
            fn clone(&self) -> RecordJsonTimestampFormat {
                *self
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for RecordJsonTimestampFormat {}
        #[automatically_derived]
        impl ::core::default::Default for RecordJsonTimestampFormat {
            #[inline]
            fn default() -> RecordJsonTimestampFormat {
                Self::Rfc3339
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for RecordJsonTimestampFormat {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for RecordJsonTimestampFormat {
            #[inline]
            fn eq(&self, other: &RecordJsonTimestampFormat) -> bool {
                let __self_discr = ::core::intrinsics::discriminant_value(self);
                let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                __self_discr == __arg1_discr
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Eq for RecordJsonTimestampFormat {
            #[inline]
            #[doc(hidden)]
            #[coverage(off)]
            fn assert_fields_are_eq(&self) {}
        }
        #[doc(hidden)]
        #[allow(
            non_upper_case_globals,
            unused_attributes,
            unused_qualifications,
            clippy::absolute_paths,
        )]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for RecordJsonTimestampFormat {
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private229::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    #[allow(non_camel_case_types)]
                    #[doc(hidden)]
                    enum __Field {
                        __field0,
                        __field1,
                    }
                    #[doc(hidden)]
                    struct __FieldVisitor;
                    #[automatically_derived]
                    impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                        type Value = __Field;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private229::Formatter,
                        ) -> _serde::__private229::fmt::Result {
                            _serde::__private229::Formatter::write_str(
                                __formatter,
                                "variant identifier",
                            )
                        }
                        fn visit_u64<__E>(
                            self,
                            __value: u64,
                        ) -> _serde::__private229::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                0u64 => _serde::__private229::Ok(__Field::__field0),
                                1u64 => _serde::__private229::Ok(__Field::__field1),
                                _ => {
                                    _serde::__private229::Err(
                                        _serde::de::Error::invalid_value(
                                            _serde::de::Unexpected::Unsigned(__value),
                                            &"variant index 0 <= i < 2",
                                        ),
                                    )
                                }
                            }
                        }
                        fn visit_str<__E>(
                            self,
                            __value: &str,
                        ) -> _serde::__private229::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                "rfc3339" => _serde::__private229::Ok(__Field::__field0),
                                "unix_nano" => _serde::__private229::Ok(__Field::__field1),
                                _ => {
                                    _serde::__private229::Err(
                                        _serde::de::Error::unknown_variant(__value, VARIANTS),
                                    )
                                }
                            }
                        }
                        fn visit_bytes<__E>(
                            self,
                            __value: &[u8],
                        ) -> _serde::__private229::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                b"rfc3339" => _serde::__private229::Ok(__Field::__field0),
                                b"unix_nano" => _serde::__private229::Ok(__Field::__field1),
                                _ => {
                                    let __value = &_serde::__private229::from_utf8_lossy(
                                        __value,
                                    );
                                    _serde::__private229::Err(
                                        _serde::de::Error::unknown_variant(__value, VARIANTS),
                                    )
                                }
                            }
                        }
                    }
                    #[automatically_derived]
                    impl<'de> _serde::Deserialize<'de> for __Field {
                        #[inline]
                        fn deserialize<__D>(
                            __deserializer: __D,
                        ) -> _serde::__private229::Result<Self, __D::Error>
                        where
                            __D: _serde::Deserializer<'de>,
                        {
                            _serde::Deserializer::deserialize_identifier(
                                __deserializer,
                                __FieldVisitor,
                            )
                        }
                    }
                    #[doc(hidden)]
                    struct __Visitor<'de> {
                        marker: _serde::__private229::PhantomData<
                            RecordJsonTimestampFormat,
                        >,
                        lifetime: _serde::__private229::PhantomData<&'de ()>,
                    }
                    #[automatically_derived]
                    impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                        type Value = RecordJsonTimestampFormat;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private229::Formatter,
                        ) -> _serde::__private229::fmt::Result {
                            _serde::__private229::Formatter::write_str(
                                __formatter,
                                "enum RecordJsonTimestampFormat",
                            )
                        }
                        fn visit_enum<__A>(
                            self,
                            __data: __A,
                        ) -> _serde::__private229::Result<Self::Value, __A::Error>
                        where
                            __A: _serde::de::EnumAccess<'de>,
                        {
                            match _serde::de::EnumAccess::variant(__data) {
                                _serde::__private229::Ok(
                                    (__Field::__field0, __variant),
                                ) => {
                                    _serde::de::VariantAccess::unit_variant(__variant)?;
                                    _serde::__private229::Ok(RecordJsonTimestampFormat::Rfc3339)
                                }
                                _serde::__private229::Ok(
                                    (__Field::__field1, __variant),
                                ) => {
                                    _serde::de::VariantAccess::unit_variant(__variant)?;
                                    _serde::__private229::Ok(
                                        RecordJsonTimestampFormat::UnixNano,
                                    )
                                }
                                _serde::__private229::Err(__err) => {
                                    _serde::__private229::Err(__err)
                                }
                            }
                        }
                    }
                    #[doc(hidden)]
                    const VARIANTS: &'static [&'static str] = &["rfc3339", "unix_nano"];
                    _serde::Deserializer::deserialize_enum(
                        __deserializer,
                        "RecordJsonTimestampFormat",
                        VARIANTS,
                        __Visitor {
                            marker: _serde::__private229::PhantomData::<
                                RecordJsonTimestampFormat,
                            >,
                            lifetime: _serde::__private229::PhantomData,
                        },
                    )
                }
            }
        };
        /// Field names supported for the `record_json` log body.
        #[serde(rename_all = "snake_case")]
        pub enum RecordJsonBodyField {
            /// Emit the log body under `body`.
            #[default]
            Body,
            /// Emit the log body under `message`.
            Message,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for RecordJsonBodyField {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(
                    f,
                    match self {
                        RecordJsonBodyField::Body => "Body",
                        RecordJsonBodyField::Message => "Message",
                    },
                )
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        unsafe impl ::core::clone::TrivialClone for RecordJsonBodyField {}
        #[automatically_derived]
        impl ::core::clone::Clone for RecordJsonBodyField {
            #[inline]
            fn clone(&self) -> RecordJsonBodyField {
                *self
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for RecordJsonBodyField {}
        #[automatically_derived]
        impl ::core::default::Default for RecordJsonBodyField {
            #[inline]
            fn default() -> RecordJsonBodyField {
                Self::Body
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for RecordJsonBodyField {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for RecordJsonBodyField {
            #[inline]
            fn eq(&self, other: &RecordJsonBodyField) -> bool {
                let __self_discr = ::core::intrinsics::discriminant_value(self);
                let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                __self_discr == __arg1_discr
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Eq for RecordJsonBodyField {
            #[inline]
            #[doc(hidden)]
            #[coverage(off)]
            fn assert_fields_are_eq(&self) {}
        }
        #[doc(hidden)]
        #[allow(
            non_upper_case_globals,
            unused_attributes,
            unused_qualifications,
            clippy::absolute_paths,
        )]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for RecordJsonBodyField {
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private229::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    #[allow(non_camel_case_types)]
                    #[doc(hidden)]
                    enum __Field {
                        __field0,
                        __field1,
                    }
                    #[doc(hidden)]
                    struct __FieldVisitor;
                    #[automatically_derived]
                    impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                        type Value = __Field;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private229::Formatter,
                        ) -> _serde::__private229::fmt::Result {
                            _serde::__private229::Formatter::write_str(
                                __formatter,
                                "variant identifier",
                            )
                        }
                        fn visit_u64<__E>(
                            self,
                            __value: u64,
                        ) -> _serde::__private229::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                0u64 => _serde::__private229::Ok(__Field::__field0),
                                1u64 => _serde::__private229::Ok(__Field::__field1),
                                _ => {
                                    _serde::__private229::Err(
                                        _serde::de::Error::invalid_value(
                                            _serde::de::Unexpected::Unsigned(__value),
                                            &"variant index 0 <= i < 2",
                                        ),
                                    )
                                }
                            }
                        }
                        fn visit_str<__E>(
                            self,
                            __value: &str,
                        ) -> _serde::__private229::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                "body" => _serde::__private229::Ok(__Field::__field0),
                                "message" => _serde::__private229::Ok(__Field::__field1),
                                _ => {
                                    _serde::__private229::Err(
                                        _serde::de::Error::unknown_variant(__value, VARIANTS),
                                    )
                                }
                            }
                        }
                        fn visit_bytes<__E>(
                            self,
                            __value: &[u8],
                        ) -> _serde::__private229::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                b"body" => _serde::__private229::Ok(__Field::__field0),
                                b"message" => _serde::__private229::Ok(__Field::__field1),
                                _ => {
                                    let __value = &_serde::__private229::from_utf8_lossy(
                                        __value,
                                    );
                                    _serde::__private229::Err(
                                        _serde::de::Error::unknown_variant(__value, VARIANTS),
                                    )
                                }
                            }
                        }
                    }
                    #[automatically_derived]
                    impl<'de> _serde::Deserialize<'de> for __Field {
                        #[inline]
                        fn deserialize<__D>(
                            __deserializer: __D,
                        ) -> _serde::__private229::Result<Self, __D::Error>
                        where
                            __D: _serde::Deserializer<'de>,
                        {
                            _serde::Deserializer::deserialize_identifier(
                                __deserializer,
                                __FieldVisitor,
                            )
                        }
                    }
                    #[doc(hidden)]
                    struct __Visitor<'de> {
                        marker: _serde::__private229::PhantomData<RecordJsonBodyField>,
                        lifetime: _serde::__private229::PhantomData<&'de ()>,
                    }
                    #[automatically_derived]
                    impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                        type Value = RecordJsonBodyField;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private229::Formatter,
                        ) -> _serde::__private229::fmt::Result {
                            _serde::__private229::Formatter::write_str(
                                __formatter,
                                "enum RecordJsonBodyField",
                            )
                        }
                        fn visit_enum<__A>(
                            self,
                            __data: __A,
                        ) -> _serde::__private229::Result<Self::Value, __A::Error>
                        where
                            __A: _serde::de::EnumAccess<'de>,
                        {
                            match _serde::de::EnumAccess::variant(__data) {
                                _serde::__private229::Ok(
                                    (__Field::__field0, __variant),
                                ) => {
                                    _serde::de::VariantAccess::unit_variant(__variant)?;
                                    _serde::__private229::Ok(RecordJsonBodyField::Body)
                                }
                                _serde::__private229::Ok(
                                    (__Field::__field1, __variant),
                                ) => {
                                    _serde::de::VariantAccess::unit_variant(__variant)?;
                                    _serde::__private229::Ok(RecordJsonBodyField::Message)
                                }
                                _serde::__private229::Err(__err) => {
                                    _serde::__private229::Err(__err)
                                }
                            }
                        }
                    }
                    #[doc(hidden)]
                    const VARIANTS: &'static [&'static str] = &["body", "message"];
                    _serde::Deserializer::deserialize_enum(
                        __deserializer,
                        "RecordJsonBodyField",
                        VARIANTS,
                        __Visitor {
                            marker: _serde::__private229::PhantomData::<
                                RecordJsonBodyField,
                            >,
                            lifetime: _serde::__private229::PhantomData,
                        },
                    )
                }
            }
        };
        /// Int64 encodings supported by `record_json`.
        #[serde(rename_all = "snake_case")]
        pub enum RecordJsonInt64Format {
            /// Emit int64 values as JSON integers.
            #[default]
            Number,
            /// Emit int64 values as decimal strings.
            String,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for RecordJsonInt64Format {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(
                    f,
                    match self {
                        RecordJsonInt64Format::Number => "Number",
                        RecordJsonInt64Format::String => "String",
                    },
                )
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        unsafe impl ::core::clone::TrivialClone for RecordJsonInt64Format {}
        #[automatically_derived]
        impl ::core::clone::Clone for RecordJsonInt64Format {
            #[inline]
            fn clone(&self) -> RecordJsonInt64Format {
                *self
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for RecordJsonInt64Format {}
        #[automatically_derived]
        impl ::core::default::Default for RecordJsonInt64Format {
            #[inline]
            fn default() -> RecordJsonInt64Format {
                Self::Number
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for RecordJsonInt64Format {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for RecordJsonInt64Format {
            #[inline]
            fn eq(&self, other: &RecordJsonInt64Format) -> bool {
                let __self_discr = ::core::intrinsics::discriminant_value(self);
                let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                __self_discr == __arg1_discr
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Eq for RecordJsonInt64Format {
            #[inline]
            #[doc(hidden)]
            #[coverage(off)]
            fn assert_fields_are_eq(&self) {}
        }
        #[doc(hidden)]
        #[allow(
            non_upper_case_globals,
            unused_attributes,
            unused_qualifications,
            clippy::absolute_paths,
        )]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for RecordJsonInt64Format {
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private229::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    #[allow(non_camel_case_types)]
                    #[doc(hidden)]
                    enum __Field {
                        __field0,
                        __field1,
                    }
                    #[doc(hidden)]
                    struct __FieldVisitor;
                    #[automatically_derived]
                    impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                        type Value = __Field;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private229::Formatter,
                        ) -> _serde::__private229::fmt::Result {
                            _serde::__private229::Formatter::write_str(
                                __formatter,
                                "variant identifier",
                            )
                        }
                        fn visit_u64<__E>(
                            self,
                            __value: u64,
                        ) -> _serde::__private229::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                0u64 => _serde::__private229::Ok(__Field::__field0),
                                1u64 => _serde::__private229::Ok(__Field::__field1),
                                _ => {
                                    _serde::__private229::Err(
                                        _serde::de::Error::invalid_value(
                                            _serde::de::Unexpected::Unsigned(__value),
                                            &"variant index 0 <= i < 2",
                                        ),
                                    )
                                }
                            }
                        }
                        fn visit_str<__E>(
                            self,
                            __value: &str,
                        ) -> _serde::__private229::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                "number" => _serde::__private229::Ok(__Field::__field0),
                                "string" => _serde::__private229::Ok(__Field::__field1),
                                _ => {
                                    _serde::__private229::Err(
                                        _serde::de::Error::unknown_variant(__value, VARIANTS),
                                    )
                                }
                            }
                        }
                        fn visit_bytes<__E>(
                            self,
                            __value: &[u8],
                        ) -> _serde::__private229::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                b"number" => _serde::__private229::Ok(__Field::__field0),
                                b"string" => _serde::__private229::Ok(__Field::__field1),
                                _ => {
                                    let __value = &_serde::__private229::from_utf8_lossy(
                                        __value,
                                    );
                                    _serde::__private229::Err(
                                        _serde::de::Error::unknown_variant(__value, VARIANTS),
                                    )
                                }
                            }
                        }
                    }
                    #[automatically_derived]
                    impl<'de> _serde::Deserialize<'de> for __Field {
                        #[inline]
                        fn deserialize<__D>(
                            __deserializer: __D,
                        ) -> _serde::__private229::Result<Self, __D::Error>
                        where
                            __D: _serde::Deserializer<'de>,
                        {
                            _serde::Deserializer::deserialize_identifier(
                                __deserializer,
                                __FieldVisitor,
                            )
                        }
                    }
                    #[doc(hidden)]
                    struct __Visitor<'de> {
                        marker: _serde::__private229::PhantomData<RecordJsonInt64Format>,
                        lifetime: _serde::__private229::PhantomData<&'de ()>,
                    }
                    #[automatically_derived]
                    impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                        type Value = RecordJsonInt64Format;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private229::Formatter,
                        ) -> _serde::__private229::fmt::Result {
                            _serde::__private229::Formatter::write_str(
                                __formatter,
                                "enum RecordJsonInt64Format",
                            )
                        }
                        fn visit_enum<__A>(
                            self,
                            __data: __A,
                        ) -> _serde::__private229::Result<Self::Value, __A::Error>
                        where
                            __A: _serde::de::EnumAccess<'de>,
                        {
                            match _serde::de::EnumAccess::variant(__data) {
                                _serde::__private229::Ok(
                                    (__Field::__field0, __variant),
                                ) => {
                                    _serde::de::VariantAccess::unit_variant(__variant)?;
                                    _serde::__private229::Ok(RecordJsonInt64Format::Number)
                                }
                                _serde::__private229::Ok(
                                    (__Field::__field1, __variant),
                                ) => {
                                    _serde::de::VariantAccess::unit_variant(__variant)?;
                                    _serde::__private229::Ok(RecordJsonInt64Format::String)
                                }
                                _serde::__private229::Err(__err) => {
                                    _serde::__private229::Err(__err)
                                }
                            }
                        }
                    }
                    #[doc(hidden)]
                    const VARIANTS: &'static [&'static str] = &["number", "string"];
                    _serde::Deserializer::deserialize_enum(
                        __deserializer,
                        "RecordJsonInt64Format",
                        VARIANTS,
                        __Visitor {
                            marker: _serde::__private229::PhantomData::<
                                RecordJsonInt64Format,
                            >,
                            lifetime: _serde::__private229::PhantomData,
                        },
                    )
                }
            }
        };
        /// Format-specific configuration for `record_json`.
        pub struct RecordJsonConfig {
            /// Timestamp encoding (default: rfc3339).
            #[serde(default)]
            pub timestamp_format: RecordJsonTimestampFormat,
            /// Log body field name (default: body).
            #[serde(default)]
            pub body_field: RecordJsonBodyField,
            /// Int64 encoding (default: number).
            #[serde(default)]
            pub int64_format: RecordJsonInt64Format,
            /// Include resource attributes in every record (default: false).
            #[serde(default)]
            pub resource: bool,
            /// Include scope context in every record (default: true).
            #[serde(default = "default_record_json_scope")]
            pub scope: bool,
            /// Include OpenTelemetry bookkeeping fields (default: false).
            #[serde(default)]
            pub otel: bool,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for RecordJsonConfig {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                let names: &'static _ = &[
                    "timestamp_format",
                    "body_field",
                    "int64_format",
                    "resource",
                    "scope",
                    "otel",
                ];
                let values: &[&dyn ::core::fmt::Debug] = &[
                    &self.timestamp_format,
                    &self.body_field,
                    &self.int64_format,
                    &self.resource,
                    &self.scope,
                    &&self.otel,
                ];
                ::core::fmt::Formatter::debug_struct_fields_finish(
                    f,
                    "RecordJsonConfig",
                    names,
                    values,
                )
            }
        }
        #[automatically_derived]
        #[doc(hidden)]
        unsafe impl ::core::clone::TrivialClone for RecordJsonConfig {}
        #[automatically_derived]
        impl ::core::clone::Clone for RecordJsonConfig {
            #[inline]
            fn clone(&self) -> RecordJsonConfig {
                let _: ::core::clone::AssertParamIsClone<RecordJsonTimestampFormat>;
                let _: ::core::clone::AssertParamIsClone<RecordJsonBodyField>;
                let _: ::core::clone::AssertParamIsClone<RecordJsonInt64Format>;
                let _: ::core::clone::AssertParamIsClone<bool>;
                *self
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for RecordJsonConfig {}
        #[doc(hidden)]
        #[allow(
            non_upper_case_globals,
            unused_attributes,
            unused_qualifications,
            clippy::absolute_paths,
        )]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for RecordJsonConfig {
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private229::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    #[allow(non_camel_case_types)]
                    #[doc(hidden)]
                    enum __Field {
                        __field0,
                        __field1,
                        __field2,
                        __field3,
                        __field4,
                        __field5,
                        __ignore,
                    }
                    #[doc(hidden)]
                    struct __FieldVisitor;
                    #[automatically_derived]
                    impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                        type Value = __Field;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private229::Formatter,
                        ) -> _serde::__private229::fmt::Result {
                            _serde::__private229::Formatter::write_str(
                                __formatter,
                                "field identifier",
                            )
                        }
                        fn visit_u64<__E>(
                            self,
                            __value: u64,
                        ) -> _serde::__private229::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                0u64 => _serde::__private229::Ok(__Field::__field0),
                                1u64 => _serde::__private229::Ok(__Field::__field1),
                                2u64 => _serde::__private229::Ok(__Field::__field2),
                                3u64 => _serde::__private229::Ok(__Field::__field3),
                                4u64 => _serde::__private229::Ok(__Field::__field4),
                                5u64 => _serde::__private229::Ok(__Field::__field5),
                                _ => _serde::__private229::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_str<__E>(
                            self,
                            __value: &str,
                        ) -> _serde::__private229::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                "timestamp_format" => {
                                    _serde::__private229::Ok(__Field::__field0)
                                }
                                "body_field" => _serde::__private229::Ok(__Field::__field1),
                                "int64_format" => {
                                    _serde::__private229::Ok(__Field::__field2)
                                }
                                "resource" => _serde::__private229::Ok(__Field::__field3),
                                "scope" => _serde::__private229::Ok(__Field::__field4),
                                "otel" => _serde::__private229::Ok(__Field::__field5),
                                _ => _serde::__private229::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_bytes<__E>(
                            self,
                            __value: &[u8],
                        ) -> _serde::__private229::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                b"timestamp_format" => {
                                    _serde::__private229::Ok(__Field::__field0)
                                }
                                b"body_field" => _serde::__private229::Ok(__Field::__field1),
                                b"int64_format" => {
                                    _serde::__private229::Ok(__Field::__field2)
                                }
                                b"resource" => _serde::__private229::Ok(__Field::__field3),
                                b"scope" => _serde::__private229::Ok(__Field::__field4),
                                b"otel" => _serde::__private229::Ok(__Field::__field5),
                                _ => _serde::__private229::Ok(__Field::__ignore),
                            }
                        }
                    }
                    #[automatically_derived]
                    impl<'de> _serde::Deserialize<'de> for __Field {
                        #[inline]
                        fn deserialize<__D>(
                            __deserializer: __D,
                        ) -> _serde::__private229::Result<Self, __D::Error>
                        where
                            __D: _serde::Deserializer<'de>,
                        {
                            _serde::Deserializer::deserialize_identifier(
                                __deserializer,
                                __FieldVisitor,
                            )
                        }
                    }
                    #[doc(hidden)]
                    struct __Visitor<'de> {
                        marker: _serde::__private229::PhantomData<RecordJsonConfig>,
                        lifetime: _serde::__private229::PhantomData<&'de ()>,
                    }
                    #[automatically_derived]
                    impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                        type Value = RecordJsonConfig;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private229::Formatter,
                        ) -> _serde::__private229::fmt::Result {
                            _serde::__private229::Formatter::write_str(
                                __formatter,
                                "struct RecordJsonConfig",
                            )
                        }
                        #[inline]
                        fn visit_seq<__A>(
                            self,
                            mut __seq: __A,
                        ) -> _serde::__private229::Result<Self::Value, __A::Error>
                        where
                            __A: _serde::de::SeqAccess<'de>,
                        {
                            let __field0 = match _serde::de::SeqAccess::next_element::<
                                RecordJsonTimestampFormat,
                            >(&mut __seq)? {
                                _serde::__private229::Some(__value) => __value,
                                _serde::__private229::None => {
                                    _serde::__private229::Default::default()
                                }
                            };
                            let __field1 = match _serde::de::SeqAccess::next_element::<
                                RecordJsonBodyField,
                            >(&mut __seq)? {
                                _serde::__private229::Some(__value) => __value,
                                _serde::__private229::None => {
                                    _serde::__private229::Default::default()
                                }
                            };
                            let __field2 = match _serde::de::SeqAccess::next_element::<
                                RecordJsonInt64Format,
                            >(&mut __seq)? {
                                _serde::__private229::Some(__value) => __value,
                                _serde::__private229::None => {
                                    _serde::__private229::Default::default()
                                }
                            };
                            let __field3 = match _serde::de::SeqAccess::next_element::<
                                bool,
                            >(&mut __seq)? {
                                _serde::__private229::Some(__value) => __value,
                                _serde::__private229::None => {
                                    _serde::__private229::Default::default()
                                }
                            };
                            let __field4 = match _serde::de::SeqAccess::next_element::<
                                bool,
                            >(&mut __seq)? {
                                _serde::__private229::Some(__value) => __value,
                                _serde::__private229::None => default_record_json_scope(),
                            };
                            let __field5 = match _serde::de::SeqAccess::next_element::<
                                bool,
                            >(&mut __seq)? {
                                _serde::__private229::Some(__value) => __value,
                                _serde::__private229::None => {
                                    _serde::__private229::Default::default()
                                }
                            };
                            _serde::__private229::Ok(RecordJsonConfig {
                                timestamp_format: __field0,
                                body_field: __field1,
                                int64_format: __field2,
                                resource: __field3,
                                scope: __field4,
                                otel: __field5,
                            })
                        }
                        #[inline]
                        fn visit_map<__A>(
                            self,
                            mut __map: __A,
                        ) -> _serde::__private229::Result<Self::Value, __A::Error>
                        where
                            __A: _serde::de::MapAccess<'de>,
                        {
                            let mut __field0: _serde::__private229::Option<
                                RecordJsonTimestampFormat,
                            > = _serde::__private229::None;
                            let mut __field1: _serde::__private229::Option<
                                RecordJsonBodyField,
                            > = _serde::__private229::None;
                            let mut __field2: _serde::__private229::Option<
                                RecordJsonInt64Format,
                            > = _serde::__private229::None;
                            let mut __field3: _serde::__private229::Option<bool> = _serde::__private229::None;
                            let mut __field4: _serde::__private229::Option<bool> = _serde::__private229::None;
                            let mut __field5: _serde::__private229::Option<bool> = _serde::__private229::None;
                            while let _serde::__private229::Some(__key) = _serde::de::MapAccess::next_key::<
                                __Field,
                            >(&mut __map)? {
                                match __key {
                                    __Field::__field0 => {
                                        if _serde::__private229::Option::is_some(&__field0) {
                                            return _serde::__private229::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "timestamp_format",
                                                ),
                                            );
                                        }
                                        __field0 = _serde::__private229::Some(
                                            _serde::de::MapAccess::next_value::<
                                                RecordJsonTimestampFormat,
                                            >(&mut __map)?,
                                        );
                                    }
                                    __Field::__field1 => {
                                        if _serde::__private229::Option::is_some(&__field1) {
                                            return _serde::__private229::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "body_field",
                                                ),
                                            );
                                        }
                                        __field1 = _serde::__private229::Some(
                                            _serde::de::MapAccess::next_value::<
                                                RecordJsonBodyField,
                                            >(&mut __map)?,
                                        );
                                    }
                                    __Field::__field2 => {
                                        if _serde::__private229::Option::is_some(&__field2) {
                                            return _serde::__private229::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "int64_format",
                                                ),
                                            );
                                        }
                                        __field2 = _serde::__private229::Some(
                                            _serde::de::MapAccess::next_value::<
                                                RecordJsonInt64Format,
                                            >(&mut __map)?,
                                        );
                                    }
                                    __Field::__field3 => {
                                        if _serde::__private229::Option::is_some(&__field3) {
                                            return _serde::__private229::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "resource",
                                                ),
                                            );
                                        }
                                        __field3 = _serde::__private229::Some(
                                            _serde::de::MapAccess::next_value::<bool>(&mut __map)?,
                                        );
                                    }
                                    __Field::__field4 => {
                                        if _serde::__private229::Option::is_some(&__field4) {
                                            return _serde::__private229::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field("scope"),
                                            );
                                        }
                                        __field4 = _serde::__private229::Some(
                                            _serde::de::MapAccess::next_value::<bool>(&mut __map)?,
                                        );
                                    }
                                    __Field::__field5 => {
                                        if _serde::__private229::Option::is_some(&__field5) {
                                            return _serde::__private229::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field("otel"),
                                            );
                                        }
                                        __field5 = _serde::__private229::Some(
                                            _serde::de::MapAccess::next_value::<bool>(&mut __map)?,
                                        );
                                    }
                                    _ => {
                                        let _ = _serde::de::MapAccess::next_value::<
                                            _serde::de::IgnoredAny,
                                        >(&mut __map)?;
                                    }
                                }
                            }
                            let __field0 = match __field0 {
                                _serde::__private229::Some(__field0) => __field0,
                                _serde::__private229::None => {
                                    _serde::__private229::Default::default()
                                }
                            };
                            let __field1 = match __field1 {
                                _serde::__private229::Some(__field1) => __field1,
                                _serde::__private229::None => {
                                    _serde::__private229::Default::default()
                                }
                            };
                            let __field2 = match __field2 {
                                _serde::__private229::Some(__field2) => __field2,
                                _serde::__private229::None => {
                                    _serde::__private229::Default::default()
                                }
                            };
                            let __field3 = match __field3 {
                                _serde::__private229::Some(__field3) => __field3,
                                _serde::__private229::None => {
                                    _serde::__private229::Default::default()
                                }
                            };
                            let __field4 = match __field4 {
                                _serde::__private229::Some(__field4) => __field4,
                                _serde::__private229::None => default_record_json_scope(),
                            };
                            let __field5 = match __field5 {
                                _serde::__private229::Some(__field5) => __field5,
                                _serde::__private229::None => {
                                    _serde::__private229::Default::default()
                                }
                            };
                            _serde::__private229::Ok(RecordJsonConfig {
                                timestamp_format: __field0,
                                body_field: __field1,
                                int64_format: __field2,
                                resource: __field3,
                                scope: __field4,
                                otel: __field5,
                            })
                        }
                    }
                    #[doc(hidden)]
                    const FIELDS: &'static [&'static str] = &[
                        "timestamp_format",
                        "body_field",
                        "int64_format",
                        "resource",
                        "scope",
                        "otel",
                    ];
                    _serde::Deserializer::deserialize_struct(
                        __deserializer,
                        "RecordJsonConfig",
                        FIELDS,
                        __Visitor {
                            marker: _serde::__private229::PhantomData::<
                                RecordJsonConfig,
                            >,
                            lifetime: _serde::__private229::PhantomData,
                        },
                    )
                }
            }
        };
        impl Default for RecordJsonConfig {
            fn default() -> Self {
                Self {
                    timestamp_format: RecordJsonTimestampFormat::default(),
                    body_field: RecordJsonBodyField::default(),
                    int64_format: RecordJsonInt64Format::default(),
                    resource: false,
                    scope: default_record_json_scope(),
                    otel: false,
                }
            }
        }
        /// Configuration for the console exporter
        pub struct ConsoleExporterConfig {
            /// Output format (default: pretty).
            #[serde(default)]
            pub format: ConsoleOutputFormat,
            /// Whether to use ANSI colors in output (default: true)
            #[serde(default = "default_color")]
            pub color: bool,
            /// Whether to use Unicode box-drawing characters (default: true)
            #[serde(default = "default_unicode")]
            pub unicode: bool,
            /// Format-specific options for `record_json`.
            #[serde(default)]
            pub record_json: RecordJsonConfig,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for ConsoleExporterConfig {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field4_finish(
                    f,
                    "ConsoleExporterConfig",
                    "format",
                    &self.format,
                    "color",
                    &self.color,
                    "unicode",
                    &self.unicode,
                    "record_json",
                    &&self.record_json,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for ConsoleExporterConfig {
            #[inline]
            fn clone(&self) -> ConsoleExporterConfig {
                ConsoleExporterConfig {
                    format: ::core::clone::Clone::clone(&self.format),
                    color: ::core::clone::Clone::clone(&self.color),
                    unicode: ::core::clone::Clone::clone(&self.unicode),
                    record_json: ::core::clone::Clone::clone(&self.record_json),
                }
            }
        }
        #[doc(hidden)]
        #[allow(
            non_upper_case_globals,
            unused_attributes,
            unused_qualifications,
            clippy::absolute_paths,
        )]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for ConsoleExporterConfig {
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private229::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    #[allow(non_camel_case_types)]
                    #[doc(hidden)]
                    enum __Field {
                        __field0,
                        __field1,
                        __field2,
                        __field3,
                        __ignore,
                    }
                    #[doc(hidden)]
                    struct __FieldVisitor;
                    #[automatically_derived]
                    impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                        type Value = __Field;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private229::Formatter,
                        ) -> _serde::__private229::fmt::Result {
                            _serde::__private229::Formatter::write_str(
                                __formatter,
                                "field identifier",
                            )
                        }
                        fn visit_u64<__E>(
                            self,
                            __value: u64,
                        ) -> _serde::__private229::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                0u64 => _serde::__private229::Ok(__Field::__field0),
                                1u64 => _serde::__private229::Ok(__Field::__field1),
                                2u64 => _serde::__private229::Ok(__Field::__field2),
                                3u64 => _serde::__private229::Ok(__Field::__field3),
                                _ => _serde::__private229::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_str<__E>(
                            self,
                            __value: &str,
                        ) -> _serde::__private229::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                "format" => _serde::__private229::Ok(__Field::__field0),
                                "color" => _serde::__private229::Ok(__Field::__field1),
                                "unicode" => _serde::__private229::Ok(__Field::__field2),
                                "record_json" => _serde::__private229::Ok(__Field::__field3),
                                _ => _serde::__private229::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_bytes<__E>(
                            self,
                            __value: &[u8],
                        ) -> _serde::__private229::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                b"format" => _serde::__private229::Ok(__Field::__field0),
                                b"color" => _serde::__private229::Ok(__Field::__field1),
                                b"unicode" => _serde::__private229::Ok(__Field::__field2),
                                b"record_json" => {
                                    _serde::__private229::Ok(__Field::__field3)
                                }
                                _ => _serde::__private229::Ok(__Field::__ignore),
                            }
                        }
                    }
                    #[automatically_derived]
                    impl<'de> _serde::Deserialize<'de> for __Field {
                        #[inline]
                        fn deserialize<__D>(
                            __deserializer: __D,
                        ) -> _serde::__private229::Result<Self, __D::Error>
                        where
                            __D: _serde::Deserializer<'de>,
                        {
                            _serde::Deserializer::deserialize_identifier(
                                __deserializer,
                                __FieldVisitor,
                            )
                        }
                    }
                    #[doc(hidden)]
                    struct __Visitor<'de> {
                        marker: _serde::__private229::PhantomData<ConsoleExporterConfig>,
                        lifetime: _serde::__private229::PhantomData<&'de ()>,
                    }
                    #[automatically_derived]
                    impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                        type Value = ConsoleExporterConfig;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private229::Formatter,
                        ) -> _serde::__private229::fmt::Result {
                            _serde::__private229::Formatter::write_str(
                                __formatter,
                                "struct ConsoleExporterConfig",
                            )
                        }
                        #[inline]
                        fn visit_seq<__A>(
                            self,
                            mut __seq: __A,
                        ) -> _serde::__private229::Result<Self::Value, __A::Error>
                        where
                            __A: _serde::de::SeqAccess<'de>,
                        {
                            let __field0 = match _serde::de::SeqAccess::next_element::<
                                ConsoleOutputFormat,
                            >(&mut __seq)? {
                                _serde::__private229::Some(__value) => __value,
                                _serde::__private229::None => {
                                    _serde::__private229::Default::default()
                                }
                            };
                            let __field1 = match _serde::de::SeqAccess::next_element::<
                                bool,
                            >(&mut __seq)? {
                                _serde::__private229::Some(__value) => __value,
                                _serde::__private229::None => default_color(),
                            };
                            let __field2 = match _serde::de::SeqAccess::next_element::<
                                bool,
                            >(&mut __seq)? {
                                _serde::__private229::Some(__value) => __value,
                                _serde::__private229::None => default_unicode(),
                            };
                            let __field3 = match _serde::de::SeqAccess::next_element::<
                                RecordJsonConfig,
                            >(&mut __seq)? {
                                _serde::__private229::Some(__value) => __value,
                                _serde::__private229::None => {
                                    _serde::__private229::Default::default()
                                }
                            };
                            _serde::__private229::Ok(ConsoleExporterConfig {
                                format: __field0,
                                color: __field1,
                                unicode: __field2,
                                record_json: __field3,
                            })
                        }
                        #[inline]
                        fn visit_map<__A>(
                            self,
                            mut __map: __A,
                        ) -> _serde::__private229::Result<Self::Value, __A::Error>
                        where
                            __A: _serde::de::MapAccess<'de>,
                        {
                            let mut __field0: _serde::__private229::Option<
                                ConsoleOutputFormat,
                            > = _serde::__private229::None;
                            let mut __field1: _serde::__private229::Option<bool> = _serde::__private229::None;
                            let mut __field2: _serde::__private229::Option<bool> = _serde::__private229::None;
                            let mut __field3: _serde::__private229::Option<
                                RecordJsonConfig,
                            > = _serde::__private229::None;
                            while let _serde::__private229::Some(__key) = _serde::de::MapAccess::next_key::<
                                __Field,
                            >(&mut __map)? {
                                match __key {
                                    __Field::__field0 => {
                                        if _serde::__private229::Option::is_some(&__field0) {
                                            return _serde::__private229::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field("format"),
                                            );
                                        }
                                        __field0 = _serde::__private229::Some(
                                            _serde::de::MapAccess::next_value::<
                                                ConsoleOutputFormat,
                                            >(&mut __map)?,
                                        );
                                    }
                                    __Field::__field1 => {
                                        if _serde::__private229::Option::is_some(&__field1) {
                                            return _serde::__private229::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field("color"),
                                            );
                                        }
                                        __field1 = _serde::__private229::Some(
                                            _serde::de::MapAccess::next_value::<bool>(&mut __map)?,
                                        );
                                    }
                                    __Field::__field2 => {
                                        if _serde::__private229::Option::is_some(&__field2) {
                                            return _serde::__private229::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "unicode",
                                                ),
                                            );
                                        }
                                        __field2 = _serde::__private229::Some(
                                            _serde::de::MapAccess::next_value::<bool>(&mut __map)?,
                                        );
                                    }
                                    __Field::__field3 => {
                                        if _serde::__private229::Option::is_some(&__field3) {
                                            return _serde::__private229::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "record_json",
                                                ),
                                            );
                                        }
                                        __field3 = _serde::__private229::Some(
                                            _serde::de::MapAccess::next_value::<
                                                RecordJsonConfig,
                                            >(&mut __map)?,
                                        );
                                    }
                                    _ => {
                                        let _ = _serde::de::MapAccess::next_value::<
                                            _serde::de::IgnoredAny,
                                        >(&mut __map)?;
                                    }
                                }
                            }
                            let __field0 = match __field0 {
                                _serde::__private229::Some(__field0) => __field0,
                                _serde::__private229::None => {
                                    _serde::__private229::Default::default()
                                }
                            };
                            let __field1 = match __field1 {
                                _serde::__private229::Some(__field1) => __field1,
                                _serde::__private229::None => default_color(),
                            };
                            let __field2 = match __field2 {
                                _serde::__private229::Some(__field2) => __field2,
                                _serde::__private229::None => default_unicode(),
                            };
                            let __field3 = match __field3 {
                                _serde::__private229::Some(__field3) => __field3,
                                _serde::__private229::None => {
                                    _serde::__private229::Default::default()
                                }
                            };
                            _serde::__private229::Ok(ConsoleExporterConfig {
                                format: __field0,
                                color: __field1,
                                unicode: __field2,
                                record_json: __field3,
                            })
                        }
                    }
                    #[doc(hidden)]
                    const FIELDS: &'static [&'static str] = &[
                        "format",
                        "color",
                        "unicode",
                        "record_json",
                    ];
                    _serde::Deserializer::deserialize_struct(
                        __deserializer,
                        "ConsoleExporterConfig",
                        FIELDS,
                        __Visitor {
                            marker: _serde::__private229::PhantomData::<
                                ConsoleExporterConfig,
                            >,
                            lifetime: _serde::__private229::PhantomData,
                        },
                    )
                }
            }
        };
        impl Default for ConsoleExporterConfig {
            fn default() -> Self {
                Self {
                    format: ConsoleOutputFormat::default(),
                    color: default_color(),
                    unicode: default_unicode(),
                    record_json: RecordJsonConfig::default(),
                }
            }
        }
        const fn default_color() -> bool {
            true
        }
        const fn default_unicode() -> bool {
            true
        }
        const fn default_record_json_scope() -> bool {
            true
        }
        /// Console exporter that prints OTLP data to stdout.
        pub struct ConsoleExporter {
            formatter: ConsoleFormatter,
        }
        impl ConsoleExporter {
            /// Create a new console exporter with the given configuration.
            #[must_use]
            pub const fn new(config: ConsoleExporterConfig) -> Self {
                let formatter = match config.format {
                    ConsoleOutputFormat::Pretty => {
                        ConsoleFormatter::Pretty(
                            HierarchicalFormatter::new(config.color, config.unicode),
                        )
                    }
                    ConsoleOutputFormat::RecordJson => {
                        ConsoleFormatter::RecordJson(
                            RecordJsonFormatter::new(config.record_json),
                        )
                    }
                };
                Self { formatter }
            }
        }
        #[used]
        #[unsafe(link_section = ".linkme_OTAP_EXPORTER_FACTORIES$b")]
        /// Declare the Console Exporter as a local exporter factory
        #[allow(unsafe_code)]
        pub static CONSOLE_EXPORTER: ExporterFactory<OtapPdata> = {
            #[allow(clippy::no_effect_underscore_binding)]
            unsafe fn __typecheck(_: ::linkme::__private37::Void) {
                #[allow(clippy::ref_option_ref)]
                let __new = || -> fn() -> &'static ExporterFactory<OtapPdata> {
                    || &CONSOLE_EXPORTER
                };
                unsafe {
                    ::linkme::DistributedSlice::private_typecheck(
                        OTAP_EXPORTER_FACTORIES,
                        __new(),
                    );
                }
            }
            ExporterFactory {
                name: CONSOLE_EXPORTER_URN,
                create: |
                    _pipeline: PipelineContext,
                    node: NodeId,
                    node_config: Arc<NodeUserConfig>,
                    exporter_config: &ExporterConfig,
                    _capabilities: &otap_df_engine::capability::registry::Capabilities|
                {
                    let config: ConsoleExporterConfig = serde_json::from_value(
                            node_config.config.clone(),
                        )
                        .map_err(|e| ConfigError::InvalidUserConfig {
                            error: ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!(
                                        "Failed to parse console exporter config: {0}",
                                        e,
                                    ),
                                )
                            }),
                        })?;
                    Ok(
                        ExporterWrapper::local(
                            ConsoleExporter::new(config),
                            node,
                            node_config,
                            exporter_config,
                        ),
                    )
                },
                wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
                validate_config: otap_df_config::validation::validate_typed_config::<
                    ConsoleExporterConfig,
                >,
            }
        };
        #[used]
        #[unsafe(link_section = ".linkme_COMPONENT_INVENTORY$b")]
        #[allow(unsafe_code)]
        #[allow(non_upper_case_globals)]
        #[doc(hidden)]
        static _COMPONENT_META_CONSOLE_EXPORTER: ::otap_df_engine::inventory::ComponentMeta = {
            #[allow(clippy::no_effect_underscore_binding)]
            unsafe fn __typecheck(_: ::linkme::__private37::Void) {
                #[allow(clippy::ref_option_ref)]
                let __new = || -> fn() -> &'static ::otap_df_engine::inventory::ComponentMeta {
                    || &_COMPONENT_META_CONSOLE_EXPORTER
                };
                unsafe {
                    ::linkme::DistributedSlice::private_typecheck(
                        ::otap_df_engine::inventory::COMPONENT_INVENTORY,
                        __new(),
                    );
                }
            }
            ::otap_df_engine::inventory::ComponentMeta {
                id: CONSOLE_EXPORTER_URN,
                category: ::otap_df_engine::inventory::Category::Exporter,
                description: ::core::option::Option::None,
                file: "crates\\core-nodes\\src\\exporters\\console_exporter\\mod.rs",
                line: 187u32,
                attributes: &[],
            }
        };
        impl Exporter<OtapPdata> for ConsoleExporter {
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn start<'async_trait>(
                self: Box<Self>,
                msg_chan: ExporterInbox<OtapPdata>,
                effect_handler: EffectHandler<OtapPdata>,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = Result<TerminalState, Error>,
                    > + 'async_trait,
                >,
            >
            where
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        Result<TerminalState, Error>,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let mut msg_chan = msg_chan;
                    let effect_handler = effect_handler;
                    let __ret: Result<TerminalState, Error> = {
                        loop {
                            match msg_chan.recv().await? {
                                Message::Control(NodeControlMsg::Shutdown { .. }) => break,
                                Message::PData(data) => {
                                    __self.export(data.payload_ref()).await;
                                    effect_handler.notify_ack(AckMsg::new(data)).await?;
                                }
                                _ => {}
                            }
                        }
                        Ok(TerminalState::default())
                    };
                    #[allow(unreachable_code)] __ret
                })
            }
        }
        impl ConsoleExporter {
            async fn export(&self, payload: &OtapPayload) {
                match payload.signal_type() {
                    SignalType::Logs => self.export_logs(payload).await,
                    SignalType::Traces => self.export_traces(payload).await,
                    SignalType::Metrics => self.export_metrics(payload).await,
                }
            }
            async fn export_logs(&self, payload: &OtapPayload) {
                match payload {
                    OtapPayload::OtlpBytes(bytes) => {
                        match RawLogsData::try_from(bytes) {
                            Ok(logs_view) => {
                                self.formatter.print_logs_data(&logs_view).await;
                            }
                            Err(e) => {
                                {
                                    const _: () = ::otap_df_telemetry::_private::validate_event_name(
                                        "console.logs_view.otlp_create_failed",
                                    );
                                    {
                                        use ::tracing::__macro_support::Callsite as _;
                                        static __CALLSITE: ::tracing::__macro_support::MacroCallsite = {
                                            static META: ::tracing::Metadata<'static> = {
                                                ::tracing_core::metadata::Metadata::new(
                                                    "console.logs_view.otlp_create_failed",
                                                    "otap-df-core-nodes",
                                                    ::tracing::Level::ERROR,
                                                    ::tracing_core::__macro_support::Option::Some(
                                                        "crates\\core-nodes\\src\\exporters\\console_exporter\\mod.rs",
                                                    ),
                                                    ::tracing_core::__macro_support::Option::Some(251u32),
                                                    ::tracing_core::__macro_support::Option::Some(
                                                        "otap_df_core_nodes::exporters::console_exporter",
                                                    ),
                                                    ::tracing_core::field::FieldSet::new(
                                                        &[
                                                            {
                                                                const NAME: ::tracing::__macro_support::FieldName<
                                                                    { ::tracing::__macro_support::FieldName::len("error") },
                                                                > = ::tracing::__macro_support::FieldName::new("error");
                                                                NAME.as_str()
                                                            },
                                                            {
                                                                const NAME: ::tracing::__macro_support::FieldName<
                                                                    { ::tracing::__macro_support::FieldName::len("message") },
                                                                > = ::tracing::__macro_support::FieldName::new("message");
                                                                NAME.as_str()
                                                            },
                                                        ],
                                                        ::tracing_core::callsite::Identifier(&__CALLSITE),
                                                    ),
                                                    ::tracing::metadata::Kind::EVENT,
                                                )
                                            };
                                            ::tracing::callsite::DefaultCallsite::new(&META)
                                        };
                                        let enabled = ::tracing::Level::ERROR
                                            <= ::tracing::level_filters::STATIC_MAX_LEVEL
                                            && ::tracing::Level::ERROR
                                                <= ::tracing::level_filters::LevelFilter::current()
                                            && {
                                                let interest = __CALLSITE.interest();
                                                !interest.is_never()
                                                    && ::tracing::__macro_support::__is_enabled(
                                                        __CALLSITE.metadata(),
                                                        interest,
                                                    )
                                            };
                                        if enabled {
                                            (|value_set: ::tracing::field::ValueSet| {
                                                let meta = __CALLSITE.metadata();
                                                ::tracing::Event::dispatch(meta, &value_set);
                                            })({
                                                #[allow(unused_imports)]
                                                use ::tracing::field::{debug, display, Value};
                                                __CALLSITE
                                                    .metadata()
                                                    .fields()
                                                    .value_set_all(
                                                        &[
                                                            (::tracing::__macro_support::Option::Some(
                                                                &::tracing::field::debug(&e) as &dyn ::tracing::field::Value,
                                                            )),
                                                            (::tracing::__macro_support::Option::Some(
                                                                &"Failed to create OTLP logs view"
                                                                    as &dyn ::tracing::field::Value,
                                                            )),
                                                        ],
                                                    )
                                            });
                                        } else {
                                        }
                                    };
                                };
                            }
                        }
                    }
                    OtapPayload::OtapArrowRecords(records) => {
                        match OtapLogsView::try_from(records) {
                            Ok(logs_view) => {
                                self.formatter.print_logs_data(&logs_view).await;
                            }
                            Err(e) => {
                                {
                                    const _: () = ::otap_df_telemetry::_private::validate_event_name(
                                        "console.logs_view.otap_create_failed",
                                    );
                                    {
                                        use ::tracing::__macro_support::Callsite as _;
                                        static __CALLSITE: ::tracing::__macro_support::MacroCallsite = {
                                            static META: ::tracing::Metadata<'static> = {
                                                ::tracing_core::metadata::Metadata::new(
                                                    "console.logs_view.otap_create_failed",
                                                    "otap-df-core-nodes",
                                                    ::tracing::Level::ERROR,
                                                    ::tracing_core::__macro_support::Option::Some(
                                                        "crates\\core-nodes\\src\\exporters\\console_exporter\\mod.rs",
                                                    ),
                                                    ::tracing_core::__macro_support::Option::Some(259u32),
                                                    ::tracing_core::__macro_support::Option::Some(
                                                        "otap_df_core_nodes::exporters::console_exporter",
                                                    ),
                                                    ::tracing_core::field::FieldSet::new(
                                                        &[
                                                            {
                                                                const NAME: ::tracing::__macro_support::FieldName<
                                                                    { ::tracing::__macro_support::FieldName::len("error") },
                                                                > = ::tracing::__macro_support::FieldName::new("error");
                                                                NAME.as_str()
                                                            },
                                                            {
                                                                const NAME: ::tracing::__macro_support::FieldName<
                                                                    { ::tracing::__macro_support::FieldName::len("message") },
                                                                > = ::tracing::__macro_support::FieldName::new("message");
                                                                NAME.as_str()
                                                            },
                                                        ],
                                                        ::tracing_core::callsite::Identifier(&__CALLSITE),
                                                    ),
                                                    ::tracing::metadata::Kind::EVENT,
                                                )
                                            };
                                            ::tracing::callsite::DefaultCallsite::new(&META)
                                        };
                                        let enabled = ::tracing::Level::ERROR
                                            <= ::tracing::level_filters::STATIC_MAX_LEVEL
                                            && ::tracing::Level::ERROR
                                                <= ::tracing::level_filters::LevelFilter::current()
                                            && {
                                                let interest = __CALLSITE.interest();
                                                !interest.is_never()
                                                    && ::tracing::__macro_support::__is_enabled(
                                                        __CALLSITE.metadata(),
                                                        interest,
                                                    )
                                            };
                                        if enabled {
                                            (|value_set: ::tracing::field::ValueSet| {
                                                let meta = __CALLSITE.metadata();
                                                ::tracing::Event::dispatch(meta, &value_set);
                                            })({
                                                #[allow(unused_imports)]
                                                use ::tracing::field::{debug, display, Value};
                                                __CALLSITE
                                                    .metadata()
                                                    .fields()
                                                    .value_set_all(
                                                        &[
                                                            (::tracing::__macro_support::Option::Some(
                                                                &::tracing::field::debug(&e) as &dyn ::tracing::field::Value,
                                                            )),
                                                            (::tracing::__macro_support::Option::Some(
                                                                &"Failed to create OTAP logs view"
                                                                    as &dyn ::tracing::field::Value,
                                                            )),
                                                        ],
                                                    )
                                            });
                                        } else {
                                        }
                                    };
                                };
                            }
                        }
                    }
                }
            }
            async fn export_traces(&self, _payload: &OtapPayload) {
                {
                    const _: () = ::otap_df_telemetry::_private::validate_event_name(
                        "console.traces.not_implemented",
                    );
                    {
                        use ::tracing::__macro_support::Callsite as _;
                        static __CALLSITE: ::tracing::__macro_support::MacroCallsite = {
                            static META: ::tracing::Metadata<'static> = {
                                ::tracing_core::metadata::Metadata::new(
                                    "console.traces.not_implemented",
                                    "otap-df-core-nodes",
                                    ::tracing::Level::ERROR,
                                    ::tracing_core::__macro_support::Option::Some(
                                        "crates\\core-nodes\\src\\exporters\\console_exporter\\mod.rs",
                                    ),
                                    ::tracing_core::__macro_support::Option::Some(267u32),
                                    ::tracing_core::__macro_support::Option::Some(
                                        "otap_df_core_nodes::exporters::console_exporter",
                                    ),
                                    ::tracing_core::field::FieldSet::new(
                                        &[
                                            {
                                                const NAME: ::tracing::__macro_support::FieldName<
                                                    { ::tracing::__macro_support::FieldName::len("message") },
                                                > = ::tracing::__macro_support::FieldName::new("message");
                                                NAME.as_str()
                                            },
                                        ],
                                        ::tracing_core::callsite::Identifier(&__CALLSITE),
                                    ),
                                    ::tracing::metadata::Kind::EVENT,
                                )
                            };
                            ::tracing::callsite::DefaultCallsite::new(&META)
                        };
                        let enabled = ::tracing::Level::ERROR
                            <= ::tracing::level_filters::STATIC_MAX_LEVEL
                            && ::tracing::Level::ERROR
                                <= ::tracing::level_filters::LevelFilter::current()
                            && {
                                let interest = __CALLSITE.interest();
                                !interest.is_never()
                                    && ::tracing::__macro_support::__is_enabled(
                                        __CALLSITE.metadata(),
                                        interest,
                                    )
                            };
                        if enabled {
                            (|value_set: ::tracing::field::ValueSet| {
                                let meta = __CALLSITE.metadata();
                                ::tracing::Event::dispatch(meta, &value_set);
                            })({
                                #[allow(unused_imports)]
                                use ::tracing::field::{debug, display, Value};
                                __CALLSITE
                                    .metadata()
                                    .fields()
                                    .value_set_all(
                                        &[
                                            (::tracing::__macro_support::Option::Some(
                                                &"Traces formatting not yet implemented"
                                                    as &dyn ::tracing::field::Value,
                                            )),
                                        ],
                                    )
                            });
                        } else {
                        }
                    };
                };
            }
            async fn export_metrics(&self, _payload: &OtapPayload) {
                {
                    const _: () = ::otap_df_telemetry::_private::validate_event_name(
                        "console.metrics.not_implemented",
                    );
                    {
                        use ::tracing::__macro_support::Callsite as _;
                        static __CALLSITE: ::tracing::__macro_support::MacroCallsite = {
                            static META: ::tracing::Metadata<'static> = {
                                ::tracing_core::metadata::Metadata::new(
                                    "console.metrics.not_implemented",
                                    "otap-df-core-nodes",
                                    ::tracing::Level::ERROR,
                                    ::tracing_core::__macro_support::Option::Some(
                                        "crates\\core-nodes\\src\\exporters\\console_exporter\\mod.rs",
                                    ),
                                    ::tracing_core::__macro_support::Option::Some(275u32),
                                    ::tracing_core::__macro_support::Option::Some(
                                        "otap_df_core_nodes::exporters::console_exporter",
                                    ),
                                    ::tracing_core::field::FieldSet::new(
                                        &[
                                            {
                                                const NAME: ::tracing::__macro_support::FieldName<
                                                    { ::tracing::__macro_support::FieldName::len("message") },
                                                > = ::tracing::__macro_support::FieldName::new("message");
                                                NAME.as_str()
                                            },
                                        ],
                                        ::tracing_core::callsite::Identifier(&__CALLSITE),
                                    ),
                                    ::tracing::metadata::Kind::EVENT,
                                )
                            };
                            ::tracing::callsite::DefaultCallsite::new(&META)
                        };
                        let enabled = ::tracing::Level::ERROR
                            <= ::tracing::level_filters::STATIC_MAX_LEVEL
                            && ::tracing::Level::ERROR
                                <= ::tracing::level_filters::LevelFilter::current()
                            && {
                                let interest = __CALLSITE.interest();
                                !interest.is_never()
                                    && ::tracing::__macro_support::__is_enabled(
                                        __CALLSITE.metadata(),
                                        interest,
                                    )
                            };
                        if enabled {
                            (|value_set: ::tracing::field::ValueSet| {
                                let meta = __CALLSITE.metadata();
                                ::tracing::Event::dispatch(meta, &value_set);
                            })({
                                #[allow(unused_imports)]
                                use ::tracing::field::{debug, display, Value};
                                __CALLSITE
                                    .metadata()
                                    .fields()
                                    .value_set_all(
                                        &[
                                            (::tracing::__macro_support::Option::Some(
                                                &"Metrics formatting not yet implemented"
                                                    as &dyn ::tracing::field::Value,
                                            )),
                                        ],
                                    )
                            });
                        } else {
                        }
                    };
                };
            }
        }
        /// Runtime-selected console formatter.
        enum ConsoleFormatter {
            Pretty(HierarchicalFormatter),
            RecordJson(RecordJsonFormatter),
        }
        impl ConsoleFormatter {
            /// Format logs and write the complete payload to stdout.
            async fn print_logs_data<L: LogsDataView>(&self, logs_data: &L) {
                let mut output = Vec::new();
                let format_result = match self {
                    Self::Pretty(formatter) => {
                        formatter.format_logs_data_to(logs_data, &mut output);
                        Ok(())
                    }
                    Self::RecordJson(formatter) => {
                        formatter.format_logs_data_to(logs_data, &mut output)
                    }
                };
                if let Err(err) = format_result {
                    {
                        const _: () = ::otap_df_telemetry::_private::validate_event_name(
                            "console.format_failed",
                        );
                        {
                            use ::tracing::__macro_support::Callsite as _;
                            static __CALLSITE: ::tracing::__macro_support::MacroCallsite = {
                                static META: ::tracing::Metadata<'static> = {
                                    ::tracing_core::metadata::Metadata::new(
                                        "console.format_failed",
                                        "otap-df-core-nodes",
                                        ::tracing::Level::ERROR,
                                        ::tracing_core::__macro_support::Option::Some(
                                            "crates\\core-nodes\\src\\exporters\\console_exporter\\mod.rs",
                                        ),
                                        ::tracing_core::__macro_support::Option::Some(301u32),
                                        ::tracing_core::__macro_support::Option::Some(
                                            "otap_df_core_nodes::exporters::console_exporter",
                                        ),
                                        ::tracing_core::field::FieldSet::new(
                                            &[
                                                {
                                                    const NAME: ::tracing::__macro_support::FieldName<
                                                        { ::tracing::__macro_support::FieldName::len("error") },
                                                    > = ::tracing::__macro_support::FieldName::new("error");
                                                    NAME.as_str()
                                                },
                                                {
                                                    const NAME: ::tracing::__macro_support::FieldName<
                                                        { ::tracing::__macro_support::FieldName::len("message") },
                                                    > = ::tracing::__macro_support::FieldName::new("message");
                                                    NAME.as_str()
                                                },
                                            ],
                                            ::tracing_core::callsite::Identifier(&__CALLSITE),
                                        ),
                                        ::tracing::metadata::Kind::EVENT,
                                    )
                                };
                                ::tracing::callsite::DefaultCallsite::new(&META)
                            };
                            let enabled = ::tracing::Level::ERROR
                                <= ::tracing::level_filters::STATIC_MAX_LEVEL
                                && ::tracing::Level::ERROR
                                    <= ::tracing::level_filters::LevelFilter::current()
                                && {
                                    let interest = __CALLSITE.interest();
                                    !interest.is_never()
                                        && ::tracing::__macro_support::__is_enabled(
                                            __CALLSITE.metadata(),
                                            interest,
                                        )
                                };
                            if enabled {
                                (|value_set: ::tracing::field::ValueSet| {
                                    let meta = __CALLSITE.metadata();
                                    ::tracing::Event::dispatch(meta, &value_set);
                                })({
                                    #[allow(unused_imports)]
                                    use ::tracing::field::{debug, display, Value};
                                    __CALLSITE
                                        .metadata()
                                        .fields()
                                        .value_set_all(
                                            &[
                                                (::tracing::__macro_support::Option::Some(
                                                    &::tracing::field::debug(&err)
                                                        as &dyn ::tracing::field::Value,
                                                )),
                                                (::tracing::__macro_support::Option::Some(
                                                    &"Could not format console output"
                                                        as &dyn ::tracing::field::Value,
                                                )),
                                            ],
                                        )
                                });
                            } else {
                            }
                        };
                    };
                    return;
                }
                use tokio::io::AsyncWriteExt;
                if let Err(err) = tokio::io::stdout().write_all(&output).await {
                    {
                        const _: () = ::otap_df_telemetry::_private::validate_event_name(
                            "console.write_failed",
                        );
                        {
                            use ::tracing::__macro_support::Callsite as _;
                            static __CALLSITE: ::tracing::__macro_support::MacroCallsite = {
                                static META: ::tracing::Metadata<'static> = {
                                    ::tracing_core::metadata::Metadata::new(
                                        "console.write_failed",
                                        "otap-df-core-nodes",
                                        ::tracing::Level::ERROR,
                                        ::tracing_core::__macro_support::Option::Some(
                                            "crates\\core-nodes\\src\\exporters\\console_exporter\\mod.rs",
                                        ),
                                        ::tracing_core::__macro_support::Option::Some(318u32),
                                        ::tracing_core::__macro_support::Option::Some(
                                            "otap_df_core_nodes::exporters::console_exporter",
                                        ),
                                        ::tracing_core::field::FieldSet::new(
                                            &[
                                                {
                                                    const NAME: ::tracing::__macro_support::FieldName<
                                                        { ::tracing::__macro_support::FieldName::len("error") },
                                                    > = ::tracing::__macro_support::FieldName::new("error");
                                                    NAME.as_str()
                                                },
                                                {
                                                    const NAME: ::tracing::__macro_support::FieldName<
                                                        { ::tracing::__macro_support::FieldName::len("message") },
                                                    > = ::tracing::__macro_support::FieldName::new("message");
                                                    NAME.as_str()
                                                },
                                            ],
                                            ::tracing_core::callsite::Identifier(&__CALLSITE),
                                        ),
                                        ::tracing::metadata::Kind::EVENT,
                                    )
                                };
                                ::tracing::callsite::DefaultCallsite::new(&META)
                            };
                            let enabled = ::tracing::Level::ERROR
                                <= ::tracing::level_filters::STATIC_MAX_LEVEL
                                && ::tracing::Level::ERROR
                                    <= ::tracing::level_filters::LevelFilter::current()
                                && {
                                    let interest = __CALLSITE.interest();
                                    !interest.is_never()
                                        && ::tracing::__macro_support::__is_enabled(
                                            __CALLSITE.metadata(),
                                            interest,
                                        )
                                };
                            if enabled {
                                (|value_set: ::tracing::field::ValueSet| {
                                    let meta = __CALLSITE.metadata();
                                    ::tracing::Event::dispatch(meta, &value_set);
                                })({
                                    #[allow(unused_imports)]
                                    use ::tracing::field::{debug, display, Value};
                                    __CALLSITE
                                        .metadata()
                                        .fields()
                                        .value_set_all(
                                            &[
                                                (::tracing::__macro_support::Option::Some(
                                                    &::tracing::field::debug(&err)
                                                        as &dyn ::tracing::field::Value,
                                                )),
                                                (::tracing::__macro_support::Option::Some(
                                                    &"Could not write to console"
                                                        as &dyn ::tracing::field::Value,
                                                )),
                                            ],
                                        )
                                });
                            } else {
                            }
                        };
                    };
                }
            }
        }
        /// Tree drawing characters (Unicode or ASCII).
        struct TreeChars {
            vertical: &'static str,
            tee: &'static str,
            corner: &'static str,
        }
        #[automatically_derived]
        #[doc(hidden)]
        unsafe impl ::core::clone::TrivialClone for TreeChars {}
        #[automatically_derived]
        impl ::core::clone::Clone for TreeChars {
            #[inline]
            fn clone(&self) -> TreeChars {
                let _: ::core::clone::AssertParamIsClone<&'static str>;
                let _: ::core::clone::AssertParamIsClone<&'static str>;
                let _: ::core::clone::AssertParamIsClone<&'static str>;
                *self
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for TreeChars {}
        impl TreeChars {
            const UNICODE: Self = Self {
                vertical: "\u{2502}",
                tee: "\u{251C}\u{2500}",
                corner: "\u{2514}\u{2500}",
            };
            const ASCII: Self = Self {
                vertical: "|",
                tee: "+-",
                corner: "\\-",
            };
        }
        /// Hierarchical formatter for OTLP data.
        pub struct HierarchicalFormatter {
            color: ColorMode,
            tree: TreeChars,
        }
        impl HierarchicalFormatter {
            /// Create a new hierarchical formatter.
            #[must_use]
            pub const fn new(use_color: bool, use_unicode: bool) -> Self {
                Self {
                    color: if use_color { ColorMode::Color } else { ColorMode::NoColor },
                    tree: if use_unicode { TreeChars::UNICODE } else { TreeChars::ASCII },
                }
            }
            /// Format logs from a LogsDataView to a writer.
            fn format_logs_data_to<L: LogsDataView>(
                &self,
                logs_data: &L,
                output: &mut Vec<u8>,
            ) {
                for resource_logs in logs_data.resources() {
                    self.format_resource_logs_to(&resource_logs, output);
                }
            }
            /// Format a ResourceLogs with its nested scopes.
            fn format_resource_logs_to<R: ResourceLogsView>(
                &self,
                resource_logs: &R,
                output: &mut Vec<u8>,
            ) {
                let first_ts = self.get_first_log_timestamp(resource_logs);
                self.format_line(
                    output,
                    |w| {
                        w.format_header_line(
                            Some(first_ts),
                            resource_logs.resource().iter().flat_map(|r| r.attributes()),
                            |w| {
                                w.write_styled(
                                    AnsiCode::Cyan,
                                    |w| {
                                        let _ = w.write_all(b"RESOURCE");
                                    },
                                );
                                let _ = w.write_all(b"   ");
                            },
                            |w| {
                                let _ = w.write_all(b"v1.Resource");
                            },
                            |_| {},
                        );
                    },
                );
                let mut scopes = resource_logs.scopes().peekable();
                while let Some(scope_logs) = scopes.next() {
                    let is_last_scope = scopes.peek().is_none();
                    self.format_scope_logs_to(&scope_logs, is_last_scope, output);
                }
            }
            /// Get the first timestamp from log records in a ResourceLogs.
            fn get_first_log_timestamp<R: ResourceLogsView>(
                &self,
                resource_logs: &R,
            ) -> SystemTime {
                for scope_logs in resource_logs.scopes() {
                    for log_record in scope_logs.log_records() {
                        if let Some(ts) = log_record.time_unix_nano() {
                            return nanos_to_time(ts);
                        }
                        if let Some(ts) = log_record.observed_time_unix_nano() {
                            return nanos_to_time(ts);
                        }
                    }
                }
                SystemTime::UNIX_EPOCH
            }
            /// Format a ScopeLogs with its nested log records.
            fn format_scope_logs_to<S: ScopeLogsView>(
                &self,
                scope_logs: &S,
                is_last_scope: bool,
                output: &mut Vec<u8>,
            ) {
                let first_ts = scope_logs
                    .log_records()
                    .find_map(|lr| {
                        lr.time_unix_nano().or_else(|| lr.observed_time_unix_nano())
                    })
                    .map(nanos_to_time)
                    .unwrap_or(SystemTime::UNIX_EPOCH);
                let prefix = self.tree.vertical;
                let scope = scope_logs.scope();
                let name = scope.as_ref().and_then(|s| s.name());
                let version = scope.as_ref().and_then(|s| s.version());
                self.format_line(
                    output,
                    |w| {
                        w.format_header_line(
                            Some(first_ts),
                            scope.iter().flat_map(|s| s.attributes()),
                            |w| {
                                let _ = w.write_all(prefix.as_bytes());
                                let _ = w.write_all(b" ");
                                w.write_styled(
                                    AnsiCode::Magenta,
                                    |w| {
                                        let _ = w.write_all(b"SCOPE");
                                    },
                                );
                                let _ = w.write_all(b"    ");
                            },
                            |w| match (name, version) {
                                (Some(n), Some(v)) => {
                                    let _ = w.write_all(n);
                                    let _ = w.write_all(b"/");
                                    let _ = w.write_all(v);
                                }
                                (Some(n), None) => {
                                    let _ = w.write_all(n);
                                }
                                _ => {
                                    let _ = w.write_all(b"v1.InstrumentationScope");
                                }
                            },
                            |_| {},
                        );
                    },
                );
                let mut records = scope_logs.log_records().peekable();
                while let Some(log_record) = records.next() {
                    let is_last_record = records.peek().is_none();
                    self.format_log_record_to(
                        &log_record,
                        is_last_scope,
                        is_last_record,
                        output,
                    );
                }
            }
            /// Format a single log record.
            fn format_log_record_to<L: LogRecordView>(
                &self,
                log_record: &L,
                is_last_scope: bool,
                is_last_record: bool,
                output: &mut Vec<u8>,
            ) {
                let time = log_record
                    .time_unix_nano()
                    .or_else(|| log_record.observed_time_unix_nano())
                    .map(nanos_to_time)
                    .unwrap_or(SystemTime::UNIX_EPOCH);
                let event_name = log_record
                    .event_name()
                    .map(|s| String::from_utf8_lossy(s).into_owned());
                let severity = log_record.severity_number();
                let severity_text = log_record.severity_text();
                let tree = self.tree;
                self.format_line(
                    output,
                    |w| {
                        w.format_log_line(
                            Some(time),
                            log_record,
                            |w| {
                                let _ = w.write_all(tree.vertical.as_bytes());
                                let _ = w.write_all(b" ");
                                if is_last_record && is_last_scope {
                                    let _ = w.write_all(tree.corner.as_bytes());
                                } else {
                                    let _ = w.write_all(tree.tee.as_bytes());
                                }
                                let _ = w.write_all(b" ");
                                w.write_severity(
                                    severity,
                                    severity_text.as_ref().map(|s| s.as_ref()),
                                );
                            },
                            |w| {
                                if let Some(name) = event_name {
                                    let _ = w.write_all(name.as_bytes());
                                }
                            },
                            |_| {},
                        );
                    },
                );
            }
            /// Format a line to the output buffer.
            fn format_line<F>(&self, output: &mut Vec<u8>, f: F)
            where
                F: FnOnce(&mut StyledBufWriter<'_>),
            {
                let mut buf = [0u8; LOG_BUFFER_SIZE];
                let mut w = StyledBufWriter::new(&mut buf, self.color);
                f(&mut w);
                let len = w.position();
                output.extend_from_slice(&buf[..len]);
            }
        }
        /// Convert nanoseconds since UNIX_EPOCH to SystemTime.
        #[inline]
        fn nanos_to_time(nanos: u64) -> SystemTime {
            SystemTime::UNIX_EPOCH + Duration::from_nanos(nanos)
        }
    }
    /// Noop exporter.
    pub mod noop_exporter {
        use async_trait::async_trait;
        use linkme::distributed_slice;
        use otap_df_config::node::NodeUserConfig;
        use otap_df_engine::config::ExporterConfig;
        use otap_df_engine::context::PipelineContext;
        use otap_df_engine::control::{AckMsg, NodeControlMsg};
        use otap_df_engine::error::Error;
        use otap_df_engine::exporter::ExporterWrapper;
        use otap_df_engine::local::exporter::{EffectHandler, Exporter};
        use otap_df_engine::message::{ExporterInbox, Message};
        use otap_df_engine::node::NodeId;
        use otap_df_engine::terminal_state::TerminalState;
        use otap_df_engine::{ConsumerEffectHandlerExtension, ExporterFactory};
        use otap_df_otap::OTAP_EXPORTER_FACTORIES;
        use otap_df_otap::pdata::OtapPdata;
        use std::sync::Arc;
        /// The URN for the noop exporter.
        pub const NOOP_EXPORTER_URN: &str = "urn:otel:exporter:noop";
        /// Exporter that does nothing.
        pub struct NoopExporter;
        #[used]
        #[unsafe(link_section = ".linkme_OTAP_EXPORTER_FACTORIES$b")]
        /// Declare the Noop Exporter as a local exporter factory.
        #[allow(unsafe_code)]
        pub static NOOP_EXPORTER: ExporterFactory<OtapPdata> = {
            #[allow(clippy::no_effect_underscore_binding)]
            unsafe fn __typecheck(_: ::linkme::__private37::Void) {
                #[allow(clippy::ref_option_ref)]
                let __new = || -> fn() -> &'static ExporterFactory<OtapPdata> {
                    || &NOOP_EXPORTER
                };
                unsafe {
                    ::linkme::DistributedSlice::private_typecheck(
                        OTAP_EXPORTER_FACTORIES,
                        __new(),
                    );
                }
            }
            ExporterFactory {
                name: NOOP_EXPORTER_URN,
                create: |
                    _pipeline: PipelineContext,
                    node: NodeId,
                    node_config: Arc<NodeUserConfig>,
                    exporter_config: &ExporterConfig,
                    _capabilities: &otap_df_engine::capability::registry::Capabilities|
                {
                    Ok(
                        ExporterWrapper::local(
                            NoopExporter {},
                            node,
                            node_config,
                            exporter_config,
                        ),
                    )
                },
                wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
                validate_config: otap_df_config::validation::no_config,
            }
        };
        #[used]
        #[unsafe(link_section = ".linkme_COMPONENT_INVENTORY$b")]
        #[allow(unsafe_code)]
        #[allow(non_upper_case_globals)]
        #[doc(hidden)]
        static _COMPONENT_META_NOOP_EXPORTER: ::otap_df_engine::inventory::ComponentMeta = {
            #[allow(clippy::no_effect_underscore_binding)]
            unsafe fn __typecheck(_: ::linkme::__private37::Void) {
                #[allow(clippy::ref_option_ref)]
                let __new = || -> fn() -> &'static ::otap_df_engine::inventory::ComponentMeta {
                    || &_COMPONENT_META_NOOP_EXPORTER
                };
                unsafe {
                    ::linkme::DistributedSlice::private_typecheck(
                        ::otap_df_engine::inventory::COMPONENT_INVENTORY,
                        __new(),
                    );
                }
            }
            ::otap_df_engine::inventory::ComponentMeta {
                id: NOOP_EXPORTER_URN,
                category: ::otap_df_engine::inventory::Category::Exporter,
                description: ::core::option::Option::None,
                file: "crates\\core-nodes\\src\\exporters\\noop_exporter\\mod.rs",
                line: 31u32,
                attributes: &[],
            }
        };
        impl Exporter<OtapPdata> for NoopExporter {
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn start<'async_trait>(
                self: Box<Self>,
                msg_chan: ExporterInbox<OtapPdata>,
                effect_handler: EffectHandler<OtapPdata>,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = Result<TerminalState, Error>,
                    > + 'async_trait,
                >,
            >
            where
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        Result<TerminalState, Error>,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let mut msg_chan = msg_chan;
                    let effect_handler = effect_handler;
                    let __ret: Result<TerminalState, Error> = {
                        loop {
                            match msg_chan.recv().await? {
                                Message::Control(NodeControlMsg::Shutdown { .. }) => break,
                                Message::PData(data) => {
                                    effect_handler.notify_ack(AckMsg::new(data)).await?;
                                }
                                _ => {}
                            }
                        }
                        Ok(TerminalState::default())
                    };
                    #[allow(unreachable_code)] __ret
                })
            }
        }
    }
}
