impl serde::Serialize for ExportTracePartialSuccess {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.rejected_spans != 0 {
            len += 1;
        }
        if !self.error_message.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("opentelemetry.proto.collector.trace.v1.ExportTracePartialSuccess", len)?;
        if self.rejected_spans != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("rejectedSpans", ToString::to_string(&self.rejected_spans).as_str())?;
        }
        if !self.error_message.is_empty() {
            struct_ser.serialize_field("errorMessage", &self.error_message)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ExportTracePartialSuccess {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "rejected_spans",
            "rejectedSpans",
            "error_message",
            "errorMessage",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            RejectedSpans,
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
                            "rejectedSpans" | "rejected_spans" => Ok(GeneratedField::RejectedSpans),
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
            type Value = ExportTracePartialSuccess;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct opentelemetry.proto.collector.trace.v1.ExportTracePartialSuccess")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ExportTracePartialSuccess, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut rejected_spans__ = None;
                let mut error_message__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::RejectedSpans => {
                            if rejected_spans__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rejectedSpans"));
                            }
                            rejected_spans__ = 
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
                Ok(ExportTracePartialSuccess {
                    rejected_spans: rejected_spans__.unwrap_or_default(),
                    error_message: error_message__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("opentelemetry.proto.collector.trace.v1.ExportTracePartialSuccess", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ExportTraceServiceRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.resource_spans.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("opentelemetry.proto.collector.trace.v1.ExportTraceServiceRequest", len)?;
        if !self.resource_spans.is_empty() {
            struct_ser.serialize_field("resourceSpans", &self.resource_spans)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ExportTraceServiceRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "resource_spans",
            "resourceSpans",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ResourceSpans,
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
                            "resourceSpans" | "resource_spans" => Ok(GeneratedField::ResourceSpans),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ExportTraceServiceRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct opentelemetry.proto.collector.trace.v1.ExportTraceServiceRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ExportTraceServiceRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut resource_spans__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ResourceSpans => {
                            if resource_spans__.is_some() {
                                return Err(serde::de::Error::duplicate_field("resourceSpans"));
                            }
                            resource_spans__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ExportTraceServiceRequest {
                    resource_spans: resource_spans__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("opentelemetry.proto.collector.trace.v1.ExportTraceServiceRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ExportTraceServiceResponse {
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
        let mut struct_ser = serializer.serialize_struct("opentelemetry.proto.collector.trace.v1.ExportTraceServiceResponse", len)?;
        if let Some(v) = self.partial_success.as_ref() {
            struct_ser.serialize_field("partialSuccess", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ExportTraceServiceResponse {
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
            type Value = ExportTraceServiceResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct opentelemetry.proto.collector.trace.v1.ExportTraceServiceResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ExportTraceServiceResponse, V::Error>
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
                Ok(ExportTraceServiceResponse {
                    partial_success: partial_success__,
                })
            }
        }
        deserializer.deserialize_struct("opentelemetry.proto.collector.trace.v1.ExportTraceServiceResponse", FIELDS, GeneratedVisitor)
    }
}
