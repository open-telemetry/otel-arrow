impl serde::Serialize for LogRecord {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.time_unix_nano != 0 {
            len += 1;
        }
        if self.observed_time_unix_nano != 0 {
            len += 1;
        }
        if self.severity_number != 0 {
            len += 1;
        }
        if !self.severity_text.is_empty() {
            len += 1;
        }
        if self.body.is_some() {
            len += 1;
        }
        if !self.attributes.is_empty() {
            len += 1;
        }
        if self.dropped_attributes_count != 0 {
            len += 1;
        }
        if self.flags != 0 {
            len += 1;
        }
        if !self.trace_id.is_empty() {
            len += 1;
        }
        if !self.span_id.is_empty() {
            len += 1;
        }
        if !self.event_name.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("opentelemetry.proto.logs.v1.LogRecord", len)?;
        if self.time_unix_nano != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("timeUnixNano", ToString::to_string(&self.time_unix_nano).as_str())?;
        }
        if self.observed_time_unix_nano != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("observedTimeUnixNano", ToString::to_string(&self.observed_time_unix_nano).as_str())?;
        }
        if self.severity_number != 0 {
            let v = SeverityNumber::try_from(self.severity_number)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.severity_number)))?;
            struct_ser.serialize_field("severityNumber", &v)?;
        }
        if !self.severity_text.is_empty() {
            struct_ser.serialize_field("severityText", &self.severity_text)?;
        }
        if let Some(v) = self.body.as_ref() {
            struct_ser.serialize_field("body", v)?;
        }
        if !self.attributes.is_empty() {
            struct_ser.serialize_field("attributes", &self.attributes)?;
        }
        if self.dropped_attributes_count != 0 {
            struct_ser.serialize_field("droppedAttributesCount", &self.dropped_attributes_count)?;
        }
        if self.flags != 0 {
            struct_ser.serialize_field("flags", &self.flags)?;
        }
        if !self.trace_id.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("traceId", pbjson::private::base64::encode(&self.trace_id).as_str())?;
        }
        if !self.span_id.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("spanId", pbjson::private::base64::encode(&self.span_id).as_str())?;
        }
        if !self.event_name.is_empty() {
            struct_ser.serialize_field("eventName", &self.event_name)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for LogRecord {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "time_unix_nano",
            "timeUnixNano",
            "observed_time_unix_nano",
            "observedTimeUnixNano",
            "severity_number",
            "severityNumber",
            "severity_text",
            "severityText",
            "body",
            "attributes",
            "dropped_attributes_count",
            "droppedAttributesCount",
            "flags",
            "trace_id",
            "traceId",
            "span_id",
            "spanId",
            "event_name",
            "eventName",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TimeUnixNano,
            ObservedTimeUnixNano,
            SeverityNumber,
            SeverityText,
            Body,
            Attributes,
            DroppedAttributesCount,
            Flags,
            TraceId,
            SpanId,
            EventName,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "timeUnixNano" | "time_unix_nano" => Ok(GeneratedField::TimeUnixNano),
                            "observedTimeUnixNano" | "observed_time_unix_nano" => Ok(GeneratedField::ObservedTimeUnixNano),
                            "severityNumber" | "severity_number" => Ok(GeneratedField::SeverityNumber),
                            "severityText" | "severity_text" => Ok(GeneratedField::SeverityText),
                            "body" => Ok(GeneratedField::Body),
                            "attributes" => Ok(GeneratedField::Attributes),
                            "droppedAttributesCount" | "dropped_attributes_count" => Ok(GeneratedField::DroppedAttributesCount),
                            "flags" => Ok(GeneratedField::Flags),
                            "traceId" | "trace_id" => Ok(GeneratedField::TraceId),
                            "spanId" | "span_id" => Ok(GeneratedField::SpanId),
                            "eventName" | "event_name" => Ok(GeneratedField::EventName),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = LogRecord;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct opentelemetry.proto.logs.v1.LogRecord")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<LogRecord, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut time_unix_nano__ = None;
                let mut observed_time_unix_nano__ = None;
                let mut severity_number__ = None;
                let mut severity_text__ = None;
                let mut body__ = None;
                let mut attributes__ = None;
                let mut dropped_attributes_count__ = None;
                let mut flags__ = None;
                let mut trace_id__ = None;
                let mut span_id__ = None;
                let mut event_name__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::TimeUnixNano => {
                            if time_unix_nano__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timeUnixNano"));
                            }
                            time_unix_nano__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ObservedTimeUnixNano => {
                            if observed_time_unix_nano__.is_some() {
                                return Err(serde::de::Error::duplicate_field("observedTimeUnixNano"));
                            }
                            observed_time_unix_nano__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::SeverityNumber => {
                            if severity_number__.is_some() {
                                return Err(serde::de::Error::duplicate_field("severityNumber"));
                            }
                            severity_number__ = Some(map_.next_value::<SeverityNumber>()? as i32);
                        }
                        GeneratedField::SeverityText => {
                            if severity_text__.is_some() {
                                return Err(serde::de::Error::duplicate_field("severityText"));
                            }
                            severity_text__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Body => {
                            if body__.is_some() {
                                return Err(serde::de::Error::duplicate_field("body"));
                            }
                            body__ = map_.next_value()?;
                        }
                        GeneratedField::Attributes => {
                            if attributes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("attributes"));
                            }
                            attributes__ = Some(map_.next_value()?);
                        }
                        GeneratedField::DroppedAttributesCount => {
                            if dropped_attributes_count__.is_some() {
                                return Err(serde::de::Error::duplicate_field("droppedAttributesCount"));
                            }
                            dropped_attributes_count__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Flags => {
                            if flags__.is_some() {
                                return Err(serde::de::Error::duplicate_field("flags"));
                            }
                            flags__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::TraceId => {
                            if trace_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("traceId"));
                            }
                            trace_id__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::SpanId => {
                            if span_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("spanId"));
                            }
                            span_id__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::EventName => {
                            if event_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("eventName"));
                            }
                            event_name__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(LogRecord {
                    time_unix_nano: time_unix_nano__.unwrap_or_default(),
                    observed_time_unix_nano: observed_time_unix_nano__.unwrap_or_default(),
                    severity_number: severity_number__.unwrap_or_default(),
                    severity_text: severity_text__.unwrap_or_default(),
                    body: body__,
                    attributes: attributes__.unwrap_or_default(),
                    dropped_attributes_count: dropped_attributes_count__.unwrap_or_default(),
                    flags: flags__.unwrap_or_default(),
                    trace_id: trace_id__.unwrap_or_default(),
                    span_id: span_id__.unwrap_or_default(),
                    event_name: event_name__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("opentelemetry.proto.logs.v1.LogRecord", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for LogRecordFlags {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::DoNotUse => "LOG_RECORD_FLAGS_DO_NOT_USE",
            Self::TraceFlagsMask => "LOG_RECORD_FLAGS_TRACE_FLAGS_MASK",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for LogRecordFlags {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "LOG_RECORD_FLAGS_DO_NOT_USE",
            "LOG_RECORD_FLAGS_TRACE_FLAGS_MASK",
        ];

        struct GeneratedVisitor;

        impl serde::de::Visitor<'_> for GeneratedVisitor {
            type Value = LogRecordFlags;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "LOG_RECORD_FLAGS_DO_NOT_USE" => Ok(LogRecordFlags::DoNotUse),
                    "LOG_RECORD_FLAGS_TRACE_FLAGS_MASK" => Ok(LogRecordFlags::TraceFlagsMask),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for LogsData {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.resource_logs.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("opentelemetry.proto.logs.v1.LogsData", len)?;
        if !self.resource_logs.is_empty() {
            struct_ser.serialize_field("resourceLogs", &self.resource_logs)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for LogsData {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "resource_logs",
            "resourceLogs",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ResourceLogs,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "resourceLogs" | "resource_logs" => Ok(GeneratedField::ResourceLogs),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = LogsData;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct opentelemetry.proto.logs.v1.LogsData")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<LogsData, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut resource_logs__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ResourceLogs => {
                            if resource_logs__.is_some() {
                                return Err(serde::de::Error::duplicate_field("resourceLogs"));
                            }
                            resource_logs__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(LogsData {
                    resource_logs: resource_logs__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("opentelemetry.proto.logs.v1.LogsData", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ResourceLogs {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.resource.is_some() {
            len += 1;
        }
        if !self.scope_logs.is_empty() {
            len += 1;
        }
        if !self.schema_url.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("opentelemetry.proto.logs.v1.ResourceLogs", len)?;
        if let Some(v) = self.resource.as_ref() {
            struct_ser.serialize_field("resource", v)?;
        }
        if !self.scope_logs.is_empty() {
            struct_ser.serialize_field("scopeLogs", &self.scope_logs)?;
        }
        if !self.schema_url.is_empty() {
            struct_ser.serialize_field("schemaUrl", &self.schema_url)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ResourceLogs {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "resource",
            "scope_logs",
            "scopeLogs",
            "schema_url",
            "schemaUrl",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Resource,
            ScopeLogs,
            SchemaUrl,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "resource" => Ok(GeneratedField::Resource),
                            "scopeLogs" | "scope_logs" => Ok(GeneratedField::ScopeLogs),
                            "schemaUrl" | "schema_url" => Ok(GeneratedField::SchemaUrl),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ResourceLogs;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct opentelemetry.proto.logs.v1.ResourceLogs")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ResourceLogs, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut resource__ = None;
                let mut scope_logs__ = None;
                let mut schema_url__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Resource => {
                            if resource__.is_some() {
                                return Err(serde::de::Error::duplicate_field("resource"));
                            }
                            resource__ = map_.next_value()?;
                        }
                        GeneratedField::ScopeLogs => {
                            if scope_logs__.is_some() {
                                return Err(serde::de::Error::duplicate_field("scopeLogs"));
                            }
                            scope_logs__ = Some(map_.next_value()?);
                        }
                        GeneratedField::SchemaUrl => {
                            if schema_url__.is_some() {
                                return Err(serde::de::Error::duplicate_field("schemaUrl"));
                            }
                            schema_url__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ResourceLogs {
                    resource: resource__,
                    scope_logs: scope_logs__.unwrap_or_default(),
                    schema_url: schema_url__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("opentelemetry.proto.logs.v1.ResourceLogs", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ScopeLogs {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.scope.is_some() {
            len += 1;
        }
        if !self.log_records.is_empty() {
            len += 1;
        }
        if !self.schema_url.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("opentelemetry.proto.logs.v1.ScopeLogs", len)?;
        if let Some(v) = self.scope.as_ref() {
            struct_ser.serialize_field("scope", v)?;
        }
        if !self.log_records.is_empty() {
            struct_ser.serialize_field("logRecords", &self.log_records)?;
        }
        if !self.schema_url.is_empty() {
            struct_ser.serialize_field("schemaUrl", &self.schema_url)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ScopeLogs {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "scope",
            "log_records",
            "logRecords",
            "schema_url",
            "schemaUrl",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Scope,
            LogRecords,
            SchemaUrl,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "scope" => Ok(GeneratedField::Scope),
                            "logRecords" | "log_records" => Ok(GeneratedField::LogRecords),
                            "schemaUrl" | "schema_url" => Ok(GeneratedField::SchemaUrl),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ScopeLogs;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct opentelemetry.proto.logs.v1.ScopeLogs")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ScopeLogs, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut scope__ = None;
                let mut log_records__ = None;
                let mut schema_url__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Scope => {
                            if scope__.is_some() {
                                return Err(serde::de::Error::duplicate_field("scope"));
                            }
                            scope__ = map_.next_value()?;
                        }
                        GeneratedField::LogRecords => {
                            if log_records__.is_some() {
                                return Err(serde::de::Error::duplicate_field("logRecords"));
                            }
                            log_records__ = Some(map_.next_value()?);
                        }
                        GeneratedField::SchemaUrl => {
                            if schema_url__.is_some() {
                                return Err(serde::de::Error::duplicate_field("schemaUrl"));
                            }
                            schema_url__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ScopeLogs {
                    scope: scope__,
                    log_records: log_records__.unwrap_or_default(),
                    schema_url: schema_url__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("opentelemetry.proto.logs.v1.ScopeLogs", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SeverityNumber {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "SEVERITY_NUMBER_UNSPECIFIED",
            Self::Trace => "SEVERITY_NUMBER_TRACE",
            Self::Trace2 => "SEVERITY_NUMBER_TRACE2",
            Self::Trace3 => "SEVERITY_NUMBER_TRACE3",
            Self::Trace4 => "SEVERITY_NUMBER_TRACE4",
            Self::Debug => "SEVERITY_NUMBER_DEBUG",
            Self::Debug2 => "SEVERITY_NUMBER_DEBUG2",
            Self::Debug3 => "SEVERITY_NUMBER_DEBUG3",
            Self::Debug4 => "SEVERITY_NUMBER_DEBUG4",
            Self::Info => "SEVERITY_NUMBER_INFO",
            Self::Info2 => "SEVERITY_NUMBER_INFO2",
            Self::Info3 => "SEVERITY_NUMBER_INFO3",
            Self::Info4 => "SEVERITY_NUMBER_INFO4",
            Self::Warn => "SEVERITY_NUMBER_WARN",
            Self::Warn2 => "SEVERITY_NUMBER_WARN2",
            Self::Warn3 => "SEVERITY_NUMBER_WARN3",
            Self::Warn4 => "SEVERITY_NUMBER_WARN4",
            Self::Error => "SEVERITY_NUMBER_ERROR",
            Self::Error2 => "SEVERITY_NUMBER_ERROR2",
            Self::Error3 => "SEVERITY_NUMBER_ERROR3",
            Self::Error4 => "SEVERITY_NUMBER_ERROR4",
            Self::Fatal => "SEVERITY_NUMBER_FATAL",
            Self::Fatal2 => "SEVERITY_NUMBER_FATAL2",
            Self::Fatal3 => "SEVERITY_NUMBER_FATAL3",
            Self::Fatal4 => "SEVERITY_NUMBER_FATAL4",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for SeverityNumber {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "SEVERITY_NUMBER_UNSPECIFIED",
            "SEVERITY_NUMBER_TRACE",
            "SEVERITY_NUMBER_TRACE2",
            "SEVERITY_NUMBER_TRACE3",
            "SEVERITY_NUMBER_TRACE4",
            "SEVERITY_NUMBER_DEBUG",
            "SEVERITY_NUMBER_DEBUG2",
            "SEVERITY_NUMBER_DEBUG3",
            "SEVERITY_NUMBER_DEBUG4",
            "SEVERITY_NUMBER_INFO",
            "SEVERITY_NUMBER_INFO2",
            "SEVERITY_NUMBER_INFO3",
            "SEVERITY_NUMBER_INFO4",
            "SEVERITY_NUMBER_WARN",
            "SEVERITY_NUMBER_WARN2",
            "SEVERITY_NUMBER_WARN3",
            "SEVERITY_NUMBER_WARN4",
            "SEVERITY_NUMBER_ERROR",
            "SEVERITY_NUMBER_ERROR2",
            "SEVERITY_NUMBER_ERROR3",
            "SEVERITY_NUMBER_ERROR4",
            "SEVERITY_NUMBER_FATAL",
            "SEVERITY_NUMBER_FATAL2",
            "SEVERITY_NUMBER_FATAL3",
            "SEVERITY_NUMBER_FATAL4",
        ];

        struct GeneratedVisitor;

        impl serde::de::Visitor<'_> for GeneratedVisitor {
            type Value = SeverityNumber;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "SEVERITY_NUMBER_UNSPECIFIED" => Ok(SeverityNumber::Unspecified),
                    "SEVERITY_NUMBER_TRACE" => Ok(SeverityNumber::Trace),
                    "SEVERITY_NUMBER_TRACE2" => Ok(SeverityNumber::Trace2),
                    "SEVERITY_NUMBER_TRACE3" => Ok(SeverityNumber::Trace3),
                    "SEVERITY_NUMBER_TRACE4" => Ok(SeverityNumber::Trace4),
                    "SEVERITY_NUMBER_DEBUG" => Ok(SeverityNumber::Debug),
                    "SEVERITY_NUMBER_DEBUG2" => Ok(SeverityNumber::Debug2),
                    "SEVERITY_NUMBER_DEBUG3" => Ok(SeverityNumber::Debug3),
                    "SEVERITY_NUMBER_DEBUG4" => Ok(SeverityNumber::Debug4),
                    "SEVERITY_NUMBER_INFO" => Ok(SeverityNumber::Info),
                    "SEVERITY_NUMBER_INFO2" => Ok(SeverityNumber::Info2),
                    "SEVERITY_NUMBER_INFO3" => Ok(SeverityNumber::Info3),
                    "SEVERITY_NUMBER_INFO4" => Ok(SeverityNumber::Info4),
                    "SEVERITY_NUMBER_WARN" => Ok(SeverityNumber::Warn),
                    "SEVERITY_NUMBER_WARN2" => Ok(SeverityNumber::Warn2),
                    "SEVERITY_NUMBER_WARN3" => Ok(SeverityNumber::Warn3),
                    "SEVERITY_NUMBER_WARN4" => Ok(SeverityNumber::Warn4),
                    "SEVERITY_NUMBER_ERROR" => Ok(SeverityNumber::Error),
                    "SEVERITY_NUMBER_ERROR2" => Ok(SeverityNumber::Error2),
                    "SEVERITY_NUMBER_ERROR3" => Ok(SeverityNumber::Error3),
                    "SEVERITY_NUMBER_ERROR4" => Ok(SeverityNumber::Error4),
                    "SEVERITY_NUMBER_FATAL" => Ok(SeverityNumber::Fatal),
                    "SEVERITY_NUMBER_FATAL2" => Ok(SeverityNumber::Fatal2),
                    "SEVERITY_NUMBER_FATAL3" => Ok(SeverityNumber::Fatal3),
                    "SEVERITY_NUMBER_FATAL4" => Ok(SeverityNumber::Fatal4),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
