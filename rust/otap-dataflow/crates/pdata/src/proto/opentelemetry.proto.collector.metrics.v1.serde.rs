impl serde::Serialize for ExportMetricsPartialSuccess {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.rejected_data_points != 0 {
            len += 1;
        }
        if !self.error_message.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("opentelemetry.proto.collector.metrics.v1.ExportMetricsPartialSuccess", len)?;
        if self.rejected_data_points != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("rejectedDataPoints", ToString::to_string(&self.rejected_data_points).as_str())?;
        }
        if !self.error_message.is_empty() {
            struct_ser.serialize_field("errorMessage", &self.error_message)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ExportMetricsPartialSuccess {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "rejected_data_points",
            "rejectedDataPoints",
            "error_message",
            "errorMessage",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            RejectedDataPoints,
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
                            "rejectedDataPoints" | "rejected_data_points" => Ok(GeneratedField::RejectedDataPoints),
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
            type Value = ExportMetricsPartialSuccess;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct opentelemetry.proto.collector.metrics.v1.ExportMetricsPartialSuccess")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ExportMetricsPartialSuccess, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut rejected_data_points__ = None;
                let mut error_message__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::RejectedDataPoints => {
                            if rejected_data_points__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rejectedDataPoints"));
                            }
                            rejected_data_points__ = 
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
                Ok(ExportMetricsPartialSuccess {
                    rejected_data_points: rejected_data_points__.unwrap_or_default(),
                    error_message: error_message__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("opentelemetry.proto.collector.metrics.v1.ExportMetricsPartialSuccess", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ExportMetricsServiceRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.resource_metrics.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("opentelemetry.proto.collector.metrics.v1.ExportMetricsServiceRequest", len)?;
        if !self.resource_metrics.is_empty() {
            struct_ser.serialize_field("resourceMetrics", &self.resource_metrics)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ExportMetricsServiceRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "resource_metrics",
            "resourceMetrics",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ResourceMetrics,
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
                            "resourceMetrics" | "resource_metrics" => Ok(GeneratedField::ResourceMetrics),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ExportMetricsServiceRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct opentelemetry.proto.collector.metrics.v1.ExportMetricsServiceRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ExportMetricsServiceRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut resource_metrics__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ResourceMetrics => {
                            if resource_metrics__.is_some() {
                                return Err(serde::de::Error::duplicate_field("resourceMetrics"));
                            }
                            resource_metrics__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ExportMetricsServiceRequest {
                    resource_metrics: resource_metrics__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("opentelemetry.proto.collector.metrics.v1.ExportMetricsServiceRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ExportMetricsServiceResponse {
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
        let mut struct_ser = serializer.serialize_struct("opentelemetry.proto.collector.metrics.v1.ExportMetricsServiceResponse", len)?;
        if let Some(v) = self.partial_success.as_ref() {
            struct_ser.serialize_field("partialSuccess", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ExportMetricsServiceResponse {
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
            type Value = ExportMetricsServiceResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct opentelemetry.proto.collector.metrics.v1.ExportMetricsServiceResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ExportMetricsServiceResponse, V::Error>
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
                Ok(ExportMetricsServiceResponse {
                    partial_success: partial_success__,
                })
            }
        }
        deserializer.deserialize_struct("opentelemetry.proto.collector.metrics.v1.ExportMetricsServiceResponse", FIELDS, GeneratedVisitor)
    }
}
