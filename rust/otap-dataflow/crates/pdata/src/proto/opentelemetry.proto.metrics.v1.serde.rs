impl serde::Serialize for AggregationTemporality {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "AGGREGATION_TEMPORALITY_UNSPECIFIED",
            Self::Delta => "AGGREGATION_TEMPORALITY_DELTA",
            Self::Cumulative => "AGGREGATION_TEMPORALITY_CUMULATIVE",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for AggregationTemporality {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "AGGREGATION_TEMPORALITY_UNSPECIFIED",
            "AGGREGATION_TEMPORALITY_DELTA",
            "AGGREGATION_TEMPORALITY_CUMULATIVE",
        ];

        struct GeneratedVisitor;

        impl serde::de::Visitor<'_> for GeneratedVisitor {
            type Value = AggregationTemporality;

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
                    "AGGREGATION_TEMPORALITY_UNSPECIFIED" => Ok(AggregationTemporality::Unspecified),
                    "AGGREGATION_TEMPORALITY_DELTA" => Ok(AggregationTemporality::Delta),
                    "AGGREGATION_TEMPORALITY_CUMULATIVE" => Ok(AggregationTemporality::Cumulative),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for DataPointFlags {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::DoNotUse => "DATA_POINT_FLAGS_DO_NOT_USE",
            Self::NoRecordedValueMask => "DATA_POINT_FLAGS_NO_RECORDED_VALUE_MASK",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for DataPointFlags {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "DATA_POINT_FLAGS_DO_NOT_USE",
            "DATA_POINT_FLAGS_NO_RECORDED_VALUE_MASK",
        ];

        struct GeneratedVisitor;

        impl serde::de::Visitor<'_> for GeneratedVisitor {
            type Value = DataPointFlags;

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
                    "DATA_POINT_FLAGS_DO_NOT_USE" => Ok(DataPointFlags::DoNotUse),
                    "DATA_POINT_FLAGS_NO_RECORDED_VALUE_MASK" => Ok(DataPointFlags::NoRecordedValueMask),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for Exemplar {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.filtered_attributes.is_empty() {
            len += 1;
        }
        if self.time_unix_nano != 0 {
            len += 1;
        }
        if !self.span_id.is_empty() {
            len += 1;
        }
        if !self.trace_id.is_empty() {
            len += 1;
        }
        if self.value.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("opentelemetry.proto.metrics.v1.Exemplar", len)?;
        if !self.filtered_attributes.is_empty() {
            struct_ser.serialize_field("filteredAttributes", &self.filtered_attributes)?;
        }
        if self.time_unix_nano != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("timeUnixNano", ToString::to_string(&self.time_unix_nano).as_str())?;
        }
        if !self.span_id.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("spanId", pbjson::private::base64::encode(&self.span_id).as_str())?;
        }
        if !self.trace_id.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("traceId", pbjson::private::base64::encode(&self.trace_id).as_str())?;
        }
        if let Some(v) = self.value.as_ref() {
            match v {
                exemplar::Value::AsDouble(v) => {
                    struct_ser.serialize_field("asDouble", v)?;
                }
                exemplar::Value::AsInt(v) => {
                    #[allow(clippy::needless_borrow)]
                    #[allow(clippy::needless_borrows_for_generic_args)]
                    struct_ser.serialize_field("asInt", ToString::to_string(&v).as_str())?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Exemplar {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "filtered_attributes",
            "filteredAttributes",
            "time_unix_nano",
            "timeUnixNano",
            "span_id",
            "spanId",
            "trace_id",
            "traceId",
            "as_double",
            "asDouble",
            "as_int",
            "asInt",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            FilteredAttributes,
            TimeUnixNano,
            SpanId,
            TraceId,
            AsDouble,
            AsInt,
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
                            "filteredAttributes" | "filtered_attributes" => Ok(GeneratedField::FilteredAttributes),
                            "timeUnixNano" | "time_unix_nano" => Ok(GeneratedField::TimeUnixNano),
                            "spanId" | "span_id" => Ok(GeneratedField::SpanId),
                            "traceId" | "trace_id" => Ok(GeneratedField::TraceId),
                            "asDouble" | "as_double" => Ok(GeneratedField::AsDouble),
                            "asInt" | "as_int" => Ok(GeneratedField::AsInt),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Exemplar;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct opentelemetry.proto.metrics.v1.Exemplar")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Exemplar, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut filtered_attributes__ = None;
                let mut time_unix_nano__ = None;
                let mut span_id__ = None;
                let mut trace_id__ = None;
                let mut value__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::FilteredAttributes => {
                            if filtered_attributes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("filteredAttributes"));
                            }
                            filtered_attributes__ = Some(map_.next_value()?);
                        }
                        GeneratedField::TimeUnixNano => {
                            if time_unix_nano__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timeUnixNano"));
                            }
                            time_unix_nano__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
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
                        GeneratedField::TraceId => {
                            if trace_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("traceId"));
                            }
                            trace_id__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::AsDouble => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("asDouble"));
                            }
                            value__ = map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| exemplar::Value::AsDouble(x.0));
                        }
                        GeneratedField::AsInt => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("asInt"));
                            }
                            value__ = map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| exemplar::Value::AsInt(x.0));
                        }
                    }
                }
                Ok(Exemplar {
                    filtered_attributes: filtered_attributes__.unwrap_or_default(),
                    time_unix_nano: time_unix_nano__.unwrap_or_default(),
                    span_id: span_id__.unwrap_or_default(),
                    trace_id: trace_id__.unwrap_or_default(),
                    value: value__,
                })
            }
        }
        deserializer.deserialize_struct("opentelemetry.proto.metrics.v1.Exemplar", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ExponentialHistogram {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.data_points.is_empty() {
            len += 1;
        }
        if self.aggregation_temporality != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("opentelemetry.proto.metrics.v1.ExponentialHistogram", len)?;
        if !self.data_points.is_empty() {
            struct_ser.serialize_field("dataPoints", &self.data_points)?;
        }
        if self.aggregation_temporality != 0 {
            let v = AggregationTemporality::try_from(self.aggregation_temporality)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.aggregation_temporality)))?;
            struct_ser.serialize_field("aggregationTemporality", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ExponentialHistogram {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "data_points",
            "dataPoints",
            "aggregation_temporality",
            "aggregationTemporality",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DataPoints,
            AggregationTemporality,
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
                            "dataPoints" | "data_points" => Ok(GeneratedField::DataPoints),
                            "aggregationTemporality" | "aggregation_temporality" => Ok(GeneratedField::AggregationTemporality),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ExponentialHistogram;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct opentelemetry.proto.metrics.v1.ExponentialHistogram")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ExponentialHistogram, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut data_points__ = None;
                let mut aggregation_temporality__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::DataPoints => {
                            if data_points__.is_some() {
                                return Err(serde::de::Error::duplicate_field("dataPoints"));
                            }
                            data_points__ = Some(map_.next_value()?);
                        }
                        GeneratedField::AggregationTemporality => {
                            if aggregation_temporality__.is_some() {
                                return Err(serde::de::Error::duplicate_field("aggregationTemporality"));
                            }
                            aggregation_temporality__ = Some(map_.next_value::<AggregationTemporality>()? as i32);
                        }
                    }
                }
                Ok(ExponentialHistogram {
                    data_points: data_points__.unwrap_or_default(),
                    aggregation_temporality: aggregation_temporality__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("opentelemetry.proto.metrics.v1.ExponentialHistogram", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ExponentialHistogramDataPoint {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.attributes.is_empty() {
            len += 1;
        }
        if self.start_time_unix_nano != 0 {
            len += 1;
        }
        if self.time_unix_nano != 0 {
            len += 1;
        }
        if self.count != 0 {
            len += 1;
        }
        if self.sum.is_some() {
            len += 1;
        }
        if self.scale != 0 {
            len += 1;
        }
        if self.zero_count != 0 {
            len += 1;
        }
        if self.positive.is_some() {
            len += 1;
        }
        if self.negative.is_some() {
            len += 1;
        }
        if self.flags != 0 {
            len += 1;
        }
        if !self.exemplars.is_empty() {
            len += 1;
        }
        if self.min.is_some() {
            len += 1;
        }
        if self.max.is_some() {
            len += 1;
        }
        if self.zero_threshold != 0. {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("opentelemetry.proto.metrics.v1.ExponentialHistogramDataPoint", len)?;
        if !self.attributes.is_empty() {
            struct_ser.serialize_field("attributes", &self.attributes)?;
        }
        if self.start_time_unix_nano != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("startTimeUnixNano", ToString::to_string(&self.start_time_unix_nano).as_str())?;
        }
        if self.time_unix_nano != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("timeUnixNano", ToString::to_string(&self.time_unix_nano).as_str())?;
        }
        if self.count != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("count", ToString::to_string(&self.count).as_str())?;
        }
        if let Some(v) = self.sum.as_ref() {
            struct_ser.serialize_field("sum", v)?;
        }
        if self.scale != 0 {
            struct_ser.serialize_field("scale", &self.scale)?;
        }
        if self.zero_count != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("zeroCount", ToString::to_string(&self.zero_count).as_str())?;
        }
        if let Some(v) = self.positive.as_ref() {
            struct_ser.serialize_field("positive", v)?;
        }
        if let Some(v) = self.negative.as_ref() {
            struct_ser.serialize_field("negative", v)?;
        }
        if self.flags != 0 {
            struct_ser.serialize_field("flags", &self.flags)?;
        }
        if !self.exemplars.is_empty() {
            struct_ser.serialize_field("exemplars", &self.exemplars)?;
        }
        if let Some(v) = self.min.as_ref() {
            struct_ser.serialize_field("min", v)?;
        }
        if let Some(v) = self.max.as_ref() {
            struct_ser.serialize_field("max", v)?;
        }
        if self.zero_threshold != 0. {
            struct_ser.serialize_field("zeroThreshold", &self.zero_threshold)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ExponentialHistogramDataPoint {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "attributes",
            "start_time_unix_nano",
            "startTimeUnixNano",
            "time_unix_nano",
            "timeUnixNano",
            "count",
            "sum",
            "scale",
            "zero_count",
            "zeroCount",
            "positive",
            "negative",
            "flags",
            "exemplars",
            "min",
            "max",
            "zero_threshold",
            "zeroThreshold",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Attributes,
            StartTimeUnixNano,
            TimeUnixNano,
            Count,
            Sum,
            Scale,
            ZeroCount,
            Positive,
            Negative,
            Flags,
            Exemplars,
            Min,
            Max,
            ZeroThreshold,
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
                            "attributes" => Ok(GeneratedField::Attributes),
                            "startTimeUnixNano" | "start_time_unix_nano" => Ok(GeneratedField::StartTimeUnixNano),
                            "timeUnixNano" | "time_unix_nano" => Ok(GeneratedField::TimeUnixNano),
                            "count" => Ok(GeneratedField::Count),
                            "sum" => Ok(GeneratedField::Sum),
                            "scale" => Ok(GeneratedField::Scale),
                            "zeroCount" | "zero_count" => Ok(GeneratedField::ZeroCount),
                            "positive" => Ok(GeneratedField::Positive),
                            "negative" => Ok(GeneratedField::Negative),
                            "flags" => Ok(GeneratedField::Flags),
                            "exemplars" => Ok(GeneratedField::Exemplars),
                            "min" => Ok(GeneratedField::Min),
                            "max" => Ok(GeneratedField::Max),
                            "zeroThreshold" | "zero_threshold" => Ok(GeneratedField::ZeroThreshold),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ExponentialHistogramDataPoint;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct opentelemetry.proto.metrics.v1.ExponentialHistogramDataPoint")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ExponentialHistogramDataPoint, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut attributes__ = None;
                let mut start_time_unix_nano__ = None;
                let mut time_unix_nano__ = None;
                let mut count__ = None;
                let mut sum__ = None;
                let mut scale__ = None;
                let mut zero_count__ = None;
                let mut positive__ = None;
                let mut negative__ = None;
                let mut flags__ = None;
                let mut exemplars__ = None;
                let mut min__ = None;
                let mut max__ = None;
                let mut zero_threshold__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Attributes => {
                            if attributes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("attributes"));
                            }
                            attributes__ = Some(map_.next_value()?);
                        }
                        GeneratedField::StartTimeUnixNano => {
                            if start_time_unix_nano__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startTimeUnixNano"));
                            }
                            start_time_unix_nano__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::TimeUnixNano => {
                            if time_unix_nano__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timeUnixNano"));
                            }
                            time_unix_nano__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Count => {
                            if count__.is_some() {
                                return Err(serde::de::Error::duplicate_field("count"));
                            }
                            count__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Sum => {
                            if sum__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sum"));
                            }
                            sum__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::Scale => {
                            if scale__.is_some() {
                                return Err(serde::de::Error::duplicate_field("scale"));
                            }
                            scale__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ZeroCount => {
                            if zero_count__.is_some() {
                                return Err(serde::de::Error::duplicate_field("zeroCount"));
                            }
                            zero_count__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Positive => {
                            if positive__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positive"));
                            }
                            positive__ = map_.next_value()?;
                        }
                        GeneratedField::Negative => {
                            if negative__.is_some() {
                                return Err(serde::de::Error::duplicate_field("negative"));
                            }
                            negative__ = map_.next_value()?;
                        }
                        GeneratedField::Flags => {
                            if flags__.is_some() {
                                return Err(serde::de::Error::duplicate_field("flags"));
                            }
                            flags__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Exemplars => {
                            if exemplars__.is_some() {
                                return Err(serde::de::Error::duplicate_field("exemplars"));
                            }
                            exemplars__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Min => {
                            if min__.is_some() {
                                return Err(serde::de::Error::duplicate_field("min"));
                            }
                            min__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::Max => {
                            if max__.is_some() {
                                return Err(serde::de::Error::duplicate_field("max"));
                            }
                            max__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::ZeroThreshold => {
                            if zero_threshold__.is_some() {
                                return Err(serde::de::Error::duplicate_field("zeroThreshold"));
                            }
                            zero_threshold__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(ExponentialHistogramDataPoint {
                    attributes: attributes__.unwrap_or_default(),
                    start_time_unix_nano: start_time_unix_nano__.unwrap_or_default(),
                    time_unix_nano: time_unix_nano__.unwrap_or_default(),
                    count: count__.unwrap_or_default(),
                    sum: sum__,
                    scale: scale__.unwrap_or_default(),
                    zero_count: zero_count__.unwrap_or_default(),
                    positive: positive__,
                    negative: negative__,
                    flags: flags__.unwrap_or_default(),
                    exemplars: exemplars__.unwrap_or_default(),
                    min: min__,
                    max: max__,
                    zero_threshold: zero_threshold__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("opentelemetry.proto.metrics.v1.ExponentialHistogramDataPoint", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for exponential_histogram_data_point::Buckets {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.offset != 0 {
            len += 1;
        }
        if !self.bucket_counts.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("opentelemetry.proto.metrics.v1.ExponentialHistogramDataPoint.Buckets", len)?;
        if self.offset != 0 {
            struct_ser.serialize_field("offset", &self.offset)?;
        }
        if !self.bucket_counts.is_empty() {
            struct_ser.serialize_field("bucketCounts", &self.bucket_counts.iter().map(ToString::to_string).collect::<Vec<_>>())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for exponential_histogram_data_point::Buckets {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "offset",
            "bucket_counts",
            "bucketCounts",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Offset,
            BucketCounts,
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
                            "offset" => Ok(GeneratedField::Offset),
                            "bucketCounts" | "bucket_counts" => Ok(GeneratedField::BucketCounts),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = exponential_histogram_data_point::Buckets;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct opentelemetry.proto.metrics.v1.ExponentialHistogramDataPoint.Buckets")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<exponential_histogram_data_point::Buckets, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut offset__ = None;
                let mut bucket_counts__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Offset => {
                            if offset__.is_some() {
                                return Err(serde::de::Error::duplicate_field("offset"));
                            }
                            offset__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::BucketCounts => {
                            if bucket_counts__.is_some() {
                                return Err(serde::de::Error::duplicate_field("bucketCounts"));
                            }
                            bucket_counts__ = 
                                Some(map_.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect())
                            ;
                        }
                    }
                }
                Ok(exponential_histogram_data_point::Buckets {
                    offset: offset__.unwrap_or_default(),
                    bucket_counts: bucket_counts__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("opentelemetry.proto.metrics.v1.ExponentialHistogramDataPoint.Buckets", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Gauge {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.data_points.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("opentelemetry.proto.metrics.v1.Gauge", len)?;
        if !self.data_points.is_empty() {
            struct_ser.serialize_field("dataPoints", &self.data_points)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Gauge {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "data_points",
            "dataPoints",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DataPoints,
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
                            "dataPoints" | "data_points" => Ok(GeneratedField::DataPoints),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Gauge;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct opentelemetry.proto.metrics.v1.Gauge")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Gauge, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut data_points__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::DataPoints => {
                            if data_points__.is_some() {
                                return Err(serde::de::Error::duplicate_field("dataPoints"));
                            }
                            data_points__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(Gauge {
                    data_points: data_points__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("opentelemetry.proto.metrics.v1.Gauge", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Histogram {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.data_points.is_empty() {
            len += 1;
        }
        if self.aggregation_temporality != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("opentelemetry.proto.metrics.v1.Histogram", len)?;
        if !self.data_points.is_empty() {
            struct_ser.serialize_field("dataPoints", &self.data_points)?;
        }
        if self.aggregation_temporality != 0 {
            let v = AggregationTemporality::try_from(self.aggregation_temporality)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.aggregation_temporality)))?;
            struct_ser.serialize_field("aggregationTemporality", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Histogram {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "data_points",
            "dataPoints",
            "aggregation_temporality",
            "aggregationTemporality",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DataPoints,
            AggregationTemporality,
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
                            "dataPoints" | "data_points" => Ok(GeneratedField::DataPoints),
                            "aggregationTemporality" | "aggregation_temporality" => Ok(GeneratedField::AggregationTemporality),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Histogram;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct opentelemetry.proto.metrics.v1.Histogram")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Histogram, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut data_points__ = None;
                let mut aggregation_temporality__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::DataPoints => {
                            if data_points__.is_some() {
                                return Err(serde::de::Error::duplicate_field("dataPoints"));
                            }
                            data_points__ = Some(map_.next_value()?);
                        }
                        GeneratedField::AggregationTemporality => {
                            if aggregation_temporality__.is_some() {
                                return Err(serde::de::Error::duplicate_field("aggregationTemporality"));
                            }
                            aggregation_temporality__ = Some(map_.next_value::<AggregationTemporality>()? as i32);
                        }
                    }
                }
                Ok(Histogram {
                    data_points: data_points__.unwrap_or_default(),
                    aggregation_temporality: aggregation_temporality__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("opentelemetry.proto.metrics.v1.Histogram", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for HistogramDataPoint {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.attributes.is_empty() {
            len += 1;
        }
        if self.start_time_unix_nano != 0 {
            len += 1;
        }
        if self.time_unix_nano != 0 {
            len += 1;
        }
        if self.count != 0 {
            len += 1;
        }
        if self.sum.is_some() {
            len += 1;
        }
        if !self.bucket_counts.is_empty() {
            len += 1;
        }
        if !self.explicit_bounds.is_empty() {
            len += 1;
        }
        if !self.exemplars.is_empty() {
            len += 1;
        }
        if self.flags != 0 {
            len += 1;
        }
        if self.min.is_some() {
            len += 1;
        }
        if self.max.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("opentelemetry.proto.metrics.v1.HistogramDataPoint", len)?;
        if !self.attributes.is_empty() {
            struct_ser.serialize_field("attributes", &self.attributes)?;
        }
        if self.start_time_unix_nano != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("startTimeUnixNano", ToString::to_string(&self.start_time_unix_nano).as_str())?;
        }
        if self.time_unix_nano != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("timeUnixNano", ToString::to_string(&self.time_unix_nano).as_str())?;
        }
        if self.count != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("count", ToString::to_string(&self.count).as_str())?;
        }
        if let Some(v) = self.sum.as_ref() {
            struct_ser.serialize_field("sum", v)?;
        }
        if !self.bucket_counts.is_empty() {
            struct_ser.serialize_field("bucketCounts", &self.bucket_counts.iter().map(ToString::to_string).collect::<Vec<_>>())?;
        }
        if !self.explicit_bounds.is_empty() {
            struct_ser.serialize_field("explicitBounds", &self.explicit_bounds)?;
        }
        if !self.exemplars.is_empty() {
            struct_ser.serialize_field("exemplars", &self.exemplars)?;
        }
        if self.flags != 0 {
            struct_ser.serialize_field("flags", &self.flags)?;
        }
        if let Some(v) = self.min.as_ref() {
            struct_ser.serialize_field("min", v)?;
        }
        if let Some(v) = self.max.as_ref() {
            struct_ser.serialize_field("max", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for HistogramDataPoint {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "attributes",
            "start_time_unix_nano",
            "startTimeUnixNano",
            "time_unix_nano",
            "timeUnixNano",
            "count",
            "sum",
            "bucket_counts",
            "bucketCounts",
            "explicit_bounds",
            "explicitBounds",
            "exemplars",
            "flags",
            "min",
            "max",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Attributes,
            StartTimeUnixNano,
            TimeUnixNano,
            Count,
            Sum,
            BucketCounts,
            ExplicitBounds,
            Exemplars,
            Flags,
            Min,
            Max,
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
                            "attributes" => Ok(GeneratedField::Attributes),
                            "startTimeUnixNano" | "start_time_unix_nano" => Ok(GeneratedField::StartTimeUnixNano),
                            "timeUnixNano" | "time_unix_nano" => Ok(GeneratedField::TimeUnixNano),
                            "count" => Ok(GeneratedField::Count),
                            "sum" => Ok(GeneratedField::Sum),
                            "bucketCounts" | "bucket_counts" => Ok(GeneratedField::BucketCounts),
                            "explicitBounds" | "explicit_bounds" => Ok(GeneratedField::ExplicitBounds),
                            "exemplars" => Ok(GeneratedField::Exemplars),
                            "flags" => Ok(GeneratedField::Flags),
                            "min" => Ok(GeneratedField::Min),
                            "max" => Ok(GeneratedField::Max),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = HistogramDataPoint;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct opentelemetry.proto.metrics.v1.HistogramDataPoint")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<HistogramDataPoint, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut attributes__ = None;
                let mut start_time_unix_nano__ = None;
                let mut time_unix_nano__ = None;
                let mut count__ = None;
                let mut sum__ = None;
                let mut bucket_counts__ = None;
                let mut explicit_bounds__ = None;
                let mut exemplars__ = None;
                let mut flags__ = None;
                let mut min__ = None;
                let mut max__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Attributes => {
                            if attributes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("attributes"));
                            }
                            attributes__ = Some(map_.next_value()?);
                        }
                        GeneratedField::StartTimeUnixNano => {
                            if start_time_unix_nano__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startTimeUnixNano"));
                            }
                            start_time_unix_nano__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::TimeUnixNano => {
                            if time_unix_nano__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timeUnixNano"));
                            }
                            time_unix_nano__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Count => {
                            if count__.is_some() {
                                return Err(serde::de::Error::duplicate_field("count"));
                            }
                            count__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Sum => {
                            if sum__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sum"));
                            }
                            sum__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::BucketCounts => {
                            if bucket_counts__.is_some() {
                                return Err(serde::de::Error::duplicate_field("bucketCounts"));
                            }
                            bucket_counts__ = 
                                Some(map_.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect())
                            ;
                        }
                        GeneratedField::ExplicitBounds => {
                            if explicit_bounds__.is_some() {
                                return Err(serde::de::Error::duplicate_field("explicitBounds"));
                            }
                            explicit_bounds__ = 
                                Some(map_.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect())
                            ;
                        }
                        GeneratedField::Exemplars => {
                            if exemplars__.is_some() {
                                return Err(serde::de::Error::duplicate_field("exemplars"));
                            }
                            exemplars__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Flags => {
                            if flags__.is_some() {
                                return Err(serde::de::Error::duplicate_field("flags"));
                            }
                            flags__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Min => {
                            if min__.is_some() {
                                return Err(serde::de::Error::duplicate_field("min"));
                            }
                            min__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::Max => {
                            if max__.is_some() {
                                return Err(serde::de::Error::duplicate_field("max"));
                            }
                            max__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                    }
                }
                Ok(HistogramDataPoint {
                    attributes: attributes__.unwrap_or_default(),
                    start_time_unix_nano: start_time_unix_nano__.unwrap_or_default(),
                    time_unix_nano: time_unix_nano__.unwrap_or_default(),
                    count: count__.unwrap_or_default(),
                    sum: sum__,
                    bucket_counts: bucket_counts__.unwrap_or_default(),
                    explicit_bounds: explicit_bounds__.unwrap_or_default(),
                    exemplars: exemplars__.unwrap_or_default(),
                    flags: flags__.unwrap_or_default(),
                    min: min__,
                    max: max__,
                })
            }
        }
        deserializer.deserialize_struct("opentelemetry.proto.metrics.v1.HistogramDataPoint", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Metric {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.name.is_empty() {
            len += 1;
        }
        if !self.description.is_empty() {
            len += 1;
        }
        if !self.unit.is_empty() {
            len += 1;
        }
        if !self.metadata.is_empty() {
            len += 1;
        }
        if self.data.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("opentelemetry.proto.metrics.v1.Metric", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if !self.description.is_empty() {
            struct_ser.serialize_field("description", &self.description)?;
        }
        if !self.unit.is_empty() {
            struct_ser.serialize_field("unit", &self.unit)?;
        }
        if !self.metadata.is_empty() {
            struct_ser.serialize_field("metadata", &self.metadata)?;
        }
        if let Some(v) = self.data.as_ref() {
            match v {
                metric::Data::Gauge(v) => {
                    struct_ser.serialize_field("gauge", v)?;
                }
                metric::Data::Sum(v) => {
                    struct_ser.serialize_field("sum", v)?;
                }
                metric::Data::Histogram(v) => {
                    struct_ser.serialize_field("histogram", v)?;
                }
                metric::Data::ExponentialHistogram(v) => {
                    struct_ser.serialize_field("exponentialHistogram", v)?;
                }
                metric::Data::Summary(v) => {
                    struct_ser.serialize_field("summary", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Metric {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "description",
            "unit",
            "metadata",
            "gauge",
            "sum",
            "histogram",
            "exponential_histogram",
            "exponentialHistogram",
            "summary",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            Description,
            Unit,
            Metadata,
            Gauge,
            Sum,
            Histogram,
            ExponentialHistogram,
            Summary,
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
                            "name" => Ok(GeneratedField::Name),
                            "description" => Ok(GeneratedField::Description),
                            "unit" => Ok(GeneratedField::Unit),
                            "metadata" => Ok(GeneratedField::Metadata),
                            "gauge" => Ok(GeneratedField::Gauge),
                            "sum" => Ok(GeneratedField::Sum),
                            "histogram" => Ok(GeneratedField::Histogram),
                            "exponentialHistogram" | "exponential_histogram" => Ok(GeneratedField::ExponentialHistogram),
                            "summary" => Ok(GeneratedField::Summary),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Metric;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct opentelemetry.proto.metrics.v1.Metric")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Metric, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut description__ = None;
                let mut unit__ = None;
                let mut metadata__ = None;
                let mut data__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Description => {
                            if description__.is_some() {
                                return Err(serde::de::Error::duplicate_field("description"));
                            }
                            description__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Unit => {
                            if unit__.is_some() {
                                return Err(serde::de::Error::duplicate_field("unit"));
                            }
                            unit__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Metadata => {
                            if metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("metadata"));
                            }
                            metadata__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Gauge => {
                            if data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("gauge"));
                            }
                            data__ = map_.next_value::<::std::option::Option<_>>()?.map(metric::Data::Gauge)
;
                        }
                        GeneratedField::Sum => {
                            if data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sum"));
                            }
                            data__ = map_.next_value::<::std::option::Option<_>>()?.map(metric::Data::Sum)
;
                        }
                        GeneratedField::Histogram => {
                            if data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("histogram"));
                            }
                            data__ = map_.next_value::<::std::option::Option<_>>()?.map(metric::Data::Histogram)
;
                        }
                        GeneratedField::ExponentialHistogram => {
                            if data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("exponentialHistogram"));
                            }
                            data__ = map_.next_value::<::std::option::Option<_>>()?.map(metric::Data::ExponentialHistogram)
;
                        }
                        GeneratedField::Summary => {
                            if data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("summary"));
                            }
                            data__ = map_.next_value::<::std::option::Option<_>>()?.map(metric::Data::Summary)
;
                        }
                    }
                }
                Ok(Metric {
                    name: name__.unwrap_or_default(),
                    description: description__.unwrap_or_default(),
                    unit: unit__.unwrap_or_default(),
                    metadata: metadata__.unwrap_or_default(),
                    data: data__,
                })
            }
        }
        deserializer.deserialize_struct("opentelemetry.proto.metrics.v1.Metric", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MetricsData {
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
        let mut struct_ser = serializer.serialize_struct("opentelemetry.proto.metrics.v1.MetricsData", len)?;
        if !self.resource_metrics.is_empty() {
            struct_ser.serialize_field("resourceMetrics", &self.resource_metrics)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MetricsData {
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
            type Value = MetricsData;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct opentelemetry.proto.metrics.v1.MetricsData")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<MetricsData, V::Error>
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
                Ok(MetricsData {
                    resource_metrics: resource_metrics__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("opentelemetry.proto.metrics.v1.MetricsData", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for NumberDataPoint {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.attributes.is_empty() {
            len += 1;
        }
        if self.start_time_unix_nano != 0 {
            len += 1;
        }
        if self.time_unix_nano != 0 {
            len += 1;
        }
        if !self.exemplars.is_empty() {
            len += 1;
        }
        if self.flags != 0 {
            len += 1;
        }
        if self.value.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("opentelemetry.proto.metrics.v1.NumberDataPoint", len)?;
        if !self.attributes.is_empty() {
            struct_ser.serialize_field("attributes", &self.attributes)?;
        }
        if self.start_time_unix_nano != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("startTimeUnixNano", ToString::to_string(&self.start_time_unix_nano).as_str())?;
        }
        if self.time_unix_nano != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("timeUnixNano", ToString::to_string(&self.time_unix_nano).as_str())?;
        }
        if !self.exemplars.is_empty() {
            struct_ser.serialize_field("exemplars", &self.exemplars)?;
        }
        if self.flags != 0 {
            struct_ser.serialize_field("flags", &self.flags)?;
        }
        if let Some(v) = self.value.as_ref() {
            match v {
                number_data_point::Value::AsDouble(v) => {
                    struct_ser.serialize_field("asDouble", v)?;
                }
                number_data_point::Value::AsInt(v) => {
                    #[allow(clippy::needless_borrow)]
                    #[allow(clippy::needless_borrows_for_generic_args)]
                    struct_ser.serialize_field("asInt", ToString::to_string(&v).as_str())?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for NumberDataPoint {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "attributes",
            "start_time_unix_nano",
            "startTimeUnixNano",
            "time_unix_nano",
            "timeUnixNano",
            "exemplars",
            "flags",
            "as_double",
            "asDouble",
            "as_int",
            "asInt",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Attributes,
            StartTimeUnixNano,
            TimeUnixNano,
            Exemplars,
            Flags,
            AsDouble,
            AsInt,
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
                            "attributes" => Ok(GeneratedField::Attributes),
                            "startTimeUnixNano" | "start_time_unix_nano" => Ok(GeneratedField::StartTimeUnixNano),
                            "timeUnixNano" | "time_unix_nano" => Ok(GeneratedField::TimeUnixNano),
                            "exemplars" => Ok(GeneratedField::Exemplars),
                            "flags" => Ok(GeneratedField::Flags),
                            "asDouble" | "as_double" => Ok(GeneratedField::AsDouble),
                            "asInt" | "as_int" => Ok(GeneratedField::AsInt),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = NumberDataPoint;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct opentelemetry.proto.metrics.v1.NumberDataPoint")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<NumberDataPoint, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut attributes__ = None;
                let mut start_time_unix_nano__ = None;
                let mut time_unix_nano__ = None;
                let mut exemplars__ = None;
                let mut flags__ = None;
                let mut value__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Attributes => {
                            if attributes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("attributes"));
                            }
                            attributes__ = Some(map_.next_value()?);
                        }
                        GeneratedField::StartTimeUnixNano => {
                            if start_time_unix_nano__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startTimeUnixNano"));
                            }
                            start_time_unix_nano__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::TimeUnixNano => {
                            if time_unix_nano__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timeUnixNano"));
                            }
                            time_unix_nano__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Exemplars => {
                            if exemplars__.is_some() {
                                return Err(serde::de::Error::duplicate_field("exemplars"));
                            }
                            exemplars__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Flags => {
                            if flags__.is_some() {
                                return Err(serde::de::Error::duplicate_field("flags"));
                            }
                            flags__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::AsDouble => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("asDouble"));
                            }
                            value__ = map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| number_data_point::Value::AsDouble(x.0));
                        }
                        GeneratedField::AsInt => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("asInt"));
                            }
                            value__ = map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| number_data_point::Value::AsInt(x.0));
                        }
                    }
                }
                Ok(NumberDataPoint {
                    attributes: attributes__.unwrap_or_default(),
                    start_time_unix_nano: start_time_unix_nano__.unwrap_or_default(),
                    time_unix_nano: time_unix_nano__.unwrap_or_default(),
                    exemplars: exemplars__.unwrap_or_default(),
                    flags: flags__.unwrap_or_default(),
                    value: value__,
                })
            }
        }
        deserializer.deserialize_struct("opentelemetry.proto.metrics.v1.NumberDataPoint", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ResourceMetrics {
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
        if !self.scope_metrics.is_empty() {
            len += 1;
        }
        if !self.schema_url.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("opentelemetry.proto.metrics.v1.ResourceMetrics", len)?;
        if let Some(v) = self.resource.as_ref() {
            struct_ser.serialize_field("resource", v)?;
        }
        if !self.scope_metrics.is_empty() {
            struct_ser.serialize_field("scopeMetrics", &self.scope_metrics)?;
        }
        if !self.schema_url.is_empty() {
            struct_ser.serialize_field("schemaUrl", &self.schema_url)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ResourceMetrics {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "resource",
            "scope_metrics",
            "scopeMetrics",
            "schema_url",
            "schemaUrl",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Resource,
            ScopeMetrics,
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
                            "scopeMetrics" | "scope_metrics" => Ok(GeneratedField::ScopeMetrics),
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
            type Value = ResourceMetrics;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct opentelemetry.proto.metrics.v1.ResourceMetrics")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ResourceMetrics, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut resource__ = None;
                let mut scope_metrics__ = None;
                let mut schema_url__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Resource => {
                            if resource__.is_some() {
                                return Err(serde::de::Error::duplicate_field("resource"));
                            }
                            resource__ = map_.next_value()?;
                        }
                        GeneratedField::ScopeMetrics => {
                            if scope_metrics__.is_some() {
                                return Err(serde::de::Error::duplicate_field("scopeMetrics"));
                            }
                            scope_metrics__ = Some(map_.next_value()?);
                        }
                        GeneratedField::SchemaUrl => {
                            if schema_url__.is_some() {
                                return Err(serde::de::Error::duplicate_field("schemaUrl"));
                            }
                            schema_url__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ResourceMetrics {
                    resource: resource__,
                    scope_metrics: scope_metrics__.unwrap_or_default(),
                    schema_url: schema_url__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("opentelemetry.proto.metrics.v1.ResourceMetrics", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ScopeMetrics {
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
        if !self.metrics.is_empty() {
            len += 1;
        }
        if !self.schema_url.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("opentelemetry.proto.metrics.v1.ScopeMetrics", len)?;
        if let Some(v) = self.scope.as_ref() {
            struct_ser.serialize_field("scope", v)?;
        }
        if !self.metrics.is_empty() {
            struct_ser.serialize_field("metrics", &self.metrics)?;
        }
        if !self.schema_url.is_empty() {
            struct_ser.serialize_field("schemaUrl", &self.schema_url)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ScopeMetrics {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "scope",
            "metrics",
            "schema_url",
            "schemaUrl",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Scope,
            Metrics,
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
                            "metrics" => Ok(GeneratedField::Metrics),
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
            type Value = ScopeMetrics;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct opentelemetry.proto.metrics.v1.ScopeMetrics")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ScopeMetrics, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut scope__ = None;
                let mut metrics__ = None;
                let mut schema_url__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Scope => {
                            if scope__.is_some() {
                                return Err(serde::de::Error::duplicate_field("scope"));
                            }
                            scope__ = map_.next_value()?;
                        }
                        GeneratedField::Metrics => {
                            if metrics__.is_some() {
                                return Err(serde::de::Error::duplicate_field("metrics"));
                            }
                            metrics__ = Some(map_.next_value()?);
                        }
                        GeneratedField::SchemaUrl => {
                            if schema_url__.is_some() {
                                return Err(serde::de::Error::duplicate_field("schemaUrl"));
                            }
                            schema_url__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ScopeMetrics {
                    scope: scope__,
                    metrics: metrics__.unwrap_or_default(),
                    schema_url: schema_url__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("opentelemetry.proto.metrics.v1.ScopeMetrics", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Sum {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.data_points.is_empty() {
            len += 1;
        }
        if self.aggregation_temporality != 0 {
            len += 1;
        }
        if self.is_monotonic {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("opentelemetry.proto.metrics.v1.Sum", len)?;
        if !self.data_points.is_empty() {
            struct_ser.serialize_field("dataPoints", &self.data_points)?;
        }
        if self.aggregation_temporality != 0 {
            let v = AggregationTemporality::try_from(self.aggregation_temporality)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.aggregation_temporality)))?;
            struct_ser.serialize_field("aggregationTemporality", &v)?;
        }
        if self.is_monotonic {
            struct_ser.serialize_field("isMonotonic", &self.is_monotonic)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Sum {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "data_points",
            "dataPoints",
            "aggregation_temporality",
            "aggregationTemporality",
            "is_monotonic",
            "isMonotonic",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DataPoints,
            AggregationTemporality,
            IsMonotonic,
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
                            "dataPoints" | "data_points" => Ok(GeneratedField::DataPoints),
                            "aggregationTemporality" | "aggregation_temporality" => Ok(GeneratedField::AggregationTemporality),
                            "isMonotonic" | "is_monotonic" => Ok(GeneratedField::IsMonotonic),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Sum;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct opentelemetry.proto.metrics.v1.Sum")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Sum, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut data_points__ = None;
                let mut aggregation_temporality__ = None;
                let mut is_monotonic__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::DataPoints => {
                            if data_points__.is_some() {
                                return Err(serde::de::Error::duplicate_field("dataPoints"));
                            }
                            data_points__ = Some(map_.next_value()?);
                        }
                        GeneratedField::AggregationTemporality => {
                            if aggregation_temporality__.is_some() {
                                return Err(serde::de::Error::duplicate_field("aggregationTemporality"));
                            }
                            aggregation_temporality__ = Some(map_.next_value::<AggregationTemporality>()? as i32);
                        }
                        GeneratedField::IsMonotonic => {
                            if is_monotonic__.is_some() {
                                return Err(serde::de::Error::duplicate_field("isMonotonic"));
                            }
                            is_monotonic__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(Sum {
                    data_points: data_points__.unwrap_or_default(),
                    aggregation_temporality: aggregation_temporality__.unwrap_or_default(),
                    is_monotonic: is_monotonic__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("opentelemetry.proto.metrics.v1.Sum", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Summary {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.data_points.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("opentelemetry.proto.metrics.v1.Summary", len)?;
        if !self.data_points.is_empty() {
            struct_ser.serialize_field("dataPoints", &self.data_points)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Summary {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "data_points",
            "dataPoints",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DataPoints,
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
                            "dataPoints" | "data_points" => Ok(GeneratedField::DataPoints),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Summary;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct opentelemetry.proto.metrics.v1.Summary")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Summary, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut data_points__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::DataPoints => {
                            if data_points__.is_some() {
                                return Err(serde::de::Error::duplicate_field("dataPoints"));
                            }
                            data_points__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(Summary {
                    data_points: data_points__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("opentelemetry.proto.metrics.v1.Summary", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SummaryDataPoint {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.attributes.is_empty() {
            len += 1;
        }
        if self.start_time_unix_nano != 0 {
            len += 1;
        }
        if self.time_unix_nano != 0 {
            len += 1;
        }
        if self.count != 0 {
            len += 1;
        }
        if self.sum != 0. {
            len += 1;
        }
        if !self.quantile_values.is_empty() {
            len += 1;
        }
        if self.flags != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("opentelemetry.proto.metrics.v1.SummaryDataPoint", len)?;
        if !self.attributes.is_empty() {
            struct_ser.serialize_field("attributes", &self.attributes)?;
        }
        if self.start_time_unix_nano != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("startTimeUnixNano", ToString::to_string(&self.start_time_unix_nano).as_str())?;
        }
        if self.time_unix_nano != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("timeUnixNano", ToString::to_string(&self.time_unix_nano).as_str())?;
        }
        if self.count != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("count", ToString::to_string(&self.count).as_str())?;
        }
        if self.sum != 0. {
            struct_ser.serialize_field("sum", &self.sum)?;
        }
        if !self.quantile_values.is_empty() {
            struct_ser.serialize_field("quantileValues", &self.quantile_values)?;
        }
        if self.flags != 0 {
            struct_ser.serialize_field("flags", &self.flags)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SummaryDataPoint {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "attributes",
            "start_time_unix_nano",
            "startTimeUnixNano",
            "time_unix_nano",
            "timeUnixNano",
            "count",
            "sum",
            "quantile_values",
            "quantileValues",
            "flags",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Attributes,
            StartTimeUnixNano,
            TimeUnixNano,
            Count,
            Sum,
            QuantileValues,
            Flags,
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
                            "attributes" => Ok(GeneratedField::Attributes),
                            "startTimeUnixNano" | "start_time_unix_nano" => Ok(GeneratedField::StartTimeUnixNano),
                            "timeUnixNano" | "time_unix_nano" => Ok(GeneratedField::TimeUnixNano),
                            "count" => Ok(GeneratedField::Count),
                            "sum" => Ok(GeneratedField::Sum),
                            "quantileValues" | "quantile_values" => Ok(GeneratedField::QuantileValues),
                            "flags" => Ok(GeneratedField::Flags),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SummaryDataPoint;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct opentelemetry.proto.metrics.v1.SummaryDataPoint")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SummaryDataPoint, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut attributes__ = None;
                let mut start_time_unix_nano__ = None;
                let mut time_unix_nano__ = None;
                let mut count__ = None;
                let mut sum__ = None;
                let mut quantile_values__ = None;
                let mut flags__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Attributes => {
                            if attributes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("attributes"));
                            }
                            attributes__ = Some(map_.next_value()?);
                        }
                        GeneratedField::StartTimeUnixNano => {
                            if start_time_unix_nano__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startTimeUnixNano"));
                            }
                            start_time_unix_nano__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::TimeUnixNano => {
                            if time_unix_nano__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timeUnixNano"));
                            }
                            time_unix_nano__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Count => {
                            if count__.is_some() {
                                return Err(serde::de::Error::duplicate_field("count"));
                            }
                            count__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Sum => {
                            if sum__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sum"));
                            }
                            sum__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::QuantileValues => {
                            if quantile_values__.is_some() {
                                return Err(serde::de::Error::duplicate_field("quantileValues"));
                            }
                            quantile_values__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Flags => {
                            if flags__.is_some() {
                                return Err(serde::de::Error::duplicate_field("flags"));
                            }
                            flags__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(SummaryDataPoint {
                    attributes: attributes__.unwrap_or_default(),
                    start_time_unix_nano: start_time_unix_nano__.unwrap_or_default(),
                    time_unix_nano: time_unix_nano__.unwrap_or_default(),
                    count: count__.unwrap_or_default(),
                    sum: sum__.unwrap_or_default(),
                    quantile_values: quantile_values__.unwrap_or_default(),
                    flags: flags__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("opentelemetry.proto.metrics.v1.SummaryDataPoint", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for summary_data_point::ValueAtQuantile {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.quantile != 0. {
            len += 1;
        }
        if self.value != 0. {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("opentelemetry.proto.metrics.v1.SummaryDataPoint.ValueAtQuantile", len)?;
        if self.quantile != 0. {
            struct_ser.serialize_field("quantile", &self.quantile)?;
        }
        if self.value != 0. {
            struct_ser.serialize_field("value", &self.value)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for summary_data_point::ValueAtQuantile {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "quantile",
            "value",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Quantile,
            Value,
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
                            "quantile" => Ok(GeneratedField::Quantile),
                            "value" => Ok(GeneratedField::Value),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = summary_data_point::ValueAtQuantile;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct opentelemetry.proto.metrics.v1.SummaryDataPoint.ValueAtQuantile")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<summary_data_point::ValueAtQuantile, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut quantile__ = None;
                let mut value__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Quantile => {
                            if quantile__.is_some() {
                                return Err(serde::de::Error::duplicate_field("quantile"));
                            }
                            quantile__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(summary_data_point::ValueAtQuantile {
                    quantile: quantile__.unwrap_or_default(),
                    value: value__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("opentelemetry.proto.metrics.v1.SummaryDataPoint.ValueAtQuantile", FIELDS, GeneratedVisitor)
    }
}
