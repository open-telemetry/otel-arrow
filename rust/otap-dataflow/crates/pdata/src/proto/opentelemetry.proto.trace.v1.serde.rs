impl serde::Serialize for ResourceSpans {
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
        if !self.scope_spans.is_empty() {
            len += 1;
        }
        if !self.schema_url.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("opentelemetry.proto.trace.v1.ResourceSpans", len)?;
        if let Some(v) = self.resource.as_ref() {
            struct_ser.serialize_field("resource", v)?;
        }
        if !self.scope_spans.is_empty() {
            struct_ser.serialize_field("scopeSpans", &self.scope_spans)?;
        }
        if !self.schema_url.is_empty() {
            struct_ser.serialize_field("schemaUrl", &self.schema_url)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ResourceSpans {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "resource",
            "scope_spans",
            "scopeSpans",
            "schema_url",
            "schemaUrl",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Resource,
            ScopeSpans,
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
                            "scopeSpans" | "scope_spans" => Ok(GeneratedField::ScopeSpans),
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
            type Value = ResourceSpans;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct opentelemetry.proto.trace.v1.ResourceSpans")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ResourceSpans, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut resource__ = None;
                let mut scope_spans__ = None;
                let mut schema_url__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Resource => {
                            if resource__.is_some() {
                                return Err(serde::de::Error::duplicate_field("resource"));
                            }
                            resource__ = map_.next_value()?;
                        }
                        GeneratedField::ScopeSpans => {
                            if scope_spans__.is_some() {
                                return Err(serde::de::Error::duplicate_field("scopeSpans"));
                            }
                            scope_spans__ = Some(map_.next_value()?);
                        }
                        GeneratedField::SchemaUrl => {
                            if schema_url__.is_some() {
                                return Err(serde::de::Error::duplicate_field("schemaUrl"));
                            }
                            schema_url__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ResourceSpans {
                    resource: resource__,
                    scope_spans: scope_spans__.unwrap_or_default(),
                    schema_url: schema_url__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("opentelemetry.proto.trace.v1.ResourceSpans", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ScopeSpans {
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
        if !self.spans.is_empty() {
            len += 1;
        }
        if !self.schema_url.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("opentelemetry.proto.trace.v1.ScopeSpans", len)?;
        if let Some(v) = self.scope.as_ref() {
            struct_ser.serialize_field("scope", v)?;
        }
        if !self.spans.is_empty() {
            struct_ser.serialize_field("spans", &self.spans)?;
        }
        if !self.schema_url.is_empty() {
            struct_ser.serialize_field("schemaUrl", &self.schema_url)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ScopeSpans {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "scope",
            "spans",
            "schema_url",
            "schemaUrl",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Scope,
            Spans,
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
                            "spans" => Ok(GeneratedField::Spans),
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
            type Value = ScopeSpans;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct opentelemetry.proto.trace.v1.ScopeSpans")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ScopeSpans, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut scope__ = None;
                let mut spans__ = None;
                let mut schema_url__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Scope => {
                            if scope__.is_some() {
                                return Err(serde::de::Error::duplicate_field("scope"));
                            }
                            scope__ = map_.next_value()?;
                        }
                        GeneratedField::Spans => {
                            if spans__.is_some() {
                                return Err(serde::de::Error::duplicate_field("spans"));
                            }
                            spans__ = Some(map_.next_value()?);
                        }
                        GeneratedField::SchemaUrl => {
                            if schema_url__.is_some() {
                                return Err(serde::de::Error::duplicate_field("schemaUrl"));
                            }
                            schema_url__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ScopeSpans {
                    scope: scope__,
                    spans: spans__.unwrap_or_default(),
                    schema_url: schema_url__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("opentelemetry.proto.trace.v1.ScopeSpans", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Span {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.trace_id.is_empty() {
            len += 1;
        }
        if !self.span_id.is_empty() {
            len += 1;
        }
        if !self.trace_state.is_empty() {
            len += 1;
        }
        if !self.parent_span_id.is_empty() {
            len += 1;
        }
        if self.flags != 0 {
            len += 1;
        }
        if !self.name.is_empty() {
            len += 1;
        }
        if self.kind != 0 {
            len += 1;
        }
        if self.start_time_unix_nano != 0 {
            len += 1;
        }
        if self.end_time_unix_nano != 0 {
            len += 1;
        }
        if !self.attributes.is_empty() {
            len += 1;
        }
        if self.dropped_attributes_count != 0 {
            len += 1;
        }
        if !self.events.is_empty() {
            len += 1;
        }
        if self.dropped_events_count != 0 {
            len += 1;
        }
        if !self.links.is_empty() {
            len += 1;
        }
        if self.dropped_links_count != 0 {
            len += 1;
        }
        if self.status.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("opentelemetry.proto.trace.v1.Span", len)?;
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
        if !self.trace_state.is_empty() {
            struct_ser.serialize_field("traceState", &self.trace_state)?;
        }
        if !self.parent_span_id.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("parentSpanId", pbjson::private::base64::encode(&self.parent_span_id).as_str())?;
        }
        if self.flags != 0 {
            struct_ser.serialize_field("flags", &self.flags)?;
        }
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if self.kind != 0 {
            let v = span::SpanKind::try_from(self.kind)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.kind)))?;
            struct_ser.serialize_field("kind", &v)?;
        }
        if self.start_time_unix_nano != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("startTimeUnixNano", ToString::to_string(&self.start_time_unix_nano).as_str())?;
        }
        if self.end_time_unix_nano != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("endTimeUnixNano", ToString::to_string(&self.end_time_unix_nano).as_str())?;
        }
        if !self.attributes.is_empty() {
            struct_ser.serialize_field("attributes", &self.attributes)?;
        }
        if self.dropped_attributes_count != 0 {
            struct_ser.serialize_field("droppedAttributesCount", &self.dropped_attributes_count)?;
        }
        if !self.events.is_empty() {
            struct_ser.serialize_field("events", &self.events)?;
        }
        if self.dropped_events_count != 0 {
            struct_ser.serialize_field("droppedEventsCount", &self.dropped_events_count)?;
        }
        if !self.links.is_empty() {
            struct_ser.serialize_field("links", &self.links)?;
        }
        if self.dropped_links_count != 0 {
            struct_ser.serialize_field("droppedLinksCount", &self.dropped_links_count)?;
        }
        if let Some(v) = self.status.as_ref() {
            struct_ser.serialize_field("status", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Span {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "trace_id",
            "traceId",
            "span_id",
            "spanId",
            "trace_state",
            "traceState",
            "parent_span_id",
            "parentSpanId",
            "flags",
            "name",
            "kind",
            "start_time_unix_nano",
            "startTimeUnixNano",
            "end_time_unix_nano",
            "endTimeUnixNano",
            "attributes",
            "dropped_attributes_count",
            "droppedAttributesCount",
            "events",
            "dropped_events_count",
            "droppedEventsCount",
            "links",
            "dropped_links_count",
            "droppedLinksCount",
            "status",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TraceId,
            SpanId,
            TraceState,
            ParentSpanId,
            Flags,
            Name,
            Kind,
            StartTimeUnixNano,
            EndTimeUnixNano,
            Attributes,
            DroppedAttributesCount,
            Events,
            DroppedEventsCount,
            Links,
            DroppedLinksCount,
            Status,
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
                            "traceId" | "trace_id" => Ok(GeneratedField::TraceId),
                            "spanId" | "span_id" => Ok(GeneratedField::SpanId),
                            "traceState" | "trace_state" => Ok(GeneratedField::TraceState),
                            "parentSpanId" | "parent_span_id" => Ok(GeneratedField::ParentSpanId),
                            "flags" => Ok(GeneratedField::Flags),
                            "name" => Ok(GeneratedField::Name),
                            "kind" => Ok(GeneratedField::Kind),
                            "startTimeUnixNano" | "start_time_unix_nano" => Ok(GeneratedField::StartTimeUnixNano),
                            "endTimeUnixNano" | "end_time_unix_nano" => Ok(GeneratedField::EndTimeUnixNano),
                            "attributes" => Ok(GeneratedField::Attributes),
                            "droppedAttributesCount" | "dropped_attributes_count" => Ok(GeneratedField::DroppedAttributesCount),
                            "events" => Ok(GeneratedField::Events),
                            "droppedEventsCount" | "dropped_events_count" => Ok(GeneratedField::DroppedEventsCount),
                            "links" => Ok(GeneratedField::Links),
                            "droppedLinksCount" | "dropped_links_count" => Ok(GeneratedField::DroppedLinksCount),
                            "status" => Ok(GeneratedField::Status),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Span;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct opentelemetry.proto.trace.v1.Span")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Span, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut trace_id__ = None;
                let mut span_id__ = None;
                let mut trace_state__ = None;
                let mut parent_span_id__ = None;
                let mut flags__ = None;
                let mut name__ = None;
                let mut kind__ = None;
                let mut start_time_unix_nano__ = None;
                let mut end_time_unix_nano__ = None;
                let mut attributes__ = None;
                let mut dropped_attributes_count__ = None;
                let mut events__ = None;
                let mut dropped_events_count__ = None;
                let mut links__ = None;
                let mut dropped_links_count__ = None;
                let mut status__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
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
                        GeneratedField::TraceState => {
                            if trace_state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("traceState"));
                            }
                            trace_state__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ParentSpanId => {
                            if parent_span_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("parentSpanId"));
                            }
                            parent_span_id__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
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
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Kind => {
                            if kind__.is_some() {
                                return Err(serde::de::Error::duplicate_field("kind"));
                            }
                            kind__ = Some(map_.next_value::<span::SpanKind>()? as i32);
                        }
                        GeneratedField::StartTimeUnixNano => {
                            if start_time_unix_nano__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startTimeUnixNano"));
                            }
                            start_time_unix_nano__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::EndTimeUnixNano => {
                            if end_time_unix_nano__.is_some() {
                                return Err(serde::de::Error::duplicate_field("endTimeUnixNano"));
                            }
                            end_time_unix_nano__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
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
                        GeneratedField::Events => {
                            if events__.is_some() {
                                return Err(serde::de::Error::duplicate_field("events"));
                            }
                            events__ = Some(map_.next_value()?);
                        }
                        GeneratedField::DroppedEventsCount => {
                            if dropped_events_count__.is_some() {
                                return Err(serde::de::Error::duplicate_field("droppedEventsCount"));
                            }
                            dropped_events_count__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Links => {
                            if links__.is_some() {
                                return Err(serde::de::Error::duplicate_field("links"));
                            }
                            links__ = Some(map_.next_value()?);
                        }
                        GeneratedField::DroppedLinksCount => {
                            if dropped_links_count__.is_some() {
                                return Err(serde::de::Error::duplicate_field("droppedLinksCount"));
                            }
                            dropped_links_count__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Status => {
                            if status__.is_some() {
                                return Err(serde::de::Error::duplicate_field("status"));
                            }
                            status__ = map_.next_value()?;
                        }
                    }
                }
                Ok(Span {
                    trace_id: trace_id__.unwrap_or_default(),
                    span_id: span_id__.unwrap_or_default(),
                    trace_state: trace_state__.unwrap_or_default(),
                    parent_span_id: parent_span_id__.unwrap_or_default(),
                    flags: flags__.unwrap_or_default(),
                    name: name__.unwrap_or_default(),
                    kind: kind__.unwrap_or_default(),
                    start_time_unix_nano: start_time_unix_nano__.unwrap_or_default(),
                    end_time_unix_nano: end_time_unix_nano__.unwrap_or_default(),
                    attributes: attributes__.unwrap_or_default(),
                    dropped_attributes_count: dropped_attributes_count__.unwrap_or_default(),
                    events: events__.unwrap_or_default(),
                    dropped_events_count: dropped_events_count__.unwrap_or_default(),
                    links: links__.unwrap_or_default(),
                    dropped_links_count: dropped_links_count__.unwrap_or_default(),
                    status: status__,
                })
            }
        }
        deserializer.deserialize_struct("opentelemetry.proto.trace.v1.Span", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for span::Event {
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
        if !self.name.is_empty() {
            len += 1;
        }
        if !self.attributes.is_empty() {
            len += 1;
        }
        if self.dropped_attributes_count != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("opentelemetry.proto.trace.v1.Span.Event", len)?;
        if self.time_unix_nano != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("timeUnixNano", ToString::to_string(&self.time_unix_nano).as_str())?;
        }
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if !self.attributes.is_empty() {
            struct_ser.serialize_field("attributes", &self.attributes)?;
        }
        if self.dropped_attributes_count != 0 {
            struct_ser.serialize_field("droppedAttributesCount", &self.dropped_attributes_count)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for span::Event {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "time_unix_nano",
            "timeUnixNano",
            "name",
            "attributes",
            "dropped_attributes_count",
            "droppedAttributesCount",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TimeUnixNano,
            Name,
            Attributes,
            DroppedAttributesCount,
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
                            "name" => Ok(GeneratedField::Name),
                            "attributes" => Ok(GeneratedField::Attributes),
                            "droppedAttributesCount" | "dropped_attributes_count" => Ok(GeneratedField::DroppedAttributesCount),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = span::Event;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct opentelemetry.proto.trace.v1.Span.Event")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<span::Event, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut time_unix_nano__ = None;
                let mut name__ = None;
                let mut attributes__ = None;
                let mut dropped_attributes_count__ = None;
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
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
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
                    }
                }
                Ok(span::Event {
                    time_unix_nano: time_unix_nano__.unwrap_or_default(),
                    name: name__.unwrap_or_default(),
                    attributes: attributes__.unwrap_or_default(),
                    dropped_attributes_count: dropped_attributes_count__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("opentelemetry.proto.trace.v1.Span.Event", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for span::Link {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.trace_id.is_empty() {
            len += 1;
        }
        if !self.span_id.is_empty() {
            len += 1;
        }
        if !self.trace_state.is_empty() {
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
        let mut struct_ser = serializer.serialize_struct("opentelemetry.proto.trace.v1.Span.Link", len)?;
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
        if !self.trace_state.is_empty() {
            struct_ser.serialize_field("traceState", &self.trace_state)?;
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
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for span::Link {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "trace_id",
            "traceId",
            "span_id",
            "spanId",
            "trace_state",
            "traceState",
            "attributes",
            "dropped_attributes_count",
            "droppedAttributesCount",
            "flags",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TraceId,
            SpanId,
            TraceState,
            Attributes,
            DroppedAttributesCount,
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
                            "traceId" | "trace_id" => Ok(GeneratedField::TraceId),
                            "spanId" | "span_id" => Ok(GeneratedField::SpanId),
                            "traceState" | "trace_state" => Ok(GeneratedField::TraceState),
                            "attributes" => Ok(GeneratedField::Attributes),
                            "droppedAttributesCount" | "dropped_attributes_count" => Ok(GeneratedField::DroppedAttributesCount),
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
            type Value = span::Link;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct opentelemetry.proto.trace.v1.Span.Link")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<span::Link, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut trace_id__ = None;
                let mut span_id__ = None;
                let mut trace_state__ = None;
                let mut attributes__ = None;
                let mut dropped_attributes_count__ = None;
                let mut flags__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
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
                        GeneratedField::TraceState => {
                            if trace_state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("traceState"));
                            }
                            trace_state__ = Some(map_.next_value()?);
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
                    }
                }
                Ok(span::Link {
                    trace_id: trace_id__.unwrap_or_default(),
                    span_id: span_id__.unwrap_or_default(),
                    trace_state: trace_state__.unwrap_or_default(),
                    attributes: attributes__.unwrap_or_default(),
                    dropped_attributes_count: dropped_attributes_count__.unwrap_or_default(),
                    flags: flags__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("opentelemetry.proto.trace.v1.Span.Link", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for span::SpanKind {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "SPAN_KIND_UNSPECIFIED",
            Self::Internal => "SPAN_KIND_INTERNAL",
            Self::Server => "SPAN_KIND_SERVER",
            Self::Client => "SPAN_KIND_CLIENT",
            Self::Producer => "SPAN_KIND_PRODUCER",
            Self::Consumer => "SPAN_KIND_CONSUMER",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for span::SpanKind {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "SPAN_KIND_UNSPECIFIED",
            "SPAN_KIND_INTERNAL",
            "SPAN_KIND_SERVER",
            "SPAN_KIND_CLIENT",
            "SPAN_KIND_PRODUCER",
            "SPAN_KIND_CONSUMER",
        ];

        struct GeneratedVisitor;

        impl serde::de::Visitor<'_> for GeneratedVisitor {
            type Value = span::SpanKind;

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
                    "SPAN_KIND_UNSPECIFIED" => Ok(span::SpanKind::Unspecified),
                    "SPAN_KIND_INTERNAL" => Ok(span::SpanKind::Internal),
                    "SPAN_KIND_SERVER" => Ok(span::SpanKind::Server),
                    "SPAN_KIND_CLIENT" => Ok(span::SpanKind::Client),
                    "SPAN_KIND_PRODUCER" => Ok(span::SpanKind::Producer),
                    "SPAN_KIND_CONSUMER" => Ok(span::SpanKind::Consumer),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for SpanFlags {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::DoNotUse => "SPAN_FLAGS_DO_NOT_USE",
            Self::TraceFlagsMask => "SPAN_FLAGS_TRACE_FLAGS_MASK",
            Self::ContextHasIsRemoteMask => "SPAN_FLAGS_CONTEXT_HAS_IS_REMOTE_MASK",
            Self::ContextIsRemoteMask => "SPAN_FLAGS_CONTEXT_IS_REMOTE_MASK",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for SpanFlags {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "SPAN_FLAGS_DO_NOT_USE",
            "SPAN_FLAGS_TRACE_FLAGS_MASK",
            "SPAN_FLAGS_CONTEXT_HAS_IS_REMOTE_MASK",
            "SPAN_FLAGS_CONTEXT_IS_REMOTE_MASK",
        ];

        struct GeneratedVisitor;

        impl serde::de::Visitor<'_> for GeneratedVisitor {
            type Value = SpanFlags;

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
                    "SPAN_FLAGS_DO_NOT_USE" => Ok(SpanFlags::DoNotUse),
                    "SPAN_FLAGS_TRACE_FLAGS_MASK" => Ok(SpanFlags::TraceFlagsMask),
                    "SPAN_FLAGS_CONTEXT_HAS_IS_REMOTE_MASK" => Ok(SpanFlags::ContextHasIsRemoteMask),
                    "SPAN_FLAGS_CONTEXT_IS_REMOTE_MASK" => Ok(SpanFlags::ContextIsRemoteMask),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for Status {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.message.is_empty() {
            len += 1;
        }
        if self.code != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("opentelemetry.proto.trace.v1.Status", len)?;
        if !self.message.is_empty() {
            struct_ser.serialize_field("message", &self.message)?;
        }
        if self.code != 0 {
            let v = status::StatusCode::try_from(self.code)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.code)))?;
            struct_ser.serialize_field("code", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Status {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "message",
            "code",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Message,
            Code,
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
                            "message" => Ok(GeneratedField::Message),
                            "code" => Ok(GeneratedField::Code),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Status;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct opentelemetry.proto.trace.v1.Status")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Status, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut message__ = None;
                let mut code__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Message => {
                            if message__.is_some() {
                                return Err(serde::de::Error::duplicate_field("message"));
                            }
                            message__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Code => {
                            if code__.is_some() {
                                return Err(serde::de::Error::duplicate_field("code"));
                            }
                            code__ = Some(map_.next_value::<status::StatusCode>()? as i32);
                        }
                    }
                }
                Ok(Status {
                    message: message__.unwrap_or_default(),
                    code: code__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("opentelemetry.proto.trace.v1.Status", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for status::StatusCode {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unset => "STATUS_CODE_UNSET",
            Self::Ok => "STATUS_CODE_OK",
            Self::Error => "STATUS_CODE_ERROR",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for status::StatusCode {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "STATUS_CODE_UNSET",
            "STATUS_CODE_OK",
            "STATUS_CODE_ERROR",
        ];

        struct GeneratedVisitor;

        impl serde::de::Visitor<'_> for GeneratedVisitor {
            type Value = status::StatusCode;

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
                    "STATUS_CODE_UNSET" => Ok(status::StatusCode::Unset),
                    "STATUS_CODE_OK" => Ok(status::StatusCode::Ok),
                    "STATUS_CODE_ERROR" => Ok(status::StatusCode::Error),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for TracesData {
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
        let mut struct_ser = serializer.serialize_struct("opentelemetry.proto.trace.v1.TracesData", len)?;
        if !self.resource_spans.is_empty() {
            struct_ser.serialize_field("resourceSpans", &self.resource_spans)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TracesData {
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
            type Value = TracesData;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct opentelemetry.proto.trace.v1.TracesData")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<TracesData, V::Error>
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
                Ok(TracesData {
                    resource_spans: resource_spans__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("opentelemetry.proto.trace.v1.TracesData", FIELDS, GeneratedVisitor)
    }
}
