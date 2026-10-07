impl serde::Serialize for ExportLogsPartialSuccess {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.rejected_log_records != 0 {
            len += 1;
        }
        if !self.error_message.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("opentelemetry.proto.collector.logs.v1.ExportLogsPartialSuccess", len)?;
        if self.rejected_log_records != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("rejectedLogRecords", ToString::to_string(&self.rejected_log_records).as_str())?;
        }
        if !self.error_message.is_empty() {
            struct_ser.serialize_field("errorMessage", &self.error_message)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ExportLogsPartialSuccess {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "rejected_log_records",
            "rejectedLogRecords",
            "error_message",
            "errorMessage",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            RejectedLogRecords,
            ErrorMessage,
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
                            "rejectedLogRecords" | "rejected_log_records" => Ok(GeneratedField::RejectedLogRecords),
                            "errorMessage" | "error_message" => Ok(GeneratedField::ErrorMessage),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ExportLogsPartialSuccess;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct opentelemetry.proto.collector.logs.v1.ExportLogsPartialSuccess")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ExportLogsPartialSuccess, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut rejected_log_records__ = None;
                let mut error_message__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::RejectedLogRecords => {
                            if rejected_log_records__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rejectedLogRecords"));
                            }
                            rejected_log_records__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ErrorMessage => {
                            if error_message__.is_some() {
                                return Err(serde::de::Error::duplicate_field("errorMessage"));
                            }
                            error_message__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ExportLogsPartialSuccess {
                    rejected_log_records: rejected_log_records__.unwrap_or_default(),
                    error_message: error_message__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("opentelemetry.proto.collector.logs.v1.ExportLogsPartialSuccess", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ExportLogsServiceRequest {
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
        let mut struct_ser = serializer.serialize_struct("opentelemetry.proto.collector.logs.v1.ExportLogsServiceRequest", len)?;
        if !self.resource_logs.is_empty() {
            struct_ser.serialize_field("resourceLogs", &self.resource_logs)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ExportLogsServiceRequest {
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
            type Value = ExportLogsServiceRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct opentelemetry.proto.collector.logs.v1.ExportLogsServiceRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ExportLogsServiceRequest, V::Error>
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
                Ok(ExportLogsServiceRequest {
                    resource_logs: resource_logs__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("opentelemetry.proto.collector.logs.v1.ExportLogsServiceRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ExportLogsServiceResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.partial_success.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("opentelemetry.proto.collector.logs.v1.ExportLogsServiceResponse", len)?;
        if let Some(v) = self.partial_success.as_ref() {
            struct_ser.serialize_field("partialSuccess", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ExportLogsServiceResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "partial_success",
            "partialSuccess",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PartialSuccess,
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
                            "partialSuccess" | "partial_success" => Ok(GeneratedField::PartialSuccess),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ExportLogsServiceResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct opentelemetry.proto.collector.logs.v1.ExportLogsServiceResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ExportLogsServiceResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut partial_success__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PartialSuccess => {
                            if partial_success__.is_some() {
                                return Err(serde::de::Error::duplicate_field("partialSuccess"));
                            }
                            partial_success__ = map_.next_value()?;
                        }
                    }
                }
                Ok(ExportLogsServiceResponse {
                    partial_success: partial_success__,
                })
            }
        }
        deserializer.deserialize_struct("opentelemetry.proto.collector.logs.v1.ExportLogsServiceResponse", FIELDS, GeneratedVisitor)
    }
}
