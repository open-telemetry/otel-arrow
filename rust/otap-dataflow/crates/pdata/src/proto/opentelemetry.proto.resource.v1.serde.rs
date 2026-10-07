impl serde::Serialize for Resource {
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
        if self.dropped_attributes_count != 0 {
            len += 1;
        }
        if !self.entity_refs.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("opentelemetry.proto.resource.v1.Resource", len)?;
        if !self.attributes.is_empty() {
            struct_ser.serialize_field("attributes", &self.attributes)?;
        }
        if self.dropped_attributes_count != 0 {
            struct_ser.serialize_field("droppedAttributesCount", &self.dropped_attributes_count)?;
        }
        if !self.entity_refs.is_empty() {
            struct_ser.serialize_field("entityRefs", &self.entity_refs)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Resource {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "attributes",
            "dropped_attributes_count",
            "droppedAttributesCount",
            "entity_refs",
            "entityRefs",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Attributes,
            DroppedAttributesCount,
            EntityRefs,
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
                            "droppedAttributesCount" | "dropped_attributes_count" => Ok(GeneratedField::DroppedAttributesCount),
                            "entityRefs" | "entity_refs" => Ok(GeneratedField::EntityRefs),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Resource;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct opentelemetry.proto.resource.v1.Resource")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Resource, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut attributes__ = None;
                let mut dropped_attributes_count__ = None;
                let mut entity_refs__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
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
                        GeneratedField::EntityRefs => {
                            if entity_refs__.is_some() {
                                return Err(serde::de::Error::duplicate_field("entityRefs"));
                            }
                            entity_refs__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(Resource {
                    attributes: attributes__.unwrap_or_default(),
                    dropped_attributes_count: dropped_attributes_count__.unwrap_or_default(),
                    entity_refs: entity_refs__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("opentelemetry.proto.resource.v1.Resource", FIELDS, GeneratedVisitor)
    }
}
